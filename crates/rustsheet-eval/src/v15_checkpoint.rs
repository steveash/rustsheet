//! The fifteen-digit parity checkpoint of decision 0094 (T-0062): per in-scope catalog function,
//! its Excel cells, its Excel cells in generated suites where it is the formula's outer call, its
//! exemption and its visibly-off cells, and whether the run meets the checkpoint overall. It also
//! measures the Phase 1 exit of decision 0114: every function covered with at most one cell in a
//! thousand visibly off, its cells counted as decision 0118 counts them (its attributed cells, or
//! every Excel cell that calls it below the evidence floor). The stricter share, over every Excel
//! cell that calls the function with decision 0088's blame, is kept as information. It is a
//! report, never a gate; "both hold on two consecutive steward runs" is the steward's to check.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use serde::{Deserialize, Serialize};

use crate::attribution::Attribution;
use crate::catalog::Catalog;
use crate::closeness::Closeness;
use crate::evidence_floor::EVIDENCE_FLOOR;
use crate::formula_calls::outer_call;
use crate::milestone_exit::{meets_rate, RATE_DEN, RATE_NUM};
use crate::report::{FunctionStat, RunReport};
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
    /// Which cells the Phase 1 rate counts (decision 0118): its attributed cells (`sole`), or
    /// every Excel cell that calls it (`all`) below [`EVIDENCE_FLOOR`] attributed cells.
    #[serde(default)]
    pub attribution: Attribution,
    /// The scored cells the Phase 1 rate counts.
    #[serde(default)]
    pub rate_cells: usize,
    /// Of those, the visibly-off cells.
    #[serde(default)]
    pub rate_visible: usize,
    /// Covered and at least 99.9% of `rate_cells` not visibly off: the Phase 1 exit (decisions
    /// 0114 and 0118).
    #[serde(default)]
    pub meets_rate: bool,
    /// Information only: the stricter count of decision 0118 item 5.
    #[serde(default)]
    pub strict: StrictRate,
}

/// The stricter Phase 1 count, which gates nothing (decision 0118 item 5): every Excel cell that
/// calls the function (`cells`), with every visibly-off cell blamed on it (`visible`).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StrictRate {
    /// Covered and at least 99.9% of `cells` not visibly off.
    pub meets_rate: bool,
}

/// Share of `cells` not visibly off, floored at 0; `None` without cells.
fn share(cells: usize, visible: usize) -> Option<f64> {
    // Cell counts stay far below 2^52, so the casts are exact.
    #[allow(clippy::cast_precision_loss)]
    let rate = (cells > 0).then(|| cells.saturating_sub(visible) as f64 / cells as f64);
    rate
}

impl V15Function {
    /// The Phase 1 share: of `rate_cells`, those not visibly off; `None` without cells.
    #[must_use]
    pub fn rate(&self) -> Option<f64> {
        share(self.rate_cells, self.rate_visible)
    }

    /// Information only: the share of its Excel cells not visibly off with every blamed cell
    /// counted; `None` without cells. Blamed cells can exceed `cells` (a cell blamed on it may
    /// come from a non-Excel suite), so the share is floored at 0.
    #[must_use]
    pub fn strict_rate(&self) -> Option<f64> {
        share(self.cells, self.visible)
    }
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
    /// Functions that meet the Phase 1 rate (decision 0114).
    #[serde(default)]
    pub at_rate: usize,
    /// At least one function, and every one meets the Phase 1 rate (decision 0114).
    #[serde(default)]
    pub phase1_met: bool,
    /// Information only: functions that meet the strict rate ([`StrictRate::meets_rate`]).
    #[serde(default)]
    pub at_strict_rate: usize,
    /// The Phase 1 rate, `rate_num / rate_den` of a function's counted cells (decision 0114):
    /// [`RATE_NUM`] / [`RATE_DEN`], written out so report readers (the progress dashboard,
    /// T-0072) take the bar from the report instead of copying it.
    #[serde(default)]
    pub rate_num: usize,
    /// The denominator of [`Self::rate_num`].
    #[serde(default)]
    pub rate_den: usize,
    /// [`Self::verdict_line`], written out for report readers (T-0072).
    #[serde(default)]
    pub verdict_line: String,
    /// [`Self::phase1_line`], written out for report readers (T-0072).
    #[serde(default)]
    pub phase1_line: String,
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

    /// The Phase 1 exit line (decision 0114, counted as decision 0118 says) the steward copies
    /// beside [`Self::verdict_line`].
    #[must_use]
    pub fn phase1_line(&self) -> String {
        format!(
            "Phase 1 (decision 0114): {} ({}/{} functions covered with ≥ 99.9% of their attributed cells within 15 digits, decision 0118)",
            if self.phase1_met { "met" } else { "not met" },
            self.at_rate,
            self.in_scope
        )
    }

    /// The stricter count of decision 0118 item 5, labelled as information.
    #[must_use]
    pub fn strict_line(&self) -> String {
        format!(
            "Information, gates nothing: {}/{} functions covered with ≥ 99.9% of every Excel cell that calls them within 15 digits, each visibly-off cell blamed on every non-glue function of its formula (decision 0088)",
            self.at_strict_rate, self.in_scope
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
    let stats: BTreeMap<&str, &FunctionStat> = report
        .functions
        .iter()
        .map(|f| (f.name.as_str(), f))
        .collect();
    let mut functions: Vec<V15Function> = catalog
        .in_scope_functions()
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
            let stat = stats.get(name).copied();
            let visible = stat.map_or(0, |s| s.visible.total());
            let (attribution, counted) =
                stat.map_or((Attribution::All, Closeness::default()), |s| {
                    let (a, c) = s.counted();
                    (a, *c)
                });
            let (rate_cells, rate_visible) = (counted.scored(), counted.visible());
            V15Function {
                name: f.name.clone(),
                cells: n,
                generated,
                exempt,
                visible,
                covered,
                meets: covered && visible == 0,
                attribution,
                rate_cells,
                rate_visible,
                meets_rate: covered
                    && meets_rate(
                        rate_cells.saturating_sub(rate_visible),
                        rate_cells,
                        RATE_NUM,
                    ),
                strict: StrictRate {
                    meets_rate: covered && meets_rate(n.saturating_sub(visible), n, RATE_NUM),
                },
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
    let at_rate = functions.iter().filter(|f| f.meets_rate).count();
    let at_strict_rate = functions.iter().filter(|f| f.strict.meets_rate).count();
    let visibly_off = report.visibly_off_cells();
    let mut checkpoint = V15Checkpoint {
        in_scope: functions.len(),
        met: !functions.is_empty() && covered == functions.len() && visibly_off == 0,
        at_rate,
        phase1_met: !functions.is_empty() && at_rate == functions.len(),
        at_strict_rate,
        functions,
        covered,
        visibly_off,
        suites: report.suites.iter().map(|s| s.name.clone()).collect(),
        rate_num: RATE_NUM,
        rate_den: RATE_DEN,
        verdict_line: String::new(),
        phase1_line: String::new(),
    };
    checkpoint.verdict_line = checkpoint.verdict_line();
    checkpoint.phase1_line = checkpoint.phase1_line();
    checkpoint
}

/// The report section: the overall line, then the functions below the bar, worst first, then the
/// exemptions.
pub fn write_markdown(checkpoint: &V15Checkpoint, md: &mut String) {
    let _ = writeln!(md, "\n## Fifteen-digit parity checkpoint\n");
    let _ = writeln!(md, "**{}**\n", checkpoint.verdict_line());
    let _ = writeln!(md, "**{}**\n", checkpoint.phase1_line());
    let _ = writeln!(md, "{}.\n", checkpoint.strict_line());
    let _ = writeln!(
        md,
        "Decision 0094: every in-scope function has {CELLS_BAR} Excel cells, {GENERATED_BAR} of them in a generated suite where it is the formula's outer call ({EVIDENCE_FLOOR} cells for an exempt function), and no cell is visibly off. Visibly-off cells counted over: {}. The Phase 1 rate counts each function's attributed cells, the Excel cells whose formula calls it as the only function outside the glue list, or every Excel cell that calls it below {EVIDENCE_FLOOR} attributed cells (decision 0118).\n",
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
        "| function | Excel cells | generated outer-call cells | exempt | visibly off | Phase 1 cells | of them visibly off | within 15 digits | Phase 1 (≥ 99.9%) | strict share (information) |"
    );
    let _ = writeln!(md, "|---|---|---|---|---|---|---|---|---|---|");
    let pct =
        |r: Option<f64>| r.map_or_else(|| "n/a".to_string(), |r| format!("{:.2}%", r * 100.0));
    for f in below {
        let _ = writeln!(
            md,
            "| {} | {} | {} | {} | {} | {} {} | {} | {} | {} | {} |",
            f.name,
            f.cells,
            f.generated,
            if f.exempt.is_some() { "yes" } else { "" },
            f.visible,
            f.rate_cells,
            match f.attribution {
                Attribution::Sole => "attributed",
                Attribution::All => "all (fallback)",
            },
            f.rate_visible,
            pct(f.rate()),
            if f.meets_rate { "met" } else { "not met" },
            pct(f.strict_rate())
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
        assert!(
            md.contains(
                "| ABS | 20 | 0 |  | 0 | 20 attributed | 0 | 100.00% | not met | 100.00% |"
            ),
            "{md}"
        );
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

    #[test]
    fn report_json_carries_the_rate_and_both_lines() {
        // T-0072: the progress dashboard reads the bar and the lines from report.json.
        let mut g = case("gen-m5/a", 1);
        g.cells = excel("PI()", 25);
        let catalog = Catalog::from_functions([func("PI", Some("no arguments"))]);
        let results = [g];
        let report = RunReport::build("e", &results, &[], &catalog);
        let v = summarize(&catalog, &results, &report, &["gen-m5"]);
        let json = serde_json::to_value(&v).unwrap();
        assert_eq!(
            (json["rate_num"].as_u64(), json["rate_den"].as_u64()),
            (Some(999), Some(1000))
        );
        assert_eq!(
            json["verdict_line"].as_str(),
            Some(v.verdict_line().as_str())
        );
        assert_eq!(json["phase1_line"].as_str(), Some(v.phase1_line().as_str()));
    }

    #[test]
    fn phase1_allows_one_visibly_off_cell_in_a_thousand() {
        // Decision 0114: covered and at most 0.1% of the Excel cells visibly off.
        let off = || CellResult {
            from_excel: true,
            actual: Value::Number(2.0),
            ..cell("PI()", Verdict::NumericMismatch)
        };
        let catalog = Catalog::from_functions([func("PI", Some("no arguments"))]);
        let mut g = case("gen-m5/a", 1);
        g.cells = excel("PI()", 999);
        g.cells.push(off());
        let results = [g.clone()];
        let report = RunReport::build("e", &results, &[], &catalog);
        let v = summarize(&catalog, &results, &report, &["gen-m5"]);
        assert!(!v.met && v.phase1_met && v.functions[0].meets_rate);
        assert_eq!(
            v.phase1_line(),
            "Phase 1 (decision 0114): met (1/1 functions covered with ≥ 99.9% of their attributed cells within 15 digits, decision 0118)"
        );
        g.cells.push(off());
        let results = [g];
        let report = RunReport::build("e", &results, &[], &catalog);
        let v = summarize(&catalog, &results, &report, &["gen-m5"]);
        assert!(!v.phase1_met && !v.functions[0].meets_rate);
        let mut md = String::new();
        write_markdown(&v, &mut md);
        assert!(
            md.contains(
                "| PI | 1001 | 1001 | yes | 2 | 1001 attributed | 2 | 99.80% | not met | 99.80% |"
            ),
            "{md}"
        );
    }

    fn off(formula: &str) -> CellResult {
        CellResult {
            from_excel: true,
            actual: Value::Number(2.0),
            ..cell(formula, Verdict::NumericMismatch)
        }
    }

    fn checkpoint(cells: Vec<CellResult>, catalog: &Catalog) -> V15Checkpoint {
        let mut r = case("probes/a", 1);
        r.cells = cells;
        let results = [r];
        let report = RunReport::build("e", &results, &[], catalog);
        summarize(catalog, &results, &report, &[])
    }

    #[test]
    fn phase1_counts_attributed_cells_and_keeps_the_strict_share_as_information() {
        // Decision 0118: an off cell of `GAMMA(BETA.DIST(..))` is attributed to neither function,
        // so GAMMA meets the rate on its own cells while the strict count, which blames the cell
        // on both, does not.
        let catalog =
            Catalog::from_functions([func("GAMMA", Some("test")), func("BETA.DIST", Some("test"))]);
        let mut cells = excel("GAMMA(1)+GAMMA(2)", 1000);
        cells.extend((0..10).map(|_| off("GAMMA(BETA.DIST(0.5,2,3,TRUE))")));
        let v = checkpoint(cells, &catalog);
        let row = |n: &str| v.functions.iter().find(|f| f.name == n).cloned();
        let gamma = row("GAMMA").unwrap_or_default();
        assert_eq!((gamma.cells, gamma.visible), (1010, 10));
        assert_eq!(
            (gamma.attribution, gamma.rate_cells, gamma.rate_visible),
            (Attribution::Sole, 1000, 0)
        );
        assert!(gamma.covered && gamma.meets_rate && !gamma.strict.meets_rate);
        assert_eq!(gamma.rate(), Some(1.0));
        assert_eq!(gamma.strict_rate().map(|r| (r * 1e4).round()), Some(9901.0));
        // BETA.DIST has no attributed cell: it falls back to every Excel cell that calls it.
        let beta = row("BETA.DIST").unwrap_or_default();
        assert_eq!(
            (beta.attribution, beta.rate_cells, beta.rate_visible),
            (Attribution::All, 10, 10)
        );
        assert!(!beta.meets_rate && !beta.strict.meets_rate);
        assert_eq!((v.at_rate, v.at_strict_rate, v.phase1_met), (1, 0, false));
        assert_eq!(
            v.strict_line(),
            "Information, gates nothing: 0/2 functions covered with ≥ 99.9% of every Excel cell \
             that calls them within 15 digits, each visibly-off cell blamed on every non-glue \
             function of its formula (decision 0088)"
        );
        let mut md = String::new();
        write_markdown(&v, &mut md);
        assert!(md.contains(&format!("{}.\n", v.strict_line())), "{md}");
        assert!(
            md.contains(
                "| GAMMA | 1010 | 0 | yes | 10 | 1000 attributed | 0 | 100.00% | met | 99.01% |"
            ),
            "{md}"
        );
        assert!(
            md.contains("| BETA.DIST | 10 | 0 | yes | 10 | 10 all (fallback) | 10 | 0.00% | not met | 0.00% |"),
            "{md}"
        );
    }

    #[test]
    fn fewer_than_the_floor_of_attributed_cells_rates_every_excel_cell() {
        let catalog = Catalog::from_functions([func("ABS", Some("test")), func("SQRT", None)]);
        // 19 attributed cells: the rate counts all 20 Excel cells that call ABS, one of them off.
        let mut cells = excel("ABS(1)", EVIDENCE_FLOOR - 1);
        cells.push(off("ABS(SQRT(4))"));
        let v = checkpoint(cells.clone(), &catalog);
        let abs = v.functions.iter().find(|f| f.name == "ABS").cloned();
        let abs = abs.unwrap_or_default();
        assert_eq!(
            (abs.attribution, abs.rate_cells, abs.rate_visible),
            (Attribution::All, 20, 1)
        );
        assert!(abs.covered && !abs.meets_rate);
        // At the floor the attributed cells alone count, and none of them is off.
        cells.extend(excel("ABS(2)", 1));
        let v = checkpoint(cells, &catalog);
        let abs = v.functions.iter().find(|f| f.name == "ABS").cloned();
        let abs = abs.unwrap_or_default();
        assert_eq!(
            (abs.attribution, abs.rate_cells, abs.rate_visible),
            (Attribution::Sole, 20, 0)
        );
        assert!(abs.meets_rate && !abs.strict.meets_rate);
        assert_eq!((abs.cells, abs.visible), (21, 1));
    }
}
