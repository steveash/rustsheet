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

/// Below this many Excel-evidence cells a function's pass rate says little (T-0020).
pub const EVIDENCE_THRESHOLD: usize = 25;

/// Whether a case's expectations come from Excel (an oracle backend or Excel-cached values),
/// not from hand-written or engine-derived values.
pub(crate) fn is_excel_evidence(expected_source: &str) -> bool {
    expected_source.starts_with("excel")
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
    /// Excel evidence per function, fewest cells first (informational; never part of the gate).
    #[serde(default)]
    pub evidence: Vec<FunctionEvidence>,
    /// Implemented functions that no Excel-evidence cell calls.
    #[serde(default)]
    pub unevidenced: Vec<String>,
}

impl RunReport {
    /// Aggregates `results`; `implemented` lists the engine's functions, so that functions
    /// without any Excel evidence show up too.
    #[must_use]
    pub fn build(engine: &str, results: &[CaseResult], implemented: &[String]) -> RunReport {
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
            s.millis += r.millis;
            for c in &r.cells {
                *s.verdicts.entry(c.verdict).or_default() += 1;
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
        let unevidenced = implemented
            .iter()
            .filter(|f| !evidence.contains_key(f.as_str()))
            .cloned()
            .collect();
        let mut evidence: Vec<FunctionEvidence> = evidence.into_values().collect();
        evidence.sort_by(|a, b| a.cells.cmp(&b.cells).then_with(|| a.name.cmp(&b.name)));
        RunReport {
            engine: engine.to_string(),
            suites: suites.into_values().collect(),
            functions,
            cases,
            slowest: slowest(results, SLOWEST_CASES),
            evidence,
            unevidenced,
        }
    }

    /// One line for docs/status.md: how many functions have enough Excel evidence.
    #[must_use]
    pub fn evidence_summary(&self) -> String {
        let thin = self
            .evidence
            .iter()
            .filter(|f| f.cells < EVIDENCE_THRESHOLD)
            .count();
        format!(
            "Excel evidence: {} functions with ≥{EVIDENCE_THRESHOLD} oracle/cached cells, {thin} with fewer, {} implemented with none",
            self.evidence.len() - thin,
            self.unevidenced.len()
        )
    }

    fn write_evidence(&self, md: &mut String) {
        let _ = writeln!(md, "\n## Excel evidence per function\n");
        let _ = writeln!(
            md,
            "{}. Cells count when their expectations come from Excel (oracle sidecar or Excel-cached values), pass or fail; hand-written expectations do not count.\n",
            self.evidence_summary()
        );
        if !self.unevidenced.is_empty() {
            let _ = writeln!(
                md,
                "Implemented, no Excel evidence ({}): {}\n",
                self.unevidenced.len(),
                self.unevidenced.join(", ")
            );
        }
        let _ = writeln!(
            md,
            "| function | Excel cells | suites | below {EVIDENCE_THRESHOLD} |"
        );
        let _ = writeln!(md, "|---|---|---|---|");
        for f in &self.evidence {
            let suites: Vec<&str> = f.suites.iter().map(String::as_str).collect();
            let flag = if f.cells < EVIDENCE_THRESHOLD {
                "yes"
            } else {
                ""
            };
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
        self.write_evidence(&mut md);
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

    fn cell(formula: &str, verdict: Verdict) -> CellResult {
        CellResult {
            sheet: "S".into(),
            cell: "A1".into(),
            formula: Some(formula.into()),
            expected: crate::value::Value::Number(1.0),
            actual: crate::value::Value::Number(1.0),
            verdict,
            from_excel: false,
        }
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
        let report = RunReport::build("e", &[oracle, cached, hand], &implemented);
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
        assert!(md.contains("Excel evidence: 0 functions with ≥25 oracle/cached cells, 3 with fewer, 2 implemented with none"));
        assert!(md.contains("Implemented, no Excel evidence (2): IF, TRIM"));
        assert!(md.contains("| SUM | 2 | probes | yes |"));
    }

    #[test]
    fn report_sums_suite_time_and_renders_timing() {
        let report = RunReport::build("e", &[case("s/a", 5), case("s/b", 7), case("t/c", 1)], &[]);
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
