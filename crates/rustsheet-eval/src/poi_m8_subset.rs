//! M8's POI corpus subset (decision 0146 item 2(b)): the scored `poi` cells whose formula (or array
//! anchor's formula) holds a defined name, a structured reference, a 3-D reference or an
//! external-workbook reference, as the engine's parser classifies it. The M8 exit row (T-0096)
//! reads this section of report.json instead of rescoring.
//!
//! Deviations leave the denominator (decision 0036); pending cases have no scored cells; a
//! crashed or errored case holding a subset formula fails the measure until it runs, listed as
//! `case!*`. poi workbooks are third-party files: a formula that does not parse is outside the
//! subset and counted as `unparsed` (CLAUDE.md directive 5).

use std::collections::BTreeSet;
use std::fmt::Write as _;

use rustsheet_engine::types::{Table, TableColumn, TableStyleInfo};
use rustsheet_engine::ReferenceFormsReader;
use serde::{Deserialize, Serialize};

use crate::cellref::Addr;
use crate::closeness::Visible;
use crate::runner::{CaseResult, CaseStatus};
use crate::xlsx_snapshot::WorkbookSnapshot;

/// The suite classified.
pub const SUITE: &str = "poi";

/// The formula cells of one case in the subset, and those whose formula does not parse, as
/// `Sheet!A1`. Only the run's own report reads them.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ReferenceFormCells {
    /// Cells whose formula holds one of the four forms.
    pub subset: BTreeSet<String>,
    /// Cells whose formula does not parse (outside the subset).
    pub unparsed: BTreeSet<String>,
}

impl ReferenceFormCells {
    /// Classifies `cells` (sheet, address, formula or array anchor's formula without `=`) of the
    /// workbook `snapshot`, parsed against its sheets, defined names and tables.
    #[must_use]
    pub fn classify<'a>(
        snapshot: &WorkbookSnapshot,
        cells: impl IntoIterator<Item = (&'a str, Addr, Option<&'a str>)>,
    ) -> Self {
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
        let mut reader = ReferenceFormsReader::new(sheets, &names, tables);
        let mut out = Self::default();
        for (sheet, addr, formula) in cells {
            let Some(formula) = formula else {
                continue;
            };
            let key = format!("{sheet}!{addr}");
            let row = i32::try_from(addr.row).unwrap_or(1);
            let column = i32::try_from(addr.col).unwrap_or(1);
            match reader.read(formula, sheet, row, column) {
                Some(forms) if forms.any() => {
                    out.subset.insert(key);
                }
                Some(_) => {}
                None => {
                    out.unparsed.insert(key);
                }
            }
        }
        out
    }
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
    use crate::runner::{CellKind, CellResult};
    use crate::value::{Value, Verdict};
    use crate::xlsx_snapshot::{DefinedName, SheetSnapshot};

    fn snapshot() -> WorkbookSnapshot {
        WorkbookSnapshot {
            application: None,
            app_version: None,
            file_app_name: None,
            date1904: false,
            sheets: ["Jan", "Feb"]
                .map(|name| SheetSnapshot {
                    name: name.to_string(),
                    cells: BTreeMap::new(),
                })
                .to_vec(),
            defined_names: vec![DefinedName {
                name: "Rate".to_string(),
                sheet: None,
                body: "Jan!$A$1".to_string(),
            }],
            tables: Vec::new(),
        }
    }

    fn at(row: u32) -> Addr {
        Addr { row, col: 1 }
    }

    #[test]
    fn classifies_cells_by_their_formula() {
        let forms = ReferenceFormCells::classify(
            &snapshot(),
            [
                ("Jan", at(1), Some("Rate*2")),
                ("Jan", at(2), Some("SUM(Jan:Feb!B1)")),
                ("Jan", at(3), Some("SUM(B1:B2)")),
                ("Jan", at(4), Some("SUM(1,")),
                ("Jan", at(5), None),
            ],
        );
        assert_eq!(
            forms.subset,
            ["Jan!A1", "Jan!A2"].map(str::to_string).into()
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
