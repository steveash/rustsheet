//! The spill-extent check of a spec case whose sidecar is Excel evidence (T-1660, decision 0063).
//!
//! The oracle records every cell Excel's formulas fill, spill members included (`formula_values`
//! in `tools/src/rustsheet_tools/snapshot.py`), so a cell the engine fills that is neither an
//! input nor recorded is one Excel left empty: it is scored as a mismatch against empty, and a
//! spill wider than Excel's cannot pass unseen (probes/radians-sqrtpi-munit B80).

use std::collections::{BTreeMap, BTreeSet, HashSet};

use crate::case::{cell_key, CaseSpec, SheetOp};
use crate::cellref::{parse_range, Addr};
use crate::engine::Evaluated;
use crate::runner::{CellKind, CellResult};
use crate::value::{compare, Value};
use crate::volatile::FormulaCell;
use crate::xlsx_snapshot::is_undecoded;

/// The largest rectangle checked when deciding whether a filled cell may belong to a pending
/// formula's spill; a larger one is given the benefit of the doubt.
const MAX_SHADOW_CELLS: u64 = 1 << 16;

/// The cells a spec case's engine result may fill, and the formulas whose extent is unknown.
pub(crate) struct SpillExtent {
    /// The case's sheets, by spec name.
    sheets: Vec<String>,
    /// `cell_key`s of the input cells and of every expectation (oracle or hand).
    known: HashSet<String>,
    /// Formula cells without a decoded oracle value: Excel's extent of their spill is unknown.
    pending: Vec<(String, Addr)>,
}

/// Where `addr` on `sheet` sits after the spec's `sheet_ops`. Excel keeps a table's last data row
/// when it is deleted (T-2338, probes/table-edits/table-delete-row-all-data-below), and keeps one
/// when a single delete takes every data row (T-3019, table-delete-rows-all-data-once): one row
/// fewer goes, so the cells below the table stay one row lower than a plain delete leaves them.
fn moved_input(spec: &CaseSpec, sheet: &str, addr: Addr) -> Addr {
    // The data rows (first, last) of each table on `sheet`, followed through the ops.
    let mut tables: Vec<(u32, u32)> = spec
        .sheets
        .iter()
        .filter(|s| s.name == sheet)
        .flat_map(|s| &s.tables)
        .filter_map(|t| {
            let (first, last) = parse_range(&t.range)?;
            let last_data = if t.totals_row {
                last.row.checked_sub(1)?
            } else {
                last.row
            };
            // Without a header row the data starts on the table's first row (T-3260).
            let first_data = first.row + u32::from(t.header_row);
            (first_data <= last_data).then_some((first_data, last_data))
        })
        .collect();
    let mut addr = addr;
    for op in &spec.sheet_ops {
        let SheetOp::DeleteRow {
            sheet: at,
            row,
            count,
        } = op
        else {
            addr = op.moved(sheet, addr);
            continue;
        };
        if at != sheet {
            continue;
        }
        let (mut row, mut count) = (*row, count.unwrap_or(1));
        let end = row.saturating_add(count);
        if tables
            .iter()
            .any(|&(first, last)| row == first && last.saturating_add(1) == end)
        {
            // The table's first data row stays, cleared; the rows after it go. Only a delete of
            // exactly the data rows is probed: one reaching the header, the totals row or rows
            // outside the table is moved as a plain delete.
            row += 1;
            count -= 1;
        }
        if count == 0 {
            continue;
        }
        for (first, last) in &mut tables {
            *first = first.saturating_sub(first.saturating_sub(row).min(count));
            *last = last.saturating_sub(last.saturating_add(1).saturating_sub(row).min(count));
        }
        let kept = SheetOp::DeleteRow {
            sheet: sheet.to_string(),
            row,
            count: Some(count),
        };
        addr = kept.moved(sheet, addr);
    }
    addr
}

impl SpillExtent {
    /// The check for `spec`, whose `expected` values include the oracle's `oracle_cells`.
    pub(crate) fn new(
        spec: &CaseSpec,
        expected: &BTreeMap<String, Value>,
        oracle_cells: &BTreeSet<String>,
        formulas: &[FormulaCell],
    ) -> Self {
        Self {
            sheets: spec.sheets.iter().map(|s| s.name.clone()).collect(),
            known: spec
                .sheets
                .iter()
                .flat_map(|s| {
                    // An input moved by a row insert is known at its new address (T-1933); a
                    // pivot's rendered cells are inputs of the engine case (T-2084).
                    let rendered = s.pivot_tables.iter().flat_map(|p| p.rendered.keys());
                    let inputs = s.cells.keys().chain(rendered).filter_map(|a1| {
                        Some(cell_key(
                            &s.name,
                            moved_input(spec, &s.name, Addr::parse(a1)?),
                        ))
                    });
                    // A data table's result cells are declared by the case; Excel saves all but
                    // the first as plain values, which the oracle does not record (T-0441).
                    let results = s.data_tables.iter().filter_map(|t| parse_range(&t.range));
                    let results = results.flat_map(|(first, last)| {
                        (first.row..=last.row).flat_map(move |row| {
                            (first.col..=last.col).map(move |col| Addr { row, col })
                        })
                    });
                    inputs.chain(results.map(|addr| cell_key(&s.name, addr)))
                })
                .chain(expected.keys().cloned())
                .collect(),
            pending: formulas
                .iter()
                .filter(|(sheet, addr, _)| {
                    let key = cell_key(sheet, *addr);
                    !oracle_cells.contains(&key) || expected.get(&key).is_some_and(is_undecoded)
                })
                .map(|(sheet, addr, _)| (sheet.clone(), *addr))
                .collect(),
        }
    }

    /// A failing [`CellResult`] for every filled cell outside [`Self::known`].
    pub(crate) fn unexpected_cells(&self, evaluated: &dyn Evaluated) -> Vec<CellResult> {
        let mut cells = Vec::new();
        for sheet in &self.sheets {
            let stored = evaluated.stored_cells(sheet);
            let stored_set: HashSet<Addr> = stored.iter().copied().collect();
            for addr in stored {
                if self.known.contains(&cell_key(sheet, addr))
                    || self.in_pending_spill(sheet, addr, &stored_set)
                {
                    continue;
                }
                if let Some(actual) = evaluated.value(sheet, addr) {
                    if !compare(&Value::Empty, &actual).is_pass() {
                        cells.push(unexpected(sheet, addr, actual));
                    }
                }
            }
        }
        cells
    }

    /// Whether `addr` may be a spill member of a pending formula: one up and to the left of it
    /// with every cell of the rectangle between them filled, as a spill fills it.
    fn in_pending_spill(&self, sheet: &str, addr: Addr, stored: &HashSet<Addr>) -> bool {
        self.pending.iter().any(|(pending_sheet, anchor)| {
            if pending_sheet != sheet || anchor.row > addr.row || anchor.col > addr.col {
                return false;
            }
            let area = u64::from(addr.row - anchor.row + 1) * u64::from(addr.col - anchor.col + 1);
            area > MAX_SHADOW_CELLS
                || (anchor.row..=addr.row).all(|row| {
                    (anchor.col..=addr.col).all(|col| stored.contains(&Addr { row, col }))
                })
        })
    }
}

/// A cell Excel left empty that the engine filled with `actual`.
fn unexpected(sheet: &str, addr: Addr, actual: Value) -> CellResult {
    CellResult {
        sheet: sheet.to_string(),
        cell: addr.to_string(),
        formula: None,
        anchor_formula: None,
        verdict: compare(&Value::Empty, &actual),
        expected: Value::Empty,
        actual,
        from_excel: true,
        listed_deviation: false,
        graph_subnormal: false,
        kind: CellKind::SpillExtent,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::case::CASE_FORMAT;
    use crate::value::Verdict;

    /// A spill result: `S!A1` = `{1,2,3}` over A1:C1 and the input `S!A2`.
    struct Spilled;
    impl Evaluated for Spilled {
        fn value(&self, sheet: &str, addr: Addr) -> Option<Value> {
            (sheet == "S").then(|| match (addr.row, addr.col) {
                (1, col @ 1..=3) => Value::Number(f64::from(col)),
                (2, 1) => Value::Text("x".into()),
                _ => Value::Empty,
            })
        }
        fn stored_cells(&self, sheet: &str) -> Vec<Addr> {
            let cells = [(1, 1), (1, 2), (1, 3), (2, 1)];
            cells
                .into_iter()
                .filter(|_| sheet == "S")
                .map(|(row, col)| Addr { row, col })
                .collect()
        }
    }

    /// The check for `=SEQUENCE(1,3)` in S!A1 beside the input S!A2, Excel having recorded the
    /// `oracle` cells.
    fn extent(oracle: &[&str]) -> SpillExtent {
        let json = format!(
            r#"{{"format":"{CASE_FORMAT}","sheets":[{{"name":"S","cells":{{"A1":"=SEQUENCE(1,3)","A2":"x"}}}}]}}"#
        );
        let spec: CaseSpec = serde_json::from_str(&json).expect("spec");
        let oracle_cells: BTreeSet<String> = oracle
            .iter()
            .map(|a1| cell_key("S", Addr::parse(a1).expect("addr")))
            .collect();
        let expected = oracle_cells
            .iter()
            .map(|key| (key.clone(), Value::Number(0.0)))
            .collect();
        let anchor = Addr::parse("A1").expect("addr");
        let formulas = vec![("S".to_string(), anchor, "SEQUENCE(1,3)".to_string())];
        SpillExtent::new(&spec, &expected, &oracle_cells, &formulas)
    }

    #[test]
    fn spill_wider_than_excel_fails_the_extra_cells() {
        let cells = extent(&["A1"]).unexpected_cells(&Spilled);
        // The report keeps only failing cells, and the M7 exit counts these from it (decision
        // 0140), so each one is a failing, marked cell.
        assert!(cells
            .iter()
            .all(|c| c.kind == CellKind::SpillExtent && !c.verdict.is_pass()));
        let extra: Vec<(String, Value, Verdict)> = cells
            .into_iter()
            .map(|c| (c.key(), c.actual, c.verdict))
            .collect();
        assert_eq!(
            extra,
            vec![
                ("S!B1".into(), Value::Number(2.0), Verdict::TypeMismatch),
                ("S!C1".into(), Value::Number(3.0), Verdict::TypeMismatch),
            ]
        );
        assert!(extent(&["A1", "B1", "C1"])
            .unexpected_cells(&Spilled)
            .is_empty());
    }

    /// Deleting row 4 of a one-row table clears it and moves nothing (T-2338), so S!C10 sits at
    /// C7 after four deletes of a four-row table, not at C6.
    #[test]
    fn a_table_keeps_its_last_data_row_when_following_moved_inputs() {
        let json = format!(
            r#"{{"format":"{CASE_FORMAT}","sheets":[{{"name":"S","cells":{{}},
            "tables":[{{"name":"T","ref":"B3:C7"}}]}}],
            "sheet_ops":[{}]}}"#,
            [r#"{"op":"delete_row","sheet":"S","row":4}"#; 4].join(",")
        );
        let spec: CaseSpec = serde_json::from_str(&json).expect("spec");
        let c10 = Addr::parse("C10").expect("addr");
        assert_eq!(
            moved_input(&spec, "S", c10),
            Addr::parse("C7").expect("addr")
        );
    }

    /// One delete of every data row keeps one cleared row too (T-3019): rows 5-7 go, so S!C10
    /// sits at C7, and a later one-row delete below the table moves it on to C6.
    #[test]
    fn a_counted_delete_of_every_data_row_keeps_one() {
        let json = format!(
            r#"{{"format":"{CASE_FORMAT}","sheets":[{{"name":"S","cells":{{}},
            "tables":[{{"name":"T","ref":"B3:C7"}}]}}],
            "sheet_ops":[{},{}]}}"#,
            r#"{"op":"delete_row","sheet":"S","row":4,"count":4}"#,
            r#"{"op":"delete_row","sheet":"S","row":5}"#
        );
        let spec: CaseSpec = serde_json::from_str(&json).expect("spec");
        let at = |a1| Addr::parse(a1).expect("addr");
        assert_eq!(moved_input(&spec, "S", at("C10")), at("C6"));
        assert_eq!(moved_input(&spec, "S", at("C2")), at("C2"));
    }

    /// A1 left pending: its spill extent in Excel is unknown, so its members are not checked.
    #[test]
    fn a_pending_formula_spill_is_not_checked() {
        let check = extent(&["A2"]);
        assert_eq!(check.pending.len(), 1);
        assert!(check.unexpected_cells(&Spilled).is_empty());
    }

    /// Spill members Excel filled with rich errors the harness cannot decode stay in the sidecar
    /// as `#UNDECODED!` (`rustsheet-oracle fill`, T-2397; gen-m7/18b80d3fd292d63a C951:E962):
    /// they are known cells, so the engine's values there are not scored against a blank.
    #[test]
    fn undecoded_spill_members_are_not_expected_blanks() {
        let json = format!(
            r#"{{"format":"{CASE_FORMAT}","sheets":[{{"name":"S","cells":{{"A1":"=SEQUENCE(1,3)","A2":"x"}}}}]}}"#
        );
        let spec: CaseSpec = serde_json::from_str(&json).expect("spec");
        let undecoded = Value::Error(crate::xlsx_snapshot::UNDECODED_RICH_ERROR.into());
        let expected: BTreeMap<String, Value> = [
            ("A1", Value::Number(1.0)),
            ("B1", undecoded.clone()),
            ("C1", undecoded),
        ]
        .into_iter()
        .map(|(a1, v)| (cell_key("S", Addr::parse(a1).expect("addr")), v))
        .collect();
        let oracle_cells = expected.keys().cloned().collect();
        let formulas = vec![(
            "S".to_string(),
            Addr::parse("A1").expect("addr"),
            "SEQUENCE(1,3)".to_string(),
        )];
        let check = SpillExtent::new(&spec, &expected, &oracle_cells, &formulas);
        assert!(check.pending.is_empty());
        assert!(check.unexpected_cells(&Spilled).is_empty());
    }
}
