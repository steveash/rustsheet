//! Milestone function scope: the M2 exit criterion "ironcalc-calc cells using only M1/M2
//! functions at 100%" (docs/milestones.md) as a reported number.
//!
//! A scored `ironcalc-calc` cell is in scope when every function its formula calls has a catalog
//! milestone of M0, M1 or M2; a formula that calls no function is in scope too. A function the
//! catalog does not list counts as in scope, so no cell drops out silently (the same
//! convention as [`Catalog::in_scope`]). An array-member cell is classified by its anchor's formula
//! (T-1877). Cells with neither a formula nor an array anchor (spill children) cannot be
//! classified; they are left out and counted on their own line so the gap stays visible.
//! A call of a workbook defined name also counts the functions of the name's formula, so
//! `gelu(A1:C1)` over a `LAMBDA` name is classified by `LAMBDA`, `TANH`, `POWER`, ... (T-1970,
//! as decision 0051 item 2 reads name bodies); names calling names are followed up to
//! [`MAX_NAME_DEPTH`] levels, each name once. Precedents are not followed: only the cell's
//! own formula and the names it calls decide.

use std::collections::{BTreeMap, BTreeSet};
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
/// How many levels of defined names calling defined names are followed; untrusted workbooks
/// can nest names arbitrarily (CLAUDE.md directive 5).
const MAX_NAME_DEPTH: usize = 16;

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
    /// Scored cells with no formula and no array anchor (spill children), left out of the counts above.
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
            let Some(formula) = c.formula.as_deref().or(c.anchor_formula.as_deref()) else {
                scope.no_formula += 1;
                scope.no_formula_failing += usize::from(c.verdict.is_failure());
                continue;
            };
            let only_early = calls_through_names(formula, &r.defined_names)
                .iter()
                .all(|f| {
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

/// The functions `formula` calls, with those of every defined name it calls, transitively
/// (upper-case names; `names` is keyed by upper-case name). Each name is expanded once, which
/// ends cycles, and at most [`MAX_NAME_DEPTH`] levels deep.
fn calls_through_names(formula: &str, names: &BTreeMap<String, String>) -> BTreeSet<String> {
    let mut calls = functions_in(formula);
    let mut expanded = BTreeSet::new();
    let mut frontier: Vec<String> = calls.iter().cloned().collect();
    for _ in 0..MAX_NAME_DEPTH {
        let mut next = Vec::new();
        for call in frontier {
            let Some(body) = names.get(&call) else {
                continue;
            };
            if !expanded.insert(call) {
                continue;
            }
            for f in functions_in(body) {
                if names.contains_key(&f) {
                    next.push(f.clone());
                }
                calls.insert(f);
            }
        }
        if next.is_empty() {
            break;
        }
        frontier = next;
    }
    calls
}

/// The report.md section for `scope`; nothing when the suite did not run.
pub fn write_markdown(scope: Option<&FunctionScope>, md: &mut String) {
    let Some(s) = scope else {
        return;
    };
    let _ = writeln!(
        md,
        "\n## Milestone function scope ({SUITE})\n\n`{SUITE}` formula cells whose functions all have \
         a catalog milestone of M0–M2 (none counts too): the M2 exit criterion. A call of a \
         defined name counts the functions of the name's formula; precedents are not followed.\n"
    );
    let _ = writeln!(
        md,
        "| scored cells | bit-exact | deviations | failing | no formula or anchor (left out) | of those failing |\n\
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
            anchor_formula: None,
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
            text: Vec::new(),
            defined_names: BTreeMap::new(),
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
    fn array_members_are_scored_by_their_anchor() {
        let catalog = Catalog::from_functions([function("IF", "M2"), function("LATE", "M9")]);
        let member = |c: &str, anchor: &str, verdict| CellResult {
            anchor_formula: Some(anchor.to_string()),
            ..cell(c, None, verdict)
        };
        let results = [case(
            SUITE,
            vec![
                member("B1", "IF(TRUE,1,2)", Verdict::Pass),
                member("B2", "IF(TRUE,1,2)", Verdict::NumericMismatch),
                member("B3", "LATE(1)", Verdict::NumericMismatch),
                cell("B4", None, Verdict::NumericMismatch),
            ],
        )];
        let s = summarize(&catalog, &results).expect("suite ran");
        assert_eq!((s.cells, s.bit_exact), (2, 1));
        assert_eq!(s.failing, vec!["ironcalc-calc/a!S!B2".to_string()]);
        assert_eq!((s.no_formula, s.no_formula_failing), (1, 1));
    }

    #[test]
    fn calls_of_defined_names_are_classified_by_the_name_formula() {
        let catalog = Catalog::from_functions([
            function("LAMBDA", "M7"),
            function("POWER", "M5"),
            function("SUM", "M1"),
            function("IF", "M2"),
        ]);
        let names: BTreeMap<String, String> = [
            (
                "GELU",
                "_xlfn.LAMBDA(_xlpm.x, 0.5*_xlpm.x*POWER(_xlpm.x,3))",
            ),
            ("EARLY", "IF(SUM(Sheet1!$A$1)>0,1,2)"),
            ("OUTER", "INNER(1)"),
            ("INNER", "POWER(2,3)"),
            ("SELF", "SELF(1)+SUM(1)"),
            ("PING", "PONG(1)"),
            ("PONG", "PING(1)"),
        ]
        .into_iter()
        .map(|(n, b)| (n.to_string(), b.to_string()))
        .collect();
        let mut results = [case(
            SUITE,
            vec![
                cell("A1", Some("gelu(A1:C1)"), Verdict::NumericMismatch),
                cell("A2", Some("_xlfn.early(1)+1"), Verdict::Pass),
                cell("A3", Some("UNDEFINED(1)"), Verdict::Pass),
                cell("A4", Some("Outer()"), Verdict::NumericMismatch),
                cell("A5", Some("SELF(2)"), Verdict::Pass),
                cell("A6", Some("PING(2)"), Verdict::Pass),
            ],
        )];
        results[0].defined_names = names;
        let s = summarize(&catalog, &results).expect("suite ran");
        assert_eq!((s.cells, s.bit_exact), (4, 4));
        assert!(s.failing.is_empty());
    }

    #[test]
    fn name_chains_stop_at_the_depth_bound() {
        // N0 -> N1 -> ... -> N{MAX_NAME_DEPTH} -> POWER: MAX_NAME_DEPTH + 1 names to expand.
        let mut names: BTreeMap<String, String> = (0..MAX_NAME_DEPTH)
            .map(|i| (format!("N{i}"), format!("N{}(1)", i + 1)))
            .collect();
        names.insert(format!("N{MAX_NAME_DEPTH}"), "POWER(1,2)".to_string());
        let calls = calls_through_names("N0(1)", &names);
        assert!(!calls.contains("POWER"));
        assert!(calls.contains(&format!("N{MAX_NAME_DEPTH}")));
        assert!(calls_through_names("N1(1)", &names).contains("POWER"));
    }

    #[test]
    fn nothing_without_the_suite() {
        let results = [case("probes", vec![cell("A1", Some("1"), Verdict::Pass)])];
        assert_eq!(summarize(&Catalog::default(), &results), None);
    }
}
