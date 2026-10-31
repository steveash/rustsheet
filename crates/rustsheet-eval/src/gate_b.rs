//! Gate B of decision 0188 R1(b): the Phase 1 rate of decision 0118 (each function's attributed
//! cells, the evidence-floor fallback, the cells decisions 0134 and 0143 leave out) counted on
//! user-representative evidence only: the gen-* suites, ironcalc-calc, corpus-bench, calipers,
//! poi, smoke and fixtures. Probes and probes-numeric, the fit-lab research sweeps among them,
//! never count here (their visibly-off numbers are Phase 1b evidence). A run that did not score
//! calipers, poi or corpus-bench measures nothing: Gate B is `n/a` there, never met. Coverage
//! stays decision 0094's, over every suite. A report, never a gate.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use serde::{Deserialize, Serialize};

use crate::closeness::sole_call;
use crate::formula_calls::functions_in;
use crate::milestone_exit::{meets_rate, RATE_NUM};
use crate::phase1_exceptions::Phase1Exceptions;
use crate::report::FunctionStat;
use crate::runner::CaseResult;
use crate::v15_checkpoint::{excepted_cells, left_out_counts, share, V15Function};
use crate::value::Verdict;

/// The user-representative suites besides the generated ones (decision 0188 R1(b)).
pub const SUITES: [&str; 6] = [
    "ironcalc-calc",
    "corpus-bench",
    "calipers",
    "poi",
    "smoke",
    "fixtures",
];
/// Every suite whose name starts with this counts too (the gen-* suites).
pub const GENERATED_PREFIX: &str = "gen-";
/// Without these (fetched) suites the run does not measure Gate B.
pub const REQUIRED: [&str; 3] = ["calipers", "poi", "corpus-bench"];

/// Whether Gate B counts the cells of `suite`.
#[must_use]
pub fn counts(suite: &str) -> bool {
    suite.starts_with(GENERATED_PREFIX) || SUITES.contains(&suite)
}

/// One in-scope function's Gate B count.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GateBFunction {
    /// Upper-case Excel name.
    pub name: String,
    /// The user-representative cells decision 0118 counts, less the left-out cells.
    pub rate_cells: usize,
    /// Of those, the visibly-off cells.
    pub rate_visible: usize,
    /// Covered (decision 0094, every suite) and at least 99.9% of `rate_cells` not visibly off.
    /// A function with no user-representative cell does not meet it.
    pub meets_rate: bool,
}

impl GateBFunction {
    /// The share of `rate_cells` not visibly off; `None` without cells.
    #[must_use]
    pub fn rate(&self) -> Option<f64> {
        share(self.rate_cells, self.rate_visible)
    }
}

/// Gate B's result for one run.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GateB {
    /// Every [`REQUIRED`] suite was scored.
    pub measured: bool,
    /// The [`REQUIRED`] suites the run did not score.
    pub missing: Vec<String>,
    /// The counted suites the run scored.
    pub suites: Vec<String>,
    /// In-scope catalog functions.
    pub in_scope: usize,
    /// Of those, the ones that meet the rate.
    pub at_rate: usize,
    /// Measured, at least one function, and every one meets the rate.
    pub met: bool,
    /// Every in-scope function, those below the rate first (most visibly-off cells first).
    pub functions: Vec<GateBFunction>,
}

impl GateB {
    /// `met`, `not met`, or `n/a` when the run did not measure it.
    #[must_use]
    pub fn verdict(&self) -> &'static str {
        match (self.measured, self.met) {
            (false, _) => "n/a",
            (true, true) => "met",
            (true, false) => "not met",
        }
    }

    /// The line the Phase 1 line and the report print.
    #[must_use]
    pub fn line(&self) -> String {
        let not_measured = if self.measured {
            String::new()
        } else {
            format!(
                ", not measured: the run did not score {}",
                self.missing.join(", ")
            )
        };
        format!(
            "Gate B (decision 0188 R1(b)): {} ({}/{} functions covered with ≥ 99.9% of their attributed cells within 15 digits on gen-*, {}{not_measured})",
            self.verdict(),
            self.at_rate,
            self.in_scope,
            SUITES.join(", ")
        )
    }
}

/// Gate B over `results`: `functions` are the all-suite rows (their names and coverage), and
/// `exceptions` the cells the rate leaves out (decisions 0134 and 0143).
#[must_use]
pub fn summarize(
    results: &[CaseResult],
    exceptions: &Phase1Exceptions,
    functions: &[V15Function],
) -> GateB {
    let mut stats: BTreeMap<String, FunctionStat> = BTreeMap::new();
    let scored: BTreeSet<&str> = results.iter().map(|r| r.suite.as_str()).collect();
    for r in results.iter().filter(|r| counts(&r.suite)) {
        // A by-design value says nothing about how well a function matches Excel
        // (`RunReport::build`, whose per-function counts these are, restricted to the suites).
        for c in r.cells.iter().filter(|c| c.verdict != Verdict::Deviation) {
            let called = c.formula.as_deref().map(functions_in).unwrap_or_default();
            let sole = sole_call(&called).cloned();
            for f in called {
                let is_sole = sole.as_ref() == Some(&f);
                stats
                    .entry(f.clone())
                    .or_insert_with(|| FunctionStat {
                        name: f,
                        ..FunctionStat::default()
                    })
                    .add_attributed(c, is_sole);
            }
        }
    }
    let (left_out, _, _) = excepted_cells(results, exceptions, counts);
    let mut rows: Vec<GateBFunction> = functions
        .iter()
        .map(|f| {
            let (attribution, counted) =
                stats
                    .get(&f.name)
                    .map_or((crate::attribution::Attribution::All, None), |s| {
                        let (a, c) = s.counted();
                        (a, Some(c))
                    });
            let (listed, desktop) = left_out_counts(&left_out, &f.name, attribution);
            let (scored_cells, visible) = counted.map_or((0, 0), |c| (c.scored(), c.visible()));
            let rate_cells = scored_cells.saturating_sub(listed + desktop);
            let rate_visible = visible.saturating_sub(listed + desktop);
            GateBFunction {
                name: f.name.clone(),
                rate_cells,
                rate_visible,
                meets_rate: f.covered
                    && meets_rate(
                        rate_cells.saturating_sub(rate_visible),
                        rate_cells,
                        RATE_NUM,
                    ),
            }
        })
        .collect();
    rows.sort_by(|a, b| {
        a.meets_rate
            .cmp(&b.meets_rate)
            .then_with(|| b.rate_visible.cmp(&a.rate_visible))
            .then_with(|| a.name.cmp(&b.name))
    });
    let missing: Vec<String> = REQUIRED
        .iter()
        .filter(|s| !scored.contains(**s))
        .map(ToString::to_string)
        .collect();
    let at_rate = rows.iter().filter(|f| f.meets_rate).count();
    let measured = missing.is_empty();
    GateB {
        measured,
        missing,
        suites: scored
            .into_iter()
            .filter(|s| counts(s))
            .map(String::from)
            .collect(),
        in_scope: rows.len(),
        at_rate,
        met: measured && !rows.is_empty() && at_rate == rows.len(),
        functions: rows,
    }
}

/// Functions shown below the rate in the report; the JSON keeps every one.
const SHOWN: usize = 30;

/// The report lines under the Phase 1 line: the worst functions below Gate B's rate.
pub fn write_markdown(gate: &GateB, md: &mut String) {
    let below: Vec<&GateBFunction> = gate.functions.iter().filter(|f| !f.meets_rate).collect();
    if below.is_empty() {
        return;
    }
    let _ = writeln!(
        md,
        "Below Gate B's rate on the counted suites ({} functions; the {} with the most visibly-off cells): {}.\n",
        below.len(),
        below.len().min(SHOWN),
        below
            .iter()
            .take(SHOWN)
            .map(|f| format!("{} {}/{}", f.name, f.rate_visible, f.rate_cells))
            .collect::<Vec<_>>()
            .join(", ")
    );
}

#[cfg(test)]
mod tests;
