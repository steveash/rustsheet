#![allow(clippy::unwrap_used, reason = "tests")]

//! TEXT applies the format's text section to text and booleans.

use crate::test::util::new_empty_model;

#[test]
fn test_text_applies_text_section_to_text_and_booleans() {
    // poi TextFormatTests A3:A10, A12:A29; GeneralFormatTests A72, A73 (Excel-cached).
    let mut model = new_empty_model();
    model._set("A1", r#"=TEXT("jello",";;;-@-@-")"#);
    model._set("A2", r#"=TEXT("jello",";;;""hi""")"#);
    model._set("A3", r#"=TEXT("jello",";;;\@@\@")"#);
    model._set("A4", r#"=TEXT(TRUE,";;;-@-")"#);
    model._set("A5", r#"=TEXT(FALSE,";;;-@-@-")"#);
    model._set("A6", r#"=TEXT(TRUE,"General")"#);
    model._set("A7", r#"=TEXT(FALSE,"General")"#);
    model._set("B1", "jello");
    model._set("B2", "TRUE");
    model._set("C1", r#"=TEXT(B1:B1,";;;-@-")"#);
    model._set("D1", r#"=TEXT({"a";TRUE},";;;<@>")"#);
    model.evaluate();
    assert_eq!(model._get_text("A1"), "-jello-jello-");
    assert_eq!(model._get_text("A2"), "hi");
    assert_eq!(model._get_text("A3"), "@jello@");
    assert_eq!(model._get_text("A4"), "-TRUE-");
    assert_eq!(model._get_text("A5"), "-FALSE-FALSE-");
    assert_eq!(model._get_text("A6"), "TRUE");
    assert_eq!(model._get_text("A7"), "FALSE");
    assert_eq!(model._get_text("C1"), "-jello-");
    assert_eq!(model._get_text("D1"), "<a>");
    assert_eq!(model._get_text("D2"), "<TRUE>");
}

#[test]
fn test_text_section_doubled_quotes_and_single_at_section() {
    // A doubled quote ends one literal and starts the next; a lone section with `@` is the text
    // section (probes/format-doubled-quotes C8, C11; poi TextFormatTests A4).
    let mut model = new_empty_model();
    model._set("A1", r#"=TEXT("jello",";;;""h""""I""")"#);
    model._set("A2", r#"=TEXT("x","@"" """"q""""""")"#);
    model.evaluate();
    assert_eq!(model._get_text("A1"), "hI");
    assert_eq!(model._get_text("A2"), "x q");
}

#[test]
fn test_text_on_text_over_255_characters_is_value_error() {
    // probes/text-format-text-limit A10, A11, A20, A24, A25.
    let mut model = new_empty_model();
    model._set("A1", r#"=LEN(TEXT(REPT("a",255),";;;@"))"#);
    model._set("A2", r#"=TEXT(REPT("a",256),";;;@")"#);
    model._set("A3", r#"=TEXT(REPT("a",256),"General")"#);
    model._set("A4", r#"=TEXT(REPT("a",128),";;;@@")"#);
    model._set("A5", r#"=LEN(TEXT(REPT("a",127),";;;@@""x"""))"#);
    model._set("A6", r#"=TEXT(REPT("a",32767),REPT("@",1000))"#);
    model.evaluate();
    assert_eq!(model._get_text("A1"), "255");
    assert_eq!(model._get_text("A2"), "#VALUE!");
    assert_eq!(model._get_text("A3"), "#VALUE!");
    assert_eq!(model._get_text("A4"), "#VALUE!");
    assert_eq!(model._get_text("A5"), "255");
    assert_eq!(model._get_text("A6"), "#VALUE!");
}
