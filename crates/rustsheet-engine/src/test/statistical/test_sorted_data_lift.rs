#![allow(clippy::unwrap_used)]

//! LARGE, SMALL, PERCENTILE*, QUARTILE*, PERCENTRANK* and TRIMMEAN lift their scalar arguments
//! over a data argument read once (SPEC-0011, T-1772). Excel values: probes/stat-whole-lift and
//! probes/stat-argument-modes (docs/excel-semantics/statistics.md "Statistical function argument
//! modes").

use crate::model::Model;
use crate::test::util::new_empty_model;

/// A model with probes/stat-whole-lift's inputs: A1:A8 = 1,2,4,3,7,5,8,6; B1:B3 = 3, #N/A, 5;
/// C1:C3 = "x", TRUE, blank; G1:G6 = 2, "x", blank, TRUE, 1, 2.
fn probe_model(formulas: &[(&str, &str)]) -> Model {
    let mut model = new_empty_model();
    for (row, value) in [1, 2, 4, 3, 7, 5, 8, 6].iter().enumerate() {
        model._set(&format!("A{}", row + 1), &value.to_string());
    }
    for (cell, value) in [
        ("B1", "3"),
        ("B2", "=NA()"),
        ("B3", "5"),
        ("C1", "x"),
        ("C2", "TRUE"),
        ("G1", "2"),
        ("G2", "x"),
        ("G4", "TRUE"),
        ("G5", "1"),
        ("G6", "2"),
    ] {
        model._set(cell, value);
    }
    for (cell, formula) in formulas {
        model._set(cell, formula);
    }
    model.evaluate();
    model
}

/// Each formula of `cases` in its own cell of column W, checked against its Excel value.
fn assert_cells(cases: &[(&str, &str)]) {
    let cells: Vec<(String, &str)> = cases
        .iter()
        .enumerate()
        .map(|(row, (formula, _))| (format!("W{}", row + 1), *formula))
        .collect();
    let formulas: Vec<(&str, &str)> = cells.iter().map(|(c, f)| (c.as_str(), *f)).collect();
    let model = probe_model(&formulas);
    for ((cell, formula), (_, expected)) in cells.iter().zip(cases) {
        assert_eq!(model._get_text(cell), *expected, "{formula}");
    }
}

/// probes/stat-argument-modes H345-H463, H549-H550: the lift cells of these functions.
#[test]
fn the_scalar_arguments_lift_over_an_array_and_a_range() {
    let mut cases = Vec::new();
    for function in [
        "LARGE(A1:A8,{})",
        "SMALL(A1:A8,{})",
        "PERCENTILE(A1:A8,{})",
        "PERCENTILE.INC(A1:A8,{})",
        "PERCENTILE.EXC(A1:A8,{})",
        "QUARTILE(A1:A8,{})",
        "QUARTILE.INC(A1:A8,{})",
        "QUARTILE.EXC(A1:A8,{})",
        "PERCENTRANK(A1:A8,{})",
        "PERCENTRANK.INC(A1:A8,{})",
        "PERCENTRANK.EXC(A1:A8,{})",
        "TRIMMEAN(A1:A8,{})",
    ] {
        cases.push(format!("=ROWS({})", function.replace("{}", "{0.3;0.3}")));
        cases.push(format!("=ROWS({})", function.replace("{}", "G5:G6")));
    }
    // QUARTILE's 0.3 is quart 0, PERCENTRANK's x 0.3 is #N/A: every element is still there.
    let cases: Vec<(&str, &str)> = cases.iter().map(|f| (f.as_str(), "2")).collect();
    assert_cells(&cases);
}

/// probes/stat-whole-lift W1-W20: per-element values and errors, shapes, typed and referenced
/// elements, a data error or no numbers beside a lifted k.
#[test]
fn large_and_small_elements_are_scalar_calls() {
    assert_cells(&[
        ("=INDEX(LARGE(A1:A8,{1;2;9}),1)", "8"),
        ("=INDEX(LARGE(A1:A8,{1;2;9}),3)", "#NUM!"),
        ("=INDEX(LARGE(A1:A8,{0.5;2.5}),1)", "#NUM!"),
        ("=INDEX(LARGE(A1:A8,{0.5;2.5}),2)", "6"),
        ("=INDEX(SMALL(A1:A8,{2.5;\"3\";TRUE}),1)", "2"),
        ("=INDEX(SMALL(A1:A8,{2.5;\"3\";TRUE}),2)", "3"),
        ("=INDEX(SMALL(A1:A8,{2.5;\"3\";TRUE}),3)", "1"),
        ("=INDEX(LARGE(A1:A8,G1:G4),1)", "7"),
        ("=INDEX(LARGE(A1:A8,G1:G4),2)", "#VALUE!"),
        ("=INDEX(LARGE(A1:A8,G1:G4),3)", "#NUM!"),
        ("=INDEX(LARGE(A1:A8,G1:G4),4)", "8"),
        ("=ROWS(LARGE(B1:B3,{1;2}))", "2"),
        ("=INDEX(LARGE(B1:B3,{1;2}),2)", "#N/A"),
        ("=COLUMNS(LARGE(A1:A8,{1,2,3}))", "3"),
        ("=INDEX(LARGE(A1:A8,{1,2;3,4}),2,2)", "5"),
        ("=INDEX(LARGE(A1:A8,{1;#DIV/0!}),2)", "#DIV/0!"),
        ("=INDEX(SMALL(C1:C3,{1;2}),1)", "#NUM!"),
        ("=ROWS(SMALL(C1:C3,{1;2}))", "2"),
        ("=INDEX(LARGE(A1:A8*1,{1;2}),2)", "7"),
        ("=ROWS(LARGE(A1:A8,G1:G1))", "1"),
        ("=ROWS(LARGE({1,2,3},{1;2}))", "2"),
        ("=ROWS(LARGE(A1:A8,IF(TRUE,G5:G6)))", "2"),
        ("=INDEX(LARGE({5,5,3},{1;2;3}),2)", "5"),
        ("=INDEX(LARGE((A1:A4,A5:A8),{1;2}),2)", "7"),
        ("=ROWS(SMALL(A1:A8,{1;2;3}+0))", "3"),
    ]);
}

/// probes/stat-whole-lift W21-W47.
#[test]
fn percentile_quartile_percentrank_and_trimmean_elements_are_scalar_calls() {
    assert_cells(&[
        ("=INDEX(PERCENTILE(A1:A8,{0;1.5;0.5}),2)", "#NUM!"),
        ("=INDEX(PERCENTILE(A1:A8,{0;1.5;0.5}),3)", "4.5"),
        ("=INDEX(PERCENTILE.EXC(A1:A8,{0.05;0.5}),1)", "#NUM!"),
        ("=INDEX(QUARTILE(A1:A8,{0;5;2.7}),2)", "#NUM!"),
        ("=INDEX(QUARTILE(A1:A8,{0;5;2.7}),3)", "4.5"),
        ("=INDEX(QUARTILE.EXC(A1:A8,{0;3}),1)", "#NUM!"),
        ("=INDEX(PERCENTRANK(A1:A8,{0;4.5}),1)", "#N/A"),
        ("=INDEX(PERCENTRANK(A1:A8,{0;4.5}),2)", "0.5"),
        ("=ROWS(PERCENTRANK(A1:A8,4.5,{1;2}))", "2"),
        ("=INDEX(PERCENTRANK(A1:A8,4.5,{1;2}),1)", "0.5"),
        ("=ROWS(PERCENTRANK.INC(A1:A8,4.5,G5:G6))", "2"),
        ("=INDEX(PERCENTRANK.EXC(A1:A8,{4.5;2.5},{1,2}),2,2)", "0.27"),
        ("=COLUMNS(PERCENTRANK.EXC(A1:A8,{4.5;2.5},{1,2}))", "2"),
        ("=INDEX(PERCENTRANK(A1:A8,4.5,{0;2}),1)", "#NUM!"),
        ("=INDEX(TRIMMEAN(A1:A8,{0.2;1}),2)", "#NUM!"),
        ("=INDEX(TRIMMEAN(A1:A8,{0.2;1}),1)", "4.5"),
        ("=ROWS(TRIMMEAN(A1:A8,G5:G6))", "2"),
        ("=INDEX(PERCENTRANK(C1:C3,{1;2}),2)", "#N/A"),
        ("=ROWS(PERCENTILE(B1:B3,G5:G6))", "2"),
        ("=INDEX(PERCENTILE(B1:B3,G5:G6),1)", "#N/A"),
        ("=PERCENTRANK(A1:A8,4.5,G6:G6)", "0.5"),
        ("=INDEX(QUARTILE.INC(A1:A8,G5:G6),2)", "4.5"),
        ("=INDEX(PERCENTILE.INC(A1:A8,{0.3;\"x\"}),2)", "#VALUE!"),
    ]);
}

/// The unlifted calls are today's: AGGREGATE 14-19 route through the same kernels.
#[test]
fn unlifted_calls_and_aggregate_keep_their_values() {
    assert_cells(&[
        ("=LARGE(A1:A8,2)", "7"),
        ("=SMALL(A1:A8,2)", "2"),
        ("=PERCENTILE(A1:A8,0.3)", "3.1"),
        ("=QUARTILE.EXC(A1:A8,2)", "4.5"),
        ("=PERCENTRANK(A1:A8,1.5)", "0.071"),
        ("=PERCENTRANK.EXC(A1:A8,1.5)", "0.166"),
        ("=TRIMMEAN(A1:A8,0.2)", "4.5"),
        ("=AGGREGATE(14,6,A1:A8,2)", "7"),
        ("=AGGREGATE(17,6,A1:A8,2)", "4.5"),
        ("=LARGE(A1:A8,{2})", "7"),
        ("=LARGE(B1:B3,1)", "#N/A"),
        ("=LARGE(A1:A8)", "#ERROR!"),
    ]);
}

/// Column A filled to row 1,000,000.
fn filled_column_model() -> Model {
    let mut model = new_empty_model();
    let sheet = model.workbook.worksheet_mut(0).unwrap();
    for row in 1..=1_000_000 {
        sheet
            .set_cell_with_number(row, 1, f64::from(row), 0)
            .unwrap();
    }
    model.evaluate();
    model
}

/// `LiftCost::Prepared`: a million cells sorted once against 200 elements runs (directive 5
/// counts cells + elements, not cells × elements); a result above the array bound is
/// `#ERROR!` before the first element.
#[test]
fn a_sorted_column_against_many_elements_runs_and_the_bound_holds() {
    let mut model = filled_column_model();
    model._set("C1", "=ROWS(LARGE(A1:A1000000,SEQUENCE(200)))");
    model._set("C2", "=INDEX(LARGE(A1:A1000000,SEQUENCE(200)),200)");
    model._set("C3", "=INDEX(SMALL(A1:A1000000,SEQUENCE(200)),200)");
    model._set("C4", "=ROWS(PERCENTILE(A1:A1000000,SEQUENCE(200)/200))");
    model._set(
        "C5",
        "=ROWS(PERCENTRANK(A1:A1000000,SEQUENCE(1100),SEQUENCE(1,1000)))",
    );
    model.evaluate();
    assert_eq!(model._get_text("C1"), "200");
    assert_eq!(model._get_text("C2"), "999801");
    assert_eq!(model._get_text("C3"), "200");
    assert_eq!(model._get_text("C4"), "200");
    assert_eq!(model._get_text("C5"), "#ERROR!");
}

/// TRIMMEAN averages the kept values per element, so it is charged elements × cells and a
/// million cells against 200 elements is refused before the first element.
#[test]
fn trimmean_is_charged_per_element() {
    let mut model = filled_column_model();
    model._set("C1", "=ROWS(TRIMMEAN(A1:A1000000,SEQUENCE(200)/1000))");
    model._set("C2", "=ROWS(TRIMMEAN(A1:A1000,SEQUENCE(200)/1000))");
    model.evaluate();
    assert_eq!(model._get_text("C1"), "#ERROR!");
    assert_eq!(model._get_text("C2"), "200");
}

/// Kernel errors carry no message, so a failing element allocates nothing.
#[test]
fn element_errors_carry_no_message() {
    let model = probe_model(&[
        ("W1", "=LARGE(A1:A8,9)"),
        ("W2", "=SMALL(A1:A8,0)"),
        ("W3", "=PERCENTILE(A1:A8,2)"),
        ("W4", "=QUARTILE.EXC(A1:A8,0)"),
        ("W5", "=PERCENTRANK(A1:A8,0)"),
        ("W6", "=TRIMMEAN(A1:A8,1)"),
    ]);
    for cell in ["W1", "W2", "W3", "W4", "W5", "W6"] {
        assert!(model._get_text(cell).starts_with('#'), "{cell}");
        assert_eq!(model._get_error_message(cell), "", "{cell}");
    }
}
