#![allow(clippy::unwrap_used)]

use crate::{
    cell::CellValue,
    test::util::{check_cells, check_cells_with_numbers, new_empty_model},
};

#[test]
fn fn_arguments() {
    let mut model = new_empty_model();
    model._set("A1", "=PMT()");
    model._set("A2", "=PMT(1,1)");
    model._set("A3", "=PMT(1,1,1,1,1,1)");

    model._set("B1", "=FV()");
    model._set("B2", "=FV(1,1)");
    model._set("B3", "=FV(1,1,1,1,1,1)");

    model._set("C1", "=PV()");
    model._set("C2", "=PV(1,1)");
    model._set("C3", "=PV(1,1,1,1,1,1)");

    model._set("D1", "=NPER()");
    model._set("D2", "=NPER(1,1)");
    model._set("D3", "=NPER(1,1,1,1,1,1)");

    model._set("E1", "=RATE()");
    model._set("E2", "=RATE(1,1)");
    model._set("E3", "=RATE(1,1,1,1,1,1,1)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#ERROR!");
    assert_eq!(model._get_text("A2"), *"#ERROR!");
    assert_eq!(model._get_text("A3"), *"#ERROR!");

    assert_eq!(model._get_text("B1"), *"#ERROR!");
    assert_eq!(model._get_text("B2"), *"#ERROR!");
    assert_eq!(model._get_text("B3"), *"#ERROR!");

    assert_eq!(model._get_text("C1"), *"#ERROR!");
    assert_eq!(model._get_text("C2"), *"#ERROR!");
    assert_eq!(model._get_text("C3"), *"#ERROR!");

    assert_eq!(model._get_text("D1"), *"#ERROR!");
    assert_eq!(model._get_text("D2"), *"#ERROR!");
    assert_eq!(model._get_text("D3"), *"#ERROR!");

    assert_eq!(model._get_text("E1"), *"#ERROR!");
    assert_eq!(model._get_text("E2"), *"#ERROR!");
    assert_eq!(model._get_text("E3"), *"#ERROR!");
}

// RATE takes 3 to 6 arguments; the sixth is the guess (T-1050, probes-numeric/m9-fin-irr-rate).
#[test]
fn fn_rate_guess_argument() {
    let mut model = new_empty_model();
    model._set("A1", "=RATE(10,-100,800,0,0,0.1)");
    model._set("A2", "=RATE(10,-100,800)");
    // A different guess reaches the same root to 1e-6 (bits: fn_rate_secant_excel_bits).
    model._set("A3", "=RATE(10,-100,800,0,0,0.5)");
    model.evaluate();

    let Ok(CellValue::Number(with_guess)) = model.get_cell_value_by_ref("Sheet1!A1") else {
        panic!("RATE with a guess should be a number");
    };
    let Ok(CellValue::Number(without_guess)) = model.get_cell_value_by_ref("Sheet1!A2") else {
        panic!("RATE without a guess should be a number");
    };
    assert!((with_guess - without_guess).abs() < 1e-9);
    let Ok(CellValue::Number(other_guess)) = model.get_cell_value_by_ref("Sheet1!A3") else {
        panic!("RATE with another guess should be a number");
    };
    assert!((other_guess - without_guess).abs() < 1e-6);
}

/// RATE is a secant on the FV residual, capped at 200 steps (financial.md "RATE is a secant on
/// the FV residual", T-2005): Excel's bits for guess 0 (Newton divides by 0 there), a guess
/// sweep, the unconverged 200th iterate, `#NUM!` past the cap and for a root below -1.
#[test]
fn fn_rate_secant_excel_bits() {
    check_cells(
        &[],
        &[
            // probes/rate-empty-arguments A3, solver-iteration-traces R!C1, C8, C11, B15
            ("A1", "=RATE(48,-200,8000,0,1,0)", "0.008052981923885287"),
            (
                "A2",
                "=RATE(48,-200,8000,0,1,0.007)",
                "0.008052981923729007",
            ),
            ("A3", "=RATE(48,-200,8000,0,1,0.01)", "0.008052981924068529"),
            ("A4", "=RATE(15,0,-1,2)", "0.0472941228234414"),
            // probes/rate-step-cap A29, A30 (the 200th iterate, unconverged), A31
            (
                "A5",
                "=RATE(360,-599.55,100000,0,0,0.4558)",
                "0.00499999319311924",
            ),
            (
                "A6",
                "=RATE(360,-599.55,100000,0,0,0.4587)",
                "0.004999993194059272",
            ),
            ("A7", "=RATE(360,-599.55,100000,0,0,0.4615)", "#NUM!"),
            // probes-numeric/m9-fin-irr-rate A371: the secant's root is below -1
            ("A8", "=RATE(10,PMT(0.3,10,10000),10000)", "#NUM!"),
        ],
    );
}

/// RATE holds on a flat secant (financial.md "RATE holds on a flat secant", T-3348): near the
/// root of a long annuity |f| is rounding noise above 1e-7, the secant stalls with f1 = f0, and
/// Excel returns the held iterate when |f| is below the cap's 1e-4 (probes/t3348-rate-fv-nonzero).
#[test]
fn fn_rate_holds_on_a_flat_secant() {
    check_cells(
        &[],
        &[
            // probes/t3348-rate-fv-nonzero A1: decision 0188's example, a zero step at |f| 3.2e-7
            (
                "A1",
                "=RATE(239,-14841.27102594,344293.4825124,-155496.6323522,0)",
                "0.04310546919282547",
            ),
            // probes/t3348-rate-fv-nonzero A10: the same run at 2^8 the scale, |f| 8.3e-5
            (
                "A2",
                "=RATE(239,-14841.27102594*2^8,344293.4825124*2^8,-155496.6323522*2^8,0)",
                "0.04310546919282547",
            ),
            // probes/t3348-rate-fv-nonzero A182: f1 = f0 at two iterates 7e-18 apart, type 1
            (
                "A3",
                "=RATE(189,-90562.52118237,2416244.605283,-2145.356961869,1)",
                "0.0389104591457023",
            ),
            // probes/t3348-rate-fv-nonzero A99: held at |f| 1.6e-4, over the cap's 1e-4
            ("A4", "=RATE(234,2469.37,-28911.03,-549000.41,1)", "#NUM!"),
            // probes/rate-zero-residual-guess A1: flat on a residual of exactly 0
            ("A5", "=RATE(1,0,0,0)", "#NUM!"),
        ],
    );
}

/// RATE's secant runs on past an iterate at or below -1 and only its result must be above -1
/// (financial.md "RATE holds on a flat secant and runs on below -1", T-3348).
#[test]
fn fn_rate_runs_on_below_minus_one() {
    check_cells(
        &[],
        &[
            // probes-numeric/t3348-rate-gate-a-sweep A171: the first step lands at -1.48
            (
                "A1",
                "=RATE(5,1650044.83915129,-2172916.57215492,294.348120976469,0)",
                "0.7069774195050786",
            ),
            // probes/t3348-rate-below-minus-one A21, A28: back above -1
            (
                "A2",
                "=RATE(3,3735.938842325,-6629.520726773,4839.177771065,1)",
                "1.2149215363749337",
            ),
            (
                "A3",
                "=RATE(12,7.219302529839,-62.09095397375,-23.2320462273,1,-0.5)",
                "0.00667837581646823",
            ),
            // probes/t3348-rate-below-minus-one A3: converges below -1
            (
                "A4",
                "=RATE(4,-29.08640117694,104.8028437104,1.6496936838,0,-0.5)",
                "#NUM!",
            ),
        ],
    );
}

/// IRR's NPV takes Excel's twice-rounded `*` and `+` (financial.md "IRR is a secant in d = 1 - v",
/// T-3205): in plain doubles both calls stop one secant iterate early, 16 ulp off.
#[test]
fn fn_irr_npv_rounds_twice() {
    check_cells(
        &[],
        &[
            // probes-numeric/m9-fin-irr-rate Calc!A214
            (
                "A1",
                "=IRR({-1000,230,200,125,200,10},5)",
                "-0.10150067605359347",
            ),
            // gen-m9/67a4c03afd63f988 Calc!A395 (its flows as a literal)
            ("A2", "=IRR({0.5,0.1,-0.5},0.5)", "-0.09501243788791103"),
            // gen-m9/9d9ceba257f8001b Calc!A85 (its Data!A8 is blank, which NPV skips)
            (
                "A3",
                "=NPV(1E-10,{-100,50,60,0,0},{0,0,0})",
                "9.999999981999995",
            ),
        ],
    );
}

#[test]
fn fn_impmt_ppmt_arguments() {
    let mut model = new_empty_model();
    model._set("A1", "=IPMT()");
    model._set("A2", "=IPMT(1,1,1)");
    model._set("A3", "=IPMT(1,1,1,1,1,1,1)");

    model._set("B1", "=PPMT()");
    model._set("B2", "=PPMT(1,1,1)");
    model._set("B3", "=PPMT(1,1,1,1,1,1,1)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#ERROR!");
    assert_eq!(model._get_text("A2"), *"#ERROR!");
    assert_eq!(model._get_text("A3"), *"#ERROR!");

    assert_eq!(model._get_text("B1"), *"#ERROR!");
    assert_eq!(model._get_text("B2"), *"#ERROR!");
    assert_eq!(model._get_text("B3"), *"#ERROR!");
}

#[test]
fn fn_irr_npv_arguments() {
    let mut model = new_empty_model();
    model._set("A1", "=NPV()");
    model._set("A2", "=NPV(1,1)");

    model._set("C1", "-2"); // v0
    model._set("C2", "5"); // v1
    model._set("B1", "=IRR()");
    model._set("B3", "=IRR(1, 2, 3, 4)");
    // r such that v0 + v1/(1+r) = 0
    // r = -v1/v0 - 1
    model._set("B4", "=IRR(C1:C2)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#ERROR!");
    assert_eq!(model._get_text("A2"), *"$0.50");

    assert_eq!(model._get_text("B1"), *"#ERROR!");
    assert_eq!(model._get_text("B3"), *"#ERROR!");
    // r = 5/2-1 = 1.5
    assert_eq!(model._get_text("B4"), *"150%");
}

#[test]
fn fn_mirr() {
    let mut model = new_empty_model();
    model._set("A2", "-120000");
    model._set("A3", "39000");
    model._set("A4", "30000");
    model._set("A5", "21000");
    model._set("A6", "37000");
    model._set("A7", "46000");
    model._set("A8", "0.1");
    model._set("A9", "0.12");

    model._set("B1", "=MIRR(A2:A7, A8, A9)");
    model._set("B2", "=MIRR(A2:A5, A8, A9)");

    model.evaluate();
    assert_eq!(
        model.get_cell_value_by_ref("Sheet1!B1"),
        Ok(CellValue::Number(0.1260941303659051))
    );
    assert_eq!(model._get_text("B1"), "13%");
    assert_eq!(model._get_text("B2"), "-5%");
}

#[test]
fn fn_mirr_has_excels_evaluation_order() {
    // T-1687: period-0 sums with running powers, `P * (1+rr)^n / (1+rr)`, Excel's `^` for the root
    // (probes-numeric/m9-fin-mirr A254, A13, A66, A146; docs/excel-semantics/financial.md
    // "MIRR's evaluation order").
    let mut model = new_empty_model();
    let cases = [
        ("A1", "=MIRR({-100,120},0,0.03)", 0.19999999999999973),
        ("A2", "=MIRR({-3,7},0.1,0.12)", 1.333333333333333),
        (
            "A3",
            "=MIRR({-954,-1515,1291,-678,-46,-1588,-17,-216,-70},0.07,1)",
            0.44791438357999236,
        ),
        (
            "A4",
            "=MIRR({-396,992,-678,-1340,-1384,1958,-1245,-384,-775,220},0,-0.5)",
            -0.274291846946638,
        ),
    ];
    for (cell, formula, _) in cases {
        model._set(cell, formula);
    }
    model.evaluate();
    for (cell, formula, expected) in cases {
        assert_eq!(
            model.get_cell_value_by_ref(&format!("Sheet1!{cell}")),
            Ok(CellValue::Number(expected)),
            "{formula}"
        );
    }
}

#[test]
fn fn_mirr_div_0() {
    // A finance rate of -1 divides a later negative flow by a zero power: #DIV/0! in Excel
    // (ironcalc-calc MIRR.xlsx E8; probes/fin-mirr-edges A12, A14)
    let mut model = new_empty_model();
    model._set("A2", "-30");
    model._set("A3", "-20");
    model._set("A4", "-10");
    model._set("A5", "5");
    model._set("A6", "5");
    model._set("A7", "5");
    model._set("A8", "-1");
    model._set("A9", "2");

    model._set("B1", "=MIRR(A2:A7, A8, A9)");

    model.evaluate();
    assert_eq!(
        model.get_cell_value_by_ref("Sheet1!B1"),
        Ok(CellValue::String("#DIV/0!".to_string()))
    );
}

#[test]
fn fn_ispmt() {
    let mut model = new_empty_model();
    model._set("A1", "1"); // rate
    model._set("A2", "2"); // per
    model._set("A3", "5"); // nper
    model._set("A4", "4"); // pv

    model._set("B1", "=ISPMT(A1, A2, A3, A4)");
    model._set("B2", "=ISPMT(A1, A2, A3, A4, 1)");
    model._set("B3", "=ISPMT(A1, A2, A3)");

    model.evaluate();

    assert_eq!(model._get_text("B1"), "-2.4");
    assert_eq!(model._get_text("B2"), *"#ERROR!");
    assert_eq!(model._get_text("B3"), *"#ERROR!");
}

#[test]
fn fn_rri() {
    let mut model = new_empty_model();
    model._set("A1", "1"); // nper
    model._set("A2", "2"); // pv
    model._set("A3", "3"); // fv

    model._set("B1", "=RRI(A1, A2, A3)");
    model._set("B2", "=RRI(A1, A2)");
    model._set("B3", "=RRI(A1, A2, A3, 1)");

    model.evaluate();

    assert_eq!(model._get_text("B1"), "0.5");
    assert_eq!(model._get_text("B2"), *"#ERROR!");
    assert_eq!(model._get_text("B3"), *"#ERROR!");
}

#[test]
fn fn_sln() {
    let mut model = new_empty_model();
    model._set("A1", "1"); // cost
    model._set("A2", "2"); // salvage
    model._set("A3", "3"); // life

    model._set("B1", "=SLN(A1, A2, A3)");
    model._set("B2", "=SLN(A1, A2)");
    model._set("B3", "=SLN(A1, A2, A3, 1)");

    model.evaluate();

    assert_eq!(
        model.get_cell_value_by_ref("Sheet1!B1"),
        Ok(CellValue::Number(-1.0 / 3.0))
    );
    assert_eq!(model._get_text("B1"), "-$0.33");
    assert_eq!(model._get_text("B2"), *"#ERROR!");
    assert_eq!(model._get_text("B3"), *"#ERROR!");
}

#[test]
fn fn_syd() {
    let mut model = new_empty_model();
    model._set("A1", "100"); // cost
    model._set("A2", "5"); // salvage
    model._set("A3", "20"); // life
    model._set("A4", "10"); // periods

    model._set("B1", "=SYD(A1, A2, A3, A4)");
    model._set("B2", "=SYD(A1, A2, A3)");
    model._set("B3", "=SYD(A1, A2, A3, A4, 1)");

    model.evaluate();

    assert_eq!(
        model.get_cell_value_by_ref("Sheet1!B1"),
        Ok(CellValue::Number(4.976190476190476))
    );
    assert_eq!(model._get_text("B1"), "$4.98");
    assert_eq!(model._get_text("B2"), *"#ERROR!");
    assert_eq!(model._get_text("B3"), *"#ERROR!");
}

#[test]
fn fn_effect() {
    let mut model = new_empty_model();
    model._set("A1", "2"); // rate
    model._set("A2", "1"); // periods

    model._set("B1", "=EFFECT(A1, A2)");
    model._set("B2", "=EFFECT(A1)");
    model._set("B3", "=EFFECT(A1, A2, A3)");

    model.evaluate();

    assert_eq!(model._get_text("B1"), "2");
    assert_eq!(model._get_text("B2"), *"#ERROR!");
    assert_eq!(model._get_text("B3"), *"#ERROR!");
}

#[test]
fn fn_nominal() {
    let mut model = new_empty_model();
    model._set("A1", "2"); // rate
    model._set("A2", "1"); // periods

    model._set("B1", "=NOMINAL(A1, A2)");
    model._set("B2", "=NOMINAL(A1)");
    model._set("B3", "=NOMINAL(A1, A2, A3)");

    model.evaluate();

    assert_eq!(model._get_text("B1"), "2");
    assert_eq!(model._get_text("B2"), *"#ERROR!");
    assert_eq!(model._get_text("B3"), *"#ERROR!");
}

// A life past i64 saturates the period count; the last-period check must not overflow (nightly
// fuzz crash on `DB(6912,0,1E+300,2,6)`, T-2570).
#[test]
fn fn_db_huge_life_does_not_overflow() {
    // A life past i32::MAX is #NUM! from period 1 on (financial.md "DB and SYD argument
    // domains"; gen-m9, probes/fin-syd-db-domain G5).
    let mut model = new_empty_model();
    model._set("A1", "=DB(6912,0,1E+300,2,6)");
    model._set("A2", "=DB(6912,1,1E+300,3)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#NUM!");
    assert_eq!(model._get_text("A2"), "#NUM!");
}

#[test]
fn fn_db() {
    let mut model = new_empty_model();
    model._set("A2", "$1,000,000"); // cost
    model._set("A3", "$100,000"); // salvage
    model._set("A4", "6"); // life

    model._set("B1", "=DB(A2,A3,A4,1,7)");
    model._set("B2", "=DB(A2,A3,A4,2,7)");
    model._set("B3", "=DB(A2,A3,A4,3,7)");
    model._set("B4", "=DB(A2,A3,A4,4,7)");
    model._set("B5", "=DB(A2,A3,A4,5,7)");
    model._set("B6", "=DB(A2,A3,A4,6,7)");
    model._set("B7", "=DB(A2,A3,A4,7,7)");

    model._set("C1", "=DB(A2,A3,A4,7,7,1)");
    model._set("C2", "=DB(A2,A3,A4)");

    model.evaluate();

    assert_eq!(model._get_text("B1"), "$186,083.33");
    assert_eq!(model._get_text("B2"), "$259,639.42");
    assert_eq!(model._get_text("B3"), "$176,814.44");
    assert_eq!(model._get_text("B4"), "$120,410.64");
    assert_eq!(model._get_text("B5"), "$81,999.64");
    assert_eq!(model._get_text("B6"), "$55,841.76");
    assert_eq!(model._get_text("B7"), "$15,845.10");

    assert_eq!(model._get_text("C1"), *"#ERROR!");
    assert_eq!(model._get_text("C2"), *"#ERROR!");
}

#[test]
fn fn_ddb() {
    let mut model = new_empty_model();
    model._set("A2", "$2,400"); // cost
    model._set("A3", "$300"); // salvage
    model._set("A4", "10"); // life

    model._set("B1", "=DDB(A2,A3,A4*365,1)");
    model._set("B2", "=DDB(A2,A3,A4*12,1,2)");
    model._set("B3", "=DDB(A2,A3,A4,1,2)");
    model._set("B4", "=DDB(A2,A3,A4,2,1.5)");
    model._set("B5", "=DDB(A2,A3,A4,10)");

    model._set("C1", "=DB(A2,A3,A4,7,7,1)");
    model._set("C2", "=DB(A2,A3,A4)");

    model.evaluate();

    assert_eq!(model._get_text("B1"), "$1.32");
    assert_eq!(model._get_text("B2"), "$40.00");
    assert_eq!(model._get_text("B3"), "$480.00");
    assert_eq!(model._get_text("B4"), "$306.00");
    assert_eq!(model._get_text("B5"), "$22.12");

    assert_eq!(model._get_text("C1"), *"#ERROR!");
    assert_eq!(model._get_text("C2"), *"#ERROR!");
}

#[test]
fn fn_tbilleq() {
    let mut model = new_empty_model();
    model._set("A2", "=DATE(2008, 3, 31)"); // settlement date
    model._set("A3", "=DATE(2008, 6, 1)"); // maturity date
    model._set("A4", "9.14%");

    model._set("B1", "=TBILLEQ(A2,A3,A4)");

    model._set("C1", "=TBILLEQ(A2,A3)");
    model._set("C2", "=TBILLEQ(A2,A3,A4,1)");

    model.evaluate();

    assert_eq!(model._get_text("B1"), "9.42%");

    assert_eq!(model._get_text("C1"), *"#ERROR!");
    assert_eq!(model._get_text("C2"), *"#ERROR!");
}

#[test]
fn fn_tbillprice() {
    let mut model = new_empty_model();
    model._set("A2", "=DATE(2008, 3, 31)"); // settlement date
    model._set("A3", "=DATE(2008, 6, 1)"); // maturity date
    model._set("A4", "9.0%");

    model._set("B1", "=TBILLPRICE(A2,A3,A4)");

    model._set("C1", "=TBILLPRICE(A2,A3)");
    model._set("C2", "=TBILLPRICE(A2,A3,A4,1)");

    model.evaluate();

    assert_eq!(model._get_text("B1"), "$98.45");

    assert_eq!(model._get_text("C1"), *"#ERROR!");
    assert_eq!(model._get_text("C2"), *"#ERROR!");
}

#[test]
fn fn_tbillyield() {
    let mut model = new_empty_model();
    model._set("A2", "=DATE(2008, 3, 31)"); // settlement date
    model._set("A3", "=DATE(2008, 6, 1)"); // maturity date
    model._set("A4", "$98.45");

    model._set("B1", "=TBILLYIELD(A2,A3,A4)");

    model._set("C1", "=TBILLYIELD(A2,A3)");
    model._set("C2", "=TBILLYIELD(A2,A3,A4,1)");

    model.evaluate();

    assert_eq!(model._get_text("B1"), "9.14%");

    assert_eq!(model._get_text("C1"), *"#ERROR!");
    assert_eq!(model._get_text("C2"), *"#ERROR!");
}

// The T-bills floor both dates, so a settlement from 0 to below 1 is serial 0, a valid date
// (probes/tbill-settlement-below-one, gen-m9 0a957119475760c5 A309, e8d55ec393460bb7 A287,
// ba4ddb817a8e700e A135; T-2487).
#[test]
fn tbills_take_a_settlement_below_one() {
    check_cells(
        &[],
        &[
            ("A1", "=TBILLPRICE(1E-15,1,100)", "72.22222222222221"),
            ("A2", "=TBILLPRICE(0.3,1,0.1)", "99.97222222222221"),
            ("A3", "=TBILLPRICE(0.5,1.9,0.1)", "99.97222222222221"),
            ("A4", "=TBILLPRICE(0.5,0.9,0.1)", "#NUM!"),
            ("A5", "=TBILLPRICE(0,366,0.1)", "89.83333333333333"),
            ("A6", "=TBILLPRICE(0,367,0.1)", "#NUM!"),
            (
                "A7",
                "=TBILLPRICE(39000.7,39001.2,0.1)",
                "99.97222222222221",
            ),
            ("A8", "=TBILLPRICE(60,426,0.1)", "89.83333333333333"),
            ("A9", "=TBILLPRICE(61,427,0.1)", "#NUM!"),
            ("A10", "=TBILLPRICE(45351,45717,0.1)", "89.83333333333333"),
            ("B1", "=TBILLYIELD(1E-15,1,98)", "7.346938775510203"),
            ("B2", "=TBILLYIELD(0.3,1,98)", "7.346938775510203"),
            ("B3", "=TBILLYIELD(0.5,1,98)", "7.346938775510203"),
            ("B4", "=TBILLYIELD(0,366,98)", "0.020073603211776512"),
            ("B5", "=TBILLYIELD(0,367,98)", "#NUM!"),
            ("B6", "=TBILLYIELD(1E-15,100,1E+300)", "-3.6"),
            ("C1", "=TBILLEQ(1E-15,1,0.05/12)", "0.004224585932707554"),
            ("C2", "=TBILLEQ(0.3,1,0.05/12)", "0.004224585932707554"),
            ("C3", "=TBILLEQ(0.5,1,0.05/12)", "0.004224585932707554"),
            ("C4", "=TBILLEQ(0,200,0.1)", "0.10685341885245037"),
            ("C5", "=TBILLEQ(0,366,0.1)", "0.11013984535998977"),
        ],
    );
}

#[test]
fn fn_dollarde() {
    let mut model = new_empty_model();
    model._set("A1", "=DOLLARDE(1.02, 16)");
    model._set("A2", "=DOLLARDE(1.1, 32)");

    model._set("C1", "=DOLLARDE(1.1)");
    model._set("C2", "=DOLLARDE(1.1, 32, 1)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), "1.125");
    assert_eq!(model._get_text("A2"), "1.3125");

    assert_eq!(model._get_text("C1"), *"#ERROR!");
    assert_eq!(model._get_text("C2"), *"#ERROR!");
}

#[test]
fn fn_dollarfr() {
    let mut model = new_empty_model();
    model._set("A1", "=DOLLARFR(1.125,16)");
    model._set("A2", "=DOLLARFR(1.125,32)");

    model._set("C1", "=DOLLARFR(1.1)");
    model._set("C2", "=DOLLARFR(1.1, 32, 1)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), "1.02");
    assert_eq!(model._get_text("A2"), "1.04");

    assert_eq!(model._get_text("C1"), *"#ERROR!");
    assert_eq!(model._get_text("C2"), *"#ERROR!");
}

#[test]
fn fn_cumipmt() {
    let mut model = new_empty_model();
    model._set("A2", "9%"); // annual interest rate
    model._set("A3", "30"); // years of the load
    model._set("A4", "$125,000"); // present value

    model._set("B1", "=CUMIPMT(A2/12,A3*12,A4,13,24,0)");
    model._set("B2", "=CUMIPMT(A2/12,A3*12,A4,1,1,0)");

    model._set("C1", "=CUMIPMT(A2/12,A3*12,A4,1,1,0,1)");
    model._set("C2", "=CUMIPMT(A2/12,A3*12,A4,1,1)");

    model.evaluate();

    assert_eq!(model._get_text("B1"), "-$11,135.23");
    assert_eq!(model._get_text("B2"), "-$937.50");

    assert_eq!(model._get_text("C1"), *"#ERROR!");
    assert_eq!(model._get_text("C2"), *"#ERROR!");
}

#[test]
fn fn_cumprinc() {
    let mut model = new_empty_model();
    model._set("A2", "9%"); // annual interest rate
    model._set("A3", "30"); // years of the load
    model._set("A4", "$125,000"); // present value

    model._set("B1", "=CUMPRINC(A2/12,A3*12,A4,13,24,0)");
    model._set("B2", "=CUMPRINC(A2/12,A3*12,A4,1,1,0)");

    model._set("C1", "=CUMPRINC(A2/12,A3*12,A4,1,1,0,1)");
    model._set("C2", "=CUMPRINC(A2/12,A3*12,A4,1,1)");

    model.evaluate();

    assert_eq!(model._get_text("B1"), "-$934.11");
    assert_eq!(model._get_text("B2"), "-$68.28");

    assert_eq!(model._get_text("C1"), *"#ERROR!");
    assert_eq!(model._get_text("C2"), *"#ERROR!");
}

#[test]
fn fn_db_misc() {
    let mut model = new_empty_model();

    model._set("B1", "=DB(0,10,1,2,2)");

    model.evaluate();

    assert_eq!(model._get_text("B1"), "$0.00");
}

// probes/t2475-stat-subexpressions A39, A92, A136, A165 and gen-m9/3c06f9298f65a3ba A50, A148
// (excel-graph; financial.md "Annuity and DB argument domains at their limits").
#[test]
fn annuity_and_db_domain_errors() {
    check_cells(
        &[("B1", "FALSE")],
        &[
            // 0^0 is #NUM!; a P of 0 (rate -1, or underflow near it) is #DIV/0!
            ("A1", "=ERROR.TYPE(PV(-1,0,-713))", "6"),
            ("A2", "=ERROR.TYPE(PV(-1+2^-52,165,-978,224,0))", "2"),
            ("A3", "=ERROR.TYPE(PV(-1,12,-100))", "2"),
            // ln(1+1E-300) is 0; ln(1-1) is LN(0)
            ("A4", "=ERROR.TYPE(NPER(1E-300,-113,3037,368,1))", "2"),
            ("A5", "=ERROR.TYPE(NPER(-1,-31,1))", "6"),
            // a boolean month from a cell is 0, not #VALUE!
            ("A6", "=ERROR.TYPE(DB(15779,385,37,B1,B1))", "6"),
            (
                "A7",
                "=DB(10000,1000,6,1,TRUE)=DB(10000,1000,6,1,1)",
                "TRUE",
            ),
        ],
    );
}

// ── whole-column range and a spill beyond the used area ─────────────────────

// A whole-column range is clipped to the sheet's used area before it is
// walked. Inside an anchor, the column may be spilled on demand during the
// walk, past the bound the walk was clipped to. The skipped part is on
// record, so the spill restarts the pass and the second run sees it all.
#[test]
fn fn_npv_whole_column_sees_a_spill_beyond_the_used_area() {
    let mut model = new_empty_model();
    model._set("A1", "=SEQUENCE(1,1,NPV(0.1,D:D))");
    model._set("C1", "=SEQUENCE(3,1,10)");
    model._set("D1", "=SEQUENCE(5)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "10.65258831");
    assert_eq!(model.evaluation.restarts_in_last_evaluation, 1);

    model.evaluate();
    assert_eq!(model._get_text("A1"), "10.65258831");
    assert_eq!(model.evaluation.restarts_in_last_evaluation, 0);
}

// Excel rejects a workbook whose XNPV call has fewer than three arguments
// (docs/excel-semantics/syntax.md, "Minimum argument counts"); we give #ERROR! and must not panic
// (T-0275). The three-argument value is probes/xnpv-argument-count D1; the four-argument row is
// only a no-panic check (no oracle evidence: such a workbook would not load either).
#[test]
fn fn_xnpv_argument_count() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("B1", "2");
    model._set("A2", "3");
    model._set("B2", "4");
    model._set("D1", "=XNPV(1,A1:B2)");
    model._set("D2", "=XNPV(1)");
    model._set("D3", "=XNPV(1,A1:B2,A1:B2,A1:B2)");
    model._set("D4", "=XNPV(1,A1:B2,A1:B2)");
    model.evaluate();

    assert_eq!(model._get_text("D1"), *"#ERROR!");
    assert_eq!(model._get_text("D2"), *"#ERROR!");
    assert_eq!(model._get_text("D3"), *"#ERROR!");
    assert_eq!(model._get_text("D4"), *"9.962109338");
}

/// When `pv*(1+rate)^nper` overflows PMT is the quotient divided through by P (limit
/// `-pv*rate`), and a P that underflows to 0 is #NUM! (probes/pmt-overflow, excel-graph, T-1405).
#[test]
fn fn_pmt_when_the_power_overflows_or_underflows() {
    let mut model = new_empty_model();
    model._set("A1", "=PMT(100,184,439103)");
    model._set("A2", "=PMT(100,184,439103,0,1)");
    model._set("A3", "=PMT(100,153,1)");
    model._set("A4", "=PMT(1,2000,0,5)");
    model._set("A5", "=PMT(-0.5,1070,0,5)");
    model._set("A6", "=IPMT(100,1,184,439103)");
    model._set("A7", "=PPMT(100,1,184,439103)");
    model._set("A8", "=FV(100,184,1)");
    model.evaluate();
    let number = |row| model.get_cell_value_by_index(0, row, 1);
    assert_eq!(number(1), Ok(CellValue::Number(-43_910_300.0)));
    assert_eq!(number(2), Ok(CellValue::Number(-43_910_300.0 / 101.0)));
    assert_eq!(number(3), Ok(CellValue::Number(-100.0)));
    assert_eq!(number(4), Ok(CellValue::Number(0.0)));
    assert_eq!(model._get_text("A5"), "#NUM!");
    assert_eq!(number(6), Ok(CellValue::Number(-43_910_300.0)));
    assert_eq!(number(7), Ok(CellValue::Number(0.0)));
    assert_eq!(model._get_text("A8"), "#NUM!");
}

/// PMT, IPMT and PPMT are the exact payment on the double arguments to Excel's 15 digits, which
/// the double forms miss where they cancel: long terms, type 1, the first periods of PPMT and
/// fractional periods (probes-numeric/m9-fin-pmt A86, A91, A503, A504; ironcalc-calc
/// `FINANCIAL/IPMT_PPMT.xlsx` G17, J13; T-2130).
#[test]
fn pmt_ipmt_and_ppmt_are_the_exact_payment() {
    let cells: [(&str, f64); 6] = [
        ("=PMT(0.1,2,10000)", -5_761.904_761_904_763),
        ("=PMT(0.1,1,10000,-500,1)", -9_545.454_545_454_548),
        ("=IPMT(0.3,180,360,10000,0,1)", -2_307.692_307_692_307),
        ("=PPMT(0.3,360,360,10000,-500)", -2_192.307_692_307_692_4),
        ("=IPMT(0.1,3.9,3,8000,10,1)", -26.866_364_667_656_377),
        ("=PPMT(0.1,3.3,30,8000)", -60.554_028_580_337_764),
    ];
    let mut model = new_empty_model();
    for (row, (formula, _)) in cells.iter().enumerate() {
        model._set(&format!("A{}", row + 1), formula);
    }
    model.evaluate();
    for (row, (formula, expected)) in cells.iter().enumerate() {
        let Ok(CellValue::Number(actual)) = model.get_cell_value_by_index(0, row as i32 + 1, 1)
        else {
            panic!("{formula} is not a number");
        };
        // Within half a unit of Excel's 15th significant digit (decision 0083).
        let half_unit = 0.5 * 10f64.powi(expected.abs().log10().floor() as i32 - 14);
        assert!(
            (actual - expected).abs() <= half_unit,
            "{formula}: {actual} vs Excel {expected}"
        );
    }
}

/// A start period below 1 is `#NUM!` as given, also when it would round up to 1 (T-2489; the
/// gen-m9 cells 0a957119475760c5 A40, 3aa168259335be54 A10, 6376e27be7a05871 A306).
#[test]
fn cumulative_start_period_below_one_is_num() {
    let mut model = new_empty_model();
    for (cell, start) in [("A1", "0.3"), ("A2", "0.5"), ("A3", "1E-15"), ("A4", "0")] {
        model._set(cell, &format!("=CUMIPMT(0.01,10,1000,{start},5,0)"));
        model._set(
            &cell.replace('A', "B"),
            &format!("=CUMPRINC(0.01,10,1000,{start},5,0)"),
        );
    }
    model._set("C1", "=CUMIPMT(0.01,10,1000,1,5,0)");
    model._set("C2", "=CUMPRINC(0.01,10,1000,1,5,0)");
    model.evaluate();
    for cell in ["A1", "A2", "A3", "A4", "B1", "B2", "B3", "B4"] {
        assert_eq!(model._get_text(cell), *"#NUM!", "{cell}");
    }
    assert_ne!(model._get_text("C1"), *"#NUM!");
    assert_ne!(model._get_text("C2"), *"#NUM!");
}

/// A guess at or above 2^53 is #VALUE!, before the flows are read (probes/irr-guess-edges,
/// irr-guess-edges-2; T-2494).
#[test]
fn fn_irr_guess_at_or_above_two_to_the_53() {
    let mut model = new_empty_model();
    model._set("A1", "=IRR({-100,60,60},2^53-1)=IRR({-100,60,60})");
    model._set("A2", "=IRR({-100,60,60},2^53)");
    model._set("A3", "=IRR({-100,60,60},1E+300)");
    model._set("A4", "=IRR(5,1E+300)");
    model._set("A5", "=IRR(5)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"TRUE");
    assert_eq!(model._get_text("A2"), *"#VALUE!");
    assert_eq!(model._get_text("A3"), *"#VALUE!");
    assert_eq!(model._get_text("A4"), *"#VALUE!");
    assert_eq!(model._get_text("A5"), *"#NUM!");
}

/// NPV sums v_i/d(i-1) left to right with d the running product of 1+rate, then divides by
/// 1+rate once: Excel's residues at IRR's root, to the bit (financial.md "NPV's residue at IRR's
/// root is not the double sum"; IRR.xlsx General!M2, M3, M8, Three Parameters!E2,
/// probes/fin-argument-modes H367, m9-fin-nper-npv A148; T-2672).
#[test]
fn fn_npv_running_product_then_one_division() {
    check_cells_with_numbers(
        &[
            ("B1", 0.079_308_261_160_397_24),
            ("B2", -0.101_500_676_053_584_7),
            ("B3", -0.443_506_941_334_327_55),
            ("B4", 1.091_399_510_766_422E-7),
        ],
        &[],
        &[
            ("A1", "=NPV(B1,-20,5,5,5,5,5)", "6.4212018552224666E-12"),
            (
                "A2",
                "=NPV(B2,-1000,230,200,125,200,10)",
                "-2.8358465972398184E-11",
            ),
            ("A3", "=NPV(B3,-70,12,15)", "-1.579429159894141E-10"),
            ("A4", "=NPV(B4,1,-2,1)", "1.187938506697367E-14"),
            (
                "A5",
                "=NPV(0.1,-10000,2750,4250,3250,2750)",
                "302.233453999042",
            ),
            (
                "A6",
                "=NPV(0.1,-1E6,250000,250000,250000,250000,250000)",
                "-47548.46149808013",
            ),
        ],
    );
}

/// NPV is #DIV/0! where 1+rate is 0 or a power a later period builds on leaves the double range;
/// the last period's power may overflow (probes/npv-power-edges, npv-power-edges-2; T-2493).
#[test]
fn fn_npv_discount_past_the_double_range() {
    let mut model = new_empty_model();
    model._set("A1", "=NPV(-1,0)");
    model._set("A2", "=NPV(-1,{1,2})");
    model._set("A3", "=NPV(1E+300,1,1E+300)=2E-300");
    model._set("A4", "=NPV(1E+300,1,0,0)");
    model._set("A5", "=NPV(1,SEQUENCE(1024))=2-2^-52");
    model._set("A6", "=NPV(1,SEQUENCE(1025))");
    model._set("A7", "=NPV(-0.5,SEQUENCE(1023,1,0,0))=0");
    model._set("A8", "=NPV(-0.5,SEQUENCE(1074,1,0,0))");
    model._set("A9", "=NPV(9E+307,1)=0");
    model._set("A10", "=NPV(-2,1,1)=0");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#DIV/0!");
    assert_eq!(model._get_text("A2"), *"#DIV/0!");
    assert_eq!(model._get_text("A3"), *"TRUE");
    assert_eq!(model._get_text("A4"), *"#DIV/0!");
    assert_eq!(model._get_text("A5"), *"TRUE");
    assert_eq!(model._get_text("A6"), *"#DIV/0!");
    assert_eq!(model._get_text("A7"), *"TRUE");
    assert_eq!(model._get_text("A8"), *"#DIV/0!");
    assert_eq!(model._get_text("A9"), *"TRUE");
    assert_eq!(model._get_text("A10"), *"TRUE");
}

/// An error in a range or an array is NPV's result only when no direct argument errs; the
/// first such error wins among them (probes/npv-power-edges A36-A51, npv-power-edges-2; T-2493).
#[test]
fn fn_npv_defers_range_and_array_errors() {
    let mut model = new_empty_model();
    model._set("Q1", "1");
    model._set("Q2", "=1/0");
    model._set("A1", "=NPV(0.1,{1,#DIV/0!},\"x\")");
    model._set("A2", "=NPV(0.1,Q1:Q2,NA())");
    model._set("A3", "=NPV(0.1,{#N/A},#DIV/0!)");
    model._set("A4", "=NPV(0.1,{1,#N/A},Q1:Q2)");
    model._set("A5", "=NPV(0.1,Q1:Q2,{1,#N/A})");
    model._set("A6", "=NPV(1.0049,0,SEQUENCE(2)/0,5)");
    model._set("A7", "=NPV(1E+300,1,1,{1,#N/A})");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#VALUE!");
    assert_eq!(model._get_text("A2"), *"#N/A");
    assert_eq!(model._get_text("A3"), *"#DIV/0!");
    assert_eq!(model._get_text("A4"), *"#N/A");
    assert_eq!(model._get_text("A5"), *"#DIV/0!");
    assert_eq!(model._get_text("A6"), *"#DIV/0!");
    assert_eq!(model._get_text("A7"), *"#N/A");
}

#[test]
fn fn_db_negative_salvage_is_an_error_value() {
    // A negative salvage is #NUM! as a value, not a NaN number ISNUMBER sees (T-2551; gen-m9
    // f066968376660329 A244).
    let mut model = new_empty_model();
    model._set("A1", "=ISNUMBER(DB(59341,-1,28,6,12))");
    model._set("A2", "=ISERROR(DB(59341,-1,28,6,12))");
    model._set("A3", "=DB(59341,-1,28,6,12)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "FALSE");
    assert_eq!(model._get_text("A2"), "TRUE");
    assert_eq!(model._get_text("A3"), "#NUM!");
}

/// XIRR reads a written empty guess as the default 0.1 but an empty cell as guess 0; IRR reads
/// both as 0 (probes/xirr-irr-empty-guess; T-2688).
#[test]
fn fn_xirr_empty_guess_is_the_default() {
    let mut model = new_empty_model();
    let flows = "{-10000,2750,4250,3250,2750},{39448,39508,39751,39859,39904}";
    model._set("A1", &format!("=XIRR({flows},)=XIRR({flows})"));
    model._set("A2", &format!("=XIRR({flows},)=XIRR({flows},0)"));
    model._set("A3", &format!("=XIRR({flows},E1)=XIRR({flows},0)"));
    let irr_flows = "{-70000,12000,15000,18000,21000,26000}";
    model._set("A4", &format!("=IRR({irr_flows},)=IRR({irr_flows},0)"));
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"TRUE");
    assert_eq!(model._get_text("A2"), *"FALSE");
    assert_eq!(model._get_text("A3"), *"TRUE");
    assert_eq!(model._get_text("A4"), *"TRUE");
}

// Past `(1+rate)^nper`'s range a fractional nper gives what an integer one does; type 1 at a rate
// that is not a whole number is `-B*(rate/(1+rate))`, at a whole rate the exact quotient
// (probes/t2665-ipmt-fractional-nper-past-range A160, A208; gen-m9 b73c6cc96e093f03 A268; T-2665).
#[test]
fn ipmt_past_the_range_fractional_nper_is_the_integer_value() {
    let mut model = new_empty_model();
    model._set("A1", "=IPMT(0.5,50,100000.5,92160,674,1)");
    model._set("A2", "=IPMT(1.005,7,18518518.44345,92160,674,1)");
    model._set("A3", "=IPMT(1.005,7,100000,92160,674,1)");
    model._set("A4", "=IPMT(42,8,290,392736,-0.5,1)");
    model._set("A5", "=IPMT(0.1,7,1000000.25,1000,0,0)");
    model.evaluate();
    let number = |row| model.get_cell_value_by_index(0, row, 1);
    assert_eq!(number(1), Ok(CellValue::Number(-30720.0)));
    assert_eq!(number(2), Ok(CellValue::Number(-46194.91271820449)));
    assert_eq!(number(3), Ok(CellValue::Number(-46194.91271820449)));
    assert_eq!(number(4), Ok(CellValue::Number(-383602.6046511628)));
    assert_eq!(number(5), Ok(CellValue::Number(-100.0)));
}

/// XIRR returns an error values argument first, then an error dates argument, before it reads the
/// values' cells (gen-m9/67a4c03afd63f988 A241, 79d87fc527927626 A228; T-2611).
#[test]
fn fn_xirr_error_arguments_before_the_value_cells() {
    let mut model = new_empty_model();
    model._set("B1", "-100");
    model._set("B2", "x");
    model._set("A1", "=XIRR(B1:B2,1/0)");
    model._set("A2", "=XIRR(#VALUE!,1/0)");
    model._set("A3", "=XIRR(B1:B2,{39448,39508})");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#DIV/0!");
    assert_eq!(model._get_text("A2"), *"#VALUE!");
    assert_eq!(model._get_text("A3"), *"#VALUE!");
}

/// XIRR's error guess comes before an error dates argument, the size checks and the values' cells
/// (gen-m9/1dce4774f5c0e736 A189, 1b60bf99d71424e7 A29; T-2611).
#[test]
fn fn_xirr_guess_error_before_the_dates_and_the_sizes() {
    let mut model = new_empty_model();
    model._set("B1", "-100");
    model._set("B2", "x");
    model._set("A1", "=XIRR(1E+21,1/0,SQRT(-1))");
    model._set("A2", "=XIRR({-1000,300,400,500},{1,2},NA())");
    model._set("A3", "=XIRR(B1:B2,{1,2,3},#DIV/0!)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#NUM!");
    assert_eq!(model._get_text("A2"), *"#N/A");
    assert_eq!(model._get_text("A3"), *"#DIV/0!");
}

/// One empty or numeric-text cell beside a range of another count is the count's `#NUM!`, but one
/// boolean cell keeps `#VALUE!` (gen-m9/1dce4774f5c0e736 A91, 89f0aee953a55a45 A330,
/// c0681a8b150ce301 A266, 1cd3096e3e87c965 A48; T-2611).
#[test]
fn fn_xirr_one_empty_or_numeric_text_cell_is_the_count_error() {
    let mut model = new_empty_model();
    model._set("B1", "-100");
    model._set("B2", "50");
    model._set("B3", "60");
    model._set("C1", "'1");
    model._set("C2", "FALSE");
    model._set("A1", "=XIRR(D1:D1,B1:B3)");
    model._set("A2", "=XIRR(B1:B3,D1,0.5)");
    model._set("A3", "=XIRR(B1:B3,C1)");
    model._set("A4", "=XIRR(B1:B3,C2)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#NUM!");
    assert_eq!(model._get_text("A2"), *"#NUM!");
    assert_eq!(model._get_text("A3"), *"#NUM!");
    assert_eq!(model._get_text("A4"), *"#VALUE!");
}
