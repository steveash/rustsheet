//! The spill-extent check of a spec case whose sidecar is Excel evidence (T-1660, decision 0063).
//!
//! The oracle records every cell Excel's formulas fill, spill members included (`formula_values`
//! in `tools/src/rustsheet_tools/snapshot.py`), so a cell the engine fills that is neither an
//! input nor recorded is one Excel left empty: it is scored as a mismatch against empty, and a
//! spill wider than Excel's cannot pass unseen (probes/radians-sqrtpi-munit B80).

use std::collections::{BTreeMap, BTreeSet, HashSet};

use crate::case::{cell_key, CaseSpec};
use crate::cellref::Addr;
use crate::engine::Evaluated;
use crate::runner::CellResult;
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
                    s.cells
                        .keys()
                        .filter_map(|a1| Some(cell_key(&s.name, Addr::parse(a1)?)))
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
        verdict: compare(&Value::Empty, &actual),
        expected: Value::Empty,
        actual,
        from_excel: true,
        listed_deviation: false,
        graph_subnormal: false,
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
        let extra: Vec<(String, Value, Verdict)> = extent(&["A1"])
            .unexpected_cells(&Spilled)
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

    /// A1 left pending: its spill extent in Excel is unknown, so its members are not checked.
    #[test]
    fn a_pending_formula_spill_is_not_checked() {
        let check = extent(&["A2"]);
        assert_eq!(check.pending.len(), 1);
        assert!(check.unexpected_cells(&Spilled).is_empty());
    }
}
