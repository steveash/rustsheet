#![allow(clippy::unwrap_used)]

//! BETA.INV, F.INV, T.INV and CONFIDENCE.T (and BETAINV, F.INV.RT, FINV, T.INV.2T, TINV) lift
//! every argument over an array and a multi-cell range, each element the scalar call, now that
//! their quantile search has a per-call cap of its own (T-1848, SPEC-0011; probes/stat-argument-modes,
//! probes/stat-quantile-lift, docs/excel-semantics/statistics.md "Quantile searches: per-call cap
//! and lifting").

use crate::test::util::new_empty_model;

/// One call per entered function, the lifted argument a two-row array in each position.
const ARRAY_CALLS: &[&str] = &[
    "BETA.INV({0.4;0.5},2,3)",
    "BETA.INV(0.4,{2;3},3)",
    "BETA.INV(0.4,2,{3;4})",
    "BETA.INV(0.4,2,3,{0;0.1})",
    "BETA.INV(0.4,2,3,0,{1;2})",
    "BETAINV(0.4,2,3,0,{1;2})",
    "F.INV({0.4;0.5},5,5)",
    "F.INV(0.4,5,{5;10})",
    "F.INV.RT(0.4,{5;10},5)",
    "FINV(0.4,5,{5;10})",
    "T.INV({0.4;0.5},5)",
    "T.INV(0.4,{5;10})",
    "T.INV.2T(0.4,{5;10})",
    "TINV({0.4;0.5},5)",
    "CONFIDENCE.T(0.4,2,{15;20})",
    "CONFIDENCE.T({0.4;0.5},2,15)",
];

#[test]
fn every_entered_function_lifts_over_an_array() {
    let mut model = new_empty_model();
    for (row, formula) in ARRAY_CALLS.iter().enumerate() {
        model._set(&format!("A{}", row + 1), &format!("=ROWS({formula})"));
    }
    model.evaluate();
    for (row, formula) in ARRAY_CALLS.iter().enumerate() {
        assert_eq!(
            model._get_text(&format!("A{}", row + 1)),
            *"2",
            "ROWS({formula})"
        );
    }
}

#[test]
fn later_arguments_lift_over_a_range_on_their_own_rows() {
    // probes/stat-quantile-lift H1-J25, K1-K8: each element is the scalar call.
    let mut model = new_empty_model();
    for (cell, value) in [
        ("G1", "3"),
        ("G2", "4"),
        ("G3", "1"),
        ("G4", "2"),
        ("G5", "5"),
    ] {
        model._set(cell, value);
    }
    model._set("G6", "10");
    model._set("G7", "15");
    model._set("G8", "20");
    model._set("A1", "=ROWS(BETA.INV(0.4,2,G1:G2))");
    model._set("A2", "=ROUND(INDEX(BETA.INV(0.4,2,G1:G2),2),12)");
    model._set("A3", "=ROUND(INDEX(BETA.INV(0.4,2,3,0,G3:G4),2),12)");
    model._set("A4", "=ROUND(INDEX(F.INV(0.4,5,G5:G6),2),12)");
    model._set("A5", "=ROUND(INDEX(FINV(0.4,5,G5:G6),2),12)");
    model._set("A6", "=ROUND(INDEX(T.INV(0.4,G5:G6),2),12)");
    model._set("A7", "=ROUND(INDEX(TINV(0.4,G5:G6),2),12)");
    model._set("A8", "=ROUND(INDEX(CONFIDENCE.T(0.4,2,G7:G8),2),12)");
    // Off the range's rows too: a lift, not an intersection.
    model._set("A20", "=ROWS(T.INV.2T(0.4,G5:G6))");
    // A one-cell range is its value (L11).
    model._set("A21", "=ROUND(T.INV(0.4,G5:G5),12)");
    model.evaluate();
    let expected = [
        ("A1", "2"),
        ("A2", "0.265568696"),
        ("A3", "0.658333007"),
        ("A4", "0.757347343"),
        ("A5", "1.141189253"),
        ("A6", "-0.260184829"),
        ("A7", "0.879057829"),
        ("A8", "0.385028791"),
        ("A20", "2"),
        ("A21", "-0.267180866"),
    ];
    for (cell, value) in expected {
        assert_eq!(model._get_text(cell), *value, "{cell}");
    }
}

#[test]
fn each_element_is_the_scalar_call_with_its_own_error() {
    // probes/stat-quantile-lift L4-L10, L12-L18, L20.
    let mut model = new_empty_model();
    model._set("A1", "=ROUND(INDEX(T.INV({0.4;\"0.4\"},5),2),12)");
    model._set("A2", "=INDEX(T.INV({0.4;\"x\"},5),2)");
    model._set("A3", "=INDEX(CONFIDENCE.T(0.4,2,{15;1}),2)");
    model._set("A4", "=INDEX(F.INV({0.4;1.5},5,5),2)");
    model._set("A5", "=INDEX(T.INV.2T({0.4;#N/A},5),2)");
    model._set("A6", "=ROWS(T.INV({0.1;0.2;0.3},{5,10}))");
    model._set("A7", "=COLUMNS(T.INV({0.1;0.2;0.3},{5,10}))");
    model._set("A8", "=INDEX(T.INV.2T({0.4;1},5),2)");
    model._set("A9", "=INDEX(CONFIDENCE.T({0.4;TRUE},2,15),2)");
    // Every argument is read before any is checked: an error wins over a bad alpha or p.
    model._set("A10", "=INDEX(BETA.INV(0.4,{2;-1},NA()),2)");
    model._set("A11", "=BETA.INV(0.4,-1,NA())");
    model._set("A12", "=CONFIDENCE.T(0.4,-2,NA())");
    model._set("A13", "=T.INV(2,NA())");
    model._set("A14", "=F.INV(2,NA(),5)");
    model._set("A15", "=CONFIDENCE.T(0.4,2,1.5)");
    model.evaluate();
    let expected = [
        ("A1", "-0.267180866"),
        ("A2", "#VALUE!"),
        ("A3", "#DIV/0!"),
        ("A4", "#NUM!"),
        ("A5", "#N/A"),
        ("A6", "3"),
        ("A7", "2"),
        ("A8", "0"),
        ("A9", "#NUM!"),
        ("A10", "#N/A"),
        ("A11", "#N/A"),
        ("A12", "#N/A"),
        ("A13", "#N/A"),
        ("A14", "#N/A"),
        ("A15", "#DIV/0!"),
    ];
    for (cell, value) in expected {
        assert_eq!(model._get_text(cell), *value, "{cell}");
    }
}

#[test]
fn quantile_searches_that_hung_or_panicked_are_bounded() {
    // statrs' inversion panicked for F.INV(1E-300,5,5) (a NaN first estimate) and never finished
    // for F.INV(1E-50,6000,4) and BETA.INV at 1-2^-53 with these shapes (an infinite Newton step);
    // the capped port is #NUM! there (a DEVIATION: Excel gives 5.081599884913164E-121, 0.000492...
    // and 0.0692..., probes/stat-quantile-lift E1, E6, E9; TODO(T-1871)). The longest search
    // seen, 377 Newton steps, still finishes with statrs' value (E8).
    let mut model = new_empty_model();
    model._set("C1", "=1-2^-53");
    model._set("A1", "=F.INV(1E-300,5,5)");
    model._set("A2", "=F.INV(1E-170,5,5)");
    model._set("A3", "=F.INV(1E-50,6000,4)");
    model._set("A4", "=BETA.INV(C1,1.0024017013258346,12177.162026553196)");
    model._set("A5", "=F.INV(3.718810279553989E-162,4532,43)");
    model._set("A6", "=F.INV(1E-20,5,5)");
    model._set("A7", "=INDEX(F.INV({0.4;1E-300},5,5),2)");
    model.evaluate();
    let expected = [
        ("A1", "#NUM!"),
        ("A2", "#NUM!"),
        ("A3", "#NUM!"),
        ("A4", "#NUM!"),
        ("A5", "0.043055298"),
        ("A6", "5.0816E-09"),
        ("A7", "#NUM!"),
    ];
    for (cell, value) in expected {
        assert_eq!(model._get_text(cell), *value, "{cell}");
    }
}

#[test]
fn quantile_kernels_are_charged_their_weight() {
    // BETA.INV and F.INV are charged 2^14 a element (up to about 167 us each): 2^27 / 2^14 = 8,192
    // elements, so a lifted call of 8,193 is refused before it runs; T.INV and CONFIDENCE.T are
    // charged 2^11 (up to about 17.5 us): 65,536 elements.
    let mut model = new_empty_model();
    model._set("A1", "=ROWS(BETA.INV(SEQUENCE(8193)/8194,2,3))");
    model._set("A2", "=ROWS(BETA.INV(SEQUENCE(8192)/8193,2,3))");
    model._set("A3", "=ROWS(F.INV.RT(0.4,5,SEQUENCE(8193)))");
    model._set("A4", "=ROWS(T.INV(0.4,SEQUENCE(65537)))");
    model._set("A5", "=ROWS(CONFIDENCE.T(0.4,2,SEQUENCE(65537)+1))");
    model._set("A6", "=ROWS(T.INV(0.4,SEQUENCE(20000)))");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"#ERROR!");
    assert_eq!(model._get_text("A2"), *"8192");
    assert_eq!(model._get_text("A3"), *"#ERROR!");
    assert_eq!(model._get_text("A4"), *"#ERROR!");
    assert_eq!(model._get_text("A5"), *"#ERROR!");
    assert_eq!(model._get_text("A6"), *"20000");
}

#[test]
fn unlifted_calls_are_the_scalar_calls() {
    let mut model = new_empty_model();
    model._set("A1", "=BETA.INV(0.4,2,3)");
    model._set("A2", "=BETAINV(0.4,2,3,1,3)");
    model._set("A3", "=F.INV(0.4,5,5)");
    model._set("A4", "=F.INV.RT(0.4,5,5)");
    model._set("A5", "=T.INV(0.4,5)");
    model._set("A6", "=T.INV.2T(0.4,5)");
    model._set("A7", "=CONFIDENCE.T(0.4,2,15)");
    model._set("A8", "=T.INV(0.4)");
    model._set("A9", "=BETA.INV(0.4,2,3,1,1)");
    model._set("A10", "=F.INV(TRUE,5,5)");
    model.evaluate();
    let expected = [
        ("A1", "0.329166503"),
        ("A2", "1.658333007"),
        ("A3", "0.787880531"),
        ("A4", "1.269227961"),
        ("A5", "-0.267180866"),
        ("A6", "0.91954378"),
        ("A7", "0.448261562"),
        ("A8", "#ERROR!"),
        ("A9", "#NUM!"),
        // TODO(T-1849): Excel reads the TRUE as 1 (probes/stat-quantile-lift L2, L19).
        ("A10", "#VALUE!"),
    ];
    for (cell, value) in expected {
        assert_eq!(model._get_text(cell), *value, "{cell}");
    }
}
