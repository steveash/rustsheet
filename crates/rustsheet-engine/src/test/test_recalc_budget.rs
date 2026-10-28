#![allow(clippy::unwrap_used, reason = "tests")]

//! The work budget of one recalculation (SPEC-0028; DEVIATION, docs/excel-semantics/recalc.md
//! "One recalculation does at most 2^32 work units, and a pass past 2^30 shares them out"). Every
//! test counts units with lowered limits (`Evaluation::recalc_limits`), never time (T-2282).

use crate::area_budget::{CELL_UNITS, MAX_FORMULA_WORK, NODE_UNITS, TOO_MUCH_WORK};
use crate::recalc_budget::{
    share, RecalcBudget, RecalcMode, MAX_PASS_WORK, MAX_RECALC_WORK, RECALC_EXHAUSTED,
    TOO_MUCH_RECALC_WORK,
};
use crate::test::util::new_empty_model;
use crate::types::Iteration;
use crate::Model;

/// The units of `=SUM(SEQUENCE(k))`: three nodes and the `k` cells SEQUENCE builds.
fn sum_units(k: u64) -> u64 {
    3 * NODE_UNITS + k * CELL_UNITS
}

/// A model with `cells` set, whose recalculations have a pass allowance of `pass` units and a
/// ceiling of `ceiling` units.
fn model_with(pass: u64, ceiling: u64, cells: &[(&str, &str)]) -> Model {
    let mut model = new_empty_model();
    model.evaluation.recalc_limits = (pass, ceiling);
    for (cell, value) in cells {
        model._set(cell, value);
    }
    model
}

/// The budget of the model's last recalculation.
fn last(model: &Model) -> &RecalcBudget {
    model.evaluation.last_recalc.as_ref().unwrap()
}

/// Every cell's text, in the order given.
fn texts(model: &Model, cells: &[&str]) -> Vec<String> {
    cells.iter().map(|cell| model._get_text(cell)).collect()
}

const FOUR_SUMS: [(&str, &str); 5] = [
    ("A1", "=SUM(SEQUENCE(100))"),
    ("A2", "=SUM(SEQUENCE(100))"),
    ("A3", "=SUM(SEQUENCE(100))"),
    ("A4", "=SUM(SEQUENCE(100))"),
    ("B1", "=1+2"),
];

#[test]
fn the_default_numbers_are_powers_of_two_and_multiples_of_the_formula_budget() {
    assert_eq!(MAX_PASS_WORK, 4 * MAX_FORMULA_WORK);
    assert_eq!(MAX_RECALC_WORK, 4 * MAX_PASS_WORK);
    assert_eq!(
        new_empty_model().evaluation.recalc_limits,
        (MAX_PASS_WORK, MAX_RECALC_WORK)
    );
}

#[test]
fn a_pass_at_its_allowance_computes_and_one_unit_more_shares() {
    let total = 4 * sum_units(100) + 3 * NODE_UNITS;
    let mut model = model_with(total, MAX_RECALC_WORK, &FOUR_SUMS);
    model.evaluate();
    assert_eq!(last(&model).mode, RecalcMode::Normal);
    assert_eq!(last(&model).committed, total);
    assert_eq!(last(&model).spent(), total);
    assert_eq!(texts(&model, &["A1", "A4", "B1"]), ["5050", "5050", "3"]);

    let mut model = model_with(total - 1, MAX_RECALC_WORK, &FOUR_SUMS);
    model.evaluate();
    // Five formula cells: each may spend (total - 1) / 20 units, enough for `1+2` only.
    let share = (total - 1) / 20;
    assert!(3 * NODE_UNITS <= share && share < sum_units(100));
    assert_eq!(last(&model).mode, RecalcMode::Share(share));
    for cell in ["A1", "A2", "A3", "A4"] {
        assert_eq!(model._get_text(cell), "#VALUE!", "{cell}");
        assert_eq!(
            model._get_error_message(cell),
            TOO_MUCH_RECALC_WORK,
            "{cell}"
        );
    }
    assert_eq!(model._get_text("B1"), "3");
    // The next recalculation starts in normal mode again.
    model.evaluation.recalc_limits = (total, MAX_RECALC_WORK);
    model.evaluate();
    assert_eq!(model._get_text("A1"), "5050");
    assert!(model.evaluation.recalc.is_none());
}

#[test]
fn the_share_is_at_least_one_unit_and_at_most_the_formula_budget() {
    assert_eq!(share(1 << 30, 100_000, 1 << 28), 2_684);
    assert_eq!(share(1 << 30, 1_000, 1 << 28), 268_435);
    assert_eq!(share(1 << 30, 1, 1 << 20), 1 << 20);
    assert_eq!(share(10, u64::MAX, 1 << 28), 1);
    assert_eq!(share(10, 0, 1 << 28), 2);
}

/// Two heavy anchors and six light ones, in the given columns: past the allowance, the light
/// ones fit their share and the heavy ones do not, whatever the order.
fn mixed_anchors(columns: &[&str]) -> Vec<(String, &'static str)> {
    columns
        .iter()
        .enumerate()
        .map(|(i, column)| {
            let formula = if i < 2 {
                "=SEQUENCE(1000)"
            } else {
                "=SEQUENCE(2)"
            };
            (format!("{column}1"), formula)
        })
        .collect()
}

fn evaluate_mixed(columns: &[&str], pass: u64, reverse_anchors: bool) -> (Model, Vec<String>) {
    let cells = mixed_anchors(columns);
    let cells: Vec<(&str, &str)> = cells.iter().map(|(c, f)| (c.as_str(), *f)).collect();
    let mut model = model_with(pass, MAX_RECALC_WORK, &cells);
    model.evaluation.recalc_limits = (MAX_PASS_WORK, MAX_RECALC_WORK);
    model.evaluate();
    if reverse_anchors {
        model.evaluation.anchor_order.reverse();
    }
    model.evaluation.recalc_limits = (pass, MAX_RECALC_WORK);
    model.evaluate();
    // Each formula's anchor and second row, in the order of `mixed_anchors`.
    let values = columns
        .iter()
        .flat_map(|column| [format!("{column}1"), format!("{column}2")])
        .map(|cell| model._get_text(&cell))
        .collect();
    (model, values)
}

#[test]
fn which_cells_give_up_does_not_depend_on_the_order() {
    let natural = ["A", "B", "C", "D", "E", "F", "G", "H"];
    let moved = ["H", "C", "A", "F", "B", "D", "G", "E"];
    let (model, _) = evaluate_mixed(&natural, MAX_PASS_WORK, false);
    let total = last(&model).committed;
    for pass in [total, total - 1] {
        let (first, values) = evaluate_mixed(&natural, pass, false);
        for (columns, reverse) in [(&natural, true), (&moved, false), (&moved, true)] {
            let (model, other) = evaluate_mixed(columns, pass, reverse);
            assert_eq!(
                values, other,
                "pass {pass}, {columns:?}, reversed {reverse}"
            );
            assert_eq!(last(&first).mode, last(&model).mode);
            assert_eq!(last(&first).committed, last(&model).committed);
        }
        if pass == total {
            assert_eq!(last(&first).mode, RecalcMode::Normal);
            assert_eq!(values[0..2], ["1", "2"]);
        } else {
            assert_eq!(last(&first).mode, RecalcMode::Share((total - 1) / 32));
            // The heavy anchors give up and spill nothing; the light ones compute.
            assert_eq!(values[0..4], ["#VALUE!", "", "#VALUE!", ""]);
            assert_eq!(values[4..6], ["1", "2"]);
            assert_eq!(values[14..16], ["1", "2"]);
        }
    }
}

#[test]
fn a_formula_past_its_share_cannot_catch_it_but_its_reader_can() {
    let mut cells = vec![
        ("A1", "=IFERROR(SUM(SEQUENCE(10000)),0)"),
        ("B1", "=IFERROR(A1,0)"),
    ];
    let lights = ["C1", "C2", "C3", "C4", "C5", "C6", "C7", "C8"];
    cells.extend(lights.iter().map(|cell| (*cell, "=1+2")));
    let mut model = model_with(1000, MAX_RECALC_WORK, &cells);
    model.evaluate();
    assert_eq!(last(&model).mode, RecalcMode::Share(25));
    assert_eq!(model._get_text("A1"), "#VALUE!");
    assert_eq!(model._get_error_message("A1"), TOO_MUCH_RECALC_WORK);
    assert_eq!(model._get_text("B1"), "0");
    assert_eq!(model._get_text("C8"), "3");
}

#[test]
fn a_formula_past_its_own_budget_in_normal_mode_keeps_its_message() {
    let mut model = model_with(
        MAX_PASS_WORK,
        MAX_RECALC_WORK,
        &[("A1", "=SUM(SEQUENCE(10))")],
    );
    model.area_budget.limit = sum_units(10) - 1;
    model.evaluate();
    assert_eq!(last(&model).mode, RecalcMode::Normal);
    assert_eq!(model._get_error_message("A1"), TOO_MUCH_WORK);
}

#[test]
fn a_dependency_run_draws_on_the_same_allowance() {
    // `B1+1` is three nodes; its dependency B1 runs inside it, with a budget of its own.
    let outer = 3 * NODE_UNITS;
    let total = outer + sum_units(100);
    for (dependency, reader) in [("B1", "A1"), ("A1", "B1")] {
        let reader_formula = format!("={dependency}+1");
        let cells = [
            (dependency, "=SUM(SEQUENCE(100))"),
            (reader, reader_formula.as_str()),
        ];
        let mut model = model_with(total, MAX_RECALC_WORK, &cells);
        model.evaluate();
        assert_eq!(last(&model).spent(), total, "{reader} reads {dependency}");
        assert_eq!(last(&model).committed, total);
        assert_eq!(model._get_text(reader), "5051");

        let mut model = model_with(total - 1, MAX_RECALC_WORK, &cells);
        model.evaluate();
        assert!(matches!(last(&model).mode, RecalcMode::Share(_)));
        assert_eq!(model._get_text(dependency), "#VALUE!");
    }
}

/// Three pairs of anchors where the first of each reads the second's spill: on a fresh sheet each
/// pair restarts the pass once.
const RESTARTING: [(&str, &str); 6] = [
    ("A1", "=SEQUENCE(1,1,B3+SUM(SEQUENCE(100)))"),
    ("B1", "=SEQUENCE(3)"),
    ("C1", "=SEQUENCE(1,1,D3+SUM(SEQUENCE(100)))"),
    ("D1", "=SEQUENCE(3)"),
    ("E1", "=SEQUENCE(1,1,F3+SUM(SEQUENCE(100)))"),
    ("F1", "=SEQUENCE(3)"),
];

#[test]
fn restarted_passes_are_thrown_away_and_past_the_allowance_share() {
    let mut model = model_with(MAX_PASS_WORK, MAX_RECALC_WORK, &RESTARTING);
    model.evaluate();
    assert!(model.evaluation.restarts_in_last_evaluation >= 3);
    let committed = last(&model).committed;
    let thrown_away = last(&model).thrown_away;
    assert!(
        thrown_away > committed,
        "{thrown_away} thrown away, {committed} committed"
    );
    assert_eq!(last(&model).spent(), committed + thrown_away);
    assert_eq!(model._get_text("A1"), "5053");

    // The same sheet again: the learned order needs no restart, and the pass commits the same.
    model.evaluate();
    assert_eq!(model.evaluation.restarts_in_last_evaluation, 0);
    assert_eq!(last(&model).committed, committed);
    assert_eq!(last(&model).thrown_away, 0);

    // A fresh sheet whose allowance fits a pass but not what the restarts threw away.
    let mut model = model_with(committed, MAX_RECALC_WORK, &RESTARTING);
    model.evaluate();
    assert!(matches!(last(&model).mode, RecalcMode::Share(_)));
    assert_eq!(model._get_text("A1"), "#VALUE!");
    assert_eq!(model._get_text("B3"), "3");
}

#[test]
fn runs_cut_short_by_the_depth_limit_are_thrown_away() {
    let mut cells: Vec<(String, String)> = (1..=20)
        .map(|row| (format!("A{row}"), format!("=A{}+1", row + 1)))
        .collect();
    cells.push(("A21".to_string(), "=SUM(SEQUENCE(100))".to_string()));
    let cells: Vec<(&str, &str)> = cells
        .iter()
        .map(|(c, f)| (c.as_str(), f.as_str()))
        .collect();

    let mut deep = model_with(MAX_PASS_WORK, MAX_RECALC_WORK, &cells);
    deep.evaluate();
    assert_eq!(last(&deep).thrown_away, 0);

    let mut shallow = model_with(MAX_PASS_WORK, MAX_RECALC_WORK, &cells);
    shallow.set_max_evaluation_depth(3);
    shallow.evaluate();
    assert_eq!(shallow._get_text("A1"), deep._get_text("A1"));
    // What the unwound runs spent before they were cut short is thrown away; the pass commits
    // what a pass without the limit commits.
    assert!(last(&shallow).thrown_away > 0);
    assert_eq!(last(&shallow).committed, last(&deep).committed);
    assert_eq!(
        last(&shallow).spent(),
        last(&shallow).committed + last(&shallow).thrown_away
    );
}

#[test]
fn past_the_ceiling_every_formula_is_value_error_and_spills_are_dropped() {
    let cells = [
        ("A1", "=SUM(SEQUENCE(100))"),
        ("A2", "=1+2"),
        ("B1", "=SEQUENCE(3)"),
        ("C1", "=A1+1"),
    ];
    let mut model = model_with(MAX_PASS_WORK, MAX_RECALC_WORK, &cells);
    model.evaluate();
    assert_eq!(model._get_text("B3"), "3");

    model.evaluation.recalc_limits = (MAX_PASS_WORK, 100);
    model.evaluate();
    assert_eq!(last(&model).mode, RecalcMode::Exhausted);
    assert!(last(&model).spent() <= 100);
    for cell in ["A1", "A2", "B1", "C1"] {
        assert_eq!(model._get_text(cell), "#VALUE!", "{cell}");
        assert_eq!(model._get_error_message(cell), RECALC_EXHAUSTED, "{cell}");
    }
    assert_eq!(model._get_text("B2"), "");
    assert_eq!(model._get_text("B3"), "");
}

#[test]
fn an_exhausted_recalculation_does_not_iterate() {
    let mut model = model_with(MAX_PASS_WORK, 10, &[("A1", "=A1+1"), ("B1", "=1+2")]);
    model.workbook.settings.iteration = Some(Iteration {
        max_iterations: 100,
        max_change: 0.001,
    });
    model.evaluate();
    assert_eq!(last(&model).mode, RecalcMode::Exhausted);
    assert_eq!(model._get_error_message("A1"), RECALC_EXHAUSTED);
    assert_eq!(model._get_error_message("B1"), RECALC_EXHAUSTED);
}

/// `A1` iterates on itself, `B1` does not: each run of either is 18 units.
fn sweeping(pass: u64, ceiling: u64) -> Model {
    let mut model = model_with(pass, ceiling, &[("A1", "=A1+1"), ("B1", "=1+2")]);
    model.workbook.settings.iteration = Some(Iteration {
        max_iterations: 100,
        max_change: 0.001,
    });
    model.evaluate();
    model
}

#[test]
fn sweeps_draw_on_the_ceiling_and_not_on_the_pass_allowance() {
    let run = 3 * NODE_UNITS;
    // The pass commits both formulas; the 100 sweep runs are spent but not committed, so an
    // allowance of one pass is enough.
    let model = sweeping(2 * run, MAX_RECALC_WORK);
    assert_eq!(last(&model).mode, RecalcMode::Normal);
    assert_eq!(last(&model).committed, 2 * run);
    assert_eq!(last(&model).spent(), 2 * run + 100 * run);
    assert_eq!(model._get_text("A1"), "100");
}

#[test]
fn past_the_ceiling_the_sweeps_stop_where_they_are() {
    let run = 3 * NODE_UNITS;
    // Ten sweep runs fit; the eleventh runs out and stores nothing: A1 keeps the value the tenth
    // gave it, as when `MAX_SWEEP_EVALUATIONS` runs out.
    for ceiling in [12 * run, 13 * run - 1] {
        let model = sweeping(MAX_PASS_WORK, ceiling);
        assert_eq!(last(&model).mode, RecalcMode::Normal, "{ceiling}");
        assert!(last(&model).spent() <= ceiling, "{ceiling}");
        assert_eq!(model._get_text("A1"), "10", "{ceiling}");
        assert_eq!(model._get_text("B1"), "3", "{ceiling}");
    }
}

/// A column table B2:B5 over `B1` (`=A1*10`) with input A1, and a reader of its results: the
/// pass is 30 units, the header read and each substitution 18.
fn tabled(ceiling: u64) -> Model {
    use crate::types::{DataTable, DataTableInput};
    let cells = [
        ("A1", "1"),
        ("B1", "=A1*10"),
        ("A2", "1"),
        ("A3", "2"),
        ("A4", "3"),
        ("A5", "4"),
        ("C1", "=SUM(B2:B5)"),
    ];
    let mut model = model_with(MAX_PASS_WORK, ceiling, &cells);
    let table = DataTable {
        range: (2, 2, 5, 2),
        row_input: None,
        column_input: Some(DataTableInput::Cell(1, 1)),
    };
    model.add_data_table(0, table).unwrap();
    model.evaluate();
    model
}

#[test]
fn past_the_ceiling_this_and_later_data_table_results_are_num_errors() {
    let run = 3 * NODE_UNITS;
    let model = tabled(MAX_RECALC_WORK);
    assert_eq!(last(&model).mode, RecalcMode::Normal);
    let results = ["B2", "B3", "B4", "B5"];
    assert_eq!(texts(&model, &results), ["10", "20", "30", "40"]);
    assert_eq!(model._get_text("C1"), "100");
    // The table's runs are spent but not committed: the second round's pass commits 30.
    assert_eq!(last(&model).committed, 30);
    assert_eq!(last(&model).spent(), 30 + 5 * run + 30);

    // Two substitutions fit after the pass and the header read; the third runs out, so it and the
    // fourth are `#NUM!`. The second round of passes then has nothing left: exhausted.
    let model = tabled(30 + 3 * run + run - 1);
    assert_eq!(texts(&model, &results), ["10", "20", "#NUM!", "#NUM!"]);
    assert_eq!(last(&model).mode, RecalcMode::Exhausted);
    assert_eq!(model._get_error_message("C1"), RECALC_EXHAUSTED);

    // Out of it at the header read: every result is `#NUM!`.
    let model = tabled(30 + run - 1);
    assert_eq!(texts(&model, &results), ["#NUM!"; 4]);
}

#[test]
fn formulas_outside_a_recalculation_draw_on_no_pool() {
    let mut model = model_with(10, 10, &[("A1", "=1")]);
    model.evaluate();
    assert!(model.evaluation.recalc.is_none());
    assert_eq!(
        model.evaluate_formula("=SUM(SEQUENCE(100))", 0),
        Some(5050.0)
    );
}

/// SPEC-0028 section 5's worked example, scaled down by 2^14: a formula budget of 2^14 units, an
/// allowance of four and a ceiling of sixteen, and heavy cells of 93% of the formula budget.
fn worked_example(heavy: usize, light: usize) -> Model {
    const LIMIT: u64 = 1 << 14;
    let mut model = model_with(4 * LIMIT, 16 * LIMIT, &[]);
    model.area_budget.limit = LIMIT;
    // 3 nodes and 5,073 cells: 15,237 units, 93% of the limit.
    for row in 1..=heavy {
        model._set(&format!("A{row}"), "=SUM(SEQUENCE(5073))");
    }
    for row in 1..=light {
        model._set(&format!("B{row}"), &format!("=C{row}*2"));
    }
    model.evaluate();
    model
}

#[test]
fn the_worked_example() {
    const LIMIT: u64 = 1 << 14;
    let heavy = sum_units(5073);
    assert_eq!(heavy, 15_237);

    // A1:A4: four heavy cells fit the allowance; every value computed.
    let model = worked_example(4, 0);
    assert_eq!(last(&model).mode, RecalcMode::Normal);
    assert_eq!(last(&model).spent(), 4 * heavy);
    assert_eq!(model._get_text("A4"), "12870201");

    // A1:A5: the fifth overruns; every heavy cell gives up at its share.
    let model = worked_example(5, 0);
    let share = 4 * LIMIT / 20;
    assert_eq!(last(&model).mode, RecalcMode::Share(share));
    for row in 1..=5 {
        assert_eq!(model._get_text(&format!("A{row}")), "#VALUE!");
    }
    assert!(last(&model).spent() <= 4 * LIMIT + 5 * share);

    // A1:A100 and B1:B700: the light cells compute, the heavy cells give up, at most a quarter of
    // the allowance spent by the share pass.
    let model = worked_example(100, 700);
    let share = 4 * LIMIT / 3200;
    assert_eq!(last(&model).mode, RecalcMode::Share(share));
    assert_eq!(model._get_text("A1"), "#VALUE!");
    assert_eq!(model._get_text("A100"), "#VALUE!");
    assert_eq!(model._get_text("B700"), "0");
    assert!(last(&model).committed <= LIMIT);
    assert!(last(&model).spent() <= 4 * LIMIT + LIMIT);
}

/// The heavy formula of the timing below: `iterations` LAMBDA calls of a 10,000-cell SUM.
fn heavy_formula(iterations: u64) -> String {
    format!("=SUM(MAP(SEQUENCE({iterations}),LAMBDA(i,SUM(SEQUENCE(10000)))))")
}

/// SPEC-0028 section 5's worked example at the default numbers, timed for recalc.md's DEVIATION
/// entry: `cargo test --release -p rustsheet_engine worked_example_timing -- --ignored --nocapture`.
#[test]
#[ignore = "a timing, not a check"]
fn worked_example_timing() {
    let units = |iterations: u64| {
        let mut model = new_empty_model();
        model._set("A1", &heavy_formula(iterations));
        model.evaluate();
        model.area_budget.last_spent
    };
    let (one, two) = (units(1), units(2));
    // 250,000,000 units, 93% of the formula budget.
    let iterations = (250_000_000 - one) / (two - one) + 1;
    let heavy = heavy_formula(iterations);
    println!("heavy formula {heavy}: {} units", units(iterations));
    for (heavy_cells, light_cells) in [(4, 0), (5, 0), (1000, 0), (1000, 99_000)] {
        let mut model = new_empty_model();
        for row in 1..=heavy_cells {
            model._set(&format!("A{row}"), &heavy);
        }
        for row in 1..=light_cells {
            model._set(&format!("B{row}"), &format!("=C{row}*2"));
        }
        let start = std::time::Instant::now();
        model.evaluate();
        let elapsed = start.elapsed();
        println!(
            "A1:A{heavy_cells}, {light_cells} light: {:?}, {} units spent, {:?}, A1 {}",
            elapsed,
            last(&model).spent(),
            last(&model).mode,
            model._get_text("A1"),
        );
    }
}
