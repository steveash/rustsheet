#![allow(clippy::unwrap_used)]
//! PROB's data-argument order and pairing (T-2468; statistics.md "PROB: an error data argument first").

use crate::test::util::new_empty_model;

#[test]
fn prob_checks_data_arguments_before_limits() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("A2", "2");
    model._set("A3", "3");
    model._set("B1", "0.5");
    model._set("B2", "0.5");
    model._set("D1", "1");
    model._set("D2", "=NA()");
    model._set("D3", "3");
    model._set("F1", "0.2");
    model._set("F2", "0.3");
    model._set("F3", "0.5");
    model._set("C1", "=PROB(1/0,{0.5;0.5},1)");
    model._set("C2", "=PROB({1;2},NA(),\"a\")");
    model._set("C5", "=PROB(A1:A3,B1:B2,1)");
    model._set("C6", "=PROB(D1:D3,F1:F3,\"a\")");
    model._set("C7", "=PROB(H1,B1,1)");
    model.evaluate();

    assert_eq!(model._get_text("C1"), *"#DIV/0!");
    assert_eq!(model._get_text("C2"), *"#N/A");
    assert_eq!(model._get_text("C5"), *"#N/A");
    assert_eq!(model._get_text("C6"), *"#VALUE!");
    assert_eq!(model._get_text("C7"), *"#VALUE!");
}

#[test]
fn prob_pairs_cells_by_position() {
    let mut model = new_empty_model();
    // x {0; blank; "b"; -0.5; blank; 1E15} against p {0.3; -1; 3.14159; 0.2; blank; 0.5}: only
    // the three positions with two numbers pair, so -1 and 3.14159 never reach the range check
    // (gen-m9/e6cf3b20a014e6a3 A229's PROB(Data!D3:D8,Data!C5:C10,..) is 0).
    for (cell, value) in [("A1", "0"), ("A3", "b"), ("A4", "-0.5"), ("A6", "1E+15")] {
        model._set(cell, value);
    }
    for (cell, value) in [
        ("B1", "0.3"),
        ("B2", "-1"),
        ("B3", "3.14159"),
        ("B4", "0.2"),
        ("B6", "0.5"),
    ] {
        model._set(cell, value);
    }
    model._set("G1", "TRUE");
    model._set("H1", "5");
    model._set("C1", "=PROB(A1:A6,B1:B6,0.3,0.3)");
    model._set("C2", "=PROB(A1:A6,B1:B6,-1,1)");
    // Counts are cells: one number and one blank against one number is unequal.
    model._set("C3", "=PROB(H1:H2,0.5,1)");
    // A single value on each side with no pair of numbers is #DIV/0!; a pair is checked as usual.
    model._set("C4", "=PROB(H1,G1,1)");
    // A bare typed boolean is inferred from a TRANSPOSE'd one (5cebac1f02bf30f2 A372); unprobed.
    model._set("C5", "=PROB(-6,1=1,0,3)");
    model._set("C6", "=PROB(3,0.1,-1,-1)");
    model._set("C7", "=PROB(H1,1,5)");
    model.evaluate();

    assert_eq!(model._get_text("C1"), *"0");
    assert_eq!(model._get_text("C2"), *"0.5");
    assert_eq!(model._get_text("C3"), *"#N/A");
    assert_eq!(model._get_text("C4"), *"#DIV/0!");
    assert_eq!(model._get_text("C5"), *"#DIV/0!");
    assert_eq!(model._get_text("C6"), *"#NUM!");
    assert_eq!(model._get_text("C7"), *"1");
}
