#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn arguments() {
    let mut model = new_empty_model();
    model._set("A1", "=CELL(\"address\",A1)");
    model._set("A2", "=CELL()");

    model._set("A3", "=INFO(\"recalc\")");
    model._set("A4", "=INFO()");

    model._set("A5", "=N(TRUE)");
    model._set("A6", "=N()");
    model._set("A7", "=N(1, 2)");

    model._set("A8", "=SHEETS()");
    model._set("A9", "=SHEETS(1)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"$A$1");
    assert_eq!(model._get_text("A2"), *"#ERROR!");

    assert_eq!(model._get_text("A3"), *"#VALUE!");
    assert_eq!(model._get_text("A4"), *"#ERROR!");

    assert_eq!(model._get_text("A5"), *"1");
    assert_eq!(model._get_text("A6"), *"#ERROR!");
    assert_eq!(model._get_text("A7"), *"#ERROR!");

    assert_eq!(model._get_text("A8"), *"1");
    assert_eq!(model._get_text("A9"), *"#N/IMPL!");
}

/// INFO is `#VALUE!` for every `type_text` (probes/info-sheets-info-types Alpha!D25:D49, excel-graph).
#[test]
fn info_is_value_error_for_every_type_text() {
    let mut model = new_empty_model();
    for (i, text) in [
        "recalc", "SYSTEM", "release", "timezone", "numfile", "memavail", "", "x",
    ]
    .iter()
    .enumerate()
    {
        model._set(&format!("A{}", i + 1), &format!("=INFO(\"{text}\")"));
    }
    model.evaluate();
    for i in 1..=8 {
        assert_eq!(model._get_text(&format!("A{i}")), *"#VALUE!");
    }
}

/// `CELL("filename")` is `#VALUE!` in every form, named workbook or not (probes/
/// info-cell-address-cross-sheet Calc!B1:B4, excel-graph, T-2052).
#[test]
fn cell_filename_is_value_error() {
    let mut model = new_empty_model();
    model.add_sheet("Data").unwrap();
    model._set("A1", "=CELL(\"filename\")");
    model._set("A2", "=CELL(\"filename\",B2)");
    model._set("A3", "=CELL(\"filename\",Data!B2)");
    model.evaluate();
    for cell in ["A1", "A2", "A3"] {
        assert_eq!(model._get_text(cell), *"#VALUE!");
    }

    model.workbook.name = String::new();
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"#VALUE!");
}

/// `CELL("address")` of a reference on another sheet is prefixed `[<file>]<sheet>!`, the sheet
/// name never quoted, a range giving its top-left cell (probes/info-cell-address-cross-sheet,
/// excel-graph, T-2042).
#[test]
fn cell_address_on_another_sheet_names_the_file() {
    let mut model = new_empty_model();
    model.workbook.name = "book".to_string();
    model.add_sheet("My Sheet").unwrap();
    model.add_sheet("Bob's").unwrap();
    model._set("A1", "=CELL(\"address\",'My Sheet'!B2)");
    model._set("A2", "=CELL(\"address\",'Bob''s'!B2:C3)");
    model._set("A3", "=CELL(\"address\",Sheet1!B2)");
    model._set("A4", "=CELL(\"address\",('My Sheet'!B2,'My Sheet'!C3))");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"[book.xlsx]My Sheet!$B$2");
    assert_eq!(model._get_text("A2"), *"[book.xlsx]Bob's!$B$2");
    assert_eq!(model._get_text("A3"), *"$B$2");
    // A union on another sheet is unprobed.
    assert_eq!(model._get_text("A4"), *"#N/IMPL!");

    // Excel's form for an unsaved workbook is unprobed.
    model.workbook.name = String::new();
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"#N/IMPL!");
    assert_eq!(model._get_text("A3"), *"$B$2");
}

/// N lifts over an array, an error element kept (probes/n-array-lift; T-1616).
#[test]
fn n_lifts_over_an_array() {
    let mut model = new_empty_model();
    model._set("A1", "=SUM(N({1,\"a\",TRUE}))");
    model._set("A2", "=N({1,#N/A})");
    model._set("A3", "=SUMPRODUCT(INDEX(D1:D6,N(IF(1,{2;3}))))");
    for (row, value) in (1..=6).enumerate() {
        model._set(&format!("D{}", row + 1), &value.to_string());
    }
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"2");
    assert_eq!(model._get_text("A2"), *"1");
    assert_eq!(model._get_text("B2"), *"#N/A");
    assert_eq!(model._get_text("A3"), *"5");
}

/// T lifts over an array value, text and errors kept and anything else "" (probes/t-array-lift,
/// legacy-array-result S!G1:G2; T-1729); a multi-cell range with a number top-left cell stays one
/// "" (probes/text-argument-modes F9).
#[test]
fn t_lifts_over_an_array() {
    let mut model = new_empty_model();
    model._set("A1", "=T(IF(1,{\"a\";\"b\"}))");
    model._set("C1", "=T(IF(1,{\"a\",1,#N/A,TRUE}))");
    model._set("E3", "=T(G1:G2)");
    model._set("G1", "5");
    model._set("G2", "y");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"a");
    assert_eq!(model._get_text("A2"), *"b");
    assert_eq!(model._get_text("C1"), *"a");
    assert_eq!(model._get_text("D1"), *"");
    assert_eq!(model._get_text("E1"), *"#N/A");
    assert_eq!(model._get_text("F1"), *"");
    assert_eq!(model._get_text("E3"), *"");
    assert_eq!(model._get_text("E4"), *"");
}

/// T reads a multi-cell range as its top-left cell, legacy or dynamic, with no implicit
/// intersection, and a legacy formula puts no `@` on the range, an OFFSET or a literal-condition
/// IF of it (probes/t-n-range-top-left F2, F10, F20, P20, Q20, R20; T-0985).
#[test]
fn t_reads_the_top_left_cell_of_a_range() {
    let mut model = new_empty_model();
    model._set("A1", "x");
    model._set("A2", "5");
    model._set("B1", "5");
    model._set("B2", "ab");
    model._set("C2", "=T(A1:A2)");
    model._set("D2", "=T(B1:B2)");
    model.evaluate();
    assert_eq!(model._get_text("C2"), *"x");
    assert_eq!(model._get_text("D2"), *"");
    for (formula, legacy) in [
        ("=T(A1:A3)", "=T(A1:A3)"),
        ("=T(OFFSET(A1:A3,0,0))", "=T(OFFSET(A1:A3,0,0))"),
        ("=T(IF(1,A1:A3))", "=T(IF(1,A1:A3))"),
        ("=T(A1:A3&\"\")", "=T(@A1:A3&\"\")"),
    ] {
        let text = model.legacy_formula_text(0, 10, 6, formula).unwrap();
        assert_eq!(text, legacy, "{formula}");
    }
}

/// A multi-cell range `info_type` lifts like an array one (gen-m5/c0651f0e0fb544d0 Calc!A293).
#[test]
fn cell_lifts_over_a_range_info_type() {
    let mut model = new_empty_model();
    model._set("D1", "row");
    model._set("D2", "col");
    model._set("F1", "=CELL(D1:D2,B3)");
    model._set("H2", "x");
    model._set("H1", "=CELL(D1:D2,B3)");
    model.evaluate();

    assert_eq!(model._get_text("F1"), *"3");
    assert_eq!(model._get_text("F2"), *"2");
    assert_eq!(model._get_text("H1"), *"#SPILL!");
}

/// `VSTACK(CELL("contents",ref),..)` is `#VALUE!` in Excel; CELL alone and `VSTACK(0,..)` are
/// fine (probes/t2414-gen-m7-unexplained, T-2457).
#[test]
fn cell_contents_as_vstack_argument() {
    let mut model = new_empty_model();
    model._set("A1", "=VSTACK(CELL(\"contents\",C11),1)");
    model._set("B2", "=VSTACK(0,1)");
    model._set("C3", "=CELL(\"contents\",C11)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#VALUE!");
    assert_eq!(model._get_text("B2"), *"0");
    assert_eq!(model._get_text("C3"), *"0");
}

/// COUNT and COUNTA count `CELL("contents", blank)` as the empty value and MODE.SNGL gives
/// `#VALUE!` (probes/t2622-contents-xmatch-take-lambda A1-A3, A9, A11, T-2622).
#[test]
fn cell_contents_of_a_blank_is_the_empty_value() {
    let mut model = new_empty_model();
    model._set("A1", "=COUNT(CELL(\"contents\",C5))");
    model._set("A2", "=COUNTA(CELL(\"contents\",C5))");
    model._set("A3", "=MODE.SNGL(CELL(\"contents\",C5))");
    model._set("A4", "=CELL(\"contents\",C5)");
    model._set("A5", "=COUNT(C5)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"1");
    assert_eq!(model._get_text("A2"), *"1");
    assert_eq!(model._get_text("A3"), *"#VALUE!");
    assert_eq!(model._get_text("A4"), *"0");
    assert_eq!(model._get_text("A5"), *"0");
}

/// TAKE of a one-cell reference, bare or bound by LET, is a reference (TYPE 1); a cut of an
/// array value is an array (64) (probes/t2622-contents-xmatch-take-lambda A29, A30, A59, A60).
#[test]
fn take_of_a_one_cell_reference_is_a_reference() {
    let mut model = new_empty_model();
    model._set("B2", "2.5");
    model._set("A1", "=TYPE(TAKE(B2,1,1))");
    model._set("A2", "=TYPE(TAKE(LET(v,B2,v),-1,-1))");
    model._set("A3", "=TYPE(TAKE(5,1,1))");
    model._set("A4", "=TYPE(TAKE(LET(v,5,v),1,1))");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"1");
    assert_eq!(model._get_text("A2"), *"1");
    assert_eq!(model._get_text("A3"), *"64");
    assert_eq!(model._get_text("A4"), *"64");
}

/// An uncalled LAMBDA joined as text or compared is `#VALUE!`, and a number `match_mode` of 3 is
/// XMATCH's regex mode on the number's text (probes/t2622-contents-xmatch-take-lambda A20,
/// A49, A50, A57).
#[test]
fn uncalled_lambda_text_and_xmatch_regex_number() {
    let mut model = new_empty_model();
    model._set("A1", "=LAMBDA(x,x)&\"\"");
    model._set("A2", "=LET(t,\"a\",v,LAMBDA(x,x),t&\"|\"&v)");
    model._set("A3", "=LAMBDA(x,x)=1");
    model._set("A4", "=XMATCH(1,{3;1;4;1;5},3)");
    model._set("A5", "=XMATCH(2,{3;1;4;1;5},3)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#VALUE!");
    assert_eq!(model._get_text("A2"), *"#VALUE!");
    assert_eq!(model._get_text("A3"), *"#VALUE!");
    assert_eq!(model._get_text("A4"), *"2");
    assert_eq!(model._get_text("A5"), *"#N/A");
}

/// COUNT and COUNTA read a `CELL("contents", ..)` argument once, so nesting them stays linear
/// (a second read doubled the work per level; T-2622 security review).
#[test]
fn nested_cell_contents_in_count_is_linear() {
    let mut formula = String::from("0");
    for _ in 0..18 {
        formula = format!("COUNT(CELL(\"contents\",INDEX(A2:A10,1+0*{formula})))");
    }
    let mut model = new_empty_model();
    model._set("A1", &format!("={formula}"));
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"1");
}

/// An error element of an array data argument is the first in row-major order for the PERCENTILE
/// family, but an error among the other arguments comes before it (probes/t2622-contents-xmatch-
/// take-lambda A89-A96, A103; main's `numeric_flat_with_element_errors` and `lift_over_sorted`).
#[test]
fn percentile_family_array_error_element_order() {
    let mut model = new_empty_model();
    model._set("A1", "=PERCENTRANK({#N/A,#DIV/0!,1},1)");
    model._set("A2", "=PERCENTRANK({#DIV/0!,#N/A,1},1)");
    model._set("A3", "=PERCENTILE({1,#N/A,3},1/0)");
    model._set("A4", "=QUARTILE({1,#N/A,3},1)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#N/A");
    assert_eq!(model._get_text("A2"), *"#DIV/0!");
    assert_eq!(model._get_text("A3"), *"#DIV/0!");
    assert_eq!(model._get_text("A4"), *"#N/A");
}
