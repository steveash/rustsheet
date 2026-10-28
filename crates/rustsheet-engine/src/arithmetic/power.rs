//! `x^y` as POWER and the operator `^` compute it: integer exponents by square-and-multiply,
//! others as `exp(y·ln x)` with Excel's own LN and EXP kernels (docs/excel-semantics/numbers.md).

use super::{extended_product, extended_quotient, extended_sqrt, finite_or_num, flush_subnormal};
use crate::{
    expressions::token::Error, number_format::to_excel_precision, x87_exp::excel_exp,
    x87_log10::excel_ln,
};

/// 2^32 - 1: from this exponent on a negative base is `#NUM!` (docs/excel-semantics/numbers.md).
const NEGATIVE_BASE_EXPONENT_LIMIT: f64 = 4_294_967_295.0;

/// `x` to the power of `y`, as POWER and the operator `^` compute it.
pub(crate) fn power(x: f64, y: f64) -> Result<f64, Error> {
    power_with(
        x,
        y,
        PowerSteps {
            ln: excel_ln,
            exp_of_product: |y, ln_x| excel_exp(extended_product(y, ln_x)),
        },
    )
}

/// The `ln x` and `exp(y·ln x)` steps of a non-integer power.
#[derive(Clone, Copy)]
struct PowerSteps {
    ln: fn(f64) -> f64,
    exp_of_product: fn(f64, f64) -> f64,
}

/// [`power`] with `exp(y·ln x)` for a non-integer `y` taken as libm `exp` of a once-rounded
/// product and libm `ln`, the operator `^` before T-0959. MIRR's root keeps it: Mac desktop
/// Excel caches `MIRR({40,-5,-5,-5,-5,-5},0.1,0.2)` as 0.3933280241536672, which this gives, where Excel for
/// the web and [`power`] give 0.3933280241536674 (ironcalc-calc FINANCIAL/MIRR.xlsx Sheet2!N16,
/// probes-numeric/power-mirr-root; decisions 0030 and 0039 keep the passing desktop cell).
pub(crate) fn power_with_libm_exp(x: f64, y: f64) -> Result<f64, Error> {
    power_with(
        x,
        y,
        PowerSteps {
            ln: f64::ln,
            exp_of_product: |y, ln_x| (y * ln_x).exp(),
        },
    )
}

/// [`power`] with `steps` as the `ln x` and `exp(y·ln x)` steps of a non-integer exponent.
fn power_with(x: f64, y: f64, steps: PowerSteps) -> Result<f64, Error> {
    if x == 0.0 && y == 0.0 {
        return Err(Error::NUM);
    }
    if y == 0.0 {
        return Ok(1.0);
    }
    // Excel: a negative base with |y| >= 2^32 - 1 is #NUM!, even for (-1)^y and for results that
    // underflow (docs/excel-semantics/numbers.md "Integer exponents beyond 360").
    if x < 0.0 && y.abs() >= NEGATIVE_BASE_EXPONENT_LIMIT {
        return Err(Error::NUM);
    }
    if let Some(result) = integer_power(x, y) {
        return result;
    }
    let result = if x < 0.0 && is_one_over_an_odd_integer(y) {
        // An odd root of a negative number is a real number, minus the root of the magnitude:
        // `(-8)^(1/3)` is -1.9999999999999998 (POWER.xlsx Sheet1!D3, F9). The exponent is the one
        // given, not 1/n: `(-8)^(1/3+8·2^-54)` is -2.0000000000000018.
        -non_integer_power(-x, y, steps)
    } else {
        non_integer_power(x, y, steps)
    };
    if result.is_infinite() {
        // POWER(0,-1) is #DIV/0! (POWER.xlsx Sheet1!D2), and so is any negative exponent whose
        // positive power underflows: Excel divides 1 by it (`1E-10^-33.5`, `0.1^-400`,
        // probes-numeric/power-non-integer-sweep A141, A186-A191). Overflow of a positive exponent
        // is #NUM!.
        return Err(if x == 0.0 || y < 0.0 {
            Error::DIV
        } else {
            Error::NUM
        });
    }
    if result.is_nan() {
        // A negative number to a power that is not an integer nor an odd root
        return Err(Error::NUM);
    }
    finite_or_num(result)
}

/// `x^y` for a non-integer `y`, as Excel computes it: `exp(y·ln x)` (`steps`; for [`power`] LN's
/// kernel [`excel_ln`], T-2694, and EXP's kernel [`excel_exp`] on a twice-rounded product), a
/// negative exponent as one over the positive power, and `y` = ±0.5 through [`extended_sqrt`]. This
/// matches all 180 numeric oracle cells over 14 bases and 13 exponents, where libm `pow` matches 93
/// (probes-numeric/power-non-integer-sweep), and `1E300^(1/3)` = `EXP(LN(1E300)*(1/3))` =
/// 9.999999999999827E+99, where libm `exp` is 2 ulps lower (probes-numeric/power-ulp-sources A13;
/// docs/excel-semantics/numbers.md "Non-integer exponents: exp(y·ln x)"). A negative `x` gives NaN.
fn non_integer_power(x: f64, y: f64, steps: PowerSteps) -> f64 {
    // -0 has the logarithm of 0: `-0^0.5` is 0, not `sqrt`'s -0 (gen-m1/6462bf910ef9f28f Calc!A19).
    let x = if x == 0.0 { 0.0 } else { x };
    if y == 0.5 {
        extended_sqrt(x)
    } else if y == -0.5 {
        // One over the twice-rounded root, divided like `/`: `x^-0.5`, `POWER(x,-0.5)` and
        // `1/SQRT(x)` agree (probes-numeric/power-neg-half-rounding, T-1541).
        extended_quotient(1.0, extended_sqrt(x))
    } else if y > 0.0 {
        positive_power(x, y, steps)
    } else {
        extended_quotient(1.0, positive_power(x, -y, steps))
    }
}

/// `exp(y·ln x)` for a positive `y`, where a product `y·ln x` that overflows is an overflow even
/// when it is -∞: `POWER(0.1,9.99999999999999E+307)` is `#NUM!` and
/// `POWER(0.1,-9.99999999999999E+307)` is 0, while `POWER(0.5,9.99999999999999E+307)` (a finite
/// product, -6.9E+307) is 0 (probes/power-huge-exponent A61-A62, A81; numbers.md "POWER when
/// y·ln x overflows").
fn positive_power(x: f64, y: f64, steps: PowerSteps) -> f64 {
    // ln 0 is -∞ (libm's, where `excel_ln` is NaN): `POWER(0,9.99999999999999E+307)` stays 0
    // (A281).
    let ln_x = if x == 0.0 {
        f64::NEG_INFINITY
    } else {
        (steps.ln)(x)
    };
    if ln_x.is_finite() && (y * ln_x).is_infinite() {
        f64::INFINITY
    } else {
        (steps.exp_of_product)(y, ln_x)
    }
}

/// `x^y` for an integer `y`, by binary exponentiation scanning the bits of `|y|` from the low end,
/// as Excel computes it (docs/excel-semantics/numbers.md "Integer exponents: right-to-left
/// square-and-multiply"; libm `pow` and `powi` round differently). A negative `y` is one over
/// the positive power, divided like the operator `/`. `None` leaves a non-integer or huge `y` to
/// the general path (bounding the loop to 54 iterations).
///
/// Out of range, for any sign of `x` (probes-numeric/power-negative-base-range,
/// power-odd-roots A157-A162): a positive power that overflows is `#NUM!` and one that underflows
/// is 0, so `(-0.1)^401` is 0; a negative exponent whose positive power underflows is `#DIV/0!`
/// (`(-0.1)^-401`, `(-0.5)^-1075`), and one whose positive power overflows is 0 (`(-10)^-401`).
fn integer_power(x: f64, y: f64) -> Option<Result<f64, Error>> {
    // 2^53: beyond it every float is an integer, and the loop would need more than 54 iterations
    const MAX_EXPONENT: f64 = 9_007_199_254_740_992.0;
    if !x.is_finite() || !y.is_finite() || y.fract() != 0.0 || y.abs() > MAX_EXPONENT {
        return None;
    }
    let positive_power =
        decimal_power_of_ten(x, y.abs()).unwrap_or_else(|| square_and_multiply(x, y.abs()));
    let result = if y > 0.0 {
        if positive_power.is_infinite() {
            return Some(Err(Error::NUM));
        }
        positive_power
    } else if positive_power.abs() < f64::MIN_POSITIVE {
        // Excel divides 1 by the positive power, and a subnormal divisor counts as zero
        // (`1E-10^-31`, numbers.md "Non-integer exponents: exp(y·ln x)").
        return Some(Err(Error::DIV));
    } else {
        extended_quotient(1.0, positive_power)
    };
    // Excel flushes a subnormal result to zero (T-1311 oracle: 0.1^310, 2^-1074, (-0.5)^1073).
    Some(Ok(flush_subnormal(result)))
}

/// `x^n` for a non-negative integer `n` <= 2^53 by scanning the bits of `n` from the low end.
/// Each product is rounded twice, like the operator `*` ([`extended_product`]): with products
/// rounded once the loop misses 18 of the 20 `SQRT(5)^n` cells (numbers.md "Integer exponents
/// beyond 360", T-1524 fit lab `power-sqrt5`).
fn square_and_multiply(x: f64, n: f64) -> f64 {
    let mut remaining = n;
    let mut base = x;
    let mut result = 1.0_f64;
    while remaining >= 1.0 {
        if remaining % 2.0 == 1.0 {
            result = extended_product(result, base);
        }
        base = extended_product(base, base);
        remaining = (remaining / 2.0).floor();
    }
    result
}

/// `10^n` for an integer `n` in 23..=308 as the nearest double to the decimal `1En`, which is what
/// Excel returns for base 10 there (square-and-multiply drifts from it: `10^33` is exactly 1E+33
/// in Excel). Smaller exponents are exact either way, and `10^23` is the square-and-multiply
/// value. Other bases, even `100` and `-10`, follow the square-and-multiply rule.
fn decimal_power_of_ten(x: f64, n: f64) -> Option<f64> {
    (x == 10.0 && (23.0..=308.0).contains(&n))
        .then(|| format!("1e{n}").parse::<f64>().ok())
        .flatten()
}

/// Whether `y` is 1/3, 1/5, -1/7 and so on, as Excel decides it: one over `y`, rounded to 15
/// significant digits, is an odd integer below 2^31 - 1 in magnitude. So `1/3 + 10·2^-54` counts
/// (1/y = 2.9999999999999951 → 3) and `1/3 - 10·2^-54` does not (3.0000000000000053 →
/// 3.00000000000001); `0.333333333333333` counts, `0.33333333333333` does not; `1/(2^31-3)`
/// counts, `1/(2^31-1)` does not (docs/excel-semantics/numbers.md "POWER and `^`",
/// probes-numeric/power-odd-root-tolerance, power-odd-root-tolerance-2).
fn is_one_over_an_odd_integer(y: f64) -> bool {
    // 2^31 - 1: the first odd denominator Excel no longer takes for a root
    const LIMIT: f64 = 2_147_483_647.0;
    let inverse = 1.0 / y;
    // Cheap rejection first: far from an integer, 15 digits cannot round to one (and formatting
    // to 15 digits allocates).
    if inverse.abs() >= LIMIT || (inverse - inverse.round()).abs() > inverse.abs() * 1e-12 {
        return false;
    }
    let inverse = to_excel_precision(inverse, 15);
    inverse.fract() == 0.0 && inverse.abs() < LIMIT && inverse % 2.0 != 0.0
}
