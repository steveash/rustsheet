#![allow(clippy::unwrap_used)]

//! TREND and GROWTH argument order, shapes and defaults (T-1726; probes/regression-trend and the
//! gen-m9 cells cited in statistics.md "TREND: defaults, shapes, single points and error inputs").

use crate::test::util::new_empty_model;

/// The probe's data: y in A1:A6, x in B1:B6, a text cell in C2, a blank in D2, an error in F2, a
/// constant x in G1:G6, `new_x` in K1:K2.
fn trend_model() -> crate::model::Model {
    let mut model = new_empty_model();
    for (row, (y, x)) in [(2, 1), (4, 2), (5, 3), (4, 4), (5, 5), (7, 6)]
        .iter()
        .enumerate()
    {
        let r = row + 1;
        model._set(&format!("A{r}"), &y.to_string());
        model._set(&format!("B{r}"), &x.to_string());
        model._set(&format!("C{r}"), &y.to_string());
        model._set(&format!("D{r}"), &y.to_string());
        model._set(&format!("F{r}"), &y.to_string());
        model._set(&format!("G{r}"), "3");
    }
    model._set("C2", "t");
    model._set("D2", "");
    model._set("F2", "=1/0");
    model._set("K1", "7");
    model._set("K2", "8");
    model
}

fn eval_in(model: &mut crate::model::Model, formula: &str) -> String {
    model._set("Z1", formula);
    model.evaluate();
    model._get_text("Z1")
}

#[test]
fn empty_arguments_are_the_defaults() {
    let mut model = trend_model();
    assert_eq!(eval_in(&mut model, "=ROWS(TREND(A1:A6,,K1:K2))"), "2");
    assert_eq!(eval_in(&mut model, "=ROWS(TREND(A1:A6,B1:B6,,FALSE))"), "6");
    assert_eq!(eval_in(&mut model, "=ROWS(TREND(A1:A6,,,FALSE))"), "6");
}

#[test]
fn shapes_follow_known_and_new_x() {
    let mut model = trend_model();
    // Same-shaped data is one variable: a row new_x gives a row (trend L6).
    assert_eq!(
        eval_in(&mut model, "=COLUMNS(TREND(A1:A6,B1:B6,{7,8}))"),
        "2"
    );
    assert_eq!(eval_in(&mut model, "=COLUMNS(TREND({2,4,5},{1,2,3}))"), "3");
    // A 2-D known_y with a 2-D known_x of its shape (trend L34).
    assert_eq!(
        eval_in(
            &mut model,
            "=ROWS(TREND(A1:B3,A4:B6))&COLUMNS(TREND(A1:B3,A4:B6))"
        ),
        "32"
    );
    assert_eq!(eval_in(&mut model, "=TREND(A1:A5,B1:B6)"), "#REF!");
    // A text known_x is one value: a size mismatch (trend L43).
    assert_eq!(eval_in(&mut model, "=TREND(A1:A6,\"a\")"), "#REF!");
    // One y against a column of x: one point of two variables, the point itself.
    assert_eq!(eval_in(&mut model, "=TREND(-1,{-0.5;1})"), "-1");
}

#[test]
fn single_points_and_constant_x() {
    let mut model = trend_model();
    assert_eq!(eval_in(&mut model, "=TREND(5)"), "5");
    assert_eq!(eval_in(&mut model, "=TREND(A1:A1,B1:B1)"), "2");
    // A zero-variance x is dropped: the mean of y (trend O20).
    assert_eq!(eval_in(&mut model, "=INDEX(TREND(A1:A6,G1:G6),1,1)"), "4.5");
}

#[test]
fn non_numbers_in_the_data() {
    let mut model = trend_model();
    assert_eq!(eval_in(&mut model, "=TREND(C1:C6,B1:B6)"), "#VALUE!");
    assert_eq!(eval_in(&mut model, "=TREND(D1:D6,B1:B6)"), "#VALUE!");
    // An error cell is `#VALUE!` too, not its own error (trend L28, L29).
    assert_eq!(eval_in(&mut model, "=TREND(F1:F6,B1:B6)"), "#VALUE!");
    assert_eq!(eval_in(&mut model, "=TREND(A1:A6,F1:F6)"), "#VALUE!");
    // An error cell in new_x is `#VALUE!` for its own prediction only (trend L37).
    assert_eq!(eval_in(&mut model, "=ROWS(TREND(A1:A6,B1:B6,F1:F2))"), "2");
    assert_eq!(
        eval_in(&mut model, "=INDEX(TREND(A1:A6,B1:B6,F1:F2),2,1)"),
        "#VALUE!"
    );
    // A text or blank new_x is `#VALUE!` for the whole call (trend L38, L39).
    assert_eq!(eval_in(&mut model, "=TREND(A1:A6,B1:B6,D1:D2)"), "#VALUE!");
}

#[test]
fn argument_errors_come_first_in_order() {
    let mut model = trend_model();
    // known_y's error beats const's; new_x's error beats the size check.
    assert_eq!(
        eval_in(&mut model, "=TREND(SQRT(-1),B1:B6,K1:K2,NA())"),
        "#NUM!"
    );
    assert_eq!(eval_in(&mut model, "=TREND(A1:A5,B1:B6,1/0)"), "#DIV/0!");
    assert_eq!(
        eval_in(&mut model, "=TREND(A1:A6,B1:B6,K1:K2,NA())"),
        "#N/A"
    );
    // A typed text const is `#VALUE!` before the sizes; a text cell waits for them.
    assert_eq!(
        eval_in(&mut model, "=TREND(A1:A5,B1:B6,K1:K2,\"x\")"),
        "#VALUE!"
    );
    assert_eq!(eval_in(&mut model, "=TREND(A1:A5,B1:B6,K1:K2,C2)"), "#REF!");
    // An error element of x waits for the sizes.
    assert_eq!(eval_in(&mut model, "=TREND(A1:A5,F1:F6)"), "#REF!");
}

#[test]
fn an_array_const_lifts() {
    let mut model = trend_model();
    model._set("Z1", "=TREND(A1:A6,B1:B6,K1,{TRUE,FALSE})");
    model.evaluate();
    assert_eq!(model._get_text("Z1"), "7.2");
    assert_eq!(model._get_text("AA1"), "8.307692308");
}

#[test]
fn growth_reads_known_y_in_order() {
    let mut model = new_empty_model();
    model._set("A1", "1.005");
    model._set("A2", "0");
    model._set("B1", "");
    model._set("B2", "1");
    model._set("C1", "t");
    model._set("C2", "-1");
    model.evaluate();
    // The first element that is no positive number decides, row-major.
    assert_eq!(eval_in(&mut model, "=GROWTH(A1:A2)"), "#NUM!");
    assert_eq!(eval_in(&mut model, "=GROWTH(B1:B2)"), "#VALUE!");
    assert_eq!(eval_in(&mut model, "=GROWTH(C1:C2)"), "#VALUE!");
    // A text known_x comes before the sign.
    assert_eq!(eval_in(&mut model, "=GROWTH(0,C1)"), "#VALUE!");
}
