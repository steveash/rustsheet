//! The CEILING/FLOOR family on the SPEC-0011 driver (T-1773): CEILING, ECMA.CEILING, FLOOR,
//! CEILING.MATH, CEILING.PRECISE, ISO.CEILING, FLOOR.MATH and FLOOR.PRECISE.

use crate::constants::EXCEL_PRECISION;
use crate::expressions::types::CellReferenceIndex;
use crate::functions::Function;
use crate::number_format::to_excel_precision;
use crate::{
    calc_result::CalcResult, expressions::parser::Node, expressions::token::Error, model::Model,
};

/// `value / significance` with Excel's precision applied, so floating-point noise does not
/// push an exact multiple over a step boundary.
fn snap_ratio(value: f64, significance: f64) -> f64 {
    to_excel_precision(value / significance, EXCEL_PRECISION)
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
