//! Milestone exit status (T-2177): the four exit measures of decision 0085, for M5 and (by its
//! "same as M5" inheritance) M6, next to their thresholds. Every input is a row the report already
//! computes, so nothing here scores anything; a missing input is `None`, never met. It is an exit
//! measure, not a gate.

use std::fmt::Write as _;

use serde::{Deserialize, Serialize};

use crate::report::RunReport;

/// The milestones this section covers. M7 to M11 need their own decision first.
pub const MILESTONES: [&str; 2] = ["M5", "M6"];
/// Fewest scored cells the generated suite must have (decision 0085 item 2).
pub const MIN_SCORED: usize = 20_000;
/// The generated suite's pass rate must reach 999/1000 of its scored cells.
const RATE_NUM: usize = 999;
const RATE_DEN: usize = 1000;

/// One milestone's four measures; `None` means the input was absent from the run.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ExitRow {
    /// `M<n>`.
    pub milestone: String,
    /// In-scope functions of the milestone and how many are covered (decision 0085 item 4).
    pub functions: Option<usize>,
    /// Of those, covered.
    pub covered: Option<usize>,
    /// Coverage is 100%.
    pub coverage_met: Option<bool>,
    /// Scored cells of `gen-m<k>`: `cells - deviations`.
    pub gen_scored: Option<usize>,
    /// Passing share of the scored cells; `None` when nothing was scored.
    pub gen_rate: Option<f64>,
    /// Scored cells that fail.
    pub gen_known_failures: Option<usize>,
    /// Failures the 0.999 rate allows: `floor(0.001 * scored)`.
    pub gen_allowance: Option<usize>,
    /// At least [`MIN_SCORED`] scored cells at the 0.999 rate.
    pub gen_met: Option<bool>,
    /// `ironcalc-calc` failing cells less the named exit exceptions.
    pub calc_failing: Option<usize>,
    /// No failing cell is left.
    pub calc_met: Option<bool>,
    /// Known failures of the milestone's probe row.
    pub probe_known_failures: Option<usize>,
    /// No probe failure is known.
    pub probes_met: Option<bool>,
    /// All four measures are met.
    pub exit: bool,
}

/// One row per milestone of [`MILESTONES`].
#[must_use]
pub fn summarize(report: &RunReport) -> Vec<ExitRow> {
    MILESTONES
        .iter()
        .map(|&m| {
            let mut row = ExitRow {
                milestone: m.to_string(),
                ..ExitRow::default()
            };
            if let Some(c) = report.catalog_coverage.iter().find(|c| c.milestone == m) {
                row.functions = Some(c.functions);
                row.covered = Some(c.covered);
                row.coverage_met = Some(c.covered == c.functions);
            }
            let suite = format!("gen-{}", m.to_ascii_lowercase());
            if let Some(s) = report.suites.iter().find(|s| s.name == suite) {
                let scored = s.cells.saturating_sub(s.deviations);
                let passed = s.cells_passed.min(scored);
                row.gen_scored = Some(scored);
                row.gen_rate = (scored > 0).then(|| s.cell_pass_rate());
                row.gen_known_failures = Some(scored - passed);
                row.gen_allowance = Some(scored / RATE_DEN);
                row.gen_met = Some(
                    scored >= MIN_SCORED
                        && passed.saturating_mul(RATE_DEN) >= scored.saturating_mul(RATE_NUM),
                );
            }
            let scope = report.function_scope.as_ref();
            if let Some(r) = scope.and_then(|s| s.rows.iter().find(|r| r.milestone == m)) {
                let failing = r.failing.len().saturating_sub(r.excepted.len());
                row.calc_failing = Some(failing);
                row.calc_met = Some(failing == 0);
            }
            if let Some(p) = report.probe_milestones.iter().find(|p| p.milestone == m) {
                row.probe_known_failures = Some(p.known_failures);
                row.probes_met = Some(p.known_failures == 0);
            }
            row.exit = [row.coverage_met, row.gen_met, row.calc_met, row.probes_met]
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
    let _ = writeln!(md, "\n## Milestone exit (decision 0085)\n");
    let _ = writeln!(
        md,
        "Thresholds: catalog coverage `covered` equals `functions`; gen suite at least {MIN_SCORED} scored cells (`cells - deviations`) at a pass rate of 0.999 or more, with known failures at most the allowance `floor(0.001 * scored)`; ironcalc-calc has no failing cell once exit exceptions are excluded; probes have no known failure. An exit measure, not a gate.\n"
    );
    let _ = writeln!(
        md,
        "| milestone | coverage (covered/functions) | gen suite (scored, rate, known failures/allowance) | ironcalc-calc (failing excl. exceptions) | probes (known failures) | exit |"
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
                "{scored}, {}, {}/{} {}",
                r.gen_rate
                    .map_or_else(|| "n/a".to_string(), |x| format!("{x:.5}")),
                count(r.gen_known_failures),
                count(r.gen_allowance),
                verdict(r.gen_met)
            ),
        };
        let _ = writeln!(
            md,
            "| {} | {coverage} | {generated} | {} {} | {} {} | {} |",
            r.milestone,
            count(r.calc_failing),
            verdict(r.calc_met),
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
    use crate::function_scope::{FunctionScope, MilestoneScope};
    use crate::probe_milestones::MilestoneRow;
    use crate::report::SuiteSummary;

    fn report(gen_cells: Option<(usize, usize)>) -> RunReport {
        let mut r = RunReport::build("t", &[], &[], &Catalog::from_functions([]));
        r.catalog_coverage = vec![MilestoneCoverage {
            milestone: "M5".into(),
            functions: 10,
            covered: 10,
            ..MilestoneCoverage::default()
        }];
        r.suites = gen_cells
            .map(|(cells, passed)| SuiteSummary {
                name: "gen-m5".into(),
                cells,
                cells_passed: passed,
                ..SuiteSummary::default()
            })
            .into_iter()
            .collect();
        r.function_scope = Some(FunctionScope {
            rows: vec![MilestoneScope {
                milestone: "M5".into(),
                failing: vec!["a!S!A1".into()],
                excepted: vec!["a!S!A1".into()],
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

    fn m5(r: &RunReport) -> ExitRow {
        summarize(r).remove(0)
    }

    #[test]
    fn all_met() {
        let r = report(Some((22_400, 22_380)));
        let rows = summarize(&r);
        let row = &rows[0];
        assert_eq!(row.gen_allowance, Some(22));
        assert_eq!(row.gen_known_failures, Some(20));
        assert_eq!(row.calc_failing, Some(0));
        assert!(row.exit, "{row:?}");
        let mut md = String::new();
        write_markdown(&rows, &mut md);
        assert!(
            md.contains("| M5 | 10/10 met | 22400, 0.99911, 20/22 met | 0 met | 0 met | met |"),
            "{md}"
        );
        assert!(
            md.contains("| M6 | n/a/n/a n/a | n/a | n/a n/a | n/a n/a | not met |"),
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
        let row = m5(&r);
        assert_eq!(row.probes_met, Some(false));
        assert!(!row.exit);
    }

    #[test]
    fn missing_gen_suite_is_not_applicable() {
        let row = m5(&report(None));
        assert_eq!(row.gen_met, None);
        assert!(!row.exit);
    }
}
