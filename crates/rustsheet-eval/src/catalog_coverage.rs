//! Catalog coverage per milestone (T-2112): for each function milestone, how many of its in-scope
//! catalog functions have the evidence floor of Excel cells (decision 0062) and at least one
//! Excel-backed scored cell in a generated suite tagged with that milestone. It turns the M5 exit
//! criterion "catalog coverage 100% for these categories" into a number.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use serde::{Deserialize, Serialize};

use crate::catalog::Catalog;
use crate::evidence_floor::EVIDENCE_FLOOR;
use crate::report::{functions_in, FunctionEvidence};
use crate::runner::CaseResult;
use crate::value::Verdict;
use crate::volatile::is_volatile;

/// The function milestones the section reports, in phase order.
pub const MILESTONES: [&str; 6] = ["M2", "M5", "M6", "M7", "M9", "M10"];

/// A function that misses the floor or the generated-suite condition.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Uncovered {
    /// Upper-case Excel name.
    pub name: String,
    /// Excel-evidence cells the run found (0 when none).
    pub cells: usize,
    /// Fewer than [`EVIDENCE_FLOOR`] Excel cells.
    pub misses_floor: bool,
    /// No Excel-backed scored cell in a generated suite of the milestone.
    pub misses_generated: bool,
    /// `oracle: none` or `volatile`, where that applies.
    pub tags: Vec<String>,
}

/// One milestone's row.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MilestoneCoverage {
    /// The function milestone, `M<n>`.
    pub milestone: String,
    /// In-scope catalog functions of exactly this milestone.
    pub functions: usize,
    /// Of those, with at least [`EVIDENCE_FLOOR`] Excel cells.
    pub with_evidence: usize,
    /// Of those, with a cell in a generated suite of the milestone.
    pub in_generated: usize,
    /// Of those, meeting both.
    pub both: usize,
    /// The functions missing either condition, by name.
    pub missing: Vec<Uncovered>,
}

/// Rows for [`MILESTONES`]. `evidence` is the report's per-function Excel evidence;
/// `generated` maps each generated suite's name to the milestones it is tagged with.
#[must_use]
pub fn summarize(
    catalog: &Catalog,
    results: &[CaseResult],
    evidence: &[FunctionEvidence],
    generated: &BTreeMap<String, Vec<String>>,
) -> Vec<MilestoneCoverage> {
    let cells: BTreeMap<&str, usize> = evidence
        .iter()
        .map(|e| (e.name.as_str(), e.cells))
        .collect();
    // milestone -> functions with an Excel-backed scored cell in one of its generated suites.
    let mut reached: BTreeMap<&str, BTreeSet<String>> = BTreeMap::new();
    for r in results {
        let Some(tags) = generated.get(&r.suite) else {
            continue;
        };
        for c in r
            .cells
            .iter()
            .filter(|c| c.from_excel && c.verdict != Verdict::Deviation)
        {
            let Some(formula) = c.formula.as_deref() else {
                continue;
            };
            for m in tags {
                reached
                    .entry(m.as_str())
                    .or_default()
                    .extend(functions_in(formula));
            }
        }
    }
    MILESTONES
        .iter()
        .map(|&m| {
            let mut row = MilestoneCoverage {
                milestone: m.to_string(),
                ..MilestoneCoverage::default()
            };
            for f in catalog
                .functions()
                .filter(|f| f.milestone == m && catalog.in_scope(&f.name))
            {
                let n = cells.get(f.name.as_str()).copied().unwrap_or(0);
                let a = n >= EVIDENCE_FLOOR;
                let b = reached.get(m).is_some_and(|s| s.contains(&f.name));
                row.functions += 1;
                row.with_evidence += usize::from(a);
                row.in_generated += usize::from(b);
                row.both += usize::from(a && b);
                if !(a && b) {
                    let mut tags = Vec::new();
                    if catalog.needs_other_evidence(&f.name) {
                        tags.push("oracle: none".to_string());
                    }
                    if is_volatile(&format!("{}(", f.name), false) {
                        tags.push("volatile".to_string());
                    }
                    row.missing.push(Uncovered {
                        name: f.name.clone(),
                        cells: n,
                        misses_floor: !a,
                        misses_generated: !b,
                        tags,
                    });
                }
            }
            row.missing.sort_by(|x, y| x.name.cmp(&y.name));
            row
        })
        .collect()
}

/// The report section.
pub fn write_markdown(rows: &[MilestoneCoverage], md: &mut String) {
    if rows.is_empty() {
        return;
    }
    let _ = writeln!(md, "\n## Catalog coverage per milestone\n");
    let _ = writeln!(
        md,
        "In-scope catalog functions of each milestone (not cumulative) with (a) at least {EVIDENCE_FLOOR} Excel cells and (b) an Excel-backed scored cell in a generated suite tagged with the milestone.\n"
    );
    let _ = writeln!(
        md,
        "| milestone | functions | (a) evidence floor | (b) generated suite | both |"
    );
    let _ = writeln!(md, "|---|---|---|---|---|");
    for r in rows {
        let _ = writeln!(
            md,
            "| {} | {} | {} | {} | {} |",
            r.milestone, r.functions, r.with_evidence, r.in_generated, r.both
        );
    }
    for r in rows.iter().filter(|r| !r.missing.is_empty()) {
        let list: Vec<String> = r
            .missing
            .iter()
            .map(|u| {
                let mut miss = Vec::new();
                if u.misses_floor {
                    miss.push(format!("a: {} cells", u.cells));
                }
                if u.misses_generated {
                    miss.push("b".to_string());
                }
                let mut tags = String::new();
                for t in &u.tags {
                    let _ = write!(tags, ", {t}");
                }
                format!("{} ({}{tags})", u.name, miss.join(", "))
            })
            .collect();
        let _ = writeln!(
            md,
            "\n{} missing ({}): {}",
            r.milestone,
            r.missing.len(),
            list.join("; ")
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::{CatalogFunction, OracleSupport, Scope};
    use crate::runner::{CaseStatus, CellResult};

    fn func(name: &str, scope: Scope, oracle: Option<OracleSupport>) -> CatalogFunction {
        CatalogFunction {
            name: name.into(),
            category: "math".into(),
            milestone: "M5".into(),
            scope,
            reason: None,
            oracle,
        }
    }

    fn result(suite: &str, formula: &str, from_excel: bool) -> CaseResult {
        let cell = CellResult {
            sheet: "S".into(),
            cell: "A1".into(),
            formula: Some(formula.into()),
            anchor_formula: None,
            expected: crate::value::Value::Number(1.0),
            actual: crate::value::Value::Number(1.0),
            verdict: Verdict::Pass,
            from_excel,
            listed_deviation: false,
            graph_subnormal: false,
        };
        CaseResult {
            id: format!("{suite}/c"),
            suite: suite.into(),
            status: CaseStatus::Pass,
            message: None,
            expected_source: String::new(),
            checked: 1,
            passed: 1,
            deviations: 0,
            skipped_volatile: 0,
            millis: 0,
            tags: Vec::new(),
            cells: vec![cell],
            text: Vec::new(),
            defined_names: BTreeMap::new(),
        }
    }

    fn ev(name: &str, cells: usize) -> FunctionEvidence {
        FunctionEvidence {
            name: name.into(),
            cells,
            ..FunctionEvidence::default()
        }
    }

    #[test]
    fn classifies_floor_generated_tags_and_scope() {
        let catalog = Catalog::from_functions([
            func("COVERED", Scope::In, None),
            func("THIN", Scope::In, None),
            func("HANDMADE", Scope::In, None),
            func("INFO", Scope::In, Some(OracleSupport::Unavailable)),
            func("RAND", Scope::In, None),
            func("GONE", Scope::Out, None),
        ]);
        let results = vec![
            result("gen-m5", "COVERED(1)", true),
            result("gen-m5", "THIN(1)", true),
            result("probes", "HANDMADE(1)", true),
            result("gen-m5", "HANDMADE(2)", false),
        ];
        let evidence = [ev("COVERED", 20), ev("THIN", 19), ev("HANDMADE", 30)];
        let generated = BTreeMap::from([("gen-m5".to_string(), vec!["M5".to_string()])]);
        let rows = summarize(&catalog, &results, &evidence, &generated);
        let m5 = rows.iter().find(|r| r.milestone == "M5").unwrap();
        assert_eq!(
            (m5.functions, m5.with_evidence, m5.in_generated, m5.both),
            (5, 2, 2, 1)
        );
        let missing: Vec<(&str, bool, bool, Vec<&str>)> = m5
            .missing
            .iter()
            .map(|u| {
                (
                    u.name.as_str(),
                    u.misses_floor,
                    u.misses_generated,
                    u.tags.iter().map(String::as_str).collect(),
                )
            })
            .collect();
        assert_eq!(
            missing,
            [
                ("HANDMADE", false, true, vec![]),
                ("INFO", true, true, vec!["oracle: none"]),
                ("RAND", true, true, vec!["volatile"]),
                ("THIN", true, false, vec![]),
            ]
        );
    }
}
