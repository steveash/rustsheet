use crate::{
    calc_result::CalcResult,
    cast::{
        array_node_to_string, array_node_to_text, NumberOrArray, StringOrArray, ValueOrArray,
        MAX_TEXT_UNITS,
    },
    expressions::{
        parser::{left_chain, ArrayNode, Node},
        token::{Error, OpCompare, OpProduct, OpSum},
        types::CellReferenceIndex,
    },
    functions::util::compare_values,
    limits::check_array_size,
    model::Model,
    text::Text,
};

/// Maps an output index `i` back to the source array index, applying Excel's
/// size-1 broadcasting rule: a length-1 dimension repeats its only element, an
/// equal-length dimension maps one-to-one, and any other mismatch is out of range.
pub(crate) fn bcast_idx(len: usize, i: usize) -> Option<usize> {
    if len == 1 {
        Some(0)
    } else if i < len {
        Some(i)
    } else {
        None
    }
}

/// The element at (`row`, `col`) of `grid` under [`bcast_idx`] broadcasting on each axis, using the
/// length of the addressed row for the column axis; `None` where either index is out of range.
pub(crate) fn bcast_get<T>(grid: &[Vec<T>], row: usize, col: usize) -> Option<&T> {
    let line = grid.get(bcast_idx(grid.len(), row)?)?;
    line.get(bcast_idx(line.len(), col)?)
}

/// `x + y`; a sum beyond the double range is `#NUM!` at once, like a product
/// (probes/arith-overflow Calc!B1:B5, excel-graph).
pub(crate) fn plus(x: f64, y: f64) -> Result<f64, Error> {
    finite_or_overflow(extended_sum(x, y))
}

/// `x - y`; a difference beyond the double range is `#NUM!` (probes/arith-overflow Calc!B2:B3).
/// A subnormal difference is kept, unlike a product's: `(3E-308-2.5E-308)=0` is FALSE
/// (probes/subnormal-function-results C69, T-1027).
pub(crate) fn minus(x: f64, y: f64) -> Result<f64, Error> {
    finite_or_overflow(extended_sum(x, -y))
}

/// `value`, or `#NUM!` beyond the double range; subnormals are kept (addition, subtraction).
fn finite_or_overflow(value: f64) -> Result<f64, Error> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(Error::NUM)
    }
}

/// `x * y`; a product beyond the double range is `#NUM!` at once, so it stays an error through
/// `&`, CONCAT, TEXT and LEN (probes-numeric/4f413cc257ce15c4 Calc!A121:A124, excel-graph).
pub(crate) fn times(x: f64, y: f64) -> Result<f64, Error> {
    finite_or_num(extended_product(x, y))
}

/// `x / y`. A divisor below the smallest normal double counts as zero, so `1/1E-308` and
/// `1/2^-1023` are `#DIV/0!` while `1/2^-1022` is finite; a quotient beyond the double range is
/// `#NUM!` (probes/arith-overflow Calc!A6:A8, A16:A18, excel-graph).
pub(crate) fn divide(x: f64, y: f64) -> Result<f64, Error> {
    if y.abs() < f64::MIN_POSITIVE {
        return Err(Error::DIV);
    }
    finite_or_num(extended_quotient(x, y))
}

/// `x / y` rounded twice, first to a 64-bit significand and then to a double, as an x87 division
/// in extended precision stored to a double would be. It differs from the correctly rounded
/// quotient (about once in 4,000 random divisions) only when the exact quotient lies within half
/// an extended ulp of a double midpoint: the first rounding lands on the midpoint and the second
/// rounds it to even. `1/(2-2^-52)` is 0.5, not 0.5000000000000001, and so is `8^(-1/3)`
/// (docs/excel-semantics/numbers.md "Division rounds twice", probes-numeric/power-reciprocal-rounding).
/// A quotient that is not a normal double, and any non-finite or zero operand, is the plain `x / y`.
pub(crate) fn extended_quotient(x: f64, y: f64) -> f64 {
    let plain = x / y;
    if !plain.is_normal() || !x.is_normal() || !y.is_normal() {
        return plain;
    }
    let (x_significand, x_exponent) = significand_and_exponent(x);
    let (y_significand, y_exponent) = significand_and_exponent(y);
    // Both significands are 53-bit integers, so the integer quotient of x << 74 by y has 74 or 75
    // bits, and the remainder makes the result exact enough to round to 64 bits.
    let numerator = u128::from(x_significand) << 74;
    let divisor = u128::from(y_significand);
    let quotient = numerator / divisor;
    let inexact = numerator % divisor != 0;
    // First rounding: to 64 bits, ties to even, the remainder breaking ties.
    let (extended, shift) = round_to_bits(quotient, 64, inexact);
    let exponent = x_exponent - y_exponent - 74 + shift;
    round_extended_to_double(
        extended,
        exponent,
        x.is_sign_negative() != y.is_sign_negative(),
    )
    .unwrap_or(plain)
}

/// `sqrt(x)` rounded twice, first to a 64-bit significand and then to a double, like
/// [`extended_quotient`] (docs/excel-semantics/numbers.md "Division rounds twice",
/// probes-numeric/double-rounding-ops A65-A80, excel-graph). The root is built bit by bit from the
/// exact significand, so the first rounding sees the true root. A subnormal `x` rounds twice too
/// (probes-numeric/power-neg-half-rounding A9-A12, B9-B12; T-1541). A zero, negative or
/// non-finite `x` is the plain `x.sqrt()`.
pub(crate) fn extended_sqrt(x: f64) -> f64 {
    let plain = x.sqrt();
    if x <= 0.0 || !x.is_finite() {
        return plain;
    }
    let (significand, mut exponent) = significand_and_exponent(x);
    let mut radicand = u128::from(significand);
    if exponent % 2 != 0 {
        // The exponent must be even to halve it; the significand then has 54 bits.
        radicand <<= 1;
        exponent -= 1;
    }
    // 27 bit pairs of the (zero-padded to 54 bits) radicand, then 39 zero pairs: a 66-bit root
    // `root·2^-39` of the significand, and `remainder != 0` says it is only the floor.
    let (mut root, mut remainder) = (0u128, 0u128);
    for pair in (0..66).rev() {
        let digits = if pair >= 39 {
            (radicand >> (2 * (pair - 39))) & 3
        } else {
            0
        };
        remainder = (remainder << 2) | digits;
        let trial = (root << 2) | 1;
        root <<= 1;
        if remainder >= trial {
            remainder -= trial;
            root |= 1;
        }
    }
    let (extended, shift) = round_to_bits(root, 64, remainder != 0);
    round_extended_to_double(extended, exponent / 2 - 39 + shift, false).unwrap_or(plain)
}

/// `x * y` rounded twice, first to a 64-bit significand and then to a double, like
/// [`extended_quotient`] (docs/excel-semantics/numbers.md "Division rounds twice",
/// probes-numeric/double-rounding-ops A1-A16, excel-graph). The exact 106-bit product is rounded to
/// 64 bits and then to 53. A zero or non-finite operand or product is the plain `x * y`.
/// The range edges follow the same two roundings (probes-numeric/double-rounding-range-edges): a
/// product just under the midpoint between `f64::MAX` and 2^1024 rounds up to 2^1024 and is
/// infinite (`#NUM!`), and one just under the midpoint between the largest subnormal and 2^-1022
/// is 2^-1022. A subnormal product is rounded on the 2^-1074 grid (callers flush it to 0 unless it reaches 2^-1022).
pub(crate) fn extended_product(x: f64, y: f64) -> f64 {
    let plain = x * y;
    if plain == 0.0 || !plain.is_finite() || !x.is_normal() || !y.is_normal() {
        return plain;
    }
    let (x_significand, x_exponent) = significand_and_exponent(x);
    let (y_significand, y_exponent) = significand_and_exponent(y);
    let product = u128::from(x_significand) * u128::from(y_significand);
    let (extended, shift) = round_to_bits(product, 64, false);
    let negative = x.is_sign_negative() != y.is_sign_negative();
    match round_extended_to_double(extended, x_exponent + y_exponent + shift, negative) {
        Some(rounded) => rounded,
        // A plain `f64::MAX` with no double-rounded double means the exponent ran past the top.
        None if plain.abs() == f64::MAX => f64::INFINITY.copysign(plain),
        None if plain.abs() < f64::MIN_POSITIVE => {
            round_extended_to_subnormal(extended, x_exponent + y_exponent + shift, negative)
                .unwrap_or(plain)
        }
        None => plain,
    }
}

/// The 64-bit `extended·2^exponent` rounded, ties to even, to a multiple of 2^-1074 (a subnormal
/// has fewer than 53 bits, so a tie of the second rounding is a tie on that grid, not on 53 bits),
/// or `None` when the grid is not coarser than the value or the result is zero.
fn round_extended_to_subnormal(extended: u128, exponent: i32, negative: bool) -> Option<f64> {
    let drop = u32::try_from(-1074 - exponent)
        .ok()
        .filter(|d| (1..128).contains(d))?;
    let mut units = extended >> drop;
    let dropped = extended & ((1 << drop) - 1);
    let half = 1 << (drop - 1);
    if dropped > half || (dropped == half && units & 1 == 1) {
        units += 1;
    }
    let units = u64::try_from(units)
        .ok()
        .filter(|u| *u != 0 && *u <= 1 << 52)?;
    // `units` is at most 2^52, so it converts exactly and 2^-1074 is a subnormal power of two.
    let magnitude = units as f64 * f64::from_bits(1);
    Some(if negative { -magnitude } else { magnitude })
}

/// Whether `plain = x + y` can differ from the twice-rounded sum. The rounding error `e` of the
/// addition is found exactly (Knuth's `TwoSum`); the first rounding to 64 bits can land on a double
/// midpoint, and so change the result, only when `|e|` is within 2^-8 of half a unit in the last
/// place of the sum (or of a quarter, below a power of two, where the next lower double is closer).
/// An exact sum and every other error keep `plain`, so the 128-bit path runs for few additions.
fn sum_may_round_twice(x: f64, y: f64, plain: f64) -> bool {
    let virtual_y = plain - x;
    let error = (x - (plain - virtual_y)) + (y - virtual_y);
    if error == 0.0 {
        return false;
    }
    // The unit in the last place of `plain`, a power of two (`plain` is normal).
    let ulp = f64::from_bits(plain.to_bits() & (0x7FF << 52)) * f64::EPSILON;
    let ratio = error.abs() / ulp;
    (ratio - 0.5).abs() <= 1.0 / 256.0 || (ratio - 0.25).abs() <= 1.0 / 256.0
}

/// `x + y` rounded twice, first to a 64-bit significand and then to a double, like
/// [`extended_quotient`] (docs/excel-semantics/numbers.md "Division rounds twice",
/// probes-numeric/double-rounding-ops A33-A64, excel-graph). The sum is formed exactly in 128 bits.
/// These give the plain `x + y`: a zero, subnormal or non-finite operand or result; a difference
/// that cancels down to 53 bits or fewer (exact in a double); and operands whose lowest-bit
/// exponents are 70 or more apart (the smaller lies far under half a 64-bit ulp of the larger, so
/// both roundings return the larger operand, which is also the plain sum).
pub(crate) fn extended_sum(x: f64, y: f64) -> f64 {
    let plain = x + y;
    if !plain.is_normal() || !x.is_normal() || !y.is_normal() || !sum_may_round_twice(x, y, plain) {
        return plain;
    }
    let (x_significand, x_exponent) = significand_and_exponent(x);
    let (y_significand, y_exponent) = significand_and_exponent(y);
    // `high` has the larger exponent for its lowest bit; `low` is aligned down to it.
    let (
        (high_significand, high_exponent, high_negative),
        (low_significand, low_exponent, low_negative),
    ) = if x_exponent >= y_exponent {
        (
            (x_significand, x_exponent, x < 0.0),
            (y_significand, y_exponent, y < 0.0),
        )
    } else {
        (
            (y_significand, y_exponent, y < 0.0),
            (x_significand, x_exponent, x < 0.0),
        )
    };
    let Ok(gap) = u32::try_from(high_exponent - low_exponent) else {
        return plain;
    };
    if gap >= 70 {
        return plain;
    }
    let aligned = u128::from(high_significand) << gap;
    let low_magnitude = u128::from(low_significand);
    let (magnitude, negative) = if high_negative == low_negative {
        (aligned + low_magnitude, high_negative)
    } else if aligned >= low_magnitude {
        (aligned - low_magnitude, high_negative)
    } else {
        (low_magnitude - aligned, low_negative)
    };
    // Cancellation can leave fewer than 53 bits (below 2^52), which a double holds exactly: the
    // plain sum.
    if magnitude >> 52 == 0 {
        return plain;
    }
    let (extended, shift) = round_to_bits(magnitude, 64, false);
    // A double-rounded value past the range (no probe reaches it) falls back to the plain sum.
    round_extended_to_double(extended, low_exponent + shift, negative).unwrap_or(plain)
}

/// The 64-bit `extended·2^exponent` rounded to a double (second rounding: the value is exact now,
/// so a tie is a tie), or `None` when the result is not a normal double.
fn round_extended_to_double(extended: u128, exponent: i32, negative: bool) -> Option<f64> {
    let (significand, shift) = round_to_bits(extended, 53, false);
    let biased = u64::try_from(exponent + shift + 1075)
        .ok()
        .filter(|b| (1..=2046).contains(b))?;
    let sign = u64::from(negative) << 63;
    // The significand has 53 bits after the second rounding, so the conversion cannot fail.
    let fraction = u64::try_from(significand).unwrap_or(0) & 0x000F_FFFF_FFFF_FFFF;
    Some(f64::from_bits(sign | (biased << 52) | fraction))
}

/// The 53-bit integer significand `m` and exponent `e` of a non-zero finite double,
/// `|x| = m·2^e`; a subnormal's significand is shifted up to 53 bits.
fn significand_and_exponent(x: f64) -> (u64, i32) {
    let bits = x.to_bits();
    let fraction = bits & 0x000F_FFFF_FFFF_FFFF;
    let biased = i32::try_from((bits >> 52) & 0x7FF).unwrap_or(0);
    if biased == 0 {
        // The top set bit of the fraction moves to bit 52 (a u64 then has 11 leading zeros).
        let shift = fraction.leading_zeros().saturating_sub(11);
        return (fraction << shift, -1074 - i32::try_from(shift).unwrap_or(0));
    }
    (fraction | (1 << 52), biased - 1075)
}

/// `value` (non-zero) rounded to nearest, ties to even, to `bits` significant bits: the rounded
/// significand and the number of low bits dropped, so the result is `significand·2^shift`.
/// `inexact` says the exact value lies a little above `value`, which turns a tie into a round up.
fn round_to_bits(value: u128, bits: u32, inexact: bool) -> (u128, i32) {
    let width = 128 - value.leading_zeros();
    if width <= bits {
        return (value, 0);
    }
    let shift = width - bits;
    let mut significand = value >> shift;
    let dropped = value & ((1 << shift) - 1);
    let half = 1 << (shift - 1);
    if dropped > half || (dropped == half && (inexact || significand & 1 == 1)) {
        significand += 1;
    }
    // At most 75 - 53 bits are dropped, so the conversion cannot fail.
    let mut shift = i32::try_from(shift).unwrap_or(0);
    // A carry out of the top bit (all ones rounded up) is one more bit to drop.
    if significand >> bits != 0 {
        significand >>= 1;
        shift += 1;
    }
    (significand, shift)
}

/// An infinite (or NaN) result is the error `#NUM!`; a result below the smallest normal double
/// is 0 (docs/excel-semantics/numbers.md "Subnormal results become 0", probes/precision-and-rounding A14).
fn finite_or_num(value: f64) -> Result<f64, Error> {
    finite_or_overflow(value).map(flush_subnormal)
}

/// 0 for a value below the smallest normal double, `value` otherwise. The postfix `%` and the
/// multiplying and dividing functions (PRODUCT, SUMPRODUCT, SUMSQ, AVERAGE) flush their result
/// like `*` and `/`; addition keeps subnormals (docs/excel-semantics/numbers.md "Subnormal results become 0",
/// probes/subnormal-function-results, T-0189).
pub(crate) fn flush_subnormal(value: f64) -> f64 {
    if value.abs() < f64::MIN_POSITIVE {
        0.0
    } else {
        value
    }
}

/// Excel's near-zero snapping of a formula's final addition: `total` is `x + y`, and when the two
/// addends have opposite signs and `total` is below 8 ULP of the smaller addend in magnitude,
/// Excel returns exactly 0. It applies when the cell formula's outermost operation is `+`/`-`
/// (not when the whole formula is parenthesized) and to every SUM call (the callers decide that).
/// Across the 1,296 probe cells, snapped results are at most 7 ULP of the smaller addend and kept ones at
/// least 8 (docs/excel-semantics/numbers.md "Near-zero snapping in addition and subtraction",
/// probes-numeric/30bc5f8fab02d0a2).
pub(crate) fn snap_near_zero(x: f64, y: f64, total: f64) -> f64 {
    if x == 0.0 || y == 0.0 || x.is_sign_negative() == y.is_sign_negative() {
        return total;
    }
    let smaller = x.abs().min(y.abs());
    // 2^floor(log2(smaller)) for a normal double (its exponent bits alone); 0 for a subnormal.
    let scale = f64::from_bits(smaller.to_bits() & 0x7FF0_0000_0000_0000);
    // 8 ULP of `smaller` is scale * 2^-49.
    if total.abs() < scale * 2f64.powi(-49) {
        0.0
    } else {
        total
    }
}

/// 2^32 - 1: from this exponent on a negative base is `#NUM!` (docs/excel-semantics/numbers.md).
const NEGATIVE_BASE_EXPONENT_LIMIT: f64 = 4_294_967_295.0;

/// `x` to the power of `y`, as POWER and the operator `^` compute it.
pub(crate) fn power(x: f64, y: f64) -> Result<f64, Error> {
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
        // `(-8)^(1/3)` is -1.9999999999999998 (POWER.xlsx Sheet1!D3, F9; tolerance UNVERIFIED, see
        // docs/excel-semantics/numbers.md).
        -non_integer_power(-x, y)
    } else {
        non_integer_power(x, y)
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

/// `x^y` for a non-integer `y`, as Excel computes it: `exp(y·ln x)`, a negative exponent as one over
/// the positive power, and `y` = ±0.5 through [`extended_sqrt`]. This matches all 180 numeric
/// oracle cells over 14 bases and 13 exponents, where libm `pow` matches 93 (probes-numeric/power-non-integer-sweep;
/// docs/excel-semantics/numbers.md "Non-integer exponents: exp(y·ln x)"). A negative `x` gives NaN.
fn non_integer_power(x: f64, y: f64) -> f64 {
    // -0 has the logarithm of 0: `-0^0.5` is 0, not `sqrt`'s -0 (gen-m1/6462bf910ef9f28f Calc!A19).
    let x = if x == 0.0 { 0.0 } else { x };
    if y == 0.5 {
        extended_sqrt(x)
    } else if y == -0.5 {
        // One over the twice-rounded root, divided like `/`: `x^-0.5`, `POWER(x,-0.5)` and
        // `1/SQRT(x)` agree (probes-numeric/power-neg-half-rounding, T-1541).
        extended_quotient(1.0, extended_sqrt(x))
    } else if y > 0.0 {
        (y * x.ln()).exp()
    } else {
        extended_quotient(1.0, (-y * x.ln()).exp())
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
fn square_and_multiply(x: f64, n: f64) -> f64 {
    let mut remaining = n;
    let mut base = x;
    let mut result = 1.0_f64;
    while remaining >= 1.0 {
        if remaining % 2.0 == 1.0 {
            result *= base;
        }
        base *= base;
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

/// Whether `y` is 1/3, 1/5, -1/7 and so on. A float cannot hold a third, so
/// this asks whether one over `y` is an odd integer as far as the precision of
/// a float can tell.
fn is_one_over_an_odd_integer(y: f64) -> bool {
    // 2^-48: the last few bits are left to the error of the two divisions
    const TOLERANCE: f64 = 1.0 / (16_777_216.0 * 16_777_216.0);
    let inverse = 1.0 / y;
    let integer = inverse.round();
    (inverse - integer).abs() <= integer.abs() * TOLERANCE && integer % 2.0 != 0.0
}

/// What an array has beyond its end, when it is combined with a larger one:
/// `#N/A`, as in Excel. It is an error like any other, so if the element on
/// the other side is an error too, the one on the left comes first.
pub(crate) static MISSING: ArrayNode = ArrayNode::Error(Error::NA);

/// The error an element holds, if it is one: the `element_error` of [`array_of_errors`] for an
/// array whose elements are not coerced.
pub(crate) fn own_error(node: &ArrayNode) -> Option<Error> {
    match node {
        ArrayNode::Error(error) => Some(error.clone()),
        _ => None,
    }
}

/// What an operator, or a function that works element by element, gives when
/// one operand is an array and the other one an error: an array of the same
/// size with an error in every element. The error of the left operand comes
/// first, so if the array is on the left `element_error` says which of its
/// elements are errors of their own.
pub(crate) fn array_of_errors(
    array: &[Vec<ArrayNode>],
    error: CalcResult,
    element_error: impl Fn(&ArrayNode) -> Option<Error>,
) -> CalcResult {
    let CalcResult::Error { error: kind, .. } = &error else {
        return error;
    };
    CalcResult::Array(
        array
            .iter()
            .map(|row| {
                row.iter()
                    .map(|node| ArrayNode::Error(element_error(node).unwrap_or(kind.clone())))
                    .collect()
            })
            .collect(),
    )
}

/// Combines two arrays element by element, as a binary operator does: the result has the larger
/// of the two heights and widths, a size-1 dimension repeats its only element and a position past
/// the end of the shorter array is [`MISSING`]. A result above the engine's array limit is the
/// limit's error before anything is allocated: a whole column against a whole row (`=D:D+4:4`)
/// would be 2^34 cells (DEVIATION, docs/excel-semantics/arrays.md "Array results above 1,048,576
/// cells").
pub(crate) fn broadcast_arrays(
    a1: &[Vec<ArrayNode>],
    a2: &[Vec<ArrayNode>],
    cell: CellReferenceIndex,
    mut combine: impl FnMut(&ArrayNode, &ArrayNode) -> ArrayNode,
) -> CalcResult {
    let n1 = a1.len();
    let m1 = a1.first().map_or(0, Vec::len);
    let n2 = a2.len();
    let m2 = a2.first().map_or(0, Vec::len);
    let rows = n1.max(n2);
    let cols = m1.max(m2);
    if let Some((error, message)) = check_array_size(rows, cols) {
        return CalcResult::new_error(error, cell, message);
    }
    let mut array = Vec::with_capacity(rows);
    for i in 0..rows {
        let row1 = bcast_idx(n1, i).and_then(|ri| a1.get(ri));
        let row2 = bcast_idx(n2, i).and_then(|ri| a2.get(ri));
        let mut data_row = Vec::with_capacity(cols);
        for j in 0..cols {
            let v1 = row1
                .and_then(|r| bcast_idx(m1, j).and_then(|cj| r.get(cj)))
                .unwrap_or(&MISSING);
            let v2 = row2
                .and_then(|r| bcast_idx(m2, j).and_then(|cj| r.get(cj)))
                .unwrap_or(&MISSING);
            data_row.push(combine(v1, v2));
        }
        array.push(data_row);
    }
    CalcResult::Array(array)
}

impl Model {
    /// Evaluates a cell formula's outermost node. Excel snaps a near-zero result of the formula's
    /// outermost `+`/`-` to 0, but not for a wholly parenthesized formula, which is a
    /// root `Group` (docs/excel-semantics/numbers.md "Near-zero snapping in addition
    /// and subtraction"); every other node evaluates as usual.
    pub(crate) fn evaluate_formula_root(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        match node {
            Node::OpSumKind { kind, left, right } => {
                let left = self.evaluate_node_in_context(left, cell);
                self.handle_outermost_sum(left, right, cell, matches!(kind, OpSum::Minus))
            }
            _ => self.evaluate_node_in_context(node, cell),
        }
    }

    /// A formula's outermost `+`/`-` over its evaluated left operand: like
    /// [`Model::handle_arithmetic`], but a scalar result is snapped to 0 by [`snap_near_zero`].
    /// Array operands are not snapped (no evidence).
    fn handle_outermost_sum(
        &mut self,
        left: CalcResult,
        right: &Node,
        cell: CellReferenceIndex,
        subtract: bool,
    ) -> CalcResult {
        let op: &dyn Fn(f64, f64) -> Result<f64, Error> = if subtract { &minus } else { &plus };
        let l = self.number_or_array(left, cell);
        let r = self.get_number_or_array(right, cell);
        // The two addends of a scalar addition (a subtraction adds the negated right operand).
        let addends = match (&l, &r) {
            (Ok(NumberOrArray::Number(x)), Ok(NumberOrArray::Number(y))) => {
                Some((*x, if subtract { -*y } else { *y }))
            }
            _ => None,
        };
        match (self.arithmetic_on_values(l, r, cell, op), addends) {
            (CalcResult::Number(total), Some((x, y))) => {
                CalcResult::Number(snap_near_zero(x, y, total))
            }
            (result, _) => result,
        }
    }

    /// Evaluates a chain of value operators (`=A1+A2*3&"x"`) in a loop down its
    /// left operands (T-0128): the first operand, then each operator over the
    /// result so far and its right operand, the same order and results as
    /// recursing into each left operand, on one stack frame however long the
    /// chain is.
    pub(crate) fn evaluate_operator_chain(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        let (links, first) = left_chain(node);
        let mut value = self.evaluate_node_in_context(first, cell);
        for link in links {
            value = match link {
                Node::OpSumKind { kind, right, .. } => match kind {
                    OpSum::Add => self.handle_arithmetic(value, right, cell, &plus),
                    OpSum::Minus => self.handle_arithmetic(value, right, cell, &minus),
                },
                Node::OpProductKind { kind, right, .. } => match kind {
                    OpProduct::Times => self.handle_arithmetic(value, right, cell, &times),
                    OpProduct::Divide => self.handle_arithmetic(value, right, cell, &divide),
                },
                Node::OpPowerKind { right, .. } => {
                    self.handle_arithmetic(value, right, cell, &power)
                }
                Node::OpConcatenateKind { right, .. } => {
                    self.handle_concatenate(value, right, cell)
                }
                Node::CompareKind { kind, right, .. } => {
                    self.handle_comparison(value, right, cell, kind)
                }
                // `left_chain` links are value operators only.
                _ => value,
            };
        }
        value
    }

    /// Applies `op` element‐wise for arrays/numbers to `left`, the evaluated left
    /// operand, and `right`.
    pub(crate) fn handle_arithmetic(
        &mut self,
        left: CalcResult,
        right: &Node,
        cell: CellReferenceIndex,
        op: &dyn Fn(f64, f64) -> Result<f64, Error>,
    ) -> CalcResult {
        let l = self.number_or_array(left, cell);
        let r = self.get_number_or_array(right, cell);
        self.arithmetic_on_values(l, r, cell, op)
    }

    /// Applies `op` element‐wise to operands that are already evaluated. An
    /// operand that is an error makes the result an error, or an array of
    /// errors if the other operand is an array.
    pub(crate) fn arithmetic_on_values(
        &mut self,
        l: Result<NumberOrArray, CalcResult>,
        r: Result<NumberOrArray, CalcResult>,
        cell: CellReferenceIndex,
        op: &dyn Fn(f64, f64) -> Result<f64, Error>,
    ) -> CalcResult {
        let (l, r) = match (l, r) {
            (Ok(l), Ok(r)) => (l, r),
            // A scalar error against an array operand is lifted element-wise like
            // any other scalar: `#VALUE!-{1;2}` is `{#VALUE!;#VALUE!}`, so the
            // result keeps the array's size (gen-m2/dd99be1f2454159f Calc!A56 is
            // #SPILL!, not #VALUE!; docs/excel-semantics/arrays.md). The left
            // operand's error comes first, so an error element of a left array
            // stays (error_broadcast.xlsx E22).
            (Err(error), Ok(NumberOrArray::Array(array))) => {
                return array_of_errors(&array, error, |_| None);
            }
            (Ok(NumberOrArray::Array(array)), Err(error)) => {
                return array_of_errors(&array, error, |node| {
                    self.array_node_to_number(node).err()
                });
            }
            (Err(error), _) | (_, Err(error)) => return error,
        };
        match (l, r) {
            // -----------------------------------------------------
            // Case 1: Both are numbers
            // -----------------------------------------------------
            (NumberOrArray::Number(f1), NumberOrArray::Number(f2)) => match op(f1, f2) {
                Ok(x) => CalcResult::Number(x),
                Err(Error::DIV) => CalcResult::Error {
                    error: Error::DIV,
                    origin: cell,
                    message: "Divide by 0".to_string(),
                },
                Err(Error::VALUE) => CalcResult::Error {
                    error: Error::VALUE,
                    origin: cell,
                    message: "Invalid number".to_string(),
                },
                Err(e) => CalcResult::Error {
                    error: e,
                    origin: cell,
                    message: "Unknown error".to_string(),
                },
            },

            // -----------------------------------------------------
            // Case 2: left is Number, right is Array
            // -----------------------------------------------------
            (NumberOrArray::Number(f1), NumberOrArray::Array(a2)) => {
                let mut array = Vec::new();
                for row in a2 {
                    let mut data_row = Vec::new();
                    for node in row {
                        match self.array_node_to_number(&node) {
                            Ok(f2) => match op(f1, f2) {
                                Ok(x) => data_row.push(ArrayNode::Number(x)),
                                Err(Error::DIV) => data_row.push(ArrayNode::Error(Error::DIV)),
                                Err(Error::VALUE) => data_row.push(ArrayNode::Error(Error::VALUE)),
                                Err(e) => data_row.push(ArrayNode::Error(e)),
                            },
                            Err(err) => data_row.push(ArrayNode::Error(err)),
                        }
                    }
                    array.push(data_row);
                }
                CalcResult::Array(array)
            }

            // -----------------------------------------------------
            // Case 3: left is Array, right is Number
            // -----------------------------------------------------
            (NumberOrArray::Array(a1), NumberOrArray::Number(f2)) => {
                let mut array = Vec::new();
                for row in a1 {
                    let mut data_row = Vec::new();
                    for node in row {
                        match self.array_node_to_number(&node) {
                            Ok(f1) => match op(f1, f2) {
                                Ok(x) => data_row.push(ArrayNode::Number(x)),
                                Err(Error::DIV) => data_row.push(ArrayNode::Error(Error::DIV)),
                                Err(Error::VALUE) => data_row.push(ArrayNode::Error(Error::VALUE)),
                                Err(e) => data_row.push(ArrayNode::Error(e)),
                            },
                            Err(err) => data_row.push(ArrayNode::Error(err)),
                        }
                    }
                    array.push(data_row);
                }
                CalcResult::Array(array)
            }

            // -----------------------------------------------------
            // Case 4: Both are arrays
            // -----------------------------------------------------
            (NumberOrArray::Array(a1), NumberOrArray::Array(a2)) => {
                broadcast_arrays(&a1, &a2, cell, |v1, v2| {
                    match (self.array_node_to_number(v1), self.array_node_to_number(v2)) {
                        (Ok(f1), Ok(f2)) => match op(f1, f2) {
                            Ok(x) => ArrayNode::Number(x),
                            Err(e) => ArrayNode::Error(e),
                        },
                        (Err(e), _) | (_, Err(e)) => ArrayNode::Error(e),
                    }
                })
            }
        }
    }

    /// Applies the concatenation operator (`&`) element-wise.
    /// When either operand is a range or array the result is an array of strings;
    /// when both are scalars the result is a single String. `left` is the evaluated left operand.
    pub(crate) fn handle_concatenate(
        &mut self,
        left: CalcResult,
        right: &Node,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        let l = self.string_or_array(left, cell);
        let r = self.get_string_or_array(right, cell);
        let (l, r) = match (l, r) {
            (Ok(l), Ok(r)) => (l, r),
            (Err(error), Ok(StringOrArray::Array(array))) => {
                return array_of_errors(&array, error, |_| None);
            }
            (Ok(StringOrArray::Array(array)), Err(error)) => {
                return array_of_errors(&array, error, |node| array_node_to_string(node).err());
            }
            (Err(error), _) | (_, Err(error)) => return error,
        };

        // Concatenates two array elements, propagating errors as error nodes.
        let concat_nodes = |a: &ArrayNode, b: &ArrayNode| -> ArrayNode {
            match (array_node_to_text(a), array_node_to_text(b)) {
                (Ok(sa), Ok(sb)) => ArrayNode::String(concat_limited(&sa, &sb)),
                (Err(e), _) | (_, Err(e)) => ArrayNode::Error(e),
            }
        };

        match (l, r) {
            (StringOrArray::String(s1), StringOrArray::String(s2)) => {
                CalcResult::String(concat_limited(&s1, &s2))
            }
            (StringOrArray::String(s1), StringOrArray::Array(a2)) => CalcResult::Array(
                a2.iter()
                    .map(|row| {
                        row.iter()
                            .map(|n| concat_nodes(&ArrayNode::String(s1.clone()), n))
                            .collect()
                    })
                    .collect(),
            ),
            (StringOrArray::Array(a1), StringOrArray::String(s2)) => CalcResult::Array(
                a1.iter()
                    .map(|row| {
                        row.iter()
                            .map(|n| concat_nodes(n, &ArrayNode::String(s2.clone())))
                            .collect()
                    })
                    .collect(),
            ),
            (StringOrArray::Array(a1), StringOrArray::Array(a2)) => {
                broadcast_arrays(&a1, &a2, cell, concat_nodes)
            }
        }
    }

    /// Applies a comparison operator element-wise.
    /// When either operand is a range or array the result is an array of booleans,
    /// with an error where an element of the operands is an error;
    /// when both are scalars the result is a single Boolean. `left` is the evaluated left operand.
    pub(crate) fn handle_comparison(
        &mut self,
        left: CalcResult,
        right: &Node,
        cell: CellReferenceIndex,
        kind: &OpCompare,
    ) -> CalcResult {
        let l = self.value_or_array(left, cell);
        let r = self.get_value_or_array(right, cell);
        let (l, r) = match (l, r) {
            (Ok(l), Ok(r)) => (l, r),
            (Err(error), Ok(ValueOrArray::Array(array))) => {
                return array_of_errors(&array, error, |_| None);
            }
            (Ok(ValueOrArray::Array(array)), Err(error)) => {
                return array_of_errors(&array, error, own_error);
            }
            (Err(error), _) | (_, Err(error)) => return error,
        };

        let apply = |lv: &CalcResult, rv: &CalcResult| -> bool {
            let cmp = compare_values(lv, rv);
            match kind {
                OpCompare::Equal => cmp == 0,
                OpCompare::LessThan => cmp == -1,
                OpCompare::GreaterThan => cmp == 1,
                OpCompare::LessOrEqualThan => cmp < 1,
                OpCompare::GreaterOrEqualThan => cmp > -1,
                OpCompare::NonEqual => cmp != 0,
            }
        };

        // Compares two elements. One that is an error is the result, the left
        // one first.
        let apply_to_elements = |lv: &CalcResult, rv: &CalcResult| -> ArrayNode {
            match (lv, rv) {
                (CalcResult::Error { error, .. }, _) | (_, CalcResult::Error { error, .. }) => {
                    ArrayNode::Error(error.clone())
                }
                _ => ArrayNode::Boolean(apply(lv, rv)),
            }
        };

        let node_to_calc = |node: &ArrayNode| -> CalcResult {
            match node {
                ArrayNode::Number(n) => CalcResult::Number(*n),
                ArrayNode::Boolean(b) => CalcResult::Boolean(*b),
                ArrayNode::String(s) => CalcResult::String(s.clone()),
                ArrayNode::Error(e) => CalcResult::Error {
                    error: e.clone(),
                    origin: cell,
                    message: String::new(),
                },
                ArrayNode::Empty => CalcResult::EmptyCell,
            }
        };

        match (l, r) {
            (ValueOrArray::Value(lv), ValueOrArray::Value(rv)) => {
                CalcResult::Boolean(apply(&lv, &rv))
            }
            (ValueOrArray::Array(la), ValueOrArray::Value(rv)) => CalcResult::Array(
                la.iter()
                    .map(|row| {
                        row.iter()
                            .map(|n| apply_to_elements(&node_to_calc(n), &rv))
                            .collect()
                    })
                    .collect(),
            ),
            (ValueOrArray::Value(lv), ValueOrArray::Array(ra)) => CalcResult::Array(
                ra.iter()
                    .map(|row| {
                        row.iter()
                            .map(|n| apply_to_elements(&lv, &node_to_calc(n)))
                            .collect()
                    })
                    .collect(),
            ),
            (ValueOrArray::Array(la), ValueOrArray::Array(ra)) => {
                broadcast_arrays(&la, &ra, cell, |l, r| {
                    apply_to_elements(&node_to_calc(l), &node_to_calc(r))
                })
            }
        }
    }
}

#[cfg(test)]
mod power_tests;

#[cfg(test)]
mod bcast_tests;

/// `a & b`, cut to the cell text limit. Excel keeps the high half of a pair the limit cuts
/// (probes/concatenate-limit-spill E8, E9, E11; docs/excel-semantics/coercion.md).
fn concat_limited(a: &Text, b: &Text) -> Text {
    let mut joined = Text::concat(a, b);
    joined.truncate_utf16(MAX_TEXT_UNITS);
    joined
}
