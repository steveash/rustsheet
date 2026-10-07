#![allow(clippy::unwrap_used)]

//! AGGREGATE options over hidden rows and nested SUBTOTAL cells, which the probe case format
//! cannot express (docs/excel-semantics/numbers.md "AGGREGATE", UNVERIFIED parts).
use crate::test::util::new_empty_model;

#[test]
fn hidden_rows_and_nested_subtotals() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("A2", "2");
    model._set("A3", "4");
    model._set("A4", "=SUBTOTAL(9,A1:A2)");
    model.set_row_hidden(0, 2, true).unwrap();
    model._set("B1", "=AGGREGATE(9,4,A1:A4)");
    model._set("B2", "=AGGREGATE(9,5,A1:A4)");
    model._set("B3", "=AGGREGATE(9,0,A1:A4)");
    model._set("B4", "=AGGREGATE(9,1,A1:A4)");
    model._set("B5", "=AGGREGATE(14,5,A1:A4,1)");
    model.evaluate();

    // Option 4 keeps everything; 5 drops hidden row 2; 0 drops the SUBTOTAL cell; 1 both.
    assert_eq!(model._get_text("B1"), *"10");
    assert_eq!(model._get_text("B2"), *"8");
    assert_eq!(model._get_text("B3"), *"7");
    assert_eq!(model._get_text("B4"), *"5");
    assert_eq!(model._get_text("B5"), *"4");
}
