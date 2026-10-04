//! Oracle calibration: compares an Excel-authored workbook's cached values against the same
//! workbook recalculated by an oracle backend (T-0004, docs/research/oracle-calibration.md).
//!
//! Uses the harness's own comparison rules ([`compare`]), volatile analysis and function
//! attribution, so calibration numbers mean the same thing as eval scores.

use std::collections::HashMap;

use serde::Serialize;

use crate::report::functions_in;
use crate::value::{compare, Value, Verdict};
use crate::volatile::{formula_cells, volatile_cells};
use crate::xlsx_snapshot::WorkbookSnapshot;

/// One formula cell of the cached workbook and what the recalculation produced for it.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CellComparison {
    pub sheet: String,
    pub cell: String,
    pub formula: String,
    pub functions: Vec<String>,
    pub cached: Value,
    /// `None` when the recalculated workbook has no such cell.
    pub recalculated: Option<Value>,
    /// `None` when the cell is missing from the recalculated workbook.
    pub verdict: Option<Verdict>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Comparison {
    /// Formula cells left out because they depend on RAND/NOW/TODAY-like functions: a
    /// recalculation at another time legitimately differs (the oracle clock is never pinned).
    pub volatile_skipped: usize,
    pub cells: Vec<CellComparison>,
}

/// Compares every non-volatile formula cell of `cached` with the same cell in `recalculated`.
/// Cells only present in `recalculated` (e.g. an embedded canary sheet) are ignored.
#[must_use]
pub fn compare_snapshots(cached: &WorkbookSnapshot, recalculated: &WorkbookSnapshot) -> Comparison {
    let volatile = volatile_cells(&formula_cells(&cached.sheets), false);
    let recalculated: HashMap<&str, _> = recalculated
        .sheets
        .iter()
        .map(|s| (s.name.as_str(), &s.cells))
        .collect();
    let mut cells = Vec::new();
    let mut volatile_skipped = 0;
    for sheet in &cached.sheets {
        for (addr, cell) in &sheet.cells {
            let Some(formula) = cell.formula.as_ref().filter(|_| cell.is_formula()) else {
                continue;
            };
            if volatile.contains(&(sheet.name.clone(), *addr)) {
                volatile_skipped += 1;
                continue;
            }
            let other = recalculated
                .get(sheet.name.as_str())
                .and_then(|cells| cells.get(addr))
                .map(|c| c.value.clone());
            cells.push(CellComparison {
                sheet: sheet.name.clone(),
                cell: addr.to_string(),
                formula: formula.clone(),
                functions: functions_in(formula).into_iter().collect(),
                verdict: other.as_ref().map(|o| compare(&cell.value, o)),
                cached: cell.value.clone(),
                recalculated: other,
            });
        }
    }
    Comparison {
        volatile_skipped,
        cells,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cellref::Addr;
    use crate::xlsx_snapshot::{CellSnapshot, FormulaKind, SheetSnapshot};
    use std::collections::BTreeMap;

    fn workbook(cells: &[(&str, Option<&str>, Value)]) -> WorkbookSnapshot {
        let mut map = BTreeMap::new();
        for (a1, formula, value) in cells {
            map.insert(
                Addr::parse(a1).expect("addr"),
                CellSnapshot {
                    formula: formula.map(str::to_string),
                    formula_kind: formula.map(|_| FormulaKind::Normal),
                    value: value.clone(),
                },
            );
        }
        WorkbookSnapshot {
            application: None,
            app_version: None,
            date1904: false,
            sheets: vec![SheetSnapshot {
                name: "S".into(),
                cells: map,
            }],
        }
    }

    #[test]
    fn compares_formula_cells_and_skips_volatile_ones() {
        let cached = workbook(&[
            ("A1", None, Value::Number(1.0)),
            ("B1", Some("SUM(A1)"), Value::Number(1.0)),
            ("B2", Some("ROUND(A1/3,2)"), Value::Number(0.33)),
            ("B3", Some("RAND()"), Value::Number(0.5)),
            ("B4", Some("B3*2"), Value::Number(1.0)),
            ("B5", Some("UPPER(\"x\")"), Value::Text("X".into())),
        ]);
        let recalculated = workbook(&[
            ("A1", None, Value::Number(1.0)),
            ("B1", Some("SUM(A1)"), Value::Number(1.0)),
            ("B2", Some("ROUND(A1/3,2)"), Value::Number(0.34)),
            ("B3", Some("RAND()"), Value::Number(0.9)),
            ("B4", Some("B3*2"), Value::Number(1.8)),
        ]);
        let got = compare_snapshots(&cached, &recalculated);
        assert_eq!(got.volatile_skipped, 2); // RAND() and its dependant
        let verdicts: Vec<_> = got
            .cells
            .iter()
            .map(|c| (c.cell.as_str(), c.verdict))
            .collect();
        assert_eq!(
            verdicts,
            vec![
                ("B1", Some(Verdict::Pass)),
                ("B2", Some(Verdict::NumericMismatch)),
                ("B5", None), // missing from the recalculation
            ]
        );
        assert_eq!(got.cells[1].functions, vec!["ROUND".to_string()]);
    }
}
