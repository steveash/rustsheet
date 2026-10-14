#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn fn_filterxml_args_number() {
    let mut model = new_empty_model();
    model._set("A1", "=FILTERXML()");
    model._set("A2", r#"=FILTERXML("<a>1</a>")"#);
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"#ERROR!");
    assert_eq!(model._get_text("A2"), *"#ERROR!");
}

// docs/excel-semantics/web.md: `#VALUE!` for a valid call; A2 is oracle-backed (first argument error).
// A3 and A4 are hand-expected (second-argument error, both erring) pending an oracle probe.
#[test]
fn fn_filterxml_value_and_error_precedence() {
    let mut model = new_empty_model();
    model._set("A1", r#"=FILTERXML("<a>1</a>","//a")"#);
    model._set("A2", r#"=FILTERXML("x"&NA(),"y")"#);
    model._set("A3", r#"=FILTERXML(1/0,NA())"#);
    model._set("A4", r#"=FILTERXML("x",NA())"#);
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"#VALUE!");
    assert_eq!(model._get_text("A2"), *"#N/A");
    assert_eq!(model._get_text("A3"), *"#DIV/0!");
    assert_eq!(model._get_text("A4"), *"#N/A");
}
