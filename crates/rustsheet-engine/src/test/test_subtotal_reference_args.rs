#![allow(clippy::unwrap_used, reason = "tests")]

//! SUBTOTAL and AREAS take references only (T-1163). Excel for the web refuses a workbook
//! holding a non-reference argument written in place, so the parser refuses it (`#ERROR!`, as
//! for `(1)(2)`); an argument that is written as a reference operand but evaluates to a value is
//! `#VALUE!`. Expected values are excel-graph scratch probes (2026-10-08), see
//! docs/excel-semantics/references.md "SUBTOTAL and AREAS take references only".

use crate::model::Model;
use crate::test::util::new_empty_model;

fn check(cases: &[(&str, &str, &str)]) -> Model {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("A2", "2");
    model._set("C1", "=1/0");
    for (cell, formula, _) in cases {
        model._set(cell, formula);
    }
    model.evaluate();
    for (cell, formula, expected) in cases {
        assert_eq!(model._get_text(cell), *expected, "{cell}: {formula}");
    }
    model
}

#[test]
fn non_reference_arguments_are_refused() {
    check(&[
        ("H1", "=SUBTOTAL(9,{1,2})", "#ERROR!"),
        ("H2", "=SUBTOTAL(2,{1,2})", "#ERROR!"),
        ("H3", "=SUBTOTAL(109,{1,2})", "#ERROR!"),
        ("H4", "=SUBTOTAL(9,A1:A2*1)", "#ERROR!"),
        ("H5", "=SUBTOTAL(9,SUM(A1:A2))", "#ERROR!"),
        ("H6", "=SUBTOTAL(9,ABS(A1:A2))", "#ERROR!"),
        ("H7", "=SUBTOTAL(9,1)", "#ERROR!"),
        ("H8", "=SUBTOTAL(9,\"A1\")", "#ERROR!"),
        ("H9", "=SUBTOTAL(9,TRUE)", "#ERROR!"),
        ("H10", "=SUBTOTAL(9,A1:A2,{1,2})", "#ERROR!"),
        ("H11", "=SUBTOTAL(9,FILTER(A1:A2,A1:A2>0))", "#ERROR!"),
        ("H12", "=SUBTOTAL(9,-A1)", "#ERROR!"),
        ("H13", "=SUBTOTAL(9,#N/A)", "#ERROR!"),
        ("H14", "=SUBTOTAL(9,VSTACK(A1:A2))", "#ERROR!"),
        ("H15", "=SUBTOTAL(9,A1,)", "#ERROR!"),
        ("H16", "=SUBTOTAL(2,A1,)", "#ERROR!"),
        ("H17", "=SUBTOTAL(9,SUBTOTAL(9,A1:A2))", "#ERROR!"),
        ("H18", "=SUBTOTAL(9,A1:A2,SUBTOTAL(9,A1:A2))", "#ERROR!"),
        ("H19", "=AREAS(1)", "#ERROR!"),
        ("H20", "=AREAS(\"A1\")", "#ERROR!"),
    ]);
}

#[test]
fn reference_operands_load() {
    check(&[
        ("H1", "=SUBTOTAL(9,INDEX(A1:A2,0))", "3"),
        ("H2", "=SUBTOTAL(9,LET(x,A1:A2,x))", "3"),
        ("H3", "=SUBTOTAL(9,(A1))", "1"),
        ("H4", "=SUBTOTAL(9,#REF!)", "#REF!"),
        ("H5", "=SUBTOTAL(9,IF(TRUE,A1:A2))", "3"),
        ("H6", "=SUBTOTAL(9,A1:A2 A1)", "1"),
        ("J1", "=SUBTOTAL(9,@A1:A2)", "1"),
        ("H8", "=SUBTOTAL(9,IF(TRUE,A1:A2,5))", "3"),
    ]);
}

#[test]
fn values_from_reference_operands_are_value_errors() {
    check(&[
        ("H1", "=SUBTOTAL(9,LAMBDA(x,x))", "#VALUE!"),
        ("H2", "=SUBTOTAL(2,LAMBDA(x,x))", "#VALUE!"),
        ("H3", "=SUBTOTAL(3,LAMBDA(x,x))", "#VALUE!"),
        ("H4", "=SUBTOTAL(109,LAMBDA(x,x))", "#VALUE!"),
        ("H5", "=SUBTOTAL(9,LET(f,LAMBDA(x,x),f))", "#VALUE!"),
        ("H6", "=SUBTOTAL(9,IF(TRUE,5))", "#VALUE!"),
        ("H7", "=SUBTOTAL(2,IF(TRUE,5))", "#VALUE!"),
        ("H8", "=SUBTOTAL(2,IF(TRUE,\"a\"))", "#VALUE!"),
        ("H9", "=SUBTOTAL(3,IF(TRUE,\"a\"))", "#VALUE!"),
        ("H10", "=SUBTOTAL(9,IF(TRUE,\"\"))", "#VALUE!"),
        ("H11", "=SUBTOTAL(9,IF(TRUE,1/0))", "#DIV/0!"),
        ("H12", "=SUBTOTAL(2,IF(TRUE,1/0))", "#DIV/0!"),
        ("H13", "=SUBTOTAL(3,IF(TRUE,1/0))", "#DIV/0!"),
        // The value is `#VALUE!` even after an error cell (C1 = #DIV/0!).
        ("H14", "=SUBTOTAL(9,C1,IF(TRUE,5))", "#VALUE!"),
        ("H15", "=SUBTOTAL(9,IF(TRUE,5),C1)", "#VALUE!"),
        ("H16", "=AREAS(IF(TRUE,5))", "#VALUE!"),
        ("H17", "=AREAS(LAMBDA(x,x))", "#VALUE!"),
    ]);
}

#[test]
fn arrays_from_reference_operands_spill_value_errors() {
    let model = check(&[
        ("H1", "=SUBTOTAL(9,IF(TRUE,{1,2}))", "#VALUE!"),
        ("H3", "=SUBTOTAL(9,LET(x,{1,2},x))", "#VALUE!"),
        ("H5", "=SUBTOTAL(9,CHOOSE(1,{1,2}))", "#VALUE!"),
        ("H7", "=SUBTOTAL(3,IF(TRUE,{1,2}))", "#VALUE!"),
        ("H9", "=SUBTOTAL(9,IF(TRUE,{1,#N/A}))", "#VALUE!"),
        ("H11", "=SUBTOTAL(9,A1:A2,IF(TRUE,{1,2}))", "#VALUE!"),
        ("H13", "=SUBTOTAL(9,IF(TRUE,{1;2}))", "#VALUE!"),
    ]);
    // The `I` cells (and H14) are the second element of each spill. Each element is `#VALUE!` unless it is an error, which stays itself.
    for cell in ["I1", "I3", "I5", "I7", "I11", "H14"] {
        assert_eq!(model._get_text(cell), "#VALUE!", "{cell}");
    }
    assert_eq!(model._get_text("I9"), "#N/A");
}

#[test]
fn take_of_let_chain_and_malformed_let_is_linear_and_total() {
    let mut model = new_empty_model();
    // A flat chain of 60 variables, each an IF over the previous one twice: exponential if the
    // answer for a variable were recomputed at each use (T-1365).
    let mut formula = String::from("=SUBTOTAL(9,TAKE(LET(v0,A1:A4");
    for i in 1..60 {
        formula.push_str(&format!(",v{i},IF(TRUE,v{},v{})", i - 1, i - 1));
    }
    formula.push_str(",v59),2))");
    model._set("B1", &formula);
    model._set("B2", "=SUBTOTAL(9,TAKE(LET(),1))");
    model._set("B3", "=SUBTOTAL(9,TAKE(LET(x,A1:A4),1))");
    model.evaluate();
    // Only termination without a panic is asserted for the chain; the cells are errors here.
    assert_eq!(model._get_text("B2"), model._get_text("B3"));
    assert!(model._get_text("B1").starts_with('#') || model._get_text("B1") == "0");
}

// A CHOOSE with an array index lifts to values even over range arms, so TAKE picks values and
// SUBTOTAL spills a 2x2 of `#VALUE!` (probes/lift-arrays-choose-arms A69:B70, T-1882); static
// analysis took it for cells, a scalar, and a debug build hit the scalar-context assertion.
#[test]
fn subtotal_of_take_of_choose_with_array_index_spills() {
    let mut model = new_empty_model();
    for (cell, value) in [("H1", "3"), ("H2", "4"), ("I1", "6"), ("I2", "8")] {
        model._set(cell, value);
    }
    model._set("A1", "=SUBTOTAL(9,TAKE(CHOOSE({1,2},H1:H2,I1:I2),2))");
    model._set("D1", "=SUBTOTAL(9,TAKE(CHOOSE(2,H1:H2,I1:I2),2))");
    model.evaluate();
    for cell in ["A1", "B1", "A2", "B2"] {
        assert_eq!(model._get_text(cell), "#VALUE!", "{cell}");
    }
    assert_eq!(model._get_text("D1"), "14");
    assert_eq!(model._get_text("D2"), "");
}

// The CHOOSE index is analysed once per level, so deep nesting stays linear (T-1882; T-1874's
// first fix analysed it twice a level, 2^20 analyses here). 20 levels of three calls is near the
// parser's nesting cap.
#[test]
fn subtotal_of_take_of_nested_choose_is_linear() {
    let mut formula = "SUBTOTAL(9,TAKE(CHOOSE(1,H1:H2),1))".to_string();
    for _ in 0..20 {
        formula = format!("SUBTOTAL(9,TAKE(CHOOSE({formula},H1:H2,I1:I2),1))");
    }
    let mut model = new_empty_model();
    model._set("H1", "1");
    model._set("A5", &format!("={formula}"));
    model.evaluate();
    assert_eq!(model._get_text("A5"), "1");
}

// SWITCH and IFS with an array selector lift to values even over range arms, as CHOOSE does, so
// TAKE picks values and SUBTOTAL spills a 2x2 of `#VALUE!` (probes/subtotal-take-array-selector
// A30:B31, A40:B41, T-1914); static analysis took them for cells, a scalar, and a debug build hit
// the scalar-context assertion (nightly fuzz). A scalar selector still picks cells.
#[test]
fn subtotal_of_take_of_switch_and_ifs_with_array_selector_spills() {
    let mut model = new_empty_model();
    for (cell, value) in [("H1", "3"), ("H2", "4"), ("I1", "6"), ("I2", "8")] {
        model._set(cell, value);
    }
    model._set("A1", "=SUBTOTAL(9,TAKE(SWITCH({1,2},1,H1:H2,2,I1:I2),2))");
    model._set(
        "A4",
        "=SUBTOTAL(9,TAKE(IFS({TRUE,FALSE},H1:H2,TRUE,I1:I2),2))",
    );
    model._set("D1", "=SUBTOTAL(9,TAKE(SWITCH(2,1,H1:H2,2,I1:I2),2))");
    model._set("D4", "=SUBTOTAL(9,TAKE(IFS(FALSE,H1:H2,TRUE,I1:I2),2))");
    model.evaluate();
    for top in [1, 4] {
        for cell in [
            format!("A{top}"),
            format!("B{top}"),
            format!("A{}", top + 1),
            format!("B{}", top + 1),
        ] {
            assert_eq!(model._get_text(&cell), "#VALUE!", "{cell}");
        }
    }
    assert_eq!(model._get_text("D1"), "14");
    assert_eq!(model._get_text("D2"), "");
    assert_eq!(model._get_text("D4"), "14");
    assert_eq!(model._get_text("D5"), "");
}

// Each SWITCH selector is analysed once per level, so deep nesting stays linear (T-1914).
#[test]
fn subtotal_of_take_of_nested_switch_is_linear() {
    let mut formula = "SUBTOTAL(9,TAKE(SWITCH(1,1,H1:H2),1))".to_string();
    for _ in 0..20 {
        formula = format!("SUBTOTAL(9,TAKE(SWITCH({formula},1,H1:H2,I1:I2),1))");
    }
    let mut model = new_empty_model();
    model._set("H1", "1");
    model._set("A5", &format!("={formula}"));
    model.evaluate();
    assert_eq!(model._get_text("A5"), "1");
}

// An IF, CHOOSE or LET that passes on a lifting SWITCH, IFS or CHOOSE lifts too, as does one whose
// selector is a LET variable holding an array, and each spills like the bare call; static analysis took these for cells too and hit the scalar-context assertion
// in a debug build (T-1914 review).
#[test]
fn subtotal_of_take_of_wrapped_array_selector_spills_like_the_bare_call() {
    let bare = "SWITCH({1,2},1,H1:H2,2,I1:I2)";
    let wrapped = [
        format!("IF(TRUE,{bare})"),
        format!("CHOOSE(1,{bare})"),
        format!("LET(x,{bare},x)"),
        format!("LET(x,{bare},y,x,IF(TRUE,y))"),
        "IF(TRUE,CHOOSE({1,2},H1:H2,I1:I2))".to_string(),
        "CHOOSE(1,IFS({TRUE,FALSE},H1:H2,TRUE,I1:I2))".to_string(),
        "LET(s,{1,2},SWITCH(s,1,H1:H2,2,I1:I2))".to_string(),
        "LET(s,{TRUE,FALSE},IFS(s,H1:H2,TRUE,I1:I2))".to_string(),
        "LET(s,{1,2},CHOOSE(s,H1:H2,I1:I2))".to_string(),
    ];
    let mut model = new_empty_model();
    for (cell, value) in [("H1", "3"), ("H2", "4"), ("I1", "6"), ("I2", "8")] {
        model._set(cell, value);
    }
    model._set("A1", &format!("=SUBTOTAL(9,TAKE({bare},2))"));
    for (index, arg) in wrapped.iter().enumerate() {
        model._set(
            &format!("A{}", 4 + 3 * index),
            &format!("=SUBTOTAL(9,TAKE({arg},2))"),
        );
    }
    model.evaluate();
    for index in 0..wrapped.len() {
        let top = 4 + 3 * index;
        for (bare_cell, cell) in [
            ("A1".to_string(), format!("A{top}")),
            ("B1".to_string(), format!("B{top}")),
            ("A2".to_string(), format!("A{}", top + 1)),
            ("B2".to_string(), format!("B{}", top + 1)),
        ] {
            assert_eq!(
                model._get_text(&cell),
                model._get_text(&bare_cell),
                "{cell}"
            );
        }
    }
    assert_eq!(model._get_text("A1"), "#VALUE!");
}

// Each IFS condition is analysed once per level, so deep nesting stays linear (T-1914).
#[test]
fn subtotal_of_take_of_nested_ifs_is_linear() {
    let mut formula = "SUBTOTAL(9,TAKE(IFS(TRUE,H1:H2),1))".to_string();
    for _ in 0..20 {
        formula = format!("SUBTOTAL(9,TAKE(IFS({formula}=1,H1:H2,TRUE,I1:I2),1))");
    }
    let mut model = new_empty_model();
    model._set("H1", "1");
    model._set("A5", &format!("={formula}"));
    model.evaluate();
    assert_eq!(model._get_text("A5"), "1");
}
