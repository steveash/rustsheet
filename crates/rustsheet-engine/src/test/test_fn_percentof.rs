#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

// PERCENTOF ranks errors across both arguments like SUM, and an error inside a reference or an
// array is #NUM! (numbers.md "PERCENTOF: an error inside a reference or array is #NUM!";
// probes/t3653-percentof-argument-kinds, excel-graph, T-3653).
#[test]
fn errors_inside_references_and_arrays_are_num() {
    let mut model = new_empty_model();
    model._set("A1", "=NA()");
    model._set("A2", "=1/0");
    model._set("A3", "1");
    model._set("A4", "3");
    model._set("C1", "=PERCENTOF(A1,A2)");
    model._set("C2", "=PERCENTOF(A3,A1:A4)");
    model._set("C3", "=PERCENTOF({1,#N/A},4)");
    model._set("C4", "=PERCENTOF({1,2}+NA(),1)");
    model._set("C5", "=PERCENTOF(IF(TRUE,A1),1)");
    model._set("C6", "=PERCENTOF(A1,0)");
    model.evaluate();
    for cell in ["C1", "C2", "C3", "C4", "C5", "C6"] {
        assert_eq!(model._get_text(cell), *"#NUM!", "{cell}");
    }
}

#[test]
fn direct_errors_win_in_argument_order() {
    let mut model = new_empty_model();
    model._set("A1", "=NA()");
    model._set("C1", "=PERCENTOF(A1,1/0)");
    model._set("C2", "=PERCENTOF(A1,\"x\")");
    model._set("C3", "=PERCENTOF(NA(),1/0)");
    model._set("C4", "=PERCENTOF(1/0,NA())");
    model._set("C5", "=PERCENTOF(A1+0,1)");
    model.evaluate();
    assert_eq!(model._get_text("C1"), *"#DIV/0!");
    assert_eq!(model._get_text("C2"), *"#VALUE!");
    assert_eq!(model._get_text("C3"), *"#N/A");
    assert_eq!(model._get_text("C4"), *"#DIV/0!");
    assert_eq!(model._get_text("C5"), *"#N/A");
}
