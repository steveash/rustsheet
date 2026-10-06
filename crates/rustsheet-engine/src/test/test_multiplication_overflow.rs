#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

/// probes-numeric/4f413cc257ce15c4 (Calc!A121:A124, excel-graph): a product beyond the double
/// range is `#NUM!` at once, so it stays an error through `&`, CONCAT, TEXT and LEN.
#[test]
fn overflowing_product_is_num_inside_formulas() {
    let mut model = new_empty_model();
    model._set("A1", "=CONCAT(1e307*100)");
    model._set("A2", "=(1e307*100)&\"\"");
    model._set("A3", "=TEXT(1e307*100,\"0.000000000000000000\")");
    model._set("A4", "=LEN((1e307*100)&\"\")");
    model._set("A5", "=10^308*10");
    model._set("A6", "=1E+307*10-10^308");
    model._set("A7", "=-1e307*100");
    model.evaluate();

    for cell in ["A1", "A2", "A3", "A4", "A5", "A7"] {
        assert_eq!(model._get_text(cell), *"#NUM!", "{cell}");
    }
    assert_eq!(model._get_text("A6"), *"0");
}

/// The array path shares the closure: only the overflowing element is an error.
#[test]
fn overflow_in_an_array_product_is_per_element() {
    let mut model = new_empty_model();
    model._set("A1", "=INDEX({1e307,1}*100,1,1)");
    model._set("B1", "=INDEX({1e307,1}*100,1,2)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#NUM!");
    assert_eq!(model._get_text("B1"), *"100");
}
