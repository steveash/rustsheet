#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

/// probes/valuetotext-format (excel-graph, T-1229).
#[test]
fn valuetotext_format_selects_quoted_text_for_one() {
    let mut model = new_empty_model();
    model._set("A1", "ab");
    model._set("A2", "a\"b");
    model._set("A4", "=NA()");
    let cases = [
        ("C1", "=VALUETOTEXT(A1,1)", "\"ab\""),
        ("C2", "=VALUETOTEXT(A2,1)", "\"a\"\"b\""),
        ("C3", "=VALUETOTEXT(A3,1)", ""),
        ("C4", "=VALUETOTEXT(A4,1)", "#N/A"),
        ("C5", "=VALUETOTEXT(A1,0)", "ab"),
        ("C6", "=VALUETOTEXT(A1,2)", "#VALUE!"),
        ("C7", "=VALUETOTEXT(A1,1.9)", "\"ab\""),
        ("C8", "=VALUETOTEXT(A1,-0.5)", "#VALUE!"),
        ("C9", "=VALUETOTEXT(A1,1E-20)", "ab"),
        ("C10", "=VALUETOTEXT(A1,#N/A)", "#N/A"),
        ("C11", "=VALUETOTEXT(A1,\"x\")", "#VALUE!"),
        ("C12", "=VALUETOTEXT(A1,\"1\")", "\"ab\""),
        ("C13", "=VALUETOTEXT(A1,TRUE)", "\"ab\""),
        ("C14", "=VALUETOTEXT(A1,A3)", "ab"),
        ("E1", "=VALUETOTEXT(A1:A2,{0,1})", "ab"),
        ("E8", "=VALUETOTEXT(A1,{0,1})", "ab"),
    ];
    for (cell, formula, _) in cases {
        model._set(cell, formula);
    }
    model.evaluate();
    for (cell, formula, expected) in cases {
        assert_eq!(model._get_text(cell), expected, "{formula}");
    }
    assert_eq!(model._get_text("F1"), "\"ab\"");
    assert_eq!(model._get_text("F2"), "\"a\"\"b\"");
    assert_eq!(model._get_text("F8"), "\"ab\"");
}
