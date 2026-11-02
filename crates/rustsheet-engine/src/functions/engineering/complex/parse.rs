use std::fmt;

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
