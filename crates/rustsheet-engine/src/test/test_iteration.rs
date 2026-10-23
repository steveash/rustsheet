#![allow(clippy::unwrap_used)]

//! Iterative calculation (docs/excel-semantics/recalc.md; probes/circular-iteration-*, T-0447).
//! The probes' values are for the oracle's three calculations; one `evaluate()` is one.

use crate::test::util::new_empty_model;
use crate::types::Iteration;
use crate::Model;

fn model_with(max_iterations: u32, max_change: f64, cells: &[(&str, &str)]) -> Model {
    let mut model = new_empty_model();
    model.workbook.settings.iteration = Some(Iteration {
        max_iterations,
        max_change,
    });
    for (cell, value) in cells {
        model._set(cell, value);
    }
    model
}

/// probes/circular-iteration-on: Gauss-Seidel sweeps in row-major order, resuming each
/// calculation from the previous values.
#[test]
fn cycles_sweep_in_sheet_order_and_resume() {
    let mut model = model_with(
        100,
        0.001,
        &[
            ("A1", "=A1+1"),
            ("A3", "=A1+10"),
            ("B1", "=C1+1"),
            ("C1", "=B1+1"),
            ("C2", "=D2+1"),
            ("D2", "=E2+1"),
            ("E2", "=C2+1"),
            ("D1", "=D1/2+1"),
            ("E1", "=MIN(E1+1,5)"),
            ("A2", "=IF(A2<50,A2+1,A2)"),
            ("B2", "=B2"),
        ],
    );
    for _ in 0..3 {
        model.evaluate();
    }
    let t = |c: &str| model._get_text(c);
    assert_eq!([t("B1"), t("C1")], ["599", "600"]);
    assert_eq!([t("C2"), t("D2"), t("E2")], ["449", "450", "450"]);
    assert_eq!([t("A1"), t("A3")], ["300", "310"]);
    assert_eq!([t("D1"), t("E1"), t("A2"), t("B2")], ["2", "5", "50", "0"]);
}

/// probes/circular-iteration-limits: iterateCount bounds every calculation.
#[test]
fn iterate_count_bounds_each_calculation() {
    let mut model = model_with(
        5,
        0.5,
        &[("A1", "=A1+1"), ("D1", "=D1+0.1"), ("F1", "=F1+0.6")],
    );
    for _ in 0..3 {
        model.evaluate();
    }
    assert_eq!(model._get_text("A1"), "15");
    assert_eq!(model._get_text("D1"), "1.5");
}

/// probes/circular-iteration-delta-boundary: a calculation stops at the first sweep whose largest
/// change is strictly below the delta; each later calculation runs one more sweep.
#[test]
fn the_delta_stops_strictly_below() {
    let mut model = model_with(100, 0.25, &[("A1", "=A1/2+1")]);
    for _ in 0..3 {
        model.evaluate();
    }
    assert_eq!(model._get_text("A1"), "1.96875");
}

/// Without iteration a cycle is still `#CIRC!` (T-0445 owns Excel's 0).
#[test]
fn without_iteration_a_cycle_is_circ() {
    let mut model = new_empty_model();
    model._set("A1", "=A1+1");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#CIRC!");
}
