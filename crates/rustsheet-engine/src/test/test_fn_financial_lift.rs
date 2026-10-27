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
    // RATE's secant has a fixed 200-step cap (Excel's, probes/rate-step-cap); each element is
    // charged the time of a run to it, 2^13 units (T-2814), so a lift runs at most 16,384.
    let weight = |function: Function| function.arg_kinds().map(|kinds| kinds.work.weight());
    assert_eq!(weight(Function::Rate), Some(1 << 13));
    assert_eq!(weight(Function::Pmt), Some(1));
}

/// A lifted RATE runs up to 2^27 / 2^13 = 16,384 elements; one more makes the call `#ERROR!`
/// before any element runs (T-2814; arrays.md, a DEVIATION).
#[test]
fn a_lifted_rate_past_its_work_cap_is_refused() {
    check(&[
        ("A1", "=ROWS(RATE(SEQUENCE(16384)*0+1,-1,100))", "16384"),
        ("A2", "=ROWS(RATE(SEQUENCE(16385)*0+1,-1,100))", "#ERROR!"),
    ]);
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
        // A period past i32::MAX is period 1 (probes/fin-syd-db-domain G1).
        ("H5", "=DB(1,0,1E15,1E15)", "$1.00"),
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
        // Hand-written, not oracle values: NPV checks its rate before its values, IRR and MIRR
        // their values first, the scalar order kept (financial.md "NPV, IRR and MIRR error
        // order", UNVERIFIED; T-2103). A values error is every element's.
        ("H10", "=INDEX(NPV({\"x\";0.1},K1:K2,F4),1)", "#VALUE!"),
        ("H11", "=INDEX(NPV({\"x\";0.1},K1:K2,F4),2)", "#DIV/0!"),
        // A referenced cell holding an error is #VALUE! for IRR (probes/fin-irr-error-array H3, T-2390).
        ("H12", "=INDEX(IRR(F4,{\"x\";0.1}),1)", "#VALUE!"),
        ("H13", "=INDEX(MIRR(F4,{\"x\";0.1},0.1),1)", "#DIV/0!"),
        ("H14", "=INDEX(MIRR(K1:K5,{\"x\";0.1},0.1),1)", "#VALUE!"),
        ("H18", "=SUM(--ISERROR(IRR(F4,{0.1;0.1})))", "2"),
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

/// The cash flows of probes/fin-argument-modes: values in K1:K5, dates in J1:J5.
const DATED_FLOWS: [(&str, &str, &str); 10] = [
    ("K1", "-10000", "-10000"),
    ("K2", "2750", "2750"),
    ("K3", "4250", "4250"),
    ("K4", "3250", "3250"),
    ("K5", "2750", "2750"),
    ("J1", "39448", "39448"),
    ("J2", "39508", "39508"),
    ("J3", "39751", "39751"),
    ("J4", "39859", "39859"),
    ("J5", "39904", "39904"),
];

/// XNPV's rate, XIRR's guess and FVSCHEDULE's principal lift over arrays and refuse a multi-cell
/// range, beside their whole cash flows, dates and schedule, each element the scalar call
/// (probes/fin-argument-modes H137-H138, H338-H339, H341-H342; T-1843).
#[test]
fn xnpv_xirr_and_fvschedule_lift_beside_their_whole_flows() {
    let mut cells = DATED_FLOWS.to_vec();
    cells.extend([
        ("H1", "=ROWS(XNPV({0.09;0.09},K1:K5,J1:J5))", "2"),
        ("H2", "=ROWS(XNPV(C1:C2,K1:K5,J1:J5))", "#VALUE!"),
        (
            "H3",
            "=INDEX(XNPV({0.09;0.2},K1:K5,J1:J5),2)=XNPV(0.2,K1:K5,J1:J5)",
            "TRUE",
        ),
        ("H4", "=ROWS(XIRR(K1:K5,J1:J5,{0.1;0.1}))", "2"),
        ("H5", "=ROWS(XIRR(K1:K5,J1:J5,C1:C2))", "#VALUE!"),
        (
            "H6",
            "=INDEX(XIRR(K1:K5,J1:J5,{0.1;0.2}),2)=XIRR(K1:K5,J1:J5,0.2)",
            "TRUE",
        ),
        ("H7", "=ROWS(FVSCHEDULE({1000;1000},{0.1,0.2}))", "2"),
        ("H8", "=ROWS(FVSCHEDULE(C1:C2,{0.1,0.2}))", "#VALUE!"),
        ("H9", "=INDEX(FVSCHEDULE({1000;2000},{0.1,0.2}),2)", "2640"),
        // The scalar order: XNPV's rate and FVSCHEDULE's principal before the flows, XIRR's
        // flows before its guess. A flows error is every element's.
        ("H10", "=INDEX(XNPV({\"x\";0.1},F4,J1),1)", "#VALUE!"),
        ("H11", "=INDEX(XNPV({\"x\";0.1},F4,J1),2)", "#DIV/0!"),
        ("H12", "=INDEX(XIRR(F4,J1:J5,{\"x\";0.1}),1)", "#DIV/0!"),
        ("H13", "=INDEX(XIRR(K1:K5,J1:J5,{\"x\";0.1}),1)", "#VALUE!"),
        ("H14", "=INDEX(FVSCHEDULE({TRUE;1000},F4),1)", "#VALUE!"),
        ("H15", "=INDEX(FVSCHEDULE({TRUE;1000},F4),2)", "#DIV/0!"),
        ("H16", "=FVSCHEDULE(TRUE,0.1)", "#VALUE!"),
        ("H17", "=XNPV(0.1,K1:K5)", "#ERROR!"),
        ("H18", "=XIRR(K1:K5)", "#ERROR!"),
        ("H19", "=FVSCHEDULE(1000)", "#ERROR!"),
    ]);
    check(&cells);
}

/// The flows, dates and schedule are evaluated once per call, before the elements: random flows
/// give every element of a constant rate the same result.
#[test]
fn xnpv_xirr_and_fvschedule_evaluate_their_flows_once() {
    let mut cells = DATED_FLOWS.to_vec();
    cells.extend([
        (
            "H1",
            "=LET(x,XNPV({0.1;0.1},RANDARRAY(5),J1:J5),INDEX(x,1)=INDEX(x,2))",
            "TRUE",
        ),
        (
            "H2",
            "=LET(x,XIRR(RANDARRAY(5)+{-10;1;1;1;1},J1:J5,{0.1;0.1}),INDEX(x,1)=INDEX(x,2))",
            "TRUE",
        ),
        (
            "H3",
            "=LET(x,FVSCHEDULE({1;1},RANDARRAY(5)),INDEX(x,1)=INDEX(x,2))",
            "TRUE",
        ),
    ]);
    check(&cells);
}

/// Every lifted XIRR element may walk the flows up to `XIRR_WEIGHT` times, and every XNPV and
/// FVSCHEDULE element walks them once, so a lift over long flows is `#ERROR!` before its first
/// element.
#[test]
fn a_lifted_xirr_over_long_flows_is_refused_by_the_work_bound() {
    let mut model = new_empty_model();
    let sheet = model.workbook.worksheet_mut(0).unwrap();
    for row in 1..=300_000 {
        sheet
            .set_cell_with_number(row, 1, if row == 1 { -1e9 } else { 1.0 }, 0)
            .unwrap();
        sheet
            .set_cell_with_number(row, 2, 39448.0 + f64::from(row), 0)
            .unwrap();
    }
    for (cell, formula) in [
        ("C1", "=ROWS(XIRR(A1:A300000,B1:B300000,{0.1;0.1}))"),
        ("C2", "=ROWS(XIRR(A1:A100,B1:B100,{0.1;0.1}))"),
        ("C3", "=ROWS(XNPV(SEQUENCE(500),A1:A300000,B1:B300000))"),
        ("C4", "=ROWS(FVSCHEDULE(SEQUENCE(500),A1:A300000))"),
        // Mismatched counts: the long dates are checked once, so each element is O(1).
        (
            "C5",
            "=SUM(--ISERROR(XNPV(SEQUENCE(100000),A1,B1:B300000)))",
        ),
        (
            "C6",
            "=SUM(--ISERROR(XIRR(A1:A2,B1:B300000,SEQUENCE(100000))))",
        ),
    ] {
        model._set(cell, formula);
    }
    model.evaluate();
    assert_eq!(model._get_text("C1"), "#ERROR!");
    assert_eq!(model._get_text("C2"), "2");
    assert_eq!(model._get_text("C3"), "#ERROR!");
    assert_eq!(model._get_text("C4"), "#ERROR!");
    assert_eq!(model._get_text("C5"), "100000");
    assert_eq!(model._get_text("C6"), "100000");
}

// ------------------------------------------- XIRR, XNPV and FVSCHEDULE argument checks (T-2428)

/// A model for the argument checks: H1 "x", H2 TRUE, H3 FALSE, H4 "1.5" as text, H5 empty,
/// G1:G5 dates, K1:K2 "ü" and 0.3, then `cells`.
fn checks_model(cells: &[(&str, &str)]) -> Model {
    let mut model = new_empty_model();
    model._set("H1", "x");
    model._set("H2", "TRUE");
    model._set("H3", "FALSE");
    model._set("H4", "'1.5");
    for (row, date) in ["39448", "39508", "39859", "39904", "40000"]
        .iter()
        .enumerate()
    {
        model._set(&format!("G{}", row + 1), date);
    }
    model._set("K1", "ü");
    model._set("K2", "0.3");
    for (cell, formula) in cells {
        model._set(cell, formula);
    }
    model.evaluate();
    model
}

fn check_args(cells: &[(&str, &str, &str)]) {
    let input: Vec<(&str, &str)> = cells.iter().map(|(c, f, _)| (*c, *f)).collect();
    let model = checks_model(&input);
    for (cell, formula, expected) in cells {
        assert_eq!(model._get_text(cell), *expected, "{cell}: {formula}");
    }
}

/// financial.md "XIRR and XNPV under the gen-m9 statistical cells": the sizes come before the
/// numbers are read.
#[test]
fn xirr_checks_sizes_before_reading_numbers() {
    check_args(&[
        // Different counts are #NUM! whatever the values hold (gen-m9/1b60bf99d71424e7 A372).
        ("A1", "=XIRR(K1:K2,G1)", "#NUM!"),
        ("A2", "=XIRR({-100,#N/A,60},G1:G5)", "#NUM!"),
        // One value with one date is #N/A (gen-m9/30e20ba3302fea8b A110).
        ("A3", "=XIRR(1E15,G1)", "#N/A"),
        // A single text or boolean date cell is #VALUE! (gen-m9/1cd3096e3e87c965 A48).
        ("A4", "=XIRR({-100,#N/A,60},H3)", "#VALUE!"),
        // An error dates argument is that error before the sizes.
        ("A5", "=XIRR({-100,50,60},1/0)", "#DIV/0!"),
        // A boolean guess keeps the element reading's #VALUE! over a size error
        // (`XIRR(<6 cells, text among them>,{3 dates},<TRUE>)`, gen-m9/b6a02893a8d6b151 A226);
        // the error-guess order is T-2611's.
        ("A7", "=XIRR(K1:K2,G1:G3,H2)", "#VALUE!"),
        // Matching counts still solve.
        // 1.1^(365/60) - 1.
        ("A6", "=ROUND(XIRR({-100,110},G1:G2),6)", "0.785688"),
    ]);
}

/// financial.md "XIRR and XNPV under the gen-m9 statistical cells": XNPV's rate, then one values
/// cell, an error dates argument, one dates cell, then the counts.
#[test]
fn xnpv_checks_rate_cells_and_sizes_in_order() {
    check_args(&[
        // A boolean rate is #VALUE! (gen-m9/1dce4774f5c0e736 A22).
        ("A1", "=XNPV(H2,{-100,50},G1)", "#VALUE!"),
        // One non-numeric values cell is #VALUE!, before the dates' error (3aa168259335be54 A334).
        ("A2", "=XNPV(0.1,H3,NA())", "#VALUE!"),
        // Numeric text is not refused there: the counts differ (81bcdc0bec0c8e6b A37).
        ("A3", "=XNPV(0.1,H4,G1:G2)", "#NUM!"),
        // An error dates argument comes before the values' cells (5c6a23c4f6597f1c A111).
        ("A4", "=XNPV(0.05,H1:H2,NA())", "#N/A"),
        // One non-numeric dates cell is #VALUE! (d0854366830dcdee A274).
        ("A5", "=XNPV(0.1,{-100,50},H1)", "#VALUE!"),
        // Different counts are #NUM! before an error value (bc04c1f478c9896c A313).
        ("A6", "=XNPV(0.1,{-100,#N/A,60},G1:G5)", "#NUM!"),
        ("A7", "=XNPV(0.2,0.3,H1:H4)", "#NUM!"),
        // Matching counts still discount.
        // -100 + 110/1.1^(60/365).
        ("A8", "=ROUND(XNPV(0.1,{-100,110},G1:G2),6)", "8.290014"),
    ]);
}

/// One empty schedule cell leaves the principal (gen-m9/30e20ba3302fea8b A250).
#[test]
fn fvschedule_of_one_empty_cell_is_the_principal() {
    check_args(&[
        ("A1", "=FVSCHEDULE(6527,H5)", "6527"),
        ("A2", "=FVSCHEDULE(100,IF(TRUE,H5,H1))", "100"),
    ]);
}
