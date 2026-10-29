//! Milestone exit status (T-2177): the four exit measures of decision 0085, for M5 and (by its
//! "same as M5" inheritance) M6, next to their thresholds, and M2's (decision 0135, from
//! docs/milestones.md and decision 0058): no coverage criterion, and its generated suite needs
//! 10,000 scored cells with none visibly off; and M7's (decision 0140): gen-m7 at M5's rate over
//! 10,000 scored cells, no spill-extent failure in gen-m7 or the M7 probes, and coverage and
//! ironcalc-calc shown but not required; and M1's and M11's (decision 0146 items 1 and 5): gen-m1
//! and smoke with nothing visibly off and the frozen `m1-probes` list as M1's probe measure, and
//! corpus-bench at 0.998 over 200,000 scored cells; and M9's (decision 0146 item 3): ironcalc-calc
//! at 0.9995 less its exit exceptions, every M9 function at the Phase 1 rate of `v15_checkpoint`,
//! and no M9 probe visibly off. Decision 0114 reads them at visible
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

/// The milestones this section covers. M10 is T-0097.
pub const MILESTONES: [&str; 8] = ["M1", "M2", "M5", "M6", "M7", "M8", "M9", "M11"];
/// Fewest scored cells the generated suite must have (decision 0085 item 2).
pub const MIN_SCORED: usize = 20_000;
/// M2's fewest scored cells of `gen-m2` (docs/milestones.md "M2", decision 0058).
pub const M2_MIN_SCORED: usize = 10_000;
/// M7's fewest scored cells of `gen-m7` (docs/milestones.md "M7", decision 0140).
pub const M7_MIN_SCORED: usize = 10_000;
/// M1's fewest scored cells of `gen-m1` (decision 0146 item 1).
pub const M1_MIN_SCORED: usize = 3_000;
/// M8's fewest scored cells of each of `gen-m8a` and `gen-m8b`: the M1 floor (decision 0146
/// item 2).
pub const M8_MIN_SCORED: usize = M1_MIN_SCORED;
/// M8's poi subset rate over [`RATE_DEN`]: 99.5% not visibly off (decision 0146 item 2(b)).
pub const M8_SUBSET_RATE_NUM: usize = 9_950;
/// The name M8's poi subset measure goes by among the row's `more_suites`.
pub const M8_SUBSET: &str = "poi M8 subset";
/// M11's fewest scored cells of `corpus-bench` (decision 0146 item 5).
pub const M11_MIN_SCORED: usize = 200_000;
/// M11's corpus-bench rate over [`RATE_DEN`]: 99.8%, the best published `SpreadsheetBench` score
/// (decision 0146 item 5).
pub const M11_RATE_NUM: usize = 9_980;
/// M9's ironcalc-calc rate over [`RATE_DEN`]: 99.95% of the row's scored cells not visibly off
/// (decision 0146 item 3).
pub const M9_CALC_RATE_NUM: usize = 9_995;
/// The frozen probe list M1's probe measure reads instead of a probe row (decision 0146 item 1).
pub const M1_PROBE_LIST: &str = "m1-probes";
/// The generated suite's pass rate must reach 99.9% of its scored cells; decision 0114's
/// per-function Phase 1 rate (`v15_checkpoint`) shares it.
pub(crate) const RATE_NUM: usize = 9_990;
/// The denominator of every rate here: fine enough for M9's 99.95% ([`M9_CALC_RATE_NUM`]).
pub(crate) const RATE_DEN: usize = 10_000;
/// 99% over [`RATE_DEN`]: the next parity tier below [`RATE_NUM`] (`attribution::Tier`, T-0068).
pub(crate) const RATE99_NUM: usize = 9_900;

/// Whether `within` of `scored` cells is a share of at least `num` / [`RATE_DEN`]; never with no
/// scored cell. The one definition of "at the rate" for the generated suite here, the Phase 1
/// rates of `v15_checkpoint` and the parity tiers of `attribution::Tier`.
#[must_use]
pub(crate) fn meets_rate(within: usize, scored: usize, num: usize) -> bool {
    scored > 0 && within.saturating_mul(RATE_DEN) >= scored.saturating_mul(num)
}

/// Visibly-off cells a rate of `num` / [`RATE_DEN`] allows over `scored` cells: the most that
/// still [`meets_rate`].
fn allowance(scored: usize, num: usize) -> usize {
    scored * (RATE_DEN - num) / RATE_DEN
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

/// What the ironcalc-calc function-scope row must meet.
#[derive(Clone, Copy)]
enum CalcRule {
    /// No visibly-off cell once exit exceptions are excluded (decision 0085).
    NoneOff,
    /// At least this share over [`RATE_DEN`] of the row's scored cells not visibly off (M9,
    /// decision 0146 item 3).
    Rate(usize),
}

/// A milestone's required suites and which other measures are its criteria.
struct Criteria {
    /// The required suites; the first is the row's `gen_*` suite (decision 0146).
    suites: Vec<SuiteNeed>,
    /// The milestone list the probe measure reads instead of a probe row (M1).
    probe_list: Option<&'static str>,
    /// Catalog coverage is an exit criterion (M5, M6; not M2, decision 0135, nor M7).
    coverage: bool,
    /// The ironcalc-calc function-scope row's exit criterion; `None` where it is none (M7,
    /// decision 0140; M1, M11).
    calc: Option<CalcRule>,
    /// Spill extents are an exit criterion (M7, decision 0140).
    extent: bool,
    /// Every in-scope function of the milestone meets `v15_checkpoint`'s Phase 1 rate (M9).
    function_rate: bool,
    /// The poi M8 subset (`poi_m8_subset`) must reach this rate over [`RATE_DEN`] with no
    /// blocked case (M8, decision 0146 item 2(b)).
    poi_subset: Option<usize>,
}

impl Criteria {
    /// Whether the row shows the coverage and ironcalc-calc counts. M2 to M7 show them, also
    /// where they are not required (M2's coverage, M7's both). M1 and M11 have none of the
    /// function measures (coverage, ironcalc-calc, spill extents), so both stay `n/a` there
    /// (decision 0146).
    fn shows_scope(&self) -> bool {
        self.coverage || self.calc.is_some() || self.extent
    }
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
            calc: None,
            extent: false,
            function_rate: false,
            poi_subset: None,
        },
        // "100%": no gen-m2 cell visibly off.
        "M2" => Criteria {
            suites: generated(M2_MIN_SCORED, RATE_DEN),
            probe_list: None,
            coverage: false,
            calc: Some(CalcRule::NoneOff),
            extent: false,
            function_rate: false,
            poi_subset: None,
        },
        "M7" => Criteria {
            suites: generated(M7_MIN_SCORED, RATE_NUM),
            probe_list: None,
            coverage: false,
            calc: None,
            extent: true,
            function_rate: false,
            poi_subset: None,
        },
        // gen-m8a and gen-m8b with none visibly off, and the poi subset; no function scope.
        "M8" => Criteria {
            suites: vec![
                SuiteNeed::named("gen-m8a", M8_MIN_SCORED, RATE_DEN),
                SuiteNeed::named("gen-m8b", M8_MIN_SCORED, RATE_DEN),
            ],
            probe_list: None,
            coverage: false,
            calc: None,
            extent: false,
            function_rate: false,
            poi_subset: Some(M8_SUBSET_RATE_NUM),
        },
        // No suite: gen-m9 alone is not a criterion, and catalog coverage is shown, not required.
        "M9" => Criteria {
            suites: Vec::new(),
            probe_list: None,
            coverage: false,
            calc: Some(CalcRule::Rate(M9_CALC_RATE_NUM)),
            extent: false,
            function_rate: true,
            poi_subset: None,
        },
        "M11" => Criteria {
            suites: vec![SuiteNeed::named(
                "corpus-bench",
                M11_MIN_SCORED,
                M11_RATE_NUM,
            )],
            probe_list: None,
            coverage: false,
            calc: None,
            extent: false,
            function_rate: false,
            poi_subset: None,
        },
        _ => Criteria {
            suites: generated(MIN_SCORED, RATE_NUM),
            probe_list: None,
            coverage: true,
            calc: Some(CalcRule::NoneOff),
            extent: false,
            function_rate: false,
            poi_subset: None,
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
    /// Cases that did not run to scoring, as `case!*`: M8's poi subset is not met while one holds
    /// a subset formula (decision 0146 item 2(b)).
    #[serde(default)]
    pub blocked_cases: Vec<String>,
}

/// M8's poi subset measure from the report's `poi_m8_subset` section, at `num` over
/// [`RATE_DEN`]; deviations are already out of its `scored`. `None` counts when poi did not run.
fn subset_exit(report: &RunReport, num: usize) -> SuiteExit {
    let mut out = SuiteExit {
        name: M8_SUBSET.to_string(),
        ..SuiteExit::default()
    };
    let Some(s) = report.poi_m8_subset.as_ref() else {
        return out;
    };
    let visible = s.visibly_off.min(s.scored);
    out.scored = Some(s.scored);
    out.visibly_off = Some(visible);
    out.rate15 = crate::v15_checkpoint::share(s.scored, visible);
    out.allowance = Some(allowance(s.scored, num));
    out.met = Some(s.blocked_cases.is_empty() && meets_rate(s.scored - visible, s.scored, num));
    out.visibly_off_cells.clone_from(&s.visibly_off_cells);
    out.blocked_cases.clone_from(&s.blocked_cases);
    out
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
    out.allowance = Some(allowance(scored, need.rate_num));
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
                .map(move |c| crate::runner::cell_id(&r.id, &c.key()))
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
    /// Scored cells of the row less its deviations and exit exceptions, for a milestone whose
    /// ironcalc-calc measure is a rate (M9, decision 0146 item 3); `None` for the others.
    #[serde(default)]
    pub calc_scored: Option<usize>,
    /// Visibly-off cells that rate allows: `floor(0.0005 * calc_scored)` for M9.
    #[serde(default)]
    pub calc_allowance: Option<usize>,
    /// Of those, the visibly-off cells (decision 0114).
    #[serde(default)]
    pub calc_visibly_off: Option<usize>,
    /// No visibly-off cell is left (at most `calc_allowance` for M9); `None` too for a
    /// milestone without an ironcalc-calc criterion (M7), whose counts are shown but not
    /// required.
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
    /// In-scope functions of the milestone in `v15_checkpoint` (M9, decision 0146 item 3);
    /// `None` for a milestone without the criterion or a run without them.
    #[serde(default)]
    pub rate_functions: Option<usize>,
    /// Of those, the functions not `meets_rate` (not covered or under the Phase 1 rate).
    #[serde(default)]
    pub below_rate: Option<usize>,
    /// No function is below the rate.
    #[serde(default)]
    pub rate_met: Option<bool>,
    /// The functions below the rate, by name, for `scripts/exit_owners.py`, which lists their
    /// `attributed_off` cells (decision 0142).
    #[serde(default)]
    pub below_rate_functions: Vec<String>,
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
            if let Some(c) = coverage.filter(|_| need.shows_scope()) {
                row.functions = Some(c.functions);
                row.covered = Some(c.covered);
                row.coverage_met = need.coverage.then_some(c.covered == c.functions);
            }
            let mut suites = need.suites.iter().map(|n| suite_exit(report, n));
            let first = suites.next().unwrap_or_default();
            row.more_suites = suites.collect();
            if let Some(num) = need.poi_subset {
                row.more_suites.push(subset_exit(report, num));
            }
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
            let scope = report
                .function_scope
                .as_ref()
                .filter(|_| need.shows_scope());
            if let Some(r) = scope.and_then(|s| s.rows.iter().find(|r| r.milestone == m)) {
                let failing = r.failing.len().saturating_sub(r.excepted.len());
                let visible = r
                    .visibly_off
                    .iter()
                    .filter(|c| !r.excepted.contains(c))
                    .count();
                row.calc_failing = Some(failing);
                row.calc_visibly_off = Some(visible);
                row.calc_met = match need.calc {
                    None => None,
                    Some(CalcRule::NoneOff) => Some(visible == 0),
                    Some(CalcRule::Rate(num)) => {
                        let scored = r.cells.saturating_sub(r.deviations + r.excepted.len());
                        row.calc_scored = Some(scored);
                        row.calc_allowance = Some(allowance(scored, num));
                        Some(meets_rate(scored.saturating_sub(visible), scored, num))
                    }
                };
            }
            if need.function_rate {
                let below: Vec<String> = report
                    .v15_checkpoint
                    .functions
                    .iter()
                    .filter(|f| f.milestone == m && !f.meets_rate)
                    .map(|f| f.name.clone())
                    .collect();
                let n = report
                    .v15_checkpoint
                    .functions
                    .iter()
                    .filter(|f| f.milestone == m)
                    .count();
                if n > 0 {
                    row.rate_functions = Some(n);
                    row.below_rate = Some(below.len());
                    row.rate_met = Some(below.is_empty());
                    row.below_rate_functions = below;
                }
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
                required(!need.suites.is_empty(), row.gen_met),
                required(need.calc.is_some(), row.calc_met),
                row.probes_met,
                required(need.extent, row.extent_met),
                required(need.function_rate, row.rate_met),
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
        Some(scored) => {
            let blocked = match s.blocked_cases.len() {
                0 => String::new(),
                n => format!(", {n} cases blocked"),
            };
            format!(
                "{scored}, {}, {}/{} [{}]{blocked} {}",
                s.rate15
                    .map_or_else(|| "n/a".to_string(), |x| format!("{x:.5}")),
                count(s.visibly_off),
                count(s.allowance),
                count(s.known_failures),
                verdict(s.met)
            )
        }
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
        "Thresholds, at visible parity (decision 0114): catalog coverage `covered` equals `functions`; gen suite at least {MIN_SCORED} scored cells (`cells - deviations`) with at least 0.999 of them not visibly off, so visibly-off cells at most the allowance `floor(0.001 * scored)`; ironcalc-calc has no visibly-off cell once exit exceptions are excluded; probes have no visibly-off cell. M2 (decision 0135): coverage is shown but not required, and gen-m2 needs {M2_MIN_SCORED} scored cells with none visibly off; its probes are the M0-M2 row. M7 (decision 0140): gen-m7 needs {M7_MIN_SCORED} scored cells at the 0.999 rate, gen-m7 and the M7 probes have no spill-extent failure (a cell the engine filled that Excel left empty, decision 0063), and coverage and ironcalc-calc are shown but not required. M1 (decision 0146): gen-m1 needs {M1_MIN_SCORED} scored cells and smoke any number, both with none visibly off, and its probes are the frozen `{M1_PROBE_LIST}` list; M11: corpus-bench needs {M11_MIN_SCORED} scored cells with at least 0.998 not visibly off, `n/a` when it was not fetched; neither has a coverage or ironcalc-calc criterion. M8 (decision 0146 item 2): gen-m8a and gen-m8b each need {M8_MIN_SCORED} scored cells with none visibly off, and the `{M8_SUBSET}` (the poi cells holding a defined name, structured, 3-D or external reference) needs at least 0.995 of its scored cells not visibly off and no crashed or errored case holding such a formula, `n/a` when poi did not run; no coverage or ironcalc-calc criterion. M9 (decision 0146 item 3): no suite is required; ironcalc-calc, less its exit exceptions and deviations, needs at least 0.9995 of its scored cells not visibly off (allowance `floor(0.0005 * scored)`); every in-scope M9 function must be covered and at the Phase 1 rate of the v1.5 checkpoint; coverage is shown, not required. A suite other than the milestone's gen suite is named in its column. Bracketed counts are the bit-level failures, Phase 1b's target. An exit measure, not a gate.\n"
    );
    let _ = writeln!(
        md,
        "| milestone | coverage (covered/functions) | gen suite (scored, 15-digit rate, visibly off/allowance [bit failures]) | ironcalc-calc (visibly off [bit failures], excl. exceptions) | probes (visibly off [bit failures]) | spill extents (failures) | functions at the Phase 1 rate (below/in scope) | exit |"
    );
    let _ = writeln!(md, "|---|---|---|---|---|---|---|---|");
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
        if r.gen_suite.is_empty() && r.more_suites.is_empty() {
            generated = "none required".to_string();
        } else if r.gen_scored.is_none() && r.more_suites.is_empty() {
            generated = "n/a".to_string();
        }
        // M9's ironcalc-calc is a rate: visibly off over the allowance, of the scored cells.
        let calc_off = match (r.calc_allowance, r.calc_scored) {
            (Some(allowance), Some(scored)) => {
                format!("{}/{allowance} of {scored}", count(r.calc_visibly_off))
            }
            _ => count(r.calc_visibly_off),
        };
        let at_rate = match r.rate_functions {
            Some(n) => format!("{}/{n} {}", count(r.below_rate), verdict(r.rate_met)),
            None => "n/a".to_string(),
        };
        let _ = writeln!(
            md,
            "| {} | {coverage} | {generated} | {calc_off} [{}] {} | {} [{}] {} | {} {} | {at_rate} | {} |",
            r.milestone,
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
mod tests;
