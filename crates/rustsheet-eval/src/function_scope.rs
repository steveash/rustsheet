//! Milestone function scope: the exit criteria of the form "ironcalc-calc cells using only the
//! functions of milestones up to Mk at 100%" (docs/milestones.md, M2 and M5) as reported numbers.
//!
//! A scored `ironcalc-calc` cell is in the row of milestone `Mk` when every function its formula
//! calls has a catalog milestone of at most `k`; a formula that calls no function is in every row.
//! A function the catalog does not list counts as in scope, so no cell drops out silently (the same
//! convention as [`Catalog::in_scope`]). The rows are the report rows of the probe milestones (M2, M5 to
//! M11), plus any other catalog milestone, in phase order; M0 and M1 functions count in the M2 row
//! (decision 0051, item 3). An
//! array-member cell is classified by its anchor's formula (T-1877). Cells with neither a formula
//! nor an array anchor (spill children) cannot be classified; they are left out and counted on
//! their own line so the gap stays visible.
//! A call of a workbook defined name also counts the functions of the name's formula, so
//! `gelu(A1:C1)` over a `LAMBDA` name is classified by `LAMBDA`, `TANH`, `POWER`, ... (T-1970,
//! as decision 0051 item 2 reads name bodies); names calling names are followed up to
//! [`MAX_NAME_DEPTH`] levels, each name once. Precedents are not followed: only the cell's
//! own formula and the names it calls decide.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use serde::{Deserialize, Serialize};

use crate::catalog::Catalog;
use crate::closeness::Visible;
use crate::exit_exceptions::{ExitExceptions, EXCEPTIONS_PATH};
use crate::formula_calls::functions_in;
use crate::probe_milestones::{parse_milestone, row_of, REPORT_ROWS};
use crate::runner::CaseResult;

/// The suite scored.
pub const SUITE: &str = "ironcalc-calc";
/// How many levels of defined names calling defined names are followed; untrusted workbooks
/// can nest names arbitrarily (CLAUDE.md directive 5).
const MAX_NAME_DEPTH: usize = 16;

/// How the cells of [`SUITE`] that use only the functions up to one milestone scored.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MilestoneScope {
    /// The function milestone, `M<n>`.
    pub milestone: String,
    /// Scored cells whose formula calls only functions up to this milestone (or none).
    pub cells: usize,
    /// Of those, the cells that match Excel bit-exact.
    pub bit_exact: usize,
    /// Of those, the cells that gave their pinned by-design value (decision 0036).
    pub deviations: usize,
    /// All failing cells of the row, as `case!Sheet!A1`, sorted.
    pub failing: Vec<String>,
    /// The failing cells a named exit exception leaves out of this row (decision 0087), sorted.
    #[serde(default)]
    pub excepted: Vec<String>,
    /// The failing cells that are visibly off (decision 0083), sorted: the Phase 1 exit counts
    /// these, the bit-level `failing` is Phase 1b's (decision 0114).
    #[serde(default)]
    pub visibly_off: Vec<String>,
}

/// How the [`SUITE`] cells scored, one cumulative row per function milestone.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FunctionScope {
    /// The rows in phase order, each a superset of the one before.
    pub rows: Vec<MilestoneScope>,
    /// Scored cells with no formula and no array anchor (spill children), left out of every row.
    pub no_formula: usize,
    /// Of those, the cells that are known failures.
    pub no_formula_failing: usize,
    /// Exit exceptions whose cell fails in no row (it passes, is a deviation or is not scored).
    #[serde(default)]
    pub stale_exceptions: Vec<String>,
}

impl FunctionScope {
    /// Fills each row's `excepted` cells and the stale exceptions from `exceptions`; `failing`
    /// is unchanged.
    pub fn apply_exceptions(&mut self, exceptions: &ExitExceptions) {
        for row in &mut self.rows {
            let Some(milestone) = parse_milestone(&row.milestone) else {
                continue;
            };
            row.excepted = row
                .failing
                .iter()
                .filter(|cell| exceptions.applies(cell, milestone))
                .cloned()
                .collect();
        }
        self.stale_exceptions = exceptions
            .cells
            .keys()
            .filter(|cell| !self.rows.iter().any(|row| row.failing.contains(cell)))
            .cloned()
            .collect();
    }
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
    // The report rows of the probe milestones (one list for both reports), plus any catalog milestone they lack.
    let limits: Vec<u32> = milestones
        .values()
        .map(|&m| row_of(m))
        .chain(REPORT_ROWS)
        .collect::<BTreeSet<u32>>()
        .into_iter()
        .collect();
    let mut scope = FunctionScope {
        rows: limits
            .iter()
            .map(|m| MilestoneScope {
                milestone: format!("M{m}"),
                ..MilestoneScope::default()
            })
            .collect(),
        ..FunctionScope::default()
    };
    for r in cases {
        for c in &r.cells {
            let Some(formula) = c.formula.as_deref().or(c.anchor_formula.as_deref()) else {
                scope.no_formula += 1;
                scope.no_formula_failing += usize::from(c.verdict.is_failure());
                continue;
            };
            // The latest milestone among the functions called; the cell is in every row from there on.
            let needed = calls_through_names(formula, &r.defined_names)
                .iter()
                .filter_map(|f| milestones.get(f.as_str()).copied())
                .max()
                .unwrap_or(0);
            let first = limits.partition_point(|&limit| limit < needed);
            for row in &mut scope.rows[first..] {
                row.cells += 1;
                if c.verdict.is_pass() {
                    row.bit_exact += 1;
                } else if c.verdict.is_failure() {
                    let key = format!("{}!{}", r.id, c.key());
                    if Visible::is_off(c) {
                        row.visibly_off.push(key.clone());
                    }
                    row.failing.push(key);
                } else {
                    row.deviations += 1;
                }
            }
        }
    }
    for row in &mut scope.rows {
        row.failing.sort();
        row.visibly_off.sort();
    }
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
         a catalog milestone up to the row's (none counts too): the exit criteria of M2 and M5. Each \
         row includes the rows above it. A call of a defined name counts the functions of the \
         name's formula; precedents are not followed. {} scored cells with no formula or anchor \
         are left out ({} failing). `failing excl. exceptions` leaves out the cells \
         `{EXCEPTIONS_PATH}` names for the row's milestone or an earlier one (decision 0087).\n",
        s.no_formula, s.no_formula_failing
    );
    let _ = writeln!(
        md,
        "| milestone | scored cells | bit-exact | deviations | failing | failing excl. exceptions |\n\
         |---|---|---|---|---|---|"
    );
    for row in &s.rows {
        let _ = writeln!(
            md,
            "| {} | {} | {} | {} | {} | {} |",
            row.milestone,
            row.cells,
            row.bit_exact,
            row.deviations,
            row.failing.len(),
            row.failing.len().saturating_sub(row.excepted.len())
        );
    }
    let mut listed: &[String] = &[];
    for row in &s.rows {
        // Each cell is listed once, under the first row that has it.
        let new: Vec<&String> = row
            .failing
            .iter()
            .filter(|id| listed.binary_search(id).is_err())
            .collect();
        if !new.is_empty() {
            let _ = writeln!(md, "\nNewly failing at {}:\n", row.milestone);
            for id in new {
                let _ = writeln!(md, "- `{id}`");
            }
        }
        listed = &row.failing;
    }
    if !s.stale_exceptions.is_empty() {
        let _ = writeln!(
            md,
            "\nStale exit exceptions (the cell fails in no row; remove its line from \
             `{EXCEPTIONS_PATH}`):\n"
        );
        for cell in &s.stale_exceptions {
            let _ = writeln!(md, "- `{cell}`");
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
            v15: None,
            v15_reason: None,
            params: Vec::new(),
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
            kind: crate::runner::CellKind::Scored,
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
            skipped_add_in: 0,
            millis: 0,
            tags: Vec::new(),
            cells,
            text: Vec::new(),
            defined_names: BTreeMap::new(),
            arguments: Vec::new(),
            reference_forms: crate::poi_m8_subset::ReferenceFormCells::default(),
        }
    }

    fn row<'a>(s: &'a FunctionScope, milestone: &str) -> &'a MilestoneScope {
        s.rows
            .iter()
            .find(|r| r.milestone == milestone)
            .expect("row")
    }

    fn counts(r: &MilestoneScope) -> (usize, usize, usize) {
        (r.cells, r.bit_exact, r.deviations)
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
        // The probe milestones' report rows, so a milestone without catalog functions has its row too.
        assert_eq!(
            s.rows
                .iter()
                .map(|r| r.milestone.as_str())
                .collect::<Vec<_>>(),
            ["M2", "M5", "M6", "M7", "M8", "M9", "M10", "M11"]
        );
        let m2 = row(&s, "M2");
        assert_eq!(counts(m2), (3, 2, 0));
        assert_eq!(m2.failing, vec!["ironcalc-calc/a!S!A2".to_string()]);
        let m9 = row(&s, "M9");
        assert_eq!(counts(m9), (5, 3, 0));
        assert_eq!(
            m9.failing,
            vec![
                "ironcalc-calc/a!S!A2".to_string(),
                "ironcalc-calc/a!S!A3".to_string()
            ]
        );
        assert_eq!((s.no_formula, s.no_formula_failing), (1, 1));
    }

    #[test]
    fn a_formula_is_in_the_rows_from_its_latest_function_on() {
        let catalog = Catalog::from_functions([
            function("SUM", "M1"),
            function("COT", "M5"),
            function("EARLIER", "M6"),
        ]);
        let results = [case(
            SUITE,
            vec![
                cell("A1", Some("1+1"), Verdict::NumericMismatch),
                cell("A2", Some("SUM(COT(1))"), Verdict::NumericMismatch),
                cell("A3", Some("COT(EARLIER(1))"), Verdict::NumericMismatch),
            ],
        )];
        let s = summarize(&catalog, &results).expect("suite ran");
        // A function-free formula is in every row; an M5 formula is not in M2; an M6 call is in neither.
        assert_eq!(row(&s, "M2").cells, 1);
        assert_eq!(row(&s, "M5").cells, 2);
        assert_eq!(row(&s, "M6").cells, 3);
        let mut md = String::new();
        write_markdown(Some(&s), &mut md);
        // Each failing cell is listed once, under the first row that has it.
        assert_eq!(md.matches("`ironcalc-calc/a!S!A1`").count(), 1);
        assert!(md.contains("Newly failing at M5:\n\n- `ironcalc-calc/a!S!A2`"));
        assert!(md.contains("Newly failing at M6:\n\n- `ironcalc-calc/a!S!A3`"));
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
        let m2 = row(&s, "M2");
        assert_eq!((m2.cells, m2.bit_exact), (2, 1));
        assert_eq!(m2.failing, vec!["ironcalc-calc/a!S!B2".to_string()]);
        assert_eq!(row(&s, "M9").cells, 3);
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
        let m2 = row(&s, "M2");
        assert_eq!((m2.cells, m2.bit_exact), (4, 4));
        assert!(m2.failing.is_empty());
        assert_eq!(
            row(&s, "M5").failing,
            vec!["ironcalc-calc/a!S!A4".to_string()]
        );
        assert_eq!(row(&s, "M7").cells, 6);
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

    #[test]
    fn exceptions_apply_from_their_milestone_on_and_stale_ones_are_listed() {
        let catalog = Catalog::from_functions([function("SUM", "M1"), function("COT", "M5")]);
        let results = [case(
            SUITE,
            vec![
                cell("A1", Some("SUM(1)"), Verdict::NumericMismatch),
                cell("A2", Some("SUM(2)"), Verdict::NumericMismatch),
                cell("A3", Some("COT(1)"), Verdict::NumericMismatch),
                cell("A4", Some("SUM(4)"), Verdict::Pass),
            ],
        )];
        let mut s = summarize(&catalog, &results).expect("suite ran");
        let exceptions = ExitExceptions::parse(
            "ironcalc-calc/a!S!A1\tM2\t0074\tT-1\n\
             ironcalc-calc/a!S!A2\tM5\t0085\tT-1\n\
             ironcalc-calc/a!S!A3\tM5\t0085\tT-1\n\
             ironcalc-calc/a!S!A4\tM5\t0085\tT-1\n",
        )
        .expect("parses");
        s.apply_exceptions(&exceptions);
        // A2's exception is from M5, so the M2 row keeps it.
        let m2 = row(&s, "M2");
        assert_eq!(m2.failing.len(), 2);
        assert_eq!(m2.excepted, vec!["ironcalc-calc/a!S!A1".to_string()]);
        assert_eq!(row(&s, "M5").excepted.len(), 3);
        assert_eq!(row(&s, "M9").excepted.len(), 3);
        // A4 passes: its exception is stale, and nothing fails.
        assert_eq!(s.stale_exceptions, vec!["ironcalc-calc/a!S!A4".to_string()]);
        let mut md = String::new();
        write_markdown(Some(&s), &mut md);
        assert!(md.contains("| M2 | 3 | 1 | 0 | 2 | 1 |"), "{md}");
        assert!(md.contains("| M5 | 4 | 1 | 0 | 3 | 0 |"), "{md}");
        assert!(md.contains("Stale exit exceptions"));
        assert!(md.contains("- `ironcalc-calc/a!S!A4`"));
    }
}
