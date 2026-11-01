use super::*;
use crate::accuracy::envelope::MathValue;
use crate::accuracy::lists::{R3Mark, R3Marks};
use crate::report::tests::{case, cell};
use crate::runner::CellResult;
use crate::value::Value;

fn pass(n: usize) -> Vec<CellResult> {
    let c = CellResult {
        from_excel: true,
        ..cell("PI()", Verdict::Pass)
    };
    vec![c; n]
}

fn off() -> CellResult {
    CellResult {
        from_excel: true,
        actual: Value::Number(2.0),
        ..cell("PI()", Verdict::NumericMismatch)
    }
}

fn pi(covered: bool) -> Vec<V15Function> {
    vec![V15Function {
        name: "PI".into(),
        covered,
        ..V15Function::default()
    }]
}

/// The three fetched suites Gate B needs, scored but empty.
fn fetched() -> Vec<CaseResult> {
    REQUIRED
        .iter()
        .map(|s| case(&format!("{s}/a"), 1))
        .collect()
}

#[test]
fn met_on_user_representative_cells_whatever_the_probes_say() {
    let mut g = case("gen-m5/a", 1);
    g.cells = pass(999);
    g.cells.push(off()); // one in a thousand is allowed (decision 0114)
    let mut probe = case("probes-numeric/sweep", 1);
    probe.cells = vec![off(); 50]; // research sweeps never count here
    let results = [vec![g, probe], fetched()].concat();
    let gate = summarize(
        &results,
        &Phase1Exceptions::default(),
        &pi(true),
        &R1cCells::default(),
    );
    assert!(gate.measured && gate.met, "{gate:?}");
    assert_eq!(gate.verdict(), "met");
    assert_eq!(
        (gate.functions[0].rate_cells, gate.functions[0].rate_visible),
        (1000, 1)
    );
    assert_eq!(gate.at_rate, 1);
    assert!(!gate.suites.contains(&"probes-numeric".to_string()));
    assert!(gate
        .line()
        .starts_with("Gate B (decision 0188 R1(b)): met (1/1 functions"));
}

#[test]
fn unmet_above_one_cell_in_a_thousand() {
    let mut g = case("ironcalc-calc/a", 1);
    g.cells = pass(998);
    g.cells.extend([off(), off()]);
    let results = [vec![g], fetched()].concat();
    let gate = summarize(
        &results,
        &Phase1Exceptions::default(),
        &pi(true),
        &R1cCells::default(),
    );
    assert!(gate.measured && !gate.met);
    assert_eq!(gate.verdict(), "not met");
    assert!(!gate.functions[0].meets_rate);
    assert_eq!(
        gate.functions[0].rate().map(|r| (r * 1e3).round()),
        Some(998.0)
    );
}

#[test]
fn not_measured_without_calipers_poi_or_corpus_bench() {
    let mut g = case("gen-m5/a", 1);
    g.cells = pass(1000);
    let results = [g, case("calipers/a", 1)];
    let gate = summarize(
        &results,
        &Phase1Exceptions::default(),
        &pi(true),
        &R1cCells::default(),
    );
    assert!(!gate.measured && !gate.met, "n/a is never met");
    assert_eq!(gate.missing, ["poi", "corpus-bench"]);
    assert_eq!(gate.verdict(), "n/a");
    assert!(gate
        .line()
        .ends_with("not measured: the run did not score poi, corpus-bench)"));
    assert!(gate.functions[0].meets_rate, "the rows are still counted");
}

#[test]
fn uncovered_or_unevidenced_functions_do_not_meet_it() {
    let mut g = case("gen-m5/a", 1);
    g.cells = pass(1000);
    let results = [vec![g], fetched()].concat();
    let uncovered = summarize(
        &results,
        &Phase1Exceptions::default(),
        &pi(false),
        &R1cCells::default(),
    );
    assert!(!uncovered.met && !uncovered.functions[0].meets_rate);
    // Probe cells only: no user-representative cell, no rate.
    let mut probe = case("probes/a", 1);
    probe.cells = pass(1000);
    let results = [vec![probe], fetched()].concat();
    let unevidenced = summarize(
        &results,
        &Phase1Exceptions::default(),
        &pi(true),
        &R1cCells::default(),
    );
    assert_eq!(unevidenced.functions[0].rate_cells, 0);
    assert!(!unevidenced.met && unevidenced.functions[0].rate().is_none());
}

#[test]
fn counted_suites() {
    for s in [
        "gen-m1",
        "gen-m8a",
        "ironcalc-calc",
        "corpus-bench",
        "calipers",
        "poi",
        "smoke",
        "fixtures",
    ] {
        assert!(counts(s), "{s}");
    }
    for s in ["probes", "probes-numeric", "accuracy", "generated"] {
        assert!(!counts(s), "{s}");
    }
}

#[test]
fn listed_cells_leave_the_rate_only_from_counted_suites() {
    let at = |address: &str| CellResult {
        cell: address.into(),
        ..off()
    };
    let mut g = case("gen-m5/a", 1);
    g.cells = pass(999);
    g.cells.extend([at("B1"), at("C1")]);
    let mut probe = case("probes/a", 1);
    probe.cells = vec![at("B1")];
    let results = [vec![g, probe], fetched()].concat();
    let none = summarize(
        &results,
        &Phase1Exceptions::default(),
        &pi(true),
        &R1cCells::default(),
    );
    assert_eq!(
        (none.functions[0].rate_cells, none.functions[0].rate_visible),
        (1001, 2)
    );
    assert!(!none.met);
    // The probes line is outside Gate B's suites: it must not take a gen-m5 cell with it.
    let listed =
        Phase1Exceptions::parse("gen-m5/a!S!B1\t0134\tT-1075\nprobes/a!S!B1\t0134\tT-1075\n")
            .expect("parses");
    let gate = summarize(&results, &listed, &pi(true), &R1cCells::default());
    assert_eq!(
        (gate.functions[0].rate_cells, gate.functions[0].rate_visible),
        (1000, 1)
    );
    assert!(gate.met);
}

/// An R3 cell where the engine returns the math counts as within 15 digits (decision 0188 R3);
/// the same cell without the mark, or with the engine off the math, stays visibly off.
#[test]
fn an_r3_cell_that_returns_the_math_passes() {
    let math = 0.25;
    let r3_cell = CellResult {
        from_excel: true,
        expected: Value::Number(0.26), // Excel 4% off the math
        actual: Value::Number(math),
        math: Some(MathValue::Number(math)),
        ..cell("PI()", Verdict::NumericMismatch)
    };
    let mut g = case("gen-m5/a", 1);
    g.cells = pass(99);
    g.cells.push(r3_cell);
    let results = [vec![g], fetched()].concat();
    let id = crate::runner::cell_id("gen-m5/a", &results[0].cells[99].key());
    let marks = R3Marks::from([(
        id,
        R3Mark {
            line: 1,
            semantics_file: "docs/excel-semantics/statistics.md".into(),
            anchor: "x".into(),
        },
    )]);
    let exceptions = Phase1Exceptions::default();
    let marked = R1cCells::build(&results, &marks, &exceptions);
    let gate = summarize(&results, &exceptions, &pi(true), &marked);
    assert_eq!(gate.functions[0].rate_visible, 0, "{gate:?}");
    assert!(gate.met);
    let unmarked = R1cCells::build(&results, &R3Marks::new(), &exceptions);
    let gate = summarize(&results, &exceptions, &pi(true), &unmarked);
    assert_eq!(gate.functions[0].rate_visible, 1);
    assert!(!gate.met);
}

#[test]
fn probe_cells_count_toward_the_random_functions_only() {
    let excel = |formula: &str| CellResult {
        from_excel: true,
        ..cell(formula, Verdict::Pass)
    };
    let mut probe = case("probes/t3631", 1);
    probe.cells = [
        vec![excel("RANDBETWEEN(5,5)"); 20],
        vec![excel("ROWS(_xlfn.RANDARRAY(3,2))"); 20],
        vec![excel("AND(RAND()>=0,RAND()<1)"); 20],
        vec![excel("PI()"); 20],
    ]
    .concat();
    let mut sweep = case("probes-numeric/sweep", 1);
    sweep.cells = vec![excel("RAND()*0"); 50];
    let results = [vec![probe, sweep], fetched()].concat();
    let functions: Vec<V15Function> = ["PI", "RAND", "RANDARRAY", "RANDBETWEEN", "ROWS"]
        .into_iter()
        .map(|name| V15Function {
            name: name.into(),
            covered: true,
            ..V15Function::default()
        })
        .collect();
    let gate = summarize(
        &results,
        &Phase1Exceptions::default(),
        &functions,
        &R1cCells::default(),
    );
    let row = |name: &str| {
        gate.functions
            .iter()
            .find(|f| f.name == name)
            .map(|f| (f.rate_cells, f.meets_rate))
    };
    // Decision 0225: the probe counts toward the three random functions (RANDARRAY by every
    // call, below the evidence floor of sole calls), never toward PI or ROWS.
    assert_eq!(row("RANDBETWEEN"), Some((20, true)));
    assert_eq!(row("RAND"), Some((20, true)), "probes-numeric never counts");
    assert_eq!(row("RANDARRAY"), Some((20, true)));
    assert_eq!(row("ROWS"), Some((0, false)));
    assert_eq!(row("PI"), Some((0, false)));
    assert!(!gate.suites.contains(&"probes".to_string()));
}

#[test]
fn only_the_three_random_functions_are_read_on_probes() {
    // Decision 0225 covers exactly these; another function needs its own decision record.
    assert_eq!(PROBED, ["RAND", "RANDARRAY", "RANDBETWEEN"]);
    assert_eq!(PROBE_SUITE, "probes");
}
