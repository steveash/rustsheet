//! A value held in an x87 register: a 64-bit significand, rounded to nearest (ties to even) after
//! every operation, and never stored to a double until [`Register::store`]. Excel's kernels keep a
//! few Horner stages in registers (T-1941: GAMMALN's (2,3] head, `docs/research/t1941/k2_ref.py`);
//! the stored ops are `arithmetic.rs`'s `extended_*`, which round twice at once, and
//! [`Register::add`] is the unstored counterpart of `extended_sum`'s exact aligned sum.
//!
//! Only finite, normal-range values are modelled: the exponent is an `i32`, far wider than the
//! x87's, and no operation here can overflow or underflow it from double operands.

use crate::arithmetic::{
    extended_div, round_extended_to_double, round_extended_to_subnormal, round_to_bits,
    significand_and_exponent,
};
use crate::x87_exp::{dd_to_extended, power_of_two};

/// 2^exponent as an exact double: the product of two in-range powers of two, so any exponent of a
/// value in the normal range (and somewhat beyond, towards 0 or infinity) is reached.
fn wide_power_of_two(exponent: i32) -> f64 {
    let half = exponent / 2;
    power_of_two(half) * power_of_two(exponent - half)
}

/// `significand * 2^exponent`, with `significand` below 2^64 (zero, or at least 2^63 once
/// normalised by an operation).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Register {
    negative: bool,
    significand: u128,
    exponent: i32,
}

impl Register {
    /// A double, exactly (a load).
    pub(crate) fn load(x: f64) -> Self {
        if x == 0.0 || !x.is_finite() {
            return Self {
                negative: x.is_sign_negative(),
                significand: 0,
                exponent: 0,
            };
        }
        let (significand, exponent) = significand_and_exponent(x);
        Self {
            negative: x < 0.0,
            significand: u128::from(significand),
            exponent,
        }
    }

    /// The sign, the significand and the exponent: the value is `(-1)^negative * significand *
    /// 2^exponent`, and a zero has a zero significand.
    pub(crate) fn parts(self) -> (bool, u128, i32) {
        (self.negative, self.significand, self.exponent)
    }

    /// A double-double `hi + lo` rounded to a 64-bit significand (RN64, ties to even), as a value
    /// formed exactly and then held in a register.
    pub(crate) fn from_double_double((hi, lo): (f64, f64)) -> Self {
        if hi == 0.0 || !hi.is_finite() {
            return Self::load(hi);
        }
        let negative = hi < 0.0;
        let magnitude = if negative { (-hi, -lo) } else { (hi, lo) };
        let (significand, exponent) = dd_to_extended(magnitude);
        Self::rounded(negative, significand, exponent, false)
    }

    /// The register's value exactly as a double-double `hi + lo` (a 64-bit significand splits into
    /// its top 53 bits and the 11 below), for a value well inside the normal double range.
    pub(crate) fn to_double_double(self) -> (f64, f64) {
        if self.significand == 0 {
            return (0.0, 0.0);
        }
        let low_bits = self.significand & 0x7FF;
        // Both parts are integers below 2^64 with at most 53 significant bits: exact as doubles.
        let scale = wide_power_of_two(self.exponent);
        let sign = if self.negative { -1.0 } else { 1.0 };
        (
            sign * ((self.significand - low_bits) as f64 * scale),
            sign * (low_bits as f64 * scale),
        )
    }

    /// `-self`, exactly.
    pub(crate) fn neg(self) -> Self {
        Self {
            negative: !self.negative,
            ..self
        }
    }

    /// Whether the register holds a negative non-zero value.
    pub(crate) fn is_negative(self) -> bool {
        self.negative && self.significand != 0
    }

    /// Whether two registers hold the same value (a load keeps 53 bits unnormalised, so the
    /// fields of equal values can differ): their difference is exactly zero.
    pub(crate) fn same_value(self, other: Self) -> bool {
        self.add(other.neg()).significand == 0
    }

    /// The register `(-1)^negative * significand * 2^exponent` for a significand below 2^64,
    /// normalised as an operation leaves it.
    pub(crate) fn from_parts(negative: bool, significand: u128, exponent: i32) -> Self {
        if significand == 0 {
            return Self::zero(negative);
        }
        Self::rounded(negative, significand, exponent, false)
    }

    /// `self * other`, rounded to a 64-bit significand.
    pub(crate) fn mul(self, other: Self) -> Self {
        let negative = self.negative != other.negative;
        if self.significand == 0 || other.significand == 0 {
            return Self::zero(negative);
        }
        // Two significands of at most 64 bits: the product fits in 128.
        Self::rounded(
            negative,
            self.significand * other.significand,
            self.exponent + other.exponent,
            false,
        )
    }

    /// `self + other`, rounded to a 64-bit significand.
    pub(crate) fn add(self, other: Self) -> Self {
        if other.significand == 0 {
            return if self.significand == 0 {
                Self::zero(self.negative && other.negative)
            } else {
                self
            };
        }
        if self.significand == 0 {
            return other;
        }
        // `large` has the larger exponent; `small` is aligned to it.
        let (large, small) = if self.exponent >= other.exponent {
            (self, other)
        } else {
            (other, self)
        };
        let gap = large.exponent - small.exponent;
        // Within 63 bits both fit exactly on the smaller grid (at most 127 bits). Beyond it the
        // larger is shifted up 63 bits and the smaller down to that grid, its lost bits folded
        // into a sticky lowest bit: 63 guard bits then lie below the 64-bit result, so the
        // sticky bit decides only which side of a tie the exact value is on.
        let (large_units, small_units, exponent) = if gap <= 63 {
            (large.significand << gap, small.significand, small.exponent)
        } else {
            let drop = gap - 63;
            let shifted = if drop >= 128 {
                0
            } else {
                small.significand >> drop
            };
            let lost = drop >= 128 || shifted << drop != small.significand;
            (
                large.significand << 63,
                shifted | u128::from(lost),
                large.exponent - 63,
            )
        };
        if large.negative == small.negative {
            return Self::rounded(large.negative, large_units + small_units, exponent, false);
        }
        if large_units == small_units {
            return Self::zero(false);
        }
        let (negative, difference) = if large_units > small_units {
            (large.negative, large_units - small_units)
        } else {
            (small.negative, small_units - large_units)
        };
        Self::rounded(negative, difference, exponent, false)
    }

    /// `self / other` for a non-zero `other`, rounded to a 64-bit significand (FDIV).
    pub(crate) fn div(self, other: Self) -> Self {
        debug_assert!(other.significand != 0, "Register::div by zero");
        let negative = self.negative != other.negative;
        if self.significand == 0 {
            return Self::zero(negative);
        }
        let (significand, exponent) = extended_div(self.full_width(), other.full_width());
        Self {
            negative,
            significand,
            exponent,
        }
    }

    /// The magnitude with its significand shifted up to the full 64 bits (a load keeps 53),
    /// as [`extended_div`] takes it. The shift is at most 64, so the `i32` conversion holds.
    fn full_width(self) -> (u128, i32) {
        let up = self.significand.leading_zeros().saturating_sub(64);
        (
            self.significand << up,
            self.exponent - i32::try_from(up).unwrap_or(0),
        )
    }

    /// The register stored to a double (the second rounding): past the double range it is an
    /// infinity, and below the normal range it rounds once more on the subnormal grid (or to 0).
    pub(crate) fn store(self) -> f64 {
        let zero = if self.negative { -0.0 } else { 0.0 };
        if self.significand == 0 {
            return zero;
        }
        if let Some(value) =
            round_extended_to_double(self.significand, self.exponent, self.negative)
        {
            return value;
        }
        // The top bit is bit 63: 2^(exponent + 63) at least 2^1024 overflows.
        if self.exponent + 63 >= 1024 {
            return if self.negative {
                f64::NEG_INFINITY
            } else {
                f64::INFINITY
            };
        }
        round_extended_to_subnormal(self.significand, self.exponent, self.negative).unwrap_or(zero)
    }

    fn zero(negative: bool) -> Self {
        Self {
            negative,
            significand: 0,
            exponent: 0,
        }
    }

    /// `value * 2^exponent` (non-zero) rounded to 64 bits and normalised so bit 63 is the top
    /// bit, as [`round_extended_to_double`] expects (a carry to 2^64 moves down, a short exact
    /// value moves up).
    fn rounded(negative: bool, value: u128, exponent: i32, inexact: bool) -> Self {
        let (mut significand, shift) = round_to_bits(value, 64, inexact);
        let mut exponent = exponent + shift;
        if significand >> 64 != 0 {
            significand >>= 1;
            exponent += 1;
        }
        let up = significand.leading_zeros().saturating_sub(64);
        significand <<= up;
        exponent -= i32::try_from(up).unwrap_or(0);
        Self {
            negative,
            significand,
            exponent,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Register;

    /// Exact values pass through, and a store rounds once to 53 bits.
    #[test]
    fn loads_and_stores_doubles() {
        for x in [1.0, -2.5, 0.1, 1e300, -1e-300, 0.0] {
            assert_eq!(Register::load(x).store(), x);
        }
    }

    /// 1 + 2^-60 is exact in 64 bits, so its store rounds to 1; 1 + 2^-64 rounds to 1 in the
    /// register already (a tie to even), and 1 + 2^-64 + 2^-70 rounds up to 1 + 2^-63.
    #[test]
    fn sums_round_to_64_bits() {
        let one = Register::load(1.0);
        let sum = one.add(Register::load(2f64.powi(-60)));
        assert_eq!(sum.significand, (1u128 << 63) + 8);
        assert_eq!(sum.exponent, -63);
        assert_eq!(sum.store(), 1.0);
        let tie = one.add(Register::load(2f64.powi(-64)));
        assert_eq!(tie.significand, 1u128 << 63);
        let above = one.add(Register::load(2f64.powi(-64) + 2f64.powi(-70)));
        assert_eq!(above.significand, (1u128 << 63) + 1);
        // Far apart: the smaller only nudges a tie.
        let far = Register::load(1.0).add(Register::load(1e-300));
        assert_eq!(far.significand, 1u128 << 63);
        // 1 - 2^-64 - 2^-116 is just below the 64-bit 1 - 2^-64 and rounds to it.
        let below = one.add(Register::load(-(2f64.powi(-64) + 2f64.powi(-116))));
        assert_eq!(below.significand, (1u128 << 64) - 1);
        assert_eq!(below.exponent, -64);
    }

    /// A product is the exact product rounded to 64 bits: (1 + 2^-52)^2 = 1 + 2^-51 + 2^-104,
    /// and the 2^-104 term is dropped.
    #[test]
    fn products_round_to_64_bits() {
        let x = Register::load(1.0 + f64::EPSILON);
        let square = x.mul(x);
        assert_eq!(square.significand, (1u128 << 63) + (1u128 << 12));
        assert_eq!(square.store(), 1.0 + 2.0 * f64::EPSILON);
        assert_eq!(Register::load(-3.0).mul(Register::load(0.5)).store(), -1.5);
    }

    /// Values beyond the double range store as infinity, tiny ones as a subnormal or 0.
    #[test]
    fn out_of_range_values_store_without_panicking() {
        let huge = Register::load(f64::MAX).mul(Register::load(f64::MAX));
        assert_eq!(huge.store(), f64::INFINITY);
        let tiny = Register::load(f64::MIN_POSITIVE).mul(Register::load(0.25));
        assert_eq!(tiny.store(), f64::MIN_POSITIVE / 4.0);
        let gone = Register::load(1e-300).mul(Register::load(1e-300));
        assert_eq!(gone.store(), 0.0);
    }

    /// A quotient is rounded to 64 bits: 1/3 is `0xAAAA_AAAA_AAAA_AAAB` * 2^-65 (the bits past
    /// the 64th, 1010..., round up), with the sign of the operands.
    #[test]
    fn quotients_round_to_64_bits() {
        let third = Register::load(1.0).div(Register::load(3.0));
        assert_eq!(third.significand, 0xAAAA_AAAA_AAAA_AAAB);
        assert_eq!(third.exponent, -65);
        assert_eq!(third.store(), 1.0 / 3.0);
        assert_eq!(Register::load(-6.0).div(Register::load(3.0)).store(), -2.0);
        assert_eq!(Register::load(0.0).div(Register::load(3.0)).store(), 0.0);
    }

    /// A register splits exactly into a double-double and back; a double-double rounds to 64
    /// bits (1 + 2^-60 + 2^-70 keeps 2^-60 and drops 2^-70); negation is exact; equal values
    /// compare equal whether loaded (53 bits) or computed (normalised to 64).
    #[test]
    fn double_double_round_trip_and_value_equality() {
        let third = Register::load(1.0).div(Register::load(-3.0));
        let (hi, lo) = third.to_double_double();
        assert_eq!(hi, -1.0 / 3.0);
        assert!(lo != 0.0);
        assert_eq!(Register::from_double_double((hi, lo)), third);
        let rounded =
            Register::from_double_double((1.0 + 2f64.powi(-52), 2f64.powi(-60) + 2f64.powi(-70)));
        assert_eq!(rounded.significand, (1u128 << 63) + (1 << 11) + 8);
        assert_eq!(Register::from_double_double((0.0, 0.0)).store(), 0.0);
        assert_eq!(third.neg().neg(), third);
        assert!(third.is_negative() && !third.neg().is_negative());
        let two = Register::load(1.5).add(Register::load(0.5));
        assert!(two.same_value(Register::load(2.0)) && two != Register::load(2.0));
        assert!(!two.same_value(Register::load(2.0 + f64::EPSILON * 2.0)));
    }

    /// Cancellation is exact.
    #[test]
    fn differences_cancel_exactly() {
        let a = Register::load(1.0 + f64::EPSILON);
        assert_eq!(a.add(Register::load(-1.0)).store(), f64::EPSILON);
        assert_eq!(a.add(Register::load(-(1.0 + f64::EPSILON))).store(), 0.0);
    }
}
