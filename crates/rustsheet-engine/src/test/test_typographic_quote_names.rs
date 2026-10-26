#![allow(clippy::unwrap_used)]

//! Typographic quotes are name characters, so a stored `“D”` is an undefined name (T-0481).
//! Excel values: probes/t0481-curly-quote-names, -b (excel-graph, 2026-10-26) and
//! corpus-bench/48354 D2:D6.

use crate::test::util::new_empty_model;

#[test]
fn curly_quoted_text_is_an_undefined_name() {
    let mut model = new_empty_model();
    model._set("B1", "domestic");
    model._set("A1", "=“D”");
    model._set("A2", "=“D”&\"x\"");
    model._set(
        "A4",
        "=IF(B1=\"domestic\",“D”,IF(B1=\"international\",“I”,“N/A”))",
    );
    model._set("A5", "=ISREF(“D”)");
    model._set("A6", "=ERROR.TYPE(“D”)");
    model._set("A7", "=FORMULATEXT(A4)");
    model._set("A10", "=IF(FALSE,“D”,1)");
    model._set("C1", "=“N/A”");
    model._set("C2", "=SUM(“A1”)");
    // A well-formed string literal is unaffected, curly quotes inside it included.
    model._set("C3", "=\"“D”\"&\"x\"");

    model.evaluate();

    for cell in ["A1", "A2", "A4", "C1", "C2"] {
        assert_eq!(model._get_text(cell), *"#NAME?", "{cell}");
    }
    assert_eq!(model._get_text("A5"), *"FALSE");
    assert_eq!(model._get_text("A6"), *"5");
    assert_eq!(
        model._get_text("A7"),
        *"=IF(B1=\"domestic\",“D”,IF(B1=\"international\",“I”,“N/A”))"
    );
    assert_eq!(model._get_text("A10"), *"1");
    assert_eq!(model._get_text("C3"), *"“D”x");
}
