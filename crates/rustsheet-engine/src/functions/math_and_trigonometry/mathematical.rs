use super::trig::{excel_sin_cos, excel_tan};
use crate::arithmetic::{divide, extended_sqrt, flush_subnormal, power};
use crate::cast::NumberOrArray;
use crate::constants::EXCEL_PRECISION;
use crate::decimal_round::{round_decimal, RoundMode};
use crate::expressions::parser::ArrayNode;
use crate::expressions::types::CellReferenceIndex;
use crate::functions::util::aggregate_args::{AggregatePolicy, ArgReading, ErrorRule, NumberRules};
use crate::number_format::to_excel_precision;
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

/// SIN, COS, TAN, COT, SEC and CSC are `#NUM!` from |x| = 2^27 (SIN(2^27-1) is a number, SIN(2^27)
/// is `#NUM!`; probes/trig-huge-argument, docs/excel-semantics/numbers.md "Trig of a huge angle").
fn trig_arg(f: f64) -> Result<f64, Error> {
    if f.abs() >= 134_217_728.0 {
        Err(Error::NUM)
    } else {
        Ok(f)
    }
}

/// MIN and MAX: a referenced-cell error ranks below a direct scalar error; the leftmost wins
/// (probes/aggregate-error-precedence C9-C24). A direct LAMBDA's `#VALUE!` ranks with the
/// referenced errors, as does an array error (every `Ranked` policy: probes/aggregate-direct-lambda-
/// product-min-max D15, D24, T-1604; probes/aggregate-array-error-vs-ref-error, T-1615).
const MIN_MAX_ARGS: AggregatePolicy = AggregatePolicy {
    reading: ArgReading::Aggregate,
    errors: ErrorRule::Ranked,
    skip_empty_args: false,
};

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
        if let Err(error) = self.for_each_aggregate_number(
            args,
            cell,
            MIN_MAX_ARGS,
            NumberRules::SUM_LIKE,
            |value| {
                result = pick(value, result);
            },
        ) {
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

        CalcResult::String(buf.into())
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

    /// Lifts `op` over `N` numeric arguments like [`Model::lift_numbers`], where the arguments
    /// past `args.len()` take their `defaults` (the optional significance and mode of the
    /// CEILING/FLOOR family).
    fn lift_with_defaults<const N: usize>(
        &mut self,
        args: &[Node],
        defaults: [f64; N],
        cell: CellReferenceIndex,
        op: impl Fn([f64; N]) -> Result<f64, Error>,
    ) -> CalcResult {
        let operands = (0..N)
            .map(|i| match args.get(i) {
                Some(arg) => self.get_number_or_array(arg, cell),
                None => Ok(NumberOrArray::Number(defaults[i])),
            })
            .collect();
        self.lift_number_operands(operands, cell, "Invalid arguments", op)
    }

    // The CEILING/FLOOR family lifts element-wise over arrays and ranges in every argument
    // (gen-m7/e5a361112d83d0c3 Calc!A9, gen-m9/89693ab0cafc29df Calc!A14). The ratio gets
    // Excel's 15-digit precision so that 0.3/0.1 style quotients round the way Excel's do.
    pub(crate) fn fn_floor(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }
        self.lift_numbers(args, cell, "Invalid arguments", |[value, significance]| {
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
        })
    }

    pub(crate) fn fn_ceiling(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }
        self.lift_numbers(args, cell, "Invalid arguments", |[value, significance]| {
            if significance == 0.0 {
                return Ok(0.0);
            }
            if significance < 0.0 && value > 0.0 {
                return Err(Error::NUM);
            }
            Ok(snap_ratio(value, significance).ceil() * significance)
        })
    }

    pub(crate) fn fn_ceiling_math(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        if !(1..=3).contains(&args.len()) {
            return CalcResult::new_args_number_error(cell);
        }
        self.lift_with_defaults(args, [0.0, 1.0, 0.0], cell, |[value, significance, mode]| {
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
        })
    }

    pub(crate) fn fn_ceiling_precise(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        if !(1..=2).contains(&args.len()) {
            return CalcResult::new_args_number_error(cell);
        }
        self.lift_with_defaults(args, [0.0, 1.0], cell, |[value, significance]| {
            let significance = significance.abs();
            if significance == 0.0 {
                return Ok(0.0);
            }
            Ok(snap_ratio(value, significance).ceil() * significance)
        })
    }

    pub(crate) fn fn_iso_ceiling(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        // ISO.CEILING is equivalent to CEILING.PRECISE
        self.fn_ceiling_precise(args, cell)
    }

    pub(crate) fn fn_floor_math(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if !(1..=3).contains(&args.len()) {
            return CalcResult::new_args_number_error(cell);
        }
        self.lift_with_defaults(args, [0.0, 1.0, 0.0], cell, |[value, significance, mode]| {
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
        })
    }

    pub(crate) fn fn_floor_precise(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        if !(1..=2).contains(&args.len()) {
            return CalcResult::new_args_number_error(cell);
        }
        self.lift_with_defaults(args, [0.0, 1.0], cell, |[value, significance]| {
            let significance = significance.abs();
            if significance == 0.0 {
                return Ok(0.0);
            }
            Ok(snap_ratio(value, significance).floor() * significance)
        })
    }

    pub(crate) fn fn_mround(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        // MROUND(number, multiple)
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let values: Vec<CalcResult> = args
            .iter()
            .map(|arg| self.evaluate_node_in_context(arg, cell))
            .collect();
        // Excel: the arguments are judged left to right. A missing one is #N/A and a boolean
        // #VALUE!, unlike most numeric functions; the first error or non-numeric text ends the
        // scan, so it wins over a missing or boolean argument after it (`MROUND(1/0,TRUE)` is
        // #DIV/0!, `MROUND("x",)` #VALUE!, `MROUND(TRUE,)` #VALUE! but `MROUND(,TRUE)` #N/A:
        // probes/ceiling-floor-mround-array-lift A23, A27:A42, docs/excel-semantics/arrays.md).
        for value in &values {
            match value {
                CalcResult::EmptyArg => {
                    return CalcResult::new_error(
                        Error::NA,
                        cell,
                        "Bad argument for MROUND".to_string(),
                    );
                }
                CalcResult::Boolean(_) => {
                    return CalcResult::new_error(
                        Error::VALUE,
                        cell,
                        "Expecting number".to_string(),
                    );
                }
                // Excel: an Analysis ToolPak function gives one #VALUE!, no spill, for a multi-cell
                // range in either argument, numbers-only too, while an array constant lifts
                // (probes/ceiling-floor-mround-array-lift A21, A43:A49, A55; compare ISEVEN).
                CalcResult::Range { left, right } if left != right => {
                    return CalcResult::new_error(
                        Error::VALUE,
                        cell,
                        "Expecting a single value".to_string(),
                    );
                }
                CalcResult::Error { .. } => break,
                CalcResult::String(text) if self.cast_number(text).is_none() => break,
                _ => {}
            }
        }
        let operands = values
            .into_iter()
            .map(|value| match self.number_or_array(value, cell) {
                // Booleans are not cast in a scalar MROUND, so not in an array one either
                // (probes/ceiling-floor-mround-array-lift A19:A20).
                Ok(NumberOrArray::Array(mut rows)) => {
                    for node in rows.iter_mut().flatten() {
                        if matches!(node, ArrayNode::Boolean(_)) {
                            *node = ArrayNode::Error(Error::VALUE);
                        }
                    }
                    Ok(NumberOrArray::Array(rows))
                }
                other => other,
            })
            .collect();
        self.lift_number_operands(operands, cell, "Invalid arguments", |[value, multiple]| {
            if multiple == 0.0 {
                return Ok(0.0);
            }
            if (value > 0.0 && multiple < 0.0) || (value < 0.0 && multiple > 0.0) {
                return Err(Error::NUM);
            }
            Ok((value / multiple).round() * multiple)
        })
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
    single_number_fn!(fn_sin, |f| trig_arg(f).map(|x| excel_sin_cos(x).0));
    single_number_fn!(fn_cos, |f| trig_arg(f).map(|x| excel_sin_cos(x).1));
    single_number_fn!(fn_tan, |f| trig_arg(f).map(excel_tan));
    single_number_fn!(fn_sinh, |f| Ok(super::hyperbolic::sinh(f)));
    single_number_fn!(fn_cosh, |f| Ok(super::hyperbolic::cosh(f)));
    single_number_fn!(fn_tanh, |f| Ok(super::hyperbolic::tanh(f)));
    // Excel's ASIN is not libm's: `atan(x / sqrt((1 - x) * (1 + x)))` in doubles gives 233 of the 268
    // cells of probes/asin-argument-sweep (libm `asin` 186), `ASIN(-0.0001)` and `ASIN(0.42)` among
    // them. Excel's is not even odd (`ASIN(0.09)` and `ASIN(-0.09)` differ in the last digit), so no
    // formulation found yet fits every cell (docs/excel-semantics/numbers.md "ASIN of a small
    // argument"). At ±1 the quotient is ±inf and atan gives ±π/2; outside [-1, 1] the square root is
    // NaN, which is `#NUM!`.
    single_number_fn!(fn_asin, |f: f64| Ok(f64::atan(
        f / ((1.0 - f) * (1.0 + f)).sqrt()
    )));
    single_number_fn!(fn_acos, |f| Ok(f64::acos(f)));
    single_number_fn!(fn_atan, |f| Ok(f64::atan(f)));
    single_number_fn!(fn_asinh, |f| Ok(f64::asinh(f)));
    single_number_fn!(fn_acosh, |f| Ok(f64::acosh(f)));
    single_number_fn!(fn_atanh, |f| Ok(f64::atanh(f)));
    single_number_fn!(fn_abs, |f| Ok(f64::abs(f)));
    single_number_fn!(fn_sqrt, |f| if f < 0.0 {
        Err(Error::NUM)
    } else {
        Ok(extended_sqrt(f))
    });
    /// SQRTPI. Unlike RADIANS, a boolean argument is `#VALUE!` (a literal, a comparison, a cell
    /// reference or an array element), while numeric text is read (probes/radians-sqrtpi-munit,
    /// docs/excel-semantics/numbers.md "RADIANS, SQRTPI and MUNIT", T-1282).
    pub(crate) fn fn_sqrtpi(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        let value = self.evaluate_node_in_context(&args[0], cell);
        // A range of several cells is one `#VALUE!`, never a spill, whatever it holds (numbers,
        // text or blanks); an array constant still lifts (probes/sqrtpi-range-operands,
        // radians-sqrtpi-munit A80; docs/excel-semantics/numbers.md "RADIANS, SQRTPI and MUNIT").
        if let CalcResult::Range { left, right } = &value {
            if left != right {
                return CalcResult::new_error(Error::VALUE, cell, "Expecting number".to_string());
            }
        }
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
    single_number_fn!(fn_cot, |f| if trig_arg(f).is_err() {
        Err(Error::NUM)
    } else if f == 0.0 {
        Err(Error::DIV)
    } else {
        // 1/TAN, not COS/SIN (probes-numeric/m9-trig-reduction F6-F81).
        Ok(1.0 / excel_tan(f))
    });
    single_number_fn!(fn_coth, |f: f64| if f == 0.0 {
        Err(Error::DIV)
    } else if f.abs() > 20.0 {
        // for values > 20.0 this is exact in f64
        Ok(f.signum())
    } else {
        Ok(f64::cosh(f) / f64::sinh(f))
    });
    single_number_fn!(fn_csc, |f| if trig_arg(f).is_err() {
        Err(Error::NUM)
    } else if f == 0.0 {
        Err(Error::DIV)
    } else {
        Ok(1.0 / excel_sin_cos(f).0)
    });
    single_number_fn!(fn_csch, |f| if f == 0.0 {
        Err(Error::DIV)
    } else {
        // Excel: a subnormal CSCH result is a true 0 (CSCH(710), docs/excel-semantics/numbers.md).
        Ok(flush_subnormal(1.0 / f64::sinh(f)))
    });
    single_number_fn!(fn_sec, |f| trig_arg(f).map(|x| 1.0 / excel_sin_cos(x).1));
    single_number_fn!(fn_sech, |f| Ok(1.0 / f64::cosh(f)));
    single_number_fn!(fn_exp, |f: f64| Ok(f64::exp(f)));
    single_number_fn!(fn_fact, |x: f64| {
        let x = x.floor();
        if x < 0.0 {
            return Err(Error::NUM);
        }
        descending_product(x, 1.0)
    });
    single_number_fn!(fn_factdouble, |x: f64| {
        let x = x.floor();
        if x < -1.0 {
            return Err(Error::NUM);
        }
        if x < 0.0 {
            return Ok(1.0);
        }
        descending_product(x, 2.0)
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

    // PERCENTOF(part, total) — returns SUM(part) / SUM(total) as a decimal fraction. Both sums snap
    // near zero like SUM: `PERCENTOF(A1:A3,1)` over 0.1, 0.2, -0.3 is 0 and `PERCENTOF(1,A1:A3)` is
    // #DIV/0! (probes/percentof-snap-overflow-union, excel-graph; numbers.md "Near-zero snapping").
    pub(crate) fn fn_percentof(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let part = self.fn_sum(&args[0..1], cell);
        percent_of(part, || self.fn_sum(&args[1..2], cell), cell)
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

/// PERCENTOF from the SUM of its subset, `part`, and the SUM of all, `total`, computed only when
/// `part` is a number (the first error wins). GROUPBY and PIVOTBY call it with a total they sum
/// once per relative set (SPEC-0006 Design step 3).
pub(crate) fn percent_of(
    part: CalcResult,
    total: impl FnOnce() -> CalcResult,
    cell: CellReferenceIndex,
) -> CalcResult {
    let CalcResult::Number(part) = part else {
        return part;
    };
    let total = match total() {
        CalcResult::Number(v) => v,
        other => return other,
    };
    // Divided like the operator `/`: a zero (or subnormal) total is #DIV/0! and an overflowing
    // quotient #NUM! (the latter two UNVERIFIED for PERCENTOF).
    match divide(part, total) {
        Ok(ratio) => CalcResult::Number(ratio),
        Err(error) => CalcResult::new_error(error, cell, "Invalid division".to_string()),
    }
}

/// `value / significance` with Excel's precision applied, so floating-point noise does not
/// push an exact multiple over a step boundary.
fn snap_ratio(value: f64, significance: f64) -> f64 {
    to_excel_precision(value / significance, EXCEL_PRECISION)
}

/// `n * (n - step) * (n - 2 step) * ...` down to 1 or 2, multiplied from `n` down, which is the
/// order Excel's FACT (`step` 1) and FACTDOUBLE (`step` 2) round in: `FACT(170)` is
/// 7.257415615308004e306 where the ascending product gives ...7994e306 (probes/fact-product-order-sweep,
/// 420 cells bit-exact; docs/excel-semantics/numbers.md "FACT and FACTDOUBLE"). `#NUM!` once the
/// product overflows (`FACT(171)`, `FACTDOUBLE(301)`); the first factors are the largest, so a huge
/// `n` overflows within a few steps and the loop stays bounded (directive 5).
fn descending_product(n: f64, step: f64) -> Result<f64, Error> {
    let mut acc = 1.0;
    let mut k = n;
    while k > 1.0 {
        acc *= k;
        if !acc.is_finite() {
            return Err(Error::NUM);
        }
        k -= step;
    }
    Ok(acc)
}
