#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn fn_encodeurl_args_number() {
    let mut model = new_empty_model();

    model._set("A1", "=ENCODEURL()");
    model._set("A2", r#"=ENCODEURL("a", "b")"#);

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#ERROR!");
    assert_eq!(model._get_text("A2"), *"#ERROR!");
}

// docs/excel-semantics/web.md: `#VALUE!` for every argument, lifted over an array
// (probes/info-logic-argument-modes-2 A1:A6, B5; gen-m9/fcf11f2ec6f7a59d A7).
#[test]
fn fn_encodeurl_is_value_error_on_the_web() {
    let mut model = new_empty_model();

    model._set("A1", r#"=ENCODEURL("a b&c")"#);
    model._set("A2", r#"=ENCODEURL("abc")"#);
    model._set("A3", "=ENCODEURL(1)");
    model._set("A4", "=ENCODEURL(TRUE)");
    model._set("D1", "a b");
    model._set("D2", "c");
    model._set("C5", "=ENCODEURL(D1:D2)");
    model._set("A6", "=ENCODEURL(1/0)");
    model._set("A7", r#"=CODE(ENCODEURL("a"))"#);

    model.evaluate();

    for cell in ["A1", "A2", "A3", "A4", "C5", "C6", "A7"] {
        assert_eq!(model._get_text(cell), *"#VALUE!", "{cell}");
    }
    assert_eq!(model._get_text("A6"), *"#DIV/0!");
}
