use super::trig::{excel_sin_cos, excel_tan, EXCEL_TRIG_LIMIT};
use crate::arithmetic::{divide, extended_sqrt, flush_subnormal, power};
use crate::cast::NumberOrArray;
use crate::decimal_round::{round_decimal, RoundMode};
use crate::expressions::parser::ArrayNode;
use crate::expressions::types::CellReferenceIndex;
use crate::functions::lift::{div_error, num_error, number_result};
use crate::functions::util::aggregate_args::{AggregatePolicy, ArgReading, ErrorRule, NumberRules};
use crate::functions::Function;
use crate::x87_exp::excel_exp;
use crate::{
    calc_result::CalcResult, expressions::parser::Node, expressions::token::Error, model::Model,
};
use std::f64::consts::{FRAC_PI_2, PI};

/// Smallest `|number/divisor|` for which Excel's `MOD` returns `#NUM!`: a decimal limit, not a
/// power of two (probes/mod-quotient-threshold5).
const MOD_MAX_QUOTIENT: f64 = 1.1259E+12;

/// The largest `min_length` BASE accepts (probes/base-min-length).
const BASE_MAX_MIN_LENGTH: f64 = 255.0;

/// SIN, COS, TAN, COT, SEC and CSC are `#NUM!` from |x| = 2^27 (SIN(2^27-1) is a number, SIN(2^27)
/// is `#NUM!`; probes/trig-huge-argument, docs/excel-semantics/numbers.md "Trig of a huge angle").
fn trig_arg(f: f64) -> Result<f64, Error> {
    if f.abs() >= EXCEL_TRIG_LIMIT {
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

    /// A one-number math function on the SPEC-0011 driver (`LIFT_1`): `op` per element. A NaN
    /// result (ACOS(2), ACOSH(-1), ...) is `#NUM!` at once: left as a number it is swallowed by an
    /// enclosing function (`x^0` is 1, `ADDRESS` takes row 0, a comparison is FALSE) instead of
    /// reaching the cell as `#NUM!` (T-1202, gen-m5/gen-m6/gen-m7 cells).
    fn number_fn(
        &mut self,
        kind: Function,
        args: &[Node],
        cell: CellReferenceIndex,
        op: fn(f64) -> Result<f64, Error>,
    ) -> CalcResult {
        self.lift_call(kind, args, cell, &mut |model, values| {
            let [value] = values else {
                return CalcResult::new_args_number_error(cell);
            };
            match model.kernel_number(value, cell) {
                Ok(number) => number_result(
                    op(number).and_then(|x| if x.is_nan() { Err(Error::NUM) } else { Ok(x) }),
                    cell,
                ),
                Err(error) => error,
            }
        })
    }

    /// A math function of `N` numbers on the SPEC-0011 driver, an omitted optional one taking its
    /// entry in `defaults` (only the trailing ones are read): `op` per element, the leftmost
    /// argument's error winning.
    pub(super) fn numbers_fn<const N: usize>(
        &mut self,
        kind: Function,
        args: &[Node],
        defaults: [f64; N],
        cell: CellReferenceIndex,
        op: impl Fn([f64; N]) -> Result<f64, Error>,
    ) -> CalcResult {
        self.lift_call(kind, args, cell, &mut |model, values| {
            let mut numbers = defaults;
            for (index, slot) in numbers.iter_mut().enumerate() {
                match model.kernel_number_or(values.get(index), *slot, cell) {
                    Ok(number) => *slot = number,
                    Err(error) => return error,
                }
            }
            number_result(op(numbers), cell)
        })
    }

    /// ROUND, ROUNDUP, ROUNDDOWN and TRUNC: `(number, num_digits)`, rounded in
    /// decimal (see [`round_decimal`]); a result beyond the double range is `#NUM!`.
    /// TRUNC alone may omit `num_digits` (0). Lifts element-wise over arrays and ranges
    /// (gen-m2/893302d6c5b99e2e Calc!A313).
    fn round_family(
        &mut self,
        kind: Function,
        args: &[Node],
        cell: CellReferenceIndex,
        mode: RoundMode,
    ) -> CalcResult {
        self.numbers_fn(kind, args, [0.0, 0.0], cell, |[value, digits]| {
            round_decimal(value, digits, mode).ok_or(Error::NUM)
        })
    }

    pub(crate) fn fn_round(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.round_family(Function::Round, args, cell, RoundMode::HalfAwayFromZero)
    }

    pub(crate) fn fn_roundup(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.round_family(Function::Roundup, args, cell, RoundMode::Up)
    }

    pub(crate) fn fn_rounddown(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.round_family(Function::Rounddown, args, cell, RoundMode::Down)
    }

    /// TRUNC is ROUNDDOWN with an optional `num_digits`: Excel cuts to 15 digits
    /// first and wraps huge digit counts the same way (docs/excel-semantics/numbers.md
    /// "TRUNC is ROUNDDOWN", probes/trunc-decimal-stages).
    pub(crate) fn fn_trunc(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.round_family(Function::Trunc, args, cell, RoundMode::Down)
    }

    // (number, divisor); lifts element-wise over arrays and ranges (probes/geomean-overflow-cut-3 E4)
    pub(crate) fn fn_mod(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.numbers_fn(Function::Mod, args, [0.0, 0.0], cell, |[value, divisor]| {
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

    /// `QUOTIENT(numerator, denominator)`, an Analysis `ToolPak` function: both arguments lift
    /// over an array and refuse a multi-cell range (probes/math-atp-array-lift C1-C2, C7-C8, C27-C28;
    /// scalar-lift-open-a Y19-Y20), and a boolean is `#VALUE!` (math-atp-array-lift C21, -b E11-E13).
    /// Within one element the numerator's error wins (C29-C30).
    pub(crate) fn fn_quotient(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Quotient, args, cell, &mut |model, values| {
            let (value, divisor) = match model.two_numbers(values, cell) {
                Ok(pair) => pair,
                Err(error) => return error,
            };
            if divisor == 0.0 {
                return div_error(cell);
            }
            let result = value / divisor;
            CalcResult::Number(result.signum() * result.abs().floor())
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

    pub(crate) fn fn_log10(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.number_fn(Function::Log10, args, cell, |f| {
            if f <= 0.0 {
                Err(Error::NUM)
            } else {
                Ok(f64::log10(f))
            }
        })
    }
    pub(crate) fn fn_ln(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.number_fn(Function::Ln, args, cell, |f| {
            if f <= 0.0 {
                Err(Error::NUM)
            } else {
                Ok(f64::ln(f))
            }
        })
    }
    pub(crate) fn fn_sin(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.number_fn(Function::Sin, args, cell, |f| {
            trig_arg(f).map(|x| excel_sin_cos(x).0)
        })
    }
    pub(crate) fn fn_cos(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.number_fn(Function::Cos, args, cell, |f| {
            trig_arg(f).map(|x| excel_sin_cos(x).1)
        })
    }
    pub(crate) fn fn_tan(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.number_fn(Function::Tan, args, cell, |f| trig_arg(f).map(excel_tan))
    }
    pub(crate) fn fn_sinh(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.number_fn(Function::Sinh, args, cell, |f| {
            Ok(super::hyperbolic::sinh(f))
        })
    }
    pub(crate) fn fn_cosh(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.number_fn(Function::Cosh, args, cell, |f| {
            Ok(super::hyperbolic::cosh(f))
        })
    }
    pub(crate) fn fn_tanh(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.number_fn(Function::Tanh, args, cell, |f| {
            Ok(super::hyperbolic::tanh(f))
        })
    }
    pub(crate) fn fn_asin(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.number_fn(Function::Asin, args, cell, |f| Ok(excel_asin(f)))
    }
    // Excel's ACOS is `PI()/2 - ASIN(x)` exactly, on ASIN's kernel: all 240 ACOS cells of
    // probes/asin-atan-sign-source (columns H and I, excel-graph, T-1797), where libm `acos` fits
    // 157 (docs/excel-semantics/numbers.md "ASIN's formulation"). Outside [-1, 1] ASIN's NaN
    // stays NaN, which is `#NUM!`.
    pub(crate) fn fn_acos(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.number_fn(
            Function::Acos,
            args,
            cell,
            |f| Ok(FRAC_PI_2 - excel_asin(f)),
        )
    }
    pub(crate) fn fn_atan(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.number_fn(Function::Atan, args, cell, |f| Ok(f64::atan(f)))
    }
    pub(crate) fn fn_asinh(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.number_fn(Function::Asinh, args, cell, |f| Ok(f64::asinh(f)))
    }
    pub(crate) fn fn_acosh(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.number_fn(Function::Acosh, args, cell, |f| Ok(f64::acosh(f)))
    }
    // ATANH(+-1) is `#NUM!` at once, like any other out-of-domain argument; f64::atanh gives an
    // infinity there, which the macro's NaN mapping misses (probes/atanh-unit-argument A1:A8,
    // excel-graph, T-1569).
    pub(crate) fn fn_atanh(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.number_fn(Function::Atanh, args, cell, |f: f64| {
            if f.abs() == 1.0 {
                Err(Error::NUM)
            } else {
                Ok(f64::atanh(f))
            }
        })
    }
    pub(crate) fn fn_abs(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.number_fn(Function::Abs, args, cell, |f| Ok(f64::abs(f)))
    }
    pub(crate) fn fn_sqrt(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.number_fn(Function::Sqrt, args, cell, |f| {
            if f < 0.0 {
                Err(Error::NUM)
            } else {
                Ok(extended_sqrt(f))
            }
        })
    }
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

    pub(crate) fn fn_acot(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.number_fn(Function::Acot, args, cell, |f| {
            let v = f64::atan(1.0 / f);
            if f >= 0.0 {
                Ok(v)
            } else {
                // To be compatible with Excel we need a different branch
                // when f < 0
                Ok(v + PI)
            }
        })
    }
    pub(crate) fn fn_acoth(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.number_fn(Function::Acoth, args, cell, |f: f64| {
            if f.abs() == 1.0 {
                Err(Error::NUM)
            } else {
                Ok(0.5 * (f64::ln((f + 1.0) / (f - 1.0))))
            }
        })
    }
    pub(crate) fn fn_cot(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.number_fn(Function::Cot, args, cell, |f| {
            if trig_arg(f).is_err() {
                Err(Error::NUM)
            } else if f == 0.0 {
                Err(Error::DIV)
            } else {
                // 1/TAN, not COS/SIN (probes-numeric/m9-trig-reduction F6-F81).
                Ok(1.0 / excel_tan(f))
            }
        })
    }
    pub(crate) fn fn_coth(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.number_fn(Function::Coth, args, cell, |f: f64| {
            if f == 0.0 {
                Err(Error::DIV)
            } else if f.abs() > 20.0 {
                // for values > 20.0 this is exact in f64
                Ok(f.signum())
            } else {
                Ok(f64::cosh(f) / f64::sinh(f))
            }
        })
    }
    pub(crate) fn fn_csc(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.number_fn(Function::Csc, args, cell, |f| {
            if trig_arg(f).is_err() {
                Err(Error::NUM)
            } else if f == 0.0 {
                Err(Error::DIV)
            } else {
                Ok(1.0 / excel_sin_cos(f).0)
            }
        })
    }
    pub(crate) fn fn_csch(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.number_fn(Function::Csch, args, cell, |f| {
            if f == 0.0 {
                Err(Error::DIV)
            } else {
                // Excel: a subnormal CSCH result is a true 0 (CSCH(710), docs/excel-semantics/numbers.md).
                Ok(flush_subnormal(1.0 / f64::sinh(f)))
            }
        })
    }
    pub(crate) fn fn_sec(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.number_fn(Function::Sec, args, cell, |f| {
            trig_arg(f).map(|x| 1.0 / excel_sin_cos(x).1)
        })
    }
    pub(crate) fn fn_sech(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.number_fn(Function::Sech, args, cell, |f| Ok(1.0 / f64::cosh(f)))
    }
    /// `EXP(number)`: Excel's own x87 evaluation, model V6, which is not the correctly rounded exp
    /// (docs/excel-semantics/numbers.md "EXP is an x87 2^(a*log2 e) evaluation, not correctly
    /// rounded"; probes-numeric `t1931-exp-*`). A subnormal result is kept (probes/
    /// subnormal-function-results C4, C6), and an overflow is infinite, which the cell turns into
    /// `#NUM!` as before.
    pub(crate) fn fn_exp(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.number_fn(Function::Exp, args, cell, |f: f64| Ok(excel_exp(f)))
    }
    pub(crate) fn fn_fact(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.number_fn(Function::Fact, args, cell, |x: f64| {
            let x = x.floor();
            if x < 0.0 {
                return Err(Error::NUM);
            }
            descending_product(x, 1.0)
        })
    }
    /// `FACTDOUBLE(number)`, an Analysis `ToolPak` function: it lifts over an array and refuses a
    /// multi-cell range (probes/math-atp-array-lift C3-C4, scalar-lift-open-a Y21), and a boolean is
    /// `#VALUE!`, typed, in a cell or as an array element (probes/math-atp-array-lift-b E1-E3, E8;
    /// docs/excel-semantics/numbers.md "QUOTIENT, FACTDOUBLE and SERIESSUM argument modes").
    pub(crate) fn fn_factdouble(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Factdouble, args, cell, &mut |model, values| {
            let [number] = values else {
                return CalcResult::new_args_number_error(cell);
            };
            let x = match model.cast_number_no_bools_ref(number, cell) {
                Ok(x) => x.floor(),
                Err(error) => return error,
            };
            if x < -1.0 {
                return num_error(cell);
            }
            if x < 0.0 {
                return CalcResult::Number(1.0);
            }
            match descending_product(x, 2.0) {
                Ok(product) => CalcResult::Number(product),
                Err(_) => num_error(cell),
            }
        })
    }
    pub(crate) fn fn_sign(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.number_fn(Function::Sign, args, cell, |f| {
            if f == 0.0 {
                Ok(0.0)
            } else {
                Ok(f64::signum(f))
            }
        })
    }
    pub(crate) fn fn_degrees(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.number_fn(Function::Degrees, args, cell, |f| Ok(f * (180.0 / PI)))
    }
    pub(crate) fn fn_radians(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.number_fn(Function::Radians, args, cell, |f| Ok(f * (PI / 180.0)))
    }
    pub(crate) fn fn_odd(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.number_fn(Function::Odd, args, cell, |f| {
            let sign = f64::signum(f);
            Ok(sign * (f64::ceil((f64::abs(f) - 1.0) / 2.0) * 2.0 + 1.0))
        })
    }
    pub(crate) fn fn_even(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.number_fn(Function::Even, args, cell, |f| {
            Ok(f64::signum(f) * f64::ceil(f64::abs(f) / 2.0) * 2.0)
        })
    }
    pub(crate) fn fn_int(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.number_fn(Function::Int, args, cell, |f| Ok(f64::floor(f)))
    }

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
        // Excel: see docs/excel-semantics/numbers.md "ATAN2 is ATAN(y/x) plus or minus pi" (probes/atan2-*).
        if x == 0.0 {
            if y == 0.0 {
                return CalcResult::Error {
                    error: Error::DIV,
                    origin: cell,
                    message: "Arguments can't be both zero".to_string(),
                };
            }
            return CalcResult::Number(FRAC_PI_2.copysign(y));
        }
        // The quotient is Excel's `/`: a subnormal x divides like zero (#DIV/0!), overflow is #NUM!.
        let q = match divide(y, x) {
            Ok(q) => q,
            Err(error) => {
                return CalcResult::Error {
                    error,
                    origin: cell,
                    message: "Invalid quotient".to_string(),
                }
            }
        };
        // `+ 0.0` turns -0 into +0, which Excel never returns.
        let a = flush_subnormal(q.atan()) + 0.0;
        CalcResult::Number(if x < 0.0 {
            if y >= 0.0 {
                a + PI
            } else {
                a - PI
            }
        } else {
            a
        })
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
        match combin(n, k) {
            Ok(result) => CalcResult::Number(result),
            Err(error) => CalcResult::Error {
                error,
                origin: cell,
                message: "Result is too large".to_string(),
            },
        }
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

/// COMBIN(n, k) for integers `0 <= k <= n`, rounded as Excel rounds it. With `j = min(k, n-k)`,
/// Excel multiplies the factors `(n-j+i)/(i+1)` for i = 1..j-1, each a rounded quotient, into one
/// double running product and then multiplies by `n`: `COMBIN(10,4)` is 209.99999999999997, not 210.
/// This matches all 841 numeric cells of probes/combin-algorithm-sweep bit for bit (numbers.md
/// "COMBIN is inexact"). Every factor is at least 1, so the product only grows: it overflows (`#NUM!`,
/// as `COMBIN(1030,515)` is in Excel) within a few hundred
/// factors whenever `j` is large, which bounds the work (directive 5).
fn combin(n: f64, k: f64) -> Result<f64, Error> {
    let j = k.min(n - k);
    if j < 1.0 {
        return Ok(1.0);
    }
    let mut acc = 1.0;
    let mut i = 1.0;
    while i < j {
        acc *= (n - j + i) / (i + 1.0);
        if !acc.is_finite() {
            return Err(Error::NUM);
        }
        i += 1.0;
    }
    let result = acc * n;
    if result.is_finite() {
        Ok(result)
    } else {
        Err(Error::NUM)
    }
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

/// Excel's ASIN is `atan(x / sqrt(y * (2 - y)))` with y = 1 - x: the factor 1 + x is formed as
/// 2 - y, and the square root rounds twice like SQRT. That fits all 570 cells of
/// probes/asin-argument-sweep and probes/asin-negative-band-sweep (excel-graph, T-1797), including
/// the ± pairs that differ (`ASIN(0.09)` and `ASIN(-0.09)`): rounding y drops the last bit of a
/// negative x in -1..-0.5, so ASIN is not odd. libm `asin` fits 186 of the sweep's 268 and
/// `atan(x / sqrt((1 - x) * (1 + x)))` 235 (docs/excel-semantics/numbers.md "ASIN's formulation").
/// At ±1 the quotient is ±inf and atan gives ±π/2; outside [-1, 1] the square root is NaN, which
/// is `#NUM!`.
fn excel_asin(f: f64) -> f64 {
    let y = 1.0 - f;
    f64::atan(f / extended_sqrt(y * (2.0 - y)))
}
