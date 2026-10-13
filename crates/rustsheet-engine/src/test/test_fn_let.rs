#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn test_let_wrong_arg_count() {
    let mut model = new_empty_model();
    // Too few args
    model._set("A1", "=LET(x,1)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#ERROR!");

    // Even number of args
    model._set("A2", "=LET(x,1,y,2)");
    model.evaluate();
    assert_eq!(model._get_text("A2"), "#ERROR!");
}

#[test]
fn test_let_two_variables() {
    let mut model = new_empty_model();
    // LET(x, 2, y, 4, x+y) → 6
    model._set("A1", "=LET(x,2,y,4,x+y)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "6");
}

#[test]
fn test_let_with_cell_reference() {
    let mut model = new_empty_model();
    model._set("A1", "5");
    // LET(x, A1, SUM(x*3, x)) → SUM(15, 5) = 20
    model._set("B1", "=LET(x,A1,SUM(x*3,x))");
    model.evaluate();
    assert_eq!(model._get_text("B1"), "20");
}

#[test]
fn test_let_nested() {
    let mut model = new_empty_model();
    // LET(x, 3, LET(x, 4, x*5)*x) → inner x=4: 4*5=20, outer x=3: 20*3=60
    model._set("A1", "=LET(x,3,LET(x,4,x*5)*x)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "60");
}

#[test]
fn test_let_inner_binding_references_outer_value() {
    let mut model = new_empty_model();
    // LET(x,1,LET(x,x+1,x)) → inner x is bound from outer x+1 = 2, result = 2
    model._set("A1", "=LET(x,1,LET(x,x+1,x))");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "2");
}
#[test]
fn test_let_rebind_same_name_within_single_let() {
    let mut model = new_empty_model();
    model._set("A1", "=LET(x,1,x,2,x)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#ERROR!");
}

#[test]
fn test_let_duplicate_names_differing_in_case_are_refused() {
    // Excel for the web refuses `LET(x,1,X,2,x)` and `LET(é,1,É,2,é)` as it refuses
    // `LET(x,1,x,2,x)` (docs/excel-semantics/references.md, T-1387).
    let mut model = new_empty_model();
    model._set("A1", "=LET(x,1,X,2,x)");
    model._set("A2", "=LET(é,1,É,2,é)");
    model._set("A3", "=LET(é,1,É+1)");
    model._set("A4", "=LET(Ä,2,ä*10)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#ERROR!");
    assert_eq!(model._get_text("A2"), "#ERROR!");
    assert_eq!(model._get_text("A3"), "2");
    assert_eq!(model._get_text("A4"), "20");
}

#[test]
fn test_let_r_and_c_variables() {
    let mut model = new_empty_model();
    // r and c must work as LET variable names
    model._set("A1", "=LET(r,3,c,4,r+c)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "7");
}

#[test]
fn test_lambda_r_and_c_parameters() {
    let mut model = new_empty_model();
    // r and c must work as LAMBDA parameter names
    model._set("A1", "=LAMBDA(r,c,r+c)(3,4)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "7");
}

#[test]
fn test_let_names_are_case_insensitive() {
    let mut model = new_empty_model();
    model._set("A1", "=LET(x,2,X+1)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "3");
}

#[test]
fn test_subtotal_of_take_over_a_selector_holding_a_computed_array_spills() {
    let mut model = new_empty_model();
    model._set("A1", "=SUBTOTAL(9,TAKE(CHOOSE(1,SEQUENCE(4)),2))");
    model._set("C1", "=SUBTOTAL(9,TAKE(SWITCH(9,1,D1:D4,SEQUENCE(4)),2))");
    model._set("E1", "=SUBTOTAL(9,TAKE(IFS(TRUE,SEQUENCE(4)),2))");
    model.evaluate();
    // probes/subtotal-reference-operands-f C46, C111 and C66: two #VALUE! each.
    for cell in ["A1", "A2", "C1", "C2", "E1", "E2"] {
        assert_eq!(model._get_text(cell), "#VALUE!", "{cell}");
    }
}

#[test]
fn test_lambda_parameter_names_are_case_insensitive() {
    let mut model = new_empty_model();
    // probes/let-lambda-name-case A1, A2, A6, A10, A11 and A15, in that order.
    model._set("A1", "=LET(x,1,LAMBDA(X,x*10)(2))");
    model._set("A2", "=LAMBDA(x,X+1)(2)");
    model._set("A3", "=LET(X,1,LAMBDA(x,X*10)(2))");
    model._set("A4", "=LAMBDA(Ab,aB*2)(4)");
    model._set("A5", "=LET(f,LAMBDA(x,x+1),F(2))");
    model._set("A6", "=LET(x,1,LAMBDA(X,x)(2))");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "20");
    assert_eq!(model._get_text("A2"), "3");
    assert_eq!(model._get_text("A3"), "20");
    assert_eq!(model._get_text("A4"), "8");
    assert_eq!(model._get_text("A5"), "3");
    assert_eq!(model._get_text("A6"), "2");
}

#[test]
fn test_let_binds_index_reference_and_keeps_at_index_value() {
    let mut model = new_empty_model();
    model._set("B1", "5");
    model._set("B2", "6");
    // An INDEX binding stays a reference (probes/let-bind-reference-forms).
    model._set("A1", "=LET(x,INDEX(B1:B2,2),ROW(x))");
    // `@INDEX(..)` is bound as a value (T-1223: unprobed, kept as it was).
    model._set("A2", "=LET(x,@INDEX(B1:B2,2),ISREF(x))");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "2");
    assert_eq!(model._get_text("A2"), "FALSE");
}
