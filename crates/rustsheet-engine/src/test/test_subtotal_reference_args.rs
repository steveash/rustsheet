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
    // Each element is `#VALUE!` unless it is an error, which stays itself.
    for cell in ["I1", "I3", "I5", "I7", "I11", "H14"] {
        assert_eq!(model._get_text(cell), "#VALUE!", "{cell}");
    }
    assert_eq!(model._get_text("I9"), "#N/A");
}
