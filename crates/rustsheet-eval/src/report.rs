//! Aggregation of case results into the scoreboard and the gap report.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::sync::LazyLock;

use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::catalog::Catalog;
use crate::evidence_floor::{self, below_floor, BelowFloor, EVIDENCE_FLOOR};
use crate::function_scope::{self, FunctionScope};
use crate::milestone_lists::{self, ListSummary};
use crate::probe_milestones::{self, MilestoneRow};
use crate::runner::{CaseResult, CaseStatus, CellResult};
use crate::text_expect;
use crate::value::Verdict;

static FUNCTION_CALL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(?:_xlfn\.|_xlws\.)?\b([A-Z][A-Z0-9]*(?:\.[A-Z0-9]+)*)\s*\(")
        .unwrap_or_else(|e| unreachable!("static regex: {e}"))
});
static STRING_LITERAL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#""(?:[^"]|"")*""#).unwrap_or_else(|e| unreachable!("static regex: {e}"))
});

/// Upper-cased function names called by a formula (string literals ignored).
#[must_use]
pub fn functions_in(formula: &str) -> BTreeSet<String> {
    let stripped = STRING_LITERAL.replace_all(formula, "\"\"");
    FUNCTION_CALL
        .captures_iter(&stripped)
        .filter_map(|c| c.get(1).map(|m| m.as_str().to_ascii_uppercase()))
        .collect()
}

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
    fn rate(&self, count: usize) -> f64 {
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

/// An out-of-scope catalog function (decision 0027) and the cells the run checked for it.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct OutOfScope {
    pub name: String,
    /// The catalog's reason for leaving it out.
    pub reason: String,
    /// Cells calling the function that the run checked (passing or not).
    pub cells: usize,
    /// Of those, the failing ones.
    pub failing: usize,
}

/// The report's lists that come from comparing the run with the catalog.
struct CatalogLists {
    unevidenced: Vec<String>,
    catalog_unevidenced: Vec<String>,
    needs_other_evidence: Vec<String>,
    out_of_scope: Vec<OutOfScope>,
}

impl CatalogLists {
    /// `evidence` holds the in-scope functions with Excel evidence, `out` the run's stats for
    /// out-of-scope functions.
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    /// Excel evidence per in-scope function, fewest cells first (informational; never part of
    /// the gate).
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
    /// How each frozen milestone probe list scored (decisions 0043, 0044); filled by the caller,
    /// which loads the lists.
    #[serde(default)]
    pub milestone_lists: Vec<ListSummary>,
    /// How the `probes` cases assigned to each milestone scored (decision 0051); filled by the
    /// caller, which loads the rules.
    #[serde(default)]
    pub probe_milestones: Vec<MilestoneRow>,
    /// How the M0–M2-only `ironcalc-calc` cells scored (the M2 exit criterion); `None` when the
    /// suite did not run.
    #[serde(default)]
    pub function_scope: Option<FunctionScope>,
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
            s.deviations += r.deviations;
            s.millis += r.millis;
            s.text_cells += r.text.len();
            s.text_passed += r.text.iter().filter(|t| t.pass).count();
            for c in &r.cells {
                *s.verdicts.entry(c.verdict).or_default() += 1;
                if c.listed_deviation && c.verdict.is_pass() {
                    s.stale_deviations.push(format!("{}\t{}", r.id, c.key()));
                }
                if c.graph_subnormal {
                    s.graph_subnormal_passes
                        .push(format!("{}\t{}", r.id, c.key()));
                }
                // A by-design value says nothing about how well a function matches Excel.
                if c.verdict == Verdict::Deviation {
                    continue;
                }
                for f in c.formula.as_deref().map(functions_in).unwrap_or_default() {
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
                }
            }
        }
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
        let cases = results
            .iter()
            .map(|r| {
                let mut r = r.clone();
                r.cells.retain(|c| c.verdict.is_failure());
                r.text.retain(|t| !t.pass);
                r
            })
            .collect();
        let lists = CatalogLists::build(catalog, implemented, &evidence, &out);
        let below_floor = below_floor(catalog, &evidence);
        let mut evidence: Vec<FunctionEvidence> = evidence.into_values().collect();
        evidence.sort_by(|a, b| a.cells.cmp(&b.cells).then_with(|| a.name.cmp(&b.name)));
        RunReport {
            engine: engine.to_string(),
            suites: suites.into_values().collect(),
            functions,
            cases,
            slowest: slowest(results, SLOWEST_CASES),
            evidence,
            unevidenced: lists.unevidenced,
            catalog_unevidenced: lists.catalog_unevidenced,
            needs_other_evidence: lists.needs_other_evidence,
            below_floor,
            out_of_scope: lists.out_of_scope,
            milestone_lists: Vec::new(),
            probe_milestones: Vec::new(),
            function_scope: function_scope::summarize(catalog, results),
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

    fn write_timing(&self, md: &mut String) {
        let _ = writeln!(md, "\n## Timing\n");
        let _ = writeln!(md, "| suite | total ms |\n|---|---|");
        for s in &self.suites {
            let _ = writeln!(md, "| {} | {} |", s.name, s.millis);
        }
        let _ = writeln!(md, "\nSlowest {} cases:\n", self.slowest.len());
        let _ = writeln!(md, "| case | ms |\n|---|---|");
        for c in &self.slowest {
            let _ = writeln!(md, "| {} | {} |", c.id, c.millis);
        }
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
        self.write_suites_table(&mut md);
        text_expect::write_markdown(&self.suites, &self.cases, &mut md);
        milestone_lists::write_markdown(&self.milestone_lists, &mut md);
        probe_milestones::write_markdown(&self.probe_milestones, &mut md);
        function_scope::write_markdown(self.function_scope.as_ref(), &mut md);
        self.write_timing(&mut md);
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
        self.write_out_of_scope(&mut md);
        md
    }

    fn write_out_of_scope(&self, md: &mut String) {
        if self.out_of_scope.is_empty() {
            return;
        }
        let _ = writeln!(
            md,
            "\n## Out of scope ({})\n\nCatalog functions whose results depend on systems outside the workbook (decision 0027). They are left out of every count above.\n",
            self.out_of_scope.len()
        );
        let _ = writeln!(md, "| function | cells checked | failing | reason |");
        let _ = writeln!(md, "|---|---|---|---|");
        for f in &self.out_of_scope {
            let _ = writeln!(
                md,
                "| {} | {} | {} | {} |",
                f.name,
                f.cells,
                f.failing,
                f.reason.replace('|', "\\|")
            );
        }
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
mod tests {
    use super::*;

    fn case(id: &str, millis: u64) -> CaseResult {
        CaseResult {
            id: id.to_string(),
            suite: id.split('/').next().unwrap_or(id).to_string(),
            status: CaseStatus::Pass,
            message: None,
            expected_source: String::new(),
            checked: 1,
            passed: 1,
            deviations: 0,
            skipped_volatile: 0,
            millis,
            tags: Vec::new(),
            cells: Vec::new(),
            text: Vec::new(),
        }
    }

    #[test]
    fn slowest_cases_are_ordered_by_time_then_id() {
        let results = [
            case("s/b", 5),
            case("s/a", 5),
            case("s/c", 9),
            case("s/d", 1),
        ];
        let ids: Vec<String> = slowest(&results, 3).into_iter().map(|c| c.id).collect();
        assert_eq!(ids, ["s/c", "s/a", "s/b"]);
    }

    fn cell(formula: &str, verdict: Verdict) -> CellResult {
        CellResult {
            sheet: "S".into(),
            cell: "A1".into(),
            formula: Some(formula.into()),
            anchor_formula: None,
            expected: crate::value::Value::Number(1.0),
            actual: crate::value::Value::Number(1.0),
            verdict,
            from_excel: false,
            listed_deviation: false,
            graph_subnormal: false,
        }
    }

    #[test]
    fn near_misses_count_toward_the_close_rate_only() {
        let mut r = case("corpus-bench/a", 1);
        r.cells = vec![
            cell("ABS(1)", Verdict::NumericUlp),
            cell("ABS(2)", Verdict::NumericClose),
            cell("ABS(3)", Verdict::NumericMismatch),
        ];
        (r.checked, r.passed) = (4, 1);
        let report = RunReport::build("e", &[r], &[], &Catalog::default());
        let s = &report.suites[0];
        assert!((s.cell_pass_rate() - 0.25).abs() < 1e-12);
        assert!((s.cell_close_rate() - 0.75).abs() < 1e-12);
    }

    #[test]
    fn deviations_leave_the_bit_exact_rate_and_stale_lines_are_listed() {
        let listed = |verdict| CellResult {
            listed_deviation: true,
            ..cell("ROWS(SEQUENCE(1048576))", verdict)
        };
        let mut r = case("probes/a", 1);
        r.cells = vec![
            cell("ABS(1)", Verdict::Pass),
            cell("ABS(2)", Verdict::NumericMismatch),
            listed(Verdict::Deviation),
            listed(Verdict::Pass),
        ];
        (r.checked, r.passed, r.deviations) = (4, 2, 1);
        let report = RunReport::build("e", &[r], &[], &Catalog::default());
        let s = &report.suites[0];
        assert_eq!((s.cells, s.deviations), (4, 1));
        assert!((s.cell_pass_rate() - 2.0 / 3.0).abs() < 1e-12);
        assert_eq!(s.stale_deviations, ["probes/a\tS!A1"]);
        // the by-design cell is neither a failing case cell nor a function statistic
        assert_eq!(report.cases[0].cells.len(), 1);
        let sequence = report.functions.iter().find(|f| f.name == "SEQUENCE");
        assert_eq!(sequence.map(|f| (f.cells, f.passed)), Some((1, 1)));
        let md = report.to_markdown(1);
        assert!(
            md.contains("| probes | 1 | 1 | 0 | 0 | 0 | 0 | 4 | 1 | 66.67 |"),
            "{md}"
        );
        assert!(md.contains("- `probes/a\tS!A1`"));
    }

    #[test]
    fn cells_passed_by_decision_0042_are_listed() {
        let mut r = case("probes/a", 1);
        r.cells = vec![
            CellResult {
                graph_subnormal: true,
                ..cell("3E-308-2.5E-308", Verdict::Pass)
            },
            cell("ABS(1)", Verdict::Pass),
        ];
        let report = RunReport::build("e", &[r], &[], &Catalog::default());
        assert_eq!(report.suites[0].graph_subnormal_passes, ["probes/a\tS!A1"]);
        assert!(report.to_markdown(1).contains("- `probes/a\tS!A1`"));
    }

    fn excel_cell(formula: &str, verdict: Verdict) -> CellResult {
        CellResult {
            from_excel: true,
            ..cell(formula, verdict)
        }
    }

    #[test]
    fn evidence_counts_only_excel_sourced_cells_and_lists_unevidenced_functions() {
        let mut oracle = case("probes/a", 1);
        oracle.expected_source = "excel-graph".into();
        oracle.cells = vec![
            excel_cell("SUM(A1:A2)", Verdict::Pass),
            excel_cell("SUM(A1)+ABS(1)", Verdict::NumericMismatch),
            // A hand-written cell beside a partly covering sidecar is not evidence.
            cell("SUM(A3)+IF(TRUE,1,0)", Verdict::Pass),
        ];
        let mut cached = case("poi/b", 1);
        cached.expected_source = "excel-cached (Microsoft Excel)".into();
        cached.cells = vec![
            excel_cell("_xlfn.XLOOKUP(1,A:A,B:B)", Verdict::Pass),
            // Non-Excel cached values (e.g. LibreOffice) do not count.
            cell("TRIM(\"a\")", Verdict::Pass),
        ];
        let mut hand = case("smoke/c", 1);
        hand.expected_source = "hand".into();
        hand.cells = vec![cell("IF(TRUE,SUM(1),0)", Verdict::Pass)];
        let implemented = ["ABS", "IF", "SUM", "TRIM", "XLOOKUP"].map(String::from);
        let report = RunReport::build(
            "e",
            &[oracle, cached, hand],
            &implemented,
            &Catalog::default(),
        );
        let got: Vec<(&str, usize, Vec<&str>)> = report
            .evidence
            .iter()
            .map(|f| {
                (
                    f.name.as_str(),
                    f.cells,
                    f.suites.iter().map(String::as_str).collect(),
                )
            })
            .collect();
        assert_eq!(
            got,
            [
                ("ABS", 1, vec!["probes"]),
                ("XLOOKUP", 1, vec!["poi"]),
                ("SUM", 2, vec!["probes"])
            ]
        );
        assert_eq!(report.unevidenced, ["IF", "TRIM"]);
        let md = report.to_markdown(1);
        assert!(md.contains("Excel evidence: 0 functions with ≥20 oracle/cached cells, 3 with fewer, 2 implemented with none, 0 in-scope catalog functions with none"));
        assert!(md.contains("Implemented, no Excel evidence (2): IF, TRIM"));
        assert!(md.contains("| SUM | 2 | probes | yes |"));
    }

    #[test]
    fn out_of_scope_functions_leave_parity_counts_and_list_with_their_reason() {
        use crate::catalog::{CatalogFunction, OracleSupport, Scope};
        let entry = |name: &str, scope, reason: Option<&str>, oracle| CatalogFunction {
            name: name.into(),
            category: "math".into(),
            milestone: "M5".into(),
            scope,
            reason: reason.map(String::from),
            oracle,
        };
        let catalog = Catalog::from_functions([
            entry("ABS", Scope::In, None, None),
            entry("COS", Scope::In, None, None),
            entry("INFO", Scope::In, None, Some(OracleSupport::Unavailable)),
            entry("RTD", Scope::Out, Some("real-time server"), None),
            entry("WEBSERVICE", Scope::Out, Some("web | service"), None),
        ]);
        let mut probe = case("probes/a", 1);
        probe.cells = vec![
            excel_cell("ABS(1)", Verdict::NumericMismatch),
            excel_cell("WEBSERVICE(\"x\")", Verdict::ErrorMismatch),
        ];
        let implemented = ["ABS", "INFO", "SUM", "WEBSERVICE"].map(String::from);
        let report = RunReport::build("e", &[probe], &implemented, &catalog);
        let parity: Vec<&str> = report.functions.iter().map(|f| f.name.as_str()).collect();
        assert_eq!(parity, ["ABS"]);
        let evidenced: Vec<&str> = report.evidence.iter().map(|f| f.name.as_str()).collect();
        assert_eq!(evidenced, ["ABS"]);
        // SUM is not in this catalog, so it stays in scope; INFO needs another source.
        assert_eq!(report.unevidenced, ["SUM"]);
        assert_eq!(report.catalog_unevidenced, ["COS"]);
        assert_eq!(report.needs_other_evidence, ["INFO"]);
        let out: Vec<(&str, usize, usize)> = report
            .out_of_scope
            .iter()
            .map(|f| (f.name.as_str(), f.cells, f.failing))
            .collect();
        assert_eq!(out, [("RTD", 0, 0), ("WEBSERVICE", 1, 1)]);
        let md = report.to_markdown(1);
        assert!(md.contains("## Functions with failing cells (1)"), "{md}");
        assert!(md.contains("In-scope catalog functions with no Excel evidence (gaps) (1): COS"));
        assert!(md.contains("(`oracle: none`) (1): INFO"));
        assert!(md.contains("## Out of scope (2)"));
        assert!(
            md.contains("| WEBSERVICE | 1 | 1 | web \\| service |"),
            "{md}"
        );
        assert!(!md.contains("### WEBSERVICE"));
    }

    #[test]
    fn report_sums_suite_time_and_renders_timing() {
        let report = RunReport::build(
            "e",
            &[case("s/a", 5), case("s/b", 7), case("t/c", 1)],
            &[],
            &Catalog::default(),
        );
        assert_eq!(report.suites[0].millis, 12);
        let md = report.to_markdown(1);
        assert!(
            md.contains("## Timing") && md.contains("| s/b | 7 |"),
            "{md}"
        );
    }

    #[test]
    fn timing_does_not_change_the_gate() {
        use crate::baseline::{diff, Baseline};
        use crate::suite::BaselineMode;
        let fast = [case("s/a", 1), case("s/b", 2)];
        let slow = [case("s/a", 900), case("s/b", 5000)];
        let base = |r: &[CaseResult]| {
            let refs: Vec<&CaseResult> = r.iter().collect();
            Baseline::from_results(BaselineMode::Cells, &refs)
        };
        assert_eq!(base(&fast), base(&slow));
        assert!(!diff(&base(&fast), &base(&slow)).is_dirty());
    }

    #[test]
    fn extracts_function_names() {
        let f =
            functions_in(r#"IF(_xlfn.XLOOKUP(A1,B:B,C:C)>0,SUM(A1:A3),"NOW()")+ERF.PRECISE(1)"#);
        let v: Vec<&str> = f.iter().map(String::as_str).collect();
        assert_eq!(v, vec!["ERF.PRECISE", "IF", "SUM", "XLOOKUP"]);
    }
}
