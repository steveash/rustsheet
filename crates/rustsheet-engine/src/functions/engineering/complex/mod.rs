use std::fmt;

use crate::arithmetic::power;
use crate::cast::number_text_too_long;
use crate::functions::lift::{num_error, value_error};
use crate::functions::math_and_trigonometry::{
    excel_atan2, excel_quarter_turn_remainder, excel_sin_cos, EXCEL_INNER_TRIG_LIMIT,
    EXCEL_TRIG_LIMIT,
};
use crate::functions::util::aggregate_args::{
    AggregatePolicy, AggregateWalk, ArgReading, ErrorRule, GridReading, ItemSource, NestedReading,
};
use crate::limits::too_many_visits;
use crate::reference_operators::RefArg;
use crate::{
    calc_result::CalcResult,
    expressions::{
        lexer::util::get_tokens,
        parser::Node,
        token::{Error, OpSum, TokenType},
        types::CellReferenceIndex,
    },
    functions::Function,
    model::Model,
    number_format::{complex_part_to_text, to_precision},
};

mod reading;
mod trig;
use reading::{has_inner_minus, strip_currency_symbols};
use trig::{imcos, imcosh, imcot, imcsc, imcsch, imsec, imsech, imsin, imsinh, imtan};

/// This implements all functions with complex arguments in the standard
/// NOTE: If performance is ever needed we should have a new entry in `CalcResult`,
/// So this functions will return `CalcResult::Complex(x,y`, Suffix)
/// and not having to parse it over and over again.

#[derive(Clone, Copy, PartialEq, Debug)]
enum Suffix {
    I,
    J,
    /// A number with no imaginary part and no suffix ("3"): it takes the suffix of the number it
    /// meets, so `IMSUM("3+4j","3")` is "6+4j" (probes/eng-imsum-improduct-3d-j-empty E1-E10).
    Real,
}

impl Suffix {
    /// The suffix two operands share, or `None` when one is `i` and the other `j`.
    fn join(self, other: Suffix) -> Option<Suffix> {
        match (self, other) {
            (Suffix::Real, suffix) | (suffix, Suffix::Real) => Some(suffix),
            (a, b) if a == b => Some(a),
            _ => None,
        }
    }
}

impl fmt::Display for Suffix {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            // A `Real` only prints when it has an imaginary part, which an unsuffixed number never has.
            Suffix::I | Suffix::Real => write!(f, "i"),
            Suffix::J => write!(f, "j"),
        }
    }
}

struct Complex {
    x: f64,
    y: f64,
    suffix: Suffix,
}

impl fmt::Display for Complex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // 15-digit rounding of a part near DBL_MAX overflows to infinity; Excel writes
        // `1.79769313486232E+308` (probes/eng-imexp-overflow-imag A22), so keep the unrounded part.
        // A subnormal part is 0, as a subnormal number prints `0` (numbers.md "Subnormal results
        // become 0"): `IMEXP("-724.227-562.884i")` is "0", not "0+0i" (gen-m9/b5aef3645cec1b95
        // A154), and `IMSEC("1.5+711i")` "0".
        let round15 = |v: f64| {
            let v = if v.is_subnormal() { 0.0 } else { v };
            Some(to_precision(v, 15))
                .filter(|r| r.is_finite())
                .unwrap_or(v)
        };
        let x = round15(self.x);
        let y = round15(self.y);
        let suffix = &self.suffix;
        // Each part is written as Excel's number-to-text writes it: plain up to 20 characters,
        // then E notation (`1.22514845490862E-16`, probes/eng-impower A9; `1E+200`,
        // probes/eng-imdiv A19).
        // The coefficient 1 is dropped only for an imaginary part that is exactly ±1: Excel writes
        // `0.1+1i` for `COMPLEX(0.1,1+2^-52,"i")` (gen-m9/62c886e21a1042af A243), so compare the
        // unrounded part.
        let y_str = if self.y == 1.0 {
            String::new()
        } else if self.y == -1.0 {
            "-".to_string()
        } else {
            complex_part_to_text(y)
        };
        let x_str = complex_part_to_text(x);
        if y == 0.0 && x == 0.0 {
            write!(f, "0")
        } else if y == 0.0 {
            write!(f, "{x_str}")
        } else if x == 0.0 {
            write!(f, "{y_str}{suffix}")
        } else if y > 0.0 {
            write!(f, "{x_str}+{y_str}{suffix}")
        } else {
            write!(f, "{x_str}{y_str}{suffix}")
        }
    }
}

fn parse_complex_number(s: &str) -> Result<(f64, f64, Suffix), String> {
    // Check for i, j, -i, -j
    // One leading `+` is read like a leading `-`: `IMABS("+i")` 1, `IMABS("+3+4i")` 5, while a second
    // sign (`"++i"`, `"+-i"`) is `#NUM!` (probes/eng-leading-plus-oct-bool A1:A8, A26).
    let (sign, s) = match s.strip_prefix('-') {
        Some(r) => (-1.0, r),
        None => (1.0, s.strip_prefix('+').unwrap_or(s)),
    };
    if s.starts_with(['+', '-']) {
        return Err(format!("Not a complex number: {s}"));
    }
    match s {
        "i" => return Ok((0.0, sign * 1.0, Suffix::I)),
        "j" => return Ok((0.0, sign * 1.0, Suffix::J)),
        _ => {
            // Let it go
        }
    };

    // TODO: This is an overuse
    let tokens = get_tokens(s);

    // There has to be 1, 2 3, or 4 tokens
    // number
    // number suffix
    // number1+suffix
    // number1+number2 suffix

    match tokens.len() {
        1 => {
            // Real number
            let number1 = match tokens[0].token {
                TokenType::Number(f) => f,
                _ => return Err(format!("Not a complex number: {s}")),
            };
            Ok((sign * number1, 0.0, Suffix::Real))
        }
        2 => {
            // number2 i
            let number2 = match tokens[0].token {
                TokenType::Number(f) => f,
                _ => return Err(format!("Not a complex number: {s}")),
            };
            let suffix = match &tokens[1].token {
                TokenType::Ident(w) => match w.as_str() {
                    "i" => Suffix::I,
                    "j" => Suffix::J,
                    _ => return Err(format!("Not a complex number: {s}")),
                },
                _ => {
                    return Err(format!("Not a complex number: {s}"));
                }
            };
            Ok((0.0, sign * number2, suffix))
        }
        3 => {
            let number1 = match tokens[0].token {
                TokenType::Number(f) => f,
                _ => return Err(format!("Not a complex number: {s}")),
            };
            let operation = match &tokens[1].token {
                TokenType::Addition(f) => f,
                _ => return Err(format!("Not a complex number: {s}")),
            };
            let suffix = match &tokens[2].token {
                TokenType::Ident(w) => match w.as_str() {
                    "i" => Suffix::I,
                    "j" => Suffix::J,
                    _ => return Err(format!("Not a complex number: {s}")),
                },
                _ => {
                    return Err(format!("Not a complex number: {s}"));
                }
            };
            let number2 = if matches!(operation, OpSum::Minus) {
                -1.0
            } else {
                1.0
            };
            Ok((sign * number1, number2, suffix))
        }
        4 => {
            let number1 = match tokens[0].token {
                TokenType::Number(f) => f,
                _ => return Err(format!("Not a complex number: {s}")),
            };
            let operation = match &tokens[1].token {
                TokenType::Addition(f) => f,
                _ => return Err(format!("Not a complex number: {s}")),
            };
            let mut number2 = match tokens[2].token {
                TokenType::Number(f) => f,
                _ => return Err(format!("Not a complex number: {s}")),
            };
            let suffix = match &tokens[3].token {
                TokenType::Ident(w) => match w.as_str() {
                    "i" => Suffix::I,
                    "j" => Suffix::J,
                    _ => return Err(format!("Not a complex number: {s}")),
                },
                _ => {
                    return Err(format!("Not a complex number: {s}"));
                }
            };
            if matches!(operation, OpSum::Minus) {
                number2 = -number2;
            }
            Ok((sign * number1, number2, suffix))
        }
        _ => Err(format!("Not a complex number: {s}")),
    }
}

/// A parsed complex number: the real part, the imaginary part and the suffix.
type ComplexParts = (f64, f64, Suffix);

/// One complex function of a parsed number (IMABS … IMTAN).
type ComplexFn = fn(ComplexParts, CellReferenceIndex) -> CalcResult;

/// The text result of a complex function.
fn complex_result(x: f64, y: f64, suffix: Suffix) -> CalcResult {
    CalcResult::String(Complex { x, y, suffix }.to_string().into())
}

/// A result part that overflows or is NaN is `#NUM!`, not `inf` text: `IMCOSH("1000")`,
/// `IMSIN("1+1000i")` (probes/eng-complex-overflow A1-A4; engineering.md "IM* results that overflow").
fn finite_complex_result(x: f64, y: f64, suffix: Suffix, cell: CellReferenceIndex) -> CalcResult {
    if x.is_finite() && y.is_finite() {
        complex_result(x, y, suffix)
    } else {
        num_error(cell)
    }
}

/// How a complex argument reads a blank and empty text.
#[derive(Clone, Copy, PartialEq, Eq)]
enum EmptyText {
    /// IMSUM and IMPRODUCT: both are parsed (their walk handles a blank cell itself).
    Parse,
    /// Every other IM* argument: a blank cell is 0 (ironcalc-calc/ENGINEERING/COMPLEXs.xlsx
    /// Sheet2 G7:G10, probes/eng-gen-sample-edges D3, D26:D32), empty text, typed or from a
    /// formula, is `#NUM!` (probes/eng-imsub A12, A13, probes/eng-gen-sample-edges D1:D25).
    ZeroIfBlank,
}

/// How IMSUM and IMPRODUCT combine their numbers.
#[derive(Clone, Copy, PartialEq, Eq)]
enum ComplexFold {
    Sum,
    Product,
}

/// IMSUM's and IMPRODUCT's walk: the first bad item in argument order is the result, whether it
/// is an error cell, a boolean (`#VALUE!`) or non-complex text (`#NUM!`): `IMSUM(M5,M4)` is
/// `#NUM!`, `IMSUM(M4,M5)` `#VALUE!`, `IMSUM(M7,M5)` `#DIV/0!` (probes/eng-imsum-improduct-whole
/// B4:B9, B27). Blank cells are kept, so a cached blank of a linked workbook is a blank too.
const COMPLEX_FOLD_POLICY: AggregatePolicy = AggregatePolicy {
    reading: ArgReading::PickedKeepingBlanks,
    errors: ErrorRule::First,
    skip_empty_args: false,
    grids: GridReading::Values,
    nested: NestedReading::Value,
};

/// The cells of the rectangle `rows` x `columns`, 0 when either is not positive.
fn rectangle_cells(rows: i32, columns: i32) -> u64 {
    let rows = u64::try_from(rows).unwrap_or(0);
    let columns = u64::try_from(columns).unwrap_or(0);
    rows.saturating_mul(columns)
}

impl Model {
    /// The complex number of one element value: its text, parsed. Non-complex text is `#NUM!`,
    /// and a boolean, typed or in a cell, is `#VALUE!` (`IMABS(TRUE)`, `IMSUB(M4,"1")`:
    /// probes/eng-argument-modes A89, probes/eng-imsum-improduct-whole C1:C15).
    fn complex_of(
        &self,
        value: &CalcResult,
        empty: EmptyText,
        cell: CellReferenceIndex,
    ) -> Result<ComplexParts, CalcResult> {
        if matches!(value, CalcResult::Boolean(_)) {
            return Err(value_error(cell));
        }
        // A number keeps its full precision, not its 15-digit text: `IMPOWER(1.6564497455696935E-10,3)`
        // is 4.54500948137352E-30 (gen-m9/756a9f7bc3a06bcc A345).
        if let CalcResult::Number(f) = value {
            return Ok((*f, 0.0, Suffix::Real));
        }
        let blank = matches!(value, CalcResult::EmptyCell | CalcResult::EmptyArg);
        let value = Self::cast_string_ref(value, cell)?;
        if value.is_empty() && empty == EmptyText::ZeroIfBlank && blank {
            // Excel: a blank takes the suffix of the number it meets, as a suffixless number does
            // (`IMSUB("3+4j",Q1)` is "3+4j"): probes/eng-imsub-imdiv-blank-j A1:A6, B1, B5.
            return Ok((0.0, 0.0, Suffix::Real));
        }
        // Excel: text over 255 characters is `#VALUE!` (`IMSECH("TRUE"&REPT("AB",300))`, gen-m9
        // b5aef3645cec1b95 A101), and a leading tab or U+3000 is `#NUM!` where a leading space is
        // read (`IMCOT(CHAR(9)&"123456789.123")`, gen-m9/12e4aa13a20cc1f8 A224, 8fa8b572ebc1f001 A377).
        if number_text_too_long(&value) {
            return Err(value_error(cell));
        }
        // A tab, U+3000, NBSP, LF or CR at either end, past leading spaces, is `#NUM!`, where a space
        // is read: `IMABS` of tab&"5", " "&tab&"5", "5"&tab, "3+4i"&LF and NBSP&"3+4i" are `#NUM!`
        // (probes/eng-inumber-reading, probes/eng-inumber-currency-shapes A13:A26).
        const UNREAD_EDGE: [char; 5] = ['\t', '\u{3000}', '\u{a0}', '\n', '\r'];
        if value.trim_start_matches(' ').starts_with(UNREAD_EDGE) || value.ends_with(UNREAD_EDGE) {
            return Err(num_error(cell));
        }
        // Text VALUE reads is a real number (`IMPOWER("$4.00",1)` "4", `IMCONJUGATE("2:24:00")` "0.1",
        // `IMABS("1/2/2020")` 43832), except text with an inner minus sign that is not an exponent's:
        // `IMABS` of "2024-01-01", "1-2-2020", "1-Jan-2020" and "$-5" is `#NUM!` where VALUE reads them,
        // while " -5", "- 5", "1E-3%" and "1,000E-2" are read (probes/eng-inumber-reading A4,
        // probes/eng-inumber-date-boundary column A).
        match parse_complex_number(&strip_currency_symbols(&value)) {
            Ok(z) => Ok(z),
            Err(_) if has_inner_minus(&value) => Err(num_error(cell)),
            Err(_) => self
                .value_text_number(&value)
                .map(|f| (f, 0.0, Suffix::Real))
                .ok_or_else(|| num_error(cell)),
        }
    }

    /// A one-argument complex function, lifted: its argument lifts over an array and refuses a
    /// multi-cell range (`ArgKind::LiftArrays`, probes/eng-argument-modes B27:B51, E27:E51).
    fn lift_complex(
        &mut self,
        kind: Function,
        args: &[Node],
        op: ComplexFn,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        self.lift_call(kind, args, cell, &mut |model, values| {
            let [value] = values else {
                return CalcResult::new_args_number_error(cell);
            };
            match model.complex_of(value, EmptyText::ZeroIfBlank, cell) {
                Ok(z) => op(z, cell),
                Err(error) => error,
            }
        })
    }

    /// Two complex arguments of one element (IMDIV, IMSUB), read left to right; empty text and
    /// different suffixes are `#NUM!` (engineering.md "T-1625 probes"). `args` are the call's
    /// argument nodes: only an argument left out of the formula is `#N/A`, not the empty value
    /// IFERROR or IFNA hand on for a blank cell, which is 0 like the blank cell itself.
    fn complex_pair_of(
        &self,
        args: &[Node],
        values: &[CalcResult],
        cell: CellReferenceIndex,
    ) -> Result<(ComplexParts, ComplexParts), CalcResult> {
        let [first, second] = values else {
            return Err(CalcResult::new_args_number_error(cell));
        };
        // Excel: an empty argument, in either place, is `#N/A` (`IMSUB("3+4j",)`, `IMDIV(,"3+4j")`):
        // probes/eng-imsub-imdiv-blank-j A3, A4, B3, B4. A variable bound to an omitted LAMBDA
        // parameter or an empty LET value hands the omission on: `LAMBDA(a,b,IMSUB(a,b))("1",)`,
        // `LET(e,,IMSUB("1",e))` (probes/eng-imsub-imdiv-lambda-let-empty A1:A4, A12).
        if args
            .iter()
            .any(|arg| matches!(arg, Node::EmptyArgKind) || self.is_omitted_variable(arg))
        {
            return Err(CalcResult::new_error(Error::NA, cell, String::new()));
        }
        let first = self.complex_of(first, EmptyText::ZeroIfBlank, cell)?;
        let second = self.complex_of(second, EmptyText::ZeroIfBlank, cell)?;
        let Some(suffix) = first.2.join(second.2) else {
            return Err(num_error(cell));
        };
        Ok(((first.0, first.1, suffix), (second.0, second.1, suffix)))
    }

    /// `COMPLEX(real_num, i_num, [suffix])`: every argument lifts over an array and refuses a
    /// multi-cell range (probes/eng-argument-modes B13:E13, probes/scalar-lift-open-a Y11,
    /// probes/eng-later-argument-lift H16).
    pub(crate) fn fn_complex(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Complex, args, cell, &mut |model, values| {
            let (x, y, suffix) = match values {
                [x, y] => (x, y, None),
                [x, y, suffix] => (x, y, Some(suffix)),
                _ => return CalcResult::new_args_number_error(cell),
            };
            // Excel reads the suffix before the numbers, so a bad or error suffix wins over an
            // error in real_num or i_num (T-2532; engineering.md, gen-m9 COMPLEX cells).
            let suffix = match suffix.map(|suffix| Self::cast_string_ref(suffix, cell)) {
                None => Suffix::I,
                Some(Ok(s)) if s == "i" || s.is_empty() => Suffix::I,
                Some(Ok(s)) if s == "j" => Suffix::J,
                Some(Ok(_)) => return value_error(cell),
                Some(Err(s)) => return s,
            };
            // A boolean is `#VALUE!`: `COMPLEX(TRUE,1)` (probes/eng-imsum-improduct-whole C12).
            let x = match model.cast_number_no_bools_ref(x, cell) {
                Ok(s) => s,
                Err(s) => return s,
            };
            let y = match model.cast_number_no_bools_ref(y, cell) {
                Ok(s) => s,
                Err(s) => return s,
            };
            complex_result(x, y, suffix)
        })
    }

    pub(crate) fn fn_imabs(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_complex(Function::Imabs, args, imabs, cell)
    }

    pub(crate) fn fn_imaginary(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_complex(Function::Imaginary, args, imaginary, cell)
    }

    pub(crate) fn fn_imargument(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_complex(Function::Imargument, args, imargument, cell)
    }

    pub(crate) fn fn_imconjugate(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_complex(Function::Imconjugate, args, imconjugate, cell)
    }

    pub(crate) fn fn_imcos(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_complex(Function::Imcos, args, imcos, cell)
    }

    pub(crate) fn fn_imcosh(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_complex(Function::Imcosh, args, imcosh, cell)
    }

    pub(crate) fn fn_imcot(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_complex(Function::Imcot, args, imcot, cell)
    }

    pub(crate) fn fn_imcsc(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_complex(Function::Imcsc, args, imcsc, cell)
    }

    pub(crate) fn fn_imcsch(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_complex(Function::Imcsch, args, imcsch, cell)
    }

    /// `IMDIV(inumber1, inumber2)`: both lift over an array and refuse a multi-cell range
    /// (probes/eng-argument-modes B36:E36, probes/scalar-lift-open-a Y13).
    pub(crate) fn fn_imdiv(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Imdiv, args, cell, &mut |model, values| {
            let ((x1, y1, suffix), (x2, y2, _)) = match model.complex_pair_of(args, values, cell) {
                Ok(pair) => pair,
                Err(error) => return error,
            };
            if x2 == 0.0 && y2 == 0.0 {
                return num_error(cell);
            }
            // Smith's algorithm: `x2 * x2 + y2 * y2` underflows for a tiny divisor, but Excel
            // gives `IMDIV("1","1E-200")` = 1E+200 (probes/eng-imdiv A19, A24).
            let (re, im) = if x2.abs() >= y2.abs() {
                let r = y2 / x2;
                let d = x2 + y2 * r;
                ((x1 + y1 * r) / d, (y1 - x1 * r) / d)
            } else {
                let r = x2 / y2;
                let d = x2 * r + y2;
                ((x1 * r + y1) / d, (y1 * r - x1) / d)
            };
            finite_complex_result(re, im, suffix, cell)
        })
    }

    pub(crate) fn fn_imexp(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_complex(Function::Imexp, args, imexp, cell)
    }

    pub(crate) fn fn_imln(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let op: ComplexFn = |p, c| complex_log(p, c, imln_parts);
        self.lift_complex(Function::Imln, args, op, cell)
    }

    pub(crate) fn fn_imlog10(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let op: ComplexFn = |p, c| complex_log(p, c, imlog10_parts);
        self.lift_complex(Function::Imlog10, args, op, cell)
    }

    pub(crate) fn fn_imlog2(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let op: ComplexFn = |p, c| complex_log(p, c, imlog2_parts);
        self.lift_complex(Function::Imlog2, args, op, cell)
    }

    // IMPOWER(imnumber, power): both lift over an array and refuse a multi-cell range
    // (probes/eng-argument-modes B41:E41, probes/scalar-lift-open-a Y14).
    // If $(r, \theta)$ is the polar representation the formula is:
    //  $$ x = r^n*\cos(n\dot\theta), y = r^n*\csin(n\dot\theta) $
    pub(crate) fn fn_impower(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Impower, args, cell, &mut |model, values| {
            let [number, n] = values else {
                return CalcResult::new_args_number_error(cell);
            };
            let (x, y, suffix) = match model.complex_of(number, EmptyText::ZeroIfBlank, cell) {
                Ok(s) => s,
                Err(error) => return error,
            };
            let n = match model.cast_number_no_bools_ref(n, cell) {
                Ok(f) => f,
                Err(s) => return s,
            };
            if n == 0.0 && x == 0.0 && y == 0.0 {
                return num_error(cell); // probes/eng-impower A6
            }
            // A base with both parts nonzero whose x²+y² overflows is `#NUM!` even when the result
            // is tiny: `IMPOWER("1E+300+1E+300i",-0.5)`, `("1E+154+1E+154i",2)`; a pure real
            // `IMPOWER("1E+300",-0.5)` is 1E-150 (probes/eng-impower-overflow A1-A12; T-2539).
            if x != 0.0 && y != 0.0 && (x * x + y * y).is_infinite() {
                return num_error(cell);
            }
            let r = modulus(x, y);
            // Excel's `^` (square-and-multiply for an integer power) and its x87-reduced sine
            // and cosine: `IMPOWER("i",2)` = `-1+1.22514845490862E-16i` and
            // `IMPOWER("1+i",100)` = `-1125899906842640-0.551513671875006i`
            // (probes/eng-impower A9, A11, A13, A22; engineering.md "T-1625 probes").
            let Ok(radius) = power(r, n) else {
                return num_error(cell);
            };
            // The angle is Excel's ATAN2 kernel, `atan(y/x)` (IMARGUMENT's [`excel_atan2`]):
            // `IMPOWER("1000+1i",1)` is "1000+1i", its imaginary part 1.0000000000000002 where
            // libm `atan2` gives exactly 1 (gen-m9/74779cff2810ccf4 Calc!A379; T-2610). Unprobed
            // (T-2736): where that kernel errs (a zero or subnormal x, an overflowing y/x) `atan2` is
            // kept; its subnormal angle is 0 (`IMPOWER("1E20+1E-300i",1)` is "1E+20") and a -0
            // imaginary part with x < 0 is +π, where `atan2` gave a tiny part and -π.
            let angle = excel_atan2(x, y).unwrap_or_else(|_| f64::atan2(y, x)) * n;
            // An underflowed radius of 0 is "0" without reading the angle
            // (`IMPOWER("0.5i",134217728)`; probes/eng-impower-imdiv-edges A28-A35).
            if radius == 0.0 {
                return complex_result(0.0, 0.0, suffix);
            }
            if angle.abs() < EXCEL_INNER_TRIG_LIMIT {
                let (sin, cos) = excel_sin_cos(angle);
                return finite_complex_result(radius * cos, radius * sin, suffix, cell);
            }
            // Past the cut Excel's sine and cosine both read as the largest double, whatever the
            // quadrant, so each part is `radius·DBL_MAX`, and `#NUM!` once that is not below
            // DBL_MAX (a radius of 1 or more): `IMPOWER("0.99999973i",85445659)` is
            // `1.71948380972175E+298+1.71948380972175E+298i` (the radius 9.56E-11 of the 85445658th
            // power times 1.797E+308), every quadrant of `0.7071067811865476±0.7071067811865476i`
            // gives two equal positive parts, and `IMPOWER("i",85445659)`, `IMPOWER("-1",42722830)`
            // and `IMPOWER("1.0000000001i",85445659)` are `#NUM!` (probes/eng-impower-past-cut,
            // eng-impower-angle-cut A18-A19; engineering.md "IMPOWER past the angle cut").
            let part = radius * f64::MAX;
            if part < f64::MAX {
                complex_result(part, part, suffix)
            } else {
                num_error(cell)
            }
        })
    }

    /// `IMPRODUCT(inumber1, ...)`: the product of every number its arguments hold, read whole
    /// ([`Model::fold_complex`]).
    pub(crate) fn fn_improduct(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.fold_complex(Function::Improduct, args, ComplexFold::Product, cell)
    }

    pub(crate) fn fn_imreal(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_complex(Function::Imreal, args, imreal, cell)
    }

    pub(crate) fn fn_imsec(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_complex(Function::Imsec, args, imsec, cell)
    }

    pub(crate) fn fn_imsech(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_complex(Function::Imsech, args, imsech, cell)
    }

    pub(crate) fn fn_imsin(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_complex(Function::Imsin, args, imsin, cell)
    }

    pub(crate) fn fn_imsinh(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_complex(Function::Imsinh, args, imsinh, cell)
    }

    pub(crate) fn fn_imsqrt(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_complex(Function::Imsqrt, args, imsqrt, cell)
    }

    /// `IMSUB(inumber1, inumber2)`: both lift over an array and refuse a multi-cell range
    /// (probes/eng-argument-modes B49:E49, probes/scalar-lift-open-a Y12, Y15).
    pub(crate) fn fn_imsub(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(
            Function::Imsub,
            args,
            cell,
            &mut |model, values| match model.complex_pair_of(args, values, cell) {
                Ok(((x1, y1, suffix), (x2, y2, _))) => {
                    // Excel: a part that overflows is 0 and the other part stays
                    // (`IMSUB("9E307","-9E307")` is "0", `IMSUB("9E307+1i","-9E307")` is "i";
                    // probes/eng-complex-overflow A7, A13-A15, A21).
                    let finite_or_zero = |part: f64| if part.is_finite() { part } else { 0.0 };
                    complex_result(finite_or_zero(x1 - x2), finite_or_zero(y1 - y2), suffix)
                }
                Err(error) => error,
            },
        )
    }

    /// `IMSUM(inumber1, ...)`: the sum of every number its arguments hold, read whole
    /// ([`Model::fold_complex`]).
    pub(crate) fn fn_imsum(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.fold_complex(Function::Imsum, args, ComplexFold::Sum, cell)
    }

    /// IMSUM and IMPRODUCT: every argument is read whole, never lifted (`ArgKind::WholeByCaller`),
    /// so the call is one value (probes/eng-argument-modes A60:A83, B/C/E42, B/C/E50;
    /// docs/excel-semantics/engineering.md "IMSUM and IMPRODUCT take their arguments whole"):
    /// - a range, a reference a function returns and an array give each of their cells, in
    ///   row-major order; a union or a 3-D reference is `#VALUE!` (`IMSUM((M1,M6))`,
    ///   probes/eng-imsum-improduct-whole B19);
    /// - a blank cell is 0, so IMPRODUCT over a range with a blank cell, also one past the used
    ///   area, is "0" (A73, B11); an empty argument is skipped (`IMPRODUCT(M1,)` is "1+2i", B18);
    /// - empty text is `#NUM!` (B13:B16, B25), and the first bad item in order is the result
    ///   ([`COMPLEX_FOLD_POLICY`]); different suffixes are `#VALUE!` (B10).
    ///
    /// The cells read are capped (directive 5): the cells a range walks (a whole column or row
    /// clipped to the used area) and the elements of an array add up, and the call is `#ERROR!`
    /// once they pass `MAX_LIFTED_CELL_VISITS`, before the range is walked. Excel has no such
    /// limit: DEVIATION (engineering.md).
    fn fold_complex(
        &mut self,
        kind: Function,
        args: &[Node],
        fold: ComplexFold,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        if !kind
            .arg_kinds()
            .is_some_and(|kinds| kinds.accepts(args.len()))
        {
            return CalcResult::new_args_number_error(cell);
        }
        // Excel: an empty first argument is `#N/A` (`IMSUM(,)`, `IMSUM(,Q1)`, `IMPRODUCT(,M8)`), where
        // blank cells are "0" and a later empty argument is skipped (`IMSUM(M8,,M8)`):
        // probes/eng-imsum-improduct-3d-j-empty F1-F4, probes/eng-imsub-imdiv-blank-j C1-C7, D1-D6.
        if matches!(args.first(), Some(Node::EmptyArgKind)) {
            return CalcResult::new_error(Error::NA, cell, String::new());
        }
        let mut total: Option<ComplexParts> = None;
        let mut has_blank = false;
        let mut visits: u64 = 0;
        let mut walk = AggregateWalk::new(COMPLEX_FOLD_POLICY);
        for arg in args {
            let read = self.evaluate_picked_reference_arg(arg, cell);
            // An empty array's `#CALC!` is `#VALUE!` (probes/empty-array-args-sequence A5:A7).
            if let RefArg::Value(value) = &read {
                if value.is_empty_array() {
                    // An earlier referenced error still wins (`IMSUM(<#DIV/0! cell>,DROP({1},1))`
                    // is `#DIV/0!`, a later one does not: probes/empty-array-args-sequence A170:A172).
                    return walk
                        .finish()
                        .err()
                        .unwrap_or_else(|| value.clone())
                        .empty_array_as_value_error(cell);
                }
            }
            // The cells of the range (blank cells included), and the cells its walk visits.
            let (cells, walked) = match &read {
                RefArg::Ranges(ranges) => match &ranges[..] {
                    [range] if range.left.sheet == range.right.sheet => {
                        let (left, right) = (range.left, range.right);
                        let corner = self.used_area_corner(
                            left.sheet,
                            left.row,
                            left.column,
                            right.row,
                            right.column,
                        );
                        let (row2, column2) = match corner {
                            Ok(corner) => corner,
                            Err(message) => {
                                return CalcResult::new_error(Error::ERROR, cell, message)
                            }
                        };
                        let cells = rectangle_cells(
                            right.row - left.row + 1,
                            right.column - left.column + 1,
                        );
                        let walked =
                            rectangle_cells(row2 - left.row + 1, column2 - left.column + 1);
                        (cells, walked)
                    }
                    _ => return value_error(cell),
                },
                RefArg::Value(CalcResult::Array(array)) => {
                    let cells: usize = array.iter().map(Vec::len).sum();
                    (0, u64::try_from(cells).unwrap_or(u64::MAX))
                }
                RefArg::Grid(grid) => {
                    let (rows, columns) = grid.dims();
                    (
                        0,
                        u64::try_from(rows.saturating_mul(columns)).unwrap_or(u64::MAX),
                    )
                }
                RefArg::Value(_) => (0, 1),
            };
            visits = visits.saturating_add(walked);
            if let Some(error) = too_many_visits(visits, cell) {
                return error;
            }
            let is_reference = matches!(*arg.ungrouped_at(cell.row), Node::ReferenceKind { .. });
            // The cells of a range the walk hands on: the others are blank.
            let mut referenced: u64 = 0;
            let walked = self.walk_aggregate_read(
                read,
                is_reference,
                cell,
                &mut walk,
                &mut |model, source, value| {
                    if source == ItemSource::ReferencedCell {
                        referenced += 1;
                    }
                    let z = match value {
                        CalcResult::EmptyArg => return Ok(()),
                        CalcResult::EmptyCell => {
                            has_blank = true;
                            return Ok(());
                        }
                        value => model.complex_of(&value, EmptyText::Parse, cell)?,
                    };
                    let next = match total {
                        None => z,
                        Some((x, y, total_suffix)) => {
                            let Some(suffix) = total_suffix.join(z.2) else {
                                return Err(value_error(cell));
                            };
                            match fold {
                                ComplexFold::Sum => (x + z.0, y + z.1, suffix),
                                // A blank cell met earlier makes the running product 0, which
                                // cannot overflow later: `IMPRODUCT(<blank>,"1E200","1E200")`
                                // is "0", the blank last is `#VALUE!` (probes/eng-im-overflow-edges
                                // A2, A3).
                                ComplexFold::Product if has_blank => (0.0, 0.0, suffix),
                                ComplexFold::Product => {
                                    (x * z.0 - y * z.1, x * z.1 + y * z.0, suffix)
                                }
                            }
                        }
                    };
                    // An overflow is `#VALUE!` at once: it beats a later bad argument, a later
                    // blank and a later cancelling term (`IMSUM("9E307","9E307","-9E307")`,
                    // `IMSUM("9E307","9E307",1,"x")`, `IMPRODUCT("1E200","1E200",<blank>)`;
                    // probes/eng-complex-overflow A5, A8-A10).
                    if !next.0.is_finite() || !next.1.is_finite() {
                        return Err(value_error(cell));
                    }
                    total = Some(next);
                    Ok(())
                },
            );
            if let Err(error) = walked {
                return error.empty_array_as_value_error(cell);
            }
            if referenced < cells {
                has_blank = true;
            }
        }
        // A referenced cell holding an empty array's `#CALC!` is `#VALUE!` too (A140..A147).
        if let Err(error) = walk.finish() {
            return error.empty_array_as_value_error(cell);
        }
        let (x, y, suffix) = total.unwrap_or((0.0, 0.0, Suffix::I));
        if fold == ComplexFold::Product && has_blank {
            return complex_result(0.0, 0.0, suffix);
        }
        // An overflowed (or NaN) sum or product is `#VALUE!`, unlike IMEXP, IMPOWER and IMDIV
        // (`#NUM!`): probes/eng-im-overflow A5-A7, A25, A30, A36-A38 (engineering.md).
        if !x.is_finite() || !y.is_finite() {
            return value_error(cell);
        }
        complex_result(x, y, suffix)
    }

    pub(crate) fn fn_imtan(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_complex(Function::Imtan, args, imtan, cell)
    }
}

// The one-argument complex functions, of a parsed number.

/// IMABS scales by the larger part, `a·sqrt(1 + (b/a)²)` with a = max(|x|, |y|): it matches 77 of
/// the 78 IMABS cells of probes-numeric im-last-digit-sweep bit for bit (`hypot` 59, `sqrt(x²+y²)`
/// 55; the miss is B33, `956.615+0.00387493i`, 2 ulps; T-2044). IMPOWER's modulus is [`modulus`].
fn imabs((x, y, _): ComplexParts, _: CellReferenceIndex) -> CalcResult {
    CalcResult::Number(scaled_modulus(x, y))
}

/// IMABS's modulus, `a·sqrt(1 + (b/a)²)` with a = max(|x|, |y|) (IMLOG2 takes it too).
fn scaled_modulus(x: f64, y: f64) -> f64 {
    let (x, y) = (x.abs(), y.abs());
    let (large, small) = if x >= y { (x, y) } else { (y, x) };
    if large == 0.0 {
        return 0.0;
    }
    let ratio = small / large;
    large * (1.0 + ratio * ratio).sqrt()
}

/// `|x+yi|` as IMPOWER takes it: it matches `hypot`, not `sqrt(x²+y²)`. They differ in the last
/// bit: `IMABS("0.7071067811865476+0.7071067811865476i")` (read as the 15-digit 0.707106781186547)
/// is 0.9999999999999993 (1-6·2^-53), where `sqrt` gives 1-7·2^-53, and IMPOWER's 170891300th
/// power of that number confirms the per-factor value (probes/eng-impower-past-cut A15,
/// eng-impower-angle-cut A9-A17; T-1950).
fn modulus(x: f64, y: f64) -> f64 {
    x.hypot(y)
}

fn imaginary((_, y, _): ComplexParts, _: CellReferenceIndex) -> CalcResult {
    CalcResult::Number(y)
}

/// IMARGUMENT is Excel's ATAN2 kernel, `atan(y/x)` plus or minus π: all 77 numeric cells of
/// probes-numeric im-last-digit-sweep (libm `atan2` 67; T-2044). "0" is `#DIV/0!` (C21).
fn imargument((x, y, _): ComplexParts, cell: CellReferenceIndex) -> CalcResult {
    match excel_atan2(x, y) {
        Ok(angle) => CalcResult::Number(angle),
        Err(error) => CalcResult::new_error(error, cell, String::new()),
    }
}

fn imconjugate((x, y, suffix): ComplexParts, _: CellReferenceIndex) -> CalcResult {
    complex_result(x, -y, suffix)
}

fn imexp((x, y, suffix): ComplexParts, cell: CellReferenceIndex) -> CalcResult {
    // Overflow of a pure real argument is `#NUM!` (probes/eng-im-overflow A9, A10;
    // `IMEXP("709.7")` is still finite). With a nonzero imaginary part Excel caps the modulus at
    // DBL_MAX instead (A12: `IMEXP("1000+1i")` is `9.71297746009434E+307+1.51270661257499E+308i`,
    // DBL_MAX times cos 1 and sin 1), unless y is within `imexp_cap_window` of a multiple of
    // pi/2, where the smaller part is treated as zero and the overflow is `#NUM!`
    // (probes/eng-imexp-overflow-cutoff*; engineering.md). IMEXP is `#NUM!` for |y| >= 2^27
    // whatever x is (A16-A24 of eng-imexp-overflow-cutoff).
    // The sine and cosine reduce y as SIN and COS do, which is what gives A3's last digits
    // (`...+4.770340154251E+302i`).
    if y.abs() >= EXCEL_TRIG_LIMIT {
        return num_error(cell);
    }
    let (sin_y, cos_y) = excel_sin_cos(y);
    let modulus = x.exp();
    if modulus == f64::INFINITY && y.is_finite() {
        let (n, remainder) = excel_quarter_turn_remainder(y);
        if remainder > imexp_cap_window(n) {
            return complex_result(f64::MAX * cos_y, f64::MAX * sin_y, suffix);
        }
    }
    let (re, im) = (modulus * cos_y, modulus * sin_y);
    finite_complex_result(re, im, suffix, cell)
}

/// Half-width of the window around a multiple of pi/2 inside which an overflowing IMEXP is
/// `#NUM!` rather than capped (the cap needs a remainder strictly past it). Around 0 it is exactly
/// 2^-26: `1000+1.4901161193847656E-8i` is `#NUM!`, `...848E-8i` is capped. Around every other
/// multiple (pi/2, pi, 2pi: (1.053, 1.054]E-8; 100pi: (1.05, 1.06]E-8; either side) it is 2^-26.5 =
/// 2^-26/sqrt(2) = 1.0537E-8 (probes/eng-imexp-cap-window; engineering.md).
fn imexp_cap_window(quarter_turns: i64) -> f64 {
    let base = f64::EPSILON.sqrt();
    if quarter_turns == 0 {
        base
    } else {
        base * std::f64::consts::FRAC_1_SQRT_2
    }
}

/// A part's square as IMSQRT and the logs take it: a subnormal square is 0. With it,
/// `IMLN("1E-154+1E-153i")` is ln 1E-153, -352.295519228089 (`hypot` gives -352.290544062662), and
/// `IMSQRT("1E-154+1E-153i")` is 2.34467878292635E-77+2.12190513569646E-77i, the root of
/// 1E-153 at half the unflushed angle (probes/eng-imln-modulus-range A79, A80; T-2572).
fn flushed_square(part: f64) -> f64 {
    let square = part * part;
    if square.is_subnormal() {
        0.0
    } else {
        square
    }
}

/// IMLN, IMLOG10 and IMLOG2: the real and imaginary parts that `parts` makes of the parts the
/// modulus is taken of and the argument `atan2(y, x)` (eng-imln-modulus-range, T-2117, T-2572):
/// - with both parts nonzero a part whose square is subnormal drops out of the modulus
///   ([`flushed_square`]), so the log is `#NUM!` when both squares flush to 0 or their sum
///   overflows (`IMLN("1E-154+1E-154i")`, `IMLN("1E154+1E154i")`);
/// - a pure real or imaginary number keeps its part (`IMLN("1E-160+0i")` is -368.413614879047).
///
/// The log of 0 is `#NUM!`, not "-inf" (T-2389).
fn complex_log(
    p: ComplexParts,
    c: CellReferenceIndex,
    parts: fn(f64, f64, f64) -> (f64, f64),
) -> CalcResult {
    let (x, y, suffix) = p;
    let (mut mx, mut my) = (x, y);
    if x != 0.0 && y != 0.0 {
        let (xx, yy) = (flushed_square(x), flushed_square(y));
        if !(xx + yy).is_normal() {
            return num_error(c);
        }
        if xx == 0.0 {
            mx = 0.0;
        }
        if yy == 0.0 {
            my = 0.0;
        }
    }
    let (re, im) = parts(mx, my, f64::atan2(y, x));
    finite_complex_result(re, im, suffix, c)
}

/// IMLN is `ln` of the [`modulus`] and the argument.
fn imln_parts(x: f64, y: f64, angle: f64) -> (f64, f64) {
    (modulus(x, y).ln(), angle)
}

/// IMLOG10 divides the argument by ln 10: multiplying by log10 e misses
/// `IMLOG10("711+1.5i")`'s 0.000916231724847909 (probes/eng-im-overflow-edges A95) and the
/// reciprocal `1/ln 10` misses `IMLOG10("1.5+710.4i")`; the division fits all 36 IMLOG10 cells
/// of the eval suites (T-2231).
fn imlog10_parts(x: f64, y: f64, angle: f64) -> (f64, f64) {
    (modulus(x, y).log10(), angle / std::f64::consts::LN_10)
}

/// IMLOG2 is `ln` of IMABS's [`scaled_modulus`] and the argument, both times the double one ulp
/// above log2 e (`1/ln 2` rounded from the double below ln 2): it fits all 103 distinct real and
/// 103 imaginary parts of IMLOG2 in the eval suites, `IMLOG2("710.4+0i")` 9.47248777146275 and
/// `IMLOG2("1E-20")` -66.4385618977473 among them, where `log2` and `atan2·log2 e` print the digit
/// below (probes/eng-im-overflow-edges A104, A107, A109; probes-numeric im-last-digit-sweep N, O;
/// T-2231). `hypot` instead of the scaled modulus misses `-0.66925-0.233532i` (sweep row 53).
fn imlog2_parts(x: f64, y: f64, angle: f64) -> (f64, f64) {
    let k = std::f64::consts::LOG2_E.next_up();
    (scaled_modulus(x, y).ln() * k, angle * k)
}

fn imreal((x, _, _): ComplexParts, _: CellReferenceIndex) -> CalcResult {
    CalcResult::Number(x)
}

/// Excel squares each part first: a part past `sqrt(DBL_MAX)` (1.34078E154) is `#NUM!`
/// (`IMSQRT("1.35E154")`, `IMSQRT("1E307")`), but when only the sum of the squares overflows
/// (`IMSQRT("1E154+1E154i")`) the result is `0`, not an error (probes/eng-imsqrt-large,
/// eng-imsqrt-large-b; docs/excel-semantics/engineering.md, T-1996). Not `modulus`: a subnormal
/// square is 0 ([`flushed_square`]), so `IMSQRT("1E-154+1E-154i")` and `IMSQRT("1E-160+0i")` are
/// "0" (probes/eng-imln-modulus A18, A20; eng-imln-modulus-range A51-A56, A80; T-2572).
/// The half angle is Excel's ATAN2 kernel (IMARGUMENT's [`excel_atan2`]) through Excel's SIN and
/// COS: `IMSQRT("-1")` is "6.1257422745431E-17+i", COS(ATAN2(-1,0)/2), where libm's cos(π/2) gives
/// 6.12323399573677E-17 (gen-m9 59a16b5ab566ee5c A6, 516488da78a6b08c A52; excel-graph, T-2536).
/// Decision 0188 R6 follows the web over the Mac 16.0300 cache of ironcalc-calc COMPLEXs.xlsx
/// AL12 and AL26, which has libm's digits (T-1075 item 16). Every other IMSQRT cell of the suites
/// prints the same either way. Where that kernel errs (x zero, subnormal or y/x overflowing) libm
/// `atan2` stands in, and a -0 imaginary part with x < 0 takes +π, as in IMPOWER (unprobed, T-2736).
fn imsqrt((x, y, suffix): ComplexParts, cell: CellReferenceIndex) -> CalcResult {
    let (xx, yy) = (flushed_square(x), flushed_square(y));
    if !xx.is_finite() || !yy.is_finite() {
        return num_error(cell);
    }
    if !(xx + yy).is_finite() {
        return complex_result(0.0, 0.0, suffix);
    }
    let r = f64::sqrt(xx + yy).sqrt();
    let a = excel_atan2(x, y).unwrap_or_else(|_| f64::atan2(y, x));
    let (sin, cos) = excel_sin_cos(a / 2.0);
    complex_result(r * cos, r * sin, suffix)
}

#[cfg(test)]
mod tests;
