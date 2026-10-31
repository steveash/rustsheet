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

use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;

use crate::functions::Function;
use crate::language::get_default_language;
use crate::language::Language;
use crate::locale::get_default_locale;
use crate::locale::Locale;
use crate::nested_array::NestedArray;
use crate::text::Text;
use crate::types::Table;

use super::lexer;
use super::token;
use super::token::OpUnary;
use super::token::TokenType;
use super::types::*;

use sheet_span::with_sheet_span;
pub(crate) use sheet_span::{is_multi_sheet_span, is_span_or_if_of_span};
use token::OpCompare;

mod call_names;
mod chain;
mod contains_function;
mod depth;
mod lambda;
mod lists;
pub mod move_formula;
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
};
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

#[derive(PartialEq, Clone, Debug)]
pub enum ArrayNode {
    Boolean(bool),
    Number(f64),
    String(Text),
    Error(token::Error),
    /// An empty (blank) cell from a range reference.
    Empty,
    /// A LAMBDA value (its lambda id), as SCAN with a LAMBDA accumulator, MAKEARRAY or VSTACK
    /// of LAMBDAs give: MAP, REDUCE and a LET name read through INDEX call it, a cell shows
    /// `#CALC!`, and a function reading it as a value takes it as `#VALUE!`
    /// (docs/excel-semantics/arrays.md "Arrays of LAMBDAs", probes/lambda-arrays, T-2039).
    /// Never in a parsed array constant.
    Lambda(usize),
    /// A cell of a GROUPBY or PIVOTBY result that holds its function's array result, or the
    /// value of a one-cell reference result (SPEC-0018 rules 1, 7, 8): a cell shows `#VALUE!`,
    /// INDEX returns the array, an element-wise lift reads its top-left ([`ArrayNode::scalar`]).
    /// Never in a parsed array constant.
    Nested(Arc<NestedArray>),
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
        levels: u16,
        child: Box<Node>,
    },
    /// A structured reference as written (`Sales[[#All],[Qty]]`), with the area it resolves to
    /// for the table list the formula was parsed against: a `ReferenceKind`, a `RangeKind` or an
    /// `ErrorKind`. Transparent to evaluation (see [`Node::ungrouped`]); the storage and in-engine
    /// printers print `reference`, the xlsx export prints `lowered` (SPEC-0016).
    StructuredReferenceKind {
        reference: Box<StructuredReference>,
        lowered: Box<Node>,
    },
    /// The `lowered` area of a `[#This Row]` structured reference: columns `column1..column2` of
    /// the row of the cell evaluating it, a data row `data_start..data_end` of the table, and
    /// `#VALUE!` off the data rows (gen-m8b this-row e1bf5aa0ffb4ed87). It does not depend on the
    /// row the formula was parsed at, so a shared formula serves a whole column (SPEC-0016 step
    /// 2, T-1699). `cell` marks one column named alone (`Sales[@Qty]`), which is a cell
    /// reference; the other forms are ranges, as their parse-time lowering was. Seen through
    /// [`Node::ungrouped_at`] and [`Node::lowered_at`], which resolve it at a row.
    ThisRowKind {
        sheet_name: Option<String>,
        sheet_index: u32,
        column1: i32,
        column2: i32,
        data_start: i32,
        data_end: i32,
        cell: bool,
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
    /// A unary plus as written (`+"a"`, `+A1`, `+-5`). The identity, so transparent to evaluation
    /// and analysis as a [`Node::Group`] is (see [`Node::ungrouped`] and [`Node::lowered`]); kept
    /// only so formula text prints it back, as Excel's FORMULATEXT and stored formula text do. A
    /// plus written directly before a number literal is not kept: it folds into the number
    /// (`=+5` is `=5`; docs/excel-semantics/syntax.md, T-2723).
    UnaryPlus {
        child: Box<Node>,
    },
    /// Whitespace written around `child`, printed back as typed: `space`'s runs ahead of its
    /// first token and behind its last (docs/excel-semantics/syntax.md "Formula text keeps typed
    /// whitespace…", SPEC-0023). Transparent to evaluation and analysis, as a [`Node::Group`] is
    /// (see [`Node::ungrouped`] and [`Node::lowered`]); only built where whitespace was written.
    Spaced {
        space: Spacing,
        child: Box<Node>,
    },
    ErrorKind(token::Error),
    /// `S!#REF!`, printed with its sheet (references.md "A sheet-qualified #REF! is #REF!"). Only
    /// in the tree as written: [`Node::strip_spaces`] makes it a bare `#REF!` (T-2855).
    SheetRefErrorKind {
        sheet_name: String,
    },
    ParseErrorKind {
        formula: String,
        message: String,
        position: usize,
        // What tokens were expected at this position.
        expecting: Vec<ExpectedTokens>,
    },
    EmptyArgKind,
}

// A `Group` must not grow the node: `Box<Node>` + `u16` fits in the largest variant (SPEC-0007).
const _: () = assert!(std::mem::size_of::<Node>() <= NODE_SIZE);

/// `size_of::<Node>()` before the `Group` variant; pinned so later variants cannot grow it unnoticed.
const NODE_SIZE: usize = 80;

impl Node {
    /// The node under any written parentheses and structured references: `((A1))` is `A1`, and
    /// `Sales[Qty]` is the area it resolves to (SPEC-0016 "Transparency"). Loops, never recurses,
    /// so a group adds no evaluator frame (SPEC-0007 "Seeing through groups").
    pub(crate) fn ungrouped(&self) -> &Node {
        let mut node = self;
        loop {
            match node {
                Node::Group { child, .. }
                | Node::UnaryPlus { child }
                | Node::Spaced { child, .. } => node = child,
                Node::StructuredReferenceKind { lowered, .. } => node = lowered,
                _ => return node,
            }
        }
    }

    /// The area a structured reference resolves to, under any unary plus (the identity) and
    /// written whitespace; any other node is itself. For the sites that see a structured reference as the reference it
    /// stood for before SPEC-0016 but treat written parentheses apart, so they do not call
    /// [`Node::ungrouped`].
    pub(crate) fn lowered(&self) -> &Node {
        let mut node = self;
        loop {
            match node {
                Node::UnaryPlus { child } | Node::Spaced { child, .. } => node = child,
                Node::StructuredReferenceKind { lowered, .. } => node = lowered,
                _ => return node,
            }
        }
    }

    /// [`Node::ungrouped`], with a `#This Row` area resolved at `row`, the row of the cell
    /// evaluating or printing it: the cell or range on that row, or `#VALUE!` off the table's
    /// data rows, the node its parse-time lowering gave before SPEC-0016 step 2 (T-1699). For the
    /// sites that match on the kind of reference an argument is.
    pub(crate) fn ungrouped_at(&self, row: i32) -> Cow<'_, Node> {
        let node = self.ungrouped();
        node.this_row_at(row)
            .map_or(Cow::Borrowed(node), Cow::Owned)
    }

    /// The node itself, but a `#This Row` structured reference (or its bare area) is its area at
    /// `row`, as [`Node::ungrouped_at`] gives it; any other wrapper is kept. For the sites that
    /// match on the kind of the node as written.
    pub(crate) fn resolved_at(&self, row: i32) -> Cow<'_, Node> {
        self.lowered()
            .this_row_at(row)
            .map_or(Cow::Borrowed(self), Cow::Owned)
    }

    /// [`Node::ungrouped`] for static analysis, which has no row: a `#This Row` area is what it is
    /// on a data row, the shape its lowering had on the rows that read the table (T-1699).
    pub(crate) fn ungrouped_static(&self) -> Cow<'_, Node> {
        let node = self.ungrouped();
        node.this_row_on_a_data_row()
            .map_or(Cow::Borrowed(node), Cow::Owned)
    }

    /// [`Node::lowered`] for static analysis, as [`Node::ungrouped_static`] resolves it.
    pub(crate) fn lowered_static(&self) -> Cow<'_, Node> {
        let node = self.lowered();
        node.this_row_on_a_data_row()
            .map_or(Cow::Borrowed(node), Cow::Owned)
    }

    /// The area of a [`Node::ThisRowKind`] on the table's first data row.
    fn this_row_on_a_data_row(&self) -> Option<Node> {
        match self {
            Node::ThisRowKind { data_start, .. } => self.this_row_at(*data_start),
            _ => None,
        }
    }

    /// The area of a [`Node::ThisRowKind`] on `row` (a `ReferenceKind`, a `RangeKind` or
    /// `#VALUE!`); `None` for any other node. A plain comparison: the bounds were computed with
    /// checked arithmetic when the formula was parsed.
    pub(crate) fn this_row_at(&self, row: i32) -> Option<Node> {
        self.this_row_on(Some(row))
    }

    /// A `#This Row` area with no row to resolve it at (a shared formula printed in R1C1): the
    /// same columns on the formula's own row, `RC1` or `RC1:RC2`. Its cells are the ones
    /// [`Node::this_row_at`] gives on every data row.
    pub(crate) fn this_row_relative(&self) -> Option<Node> {
        self.this_row_on(None)
    }

    /// [`Node::this_row_at`] on `Some(row)`, [`Node::this_row_relative`] on `None`.
    fn this_row_on(&self, row: Option<i32>) -> Option<Node> {
        let Node::ThisRowKind {
            sheet_name,
            sheet_index,
            column1,
            column2,
            data_start,
            data_end,
            cell,
        } = self
        else {
            return None;
        };
        let at = match row {
            Some(row) if row < *data_start || row > *data_end => {
                return Some(Node::ErrorKind(token::Error::VALUE));
            }
            Some(row) => (true, row),
            None => (false, 0),
        };
        Some(Node::this_row_reference(
            sheet_name.clone(),
            *sheet_index,
            (*column1, *column2, *cell),
            at,
        ))
    }

    /// The reference to `columns` (first, last, whether it is one cell) on a row, absolute or an
    /// offset from the formula's row.
    fn this_row_reference(
        sheet_name: Option<String>,
        sheet_index: u32,
        (column1, column2, cell): (i32, i32, bool),
        (absolute_row, row): (bool, i32),
    ) -> Node {
        if cell {
            return Node::ReferenceKind {
                sheet_name,
                sheet_index,
                absolute_row,
                absolute_column: true,
                row,
                column: column1,
            };
        }
        Node::RangeKind {
            sheet_name,
            sheet_index,
            absolute_row1: absolute_row,
            absolute_column1: true,
            row1: row,
            column1,
            absolute_row2: absolute_row,
            absolute_column2: true,
            row2: row,
            column2,
        }
    }

    /// Names the sheet `sheet_index` `new_name` in a cell, range or `#This Row` reference that
    /// names its sheet (one on the formula's own sheet names none).
    pub(crate) fn rename_reference_sheet(&mut self, sheet_index: u32, new_name: &str) {
        if let Node::ReferenceKind {
            sheet_name: Some(name),
            sheet_index: index,
            ..
        }
        | Node::RangeKind {
            sheet_name: Some(name),
            sheet_index: index,
            ..
        }
        | Node::ThisRowKind {
            sheet_name: Some(name),
            sheet_index: index,
            ..
        } = self
        {
            if *index == sheet_index {
                new_name.clone_into(name);
            }
        }
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

/// The positions of `names` by scope and lower-cased name, the first of names that fold alike
/// (`Parser::name_index`).
fn index_names(names: &[DefinedNameS]) -> HashMap<(Option<u32>, String), usize> {
    let mut index = HashMap::with_capacity(names.len());
    for (position, (name, scope, _)) in names.iter().enumerate() {
        index
            .entry((*scope, name.to_lowercase()))
            .or_insert(position);
    }
    index
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
        let name_index = index_names(&defined_names);
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
        self.name_index = index_names(&defined_names);
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

    // Returns:
    //  * None: If there is no defined name by that name
    //  * Some((Some(index), formula)): If there is a defined name local to that sheet
    //  * Some(None): If there is a global defined name
    fn get_defined_name(&self, name: &str, sheet: u32) -> Option<(Option<u32>, String)> {
        let lower = name.to_lowercase();
        if let Some(formula) = self.defined_name_formula(Some(sheet), &lower) {
            return Some((Some(sheet), formula.to_owned()));
        }
        self.defined_name_formula(None, &lower)
            .map(|formula| (None, formula.to_owned()))
    }

    /// The body of the name `lower` (already lower-cased) in `scope`, the first of the names that
    /// fold to it; `None` when there is none.
    pub(super) fn defined_name_formula(&self, scope: Option<u32>, lower: &str) -> Option<&str> {
        let index = *self.name_index.get(&(scope, lower.to_owned()))?;
        self.defined_names
            .get(index)
            .map(|(_, _, formula)| formula.as_str())
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
        let (before, signs, counted) = self.parse_prefix();
        if self.open_signs + counted > MAX_SIGNS {
            return self.too_deep();
        }
        self.open_signs += counted;
        let t = self.parse_bare_union();
        self.open_signs -= counted;
        if let Node::ParseErrorKind { .. } = t {
            return t;
        }
        match self.finish_power(t, before, signs) {
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
        // and [`MAX_SIGNS`] bounds how many are open at once.
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
