#![allow(clippy::unwrap_used)]

//! The work budget of one formula evaluation (SPEC-0024; DEVIATION, docs/excel-semantics/arrays.md
//! "One formula evaluation does at most 268,435,456 work units"). Every test counts units
//! (`AreaBudget::last_spent`) or cell reads, never time (T-2282).

use crate::area_budget::{
    CELL_UNITS, LAMBDA_UNITS, MAX_FORMULA_WORK, NODE_UNITS, TEXT_BYTES_PER_UNIT, TOO_MUCH_WORK,
};
use crate::model::Model;
use crate::test::util::new_empty_model;

/// A model whose formulas each start from `limit` work units.
fn model_with_limit(limit: u64) -> Model {
    let mut model = new_empty_model();
    model.area_budget.limit = limit;
    model
}

/// The units `formula`, alone in A1, spends.
fn units(formula: &str) -> u64 {
    let mut model = new_empty_model();
    model._set("A1", formula);
    model.evaluate();
    model.area_budget.last_spent
}

#[test]
fn a_formula_spends_units_per_node_cell_built_and_text_byte() {
    const N: u64 = NODE_UNITS;
    const C: u64 = CELL_UNITS;
    // `1+2`: the sum and its two operands.
    assert_eq!(units("=1+2"), 3 * N);
    // SUM, SEQUENCE and 10: three nodes, and the ten cells SEQUENCE builds.
    assert_eq!(units("=SUM(SEQUENCE(10))"), 3 * N + 10 * C);
    // LEN, REPT, "a" (1 byte) and 640: four nodes, then the 640 bytes REPT gives.
    assert_eq!(
        units(r#"=LEN(REPT("a",640))"#),
        4 * N + 1 + 640 / TEXT_BYTES_PER_UNIT
    );
    // Each link of an operator chain is a node and is charged the array it builds: SUM, the
    // chain, SEQUENCE(10) and its 10 cells, the inner `+1` (its link, its operand and 10 cells),
    // the outer `+1` (its operand, and the chain's 10 cells).
    assert_eq!(units("=SUM(SEQUENCE(10)+1+1)"), 7 * N + 30 * C);
}

#[test]
fn a_lambda_made_is_charged_for_storing_it_and_each_body_clone() {
    const N: u64 = NODE_UNITS;
    // The call, the LAMBDA, its argument 1 and its body `x` are four nodes run; storing the LAMBDA
    // costs LAMBDA_UNITS plus its one-node body, and the call's patched copy of the body one more.
    assert_eq!(units("=LAMBDA(x,x)(1)"), LAMBDA_UNITS + 6 * N);
    // A body of three nodes costs three a clone.
    assert_eq!(units("=LAMBDA(x,x+1)(1)"), LAMBDA_UNITS + 12 * N);
}

#[test]
fn a_formula_at_its_limit_computes_and_one_unit_over_is_value_error() {
    let formula = "=SUM(SEQUENCE(10))";
    let limit = units(formula);
    let mut model = model_with_limit(limit);
    model._set("A1", formula);
    model.evaluate();
    assert_eq!(model._get_text("A1"), "55");
    let mut model = model_with_limit(limit - 1);
    model._set("A1", formula);
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#VALUE!");
    assert_eq!(model._get_error_message("A1"), TOO_MUCH_WORK);
}

#[test]
fn the_budget_error_cannot_be_caught() {
    let mut model = model_with_limit(100);
    model._set("A1", "=IFERROR(SUM(SEQUENCE(1000)),0)");
    model._set("A2", "=IF(ISERROR(SUM(SEQUENCE(1000))),0,1)");
    model._set("A3", "=IFERROR(SUM(SEQUENCE(10)),0)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#VALUE!");
    assert_eq!(model._get_error_message("A1"), TOO_MUCH_WORK);
    assert_eq!(model._get_text("A2"), "#VALUE!");
    assert_eq!(model._get_text("A3"), "55");
}

#[test]
fn a_dependency_has_its_own_budget_and_its_error_is_a_value() {
    let mut model = model_with_limit(1_000);
    // A1 runs out of its own budget, so its value is `#VALUE!`, which B1 may catch; C1, after
    // both, has a full budget.
    model._set("A1", "=SUM(SEQUENCE(1000))");
    model._set("B1", "=IFERROR(A1,0)");
    model._set("C1", "=SUM(SEQUENCE(50))");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#VALUE!");
    assert_eq!(model._get_text("B1"), "0");
    assert_eq!(model._get_text("C1"), "1275");
}

#[test]
fn past_the_limit_a_lambda_helper_makes_no_more_calls() {
    // Each call reads Z1 once and spends a few units, so with 1,000 units MAKEARRAY stops after
    // a few hundred of its million calls: the rest are refused without running.
    let mut model = model_with_limit(1_000);
    model._set("Z1", "1");
    model._set("A1", "=SUM(MAKEARRAY(1000,1000,LAMBDA(r,c,$Z$1+r)))");
    let before = model.evaluation.cell_reads;
    model.evaluate();
    let reads = model.evaluation.cell_reads - before;
    assert_eq!(model._get_text("A1"), "#VALUE!");
    assert_eq!(model._get_error_message("A1"), TOO_MUCH_WORK);
    assert!(reads < 1_000, "{reads} cell reads");
}

#[test]
fn the_problem_formulas_stop_at_the_limit() {
    // SPEC-0024's three formulas, under a lower limit so the test stays fast: each is `#VALUE!`
    // after spending the limit, not the 1E10 steps it would take to compute.
    const LIMIT: u64 = 1 << 20;
    for formula in [
        "=REDUCE(0,SEQUENCE(100000),LAMBDA(a,b,a+SUM(SEQUENCE(100000))))",
        "=SUM(MAP(SEQUENCE(1000),LAMBDA(x,SUM(MAP(SEQUENCE(1000),LAMBDA(y,SUM(SEQUENCE(1000))))))))",
        "=SUM(MAKEARRAY(1000,1000,LAMBDA(r,c,SUM(SEQUENCE(1000)))))",
        "=SUM(MAP(SEQUENCE(1000),LAMBDA(x,SUM(MAKEARRAY(1024,1024,LAMBDA(r,c,MAP(SEQUENCE(1),LAMBDA(z,r+c+z))))))))",
    ] {
        let mut model = model_with_limit(LIMIT);
        model._set("A1", formula);
        model.evaluate();
        assert_eq!(model._get_text("A1"), "#VALUE!", "{formula}");
        assert_eq!(model._get_error_message("A1"), TOO_MUCH_WORK, "{formula}");
        // Spent most of the limit: the last charge, at most one loop step's, did not fit.
        assert!(model.area_budget.last_spent > LIMIT / 2, "{formula}");
    }
}

#[test]
fn a_regex_call_that_runs_out_of_its_budget_is_charged_the_whole_budget() {
    // This search holds more than the call's 64 MiB (fn_regex_backtrack.rs
    // `the_visited_set_is_held_memory`): the call is `#ERROR!` and the formula pays for both whole
    // budgets, so a MAP of such calls reaches the formula's limit after a handful of them instead
    // of running each one (SPEC-0024).
    let call = r#"REGEXTEST("a"&CHAR(10)&REPT("a",32764)&CHAR(10),"(?:b[a-z]{20000}|a)*b$")"#;
    let mut model = new_empty_model();
    model._set("A1", &format!("={call}"));
    model._set(
        "A2",
        &format!("=SUM(MAP(SEQUENCE(1000),LAMBDA(x,IFERROR(--{call},0))))"),
    );
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#ERROR!");
    assert!(units(&format!("={call}")) > (1 << 25));
    assert_eq!(model._get_text("A2"), "#VALUE!");
    assert_eq!(model._get_error_message("A2"), TOO_MUCH_WORK);
}

#[test]
fn an_oversized_charge_fails_without_wrapping() {
    let mut model = new_empty_model();
    let cell = model._parse_reference("A1");
    assert!(model.spend_work(u64::MAX, cell).is_err());
    assert!(model.work_exceeded());
    // Once exceeded, every charge fails, even an empty one.
    assert!(model.spend_work(0, cell).is_err());
    assert!(model.spend_cells(usize::MAX, usize::MAX, cell).is_err());
}

#[test]
fn the_default_limit_is_two_to_the_28() {
    assert_eq!(MAX_FORMULA_WORK, 1 << 28);
    assert_eq!(new_empty_model().area_budget.limit, MAX_FORMULA_WORK);
}
