#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn test_simple_circ() {
    let mut model = new_empty_model();
    model._set("A1", "=A1+1");
    model.evaluate();
    assert!(model._is_circular("A1"));
}

#[test]
fn test_simple_circ_propagate() {
    let mut model = new_empty_model();
    model._set("A1", "=B6");
    model._set("A2", "=A1+1");
    model._set("A3", "=A2+1");
    model._set("A4", "=A3+5");
    model._set("B6", "=A4*7");
    model.evaluate();
    assert!(model._is_circular("A1"));
    assert!(model._is_circular("A2"));
    assert!(model._is_circular("A3"));
    assert!(model._is_circular("A4"));
    assert!(model._is_circular("B6"));
}

#[test]
fn every_reader_of_a_loop_is_zero_however_indirect() {
    // probes/circular-iteration-off and probes/t0445-circular-readers (excel-graph): with
    // iteration off, a loop and every cell that reads it, at any depth and whatever it
    // computes, are the number 0.
    let mut model = new_empty_model();
    model._set("A1", "=A1+1");
    model._set("B1", "=A1+10");
    model._set("C1", "=B1+1");
    model._set("D1", "=C1*2+3");
    model._set("E1", "=IFERROR(A1,5)");
    model._set("F1", "=ISERROR(A1)");
    model._set("G1", r#"=IF(ISNUMBER(B1),"n","x")"#);
    model._set("H1", "=SUM(A1:A3)+1");
    model._set("A2", "=1+2");
    model._set("D2", r#"=LEN("abc")+IF(FALSE,A1,1)"#);
    model.evaluate();
    for cell in ["A1", "B1", "C1", "D1", "E1", "F1", "G1", "H1"] {
        assert!(model._is_circular(cell), "{cell}");
    }
    assert_eq!(model._get_text("A2"), "3");
    // A reference IF does not evaluate is no read.
    assert_eq!(model._get_text("D2"), "4");
    // The verdict does not depend on which cell the evaluation starts from.
    model._set("A3", "=C1");
    model.evaluate();
    assert!(model._is_circular("A3"));
    assert!(model._is_circular("H1"));
}
