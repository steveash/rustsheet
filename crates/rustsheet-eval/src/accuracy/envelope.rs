//! Gate A's envelope (SPEC-0029 "The check" step 3, decision 0188 R2): the one definition of
//! whether an engine value is accurate enough, shared by the accuracy check, Gate B's R3 cells
//! and R1(c)'s probe rows (T-3359, T-3380).
//!
//! A value is inside when it is within 15 significant digits of Excel's value (the eval's own
//! visibly-off test, [`Visible::is_off`], never a second 15-digit rule) or within ε = 1e-9
//! relative of the 50-digit math, with an absolute floor of 1e-300. An error against a number,
//! either way, is never inside.

use serde::Serialize;

use crate::closeness::Visible;
use crate::runner::CellResult;
use crate::value::{Value, Verdict};

/// ε of decision 0188 R2: the relative distance from the math that is always accurate enough.
pub const EPSILON: f64 = 1e-9;

/// The absolute floor of decision 0188 R2: a value within this of the math is inside, so results
/// near 0 (an underflow, a subnormal Excel flushes to 0) never depend on the flush.
pub const ABSOLUTE_FLOOR: f64 = 1e-300;

/// G of decision 0188 R3: an R3 mark needs Excel off the math by more than this, relatively.
pub const R3_GAP: f64 = 1e-4;

/// A point's 50-digit math reference, parsed to the nearest double (SPEC-0029 "Point fields").
#[derive(Debug, Clone, PartialEq)]
pub enum MathValue {
    /// The math value, the double nearest its 50 significant digits.
    Number(f64),
    /// The arguments are outside the function's domain; the text says why. Excel's error code is
    /// the target, so the math carries none.
    DomainError(String),
}

impl Serialize for MathValue {
    /// `{"t":"n","v":<double>}` or `{"t":"e","why":<text>}`, the sidecar's tags.
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        let mut map = s.serialize_map(Some(2))?;
        match self {
            Self::Number(v) => {
                map.serialize_entry("t", "n")?;
                map.serialize_entry("v", v)?;
            }
            Self::DomainError(why) => {
                map.serialize_entry("t", "e")?;
                map.serialize_entry("why", why)?;
            }
        }
        map.end()
    }
}

/// Where a point falls (SPEC-0029 "The check" step 3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Envelope {
    /// Within 15 significant digits of Excel, not within ε of the math.
    Excel15,
    /// Within ε of the math (or the math's domain error matched), not within 15 digits of Excel
    /// (or Excel's value is pending, or the point is an R3 point).
    Math,
    /// Inside on both sides.
    Both,
    /// Inside on neither side: a failing point.
    Outside,
    /// An error against a number, either way, or two different error codes: a failing point that
    /// no R3 mark excuses.
    ErrorMismatch,
    /// A defect of the table, not of the engine: no math value (absent or stale), or a math
    /// domain error against an Excel number.
    Table,
}

impl Envelope {
    /// Whether the point is inside the envelope.
    #[must_use]
    pub fn is_inside(self) -> bool {
        matches!(self, Self::Excel15 | Self::Math | Self::Both)
    }
}

/// Classifies one point.
///
/// `excel` is the eval's scored cell for the point when Excel's value is filled (its `expected`
/// is Excel's value, its `actual` the engine's), `None` while pending. `engine` is the engine's
/// value, `math` the point's math reference (`None` when absent or stale), and `r3` whether
/// `math-over-excel.tsv` marks the point, which makes the Excel side count for nothing.
#[must_use]
pub fn envelope(
    excel: Option<&CellResult>,
    engine: &Value,
    math: Option<&MathValue>,
    r3: bool,
) -> Envelope {
    let Some(math) = math else {
        return Envelope::Table;
    };
    let excel_value = excel.map(|c| &c.expected);
    if matches!(math, MathValue::DomainError(_)) && matches!(excel_value, Some(Value::Number(_))) {
        return Envelope::Table;
    }
    // The reference the engine's value or error is held to: Excel's when filled, else the math.
    let reference_is_error = match excel_value {
        Some(Value::Error(_)) => Some(true),
        Some(Value::Number(_)) => Some(false),
        Some(_) => None,
        None => Some(matches!(math, MathValue::DomainError(_))),
    };
    match (engine, reference_is_error, excel_value) {
        (Value::Error(_), Some(false), _) | (Value::Number(_), Some(true), _) => {
            return Envelope::ErrorMismatch;
        }
        (Value::Error(a), _, Some(Value::Error(e))) if a != e => return Envelope::ErrorMismatch,
        _ => {}
    }
    // R3 drops Excel's side only where Excel gives a number: its errors are still matched
    // (decision 0188 R3). An unsupported engine result (`#NAME?`, `#N/IMPL`) is never inside,
    // though the closeness buckets leave it out of the visibly-off count.
    let r3_drops_excel = r3 && matches!(excel_value, Some(Value::Number(_)));
    let excel_side = !r3_drops_excel
        && excel.is_some_and(|c| c.verdict != Verdict::Unsupported && !Visible::is_off(c));
    let math_side = match (math, engine) {
        (MathValue::Number(m), Value::Number(e)) => within_math(*e, *m),
        (MathValue::DomainError(_), Value::Error(code)) => match excel_value {
            Some(Value::Error(excel_code)) => excel_code == code,
            Some(_) => false,
            None => true,
        },
        _ => false,
    };
    match (excel_side, math_side) {
        (true, true) => Envelope::Both,
        (true, false) => Envelope::Excel15,
        (false, true) => Envelope::Math,
        (false, false) => Envelope::Outside,
    }
}

/// Whether `engine` is within ε relative of `math`, or within the absolute floor.
#[must_use]
pub fn within_math(engine: f64, math: f64) -> bool {
    (engine - math).abs() <= (EPSILON * math.abs()).max(ABSOLUTE_FLOOR)
}

/// `|value − math| / |math|`: how far a value is from the math, relatively. Infinite for a
/// nonzero value against a math 0, 0 when both are 0.
#[must_use]
pub fn relative_error(value: f64, math: f64) -> f64 {
    let gap = (value - math).abs();
    if gap == 0.0 {
        0.0
    } else {
        gap / math.abs()
    }
}

/// Whether Excel's value is off the math by more than [`R3_GAP`]: the evidence an R3 mark on a
/// filled point needs (decision 0188 R3). Only two numbers can be: R3 never applies to an error.
/// The absolute floor does not apply here: a nonzero Excel value against an exact-0 math value is
/// infinitely off ([`relative_error`]), so it counts as more than G.
#[must_use]
pub fn excel_off_math(excel: &Value, math: &MathValue) -> bool {
    match (excel, math) {
        (Value::Number(x), MathValue::Number(m)) => relative_error(*x, *m) > R3_GAP,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::compare;

    /// The cell the runner scores for a filled point: Excel's value against the engine's.
    fn filled(excel: Value, engine: &Value) -> CellResult {
        let mut cell = CellResult::for_test("A1", Some("F(1)"), compare(&excel, engine));
        cell.expected = excel;
        cell.actual = engine.clone();
        cell
    }

    fn n(v: f64) -> Value {
        Value::Number(v)
    }

    fn err(code: &str) -> Value {
        Value::Error(code.into())
    }

    fn class(excel: Option<Value>, engine: &Value, math: Option<MathValue>, r3: bool) -> Envelope {
        let cell = excel.map(|x| filled(x, engine));
        match math {
            Some(m) => envelope(cell.as_ref(), engine, Some(&m), r3),
            None => envelope(cell.as_ref(), engine, None, r3),
        }
    }

    const MATH: f64 = 1.234_567_890_123_456;

    #[test]
    fn a_point_in_both() {
        let math = Some(MathValue::Number(MATH));
        assert_eq!(class(Some(n(MATH)), &n(MATH), math, false), Envelope::Both);
    }

    #[test]
    fn within_epsilon_of_the_math_only() {
        // 1e-12 relative: far inside ε, but visibly off Excel's value 1e-6 away.
        let engine = n(MATH * (1.0 + 1e-12));
        let excel = Some(n(MATH * (1.0 + 1e-6)));
        let math = Some(MathValue::Number(MATH));
        assert_eq!(class(excel, &engine, math, false), Envelope::Math);
    }

    #[test]
    fn within_fifteen_digits_of_excel_only() {
        // Excel and the engine agree; both are 1e-6 off the math (Excel's imprecision is a target).
        let both = n(MATH * (1.0 + 1e-6));
        let math = Some(MathValue::Number(MATH));
        assert_eq!(
            class(Some(both.clone()), &both, math, false),
            Envelope::Excel15
        );
    }

    #[test]
    fn outside_both() {
        let math = Some(MathValue::Number(MATH));
        let excel = Some(n(MATH));
        assert_eq!(
            class(excel, &n(MATH * (1.0 + 1e-6)), math, false),
            Envelope::Outside
        );
    }

    #[test]
    fn a_pending_point_is_checked_against_the_math() {
        let math = || Some(MathValue::Number(MATH));
        assert_eq!(
            class(None, &n(MATH * (1.0 + 1e-10)), math(), false),
            Envelope::Math
        );
        assert_eq!(
            class(None, &n(MATH * (1.0 + 1e-8)), math(), false),
            Envelope::Outside
        );
    }

    #[test]
    fn the_absolute_floor_holds_values_near_zero() {
        let math = || Some(MathValue::Number(1e-310));
        // A flushed 0 and a kept subnormal are both within 1e-300 of a subnormal math value.
        assert_eq!(class(None, &n(0.0), math(), false), Envelope::Math);
        assert_eq!(class(None, &n(-5e-301), math(), false), Envelope::Math);
        assert_eq!(class(None, &n(2e-300), math(), false), Envelope::Outside);
        assert!(within_math(0.0, 0.0));
        assert!(!within_math(1e-299, 0.0));
    }

    #[test]
    fn an_error_against_a_number_either_way() {
        let math = || Some(MathValue::Number(MATH));
        assert_eq!(
            class(Some(n(MATH)), &err("#NUM!"), math(), false),
            Envelope::ErrorMismatch
        );
        // The engine's "not implemented" #NAME? is an error too, though the eval buckets it apart.
        assert_eq!(
            class(Some(n(MATH)), &err("#NAME?"), math(), false),
            Envelope::ErrorMismatch
        );
        assert_eq!(
            class(Some(err("#NUM!")), &n(MATH), math(), false),
            Envelope::ErrorMismatch
        );
        // Pending: the math is the reference.
        assert_eq!(
            class(None, &err("#NUM!"), math(), false),
            Envelope::ErrorMismatch
        );
        let domain = Some(MathValue::DomainError("p outside (0,1)".into()));
        assert_eq!(class(None, &n(1.0), domain, false), Envelope::ErrorMismatch);
    }

    #[test]
    fn two_different_error_codes() {
        let domain = || Some(MathValue::DomainError("df < 1".into()));
        assert_eq!(
            class(Some(err("#NUM!")), &err("#VALUE!"), domain(), false),
            Envelope::ErrorMismatch
        );
        assert_eq!(
            class(Some(err("#NUM!")), &err("#NAME?"), domain(), false),
            Envelope::ErrorMismatch
        );
    }

    #[test]
    fn a_math_domain_error_matched_by_an_error() {
        let domain = || Some(MathValue::DomainError("df < 1".into()));
        assert_eq!(
            class(Some(err("#NUM!")), &err("#NUM!"), domain(), false),
            Envelope::Both
        );
        // Pending: any error matches the math's domain error.
        assert_eq!(
            class(None, &err("#DIV/0!"), domain(), false),
            Envelope::Math
        );
        // Excel answers an error where the math has a number: Excel's error is the target.
        assert_eq!(
            class(
                Some(err("#NUM!")),
                &err("#NUM!"),
                Some(MathValue::Number(MATH)),
                false
            ),
            Envelope::Excel15
        );
    }

    #[test]
    fn an_r3_point_is_checked_against_the_math_only() {
        // T.INV(1.87E-297,3937): Excel -156.96, math -40.26 (decision 0188 R3 case 1).
        let math = || Some(MathValue::Number(-40.26));
        let excel = || Some(n(-156.96));
        assert_eq!(
            class(excel(), &n(-156.96), math(), true),
            Envelope::Outside,
            "within 15 digits of Excel but not ε of the math"
        );
        assert_eq!(class(excel(), &n(-40.26), math(), true), Envelope::Math);
        assert_eq!(
            class(excel(), &n(-156.96), math(), false),
            Envelope::Excel15
        );
        // R3 never excuses an error against a number.
        assert_eq!(
            class(excel(), &err("#NUM!"), math(), true),
            Envelope::ErrorMismatch
        );
    }

    #[test]
    fn an_unsupported_engine_result_is_never_inside_on_the_excel_side() {
        // Excel text against the engine's #NAME?: compare() gives `Unsupported`, which the
        // closeness buckets do not count as visibly off.
        let math = || Some(MathValue::Number(MATH));
        for code in ["#NAME?", "#N/IMPL"] {
            let engine = err(code);
            let cell = filled(Value::Text("x".into()), &engine);
            assert_eq!(cell.verdict, Verdict::Unsupported);
            assert!(!Visible::is_off(&cell));
            assert_eq!(
                envelope(Some(&cell), &engine, math().as_ref(), false),
                Envelope::Outside,
                "{code}"
            );
        }
    }

    #[test]
    fn an_r3_point_still_matches_excel_errors() {
        let math = || Some(MathValue::Number(-40.26));
        assert_eq!(
            class(Some(err("#NUM!")), &err("#NUM!"), math(), true),
            Envelope::Excel15
        );
        assert_eq!(
            class(Some(err("#NUM!")), &err("#VALUE!"), math(), true),
            Envelope::ErrorMismatch
        );
        assert_eq!(
            class(Some(err("#NUM!")), &n(-40.26), math(), true),
            Envelope::ErrorMismatch
        );
    }

    #[test]
    fn r3_evidence_has_no_absolute_floor() {
        let zero = MathValue::Number(0.0);
        assert!(excel_off_math(&n(1e-310), &zero));
        assert!(!excel_off_math(&n(0.0), &zero));
    }

    #[test]
    fn a_missing_or_stale_math_value_is_a_table_defect() {
        assert_eq!(class(Some(n(1.0)), &n(1.0), None, false), Envelope::Table);
        assert_eq!(class(None, &n(1.0), None, false), Envelope::Table);
    }

    #[test]
    fn a_math_domain_error_against_an_excel_number_is_a_table_defect() {
        let domain = Some(MathValue::DomainError("p outside (0,1)".into()));
        assert_eq!(class(Some(n(0.5)), &n(0.5), domain, false), Envelope::Table);
    }

    #[test]
    fn r3_evidence_needs_excel_off_the_math_by_more_than_g() {
        let math = MathValue::Number(100.0);
        assert!(excel_off_math(&n(100.02), &math));
        assert!(!excel_off_math(&n(100.005), &math));
        assert!(!excel_off_math(&err("#NUM!"), &math));
        assert!(!excel_off_math(
            &n(1.0),
            &MathValue::DomainError("x".into())
        ));
        assert!(relative_error(1.0, 0.0).is_infinite());
        assert_eq!(relative_error(0.0, 0.0).to_bits(), 0);
    }
}
