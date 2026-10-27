//! Milestone exit status (T-2177): the four exit measures of decision 0085, for M5 and (by its
//! "same as M5" inheritance) M6, next to their thresholds, and M2's (decision 0135, from
//! docs/milestones.md and decision 0058): no coverage criterion, and its generated suite needs
//! 10,000 scored cells with none visibly off. Decision 0114 reads them at visible
//! parity: the generated suite, ironcalc-calc and probes count visibly-off cells, and the bit-level
//! counts are shown beside them for Phase 1b. Every input is a row the report already computes, so
//! nothing here scores anything; a missing input is `None`, never met. It is an exit measure, not
//! a gate.

use std::fmt::Write as _;

use serde::{Deserialize, Serialize};

use crate::closeness::Visible;
use crate::probe_milestones::{parse_milestone, row_label};
use crate::report::RunReport;

/// The milestones this section covers. M7 to M11 need their own decision first.
pub const MILESTONES: [&str; 3] = ["M2", "M5", "M6"];
/// Fewest scored cells the generated suite must have (decision 0085 item 2).
pub const MIN_SCORED: usize = 20_000;
/// M2's fewest scored cells of `gen-m2` (docs/milestones.md "M2", decision 0058).
pub const M2_MIN_SCORED: usize = 10_000;
/// The generated suite's pass rate must reach 999/1000 of its scored cells; decision 0114's
/// per-function Phase 1 rate (`v15_checkpoint`) shares it.
pub(crate) const RATE_NUM: usize = 999;
/// The denominator of [`RATE_NUM`].
pub(crate) const RATE_DEN: usize = 1000;
/// 99% over [`RATE_DEN`]: the next parity tier below [`RATE_NUM`] (`attribution::Tier`, T-0068).
pub(crate) const RATE99_NUM: usize = 990;

/// Whether `within` of `scored` cells is a share of at least `num` / [`RATE_DEN`]; never with no
/// scored cell. The one definition of "at the rate" for the generated suite here, the Phase 1
/// rates of `v15_checkpoint` and the parity tiers of `attribution::Tier`.
#[must_use]
pub(crate) fn meets_rate(within: usize, scored: usize, num: usize) -> bool {
    scored > 0 && within.saturating_mul(RATE_DEN) >= scored.saturating_mul(num)
}

/// A milestone's generated-suite thresholds and whether coverage is one of its criteria.
struct Criteria {
    /// Fewest scored cells.
    min_scored: usize,
    /// Share of the scored cells, over [`RATE_DEN`], that must not be visibly off.
    rate_num: usize,
    /// Catalog coverage is an exit criterion (M5, M6; not M2, decision 0135).
    coverage: bool,
}

fn criteria(milestone: &str) -> Criteria {
    if milestone == "M2" {
        // "100%": no gen-m2 cell visibly off.
        Criteria {
            min_scored: M2_MIN_SCORED,
            rate_num: RATE_DEN,
            coverage: false,
        }
    } else {
        Criteria {
            min_scored: MIN_SCORED,
            rate_num: RATE_NUM,
            coverage: true,
        }
    }
}

/// One milestone's four measures; `None` means the input was absent from the run.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ExitRow {
    /// `M<n>`.
    pub milestone: String,
    /// In-scope functions of the milestone and how many are covered (decision 0085 item 4).
    pub functions: Option<usize>,
    /// Of those, covered.
    pub covered: Option<usize>,
    /// Coverage is 100%; `None` too for a milestone without a coverage criterion (M2), whose
    /// counts are shown but not required.
    pub coverage_met: Option<bool>,
    /// Scored cells of `gen-m<k>`: `cells - deviations`.
    pub gen_scored: Option<usize>,
    /// Bit-exact share of the scored cells (Phase 1b); `None` when nothing was scored.
    pub gen_rate: Option<f64>,
    /// Scored cells that fail bit for bit (Phase 1b).
    pub gen_known_failures: Option<usize>,
    /// Scored cells that are visibly off (decision 0114).
    #[serde(default)]
    pub gen_visibly_off: Option<usize>,
    /// Share of the scored cells that are not visibly off; `None` when nothing was scored.
    #[serde(default)]
    pub gen_rate15: Option<f64>,
    /// Visibly-off cells the rate allows: `floor(0.001 * scored)`, or 0 for M2.
    pub gen_allowance: Option<usize>,
    /// At least [`MIN_SCORED`] ([`M2_MIN_SCORED`] for M2) scored cells with at most the allowance
    /// visibly off.
    pub gen_met: Option<bool>,
    /// The visibly-off cells of the generated suite as `case!Sheet!A1`, for
    /// `scripts/exit_owners.py` (decision 0135).
    #[serde(default)]
    pub gen_visibly_off_cells: Vec<String>,
    /// `ironcalc-calc` failing cells less the named exit exceptions (Phase 1b).
    pub calc_failing: Option<usize>,
    /// Of those, the visibly-off cells (decision 0114).
    #[serde(default)]
    pub calc_visibly_off: Option<usize>,
    /// No visibly-off cell is left.
    pub calc_met: Option<bool>,
    /// Known failures of the milestone's probe row (Phase 1b).
    pub probe_known_failures: Option<usize>,
    /// Visibly-off cells of the milestone's probe row (decision 0114).
    #[serde(default)]
    pub probe_visibly_off: Option<usize>,
    /// No probe cell is visibly off.
    pub probes_met: Option<bool>,
    /// The "Probe milestones" row the probe measure reads: `M0-M2` for M2 (decision 0051).
    #[serde(default)]
    pub probe_row: String,
    /// All four measures are met.
    pub exit: bool,
}

/// One row per milestone of [`MILESTONES`].
#[must_use]
pub fn summarize(report: &RunReport) -> Vec<ExitRow> {
    MILESTONES
        .iter()
        .map(|&m| {
            let need = criteria(m);
            let mut row = ExitRow {
                milestone: m.to_string(),
                probe_row: parse_milestone(m).map_or_else(|| m.to_string(), row_label),
                ..ExitRow::default()
            };
            if let Some(c) = report.catalog_coverage.iter().find(|c| c.milestone == m) {
                row.functions = Some(c.functions);
                row.covered = Some(c.covered);
                row.coverage_met = need.coverage.then_some(c.covered == c.functions);
            }
            let suite = format!("gen-{}", m.to_ascii_lowercase());
            if let Some(s) = report.suites.iter().find(|s| s.name == suite) {
                let scored = s.cells.saturating_sub(s.deviations);
                let passed = s.cells_passed.min(scored);
                let visible = (s.closeness.visible_numeric + s.closeness.visible_other).min(scored);
                row.gen_scored = Some(scored);
                row.gen_rate = (scored > 0).then(|| s.cell_pass_rate());
                row.gen_known_failures = Some(scored - passed);
                row.gen_visibly_off = Some(visible);
                row.gen_rate15 = (scored > 0).then(|| s.rate(scored - visible));
                row.gen_allowance = Some(scored * (RATE_DEN - need.rate_num) / RATE_DEN);
                row.gen_met = Some(
                    scored >= need.min_scored
                        && meets_rate(scored - visible, scored, need.rate_num),
                );
                row.gen_visibly_off_cells = report
                    .cases
                    .iter()
                    .filter(|r| r.suite == suite)
                    .flat_map(|r| {
                        r.cells
                            .iter()
                            .filter(|c| Visible::is_off(c))
                            .map(move |c| r.cell_id(c))
                    })
                    .collect();
            }
            let scope = report.function_scope.as_ref();
            if let Some(r) = scope.and_then(|s| s.rows.iter().find(|r| r.milestone == m)) {
                let failing = r.failing.len().saturating_sub(r.excepted.len());
                let visible = r
                    .visibly_off
                    .iter()
                    .filter(|c| !r.excepted.contains(c))
                    .count();
                row.calc_failing = Some(failing);
                row.calc_visibly_off = Some(visible);
                row.calc_met = Some(visible == 0);
            }
            let probes = &report.probe_milestones;
            if let Some(p) = probes.iter().find(|p| p.milestone == row.probe_row) {
                row.probe_known_failures = Some(p.known_failures);
                row.probe_visibly_off = Some(p.visibly_off);
                row.probes_met = Some(p.visibly_off == 0);
            }
            let coverage = if need.coverage {
                row.coverage_met
            } else {
                Some(true)
            };
            row.exit = [coverage, row.gen_met, row.calc_met, row.probes_met]
                .iter()
                .all(|v| *v == Some(true));
            row
        })
        .collect()
}

fn verdict(met: Option<bool>) -> &'static str {
    match met {
        Some(true) => "met",
        Some(false) => "not met",
        None => "n/a",
    }
}

fn count(n: Option<usize>) -> String {
    n.map_or_else(|| "n/a".to_string(), |n| n.to_string())
}

/// The report section.
pub fn write_markdown(rows: &[ExitRow], md: &mut String) {
    if rows.is_empty() {
        return;
    }
    let _ = writeln!(md, "\n## Milestone exit (decisions 0085, 0114)\n");
    let _ = writeln!(
        md,
        "Thresholds, at visible parity (decision 0114): catalog coverage `covered` equals `functions`; gen suite at least {MIN_SCORED} scored cells (`cells - deviations`) with at least 0.999 of them not visibly off, so visibly-off cells at most the allowance `floor(0.001 * scored)`; ironcalc-calc has no visibly-off cell once exit exceptions are excluded; probes have no visibly-off cell. M2 (decision 0135): coverage is shown but not required, and gen-m2 needs {M2_MIN_SCORED} scored cells with none visibly off; its probes are the M0-M2 row. Bracketed counts are the bit-level failures, Phase 1b's target. An exit measure, not a gate.\n"
    );
    let _ = writeln!(
        md,
        "| milestone | coverage (covered/functions) | gen suite (scored, 15-digit rate, visibly off/allowance [bit failures]) | ironcalc-calc (visibly off [bit failures], excl. exceptions) | probes (visibly off [bit failures]) | exit |"
    );
    let _ = writeln!(md, "|---|---|---|---|---|---|");
    for r in rows {
        let coverage = format!(
            "{}/{} {}",
            count(r.covered),
            count(r.functions),
            verdict(r.coverage_met)
        );
        let generated = match r.gen_scored {
            None => "n/a".to_string(),
            Some(scored) => format!(
                "{scored}, {}, {}/{} [{}] {}",
                r.gen_rate15
                    .map_or_else(|| "n/a".to_string(), |x| format!("{x:.5}")),
                count(r.gen_visibly_off),
                count(r.gen_allowance),
                count(r.gen_known_failures),
                verdict(r.gen_met)
            ),
        };
        let _ = writeln!(
            md,
            "| {} | {coverage} | {generated} | {} [{}] {} | {} [{}] {} | {} |",
            r.milestone,
            count(r.calc_visibly_off),
            count(r.calc_failing),
            verdict(r.calc_met),
            count(r.probe_visibly_off),
            count(r.probe_known_failures),
            verdict(r.probes_met),
            if r.exit { "met" } else { "not met" }
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::Catalog;
    use crate::catalog_coverage::MilestoneCoverage;
    use crate::closeness::Closeness;
    use crate::function_scope::{FunctionScope, MilestoneScope};
    use crate::probe_milestones::MilestoneRow;
    use crate::report::SuiteSummary;
    use crate::runner::CellResult;

    fn report(gen_cells: Option<(usize, usize)>) -> RunReport {
        report15(gen_cells.map(|(cells, passed)| (cells, passed, cells - passed)))
    }

    /// `(cells, bit-exact, visibly off)` of the generated suite.
    fn report15(gen_cells: Option<(usize, usize, usize)>) -> RunReport {
        let mut r = RunReport::build("t", &[], &[], &Catalog::from_functions([]));
        r.catalog_coverage = vec![MilestoneCoverage {
            milestone: "M5".into(),
            functions: 10,
            covered: 10,
            ..MilestoneCoverage::default()
        }];
        r.suites = gen_cells
            .map(|(cells, passed, visible)| SuiteSummary {
                name: "gen-m5".into(),
                cells,
                cells_passed: passed,
                closeness: Closeness {
                    visible_numeric: visible,
                    ..Closeness::default()
                },
                ..SuiteSummary::default()
            })
            .into_iter()
            .collect();
        r.function_scope = Some(FunctionScope {
            rows: vec![MilestoneScope {
                milestone: "M5".into(),
                failing: vec!["a!S!A1".into()],
                excepted: vec!["a!S!A1".into()],
                visibly_off: vec!["a!S!A1".into()],
                ..MilestoneScope::default()
            }],
            ..FunctionScope::default()
        });
        r.probe_milestones = vec![MilestoneRow {
            milestone: "M5".into(),
            ..MilestoneRow::default()
        }];
        r
    }

    fn row(r: &RunReport, m: &str) -> ExitRow {
        summarize(r)
            .into_iter()
            .find(|row| row.milestone == m)
            .unwrap_or_default()
    }

    fn m5(r: &RunReport) -> ExitRow {
        row(r, "M5")
    }

    #[test]
    fn the_rate_is_a_share_of_at_least_num_over_den_and_never_of_no_cells() {
        assert!(meets_rate(999, 1000, RATE_NUM) && !meets_rate(998, 1000, RATE_NUM));
        assert!(meets_rate(990, 1000, RATE99_NUM) && !meets_rate(989, 1000, RATE99_NUM));
        assert!(!meets_rate(0, 0, RATE_NUM));
    }

    #[test]
    fn all_met() {
        let r = report(Some((22_400, 22_380)));
        let rows = summarize(&r);
        let row = &rows[1];
        assert_eq!(row.gen_allowance, Some(22));
        assert_eq!(row.gen_known_failures, Some(20));
        assert_eq!(row.calc_failing, Some(0));
        assert!(row.exit, "{row:?}");
        let mut md = String::new();
        write_markdown(&rows, &mut md);
        assert!(
            md.contains(
                "| M5 | 10/10 met | 22400, 0.99911, 20/22 [20] met | 0 [0] met | 0 [0] met | met |"
            ),
            "{md}"
        );
        assert!(
            md.contains("| M6 | n/a/n/a n/a | n/a | n/a [n/a] n/a | n/a [n/a] n/a | not met |"),
            "{md}"
        );
    }

    #[test]
    fn coverage_short_fails_alone() {
        let mut r = report(Some((22_400, 22_400)));
        r.catalog_coverage[0].covered = 9;
        let row = m5(&r);
        assert_eq!(row.coverage_met, Some(false));
        assert!(!row.exit);
    }

    #[test]
    fn too_few_cells_fails_alone() {
        let row = m5(&report(Some((19_999, 19_999))));
        assert_eq!(row.gen_met, Some(false));
        assert_eq!(row.coverage_met, Some(true));
        assert!(!row.exit);
    }

    #[test]
    fn low_rate_fails_alone() {
        // 0.998 of 22,400 scored cells: 45 failures against an allowance of 22.
        let row = m5(&report(Some((22_400, 22_355))));
        assert_eq!(row.gen_met, Some(false));
        assert!(!row.exit);
    }

    #[test]
    fn calc_failure_fails_alone() {
        let mut r = report(Some((22_400, 22_400)));
        r.function_scope.as_mut().unwrap().rows[0].excepted.clear();
        let row = m5(&r);
        assert_eq!(row.calc_met, Some(false));
        assert!(!row.exit);
    }

    #[test]
    fn probe_failure_fails_alone() {
        let mut r = report(Some((22_400, 22_400)));
        r.probe_milestones[0].known_failures = 1;
        assert!(
            m5(&r).exit,
            "a bit-level probe failure is Phase 1b's (decision 0114)"
        );
        r.probe_milestones[0].visibly_off = 1;
        let row = m5(&r);
        assert_eq!(row.probes_met, Some(false));
        assert!(!row.exit);
    }

    #[test]
    fn bit_failures_within_fifteen_digits_do_not_block_the_exit() {
        // Decision 0114: 400 bit failures, 20 of them visibly off, is met; 23 visibly off is not.
        let row = m5(&report15(Some((22_400, 22_000, 20))));
        assert_eq!(row.gen_known_failures, Some(400));
        assert_eq!(row.gen_visibly_off, Some(20));
        assert!(row.exit, "{row:?}");
        assert!(!m5(&report15(Some((22_400, 22_000, 23)))).exit);
        let mut r = report(Some((22_400, 22_400)));
        let scope = r.function_scope.as_mut().unwrap();
        scope.rows[0].failing.push("b!S!A1".into());
        assert!(
            m5(&r).exit,
            "a bit-only ironcalc-calc failure is Phase 1b's"
        );
        r.function_scope.as_mut().unwrap().rows[0]
            .visibly_off
            .push("b!S!A1".into());
        assert_eq!(m5(&r).calc_met, Some(false));
    }

    #[test]
    fn missing_gen_suite_is_not_applicable() {
        let row = m5(&report(None));
        assert_eq!(row.gen_met, None);
        assert!(!row.exit);
    }

    /// A run whose M2 inputs meet M2's exit: `gen_cells` (cells, visibly off) of gen-m2.
    fn m2_report(gen_cells: (usize, usize)) -> RunReport {
        let mut r = report(None);
        let (cells, visible) = gen_cells;
        r.catalog_coverage.push(MilestoneCoverage {
            milestone: "M2".into(),
            functions: 37,
            covered: 30,
            ..MilestoneCoverage::default()
        });
        r.suites.push(SuiteSummary {
            name: "gen-m2".into(),
            cells,
            cells_passed: cells - visible,
            closeness: Closeness {
                visible_other: visible,
                ..Closeness::default()
            },
            ..SuiteSummary::default()
        });
        r.function_scope
            .as_mut()
            .unwrap()
            .rows
            .push(MilestoneScope {
                milestone: "M2".into(),
                failing: vec!["ironcalc-calc/IF_ARRAY.xlsx!Sheet1!C11".into()],
                excepted: vec!["ironcalc-calc/IF_ARRAY.xlsx!Sheet1!C11".into()],
                visibly_off: vec!["ironcalc-calc/IF_ARRAY.xlsx!Sheet1!C11".into()],
                ..MilestoneScope::default()
            });
        r.probe_milestones.push(MilestoneRow {
            milestone: "M0-M2".into(),
            known_failures: 3,
            ..MilestoneRow::default()
        });
        r
    }

    #[test]
    fn m2_needs_ten_thousand_gen_cells_none_visibly_off_and_no_coverage() {
        let r = m2_report((M2_MIN_SCORED, 0));
        let m2 = row(&r, "M2");
        assert_eq!(
            (m2.covered, m2.functions, m2.coverage_met),
            (Some(30), Some(37), None),
            "coverage is shown, not required"
        );
        assert_eq!((m2.gen_allowance, m2.gen_met), (Some(0), Some(true)));
        assert_eq!((m2.calc_visibly_off, m2.calc_met), (Some(0), Some(true)));
        assert_eq!(m2.probe_row, "M0-M2");
        assert_eq!(
            (m2.probe_known_failures, m2.probes_met),
            (Some(3), Some(true))
        );
        assert!(m2.exit, "{m2:?}");
        // The M5 row keeps its own thresholds and reads its own probe row.
        assert_eq!(m5(&r).probe_row, "M5");
        assert_eq!(m5(&r).coverage_met, Some(true));
        let mut md = String::new();
        write_markdown(&summarize(&r), &mut md);
        assert!(
            md.contains(
                "| M2 | 30/37 n/a | 10000, 1.00000, 0/0 [0] met | 0 [0] met | 0 [3] met | met |"
            ),
            "{md}"
        );
        // 9,999 scored cells, or one visibly-off cell, is not met.
        let m2 = row(&m2_report((M2_MIN_SCORED - 1, 0)), "M2");
        assert_eq!(m2.gen_met, Some(false));
        assert!(!m2.exit);
        let m2 = row(&m2_report((50_000, 1)), "M2");
        assert_eq!((m2.gen_allowance, m2.gen_met), (Some(0), Some(false)));
        assert!(!m2.exit);
        // A visibly-off probe of the M0-M2 row is not met.
        let mut r = m2_report((M2_MIN_SCORED, 0));
        r.probe_milestones[1].visibly_off = 1;
        assert_eq!(row(&r, "M2").probes_met, Some(false));
    }

    #[test]
    fn gen_visibly_off_cells_are_listed_for_exit_owners() {
        use crate::report::tests::{case, cell};
        use crate::value::{Value, Verdict};
        let mut r = m2_report((M2_MIN_SCORED, 1));
        let mut g = case("gen-m2/abc.case.json", 1);
        let close = CellResult {
            cell: "A2".into(),
            expected: Value::Number(1.0),
            actual: Value::Number(1.0 + 1e-17),
            ..cell("ABS(1)", Verdict::NumericClose)
        };
        g.cells = vec![cell("ABS(1)", Verdict::ErrorMismatch), close];
        r.cases = vec![g];
        let m2 = row(&r, "M2");
        assert_eq!(m2.gen_visibly_off_cells, ["gen-m2/abc.case.json!S!A1"]);
    }
}
