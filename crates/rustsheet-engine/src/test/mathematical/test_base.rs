#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn base_min_length_is_at_most_255() {
    // Excel (probes/base-min-length): min_length truncates and may be at most 255; beyond it BASE is
    // #NUM! without padding, so `BASE(245,30,1E+15)` must not allocate 10^15 characters (T-1243).
    let mut model = new_empty_model();
    model._set("A1", "=LEN(BASE(1,2,255.9))");
    model._set("A2", "=BASE(1,2,256)");
    model._set("A3", "=BASE(245,30,1E+15)");
    model._set("A4", "=LEN(BASE(1,2,254.5))");
    model._set("A5", "=BASE(\"x\",2,1E+15)");
    model._set("A6", "=BASE(1,2,\"300\")");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "255");
    assert_eq!(model._get_text("A2"), "#NUM!");
    assert_eq!(model._get_text("A3"), "#NUM!");
    assert_eq!(model._get_text("A4"), "254");
    assert_eq!(model._get_text("A5"), "#VALUE!");
    assert_eq!(model._get_text("A6"), "#NUM!");
}
