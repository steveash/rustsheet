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
use crate::language::Language;
use crate::locale::get_default_locale;
use crate::locale::Locale;
use crate::types::Table;

use super::lexer;
use super::token;
use super::token::TokenType;
use super::types::*;

use sheet_span::with_sheet_span;
pub(crate) use sheet_span::{is_multi_sheet_span, is_span_or_if_of_span};

mod call_names;
mod chain;
mod contains_function;
mod defined_names;
mod depth;
mod lambda;
mod lists;
pub mod move_formula;
mod node;
mod operands;
mod reference_operators;
mod sheet_span;
mod spacing;
pub mod static_analysis;
pub mod stringify;
mod structured_reference;
mod unparsed;

pub use chain::ChainLeft;
use chain::ChainLevel;
pub(crate) use chain::{for_each_chain_operand_mut, left_chain, value_operands};
pub(crate) use depth::is_limit_error;
pub use depth::{
    MAX_FORMULA_DEPTH, MAX_NESTED_CALLS, MAX_PARENTHESES, MAX_SIGNS, MAX_VALUE_OPERATORS,
    TOO_DEEP_MESSAGE, TOO_MANY_NESTED_CALLS_MESSAGE, TOO_MANY_OPERATORS_MESSAGE,
    TOO_MANY_SIGNS_MESSAGE,
};
pub use node::{ArrayNode, CompletionContext, ExpectedTokens, NamedVariable, Node};
pub(crate) use spacing::Run;
pub use spacing::Spacing;
pub use structured_reference::{parse_range, StructuredReference};

pub(crate) use call_names::CallNameSpellings;

#[cfg(test)]
mod tests;

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

#[derive(Clone)]
pub struct Parser {
    lexer: lexer::Lexer,
    worksheets: Vec<String>,
    /// The defined names. Whatever sets this field sets `name_index` too.
    defined_names: Vec<DefinedNameS>,
    /// Each defined name's position in `defined_names`, by its scope and lower-cased name (the
    /// key `Model::parsed_defined_names` uses), the first of names that fold alike: a lookup is
    /// one probe, not a scan with two lower-casings per name (T-3195).
    name_index: HashMap<(Option<u32>, String), usize>,
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
    /// Half levels reserved by the primaries (operands) open on the current parse path, bounded
    /// by [`depth::MAX_NESTING`]. Checked before descending so the parser's own recursion stays
    /// bounded.
    nesting: usize,
    /// Depth of the tree the last `parse_*` call returned.
    depth: usize,
    /// Deepest subexpression parsed so far inside the primary being parsed.
    inner_depth: usize,
    /// An operand parsed ahead, with its depth, for the next [`Parser::parse_primary`] to
    /// return: the inner group of a run of parentheses ([`Parser::parse_parenthesis_run`]).
    pending_operand: Option<(Node, usize)>,
    /// Where the pending operand's text starts (a char index), for the intersection that reads
    /// it back ([`Parser::parse_intersection`]).
    pending_start: usize,
    /// The signs written before the pending operand, read ahead with it.
    pending_prefix: Option<depth::Prefix>,
    /// The table an unqualified structured reference names in a formula a row insert moves with
    /// that table, before the parser's tables show it there (`Model::move_cell`, T-3037).
    moved_table: Option<String>,
    /// Written groups open on the current parse path, bounded by [`MAX_PARENTHESES`].
    open_groups: usize,
    /// A group's content is about to be parsed: its first operand, unless another group, reserves
    /// less nesting ([`Parser::parse_primary`], T-3197).
    group_head: bool,
    /// Unary signs open on the current parse path, bounded by [`MAX_SIGNS`].
    open_signs: usize,
    /// Value operators in the formula so far, bounded by [`MAX_VALUE_OPERATORS`].
    value_operators: usize,
    /// The parser is outside every parenthesis and argument list, where a comma is the bare
    /// union operator (`=A1,B2`, T-1073).
    top_level: bool,
    /// Names the enclosing LET and LAMBDA calls bind so far. A name spelled like a Lotus-style
    /// range or a cell is the variable where bound and the range or cell elsewhere
    /// (`LET(R1.C1,5,R1.C1*2)` is 10, probes/r1c1-let-name-r1-c1, lotus-dot-range, T-1659;
    /// `LET(f1,1,f1+1)` is 2, probes/let-cell-shaped-name-stored, T-2040); the variable shadows
    /// a defined name (T-2057) and, called, a function of its name (T-2567).
    bound_names: Vec<String>,
    /// The formula is being typed (API `Input::Formula`/`Parse`), not read as stored: a LET name
    /// or LAMBDA parameter spelled like a cell then refuses the formula, as Excel refuses it
    /// typed (decision 0103, T-2497). Stored text (xlsx import, re-parses) keeps binding it.
    typed: bool,
    /// The formula is read from a file (xlsx import, `Input::StoredFormula`): a structured
    /// reference to an unknown table or column is `#REF!` (see [`Parser::set_loaded`]).
    loaded: bool,
    /// The first cell-shaped name the last parse bound (see `typed`).
    cell_shaped_binding: Option<String>,
    /// The first call of a function with an argument count Excel refuses that the last parse
    /// read (see [`Parser::typed_refusal`]).
    wrong_count_call: Option<String>,
    /// Function calls open on the current parse path ([`Parser::in_call`]).
    open_calls: usize,
    /// The most function calls open at once in the last parse (see
    /// [`Parser::nests_too_many_calls`]).
    deepest_calls: usize,
    /// The workbook's file name (`Budget.xlsx`), which prefixes a workbook name read through a
    /// sheet (`=Feb!loc` is `=Budget.xlsx!loc`); `None` for an unnamed workbook.
    workbook_file: Option<String>,
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
        let name_index = defined_names::index_names(&defined_names);
        Parser {
            lexer,
            worksheets,
            defined_names,
            name_index,
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
            pending_operand: None,
            pending_start: 0,
            pending_prefix: None,
            moved_table: None,
            open_groups: 0,
            group_head: false,
            open_signs: 0,
            value_operators: 0,
            bound_names: Vec::new(),
            typed: false,
            loaded: false,
            cell_shaped_binding: None,
            wrong_count_call: None,
            open_calls: 0,
            deepest_calls: 0,
            workbook_file: None,
        }
    }
    /// Reads the next formulas as a file loader does (`true`), or not (`false`, the default).
    /// Excel loads a formula naming a table or column the file lacks with the reference rewritten
    /// to `#REF!`, FORMULATEXT `=SUM(#REF!)` (docs/excel-semantics/references.md "Structured
    /// references when the table changes", probes/table-edits/table-unknown-name-load, T-2197);
    /// anywhere else (typed text, a re-parse after `remove_table`) it stays a parse error.
    pub fn set_loaded(&mut self, loaded: bool) {
        self.loaded = loaded;
    }

    /// Reads the next formulas as typed (`true`) or stored (`false`); see [`Parser::typed_refusal`].
    pub(crate) fn set_typed(&mut self, typed: bool) {
        self.typed = typed;
        self.lexer.set_typed(typed);
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
        self.name_index = defined_names::index_names(&defined_names);
        self.defined_names = defined_names;
    }

    /// Sets the workbook's file name (`Budget.xlsx`, `None` for an unnamed workbook); see
    /// `Parser::workbook_file`.
    pub fn set_workbook_file(&mut self, file_name: Option<String>) {
        self.workbook_file = file_name;
    }

    /// Replaces the tables structured references resolve against.
    pub fn set_tables(&mut self, tables: HashMap<String, Table>) {
        self.tables = tables;
    }

    pub fn parse(&mut self, formula: &str, context: &CellReferenceRC) -> Node {
        if !self.lexer.is_a1_mode() {
            if let Some(stored) = unparsed::parse_stored_text(formula) {
                return stored;
            }
        }
        // A file's stored text drops the whitespace it starts with, where the same text typed
        // after `=` keeps it: `<f> 1+2</f>` prints `=1+2` (probes/t2776-formula-text-whitespace-shapes
        // E7, E8; syntax.md "Formula text keeps typed whitespace…", T-2784), and so does a
        // leading line feed (`<f>` LF `1+2</f>` prints `=1+2`; probes/t2949-stored-5).
        let formula = if self.loaded {
            formula.trim_start_matches([' ', '\n'])
        } else {
            formula
        };
        self.lexer.set_formula(formula);
        self.lexer.set_engine_text(!self.typed && !self.loaded);
        self.context = context.clone();
        // At the top level a formula may start with an expression or a range.
        self.expecting_here = vec![ExpectedTokens::Range, ExpectedTokens::Other];
        self.trailing_name = None;
        self.nesting = 0;
        self.depth = 0;
        self.inner_depth = 0;
        self.pending_operand = None;
        self.pending_prefix = None;
        self.open_groups = 0;
        self.group_head = false;
        self.open_signs = 0;
        self.value_operators = 0;
        self.bound_names.clear();
        self.cell_shaped_binding = None;
        self.wrong_count_call = None;
        self.open_calls = 0;
        self.deepest_calls = 0;
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

    /// An expression at the current level; parentheses and argument lists call
    /// [`Parser::parse_enclosed_expr`] instead, so their commas are not bare unions.
    pub(super) fn parse_expr(&mut self) -> Node {
        let t = self.parse_chain(ChainLevel::Compare);
        if !matches!(t, Node::ParseErrorKind { .. }) {
            self.inner_depth = self.inner_depth.max(self.depth);
        }
        t
    }

    /// `power => (unaryOp)* union '%'*`. The whitespace typed before the operand, after each of
    /// its signs, before each `%` and after it is kept as [`Node::Spaced`] (`=- 5`, `=5 %`,
    /// `=1 + 2`; SPEC-0023). Every nesting level of a formula passes through here, so the work
    /// before and after the operand is in helpers of their own, to keep this frame small in
    /// unoptimized builds (T-1110).
    pub(super) fn parse_power(&mut self) -> Node {
        let prefix = self.parse_prefix();
        let t = self.parse_signed_operand(&prefix);
        if let Node::ParseErrorKind { .. } = t {
            return t;
        }
        match self.finish_power(t, prefix.before, prefix.signs) {
            Ok(node) | Err(node) => node,
        }
    }

    /// Operand `t` of [`Parser::parse_power`] under its `signs`, then its `%`s, with the runs
    /// around it.
    #[inline(never)]
    fn finish_power(
        &mut self,
        mut t: Node,
        before: Run,
        signs: Vec<(bool, Run)>,
    ) -> Result<Node, Node> {
        // A sign adds no depth: its consumers walk a run in a loop (`Model::apply_unary_chain`),
        // and [`MAX_SIGNS`] bounds how many are open at once (`Parser::parse_signed_operand`).
        for (is_minus, after_sign) in signs.into_iter().rev() {
            t = t.spaced(after_sign, Run::default());
            let right = Box::new(t);
            t = if is_minus {
                Node::UnaryKind {
                    kind: token::OpUnary::Minus,
                    right,
                }
            } else {
                Node::UnaryPlus { child: right }
            };
        }
        while *self.lexer.peek() == TokenType::Percent {
            t = t.spaced(Run::default(), self.run_ahead());
            self.lexer.advance_token();
            self.grow(self.depth, 0)?;
            t = Node::UnaryKind {
                kind: token::OpUnary::Percentage,
                right: Box::new(t),
            };
        }
        let after = self.run_after_operand();
        Ok(t.spaced(before, after))
    }

    fn parse_operand(&mut self) -> Node {
        let start = self.lexer.get_position_index();
        let next_token = self.lexer.next_token();
        match next_token {
            TokenType::LeftParenthesis => self.parse_parenthesis_run(),
            TokenType::Number(s) => Node::NumberKind(s),
            TokenType::String(s) => Node::StringKind(s),
            TokenType::LeftBrace => self.parse_array_operand(),
            TokenType::Reference {
                sheet,
                row,
                column,
                absolute_column,
                absolute_row,
            } => {
                let text = self.lexer.source_since(start);
                if sheet.is_none() && self.binds_reference_shaped_name(&text) {
                    return self.parse_identifier(text);
                }
                self.reference_or_call_operand(sheet, row, column, absolute_column, absolute_row)
            }
            TokenType::Range { sheet, left, right } => {
                self.range_or_call_operand(sheet, left, right)
            }
            TokenType::Ident(name) => self.parse_identifier(name),
            TokenType::Error(kind) => Node::ErrorKind(kind),
            TokenType::SheetRefError(sheet_name) => Node::SheetRefErrorKind { sheet_name },
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
            TokenType::LeftBracket => self.unqualified_structured_reference(),
            TokenType::StructuredReference {
                table_name,
                specifier,
                table_reference,
                spacing,
            } => self.structured_reference_operand(
                table_name,
                specifier,
                table_reference,
                spacing,
                false,
            ),
        }
    }
}
