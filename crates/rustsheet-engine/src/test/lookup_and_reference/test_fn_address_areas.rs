#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

// ── ADDRESS ───────────────────────────────────────────────────────────────────

#[test]
fn test_address_absolute() {
    let mut model = new_empty_model();
    model._set("A1", "=ADDRESS(1,1)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "$A$1");
}

#[test]
fn test_address_row_abs_col_rel() {
    let mut model = new_empty_model();
    model._set("A1", "=ADDRESS(3,5,2)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "E$3");
}

#[test]
fn test_address_row_rel_col_abs() {
    let mut model = new_empty_model();
    model._set("A1", "=ADDRESS(3,5,3)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "$E3");
}

#[test]
fn test_address_relative() {
    let mut model = new_empty_model();
    model._set("A1", "=ADDRESS(3,5,4)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "E3");
}

// Bounds and flooring per abs_num and a1 (probes/t2275-address-bounds, excel-graph).
#[test]
fn test_address_bounds_and_relative_offsets() {
    let cases = [
        ("=ADDRESS(2.9,3)", "$C$2"),
        ("=ADDRESS(1048576.9,3,4)", "C1048576"),
        ("=ADDRESS(1048577,3)", "#VALUE!"),
        ("=ADDRESS(5,16385,4,FALSE)", "#VALUE!"),
        ("=ADDRESS(0.5,3)", "#VALUE!"),
        ("=ADDRESS(0,3,3,FALSE)", "RC3"),
        ("=ADDRESS(9,0,2,FALSE)", "R9C"),
        ("=ADDRESS(10,-2.5,4,FALSE)", "R[10]C[-3]"),
        ("=ADDRESS(-0.5,3,3,FALSE)", "R[-1]C3"),
        ("=ADDRESS(-1048575,3,3,FALSE)", "R[-1048575]C3"),
        ("=ADDRESS(-1048576,3,3,FALSE)", "#VALUE!"),
        ("=ADDRESS(1048576,3,3,FALSE)", "#VALUE!"),
        ("=ADDRESS(5,16384,2,FALSE)", "#VALUE!"),
        ("=ADDRESS(5,16384,1,FALSE)", "R5C16384"),
        ("=ADDRESS(-1,3.14159,2,NA())", "#N/A"),
    ];
    let mut model = new_empty_model();
    for (i, (formula, _)) in cases.iter().enumerate() {
        model._set(&format!("A{}", i + 1), formula);
    }
    model.evaluate();
    for (i, (formula, expected)) in cases.iter().enumerate() {
        assert_eq!(
            model._get_text(&format!("A{}", i + 1)),
            *expected,
            "{formula}"
        );
    }
}

#[test]
fn test_address_with_sheet() {
    let mut model = new_empty_model();
    model._set("A1", r#"=ADDRESS(1,1,1,TRUE,"Sheet1")"#);
    model.evaluate();
    assert_eq!(model._get_text("A1"), "Sheet1!$A$1");
}

#[test]
fn test_address_r1c1_absolute() {
    let mut model = new_empty_model();
    model._set("A1", "=ADDRESS(3,5,1,FALSE)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "R3C5");
}

#[test]
fn test_address_r1c1_relative() {
    let mut model = new_empty_model();
    model._set("A1", "=ADDRESS(3,5,4,FALSE)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "R[3]C[5]");
}

#[test]
fn test_address_large_column() {
    let mut model = new_empty_model();
    model._set("A1", "=ADDRESS(1,27)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "$AA$1");
}

#[test]
fn test_address_empty_sheet_text() {
    let mut model = new_empty_model();
    // A present-but-empty sheet_text still emits the "!" separator.
    model._set("A1", r#"=ADDRESS(1,1,1,TRUE,"")"#);
    // An omitted sheet_text emits no prefix.
    model._set("A2", "=ADDRESS(1,1)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "!$A$1");
    assert_eq!(model._get_text("A2"), "$A$1");
}

#[test]
fn test_address_quoted_sheet_text() {
    let mut model = new_empty_model();
    // A name that needs quoting is wrapped in single quotes.
    model._set("A1", r#"=ADDRESS(1,1,1,TRUE,"My Sheet")"#);
    // The maximum column still renders correctly with a sheet prefix.
    model._set("A2", "=ADDRESS(1,16384)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "'My Sheet'!$A$1");
    assert_eq!(model._get_text("A2"), "$XFD$1");
}

// ── AREAS ─────────────────────────────────────────────────────────────────────

#[test]
fn test_areas_single_cell() {
    let mut model = new_empty_model();
    model._set("A1", "=AREAS(B2)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "1");
}

#[test]
fn test_areas_range() {
    let mut model = new_empty_model();
    model._set("A1", "=AREAS(B2:D4)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "1");
}

#[test]
fn test_address_book_sheet_text_unquoted() {
    // Excel-cached: poi FormulaEvalTestData_Copy EverythingTests!F128.
    let mut model = new_empty_model();
    model._set("A1", "=ADDRESS(2,3,3,TRUE,\"[Book1]Sheet1\")");
    model._set("A2", "=ADDRESS(1,1,1,TRUE,\"Sheet 1\")");
    model._set("A3", "=ADDRESS(1,1,1,TRUE,\"Sheet2\")");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "[Book1]Sheet1!$C2");
    assert_eq!(model._get_text("A2"), "'Sheet 1'!$A$1");
    assert_eq!(model._get_text("A3"), "Sheet2!$A$1");
}

#[test]
fn test_address_argument_error_beats_own_value_error() {
    // gen-m6/0fd242eb9b2456a8 A280, 870c5e38f51ce5d4 A229: a later argument's error wins over a
    // bad row or abs_num.
    let mut model = new_empty_model();
    model._set("A1", "=ADDRESS(-1,1,1,NA())");
    model._set("A2", "=ADDRESS(1,1,0,NA())");
    model._set("A3", "=ADDRESS(1,1,0)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#N/A");
    assert_eq!(model._get_text("A2"), "#N/A");
    assert_eq!(model._get_text("A3"), "#VALUE!");
}

#[test]
fn test_address_boolean_sheet_text_is_quoted() {
    // gen-m6/c96ac1ada46179c4 A197, ee56a4ed5a612a0d A306.
    let mut model = new_empty_model();
    model._set("A1", "=ADDRESS(1,1,1,TRUE,\"FALSE\")");
    model._set("A2", "=ADDRESS(9,26,3,TRUE,\"TRUE\")");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "'FALSE'!$A$1");
    assert_eq!(model._get_text("A2"), "'TRUE'!$Z9");
}
