//! Aggregation of case results into the scoreboard and the gap report.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::sync::LazyLock;

use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::runner::{CaseResult, CaseStatus, CellResult};
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
    pub verdicts: BTreeMap<Verdict, usize>,
    /// Total wall time of the suite's cases (informational; never part of the gate).
    #[serde(default)]
    pub millis: u64,
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
    #[must_use]
    pub fn cell_pass_rate(&self) -> f64 {
        if self.cells == 0 {
            return 0.0;
        }
        #[allow(clippy::cast_precision_loss)]
        let rate = self.cells_passed as f64 / self.cells as f64;
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunReport {
    pub engine: String,
    pub suites: Vec<SuiteSummary>,
    /// Sorted by number of failing cells, descending.
    pub functions: Vec<FunctionStat>,
    /// Cases with only their *failing* cells retained.
    pub cases: Vec<CaseResult>,
    /// Slowest cases of the run (informational; never part of the gate).
    #[serde(default)]
    pub slowest: Vec<SlowCase>,
}

impl RunReport {
    #[must_use]
    pub fn build(engine: &str, results: &[CaseResult]) -> RunReport {
        let mut suites: BTreeMap<String, SuiteSummary> = BTreeMap::new();
        let mut functions: BTreeMap<String, FunctionStat> = BTreeMap::new();
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
            s.millis += r.millis;
            for c in &r.cells {
                *s.verdicts.entry(c.verdict).or_default() += 1;
                for f in c.formula.as_deref().map(functions_in).unwrap_or_default() {
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
        let mut functions: Vec<FunctionStat> = functions.into_values().collect();
        functions.sort_by(|a, b| {
            (b.cells - b.passed)
                .cmp(&(a.cells - a.passed))
                .then(a.name.cmp(&b.name))
        });
        let cases = results
            .iter()
            .map(|r| {
                let mut r = r.clone();
                r.cells.retain(|c| !c.verdict.is_pass());
                r
            })
            .collect();
        RunReport {
            engine: engine.to_string(),
            suites: suites.into_values().collect(),
            functions,
            cases,
            slowest: slowest(results, SLOWEST_CASES),
        }
    }

    fn write_suites_table(&self, md: &mut String) {
        let _ = writeln!(md, "## Suites\n");
        let _ = writeln!(md, "| suite | cases | pass | fail | pending | crashes | errors | cells | cell pass % | ulp | close | mismatch | unsupported | type |");
        let _ = writeln!(
            md,
            "|---|---|---|---|---|---|---|---|---|---|---|---|---|---|"
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
                "| {} | {} | {} | {} | {} | {} | {} | {} | {:.2} | {} | {} | {} | {} | {} |",
                s.name,
                s.cases,
                st(CaseStatus::Pass),
                st(CaseStatus::Fail),
                st(CaseStatus::Pending),
                crashes,
                errors,
                s.cells,
                100.0 * s.cell_pass_rate(),
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
                .filter(|(k, _)| !k.is_pass())
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
            skipped_volatile: 0,
            millis,
            cells: Vec::new(),
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

    #[test]
    fn report_sums_suite_time_and_renders_timing() {
        let report = RunReport::build("e", &[case("s/a", 5), case("s/b", 7), case("t/c", 1)]);
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
