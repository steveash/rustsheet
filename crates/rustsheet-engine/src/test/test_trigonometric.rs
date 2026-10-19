#![allow(clippy::unwrap_used)]

use crate::cell::CellValue;
use crate::test::util::new_empty_model;

#[test]
fn test_fn_pi_arguments() {
    let mut model = new_empty_model();
    model._set("A1", "=PI(1)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#ERROR!");
}

#[test]
fn test_fn_atan2_arguments() {
    let mut model = new_empty_model();
    model._set("A1", "=ATAN2(1)");
    model._set("A2", "=ATAN2(1,1)");
    model._set("A3", "=ATAN2(1,1,1)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#ERROR!");
    assert_eq!(model._get_text("A2"), *"0.785398163");
    assert_eq!(model._get_text("A3"), *"#ERROR!");
}

#[test]
fn test_fn_trigonometric_arguments() {
    let mut model = new_empty_model();
    model._set("A1", "=SIN()");
    model._set("A2", "=COS()");
    model._set("A3", "=TAN()");

    model._set("A5", "=ASIN()");
    model._set("A6", "=ACOS()");
    model._set("A7", "=ATAN()");

    model._set("A9", "=SINH()");
    model._set("A10", "=COSH()");
    model._set("A11", "=TANH()");

    model._set("A13", "=ASINH()");
    model._set("A14", "=ACOSH()");
    model._set("A15", "=ATANH()");

    model._set("B1", "=SIN(1,2)");
    model._set("B2", "=COS(1,2)");
    model._set("B3", "=TAN(1,2)");

    model._set("B5", "=ASIN(1,2)");
    model._set("B6", "=ACOS(1,2)");
    model._set("B7", "=ATAN(1,2)");

    model._set("B9", "=SINH(1,2)");
    model._set("B10", "=COSH(1,2)");
    model._set("B11", "=TANH(1,2)");

    model._set("B13", "=ASINH(1,2)");
    model._set("B14", "=ACOSH(1,2)");
    model._set("B15", "=ATANH(1,2)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#ERROR!");
    assert_eq!(model._get_text("A2"), *"#ERROR!");
    assert_eq!(model._get_text("A3"), *"#ERROR!");

    assert_eq!(model._get_text("A5"), *"#ERROR!");
    assert_eq!(model._get_text("A6"), *"#ERROR!");
    assert_eq!(model._get_text("A7"), *"#ERROR!");

    assert_eq!(model._get_text("A9"), *"#ERROR!");
    assert_eq!(model._get_text("A10"), *"#ERROR!");
    assert_eq!(model._get_text("A11"), *"#ERROR!");

    assert_eq!(model._get_text("B1"), *"#ERROR!");
    assert_eq!(model._get_text("B2"), *"#ERROR!");
    assert_eq!(model._get_text("B3"), *"#ERROR!");

    assert_eq!(model._get_text("B5"), *"#ERROR!");
    assert_eq!(model._get_text("B6"), *"#ERROR!");
    assert_eq!(model._get_text("B7"), *"#ERROR!");

    assert_eq!(model._get_text("B9"), *"#ERROR!");
    assert_eq!(model._get_text("B10"), *"#ERROR!");
    assert_eq!(model._get_text("B11"), *"#ERROR!");
}

#[test]
fn test_fn_tan_pi2() {
    let mut model = new_empty_model();
    model._set("A1", "=TAN(PI()/2)");
    model.evaluate();

    // Excel reduces by the x87 64-bit pi: 1.6324552277619072E+16, not IEEE's 1.633123935319537E+16
    // (probes-numeric/m9-trig-reduction C82, T-1563).
    assert_eq!(model._get_text("A1"), *"1.63246E+16");
}

#[test]
fn test_trigonometric_identity() {
    let mut model = new_empty_model();
    model._set("A1", "=COTH(1)*CSCH(1)");
    model._set("A2", "=COSH(1)/(SINH(1))^2");
    model._set("A3", "=A1=A2");
    model.evaluate();

    assert_eq!(model._get_text("A3"), *"TRUE");
}

// T-1202: an out-of-domain argument is `#NUM!` at once, so an enclosing function cannot swallow
// the NaN (gen-m5/cd883013bea3b305, gen-m6/3ee7ffc8d6ab061f, gen-m7/93976dd9a3578ed9; ATANH by the same macro).
#[test]
fn out_of_domain_inverse_functions_nested_are_num() {
    let mut model = new_empty_model();
    model._set("A1", "=SERIESSUM(ACOS(1E+15),0,1,1)");
    model._set("A2", "=ADDRESS(ACOSH(-1),4)");
    model._set("A3", "=CHOOSECOLS(0,ASIN(100))");
    model._set("A4", "=(1<ASIN(PI()))");
    model._set("A5", "=SUM(ATANH(2),1)");
    model.evaluate();

    for cell in ["A1", "A2", "A3", "A4", "A5"] {
        assert_eq!(model._get_text(cell), *"#NUM!", "{cell}");
    }
}

#[test]
fn asin_rounds_like_excel() {
    // probes/asin-argument-sweep, gen-m6/2f6b96385ed3facd A38, gen-m7/cb748b6a9b81fbb7 A38 (T-1259):
    // libm `asin` is one ulp off on each of these.
    let mut model = new_empty_model();
    model._set("A1", "=ASIN(-0.0001)");
    model._set("A2", "=ASIN(0.42)");
    model._set("A3", "=ASIN(-0.96)");
    model._set("A4", "=ASIN(1)");
    model._set("A5", "=ASIN(-1)");
    model._set("A6", "=ASIN(1.0000001)");
    model._set("A7", "=ASIN(1E-13)");
    model._set("A8", "=ASIN(-1E-13)");
    model._set("A9", "=ASIN(1E-8)");
    // T-1797: ASIN is not odd (sweep B211, B256, B109, B91).
    model._set("A10", "=ASIN(-1E-8)");
    model._set("A11", "=ASIN(-0.999)");
    model._set("A12", "=ASIN(0.999)");
    model._set("A13", "=ASIN(0.09)");
    model._set("A14", "=ASIN(-0.09)");
    model.evaluate();
    let bits = |c: &str| match model.get_cell_value_by_ref(&format!("Sheet1!{c}")) {
        Ok(CellValue::Number(n)) => n.to_bits(),
        other => panic!("expected number, got {other:?}"),
    };
    assert_eq!(bits("A1"), (-0.000_100_000_000_166_666_68_f64).to_bits());
    assert_eq!(bits("A2"), 0.433_445_320_069_885_9_f64.to_bits());
    assert_eq!(bits("A3"), (-1.287_002_217_586_568_5_f64).to_bits());
    assert_eq!(bits("A4"), std::f64::consts::FRAC_PI_2.to_bits());
    assert_eq!(bits("A5"), (-std::f64::consts::FRAC_PI_2).to_bits());
    assert_eq!(model._get_text("A6"), *"#NUM!");
    assert_eq!(bits("A7"), 1e-13_f64.to_bits());
    assert_eq!(bits("A8"), (-1e-13_f64).to_bits());
    assert_eq!(bits("A9"), 1.000_000_000_000_000_2e-8_f64.to_bits());
    assert_eq!(bits("A10"), (-1e-8_f64).to_bits());
    assert_eq!(bits("A11"), (-1.526_071_239_626_165_5_f64).to_bits());
    assert_eq!(bits("A12"), 1.526_071_239_626_163_f64.to_bits());
    assert_eq!(bits("A13"), 0.090_121_945_014_595_26_f64.to_bits());
    assert_eq!(bits("A14"), (-0.090_121_945_014_595_25_f64).to_bits());
}

#[test]
fn acos_is_half_pi_minus_asin() {
    // probes/asin-atan-sign-source H and I (excel-graph, T-1797) and poi/48539.xlsx E23 (T-0019):
    // libm `acos` is one or more ulp off on each of these.
    let mut model = new_empty_model();
    model._set("A1", "=ACOS(0.5)");
    model._set("A2", "=ACOS(-0.999)");
    model._set("A3", "=ACOS(0.999)");
    model._set("A4", "=ACOS(-0.5)");
    model._set("A5", "=ACOS(1)");
    model._set("A6", "=ACOS(-1)");
    model._set("A7", "=ACOS(1.0000001)");
    model.evaluate();
    let bits = |c: &str| match model.get_cell_value_by_ref(&format!("Sheet1!{c}")) {
        Ok(CellValue::Number(n)) => n.to_bits(),
        other => panic!("expected number, got {other:?}"),
    };
    assert_eq!(bits("A1"), 1.047_197_551_196_597_6_f64.to_bits());
    assert_eq!(bits("A2"), 3.096_867_566_421_062_f64.to_bits());
    assert_eq!(bits("A3"), 0.044_725_087_168_733_47_f64.to_bits());
    assert_eq!(bits("A4"), 2.094_395_102_393_195_7_f64.to_bits());
    assert_eq!(bits("A5"), 0.0_f64.to_bits());
    assert_eq!(bits("A6"), std::f64::consts::PI.to_bits());
    assert_eq!(model._get_text("A7"), *"#NUM!");
}

#[test]
fn asinh_acosh_are_the_log_formulas() {
    // probes-numeric/m9-hyperbolic A1 and A76 (excel-graph, T-1005, T-2031): Excel evaluates
    // ln(x + sqrt(x*x +- 1)) in doubles, losing precision near 0 (ASINH) and 1 (ACOSH).
    let mut model = new_empty_model();
    model._set("A1", "=ASINH(1E-10)");
    model._set("A2", "=ASINH(-1E-10)");
    model._set("A3", "=ACOSH(1.000000000001)");
    model._set("A4", "=ACOSH(0.5)");
    model._set("A5", "=ASINH(1E+200)");
    model.evaluate();
    let bits = |c: &str| match model.get_cell_value_by_ref(&format!("Sheet1!{c}")) {
        Ok(CellValue::Number(n)) => n.to_bits(),
        other => panic!("expected number, got {other:?}"),
    };
    assert_eq!(bits("A1"), 1.000_000_082_690_371e-10_f64.to_bits());
    assert_eq!(bits("A2"), (-1.000_000_082_690_371e-10_f64).to_bits());
    assert_eq!(bits("A3"), 1.414_276_423_083_533e-6_f64.to_bits());
    assert_eq!(model._get_text("A4"), *"#NUM!");
    // x*x overflows: libm's value, since no Excel value is known there.
    assert_eq!(bits("A5"), 1e200_f64.asinh().to_bits());
}
