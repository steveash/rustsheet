#![allow(clippy::unwrap_used)]

//! The financial functions on the SPEC-0011 driver (T-1372): the core worksheet functions lift
//! over arrays and ranges (`Lift`), the Analysis `ToolPak` ones over arrays only (`LiftArrays`).
//! Evidence: probes/fin-argument-modes, probes/fin-later-argument-ranges, probes/scalar-lift-shapes
//! and -b, probes/scalar-lift-open-a, -b and -d (docs/excel-semantics/financial.md "Financial
//! function argument modes").

use crate::model::Model;
use crate::test::util::new_empty_model;

/// A model with `cells` set and evaluated. Column C holds rates (0.1), D periods (10), B dates
/// (45000), M frequencies (4), each in rows 1-4; F1:F4 is 0.1, blank, "x", `=1/0`.
fn model_with(cells: &[(&str, &str)]) -> Model {
    let mut model = new_empty_model();
    for row in 1..=4 {
        model._set(&format!("C{row}"), "0.1");
        model._set(&format!("D{row}"), "10");
        model._set(&format!("B{row}"), "45000");
        model._set(&format!("M{row}"), "4");
    }
    model._set("F1", "0.1");
    model._set("F3", "x");
    model._set("F4", "=1/0");
    for (cell, formula) in cells {
        model._set(cell, formula);
    }
    model.evaluate();
    model
}

fn check(cells: &[(&str, &str, &str)]) {
    let input: Vec<(&str, &str)> = cells.iter().map(|(c, f, _)| (*c, *f)).collect();
    let model = model_with(&input);
    for (cell, formula, expected) in cells {
        assert_eq!(model._get_text(cell), *expected, "{cell}: {formula}");
    }
}

// ------------------------------------------------------------------ core worksheet functions

#[test]
fn core_functions_lift_over_a_range_on_its_own_rows() {
    // probes/scalar-lift-shapes H13, H14 and fin-argument-modes H90-H330 (two-cell ranges).
    check(&[
        ("H1", "=ROWS(PMT(C1:C4,10,100))", "4"),
        ("H2", "=ROWS(PMT(0.1,D1:D4,100))", "4"),
        ("H3", "=INDEX(PMT(C1:C4,10,100),2)=PMT(0.1,10,100)", "TRUE"),
        ("H4", "=ROWS(PV(C1:C2,36,-100,0,0))", "2"),
        ("H5", "=ROWS(FV(0.01,12,-100,-1000,C1:C2))", "2"),
        ("H6", "=ROWS(NPER(C1:C2,-100,1000,0,0))", "2"),
        ("H7", "=ROWS(IPMT(C1:C2,2,36,8000,0,0))", "2"),
        ("H8", "=ROWS(PPMT(C1:C2,2,36,8000,0,0))", "2"),
        ("H9", "=ROWS(ISPMT(C1:C2,2,36,8000))", "2"),
        ("H10", "=ROWS(RRI(D1:D2,10000,11000))", "2"),
        ("H11", "=ROWS(PDURATION(C1:C2,2000,2200))", "2"),
        ("H12", "=ROWS(SLN(30000,7500,D1:D2))", "2"),
        ("H13", "=ROWS(SYD(30000,7500,D1:D2,1))", "2"),
        ("H14", "=ROWS(DDB(10000,1000,D1:D2,2,2))", "2"),
    ]);
}

#[test]
fn rate_lifts_its_guess_and_keeps_its_scalar_value() {
    // probes/fin-argument-modes H284 and fin-later-argument-ranges K7.
    check(&[
        ("H1", "=ROWS(RATE(36,-300,8000,0,0,C1:C2))", "2"),
        (
            "H2",
            "=INDEX(RATE(36,-300,8000,0,0,C1:C2),2)=RATE(36,-300,8000,0,0,0.1)",
            "TRUE",
        ),
        ("H3", "=ROWS(RATE({36;48},-300,8000))", "2"),
    ]);
}

#[test]
fn core_shapes_pad_with_na_and_broadcast_rows_against_columns() {
    // probes/scalar-lift-shapes H17-H25, H47, H48.
    check(&[
        ("H1", "=ROWS(PMT({0.1;0.2;0.3},{10;20},100))", "3"),
        ("H2", "=INDEX(PMT({0.1;0.2;0.3},{10;20},100),3)", "#N/A"),
        (
            "H3",
            "=INDEX(PMT({0.1;0.2;0.3},{10;20},100),2)=PMT(0.2,20,100)",
            "TRUE",
        ),
        (
            "H4",
            "=INDEX(PMT({0.1,0.2;0.3,0.4},{10,20,30},100),2,3)",
            "#N/A",
        ),
        (
            "H5",
            "=INDEX(PMT({0.1;0.2},{10,20,30},100),2,3)=PMT(0.2,30,100)",
            "TRUE",
        ),
        ("H6", "=ROWS(PMT(C1:C4,{10;20},100))", "4"),
        ("H7", "=INDEX(PMT(C1:C4,{10;20},100),3)", "#N/A"),
        // gen-m9/b58b6d7cd39d195f A21: an identity matrix in PPMT's rate is a 3x3 result.
        ("H8", "=ROWS(PPMT(MUNIT(3),1,359,117633,154))", "3"),
        ("H9", "=COLUMNS(PPMT(MUNIT(3),1,359,117633,154))", "3"),
    ]);
}

#[test]
fn core_element_errors_follow_the_kernel_order() {
    // probes/scalar-lift-shapes H26-H33, H41-H45; probes/scalar-lift-open-a W14, W15, W20, W21.
    check(&[
        ("H1", "=INDEX(PMT({0.1;\"x\"},10,100),2)", "#VALUE!"),
        ("H2", "=INDEX(PMT({1;#N/A},{#DIV/0!;1},100),1)", "#DIV/0!"),
        ("H3", "=INDEX(PMT({1;#N/A},{#DIV/0!;#NUM!},100),2)", "#N/A"),
        ("H4", "=INDEX(PMT({#NUM!},{#N/A},100),1)", "#NUM!"),
        ("H5", "=INDEX(PMT({#N/A},{#NUM!},100),1)", "#N/A"),
        ("H6", "=ROWS(PMT(NA(),{10;20},100))", "2"),
        ("H7", "=INDEX(PMT(NA(),{10;20},100),2)", "#N/A"),
        ("H8", "=INDEX(PMT(\"x\",{10;20},100),2)", "#VALUE!"),
        // A blank cell is 0 and a text cell #VALUE! (H41-H43).
        ("H9", "=INDEX(PMT(F1:F4,10,100),2)", "-10"),
        ("H10", "=INDEX(PMT(F1:F4,10,100),3)", "#VALUE!"),
        ("H11", "=INDEX(PMT(F1:F4,10,100),4)", "#DIV/0!"),
    ]);
}

#[test]
fn a_3d_range_in_a_core_argument_is_ref_in_every_element() {
    // probes/scalar-lift-open-b W2, W9 and -d W17, W18.
    let mut model = new_empty_model();
    model.add_sheet("Sheet2").unwrap();
    model._set("C1", "100");
    model._set("C2", "100");
    model._set("H1", "=ROWS(PMT({0.1;0.2},10,Sheet1:Sheet2!C1:C2))");
    model._set("H2", "=INDEX(PMT({0.1;0.2},10,Sheet1:Sheet2!C1:C2),1)");
    model._set("H3", "=INDEX(PMT({0.1;0.2},10,Sheet1:Sheet2!C1:C2),2)");
    model._set("H4", "=ROWS(PMT(Sheet1:Sheet2!C1:C2,10,100))");
    model.evaluate();
    assert_eq!(model._get_text("H1"), "2");
    assert_eq!(model._get_text("H2"), "#REF!");
    assert_eq!(model._get_text("H3"), "#REF!");
    assert_eq!(model._get_text("H4"), "#REF!");
}

// ------------------------------------------------------------------ Analysis ToolPak functions

#[test]
fn effect_lifts_arrays_and_refuses_ranges_in_both_arguments() {
    // probes/scalar-lift-shapes H11, H12, -b H, I; scalar-lift-open-a W3, W4.
    check(&[
        ("H1", "=ROWS(EFFECT(C1:C4,4))", "#VALUE!"),
        ("H2", "=ROWS(EFFECT(0.1,M1:M4))", "#VALUE!"),
        ("H3", "=ROWS(EFFECT({0.05;0.05},4))", "2"),
        ("H4", "=INDEX(EFFECT({0.05;0.1},4),2)=EFFECT(0.1,4)", "TRUE"),
        ("H5", "=INDEX(EFFECT({0.1;0.2},M1:M4),1)", "#VALUE!"),
        ("H6", "=INDEX(EFFECT({0.1;0.2},M1:M4),2)", "#VALUE!"),
        ("H7", "=INDEX(EFFECT(1/0,C1:C4),1)", "#DIV/0!"),
        ("H8", "=INDEX(EFFECT(C1:C4,1/0),1)", "#VALUE!"),
        ("H9", "=ROWS(NOMINAL(0.05,M1:M2))", "#VALUE!"),
        ("H10", "=ROWS(NOMINAL(0.05,{4;4}))", "2"),
        ("H11", "=ROWS(DOLLARDE(1.02,{16;16}))", "2"),
        ("H12", "=ROWS(DOLLARFR(1.1,M1:M2))", "#VALUE!"),
        // A one-cell range is its value.
        ("H13", "=EFFECT(C1,M1)=EFFECT(0.1,4)", "TRUE"),
    ]);
}

#[test]
fn accrintm_lifts_its_basis_with_element_values_and_errors() {
    // probes/fin-later-argument-ranges J1-J15, H9.
    check(&[
        ("H1", "=ROWS(ACCRINTM(39083,39448,0.1,1000,{0;1;2}))", "3"),
        ("H2", "=INDEX(ACCRINTM(39083,39448,0.1,1000,{0;1;2;3;4}),1)", "100"),
        (
            "H3",
            "=INDEX(ACCRINTM(39083,39448,0.1,1000,{0;1;2;3;4}),3)=ACCRINTM(39083,39448,0.1,1000,2)",
            "TRUE",
        ),
        ("H4", "=INDEX(ACCRINTM(39083,39448,0.1,1000,{5;-1}),2)", "#NUM!"),
        ("H5", "=INDEX(ACCRINTM(39083,39448,0.1,1000,{TRUE;\"1\"}),1)", "#VALUE!"),
        ("H6", "=INDEX(ACCRINTM(39083,39448,0.1,1000,{TRUE;\"1\"}),2)", "100"),
        ("H7", "=INDEX(ACCRINTM(39083,39448,0.1,1000,{1.9;\"x\"}),2)", "#VALUE!"),
        ("H8", "=COLUMNS(ACCRINTM({39083;39100},39448,0.1,1000,{0,1}))", "2"),
        (
            "H9",
            "=INDEX(ACCRINTM({39083;39100},39448,0.1,1000,{0,1}),2,2)=ACCRINTM(39100,39448,0.1,1000,1)",
            "TRUE",
        ),
        ("H10", "=ROWS(ACCRINTM(39083,39448,0.1,1000,B1:B2))", "#VALUE!"),
        ("H11", "=ROWS(ACCRINTM(39083,39448,0.1,B1:B2,0))", "#VALUE!"),
    ]);
}

#[test]
fn accrintm_reads_every_argument_before_it_checks_values() {
    // An error basis wins over a bad rate (probes/fin-later-argument-ranges J17, J20), and the
    // range refusal follows the read order (probes/scalar-lift-open-d W8-W16).
    check(&[
        ("H1", "=ACCRINTM(39083,39448,-0.1,1000,NA())", "#N/A"),
        (
            "H2",
            "=INDEX(ACCRINTM(39083,39448,-0.1,1000,{#N/A}),1)",
            "#N/A",
        ),
        ("H3", "=ACCRINTM(39448,39083,0.1,1000,5)", "#NUM!"),
        ("H4", "=INDEX(ACCRINTM(NA(),B1:B4,0.1,1000),1)", "#N/A"),
        ("H5", "=INDEX(ACCRINTM(B1:B4,NA(),0.1,1000),1)", "#VALUE!"),
        ("H6", "=INDEX(ACCRINTM(45000,46000,NA(),B1:B4),1)", "#N/A"),
        (
            "H7",
            "=INDEX(ACCRINTM(45000,46000,B1:B4,NA()),1)",
            "#VALUE!",
        ),
        (
            "H8",
            "=INDEX(ACCRINTM({45000;45100},B1:B4,NA(),1000),1)",
            "#VALUE!",
        ),
        (
            "H9",
            "=INDEX(ACCRINTM(1/0,{45000;45100},NA(),B1:B4),1)",
            "#DIV/0!",
        ),
        (
            "H10",
            "=INDEX(ACCRINTM({45000;45100},45500,0.1,1000),1)=ACCRINTM(45000,45500,0.1,1000)",
            "TRUE",
        ),
    ]);
}

#[test]
fn discount_securities_and_t_bills_lift_arrays_and_refuse_ranges() {
    // probes/fin-argument-modes and probes/fin-later-argument-ranges H11-H82.
    check(&[
        ("H1", "=ROWS(PRICEDISC(39448,{43831;43831},0.0525,100,0))", "2"),
        ("H2", "=ROWS(PRICEDISC(39448,43831,0.0525,100,M1:M2))", "#VALUE!"),
        ("H3", "=ROWS(DISC(39448,43831,{97;97},100,0))", "2"),
        ("H4", "=ROWS(INTRATE(39448,43831,95000,{100000;100000},0))", "2"),
        ("H5", "=ROWS(PRICEMAT(39448,43831,39083,0.061,{0.061;0.061},0))", "2"),
        ("H6", "=ROWS(RECEIVED(39448,43831,1000000,{0.0575;0.0575},0))", "2"),
        ("H7", "=ROWS(YIELDDISC(39448,43831,99.795,100,{0;0}))", "2"),
        ("H8", "=ROWS(YIELDMAT(39448,43831,{39083;39083},0.0625,100.0123,0))", "2"),
        ("H9", "=ROWS(TBILLEQ(39508,{39691;39691},0.0914))", "2"),
        ("H10", "=ROWS(TBILLPRICE(39508,39691,{0.0914;0.0914}))", "2"),
        ("H11", "=ROWS(TBILLYIELD(39508,39691,C1:C2))", "#VALUE!"),
        (
            "H12",
            "=INDEX(TBILLPRICE(39508,39691,{0.0914;0.05}),2)=TBILLPRICE(39508,39691,0.05)",
            "TRUE",
        ),
        (
            "H13",
            "=INDEX(YIELDMAT(39448,43831,39083,0.0625,{100.0123;99},0),2)=YIELDMAT(39448,43831,39083,0.0625,99,0)",
            "TRUE",
        ),
    ]);
}

#[test]
fn the_table_enters_rate_as_weighted() {
    use crate::functions::Function;
    // RATE's Newton iteration has a fixed 50-step cap; each element is charged that weight.
    let weight = |function: Function| function.arg_kinds().map(|kinds| kinds.work.weight());
    assert_eq!(weight(Function::Rate), Some(50));
    assert_eq!(weight(Function::Pmt), Some(1));
}

/// DB and VDB lift every argument (probes/fin-argument-modes; T-1820), each element the scalar
/// call, and stay bounded: a period past 100,000 is `#NUM!` (docs/excel-semantics/financial.md
/// "DB and VDB period cap", DEVIATION) and a lift is charged by each element's period.
#[test]
fn db_and_vdb_lift_and_bound_their_periods() {
    check(&[
        ("G1", "1000", "1000"),
        ("G2", "2000", "2000"),
        ("H1", "=ROWS(DB(G1:G2,100,6,2,7))", "2"),
        ("H2", "=ROWS(VDB(G1:G2,300,10,0,1,2,FALSE))", "2"),
        (
            "H3",
            "=INDEX(DB({1000;2000},100,6,2,7),2)=DB(2000,100,6,2,7)",
            "TRUE",
        ),
        (
            "H4",
            "=INDEX(VDB(2400,300,10,0,{1;2},2,FALSE),2)=VDB(2400,300,10,0,2,2,FALSE)",
            "TRUE",
        ),
        ("H5", "=DB(1,0,1E15,1E15)", "#NUM!"),
        ("H6", "=VDB(1000,0,1E9,0,1E9)", "#NUM!"),
        ("H7", "=DB(1000,1,1E6,100000)>=0", "TRUE"),
        (
            "H8",
            "=ROWS(DB(1000,1,1E6,SEQUENCE(1100,1,100000,0)))",
            "#ERROR!",
        ),
        ("H9", "=ROWS(DB(1000,1,1E6,SEQUENCE(3000,1,1,0)))", "3000"),
        (
            "H10",
            "=ROWS(VDB(1000,1,1E6,0,SEQUENCE(1100,1,100000,0)))",
            "#ERROR!",
        ),
        // Excel's own checks come before the cap: a period past life + 1 is `#NUM!` either way.
        ("H11", "=DB(1000,100,6,8)", "#NUM!"),
    ]);
}

/// NPV's rate, IRR's guess and MIRR's rates lift over arrays and ranges beside their whole cash
/// flows, each element the scalar call (probes/fin-argument-modes H155-H156, H172-H174,
/// H187-H188; a values array does not lift: H386; T-1821).
#[test]
fn npv_irr_and_mirr_lift_their_rates_beside_whole_cash_flows() {
    check(&[
        ("K1", "-10000", "-10000"),
        ("K2", "2750", "2750"),
        ("K3", "4250", "4250"),
        ("K4", "3250", "3250"),
        ("K5", "2750", "2750"),
        ("H1", "=ROWS(NPV(C1:C2,K1:K5))", "2"),
        ("H2", "=ROWS(NPV({0.1;0.1},K1:K5))", "2"),
        ("H3", "=ROWS(NPV(0.1,{1;2}))", "1"),
        (
            "H4",
            "=INDEX(NPV({0.1;0.2},K1:K5),2)=NPV(0.2,K1:K5)",
            "TRUE",
        ),
        ("H5", "=ROWS(IRR(K1:K5,C1:C2))", "2"),
        (
            "H6",
            "=INDEX(IRR(K1:K5,{0.1;0.2}),2)=IRR(K1:K5,0.2)",
            "TRUE",
        ),
        ("H7", "=ROWS(MIRR(K1:K5,C1:C2,0.12))", "2"),
        ("H8", "=ROWS(MIRR(K1:K5,0.1,{0.12;0.12}))", "2"),
        (
            "H9",
            "=INDEX(MIRR(K1:K5,0.1,{0.12;0.2}),2)=MIRR(K1:K5,0.1,0.2)",
            "TRUE",
        ),
        // NPV reads its rate before its values; IRR and MIRR their values first.
        ("H10", "=INDEX(NPV({\"x\";0.1},K1:K2,F4),1)", "#VALUE!"),
        ("H11", "=INDEX(NPV({\"x\";0.1},K1:K2,F4),2)", "#DIV/0!"),
        ("H12", "=INDEX(IRR(F4,{\"x\";0.1}),1)", "#DIV/0!"),
        ("H13", "=INDEX(MIRR(F4,{\"x\";0.1},0.1),1)", "#DIV/0!"),
        ("H14", "=INDEX(MIRR(K1:K5,{\"x\";0.1},0.1),1)", "#VALUE!"),
        ("H15", "=NPV(0.1)", "#ERROR!"),
        ("H16", "=IRR(K1:K5,0.1,0.1)", "#ERROR!"),
        ("H17", "=MIRR(K1:K5,0.1)", "#ERROR!"),
    ]);
}

/// The cash flows are evaluated once per call, before the elements: random flows give every
/// element of a constant rate the same result.
#[test]
fn npv_irr_and_mirr_evaluate_their_cash_flows_once() {
    check(&[
        (
            "H1",
            "=LET(x,NPV({0.1;0.1},RANDARRAY(5)),INDEX(x,1)=INDEX(x,2))",
            "TRUE",
        ),
        (
            "H2",
            "=LET(x,IRR(RANDARRAY(5)+{-10;1;1;1;1},{0.1;0.1}),INDEX(x,1)=INDEX(x,2))",
            "TRUE",
        ),
        (
            "H3",
            "=LET(x,MIRR(RANDARRAY(5)+{-10;1;1;1;1},{0.1;0.1},0.1),INDEX(x,1)=INDEX(x,2))",
            "TRUE",
        ),
    ]);
}

/// Every lifted element walks the whole cash flow (IRR up to `IRR_WEIGHT` times), so a lift over
/// long flows is `#ERROR!` before its first element, while the unlifted call runs.
#[test]
fn a_lift_over_long_cash_flows_is_refused_by_the_work_bound() {
    let mut model = new_empty_model();
    let sheet = model.workbook.worksheet_mut(0).unwrap();
    for row in 1..=300_000 {
        sheet
            .set_cell_with_number(row, 1, f64::from(row), 0)
            .unwrap();
    }
    for (cell, formula) in [
        ("C1", "=ROWS(IRR(A1:A300000,{0.1;0.1}))"),
        ("C2", "=IRR(A1:A300000)"),
        ("C3", "=ROWS(IRR(A1:A100,{0.1;0.1}))"),
        ("C4", "=ROWS(NPV(SEQUENCE(500),A1:A300000))"),
        ("C5", "=ROWS(NPV(SEQUENCE(100),A1:A300000))"),
        ("C6", "=ROWS(MIRR(A1:A300000,SEQUENCE(500),0.1))"),
    ] {
        model._set(cell, formula);
    }
    model.evaluate();
    assert_eq!(model._get_text("C1"), "#ERROR!");
    assert_eq!(model._get_text("C2"), "#NUM!");
    assert_eq!(model._get_text("C3"), "2");
    assert_eq!(model._get_text("C4"), "#ERROR!");
    assert_eq!(model._get_text("C5"), "100");
    assert_eq!(model._get_text("C6"), "#ERROR!");
}
