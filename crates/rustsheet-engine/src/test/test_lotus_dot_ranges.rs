#![allow(clippy::unwrap_used)]

//! Lotus-style dotted ranges: `A1.B2` in a formula is the range `A1:B2` (docs/excel-semantics/
//! references.md "R1C1-shaped sheet and defined names"; probes/lotus-dot-range*, excel-graph,
//! T-1659).

use crate::test::util::new_empty_model;
use crate::Model;

fn model_with_values() -> Model {
    let mut model = new_empty_model();
    model.add_sheet("Data").unwrap();
    for (cell, value) in [
        ("A1", "1"),
        ("B1", "4"),
        ("A2", "8"),
        ("B2", "2"),
        ("C3", "16"),
    ] {
        model._set(cell, value);
    }
    model._set("Data!A1", "10");
    model._set("Data!B2", "20");
    model
}

#[test]
fn dotted_ranges_read_and_print_as_colon_ranges() {
    let mut model = model_with_values();
    let cases = [
        ("=SUM(A1.B2)", "=SUM(A1:B2)", "15"),
        ("=SUM(B2.A1)", "=SUM(A1:B2)", "15"),
        ("=SUM(a1..b2)", "=SUM(A1:B2)", "15"),
        ("=SUM(A1...B2)", "=SUM(A1:B2)", "15"),
        ("=SUM($A$1.B2)", "=SUM($A$1:B2)", "15"),
        ("=SUM(A$1..$B$2)", "=SUM(A$1:$B$2)", "15"),
        ("=SUM(A1.$B$2)", "=SUM(A1:$B$2)", "15"),
        ("=SUM(A1 . B2)", "=SUM(A1:B2)", "15"),
        ("=SUM(Data!A1.B2)", "=SUM(Data!A1:B2)", "30"),
        ("=SUM('Data'!B2..A1)", "=SUM(Data!A1:B2)", "30"),
        ("=ISREF(A1.B2)", "=ISREF(A1:B2)", "TRUE"),
        ("=SUM(A1.B2 B1:B3)", "=SUM(A1:B2 B1:B3)", "6"),
        ("=ROWS(A1.B2:C3)", "=ROWS(A1:B2:C3)", "3"),
    ];
    for (row, (formula, ..)) in cases.iter().enumerate() {
        model._set(&format!("F{}", row + 1), formula);
    }
    model.evaluate();
    let mut reloaded = Model::from_bytes(&model.to_bytes(), "en").unwrap();
    reloaded.evaluate();
    for (row, (formula, text, value)) in cases.iter().enumerate() {
        let cell = format!("F{}", row + 1);
        for m in [&model, &reloaded] {
            assert_eq!(m._get_formula(&cell), *text, "{formula}");
            assert_eq!(m._get_text(&cell), *value, "{formula}");
        }
    }
}

// probes/lotus-dot-range-{column,column-dotdot,number,offgrid,trailing-letter,name-second}:
// no range, a name Excel cannot resolve, kept as typed. `A1.B2.C3` makes Excel reject the file.
#[test]
fn other_dotted_names_stay_names() {
    let mut model = model_with_values();
    let formulas = [
        "=SUM(A.B)",
        "=SUM(A..B)",
        "=SUM(A1.5)",
        "=SUM(A1.XFE1)",
        "=SUM(A1.B2D)",
        "=SUM(A1.foo)",
        "=SUM(A1.B2.C3)",
    ];
    for (row, formula) in formulas.iter().enumerate() {
        model._set(&format!("F{}", row + 1), formula);
    }
    model.evaluate();
    for (row, formula) in formulas.iter().enumerate() {
        let cell = format!("F{}", row + 1);
        assert_eq!(model._get_text(&cell), "#NAME?", "{formula}");
        assert_eq!(model._get_formula(&cell), *formula);
    }
}

// probes/r1c1-name-r1-dot-c1, -c1-c1-dot: the range wins over a defined name spelled so.
#[test]
fn a_dotted_range_shadows_a_defined_name() {
    let mut model = new_empty_model();
    model._set("B1", "7");
    model.define_name("R1.C1", None, "=Sheet1!$B$1").unwrap();
    model.define_name("C1.C1", None, "=Sheet1!$B$1").unwrap();
    model._set("A1", "=R1.C1*1");
    model._set("A3", "=C1.C1*1");
    model._set("A4", "=ISREF(R1.C1)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#SPILL!");
    assert_eq!(model._get_text("A3"), "0");
    assert_eq!(model._get_text("A4"), "TRUE");
}

// probes/r1c1-let-name-r1-c1: a LET or LAMBDA binding the name makes it the variable within the
// call; outside it the name is the range again (printed with `:`; the formulas below are written
// as Excel prints them).
#[test]
fn let_and_lambda_names_spelled_like_dotted_ranges_are_variables_in_scope() {
    let mut model = model_with_values();
    let cases = [
        ("=LET(R1.C1,5,R1.C1*2)", "10"),
        ("=LET(a1.b2,5,A1.B2*2)", "10"),
        ("=LET(x,1,SUM(A1:B2))", "15"),
        ("=LET(A1.B2,5,A1.B2)+SUM(A1:B2)", "20"),
        ("=LET(x,SUM(A1:B2),A1.B2,x,A1.B2+1)", "16"),
        ("=LAMBDA(A1.B2,A1.B2*2)(4)", "8"),
        ("=LAMBDA([A1.B2],ISOMITTED(A1.B2))()", "TRUE"),
        ("=LAMBDA(x,LAMBDA(A1.B2,A1.B2+x)(1)+SUM(A1:B2))(2)", "18"),
    ];
    for (row, (formula, _)) in cases.iter().enumerate() {
        model._set(&format!("F{}", row + 1), formula);
    }
    model.evaluate();
    let mut reloaded = Model::from_bytes(&model.to_bytes(), "en").unwrap();
    reloaded.evaluate();
    for (row, (formula, value)) in cases.iter().enumerate() {
        let cell = format!("F{}", row + 1);
        for m in [&model, &reloaded] {
            assert_eq!(m._get_text(&cell), *value, "{formula}");
            assert_eq!(m._get_formula(&cell), *formula);
        }
    }
}
