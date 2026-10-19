//! The lifting table's date, information, lookup and math entries (SPEC-0011 "The argument-kind
//! table"): EDATE to YEARFRAC, the IS* tests, ADDRESS and HYPERLINK, and the math functions.

use super::ArgKind::{Lift, LiftArrays, WholeByCaller};
use super::KernelWork::Constant;
use super::{ArgKinds, ARRAYS_1, ATP_1_OF_2, ATP_2, ATP_2_OF_3, LIFT_1, LIFT_1_OF_2, LIFT_2};
use crate::functions::Function;

/// `ADDRESS(row_num, column_num, [abs_num], [a1], [sheet_text])`: every argument lifts.
const ADDRESS: ArgKinds = ArgKinds::fixed(2, &[Lift, Lift, Lift, Lift, Lift], Constant);
/// `CEILING.MATH(number, [significance], [mode])` and FLOOR.MATH.
const LIFT_1_OF_3: ArgKinds = ArgKinds::fixed(1, &[Lift, Lift, Lift], Constant);

// --- Math (T-1802) --------------------------------------------------------------------------
/// `SERIESSUM(x, n, m, coefficients)`: x, n and m are Analysis `ToolPak` arguments, and the
/// coefficients are read once by the function itself (`Model::series_coefficients`), which keeps
/// the scalar reading of a one-cell text reference. Each element walks the coefficients, so the
/// call is charged their cells per element (`LiftCost::Cells`); the kernel itself loops once per
/// coefficient, a fixed count for the call.
const SERIES: ArgKinds = ArgKinds::fixed(
    4,
    &[LiftArrays, LiftArrays, LiftArrays, WholeByCaller],
    Constant,
);
// --- End math (T-1802) ----------------------------------------------------------------------

/// This family's part of the lifting table (`Function::arg_kinds`).
pub(super) fn arg_kinds(function: &Function) -> Option<&'static ArgKinds> {
    match function {
        // Every argument lifts over an array and refuses a multi-cell range, also a
        // reference that a function returns (`EDATE(IF(TRUE,A1:A60),1)`, `EOMONTH(..,TAKE(..))`):
        // probes/scalar-lift-shapes H1, H2, H6, H50, -b K3, gen-m9/b58b6d7cd39d195f A9, and
        // probes/date-argument-modes F/G 20-26 (docs/excel-semantics/dates.md "Date function
        // argument modes", arrays.md "Scalar arguments lift"). EOMONTH and WEEKNUM run in
        // O(1); WEEKNUM's weekday search steps back at most six days.
        Function::Edate | Function::Eomonth => Some(&ATP_2),
        Function::Weeknum => Some(&ATP_1_OF_2),
        // YEARFRAC's basis over a range: probes/scalar-lift-shapes H6; its dates:
        // date-argument-modes F/G 54, 56. Basis 1 counts the year lengths in closed form.
        Function::Yearfrac => Some(&ATP_2_OF_3),
        // The IS* tests lift over arrays and ranges, an element as stored (`EmptyCell` is
        // ISBLANK, an error element ISERROR): probes/is-functions-array-lift,
        // probes/info-logic-argument-modes (docs/excel-semantics/information.md "IS* type
        // tests lift over arrays and ranges"). ERROR.TYPE follows (probes/rich-errors-arg-kinds
        // F10:G10, F40:G41), and so does ENCODEURL (docs/excel-semantics/web.md).
        Function::Isnumber
        | Function::Istext
        | Function::Isnontext
        | Function::Islogical
        | Function::Isblank
        | Function::Iserror
        | Function::Iserr
        | Function::Isna
        | Function::ErrorType
        | Function::Encodeurl => Some(&LIFT_1),
        // A multi-cell range is `#VALUE!` with no intersection; an array constant lifts
        // (probes/is-functions-array-lift J2, K2, L3, M30, M32; information.md "ISEVEN/ISODD
        // over a range").
        Function::Iseven | Function::Isodd => Some(&ARRAYS_1),
        // ADDRESS's row_num, column_num and abs_num, and HYPERLINK's link_location and
        // friendly_name, lift over a range and an array (probes/lookup-argument-modes F1:G4,
        // F11:G13; docs/excel-semantics/lookup.md "Lookup and reference function argument
        // modes"); so do ADDRESS's a1 (gen-m7/f68fafa0ca2eba26 A7) and HYPERLINK's
        // friendly_name over an array (gen-m7/2bb922eef72e785e A22). ADDRESS's sheet_text is
        // unprobed and follows SPEC-0011's rule for scalar parameters.
        Function::Address => Some(&ADDRESS),
        Function::Hyperlink => Some(&LIFT_1_OF_2),
        // --- Math (T-1802) ---
        // QUOTIENT, FACTDOUBLE and SERIESSUM's x, n and m are Analysis ToolPak arguments: they
        // lift over an array (probes/math-atp-array-lift C1-C16) and refuse a multi-cell
        // range, also on its own rows (probes/scalar-lift-open-a Y19-Y21, Y24-Y26); a range
        // beside an array is `#VALUE!` in every element (math-atp-array-lift C27-C28).
        // SERIESSUM's coefficients are read whole (C13, C40; -a Y27). MULTINOMIAL, GCD and LCM
        // have no lifted argument: they give one value over a range or an array (-a Y22,
        // Y23, Y28-Y30; math-atp-array-lift C34-C39), so they keep their own readers and no
        // entry (docs/excel-semantics/numbers.md "QUOTIENT, FACTDOUBLE and SERIESSUM argument
        // modes"). QUOTIENT is O(1); FACTDOUBLE's product of n, n-2, ... overflows a double
        // within 151 steps whatever n is (`FACTDOUBLE(301)` is `#NUM!`), a fixed cap.
        Function::Quotient => Some(&ATP_2),
        Function::Factdouble => Some(&ARRAYS_1),
        Function::Seriessum => Some(&SERIES),
        // --- End math (T-1802) ---
        // --- Math number lifts (T-1773) ---
        // The one-number math functions, the ROUND family, MOD and the CEILING/FLOOR family
        // lift element-wise over arrays and ranges in every argument, a range on its own rows
        // included (gen-m2/893302d6c5b99e2e Calc!A313, probes/geomean-overflow-cut-3 E4,
        // gen-m7/e5a361112d83d0c3 Calc!A9, gen-m9/89693ab0cafc29df Calc!A14). Each is O(1),
        // FACT's product included: it overflows a double within 171 steps whatever n is.
        // Direct trigonometry:
        Function::Sin
        | Function::Cos
        | Function::Tan
        | Function::Cot
        | Function::Csc
        | Function::Sec
        // Inverse trigonometry:
        | Function::Asin
        | Function::Acos
        | Function::Atan
        | Function::Acot
        // Hyperbolic:
        | Function::Sinh
        | Function::Cosh
        | Function::Tanh
        | Function::Coth
        | Function::Csch
        | Function::Sech
        // Inverse hyperbolic:
        | Function::Asinh
        | Function::Acosh
        | Function::Atanh
        | Function::Acoth
        // Logarithms, powers:
        | Function::Ln
        | Function::Log10
        | Function::Exp
        | Function::Sqrt
        // Sign, rounding to integers, factorial, angles:
        | Function::Abs
        | Function::Sign
        | Function::Int
        | Function::Even
        | Function::Odd
        | Function::Fact
        | Function::Degrees
        | Function::Radians => Some(&LIFT_1),
        Function::Round
        | Function::Roundup
        | Function::Rounddown
        | Function::Mod
        | Function::Floor
        | Function::Ceiling
        | Function::EcmaCeiling => Some(&LIFT_2),
        Function::Trunc
        | Function::CeilingPrecise
        | Function::IsoCeiling
        | Function::FloorPrecise => Some(&LIFT_1_OF_2),
        Function::CeilingMath | Function::FloorMath => Some(&LIFT_1_OF_3),
        // --- End math number lifts (T-1773) ---
        _ => None,
    }
}
