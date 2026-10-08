#![allow(clippy::unwrap_used, reason = "tests")]

//! The bare union: a comma outside every parenthesis and argument list (`=A1,B2`) is the union
//! operator, binding tighter than arithmetic and looser than the space (T-1073). Expected
//! values and formula texts are Excel's from probes/multiarea-bare and excel-graph scratch
//! probes, see docs/excel-semantics/references.md "Bare union".

use crate::test::util::new_empty_model;

#[test]
fn bare_unions_load_and_print_as_written() {
    let mut model = new_empty_model();
    for (cell, value) in [("A1", "1"), ("B2", "5"), ("C3", "7")] {
        model._set(cell, value);
    }
    let cases = [
        ("H1", "=A1,B2", "#VALUE!"),
        ("H2", "=A1:A2,B2,C3", "#VALUE!"),
        ("H3", "=-A1,B2", "#VALUE!"),
        ("H4", "=A1,B2+1", "#VALUE!"),
        ("H5", "=(A1,B2),C3", "#VALUE!"),
        ("H6", "=A1 B1,B2", "#NULL!"),
        ("H7", "=1+A1,B2", "#VALUE!"),
        ("H8", "=A1,B2+NA()", "#VALUE!"),
        ("H9", "=NA()+A1,B2", "#N/A"),
        ("H10", "=A1,B2*0+(A1 B1)", "#VALUE!"),
        ("H11", "=A1,B2 C3", "#NULL!"),
    ];
    for (cell, formula, _) in cases {
        model._set(cell, formula);
    }
    model.evaluate();
    for (cell, formula, expected) in cases {
        assert_eq!(model._get_text(cell), expected, "{cell}: {formula}");
        // The engine drops the parentheses of an intersection operand (`+(A1 B1)` prints
        // `+A1 B1`, the same tree), as outside a bare union.
        if cell != "H10" {
            assert_eq!(model._get_formula(cell), formula, "{cell}");
        }
    }
}

#[test]
fn bare_union_operands_must_be_references() {
    let mut model = new_empty_model();
    let cases = ["=1,A1", "=A1,1", "=A1,SUM(B2)", "=SUM(A1),B2", "=A1,B2,"];
    for (row, formula) in cases.iter().enumerate() {
        model._set(&format!("H{}", row + 1), formula);
    }
    model.evaluate();
    for (row, formula) in cases.iter().enumerate() {
        assert_eq!(
            model._get_text(&format!("H{}", row + 1)),
            "#ERROR!",
            "{formula}"
        );
    }
}

#[test]
fn commas_inside_calls_and_parentheses_are_unchanged() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("B2", "5");
    model._set("H1", "=SUM(A1,B2)");
    model._set("H2", "=SUM((A1,B2))");
    model._set("H3", "=IF(TRUE,A1,B2)");
    model._set("H4", "=SUM(A1,B2)+1");
    model.evaluate();
    assert_eq!(model._get_text("H1"), "6");
    assert_eq!(model._get_text("H2"), "6");
    assert_eq!(model._get_text("H3"), "1");
    assert_eq!(model._get_text("H4"), "7");
    assert_eq!(model._get_formula("H2"), "=SUM((A1,B2))");
}
