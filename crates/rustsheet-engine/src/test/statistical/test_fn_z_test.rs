#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn test_fn_z_test_smoke() {
    let mut model = new_empty_model();
    model._set("A2", "3");
    model._set("A3", "6");
    model._set("A4", "7");
    model._set("A5", "8");
    model._set("A6", "6");
    model._set("A7", "5");
    model._set("A8", "4");
    model._set("A9", "2");
    model._set("A10", "1");
    model._set("A11", "9");

    model._set("G1", "=Z.TEST(A2:A11, 4)");
    model._set("G2", "=Z.TEST(A2:A11, 6)");
    model.evaluate();

    assert_eq!(model._get_text("G1"), *"0.090574197");
    assert_eq!(model._get_text("G2"), *"0.863043389");
}

#[test]
fn arrays() {
    let mut model = new_empty_model();
    model._set("D1", "=Z.TEST({5,2,3,4}, 4, 123)");
    model._set("D2", "=Z.TEST({5,2,3,4}, 4)");
    model.evaluate();

    assert_eq!(model._get_text("D1"), *"0.503243397");
    assert_eq!(model._get_text("D2"), *"0.780710987");
}

/// A data sum that overflows makes the omitted sigma NaN; the result is an error, never a NaN
/// number. `#NUM!` is not probed yet (T-1942).
#[test]
fn overflowing_data_is_an_error() {
    let mut model = new_empty_model();
    model._set("A1", "9E307");
    model._set("A2", "9E307");
    model._set("A3", "1");
    model._set("D1", "=Z.TEST(A1:A3,0)");
    model.evaluate();

    assert_eq!(model._get_text("D1"), *"#NUM!");
}

/// A data argument that is an error value comes first in every element; an error inside a range
/// or array comes after `x` (probes/t2428-ztest-error-data).
#[test]
fn an_error_data_value_comes_before_x() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("A2", "=NA()");
    for (cell, formula) in [
        ("D1", "=Z.TEST(NA(),1/0)"),
        ("D2", "=Z.TEST(NA(),\"a\")"),
        ("D3", "=Z.TEST(A1:A2,1/0)"),
        ("D4", "=Z.TEST({1,2,#N/A},1/0)"),
        ("D5", "=COLUMNS(Z.TEST(NA(),{1,2}))"),
        ("D6", "=INDEX(Z.TEST(NA(),{1,2}),1,2)"),
    ] {
        model._set(cell, formula);
    }
    model.evaluate();

    assert_eq!(model._get_text("D1"), *"#N/A");
    assert_eq!(model._get_text("D2"), *"#N/A");
    assert_eq!(model._get_text("D3"), *"#DIV/0!");
    assert_eq!(model._get_text("D4"), *"#DIV/0!");
    assert_eq!(model._get_text("D5"), *"2");
    assert_eq!(model._get_text("D6"), *"#N/A");
}

/// A one-value data argument is `#DIV/0!` before a bad sigma; a lone cell reference without a
/// number is `#N/A` (gen-m9 T-2256 cells; statistics.md "Z.TEST: an error array first").
#[test]
fn lone_values_and_cells() {
    let mut model = new_empty_model();
    model._set("A1", "x");
    model._set("B1", "5");
    for (i, v) in [3, 4, 5, 6].iter().enumerate() {
        model._set(&format!("C{}", i + 1), &v.to_string());
    }
    for (cell, formula) in [
        ("D1", "=Z.TEST(1/0,\"x\")"),
        ("D2", "=Z.TEST({5},3,-1)"),
        ("D3", "=Z.TEST(A1,1)"),
        ("D4", "=Z.TEST(A1:A1,1)"),
        ("D5", "=Z.TEST(C1:C4,4,0)"),
        ("D6", "=Z.TEST(Z9,1)"),
        ("D7", "=Z.TEST(B1,3,-1)"),
    ] {
        model._set(cell, formula);
    }
    model.evaluate();
    for (cell, want) in [
        ("D1", "#DIV/0!"),
        ("D2", "#DIV/0!"),
        ("D3", "#N/A"),
        ("D4", "#N/A"),
        ("D5", "#NUM!"),
        ("D6", "#N/A"),
        ("D7", "#DIV/0!"),
    ] {
        assert_eq!(model._get_text(cell), *want, "{cell}");
    }
}
