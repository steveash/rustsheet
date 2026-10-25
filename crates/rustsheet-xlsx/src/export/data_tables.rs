//! What-if data tables on export: the anchor (top-left result cell) of each table carries
//! `<f t="dataTable" ref=… dt2D=… dtr=… r1=… r2=…/>` before its value; the other result cells
//! are plain values, as Excel saves them (docs/excel-semantics/recalc.md, T-0441).

use std::collections::HashMap;

use rustsheet_engine::{
    expressions::utils::number_to_column,
    types::{DataTable, DataTableInput},
};

fn a1(row: i32, column: i32) -> String {
    number_to_column(column).map_or_else(String::new, |c| format!("{c}{row}"))
}

/// The `<f>` element of each table, keyed by its anchor's (row, column).
pub(super) fn anchor_formulas(tables: &[DataTable]) -> HashMap<(i32, i32), String> {
    tables
        .iter()
        .map(|t| {
            let (top, left, bottom, right) = t.range;
            // A two-variable table stores the row input as r1 and the column input as r2; a
            // one-variable table stores its only input as r1, with dtr set for a row input.
            let (two, row, inputs) = match (t.row_input, t.column_input) {
                (Some(r), Some(c)) => (true, true, [Some(r), Some(c)]),
                (Some(r), None) => (false, true, [Some(r), None]),
                (None, c) => (false, false, [c, None]),
            };
            let mut f = format!(
                r#"<f t="dataTable" ref="{}:{}" dt2D="{}" dtr="{}""#,
                a1(top, left),
                a1(bottom, right),
                u8::from(two),
                u8::from(row)
            );
            for (i, input) in inputs.iter().enumerate() {
                match input {
                    Some(DataTableInput::Cell(r, c)) => {
                        f.push_str(&format!(r#" r{}="{}""#, i + 1, a1(*r, *c)));
                    }
                    Some(DataTableInput::Deleted) => f.push_str(&format!(r#" del{}="1""#, i + 1)),
                    None => {}
                }
            }
            f.push_str("/>");
            ((top, left), f)
        })
        .collect()
}

/// `cell` (one `<c>` element) with `formula` as its first child.
pub(super) fn with_formula(cell: &str, formula: &str) -> String {
    if let Some(open) = cell.strip_suffix("/>").filter(|c| !c.contains('>')) {
        return format!("{open}>{formula}</c>");
    }
    match cell.find('>') {
        Some(end) => format!("{}{formula}{}", &cell[..=end], &cell[end + 1..]),
        None => cell.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formulas_follow_the_table_kind() {
        let t = |row_input, column_input| DataTable {
            range: (4, 2, 6, 3),
            row_input,
            column_input,
        };
        let cell = Some(DataTableInput::Cell(1, 1));
        let f = |table| anchor_formulas(&[table])[&(4, 2)].clone();
        assert_eq!(
            f(t(None, cell)),
            r#"<f t="dataTable" ref="B4:C6" dt2D="0" dtr="0" r1="A1"/>"#
        );
        assert_eq!(
            f(t(cell, None)),
            r#"<f t="dataTable" ref="B4:C6" dt2D="0" dtr="1" r1="A1"/>"#
        );
        assert_eq!(
            f(t(
                Some(DataTableInput::Cell(1, 2)),
                Some(DataTableInput::Deleted)
            )),
            r#"<f t="dataTable" ref="B4:C6" dt2D="1" dtr="1" r1="B1" del2="1"/>"#
        );
    }

    #[test]
    fn the_formula_goes_before_the_value() {
        assert_eq!(
            with_formula(r#"<c r="B4" s="1"><v>12</v></c>"#, "<f/>"),
            r#"<c r="B4" s="1"><f/><v>12</v></c>"#
        );
        assert_eq!(
            with_formula(r#"<c r="B4"/>"#, "<f/>"),
            r#"<c r="B4"><f/></c>"#
        );
    }
}
