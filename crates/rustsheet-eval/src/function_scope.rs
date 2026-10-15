//! Milestone function scope: the M2 exit criterion "ironcalc-calc cells using only M1/M2
//! functions at 100%" (docs/milestones.md) as a reported number.
//!
//! A scored `ironcalc-calc` cell is in scope when every function its formula calls has a catalog
//! milestone of M0, M1 or M2; a formula that calls no function is in scope too. A function the
//! catalog does not list counts as in scope, so no cell drops out silently (the same
//! convention as [`Catalog::in_scope`]). Cells without a formula of their own (spill children) cannot be
//! classified; they are left out and counted on their own line so the gap stays visible.
//! Precedents are not followed transitively: only the cell's own formula decides.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use serde::{Deserialize, Serialize};

use crate::catalog::Catalog;
use crate::probe_milestones::parse_milestone;
use crate::report::functions_in;
use crate::runner::CaseResult;

/// The suite scored.
pub const SUITE: &str = "ironcalc-calc";
/// The highest catalog milestone that counts toward M2's exit.
const MAX_MILESTONE: u32 = 2;

/// How the M0–M2-only cells of [`SUITE`] scored.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FunctionScope {
    /// Scored cells whose formula calls only M0–M2 functions (or none).
    pub cells: usize,
    /// Of those, the cells that match Excel bit-exact.
    pub bit_exact: usize,
    /// Of those, the cells that gave their pinned by-design value (decision 0036).
    pub deviations: usize,
    /// The failing cells, as `case!Sheet!A1`.
    pub failing: Vec<String>,
    /// Scored cells with no formula of their own (spill children), left out of the counts above.
    pub no_formula: usize,
    /// Of those, the cells that are known failures.
    pub no_formula_failing: usize,
}

/// Scores the [`SUITE`] cells of `results`; `None` when the run executed no such case.
#[must_use]
pub fn summarize(catalog: &Catalog, results: &[CaseResult]) -> Option<FunctionScope> {
    let cases: Vec<&CaseResult> = results.iter().filter(|r| r.suite == SUITE).collect();
    if cases.is_empty() {
        return None;
    }
    let milestones: BTreeMap<&str, u32> = catalog
        .functions()
        .filter_map(|f| Some((f.name.as_str(), parse_milestone(&f.milestone)?)))
        .collect();
    let mut scope = FunctionScope::default();
    for r in cases {
        for c in &r.cells {
            let Some(formula) = c.formula.as_deref() else {
                scope.no_formula += 1;
                scope.no_formula_failing += usize::from(c.verdict.is_failure());
                continue;
            };
            let only_early = functions_in(formula).iter().all(|f| {
                milestones
                    .get(f.as_str())
                    .is_none_or(|&m| m <= MAX_MILESTONE)
            });
            if !only_early {
                continue;
            }
            scope.cells += 1;
            if c.verdict.is_pass() {
                scope.bit_exact += 1;
            } else if c.verdict.is_failure() {
                scope.failing.push(format!("{}!{}", r.id, c.key()));
            } else {
                scope.deviations += 1;
            }
        }
    }
    scope.failing.sort();
    Some(scope)
}

/// The report.md section for `scope`; nothing when the suite did not run.
pub fn write_markdown(scope: Option<&FunctionScope>, md: &mut String) {
    let Some(s) = scope else {
        return;
    };
    let _ = writeln!(
        md,
        "\n## Milestone function scope ({SUITE})\n\n`{SUITE}` formula cells whose functions all have \
         a catalog milestone of M0–M2 (none counts too): the M2 exit criterion. Precedents are \
         not followed.\n"
    );
    let _ = writeln!(
        md,
        "| scored cells | bit-exact | deviations | failing | no formula (left out) | of those failing |\n\
         |---|---|---|---|---|---|\n| {} | {} | {} | {} | {} | {} |",
        s.cells,
        s.bit_exact,
        s.deviations,
        s.failing.len(),
        s.no_formula,
        s.no_formula_failing
    );
    if !s.failing.is_empty() {
        let _ = writeln!(md, "\nFailing cells:\n");
        for id in &s.failing {
            let _ = writeln!(md, "- `{id}`");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::{CatalogFunction, Scope};
    use crate::runner::{CaseStatus, CellResult};
    use crate::value::{Value, Verdict};

    fn function(name: &str, milestone: &str) -> CatalogFunction {
        CatalogFunction {
            name: name.to_string(),
            category: "math".to_string(),
            milestone: milestone.to_string(),
            scope: Scope::In,
            reason: None,
            oracle: None,
        }
    }

    fn cell(cell: &str, formula: Option<&str>, verdict: Verdict) -> CellResult {
        CellResult {
            sheet: "S".to_string(),
            cell: cell.to_string(),
            formula: formula.map(str::to_string),
            expected: Value::Number(1.0),
            actual: Value::Number(1.0),
            verdict,
            from_excel: true,
            listed_deviation: false,
            graph_subnormal: false,
        }
    }

    fn case(suite: &str, cells: Vec<CellResult>) -> CaseResult {
        CaseResult {
            id: format!("{suite}/a"),
            suite: suite.to_string(),
            status: CaseStatus::Pass,
            message: None,
            expected_source: String::new(),
            checked: cells.len(),
            passed: 0,
            deviations: 0,
            skipped_volatile: 0,
            millis: 0,
            tags: Vec::new(),
            cells,
        }
    }

    #[test]
    fn scopes_function_free_early_and_late_formulas() {
        let catalog = Catalog::from_functions([function("SUM", "M1"), function("LATE", "M9")]);
        let results = [
            case(
                SUITE,
                vec![
                    cell("A1", Some("1+2"), Verdict::Pass),
                    cell("A2", Some("SUM(A1:A1)"), Verdict::NumericMismatch),
                    cell("A3", Some("LATE(1)"), Verdict::NumericMismatch),
                    cell("A4", Some("SUM(LATE(1))"), Verdict::Pass),
                    cell("A5", None, Verdict::NumericMismatch),
                    cell("A6", Some("UNLISTED(1)"), Verdict::Pass),
                ],
            ),
            case(
                "probes",
                vec![cell("A1", Some("1"), Verdict::NumericMismatch)],
            ),
        ];
        let s = summarize(&catalog, &results).expect("suite ran");
        assert_eq!((s.cells, s.bit_exact, s.deviations), (3, 2, 0));
        assert_eq!(s.failing, vec!["ironcalc-calc/a!S!A2".to_string()]);
        assert_eq!((s.no_formula, s.no_formula_failing), (1, 1));
    }

    #[test]
    fn nothing_without_the_suite() {
        let results = [case("probes", vec![cell("A1", Some("1"), Verdict::Pass)])];
        assert_eq!(summarize(&Catalog::default(), &results), None);
    }
}
