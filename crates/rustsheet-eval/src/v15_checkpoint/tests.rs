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
        returns: None,
        params: Vec::new(),
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
    let v = summarize(
        &catalog,
        &results,
        &report,
        &["gen-m5"],
        &Phase1Exceptions::default(),
    );
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
    let catalog = Catalog::from_functions([func("PI", Some("no arguments")), func("ABS", None)]);
    let report = RunReport::build("e", &results, &[], &catalog);
    let v = summarize(
        &catalog,
        &results,
        &report,
        &[],
        &Phase1Exceptions::default(),
    );
    assert_eq!(v.functions[0].name, "ABS", "below the bar comes first");
    assert!(!v.functions[0].covered);
    assert!(v.functions[1].meets && v.functions[1].exempt.as_deref() == Some("no arguments"));
    assert_eq!((v.covered, v.in_scope, v.met), (1, 2, false));
    let mut md = String::new();
    write_markdown(&v, &mut md);
    assert!(
        md.contains(
            "| ABS | 20 | 0 |  | 0 | 20 attributed | 0 | 0 | 100.00% | not met | 100.00% |"
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
    let v = summarize(
        &catalog,
        &results,
        &report,
        &["gen-m5"],
        &Phase1Exceptions::default(),
    );
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
    let v = summarize(
        &catalog,
        &results,
        &report,
        &["gen-m5"],
        &Phase1Exceptions::default(),
    );
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
    let v = summarize(
        &catalog,
        &results,
        &report,
        &["gen-m5"],
        &Phase1Exceptions::default(),
    );
    let json = serde_json::to_value(&v).unwrap();
    assert_eq!(
        (json["rate_num"].as_u64(), json["rate_den"].as_u64()),
        (Some(9_990), Some(10_000))
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
    let v = summarize(
        &catalog,
        &results,
        &report,
        &["gen-m5"],
        &Phase1Exceptions::default(),
    );
    // Gate B counts the gen-m5 cells but is not measured without calipers, poi, corpus-bench.
    assert!(!v.met && !v.phase1_met && v.at_rate == 1 && v.functions[0].meets_rate);
    assert_eq!(
        v.phase1_line(),
        "Phase 1 rate, Gate B (decision 0188 R1(b)): n/a (1/1 functions covered with ≥ 99.9% of their attributed cells within 15 digits on gen-*, ironcalc-calc, corpus-bench, calipers, poi, smoke, fixtures, not measured: the run did not score calipers, poi, corpus-bench); every suite, information: 1/1 (decision 0118)"
    );
    g.cells.push(off());
    let results = [g];
    let report = RunReport::build("e", &results, &[], &catalog);
    let v = summarize(
        &catalog,
        &results,
        &report,
        &["gen-m5"],
        &Phase1Exceptions::default(),
    );
    assert!(!v.phase1_met && !v.functions[0].meets_rate);
    let mut md = String::new();
    write_markdown(&v, &mut md);
    assert!(
        md.contains(
            "| PI | 1001 | 1001 | yes | 2 | 1001 attributed | 2 | 0 | 99.80% | not met | 99.80% |"
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
    summarize(
        catalog,
        &results,
        &report,
        &[],
        &Phase1Exceptions::default(),
    )
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
            "| GAMMA | 1010 | 0 | yes | 10 | 1000 attributed | 0 | 0 | 100.00% | met | 99.01% |"
        ),
        "{md}"
    );
    assert!(
        md.contains("| BETA.DIST | 10 | 0 | yes | 10 | 10 all (fallback) | 10 | 0 | 0.00% | not met | 0.00% |"),
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

fn at(c: CellResult, a1: &str) -> CellResult {
    CellResult {
        cell: a1.into(),
        ..c
    }
}

#[test]
fn listed_cells_leave_the_phase1_rate_and_unlisted_ones_still_count() {
    // Decision 0134: a listed visibly-off cell leaves the count of the function it counts
    // toward; an unlisted one, a listed passing cell and a listed cell the run lacks do not.
    let catalog = Catalog::from_functions([func("PI", Some("test")), func("ABS", Some("test"))]);
    let mut cells = excel("PI()", 999);
    cells.push(at(off("PI()"), "B1"));
    cells.push(at(off("PI()"), "B2"));
    cells.push(at(excel("PI()", 1).remove(0), "C1"));
    // ABS has fewer attributed cells than the floor, so every Excel cell that calls it counts.
    cells.extend(excel("ABS(1)", EVIDENCE_FLOOR - 1));
    cells.push(at(off("ABS(PI())"), "D1"));
    let mut r = case("probes/a", 1);
    r.cells = cells;
    let results = [r];
    let report = RunReport::build("e", &results, &[], &catalog);
    let none = summarize(
        &catalog,
        &results,
        &report,
        &[],
        &Phase1Exceptions::default(),
    );
    let pi = |v: &V15Checkpoint| v.functions.iter().find(|f| f.name == "PI").cloned();
    let before = pi(&none).unwrap_or_default();
    assert_eq!((before.rate_cells, before.rate_visible), (1002, 2));
    assert!(!before.meets_rate);
    let listed = Phase1Exceptions::parse(
        "probes/a!S!B1\t0134\tT-1075\nprobes/a!S!C1\t0134\tT-1075\n\
         probes/a!S!Z9\t0134\tT-1075\nprobes/a!S!D1\t0134\tT-1075\n\
         poi/b.xlsx!S!G9\t0134\tT-1075\n",
    )
    .expect("parses");
    let v = summarize(&catalog, &results, &report, &[], &listed);
    let after = pi(&v).unwrap_or_default();
    assert_eq!(
        (after.rate_cells, after.rate_visible, after.rate_excepted),
        (1001, 1, 1)
    );
    assert!(after.meets_rate && !after.strict.meets_rate);
    // The information count and the decision 0094 checkpoint keep the listed cell.
    assert_eq!((after.cells, after.visible), (before.cells, before.visible));
    let abs = v.functions.iter().find(|f| f.name == "ABS").cloned();
    let abs = abs.unwrap_or_default();
    assert_eq!(
        (
            abs.attribution,
            abs.rate_cells,
            abs.rate_visible,
            abs.rate_excepted
        ),
        (Attribution::All, 19, 0, 1)
    );
    assert_eq!(v.excepted, 2);
    assert_eq!(v.stale_exceptions, ["probes/a!S!C1", "probes/a!S!Z9"]);
    // A line in a suite the run did not score (poi not fetched) is unchecked, not stale.
    assert_eq!(v.unchecked_exceptions, ["poi/b.xlsx!S!G9"]);
    let mut md = String::new();
    write_markdown(&v, &mut md);
    assert!(md.contains("1 listed cells were not checked: their suite was skipped"));
    assert!(md.contains("Warning: 2 listed cells are not scored"));
    assert!(
        md.contains("decision 0134, eval/milestones/phase1-exceptions.tsv): 2 visibly-off cells (")
            && md.contains("ABS 1")
            && md.contains("PI 1"),
        "{md}"
    );
    assert!(
        md.contains("Warning: 2 listed cells are not scored or not visibly off in this run; remove their lines:\n\n- probes/a!S!C1\n- probes/a!S!Z9\n"),
        "{md}"
    );
    let mut md = String::new();
    write_markdown(&none, &mut md);
    assert!(!md.contains("decision 0134"), "{md}");
}

#[test]
fn desktop_owned_cells_leave_the_phase1_rate_apart_from_listed_ones() {
    // Decision 0143: PI's two off cells are owned only by M14 tasks, so it meets the rate;
    // ABS has one off cell no M14 task alone owns (the generator leaves it out), so it does
    // not. A cell both lists hold counts once, as listed; a desktop-owned cell that passes is
    // neither left out nor stale.
    let catalog = Catalog::from_functions([func("PI", Some("test")), func("ABS", Some("test"))]);
    let mut cells = excel("PI()", 998);
    cells.push(at(off("PI()"), "B1"));
    cells.push(at(off("PI()"), "B2"));
    cells.push(at(off("PI()"), "B3"));
    cells.push(at(excel("PI()", 1).remove(0), "C1"));
    cells.extend(excel("ABS(1)", 998).into_iter().map(|c| at(c, "E1")));
    cells.push(at(off("ABS(1)"), "D1"));
    cells.push(at(off("ABS(1)"), "D2"));
    let mut r = case("probes/a", 1);
    r.cells = cells;
    let results = [r];
    let report = RunReport::build("e", &results, &[], &catalog);
    let lists = Phase1Exceptions::parse("probes/a!S!B3\t0134\tT-1075\n")
        .and_then(|e| {
            e.with_desktop_owned(
                "probes/a!S!B1\tT-2425\nprobes/a!S!B2\tT-2425\nprobes/a!S!B3\tT-1075\n\
                 probes/a!S!C1\tT-2425\nprobes/a!S!D1\tT-2425\n",
            )
        })
        .expect("parses");
    let v = summarize(&catalog, &results, &report, &[], &lists);
    let row = |n: &str| v.functions.iter().find(|f| f.name == n).cloned();
    let pi = row("PI").unwrap_or_default();
    assert_eq!(
        (
            pi.rate_cells,
            pi.rate_visible,
            pi.rate_excepted,
            pi.desktop_owned
        ),
        (999, 0, 1, 2)
    );
    assert!(pi.meets_rate);
    assert_eq!((pi.cells, pi.visible), (1002, 3));
    let abs = row("ABS").unwrap_or_default();
    assert_eq!(
        (abs.rate_cells, abs.rate_visible, abs.desktop_owned),
        (999, 1, 1)
    );
    assert!(!abs.meets_rate);
    assert_eq!((v.excepted, v.desktop_owned), (1, 3));
    assert!(v.stale_exceptions.is_empty(), "{:?}", v.stale_exceptions);
    let mut md = String::new();
    write_markdown(&v, &mut md);
    assert!(
        md.contains(
            "Left out of the Phase 1 rate as owned only by open M14 tasks (decision 0143, \
             eval/runs/desktop-owned.tsv): 3 visibly-off cells (PI 2, ABS 1); they stay known \
             failures.\n"
        ),
        "{md}"
    );
    assert!(
        md.contains("| ABS | 1000 | 0 | yes | 2 | 999 attributed | 1 | 1 | 99.90% | not met |"),
        "{md}"
    );
}
