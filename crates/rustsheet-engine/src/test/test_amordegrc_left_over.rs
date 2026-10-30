#![allow(clippy::unwrap_used)]
//! AMORDEGRC below period 1 returns the value the last AMORDEGRC or AMORLINC Excel ran before it
//! left, in Excel's fresh-load order (docs/excel-semantics/financial.md "AMORDEGRC below period
//! 1", decision 0131; the order is SPEC-0017's, `excel_order.rs`). Excel runs the cells bottom row
//! first and each row right to left.

use crate::test::util::new_empty_model;
use crate::Model;

const FRACTION: &str = "=AMORDEGRC(10000,39000,39400,500,0.3,0.3,0)";
/// 485, and it leaves 485.
const CHARGE_485: &str = "=AMORDEGRC(2400,39679,39813,300,2,0.15,1)";
/// 776, and it leaves 776.
const CHARGE_776: &str = "=AMORDEGRC(2400,39679,39813,300,1,0.15,1)";

fn evaluated(cells: &[(&str, &str)]) -> Model {
    let mut model = new_empty_model();
    for (cell, formula) in cells {
        model._set(cell, formula);
    }
    model.evaluate();
    model
}

/// The fraction's value in A1 with `below` in A2 and the 485 charge in A3.
fn above(below: &str) -> String {
    let model = evaluated(&[("A1", FRACTION), ("A2", below), ("A3", CHARGE_485)]);
    model._get_text("A1")
}

#[test]
fn alone_it_is_0_and_runs_one_pass_only_without_a_reader() {
    let model = evaluated(&[("A1", FRACTION)]);
    assert_eq!(model._get_text("A1"), "0");
    let model = evaluated(&[("A1", CHARGE_485), ("A2", CHARGE_776)]);
    assert_eq!(model.evaluation.passes_in_last_evaluation, 1);
    assert!(!model.evaluation.excel_order);
}

#[test]
fn it_reads_the_cell_excel_ran_last() {
    // probes/amordegrc-fraction-state-p, -q, -n and alone-e.
    assert_eq!(above(CHARGE_776), "776");
    let model = evaluated(&[("A1", FRACTION), ("B1", CHARGE_485)]);
    assert_eq!(model._get_text("A1"), "485");
    let model = evaluated(&[("A1", CHARGE_485), ("A2", FRACTION)]);
    assert_eq!(model._get_text("A2"), "0");
    let mut model = new_empty_model();
    model.new_sheet();
    model._set("A1", FRACTION);
    model._set("Sheet2!A1", CHARGE_485);
    model.evaluate();
    assert_eq!(model._get_text("A1"), "485");
}

#[test]
fn a_precedent_runs_first() {
    // probes/amordegrc-fraction-state-o.
    let model = evaluated(&[
        ("A1", CHARGE_485),
        ("A2", "=AMORDEGRC(10000,39000,39400,500,0.3+0*A1,0.3,0)"),
    ]);
    assert_eq!(model._get_text("A2"), "485");
}

#[test]
fn what_each_call_leaves() {
    // AMORLINC leaves its first period's charge rounded (131.803... and 360); a period-0
    // AMORDEGRC, an error and a period past the life leave the value as it was
    // (probes/amordegrc-fraction-state-j, -y, -z, -m, -u, -v).
    assert_eq!(above("=AMORLINC(2400,39679,39813,300,1,0.15,1)"), "132");
    assert_eq!(above("=AMORLINC(2400,39813,39813,300,1,0.15,1)"), "360");
    assert_eq!(above("=AMORDEGRC(46557,0,39881,440,0,0.1)"), "485");
    assert_eq!(above("=AMORDEGRC(2400,39679,39813,-1,2,0.15,1)"), "485");
    assert_eq!(above("=AMORDEGRC(2400,39679,39813,300,20,0.15,1)"), "485");
    assert_eq!(above("=AMORLINC(2400,39679,39813,300,20,0.15,1)"), "485");
    // Other functions leave it as it was (-aa, -ab).
    assert_eq!(above("=EFFECT(1,1)"), "485");
    assert_eq!(above("=SUM(1,2)"), "485");
}

#[test]
fn lifted_over_an_array_it_is_0() {
    // probes/amordegrc-fraction-state-x.
    let model = evaluated(&[
        (
            "A1",
            "=SUM(AMORDEGRC(10000,39000,39400,500,{0.3,0.5},0.3,0))",
        ),
        ("A2", CHARGE_485),
    ]);
    assert_eq!(model._get_text("A1"), "0");
}

#[test]
fn a_stopped_run_leaves_the_value_as_it_was() {
    // A2 leaves 776 and then reads A1, not yet run: that run stops and A1 must read A3's 485.
    let model = evaluated(&[
        ("A1", FRACTION),
        ("A2", &format!("{CHARGE_776}+0*A1")),
        ("A3", CHARGE_485),
    ]);
    assert_eq!(model._get_text("A1"), "485");
    assert_eq!(model._get_text("A2"), "776");
}

#[test]
fn it_shares_the_order_with_weeknum_but_not_the_state() {
    // A WEEKNUM reader and a fraction on one sheet: each reads its own state.
    let model = evaluated(&[
        ("A1", FRACTION),
        ("B1", "=WEEKNUM(100,99)"),
        ("A2", CHARGE_776),
        ("B2", "=WEEKNUM(100,2)"),
    ]);
    assert_eq!(model._get_text("A1"), "776");
    assert_eq!(model._get_text("B1"), "16");
}

/// A lifted call over `periods` in A1, summed or not.
fn lifted(periods: &str) -> Model {
    let formula = format!("=AMORDEGRC(2400,39679,39813,300,{periods},0.15,1)");
    let mut model = new_empty_model();
    model._set("A1", &formula);
    model.evaluate();
    model
}

#[test]
fn a_lifted_call_reads_what_its_own_earlier_elements_left() {
    // probes/amordegrc-fraction-state-ae and -af: {1,0.5} is 776 + 776, {0.5,1} is 0 + 776.
    let model = evaluated(&[("A1", "=SUM(AMORDEGRC(2400,39679,39813,300,{1,0.5},0.15,1))")]);
    assert_eq!(model._get_text("A1"), "1552");
    let model = evaluated(&[("A1", "=SUM(AMORDEGRC(2400,39679,39813,300,{0.5,1},0.15,1))")]);
    assert_eq!(model._get_text("A1"), "776");
}

#[test]
fn an_error_element_stays_an_error_beside_a_fraction() {
    let model = lifted("{-1,0.5}");
    assert_eq!(model._get_text("A1"), "#NUM!");
    assert_eq!(model._get_text("B1"), "0");
    let model = lifted("{1,-1,0.5}");
    assert_eq!(model._get_text("A1"), "776");
    assert_eq!(model._get_text("B1"), "#NUM!");
    assert_eq!(model._get_text("C1"), "776");
}
