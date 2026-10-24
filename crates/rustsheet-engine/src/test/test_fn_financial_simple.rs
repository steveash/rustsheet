#![allow(clippy::unwrap_used)]

use crate::cell::CellValue;
use crate::test::util::new_empty_model;

// =============================================================================
// DISC
// =============================================================================

#[test]
fn fn_disc_example() {
    // settlement=2008-01-25 (39472), maturity=2008-06-15 (39614)
    // pr=97.975, redemption=100, basis=1 (actual/actual)
    // 142 actual days / 366 (2008 leap year) = 0.38798 yearfrac
    // DISC = (1 - 97.975/100) / 0.38798 ≈ 0.052193662
    let mut model = new_empty_model();
    model._set("A1", "=DISC(39472, 39614, 97.975, 100, 1)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"0.052193662");
}

#[test]
fn fn_disc_default_basis() {
    let mut model = new_empty_model();
    model._set("A1", "=DISC(39472, 39614, 97.975, 100)");
    model.evaluate();
    // basis 0: days360(39472,39614)/360
    assert!(model._get_text("A1").starts_with("0.05"));
}

#[test]
fn fn_disc_error_pr_zero() {
    let mut model = new_empty_model();
    model._set("A1", "=DISC(39472, 39614, 0, 100, 1)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"#NUM!");
}

#[test]
fn fn_disc_error_settlement_after_maturity() {
    let mut model = new_empty_model();
    model._set("A1", "=DISC(39614, 39472, 97.975, 100, 1)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"#NUM!");
}

// =============================================================================
// FVSCHEDULE
// =============================================================================

#[test]
fn fn_fvschedule_example() {
    // principal=1, schedule={0.09, 0.11, 0.1}
    let mut model = new_empty_model();
    model._set("B1", "0.09");
    model._set("B2", "0.11");
    model._set("B3", "0.1");
    model._set("A1", "=FVSCHEDULE(1, B1:B3)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"1.33089");
}

#[test]
fn fn_fvschedule_single_rate() {
    let mut model = new_empty_model();
    model._set("B1", "0.05");
    model._set("A1", "=FVSCHEDULE(1000, B1:B1)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"1050");
}

#[test]
fn fn_fvschedule_error_rate_too_low() {
    let mut model = new_empty_model();
    model._set("B1", "-1.5");
    model._set("A1", "=FVSCHEDULE(1000, B1:B1)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"-500");
}

#[test]
fn fn_fvschedule_more() {
    let mut model = new_empty_model();
    model._set("A1", "=FVSCHEDULE(120, {0.2, 0.4, 0.6, 0.8})");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"580.608");
}

// =============================================================================
// INTRATE
// =============================================================================

#[test]
fn fn_intrate_example() {
    // settlement=2008-02-15 (39493), maturity=2008-05-15 (39583)
    // investment=1000000, redemption=1014420, basis=2
    let mut model = new_empty_model();
    model._set("A1", "=INTRATE(39493, 39583, 1000000, 1014420, 2)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"0.05768");
}

#[test]
fn fn_intrate_error_investment_zero() {
    let mut model = new_empty_model();
    model._set("A1", "=INTRATE(39493, 39583, 0, 1014420, 2)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"#NUM!");
}

// =============================================================================
// PRICEDISC
// =============================================================================

#[test]
fn fn_pricedisc_example() {
    // settlement=2008-02-16 (39494), maturity=2008-03-01 (39508)
    // discount=0.0525, redemption=100, basis=2 (actual/360)
    // 14 actual days / 360 → yearfrac=0.038889
    // PRICEDISC = 100 * (1 - 0.0525 * 0.038889) ≈ 99.795833333
    let mut model = new_empty_model();
    model._set("A1", "=PRICEDISC(39494, 39508, 0.0525, 100, 2)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"99.79583333");
}

#[test]
fn fn_pricedisc_error_discount_zero() {
    let mut model = new_empty_model();
    model._set("A1", "=PRICEDISC(39494, 39508, 0, 100, 2)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"#NUM!");
}

// =============================================================================
// PRICEMAT
// =============================================================================

#[test]
fn fn_pricemat_example() {
    // settlement=2008-02-15 (39493), maturity=2008-04-13 (39551)
    // issue=2007-11-11 (39397), rate=0.0610, yld=0.0625, basis=0 (US 30/360)
    // par * [(1 + rate * dim) / (1 + yld * dsm) - rate * dis]
    let mut model = new_empty_model();
    model._set("A1", "=PRICEMAT(39493, 39551, 39397, 0.0610, 0.0625, 0)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"99.96019575");
}

#[test]
fn fn_pricemat_error_settlement_after_maturity() {
    let mut model = new_empty_model();
    model._set("A1", "=PRICEMAT(39551, 39493, 39397, 0.061, 0.0625, 0)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"#NUM!");
}

// =============================================================================
// RECEIVED
// =============================================================================

#[test]
fn fn_received_example() {
    // settlement=2008-02-15 (39493), maturity=2008-05-15 (39583)
    // investment=1000000, discount=0.0575, basis=2 (actual/360)
    // 90 days / 360 = 0.25 yearfrac
    // RECEIVED = 1000000 / (1 - 0.0575 * 0.25) ≈ 1014584.654
    let mut model = new_empty_model();
    model._set("A1", "=RECEIVED(39493, 39583, 1000000, 0.0575, 2)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"1014584.654");
}

#[test]
fn fn_received_error_investment_zero() {
    let mut model = new_empty_model();
    model._set("A1", "=RECEIVED(39493, 39583, 0, 0.0575, 2)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"#NUM!");
}

// =============================================================================
// YIELDDISC
// =============================================================================

#[test]
fn fn_yielddisc_example() {
    // settlement=2008-02-16 (39494), maturity=2008-03-01 (39508)
    // pr=97.975, redemption=100, basis=2 (actual/360)
    // 14 actual days / 360 → yearfrac=0.038889
    // YIELDDISC = (100/97.975 - 1) / 0.038889 ≈ 0.531476689
    let mut model = new_empty_model();
    model._set("A1", "=YIELDDISC(39494, 39508, 97.975, 100, 2)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"0.531476689");
}

#[test]
fn fn_yielddisc_error_pr_zero() {
    let mut model = new_empty_model();
    model._set("A1", "=YIELDDISC(39494, 39508, 0, 100, 2)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"#NUM!");
}

// =============================================================================
// YIELDMAT
// =============================================================================

#[test]
fn fn_yieldmat_example() {
    // settlement=2008-03-15 (39522), maturity=2008-11-03 (39755)
    // issue=2007-11-08 (39394), rate=0.0625, price=100.0123, basis=0 (US 30/360)
    // YIELDMAT = [(1 + rate*dim) / (price/100 + rate*dis) - 1] / dsm
    let mut model = new_empty_model();
    model._set("A1", "=YIELDMAT(39522, 39755, 39394, 0.0625, 100.0123, 0)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"0.060954334");
}

#[test]
fn fn_yieldmat_error_settlement_after_maturity() {
    let mut model = new_empty_model();
    model._set("A1", "=YIELDMAT(39755, 39522, 39394, 0.0625, 100.0123, 0)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"#NUM!");
}

// Settlement serial 0: INTRATE, PRICEDISC and YIELDDISC compute, RECEIVED is #NUM! (T-1757;
// docs/excel-semantics/financial.md "Discount-security functions: settlement 0").
#[test]
fn discount_security_settlement_zero() {
    let mut model = new_empty_model();
    model._set("A1", "=INTRATE(0,43466,1000,1100)");
    model._set("A2", "=PRICEDISC(0,43466,0.05,100)");
    model._set("A3", "=YIELDDISC(0,43466,95,100)");
    model._set("A4", "=RECEIVED(0,43466,1000,0.05)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"0.000840317");
    assert_eq!(model._get_text("A2"), *"-495.0138889");
    assert_eq!(model._get_text("A3"), *"0.000442272");
    assert_eq!(model._get_text("A4"), *"#NUM!");
}

#[test]
fn fn_fvschedule_boolean_principal_is_value_error() {
    let mut model = new_empty_model();
    model._set("A1", "=FVSCHEDULE(TRUE, {0.1})");
    model._set("A2", "=FVSCHEDULE(\"1000\", {0.1})");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"#VALUE!");
    assert_eq!(model._get_text("A2"), *"1100");
}

#[test]
fn discount_securities_divide_by_day_count_over_year_length() {
    // Excel's bits (financial.md "Discount securities: day count and year length", T-2032):
    // ironcalc-calc DISC.xlsx F2 and INTRATE.xlsx F2 (Excel-cached), probes/discount-security-modes
    // A14, A80, A113, A126, A103 and A43 (excel-graph).
    let cases: [(&str, &str, f64); 8] = [
        (
            "A1",
            "=DISC(39472,39614,97.975,100,0)",
            0.052_071_428_571_428_84,
        ),
        ("A2", "=INTRATE(46433,46522,2000000,2040000,0)", 0.08),
        (
            "A3",
            "=INTRATE(43159,43189,1000,1100,0)",
            1.200_000_000_000_000_2,
        ),
        (
            "A4",
            "=YIELDDISC(42400,42460,95,100,1)",
            0.321_052_631_578_947_3,
        ),
        (
            "A5",
            "=YIELDMAT(43101,43466,42736,0.06,98,0)",
            0.076_923_076_923_076_98,
        ),
        (
            "A6",
            "=YIELDMAT(43159,43189,42736,0.06,98,0)",
            0.271_557_884_707_002,
        ),
        (
            "A7",
            "=PRICEMAT(43159,43189,42736,0.06,0.05,0)",
            100.057_743_362_831_86,
        ),
        ("A8", "=PRICEDISC(0,43466,0.05,100)", -495.013_888_888_888_9),
    ];
    let mut model = new_empty_model();
    for (cell, formula, _) in cases {
        model._set(cell, formula);
    }
    model.evaluate();
    for (cell, formula, excel) in cases {
        match model.get_cell_value_by_ref(&format!("Sheet1!{cell}")) {
            Ok(CellValue::Number(n)) => assert_eq!(n.to_bits(), excel.to_bits(), "{formula}: {n}"),
            other => panic!("{formula}: expected a number, got {other:?}"),
        }
    }
}

#[test]
fn effect_and_nominal_use_excels_power() {
    // Excel's `^` (square-and-multiply for EFFECT's integer n, exp(ln(1+e)*(1/n)) for NOMINAL's
    // 1/n), not libm pow: ironcalc-calc FINANCIAL/NOMINAL_EFFECT.xlsx D2, C8 and
    // probes/fin-argument-modes H125 (T-2034).
    let mut model = new_empty_model();
    let cases = [
        ("A1", "=EFFECT(0.5,10)", 0.628_894_626_777_441_6),
        ("A2", "=NOMINAL(0.5,4.5)", 0.426_727_678_801_286_8),
        ("A3", "=EFFECT(0.05,4)", 0.050_945_336_914_062_445),
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
fn effect_and_nominal_overflow_is_num_error() {
    // probes/effect-overflow-power-of-ten (excel-graph): an overflowing `^` is #NUM!, and
    // EFFECT(270,30) is 1E30 (T-2241).
    let mut model = new_empty_model();
    model._set("A1", "=EFFECT(1E300,2)");
    model._set("A2", "=NOMINAL(1E300,0.5)");
    model._set("A3", "=EFFECT(270,30)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"#NUM!");
    assert_eq!(model._get_text("A2"), *"#NUM!");
    assert_eq!(
        model.get_cell_value_by_ref("Sheet1!A3"),
        Ok(CellValue::Number(1e30))
    );
}

// =============================================================================
// Argument order and booleans (T-2455, probes/t2455-transpose-single-value)
// =============================================================================

// The discount securities read basis first (A21, A64, A65); a boolean ACCRINTM par, ACCRINT issue
// and AMORDEGRC rate are `#VALUE!` (A41, A52, A51), after an out-of-range earlier date's `#NUM!`.
#[test]
fn financial_argument_order_and_booleans() {
    let mut model = new_empty_model();
    model._set("A1", r#"=RECEIVED(37025,2.5,1E+300,1/0,"abc")"#);
    model._set("A2", r#"=DISC(37025,2.5,1/0,100,"abc")"#);
    model._set("A3", "=ACCRINTM(34825,37622,1E-9,TRUE,4)");
    model._set("A4", "=ACCRINT(TRUE,SQRT(-1),1e-15,100,9984,2,1.9)");
    model._set("A5", "=AMORDEGRC(90812,36624,38707,453,3,TRUE,4)");
    model._set("A6", "=AMORDEGRC(70537,36888,7E13,46,12,TRUE,0)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#VALUE!");
    assert_eq!(model._get_text("A2"), "#VALUE!");
    assert_eq!(model._get_text("A3"), "#VALUE!");
    assert_eq!(model._get_text("A4"), "#VALUE!");
    assert_eq!(model._get_text("A5"), "#VALUE!");
    assert_eq!(model._get_text("A6"), "#NUM!");
}

// The discount securities read the basis first (ACCRINTM last), a date off the calendar is #NUM!
// as it is read, and the date order and amounts are checked after every read
// (probes/discount-argument-check-order, T-2484).
#[test]
fn discount_securities_check_order() {
    let cases = [
        // A date off the calendar beats a later error, text or boolean ...
        ("=DISC(-1,NA(),95,100,0)", "#NUM!"),
        ("=INTRATE(39448,2958466,\"x\",1100,0)", "#NUM!"),
        ("=PRICEDISC(39668,-1,\"x\",106)", "#NUM!"),
        ("=RECEIVED(123456789.123,44370,TRUE,0.05,4)", "#NUM!"),
        ("=PRICEMAT(39448,43831,-1,NA(),0.05,0)", "#NUM!"),
        ("=YIELDMAT(2958466,43831,39000,0.06,TRUE,0)", "#NUM!"),
        ("=ACCRINTM(-1,39448,0.1,1000,NA())", "#NUM!"),
        // ... but not an earlier argument's error, nor (except ACCRINTM) the basis.
        ("=DISC(NA(),-1,95,100,0)", "#N/A"),
        ("=YIELDDISC(-1,43831,95,100,\"x\")", "#VALUE!"),
        ("=PRICEMAT(-1,43831,39000,0.06,0.05,NA())", "#N/A"),
        // Serial 0.5 passes the read.
        ("=DISC(0.5,NA(),95,100,0)", "#N/A"),
        // A basis off 0-4 is #NUM! before any other argument is read.
        ("=YIELDDISC(9.33,1E+15,0,NA(),5)", "#NUM!"),
        ("=YIELDMAT(NA(),43831,39000,0.06,98,-1)", "#NUM!"),
        ("=ACCRINTM(NA(),39448,0.1,1000,5)", "#N/A"),
        // The date order and the amounts come after every argument is read.
        ("=INTRATE(39448,39000,1000,NA(),0)", "#N/A"),
        ("=RECEIVED(39448,43831,0,\"x\",0)", "#VALUE!"),
        ("=PRICEMAT(39448,43831,40000,0.06,TRUE,0)", "#VALUE!"),
        ("=ACCRINTM(39448,38000,TRUE,1000,0)", "#VALUE!"),
        // Two errors: the earlier one in reading order.
        ("=DISC(39448,43831,1/0,NA(),0)", "#DIV/0!"),
        ("=DISC(NA(),43831,95,100,1/0)", "#DIV/0!"),
        ("=ACCRINTM(39000,NA(),0.1,1000,1/0)", "#N/A"),
    ];
    let mut model = new_empty_model();
    for (row, (formula, _)) in cases.iter().enumerate() {
        model._set(&format!("A{}", row + 1), formula);
    }
    model.evaluate();
    for (row, (formula, expected)) in cases.iter().enumerate() {
        assert_eq!(model._get_text(&format!("A{}", row + 1)), *expected, "{formula}");
    }
}
