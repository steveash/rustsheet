//! How close a function's scored cells come to Excel: the six buckets of the parity page
//! (`tools/src/rustsheet_tools/progress_report/parity.py`, T-2041), counted by the report (T-2036).
//!
//! The buckets follow decision 0083's order of parity: a cell is *visibly* right when the 15
//! significant digits Excel shows are right (`digits15` and better), and visibly off otherwise
//! (`visible_numeric`, `visible_other`; T-0061). The ulp buckets take precedence: a number within
//! 4 ulp is never visibly off, even where 4 ulp exceed half a unit in the 15th digit (decision
//! 0088).

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::runner::CellResult;
use crate::value::{ulp_distance, Value, Verdict};

/// Scored cells per closeness bucket. `unsupported` cells are not scored and count in none.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Closeness {
    /// Bit-exact, or an accepted deviation's pinned value (decision 0036).
    pub pass: usize,
    /// Numbers 1 ulp apart.
    pub ulp1: usize,
    /// Numbers 2 to 4 ulp apart.
    pub ulp4: usize,
    /// Numbers more than 4 ulp apart but within half a unit in the 15th significant digit of
    /// the expected value ([`within_fifteen_digits`]). They may still display differently when
    /// they straddle a rounding boundary of that digit.
    pub digits15: usize,
    /// Numbers further apart than half a unit in the 15th significant digit, including 0
    /// against nonzero and non-finite values.
    pub visible_numeric: usize,
    /// A wrong type, error, text or boolean.
    pub visible_other: usize,
}

/// Which bucket a scored cell falls in; `None` for an `unsupported` cell.
fn bucket(cell: &CellResult) -> Option<Bucket> {
    Some(match cell.verdict {
        Verdict::Pass | Verdict::Deviation => Bucket::Pass,
        Verdict::Unsupported => return None,
        Verdict::NumericUlp => match (&cell.expected, &cell.actual) {
            (Value::Number(e), Value::Number(a)) if ulp_distance(*e, *a) <= 1 => Bucket::Ulp1,
            _ => Bucket::Ulp4,
        },
        Verdict::NumericClose | Verdict::NumericMismatch => match (&cell.expected, &cell.actual) {
            (Value::Number(e), Value::Number(a)) if within_fifteen_digits(*e, *a) => {
                Bucket::Digits15
            }
            _ => Bucket::VisibleNumeric,
        },
        Verdict::TextMismatch
        | Verdict::BoolMismatch
        | Verdict::ErrorMismatch
        | Verdict::TypeMismatch => Bucket::VisibleOther,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Bucket {
    Pass,
    Ulp1,
    Ulp4,
    Digits15,
    VisibleNumeric,
    VisibleOther,
}

impl Closeness {
    /// Counts `cell` in its bucket; an `unsupported` cell is skipped.
    pub fn add(&mut self, cell: &CellResult) {
        let Some(b) = bucket(cell) else { return };
        *match b {
            Bucket::Pass => &mut self.pass,
            Bucket::Ulp1 => &mut self.ulp1,
            Bucket::Ulp4 => &mut self.ulp4,
            Bucket::Digits15 => &mut self.digits15,
            Bucket::VisibleNumeric => &mut self.visible_numeric,
            Bucket::VisibleOther => &mut self.visible_other,
        } += 1;
    }

    /// The scored cells, every bucket summed.
    #[must_use]
    pub fn scored(&self) -> usize {
        self.pass + self.ulp1 + self.ulp4 + self.digits15 + self.visible()
    }

    /// The visibly-off cells: `visible_numeric` and `visible_other` (decision 0083).
    #[must_use]
    pub fn visible(&self) -> usize {
        self.visible_numeric + self.visible_other
    }
}

/// Visibly-off cells blamed on one function ([`blamed`]), split as the buckets split them.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Visible {
    /// Cells in the `visible_numeric` bucket.
    pub numeric: usize,
    /// Cells in the `visible_other` bucket.
    pub other: usize,
}

impl Visible {
    /// `cell`'s contribution: one cell in its visible bucket, or nothing.
    #[must_use]
    pub fn of(cell: &CellResult) -> Visible {
        match bucket(cell) {
            Some(Bucket::VisibleNumeric) => Visible {
                numeric: 1,
                other: 0,
            },
            Some(Bucket::VisibleOther) => Visible {
                numeric: 0,
                other: 1,
            },
            _ => Visible::default(),
        }
    }

    /// Adds `other`'s counts to these.
    pub fn merge(&mut self, other: Visible) {
        self.numeric += other.numeric;
        self.other += other.other;
    }

    /// Both kinds summed.
    #[must_use]
    pub fn total(&self) -> usize {
        self.numeric + self.other
    }

    /// Whether `cell` is visibly off (decision 0083): the one test every report section uses.
    #[must_use]
    pub fn is_off(cell: &CellResult) -> bool {
        Visible::of(cell).total() > 0
    }
}

/// Whether `actual` is within half a unit in the 15th significant digit of `expected`: the
/// digits Excel displays are the same, or differ only by rounding at that digit (decision 0083).
///
/// The boundary itself counts as within. An expected 0 has no 15th digit, so only an exact 0
/// (either sign) is within. For a subnormal `expected` (and the smallest normals) half the unit
/// underflows to 0 or a few subnormal steps, so in practice only an equal value is within.
/// Non-finite values are never within. The half unit is the double nearest `5e(E-15)` for the
/// decimal exponent `E` of `expected`, so a gap within half an ulp of it may land on either side;
/// the unit tests pin the exactly representable boundary.
#[must_use]
pub fn within_fifteen_digits(expected: f64, actual: f64) -> bool {
    if !expected.is_finite() || !actual.is_finite() {
        return false;
    }
    let gap = (actual - expected).abs();
    if expected == 0.0 {
        return gap == 0.0;
    }
    // `{:e}` prints the shortest round-trip digits, whose exponent is the true decimal exponent
    // of `expected` (a double never rounds up to the next power of ten in that form).
    let exponent = format!("{:e}", expected.abs())
        .split_once('e')
        .and_then(|(_, e)| e.parse::<i32>().ok());
    let Some(exponent) = exponent else {
        return false;
    };
    let half_unit = format!("5e{}", exponent - 15).parse::<f64>().unwrap_or(0.0);
    gap <= half_unit
}

/// Functions that only route or combine values (control flow, plain sums, reference picks,
/// dates built from parts). A visibly-off cell is blamed on the other functions it calls, so
/// `IF(x, GAMMA(y))` counts against GAMMA, not IF (decision 0083's attribution).
pub const GLUE: &[&str] = &[
    "AND", "CHOOSE", "COLUMN", "DATE", "FALSE", "IF", "IFERROR", "IFNA", "IFS", "INDEX", "LET",
    "NOT", "OR", "ROW", "SUM", "SWITCH", "TIME", "TRUE",
];

/// The functions of `called` that a visibly-off cell is blamed on: every function not in
/// [`GLUE`], or all of `called` when the formula calls only glue.
#[must_use]
pub fn blamed(called: &BTreeSet<String>) -> Vec<&String> {
    let specialised: Vec<&String> = called
        .iter()
        .filter(|f| !GLUE.contains(&f.as_str()))
        .collect();
    if specialised.is_empty() {
        called.iter().collect()
    } else {
        specialised
    }
}

/// The function a cell is attributed to under decision 0118: the one function of `called` not in
/// [`GLUE`], or, when the formula calls only glue, its one glue function. `None` when the formula
/// calls two or more distinct non-glue functions, or only glue and two or more distinct ones (or
/// nothing). `called` is the distinct-call set [`blamed`] takes, so repeated calls of one function
/// count once: `GAMMA(1)+GAMMA(2)` is GAMMA's. Unlike [`blamed`], `IF(AND(..),..)` is nobody's.
#[must_use]
pub fn sole_call(called: &BTreeSet<String>) -> Option<&String> {
    let mut specialised = called.iter().filter(|f| !GLUE.contains(&f.as_str()));
    match (specialised.next(), specialised.next()) {
        (Some(f), None) => Some(f),
        (None, _) if called.len() == 1 => called.first(),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cell(expected: Value, actual: Value, verdict: Verdict) -> CellResult {
        CellResult {
            sheet: "S".into(),
            cell: "A1".into(),
            formula: None,
            anchor_formula: None,
            expected,
            actual,
            verdict,
            from_excel: true,
            listed_deviation: false,
            graph_subnormal: false,
        }
    }

    #[test]
    fn cells_fall_in_their_bucket_and_unsupported_in_none() {
        let one = 1.0_f64;
        let up = |n: u64| f64::from_bits(one.to_bits() + n);
        let n = Value::Number;
        let mut c = Closeness::default();
        for x in [
            cell(n(one), n(one), Verdict::Pass),
            cell(n(one), n(9.0), Verdict::Deviation),
            cell(n(one), n(up(1)), Verdict::NumericUlp),
            cell(n(one), n(up(4)), Verdict::NumericUlp),
            cell(n(one), n(up(5)), Verdict::NumericClose),
            cell(n(one), n(one + 1e-12), Verdict::NumericClose),
            cell(n(one), n(2.0), Verdict::NumericMismatch),
            cell(Value::Text("a".into()), n(one), Verdict::TypeMismatch),
            cell(n(one), Value::Error("#NAME?".into()), Verdict::Unsupported),
        ] {
            c.add(&x);
        }
        let expected = Closeness {
            pass: 2,
            ulp1: 1,
            ulp4: 1,
            digits15: 1,
            visible_numeric: 2,
            visible_other: 1,
        };
        assert_eq!(c, expected);
        assert_eq!(c.scored(), 8);
        assert_eq!(c.visible(), 3);
    }

    #[test]
    fn half_a_unit_in_the_fifteenth_digit_is_the_boundary() {
        // 1e15 has 16 digits, so its 15th-digit unit is 10 and half of it, 5, is exact.
        assert!(within_fifteen_digits(1e15, 1e15 + 5.0));
        assert!(within_fifteen_digits(1e15, 1e15 - 5.0));
        assert!(!within_fifteen_digits(1e15, 1e15 + 6.0));
        assert!(!within_fifteen_digits(-1e15, -1e15 - 6.0));
        // 9.99e15 still has exponent 15; 1e16 moves the unit to 100.
        assert!(within_fifteen_digits(9.99e15, 9.99e15 + 4.0));
        assert!(within_fifteen_digits(1e16, 1e16 + 50.0));
        assert!(!within_fifteen_digits(1e16, 1e16 + 52.0));
        // Fractions: 1 shows 14 decimals, so 5e-15 is half a unit and 2e-14 is not.
        assert!(within_fifteen_digits(1.0, 1.0 + 4e-15));
        assert!(!within_fifteen_digits(1.0, 1.0 + 2e-14));
        assert!(within_fifteen_digits(0.001_234, 0.001_234 + 4e-18));
        assert!(!within_fifteen_digits(0.001_234, 0.001_234 + 2e-17));
    }

    #[test]
    fn zero_subnormal_and_non_finite_expectations() {
        assert!(within_fifteen_digits(0.0, -0.0));
        assert!(!within_fifteen_digits(0.0, 1e-300));
        assert!(!within_fifteen_digits(1e-300, 0.0));
        let tiny = f64::from_bits(1000);
        assert!(within_fifteen_digits(tiny, tiny));
        assert!(!within_fifteen_digits(tiny, f64::from_bits(1010)));
        // The smallest normal still has a representable half unit.
        let normal = f64::MIN_POSITIVE;
        assert!(within_fifteen_digits(normal, normal * (1.0 + 1e-15)));
        assert!(!within_fifteen_digits(f64::NAN, f64::NAN));
        assert!(!within_fifteen_digits(1.0, f64::INFINITY));
    }

    #[test]
    fn visibly_off_cells_blame_the_specialised_functions() {
        let set = |names: &[&str]| -> BTreeSet<String> {
            names.iter().map(|s| (*s).to_string()).collect()
        };
        let gamma_in_if = set(&["IF", "GAMMA", "SUM"]);
        assert_eq!(blamed(&gamma_in_if), vec!["GAMMA"]);
        let glue_only = set(&["DATE", "IF"]);
        assert_eq!(blamed(&glue_only), vec!["DATE", "IF"]);
        assert!(blamed(&BTreeSet::new()).is_empty());
        let n = Value::Number;
        let mut v = Visible::default();
        for x in [
            cell(n(1.0), n(2.0), Verdict::NumericMismatch),
            cell(n(1.0), n(1.0 + 4e-15), Verdict::NumericClose),
            cell(Value::Bool(true), Value::Bool(false), Verdict::BoolMismatch),
            cell(n(1.0), n(1.0), Verdict::Pass),
        ] {
            v.merge(Visible::of(&x));
        }
        assert_eq!(
            v,
            Visible {
                numeric: 1,
                other: 1
            }
        );
        assert_eq!(v.total(), 2);
    }

    #[test]
    fn a_cell_is_attributed_to_its_sole_non_glue_call() {
        use crate::formula_calls::functions_in;
        let sole = |formula: &str| sole_call(&functions_in(formula)).cloned();
        let name = |f: &str| Some(f.to_string());
        assert_eq!(sole("IF(A1,GAMMA(B1))"), name("GAMMA"));
        assert_eq!(sole("GAMMA(1)+GAMMA(2)"), name("GAMMA"));
        assert_eq!(sole("SUM(A1:A3)"), name("SUM"));
        assert_eq!(sole("IF(A1,1,2)"), name("IF"));
        assert_eq!(sole("GAMMA(BETA.DIST(0.5,2,3,TRUE))"), None);
        assert_eq!(sole("A1+1"), None);
        // Decision 0118: a glue-only formula with two distinct calls is nobody's, though
        // decision 0088's `blamed` still blames both.
        let if_and = functions_in("IF(AND(A1,B1),1,2)");
        assert_eq!(sole_call(&if_and), None);
        assert_eq!(blamed(&if_and), vec!["AND", "IF"]);
    }
}
