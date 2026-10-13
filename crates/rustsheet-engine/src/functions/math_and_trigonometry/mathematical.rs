use crate::arithmetic::{flush_subnormal, power};
use crate::cast::NumberOrArray;
use crate::constants::EXCEL_PRECISION;
use crate::decimal_round::{round_decimal, RoundMode};
use crate::expressions::parser::ArrayNode;
use crate::expressions::types::CellReferenceIndex;
use crate::number_format::to_excel_precision;
use crate::reference_operators::RefArg;
use crate::single_number_fn;
use crate::{
    calc_result::CalcResult, expressions::parser::Node, expressions::token::Error, model::Model,
};
use std::f64::consts::PI;

/// Smallest `|number/divisor|` for which Excel's `MOD` returns `#NUM!`: a decimal limit, not a
/// power of two (probes/mod-quotient-threshold5).
const MOD_MAX_QUOTIENT: f64 = 1.1259E+12;

/// The largest `min_length` BASE accepts (probes/base-min-length).
const BASE_MAX_MIN_LENGTH: f64 = 255.0;

impl Model {
    /// MIN and MAX: folds every number of `args` with `pick`; 0 when there is none.
    ///
    /// Excel (probes/min-max-value-args, T-0217; docs/excel-semantics/coercion.md): ranges,
    /// references (also via IF/INDEX) and arrays contribute only their numbers, while a value
    /// argument is coerced as SUM does: text that is not a number is `#VALUE!`, `"2"` is 2,
    /// TRUE is 1 and an empty argument is 0.
    fn fold_min_max(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
        pick: fn(f64, f64) -> f64,
    ) -> CalcResult {
        let mut result = f64::NAN;
        // A referenced-cell error ranks below a direct scalar error; the leftmost wins
        // (probes/aggregate-error-precedence C9-C24).
        let mut reference_error = None;
        for arg in args {
            match self.evaluate_picked_reference_arg(arg, cell) {
                RefArg::Ranges(areas) => {
                    if reference_error.is_none() {
                        if let Err(error) = self.for_each_number_in_areas(&areas, cell, |value| {
                            result = pick(value, result);
                        }) {
                            reference_error = Some(error);
                        }
                    }
                }
                RefArg::Value(CalcResult::Array(array)) => {
                    for node in array.into_iter().flatten() {
                        match node {
                            ArrayNode::Number(value) => result = pick(value, result),
                            ArrayNode::Error(error) => {
                                return CalcResult::new_error(error, cell, String::new())
                            }
                            // Booleans and text in an array are ignored.
                            _ => {}
                        }
                    }
                }
                RefArg::Value(value) => match self.cast_aggregate_arg_to_number(value, cell) {
                    Ok(value) => result = pick(value, result),
                    Err(error) => return error,
                },
            }
        }
        if let Some(error) = reference_error {
            return error;
        }
        if result.is_nan() || result.is_infinite() {
            return CalcResult::Number(0.0);
        }
        CalcResult::Number(result)
    }

    pub(crate) fn fn_min(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.fold_min_max(args, cell, f64::min)
    }

    pub(crate) fn fn_max(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.fold_min_max(args, cell, f64::max)
    }

    pub(crate) fn fn_base(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let arg_count = args.len();
        if !(2..=3).contains(&arg_count) {
            return CalcResult::new_args_number_error(cell);
        }

        // number to convert
        let mut value = match self.get_number(&args[0], cell) {
            Ok(f) => f.trunc() as i64,
            Err(s) => return s,
        };
        // radix
        let radix = match self.get_number(&args[1], cell) {
            Ok(f) => f.trunc() as i64,
            Err(s) => return s,
        };
        // optional min_length
        let min_length = if arg_count == 3 {
            match self.get_number(&args[2], cell) {
                Ok(f) => {
                    if f < 0.0 {
                        return CalcResult::Error {
                            error: Error::NUM,
                            origin: cell,
                            message: "Minimum length must be non-negative".to_string(),
                        };
                    }
                    // Excel: min_length truncates and may be at most 255 (`BASE(1,2,255.9)` pads to 255,
                    // `BASE(1,2,256)` is #NUM!; probes/base-min-length). Checking before padding also
                    // bounds the allocation (CLAUDE.md directive 5).
                    let min_length = f.trunc();
                    if min_length > BASE_MAX_MIN_LENGTH {
                        return CalcResult::Error {
                            error: Error::NUM,
                            origin: cell,
                            message: "Minimum length must be at most 255".to_string(),
                        };
                    }
                    min_length as usize
                }
                Err(s) => return s,
            }
        } else {
            0
        };

        if !(2..=36).contains(&radix) {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Radix must be between 2 and 36".to_string(),
            };
        }

        // number must be >= 0
        if value < 0 {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Number must be non-negative".to_string(),
            };
        }

        let mut buf = String::new();
        if value == 0 {
            buf.push('0');
        } else {
            while value > 0 {
                let digit = (value % radix) as u8;
                let ch = match digit {
                    0..=9 => (b'0' + digit) as char,
                    10..=35 => (b'A' + (digit - 10)) as char,
                    _ => unreachable!(),
                };
                buf.push(ch);
                value /= radix;
            }
            // we built it in reverse
            buf = buf.chars().rev().collect();
        }

        // pad with leading zeros if needed
        if buf.len() < min_length {
            let mut padded = String::with_capacity(min_length);
            for _ in 0..(min_length - buf.len()) {
                padded.push('0');
            }
            padded.push_str(&buf);
            buf = padded;
        }

        CalcResult::String(buf)
    }

    pub(crate) fn fn_sumsq(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }

        let mut result = 0.0;
        if let Err(error) = self.for_each_value_skip_empty(args, cell, |value| {
            result += flush_subnormal(value * value);
        }) {
            return error;
        }
        CalcResult::finite_number_or_num(flush_subnormal(result), cell)
    }

    /// `SUMIF(criteria_range`, criteria, [`sum_range`])
    /// if `sum_range` is missing then `criteria_range` will be used.
    ///
    /// The `criteria` argument may be a single value, a range or an array; in the
    /// latter two cases SUMIF spills one sum per criterion (see `Model::lift_criteria`).
    pub(crate) fn fn_sumif(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        // When sum_range is missing, criteria_range doubles as the sum_range.
        let (criteria_range, criteria, sum_range) = match args.len() {
            2 => (&args[0], &args[1], &args[0]),
            3 => (&args[0], &args[1], &args[2]),
            _ => return CalcResult::new_args_number_error(cell),
        };
        self.sumif(criteria_range, criteria, sum_range, cell)
    }

    /// `SUMIFS(sum_range`, `criteria_range1`, criteria1, [`criteria_range2`, criteria2], ...)
    pub(crate) fn fn_sumifs(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.sum_ifs(args, cell, true)
    }

    /// ROUND, ROUNDUP, ROUNDDOWN and TRUNC: `(number, num_digits)`, rounded in
    /// decimal (see [`round_decimal`]); a result beyond the double range is `#NUM!`.
    /// TRUNC alone may omit `num_digits` (0).
    fn round_family(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
        mode: RoundMode,
        num_digits_optional: bool,
    ) -> CalcResult {
        let min_args = if num_digits_optional { 1 } else { 2 };
        if !(min_args..=2).contains(&args.len()) {
            return CalcResult::new_args_number_error(cell);
        }
        // Lifts element-wise over arrays and ranges (gen-m2/893302d6c5b99e2e Calc!A313).
        let round = |value: f64, digits: f64| round_decimal(value, digits, mode).ok_or(Error::NUM);
        if args.len() == 1 {
            self.lift_numbers(args, cell, "Result is too large", |[value]| {
                round(value, 0.0)
            })
        } else {
            self.lift_numbers(args, cell, "Result is too large", |[value, digits]| {
                round(value, digits)
            })
        }
    }

    pub(crate) fn fn_round(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.round_family(args, cell, RoundMode::HalfAwayFromZero, false)
    }

    pub(crate) fn fn_roundup(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.round_family(args, cell, RoundMode::Up, false)
    }

    pub(crate) fn fn_rounddown(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.round_family(args, cell, RoundMode::Down, false)
    }

    /// TRUNC is ROUNDDOWN with an optional `num_digits`: Excel cuts to 15 digits
    /// first and wraps huge digit counts the same way (docs/excel-semantics/numbers.md
    /// "TRUNC is ROUNDDOWN", probes/trunc-decimal-stages).
    pub(crate) fn fn_trunc(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.round_family(args, cell, RoundMode::Down, true)
    }

    // (number, divisor); lifts element-wise over arrays and ranges (probes/geomean-overflow-cut-3 E4)
    pub(crate) fn fn_mod(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }
        self.lift_numbers(args, cell, "Invalid arguments", |[value, divisor]| {
            if divisor == 0.0 {
                return Err(Error::DIV);
            }
            // Excel: #NUM! once |number/divisor| reaches 1.1259E+12, whatever the signs
            // (docs/excel-semantics/numbers.md "MOD is #NUM! for a huge quotient").
            if (value / divisor).abs() >= MOD_MAX_QUOTIENT {
                return Err(Error::NUM);
            }
            Ok(crate::functions::util::excel_mod(value, divisor))
        })
    }

    pub(crate) fn fn_quotient(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let value = match self.get_number_no_bools(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let divisor = match self.get_number_no_bools(&args[1], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        if divisor == 0.0 {
            return CalcResult::Error {
                error: Error::DIV,
                origin: cell,
                message: "Divide by 0".to_string(),
            };
        }

        let result = value / divisor;
        CalcResult::Number(result.signum() * result.abs().floor())
    }

    pub(crate) fn fn_floor(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let value = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let significance = match self.get_number(&args[1], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        if significance == 0.0 {
            if value == 0.0 {
                return CalcResult::Number(0.0);
            }
            return CalcResult::Error {
                error: Error::DIV,
                origin: cell,
                message: "Divide by 0".to_string(),
            };
        }
        if significance < 0.0 && value > 0.0 {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Significance must be positive when value is positive".to_string(),
            };
        }

        // Apply Excel precision to the ratio to handle floating-point rounding errors
        let ratio = to_excel_precision(value / significance, EXCEL_PRECISION);
        let result = f64::floor(ratio) * significance;
        CalcResult::Number(result)
    }

    pub(crate) fn fn_ceiling(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let value = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let significance = match self.get_number(&args[1], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        if significance == 0.0 {
            // This behaviour is different from FLOOR where division by zero returns an error
            return CalcResult::Number(0.0);
        }
        if significance < 0.0 && value > 0.0 {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Significance must be positive when value is positive".to_string(),
            };
        }
        // Apply Excel precision fix
        let ratio = to_excel_precision(value / significance, EXCEL_PRECISION);
        let result = f64::ceil(ratio) * significance;
        CalcResult::Number(result)
    }

    pub(crate) fn fn_ceiling_math(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        let arg_count = args.len();
        if !(1..=3).contains(&arg_count) {
            return CalcResult::new_args_number_error(cell);
        }
        let value = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let significance = if arg_count > 1 {
            match self.get_number(&args[1], cell) {
                Ok(f) => f.abs(),
                Err(s) => return s,
            }
        } else {
            1.0
        };
        let mode = if arg_count > 2 {
            match self.get_number(&args[2], cell) {
                Ok(f) => f,
                Err(s) => return s,
            }
        } else {
            0.0
        };
        if significance == 0.0 {
            return CalcResult::Number(0.0);
        }
        if value < 0.0 && mode != 0.0 {
            // Apply Excel precision fix
            let ratio = to_excel_precision(value / significance, EXCEL_PRECISION);
            let result = f64::floor(ratio) * significance;
            CalcResult::Number(result)
        } else {
            // Apply Excel precision fix
            let ratio = to_excel_precision(value / significance, EXCEL_PRECISION);
            let result = f64::ceil(ratio) * significance;
            CalcResult::Number(result)
        }
    }

    pub(crate) fn fn_ceiling_precise(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        let arg_count = args.len();
        if !(1..=2).contains(&arg_count) {
            return CalcResult::new_args_number_error(cell);
        }
        let value = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let significance = if arg_count > 1 {
            match self.get_number(&args[1], cell) {
                Ok(f) => f.abs(),
                Err(s) => return s,
            }
        } else {
            1.0
        };
        if significance == 0.0 {
            return CalcResult::Number(0.0);
        }
        // Apply Excel precision fix
        let ratio = to_excel_precision(value / significance, EXCEL_PRECISION);
        let result = f64::ceil(ratio) * significance;
        CalcResult::Number(result)
    }

    pub(crate) fn fn_iso_ceiling(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        // ISO.CEILING is equivalent to CEILING.PRECISE
        self.fn_ceiling_precise(args, cell)
    }

    pub(crate) fn fn_floor_math(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let arg_count = args.len();
        if !(1..=3).contains(&arg_count) {
            return CalcResult::new_args_number_error(cell);
        }
        let value = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let significance = if arg_count > 1 {
            match self.get_number(&args[1], cell) {
                Ok(f) => f,
                Err(s) => return s,
            }
        } else {
            1.0
        };
        let mode = if arg_count > 2 {
            match self.get_number(&args[2], cell) {
                Ok(f) => f,
                Err(s) => return s,
            }
        } else {
            0.0
        };
        if significance == 0.0 {
            return CalcResult::Number(0.0);
        }
        let significance = significance.abs();
        if value < 0.0 && mode != 0.0 {
            // Apply Excel precision to handle floating-point rounding errors
            let ratio = to_excel_precision(value / significance, EXCEL_PRECISION);
            let result = f64::ceil(ratio) * significance;
            CalcResult::Number(result)
        } else {
            // Apply Excel precision to handle floating-point rounding errors
            let ratio = to_excel_precision(value / significance, EXCEL_PRECISION);
            let result = f64::floor(ratio) * significance;
            CalcResult::Number(result)
        }
    }

    pub(crate) fn fn_floor_precise(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        let arg_count = args.len();
        if !(1..=2).contains(&arg_count) {
            return CalcResult::new_args_number_error(cell);
        }
        let value = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let significance = if arg_count > 1 {
            match self.get_number(&args[1], cell) {
                Ok(f) => f.abs(),
                Err(s) => return s,
            }
        } else {
            1.0
        };
        if significance == 0.0 {
            return CalcResult::Number(0.0);
        }

        // Apply Excel precision to handle floating-point rounding errors
        let ratio = to_excel_precision(value / significance, EXCEL_PRECISION);
        let result = f64::floor(ratio) * significance;
        CalcResult::Number(result)
    }

    pub(crate) fn fn_mround(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        // MROUND(number, multiple)
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }

        let value = self.evaluate_node_in_context(&args[0], cell);

        let multiple = self.evaluate_node_in_context(&args[1], cell);

        // if either is empty => #N/A
        if matches!(value, CalcResult::EmptyArg) || matches!(multiple, CalcResult::EmptyArg) {
            return CalcResult::Error {
                error: Error::NA,
                origin: cell,
                message: "Bad argument for MROUND".to_string(),
            };
        }

        // Booleans are not cast
        if matches!(value, CalcResult::Boolean(_)) {
            return CalcResult::new_error(Error::VALUE, cell, "Expecting number".to_string());
        }

        if matches!(multiple, CalcResult::Boolean(_)) {
            return CalcResult::new_error(Error::VALUE, cell, "Expecting number".to_string());
        }

        let value = match self.cast_to_number(value, cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let multiple = match self.cast_to_number(multiple, cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        if multiple == 0.0 {
            return CalcResult::Number(0.0);
        }
        if (value > 0.0 && multiple < 0.0) || (value < 0.0 && multiple > 0.0) {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "number and multiple must have the same sign".to_string(),
            };
        }
        let result = (value / multiple).round() * multiple;
        CalcResult::Number(result)
    }

    single_number_fn!(fn_log10, |f| if f <= 0.0 {
        Err(Error::NUM)
    } else {
        Ok(f64::log10(f))
    });
    single_number_fn!(fn_ln, |f| if f <= 0.0 {
        Err(Error::NUM)
    } else {
        Ok(f64::ln(f))
    });
    single_number_fn!(fn_sin, |f| Ok(f64::sin(f)));
    single_number_fn!(fn_cos, |f| Ok(f64::cos(f)));
    single_number_fn!(fn_tan, |f| Ok(f64::tan(f)));
    single_number_fn!(fn_sinh, |f| Ok(super::hyperbolic::sinh(f)));
    single_number_fn!(fn_cosh, |f| Ok(super::hyperbolic::cosh(f)));
    single_number_fn!(fn_tanh, |f| Ok(super::hyperbolic::tanh(f)));
    single_number_fn!(fn_asin, |f| Ok(f64::asin(f)));
    single_number_fn!(fn_acos, |f| Ok(f64::acos(f)));
    single_number_fn!(fn_atan, |f| Ok(f64::atan(f)));
    single_number_fn!(fn_asinh, |f| Ok(f64::asinh(f)));
    single_number_fn!(fn_acosh, |f| Ok(f64::acosh(f)));
    single_number_fn!(fn_atanh, |f| Ok(f64::atanh(f)));
    single_number_fn!(fn_abs, |f| Ok(f64::abs(f)));
    single_number_fn!(fn_sqrt, |f| if f < 0.0 {
        Err(Error::NUM)
    } else {
        Ok(f64::sqrt(f))
    });
    /// SQRTPI. Unlike RADIANS, a boolean argument is `#VALUE!` (a literal, a comparison, a cell
    /// reference or an array element), while numeric text is read (probes/radians-sqrtpi-munit,
    /// docs/excel-semantics/numbers.md "RADIANS, SQRTPI and MUNIT", T-1282).
    pub(crate) fn fn_sqrtpi(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        let value = self.evaluate_node_in_context(&args[0], cell);
        if matches!(value, CalcResult::Boolean(_)) {
            return CalcResult::new_error(Error::VALUE, cell, "Expecting number".to_string());
        }
        let operand = match self.number_or_array(value, cell) {
            Ok(NumberOrArray::Array(mut rows)) => {
                for node in rows.iter_mut().flatten() {
                    if matches!(node, ArrayNode::Boolean(_)) {
                        *node = ArrayNode::Error(Error::VALUE);
                    }
                }
                Ok(NumberOrArray::Array(rows))
            }
            other => other,
        };
        self.lift_number_operands::<1>(vec![operand], cell, "Invalid number", |[f]| {
            if f < 0.0 {
                Err(Error::NUM)
            } else {
                Ok((f * PI).sqrt())
            }
        })
    }

    single_number_fn!(fn_acot, |f| {
        let v = f64::atan(1.0 / f);
        if f >= 0.0 {
            Ok(v)
        } else {
            // To be compatible with Excel we need a different branch
            // when f < 0
            Ok(v + PI)
        }
    });
    single_number_fn!(fn_acoth, |f: f64| if f.abs() == 1.0 {
        Err(Error::NUM)
    } else {
        Ok(0.5 * (f64::ln((f + 1.0) / (f - 1.0))))
    });
    single_number_fn!(fn_cot, |f| if f == 0.0 {
        Err(Error::DIV)
    } else {
        Ok(f64::cos(f) / f64::sin(f))
    });
    single_number_fn!(fn_coth, |f: f64| if f == 0.0 {
        Err(Error::DIV)
    } else if f.abs() > 20.0 {
        // for values > 20.0 this is exact in f64
        Ok(f.signum())
    } else {
        Ok(f64::cosh(f) / f64::sinh(f))
    });
    single_number_fn!(fn_csc, |f| if f == 0.0 {
        Err(Error::DIV)
    } else {
        Ok(1.0 / f64::sin(f))
    });
    single_number_fn!(fn_csch, |f| if f == 0.0 {
        Err(Error::DIV)
    } else {
        // Excel: a subnormal CSCH result is a true 0 (CSCH(710), docs/excel-semantics/numbers.md).
        Ok(flush_subnormal(1.0 / f64::sinh(f)))
    });
    single_number_fn!(fn_sec, |f| Ok(1.0 / f64::cos(f)));
    single_number_fn!(fn_sech, |f| Ok(1.0 / f64::cosh(f)));
    single_number_fn!(fn_exp, |f: f64| Ok(f64::exp(f)));
    single_number_fn!(fn_fact, |x: f64| {
        let x = x.floor();
        if x < 0.0 {
            return Err(Error::NUM);
        }
        let mut acc = 1.0;
        let mut k = 2.0;
        while k <= x {
            acc *= k;
            // Excel: #NUM! at overflow, which also bounds the loop (numbers.md "FACT and FACTDOUBLE").
            if !acc.is_finite() {
                return Err(Error::NUM);
            }
            k += 1.0;
        }
        Ok(acc)
    });
    single_number_fn!(fn_factdouble, |x: f64| {
        let x = x.floor();
        if x < -1.0 {
            return Err(Error::NUM);
        }
        if x < 0.0 {
            return Ok(1.0);
        }
        let mut acc = 1.0;
        let mut k = if x % 2.0 == 0.0 { 2.0 } else { 1.0 };
        while k <= x {
            acc *= k;
            if !acc.is_finite() {
                return Err(Error::NUM);
            }
            k += 2.0;
        }
        Ok(acc)
    });
    single_number_fn!(fn_sign, |f| {
        if f == 0.0 {
            Ok(0.0)
        } else {
            Ok(f64::signum(f))
        }
    });
    single_number_fn!(fn_degrees, |f| Ok(f * (180.0 / PI)));
    single_number_fn!(fn_radians, |f| Ok(f * (PI / 180.0)));
    single_number_fn!(fn_odd, |f| {
        let sign = f64::signum(f);
        Ok(sign * (f64::ceil((f64::abs(f) - 1.0) / 2.0) * 2.0 + 1.0))
    });
    single_number_fn!(fn_even, |f| Ok(f64::signum(f)
        * f64::ceil(f64::abs(f) / 2.0)
        * 2.0));
    single_number_fn!(fn_int, |f| Ok(f64::floor(f)));

    pub(crate) fn fn_pi(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if !args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }
        CalcResult::Number(PI)
    }

    pub(crate) fn fn_atan2(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let x = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let y = match self.get_number(&args[1], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        if x == 0.0 && y == 0.0 {
            return CalcResult::Error {
                error: Error::DIV,
                origin: cell,
                message: "Arguments can't be both zero".to_string(),
            };
        }
        CalcResult::Number(f64::atan2(y, x))
    }

    pub(crate) fn fn_log(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let n_args = args.len();
        if !(1..=2).contains(&n_args) {
            return CalcResult::new_args_number_error(cell);
        }
        let x = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let y = if n_args == 1 {
            10.0
        } else {
            match self.get_number(&args[1], cell) {
                Ok(f) => f,
                Err(s) => return s,
            }
        };
        if x <= 0.0 {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Number must be positive".to_string(),
            };
        }
        if y == 1.0 {
            return CalcResult::Error {
                error: Error::DIV,
                origin: cell,
                message: "Logarithm base cannot be 1".to_string(),
            };
        }
        if y <= 0.0 {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Logarithm base must be positive".to_string(),
            };
        }
        CalcResult::Number(f64::log(x, y))
    }

    // POWER(number, power)
    // If the number or the power are arrays it works element by element.
    pub(crate) fn fn_power(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let x = self.get_number_or_array(&args[0], cell);
        let y = self.get_number_or_array(&args[1], cell);
        let (x, y) = match (x, y) {
            (Ok(NumberOrArray::Number(x)), Ok(NumberOrArray::Number(y))) => (x, y),
            (x, y) => return self.arithmetic_on_values(x, y, cell, &power),
        };
        match power(x, y) {
            Ok(result) => CalcResult::Number(result),
            Err(error) => {
                let message = if x == 0.0 && y == 0.0 {
                    "Arguments can't be both zero"
                } else if error == Error::DIV {
                    "Zero to a negative power"
                } else {
                    // This might happen for some combinations of negative base and exponent
                    "Invalid arguments for POWER"
                };
                CalcResult::Error {
                    error,
                    origin: cell,
                    message: message.to_string(),
                }
            }
        }
    }

    pub(crate) fn fn_combin(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let n = match self.get_number(&args[0], cell) {
            Ok(f) => f.floor(),
            Err(s) => return s,
        };
        let k = match self.get_number(&args[1], cell) {
            Ok(f) => f.floor(),
            Err(s) => return s,
        };
        if n < 0.0 || k < 0.0 {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Arguments must be non-negative integers".to_string(),
            };
        }
        if k > n {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "k cannot be greater than n".to_string(),
            };
        }
        let k = k as usize;
        let mut result = 1.0;
        for i in 0..k {
            let t = i as f64;
            result *= (n - t) / (t + 1.0);
        }
        CalcResult::Number(result)
    }

    // PERCENTOF(part, total) — returns SUM(part) / SUM(total) as a decimal fraction.
    pub(crate) fn fn_percentof(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let part = match self.fn_sum_unsnapped(&args[0..1], cell) {
            CalcResult::Number(v) => v,
            other => return other,
        };
        let total = match self.fn_sum_unsnapped(&args[1..2], cell) {
            CalcResult::Number(v) => v,
            other => return other,
        };
        if total == 0.0 {
            return CalcResult::new_error(Error::DIV, cell, "Division by zero".to_string());
        }
        CalcResult::Number(part / total)
    }

    pub(crate) fn fn_combina(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let n = match self.get_number(&args[0], cell) {
            Ok(f) => f.floor(),
            Err(s) => return s,
        };
        let k = match self.get_number(&args[1], cell) {
            Ok(f) => f.floor(),
            Err(s) => return s,
        };
        if n < 0.0 || k < 0.0 || (n == 0.0 && k > 0.0) {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Arguments must be non-negative integers".to_string(),
            };
        }
        let k = k as usize;
        let mut result = 1.0;
        for i in 0..k {
            let t = i as f64;
            result *= (n + t) / (t + 1.0);
        }
        CalcResult::Number(result)
    }
}
