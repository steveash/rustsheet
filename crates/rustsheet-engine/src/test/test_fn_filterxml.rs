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

// docs/excel-semantics/web.md: `#VALUE!` for a valid call; A2 is backed by
// gen-m6/abcfd9979f316267 A9, A3 and A4 by probes/filterxml-error-arguments (T-1570).
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

// probes/filterxml-evidence-floor D28 (T-1704): an array argument gives `#VALUE!` per element.
#[test]
fn fn_filterxml_array_xml_lifts() {
    let mut model = new_empty_model();
    model._set("A1", r#"=FILTERXML({"<a>1</a>";"<a>2</a>"},"//a")"#);
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"#VALUE!");
    assert_eq!(model._get_text("A2"), *"#VALUE!");
}

// Errors pass
// through per element, and a scalar error next to an array broadcasts.
#[test]
fn fn_filterxml_array_error_elements() {
    let mut model = new_empty_model();
    model._set("A1", r#"=FILTERXML({"<a>1</a>";#N/A},"//a")"#);
    model._set("B1", r#"=FILTERXML(NA(),{"//a";"//b"})"#);
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"#VALUE!");
    assert_eq!(model._get_text("A2"), *"#N/A");
    assert_eq!(model._get_text("B1"), *"#N/A");
    assert_eq!(model._get_text("B2"), *"#N/A");
}
