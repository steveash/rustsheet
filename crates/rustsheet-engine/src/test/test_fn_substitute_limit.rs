#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

// Oracle: probes/text-result-limit C1-C7, C11 (excel-graph): a SUBSTITUTE result over 32767 chars is
// #VALUE!, exactly 32767 is fine.
#[test]
fn substitute_result_over_32767_chars_is_value_error() {
    let mut model = new_empty_model();
    model._set("A1", "=LEN(SUBSTITUTE(REPT(\"a\",20000),\"a\",\"bb\"))");
    model._set("A2", "=LEN(SUBSTITUTE(REPT(\"a\",16383),\"a\",\"bb\"))");
    model._set("A3", "=LEN(SUBSTITUTE(REPT(\"a\",16384),\"a\",\"bb\"))");
    model._set("A4", "=LEN(SUBSTITUTE(REPT(\"a\",32767),\"a\",\"b\"))");
    model._set("A5", "=LEN(SUBSTITUTE(REPT(\"a\",32767),\"a\",\"bb\",1))");
    model._set("A6", "=LEN(SUBSTITUTE(REPT(\"a\",32767),\"a\",\"bb\",2))");
    model._set(
        "A7",
        "=LEN(SUBSTITUTE(REPT(\"a\",32767),\"a\",REPT(\"b\",32767),1))",
    );
    model._set(
        "A8",
        "=LEN(SUBSTITUTE(REPT(\"a\",32767),\"z\",REPT(\"b\",32767),1))",
    );

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#VALUE!");
    assert_eq!(model._get_text("A2"), *"32766");
    assert_eq!(model._get_text("A3"), *"#VALUE!");
    assert_eq!(model._get_text("A4"), *"32767");
    assert_eq!(model._get_text("A5"), *"#VALUE!");
    assert_eq!(model._get_text("A6"), *"#VALUE!");
    assert_eq!(model._get_text("A7"), *"#VALUE!");
    assert_eq!(model._get_text("A8"), *"32767");
}

// Oracle: probes/substitute-instance-num-bound (excel-graph, T-2279): instance_num is valid for
// 1..=2147483646 after flooring; 2147483647 and above are #VALUE!, match or not.
#[test]
fn substitute_instance_num_above_2_pow_31_minus_2_is_value_error() {
    let mut model = new_empty_model();
    model._set("A1", "=SUBSTITUTE(\"bbb\",\"b\",\"x\",2147483646.9)");
    model._set("A2", "=SUBSTITUTE(\"bbb\",\"b\",\"x\",2147483647)");
    model._set("A3", "=SUBSTITUTE(\"abc\",\"z\",\"x\",1E+15)");
    model._set("A4", "=SUBSTITUTE(\"abc\",\"\",\"x\",1E+15)");
    model._set("A5", "=SUBSTITUTE(\"bbb\",\"b\",\"x\",1.5)");
    model._set("A6", "=SUBSTITUTE(\"\",\"b\",\"x\",1E+15)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"bbb");
    assert_eq!(model._get_text("A2"), *"#VALUE!");
    assert_eq!(model._get_text("A3"), *"#VALUE!");
    assert_eq!(model._get_text("A4"), *"#VALUE!");
    assert_eq!(model._get_text("A5"), *"xbb");
    assert_eq!(model._get_text("A6"), *"#VALUE!");
}
