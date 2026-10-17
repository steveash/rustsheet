//! The argument-kind table (SPEC-0011 "The argument-kind table"): for each parameter of each
//! function the driver lifts, how it takes an array or a range. Static analysis (the legacy `@`
//! and the static shape of a call) is derived from the same table, so it is the single source of
//! truth: a function with an entry has no hand-written arm in static analysis.

use crate::functions::Function;

/// How one parameter takes an array or a range (SPEC-0011).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ArgKind {
    /// Lifts over an array and over a multi-cell range or reference (PMT, NORM.DIST, BITAND, DATE).
    Lift,
    /// Lifts over an array. A multi-cell range or reference is `#VALUE!` in its position, with no
    /// intersection (the Analysis `ToolPak` functions: EDATE, EFFECT, BESSELJ, MROUND).
    LiftArrays,
    /// Lifts over an array; a range is handed to the kernel whole, as data (the D-functions'
    /// `criteria`, probes/db-argument-modes F11:F23; entered by T-1389).
    #[allow(dead_code)] // TODO(T-1389): entered with the D-functions.
    LiftArraysRangeWhole,
    /// Read once as written (`evaluate_node_with_reference`) and handed to every element whole
    /// (LARGE's array, holidays, LOOKUP's vectors).
    #[allow(dead_code)] // TODO(T-1236): WORKDAY's holidays are the first `Whole` entry.
    Whole,
    /// Read by the caller before the driver runs (an aggregate walk, a prepared search vector),
    /// and captured by the kernel. The driver does not evaluate it, and its slot is `EmptyArg`.
    WholeByCaller,
}

/// What one kernel call may cost, beyond reading its arguments (SPEC-0011 "Security rule for
/// kernel work").
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum KernelWork {
    /// O(1), or a loop with a fixed iteration cap that does not depend on argument values.
    Constant,
    /// A loop whose length depends on argument values. One call is charged `weight` units
    /// against the visit cap, and the kernel also keeps its own per-call cap, as T-1197 bounds
    /// MULTINOMIAL's loop.
    #[allow(dead_code)]
    // TODO(T-1290): BINOM.DIST, POISSON and the other loops enter weighted.
    Weighted(u32),
}

impl KernelWork {
    /// The units one kernel call is charged.
    pub(crate) fn weight(self) -> u64 {
        match self {
            KernelWork::Constant => 1,
            KernelWork::Weighted(weight) => u64::from(weight),
        }
    }
}

/// The parameters of one function: `params[i]` is the kind of argument `i`. Arguments past
/// `params` take `repeat`, and the call is valid with `required..=max` arguments.
pub(crate) struct ArgKinds {
    pub(crate) required: usize,
    pub(crate) params: &'static [ArgKind],
    pub(crate) repeat: Option<ArgKind>,
    pub(crate) max: usize,
    pub(crate) work: KernelWork,
}

impl ArgKinds {
    /// `required` arguments, then optional ones up to `params.len()`, with no repeat.
    pub(crate) const fn fixed(
        required: usize,
        params: &'static [ArgKind],
        work: KernelWork,
    ) -> Self {
        Self {
            required,
            params,
            repeat: None,
            max: params.len(),
            work,
        }
    }

    /// The kind of argument `index`, `None` past the last valid argument.
    pub(crate) fn kind(&self, index: usize) -> Option<ArgKind> {
        if index >= self.max {
            return None;
        }
        self.params.get(index).copied().or(self.repeat)
    }

    /// Whether a call with `count` arguments is valid.
    pub(crate) fn accepts(&self, count: usize) -> bool {
        (self.required..=self.max).contains(&count)
    }
}

use ArgKind::{Lift, LiftArrays, WholeByCaller};
use KernelWork::Constant;

/// Two Analysis `ToolPak` arguments, both required: `EDATE(start_date, months)` and
/// `EOMONTH(start_date, months)`.
const ATP_2: ArgKinds = ArgKinds::fixed(2, &[LiftArrays, LiftArrays], Constant);
/// One required Analysis `ToolPak` argument and one optional:
/// `WEEKNUM(serial_number, [return_type])`.
const ATP_1_OF_2: ArgKinds = ArgKinds::fixed(1, &[LiftArrays, LiftArrays], Constant);
/// Two required Analysis `ToolPak` arguments and one optional:
/// `YEARFRAC(start_date, end_date, [basis])`.
const ATP_2_OF_3: ArgKinds = ArgKinds::fixed(2, &[LiftArrays, LiftArrays, LiftArrays], Constant);

/// A data argument its function reads itself, then one lifted worksheet argument:
/// `LARGE(array, k)`, `PERCENTILE(array, k)`, `QUARTILE(array, quart)`, `TRIMMEAN(array, percent)`.
const DATA_THEN_1: ArgKinds = ArgKinds::fixed(2, &[WholeByCaller, Lift], Constant);
/// The same with an optional second lifted argument: `PERCENTRANK(array, x, [significance])`.
const DATA_THEN_1_OF_2: ArgKinds = ArgKinds::fixed(2, &[WholeByCaller, Lift, Lift], Constant);

/// One required argument that lifts over an array and a range: the IS* type tests, ERROR.TYPE
/// and ENCODEURL.
const LIFT_1: ArgKinds = ArgKinds::fixed(1, &[Lift], Constant);
/// ISEVEN and ISODD: one argument, lifted over arrays only.
const ARRAYS_1: ArgKinds = ArgKinds::fixed(1, &[LiftArrays], Constant);
/// `ADDRESS(row_num, column_num, [abs_num], [a1], [sheet_text])`: every argument lifts.
const ADDRESS: ArgKinds = ArgKinds::fixed(2, &[Lift, Lift, Lift, Lift, Lift], Constant);
/// `HYPERLINK(link_location, [friendly_name])`: both arguments lift.
const LIFT_1_OF_2: ArgKinds = ArgKinds::fixed(1, &[Lift, Lift], Constant);

/// The functions of [`Function::arg_kinds`], as a pattern: the arm of an exhaustive `match` on a
/// `Function` that has no hand-written arm because the table derives it (static analysis).
/// Keep it equal to the table; `static_analysis::lifted`'s test checks that no function in the
/// table has a fallback arm.
macro_rules! lifting_table_functions {
    () => {
        Function::Edate
            | Function::Eomonth
            | Function::Weeknum
            | Function::Yearfrac
            | Function::Isnumber
            | Function::Istext
            | Function::Isnontext
            | Function::Islogical
            | Function::Isblank
            | Function::Iserror
            | Function::Iserr
            | Function::Isna
            | Function::ErrorType
            | Function::Encodeurl
            | Function::Iseven
            | Function::Isodd
            | Function::Large
            | Function::Small
            | Function::PercentileCompat
            | Function::PercentileInc
            | Function::PercentileExc
            | Function::PercentrankCompat
            | Function::PercentrankInc
            | Function::PercentrankExc
            | Function::QuartileCompat
            | Function::QuartileInc
            | Function::QuartileExc
            | Function::Trimmean
            | Function::Address
            | Function::Hyperlink
    };
}
pub(crate) use lifting_table_functions;

impl Function {
    /// The lifting table: `None` for a function the driver does not lift (yet).
    pub(crate) fn arg_kinds(&self) -> Option<&'static ArgKinds> {
        match self {
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
            // The scalar arguments of these worksheet functions lift over an array and over a
            // multi-cell range, also PERCENTRANK's significance; each element is the scalar call
            // (probes/stat-argument-modes H345-H463, H549-H550; probes/stat-whole-lift W1-W48;
            // statistics.md "Statistical function argument modes"). The data argument is read by
            // the function's own reader (`statistical/sorted_data.rs`) and sorted once per call.
            // A kernel looks up its sorted numbers in O(1) or by binary search (PERCENTRANK); the
            // call pays for the numbers through its `LiftCost`.
            Function::Large
            | Function::Small
            | Function::PercentileCompat
            | Function::PercentileInc
            | Function::PercentileExc
            | Function::QuartileCompat
            | Function::QuartileInc
            | Function::QuartileExc
            | Function::Trimmean => Some(&DATA_THEN_1),
            Function::PercentrankCompat | Function::PercentrankInc | Function::PercentrankExc => {
                Some(&DATA_THEN_1_OF_2)
            }
            // ADDRESS's row_num, column_num and abs_num, and HYPERLINK's link_location and
            // friendly_name, lift over a range and an array (probes/lookup-argument-modes F1:G4,
            // F11:G13; docs/excel-semantics/lookup.md "Lookup and reference function argument
            // modes"); so do ADDRESS's a1 (gen-m7/f68fafa0ca2eba26 A7) and HYPERLINK's
            // friendly_name over an array (gen-m7/2bb922eef72e785e A22). ADDRESS's sheet_text is
            // unprobed and follows SPEC-0011's rule for scalar parameters.
            Function::Address => Some(&ADDRESS),
            Function::Hyperlink => Some(&LIFT_1_OF_2),
            _ => None,
        }
    }
}
