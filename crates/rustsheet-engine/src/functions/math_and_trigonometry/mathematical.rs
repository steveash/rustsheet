use super::trig::{excel_cos, excel_sin, excel_tan, EXCEL_TRIG_LIMIT};
use crate::arithmetic::{divide, extended_quotient, extended_sqrt, flush_subnormal, power};
use crate::cast::NumberOrArray;
use crate::decimal_round::{round_decimal, round_function, RoundMode};
use crate::expressions::types::CellReferenceIndex;
use crate::functions::lift::{div_error, num_error, number_result, value_error};
use crate::functions::statistical::IfsShape;
use crate::functions::util::aggregate_args::{
    AggregatePolicy, ArgReading, ErrorRule, GridReading, NestedReading, NumberRules,
};
use crate::functions::Function;
use crate::x87_exp::excel_exp;
use crate::x87_log10::{excel_ln, excel_log10};
use crate::{
    calc_result::CalcResult, expressions::parser::Node, expressions::token::Error, model::Model,
};
use std::f64::consts::{FRAC_PI_2, PI};

/// Smallest `|number/divisor|` for which Excel's `MOD` returns `#NUM!`: a decimal limit, not a
/// power of two (probes/mod-quotient-threshold5).
const MOD_MAX_QUOTIENT: f64 = 1.1259E+12;

/// The largest `min_length` BASE accepts (probes/base-min-length).
const BASE_MAX_MIN_LENGTH: f64 = 255.0;
/// BASE refuses a `number` of 2^53 or more (2^53 - 1 is accepted).
const BASE_NUMBER_LIMIT: f64 = 9_007_199_254_740_992.0;
/// COMBIN's `number`, and COMBINA's `number + number_chosen - 1`, at or above which Excel gives
/// `#NUM!`, whatever the coefficient: 2^31-1 (probes/t2199-text-length-and-combin-cuts A20-A37,
/// B20-B34; numbers.md "COMBINA is COMBIN(n+k-1, k)", "Bound").
const COMBIN_MAX_NUMBER: f64 = crate::limits::COUNT_LIMIT_EXCLUSIVE;

/// SIN, COS, TAN, COT, SEC and CSC are `#NUM!` from |x| = 2^27 (SIN(2^27-1) is a number, SIN(2^27)
/// is `#NUM!`; probes/trig-huge-argument, docs/excel-semantics/numbers.md "Trig of a huge angle").
fn trig_arg(f: f64) -> Result<f64, Error> {
    if f.abs() >= EXCEL_TRIG_LIMIT {
        Err(Error::NUM)
    } else {
        Ok(f)
    }
}

/// COT, CSC and SEC (and COTH, CSCH, SECH): 1 over the rounded TAN, SIN or COS (TANH, SINH,
/// COSH), divided as the `/` operator divides
/// (rounded to 64 bits, then to a double), so `COT(PI()/4)` is 1 and `CSC(PI()/6)` 2 like
/// `1/TAN(PI()/4)` and `1/SIN(PI()/6)` (probes/trig-reciprocal-special-angles; ironcalc-calc
/// TRIGONOMETRIC.xlsx `COT_CSC_SEC` B47, C48; numbers.md "Division rounds twice").
fn reciprocal_trig(value: f64) -> f64 {
    extended_quotient(1.0, value)
}

/// MIN and MAX: a referenced-cell error ranks below a direct scalar error; the leftmost wins
/// (probes/aggregate-error-precedence C9-C24). A direct LAMBDA's `#VALUE!` ranks with the
/// referenced errors, as does an array error (every `Ranked` policy: probes/aggregate-direct-lambda-
/// product-min-max D15, D24, T-1604; probes/aggregate-array-error-vs-ref-error, T-1615).
const MIN_MAX_ARGS: AggregatePolicy = AggregatePolicy {
    reading: ArgReading::Aggregate,
    errors: ErrorRule::Ranked,
    skip_empty_args: false,
    grids: GridReading::TopLeft,
    nested: NestedReading::NotAvailable,
};

/// LOG of `x` in `base`, or LOG10 with the base omitted: not ln(x)/ln(10), as `LOG(1000)` is 3
/// (numbers.md "LOG with one argument is LOG10"; probes/log10-kernel, T-2004).
fn excel_log(x: f64, base: Option<f64>) -> Result<f64, Error> {
    if x <= 0.0 {
        return Err(Error::NUM);
    }
    match base {
        None => Ok(excel_log10(x)),
        Some(1.0) => Err(Error::DIV),
        Some(base) if base <= 0.0 => Err(Error::NUM),
        Some(base) => Ok(f64::log(x, base)),
    }
}

/// BASE's text of `number` in `radix`, padded with zeros to `min_length`.
fn base_text(number: f64, radix: i64, min_length: f64) -> Result<String, Error> {
    // Excel: min_length truncates and may be at most 255 (`BASE(1,2,255.9)` pads to 255,
    // `BASE(1,2,256)` is #NUM!; probes/base-min-length). Checking before padding also bounds the
    // allocation (CLAUDE.md directive 5).
    if min_length < 0.0 || min_length.trunc() > BASE_MAX_MIN_LENGTH {
        return Err(Error::NUM);
    }
    let min_length = min_length.trunc() as usize;
    if !(2..=36).contains(&radix) {
        return Err(Error::NUM);
    }
    // number must be >= 0, checked before truncation: `BASE(-0.0001,5,4)` is `#NUM!`
    // (gen-m5/a6fec430ac350c1a Calc!A31; numbers.md "ROMAN truncates before its sign check; BASE checks the sign first").
    if number < 0.0 {
        return Err(Error::NUM);
    }
    // Excel: 2^53 - 1 is accepted, 2^53 and above are #NUM! in every radix, with or without
    // `min_length` (probes/base-number-upper-bound; numbers.md "BASE past 2^53 is #NUM!").
    if number >= BASE_NUMBER_LIMIT {
        return Err(Error::NUM);
    }
    let mut value = number.trunc() as i64;
    // Excel: a zero has no digits of its own, only the padding to `min_length`, so an explicit
    // `min_length` of 0 gives empty text (`BASE(0.2,29,0.3)`, `LENB` 0) while an omitted one
    // defaults to 1 and gives "0" (numbers.md "BASE of zero with min_length 0 is empty").
    let mut digits = Vec::new();
    while value > 0 {
        // `radix` is 2..=36, so the digit is below 36 and `from_digit` never fails: the `'0'`
        // fallback is unreachable.
        digits.push(
            char::from_digit((value % radix) as u32, 36).map_or('0', |c| c.to_ascii_uppercase()),
        );
        value /= radix;
    }
    let padding = min_length.saturating_sub(digits.len());
    let mut text = String::with_capacity(padding + digits.len());
    text.extend(std::iter::repeat_n('0', padding));
    text.extend(digits.iter().rev());
    Ok(text)
}

/// Excel's ATAN2(x, y): `ATAN(y/x)` plus or minus π, not libm `atan2(y, x)`; see
/// docs/excel-semantics/numbers.md "ATAN2 is ATAN(y/x) plus or minus π" (probes/atan2-*).
/// IMARGUMENT is this kernel too (probes-numeric im-last-digit-sweep, T-2044).
pub(crate) fn excel_atan2(x: f64, y: f64) -> Result<f64, Error> {
    if x == 0.0 {
        if y == 0.0 {
            return Err(Error::DIV);
        }
        return Ok(FRAC_PI_2.copysign(y));
    }
    // The quotient is Excel's `/`: a subnormal x divides like zero (#DIV/0!), overflow is #NUM!.
    let q = divide(y, x)?;
    // `+ 0.0` turns -0 into +0, which Excel never returns.
    let a = flush_subnormal(q.atan()) + 0.0;
    Ok(if x < 0.0 {
        if y >= 0.0 {
            a + PI
        } else {
            a - PI
        }
    } else {
        a
    })
}

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

    /// `BASE(number, radix, [min_length])`: every argument lifts over arrays and ranges
    /// (probes/t2198-m5-lift-argument-kinds; arrays.md). The arguments are read left to right,
    /// the first error winning, then checked: `min_length`, the radix, the number's sign.
    pub(crate) fn fn_base(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Base, args, cell, &mut |model, values| {
            let (number, radix, min_length) = match values {
                [number, radix] => (number, radix, None),
                [number, radix, min_length] => (number, radix, Some(min_length)),
                _ => return CalcResult::new_args_number_error(cell),
            };
            let number = match model.kernel_number(number, cell) {
                Ok(f) => f,
                Err(error) => return error,
            };
            let radix = match model.kernel_number(radix, cell) {
                Ok(f) => f.trunc() as i64,
                Err(error) => return error,
            };
            let min_length = match model.kernel_number_or(min_length, 1.0, cell) {
                Ok(f) => f,
                Err(error) => return error,
            };
            match base_text(number, radix, min_length) {
                Ok(text) => CalcResult::String(text.into()),
                Err(error) => CalcResult::new_error(error, cell, String::new()),
            }
        })
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
            2 => (&args[0], &args[1], None),
            3 => (&args[0], &args[1], Some(&args[2])),
            _ => return CalcResult::new_args_number_error(cell),
        };
        self.sumif(criteria_range, criteria, sum_range, cell)
    }

    /// `SUMIFS(sum_range`, `criteria_range1`, criteria1, [`criteria_range2`, criteria2], ...)
    pub(crate) fn fn_sumifs(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.sum_ifs(args, cell, IfsShape::Strict)
    }

    /// A one-number math function on the SPEC-0011 driver (`LIFT_1`): `op` per element. A
    /// non-finite result (NaN: ACOS(2), ACOSH(-1); infinite: SINH(1E+9), EXP(2^53+1)) is `#NUM!`
    /// at once: left as a number it is swallowed by an enclosing function (`x^0` is 1, `ADDRESS`
    /// takes row 0, a comparison is FALSE) instead of reaching the cell as `#NUM!` (T-1202,
    /// T-2169, gen-m5/gen-m6/gen-m7 cells).
    fn number_fn(
        &mut self,
        kind: Function,
        args: &[Node],
        cell: CellReferenceIndex,
        op: fn(f64) -> Result<f64, Error>,
    ) -> CalcResult {
        // `numbers_fn` reads its one number as `kernel_number` does (`LIFT_1` takes one argument).
        self.numbers_fn(kind, args, [0.0], cell, |[number]| {
            op(number).and_then(|x| {
                if x.is_finite() {
                    Ok(x)
                } else {
                    Err(Error::NUM)
                }
            })
        })
    }

    /// A math function of `N` numbers on the SPEC-0011 driver, an omitted optional one taking its
    /// entry in `defaults` (only the trailing ones are read): `op` per element, the leftmost
    /// argument's error winning.
    pub(crate) fn numbers_fn<const N: usize>(
        &mut self,
        kind: Function,
        args: &[Node],
        defaults: [f64; N],
        cell: CellReferenceIndex,
        op: impl Fn([f64; N]) -> Result<f64, Error>,
    ) -> CalcResult {
        // The driver's shortcut (T-1932): the arguments written, all numbers, then the defaults.
        let shortcut = |present: &[f64]| {
            let mut numbers = defaults;
            for (slot, number) in numbers.iter_mut().zip(present) {
                *slot = *number;
            }
            op(numbers)
        };
        self.lift_call_numbers(kind, args, cell, &shortcut, &mut |model, values| {
            let mut numbers = defaults;
            for (index, slot) in numbers.iter_mut().enumerate() {
                // A number, the usual argument, skips the coercion chain (T-1785 bench).
                if let Some(CalcResult::Number(number)) = values.get(index) {
                    *slot = *number;
                    continue;
                }
                match model.kernel_number_or(values.get(index), *slot, cell) {
                    Ok(number) => *slot = number,
                    Err(error) => return error,
                }
            }
            number_result(op(numbers), cell)
        })
    }

    /// ROUND, ROUNDUP, ROUNDDOWN and TRUNC: `(number, num_digits)`, rounded in
    /// decimal (see [`round_function`]); a result beyond the double range is `#NUM!`.
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
            round_function(value, digits, mode).ok_or(Error::NUM)
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
            crate::functions::util::excel_mod(value, divisor).ok_or(Error::NUM)
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

    /// MROUND(number, multiple), an Analysis `ToolPak` function on the SPEC-0011 driver (`ATP_2`):
    /// it lifts over an array and refuses a multi-cell range (probes/ceiling-floor-mround-array-lift
    /// A21, A43:A49, A55, date-math-lift-driver-edges G19-G20).
    pub(crate) fn fn_mround(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Mround, args, cell, &mut |model, values| {
            // Excel: the arguments are judged left to right. A missing one is #N/A and a boolean
            // #VALUE!, unlike most numeric functions (an array element and a referenced cell too);
            // the first error or non-numeric text ends the scan, so it wins over a missing or
            // boolean argument after it (`MROUND(1/0,TRUE)` is #DIV/0!, `MROUND("x",)` #VALUE!,
            // `MROUND(TRUE,)` #VALUE! but `MROUND(,TRUE)` #N/A: probes/ceiling-floor-mround-array-
            // lift A19:A20, A23, A27:A42, date-math-lift-driver-edges G44-G47, G51-G52;
            // docs/excel-semantics/arrays.md).
            let mut numbers = [0.0; 2];
            for (slot, value) in numbers.iter_mut().zip(values) {
                let number = match value {
                    CalcResult::EmptyArg => {
                        Err(CalcResult::new_error(Error::NA, cell, String::new()))
                    }
                    CalcResult::Boolean(_) => Err(value_error(cell)),
                    other => model.kernel_number(other, cell),
                };
                match number {
                    Ok(number) => *slot = number,
                    Err(error) => return error,
                }
            }
            let [value, multiple] = numbers;
            if multiple == 0.0 {
                return CalcResult::Number(0.0);
            }
            if (value > 0.0 && multiple < 0.0) || (value < 0.0 && multiple > 0.0) {
                return num_error(cell);
            }
            CalcResult::Number(super::ceiling_floor::mround_value(value, multiple))
        })
    }

    pub(crate) fn fn_log10(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.number_fn(Function::Log10, args, cell, |f| {
            if f <= 0.0 {
                Err(Error::NUM)
            } else {
                // Excel's x87 log10, not the platform's (numbers.md "LOG with one argument is
                // LOG10", T-2004).
                Ok(excel_log10(f))
            }
        })
    }
    pub(crate) fn fn_ln(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.number_fn(Function::Ln, args, cell, |f| {
            if f <= 0.0 {
                Err(Error::NUM)
            } else {
                // Excel's x87 FLDLN2 + FYL2X, not the platform's ln (numbers.md "LN is FLDLN2
                // then FYL2X", T-2123).
                Ok(excel_ln(f))
            }
        })
    }
    pub(crate) fn fn_sin(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.number_fn(Function::Sin, args, cell, |f| trig_arg(f).map(excel_sin))
    }
    pub(crate) fn fn_cos(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.number_fn(Function::Cos, args, cell, |f| trig_arg(f).map(excel_cos))
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
    // ASINH and ACOSH are the textbook log formulas in doubles, not libm, so small arguments lose
    // precision exactly as Excel's do: ASINH(1E-10) = 1.000000082690371E-10 and
    // ACOSH(1.000000000001) = 1.414276423083533E-6 (docs/excel-semantics/numbers.md, "Inverse
    // hyperbolics, COTH/CSCH and the SINH/TANH thresholds", probes-numeric/m9-hyperbolic, T-2031).
    // The log is the worksheet LN kernel (`excel_ln`, FYL2X), not libm `ln`: at 240 points chosen
    // where the two differ it is 238 right against 2 (probes-numeric/t2133-asinh-acosh-ln, T-2133).
    // Where `x*x` overflows (|x| above about 1.3407807929942596E154) Excel gives `#NUM!`
    // (probes-numeric/t2062-asinh-acosh-overflow, excel-graph, T-2062).
    pub(crate) fn fn_asinh(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.number_fn(Function::Asinh, args, cell, |f: f64| {
            let square = f * f;
            if square.is_infinite() {
                Err(Error::NUM)
            } else if f < 0.0 {
                Ok(-excel_ln(-f + (square + 1.0).sqrt()))
            } else {
                Ok(excel_ln(f + (square + 1.0).sqrt()))
            }
        })
    }
    pub(crate) fn fn_acosh(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.number_fn(Function::Acosh, args, cell, |f: f64| {
            // Checked first: for a large negative argument `f + sqrt(f*f - 1)` rounds to 0 and the
            // log is -inf, not NaN.
            if f < 1.0 {
                return Err(Error::NUM);
            }
            let square = f * f;
            if square.is_infinite() {
                Err(Error::NUM)
            } else {
                Ok(excel_ln(f + (square - 1.0).sqrt()))
            }
        })
    }
    // ATANH(+-1) is `#NUM!` at once, like any other out-of-domain argument (probes/atanh-unit-
    // argument A1:A8, excel-graph, T-1569). Inside, Excel's x87 formulation (`hyperbolic::atanh`).
    pub(crate) fn fn_atanh(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.number_fn(Function::Atanh, args, cell, |f: f64| {
            if f.abs() >= 1.0 {
                Err(Error::NUM)
            } else {
                Ok(super::hyperbolic::atanh(f))
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
    /// SQRTPI, an Analysis `ToolPak` function on the SPEC-0011 driver (`ARRAYS_1`): a range of
    /// several cells is one `#VALUE!`, never a spill, whatever it holds, while an array constant
    /// lifts (probes/sqrtpi-range-operands, radians-sqrtpi-munit A80). Unlike RADIANS, a boolean
    /// argument is `#VALUE!` (a literal, a comparison, a cell reference or an array element),
    /// while numeric text is read (probes/radians-sqrtpi-munit, docs/excel-semantics/numbers.md
    /// "RADIANS, SQRTPI and MUNIT", T-1282).
    pub(crate) fn fn_sqrtpi(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Sqrtpi, args, cell, &mut |model, values| {
            let number = match values {
                [CalcResult::Boolean(_)] => return value_error(cell),
                [value] => model.kernel_number(value, cell),
                _ => return CalcResult::new_args_number_error(cell),
            };
            match number {
                Ok(f) if f < 0.0 => num_error(cell),
                Ok(f) => CalcResult::Number((f * PI).sqrt()),
                Err(error) => error,
            }
        })
    }

    pub(crate) fn fn_acot(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.number_fn(Function::Acot, args, cell, |f| {
            if f == 0.0 {
                // Excel has no negative zero (numbers.md "Negative zero prints as 0"): ACOT of
                // -0.0 (e.g. CEILING.PRECISE(-0.0001)) is PI()/2 like ACOT(0), T-2376.
                return Ok(PI / 2.0);
            }
            let v = f64::atan(1.0 / f);
            if f > 0.0 {
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
            if f.abs() <= 1.0 {
                Err(Error::NUM)
            } else {
                Ok(super::hyperbolic::acoth(f))
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
                // 1/TAN, not COS/SIN (probes-numeric/m9-trig-reduction F6-F81), divided as `/`
                // divides: rounded twice, so COT(PI()/4) is 1 (`reciprocal_trig`).
                Ok(reciprocal_trig(excel_tan(f)))
            }
        })
    }
    // COTH, CSCH and SECH are 1 over Excel's own TANH, SINH and COSH, divided as `/` divides: all
    // 66 pairs of probes-numeric t0917-hyperbolic-switchover agree bit for bit, including where
    // those are an ulp off libm (docs/excel-semantics/numbers.md "COTH, CSCH and SECH", T-0917).
    pub(crate) fn fn_coth(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.number_fn(Function::Coth, args, cell, |f: f64| {
            if f == 0.0 {
                Err(Error::DIV)
            } else {
                Ok(reciprocal_trig(super::hyperbolic::tanh(f)))
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
                Ok(reciprocal_trig(excel_sin(f)))
            }
        })
    }
    pub(crate) fn fn_csch(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.number_fn(Function::Csch, args, cell, |f| {
            if f == 0.0 {
                Err(Error::DIV)
            } else {
                // Excel: a subnormal CSCH result is a true 0 (CSCH(710), docs/excel-semantics/numbers.md).
                Ok(flush_subnormal(reciprocal_trig(super::hyperbolic::sinh(f))))
            }
        })
    }
    pub(crate) fn fn_sec(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.number_fn(Function::Sec, args, cell, |f| {
            trig_arg(f).map(|x| reciprocal_trig(excel_cos(x)))
        })
    }
    pub(crate) fn fn_sech(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.number_fn(Function::Sech, args, cell, |f| {
            Ok(reciprocal_trig(super::hyperbolic::cosh(f)))
        })
    }
    /// `EXP(number)`: Excel's own x87 evaluation, model V6, which is not the correctly rounded exp
    /// (docs/excel-semantics/numbers.md "EXP is an x87 2^(a*log2 e) evaluation, not correctly
    /// rounded"; probes-numeric `t1931-exp-*`). A subnormal result is kept (probes/
    /// subnormal-function-results C4, C6), and an overflow is `#NUM!` at once (`number_fn`).
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
        // Excel cuts to 15 significant digits first, then rounds down: INT(1-2^-53) = 1
        // (docs/excel-semantics/numbers.md "INT cuts to 15 significant digits first", T-2171).
        // On the cut value, down is toward zero for a positive number and away from zero for a
        // negative one; negatives and the 1E+15 boundary are probed (probes/t2215-int-negatives-boundary).
        // From 1E+15 up the cut would drop integer digits (`INT(2^53+1)` = 2^53, a sum of 1000000000000006
        // stays 1000000000000006: gen-m5/d586a1e0c9896942 Calc!A64, gen-m5/1315170eef6c413e Calc!A332),
        // so a value that large is floored as it is.
        self.number_fn(Function::Int, args, cell, |f| {
            if f.abs() >= 1e15 {
                return Ok(f.floor());
            }
            let mode = if f < 0.0 {
                RoundMode::Up
            } else {
                RoundMode::Down
            };
            Ok(round_decimal(f, 0.0, mode).unwrap_or_else(|| f.floor()))
        })
    }

    pub(crate) fn fn_pi(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if !args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }
        CalcResult::Number(PI)
    }

    /// `ATAN2(x_num, y_num)`: both arguments lift over arrays and ranges
    /// (probes/t2198-m5-lift-argument-kinds; arrays.md).
    pub(crate) fn fn_atan2(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.numbers_fn(Function::Atan2, args, [0.0, 0.0], cell, |[x, y]| {
            excel_atan2(x, y)
        })
    }

    /// `LOG(number, [base])`: both arguments lift over arrays and ranges
    /// (probes/t2198-m5-lift-argument-kinds; arrays.md).
    pub(crate) fn fn_log(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Log, args, cell, &mut |model, values| {
            let (number, base) = match values {
                [number] => (number, None),
                [number, base] => (number, Some(base)),
                _ => return CalcResult::new_args_number_error(cell),
            };
            let x = match model.kernel_number(number, cell) {
                Ok(x) => x,
                Err(error) => return error,
            };
            let base = match base.map(|base| model.kernel_number(base, cell)).transpose() {
                Ok(base) => base,
                Err(error) => return error,
            };
            number_result(excel_log(x, base), cell)
        })
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

    /// COMBIN; both arguments lift over a range or an array, as COMBINA's do:
    /// `SUM(COMBIN(18,FILTER(..)))` and `SUM(COMBIN(D1:D3,1))` add the elements' results
    /// (probes/t2579-array-value-edges B60-B68, excel-graph; T-2579).
    pub(crate) fn fn_combin(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Combin, args, cell, &mut |model, values| {
            let [n, k] = values else {
                return CalcResult::new_args_number_error(cell);
            };
            let n = match model.kernel_number(n, cell) {
                Ok(f) => f.floor(),
                Err(error) => return error,
            };
            let k = match model.kernel_number(k, cell) {
                Ok(f) => f.floor(),
                Err(error) => return error,
            };
            Self::combin_of(n, k, cell)
        })
    }

    /// COMBIN on a floored `n` and `k` (see `fn_combin`).
    fn combin_of(n: f64, k: f64, cell: CellReferenceIndex) -> CalcResult {
        if n < 0.0 || k < 0.0 || k > n || n >= COMBIN_MAX_NUMBER {
            return num_error(cell);
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
        self.lift_call(Function::Combina, args, cell, &mut |model, values| {
            let [n, k] = values else {
                return CalcResult::new_args_number_error(cell);
            };
            // `number` truncates toward zero before its sign check, and number_chosen floors, so
            // one in (-1, 0) is #NUM! (`COMBINA(7,1-1.005)`, `COMBINA(24,-0.0001)`; gen-m5
            // c688ca172fe533dc A118, gen-m6 44194b2fefa30de2 A222), except with a number of 0,
            // where it truncates too: `COMBINA(-0.5,-0.0001)` is 1 (probes/t2475-stat-subexpressions
            // P!A35; statistics.md "Statistical argument-domain errors and extreme values (T-2621)").
            let n = match model.kernel_number(n, cell) {
                Ok(f) => f.trunc(),
                Err(error) => return error,
            };
            let k = match model.kernel_number(k, cell) {
                Ok(f) if n == 0.0 => f.trunc(),
                Ok(f) => f.floor(),
                Err(error) => return error,
            };
            Self::combina_of(n, k, cell)
        })
    }

    /// COMBINA on an integral `n` and `k` (see `fn_combina`).
    fn combina_of(n: f64, k: f64, cell: CellReferenceIndex) -> CalcResult {
        if n < 0.0 || k < 0.0 || (n == 0.0 && k > 0.0) {
            return num_error(cell);
        }
        // COMBINA(n, k) is Excel's COMBIN(n + k - 1, k), rounding included: `COMBINA(20,19)` is
        // 35345263799.999985 (numbers.md "COMBINA is COMBIN(n+k-1, k)"), and so is its bound:
        // `COMBINA(2^31-2,1)` is 2147483646, `COMBINA(2^31-2,2)` and `COMBINA(2^31-1,1)` are #NUM!.
        // That also caps `combin`'s loop below 2^31 steps (min(k, n - 1)), and it stops at the first
        // infinity, within about 1,030 (the lifting table's weight, `lift/kinds/general.rs`).
        if n + k - 1.0 >= COMBIN_MAX_NUMBER {
            return num_error(cell);
        }
        match combin(n + k - 1.0, k) {
            Ok(result) => CalcResult::Number(result),
            Err(error) => CalcResult::Error {
                error,
                origin: cell,
                message: "Result is too large".to_string(),
            },
        }
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
/// is `#NUM!`. Just below -1 `1 - f` rounds to 2 and the radicand to 0, so the domain is checked
/// first (`ASIN(-1-2^-52)` is `#NUM!`, numbers.md "Just below -1 as well", T-2170).
fn excel_asin(f: f64) -> f64 {
    if f.abs() > 1.0 {
        return f64::NAN;
    }
    let y = 1.0 - f;
    f64::atan(f / extended_sqrt(y * (2.0 - y)))
}
