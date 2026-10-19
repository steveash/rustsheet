//! How close a function's scored cells come to Excel: the five buckets of the parity page
//! (`tools/src/rustsheet_tools/parity_report.py`, T-2041), counted by the report (T-2036).

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
    /// Numbers within 1e-9 relative (`numeric_close`).
    pub close: usize,
    /// Every other failure.
    pub off: usize,
}

impl Closeness {
    /// Counts `cell` in its bucket; an `unsupported` cell is skipped.
    pub fn add(&mut self, cell: &CellResult) {
        let bucket = match cell.verdict {
            Verdict::Pass | Verdict::Deviation => &mut self.pass,
            Verdict::Unsupported => return,
            Verdict::NumericUlp => match (&cell.expected, &cell.actual) {
                (Value::Number(e), Value::Number(a)) if ulp_distance(*e, *a) <= 1 => &mut self.ulp1,
                _ => &mut self.ulp4,
            },
            Verdict::NumericClose => &mut self.close,
            Verdict::NumericMismatch
            | Verdict::TextMismatch
            | Verdict::BoolMismatch
            | Verdict::ErrorMismatch
            | Verdict::TypeMismatch => &mut self.off,
        };
        *bucket += 1;
    }

    /// The scored cells, every bucket summed.
    #[must_use]
    pub fn scored(&self) -> usize {
        self.pass + self.ulp1 + self.ulp4 + self.close + self.off
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
            close: 1,
            off: 2,
        };
        assert_eq!(c, expected);
        assert_eq!(c.scored(), 7);
    }
}
