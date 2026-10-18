//! Oracle calibration: compares an Excel-authored workbook's cached values against the same
//! workbook recalculated by an oracle backend (T-0004, docs/research/oracle-calibration.md).
//!
//! Uses the harness's own comparison rules ([`compare`]), volatile analysis and function
//! attribution, so calibration numbers mean the same thing as eval scores.

use serde::Serialize;

use crate::report::functions_in;
use crate::value::{compare, Value, Verdict};
use crate::volatile::scorable_cells;
use crate::xlsx_snapshot::WorkbookSnapshot;

/// One formula cell of the cached workbook and what the recalculation produced for it.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CellComparison {
    pub sheet: String,
    /// A1 address.
    pub cell: String,
    /// Formula text without `=` (for shared-formula members, the group master's text); empty
    /// for array/spill members, which have no formula text of their own.
    pub formula: String,
    /// Functions the formula calls ([`functions_in`]), for per-function attribution.
    pub functions: Vec<String>,
    /// The value Excel stored when it last saved the workbook.
    pub cached: Value,
    /// `None` when the recalculated workbook has no such cell.
    pub recalculated: Option<Value>,
    /// `None` when the cell is missing from the recalculated workbook.
    pub verdict: Option<Verdict>,
}

/// Calibration result for one workbook: its cached values against an oracle recalculation.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Comparison {
    /// Whether `cached` was last saved by Excel; otherwise its cached values are not Excel's
    /// answers and calibration (like suites with `require_excel_authored`) should skip it.
    pub excel_authored: bool,
    /// Formula cells left out because they depend on RAND/NOW/TODAY-like functions: a
    /// recalculation at another time legitimately differs (the oracle clock is never pinned).
    pub volatile_skipped: usize,
    pub cells: Vec<CellComparison>,
}

/// Compares every non-volatile formula cell of `cached` with the same cell in `recalculated`.
/// Cells only present in `recalculated` (e.g. an embedded canary sheet) are ignored.
#[must_use]
pub fn compare_snapshots(cached: &WorkbookSnapshot, recalculated: &WorkbookSnapshot) -> Comparison {
    let (scorable, volatile_skipped) = scorable_cells(&cached, false);
    let others = recalculated.values();
    let cells = scorable
        .into_iter()
        .map(|(sheet, addr, cell)| {
            let formula = cell.formula.clone().unwrap_or_default();
            let other = others.get(&(sheet.to_string(), addr)).cloned();
            CellComparison {
                sheet: sheet.to_string(),
                cell: addr.to_string(),
                functions: functions_in(&formula).into_iter().collect(),
                formula,
                verdict: other.as_ref().map(|o| compare(&cell.value, o)),
                cached: cell.value.clone(),
                recalculated: other,
            }
        })
        .collect();
    Comparison {
        excel_authored: cached.is_excel_authored(),
        volatile_skipped,
        cells,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cellref::Addr;
    use crate::xlsx_snapshot::{CellSnapshot, FormulaKind, SheetSnapshot};
    use rustsheet_engine::types::DateSystem;
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
            date1904: DateSystem::default() == DateSystem::Excel1904,
            sheets: vec![SheetSnapshot {
                name: "S".into(),
                cells: map,
            }],
            defined_names: Vec::new(),
            tables: Vec::new(),
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
        let mut cached = cached;
        let mut recalculated = recalculated;
        // A spilled value: a formula result without formula text of its own.
        for (wb, v) in [(&mut cached, 0.25), (&mut recalculated, 0.25)] {
            wb.sheets[0].cells.insert(
                Addr::parse("C1").expect("addr"),
                CellSnapshot {
                    formula: None,
                    formula_kind: Some(FormulaKind::ArrayMember {
                        anchor: "B1".into(),
                    }),
                    value: Value::Number(v),
                },
            );
        }
        let got = compare_snapshots(&cached, &recalculated);
        assert!(!got.excel_authored); // no <Application> in docProps/app.xml
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
                ("C1", Some(Verdict::Pass)), // row order: C1 before B2
                ("B2", Some(Verdict::NumericMismatch)),
                ("B5", None), // missing from the recalculation
            ]
        );
        let member = got.cells.iter().find(|c| c.cell == "C1").expect("C1");
        assert_eq!((member.formula.as_str(), member.functions.len()), ("", 0));
        assert_eq!(got.cells[2].functions, vec!["ROUND".to_string()]);
    }
}
