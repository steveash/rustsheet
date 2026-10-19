//! The argument-kind table (SPEC-0011 "The argument-kind table"): for each parameter of each
//! function the driver lifts, how it takes an array or a range. Static analysis (the legacy `@`
//! and the static shape of a call) is derived from the same table, so it is the single source of
//! truth: a function with an entry has no hand-written arm in static analysis.
//!
//! The table is split by family, each file with its weight constants and their measurements next
//! to its entries: `general.rs` (dates, information, lookup, math), `financial.rs`,
//! `statistical.rs`, `engineering.rs` and `text.rs`. The argument shapes several families share
//! are here.

mod engineering;
mod financial;
mod general;
mod statistical;
mod text;

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
    /// A value-dependent kernel names that cap in `lift/tests/kernel_work.rs` (T-1893).
    Constant,
    /// A loop whose length depends on argument values. One call is charged `weight` units
    /// against the visit cap, and the kernel also keeps its own per-call cap, as T-1197 bounds
    /// MULTINOMIAL's loop.
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

use ArgKind::{Lift, LiftArrays};
use KernelWork::Constant;

// The argument shapes more than one family's entries use. The shapes, here and in the family files,
// are named by kind and (required, total) argument counts: `LIFT_4_OF_6` is four required and two
// optional arguments, every one `Lift`; `ATP_*` are the `LiftArrays` shapes.

/// Two Analysis `ToolPak` arguments, both required: `EDATE(start_date, months)`,
/// `EOMONTH(start_date, months)`, EFFECT, NOMINAL, DOLLARDE and DOLLARFR.
const ATP_2: ArgKinds = ArgKinds::fixed(2, &[LiftArrays, LiftArrays], Constant);
/// One required Analysis `ToolPak` argument and one optional:
/// `WEEKNUM(serial_number, [return_type])`.
const ATP_1_OF_2: ArgKinds = ArgKinds::fixed(1, &[LiftArrays, LiftArrays], Constant);
/// Two required Analysis `ToolPak` arguments and one optional:
/// `YEARFRAC(start_date, end_date, [basis])`.
const ATP_2_OF_3: ArgKinds = ArgKinds::fixed(2, &[LiftArrays, LiftArrays, LiftArrays], Constant);
/// Three required Analysis `ToolPak` arguments: `CONVERT(number, from_unit, to_unit)` and the
/// T-bills (`TBILLEQ(settlement, maturity, discount)`, TBILLPRICE, TBILLYIELD).
const ATP_3: ArgKinds = ArgKinds::fixed(3, &[LiftArrays, LiftArrays, LiftArrays], Constant);

/// One required argument that lifts over an array and a range: the IS* type tests, ERROR.TYPE,
/// ENCODEURL and the one-number math functions (ABS, SIN, LN, ...).
const LIFT_1: ArgKinds = ArgKinds::fixed(1, &[Lift], Constant);
/// One argument, lifted over arrays only: ISEVEN, ISODD and the one-argument Analysis `ToolPak`
/// functions (BIN2DEC, HEX2DEC, OCT2DEC, ERF.PRECISE, ERFC, ERFC.PRECISE, the one-argument IM*).
const ARRAYS_1: ArgKinds = ArgKinds::fixed(1, &[LiftArrays], Constant);
/// `HYPERLINK(link_location, [friendly_name])`: both arguments lift. So do TRUNC,
/// CEILING.PRECISE, ISO.CEILING and FLOOR.PRECISE.
const LIFT_1_OF_2: ArgKinds = ArgKinds::fixed(1, &[Lift, Lift], Constant);

/// Three or four worksheet arguments that lift over arrays and ranges: RRI, PDURATION, SLN;
/// ISPMT and SYD.
const LIFT_3: ArgKinds = ArgKinds::fixed(3, &[Lift, Lift, Lift], Constant);
/// `DAYS360(start_date, end_date, [method])`, FIND and SEARCH.
const LIFT_2_OF_3: ArgKinds = ArgKinds::fixed(2, &[Lift, Lift, Lift], Constant);
/// `BINOM.DIST.RANGE(trials, probability_s, number_s, [number_s2])` and SUBSTITUTE.
const LIFT_3_OF_4: ArgKinds = ArgKinds::fixed(3, &[Lift, Lift, Lift, Lift], Constant);
const LIFT_4: ArgKinds = ArgKinds::fixed(4, &[Lift, Lift, Lift, Lift], Constant);
/// `PMT(rate, nper, pv, [fv], [type])` and PV, FV, NPER alike.
const LIFT_3_OF_5: ArgKinds = ArgKinds::fixed(3, &[Lift, Lift, Lift, Lift, Lift], Constant);
/// `IPMT(rate, per, nper, pv, [fv], [type])` and PPMT.
const LIFT_4_OF_6: ArgKinds = ArgKinds::fixed(4, &[Lift, Lift, Lift, Lift, Lift, Lift], Constant);
/// Two worksheet arguments that lift over an array and a range: BITAND, BITOR, BITXOR,
/// BITLSHIFT, BITRSHIFT, the ROUND family, MOD, FLOOR and CEILING.
const LIFT_2: ArgKinds = ArgKinds::fixed(2, &[Lift, Lift], Constant);

impl Function {
    /// The lifting table: `None` for a function the driver does not lift (yet). Each family's
    /// entries are disjoint (`no_function_is_in_two_families`).
    pub(crate) fn arg_kinds(&self) -> Option<&'static ArgKinds> {
        general::arg_kinds(self)
            .or_else(|| financial::arg_kinds(self))
            .or_else(|| statistical::arg_kinds(self))
            .or_else(|| engineering::arg_kinds(self))
            .or_else(|| text::arg_kinds(self))
    }
}

#[cfg(test)]
mod tests {
    use super::{engineering, financial, general, statistical, text};
    use crate::functions::Function;

    /// One `match` caught a function entered twice as an unreachable pattern; across the
    /// family files, this test does.
    #[test]
    fn no_function_is_in_two_families() {
        for function in Function::into_iter() {
            let families = [
                general::arg_kinds(&function),
                financial::arg_kinds(&function),
                statistical::arg_kinds(&function),
                engineering::arg_kinds(&function),
                text::arg_kinds(&function),
            ];
            let entered = families.iter().filter(|kinds| kinds.is_some()).count();
            assert!(entered <= 1, "{function:?} is in {entered} families");
        }
    }
}
