#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn wrong_number_of_arguments() {
    let mut model = new_empty_model();
    model._set("A1", "=FORMULATEXT()");
    model._set("A2", "=FORMULATEXT(\"B\",\"A\")");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#ERROR!");
    assert_eq!(model._get_text("A2"), *"#ERROR!");
}

#[test]
fn multi_sheet_ref() {
    let mut model = new_empty_model();
    model.new_sheet();
    model._set("A1", "=FORMULATEXT(Sheet1!A1:Sheet2!A1)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#ERROR!");
}

#[test]
fn implicit_intersection() {
    let mut model = new_empty_model();
    model._set("A1", "=FORMULATEXT(C1:C2)");
    model._set("A2", "=FORMULATEXT(D1:E1)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#N/IMPL!");
    assert_eq!(model._get_text("A2"), *"#N/IMPL!");
}

#[test]
fn implicit_intersection_operator() {
    // @A:A in B1 intersects to A1, so FORMULATEXT acts on that single cell.
    let mut model = new_empty_model();
    model._set("A1", "=1 +  2");
    model._set("B1", "=FORMULATEXT(@A:A)");
    model.evaluate();

    assert_eq!(model._get_text("B1"), *"=1+2");
}

#[test]
fn non_reference() {
    let mut model = new_empty_model();
    model._set("A1", "=FORMULATEXT(42)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#ERROR!");
}

#[test]
fn test_language_independence() {
    let mut model = new_empty_model();
    model._set("A1", "=SUM(1, 2)");
    model._set("B1", "=FORMULATEXT(A1)");

    model.evaluate();
    model.set_language("fr").unwrap();
    model.evaluate();

    assert_eq!(model._get_formula("A1"), *"=SOMME(1,2)");
    assert_eq!(model._get_text("B1"), *"=SUM(1,2)");
}

#[test]
fn test_locale() {
    let mut model = new_empty_model();

    model._set("A1", "=SUM(1.123, 2)");
    model._set("B1", "=FORMULATEXT(A1)");
    model.evaluate();
    model.set_language("fr").unwrap();
    model.set_locale("fr").unwrap();
    model.evaluate();

    assert_eq!(model._get_formula("A1"), *"=SOMME(1,123;2)");
    assert_eq!(model._get_text("B1"), *"=SUM(1,123;2)");
}

#[test]
fn test_sign_prefixed() {
    let mut model = new_empty_model();

    model._set("A2", "-B1-B2");
    assert_eq!(model._get_formula("A2"), *"=-B1-B2");

    model._set("A4", "-4-2");
    assert_eq!(model._get_formula("A4"), *"=-4-2");

    model._set("A5", "-42");
    assert_eq!(model._get_formula("A5"), *"");

    model._set("A6", "+42");
    assert_eq!(model._get_formula("A6"), *"");

    // Sign-prefixed input is classified with no clock, as before year-less dates (T-0167).
    for (cell, input) in [("A7", "+$5"), ("A8", "- 5"), ("A9", "-Jul-45")] {
        model._set(cell, input);
        assert_eq!(model._get_formula(cell), *"", "{input}");
    }

    model._set("A7", "-B");
    assert_eq!(model._get_formula("A7"), *"=-B");

    model._set("A9", "-B1");
    assert_eq!(model._get_formula("A9"), *"=-B1");

    model.evaluate();

    assert_eq!(model._get_text("A4"), *"-6");
    assert_eq!(model._get_text("A5"), *"-42");
    assert_eq!(model._get_text("A6"), *"42");
    assert_eq!(model._get_text("A7"), *"#NAME?");
}

#[test]
fn an_unknown_call_name_keeps_the_first_spelling_met() {
    // Excel interns the spelling workbook-wide, case-insensitively, first met wins
    // (probes/unknown-function-name-spelling, T-1719).
    let mut model = new_empty_model();
    model._set("A1", "=my_Fn(1,2)+sum(3)");
    model._set("A2", "=MY_FN(1)");
    model._set("A3", "=Xyz2(1)");
    model._set("A4", "=SUM(A1.INDEX(B2:B2,1))");
    model.evaluate();

    assert_eq!(model._get_formula("A1"), "=my_Fn(1,2)+SUM(3)");
    assert_eq!(model._get_formula("A2"), "=my_Fn(1)");
    assert_eq!(model._get_formula("A3"), "=Xyz2(1)");
    assert_eq!(model._get_formula("A4"), "=SUM(A1.INDEX(B2:B2,1))");
}

#[test]
fn an_unknown_name_used_as_a_value_shares_the_call_name_spelling() {
    // Either order: the first spelling met wins (probes/unknown-function-name-reach B9/B11, T-1760).
    let mut model = new_empty_model();
    model._set("A1", "=Qux+1");
    model._set("A2", "=qux(1)");
    model._set("A3", "=wib(1)");
    model._set("A4", "=Wib+1");
    model.evaluate();

    assert_eq!(model._get_formula("A2"), "=Qux(1)");
    assert_eq!(model._get_formula("A4"), "=wib+1");
}

#[test]
fn a_reload_keeps_the_spelling_of_an_unknown_call_name() {
    let mut model = new_empty_model();
    model._set("A1", "=Foo(1)");
    model._set("A2", "=FOO(2)");
    model.evaluate();

    let mut reloaded = crate::Model::from_bytes(&model.to_bytes(), "en").unwrap();
    assert_eq!(reloaded._get_formula("A1"), "=Foo(1)");
    assert_eq!(reloaded._get_formula("A2"), "=Foo(2)");
    // The reloaded workbook still knows the spelling for a new formula, and the call is still
    // one of an unknown name.
    reloaded._set("A3", "=foo(3)");
    reloaded.evaluate();
    assert_eq!(reloaded._get_formula("A3"), "=Foo(3)");
    assert_eq!(reloaded._get_text("A3"), *"#NAME?");
}

#[test]
fn a_let_bound_callee_takes_no_workbook_spelling() {
    let mut model = new_empty_model();
    model._set("A1", "=F(1)");
    model._set("A2", "=LET(f,LAMBDA(x,x*2),F(3))");
    model.evaluate();

    assert_eq!(model._get_formula("A1"), "=F(1)");
    assert_eq!(model._get_formula("A2"), "=LET(f,LAMBDA(x,x*2),f(3))");
    assert_eq!(model._get_text("A2"), *"6");
}

#[test]
fn a_bound_name_use_prints_in_the_spelling_of_its_declaration() {
    // probes/let-name-use-spelling (T-1720): the innermost declaration decides.
    let mut model = new_empty_model();
    model._set("A1", "=LET(x,5,X*2)");
    model._set("A2", "=LAMBDA(Ab,aB*2)(4)");
    model._set("A3", "=LET(x,1,LET(X,5,x))");
    model._set("A4", "=LET(Ab,LAMBDA(Q,q+1),aB(2))");
    model.evaluate();

    let reloaded = crate::Model::from_bytes(&model.to_bytes(), "en").unwrap();
    for model in [&model, &reloaded] {
        assert_eq!(model._get_formula("A1"), "=LET(x,5,x*2)");
        assert_eq!(model._get_formula("A2"), "=LAMBDA(Ab,Ab*2)(4)");
        assert_eq!(model._get_formula("A3"), "=LET(x,1,LET(X,5,X))");
        assert_eq!(model._get_formula("A4"), "=LET(Ab,LAMBDA(Q,Q+1),Ab(2))");
    }
}
