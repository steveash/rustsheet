//! Catalog coverage per milestone (T-2112): for each function milestone, how many of its in-scope
//! catalog functions have the evidence floor of Excel cells (decision 0062) and at least one
//! Excel-backed scored cell in a generated suite tagged with that milestone. It turns the M5 exit
//! criterion "catalog coverage 100% for these categories" into a number. `covered` applies the
//! exemption of decision 0085 item 4: volatile and `oracle: none` functions need (a) only.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use serde::{Deserialize, Serialize};

use crate::catalog::Catalog;
use crate::evidence_floor::EVIDENCE_FLOOR;
use crate::formula_calls::functions_in;
use crate::probe_milestones::{parse_milestone, row_of, REPORT_ROWS};
use crate::report::FunctionEvidence;
use crate::runner::CaseResult;
use crate::value::Verdict;
use crate::volatile::is_volatile_function;

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
    /// Of those, meeting the criterion of decision 0085 item 4: `both`, or (a) alone for a
    /// volatile or `oracle: none` function.
    pub covered: usize,
    /// The functions still missing the criterion, by name.
    pub missing: Vec<Uncovered>,
    /// Functions with (a) but no generated cell that decision 0085 item 4 exempts from (b).
    pub exempt: Vec<Uncovered>,
}

/// One row per report row of the probe milestones ([`REPORT_ROWS`], plus any catalog milestone
/// they lack), as in `function_scope`; M0 and M1 functions fold into the M2 row. `evidence` is the report's per-function Excel evidence;
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
    // row -> functions with an Excel-backed scored cell in one of its generated suites.
    let mut reached: BTreeMap<u32, BTreeSet<String>> = BTreeMap::new();
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
            for m in tags.iter().filter_map(|t| parse_milestone(t)) {
                reached
                    .entry(row_of(m))
                    .or_default()
                    .extend(functions_in(formula));
            }
        }
    }
    let rows: BTreeSet<u32> = catalog
        .functions()
        .filter_map(|f| parse_milestone(&f.milestone).map(row_of))
        .chain(REPORT_ROWS)
        .collect();
    rows.into_iter()
        .map(|m| {
            let mut row = MilestoneCoverage {
                milestone: format!("M{m}"),
                ..MilestoneCoverage::default()
            };
            for f in catalog.functions().filter(|f| {
                parse_milestone(&f.milestone).map(row_of) == Some(m) && catalog.in_scope(&f.name)
            }) {
                let n = cells.get(f.name.as_str()).copied().unwrap_or(0);
                let a = n >= EVIDENCE_FLOOR;
                let b = reached.get(&m).is_some_and(|s| s.contains(&f.name));
                row.functions += 1;
                row.with_evidence += usize::from(a);
                row.in_generated += usize::from(b);
                row.both += usize::from(a && b);
                if a && b {
                    row.covered += 1;
                } else {
                    let mut tags = Vec::new();
                    if catalog.needs_other_evidence(&f.name) {
                        tags.push("oracle: none".to_string());
                    }
                    if is_volatile_function(&f.name) {
                        tags.push("volatile".to_string());
                    }
                    // Decision 0085 item 4: these classes cannot have a scored generated cell.
                    let exempt = a && !tags.is_empty();
                    row.covered += usize::from(exempt);
                    let u = Uncovered {
                        name: f.name.clone(),
                        cells: n,
                        misses_floor: !a,
                        misses_generated: !b,
                        tags,
                    };
                    if exempt {
                        row.exempt.push(u);
                    } else {
                        row.missing.push(u);
                    }
                }
            }
            row.missing.sort_by(|x, y| x.name.cmp(&y.name));
            row.exempt.sort_by(|x, y| x.name.cmp(&y.name));
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
        "| milestone | functions | (a) evidence floor | (b) generated suite | both | covered (decision 0085) |"
    );
    let _ = writeln!(md, "|---|---|---|---|---|---|");
    for r in rows {
        let _ = writeln!(
            md,
            "| {} | {} | {} | {} | {} | {} |",
            r.milestone, r.functions, r.with_evidence, r.in_generated, r.both, r.covered
        );
    }
    for r in rows.iter().filter(|r| !r.exempt.is_empty()) {
        let list: Vec<String> = r
            .exempt
            .iter()
            .map(|u| format!("{} (exempt from b, {})", u.name, u.tags.join(", ")))
            .collect();
        let _ = writeln!(
            md,
            "\n{} exempt from b ({}): {}",
            r.milestone,
            r.exempt.len(),
            list.join("; ")
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
    use crate::runner::CellResult;

    fn func(name: &str, scope: Scope, oracle: Option<OracleSupport>) -> CatalogFunction {
        CatalogFunction {
            name: name.into(),
            category: "math".into(),
            milestone: "M5".into(),
            scope,
            reason: None,
            oracle,
            v15: None,
            v15_reason: None,
            returns: None,
            params: Vec::new(),
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
            kind: crate::runner::CellKind::Scored,
        };
        CaseResult {
            cells: vec![cell],
            ..CaseResult::for_test(&format!("{suite}/c"), suite)
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
            func("RANDBETWEEN", Scope::In, None),
            func("GONE", Scope::Out, None),
        ]);
        let results = vec![
            result("gen-m5", "COVERED(1)", true),
            result("gen-m5", "THIN(1)", true),
            result("probes", "HANDMADE(1)", true),
            result("gen-m5", "HANDMADE(2)", false),
        ];
        let evidence = [
            ev("COVERED", 20),
            ev("THIN", 19),
            ev("HANDMADE", 30),
            ev("INFO", 20),
            ev("RAND", 19),
            ev("RANDBETWEEN", 25),
        ];
        let generated = BTreeMap::from([("gen-m5".to_string(), vec!["M5".to_string()])]);
        let rows = summarize(&catalog, &results, &evidence, &generated);
        let m5 = rows.iter().find(|r| r.milestone == "M5").unwrap();
        assert_eq!(
            (m5.functions, m5.with_evidence, m5.in_generated, m5.both),
            (6, 4, 2, 1)
        );
        // COVERED by both, INFO (oracle: none) and RANDBETWEEN (volatile) by exemption; RAND has
        // too few cells and HANDMADE has no generated cell, so neither is exempt.
        assert_eq!(m5.covered, 3);
        let exempt: Vec<&str> = m5.exempt.iter().map(|u| u.name.as_str()).collect();
        assert_eq!(exempt, ["INFO", "RANDBETWEEN"]);
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
        let mut md = String::new();
        write_markdown(&rows, &mut md);
        assert!(md.contains("| M5 | 6 | 4 | 2 | 1 | 3 |"), "{md}");
        assert!(
            md.contains(
                "M5 missing (3): HANDMADE (b); RAND (a: 19 cells, b, volatile); THIN (a: 19 cells)"
            ),
            "{md}"
        );
        assert!(
            md.contains("M5 exempt from b (2): INFO (exempt from b, oracle: none); RANDBETWEEN (exempt from b, volatile)"),
            "{md}"
        );
        assert_eq!(
            missing,
            [
                ("HANDMADE", false, true, vec![]),
                ("RAND", true, true, vec!["volatile"]),
                ("THIN", true, false, vec![]),
            ]
        );
    }
}
