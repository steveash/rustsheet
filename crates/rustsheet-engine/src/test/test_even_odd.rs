#![allow(clippy::unwrap_used)]

use crate::cell::CellValue;
use crate::test::util::new_empty_model;

#[test]
fn arguments() {
    let mut model = new_empty_model();
    model._set("A1", "=EVEN(2)");
    model._set("A2", "=ODD(2)");
    model._set("A3", "=EVEN()");
    model._set("A4", "=ODD()");
    model._set("A5", "=EVEN(1, 2)");
    model._set("A6", "=ODD(1, 2)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"2");
    assert_eq!(model._get_text("A2"), *"3");
    assert_eq!(model._get_text("A3"), *"#ERROR!");
    assert_eq!(model._get_text("A4"), *"#ERROR!");
    assert_eq!(model._get_text("A5"), *"#ERROR!");
    assert_eq!(model._get_text("A6"), *"#ERROR!");
}

#[test]
fn odd_above_2_53_adds_one_in_binary() {
    // probes-numeric/trig-odd-even-sumxmy2-sweep (T-2172, excel-graph): between 2^53 and 2^54
    // `ceil(|x|) + 1` rounds half to even, so x = 2 (mod 4) goes up by 2 and x = 0 (mod 4) stays.
    let mut model = new_empty_model();
    model._set("A1", "=ODD(2^53+2)");
    model._set("A2", "=ODD(2^53+4)");
    model._set("A3", "=ODD(-(2^53+2))");
    model._set("A4", "=ODD(2^53)");
    model._set("A5", "=ODD(2^54+4)");
    model._set("A6", "=ODD(9999999999999999)");
    model._set("A7", "=ODD(1E16-2)");
    model._set("A8", "=ODD(2.5)");
    model._set("A9", "=ODD(-0.5)");
    model.evaluate();
    let number = |c: &str| match model.get_cell_value_by_ref(&format!("Sheet1!{c}")) {
        Ok(CellValue::Number(n)) => n,
        other => panic!("expected number, got {other:?}"),
    };
    assert_eq!(number("A1"), 9_007_199_254_740_996.0);
    assert_eq!(number("A2"), 9_007_199_254_740_996.0);
    assert_eq!(number("A3"), -9_007_199_254_740_996.0);
    assert_eq!(number("A4"), 9_007_199_254_740_992.0);
    assert_eq!(number("A5"), 18_014_398_509_481_988.0);
    assert_eq!(number("A6"), 9_999_999_999_999_992.0);
    assert_eq!(number("A7"), 1e16);
    assert_eq!(number("A8"), 3.0);
    assert_eq!(number("A9"), -1.0);
}
