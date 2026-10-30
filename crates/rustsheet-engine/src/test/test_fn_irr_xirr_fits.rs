#![allow(
    clippy::panic,
    reason = "a test fails by panicking on a non-number result"
)]

//! IRR and XIRR as fitted against the excel-graph solver traces (T-0916; fit lab `irr`, `xirr`;
//! docs/excel-semantics/financial.md "IRR is a secant in d = 1 - v" and "XIRR is a bracketing bisection").
//! Each value is the oracle's; the guess changes Excel's last digits, so each one pins the path.

use crate::cell::CellValue;
use crate::model::Model;
use crate::test::util::new_empty_model;

/// A model with `formulas` in A1, A2, ... of Sheet1, evaluated.
fn evaluated(formulas: &[&str]) -> Model {
    let mut model = new_empty_model();
    for (i, formula) in formulas.iter().enumerate() {
        model._set(&format!("A{}", i + 1), formula);
    }
    model.evaluate();
    model
}

fn results(formulas: &[&str]) -> Vec<u64> {
    let model = evaluated(formulas);
    (1..=formulas.len())
        .map(
            |row| match model.get_cell_value_by_ref(&format!("Sheet1!A{row}")) {
                Ok(CellValue::Number(n)) => n.to_bits(),
                other => panic!("A{row}: expected a number, got {other:?}"),
            },
        )
        .collect()
}

#[test]
fn irr_is_a_secant_in_the_discount_factor() {
    // probes/solver-iteration-traces I!A1, I!A10
    let got = results(&["=IRR({-20,5,5,5,5,5},0.05)", "=IRR({-20,5,5,5,5,5},0.0725)"]);
    let expected = [0.079_308_261_158_238_75_f64, 0.079_308_261_160_528_69];
    assert_eq!(got, expected.map(f64::to_bits));
}

#[test]
fn irr_far_roots_sit_on_the_discount_grid() {
    // probes/irr-fallback-roots P!A25, A28, A39 and t2565-irr-guess-sweeps P!A107 (a zero step
    // that stops on the residual relative to the largest flow); T-3120
    let got = results(&[
        "=IRR({-1,1E+4})",
        "=IRR({-1,3E+6})",
        "=IRR({-1,0,1E+14})",
        "=IRR({-2500000,-1000000,123456789123000})",
    ]);
    let expected = [
        9_999.000_000_001_1_f64,
        2_999_998.999_580_666,
        9_999_998.994_161_328,
        7_026.083_663_633_989,
    ];
    assert_eq!(got, expected.map(f64::to_bits));
}

#[test]
fn irr_drops_leading_zero_flows() {
    // probes/irr-guess-edges P!A33, probes-numeric/m9-fin-irr-rate Calc!A173, A175; T-3120
    let got = results(&[
        "=IRR({0,-100,110},0.05)",
        "=IRR({0,0,-100,110},-0.5)",
        "=IRR({0,0,-100,110},10)",
    ]);
    let expected = [
        0.099_999_999_999_999_87_f64,
        0.100_000_000_000_000_09,
        0.099_999_999_999_999_87,
    ];
    assert_eq!(got, expected.map(f64::to_bits));
}

#[test]
fn xirr_bisects_the_walked_bracket() {
    // probes/solver-iteration-traces X!A1, X!A10
    let got = results(&[
        "=XIRR({-1,1.013},{45000,45365},1)",
        "=XIRR({-1,1.3613},{45000,45365},1)",
    ]);
    let expected = [0.013_000_000_268_220_901_f64, 0.361_299_999_058_246_6];
    assert_eq!(got, expected.map(f64::to_bits));
}

#[test]
fn xirr_with_a_zero_first_value_needs_no_later_date() {
    // probes/xirr-span-zero-first A28 (first date after the others); T-2329
    let got = results(&["=XIRR({0,-1000,1100},{46000,45000,45365})"]);
    let expected = [2.980_232_238_769_531_4e-9_f64];
    assert_eq!(got, expected.map(f64::to_bits));
}

#[test]
fn xirr_with_a_zero_first_value_brackets_at_the_first_chain_point() {
    // probes/xirr-zero-first-bracket A1, A3, A8, A9, A10: the flows do not matter, the guess does
    let got = results(&[
        "=XIRR({0,-1000,2000},{45000,45000,45365})",
        "=XIRR({0,-1000,2000},{45000,45000,45365},0.3)",
        "=XIRR({0,-100,110},{45000,45000,45000})",
        "=XIRR({0,-100,110},{45000,45000,45000},0)",
        "=XIRR({0,-100,110},{45000,45000,45000},2)",
    ]);
    let expected = [
        2.980_232_238_769_531_4e-9_f64,
        4.470_348_358_154_297e-9,
        2.980_232_238_769_531_4e-9,
        4.882_812_5e-9,
        3.725_290_298_461_914e-9,
    ];
    assert_eq!(got, expected.map(f64::to_bits));
}

fn assert_num_errors(formulas: &[&str]) {
    let cases: Vec<(&str, &str)> = formulas.iter().map(|formula| (*formula, "#NUM!")).collect();
    assert_errors(&cases);
}

#[test]
fn xirr_with_a_zero_first_value_and_a_negative_guess_is_num() {
    // probes/xirr-zero-first-bracket A6, A7, A11 (A6 has a root on the downward chain)
    assert_num_errors(&[
        "=XIRR({0,-1000,900},{45000,45000,45365},-0.2)",
        "=XIRR({0,-1000,1100},{45000,45000,45365},-0.5)",
        "=XIRR({0,-100,110},{45000,45000,45000},-0.5)",
    ]);
}

#[test]
fn xirr_with_a_nonzero_first_value_still_needs_a_later_date() {
    // probes/xirr-mismatched-empties X9
    assert_num_errors(&["=XIRR({-1000,1100},{45365,45000})"]);
}

/// Asserts each formula's number is within `relative` of Excel's value.
fn assert_numbers_near(cases: &[(&str, f64)], relative: f64) {
    let formulas: Vec<&str> = cases.iter().map(|case| case.0).collect();
    let model = evaluated(&formulas);
    for (row, (_, want)) in (1..).zip(cases) {
        let value = model.get_cell_value_by_ref(&format!("Sheet1!A{row}"));
        assert!(
            matches!(value, Ok(CellValue::Number(n)) if (n - want).abs() <= relative * want.abs()),
            "A{row}: {value:?}, want {want}"
        );
    }
}

#[test]
fn irr_is_num_at_a_far_root() {
    // probes/irr-fallback-roots A29, A39 and irr-guess-edges-2 A15, A31: 1 + r at or past 1E+7
    assert_num_errors(&["=IRR({-1,1E+7})", "=IRR({-1,1E+13})", "=IRR({-1E-15,1})"]);
    // A28, A39: 3E+6 and r = 9999998.994 still return (their digits are T-2131's)
    assert_numbers_near(
        &[
            ("=IRR({-1,3E+6})", 2_999_998.999_580_666),
            ("=IRR({-1,0,1E+14})", 9_999_998.994_161_328),
        ],
        1e-9,
    );
}

#[test]
fn irr_where_the_secant_fails_is_num_at_a_root_near_minus_one() {
    // probes/irr-fallback-roots A1, A5, A8, A10: the NPV polynomial's root nearest v = 0 is negative
    assert_num_errors(&[
        "=IRR({1,7,-1})",
        "=IRR({1,7,-1},1)",
        "=IRR({1,20,-1})",
        "=IRR({1,3,-1})",
    ]);
}

#[test]
fn irr_where_the_secant_fails_finds_the_root_nearest_v_zero() {
    // probes/irr-fallback-roots A12, A15, A16, A22: within 1E-14 relative of Excel's
    assert_numbers_near(
        &[
            ("=IRR({42,1,1,-1},0.5)", -0.746_704_476_653_737_1),
            ("=IRR({42,1,1,-1},0)", -0.746_704_476_653_737_1),
            ("=IRR({0.3,-2.5,1.005},0.01)", 6.909_808_543_634_611),
            ("=IRR({-400,1,1,1,1,1},0.5)", -0.673_736_675_180_840_5),
        ],
        1e-14,
    );
}

#[test]
fn xirr_at_an_exact_root_of_zero_brackets_at_the_first_chain_point() {
    // probes/irr-guess-edges A41-A47 (T-2728): the flows sum to 0, so XNPV(0) is exactly 0 and
    // the first point of the upward chain brackets: the result is the guess halved down to 0
    let got = results(&[
        "=XIRR({-100,10,20,30,40},{0,1,59,60,61})",
        "=XIRR({-100,10,20,30,40},{0,1,59,60,61},1000)",
        "=XIRR({-100,10,20,30,40},{0,1,59,60,61},123456789.123)",
        "=XIRR({-100,10,20,30,40},{0,1,59,60,61},1E+10)",
        // probes/t2728-irr-guess-ref-mark A73-A81: a negative guess halves toward 0 from below
        "=XIRR({-100,10,20,30,40},{0,1,59,60,61},-0.5)",
        "=XIRR({-100,10,20,30,40},{0,1,59,60,61},0)",
        "=XIRR({-100,10,20,30,40},{0,1,59,60,61},1E+20)",
        "=XIRR({100,-10,-20,-30,-40},{0,1,59,60,61})",
        "=XIRR({-100,10,20,30,40},{0,1,59,60,61},1E+15)",
    ]);
    let expected = [
        2.980_232_238_769_531_4e-9_f64,
        3.637_978_807_091_713e-9,
        3.426_614_245_766_179_6e-9,
        4.336_808_689_942_018e-9,
        -3.725_290_298_461_914e-9,
        4.882_812_5e-9,
        2.524_354_896_707_237_8e-9,
        2.980_232_238_769_531_4e-9,
        3.308_722_450_212_110_7e-9,
    ];
    assert_eq!(got, expected.map(f64::to_bits));
    assert_num_errors(&["=XIRR({-100,10,20,30,40},{0,1,59,60,61},1E+300)"]);
}

/// Asserts each formula's error.
fn assert_errors(cases: &[(&str, &str)]) {
    let formulas: Vec<&str> = cases.iter().map(|case| case.0).collect();
    let model = evaluated(&formulas);
    for (row, (formula, want)) in (1..).zip(cases) {
        assert_eq!(
            model.get_cell_value_by_ref(&format!("Sheet1!A{row}")),
            Ok(CellValue::String((*want).to_string())),
            "A{row}: {formula}"
        );
    }
}

#[test]
fn irr_rejects_a_number_straight_from_cell_or_error_type() {
    // probes/irr-guess-edges-2, probes/t2728-irr-guess-ref-mark A51, A63 (T-2728)
    assert_errors(&[
        ("=IRR(CELL(\"row\",D2))", "#VALUE!"),
        ("=IRR(CELL(\"col\",D2),0.5)", "#VALUE!"),
        ("=IRR(ERROR.TYPE(#REF!))", "#VALUE!"),
        ("=IRR(CELL(\"row\",D2)+0)", "#NUM!"),
        ("=IRR(@CELL(\"row\",D2))", "#NUM!"),
        ("=IRR(ROW(D2))", "#NUM!"),
    ]);
}

#[test]
fn irr_reads_the_guess_between_a_flows_error_and_an_element_error() {
    // probes/t2728-irr-guess-ref-mark A1, A39, A41 and -2 A1, A3, A5 (T-2728)
    assert_errors(&[
        ("=IRR(1/0,NA())", "#DIV/0!"),
        ("=IRR(1/0,1E+300)", "#DIV/0!"),
        ("=IRR({1,\"a\"}+0,1/0)", "#DIV/0!"),
        ("=IRR({1,\"a\"}+0,0.1)", "#VALUE!"),
        ("=IRR(CELL(\"row\",D2),1/0)", "#DIV/0!"),
        ("=IRR(ERROR.TYPE(#REF!),NA())", "#N/A"),
        ("=IRR({1,\"a\"}+0,INDEX(B3:C9,2))", "#REF!"),
    ]);
}

#[test]
fn xnpv_discounts_with_excels_caret_at_an_exact_root() {
    // probes/xirr-exact-root-sign: a few ulps from 0, with Excel's sign (financial.md "XNPV
    // discounts with Excel's `^`").
    let got = results(&["=XNPV(0.1,{-1,6.727499949325611},{45000,52300})"]);
    assert_eq!(got, [2.220_446_049_250_313e-16_f64.to_bits()]);
}

#[test]
fn xnpv_keeps_a_flow_whose_power_overflows_as_zero() {
    // Unprobed: unchanged from the `powf` days (value/inf = 0), not Excel evidence.
    let got = results(&["=XNPV(1E+300,{-100,100},{39448,40000})"]);
    assert_eq!(got, [(-100.0_f64).to_bits()]);
}
