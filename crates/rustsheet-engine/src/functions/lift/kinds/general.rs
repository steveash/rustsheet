//! The lifting table's date, information, lookup and math entries (SPEC-0011 "The argument-kind
//! table"): EDATE to YEARFRAC, the IS* tests, ADDRESS and HYPERLINK, and the math functions.

use super::ArgKind::{Lift, LiftArrays, Whole, WholeByCaller};
use super::KernelWork::{Constant, Output, Weighted};
use super::{
    ArgKinds, CallResult, ARRAYS_1, ATP_1_OF_2, ATP_2, ATP_2_OF_3, LIFT_1, LIFT_1_OF_2,
    LIFT_1_OF_3, LIFT_2, LIFT_2_OF_3, LIFT_3,
};
use crate::functions::Function;

/// `COMBIN(number, number_chosen)` and `COMBINA(number, number_chosen)`: both arguments lift
/// (T-2579). `combin`'s loop multiplies factors of at least 1 and stops at the first infinity, so
/// it runs at most about 1,030 steps (T-2453); each element is charged 2^10.
const COMBIN_FAMILY: ArgKinds = ArgKinds::fixed(2, &[Lift, Lift], Weighted(1 << 10));
/// `ADDRESS(row_num, column_num, [abs_num], [a1], [sheet_text])`: every argument lifts.
const ADDRESS: ArgKinds = ArgKinds::fixed(2, &[Lift, Lift, Lift, Lift, Lift], Constant);
/// `WORKDAY(start_date, days, [holidays])` and `NETWORKDAYS(start_date, end_date, [holidays])`:
/// the dates are Analysis `ToolPak` arguments, and the holidays are read once by the function
/// itself (`Model::holiday_serials`) and prepared, so the call is charged their cells once
/// (`LiftCost::Prepared`).
const WORKDAYS: ArgKinds = ArgKinds::fixed(2, &[LiftArrays, LiftArrays, WholeByCaller], Constant);
/// `WORKDAY.INTL(start_date, days, [weekend], [holidays])` and NETWORKDAYS.INTL: the weekend lifts
/// over a range too.
const WORKDAYS_INTL: ArgKinds =
    ArgKinds::fixed(2, &[LiftArrays, LiftArrays, Lift, WholeByCaller], Constant);

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

// --- Array-valued (T-1834) -------------------------------------------------------------------
/// `SEQUENCE(rows, [columns], [start], [step])`: every argument lifts over an array and a range
/// (`ROWS(SEQUENCE(A1:A3))` is 3, `SEQUENCE({1;2;3})` and `SEQUENCE(2,2,{1;10})` are 3x1 and 2x1,
/// probes/m7-argument-modes F1:H3; columns and step, probes/m7-lift-take-drop-sortby rows 17-18).
/// Each lifted element is charged the cells it builds.
const SEQUENCE: ArgKinds =
    ArgKinds::fixed(1, &[Lift, Lift, Lift, Lift], Output { weight: 1 }).array();
/// `MAKEARRAY(rows, cols, lambda)`: rows and cols lift (`MAKEARRAY(A1:A2,2,LAMBDA(r,c,r+c))` is
/// 2x1 with first element 2, probes/m7-argument-modes F23:H23), the LAMBDA is read whole. Its
/// kernel calls the LAMBDA once per cell, so a lifted element is charged 128 per cell (SPEC-0011
/// "Array-valued functions" rule 3).
const MAKEARRAY: ArgKinds =
    ArgKinds::fixed(3, &[Lift, Lift, Whole], Output { weight: 128 }).array();
/// `TAKE(array, rows, [columns])` and `DROP(array, rows, [columns])`: rows and columns lift over
/// an array and a range (`ROWS(TAKE(A1:B3,A1:A2))` and `ROWS(DROP(A1:B3,{1;2}))` are 2,
/// probes/m7-argument-modes F4:F6; columns, probes/m7-lift-take-drop-sortby rows 27-28), the array
/// is read whole by the function itself, a reference kept (`Model::take_drop_reference`, T-1838).
/// Each lifted element is charged the cells it builds.
const TAKE_DROP: ArgKinds = ArgKinds::fixed(2, &[Whole, Lift, Lift], Output { weight: 1 }).array();
// --- M7 (T-1836) ---
/// `TOCOL(array, [ignore], [scan_by_column])`: ignore lifts over an array and a range
/// (`TOCOL(A1:B3,{0;1})` is 2x1 with first element 1, probes/m7-argument-modes F9:H9, K8; over an
/// array, gen-m9/1782efcecd5d4b1b Calc!A19), `scan_by_column` too (probes/m7-lift-take-drop-sortby
/// row 24). The array is read once by the function and handed to every element whole; each
/// element walks it (`LiftCost::Cells`) and is charged the cells it builds. TOROW shares it
/// (`TOROW(Data!B3:E8,3,WRAPCOLS(Data!D3:D11,1))` is 1x9, gen-m7/7d67c559ce7536fa A619; T-2313).
const TOCOL: ArgKinds =
    ArgKinds::fixed(1, &[WholeByCaller, Lift, Lift], Output { weight: 1 }).array();
/// `UNIQUE(array, [by_col], [exactly_once])`: `by_col` lifts over an array and a range
/// (`UNIQUE(D1:D3,{FALSE;TRUE})` is 2x1, probes/m7-argument-modes F12:H12; over a range,
/// gen-m9/4df1b22d3ed63f58 Calc!A36), `exactly_once` too (probes/m7-lift-take-drop-sortby rows
/// 25-26). Its later elements are empty unless every entry is equal (`CallResult::ArrayFirst`,
/// probes/m7-unique-lift-elements; T-2300). Read and charged as [`TOCOL`].
const UNIQUE: ArgKinds =
    ArgKinds::fixed(1, &[WholeByCaller, Lift, Lift], Output { weight: 1 }).array_first_column();
// --- End M7 (T-1836) ---
// --- M7 (T-1839) ---
/// `SORTBY(array, by_array1, [sort_order1], [by_array2, sort_order2], ...)`: each sort order lifts
/// over an array and a range (`SORTBY(A1:A3,B1:B3,{1;-1})` is 2x1, probes/m7-argument-modes
/// F26:H26), the array and the by arrays are read once by the function. Unless every entry is
/// equal only the first element keeps its call's top-left (`CallResult::ArrayFirst`;
/// `SORTBY(A1:A3,C1:C3,{1;-1;1})` is {1;0;0}, probes/sort-error-args H97-H99), `#VALUE!` once the
/// call has a second pair (probes/m7-lift-take-drop-sortby sheet Sort). Each element sorts the
/// whole array: read and charged as [`TOCOL`].
const SORTBY: ArgKinds = ArgKinds {
    required: 2,
    params: &[WholeByCaller, WholeByCaller, Lift],
    repeat: &[WholeByCaller, Lift],
    max: 255,
    work: Output { weight: 1 },
    result: CallResult::ArrayFirst,
};
// --- End M7 (T-1839) ---
// --- M7 (T-2319) ---
/// `SORT(array, [sort_index], [sort_order], [by_col])`: `by_col` lifts over an array
/// (`SORT(Data!D6:E7,3,1e-15,<2x4 booleans>)` is a 2x4 `#VALUE!`, gen-m7/7307132bb8820f17 A1076);
/// the array and the `sort_index` and `sort_order` lists are read once by the function. Each element
/// sorts the whole array: read and charged as [`TOCOL`]. Later elements are not
/// empty (`.array()`, not SORTBY's `ArrayFirst`: probes/t2366-sort-lifted-by-col); a TRUE element past the first column with
/// `sort_index` 1 is `#VALUE!` in Excel, not the call's top-left (T-2922).
const SORT: ArgKinds = ArgKinds::fixed(
    1,
    &[WholeByCaller, WholeByCaller, WholeByCaller, Lift],
    Output { weight: 1 },
)
.array();
// --- End M7 (T-2319) ---
// --- M7 (T-1835) ---
/// `EXPAND(array, rows, [columns], [pad_with])`: rows and columns lift over an array and a range
/// (`ROWS(EXPAND(A1:A2,{3;4}))` is 2, probes/m7-argument-modes F7:H7; columns,
/// probes/m7-lift-take-drop-sortby row 20), `pad_with` does not: an array there is `#VALUE!` (same
/// case row 21). The array is read once by the function and handed to every element whole; each
/// element walks it (`LiftCost::Cells`) and is charged the cells it builds.
const EXPAND: ArgKinds =
    ArgKinds::fixed(2, &[WholeByCaller, Lift, Lift, Whole], Output { weight: 1 }).array();
/// `WRAPROWS(vector, wrap_count, [pad_with])` and WRAPCOLS: `wrap_count` and `pad_with` lift over an
/// array and a range (`ROWS(WRAPROWS(A1:A3,{1;2}))` is 2, probes/m7-argument-modes F8:H8, G42;
/// probes/m7-lift-take-drop-sortby rows 22-23). Read and charged as [`EXPAND`].
const WRAP: ArgKinds =
    ArgKinds::fixed(2, &[WholeByCaller, Lift, Lift], Output { weight: 1 }).array();
// --- End M7 (T-1835) ---
// --- End array-valued (T-1834) ---------------------------------------------------------------

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
        // The dates lift over an array and refuse a range (probes/date-argument-modes F/G 35-40,
        // 57-62), the weekend lifts over both (F/G 41, 42, 63, 64), and the holidays take the
        // whole range (dates.md "WORKDAY holidays take the whole range"). An element costs
        // O(log holidays): WORKDAY's search makes at most about 44 counts over the date range,
        // and NETWORKDAYS counts in closed form (`functions/date_and_time/holidays.rs`).
        Function::Workday | Function::Networkdays => Some(&WORKDAYS),
        Function::WorkdayIntl | Function::NetworkdaysIntl => Some(&WORKDAYS_INTL),
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
        Function::Sequence => Some(&SEQUENCE),
        Function::Makearray => Some(&MAKEARRAY),
        Function::Take | Function::Drop => Some(&TAKE_DROP),
        Function::Expand => Some(&EXPAND),
        Function::Wrapcols | Function::Wraprows => Some(&WRAP),
        Function::Tocol | Function::Torow => Some(&TOCOL),
        Function::Unique => Some(&UNIQUE),
        Function::Sortby => Some(&SORTBY),
        Function::Sort => Some(&SORT),
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
        // --- Date and time lifts, MROUND, SQRTPI (T-1785) ---
        // The worksheet date and time functions lift over arrays and ranges in every
        // argument, on the range's own rows too (probes/date-argument-modes; docs/excel-
        // semantics/dates.md "Date function argument modes"), DATEDIF's unit and DAYS360's
        // method included (probes/date-math-lift-driver-edges G1-G10, G40-G41). A 3-D
        // reference is `#REF!` (G24-G26, G30-G32). Each is O(1).
        Function::Day
        | Function::Month
        | Function::Year
        | Function::Hour
        | Function::Minute
        | Function::Second
        | Function::Isoweeknum => Some(&LIFT_1),
        Function::Weekday => Some(&LIFT_1_OF_2),
        Function::Days => Some(&LIFT_2),
        Function::Days360 => Some(&LIFT_2_OF_3),
        Function::Date | Function::Time | Function::Datedif => Some(&LIFT_3),
        // Analysis ToolPak functions: they lift over an array and refuse a multi-cell range,
        // also one a function returns, and a 3-D reference (probes/ceiling-floor-mround-
        // array-lift A21, A43:A49, probes/sqrtpi-range-operands, date-math-lift-driver-edges
        // G19-G23, G27-G28; numbers.md "RADIANS, SQRTPI and MUNIT").
        Function::Mround => Some(&ATP_2),
        Function::Sqrtpi => Some(&ARRAYS_1),
        // --- End date and time lifts, MROUND, SQRTPI (T-1785) ---
        // --- LOG, DECIMAL, BASE, ATAN2, ARABIC (T-2159) ---
        // Every argument lifts over an array constant and over a multi-cell range given
        // directly, on rows off the range too (probes/t2198-m5-lift-argument-kinds rows 10-27,
        // 36; docs/excel-semantics/arrays.md "LOG, DECIMAL, BASE, ATAN2, ARABIC, DOLLAR, FIXED and
        // NUMBERVALUE return arrays over array arguments"). LOG and ATAN2 are O(1); DECIMAL reads
        // at most 255 units, BASE writes at most 64 digits and 255 padding, ARABIC makes one pass
        // over its text (`lift/tests/kernel_work.rs`).
        Function::Log => Some(&LIFT_1_OF_2),
        Function::Decimal | Function::Atan2 => Some(&LIFT_2),
        Function::Combin | Function::Combina => Some(&COMBIN_FAMILY),
        Function::Base => Some(&LIFT_2_OF_3),
        Function::Arabic => Some(&LIFT_1),
        Function::Roman => Some(&LIFT_1_OF_2),
        // --- End LOG, DECIMAL, BASE, ATAN2, ARABIC (T-2159) ---
        _ => None,
    }
}
