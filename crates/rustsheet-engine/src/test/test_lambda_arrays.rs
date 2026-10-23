#![allow(clippy::unwrap_used)]

//! Arrays whose elements are LAMBDAs (T-2039, probes/lambda-arrays).

use crate::test::util::new_empty_model;

/// Three LAMBDAs: the n-th one maps z to z*10^n plus the digits 1..n.
const SCAN: &str = "SCAN(LAMBDA(z,z),{1,2,3},LAMBDA(f,x,LAMBDA(z,f(z)*10+x)))";

#[test]
fn map_calls_the_lambdas_scan_gives() {
    let mut model = new_empty_model();
    model._set("A1", &format!("=MAP({SCAN},LAMBDA(g,g(0)))"));
    model._set("A2", &format!("=MAP({SCAN},{{1,2,3}},LAMBDA(g,y,g(y)))"));
    model._set("A3", &format!("=REDUCE(0,{SCAN},LAMBDA(a,g,a+g(1)))"));
    model._set("A4", &format!("=LET(s,{SCAN},g,INDEX(s,1,3),g(0))"));
    model.evaluate();
    assert_eq!(model._get_text("A1"), "1");
    assert_eq!(model._get_text("B1"), "12");
    assert_eq!(model._get_text("C1"), "123");
    assert_eq!(model._get_text("C2"), "3123");
    assert_eq!(model._get_text("A3"), "1246");
    assert_eq!(model._get_text("A4"), "123");
}

#[test]
fn a_lambda_element_shows_calc_and_reads_as_value_error() {
    let mut model = new_empty_model();
    model._set("A1", &format!("={SCAN}"));
    model._set("A2", &format!("=ROWS({SCAN})+COLUMNS({SCAN})*10"));
    model._set("A3", &format!("=TYPE(INDEX({SCAN},1,1))"));
    model._set("A4", &format!("=SUM({SCAN})"));
    model._set("A5", &format!("=COUNTA({SCAN})*10+COUNT({SCAN})"));
    model._set("A6", &format!("=ISERROR({SCAN})"));
    model._set("A7", &format!("={SCAN}=1"));
    model._set("A8", &format!("=ARRAYTOTEXT({SCAN})"));
    model._set("A9", &format!("=LET(s,{SCAN},s(0))"));
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#CALC!");
    assert_eq!(model._get_text("C1"), "#CALC!");
    assert_eq!(model._get_text("A2"), "31");
    assert_eq!(model._get_text("A3"), "128");
    assert_eq!(model._get_text("A4"), "#VALUE!");
    assert_eq!(model._get_text("A5"), "30");
    assert_eq!(model._get_text("B6"), "FALSE");
    assert_eq!(model._get_text("A7"), "#VALUE!");
    assert_eq!(model._get_text("A8"), "\u{3bb}, \u{3bb}, \u{3bb}");
    assert_eq!(model._get_text("A9"), "#VALUE!");
}

#[test]
fn stacked_and_made_lambdas_are_elements() {
    let mut model = new_empty_model();
    model._set(
        "A1",
        "=MAP(VSTACK(LAMBDA(x,x),LAMBDA(x,x*2)),LAMBDA(g,g(5)))",
    );
    model._set(
        "B1",
        "=MAP(MAKEARRAY(1,2,LAMBDA(r,c,LAMBDA(x,x+c))),LAMBDA(g,g(10)))",
    );
    model._set("D1", "=MAP(LAMBDA(x,x*3),LAMBDA(g,g(5)))");
    model._set("E1", &format!("=MAP(TAKE({SCAN},,-1),LAMBDA(g,g(1)))"));
    model.evaluate();
    assert_eq!(model._get_text("A1"), "5");
    assert_eq!(model._get_text("A2"), "10");
    assert_eq!(model._get_text("B1"), "11");
    assert_eq!(model._get_text("C1"), "12");
    assert_eq!(model._get_text("D1"), "15");
    assert_eq!(model._get_text("E1"), "1123");
}

#[test]
fn an_array_of_lambdas_leaving_a_let_keeps_its_captures() {
    let mut model = new_empty_model();
    model._set(
        "A1",
        "=MAP(LET(k,100,SCAN(LAMBDA(z,z),{1,2},LAMBDA(f,x,LAMBDA(z,f(z)*k+x)))),LAMBDA(g,g(0)))",
    );
    model.evaluate();
    assert_eq!(model._get_text("A1"), "1");
    assert_eq!(model._get_text("B1"), "102");
}

#[test]
fn an_omitted_take_or_drop_count_keeps_everything() {
    let mut model = new_empty_model();
    model._set("A1", "=TAKE({1,2,3;4,5,6},,-1)");
    model._set("C1", "=DROP({1,2,3;4,5,6},,1)");
    model._set("A4", "=TAKE({1,2,3;4,5,6},Z1,1)");
    model._set("A5", "=ROWS(DROP({1,2,3;4,5,6},,))");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "3");
    assert_eq!(model._get_text("A2"), "6");
    assert_eq!(model._get_text("C1"), "2");
    assert_eq!(model._get_text("D2"), "6");
    assert_eq!(model._get_text("A4"), "#CALC!");
    assert_eq!(model._get_text("A5"), "2");
}

/// gen-m9/165db58d1540f014 A273 and gen-m9/ee2497a5e683c5d9 A124: a LAMBDA element read as a
/// number is `#VALUE!`.
#[test]
fn a_lambda_element_read_as_a_number_is_a_value_error() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("A2", "2");
    model._set(
        "B1",
        "=CONFIDENCE.NORM(BYROW(A1,LAMBDA(acc,LAMBDA(v,v))),0.5,22)",
    );
    model._set("B2", "=INTERCEPT(5,BYCOL(A1:A2,LAMBDA(x,LAMBDA(w,w))))");
    model._set(
        "B3",
        "=CONFIDENCE.NORM(MAP(A1,LAMBDA(acc,LAMBDA(v,v))),0.5,22)",
    );
    model._set("B4", "=NPV(0.1,MAP({1,2},LAMBDA(acc,LAMBDA(v,v))))");
    model.evaluate();
    for cell in ["B1", "B2", "B3", "B4"] {
        assert_eq!(model._get_text(cell), "#VALUE!", "{cell}");
    }
}
