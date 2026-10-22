#![allow(clippy::unwrap_used)]
//! WEEKNUM's stale `return_type` state and the pass in Excel's fresh-load order (SPEC-0017,
//! docs/excel-semantics/dates.md "WEEKNUM's stale `return_type` state"). A reader is
//! `WEEKNUM(100,99)`: 16 after a Monday start, `SUNDAY` below after a Sunday one, `#NUM!` when
//! the state is empty. Excel runs the cells bottom row first and each row right to left.

use crate::test::util::new_empty_model;
use crate::Model;

const READER: &str = "=WEEKNUM(100,99)";
const MONDAY: &str = "16";
const EMPTY: &str = "#NUM!";

/// What a formula gives on a sheet of its own.
fn alone(formula: &str) -> String {
    let mut model = new_empty_model();
    model._set("A1", formula);
    model.evaluate();
    model._get_text("A1")
}

fn sunday() -> String {
    alone("=WEEKNUM(100,1)")
}

fn evaluated(cells: &[(&str, &str)]) -> Model {
    let mut model = new_empty_model();
    for (cell, formula) in cells {
        model._set(cell, formula);
    }
    model.evaluate();
    model
}

#[test]
fn an_empty_state_reads_num() {
    assert_eq!(alone(READER), EMPTY);
    assert_eq!(alone("=WEEKNUM(100,-1)"), EMPTY);
    // A reader leaves the state as it is: both reads in one formula see it.
    assert_eq!(
        evaluated(&[
            ("A1", "=WEEKNUM(100,99)&WEEKNUM(100,-5)"),
            ("A2", "=WEEKNUM(1,2)")
        ])
        ._get_text("A1"),
        "1616"
    );
}

#[test]
fn the_bottom_row_and_the_rightmost_column_run_first() {
    // The setter below runs first, the one above runs after the reader.
    assert_eq!(
        evaluated(&[("A1", READER), ("A2", "=WEEKNUM(1,2)")])._get_text("A1"),
        MONDAY
    );
    assert_eq!(
        evaluated(&[("A1", "=WEEKNUM(1,2)"), ("A2", READER)])._get_text("A2"),
        EMPTY
    );
    assert_eq!(
        evaluated(&[("A1", READER), ("B1", "=WEEKNUM(1,2)")])._get_text("A1"),
        MONDAY
    );
    assert_eq!(
        evaluated(&[("A1", "=WEEKNUM(1,2)"), ("B1", READER)])._get_text("B1"),
        EMPTY
    );
}

#[test]
fn the_last_sheet_runs_first() {
    let mut model = new_empty_model();
    model.add_sheet("Second").unwrap();
    model._set("A1", READER);
    model._set("Second!A1", "=ISOWEEKNUM(1)");
    model._set("Second!A2", READER);
    model.evaluate();
    assert_eq!(model._get_text("A1"), MONDAY);
    assert_eq!(model._get_text("Second!A2"), EMPTY);
}

#[test]
fn which_calls_set_empty_or_leave_the_state() {
    let sunday = sunday();
    // (call under test, the marker that runs before it, what the reader above them reads)
    let cases: [(&str, &str, &str); 22] = [
        ("=WEEKNUM(1)", "=WEEKNUM(1,2)", &sunday),
        ("=WEEKNUM(1,1)", "=WEEKNUM(1,2)", &sunday),
        ("=WEEKNUM(1,17)", "=WEEKNUM(1,2)", &sunday),
        ("=WEEKNUM(1,2)", "=WEEKNUM(1,1)", MONDAY),
        ("=WEEKNUM(1,11)", "=WEEKNUM(1,1)", MONDAY),
        ("=WEEKNUM(1,21)", "=WEEKNUM(1,1)", MONDAY),
        ("=WEEKNUM(1,\"2\")", "=WEEKNUM(1,1)", MONDAY),
        ("=WEEKNUM(1,2.9)", "=WEEKNUM(1,1)", MONDAY),
        ("=WEEKNUM(0,2)", "=WEEKNUM(1,1)", MONDAY),
        ("=ISOWEEKNUM(0)", "=WEEKNUM(1,1)", MONDAY),
        // Emptying: Tuesday to Saturday starts, the always-#NUM! types, a lifted call.
        ("=WEEKNUM(1,12)", "=WEEKNUM(1,2)", EMPTY),
        ("=WEEKNUM(1,16)", "=WEEKNUM(1,2)", EMPTY),
        ("=WEEKNUM(1,0)", "=WEEKNUM(1,2)", EMPTY),
        ("=WEEKNUM(1,3)", "=WEEKNUM(1,2)", EMPTY),
        ("=WEEKNUM(1,20)", "=WEEKNUM(1,2)", EMPTY),
        ("=SUM(WEEKNUM(100,{2,2}))", "=WEEKNUM(1,2)", EMPTY),
        ("=SUM(WEEKNUM({100,101},2))", "=WEEKNUM(1,2)", EMPTY),
        // No effect: an error serial, a #VALUE! return_type, WEEKDAY, a call IF does not run.
        ("=WEEKNUM(-1,2)", "=WEEKNUM(1,1)", &sunday),
        ("=ISOWEEKNUM(-1)", "=WEEKNUM(1,1)", &sunday),
        ("=WEEKNUM(1,TRUE)", "=WEEKNUM(1,2)", MONDAY),
        ("=WEEKDAY(1,2)", "=WEEKNUM(1,1)", &sunday),
        ("=IF(FALSE,WEEKNUM(1,2),0)", "=WEEKNUM(1,1)", &sunday),
    ];
    for (call, marker, expected) in cases {
        let model = evaluated(&[("A1", READER), ("A2", call), ("A3", marker)]);
        assert_eq!(model._get_text("A1"), expected, "{call} after {marker}");
    }
}

#[test]
fn a_run_stopped_at_an_uncalculated_read_has_no_effect() {
    // A3 sets Monday, then reads A1, not calculated yet: its run stops with no effect, and A2
    // reads nothing (probes/weeknum-stale-precedent-abort-keeps).
    let cells = [("A1", "=ROW()"), ("A2", READER), ("A3", "=WEEKNUM(1,2)&A1")];
    let model = evaluated(&cells);
    assert_eq!(model._get_text("A2"), EMPTY);
    assert_eq!(model._get_text("A3"), "11");
    // With a constant in A1 nothing stops.
    let cells = [("A1", "1"), ("A2", READER), ("A3", "=WEEKNUM(1,2)&A1")];
    assert_eq!(evaluated(&cells)._get_text("A2"), MONDAY);
}

#[test]
fn a_stopped_cell_runs_again_just_after_its_precedent() {
    // The order is A2, A1, A3, not A1, A3, A2 (probes/weeknum-stale-precedent-between).
    let sunday = sunday();
    let cells = [
        ("A1", "=WEEKNUM(1,2)"),
        ("A2", "=IFERROR(WEEKNUM(100,99),\"none\")&WEEKNUM(1,1)"),
        ("A3", "=A1&IFERROR(WEEKNUM(100,99),\"none\")&WEEKNUM(1,1)"),
    ];
    let model = evaluated(&cells);
    assert_eq!(model._get_text("A2"), "none1");
    assert_eq!(model._get_text("A3"), format!("1{MONDAY}1"));
    // The re-run starts with its own calls: type 12 empties before the read.
    let cells = [
        ("A1", "=WEEKNUM(1,1)"),
        ("A2", "=WEEKNUM(1,12)&A1&WEEKNUM(100,99)"),
    ];
    assert_eq!(evaluated(&cells)._get_text("A2"), EMPTY);
    assert_ne!(sunday, EMPTY);
}

#[test]
fn a_spilling_formula_empties_the_state_at_its_place() {
    // gen-m6 shows it for dynamic-array formulas that gave #SPILL!, an error or one value; that
    // a formula that does spill does it too is our reading, unprobed (T-2070).
    let cells = [
        ("A1", READER),
        ("A2", "=SEQUENCE(1,2)"),
        ("A3", "=WEEKNUM(1,2)"),
    ];
    let model = evaluated(&cells);
    assert_eq!(model._get_text("B2"), "2");
    assert_eq!(model._get_text("A1"), EMPTY);
    let cells = [
        ("A1", READER),
        ("A2", "=SUM(SEQUENCE(1,2))"),
        ("A3", "=WEEKNUM(1,2)"),
    ];
    assert_eq!(evaluated(&cells)._get_text("A1"), MONDAY);
}

#[test]
fn a_reader_after_a_restart() {
    // A1 reads B3 before B1 spills there: the pass restarts with B1 first.
    let cells = [
        ("A1", "=SEQUENCE(1,1,B3)"),
        ("B1", "=SEQUENCE(3)"),
        ("C9", READER),
        ("C10", "=WEEKNUM(1,2)"),
    ];
    let model = evaluated(&cells);
    assert_eq!(model._get_text("A1"), "3");
    assert_eq!(model._get_text("C9"), MONDAY);
    assert!(model.evaluation.restarts_in_last_evaluation >= 1);
}

#[test]
fn excel_order_runs_only_where_a_reader_ran() {
    // No reader: one pass, in the default order.
    let mut model = evaluated(&[("A1", "=WEEKNUM(1,2)"), ("A2", "=A1+1")]);
    assert_eq!(model.evaluation.passes_in_last_evaluation, 1);
    assert!(!model.evaluation.excel_order);
    // A reader: the default pass, then one in Excel order; from then on Excel order only.
    model._set("A3", READER);
    model.evaluate();
    assert_eq!(model.evaluation.passes_in_last_evaluation, 2);
    assert!(model.evaluation.excel_order);
    model.evaluate();
    assert_eq!(model.evaluation.passes_in_last_evaluation, 1);
    assert_eq!(model._get_text("A3"), EMPTY);
    // The reader gone, the next pass in Excel order clears the flag.
    model._set("A3", "1");
    model.evaluate();
    assert_eq!(model.evaluation.passes_in_last_evaluation, 1);
    assert!(!model.evaluation.excel_order);
}

#[test]
fn a_long_running_total_with_a_reader() {
    // Each cell reads the one above: in reverse order every cell stops once, and the reader at
    // the top runs after all of them, after the setter at the bottom.
    let mut model = new_empty_model();
    model._set("A1", "1");
    for row in 2..=300 {
        model._set(&format!("A{row}"), &format!("=A{}+1", row - 1));
    }
    model._set("B1", READER);
    model._set("B300", "=WEEKNUM(1,2)");
    model.evaluate();
    assert_eq!(model._get_text("A300"), "300");
    assert_eq!(model._get_text("B1"), MONDAY);
}

#[test]
fn a_cycle_with_a_reader_ends() {
    let model = evaluated(&[
        ("A1", "=B1+1"),
        ("B1", "=A1+1"),
        ("C1", READER),
        ("C2", "=WEEKNUM(1,2)"),
    ]);
    assert_eq!(model._get_text("C1"), MONDAY);
    assert!(model._is_circular("A1"));
}
