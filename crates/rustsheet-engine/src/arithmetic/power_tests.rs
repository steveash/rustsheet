use super::{
    divide, extended_product, extended_quotient, extended_sqrt, extended_sum, minus, plus, power,
    times,
};
use crate::expressions::token::Error;

// probes-numeric/double-rounding-ops A33 (excel-graph, T-1496): a sum whose single rounding is
// the next double up.
#[test]
fn sum_rounds_twice() {
    let a = 1.0 + 256_568_457_449_524.0 * f64::EPSILON;
    let b = (1.0 + 26_897_232_841_496.0 * f64::EPSILON) * 2f64.powi(-32);
    assert_eq!(plus(a, b), Ok(1.056_969_642_004_825_3));
    assert_eq!(minus(a, -b), Ok(1.056_969_642_004_825_3));
    assert_eq!(plus(-a, -b), Ok(-1.056_969_642_004_825_3));
    // Mixed signs with partial cancellation, and the 70-bit gap boundary.
    assert_eq!(plus(a, -(b / 2.0)), Ok(a - b / 2.0));
    assert_eq!(extended_sum(1.0, 2f64.powi(-70)), 1.0);
    assert_eq!(extended_sum(1.0, 1e-300), 1.0);
    assert_eq!(extended_sum(0.5, -0.5), 0.0);
    assert_eq!(extended_sum(0.0, 5.0), 5.0);
    assert_eq!(extended_sum(f64::MAX, f64::MAX), f64::INFINITY);
}

// probes-numeric/double-rounding-ops A1 (excel-graph, T-1496): a full-significand product whose
// single rounding is a different double.
#[test]
fn product_rounds_twice() {
    let a = 1.0 + 57_546_701_686_154.0 * f64::EPSILON;
    let b = 1.0 + 791_744_674_158_799.0 * f64::EPSILON;
    assert_eq!(times(a, b), Ok(1.190_826_962_574_648_4));
    assert_eq!(extended_product(-a, b), -1.190_826_962_574_648_4);
    assert_eq!(extended_product(3.0, 0.5), 1.5);
    assert_eq!(extended_product(0.0, 5.0), 0.0);
    assert_eq!(extended_product(1e200, 1e200), f64::INFINITY);
    // probes-numeric/double-rounding-range-edges rows 1 and 6 (excel-graph: #NUM!, 2^-1022).
    let operand = |hi: u64, lo: u64, e: i32| (hi * 67_108_864 + lo) as f64 * 2f64.powi(e);
    let (a, b) = (
        operand(75_071_253, 20_324_045, 460),
        operand(119_982_001, 58_898_702, 459),
    );
    assert_eq!((a * b, extended_product(a, b)), (f64::MAX, f64::INFINITY));
    assert_eq!(extended_product(-a, b), f64::NEG_INFINITY);
    let (c, d) = (
        operand(67_169_229, 53_767_835, -564),
        operand(134_097_104, 60_434_787, -563),
    );
    assert_eq!(extended_product(c, d), f64::MIN_POSITIVE);
}

// Excel's doubles where rounding 1/r once gives the next double up
// (probes-numeric/power-reciprocal-rounding A4, A8, A49, A55, A61, A71, A72; excel-graph).
#[test]
fn division_rounds_twice() {
    let r = 1.0 + 26_955_297_717_044.0 * f64::EPSILON;
    assert_eq!(divide(1.0, r), Ok(0.994_050_331_987_417));
    assert_eq!(power(r, -1.0), Ok(0.994_050_331_987_417));
    let r = 1.0 + 780_397_999_260_180.0 * f64::EPSILON;
    assert_eq!(divide(1.0, r), Ok(0.852_309_169_230_683_7));
    assert_eq!(divide(-1.0, r), Ok(-0.852_309_169_230_683_7));
    let r = 1.0 + 420_768_487_504_071.0 * f64::EPSILON;
    assert_eq!(power(r, -3.0), Ok(0.764_940_740_225_364_1));
    let r = 1.0 + 368_921_494_764_048.0 * f64::EPSILON;
    assert_eq!(power(r, -0.5), Ok(0.961_397_574_339_056_6));
    let r = 1.0 + 795_749_138_188_910.0 * f64::EPSILON;
    assert_eq!(power(r, -0.3), Ok(0.952_360_077_234_193_6));
    assert_eq!(divide(1.0, 2.0 - f64::EPSILON), Ok(0.5));
    assert_eq!(power(8.0, -1.0 / 3.0), Ok(0.5));
    assert_eq!(power(-8.0, -1.0 / 3.0), Ok(-0.5));
}

// Away from a double midpoint the quotient is the correctly rounded one, at every scale.
#[test]
fn extended_quotient_is_the_plain_quotient_elsewhere() {
    for k in 1..2_000_u32 {
        // A divisor with a full significand, as the probe writes it: 1 + k·2^-52.
        let y = 1.0 + f64::from(k * 1_000_003) * f64::EPSILON;
        for x in [1.0, -3.5, 1e300, 7e-300, 0.1] {
            let plain = x / y;
            let twice = extended_quotient(x, y);
            // At most one ulp apart: the second rounding only moves a midpoint tie.
            assert!((twice - plain).abs() <= plain.abs() * f64::EPSILON);
        }
    }
    assert_eq!(extended_quotient(6.0, 3.0), 2.0);
    assert_eq!(extended_quotient(1.0, 3.0), 1.0 / 3.0);
    assert_eq!(extended_quotient(1e-308, 1e10), 1e-308 / 1e10);
    assert!(extended_quotient(1e308, 1e-10).is_infinite());
    assert_eq!(extended_quotient(1.0, f64::INFINITY), 0.0);
}

// Excel, for negative and positive bases alike (probes-numeric/power-negative-base-range,
// power-odd-roots A157-A162).
#[test]
fn integer_powers_out_of_range() {
    assert_eq!(power(-0.1, 401.0), Ok(0.0));
    assert_eq!(power(-0.1, -401.0), Err(Error::DIV));
    assert_eq!(power(-0.1, -400.0), Err(Error::DIV));
    assert_eq!(power(-1e-10, -31.0), Err(Error::DIV));
    assert_eq!(power(-0.5, -1075.0), Err(Error::DIV));
    assert_eq!(power(-10.0, 401.0), Err(Error::NUM));
    assert_eq!(power(-10.0, -401.0), Ok(0.0));
    assert_eq!(power(-2.0, -1074.0), Ok(0.0));
    assert_eq!(power(-1e200, -2.0), Ok(0.0));
}

// Oracle doubles: gen-m1/6462bf910ef9f28f A33, gen-m1/57fe331cb7611dbf A104, gen-m2
// f0d1bacec5de8146 A196 (docs/excel-semantics/numbers.md "Integer exponents").
#[test]
fn integer_exponents_use_square_and_multiply() {
    assert_eq!(power(1e-15, 3.0), Ok(1.000_000_000_000_000_1e-45));
    assert_eq!(power(3.25, 100.0), Ok(1.54289402744788e+51));
    assert_eq!(power(100.0, 100.0), Ok(1.000_000_000_000_000_5e200));
    assert_eq!(power(2.0, -2.0), Ok(0.25));
    assert_eq!(power(-2.0, 3.0), Ok(-8.0));
}

#[test]
fn special_cases_keep_the_general_path() {
    assert_eq!(power(0.0, -1.0), Err(Error::DIV));
    assert_eq!(power(0.0, 0.0), Err(Error::NUM));
    // Excel's cached -1.9999999999999998 (ironcalc-calc POWER.xlsx Sheet1!D3, F9), exp(ln 8/3).
    assert_eq!(power(-8.0, 1.0 / 3.0), Ok(-1.999_999_999_999_999_8));
    assert_eq!(power(10.0, 400.0), Err(Error::NUM));
    assert_eq!(power(-0.0, 0.5), Ok(0.0));
    assert_eq!(power(0.0, -0.5), Err(Error::DIV));
    // The positive power underflows, so Excel's 1/x^|y| is #DIV/0! (sweep A141, A186).
    assert_eq!(power(1e-10, -33.5), Err(Error::DIV));
    assert_eq!(power(0.1, -400.0), Err(Error::DIV));
}

// Excel's doubles, where libm `pow` differs (probes-numeric/power-non-integer-sweep A17, A42,
// A75; gen-m1/6462bf910ef9f28f Calc!A41, A207; gen-m2/1b6ceed7489e0800 Calc!A62).
#[test]
fn non_integer_exponents_use_exp_of_y_ln_x() {
    assert_eq!(power(3.0, -0.5), Ok(0.577_350_269_189_625_8));
    // One ulp below the double nearest √2.
    let below_sqrt_2 = f64::from_bits(std::f64::consts::SQRT_2.to_bits() - 1);
    assert_eq!(power(0.5, -0.5), Ok(below_sqrt_2));
    assert_eq!(power(2.0, -0.5), Ok(0.707_106_781_186_547_5));
    assert_eq!(power(2.0, 0.5), Ok(std::f64::consts::SQRT_2));
    assert_eq!(power(6.0, 2.5), Ok(88.181_630_740_194_38));
    assert_eq!(power(10.0, 33.5), Ok(3.162_277_660_168_401e33));
    assert_eq!(power(10.0, -33.5), Ok(3.162_277_660_168_357_6e-34));
}

// The root is exact where it is representable and the plain `sqrt` away from a double midpoint, at
// odd and even exponents.
#[test]
fn extended_sqrt_is_the_plain_root_away_from_midpoints() {
    for x in [
        4.0,
        2.25,
        1e-300,
        1e300,
        0.5,
        2.0,
        3.0,
        123_456.789,
        1e-5,
        7e22,
    ] {
        assert_eq!(extended_sqrt(x), x.sqrt(), "{x}");
    }
    assert_eq!(extended_sqrt(0.0), 0.0);
    assert!(extended_sqrt(-1.0).is_nan());
    assert_eq!(extended_sqrt(f64::INFINITY), f64::INFINITY);
    assert_eq!(extended_sqrt(5e-324), 5e-324_f64.sqrt());
}
