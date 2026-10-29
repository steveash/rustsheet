//! Milestone exit status (T-2177): the four exit measures of decision 0085, for M5 and (by its
//! "same as M5" inheritance) M6, next to their thresholds, and M2's (decision 0135, from
//! docs/milestones.md and decision 0058): no coverage criterion, and its generated suite needs
//! 10,000 scored cells with none visibly off; and M7's (decision 0140): gen-m7 at M5's rate over
//! 10,000 scored cells, no spill-extent failure in gen-m7 or the M7 probes, and coverage and
//! ironcalc-calc shown but not required; and M1's and M11's (decision 0146 items 1 and 5): gen-m1
//! and smoke with nothing visibly off and the frozen `m1-probes` list as M1's probe measure, and
//! corpus-bench at 0.998 over 200,000 scored cells. Decision 0114 reads them at visible
//! parity: the generated suite, ironcalc-calc and probes count visibly-off cells, and the bit-level
//! counts are shown beside them for Phase 1b. Every input is a row the report already computes, so
//! nothing here scores anything; a missing input is `None`, never met. It is an exit measure, not
//! a gate.

use std::fmt::Write as _;

use serde::{Deserialize, Serialize};

use crate::closeness::Visible;
use crate::probe_milestones::{parse_milestone, row_label};
use crate::report::RunReport;
use crate::runner::CellKind;

/// The milestones this section covers. M8, M9 and M10 are T-0096, T-0095 and T-0097.
pub const MILESTONES: [&str; 6] = ["M1", "M2", "M5", "M6", "M7", "M11"];
/// Fewest scored cells the generated suite must have (decision 0085 item 2).
pub const MIN_SCORED: usize = 20_000;
/// M2's fewest scored cells of `gen-m2` (docs/milestones.md "M2", decision 0058).
pub const M2_MIN_SCORED: usize = 10_000;
/// M7's fewest scored cells of `gen-m7` (docs/milestones.md "M7", decision 0140).
pub const M7_MIN_SCORED: usize = 10_000;
/// M1's fewest scored cells of `gen-m1` (decision 0146 item 1).
pub const M1_MIN_SCORED: usize = 3_000;
/// M11's fewest scored cells of `corpus-bench` (decision 0146 item 5).
pub const M11_MIN_SCORED: usize = 200_000;
/// M11's corpus-bench rate over [`RATE_DEN`]: 99.8%, the best published `SpreadsheetBench` score
/// (decision 0146 item 5).
pub const M11_RATE_NUM: usize = 998;
/// The frozen probe list M1's probe measure reads instead of a probe row (decision 0146 item 1).
pub const M1_PROBE_LIST: &str = "m1-probes";
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

/// One suite a milestone's exit requires.
struct SuiteNeed {
    /// The suite's name.
    name: String,
    /// Fewest scored cells.
    min_scored: usize,
    /// Share of the scored cells, over [`RATE_DEN`], that must not be visibly off.
    rate_num: usize,
}

impl SuiteNeed {
    /// The milestone's generated suite `gen-m<k>`.
    fn generated(milestone: &str, min_scored: usize, rate_num: usize) -> Self {
        Self::named(
            &format!("gen-{}", milestone.to_ascii_lowercase()),
            min_scored,
            rate_num,
        )
    }

    fn named(name: &str, min_scored: usize, rate_num: usize) -> Self {
        Self {
            name: name.to_string(),
            min_scored,
            rate_num,
        }
    }
}

/// A milestone's required suites and which other measures are its criteria.
struct Criteria {
    /// The required suites; the first is the row's `gen_*` suite (decision 0146).
    suites: Vec<SuiteNeed>,
    /// The milestone list the probe measure reads instead of a probe row (M1).
    probe_list: Option<&'static str>,
    /// Catalog coverage is an exit criterion (M5, M6; not M2, decision 0135, nor M7).
    coverage: bool,
    /// The ironcalc-calc function-scope row is an exit criterion (not M7, decision 0140).
    calc: bool,
    /// Spill extents are an exit criterion (M7, decision 0140).
    extent: bool,
    /// Coverage and ironcalc-calc counts are shown even where not required (M2, M7); M1 and M11
    /// have neither measure, so both stay `n/a` (decision 0146).
    scope_shown: bool,
}

fn criteria(milestone: &str) -> Criteria {
    let generated =
        |min_scored, rate_num| vec![SuiteNeed::generated(milestone, min_scored, rate_num)];
    match milestone {
        // gen-m1 and smoke with none visibly off; no coverage or function scope (decision 0146).
        "M1" => Criteria {
            suites: vec![
                SuiteNeed::generated(milestone, M1_MIN_SCORED, RATE_DEN),
                SuiteNeed::named("smoke", 0, RATE_DEN),
            ],
            probe_list: Some(M1_PROBE_LIST),
            coverage: false,
            calc: false,
            extent: false,
            scope_shown: false,
        },
        // "100%": no gen-m2 cell visibly off.
        "M2" => Criteria {
            suites: generated(M2_MIN_SCORED, RATE_DEN),
            probe_list: None,
            coverage: false,
            calc: true,
            extent: false,
            scope_shown: true,
        },
        "M7" => Criteria {
            suites: generated(M7_MIN_SCORED, RATE_NUM),
            probe_list: None,
            coverage: false,
            calc: false,
            extent: true,
            scope_shown: true,
        },
        "M11" => Criteria {
            suites: vec![SuiteNeed::named(
                "corpus-bench",
                M11_MIN_SCORED,
                M11_RATE_NUM,
            )],
            probe_list: None,
            coverage: false,
            calc: false,
            extent: false,
            scope_shown: false,
        },
        _ => Criteria {
            suites: generated(MIN_SCORED, RATE_NUM),
            probe_list: None,
            coverage: true,
            calc: true,
            extent: false,
            scope_shown: true,
        },
    }
}

/// One required suite's measure; `None` means the suite was absent from the run.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SuiteExit {
    /// The suite's name.
    pub name: String,
    /// Scored cells: `cells - deviations`.
    pub scored: Option<usize>,
    /// Bit-exact share of the scored cells (Phase 1b); `None` when nothing was scored.
    pub rate: Option<f64>,
    /// Scored cells that fail bit for bit (Phase 1b).
    pub known_failures: Option<usize>,
    /// Scored cells that are visibly off (decision 0114).
    pub visibly_off: Option<usize>,
    /// Share of the scored cells that are not visibly off; `None` when nothing was scored.
    pub rate15: Option<f64>,
    /// Visibly-off cells the rate allows.
    pub allowance: Option<usize>,
    /// At least the floor of scored cells with at most the allowance visibly off.
    pub met: Option<bool>,
    /// The visibly-off cells as `case!Sheet!A1`, for `scripts/exit_owners.py` (decision 0135).
    #[serde(default)]
    pub visibly_off_cells: Vec<String>,
}

/// `need`'s measure from the report's suite row and its cases.
fn suite_exit(report: &RunReport, need: &SuiteNeed) -> SuiteExit {
    let mut out = SuiteExit {
        name: need.name.clone(),
        ..SuiteExit::default()
    };
    let Some(s) = report.suites.iter().find(|s| s.name == need.name) else {
        return out;
    };
    let scored = s.cells.saturating_sub(s.deviations);
    let passed = s.cells_passed.min(scored);
    let visible = (s.closeness.visible_numeric + s.closeness.visible_other).min(scored);
    out.scored = Some(scored);
    out.rate = (scored > 0).then(|| s.cell_pass_rate());
    out.known_failures = Some(scored - passed);
    out.visibly_off = Some(visible);
    out.rate15 = (scored > 0).then(|| s.rate(scored - visible));
    out.allowance = Some(scored * (RATE_DEN - need.rate_num) / RATE_DEN);
    out.met =
        Some(scored >= need.min_scored && meets_rate(scored - visible, scored, need.rate_num));
    out.visibly_off_cells = report
        .cases
        .iter()
        .filter(|r| r.suite == need.name)
        .flat_map(|r| {
            r.cells
                .iter()
                .filter(|c| Visible::is_off(c))
                .map(move |c| format!("{}!{}", r.id, c.key()))
        })
        .collect();
    out
}

/// One milestone's four measures; `None` means the input was absent from the run.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ExitRow {
    /// `M<n>`.
    pub milestone: String,
    /// The first required suite, whose measure the `gen_*` fields hold: `gen-m<k>`, or
    /// `corpus-bench` for M11 (decision 0146).
    #[serde(default)]
    pub gen_suite: String,
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
    /// The other required suites (M1's smoke, decision 0146 item 1), each with its own floor
    /// and rate.
    #[serde(default)]
    pub more_suites: Vec<SuiteExit>,
    /// `ironcalc-calc` failing cells less the named exit exceptions (Phase 1b).
    pub calc_failing: Option<usize>,
    /// Of those, the visibly-off cells (decision 0114).
    #[serde(default)]
    pub calc_visibly_off: Option<usize>,
    /// No visibly-off cell is left; `None` too for a milestone without an ironcalc-calc
    /// criterion (M7), whose counts are shown but not required.
    pub calc_met: Option<bool>,
    /// Known failures of the milestone's probe row (Phase 1b).
    pub probe_known_failures: Option<usize>,
    /// Visibly-off cells of the milestone's probe row (decision 0114).
    #[serde(default)]
    pub probe_visibly_off: Option<usize>,
    /// No probe cell is visibly off.
    pub probes_met: Option<bool>,
    /// The "Probe milestones" row the probe measure reads: `M0-M2` for M2 (decision 0051); for M1
    /// the name of the milestone list it reads instead, `m1-probes` (decision 0146 item 1).
    #[serde(default)]
    pub probe_row: String,
    /// Spill-extent failures of the generated suite and the probe row (decision 0063); `None`
    /// for a milestone without the criterion (only M7 has it, decision 0140).
    #[serde(default)]
    pub extent_failures: Option<usize>,
    /// No spill-extent failure.
    #[serde(default)]
    pub extent_met: Option<bool>,
    /// Every measure the milestone requires is met.
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
            let coverage = report.catalog_coverage.iter().find(|c| c.milestone == m);
            if let Some(c) = coverage.filter(|_| need.scope_shown) {
                row.functions = Some(c.functions);
                row.covered = Some(c.covered);
                row.coverage_met = need.coverage.then_some(c.covered == c.functions);
            }
            let mut suites = need.suites.iter().map(|n| suite_exit(report, n));
            let first = suites.next().unwrap_or_default();
            row.more_suites = suites.collect();
            row.gen_scored = first.scored;
            row.gen_rate = first.rate;
            row.gen_known_failures = first.known_failures;
            row.gen_visibly_off = first.visibly_off;
            row.gen_rate15 = first.rate15;
            row.gen_allowance = first.allowance;
            row.gen_met = first.met;
            row.gen_visibly_off_cells = first.visibly_off_cells;
            row.gen_suite = first.name;
            let suite = &row.gen_suite;
            let scope = report.function_scope.as_ref().filter(|_| need.scope_shown);
            if let Some(r) = scope.and_then(|s| s.rows.iter().find(|r| r.milestone == m)) {
                let failing = r.failing.len().saturating_sub(r.excepted.len());
                let visible = r
                    .visibly_off
                    .iter()
                    .filter(|c| !r.excepted.contains(c))
                    .count();
                row.calc_failing = Some(failing);
                row.calc_visibly_off = Some(visible);
                row.calc_met = need.calc.then_some(visible == 0);
            }
            let probes = &report.probe_milestones;
            let probe = probes.iter().find(|p| p.milestone == row.probe_row);
            if let Some(list) = need.probe_list {
                row.probe_row = list.to_string();
                let lists = &report.milestone_lists;
                if let Some(l) = lists.iter().find(|l| l.name == list && l.cases_run > 0) {
                    row.probe_known_failures = Some(l.known_failures);
                    row.probe_visibly_off = Some(l.visibly_off);
                    row.probes_met = Some(l.visibly_off == 0);
                }
            } else if let Some(p) = probe {
                row.probe_known_failures = Some(p.known_failures);
                row.probe_visibly_off = Some(p.visibly_off);
                row.probes_met = Some(p.visibly_off == 0);
            }
            if need.extent && row.gen_scored.is_some() {
                if let Some(p) = probe {
                    // Extent cells always fail, so the report's failing-cell lists keep them all.
                    let generated = report
                        .cases
                        .iter()
                        .filter(|r| &r.suite == suite)
                        .flat_map(|r| &r.cells)
                        .filter(|c| c.kind == CellKind::SpillExtent)
                        .count();
                    let failures = generated + p.spill_extent_failures;
                    row.extent_failures = Some(failures);
                    row.extent_met = Some(failures == 0);
                }
            }
            let required =
                |criterion: bool, met: Option<bool>| if criterion { met } else { Some(true) };
            row.exit = [
                required(need.coverage, row.coverage_met),
                row.gen_met,
                required(need.calc, row.calc_met),
                row.probes_met,
                required(need.extent, row.extent_met),
            ]
            .into_iter()
            .chain(row.more_suites.iter().map(|s| s.met))
            .all(|v| v == Some(true));
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

/// One suite's cell of the report table: scored cells, 15-digit rate, visibly off over the
/// allowance, bit failures and the verdict.
fn suite_text(s: &SuiteExit) -> String {
    match s.scored {
        None => "n/a".to_string(),
        Some(scored) => format!(
            "{scored}, {}, {}/{} [{}] {}",
            s.rate15
                .map_or_else(|| "n/a".to_string(), |x| format!("{x:.5}")),
            count(s.visibly_off),
            count(s.allowance),
            count(s.known_failures),
            verdict(s.met)
        ),
    }
}

/// The report section.
pub fn write_markdown(rows: &[ExitRow], md: &mut String) {
    if rows.is_empty() {
        return;
    }
    let _ = writeln!(md, "\n## Milestone exit (decisions 0085, 0114)\n");
    let _ = writeln!(
        md,
        "Thresholds, at visible parity (decision 0114): catalog coverage `covered` equals `functions`; gen suite at least {MIN_SCORED} scored cells (`cells - deviations`) with at least 0.999 of them not visibly off, so visibly-off cells at most the allowance `floor(0.001 * scored)`; ironcalc-calc has no visibly-off cell once exit exceptions are excluded; probes have no visibly-off cell. M2 (decision 0135): coverage is shown but not required, and gen-m2 needs {M2_MIN_SCORED} scored cells with none visibly off; its probes are the M0-M2 row. M7 (decision 0140): gen-m7 needs {M7_MIN_SCORED} scored cells at the 0.999 rate, gen-m7 and the M7 probes have no spill-extent failure (a cell the engine filled that Excel left empty, decision 0063), and coverage and ironcalc-calc are shown but not required. M1 (decision 0146): gen-m1 needs {M1_MIN_SCORED} scored cells and smoke any number, both with none visibly off, and its probes are the frozen `{M1_PROBE_LIST}` list; M11: corpus-bench needs {M11_MIN_SCORED} scored cells with at least 0.998 not visibly off, `n/a` when it was not fetched; neither has a coverage or ironcalc-calc criterion. A suite other than the milestone's gen suite is named in its column. Bracketed counts are the bit-level failures, Phase 1b's target. An exit measure, not a gate.\n"
    );
    let _ = writeln!(
        md,
        "| milestone | coverage (covered/functions) | gen suite (scored, 15-digit rate, visibly off/allowance [bit failures]) | ironcalc-calc (visibly off [bit failures], excl. exceptions) | probes (visibly off [bit failures]) | spill extents (failures) | exit |"
    );
    let _ = writeln!(md, "|---|---|---|---|---|---|---|");
    for r in rows {
        let coverage = format!(
            "{}/{} {}",
            count(r.covered),
            count(r.functions),
            verdict(r.coverage_met)
        );
        let first = SuiteExit {
            name: r.gen_suite.clone(),
            scored: r.gen_scored,
            rate15: r.gen_rate15,
            visibly_off: r.gen_visibly_off,
            allowance: r.gen_allowance,
            known_failures: r.gen_known_failures,
            met: r.gen_met,
            ..SuiteExit::default()
        };
        // The generated suite `gen-m<k>` goes unnamed, as before decision 0146; any other is named.
        let generated = format!("gen-{}", r.milestone.to_ascii_lowercase());
        let mut generated = std::iter::once(&first)
            .chain(&r.more_suites)
            .map(|s| {
                let text = suite_text(s);
                if s.name.is_empty() || s.name == generated {
                    text
                } else {
                    format!("{}: {text}", s.name)
                }
            })
            .collect::<Vec<_>>()
            .join("; ");
        if r.gen_scored.is_none() && r.more_suites.is_empty() {
            generated = "n/a".to_string();
        }
        let _ = writeln!(
            md,
            "| {} | {coverage} | {generated} | {} [{}] {} | {} [{}] {} | {} {} | {} |",
            r.milestone,
            count(r.calc_visibly_off),
            count(r.calc_failing),
            verdict(r.calc_met),
            count(r.probe_visibly_off),
            count(r.probe_known_failures),
            verdict(r.probes_met),
            count(r.extent_failures),
            verdict(r.extent_met),
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
        let row = rows.iter().find(|row| row.milestone == "M5").unwrap();
        assert_eq!(row.gen_allowance, Some(22));
        assert_eq!(row.gen_known_failures, Some(20));
        assert_eq!(row.calc_failing, Some(0));
        assert!(row.exit, "{row:?}");
        let mut md = String::new();
        write_markdown(&rows, &mut md);
        assert!(
            md.contains(
                "| M5 | 10/10 met | 22400, 0.99911, 20/22 [20] met | 0 [0] met | 0 [0] met | n/a n/a | met |"
            ),
            "{md}"
        );
        assert!(
            md.contains(
                "| M6 | n/a/n/a n/a | n/a | n/a [n/a] n/a | n/a [n/a] n/a | n/a n/a | not met |"
            ),
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

    /// One milestone's inputs added to [`report`]`(None)`: `gen` (cells, visibly off) of its
    /// generated suite, its catalog coverage (covered, functions), its function-scope row
    /// (`failing`, of which `excepted`) and its probe row with `probe_failures` bit-level failures.
    struct Inputs<'a> {
        milestone: &'a str,
        probe_row: &'a str,
        gen: (usize, usize),
        coverage: (usize, usize),
        failing: &'a str,
        excepted: bool,
        probe_failures: usize,
    }

    fn milestone_report(i: &Inputs<'_>) -> RunReport {
        let mut r = report(None);
        let (cells, visible) = i.gen;
        r.catalog_coverage.push(MilestoneCoverage {
            milestone: i.milestone.into(),
            covered: i.coverage.0,
            functions: i.coverage.1,
            ..MilestoneCoverage::default()
        });
        r.suites.push(SuiteSummary {
            name: format!("gen-{}", i.milestone.to_ascii_lowercase()),
            cells,
            cells_passed: cells - visible,
            closeness: Closeness {
                visible_other: visible,
                ..Closeness::default()
            },
            ..SuiteSummary::default()
        });
        let failing = vec![i.failing.to_string()];
        r.function_scope
            .as_mut()
            .unwrap()
            .rows
            .push(MilestoneScope {
                milestone: i.milestone.into(),
                excepted: if i.excepted {
                    failing.clone()
                } else {
                    Vec::new()
                },
                visibly_off: failing.clone(),
                failing,
                ..MilestoneScope::default()
            });
        r.probe_milestones.push(MilestoneRow {
            milestone: i.probe_row.into(),
            known_failures: i.probe_failures,
            ..MilestoneRow::default()
        });
        r
    }

    /// A run whose M2 inputs meet M2's exit: `gen_cells` (cells, visibly off) of gen-m2.
    fn m2_report(gen_cells: (usize, usize)) -> RunReport {
        milestone_report(&Inputs {
            milestone: "M2",
            probe_row: "M0-M2",
            gen: gen_cells,
            coverage: (30, 37),
            failing: "ironcalc-calc/IF_ARRAY.xlsx!Sheet1!C11",
            excepted: true,
            probe_failures: 3,
        })
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
                "| M2 | 30/37 n/a | 10000, 1.00000, 0/0 [0] met | 0 [0] met | 0 [3] met | n/a n/a | met |"
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

    /// A run whose M7 inputs meet M7's exit: `gen_cells` (cells, visibly off) of gen-m7, an
    /// ironcalc-calc M7 row with a visibly-off cell and coverage short (neither required).
    fn m7_report(gen_cells: (usize, usize)) -> RunReport {
        milestone_report(&Inputs {
            milestone: "M7",
            probe_row: "M7",
            gen: gen_cells,
            coverage: (39, 40),
            failing: "c!S!A1",
            excepted: false,
            probe_failures: 2,
        })
    }

    /// A spill-extent cell of gen-m7, as `spill_extent.rs` adds it.
    fn extent_case() -> crate::runner::CaseResult {
        use crate::report::tests::{case, cell};
        use crate::value::Verdict;
        let mut g = case("gen-m7/abc.case.json", 1);
        g.cells = vec![CellResult {
            kind: CellKind::SpillExtent,
            ..cell("SEQUENCE(2)", Verdict::TypeMismatch)
        }];
        g
    }

    #[test]
    fn m7_needs_ten_thousand_gen_cells_at_the_rate_and_no_extent_failure() {
        let r = m7_report((M7_MIN_SCORED, 10));
        let m7 = row(&r, "M7");
        assert_eq!((m7.gen_allowance, m7.gen_met), (Some(10), Some(true)));
        assert_eq!(
            (m7.covered, m7.functions, m7.coverage_met),
            (Some(39), Some(40), None),
            "coverage is shown, not required"
        );
        assert_eq!(
            (m7.calc_failing, m7.calc_visibly_off, m7.calc_met),
            (Some(1), Some(1), None),
            "ironcalc-calc is shown, not required"
        );
        assert_eq!(m7.probe_row, "M7");
        assert_eq!(
            (m7.probe_known_failures, m7.probes_met),
            (Some(2), Some(true))
        );
        assert_eq!((m7.extent_failures, m7.extent_met), (Some(0), Some(true)));
        assert!(m7.exit, "{m7:?}");
        let mut md = String::new();
        write_markdown(&summarize(&r), &mut md);
        assert!(
            md.contains(
                "| M7 | 39/40 n/a | 10000, 0.99900, 10/10 [10] met | 1 [1] n/a | 0 [2] met | 0 met | met |"
            ),
            "{md}"
        );
        // Other rows have no extent criterion.
        assert_eq!(m5(&r).extent_failures, None);
        // 9,999 scored cells is not met.
        let m7 = row(&m7_report((M7_MIN_SCORED - 1, 0)), "M7");
        assert_eq!(m7.gen_met, Some(false));
        assert!(!m7.exit);
        // The allowance plus one visibly-off cell is not met.
        let m7 = row(&m7_report((M7_MIN_SCORED, 11)), "M7");
        assert_eq!((m7.gen_allowance, m7.gen_met), (Some(10), Some(false)));
        assert!(!m7.exit);
        // A visibly-off M7 probe cell is not met.
        let mut r = m7_report((M7_MIN_SCORED, 0));
        r.probe_milestones[1].visibly_off = 1;
        assert!(!row(&r, "M7").exit);
    }

    #[test]
    fn one_m7_extent_failure_is_not_met() {
        let mut r = m7_report((M7_MIN_SCORED, 0));
        r.cases = vec![extent_case()];
        let m7 = row(&r, "M7");
        assert_eq!((m7.extent_failures, m7.extent_met), (Some(1), Some(false)));
        assert!(!m7.exit);
        let mut r = m7_report((M7_MIN_SCORED, 0));
        r.probe_milestones[1].spill_extent_failures = 1;
        let m7 = row(&r, "M7");
        assert_eq!((m7.extent_failures, m7.extent_met), (Some(1), Some(false)));
        assert!(!m7.exit);
        // Without the gen suite or the probe row the measure is absent, never met.
        let mut r = m7_report((M7_MIN_SCORED, 0));
        r.probe_milestones.pop();
        assert_eq!(row(&r, "M7").extent_met, None);
    }

    /// A suite row of `cells` scored cells, `visible` of them visibly off.
    fn suite(name: &str, cells: usize, visible: usize) -> SuiteSummary {
        SuiteSummary {
            name: name.into(),
            cells,
            cells_passed: cells - visible,
            closeness: Closeness {
                visible_numeric: visible,
                ..Closeness::default()
            },
            ..SuiteSummary::default()
        }
    }

    /// A run whose M1 inputs meet M1's exit: gen-m1 and smoke cells and the `m1-probes` list's
    /// visibly-off cells (with one bit-level failure either way).
    fn m1_report(gen: usize, smoke_off: usize, list_off: usize) -> RunReport {
        let mut r = report(None);
        r.suites.push(suite("gen-m1", gen, 0));
        r.suites.push(suite("smoke", 46, smoke_off));
        r.milestone_lists.push(crate::milestone_lists::ListSummary {
            name: M1_PROBE_LIST.into(),
            cases: 89,
            cases_run: 89,
            cells: 1858,
            known_failures: 1,
            visibly_off: list_off,
            ..crate::milestone_lists::ListSummary::default()
        });
        r
    }

    #[test]
    fn m1_needs_gen_m1_and_smoke_none_visibly_off_and_the_m1_probes_list() {
        let r = m1_report(3_750, 0, 0);
        let m1 = row(&r, "M1");
        assert_eq!(m1.gen_suite, "gen-m1");
        assert_eq!((m1.gen_allowance, m1.gen_met), (Some(0), Some(true)));
        assert_eq!(m1.more_suites.len(), 1);
        assert_eq!(
            (m1.more_suites[0].name.as_str(), m1.more_suites[0].met),
            ("smoke", Some(true))
        );
        assert_eq!(m1.probe_row, M1_PROBE_LIST);
        assert_eq!(
            (m1.probe_known_failures, m1.probe_visibly_off, m1.probes_met),
            (Some(1), Some(0), Some(true))
        );
        assert_eq!((m1.coverage_met, m1.calc_met), (None, None));
        assert!(m1.exit, "{m1:?}");
        let mut md = String::new();
        write_markdown(&summarize(&r), &mut md);
        assert!(
            md.contains(
                "| M1 | n/a/n/a n/a | 3750, 1.00000, 0/0 [0] met; smoke: 46, 1.00000, 0/0 [0] met | n/a [n/a] n/a | 0 [1] met | n/a n/a | met |"
            ),
            "{md}"
        );
        // gen-m1 short of 3,000 cells, a visibly-off smoke cell or a visibly-off listed cell
        // is not met.
        let m1 = row(&m1_report(M1_MIN_SCORED - 1, 0, 0), "M1");
        assert_eq!(m1.gen_met, Some(false));
        assert!(!m1.exit);
        let m1 = row(&m1_report(3_750, 1, 0), "M1");
        assert_eq!(
            (m1.gen_met, m1.more_suites[0].met),
            (Some(true), Some(false))
        );
        assert!(!m1.exit);
        let m1 = row(&m1_report(3_750, 0, 1), "M1");
        assert_eq!(m1.probes_met, Some(false));
        assert!(!m1.exit);
        // A run that did not execute the list (`--suite gen-m1`) has no probe measure.
        let mut r = m1_report(3_750, 0, 0);
        r.milestone_lists[0].cases_run = 0;
        let m1 = row(&r, "M1");
        assert_eq!(m1.probes_met, None);
        assert!(!m1.exit);
    }

    /// A run with `corpus` (scored cells, visibly off) of corpus-bench, when fetched, and a
    /// clean M11 probe row.
    fn m11_report(corpus: Option<(usize, usize)>) -> RunReport {
        let mut r = report(None);
        if let Some((cells, visible)) = corpus {
            r.suites.push(suite("corpus-bench", cells, visible));
        }
        r.probe_milestones.push(MilestoneRow {
            milestone: "M11".into(),
            ..MilestoneRow::default()
        });
        // A real report has M11 coverage and function-scope rows; M11's exit shows neither.
        r.catalog_coverage.push(MilestoneCoverage {
            milestone: "M11".into(),
            ..MilestoneCoverage::default()
        });
        r.function_scope
            .as_mut()
            .unwrap()
            .rows
            .push(MilestoneScope {
                milestone: "M11".into(),
                failing: vec!["c!S!A1".into()],
                visibly_off: vec!["c!S!A1".into()],
                ..MilestoneScope::default()
            });
        r
    }

    #[test]
    fn m11_needs_corpus_bench_at_the_leaderboard_rate_and_is_na_without_it() {
        // 99.834% of 205,366 cells (decision 0146's numbers): the allowance is 410.
        let r = m11_report(Some((205_366, 340)));
        let m11 = row(&r, "M11");
        assert_eq!(m11.gen_suite, "corpus-bench");
        assert_eq!((m11.gen_allowance, m11.gen_met), (Some(410), Some(true)));
        assert_eq!(m11.probe_row, "M11");
        assert_eq!(m11.probes_met, Some(true));
        assert_eq!((m11.coverage_met, m11.calc_met), (None, None));
        assert!(m11.exit, "{m11:?}");
        let mut md = String::new();
        write_markdown(&summarize(&r), &mut md);
        assert!(
            md.contains(
                "| M11 | n/a/n/a n/a | corpus-bench: 205366, 0.99834, 340/410 [340] met | n/a [n/a] n/a | 0 [0] met | n/a n/a | met |"
            ),
            "{md}"
        );
        // Past the allowance, or short of 200,000 cells, is not met.
        let m11 = row(&m11_report(Some((205_366, 411))), "M11");
        assert_eq!(m11.gen_met, Some(false));
        assert!(!m11.exit);
        let m11 = row(&m11_report(Some((M11_MIN_SCORED - 1, 0))), "M11");
        assert_eq!(m11.gen_met, Some(false));
        assert!(!m11.exit);
        // corpus-bench not fetched: n/a, never met.
        let r = m11_report(None);
        let m11 = row(&r, "M11");
        assert_eq!((m11.gen_scored, m11.gen_met), (None, None));
        assert!(!m11.exit);
        let mut md = String::new();
        write_markdown(&summarize(&r), &mut md);
        assert!(
            md.contains(
                "| M11 | n/a/n/a n/a | n/a | n/a [n/a] n/a | 0 [0] met | n/a n/a | not met |"
            ),
            "{md}"
        );
    }
}
