#![allow(clippy::unwrap_used)]

//! The coupon-schedule Analysis `ToolPak` functions on the SPEC-0011 driver (T-1822): ACCRINT,
//! the COUP* family, DURATION, MDURATION and PRICE lift over arrays in every argument and refuse
//! a multi-cell range there (`LiftArrays`). Evidence: probes/fin-argument-modes,
//! probes/fin-remaining-later-argument-ranges, probes/scalar-lift-open-a Y1-Y7 and
//! probes/fin-coupon-walk-bounds (docs/excel-semantics/financial.md "Financial function argument
//! modes", "Coupon settlement before the calendar").

use crate::test::util::new_empty_model;

/// Evaluates `cells` on a sheet whose G1:G2 is 43831 (a maturity), G3:G4 is 2 (a frequency) and
/// G5:G6 is 0.065 (a yield), and checks each cell's text.
fn check(cells: &[(&str, &str, &str)]) {
    let mut model = new_empty_model();
    for (cell, value) in [
        ("G1", "43831"),
        ("G2", "43831"),
        ("G3", "2"),
        ("G4", "2"),
        ("G5", "0.065"),
        ("G6", "0.065"),
    ] {
        model._set(cell, value);
    }
    for (cell, formula, _) in cells {
        model._set(cell, formula);
    }
    model.evaluate();
    for (cell, formula, expected) in cells {
        assert_eq!(model._get_text(cell), *expected, "{cell}: {formula}");
    }
}

#[test]
fn every_argument_lifts_over_an_array() {
    // probes/fin-argument-modes H2-H10, H37-H71, H118-H124, H164-H170, H252-H259.
    check(&[
        (
            "A1",
            "=ROWS(ACCRINT(39083,39264,39448,0.1,1000,2,0,{TRUE;TRUE}))",
            "2",
        ),
        (
            "A2",
            "=ROWS(ACCRINT({39083;39083},39264,39448,0.1,1000,2,0,TRUE))",
            "2",
        ),
        ("A3", "=ROWS(COUPDAYBS(39448,43831,2,{0;0}))", "2"),
        ("A4", "=ROWS(COUPDAYS(39448,{43831;43831},2,0))", "2"),
        ("A5", "=ROWS(COUPDAYSNC(39448,43831,{2;2},0))", "2"),
        ("A6", "=ROWS(COUPNCD({39448;39448},43831,2,0))", "2"),
        ("A7", "=ROWS(COUPNUM(39448,43831,{2;2},0))", "2"),
        ("A8", "=ROWS(COUPPCD(39448,43831,2,{0;0}))", "2"),
        (
            "A9",
            "=ROWS(DURATION(39448,43831,0.08,{0.09;0.09},2,0))",
            "2",
        ),
        (
            "A10",
            "=ROWS(MDURATION(39448,43831,0.08,0.09,{2;2},0))",
            "2",
        ),
        (
            "A11",
            "=ROWS(PRICE(39448,43831,0.0575,0.065,{100;100},2,0))",
            "2",
        ),
        // A row against a column broadcasts to a grid.
        (
            "A12",
            "=COLUMNS(COUPNUM({39448,39630},{43831;44196},2))",
            "2",
        ),
        ("A13", "=ROWS(COUPNUM({39448,39630},{43831;44196},2))", "2"),
    ]);
}

#[test]
fn each_element_is_the_scalar_call() {
    // probes/fin-remaining-later-argument-ranges K12, K13.
    check(&[
        ("A1", "=INDEX(COUPNUM(39448,{43831;44196},2,0),2)", "26"),
        (
            "A2",
            "=INDEX(PRICE(39448,43831,0.0575,{0.065;0.07},100,2,0),2)=PRICE(39448,43831,0.0575,0.07,100,2,0)",
            "TRUE",
        ),
        (
            "A3",
            "=INDEX(MDURATION(39448,43831,0.08,{0.09;0.1},2,0),2)=MDURATION(39448,43831,0.08,0.1,2,0)",
            "TRUE",
        ),
        (
            "A4",
            "=INDEX(ACCRINT(39083,39264,39448,0.1,1000,{2;4},0,TRUE),2)=ACCRINT(39083,39264,39448,0.1,1000,4,0,TRUE)",
            "TRUE",
        ),
        // An element error stays in its position, the others are computed.
        ("A5", "=INDEX(COUPDAYS(39448,43831,{2;3},0),1)", "180"),
        ("A6", "=INDEX(COUPDAYS(39448,43831,{2;3},0),2)", "#NUM!"),
        ("A7", "=INDEX(COUPPCD(39448,43831,{2;\"x\"}),2)", "#VALUE!"),
    ]);
}

#[test]
fn a_range_is_refused_in_every_argument() {
    // probes/fin-remaining-later-argument-ranges H1-H172 (on the range's own rows),
    // probes/scalar-lift-open-a Y1-Y7, fin-argument-modes H3, H38, H119, H253.
    check(&[
        ("A1", "=ROWS(COUPDAYBS(39448,G1:G2,2,0))", "#VALUE!"),
        ("A2", "=ROWS(COUPNCD(39448,43831,G3:G4,0))", "#VALUE!"),
        (
            "A3",
            "=ROWS(PRICE(39448,43831,0.0575,G5:G6,100,2,0))",
            "#VALUE!",
        ),
        (
            "A4",
            "=ROWS(DURATION(39448,G1:G2,0.08,0.09,2,0))",
            "#VALUE!",
        ),
        (
            "A5",
            "=ROWS(MDURATION(39448,43831,0.08,0.09,G3:G4,0))",
            "#VALUE!",
        ),
        (
            "A6",
            "=ROWS(ACCRINT(39083,39264,39448,0.1,1000,G3:G4,0,TRUE))",
            "#VALUE!",
        ),
        // The explicit intersection on the range's rows is the scalar call (the I controls).
        ("B1", "=COUPDAYSNC(39448,@G1:G2,2,0)", "180"),
        // Beside an array the range is every element's `#VALUE!`.
        ("A8", "=INDEX(COUPNUM({39448;39630},G1:G2,2),2)", "#VALUE!"),
        ("A9", "=ROWS(COUPNUM({39448;39630},G1:G2,2))", "2"),
    ]);
}

#[test]
fn mduration_reads_its_arguments_in_order() {
    // MDURATION reads every argument in order, as DURATION does: the settlement's `#N/A` wins over
    // the typed text yld (probes/fin-coupon-walk-bounds-c M8, N8; excel-graph, T-1844).
    check(&[
        ("A1", "=MDURATION(NA(),43831,0.08,\"x\",2)", "#N/A"),
        ("A2", "=DURATION(NA(),43831,0.08,\"x\",2)", "#N/A"),
    ]);
}

#[test]
fn a_settlement_before_the_calendar_is_num() {
    // probes/fin-coupon-walk-bounds A1-A13, A19, A20 and -b A11: a settlement below serial 1 is
    // #NUM! (below 0 for COUPPCD and the COUPDAYS functions, T-1844), so the schedule walk back from the maturity stays inside the calendar
    // (`COUPNUM(-1E15,..)` stepped about 10^6 quarters before T-1822).
    check(&[
        ("A1", "=COUPNUM(-1E15,43831,4)", "#NUM!"),
        ("A2", "=COUPNUM(0,43831,4)", "#NUM!"),
        ("A3", "=COUPNUM(-0.5,43831,2)", "#NUM!"),
        ("A4", "=COUPPCD(-1,43831,2)", "#NUM!"),
        ("A5", "=COUPDAYSNC(-1,43831,2)", "#NUM!"),
        ("A6", "=PRICE(0,43831,0.05,0.06,100,4)", "#NUM!"),
        ("A7", "=YIELD(-1,43831,0.05,90,100,2)", "#NUM!"),
        ("A8", "=MDURATION(-1,43831,0.05,0.06,2)", "#NUM!"),
        ("A9", "=COUPNUM(39448,43831,2)", "24"),
    ]);
}

#[test]
fn a_lifted_coupon_call_is_charged_for_its_schedule_walks() {
    // SPEC-0011 "Security rule for kernel work": the COUP* functions and ACCRINT are charged two
    // schedule walks per element, DURATION, MDURATION and PRICE four, each in proportion to the
    // element's period count (T-1846): short schedules lift far past the 1,024 / 512 elements
    // the longest schedule allows; financial.md "Coupon schedule work cap" (a DEVIATION).
    check(&[
        ("A1", "=ROWS(COUPNUM(SEQUENCE(1025)+39000,43831,2))", "1025"),
        (
            "A2",
            "=ROWS(COUPNUM(SEQUENCE(1025)*0+1,DATE(9999,12,31),4))",
            "#ERROR!",
        ),
        (
            "A3",
            "=ROWS(ACCRINT(39083,39264,SEQUENCE(1025)+39448,0.1,1000,2))",
            "1025",
        ),
        (
            "A4",
            "=ROWS(ACCRINT(1,39264,SEQUENCE(1025)*0+2958465,0.1,1000,4))",
            "#ERROR!",
        ),
        (
            "A5",
            "=ROWS(PRICE(SEQUENCE(513)+39000,43831,0.05,0.04,100,2))",
            "513",
        ),
        (
            "A6",
            "=ROWS(PRICE(SEQUENCE(513)*0+1,DATE(9999,12,31),0.05,0.06,100,4))",
            "#ERROR!",
        ),
        (
            "A7",
            "=ROWS(MDURATION(SEQUENCE(513)*0+1,DATE(9999,12,31),0.05,0.06,4))",
            "#ERROR!",
        ),
        (
            "A8",
            "=ROWS(DURATION(SEQUENCE(4096)+39000,43831,0.05,0.06,2))",
            "4096",
        ),
        // A text date may read as any date, so it is charged as the longest schedule.
        (
            "A9",
            "=ROWS(COUPNUM(IF(SEQUENCE(1025),\"2020-01-01\"),43831,4))",
            "#ERROR!",
        ),
    ]);
}
