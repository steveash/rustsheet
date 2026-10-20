//! x87 register arithmetic on positive values with a 64-bit significand: the FMUL and FDIV
//! roundings and the store to a double, for kernels that keep intermediates in a register
//! (ATANH/ACOTH's series, T-0917).

use super::{round_extended_to_double, round_to_bits, significand_and_exponent};

/// A positive x87 register value (ATANH/ACOTH's series, T-0917) `significand * 2^exponent`, the significand of 64 bits.
pub(crate) type Extended = (u128, i32);

pub(crate) fn extended_of(x: f64) -> Extended {
    let (significand, exponent) = significand_and_exponent(x.abs());
    (u128::from(significand) << 11, exponent - 11)
}

/// `a * b` rounded to 64 bits (FMUL).
pub(crate) fn extended_mul(a: Extended, b: Extended) -> Extended {
    let (product, shift) = round_to_bits(a.0 * b.0, 64, false);
    (product, a.1 + b.1 + shift)
}

/// `a / b` rounded to 64 bits (FDIV); `b` is nonzero.
pub(crate) fn extended_div(a: Extended, b: Extended) -> Extended {
    // 64 or 65 quotient bits, then two more from the remainder (under 2^66, no overflow) so the
    // rounding always has a guard bit, and what is left is the sticky bit.
    let numerator = a.0 << 64;
    let remainder = (numerator % b.0) << 2;
    let quotient = ((numerator / b.0) << 2) | (remainder / b.0);
    let (rounded, shift) = round_to_bits(quotient, 64, !remainder.is_multiple_of(b.0));
    (rounded, a.1 - 66 - b.1 + shift)
}

/// The register value stored to a double (the second rounding), or `None` outside the normal
/// range.
pub(crate) fn store_extended(value: Extended) -> Option<f64> {
    round_extended_to_double(value.0, value.1, false)
}
