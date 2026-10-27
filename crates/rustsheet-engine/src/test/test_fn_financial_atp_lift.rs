#![allow(clippy::unwrap_used)]

//! The coupon-schedule Analysis `ToolPak` functions on the SPEC-0011 driver (T-1822): ACCRINT,
//! the COUP* family, DURATION, MDURATION and PRICE lift over arrays in every argument and refuse
//! a multi-cell range there (`LiftArrays`). Evidence: probes/fin-argument-modes,
//! probes/fin-remaining-later-argument-ranges, probes/scalar-lift-open-a Y1-Y7 and
//! probes/fin-coupon-walk-bounds (docs/excel-semantics/financial.md "Financial function argument
//! modes", "Coupon settlement before the calendar").

use crate::cell::CellValue;
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
fn bond_functions_check_their_arguments_in_excel_order() {
    // probes/bond-argument-check-order (excel-graph, T-2482): PRICE and YIELD read the basis
    // first; a date off the calendar is #NUM! as it is read; DURATION checks its frequency before
    // it reads the basis; the date order and the value checks come after every read.
    check(&[
        // An out-of-domain argument before an error one.
        ("A1", "=PRICE(-1,43831,NA(),0.065,100,2,0)", "#NUM!"),
        ("A2", "=YIELD(2958466,\"x\",0.0575,95.04,100,2,0)", "#NUM!"),
        ("A3", "=DURATION(39448,-1,0.08,NA(),2,1)", "#NUM!"),
        ("A4", "=MDURATION(39448,43831,0.08,0.09,3,NA())", "#NUM!"),
        ("A5", "=PRICE(NA(),43831,0.0575,0.065,100,2,5)", "#NUM!"),
        (
            "A6",
            "=YIELD(39448,43831,0.0575,95.04,100,\"x\",5)",
            "#NUM!",
        ),
        // An error argument before an out-of-domain one.
        ("B1", "=PRICE(39448,39000,\"x\",0.065,100,2,0)", "#VALUE!"),
        ("B2", "=YIELD(39448,43831,0.0575,0,100,2,NA())", "#N/A"),
        ("B3", "=DURATION(39448,43831,-0.05,NA(),2,1)", "#N/A"),
        ("B4", "=MDURATION(39448,43831,0.08,0.09,NA(),5)", "#N/A"),
        ("B5", "=PRICE(-1,43831,0.0575,0.065,100,2,NA())", "#N/A"),
        ("B6", "=YIELD(39448,43831,0.0575,95.04,100,3,NA())", "#N/A"),
        // Settlement 0 passes the read; it is #NUM! only after the later arguments.
        ("C1", "=PRICE(0,43831,NA(),0.065,100,2,0)", "#N/A"),
        ("C2", "=DURATION(0,43831,0.08,0.09,2,1)", "#NUM!"),
    ]);
}

#[test]
fn odd_coupon_functions_check_their_arguments_in_excel_order() {
    // probes/odd-bond-argument-check-order (excel-graph, T-2483): a date below 0 or past
    // 9999-12-31 is #NUM! as it is read, and the frequency before the basis is read; the date
    // order and the value checks come after every read.
    check(&[
        // An out-of-domain argument before an error one.
        (
            "A1",
            "=ODDFPRICE(2958466,NA(),39300,39630,0.0575,0.065,100,2,0)",
            "#NUM!",
        ),
        (
            "A2",
            "=ODDFYIELD(39448,43831,39300,-1,0.0575,\"x\",100,2,0)",
            "#NUM!",
        ),
        (
            "A3",
            "=ODDLPRICE(43000,43200,2958466,0.0575,0.065,100,2,NA())",
            "#NUM!",
        ),
        (
            "A4",
            "=ODDLYIELD(43000,43200,42917,0.0575,99,100,3,NA())",
            "#NUM!",
        ),
        // An error argument before an out-of-domain one.
        (
            "B1",
            "=ODDFPRICE(43832,43831,39300,39630,0.0575,0.065,NA(),2,0)",
            "#N/A",
        ),
        (
            "B2",
            "=ODDFYIELD(39448,43831,39700,39630,0.0575,95,100,2,\"x\")",
            "#VALUE!",
        ),
        (
            "B3",
            "=ODDLPRICE(43000,43200,42917,-0.05,0.065,100,2,TRUE)",
            "#VALUE!",
        ),
        (
            "B4",
            "=ODDLYIELD(NA(),43200,42917,0.0575,99,100,2,5)",
            "#N/A",
        ),
        // A date of 0, 0.5 or 2958465 passes the read.
        (
            "C1",
            "=ODDLPRICE(43000,2958465,42917,NA(),0.065,100,2,0)",
            "#N/A",
        ),
        (
            "C2",
            "=ODDFPRICE(0.5,43831,39300,39630,NA(),0.065,100,2,0)",
            "#N/A",
        ),
    ]);
}

#[test]
fn oddl_settlement_must_follow_the_last_coupon() {
    // probes/odd-bond-argument-check-order A909:A922 (excel-graph, T-2483): a settlement before
    // or on the last coupon is #NUM!, a day after it is priced; its error comes after the reads.
    check(&[
        (
            "A1",
            "=ODDLPRICE(42800,43200,42917,0.0575,0.065,100,2,0)",
            "#NUM!",
        ),
        (
            "A2",
            "=ODDLPRICE(42917,43200,42917,0.0575,0.065,100,2,0)",
            "#NUM!",
        ),
        (
            "A3",
            "=ODDLYIELD(42916,43200,42917,0.0575,99,100,2,0)",
            "#NUM!",
        ),
        (
            "A4",
            "=ODDLYIELD(42917,43200,42917,0.0575,99,100,2,0)",
            "#NUM!",
        ),
        (
            "A5",
            "=ODDLPRICE(42800,43200,42917,0.0575,0.065,NA(),2,0)",
            "#N/A",
        ),
    ]);
    let mut model = new_empty_model();
    model._set("A1", "=ODDLPRICE(42918,43200,42917,0.0575,0.065,100,2,0)");
    model._set("A2", "=ODDLYIELD(42918,43200,42917,0.0575,99,100,2,0)");
    model.evaluate();
    // Excel's values, to the bit (A912, A919).
    assert_eq!(
        model.get_cell_value_by_ref("Sheet1!A1"),
        Ok(CellValue::Number(99.447_751_426_837_48))
    );
    assert_eq!(
        model.get_cell_value_by_ref("Sheet1!A2"),
        Ok(CellValue::Number(0.071_149_773_826_044_15))
    );
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
        // A text is charged as the engine reads it (T-2879): a date text as its date, so these
        // one-period walks lift; a text that reads as no date is #VALUE! before any walk.
        (
            "A9",
            "=ROWS(COUPNUM(IF(SEQUENCE(1025),\"2020-01-01\"),43831,4))",
            "1025",
        ),
        (
            "A10",
            "=ROWS(COUPNUM(IF(SEQUENCE(1025),\"abc\"),2958465,4))",
            "1025",
        ),
    ]);
}
