#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

// ROWS/COLUMNS count an array without reading it, so all-error arrays still count
// (probes/error-operand-broadcast C1, C3, C6-C8, C10).
#[test]
fn test_rows_columns_of_array_values() {
    let mut model = new_empty_model();
    model._set("A1", "=ROWS({1;2;3})");
    model._set("A2", "=COLUMNS({1,2,3,4})");
    model._set("A3", "=ROWS(NA()-{1;2;3})");
    model._set("A4", "=ROWS({1;2;3}/NA())");
    model._set("A5", "=ROWS(NA()+B1:B3)");
    model._set("A6", "=ROWS(\"abc\"*{1,2})");
    model._set("A7", "=COLUMNS(1/0+{1,2,3,4})");
    model._set("A8", "=ROWS(B1:B3^NA())");
    model._set("A9", "=COLUMNS({1,2;3,4})");
    model._set("A10", "=ROWS(B1:C4)");
    model.evaluate();
    let expected = ["3", "4", "3", "3", "3", "1", "4", "3", "2", "4"];
    for (i, want) in expected.iter().enumerate() {
        assert_eq!(model._get_text(&format!("A{}", i + 1)), *want, "A{}", i + 1);
    }
}
