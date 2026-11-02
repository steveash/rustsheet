//! The parse tree: [`Node`], [`ArrayNode`] and the completion types.

use super::{ChainLeft, DefinedNameS, Spacing, StructuredReference};
use crate::expressions::token;
use crate::expressions::token::OpCompare;
use crate::expressions::token::OpUnary;
use crate::functions::Function;
use crate::nested_array::NestedArray;
use crate::text::Text;
use std::borrow::Cow;
use std::sync::Arc;

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
