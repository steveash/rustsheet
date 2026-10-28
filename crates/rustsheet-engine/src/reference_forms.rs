//! Which reference forms a formula holds (defined name, structured, 3-D, external workbook), for
//! the harness's M8 subset of the poi suite (decision 0146 item 2(b), T-0094). The harness reuses
//! the engine's parser instead of matching formula text.

use std::collections::HashMap;

use crate::expressions::parser::{new_parser_english, DefinedNameS, Node, Parser};
use crate::expressions::types::CellReferenceRC;
use crate::reference_operators::names_other_workbook;
use crate::types::Table;

/// The reference forms one formula holds, as the parser reads it.
#[doc(hidden)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ReferenceForms {
    /// A defined name of this workbook (a LET or LAMBDA name is not one).
    pub defined_name: bool,
    /// A structured reference (`Sales[Qty]`, a bare table name, `[@Qty]`).
    pub structured: bool,
    /// A 3-D reference (`Jan:Mar!A1`).
    pub three_d: bool,
    /// A reference to another workbook (`[1]Sheet!A1`, `[1]!Name`).
    pub external: bool,
}

impl ReferenceForms {
    /// Whether the formula holds any of the four forms.
    #[must_use]
    pub fn any(self) -> bool {
        self.defined_name || self.structured || self.three_d || self.external
    }
}

/// Testing hook, not public API (no stability promise): classifies formulas of one workbook,
/// parsed as its xlsx import parses them (stored text, the workbook's sheets, defined names and
/// tables). One parser serves every formula: building one clones the names and tables.
#[doc(hidden)]
pub struct ReferenceFormsReader {
    parser: Parser,
}

impl ReferenceFormsReader {
    /// A reader for a workbook with `sheets` in tab order, `defined_names` as
    /// `(name, sheet the name is local to, body)`, and `tables`.
    #[must_use]
    pub fn new(
        sheets: Vec<String>,
        defined_names: &[(String, Option<String>, String)],
        tables: Vec<Table>,
    ) -> Self {
        let names: Vec<DefinedNameS> = defined_names
            .iter()
            .filter_map(|(name, sheet, body)| {
                let scope = match sheet {
                    None => None,
                    // A name local to a sheet the workbook lacks names nothing.
                    Some(sheet) => Some(
                        u32::try_from(sheets.iter().position(|s| s.eq_ignore_ascii_case(sheet))?)
                            .ok()?,
                    ),
                };
                Some((name.clone(), scope, body.clone()))
            })
            .collect();
        let tables: HashMap<String, Table> =
            tables.into_iter().map(|t| (t.name.clone(), t)).collect();
        let mut parser = new_parser_english(sheets, names, tables);
        parser.set_loaded(true);
        Self { parser }
    }

    /// The forms `formula` (stored text, no `=`) holds, parsed as written in `sheet` at 1-based
    /// (`row`, `column`); `None` when it does not parse (a parse error anywhere in its tree,
    /// including a formula nested past the parser's depth bound).
    pub fn read(
        &mut self,
        formula: &str,
        sheet: &str,
        row: i32,
        column: i32,
    ) -> Option<ReferenceForms> {
        let context = CellReferenceRC {
            sheet: sheet.to_string(),
            row,
            column,
        };
        let tree = self.parser.parse(formula.trim_start_matches('='), &context);
        let mut forms = ReferenceForms::default();
        // An explicit stack keeps the walk off the call stack: a formula's tree is untrusted input.
        let mut pending = vec![&tree];
        while let Some(node) = pending.pop() {
            match node {
                Node::ParseErrorKind { .. } => return None,
                // `[1]!Name`, a name of a linked workbook, is an external reference.
                Node::DefinedNameKind((name, ..)) if name.starts_with('[') => forms.external = true,
                Node::DefinedNameKind(_) => forms.defined_name = true,
                Node::StructuredReferenceKind { .. } | Node::ThisRowKind { .. } => {
                    forms.structured = true;
                }
                // A span of a linked workbook's sheets reads as a reference on `[1]First:Last`.
                Node::SheetSpanKind { .. } => forms.three_d = true,
                Node::ReferenceKind { sheet_name, .. }
                | Node::RangeKind { sheet_name, .. }
                | Node::WrongReferenceKind { sheet_name, .. }
                | Node::WrongRangeKind { sheet_name, .. }
                    if names_other_workbook(sheet_name.as_deref()) =>
                {
                    forms.external = true;
                }
                _ => {}
            }
            node.push_children(&mut pending);
        }
        Some(forms)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::TableColumn;

    fn reader() -> ReferenceFormsReader {
        let table = Table {
            name: "Sales".to_string(),
            display_name: "Sales".to_string(),
            sheet_name: "Jan".to_string(),
            reference: "A1:B4".to_string(),
            totals_row_count: 0,
            header_row_count: 1,
            header_row_dxf_id: None,
            data_dxf_id: None,
            totals_row_dxf_id: None,
            columns: ["Item", "Qty"]
                .iter()
                .zip(1..)
                .map(|(name, id)| TableColumn {
                    id,
                    name: (*name).to_string(),
                    ..TableColumn::default()
                })
                .collect(),
            style_info: crate::types::TableStyleInfo::default(),
            has_filters: false,
        };
        ReferenceFormsReader::new(
            ["Jan", "Feb", "Mar"].map(str::to_string).to_vec(),
            &[
                ("Rate".to_string(), None, "Jan!$A$1".to_string()),
                (
                    "Local".to_string(),
                    Some("Feb".to_string()),
                    "Feb!$A$1".to_string(),
                ),
            ],
            vec![table],
        )
    }

    fn read(formula: &str) -> Option<ReferenceForms> {
        reader().read(formula, "Jan", 10, 1)
    }

    #[test]
    fn each_form_alone() {
        let only = |f: fn(&mut ReferenceForms)| {
            let mut forms = ReferenceForms::default();
            f(&mut forms);
            Some(forms)
        };
        assert_eq!(read("Rate*2"), only(|f| f.defined_name = true));
        assert_eq!(read("SUM(Sales[Qty])"), only(|f| f.structured = true));
        assert_eq!(read("ROWS(Sales)"), only(|f| f.structured = true));
        assert_eq!(read("SUM(Jan:Mar!A1)"), only(|f| f.three_d = true));
        assert_eq!(read("[1]Sheet1!A1+1"), only(|f| f.external = true));
        assert_eq!(read("SUM([1]Sheet1!A1:B2)"), only(|f| f.external = true));
        assert_eq!(read("[1]!Price*2"), only(|f| f.external = true));
    }

    #[test]
    fn a_formula_with_none() {
        let forms = read("SUM(A1:B2)+Feb!C3*2").expect("parses");
        assert!(!forms.any());
        // An unknown name is no defined name.
        assert_eq!(read("Unknown+1").map(ReferenceForms::any), Some(false));
    }

    #[test]
    fn a_let_or_lambda_name_is_not_a_defined_name() {
        assert_eq!(
            read("LET(Rate,2,Rate*3)").map(ReferenceForms::any),
            Some(false)
        );
        assert_eq!(
            read("LAMBDA(Rate,Rate*3)(2)").map(ReferenceForms::any),
            Some(false)
        );
    }

    #[test]
    fn a_sheet_local_name_counts_only_on_its_sheet() {
        let mut reader = reader();
        assert_eq!(
            reader.read("Local+1", "Feb", 1, 1).map(|f| f.defined_name),
            Some(true)
        );
        assert_eq!(
            reader.read("Local+1", "Jan", 1, 1).map(|f| f.defined_name),
            Some(false)
        );
    }

    #[test]
    fn an_unparsable_formula_is_none() {
        assert_eq!(read("SUM(1,"), None);
        assert_eq!(read("1+*2"), None);
    }

    #[test]
    fn a_formula_nested_past_the_depth_bound_is_none() {
        let deep = format!("{}1{}", "(".repeat(100_000), ")".repeat(100_000));
        assert_eq!(read(&deep), None);
        let calls = format!("{}1{}", "ABS(".repeat(100_000), ")".repeat(100_000));
        assert_eq!(read(&calls), None);
    }
}
