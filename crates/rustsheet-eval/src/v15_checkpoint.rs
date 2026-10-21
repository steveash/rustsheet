//! The fifteen-digit parity checkpoint of decision 0094 (T-0062): per in-scope catalog function,
//! its Excel cells, its Excel cells in generated suites where it is the formula's outer call, its
//! exemption and its visibly-off cells, and whether the run meets the checkpoint overall. It is a
//! report, never a gate; "both hold on two consecutive steward runs" is the steward's to check.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use serde::{Deserialize, Serialize};

use crate::catalog::Catalog;
use crate::evidence_floor::EVIDENCE_FLOOR;
use crate::formula_calls::outer_call;
use crate::report::RunReport;
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
}

/// Scores `report` (already built from `results`); `generated` names the generated suites.
#[must_use]
pub fn summarize(
    catalog: &Catalog,
    results: &[CaseResult],
    report: &RunReport,
    generated: &[&str],
) -> V15Checkpoint {
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
    let visible: BTreeMap<&str, usize> = report
        .functions
        .iter()
        .map(|f| (f.name.as_str(), f.visible.total()))
        .collect();
    let mut functions: Vec<V15Function> = catalog
        .functions()
        .filter(|f| catalog.in_scope(&f.name))
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
            let visible = visible.get(name).copied().unwrap_or(0);
            V15Function {
                name: f.name.clone(),
                cells: n,
                generated,
                exempt,
                visible,
                covered,
                meets: covered && visible == 0,
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
    let visibly_off = report.visibly_off_cells();
    V15Checkpoint {
        in_scope: functions.len(),
        met: !functions.is_empty() && covered == functions.len() && visibly_off == 0,
        functions,
        covered,
        visibly_off,
        suites: report.suites.iter().map(|s| s.name.clone()).collect(),
    }
}

/// The report section: the overall line, then the functions below the bar, worst first, then the
/// exemptions.
pub fn write_markdown(checkpoint: &V15Checkpoint, md: &mut String) {
    let _ = writeln!(md, "\n## Fifteen-digit parity checkpoint\n");
    let _ = writeln!(md, "**{}**\n", checkpoint.verdict_line());
    let _ = writeln!(
        md,
        "Decision 0094: every in-scope function has {CELLS_BAR} Excel cells, {GENERATED_BAR} of them in a generated suite where it is the formula's outer call ({EVIDENCE_FLOOR} cells for an exempt function), and no cell is visibly off. Visibly-off cells counted over: {}.\n",
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
        "| function | Excel cells | generated outer-call cells | exempt | visibly off |"
    );
    let _ = writeln!(md, "|---|---|---|---|---|");
    for f in below {
        let _ = writeln!(
            md,
            "| {} | {} | {} | {} | {} |",
            f.name,
            f.cells,
            f.generated,
            if f.exempt.is_some() { "yes" } else { "" },
            f.visible
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
mod tests {
    use super::*;
    use crate::catalog::{CatalogFunction, Scope, V15};
    use crate::report::tests::{case, cell};
    use crate::runner::CellResult;
    use crate::value::Value;

    fn func(name: &str, v15_reason: Option<&str>) -> CatalogFunction {
        CatalogFunction {
            name: name.into(),
            category: "math".into(),
            milestone: "M5".into(),
            scope: Scope::In,
            reason: None,
            oracle: None,
            v15: v15_reason.map(|_| V15::Exempt),
            v15_reason: v15_reason.map(String::from),
        }
    }

    fn excel(formula: &str, n: usize) -> Vec<CellResult> {
        let c = CellResult {
            from_excel: true,
            ..cell(formula, Verdict::Pass)
        };
        vec![c; n]
    }

    #[test]
    fn generated_cells_count_only_for_the_outer_call() {
        let mut g = case("gen-m5/a", 1);
        g.cells = [
            excel("=ABS(SQRT(4))", 3),
            excel("ABS(1)+SQRT(4)", 2),
            excel("=SQRT(\"(\")", 1),
        ]
        .concat();
        let mut p = case("probes/a", 1);
        p.cells = excel("ABS(2)", 5);
        let results = [g, p];
        let catalog = Catalog::from_functions([func("ABS", None), func("SQRT", None)]);
        let report = RunReport::build("e", &results, &[], &catalog);
        let v = summarize(&catalog, &results, &report, &["gen-m5"]);
        let row = |n: &str| {
            v.functions
                .iter()
                .find(|f| f.name == n)
                .cloned()
                .unwrap_or_default()
        };
        assert_eq!((row("ABS").cells, row("ABS").generated), (10, 3));
        assert_eq!((row("SQRT").cells, row("SQRT").generated), (6, 1));
    }

    #[test]
    fn exempt_function_needs_the_evidence_floor_only() {
        let mut r = case("probes/a", 1);
        r.cells = [
            excel("PI()", EVIDENCE_FLOOR),
            excel("ABS(1)", EVIDENCE_FLOOR),
        ]
        .concat();
        let results = [r];
        let catalog =
            Catalog::from_functions([func("PI", Some("no arguments")), func("ABS", None)]);
        let report = RunReport::build("e", &results, &[], &catalog);
        let v = summarize(&catalog, &results, &report, &[]);
        assert_eq!(v.functions[0].name, "ABS", "below the bar comes first");
        assert!(!v.functions[0].covered);
        assert!(v.functions[1].meets && v.functions[1].exempt.as_deref() == Some("no arguments"));
        assert_eq!((v.covered, v.in_scope, v.met), (1, 2, false));
        let mut md = String::new();
        write_markdown(&v, &mut md);
        assert!(md.contains("| ABS | 20 | 0 |  | 0 |"), "{md}");
        assert!(md.contains("| PI | no arguments |"), "{md}");
    }

    #[test]
    fn verdict_needs_every_function_covered_and_no_visible_cell() {
        let mut g = case("gen-m5/a", 1);
        g.cells = excel("PI()", 25);
        let catalog = Catalog::from_functions([func("PI", Some("no arguments"))]);
        let results = [g.clone()];
        let report = RunReport::build("e", &results, &[], &catalog);
        let v = summarize(&catalog, &results, &report, &["gen-m5"]);
        assert!(v.met);
        assert_eq!(
            v.verdict_line(),
            "Fifteen-digit parity: met (coverage 1/1, visibly-off cells 0)"
        );
        assert_eq!(v.suites, ["gen-m5"]);
        g.cells.push(CellResult {
            actual: Value::Number(2.0),
            ..cell("PI()", Verdict::NumericMismatch)
        });
        let results = [g];
        let report = RunReport::build("e", &results, &[], &catalog);
        let v = summarize(&catalog, &results, &report, &["gen-m5"]);
        assert_eq!(
            v.verdict_line(),
            "Fifteen-digit parity: not met (coverage 1/1, visibly-off cells 1)"
        );
        assert!(!v.functions[0].meets && v.functions[0].covered);
    }
}
