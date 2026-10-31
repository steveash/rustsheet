//! The fifteen-digit parity checkpoint of decision 0094 (T-0062): per in-scope catalog function,
//! its Excel cells, its Excel cells in generated suites where it is the formula's outer call, its
//! exemption and its visibly-off cells, and whether the run meets the checkpoint overall. It also
//! measures the Phase 1 exit of decision 0114: every function covered with at most one cell in a
//! thousand visibly off, its cells counted as decision 0118 counts them (its attributed cells, or
//! every Excel cell that calls it below the evidence floor). The stricter share, over every Excel
//! cell that calls the function with decision 0088's blame, is kept as information. The cells of
//! [`Phase1Exceptions`] are left out of the Phase 1 rate: its listed cells (decision 0134) and the
//! cells only open M14 tasks own (decision 0143). It is a report, never a gate; "both hold on two
//! consecutive steward runs" is the steward's to check.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use serde::{Deserialize, Serialize};

use crate::attribution::Attribution;
use crate::catalog::Catalog;
use crate::closeness::{sole_call, Closeness, Visible};
use crate::evidence_floor::EVIDENCE_FLOOR;
use crate::formula_calls::{functions_in, outer_call};
use crate::gate_b::{self, GateB};
use crate::milestone_exit::{meets_rate, RATE_DEN, RATE_NUM};
use crate::phase1_exceptions::{Phase1Exceptions, DESKTOP_OWNED_PATH, PHASE1_EXCEPTIONS_PATH};
use crate::report::{FunctionStat, RunReport};
use crate::runner::CaseResult;
use crate::value::Verdict;

/// Excel cells a function needs (decision 0094, item 1).
pub const CELLS_BAR: usize = 400;
/// Of those, cells in a generated suite with the function as the outer call.
pub const GENERATED_BAR: usize = 200;

/// One in-scope catalog function's standing against the bar.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct V15Function {
    /// Upper-case Excel name.
    pub name: String,
    /// The catalog milestone, `M<n>`: the M9 exit reads the rows of its own milestone
    /// (decision 0146 item 3).
    #[serde(default)]
    pub milestone: String,
    /// Excel-evidence cells that call it (the report's `evidence`).
    pub cells: usize,
    /// Excel-evidence cells of generated suites whose formula is one call of it.
    pub generated: usize,
    /// The catalog's `v15_reason` when the catalog exempts it from the cell counts.
    pub exempt: Option<String>,
    /// Visibly-off cells blamed on it (decision 0083).
    pub visible: usize,
    /// Enough cells: [`CELLS_BAR`] and [`GENERATED_BAR`], or [`EVIDENCE_FLOOR`] when exempt.
    pub covered: bool,
    /// Covered and no visibly-off cell.
    pub meets: bool,
    /// Which cells the Phase 1 rate counts (decision 0118): its attributed cells (`sole`), or
    /// every Excel cell that calls it (`all`) below [`EVIDENCE_FLOOR`] attributed cells.
    #[serde(default)]
    pub attribution: Attribution,
    /// The scored cells the Phase 1 rate counts, without the [`Self::rate_excepted`] and
    /// [`Self::desktop_owned`] cells.
    #[serde(default)]
    pub rate_cells: usize,
    /// Of those, the visibly-off cells.
    #[serde(default)]
    pub rate_visible: usize,
    /// Visibly-off cells of the attribution's count that the Phase 1 rate leaves out because
    /// [`Phase1Exceptions`] lists them (decision 0134); they are in neither `rate_cells` nor
    /// `rate_visible`.
    #[serde(default)]
    pub rate_excepted: usize,
    /// Visibly-off cells of the attribution's count that the Phase 1 rate leaves out because
    /// only open M14 tasks own them (decision 0143) and [`Self::rate_excepted`] does not hold
    /// them; they are in neither `rate_cells` nor `rate_visible`.
    #[serde(default)]
    pub desktop_owned: usize,
    /// Covered and at least 99.9% of `rate_cells` not visibly off: the Phase 1 exit (decisions
    /// 0114 and 0118).
    #[serde(default)]
    pub meets_rate: bool,
    /// Information only: the stricter count of decision 0118 item 5.
    #[serde(default)]
    pub strict: StrictRate,
}

/// The stricter Phase 1 count, which gates nothing (decision 0118 item 5): every Excel cell that
/// calls the function (`cells`), with every visibly-off cell blamed on it (`visible`).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StrictRate {
    /// Covered and at least 99.9% of `cells` not visibly off.
    pub meets_rate: bool,
}

/// Share of `cells` not visibly off, floored at 0; `None` without cells.
pub(crate) fn share(cells: usize, visible: usize) -> Option<f64> {
    // Cell counts stay far below 2^52, so the casts are exact.
    #[allow(clippy::cast_precision_loss)]
    let rate = (cells > 0).then(|| cells.saturating_sub(visible) as f64 / cells as f64);
    rate
}

impl V15Function {
    /// The Phase 1 share: of `rate_cells`, those not visibly off; `None` without cells.
    #[must_use]
    pub fn rate(&self) -> Option<f64> {
        share(self.rate_cells, self.rate_visible)
    }

    /// Information only: the share of its Excel cells not visibly off with every blamed cell
    /// counted; `None` without cells. Blamed cells can exceed `cells` (a cell blamed on it may
    /// come from a non-Excel suite), so the share is floored at 0.
    #[must_use]
    pub fn strict_rate(&self) -> Option<f64> {
        share(self.cells, self.visible)
    }
}

/// The checkpoint section of the report.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct V15Checkpoint {
    /// Every in-scope catalog function, worst first: most visibly-off cells, then fewest cells.
    pub functions: Vec<V15Function>,
    /// In-scope catalog functions.
    pub in_scope: usize,
    /// Of those, covered.
    pub covered: usize,
    /// In-scope cells visibly off in every suite of the run.
    pub visibly_off: usize,
    /// The suites the run scored, vendored and fetched, which `visibly_off` counts.
    pub suites: Vec<String>,
    /// At least one function, every one covered, and no visibly-off cell.
    pub met: bool,
    /// Functions that meet the Phase 1 rate (decision 0114).
    #[serde(default)]
    pub at_rate: usize,
    /// The Phase 1 rate holds: [`GateB::met`] (decision 0188 R1(b) counts decision 0114's rate
    /// on user-representative suites only; `at_rate` is the every-suite count, information).
    #[serde(default)]
    pub phase1_met: bool,
    /// Information only: functions that meet the strict rate ([`StrictRate::meets_rate`]).
    #[serde(default)]
    pub at_strict_rate: usize,
    /// The Phase 1 rate, `rate_num / rate_den` of a function's counted cells (decision 0114):
    /// [`RATE_NUM`] / [`RATE_DEN`], written out so report readers (the progress dashboard,
    /// T-0072) take the bar from the report instead of copying it.
    #[serde(default)]
    pub rate_num: usize,
    /// The denominator of [`Self::rate_num`].
    #[serde(default)]
    pub rate_den: usize,
    /// [`Self::verdict_line`], written out for report readers (T-0072).
    #[serde(default)]
    pub verdict_line: String,
    /// [`Self::phase1_line`], written out for report readers (T-0072).
    #[serde(default)]
    pub phase1_line: String,
    /// Listed cells the Phase 1 rate left out (decision 0134): visibly-off, non-deviation Excel
    /// cells of the run.
    #[serde(default)]
    pub excepted: usize,
    /// Listed cells that are not left out because the run did not score them, or they are not
    /// visibly off (`case!Sheet!A1`). A warning only: the line should go (decision 0134).
    #[serde(default)]
    pub stale_exceptions: Vec<String>,
    /// Listed cells whose suite has no case in this run (not fetched or not selected): neither
    /// stale nor confirmed, so the report does not tell anyone to remove their lines.
    #[serde(default)]
    pub unchecked_exceptions: Vec<String>,
    /// Cells the Phase 1 rate left out because only open M14 tasks own them (decision 0143):
    /// visibly-off, non-deviation Excel cells of the run that [`Self::excepted`] does not count.
    #[serde(default)]
    pub desktop_owned: usize,
    /// Gate B (decision 0188 R1(b)): the Phase 1 rate on user-representative suites only.
    #[serde(default)]
    pub gate_b: GateB,
}

impl V15Checkpoint {
    /// The overall line the steward copies into docs/status.md.
    #[must_use]
    pub fn verdict_line(&self) -> String {
        format!(
            "Fifteen-digit parity: {} (coverage {}/{}, visibly-off cells {})",
            if self.met { "met" } else { "not met" },
            self.covered,
            self.in_scope,
            self.visibly_off
        )
    }

    /// The Phase 1 rate line the steward copies beside [`Self::verdict_line`]: Gate B (decision
    /// 0188 R1(b)), with decision 0118's every-suite count beside it as information.
    #[must_use]
    pub fn phase1_line(&self) -> String {
        format!(
            "Phase 1 rate, {}; every suite, information: {}/{} (decision 0118)",
            self.gate_b.line(),
            self.at_rate,
            self.in_scope
        )
    }

    /// The stricter count of decision 0118 item 5, labelled as information.
    #[must_use]
    pub fn strict_line(&self) -> String {
        format!(
            "Information, gates nothing: {}/{} functions covered with ≥ 99.9% of every Excel cell that calls them within 15 digits, each visibly-off cell blamed on every non-glue function of its formula (decision 0088)",
            self.at_strict_rate, self.in_scope
        )
    }
}

/// A cell the Phase 1 rate leaves out, by the functions it counts toward.
pub(crate) struct LeftOut {
    /// The functions its formula calls.
    called: BTreeSet<String>,
    /// Its sole non-glue call ([`sole_call`]).
    sole: Option<String>,
    /// Left out as owned only by open M14 tasks (decision 0143), not as a listed cell (decision
    /// 0134).
    desktop: bool,
}

impl LeftOut {
    /// Whether the cell is in the count `attribution` measures `name` by (decision 0118).
    fn counts_toward(&self, name: &str, attribution: Attribution) -> bool {
        match attribution {
            Attribution::All => self.called.contains(name),
            Attribution::Sole => self.sole.as_deref() == Some(name),
        }
    }
}

/// How many of `left_out` count toward `name` measured by `attribution`: listed (decision 0134),
/// then desktop-owned (decision 0143).
pub(crate) fn left_out_counts(
    left_out: &[LeftOut],
    name: &str,
    attribution: Attribution,
) -> (usize, usize) {
    let counted: Vec<&LeftOut> = left_out
        .iter()
        .filter(|cell| cell.counts_toward(name, attribution))
        .collect();
    let desktop = counted.iter().filter(|cell| cell.desktop).count();
    (counted.len() - desktop, desktop)
}

/// One function's Phase 1 rate count (decision 0118): which cells it is measured by, those cells
/// less the left-out ones, and whether a covered function meets the rate on them.
pub(crate) struct RateCount {
    pub attribution: Attribution,
    pub rate_cells: usize,
    pub rate_visible: usize,
    /// Left out as listed (decision 0134).
    pub listed: usize,
    /// Left out as owned only by open M14 tasks (decision 0143).
    pub desktop: usize,
    pub meets_rate: bool,
}

/// [`RateCount`] of `name` from its counts `stat` (none: no cell) and the `left_out` cells.
pub(crate) fn rate_count(
    stat: Option<&FunctionStat>,
    left_out: &[LeftOut],
    name: &str,
    covered: bool,
) -> RateCount {
    let (attribution, counted) = stat.map_or((Attribution::All, Closeness::default()), |s| {
        let (a, c) = s.counted();
        (a, *c)
    });
    let (listed, desktop) = left_out_counts(left_out, name, attribution);
    let rate_cells = counted.scored().saturating_sub(listed + desktop);
    let rate_visible = counted.visible().saturating_sub(listed + desktop);
    RateCount {
        attribution,
        rate_cells,
        rate_visible,
        listed,
        desktop,
        meets_rate: covered
            && meets_rate(
                rate_cells.saturating_sub(rate_visible),
                rate_cells,
                RATE_NUM,
            ),
    }
}

/// The cells the Phase 1 rate leaves out: each visibly-off, non-deviation Excel cell of `results`
/// that `exceptions` lists or (decision 0143) says only M14 tasks own; and the listed cells that
/// are not such a cell, as `case!Sheet!A1`. A desktop-owned cell that is not one is not stale: the
/// generated list holds every known failure M14 owns, within 15 digits or not. A listed cell
/// whose suite (the first segment of its case id) has no case in `results` is returned as
/// unchecked instead: the run cannot tell whether it is stale. Only the suites `keep` accepts are
/// read (Gate B's, or every suite).
pub(crate) fn excepted_cells(
    results: &[CaseResult],
    exceptions: &Phase1Exceptions,
    keep: impl Fn(&str) -> bool,
) -> (Vec<LeftOut>, Vec<String>, Vec<String>) {
    let scored: BTreeSet<&str> = results.iter().map(|r| r.suite.as_str()).collect();
    let mut left_out = Vec::new();
    let mut found = BTreeSet::new();
    for r in results.iter().filter(|r| keep(&r.suite)) {
        for c in &r.cells {
            let key = c.key();
            let listed = exceptions.contains(&r.id, &key);
            if !(listed || exceptions.desktop_owns(&r.id, &key))
                || !c.from_excel
                || c.verdict == Verdict::Deviation
                || !Visible::is_off(c)
            {
                continue;
            }
            let called = c.formula.as_deref().map(functions_in).unwrap_or_default();
            let sole = sole_call(&called).cloned();
            left_out.push(LeftOut {
                called,
                sole,
                desktop: !listed,
            });
            if listed {
                found.insert((r.id.clone(), key));
            }
        }
    }
    let (mut stale, mut unchecked) = (Vec::new(), Vec::new());
    for cell in exceptions
        .cells
        .keys()
        .filter(|cell| !found.contains(*cell))
    {
        let id = crate::runner::cell_id(&cell.0, &cell.1);
        let suite = cell.0.split('/').next().unwrap_or_default();
        if scored.contains(suite) {
            stale.push(id);
        } else {
            unchecked.push(id);
        }
    }
    (left_out, stale, unchecked)
}

/// Scores `report` (already built from `results`); `generated` names the generated suites, and
/// `exceptions` the cells the Phase 1 rate leaves out (decision 0134).
#[must_use]
pub fn summarize(
    catalog: &Catalog,
    results: &[CaseResult],
    report: &RunReport,
    generated: &[&str],
    exceptions: &Phase1Exceptions,
) -> V15Checkpoint {
    let (left_out, stale_exceptions, unchecked_exceptions) =
        excepted_cells(results, exceptions, |_| true);
    let mut outer: BTreeMap<String, usize> = BTreeMap::new();
    for r in results
        .iter()
        .filter(|r| generated.contains(&r.suite.as_str()))
    {
        for c in r
            .cells
            .iter()
            .filter(|c| c.from_excel && c.verdict != Verdict::Deviation)
        {
            if let Some(name) = c.formula.as_deref().and_then(outer_call) {
                *outer.entry(name).or_default() += 1;
            }
        }
    }
    let cells: BTreeMap<&str, usize> = report
        .evidence
        .iter()
        .map(|e| (e.name.as_str(), e.cells))
        .collect();
    let stats: BTreeMap<&str, &FunctionStat> = report
        .functions
        .iter()
        .map(|f| (f.name.as_str(), f))
        .collect();
    let mut functions: Vec<V15Function> = catalog
        .in_scope_functions()
        .map(|f| {
            let name = f.name.as_str();
            let n = cells.get(name).copied().unwrap_or(0);
            let generated = outer.get(name).copied().unwrap_or(0);
            let exempt = catalog.v15_exemption(name).map(String::from);
            let covered = if exempt.is_some() {
                n >= EVIDENCE_FLOOR
            } else {
                n >= CELLS_BAR && generated >= GENERATED_BAR
            };
            let stat = stats.get(name).copied();
            let visible = stat.map_or(0, |s| s.visible.total());
            let rate = rate_count(stat, &left_out, name, covered);
            V15Function {
                name: f.name.clone(),
                milestone: f.milestone.clone(),
                cells: n,
                generated,
                exempt,
                visible,
                covered,
                meets: covered && visible == 0,
                attribution: rate.attribution,
                rate_cells: rate.rate_cells,
                rate_visible: rate.rate_visible,
                rate_excepted: rate.listed,
                desktop_owned: rate.desktop,
                meets_rate: rate.meets_rate,
                strict: StrictRate {
                    meets_rate: covered && meets_rate(n.saturating_sub(visible), n, RATE_NUM),
                },
            }
        })
        .collect();
    functions.sort_by(|a, b| {
        a.meets
            .cmp(&b.meets)
            .then_with(|| b.visible.cmp(&a.visible))
            .then_with(|| a.cells.cmp(&b.cells))
            .then_with(|| a.name.cmp(&b.name))
    });
    let covered = functions.iter().filter(|f| f.covered).count();
    let at_rate = functions.iter().filter(|f| f.meets_rate).count();
    let at_strict_rate = functions.iter().filter(|f| f.strict.meets_rate).count();
    let visibly_off = report.visibly_off_cells();
    let gate_b = gate_b::summarize(results, exceptions, &functions);
    let mut checkpoint = V15Checkpoint {
        in_scope: functions.len(),
        met: !functions.is_empty() && covered == functions.len() && visibly_off == 0,
        at_rate,
        phase1_met: gate_b.met,
        at_strict_rate,
        functions,
        covered,
        visibly_off,
        suites: report.suites.iter().map(|s| s.name.clone()).collect(),
        rate_num: RATE_NUM,
        rate_den: RATE_DEN,
        verdict_line: String::new(),
        phase1_line: String::new(),
        excepted: left_out.iter().filter(|c| !c.desktop).count(),
        stale_exceptions,
        unchecked_exceptions,
        desktop_owned: left_out.iter().filter(|c| c.desktop).count(),
        gate_b,
    };
    checkpoint.verdict_line = checkpoint.verdict_line();
    checkpoint.phase1_line = checkpoint.phase1_line();
    checkpoint
}

/// The cells left out of the Phase 1 rate (decisions 0134 and 0143), per function, and any stale
/// line.
fn write_exceptions(checkpoint: &V15Checkpoint, md: &mut String) {
    if checkpoint.desktop_owned > 0 {
        let _ = writeln!(
            md,
            "Left out of the Phase 1 rate as owned only by open M14 tasks (decision 0143, {DESKTOP_OWNED_PATH}): {} visibly-off cells ({}); they stay known failures.\n",
            checkpoint.desktop_owned,
            per_function(checkpoint, |f| f.desktop_owned)
        );
    }
    if checkpoint.excepted == 0
        && checkpoint.stale_exceptions.is_empty()
        && checkpoint.unchecked_exceptions.is_empty()
    {
        return;
    }
    let per_function = per_function(checkpoint, |f| f.rate_excepted);
    let _ = writeln!(
        md,
        "Left out of the Phase 1 rate as M14 desktop-only evidence (decision 0134, {}): {} visibly-off cells{}; they stay known failures.\n",
        PHASE1_EXCEPTIONS_PATH,
        checkpoint.excepted,
        if per_function.is_empty() {
            String::new()
        } else {
            format!(" ({per_function})")
        }
    );
    if !checkpoint.stale_exceptions.is_empty() {
        let _ = writeln!(
            md,
            "Warning: {} listed cells are not scored or not visibly off in this run; remove their lines:\n",
            checkpoint.stale_exceptions.len()
        );
        for cell in &checkpoint.stale_exceptions {
            let _ = writeln!(md, "- {cell}");
        }
        let _ = writeln!(md);
    }
    if !checkpoint.unchecked_exceptions.is_empty() {
        let _ = writeln!(
            md,
            "{} listed cells were not checked: their suite was skipped:\n",
            checkpoint.unchecked_exceptions.len()
        );
        for cell in &checkpoint.unchecked_exceptions {
            let _ = writeln!(md, "- {cell}");
        }
        let _ = writeln!(md);
    }
}

/// `NAME count` for each function whose `count` is not 0, comma-separated.
fn per_function(checkpoint: &V15Checkpoint, count: impl Fn(&V15Function) -> usize) -> String {
    checkpoint
        .functions
        .iter()
        .filter(|f| count(f) > 0)
        .map(|f| format!("{} {}", f.name, count(f)))
        .collect::<Vec<_>>()
        .join(", ")
}

/// The report section: the overall line, then the functions below the bar, worst first, then the
/// exemptions.
pub fn write_markdown(checkpoint: &V15Checkpoint, md: &mut String) {
    let _ = writeln!(md, "\n## Fifteen-digit parity checkpoint\n");
    let _ = writeln!(md, "**{}**\n", checkpoint.verdict_line());
    let _ = writeln!(md, "**{}**\n", checkpoint.phase1_line());
    gate_b::write_markdown(&checkpoint.gate_b, md);
    write_exceptions(checkpoint, md);
    let _ = writeln!(md, "{}.\n", checkpoint.strict_line());
    let _ = writeln!(
        md,
        "Decision 0094: every in-scope function has {CELLS_BAR} Excel cells, {GENERATED_BAR} of them in a generated suite where it is the formula's outer call ({EVIDENCE_FLOOR} cells for an exempt function), and no cell is visibly off. Visibly-off cells counted over: {}. The Phase 1 rate counts each function's attributed cells, the Excel cells whose formula calls it as the only function outside the glue list, or every Excel cell that calls it below {EVIDENCE_FLOOR} attributed cells (decision 0118).\n",
        checkpoint.suites.join(", ")
    );
    let below: Vec<&V15Function> = checkpoint.functions.iter().filter(|f| !f.meets).collect();
    let _ = writeln!(
        md,
        "Functions below the bar ({}), worst first:\n",
        below.len()
    );
    let _ = writeln!(
        md,
        "| function | Excel cells | generated outer-call cells | exempt | visibly off | Phase 1 cells | of them visibly off | M14-owned, left out | within 15 digits | Phase 1 (≥ 99.9%) | strict share (information) |"
    );
    let _ = writeln!(md, "|---|---|---|---|---|---|---|---|---|---|---|");
    let pct =
        |r: Option<f64>| r.map_or_else(|| "n/a".to_string(), |r| format!("{:.2}%", r * 100.0));
    for f in below {
        let _ = writeln!(
            md,
            "| {} | {} | {} | {} | {} | {} {} | {} | {} | {} | {} | {} |",
            f.name,
            f.cells,
            f.generated,
            if f.exempt.is_some() { "yes" } else { "" },
            f.visible,
            f.rate_cells,
            match f.attribution {
                Attribution::Sole => "attributed",
                Attribution::All => "all (fallback)",
            },
            f.rate_visible,
            f.desktop_owned,
            pct(f.rate()),
            if f.meets_rate { "met" } else { "not met" },
            pct(f.strict_rate())
        );
    }
    let _ = writeln!(md, "\nExemptions from the cell counts:\n");
    let _ = writeln!(md, "| function | reason |\n|---|---|");
    for f in &checkpoint.functions {
        if let Some(reason) = &f.exempt {
            let _ = writeln!(md, "| {} | {} |", f.name, reason);
        }
    }
}

#[cfg(test)]
mod tests;
