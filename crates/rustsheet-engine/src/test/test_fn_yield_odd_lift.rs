#![allow(clippy::unwrap_used)]

//! YIELD, ODDFPRICE, ODDFYIELD, ODDLPRICE and ODDLYIELD on the SPEC-0011 driver (T-1842): every
//! argument lifts over an array and refuses a multi-cell range (`LiftArrays`), each element the
//! scalar call, and a lifted call is charged its solver and schedule work. Evidence:
//! probes/fin-argument-modes, probes/fin-remaining-later-argument-ranges and
//! probes/fin-yield-odd-lift (docs/excel-semantics/financial.md "Financial function argument
//! modes", "Yield and odd-coupon work cap").

use crate::test::util::new_empty_model;

/// Evaluates `cells` on a sheet whose G1:G2 is 39448 (a settlement), G3:G4 is 2 (a frequency),
/// G5:G6 is 43831 (a maturity) and M1 is 1E300, and checks each cell's text.
fn check(cells: &[(&str, &str, &str)]) {
    let mut model = new_empty_model();
    for (cell, value) in [
        ("G1", "39448"),
        ("G2", "39448"),
        ("G3", "2"),
        ("G4", "2"),
        ("G5", "43831"),
        ("G6", "43831"),
        ("M1", "1E300"),
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
    // probes/fin-argument-modes H190-H230, H344-H351; fin-yield-odd-lift E3, E4.
    check(&[
        (
            "A1",
            "=ROWS(YIELD({39448;39448},43831,0.0575,95.04,100,2,0))",
            "2",
        ),
        (
            "A2",
            "=ROWS(YIELD(39448,43831,0.0575,95.04,100,2,{0;0}))",
            "2",
        ),
        (
            "A3",
            "=ROWS(ODDFPRICE(39448,43831,{39083;39083},39630,0.08,0.09,100,2,0))",
            "2",
        ),
        (
            "A4",
            "=ROWS(ODDFPRICE(39448,43831,39083,39630,0.08,0.09,100,2,{0;0}))",
            "2",
        ),
        (
            "A5",
            "=ROWS(ODDFYIELD(39448,43831,39083,{39630;39630},0.08,95,100,2,0))",
            "2",
        ),
        (
            "A6",
            "=ROWS(ODDFYIELD(39448,43831,39083,39630,0.08,95,100,{2;2},0))",
            "2",
        ),
        (
            "A7",
            "=ROWS(ODDLPRICE(39448,39630,39264,0.08,{0.09;0.09},100,2,0))",
            "2",
        ),
        (
            "A8",
            "=ROWS(ODDLYIELD(39448,39630,{39264;39264},0.08,99,100,2,0))",
            "2",
        ),
        (
            "A9",
            "=ROWS(ODDLYIELD(39448,39630,39264,0.08,99,100,2,{0;0}))",
            "2",
        ),
        // A row against a column broadcasts to a grid.
        (
            "A10",
            "=COLUMNS(YIELD(39448,{43831,44196},0.0575,{95.04;98},100,2,0))",
            "2",
        ),
        (
            "A11",
            "=ROWS(YIELD(39448,{43831,44196},0.0575,{95.04;98},100,2,0))",
            "2",
        ),
    ]);
}

#[test]
fn each_element_is_the_scalar_call() {
    // probes/fin-yield-odd-lift A1-A10, B1-B6, C1-C8, E5.
    check(&[
        (
            "A1",
            "=INDEX(YIELD(39448,43831,0.0575,{95.04;98},100,2,0),2)=YIELD(39448,43831,0.0575,98,100,2,0)",
            "TRUE",
        ),
        (
            "A2",
            "=INDEX(ODDFPRICE(39448,43831,39083,39630,0.08,{0.09;0.1},100,2,0),2)=ODDFPRICE(39448,43831,39083,39630,0.08,0.1,100,2,0)",
            "TRUE",
        ),
        (
            "A3",
            "=INDEX(ODDFYIELD(39448,43831,39083,39630,0.08,{95;97},100,2,0),2)=ODDFYIELD(39448,43831,39083,39630,0.08,97,100,2,0)",
            "TRUE",
        ),
        (
            "A4",
            "=INDEX(ODDLPRICE(39448,39630,39264,0.08,{0.09;0.1},100,2,0),2)",
            "98.85714286",
        ),
        (
            "A5",
            "=INDEX(ODDLYIELD(39448,39630,39264,0.08,{99;98},100,2,0),2)",
            "0.117647059",
        ),
        (
            "A6",
            "=INDEX(YIELD(39448,{43831,44196},0.0575,{95.04;98},100,2,0),2,2)=YIELD(39448,44196,0.0575,98,100,2,0)",
            "TRUE",
        ),
        // An element error stays in its position, the others are computed.
        (
            "B1",
            "=INDEX(YIELD(39448,43831,0.0575,95.04,100,{2;3},0),2)",
            "#NUM!",
        ),
        (
            "B2",
            "=INDEX(YIELD(39448,43831,0.0575,95.04,100,{2;3},0),1)=YIELD(39448,43831,0.0575,95.04,100,2,0)",
            "TRUE",
        ),
        (
            "B3",
            "=INDEX(ODDFPRICE(39448,43831,39083,39630,0.08,{\"x\";0.09},100,2,0),1)",
            "#VALUE!",
        ),
        (
            "B4",
            "=INDEX(ODDLYIELD(39448,39630,39264,0.08,{99;-1},100,2,0),2)",
            "#NUM!",
        ),
        (
            "B5",
            "=INDEX(ODDFYIELD(39448,43831,39083,39630,0.08,95,100,2,{0;5}),2)",
            "#NUM!",
        ),
        (
            "B6",
            "=INDEX(ODDLPRICE({39448;39700},39630,39264,0.08,0.09,100,2,0),2)",
            "#NUM!",
        ),
        // A boolean element is #VALUE!, as a typed boolean is.
        (
            "C1",
            "=INDEX(YIELD(39448,43831,0.0575,95.04,100,2,{0;TRUE}),2)",
            "#VALUE!",
        ),
        (
            "C2",
            "=INDEX(ODDFYIELD(39448,43831,39083,39630,0.08,95,100,2,{0;TRUE}),2)",
            "#VALUE!",
        ),
        (
            "C3",
            "=INDEX(ODDLPRICE(39448,39630,39264,0.08,0.09,100,2,{0;TRUE}),2)",
            "#VALUE!",
        ),
    ]);
}

#[test]
fn a_range_is_refused_in_every_argument() {
    // probes/fin-yield-odd-lift H5-I13, D1-D4, E1, E2; fin-remaining-later-argument-ranges
    // H105-H183 (on the range's own rows here: G1:G6 beside rows 1-6).
    check(&[
        (
            "A1",
            "=ROWS(YIELD(G1:G2,43831,0.0575,95.04,100,2,0))",
            "#VALUE!",
        ),
        (
            "A2",
            "=ROWS(YIELD(39448,43831,0.0575,95.04,100,G3:G4,0))",
            "#VALUE!",
        ),
        (
            "A3",
            "=ROWS(ODDFPRICE(G1:G2,43831,39083,39630,0.08,0.09,100,2,0))",
            "#VALUE!",
        ),
        (
            "A4",
            "=ROWS(ODDFYIELD(39448,G5:G6,39083,39630,0.08,95,100,2,0))",
            "#VALUE!",
        ),
        (
            "A5",
            "=ROWS(ODDLPRICE(39448,39630,39264,0.08,0.09,100,G3:G4,0))",
            "#VALUE!",
        ),
        (
            "A6",
            "=ROWS(ODDLYIELD(G1:G2,39630,39264,0.08,99,100,2,0))",
            "#VALUE!",
        ),
        // The explicit intersection on the range's rows is the scalar call (the I controls).
        (
            "B1",
            "=ODDLPRICE(@G1:G2,39630,39264,0.08,0.09,100,2,0)",
            "99.3492823",
        ),
        (
            "B3",
            "=ODDLYIELD(39448,39630,39264,0.08,99,100,@G3:G4,0)",
            "0.097087379",
        ),
        // An error scalar beside the range follows the kernel's check order.
        ("C1", "=YIELD(NA(),G5:G6,0.0575,95.04,100,2,0)", "#N/A"),
        ("C2", "=YIELD(G1:G2,NA(),0.0575,95.04,100,2,0)", "#VALUE!"),
        (
            "C3",
            "=ODDLYIELD(G1:G2,39630,39264,0.08,99,100,NA(),0)",
            "#VALUE!",
        ),
        (
            "C4",
            "=ODDFPRICE(39448,43831,39083,39630,NA(),0.09,100,G3:G4,0)",
            "#N/A",
        ),
        // Beside an array the range is every element's `#VALUE!`.
        (
            "D1",
            "=INDEX(ODDLPRICE({39448;39448},G5:G6,39264,0.08,0.09,100,2,0),2)",
            "#VALUE!",
        ),
        (
            "D2",
            "=ROWS(ODDLPRICE({39448;39448},G5:G6,39264,0.08,0.09,100,2,0))",
            "2",
        ),
    ]);
}

#[test]
fn arguments_far_out_of_range_are_errors_without_a_panic() {
    // probes/fin-yield-odd-lift L1-L4, L6-L8 (M1 is 1E300), with the values in cells and as
    // elements. L5, a basis of -1E300, is #NUM! in Excel and reads as basis 0 here (T-1870).
    check(&[
        ("A1", "=YIELD(M1,43831,0.0575,95.04,100,2,0)", "#NUM!"),
        (
            "A2",
            "=ODDFPRICE(39448,43831,39083,39630,0.08,0.09,100,M1,0)",
            "#NUM!",
        ),
        ("A3", "=ODDLYIELD(39448,39630,39264,0.08,M1,100,2,0)", "-2"),
        (
            "A4",
            "=ODDFYIELD(39448,43831,39083,39630,0.08,95,100,2,M1)",
            "#NUM!",
        ),
        (
            "A6",
            "=ODDLPRICE(-M1,39630,39264,0.08,0.09,100,2,0)",
            "#NUM!",
        ),
        (
            "A7",
            "=INDEX(ODDFYIELD({39448;-1E300},43831,39083,39630,0.08,95,100,2,0),2)",
            "#NUM!",
        ),
        (
            "A8",
            "=ROWS(ODDFPRICE(M1,{43831;-1E300},M1,-M1,M1,-M1,M1,M1,M1))",
            "2",
        ),
        (
            "A9",
            "=ROWS(YIELD({1E300;-1E300},M1,-M1,M1,-M1,{4;1E300},-M1))",
            "2",
        ),
    ]);
}

#[test]
fn a_lifted_call_is_charged_for_its_solver_and_schedule_work() {
    // SPEC-0011 "Security rule for kernel work": YIELD and ODDFYIELD elements are charged 2^24
    // (8 elements), ODDFPRICE eight schedule walks (256) and ODDLPRICE and ODDLYIELD two (1,024);
    // financial.md "Yield and odd-coupon work cap" (a DEVIATION). The ninth YIELD element makes
    // the call #ERROR! before any element runs, whatever the schedule.
    check(&[
        (
            "A1",
            "=ROWS(YIELD(SEQUENCE(8)+39000,43831,0.05,95,100,2))",
            "8",
        ),
        (
            "A2",
            "=ROWS(YIELD(SEQUENCE(9)+39000,43831,0.05,95,100,2))",
            "#ERROR!",
        ),
        (
            "A3",
            "=ROWS(YIELD(SEQUENCE(9)*0+1,2958465,0.05,1E300,100,4,1))",
            "#ERROR!",
        ),
        (
            "A4",
            "=ROWS(ODDFYIELD(SEQUENCE(9)*0+39448,43831,39083,39630,0.08,95,100,2,0))",
            "#ERROR!",
        ),
        (
            "A5",
            "=ROWS(ODDFPRICE(SEQUENCE(256)*0+39448,43831,39083,39630,0.08,0.09,100,2,0))",
            "256",
        ),
        (
            "A6",
            "=ROWS(ODDFPRICE(SEQUENCE(257)*0+39448,43831,39083,39630,0.08,0.09,100,2,0))",
            "#ERROR!",
        ),
        (
            "A7",
            "=ROWS(ODDLPRICE(SEQUENCE(1024)*0+39448,39630,39264,0.08,0.09,100,2,0))",
            "1024",
        ),
        (
            "A8",
            "=ROWS(ODDLYIELD(SEQUENCE(1025)*0+39448,39630,39264,0.08,99,100,2,0))",
            "#ERROR!",
        ),
        // An unlifted call is never charged.
        (
            "A9",
            "=ODDLYIELD(39448,39630,39264,0.08,99,100,2,0)",
            "0.097087379",
        ),
    ]);
}
