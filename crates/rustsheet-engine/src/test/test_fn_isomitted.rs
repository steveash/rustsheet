#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn test_isomitted_omitted_optional_arg_returns_true() {
    let mut model = new_empty_model();
    // [b] is optional; calling with only one arg leaves b as EmptyArg → TRUE
    model._set("A1", "=LAMBDA(a, [b], ISOMITTED(b))(12)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"TRUE");
}

#[test]
fn test_isomitted_provided_arg_returns_false() {
    let mut model = new_empty_model();
    model._set("A1", "=LAMBDA(a, [b], ISOMITTED(b))(12, 3)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"FALSE");
}

#[test]
fn test_isomitted_empty_string_is_not_omitted() {
    let mut model = new_empty_model();
    model._set("A1", "=LAMBDA(a, [b], ISOMITTED(b))(12, \"\")");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"FALSE");
}

#[test]
fn test_isomitted_zero_is_not_omitted() {
    let mut model = new_empty_model();
    model._set("A1", "=LAMBDA(a, [b], ISOMITTED(b))(12, 0)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"FALSE");
}

#[test]
fn test_isomitted_wrong_arg_count() {
    let mut model = new_empty_model();
    model._set("A1", "=ISOMITTED()");
    model._set("A2", "=ISOMITTED(1, 2)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"#ERROR!");
    assert_eq!(model._get_text("A2"), *"#ERROR!");
}

#[test]
fn test_isomitted_optional_arg_default_pattern() {
    // Classic ISOMITTED usage: provide a default when the optional arg is missing.
    // LAMBDA(a, [b], IF(ISOMITTED(b), a/2, a*b))
    let mut model = new_empty_model();
    model._set("A1", "=LAMBDA(a, [b], IF(ISOMITTED(b), a/2, a*b))(12)");
    model._set("A2", "=LAMBDA(a, [b], IF(ISOMITTED(b), a/2, a*b))(12, 3)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"6");
    assert_eq!(model._get_text("A2"), *"36");
}

#[test]
fn test_isomitted_function_result_empty_value_is_not_omitted() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("B1", "5");
    // LOOKUP hands back the blank cell B2 as the empty value, which is not an omitted argument.
    model._set("D1", "=ISOMITTED(LOOKUP(1,A1:A1,B2:B2))");
    model.evaluate();
    assert_eq!(model._get_text("D1"), *"FALSE");
}

#[test]
fn test_isomitted_parameter_bound_to_the_empty_value_is_not_omitted() {
    // probes/t2461-blank-array-sources A12:A14 (excel-graph): Z1:Z2 blank. The empty value a
    // parameter is bound to is supplied; an empty slot or a missing argument stays omitted.
    let mut model = new_empty_model();
    model._set(
        "A1",
        r#"=TEXTJOIN(",",FALSE,MAP(HSTACK(Z1:Z2,{1;2}),LAMBDA(v,ISOMITTED(v))))"#,
    );
    model._set("A2", "=LAMBDA(x,ISOMITTED(x))(IFERROR(Z1,1))");
    model._set(
        "A3",
        "=LAMBDA(x,ISOMITTED(x))(INDEX(HSTACK(Z1:Z2,{1;2}),1,1))",
    );
    model._set("A4", "=LAMBDA(x,[y],ISOMITTED(y))(1,)");
    model._set("A5", "=LAMBDA(x,[y],ISOMITTED(x))(,2)");
    model._set("A6", "=LAMBDA(x,[y],LET(z,y,ISOMITTED(z)))(1)");
    model._set("A7", "=LAMBDA(x,[y],ISOMITTED((y)))(1)");
    model._set("A8", "=LAMBDA(x,COUNT(x)&ISBLANK(x))(IFERROR(Z1,1))");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"FALSE,FALSE,FALSE,FALSE");
    assert_eq!(model._get_text("A2"), *"FALSE");
    assert_eq!(model._get_text("A3"), *"FALSE");
    assert_eq!(model._get_text("A4"), *"TRUE");
    assert_eq!(model._get_text("A5"), *"TRUE");
    assert_eq!(model._get_text("A6"), *"TRUE");
    assert_eq!(model._get_text("A7"), *"TRUE");
    assert_eq!(model._get_text("A8"), *"1TRUE");
}

#[test]
fn test_isomitted_let_name_bound_to_an_empty_slot_is_omitted() {
    // As before T-2649 (unprobed): a LET name bound to an empty slot reads as omitted.
    let mut model = new_empty_model();
    model._set("A1", "=LET(z,,ISOMITTED(z))");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"TRUE");
}
