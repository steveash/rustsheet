#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn test_fn_f_test_sanity() {
    let mut model = new_empty_model();

    // Valid call
    model._set("A1", "=F.TEST(A2:A7, B2:B7)");
    model._set("A2", "9");
    model._set("A3", "12");
    model._set("A4", "14");
    model._set("A5", "16");
    model._set("A6", "18");
    model._set("A7", "20");
    model._set("B2", "11");
    model._set("B3", "10");
    model._set("B4", "15");
    model._set("B5", "17");
    model._set("B6", "19");
    model._set("B7", "21");

    // Too few args
    model._set("A8", "=F.TEST(A2:A7)");

    // Too many args
    model._set("A9", "=F.TEST(A2:A7, B2:B7, C2:C7)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"0.859284302");
    assert_eq!(model._get_text("A8"), *"#ERROR!");
    assert_eq!(model._get_text("A9"), *"#ERROR!");
}

// A single value is a sample of one (`#DIV/0!`) whatever its type, but a one-cell reference to a
// blank cell is `#VALUE!`, ahead of the other sample's error (probes/t2455-transpose-single-value
// A8:A40, T-2455).
#[test]
fn f_test_of_a_single_value() {
    let mut model = new_empty_model();
    model._set("B1", "5");
    model._set("A1", "=FTEST(TRANSPOSE(5),{1;2;3})");
    model._set("A2", r#"=FTEST("abc",{1;2;3})"#);
    model._set("A3", "=FTEST({1;2;3},B1)");
    model._set("A4", "=FTEST(B10,{1;2;3})");
    model._set("A5", "=FTEST(1/0,B10:B10)");
    model._set("A6", "=FTEST(B10:B11,{1;2;3})");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#DIV/0!");
    assert_eq!(model._get_text("A2"), "#DIV/0!");
    assert_eq!(model._get_text("A3"), "#DIV/0!");
    assert_eq!(model._get_text("A4"), "#VALUE!");
    assert_eq!(model._get_text("A5"), "#VALUE!");
    assert_eq!(model._get_text("A6"), "#DIV/0!");
}
