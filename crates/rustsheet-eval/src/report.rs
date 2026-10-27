//! Aggregation of case results into the scoreboard and the gap report.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use crate::argument_coverage::{self, FunctionArguments};
use crate::attribution::{self, Attribution, Tier, Workbooks};
use crate::catalog::Catalog;
use crate::catalog_coverage::{self, MilestoneCoverage};
use crate::closeness::{self, Closeness, Visible};
use crate::evidence_floor::{self, below_floor, BelowFloor, EVIDENCE_FLOOR};
use crate::formula_calls::{calls_only_in_scope, functions_in};
use crate::function_scope::{self, FunctionScope};
use crate::milestone_lists::{self, ListSummary};
use crate::out_of_scope::{self, OutOfScope};
use crate::probe_milestones::{self, MilestoneRow};
use crate::run_meta::{self, RunMeta};
use crate::runner::{CaseResult, CaseStatus, CellResult};
use crate::text_expect;
use crate::v15_checkpoint::{self, V15Checkpoint};
use crate::value::Verdict;
use crate::visible_parity;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SuiteSummary {
    pub name: String,
    pub cases: usize,
    pub statuses: BTreeMap<CaseStatus, usize>,
    pub cells: usize,
    pub cells_passed: usize,
    /// Cells that gave their pinned by-design value, outside the bit-exact denominator
    /// (decision 0036).
    #[serde(default)]
    pub deviations: usize,
    pub verdicts: BTreeMap<Verdict, usize>,
    /// Listed deviation cells that match Excel: their `deviations` lines are stale.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stale_deviations: Vec<String>,
    /// Cells decision 0042 passed (excel-graph 0 against a kept subnormal), as `case<TAB>cell`;
    /// each has a FALSE `(x)=0` companion in its case (T-1142).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub graph_subnormal_passes: Vec<String>,
    /// Total wall time of the suite's cases (informational; never part of the gate).
    #[serde(default)]
    pub millis: u64,
    /// Cells whose displayed text was scored, and those that matched (T-1610; never part of the
    /// gate).
    #[serde(default)]
    pub text_cells: usize,
    /// Scored text cells whose text matched Excel's.
    #[serde(default)]
    pub text_passed: usize,
    /// Formula cells left unscored because they call an add-in function (`_xll.`, decision 0027;
    /// T-0462): they are in none of the counts above.
    #[serde(default)]
    pub skipped_add_in: usize,
    /// The suite's in-scope scored cells per closeness bucket: cells that call no out-of-scope
    /// catalog function (decision 0027). Its visibly-off cells are what the visible-parity
    /// ratchet counts (T-0061, `visible_parity`).
    #[serde(default)]
    pub closeness: Closeness,
    /// The cases with a scored cell by their worst `closeness` cell (T-0068).
    #[serde(default)]
    pub workbooks: Workbooks,
}

/// Number of slowest cases kept in the report.
const SLOWEST_CASES: usize = 10;

/// A case id with its wall time, for the report's slowest-cases table.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SlowCase {
    /// `suite/case` id.
    pub id: String,
    /// Wall time of the case in milliseconds.
    pub millis: u64,
}

/// The `n` slowest cases, slowest first; ties are broken by case id so output is deterministic.
fn slowest(results: &[CaseResult], n: usize) -> Vec<SlowCase> {
    let mut all: Vec<SlowCase> = results
        .iter()
        .map(|r| SlowCase {
            id: r.id.clone(),
            millis: r.millis,
        })
        .collect();
    all.sort_by(|a, b| b.millis.cmp(&a.millis).then_with(|| a.id.cmp(&b.id)));
    all.truncate(n);
    all
}

impl SuiteSummary {
    /// Bit-exact rate over the scored cells that are not accepted deviations (decision 0036).
    #[must_use]
    pub fn cell_pass_rate(&self) -> f64 {
        self.rate(self.cells_passed)
    }

    /// Rate of the same scored cells that are bit-exact or within 1e-9 relative (`NumericUlp`,
    /// `NumericClose`): the comparator xlsx-corpus-bench's leaderboard uses, reported beside our
    /// bit-exact score so corpus-bench can be read against it (T-0427). Near misses are counted over
    /// all cells, so a stale `deviations` line that is now a near miss counts too; `rate` caps it.
    #[must_use]
    pub fn cell_close_rate(&self) -> f64 {
        let near = |k| self.verdicts.get(&k).copied().unwrap_or(0);
        self.rate(self.cells_passed + near(Verdict::NumericUlp) + near(Verdict::NumericClose))
    }

    /// `count` over the scored cells (all but accepted deviations), at most 1.
    pub(crate) fn rate(&self, count: usize) -> f64 {
        let scored = self.cells.saturating_sub(self.deviations);
        if scored == 0 {
            return 0.0;
        }
        // Cell counts stay far below 2^52, so the casts are exact.
        #[allow(clippy::cast_precision_loss)]
        let rate = count.min(scored) as f64 / scored as f64;
        rate
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FunctionStat {
    pub name: String,
    pub cells: usize,
    pub passed: usize,
    pub verdicts: BTreeMap<Verdict, usize>,
    /// The scored cells per closeness bucket, which the parity page reads (T-2036).
    #[serde(default)]
    pub closeness: Closeness,
    /// The visibly-off cells blamed on this function rather than on the glue around it
    /// ([`closeness::blamed`], T-0061).
    #[serde(default)]
    pub visible: Visible,
    /// The non-deviation Excel-evidence cells that call this function (decision 0118).
    #[serde(default)]
    pub excel: Closeness,
    /// Of `excel`, the cells whose sole non-glue call it is ([`closeness::sole_call`]).
    #[serde(default)]
    pub attributed: Closeness,
    /// The worst-cell parity tier of `attributed`, or of `excel` below the evidence floor.
    #[serde(default)]
    pub tier: Tier,
    /// Which of the two placed `tier`.
    #[serde(default)]
    pub attribution: Attribution,
    /// Formula cells of the Excel-saved corpora ([`REAL_WORLD_SUITES`]) that call this function,
    /// deviations included: how often real workbooks use it (T-0079).
    #[serde(default)]
    pub real_world_calls: usize,
}

/// The suites of Excel-saved real-world workbooks whose formula cells weight functions by usage
/// (T-0079, SPEC-0022).
pub const REAL_WORLD_SUITES: [&str; 2] = ["calipers", "poi"];

/// Counts into `functions` the formula cells of the [`REAL_WORLD_SUITES`] that call each function
/// (T-0079); a function only such cells call gets a stat of its own.
fn add_real_world_calls(results: &[CaseResult], functions: &mut BTreeMap<String, FunctionStat>) {
    let corpora = results
        .iter()
        .filter(|r| REAL_WORLD_SUITES.contains(&r.suite.as_str()));
    for formula in corpora.flat_map(|r| r.cells.iter().filter_map(|c| c.formula.as_deref())) {
        for f in functions_in(formula) {
            functions
                .entry(f.clone())
                .or_insert_with(|| FunctionStat {
                    name: f,
                    ..FunctionStat::default()
                })
                .real_world_calls += 1;
        }
    }
}

/// Scored cells with Excel-sourced expectations that call one function.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FunctionEvidence {
    pub name: String,
    /// Cells checked against Excel values (passing or not).
    pub cells: usize,
    /// Suites those cells come from.
    pub suites: BTreeSet<String>,
}

/// The report's lists that come from comparing the run with the catalog.
struct CatalogLists {
    unevidenced: Vec<String>,
    catalog_unevidenced: Vec<String>,
    needs_other_evidence: Vec<String>,
    out_of_scope: Vec<OutOfScope>,
}

impl CatalogLists {
    /// `evidence`: in-scope functions with Excel evidence; `out`: stats of out-of-scope ones.
    fn build(
        catalog: &Catalog,
        implemented: &[String],
        evidence: &BTreeMap<String, FunctionEvidence>,
        out: &BTreeMap<String, FunctionStat>,
    ) -> CatalogLists {
        let lacks_evidence = |f: &str| {
            !evidence.contains_key(f) && catalog.in_scope(f) && !catalog.needs_other_evidence(f)
        };
        let unevidenced = implemented
            .iter()
            .filter(|f| lacks_evidence(f))
            .cloned()
            .collect();
        let catalog_unevidenced = catalog
            .functions()
            .map(|f| &f.name)
            .filter(|f| lacks_evidence(f))
            .cloned()
            .collect();
        let needs_other_evidence = catalog
            .functions()
            .map(|f| &f.name)
            .filter(|f| catalog.needs_other_evidence(f))
            .cloned()
            .collect();
        let out_of_scope = catalog
            .functions()
            .filter(|f| !catalog.in_scope(&f.name))
            .map(|f| {
                let stat = out.get(&f.name);
                OutOfScope {
                    name: f.name.clone(),
                    reason: f.reason.clone().unwrap_or_default(),
                    cells: stat.map_or(0, |s| s.cells),
                    failing: stat.map_or(0, |s| s.cells.saturating_sub(s.passed)),
                }
            })
            .collect();
        CatalogLists {
            unevidenced,
            catalog_unevidenced,
            needs_other_evidence,
            out_of_scope,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RunReport {
    pub engine: String,
    pub suites: Vec<SuiteSummary>,
    /// In-scope functions only (decision 0027), sorted by number of failing cells, descending.
    pub functions: Vec<FunctionStat>,
    /// Cases with only their *failing* cells retained.
    pub cases: Vec<CaseResult>,
    /// Slowest cases of the run (informational; never part of the gate).
    #[serde(default)]
    pub slowest: Vec<SlowCase>,
    /// Excel evidence per in-scope function, fewest cells first (informational).
    #[serde(default)]
    pub evidence: Vec<FunctionEvidence>,
    /// Implemented in-scope functions that no Excel-evidence cell calls.
    #[serde(default)]
    pub unevidenced: Vec<String>,
    /// In-scope catalog functions, implemented or not, that no Excel-evidence cell calls (the
    /// oracle can evaluate them). Includes the catalog-listed entries of `unevidenced`.
    #[serde(default)]
    pub catalog_unevidenced: Vec<String>,
    /// In-scope catalog functions the oracle cannot evaluate (`oracle: none`): their evidence
    /// must come from another source.
    #[serde(default)]
    pub needs_other_evidence: Vec<String>,
    /// In-scope catalog functions the oracle can evaluate with fewer than [`EVIDENCE_FLOOR`]
    /// Excel cells, by category (T-0056).
    #[serde(default)]
    pub below_floor: Vec<BelowFloor>,
    /// Out-of-scope catalog functions, kept out of every parity count above.
    #[serde(default)]
    pub out_of_scope: Vec<OutOfScope>,
    /// How each frozen milestone probe list scored (decisions 0043, 0044); filled by the caller.
    #[serde(default)]
    pub milestone_lists: Vec<ListSummary>,
    /// How the `probes` cases assigned to each milestone scored (decision 0051); filled by caller.
    #[serde(default)]
    pub probe_milestones: Vec<MilestoneRow>,
    /// How the M0–M2-only `ironcalc-calc` cells scored (M2 exit); `None` if the suite did not run.
    #[serde(default)]
    pub function_scope: Option<FunctionScope>,
    /// Catalog coverage per function milestone (T-2112); filled by the caller, which knows the
    /// suites' origin and milestone tags.
    #[serde(default)]
    pub catalog_coverage: Vec<MilestoneCoverage>,
    /// The exit measures of decision 0085 per milestone (T-2177); derived from the rows above.
    #[serde(default)]
    pub milestone_exit: Vec<crate::milestone_exit::ExitRow>,
    /// The fifteen-digit parity checkpoint of decision 0094 (T-0062); filled by the caller, which
    /// knows which suites are generated.
    #[serde(default)]
    pub v15_checkpoint: V15Checkpoint,
    /// In-scope catalog functions per parity tier ([`attribution::tier_counts`], T-0068).
    #[serde(default)]
    pub tiers: Vec<attribution::TierCount>,
    /// [`REAL_WORLD_SUITES`], whose formula cells `functions[].real_world_calls` counts (T-0079).
    #[serde(default)]
    pub real_world_suites: Vec<String>,
    /// Argument classes per in-scope function and catalog parameter (T-0077).
    #[serde(default)]
    pub argument_coverage: Vec<FunctionArguments>,
    /// Visibly-off, in-scope Excel cells attributed to no function (decision 0118 item 6, T-0082),
    /// in case order. Deviation cells are not among them.
    #[serde(default)]
    pub unattributed_off: Vec<attribution::UnattributedCell>,
    /// Commit, date, dirty flag and skipped suites ([`RunMeta`], T-0080); filled by the caller.
    #[serde(default)]
    pub run: RunMeta,
}

/// `r` with only its failing cells and text checks, and no argument classes: the form
/// report.json keeps cases in.
fn failing_only(r: &CaseResult) -> CaseResult {
    let mut r = r.clone();
    r.cells.retain(|c| c.verdict.is_failure());
    r.text.retain(|t| !t.pass);
    r.arguments.clear();
    r
}

impl RunReport {
    /// Aggregates `results`; `implemented` lists the engine's functions and `catalog` every
    /// Excel function, so that functions without any Excel evidence show up too. Functions the
    /// catalog puts out of scope are kept out of the parity counts and listed on their own.
    #[must_use]
    pub fn build(
        engine: &str,
        results: &[CaseResult],
        implemented: &[String],
        catalog: &Catalog,
    ) -> RunReport {
        let mut suites: BTreeMap<String, SuiteSummary> = BTreeMap::new();
        let mut functions: BTreeMap<String, FunctionStat> = BTreeMap::new();
        let mut evidence: BTreeMap<String, FunctionEvidence> = BTreeMap::new();
        for r in results {
            let s = suites
                .entry(r.suite.clone())
                .or_insert_with(|| SuiteSummary {
                    name: r.suite.clone(),
                    ..SuiteSummary::default()
                });
            s.cases += 1;
            *s.statuses.entry(r.status).or_default() += 1;
            s.cells += r.checked;
            s.cells_passed += r.passed;
            s.skipped_add_in += r.skipped_add_in;
            s.deviations += r.deviations;
            s.millis += r.millis;
            s.text_cells += r.text.len();
            s.text_passed += r.text.iter().filter(|t| t.pass).count();
            let mut book = Closeness::default();
            for c in &r.cells {
                *s.verdicts.entry(c.verdict).or_default() += 1;
                if c.listed_deviation && c.verdict.is_pass() {
                    s.stale_deviations.push(format!("{}\t{}", r.id, c.key()));
                }
                if c.graph_subnormal {
                    s.graph_subnormal_passes
                        .push(format!("{}\t{}", r.id, c.key()));
                }
                let called = c.formula.as_deref().map(functions_in).unwrap_or_default();
                if calls_only_in_scope(catalog, &called) {
                    s.closeness.add(c);
                    book.add(c);
                }
                // A by-design value says nothing about how well a function matches Excel.
                if c.verdict == Verdict::Deviation {
                    continue;
                }
                let visible = Visible::of(c);
                if visible.total() > 0 {
                    for f in closeness::blamed(&called) {
                        let st = functions.entry(f.clone()).or_insert_with(|| FunctionStat {
                            name: f.clone(),
                            ..FunctionStat::default()
                        });
                        st.visible.merge(visible);
                    }
                }
                let sole = closeness::sole_call(&called).cloned();
                for f in called {
                    let is_sole = sole.as_ref() == Some(&f);
                    if c.from_excel {
                        let ev = evidence
                            .entry(f.clone())
                            .or_insert_with(|| FunctionEvidence {
                                name: f.clone(),
                                ..FunctionEvidence::default()
                            });
                        ev.cells += 1;
                        ev.suites.insert(r.suite.clone());
                    }
                    let st = functions.entry(f.clone()).or_insert_with(|| FunctionStat {
                        name: f,
                        ..FunctionStat::default()
                    });
                    st.cells += 1;
                    if c.verdict.is_pass() {
                        st.passed += 1;
                    }
                    *st.verdicts.entry(c.verdict).or_default() += 1;
                    st.closeness.add(c);
                    st.add_attributed(c, is_sole);
                }
            }
            s.workbooks.add(&book);
        }
        add_real_world_calls(results, &mut functions);
        functions.values_mut().for_each(FunctionStat::place_tier);
        let (mut functions, out): (Vec<FunctionStat>, Vec<FunctionStat>) = functions
            .into_values()
            .partition(|f| catalog.in_scope(&f.name));
        let out: BTreeMap<String, FunctionStat> =
            out.into_iter().map(|f| (f.name.clone(), f)).collect();
        evidence.retain(|name, _| catalog.in_scope(name));
        functions.sort_by(|a, b| {
            (b.cells - b.passed)
                .cmp(&(a.cells - a.passed))
                .then(a.name.cmp(&b.name))
        });
        let lists = CatalogLists::build(catalog, implemented, &evidence, &out);
        let below_floor = below_floor(catalog, &evidence);
        let mut evidence: Vec<FunctionEvidence> = evidence.into_values().collect();
        evidence.sort_by(|a, b| a.cells.cmp(&b.cells).then_with(|| a.name.cmp(&b.name)));
        RunReport {
            engine: engine.to_string(),
            suites: suites.into_values().collect(),
            tiers: attribution::tier_counts(catalog, &functions),
            functions,
            cases: results.iter().map(failing_only).collect(),
            slowest: slowest(results, SLOWEST_CASES),
            evidence,
            unevidenced: lists.unevidenced,
            catalog_unevidenced: lists.catalog_unevidenced,
            needs_other_evidence: lists.needs_other_evidence,
            below_floor,
            out_of_scope: lists.out_of_scope,
            function_scope: function_scope::summarize(catalog, results),
            argument_coverage: argument_coverage::summarize(catalog, results),
            real_world_suites: REAL_WORLD_SUITES.iter().map(ToString::to_string).collect(),
            unattributed_off: attribution::unattributed_off(catalog, results),
            // The caller fills the milestone sections, the checkpoint and `run`.
            ..RunReport::default()
        }
    }

    /// One line for docs/status.md: how many functions have enough Excel evidence.
    #[must_use]
    pub fn evidence_summary(&self) -> String {
        let thin = self
            .evidence
            .iter()
            .filter(|f| f.cells < EVIDENCE_FLOOR)
            .count();
        format!(
            "Excel evidence: {} functions with ≥{EVIDENCE_FLOOR} oracle/cached cells, {thin} with fewer, {} implemented with none, {} in-scope catalog functions with none",
            self.evidence.len() - thin,
            self.unevidenced.len(),
            self.catalog_unevidenced.len()
        )
    }

    fn write_evidence(&self, md: &mut String) {
        let _ = writeln!(md, "\n## Excel evidence per function\n");
        let _ = writeln!(
            md,
            "{}. Cells count when their expectations come from Excel (oracle sidecar or Excel-cached values), pass or fail; hand-written expectations do not count.\n",
            self.evidence_summary()
        );
        let lists = [
            ("Implemented, no Excel evidence", &self.unevidenced),
            (
                "In-scope catalog functions with no Excel evidence (gaps)",
                &self.catalog_unevidenced,
            ),
            (
                "Need an evidence source other than the oracle (`oracle: none`)",
                &self.needs_other_evidence,
            ),
        ];
        for (title, names) in lists {
            if !names.is_empty() {
                let _ = writeln!(md, "{title} ({}): {}\n", names.len(), names.join(", "));
            }
        }
        evidence_floor::write_markdown(md, &self.below_floor);
        let _ = writeln!(md);
        let _ = writeln!(
            md,
            "| function | Excel cells | suites | below {EVIDENCE_FLOOR} |"
        );
        let _ = writeln!(md, "|---|---|---|---|");
        for f in &self.evidence {
            let suites: Vec<&str> = f.suites.iter().map(String::as_str).collect();
            let flag = if f.cells < EVIDENCE_FLOOR { "yes" } else { "" };
            let _ = writeln!(
                md,
                "| {} | {} | {} | {flag} |",
                f.name,
                f.cells,
                suites.join(", ")
            );
        }
    }

    fn write_suites_table(&self, md: &mut String) {
        let _ = writeln!(md, "## Suites\n");
        let _ = writeln!(md, "| suite | cases | pass | fail | pending | crashes | errors | cells | deviations | cell pass % | 1e-9 % | ulp | close | mismatch | unsupported | type |");
        let _ = writeln!(
            md,
            "|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|"
        );
        for s in &self.suites {
            let st = |k: CaseStatus| s.statuses.get(&k).copied().unwrap_or(0);
            let v = |k: Verdict| s.verdicts.get(&k).copied().unwrap_or(0);
            let errors = st(CaseStatus::EngineError)
                + st(CaseStatus::Panic)
                + st(CaseStatus::Timeout)
                + st(CaseStatus::InvalidCase);
            let crashes =
                st(CaseStatus::EngineError) + st(CaseStatus::Panic) + st(CaseStatus::Timeout);
            let _ = writeln!(
                md,
                "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {:.2} | {:.2} | {} | {} | {} | {} | {} |",
                s.name,
                s.cases,
                st(CaseStatus::Pass),
                st(CaseStatus::Fail),
                st(CaseStatus::Pending),
                crashes,
                errors,
                s.cells,
                s.deviations,
                100.0 * s.cell_pass_rate(),
                100.0 * s.cell_close_rate(),
                v(Verdict::NumericUlp),
                v(Verdict::NumericClose),
                v(Verdict::NumericMismatch)
                    + v(Verdict::TextMismatch)
                    + v(Verdict::BoolMismatch)
                    + v(Verdict::ErrorMismatch),
                v(Verdict::Unsupported),
                v(Verdict::TypeMismatch),
            );
        }
        let stale: Vec<&String> = self
            .suites
            .iter()
            .flat_map(|s| &s.stale_deviations)
            .collect();
        if !stale.is_empty() {
            let _ = writeln!(
                md,
                "\nStale `deviations` lines (the cell now matches Excel; remove the line):\n"
            );
            for line in stale {
                let _ = writeln!(md, "- `{line}`");
            }
        }
        let kept: Vec<&String> = self
            .suites
            .iter()
            .flat_map(|s| &s.graph_subnormal_passes)
            .collect();
        if !kept.is_empty() {
            let _ = writeln!(
                md,
                "\nCells passed by decision 0042 (excel-graph 0 against a kept subnormal; each has a \
                 FALSE `(x)=0` companion in its case):\n"
            );
            for line in kept {
                let _ = writeln!(md, "- `{line}`");
            }
        }
    }

    /// In-scope cells visibly off from Excel in every suite of the run (T-0061).
    #[must_use]
    pub fn visibly_off_cells(&self) -> usize {
        self.suites.iter().map(|s| s.closeness.visible()).sum()
    }

    /// Human/agent-readable gap report.
    #[must_use]
    pub fn to_markdown(&self, examples_per_function: usize) -> String {
        let mut md = String::new();
        let _ = writeln!(
            md,
            "# RustSheet compatibility report (engine: `{}`)\n",
            self.engine
        );
        run_meta::write_markdown(&self.run, &mut md);
        self.write_suites_table(&mut md);
        crate::milestone_exit::write_markdown(&self.milestone_exit, &mut md);
        visible_parity::write_markdown(self, &mut md);
        v15_checkpoint::write_markdown(&self.v15_checkpoint, &mut md);
        attribution::write_markdown(self, &mut md);
        text_expect::write_markdown(&self.suites, &self.cases, &mut md);
        milestone_lists::write_markdown(&self.milestone_lists, &mut md);
        probe_milestones::write_markdown(&self.probe_milestones, &mut md);
        function_scope::write_markdown(self.function_scope.as_ref(), &mut md);
        run_meta::write_timing(self, &mut md);
        let broken: Vec<&CaseResult> = self
            .cases
            .iter()
            .filter(|c| {
                matches!(
                    c.status,
                    CaseStatus::EngineError
                        | CaseStatus::Panic
                        | CaseStatus::Timeout
                        | CaseStatus::InvalidCase
                )
            })
            .collect();
        if !broken.is_empty() {
            let _ = writeln!(md, "\n## Cases that did not run ({})\n", broken.len());
            for c in broken.iter().take(50) {
                let _ = writeln!(
                    md,
                    "- `{}` — {:?}: {}",
                    c.id,
                    c.status,
                    c.message.as_deref().unwrap_or("")
                );
            }
        }
        let failing: Vec<&FunctionStat> = self
            .functions
            .iter()
            .filter(|f| f.passed < f.cells)
            .collect();
        let _ = writeln!(
            md,
            "\n## Functions with failing cells ({})\n",
            failing.len()
        );
        let _ = writeln!(
            md,
            "| function | failing | checked | pass % | dominant verdict |"
        );
        let _ = writeln!(md, "|---|---|---|---|---|");
        for f in &failing {
            let dominant = f
                .verdicts
                .iter()
                .filter(|(k, _)| k.is_failure())
                .max_by_key(|(_, n)| **n)
                .map(|(k, _)| format!("{k:?}"))
                .unwrap_or_default();
            #[allow(clippy::cast_precision_loss)]
            let pct = 100.0 * f.passed as f64 / f.cells as f64;
            let _ = writeln!(
                md,
                "| {} | {} | {} | {pct:.1} | {dominant} |",
                f.name,
                f.cells - f.passed,
                f.cells
            );
        }
        let _ = writeln!(
            md,
            "\n## Examples (first {examples_per_function} failing cells per function)\n"
        );
        for f in failing.iter().take(40) {
            let _ = writeln!(md, "### {}\n", f.name);
            let examples = self.examples_for(&f.name, examples_per_function);
            for (case, cell) in examples {
                let _ = writeln!(
                    md,
                    "- `{}` `{}` `={}` → expected {} got {} ({:?})",
                    case,
                    cell.key(),
                    cell.formula.as_deref().unwrap_or("?"),
                    cell.expected,
                    cell.actual,
                    cell.verdict
                );
            }
            let _ = writeln!(md);
        }
        self.write_evidence(&mut md);
        catalog_coverage::write_markdown(&self.catalog_coverage, &mut md);
        out_of_scope::write_markdown(&self.out_of_scope, &self.suites, &mut md);
        md
    }

    fn examples_for(&self, function: &str, n: usize) -> Vec<(&str, &CellResult)> {
        let mut out = Vec::new();
        for case in &self.cases {
            for cell in &case.cells {
                if cell
                    .formula
                    .as_deref()
                    .is_some_and(|f| functions_in(f).contains(function))
                {
                    out.push((case.id.as_str(), cell));
                    if out.len() >= n {
                        return out;
                    }
                }
            }
        }
        out
    }
}

#[cfg(test)]
pub(crate) mod tests;
