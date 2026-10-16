/*!
# GRAMMAR

<pre class="rust">
opComp   => '=' | '<' | '>' | '<=' } '>=' | '<>'
opFactor => '*' | '/'
unaryOp  => '-' | '+'

expr    => concat (opComp concat)*
concat  => term ('&' term)*
term    => factor (opFactor factor)*
factor  => prod (opProd prod)*
prod    => power ('^' power)*
power   => (unaryOp)* union '%'*
union   => intersect (',' intersect)*   only directly inside '(' ')'
intersect => range (' ' range)*
range   => implicit (':' primary)?
implicit=> '@' primary | primary '#' | primary
primary => '(' expr ')'
        => number
        => function '(' f_args ')'
        => LAMBDA '(' f_args ')' '(' f_args ')'
        => name
        => string
        => '{' a_args '}'
        => bool
        => bool()
        => error

f_args  => e (',' e)*
</pre>
*/

use std::collections::HashMap;

use crate::functions::Function;
use crate::language::get_default_language;
use crate::language::get_language;
use crate::language::Language;
use crate::locale::get_default_locale;
use crate::locale::get_locale;
use crate::locale::Locale;
use crate::text::Text;
use crate::types::Table;

use super::lexer;
use super::token;
use super::token::OpUnary;
use super::token::TokenType;
use super::types::*;

use sheet_span::with_sheet_span;
use token::OpCompare;

mod chain;
mod contains_function;
mod depth;
mod lambda;
mod lists;
pub mod move_formula;
mod operands;
mod reference_operators;
mod sheet_span;
pub mod static_analysis;
pub mod stringify;

pub use chain::ChainLeft;
use chain::ChainLevel;
pub(crate) use chain::{for_each_chain_operand_mut, left_chain, value_operands};
pub(crate) use depth::is_limit_error;
pub use depth::{
    MAX_FORMULA_DEPTH, MAX_VALUE_OPERATORS, TOO_DEEP_MESSAGE, TOO_MANY_OPERATORS_MESSAGE,
};

#[cfg(test)]
mod tests;

pub(crate) fn parse_range(formula: &str) -> Result<(i32, i32, i32, i32), String> {
    let mut lexer = lexer::Lexer::new(
        formula,
        lexer::LexerMode::A1,
        #[allow(clippy::expect_used)]
        get_locale("en").expect(""),
        #[allow(clippy::expect_used)]
        get_language("en").expect(""),
    );
    if let TokenType::Range {
        left,
        right,
        sheet: _,
    } = lexer.next_token()
    {
        Ok((left.column, left.row, right.column, right.row))
    } else {
        Err("Not a range".to_string())
    }
}

fn get_table_column_by_name(table_column_name: &str, table: &Table) -> Option<i32> {
    let wanted = table_column_name.to_lowercase();
    for (index, table_column) in table.columns.iter().enumerate() {
        if table_column.name.to_lowercase() == wanted {
            return Some(index as i32);
        }
    }
    None
}

// DefinedNameS is a tuple with the name of the defined name, the index of the sheet and the formula
pub type DefinedNameS = (String, Option<u32>, String);

pub(crate) struct Reference<'a> {
    sheet_name: &'a Option<String>,
    sheet_index: u32,
    absolute_row: bool,
    absolute_column: bool,
    row: i32,
    column: i32,
}

#[derive(PartialEq, Clone, Debug)]
pub enum ArrayNode {
    Boolean(bool),
    Number(f64),
    String(Text),
    Error(token::Error),
    /// An empty (blank) cell from a range reference.
    Empty,
}

#[derive(PartialEq, Clone, Debug)]
pub struct NamedVariable {
    pub(crate) name: String,
    pub(crate) id: Option<u32>,
    pub(crate) is_optional: bool,
}

#[derive(PartialEq, Clone, Debug, serde::Serialize)]
pub enum ExpectedTokens {
    // We know the next token could be a range
    Range,
    // We know the next token could be a function that starts with the given name
    FunctionName(String),
    // We know the next token could be an argument to a function (name, arg index)
    Argument(String, u32),
    Other,
}

/// What the grammar accepts at a cursor position, plus the span the UI should
/// replace when it inserts a completion. Returned by [`Parser::parse_at_cursor`].
#[derive(PartialEq, Clone, Debug, serde::Serialize)]
pub struct CompletionContext {
    /// What the grammar accepts at the cursor.
    pub expecting: Vec<ExpectedTokens>,
    /// The span `[replace_from, cursor)` the UI should replace — e.g. the `F`
    /// in `A1+F`. Equals the cursor when there is nothing to replace (right
    /// after `SUM(`).
    pub replace_from: usize,
}

/// A parsed formula. A value operator's left operand is a [`ChainLeft`] (see `chain.rs`).
#[derive(PartialEq, Clone, Debug)]
pub enum Node {
    BooleanKind(bool),
    NumberKind(f64),
    StringKind(String),
    ReferenceKind {
        sheet_name: Option<String>,
        sheet_index: u32,
        absolute_row: bool,
        absolute_column: bool,
        row: i32,
        column: i32,
    },
    RangeKind {
        sheet_name: Option<String>,
        sheet_index: u32,
        absolute_row1: bool,
        absolute_column1: bool,
        row1: i32,
        column1: i32,
        absolute_row2: bool,
        absolute_column2: bool,
        row2: i32,
        column2: i32,
    },
    /// `First:Last!A1` or `First:Last!A1:B2`, a 3-D reference: the same cells on every sheet
    /// from `First` to `Last` in tab order (docs/excel-semantics/references.md, "3-D
    /// sheet-range references"). `reference` is the `ReferenceKind` or `RangeKind` on the first
    /// sheet, written without a sheet prefix.
    SheetSpanKind {
        first_sheet: String,
        last_sheet: String,
        first_index: u32,
        last_index: u32,
        reference: Box<Node>,
    },
    WrongReferenceKind {
        sheet_name: Option<String>,
        absolute_row: bool,
        absolute_column: bool,
        row: i32,
        column: i32,
    },
    WrongRangeKind {
        sheet_name: Option<String>,
        absolute_row1: bool,
        absolute_column1: bool,
        row1: i32,
        column1: i32,
        absolute_row2: bool,
        absolute_column2: bool,
        row2: i32,
        column2: i32,
    },
    OpRangeKind {
        left: Box<Node>,
        right: Box<Node>,
    },
    /// `left right`: the cells both references contain (Excel's space operator).
    OpIntersectKind {
        left: Box<Node>,
        right: Box<Node>,
    },
    /// `(left,right)`: the areas of both references (Excel's union operator).
    OpUnionKind {
        left: Box<Node>,
        right: Box<Node>,
        /// Written outside any parentheses or argument list, where the comma is a reference
        /// operator binding tighter than arithmetic: `=A1,B2`, `=1+A1,B2` (T-1073).
        bare: bool,
    },
    OpConcatenateKind {
        left: ChainLeft,
        right: Box<Node>,
    },
    OpSumKind {
        kind: token::OpSum,
        left: ChainLeft,
        right: Box<Node>,
    },
    OpProductKind {
        kind: token::OpProduct,
        left: ChainLeft,
        right: Box<Node>,
    },
    OpPowerKind {
        left: ChainLeft,
        right: Box<Node>,
    },
    FunctionKind {
        kind: Function,
        args: Vec<Node>,
    },
    // LAMBDA(a,b, SQRT(a*a+b*b))
    LambdaDefKind {
        parameters: Vec<NamedVariable>,
        body: Box<Node>,
    },
    // LAMBDA(a,b, SQRT(a*a+b*b))(3,4)
    LambdaCallKind {
        lambda: Box<Node>,
        args: Vec<Node>,
    },
    NamedFunctionKind {
        id: Option<u32>,
        name: String,
        args: Vec<Node>,
    },
    ArrayKind(Vec<Vec<ArrayNode>>),
    DefinedNameKind(DefinedNameS),
    NamedVariableKind {
        name: String,
        id: Option<u32>,
    },
    /// `@child`. The parentheses written around its operand, around part of it or around the
    /// whole `@` are [`Node::Group`]s where they were written (`@(A1)`, `@(A1) B1`, `(@1)#`;
    /// SPEC-0007); the automatic `@` of an xlsx import has none.
    ImplicitIntersection {
        automatic: bool,
        child: Box<Node>,
    },
    SpillRangeOperator {
        child: Box<Node>,
    },
    /// Parentheses written around `child`: `levels` directly nested pairs, `((x))` is 2.
    /// Transparent to evaluation (see [`Node::ungrouped`]); printed back as written, as Excel's
    /// FORMULATEXT and stored formula text keep every written parenthesis
    /// (docs/excel-semantics/syntax.md, SPEC-0007).
    Group {
        levels: u8,
        child: Box<Node>,
    },
    CompareKind {
        kind: OpCompare,
        left: ChainLeft,
        right: Box<Node>,
    },
    UnaryKind {
        kind: OpUnary,
        right: Box<Node>,
    },
    ErrorKind(token::Error),
    ParseErrorKind {
        formula: String,
        message: String,
        position: usize,
        // What tokens were expected at this position.
        expecting: Vec<ExpectedTokens>,
    },
    EmptyArgKind,
}

// A `Group` must not grow the node: `Box<Node>` + `u8` fits in the largest variant (SPEC-0007).
const _: () = assert!(std::mem::size_of::<Node>() <= NODE_SIZE);

/// `size_of::<Node>()` before the `Group` variant; pinned so later variants cannot grow it unnoticed.
const NODE_SIZE: usize = 80;

impl Node {
    /// The node under any written parentheses: `((A1))` is `A1`. Loops, never recurses, so a
    /// group adds no evaluator frame (SPEC-0007 "Seeing through groups").
    pub(crate) fn ungrouped(&self) -> &Node {
        let mut node = self;
        while let Node::Group { child, .. } = node {
            node = child;
        }
        node
    }

    /// The `@` an xlsx import adds around a range-valued operand.
    pub(crate) fn automatic_at(child: Node) -> Node {
        Node::at(child, true)
    }
    /// `@child`, typed (`automatic` false) or added by an xlsx import.
    pub(super) fn at(child: Node, automatic: bool) -> Node {
        Node::ImplicitIntersection {
            automatic,
            child: Box::new(child),
        }
    }
}

#[derive(Clone)]
pub struct Parser {
    lexer: lexer::Lexer,
    worksheets: Vec<String>,
    defined_names: Vec<DefinedNameS>,
    context: CellReferenceRC,
    tables: HashMap<String, Table>,
    locale: &'static Locale,
    language: &'static Language,
    /// Completion hint for the position currently being parsed. The deepest
    /// frame that hits EOF stamps this onto its error. See `parse_at_cursor`.
    expecting_here: Vec<ExpectedTokens>,
    /// Set while parsing when an identifier turns out to be the last token in
    /// the input (its `peek` is EOF): the user is mid-typing a name there.
    /// Holds `(prefix, start_offset)`. Read by `parse_at_cursor` to offer
    /// function/name completion even when the surrounding parse succeeds (`SU`,
    /// `A1+F`) or fails for an unrelated reason (`IF(VLOOK`). See that method.
    trailing_name: Option<(String, usize)>,
    /// Primaries (operands) open on the current parse path. Checked before
    /// descending so the parser's own recursion stays bounded.
    nesting: usize,
    /// Depth of the tree the last `parse_*` call returned.
    depth: usize,
    /// Deepest subexpression parsed so far inside the primary being parsed.
    inner_depth: usize,
    /// Value operators in the formula so far, bounded by [`MAX_VALUE_OPERATORS`].
    value_operators: usize,
    /// The parser is outside every parenthesis and argument list, where a comma is the bare
    /// union operator (`=A1,B2`, T-1073).
    top_level: bool,
}

pub fn new_parser_english(
    worksheets: Vec<String>,
    defined_names: Vec<DefinedNameS>,
    tables: HashMap<String, Table>,
) -> Parser {
    let locale = get_default_locale();
    let language = get_default_language();
    Parser::new(worksheets, defined_names, tables, locale, language)
}

impl Parser {
    pub fn new(
        worksheets: Vec<String>,
        defined_names: Vec<DefinedNameS>,
        tables: HashMap<String, Table>,
        locale: &'static Locale,
        language: &'static Language,
    ) -> Parser {
        let lexer = lexer::Lexer::new("", lexer::LexerMode::A1, locale, language);
        let context = CellReferenceRC {
            sheet: worksheets.first().map_or("", |v| v).to_string(),
            column: 1,
            row: 1,
        };
        Parser {
            lexer,
            worksheets,
            defined_names,
            context,
            tables,
            locale,
            language,
            expecting_here: vec![ExpectedTokens::Other],
            trailing_name: None,
            nesting: 0,
            top_level: false,
            depth: 0,
            inner_depth: 0,
            value_operators: 0,
        }
    }
    pub fn set_lexer_mode(&mut self, mode: lexer::LexerMode) {
        self.lexer.set_lexer_mode(mode);
    }

    pub fn set_locale(&mut self, locale: &'static Locale) {
        self.locale = locale;
        self.lexer.set_locale(locale);
    }

    pub fn set_language(&mut self, language: &'static Language) {
        self.language = language;
        self.lexer.set_language(language);
    }

    pub fn set_worksheets_and_names(
        &mut self,
        worksheets: Vec<String>,
        defined_names: Vec<DefinedNameS>,
    ) {
        self.worksheets = worksheets;
        self.defined_names = defined_names;
    }

    /// Replaces the tables structured references resolve against.
    pub fn set_tables(&mut self, tables: HashMap<String, Table>) {
        self.tables = tables;
    }

    pub fn parse(&mut self, formula: &str, context: &CellReferenceRC) -> Node {
        self.lexer.set_formula(formula);
        self.context = context.clone();
        // At the top level a formula may start with an expression or a range.
        self.expecting_here = vec![ExpectedTokens::Range, ExpectedTokens::Other];
        self.trailing_name = None;
        self.nesting = 0;
        self.depth = 0;
        self.inner_depth = 0;
        self.value_operators = 0;
        self.top_level = true;
        let node = self.parse_expr();
        self.top_level = false;
        self.expect_end(node)
    }

    /// Parses `formula` up to `cursor` (a char offset) and reports what the
    /// grammar would accept at that position, so callers can offer completions.
    pub fn parse_at_cursor(
        &mut self,
        formula: &str,
        cursor: usize,
        context: &CellReferenceRC,
    ) -> CompletionContext {
        let head: String = formula.chars().take(cursor).collect();

        let node = self.parse(&head, context);

        // If the cursor sits on an identifier the user is mid-typing, the parser
        // recorded it as `trailing_name` while consuming it in a name position
        // (see `parse_primary`). That takes priority over the parse outcome: it
        // wins both when the surrounding parse *succeeds* (`SU`, `A1+F`) — where
        // there is no error to carry an `expecting` — and when it *fails* for an
        // unrelated reason (`IF(VLOOK`, where the EOF frame would otherwise
        // report the "argument of IF" hint).
        if let Some((prefix, start)) = self.trailing_name.take() {
            return CompletionContext {
                expecting: vec![ExpectedTokens::FunctionName(prefix)],
                replace_from: start,
            };
        }

        match node {
            // (a) The prefix is incomplete: the EOF frame stamped `expecting`.
            Node::ParseErrorKind {
                expecting,
                position,
                ..
            } => CompletionContext {
                expecting,
                replace_from: position,
            },
            // (b) The prefix parsed cleanly and there is no trailing bare name,
            // so there is nothing grammar-specific to offer.
            _ => CompletionContext {
                expecting: vec![ExpectedTokens::Other],
                replace_from: cursor,
            },
        }
    }

    // Returns the token used to separate arguments in functions and arrays
    // If the locale decimal separator is '.', then it is a comma ','
    // Otherwise, it is a semicolon ';'
    fn get_argument_separator_token(&self) -> TokenType {
        if self.locale.numbers.symbols.decimal == "." {
            TokenType::Comma
        } else {
            TokenType::Semicolon
        }
    }

    // Returns the token used to separate columns in arrays
    // If the locale decimal separator is '.', then it is a semicolon ';'
    fn get_column_separator_token(&self) -> TokenType {
        if self.locale.numbers.symbols.decimal == "." {
            TokenType::Semicolon
        } else {
            TokenType::Backslash
        }
    }

    // Returns:
    //  * None: If there is no defined name by that name
    //  * Some((Some(index), formula)): If there is a defined name local to that sheet
    //  * Some(None): If there is a global defined name
    fn get_defined_name(&self, name: &str, sheet: u32) -> Option<(Option<u32>, String)> {
        for (df_name, df_scope, df_formula) in &self.defined_names {
            if name.to_lowercase() == df_name.to_lowercase() && df_scope == &Some(sheet) {
                return Some((*df_scope, df_formula.to_owned()));
            }
        }
        for (df_name, df_scope, df_formula) in &self.defined_names {
            if name.to_lowercase() == df_name.to_lowercase() && df_scope.is_none() {
                return Some((None, df_formula.to_owned()));
            }
        }
        None
    }

    /// An expression at the current level; parentheses and argument lists call
    /// [`Parser::parse_enclosed_expr`] instead, so their commas are not bare unions.
    pub(super) fn parse_expr(&mut self) -> Node {
        let t = self.parse_chain(ChainLevel::Compare);
        if !matches!(t, Node::ParseErrorKind { .. }) {
            self.inner_depth = self.inner_depth.max(self.depth);
        }
        t
    }

    pub(super) fn parse_power(&mut self) -> Node {
        let mut next_token = self.lexer.peek_token();
        // Every minus is kept (`--x` coerces `x` to a number); a plus is a no-op.
        let mut minus_count = 0;
        while let TokenType::Addition(op) = next_token {
            self.lexer.advance_token();
            minus_count += usize::from(op == token::OpSum::Minus);
            next_token = self.lexer.peek_token();
        }

        let mut t = self.parse_bare_union();
        if let Node::ParseErrorKind { .. } = t {
            return t;
        }
        for _ in 0..minus_count {
            if let Err(error) = self.grow(self.depth, 0) {
                return error;
            }
            t = Node::UnaryKind {
                kind: token::OpUnary::Minus,
                right: Box::new(t),
            }
        }
        next_token = self.lexer.peek_token();
        while next_token == TokenType::Percent {
            self.lexer.advance_token();
            if let Err(error) = self.grow(self.depth, 0) {
                return error;
            }
            t = Node::UnaryKind {
                kind: token::OpUnary::Percentage,
                right: Box::new(t),
            };
            next_token = self.lexer.peek_token();
        }
        t
    }

    fn parse_operand(&mut self) -> Node {
        let next_token = self.lexer.next_token();
        match next_token {
            TokenType::LeftParenthesis => self.parse_parenthesized_call(),
            TokenType::Number(s) => Node::NumberKind(s),
            TokenType::String(s) => Node::StringKind(s),
            TokenType::LeftBrace => self.parse_array_operand(),
            TokenType::Reference {
                sheet,
                row,
                column,
                absolute_column,
                absolute_row,
            } => self.reference_operand(sheet, row, column, absolute_column, absolute_row),
            TokenType::Range { sheet, left, right } => self.range_operand(sheet, left, right),
            TokenType::Ident(name) => self.parse_identifier(name),
            TokenType::Error(kind) => Node::ErrorKind(kind),
            TokenType::Illegal(error) => Node::ParseErrorKind {
                formula: self.lexer.get_formula(),
                expecting: vec![ExpectedTokens::Other],
                position: error.position,
                message: error.message,
            },
            TokenType::EOF => Node::ParseErrorKind {
                formula: self.lexer.get_formula(),
                // The deepest frame to reach EOF knows what it was expecting.
                expecting: self.expecting_here.clone(),
                position: self.lexer.get_position() as usize,
                message: "Unexpected end of input.".to_string(),
            },
            TokenType::Boolean(value) => self.parse_boolean_operand(value),
            TokenType::Compare(_) => {
                self.unexpected_operand("Unexpected token: 'COMPARE'".to_string())
            }
            TokenType::Addition(_) => {
                self.unexpected_operand("Unexpected token: 'SUM'".to_string())
            }
            TokenType::Product(_) => {
                self.unexpected_operand("Unexpected token: 'PRODUCT'".to_string())
            }
            TokenType::Power => self.unexpected_operand("Unexpected token: 'POWER'".to_string()),
            TokenType::At => self.unexpected_operand("Unexpected token: '@'".to_string()),
            TokenType::RightParenthesis
            | TokenType::RightBracket
            | TokenType::Colon
            | TokenType::Semicolon
            | TokenType::Backslash
            | TokenType::RightBrace
            | TokenType::Comma
            | TokenType::Bang
            | TokenType::And
            | TokenType::Spill
            | TokenType::Percent => {
                self.unexpected_operand(format!("Unexpected token: '{next_token:?}'"))
            }
            TokenType::LeftBracket => self.unexpected_operand("Unexpected token: '['".to_string()),
            TokenType::StructuredReference {
                table_name,
                specifier,
                table_reference,
            } => self.structured_reference_operand(table_name, specifier, table_reference),
        }
    }
}
