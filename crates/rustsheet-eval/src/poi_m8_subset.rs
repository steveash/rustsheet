//! M8's POI corpus subset (decision 0146 item 2(b)): the scored `poi` cells whose formula (or array
//! anchor's formula) holds a defined name, a structured reference, a 3-D reference or an
//! external-workbook reference, as the engine's parser classifies it. The M8 exit row (T-0096)
//! reads this section of report.json instead of rescoring.
//!
//! Deviations leave the denominator (decision 0036); pending cases have no scored cells; a
//! crashed or errored case holding a subset formula fails the measure until it runs, listed as
//! `case!*`. poi workbooks are third-party files: a formula that does not parse is outside the
//! subset and counted as `unparsed` (CLAUDE.md directive 5).

use std::collections::{BTreeSet, HashMap};
use std::fmt::Write as _;

use rustsheet_engine::types::{Table, TableColumn, TableStyleInfo};
use rustsheet_engine::{ReferenceForms, ReferenceFormsReader};
use serde::{Deserialize, Serialize};

use crate::closeness::Visible;
use crate::runner::{CaseResult, CaseStatus};
use crate::xlsx_snapshot::{FormulaKind, WorkbookSnapshot};

/// The suite classified.
pub const SUITE: &str = "poi";

/// The formula cells (and array members) of one case in the subset, and those whose formula does
/// not parse, as `Sheet!A1`. Only the run's own report reads them.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ReferenceFormCells {
    /// Cells whose formula holds one of the four forms.
    pub subset: BTreeSet<String>,
    /// Cells whose formula does not parse (outside the subset).
    pub unparsed: BTreeSet<String>,
}

impl ReferenceFormCells {
    /// Classifies every formula cell of the workbook `snapshot` (an array member by its anchor's
    /// formula), parsed against its sheets, defined names and tables. Each distinct formula text of
    /// a sheet is parsed once: a shared formula repeats it down a column.
    #[must_use]
    pub fn classify(snapshot: &WorkbookSnapshot) -> Self {
        let mut reader = reader(snapshot);
        let mut out = Self::default();
        for sheet in &snapshot.sheets {
            let mut seen: HashMap<&str, Option<bool>> = HashMap::new();
            for (addr, cell) in &sheet.cells {
                let Some(formula) = cell.formula.as_deref() else {
                    continue;
                };
                let in_subset = *seen.entry(formula).or_insert_with(|| {
                    let row = i32::try_from(addr.row).unwrap_or(1);
                    let column = i32::try_from(addr.col).unwrap_or(1);
                    reader
                        .read(formula, &sheet.name, row, column)
                        .map(ReferenceForms::any)
                });
                out.insert(format!("{}!{addr}", sheet.name), in_subset);
            }
            for (addr, cell) in &sheet.cells {
                let Some(FormulaKind::ArrayMember { anchor }) = &cell.formula_kind else {
                    continue;
                };
                if cell.formula.is_some() {
                    continue;
                }
                let anchor = format!("{}!{anchor}", sheet.name);
                let in_subset = if out.subset.contains(&anchor) {
                    Some(true)
                } else if out.unparsed.contains(&anchor) {
                    None
                } else {
                    Some(false)
                };
                out.insert(format!("{}!{addr}", sheet.name), in_subset);
            }
        }
        out
    }

    /// Records `key` as in the subset (`Some(true)`), outside it (`Some(false)`) or unparsed (`None`).
    fn insert(&mut self, key: String, in_subset: Option<bool>) {
        match in_subset {
            Some(true) => {
                self.subset.insert(key);
            }
            Some(false) => {}
            None => {
                self.unparsed.insert(key);
            }
        }
    }
}

/// A reader for the formulas of `snapshot`, which knows its sheets, defined names and tables.
fn reader(snapshot: &WorkbookSnapshot) -> ReferenceFormsReader {
    let sheets: Vec<String> = snapshot.sheets.iter().map(|s| s.name.clone()).collect();
    let names: Vec<(String, Option<String>, String)> = snapshot
        .defined_names
        .iter()
        .map(|n| (n.name.clone(), n.sheet.clone(), n.body.clone()))
        .collect();
    let tables = snapshot
        .tables
        .iter()
        .map(|t| Table {
            name: t.name.clone(),
            display_name: t.name.clone(),
            sheet_name: t.sheet.clone(),
            reference: t.range.clone(),
            totals_row_count: t.totals_rows,
            header_row_count: t.header_rows,
            header_row_dxf_id: None,
            data_dxf_id: None,
            totals_row_dxf_id: None,
            columns: t
                .columns
                .iter()
                .zip(1..)
                .map(|(name, id)| TableColumn {
                    id,
                    name: name.clone(),
                    ..TableColumn::default()
                })
                .collect(),
            style_info: TableStyleInfo::default(),
            has_filters: false,
        })
        .collect();
    ReferenceFormsReader::new(sheets, &names, tables)
}

/// How the subset scored.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PoiM8Subset {
    /// Scored subset cells, deviations left out.
    pub scored: usize,
    /// Of those, the cells that are visibly off (decision 0083).
    pub visibly_off: usize,
    /// The visibly-off cells as `case!Sheet!A1`, sorted.
    pub visibly_off_cells: Vec<String>,
    /// Subset cells that gave their pinned by-design value (decision 0036).
    pub deviations: usize,
    /// Scored cells whose formula does not parse (not in the subset).
    pub unparsed: usize,
    /// Crashed or errored cases holding a subset formula, as `case!*`, sorted.
    pub blocked_cases: Vec<String>,
}

/// Whether a case did not run to scoring.
fn blocked(status: &CaseStatus) -> bool {
    matches!(
        status,
        CaseStatus::EngineError | CaseStatus::Panic | CaseStatus::Timeout
    )
}

/// Scores the subset over the [`SUITE`] cases of `results`; `None` when the run executed none.
#[must_use]
pub fn summarize(results: &[CaseResult]) -> Option<PoiM8Subset> {
    let cases: Vec<&CaseResult> = results.iter().filter(|r| r.suite == SUITE).collect();
    if cases.is_empty() {
        return None;
    }
    let mut out = PoiM8Subset::default();
    for r in cases {
        if blocked(&r.status) {
            if !r.reference_forms.subset.is_empty() {
                out.blocked_cases.push(format!("{}!*", r.id));
            }
            continue;
        }
        for c in &r.cells {
            let key = c.key();
            if r.reference_forms.unparsed.contains(&key) {
                out.unparsed += 1;
            }
            if !r.reference_forms.subset.contains(&key) {
                continue;
            }
            if !c.verdict.is_pass() && !c.verdict.is_failure() {
                out.deviations += 1;
                continue;
            }
            out.scored += 1;
            if c.verdict.is_failure() && Visible::is_off(c) {
                out.visibly_off_cells.push(format!("{}!{key}", r.id));
            }
        }
    }
    out.visibly_off = out.visibly_off_cells.len();
    out.visibly_off_cells.sort();
    out.blocked_cases.sort();
    Some(out)
}

/// The report.md line for `subset`; nothing when poi did not run.
pub fn write_markdown(subset: Option<&PoiM8Subset>, md: &mut String) {
    let Some(s) = subset else {
        return;
    };
    let _ = writeln!(
        md,
        "\n## POI M8 subset\n\n{} scored `{SUITE}` cells hold a defined name, structured, 3-D or \
         external reference (decision 0146 item 2(b)): {} visibly off, {} deviations left out, \
         {} unparsed formulas outside the subset, {} crashed or errored cases holding one.",
        s.scored,
        s.visibly_off,
        s.deviations,
        s.unparsed,
        s.blocked_cases.len()
    );
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use crate::cellref::Addr;
    use crate::runner::{CellKind, CellResult};
    use crate::value::{Value, Verdict};
    use crate::xlsx_snapshot::{CellSnapshot, DefinedName, SheetSnapshot};

    fn formula(text: &str, kind: FormulaKind) -> CellSnapshot {
        CellSnapshot {
            formula: Some(text.to_string()),
            formula_kind: Some(kind),
            value: Value::Number(0.0),
        }
    }

    fn snapshot() -> WorkbookSnapshot {
        let member = |anchor: &str| CellSnapshot {
            formula: None,
            formula_kind: Some(FormulaKind::ArrayMember {
                anchor: anchor.to_string(),
            }),
            value: Value::Number(0.0),
        };
        let cells: BTreeMap<Addr, CellSnapshot> = [
            ("A1", formula("Rate*2", FormulaKind::Normal)),
            ("A2", formula("SUM(Jan:Feb!B1)", FormulaKind::Normal)),
            ("A3", formula("SUM(B1:B2)", FormulaKind::Normal)),
            ("A4", formula("SUM(1,", FormulaKind::Normal)),
            (
                "A5",
                formula(
                    "Rate*{1;2}",
                    FormulaKind::Array {
                        range: "A5:A6".to_string(),
                    },
                ),
            ),
            ("A6", member("A5")),
            ("A7", member("A3")),
            (
                "A8",
                CellSnapshot {
                    formula: None,
                    formula_kind: None,
                    value: Value::Number(1.0),
                },
            ),
        ]
        .into_iter()
        .filter_map(|(a, c)| Some((Addr::parse(a)?, c)))
        .collect();
        WorkbookSnapshot {
            application: None,
            app_version: None,
            file_app_name: None,
            date1904: false,
            sheets: vec![
                SheetSnapshot {
                    name: "Jan".to_string(),
                    cells,
                },
                SheetSnapshot {
                    name: "Feb".to_string(),
                    cells: BTreeMap::new(),
                },
            ],
            defined_names: vec![DefinedName {
                name: "Rate".to_string(),
                sheet: None,
                body: "Jan!$A$1".to_string(),
            }],
            tables: Vec::new(),
        }
    }

    #[test]
    fn classifies_formula_cells_and_array_members_by_their_anchor() {
        let forms = ReferenceFormCells::classify(&snapshot());
        assert_eq!(
            forms.subset,
            ["Jan!A1", "Jan!A2", "Jan!A5", "Jan!A6"]
                .map(str::to_string)
                .into()
        );
        assert_eq!(forms.unparsed, ["Jan!A4".to_string()].into());
    }

    fn cell(cell: &str, verdict: Verdict, actual: f64) -> CellResult {
        CellResult {
            sheet: "S".to_string(),
            cell: cell.to_string(),
            formula: Some("Rate".to_string()),
            anchor_formula: None,
            expected: Value::Number(1.0),
            actual: Value::Number(actual),
            verdict,
            from_excel: true,
            listed_deviation: false,
            graph_subnormal: false,
            kind: CellKind::Scored,
        }
    }

    fn case(id: &str, status: CaseStatus, cells: Vec<CellResult>, subset: &[&str]) -> CaseResult {
        CaseResult {
            id: format!("{SUITE}/{id}"),
            suite: SUITE.to_string(),
            status,
            message: None,
            expected_source: String::new(),
            checked: cells.len(),
            passed: 0,
            deviations: 0,
            skipped_volatile: 0,
            skipped_add_in: 0,
            millis: 0,
            tags: Vec::new(),
            cells,
            text: Vec::new(),
            defined_names: BTreeMap::new(),
            arguments: Vec::new(),
            reference_forms: ReferenceFormCells {
                subset: subset.iter().map(|s| (*s).to_string()).collect(),
                unparsed: ["S!A9".to_string()].into(),
            },
        }
    }

    #[test]
    fn scores_subset_cells_and_lists_blocked_cases() {
        let results = [
            case(
                "a",
                CaseStatus::Fail,
                vec![
                    cell("A1", Verdict::Pass, 1.0),
                    cell("A2", Verdict::NumericMismatch, 2.0),
                    cell("A3", Verdict::Deviation, 1.0),
                    cell("A4", Verdict::NumericMismatch, 2.0),
                    cell("A9", Verdict::Pass, 1.0),
                ],
                &["S!A1", "S!A2", "S!A3"],
            ),
            case("b", CaseStatus::Panic, Vec::new(), &["S!A1"]),
            case("c", CaseStatus::Timeout, Vec::new(), &[]),
        ];
        let s = summarize(&results).expect("poi ran");
        assert_eq!(
            (s.scored, s.visibly_off, s.deviations, s.unparsed),
            (2, 1, 1, 1)
        );
        assert_eq!(s.visibly_off_cells, vec!["poi/a!S!A2".to_string()]);
        assert_eq!(s.blocked_cases, vec!["poi/b!*".to_string()]);
        let mut md = String::new();
        write_markdown(Some(&s), &mut md);
        assert!(md.contains("2 scored `poi` cells"), "{md}");
        assert!(md.contains("1 crashed or errored"), "{md}");
    }

    #[test]
    fn nothing_without_the_suite() {
        assert_eq!(summarize(&[]), None);
        let mut md = String::new();
        write_markdown(None, &mut md);
        assert!(md.is_empty());
    }
}
