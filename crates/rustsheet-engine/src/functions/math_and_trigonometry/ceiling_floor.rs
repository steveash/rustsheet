//! The CEILING/FLOOR family on the SPEC-0011 driver (T-1773): CEILING, ECMA.CEILING, FLOOR,
//! CEILING.MATH, CEILING.PRECISE, ISO.CEILING, FLOOR.MATH and FLOOR.PRECISE.

use crate::arithmetic::flush_subnormal;
use crate::decimal_round::cut_to_15_digits;
use crate::expressions::types::CellReferenceIndex;
use crate::functions::Function;
use crate::{
    calc_result::CalcResult, expressions::parser::Node, expressions::token::Error, model::Model,
};

/// `value / significance` cut to 15 significant digits, so floating-point noise does not push an
/// exact multiple over a step boundary. An exact tie at the 16th digit goes away from zero, as
/// INT's cut does (numbers.md "FLOOR.PRECISE: a 16th-digit tie in the ratio cut goes away from
/// zero", probes/t2652-floor-ceiling-ties). A subnormal ratio is 0, as `/` flushes it, so the
/// call is 0 (numbers.md "CEILING and FLOOR of a subnormal ratio are 0",
/// probes/t2715-ceiling-floor-subnormal-ratio).
fn snap_ratio(value: f64, significance: f64) -> f64 {
    cut_to_15_digits(flush_subnormal(value / significance), true)
}

/// MROUND of two non-zero, same-signed numbers. Excel rounds the double quotient `|n / m|` up
/// when its fraction is within 5E-15 of a half (not a 15-digit cut of the quotient) and multiplies
/// in binary; a zero result is +0 (numbers.md "MROUND rounds the quotient up within 5E-15 of a
/// half, and multiplies in binary", probes/t2201-mround-quotient-rounding).
pub(super) fn mround_value(value: f64, multiple: f64) -> f64 {
    let quotient = (value / multiple).abs();
    let floor = quotient.floor();
    let steps = if quotient - floor >= 0.5 - 5e-15 {
        floor + 1.0
    } else {
        floor
    };
    let result = steps * multiple.abs();
    if result == 0.0 {
        0.0
    } else {
        result.copysign(value)
    }
}

impl Model {
    // The CEILING/FLOOR family lifts element-wise over arrays and ranges in every argument
    // (gen-m7/e5a361112d83d0c3 Calc!A9, gen-m9/89693ab0cafc29df Calc!A14). The ratio gets
    // Excel's 15-digit precision so that 0.3/0.1 style quotients round the way Excel's do. The
    // optional significance is 1 and the optional mode 0.
    pub(crate) fn fn_floor(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.numbers_fn(
            Function::Floor,
            args,
            [0.0, 0.0],
            cell,
            |[value, significance]| {
                if significance == 0.0 {
                    // Unlike CEILING, FLOOR divides by zero unless the number is 0 too.
                    return if value == 0.0 {
                        Ok(0.0)
                    } else {
                        Err(Error::DIV)
                    };
                }
                if significance < 0.0 && value > 0.0 {
                    return Err(Error::NUM);
                }
                Ok(snap_ratio(value, significance).floor() * significance)
            },
        )
    }

    /// CEILING and ECMA.CEILING, one table entry.
    pub(crate) fn fn_ceiling(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.numbers_fn(
            Function::Ceiling,
            args,
            [0.0, 0.0],
            cell,
            |[value, significance]| {
                if significance == 0.0 {
                    return Ok(0.0);
                }
                if significance < 0.0 && value > 0.0 {
                    return Err(Error::NUM);
                }
                Ok(snap_ratio(value, significance).ceil() * significance)
            },
        )
    }

    pub(crate) fn fn_ceiling_math(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        self.numbers_fn(
            Function::CeilingMath,
            args,
            [0.0, 1.0, 0.0],
            cell,
            |[value, significance, mode]| {
                let significance = significance.abs();
                if significance == 0.0 {
                    return Ok(0.0);
                }
                let ratio = snap_ratio(value, significance);
                let steps = if value < 0.0 && mode != 0.0 {
                    ratio.floor()
                } else {
                    ratio.ceil()
                };
                Ok(steps * significance)
            },
        )
    }

    /// CEILING.PRECISE and ISO.CEILING, one table entry.
    pub(crate) fn fn_ceiling_precise(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        self.numbers_fn(
            Function::CeilingPrecise,
            args,
            [0.0, 1.0],
            cell,
            |[value, significance]| {
                let significance = significance.abs();
                if significance == 0.0 {
                    return Ok(0.0);
                }
                Ok(snap_ratio(value, significance).ceil() * significance)
            },
        )
    }

    pub(crate) fn fn_iso_ceiling(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        // ISO.CEILING is equivalent to CEILING.PRECISE
        self.fn_ceiling_precise(args, cell)
    }

    pub(crate) fn fn_floor_math(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.numbers_fn(
            Function::FloorMath,
            args,
            [0.0, 1.0, 0.0],
            cell,
            |[value, significance, mode]| {
                let significance = significance.abs();
                if significance == 0.0 {
                    return Ok(0.0);
                }
                let ratio = snap_ratio(value, significance);
                let steps = if value < 0.0 && mode != 0.0 {
                    ratio.ceil()
                } else {
                    ratio.floor()
                };
                Ok(steps * significance)
            },
        )
    }

    pub(crate) fn fn_floor_precise(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        self.numbers_fn(
            Function::FloorPrecise,
            args,
            [0.0, 1.0],
            cell,
            |[value, significance]| {
                let significance = significance.abs();
                if significance == 0.0 {
                    return Ok(0.0);
                }
                Ok(snap_ratio(value, significance).floor() * significance)
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{mround_value, snap_ratio};

    #[test]
    fn snap_ratio_tie_goes_away_from_zero() {
        // probes/t2652-floor-ceiling-ties: a 16th-digit tie is not cut to even.
        assert_eq!(
            snap_ratio(-333_333_333_333_266.5, 1.0),
            -333_333_333_333_267.0
        );
        assert_eq!(
            snap_ratio(333_333_333_333_266.5, 1.0),
            333_333_333_333_267.0
        );
        assert_eq!(
            snap_ratio(333_333_333_333_265.5, 1.0),
            333_333_333_333_266.0
        );
        assert_eq!(snap_ratio(999_999_999_999_999.5, 1.0), 1e15);
        assert_eq!(snap_ratio(0.0, 1.0), 0.0);
        assert!(snap_ratio(1.0, 0.0).is_infinite());
        // probes/t2715-ceiling-floor-subnormal-ratio: a subnormal ratio is 0; the smallest normal
        // is kept (then cut to 15 digits), so its ceiling is still one step.
        assert_eq!(snap_ratio(1e-307, 4.5), 0.0);
        assert_eq!(snap_ratio(-1e-300, 1e8), 0.0);
        assert_eq!(snap_ratio(f64::MIN_POSITIVE, 1.0).ceil(), 1.0);
        assert_eq!(snap_ratio(-f64::MIN_POSITIVE, 1.0).floor(), -1.0);
    }

    #[test]
    fn mround_quotient_tolerance() {
        // probes/t2201-mround-quotient-rounding: q = 1.4999999999999998 rounds up.
        assert_eq!(mround_value(0.3, 0.2), 0.4);
        // Binary product, not the decimal 0.6.
        assert_eq!(mround_value(0.5, 0.2), 0.600_000_000_000_000_1);
        // One ulp below the half at 1234.5 is more than 5E-15 away: rounds down.
        assert_eq!(mround_value(1_234.499_999_999_999_8, 1.0), 1234.0);
        // 20 ulp below 1.5 (4.4E-15) is inside the tolerance; 0.5 - 5.05E-15 is outside.
        assert_eq!(mround_value(1.499_999_999_999_995_6, 1.0), 2.0);
        assert_eq!(mround_value(0.499_999_999_999_994_95, 1.0), 0.0);
        // Negatives keep the sign of the number; a zero result is +0.
        assert_eq!(mround_value(-0.3, -0.2), -0.4);
        assert!(mround_value(-0.499_999_999_999_994_95, -1.0).is_sign_positive());
    }
}
