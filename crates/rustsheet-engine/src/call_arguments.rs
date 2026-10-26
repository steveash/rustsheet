//! The arguments of every function call in a formula, by shape, for the harness's argument
//! coverage census (T-0077). The harness reuses the engine's parser instead of a second one.

use std::collections::HashMap;

use crate::expressions::parser::{new_parser_english, Node};
use crate::expressions::token::OpUnary;
use crate::expressions::types::CellReferenceRC;
use crate::language::get_default_language;

/// What one argument of a call is, as written.
#[doc(hidden)]
#[derive(Debug, Clone, PartialEq)]
pub enum ArgumentShape {
    /// A number literal, a leading minus folded in (`-2`).
    Number(f64),
    /// A string literal.
    Text(String),
    /// `TRUE` or `FALSE`.
    Bool(bool),
    /// An error literal (`#N/A`).
    Error,
    /// A single-cell reference, resolved against the formula's cell: `sheet` is `None` for the
    /// formula's own sheet; `row` and `column` are 1-based.
    Cell {
        /// The sheet the reference names, if any.
        sheet: Option<String>,
        /// 1-based row.
        row: i32,
        /// 1-based column.
        column: i32,
    },
    /// A multi-cell range, a 3-D reference or an array constant.
    Area,
    /// An argument left out (`ROUND(1,)`).
    Omitted,
    /// Anything else: a nested call, an operator, a name.
    Other,
}

/// One call of a worksheet function and its arguments.
#[doc(hidden)]
#[derive(Debug, Clone, PartialEq)]
pub struct CallArguments {
    /// The en-US name, e.g. `XLOOKUP`.
    pub name: String,
    /// Each argument, in order.
    pub args: Vec<ArgumentShape>,
}

/// Testing hook, not public API (no stability promise): every worksheet function call in
/// `formula`, parsed as written in `sheet` at 1-based (`row`, `column`) of a workbook with
/// `sheets`, each call before the calls nested in it. Names, LAMBDA calls and unknown functions
/// are left out; a formula that does not parse has none.
#[doc(hidden)]
#[must_use]
pub fn call_arguments(
    formula: &str,
    sheets: &[String],
    sheet: &str,
    row: i32,
    column: i32,
) -> Vec<CallArguments> {
    let mut parser = new_parser_english(sheets.to_vec(), Vec::new(), HashMap::new());
    let context = CellReferenceRC {
        sheet: sheet.to_string(),
        row,
        column,
    };
    let tree = parser.parse(formula.trim_start_matches('='), &context);
    let english = get_default_language();
    let mut calls = Vec::new();
    // An explicit stack keeps the walk off the call stack: a formula's tree is untrusted input.
    let mut pending = vec![&tree];
    while let Some(node) = pending.pop() {
        if let Node::FunctionKind { kind, args } = node {
            calls.push(CallArguments {
                name: kind.to_localized_name(english),
                args: args.iter().map(|a| shape(a, &context)).collect(),
            });
        }
        node.push_children(&mut pending);
    }
    calls
}

/// The shape of one argument written in the cell at `context`.
fn shape(node: &Node, context: &CellReferenceRC) -> ArgumentShape {
    match node.ungrouped() {
        Node::NumberKind(n) => ArgumentShape::Number(*n),
        Node::UnaryKind {
            kind: OpUnary::Minus,
            right,
        } => match right.ungrouped() {
            Node::NumberKind(n) => ArgumentShape::Number(-n),
            _ => ArgumentShape::Other,
        },
        Node::StringKind(s) => ArgumentShape::Text(s.clone()),
        Node::BooleanKind(b) => ArgumentShape::Bool(*b),
        Node::ErrorKind(_) => ArgumentShape::Error,
        Node::EmptyArgKind => ArgumentShape::Omitted,
        Node::ReferenceKind {
            sheet_name,
            absolute_row,
            absolute_column,
            row,
            column,
            ..
        } => ArgumentShape::Cell {
            sheet: sheet_name.clone(),
            // A relative reference is stored as an offset from the formula's cell (A1 mode).
            row: if *absolute_row {
                *row
            } else {
                row.saturating_add(context.row)
            },
            column: if *absolute_column {
                *column
            } else {
                column.saturating_add(context.column)
            },
        },
        Node::RangeKind { .. }
        | Node::SheetSpanKind { .. }
        | Node::ArrayKind(_)
        | Node::OpRangeKind { .. } => ArgumentShape::Area,
        _ => ArgumentShape::Other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn calls(formula: &str) -> Vec<CallArguments> {
        let sheets = vec!["Calc".to_string(), "Data".to_string()];
        call_arguments(formula, &sheets, "Calc", 5, 3)
    }

    #[test]
    fn classifies_literal_reference_and_area_arguments() {
        let got = calls(r#"=IF(TRUE,ROUND(-2.5,),SUM(Data!A1:B2,{1,2},"x",#N/A,$B$2,D6))"#);
        let names: Vec<&str> = got.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(names.len(), 3);
        assert_eq!(names[0], "IF");
        let round = got
            .iter()
            .find(|c| c.name == "ROUND")
            .map(|c| c.args.clone());
        assert_eq!(
            round,
            Some(vec![ArgumentShape::Number(-2.5), ArgumentShape::Omitted])
        );
        let sum = got.iter().find(|c| c.name == "SUM").map(|c| c.args.clone());
        assert_eq!(
            sum,
            Some(vec![
                ArgumentShape::Area,
                ArgumentShape::Area,
                ArgumentShape::Text("x".to_string()),
                ArgumentShape::Error,
                ArgumentShape::Cell {
                    sheet: None,
                    row: 2,
                    column: 2
                },
                // D6 written in C5.
                ArgumentShape::Cell {
                    sheet: None,
                    row: 6,
                    column: 4
                },
            ])
        );
        assert_eq!(got[0].args[0], ArgumentShape::Bool(true));
        assert_eq!(got[0].args[1], ArgumentShape::Other);
    }

    #[test]
    fn sheet_qualified_and_parenthesized_arguments() {
        let got = calls("ABS((Data!B3))");
        assert_eq!(
            got[0].args,
            vec![ArgumentShape::Cell {
                sheet: Some("Data".to_string()),
                row: 3,
                column: 2
            }]
        );
        assert!(calls("=foo(").is_empty());
        assert!(calls("=1+").is_empty());
    }
}
