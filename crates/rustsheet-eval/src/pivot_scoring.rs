//! Scoring a case's pivots against the ones Excel saved (T-4124, M16).
//!
//! An oracle sidecar's `pivots` section (T-4123) holds every cell of each pivot's area and Excel's
//! saved layout. The cells become ordinary scored expectations, so a pivot the engine does not
//! render fails cell by cell, like any wrong value, and enters the baseline ratchet. The layout is
//! compared apart, per pivot: whether the engine has the pivot at all, whether its location is
//! Excel's, and (once the engine emits them, T-4147) whether its `rowItems`/`colItems` are Excel's.
//! report.md lists every pivot whose layout is not Excel's.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use serde::{Deserialize, Serialize};

use crate::case::{cell_key, OraclePivot};
use crate::cellref::Addr;
use crate::engine::Evaluated;
use crate::runner::CaseResult;
use crate::value::Value;
use crate::xlsx_snapshot::{read_snapshot, PivotDef};

/// Every cell of every sidecar pivot's area, as (sheet, cell, Excel's value). A cell key that does
/// not parse is left out (the sidecar reader wrote it, so it never happens in practice).
pub(crate) fn pivot_cells(pivots: &[OraclePivot]) -> Vec<(String, Addr, Value)> {
    pivots
        .iter()
        .flat_map(|p| {
            p.cells.iter().filter_map(|(a1, v)| {
                Some((p.definition.sheet.clone(), Addr::parse(a1)?, v.clone()))
            })
        })
        .collect()
}

/// Adds every sidecar pivot-area cell to `expected` (key from [`cell_key`]) unless the sidecar's
/// sheets already give it, and returns their keys. They are scored as Excel evidence but kept out
/// of the oracle's own cell set, which the spill-extent check reads for formula spills.
pub(crate) fn add_expectations(
    pivots: &[OraclePivot],
    oracle_cells: &BTreeSet<String>,
    expected: &mut BTreeMap<String, Value>,
) -> BTreeSet<String> {
    let mut keys = BTreeSet::new();
    for (sheet, addr, v) in pivot_cells(pivots) {
        let key = cell_key(&sheet, addr);
        if !oracle_cells.contains(&key) && keys.insert(key.clone()) {
            expected.insert(key, v);
        }
    }
    keys
}

/// How a pivot's layout compares with Excel's.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LayoutVerdict {
    /// The engine has no pivot of that name on that sheet.
    Missing,
    /// The engine's location differs from Excel's.
    LocationDiffers,
    /// The location is Excel's, but the engine emits no `rowItems`/`colItems` to compare yet
    /// (T-4147).
    LinesNotEmitted,
    /// The location is Excel's, but the `rowItems` or `colItems` the engine saves are not
    /// (engine-export cases, T-4177).
    LinesDiffer,
}

/// One sidecar pivot whose layout the engine does not match.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PivotLayout {
    pub sheet: String,
    pub name: String,
    /// Excel's `<location ref>`.
    pub excel_location: String,
    /// The engine's location, when it has the pivot.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub engine_location: Option<String>,
    /// How many `rowItems` and `colItems` lines Excel saved.
    pub excel_lines: (usize, usize),
    /// How many the engine saves, when its lines are compared.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub engine_lines: Option<(usize, usize)>,
    pub verdict: LayoutVerdict,
}

/// The pivots of the package `evaluated` saves, with the `rowItems`/`colItems` the engine
/// writes for them (T-4177).
///
/// # Errors
/// When the engine cannot save the workbook or the snapshot cannot read the package.
pub(crate) fn saved_lines(evaluated: &dyn Evaluated) -> Result<Vec<PivotDef>, String> {
    let bytes = evaluated.saved_package()?;
    Ok(read_snapshot(&bytes).map_err(|e| e.to_string())?.pivots)
}

/// The layout of each sidecar pivot against the engine's pivots, matched by sheet and name
/// (ignoring case, as Excel names them). With `saved`, the pivots of the package the engine
/// saves ([`saved_lines`]), their `rowItems`/`colItems` are compared too, and a pivot whose
/// location and lines are Excel's is left out.
pub(crate) fn compare_layouts(
    pivots: &[OraclePivot],
    evaluated: &dyn Evaluated,
    saved: Option<&[PivotDef]>,
) -> Vec<PivotLayout> {
    let places = evaluated.pivot_places();
    let same = |sheet: &str, name: &str, d: &PivotDef| {
        d.sheet.eq_ignore_ascii_case(sheet) && d.name.eq_ignore_ascii_case(name)
    };
    pivots
        .iter()
        .filter_map(|p| {
            let d = &p.definition;
            let place = places.iter().find(|e| {
                e.sheet.eq_ignore_ascii_case(&d.sheet) && e.name.eq_ignore_ascii_case(&d.name)
            });
            let engine_location = place.map(|e| {
                let corner = |row: u32, col: u32| Addr {
                    row: row.saturating_add(1),
                    col: col.saturating_add(1),
                };
                let (first, last) = (corner(e.top, e.left), corner(e.bottom, e.right));
                if first == last {
                    first.to_string()
                } else {
                    format!("{first}:{last}")
                }
            });
            let engine = saved.map(|all| all.iter().find(|e| same(&d.sheet, &d.name, e)));
            let verdict = match (&engine_location, engine) {
                (None, _) => LayoutVerdict::Missing,
                (Some(loc), _) if !loc.eq_ignore_ascii_case(&d.location) => {
                    LayoutVerdict::LocationDiffers
                }
                (Some(_), None) => LayoutVerdict::LinesNotEmitted,
                (Some(_), Some(Some(e)))
                    if e.row_items == d.row_items && e.col_items == d.col_items =>
                {
                    return None
                }
                (Some(_), Some(_)) => LayoutVerdict::LinesDiffer,
            };
            Some(PivotLayout {
                sheet: d.sheet.clone(),
                name: d.name.clone(),
                excel_location: d.location.clone(),
                engine_location,
                excel_lines: (d.row_items.len(), d.col_items.len()),
                engine_lines: engine
                    .flatten()
                    .map(|e| (e.row_items.len(), e.col_items.len())),
                verdict,
            })
        })
        .collect()
}

/// report.md's "Pivot layout" section: every case pivot whose layout is not Excel's, by verdict.
pub(crate) fn write_markdown(cases: &[CaseResult], md: &mut String) {
    let rows: Vec<(&str, &PivotLayout)> = cases
        .iter()
        .flat_map(|c| c.pivot_layout.iter().map(move |p| (c.id.as_str(), p)))
        .collect();
    if rows.is_empty() {
        return;
    }
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for (_, p) in &rows {
        *counts.entry(format!("{:?}", p.verdict)).or_default() += 1;
    }
    let summary: Vec<String> = counts.iter().map(|(k, n)| format!("{k} {n}")).collect();
    let _ = writeln!(
        md,
        "## Pivot layout\n\nPivots whose layout is not Excel's (T-4124): {}.\n",
        summary.join(", ")
    );
    let _ = writeln!(
        md,
        "| case | pivot | Excel location | engine location | Excel lines (rows, cols) | verdict |"
    );
    let _ = writeln!(md, "|---|---|---|---|---|---|");
    for (case, p) in rows {
        let _ = writeln!(
            md,
            "| {case} | {}!{} | {} | {} | {}, {} | {:?} |",
            p.sheet,
            p.name,
            p.excel_location,
            p.engine_location.as_deref().unwrap_or("-"),
            p.excel_lines.0,
            p.excel_lines.1,
            p.verdict
        );
    }
    let _ = writeln!(md);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runner::{CaseResult, CaseStatus};

    fn pivot(sheet: &str, name: &str, location: &str, cells: &[(&str, Value)]) -> OraclePivot {
        let json = format!(
            r#"{{"name":"{name}","sheet":"{sheet}","location":"{location}","first_header_row":1,
            "first_data_row":1,"first_data_col":1,"row_items":[{{"r":0,"i":0,"x":[0]}}],
            "col_items":[],"cells":{{}}}}"#
        );
        let mut p: OraclePivot = serde_json::from_str(&json).expect("pivot");
        p.cells = cells
            .iter()
            .map(|(a, v)| ((*a).to_string(), v.clone()))
            .collect();
        p
    }

    struct Places(Vec<rustsheet::PivotPlace>);
    impl Evaluated for Places {
        fn value(&self, _: &str, _: Addr) -> Option<Value> {
            None
        }
        fn stored_cells(&self, _: &str) -> Vec<Addr> {
            Vec::new()
        }
        fn pivot_places(&self) -> Vec<rustsheet::PivotPlace> {
            self.0.clone()
        }
    }

    fn place(name: &str, bottom: u32) -> rustsheet::PivotPlace {
        rustsheet::PivotPlace {
            sheet: "P".into(),
            name: name.into(),
            top: 2,
            left: 0,
            bottom,
            right: 1,
        }
    }

    #[test]
    fn pivot_cells_become_expectations_unless_the_sheets_give_them() {
        let p = pivot(
            "P",
            "Pv",
            "A3:B4",
            &[
                ("A3", Value::Text("Row Labels".into())),
                ("B4", Value::Empty),
            ],
        );
        let mut expected = BTreeMap::new();
        let given: BTreeSet<String> = [cell_key("P", Addr { row: 3, col: 1 })].into();
        expected.insert(cell_key("P", Addr { row: 3, col: 1 }), Value::Number(1.0));
        let keys = add_expectations(&[p], &given, &mut expected);
        assert_eq!(keys.len(), 1);
        assert_eq!(
            expected.get(&cell_key("P", Addr { row: 4, col: 2 })),
            Some(&Value::Empty)
        );
        // The sheets' own value for A3 is kept.
        assert_eq!(
            expected.get(&cell_key("P", Addr { row: 3, col: 1 })),
            Some(&Value::Number(1.0))
        );
    }

    #[test]
    fn layouts_are_missing_misplaced_or_not_yet_emitted() {
        let pivots = [
            pivot("P", "Gone", "A3:B4", &[]),
            pivot("P", "Moved", "A3:B4", &[]),
            pivot("p", "here", "A3:B4", &[]),
        ];
        let engine = Places(vec![place("Moved", 4), place("Here", 3)]);
        let got: Vec<LayoutVerdict> = compare_layouts(&pivots, &engine, None)
            .iter()
            .map(|l| l.verdict)
            .collect();
        assert_eq!(
            got,
            vec![
                LayoutVerdict::Missing,
                LayoutVerdict::LocationDiffers,
                LayoutVerdict::LinesNotEmitted
            ]
        );
        let moved = &compare_layouts(&pivots, &engine, None)[1];
        assert_eq!(moved.engine_location.as_deref(), Some("A3:B5"));
        assert_eq!(moved.excel_lines, (1, 0));
    }

    #[test]
    fn the_report_lists_each_mismatch_and_nothing_when_there_is_none() {
        let mut md = String::new();
        write_markdown(
            &[CaseResult::empty("probes/x", "probes", CaseStatus::Pass)],
            &mut md,
        );
        assert!(md.is_empty());
        let mut case = CaseResult::empty("probes/x", "probes", CaseStatus::Fail);
        case.pivot_layout = compare_layouts(
            &[pivot("P", "Gone", "A3:B4", &[])],
            &Places(Vec::new()),
            None,
        );
        write_markdown(&[case], &mut md);
        assert!(md.contains("## Pivot layout"));
        assert!(md.contains("| probes/x | P!Gone | A3:B4 | - | 1, 0 | Missing |"));
    }
}
