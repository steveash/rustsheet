//! TEXTSPLIT checks the size of its padded result before padding any row, as `&`, the element-wise
//! text functions and HSTACK/VSTACK do (T-1781, CLAUDE.md directive 5). DEVIATION (decision 0014):
//! a 32,767-char text pads up to ~2^28 cells; we refuse one above a whole column
//! (docs/excel-semantics/arrays.md "Array results above 1,048,576 cells").

use crate::test::util::new_empty_model;

#[test]
fn padded_result_above_the_limit_is_bounded() {
    let mut model = new_empty_model();
    // 1501 x 1501 = 2,253,001 cells; 1025 x 1025 = 1,050,625, just above 2^20.
    model._set(
        "A1",
        r#"=ROWS(TEXTSPLIT(REPT(",",1500)&REPT(";",1500),",",";"))"#,
    );
    model._set(
        "A2",
        r#"=COLUMNS(TEXTSPLIT(REPT(",",1500)&REPT(";",1500),",",";"))"#,
    );
    model._set(
        "A3",
        r#"=ROWS(TEXTSPLIT(REPT(",",1024)&REPT(";",1024),",",";"))"#,
    );
    model._set("A4", r#"=TEXTSPLIT(REPT(",",1500)&REPT(";",1500),",",";")"#);

    model.evaluate();

    for cell in ["A1", "A2", "A3", "A4"] {
        assert_eq!(model._get_text(cell), *"#ERROR!", "{cell}");
    }
}

#[test]
fn results_under_the_limit_are_unchanged() {
    let mut model = new_empty_model();
    // 1024 x 1024 = 2^20 cells exactly fits.
    model._set(
        "A1",
        r#"=ROWS(TEXTSPLIT(REPT(",",1023)&REPT(";",1023),",",";"))"#,
    );
    model._set(
        "A2",
        r#"=COLUMNS(TEXTSPLIT(REPT(",",1023)&REPT(";",1023),",",";"))"#,
    );
    // Padding with #N/A or pad_with below the limit still applies.
    model._set("C1", r#"=TEXTSPLIT("a,b;c",",",";")"#);
    model._set("C4", r#"=TEXTSPLIT("a,b;c",",",";",,,"-")"#);

    model.evaluate();

    assert_eq!(model._get_text("A1"), "1024");
    assert_eq!(model._get_text("A2"), "1024");
    assert_eq!(model._get_text("C1"), "a");
    assert_eq!(model._get_text("D1"), "b");
    assert_eq!(model._get_text("C2"), "c");
    assert_eq!(model._get_text("D2"), "#N/A");
    assert_eq!(model._get_text("D5"), "-");
}

#[test]
fn wide_row_under_the_cell_limit_is_unchanged() {
    let mut model = new_empty_model();
    // 1 x 16,385 cells: wider than the grid but far under 2^20. Excel builds it (probe
    // textsplit-wide A1:A2).
    model._set("A1", r#"=COLUMNS(TEXTSPLIT(REPT(",",16384),","))"#);
    model._set("A2", r#"=COLUMNS(TEXTSPLIT(REPT(",",16383),","))"#);
    model._set("A3", r#"=ROWS(TEXTSPLIT(REPT(";",32000),,";"))"#);

    model.evaluate();

    assert_eq!(model._get_text("A1"), "16385");
    assert_eq!(model._get_text("A2"), "16384");
    assert_eq!(model._get_text("A3"), "32001");
}

#[test]
fn many_delimiters_over_a_long_text_are_bounded_by_the_work_cap() {
    let mut model = new_empty_model();
    // 32,766 commas split into 32,767 pieces. Each delimiter's next match is cached, so 200 more
    // delimiters that never match cost one scan each, not one per piece (T-1795); 5,000 of them
    // pass the lifted-text work cap (text length x delimiters, text.md "Lifted TEXTSPLIT,
    // TEXTBEFORE, TEXTAFTER and REGEX* calls are capped") and are `#ERROR!` at once.
    model._set(
        "A1",
        r#"=COLUMNS(TEXTSPLIT(REPT(",",32766),VSTACK(",","q"&SEQUENCE(200))))"#,
    );
    model._set(
        "A2",
        r#"=COLUMNS(TEXTSPLIT(REPT(",",32766),VSTACK(",","q"&SEQUENCE(5000))))"#,
    );

    model.evaluate();

    assert_eq!(model._get_text("A1"), "32767");
    assert_eq!(model._get_text("A2"), "#ERROR!");
}
