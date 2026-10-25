//! Milestone exit status (T-2177): the four exit measures of decision 0085, for M5 and (by its
//! "same as M5" inheritance) M6, next to their thresholds. Decision 0114 reads them at visible
//! parity: the generated suite, ironcalc-calc and probes count visibly-off cells, and the bit-level
//! counts are shown beside them for Phase 1b. Every input is a row the report already computes, so
//! nothing here scores anything; a missing input is `None`, never met. It is an exit measure, not
//! a gate.

use std::fmt::Write as _;

use serde::{Deserialize, Serialize};

use crate::report::RunReport;

/// The milestones this section covers. M7 to M11 need their own decision first.
pub const MILESTONES: [&str; 2] = ["M5", "M6"];
/// Fewest scored cells the generated suite must have (decision 0085 item 2).
pub const MIN_SCORED: usize = 20_000;
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
    /// Visibly-off cells the 0.999 rate allows: `floor(0.001 * scored)`.
    pub gen_allowance: Option<usize>,
    /// At least [`MIN_SCORED`] scored cells with at most the allowance visibly off.
    pub gen_met: Option<bool>,
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
                let visible = (s.closeness.visible_numeric + s.closeness.visible_other).min(scored);
                row.gen_scored = Some(scored);
                row.gen_rate = (scored > 0).then(|| s.cell_pass_rate());
                row.gen_known_failures = Some(scored - passed);
                row.gen_visibly_off = Some(visible);
                row.gen_rate15 = (scored > 0).then(|| s.rate(scored - visible));
                row.gen_allowance = Some(scored / RATE_DEN);
                row.gen_met =
                    Some(scored >= MIN_SCORED && meets_rate(scored - visible, scored, RATE_NUM));
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
            if let Some(p) = report.probe_milestones.iter().find(|p| p.milestone == m) {
                row.probe_known_failures = Some(p.known_failures);
                row.probe_visibly_off = Some(p.visibly_off);
                row.probes_met = Some(p.visibly_off == 0);
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
    let _ = writeln!(md, "\n## Milestone exit (decisions 0085, 0114)\n");
    let _ = writeln!(
        md,
        "Thresholds, at visible parity (decision 0114): catalog coverage `covered` equals `functions`; gen suite at least {MIN_SCORED} scored cells (`cells - deviations`) with at least 0.999 of them not visibly off, so visibly-off cells at most the allowance `floor(0.001 * scored)`; ironcalc-calc has no visibly-off cell once exit exceptions are excluded; probes have no visibly-off cell. Bracketed counts are the bit-level failures, Phase 1b's target. An exit measure, not a gate.\n"
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

    fn m5(r: &RunReport) -> ExitRow {
        summarize(r).remove(0)
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
        let row = &rows[0];
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
}
