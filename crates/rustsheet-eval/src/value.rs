//! Engine-independent cell values and the comparison rules used to score them.
//!
//! Comparison is deliberately strict: a cell only *passes* when it is exactly
//! what Excel produced (bit-identical numbers, identical text/booleans/errors).
//! Near misses are still classified (ULP distance, relative closeness) so the
//! gap report can separate "precision" work from "wrong semantics" work.

use serde::{Deserialize, Serialize};
use std::fmt;

/// A computed cell value, as Excel would report it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "t", content = "v")]
pub enum Value {
    #[serde(rename = "n")]
    Number(f64),
    #[serde(rename = "s")]
    Text(String),
    #[serde(rename = "b")]
    Bool(bool),
    /// Canonical Excel error literal, e.g. `#DIV/0!`.
    #[serde(rename = "e")]
    Error(String),
    #[serde(rename = "empty")]
    Empty,
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{n:?}"),
            Value::Text(s) => write!(f, "{s:?}"),
            Value::Bool(b) => write!(f, "{}", if *b { "TRUE" } else { "FALSE" }),
            Value::Error(e) => write!(f, "{e}"),
            Value::Empty => write!(f, "<empty>"),
        }
    }
}

impl Value {
    #[must_use]
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Number(_) => "number",
            Value::Text(_) => "text",
            Value::Bool(_) => "bool",
            Value::Error(_) => "error",
            Value::Empty => "empty",
        }
    }
}

/// Outcome of comparing an engine value against the oracle value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    /// Bit-identical (−0 and +0 are considered identical).
    Pass,
    /// Numbers within 4 ULPs: almost certainly an operation-order / rounding difference.
    NumericUlp,
    /// Numbers with relative error ≤ 1e-9: a precision/algorithm difference.
    NumericClose,
    /// Numbers that genuinely differ.
    NumericMismatch,
    TextMismatch,
    BoolMismatch,
    /// Both errors, but different error kinds.
    ErrorMismatch,
    /// Engine produced `#NAME?`-like "unknown/unimplemented" output where Excel did not.
    Unsupported,
    /// Value types differ (e.g. number vs text, value vs error).
    TypeMismatch,
    /// A listed cell gives its pinned by-design value (decision 0036): neither a pass nor a
    /// failure, and outside the bit-exact denominator.
    Deviation,
}

impl Verdict {
    #[must_use]
    pub fn is_pass(self) -> bool {
        self == Verdict::Pass
    }

    /// Whether the cell is a known failure: neither a pass nor an accepted deviation.
    #[must_use]
    pub fn is_failure(self) -> bool {
        !matches!(self, Verdict::Pass | Verdict::Deviation)
    }
}

/// Errors the engine uses to signal "I don't implement this".
const UNSUPPORTED_ERRORS: &[&str] = &["#N/IMPL", "#NAME?"];

/// Distance between two finite doubles in units in the last place.
///
/// `ulp_distance` in `tools/src/rustsheet_tools/fitlab/score.py` mirrors it; the fit lab and
/// `tools/src/rustsheet_tools/parity_report.py` use that copy, so change both together.
#[must_use]
pub fn ulp_distance(a: f64, b: f64) -> u64 {
    fn ordered(x: f64) -> i64 {
        // Map the IEEE-754 bit pattern onto a monotonically ordered integer line.
        let bits = x.to_bits().cast_signed();
        if bits < 0 {
            i64::MIN - bits
        } else {
            bits
        }
    }
    ordered(a).abs_diff(ordered(b))
}

/// Compares an engine value (`actual`) against the oracle value (`expected`).
#[must_use]
pub fn compare(expected: &Value, actual: &Value) -> Verdict {
    match (expected, actual) {
        (Value::Number(e), Value::Number(a)) => compare_numbers(*e, *a),
        (Value::Text(e), Value::Text(a)) => {
            if e == a {
                Verdict::Pass
            } else {
                Verdict::TextMismatch
            }
        }
        (Value::Bool(e), Value::Bool(a)) => {
            if e == a {
                Verdict::Pass
            } else {
                Verdict::BoolMismatch
            }
        }
        (Value::Error(e), Value::Error(a)) => {
            if e == a {
                Verdict::Pass
            } else if UNSUPPORTED_ERRORS.contains(&a.as_str()) {
                Verdict::Unsupported
            } else {
                Verdict::ErrorMismatch
            }
        }
        (Value::Empty, Value::Empty) => Verdict::Pass,
        (_, Value::Error(a)) if UNSUPPORTED_ERRORS.contains(&a.as_str()) => Verdict::Unsupported,
        _ => Verdict::TypeMismatch,
    }
}

#[allow(clippy::float_cmp, reason = "bit-exact equality is the pass criterion")]
fn compare_numbers(e: f64, a: f64) -> Verdict {
    if e == a || (e.is_nan() && a.is_nan()) {
        return Verdict::Pass;
    }
    if !e.is_finite() || !a.is_finite() {
        return Verdict::NumericMismatch;
    }
    if ulp_distance(e, a) <= 4 {
        return Verdict::NumericUlp;
    }
    let scale = e.abs().max(a.abs());
    if (e - a).abs() <= 1e-9 * scale {
        Verdict::NumericClose
    } else {
        Verdict::NumericMismatch
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_numbers_pass_and_signed_zero_is_equal() {
        assert_eq!(
            compare(&Value::Number(1.5), &Value::Number(1.5)),
            Verdict::Pass
        );
        assert_eq!(
            compare(&Value::Number(0.0), &Value::Number(-0.0)),
            Verdict::Pass
        );
    }

    #[test]
    fn near_numbers_are_classified() {
        let x = 0.1 + 0.2;
        assert_eq!(
            compare(&Value::Number(0.3), &Value::Number(x)),
            Verdict::NumericUlp
        );
        assert_eq!(
            compare(&Value::Number(1.0), &Value::Number(1.0 + 1e-12)),
            Verdict::NumericClose
        );
        assert_eq!(
            compare(&Value::Number(1.0), &Value::Number(1.001)),
            Verdict::NumericMismatch
        );
    }

    #[test]
    fn unsupported_function_is_its_own_bucket() {
        assert_eq!(
            compare(&Value::Number(1.0), &Value::Error("#NAME?".into())),
            Verdict::Unsupported
        );
        assert_eq!(
            compare(
                &Value::Error("#NAME?".into()),
                &Value::Error("#NAME?".into())
            ),
            Verdict::Pass
        );
        assert_eq!(
            compare(
                &Value::Error("#N/A".into()),
                &Value::Error("#VALUE!".into())
            ),
            Verdict::ErrorMismatch
        );
    }

    #[test]
    fn ulp_distance_crosses_zero() {
        assert_eq!(ulp_distance(0.0, -0.0), 0);
        assert_eq!(ulp_distance(f64::from_bits(1), -f64::from_bits(1)), 2);
    }

    #[test]
    fn value_json_roundtrip() {
        let v = vec![
            Value::Number(0.1),
            Value::Text("x".into()),
            Value::Bool(true),
            Value::Error("#N/A".into()),
            Value::Empty,
        ];
        let s = serde_json::to_string(&v).unwrap_or_default();
        assert_eq!(
            s,
            r##"[{"t":"n","v":0.1},{"t":"s","v":"x"},{"t":"b","v":true},{"t":"e","v":"#N/A"},{"t":"empty"}]"##
        );
        let back: Vec<Value> = serde_json::from_str(&s).unwrap_or_default();
        assert_eq!(back, v);
    }

    /// Oracle sidecars hold shortest-round-trip doubles. Without `serde_json`'s `float_roundtrip`
    /// feature these parsed 1 ulp off, so every oracle-scored cell near them read as `NumericUlp`
    /// (poi/45544.xlsx Profile of Class!B15, 2026-10-04).
    #[test]
    fn json_numbers_parse_to_the_exact_double() {
        for (text, want) in [
            ("0.010582010582010581", 0.010_582_010_582_010_581_f64),
            ("0.16390423572744015", 0.163_904_235_727_440_15),
        ] {
            let json = format!(r#"{{"t":"n","v":{text}}}"#);
            let got: Option<Value> = serde_json::from_str(&json).ok();
            assert_eq!(got, Some(Value::Number(want)), "{text}");
        }
    }
}
