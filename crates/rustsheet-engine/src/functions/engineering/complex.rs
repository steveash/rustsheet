use std::fmt;

use crate::functions::lift::{num_error, value_error};
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
    number_format::to_precision,
};

/// This implements all functions with complex arguments in the standard
/// NOTE: If performance is ever needed we should have a new entry in `CalcResult`,
/// So this functions will return `CalcResult::Complex(x,y`, Suffix)
/// and not having to parse it over and over again.

#[derive(PartialEq, Debug)]
enum Suffix {
    I,
    J,
}

impl fmt::Display for Suffix {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Suffix::I => write!(f, "i"),
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
        let x = to_precision(self.x, 15);
        let y = to_precision(self.y, 15);
        let suffix = &self.suffix;
        // it is a bit weird what Excel does but it seems it uses general notation for
        // numbers > 1e-20 and scientific notation for the rest
        let y_str = if y.abs() <= 9e-20 {
            format!("{y:E}")
        } else if y == 1.0 {
            "".to_string()
        } else if y == -1.0 {
            "-".to_string()
        } else {
            format!("{y}")
        };
        let x_str = if x.abs() <= 9e-20 {
            format!("{x:E}")
        } else {
            format!("{x}")
        };
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
    let (sign, s) = match s.strip_prefix('-') {
        Some(r) => (-1.0, r),
        None => (1.0, s),
    };
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
            // i is the default
            Ok((sign * number1, 0.0, Suffix::I))
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

/// How a complex argument reads empty text: IMCOS, IMCOSH, IMCOT, IMCSC and IMCSCH parse it
/// (`#NUM!`), the other functions read it as 0. `Zero` is our older behavior, not Excel's rule:
/// Excel gives `#NUM!` for empty text in IMSUB, IMDIV and IMPOWER (engineering.md "T-1625
/// probes") and probably IMSQRT (T-1385).
#[derive(Clone, Copy, PartialEq, Eq)]
enum EmptyText {
    Zero,
    Parse,
}

impl Model {
    fn get_complex_number(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> Result<ComplexParts, CalcResult> {
        let value = self.evaluate_node_in_context(node, cell);
        let value = self.dereference(value, cell);
        Self::complex_of(&value, EmptyText::Zero, cell)
    }

    /// The complex number of one element value: its text, parsed. Non-complex text is `#NUM!`.
    fn complex_of(
        value: &CalcResult,
        empty: EmptyText,
        cell: CellReferenceIndex,
    ) -> Result<ComplexParts, CalcResult> {
        let value = Self::cast_string_ref(value, cell)?;
        if value.is_empty() && empty == EmptyText::Zero {
            return Ok((0.0, 0.0, Suffix::I));
        }
        parse_complex_number(&value).map_err(|_| num_error(cell))
    }

    /// A one-argument complex function, lifted: its argument lifts over an array and refuses a
    /// multi-cell range (`ArgKind::LiftArrays`, probes/eng-argument-modes B27:B51, E27:E51).
    fn lift_complex(
        &mut self,
        kind: Function,
        args: &[Node],
        empty: EmptyText,
        op: ComplexFn,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        self.lift_call(kind, args, cell, &mut |_, values| {
            let [value] = values else {
                return CalcResult::new_args_number_error(cell);
            };
            match Self::complex_of(value, empty, cell) {
                Ok(z) => op(z, cell),
                Err(error) => error,
            }
        })
    }

    /// Two complex arguments of one element (IMDIV, IMSUB), read left to right; different
    /// suffixes are `#VALUE!`.
    fn complex_pair_of(
        values: &[CalcResult],
        cell: CellReferenceIndex,
    ) -> Result<(ComplexParts, ComplexParts), CalcResult> {
        let [first, second] = values else {
            return Err(CalcResult::new_args_number_error(cell));
        };
        let first = Self::complex_of(first, EmptyText::Zero, cell)?;
        let second = Self::complex_of(second, EmptyText::Zero, cell)?;
        if first.2 != second.2 {
            return Err(value_error(cell));
        }
        Ok((first, second))
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
            let x = match model.cast_number_ref(x, cell) {
                Ok(s) => s,
                Err(s) => return s,
            };
            let y = match model.cast_number_ref(y, cell) {
                Ok(s) => s,
                Err(s) => return s,
            };
            let suffix = match suffix.map(|suffix| Self::cast_string_ref(suffix, cell)) {
                None => Suffix::I,
                Some(Ok(s)) if s == "i" || s.is_empty() => Suffix::I,
                Some(Ok(s)) if s == "j" => Suffix::J,
                Some(Ok(_)) => return value_error(cell),
                Some(Err(s)) => return s,
            };
            complex_result(x, y, suffix)
        })
    }

    pub(crate) fn fn_imabs(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_complex(Function::Imabs, args, EmptyText::Zero, imabs, cell)
    }

    pub(crate) fn fn_imaginary(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_complex(Function::Imaginary, args, EmptyText::Zero, imaginary, cell)
    }

    pub(crate) fn fn_imargument(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_complex(
            Function::Imargument,
            args,
            EmptyText::Zero,
            imargument,
            cell,
        )
    }

    pub(crate) fn fn_imconjugate(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_complex(
            Function::Imconjugate,
            args,
            EmptyText::Zero,
            imconjugate,
            cell,
        )
    }

    pub(crate) fn fn_imcos(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_complex(Function::Imcos, args, EmptyText::Parse, imcos, cell)
    }

    pub(crate) fn fn_imcosh(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_complex(Function::Imcosh, args, EmptyText::Parse, imcosh, cell)
    }

    pub(crate) fn fn_imcot(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_complex(Function::Imcot, args, EmptyText::Parse, imcot, cell)
    }

    pub(crate) fn fn_imcsc(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_complex(Function::Imcsc, args, EmptyText::Parse, imcsc, cell)
    }

    pub(crate) fn fn_imcsch(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_complex(Function::Imcsch, args, EmptyText::Parse, imcsch, cell)
    }

    /// `IMDIV(inumber1, inumber2)`: both lift over an array and refuse a multi-cell range
    /// (probes/eng-argument-modes B36:E36, probes/scalar-lift-open-a Y13).
    pub(crate) fn fn_imdiv(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Imdiv, args, cell, &mut |_, values| {
            let ((x1, y1, suffix), (x2, y2, _)) = match Self::complex_pair_of(values, cell) {
                Ok(pair) => pair,
                Err(error) => return error,
            };
            let t = x2 * x2 + y2 * y2;
            if t == 0.0 {
                return num_error(cell);
            }
            complex_result((x1 * x2 + y1 * y2) / t, (-x1 * y2 + y1 * x2) / t, suffix)
        })
    }

    pub(crate) fn fn_imexp(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_complex(Function::Imexp, args, EmptyText::Zero, imexp, cell)
    }

    pub(crate) fn fn_imln(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_complex(Function::Imln, args, EmptyText::Zero, imln, cell)
    }

    pub(crate) fn fn_imlog10(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_complex(Function::Imlog10, args, EmptyText::Zero, imlog10, cell)
    }

    pub(crate) fn fn_imlog2(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_complex(Function::Imlog2, args, EmptyText::Zero, imlog2, cell)
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
            let (x, y, suffix) = match Self::complex_of(number, EmptyText::Zero, cell) {
                Ok(s) => s,
                Err(error) => return error,
            };
            let n = match model.cast_number_no_bools_ref(n, cell) {
                Ok(f) => f,
                Err(s) => return s,
            };
            let r = f64::sqrt(x * x + y * y);
            let a = f64::atan2(y, x);

            let x = r.powf(n) * f64::cos(a * n);
            let y = r.powf(n) * f64::sin(a * n);

            if x.is_infinite() || y.is_infinite() || x.is_nan() || y.is_nan() {
                return num_error(cell);
            }
            complex_result(x, y, suffix)
        })
    }

    pub(crate) fn fn_improduct(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }

        let (x1, y1, suffix) = match self.get_complex_number(&args[0], cell) {
            Ok(s) => s,
            Err(error) => return error,
        };

        let (x2, y2, suffix2) = match self.get_complex_number(&args[1], cell) {
            Ok(s) => s,
            Err(error) => return error,
        };

        if suffix != suffix2 {
            return CalcResult::new_error(Error::VALUE, cell, "Different suffixes".to_string());
        }
        let complex = Complex {
            x: x1 * x2 - y1 * y2,
            y: x1 * y2 + y1 * x2,
            suffix,
        };
        CalcResult::String(complex.to_string().into())
    }

    pub(crate) fn fn_imreal(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_complex(Function::Imreal, args, EmptyText::Zero, imreal, cell)
    }

    pub(crate) fn fn_imsec(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_complex(Function::Imsec, args, EmptyText::Zero, imsec, cell)
    }

    pub(crate) fn fn_imsech(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_complex(Function::Imsech, args, EmptyText::Zero, imsech, cell)
    }

    pub(crate) fn fn_imsin(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_complex(Function::Imsin, args, EmptyText::Zero, imsin, cell)
    }

    pub(crate) fn fn_imsinh(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_complex(Function::Imsinh, args, EmptyText::Zero, imsinh, cell)
    }

    pub(crate) fn fn_imsqrt(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_complex(Function::Imsqrt, args, EmptyText::Zero, imsqrt, cell)
    }

    /// `IMSUB(inumber1, inumber2)`: both lift over an array and refuse a multi-cell range
    /// (probes/eng-argument-modes B49:E49, probes/scalar-lift-open-a Y12, Y15).
    pub(crate) fn fn_imsub(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(
            Function::Imsub,
            args,
            cell,
            &mut |_, values| match Self::complex_pair_of(values, cell) {
                Ok(((x1, y1, suffix), (x2, y2, _))) => complex_result(x1 - x2, y1 - y2, suffix),
                Err(error) => error,
            },
        )
    }

    pub(crate) fn fn_imsum(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let (x1, y1, suffix) = match self.get_complex_number(&args[0], cell) {
            Ok(s) => s,
            Err(error) => return error,
        };
        let (x2, y2, suffix2) = match self.get_complex_number(&args[1], cell) {
            Ok(s) => s,
            Err(error) => return error,
        };
        if suffix != suffix2 {
            return CalcResult::new_error(Error::VALUE, cell, "Different suffixes".to_string());
        }
        let complex = Complex {
            x: x1 + x2,
            y: y1 + y2,
            suffix,
        };
        CalcResult::String(complex.to_string().into())
    }

    pub(crate) fn fn_imtan(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_complex(Function::Imtan, args, EmptyText::Zero, imtan, cell)
    }
}

// The one-argument complex functions, of a parsed number.

fn imabs((x, y, _): ComplexParts, _: CellReferenceIndex) -> CalcResult {
    CalcResult::Number(f64::sqrt(x * x + y * y))
}

fn imaginary((_, y, _): ComplexParts, _: CellReferenceIndex) -> CalcResult {
    CalcResult::Number(y)
}

fn imargument((x, y, _): ComplexParts, cell: CellReferenceIndex) -> CalcResult {
    if x == 0.0 && y == 0.0 {
        return CalcResult::new_error(Error::DIV, cell, String::new());
    }
    CalcResult::Number(f64::atan2(y, x))
}

fn imconjugate((x, y, suffix): ComplexParts, _: CellReferenceIndex) -> CalcResult {
    complex_result(x, -y, suffix)
}

fn imcos((x, y, suffix): ComplexParts, _: CellReferenceIndex) -> CalcResult {
    complex_result(x.cos() * y.cosh(), -x.sin() * y.sinh(), suffix)
}

fn imcosh((x, y, suffix): ComplexParts, _: CellReferenceIndex) -> CalcResult {
    complex_result(x.cosh() * y.cos(), x.sinh() * y.sin(), suffix)
}

fn imcot((x, y, suffix): ComplexParts, cell: CellReferenceIndex) -> CalcResult {
    if x == 0.0 && y != 0.0 {
        return complex_result(0.0, -1.0 / y.tanh(), suffix);
    } else if y == 0.0 {
        return complex_result(1.0 / x.tan(), 0.0, suffix);
    }

    let x_cot = 1.0 / x.tan();
    let y_coth = 1.0 / y.tanh();

    let t = x_cot * x_cot + y_coth * y_coth;
    let x = (x_cot * y_coth * y_coth - x_cot) / t;
    let y = (-x_cot * x_cot * y_coth - y_coth) / t;

    if x.is_infinite() || y.is_infinite() || x.is_nan() || y.is_nan() {
        return num_error(cell);
    }
    complex_result(x, y, suffix)
}

fn imcsc((x, y, suffix): ComplexParts, _: CellReferenceIndex) -> CalcResult {
    let x_cos = x.cos();
    let x_sin = x.sin();

    let y_cosh = y.cosh();
    let y_sinh = y.sinh();

    let t = x_sin * x_sin * y_cosh * y_cosh + x_cos * x_cos * y_sinh * y_sinh;
    complex_result(x_sin * y_cosh / t, -x_cos * y_sinh / t, suffix)
}

fn imcsch((x, y, suffix): ComplexParts, _: CellReferenceIndex) -> CalcResult {
    let x_cosh = x.cosh();
    let x_sinh = x.sinh();

    let y_cos = y.cos();
    let y_sin = y.sin();

    let t = x_sinh * x_sinh * y_cos * y_cos + x_cosh * x_cosh * y_sin * y_sin;
    complex_result(x_sinh * y_cos / t, -x_cosh * y_sin / t, suffix)
}

fn imexp((x, y, suffix): ComplexParts, _: CellReferenceIndex) -> CalcResult {
    complex_result(x.exp() * y.cos(), x.exp() * y.sin(), suffix)
}

fn imln((x, y, suffix): ComplexParts, _: CellReferenceIndex) -> CalcResult {
    let r = f64::sqrt(x * x + y * y);
    let a = f64::atan2(y, x);
    complex_result(r.ln(), a, suffix)
}

fn imlog10((x, y, suffix): ComplexParts, _: CellReferenceIndex) -> CalcResult {
    let r = f64::sqrt(x * x + y * y);
    let a = f64::atan2(y, x);
    complex_result(r.log10(), a * f64::log10(f64::exp(1.0)), suffix)
}

fn imlog2((x, y, suffix): ComplexParts, _: CellReferenceIndex) -> CalcResult {
    let r = f64::sqrt(x * x + y * y);
    let a = f64::atan2(y, x);
    complex_result(r.log2(), a * f64::log2(f64::exp(1.0)), suffix)
}

fn imreal((x, _, _): ComplexParts, _: CellReferenceIndex) -> CalcResult {
    CalcResult::Number(x)
}

fn imsec((x, y, suffix): ComplexParts, _: CellReferenceIndex) -> CalcResult {
    let x_cos = x.cos();
    let x_sin = x.sin();

    let y_cosh = y.cosh();
    let y_sinh = y.sinh();

    let t = x_cos * x_cos * y_cosh * y_cosh + x_sin * x_sin * y_sinh * y_sinh;
    complex_result(x_cos * y_cosh / t, x_sin * y_sinh / t, suffix)
}

fn imsech((x, y, suffix): ComplexParts, _: CellReferenceIndex) -> CalcResult {
    let x_cosh = x.cosh();
    let x_sinh = x.sinh();

    let y_cos = y.cos();
    let y_sin = y.sin();

    let t = x_cosh * x_cosh * y_cos * y_cos + x_sinh * x_sinh * y_sin * y_sin;
    complex_result(x_cosh * y_cos / t, -x_sinh * y_sin / t, suffix)
}

fn imsin((x, y, suffix): ComplexParts, _: CellReferenceIndex) -> CalcResult {
    complex_result(x.sin() * y.cosh(), x.cos() * y.sinh(), suffix)
}

fn imsinh((x, y, suffix): ComplexParts, _: CellReferenceIndex) -> CalcResult {
    complex_result(x.sinh() * y.cos(), x.cosh() * y.sin(), suffix)
}

fn imsqrt((x, y, suffix): ComplexParts, _: CellReferenceIndex) -> CalcResult {
    let r = f64::sqrt(x * x + y * y).sqrt();
    let a = f64::atan2(y, x);
    complex_result(r * f64::cos(a / 2.0), r * f64::sin(a / 2.0), suffix)
}

fn imtan((x, y, suffix): ComplexParts, _: CellReferenceIndex) -> CalcResult {
    let x_tan = x.tan();
    let y_tanh = y.tanh();

    let t = 1.0 + x_tan * x_tan * y_tanh * y_tanh;
    complex_result(
        (x_tan - x_tan * y_tanh * y_tanh) / t,
        (y_tanh + x_tan * x_tan * y_tanh) / t,
        suffix,
    )
}

#[cfg(test)]
mod tests {
    use crate::functions::engineering::complex::Suffix;

    use super::parse_complex_number as parse;

    #[test]
    fn test_parse_complex() {
        assert_eq!(parse("1+2i"), Ok((1.0, 2.0, Suffix::I)));
        assert_eq!(parse("2i"), Ok((0.0, 2.0, Suffix::I)));
        assert_eq!(parse("7.5"), Ok((7.5, 0.0, Suffix::I)));
        assert_eq!(parse("-7.5"), Ok((-7.5, 0.0, Suffix::I)));
        assert_eq!(parse("7-5i"), Ok((7.0, -5.0, Suffix::I)));
        assert_eq!(parse("i"), Ok((0.0, 1.0, Suffix::I)));
        assert_eq!(parse("7+i"), Ok((7.0, 1.0, Suffix::I)));
        assert_eq!(parse("7-i"), Ok((7.0, -1.0, Suffix::I)));
        assert_eq!(parse("-i"), Ok((0.0, -1.0, Suffix::I)));
        assert_eq!(parse("0"), Ok((0.0, 0.0, Suffix::I)));
    }
}
