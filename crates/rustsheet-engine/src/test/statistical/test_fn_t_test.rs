#![allow(clippy::unwrap_used)]
use crate::test::util::new_empty_model;
#[test]
fn test_fn_t_test_smoke() {
    let mut model = new_empty_model();
    model._set("A2", "3");
    model._set("A3", "4");
    model._set("A4", "5");
    model._set("A5", "6");
    model._set("A6", "10");
    model._set("A7", "3");
    model._set("A8", "2");
    model._set("A9", "4");
    model._set("A10", "7");

    model._set("B2", "6");
    model._set("B3", "19");
    model._set("B4", "3");
    model._set("B5", "2");
    model._set("B6", "13");
    model._set("B7", "4");
    model._set("B8", "5");
    model._set("B9", "17");
    model._set("B10", "3");

    model._set("C1", "=T.TEST(A2:A10, B2:B10, 1, 1)");
    model._set("C2", "=T.TEST(A2:A10, B2:B10, 1, 2)");
    model._set("C3", "=T.TEST(A2:A10, B2:B10, 1, 3)");
    model._set("C4", "=T.TEST(A2:A10, B2:B10, 2, 1)");
    model._set("C5", "=T.TEST(A2:A10, B2:B10, 2, 2)");
    model._set("C6", "=T.TEST(A2:A10, B2:B10, 2, 3)");

    model.evaluate();

    assert_eq!(model._get_text("C1"), *"0.103836888");
    assert_eq!(model._get_text("C2"), *"0.100244599");
    assert_eq!(model._get_text("C3"), *"0.105360319");
    assert_eq!(model._get_text("C4"), *"0.207673777");
    assert_eq!(model._get_text("C5"), *"0.200489197");
    assert_eq!(model._get_text("C6"), *"0.210720639");
}

#[test]
fn test_fn_t_test_error_sample_before_tails_and_type() {
    let mut model = new_empty_model();
    for (i, v) in ["3", "4", "5", "6"].iter().enumerate() {
        model._set(&format!("A{}", i + 1), v);
        model._set(&format!("B{}", i + 1), &(i * 2 + 1).to_string());
    }
    model._set("B2", "#N/A");
    model._set("D1", "=T.TEST(1/0,A1:A3,3,2)");
    model._set("D2", "=T.TEST(A1:A3,NA(),1,9)");
    model._set("D3", "=T.TEST(A1:A3,A1:A3,-2.5,\"a\")");
    // A hand expectation: Excel's evidence is the computed empty text (gen-m9 46ac3ef5e61583ca A344); T-2853 probes
    // the typed literal.
    model._set("D4", "=T.TEST(\"\",A1:A4,1,3)");
    // An error inside a range still loses to a bad tails (T-1813).
    model._set("D5", "=T.TEST(B1:B3,A1:A3,3,2)");

    model.evaluate();

    assert_eq!(model._get_text("D1"), *"#DIV/0!");
    assert_eq!(model._get_text("D2"), *"#N/A");
    assert_eq!(model._get_text("D3"), *"#VALUE!");
    assert_eq!(model._get_text("D4"), *"#DIV/0!");
    assert_eq!(model._get_text("D5"), *"#NUM!");
}
