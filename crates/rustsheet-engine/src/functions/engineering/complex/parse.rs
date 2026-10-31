use std::fmt;

use crate::expressions::{
    lexer::util::get_tokens,
    token::{OpSum, TokenType},
};
use crate::number_format::{complex_part_to_text, to_precision};

#[derive(Clone, Copy, PartialEq, Debug)]
pub(super) enum Suffix {
    I,
    J,
    /// A number with no imaginary part and no suffix ("3"): it takes the suffix of the number it
    /// meets, so `IMSUM("3+4j","3")` is "6+4j" (probes/eng-imsum-improduct-3d-j-empty E1-E10).
    Real,
}

impl Suffix {
    /// The suffix two operands share, or `None` when one is `i` and the other `j`.
    pub(super) fn join(self, other: Suffix) -> Option<Suffix> {
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

pub(super) struct Complex {
    pub(super) x: f64,
    pub(super) y: f64,
    pub(super) suffix: Suffix,
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

pub(super) fn parse_complex_number(s: &str) -> Result<(f64, f64, Suffix), String> {
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
