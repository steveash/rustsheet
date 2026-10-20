//! The argument-kind table (SPEC-0011 "The argument-kind table"): for each parameter of each
//! function the driver lifts, how it takes an array or a range. Static analysis (the legacy `@`
//! and the static shape of a call) is derived from the same table, so it is the single source of
//! truth: a function with an entry has no hand-written arm in static analysis.
//!
//! The table is split by family, each file with its weight constants and their measurements next
//! to its entries: `general.rs` (dates, information, lookup, math), `financial.rs`,
//! `statistical.rs` and `engineering.rs`. The argument shapes several families share are here.

mod engineering;
mod financial;
mod general;
mod statistical;

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
    LiftArraysRangeWhole,
    /// Read once as written (`evaluate_node_with_reference`) and handed to every element whole
    /// (LARGE's array, the D-functions' database, LOOKUP's vectors).
    Whole,
    /// Read by the caller before the driver runs (an aggregate walk, a prepared search vector,
    /// WORKDAY's holidays),
    /// and captured by the kernel. The driver does not evaluate it, and its slot is `EmptyArg`.
    WholeByCaller,
    /// Read by the caller, which needs a reference there (RANK's `ref`): a reference is data,
    /// captured by the kernel, and its slot is `EmptyArg`. An array that is not a reference is an
    /// array of errors of its own shape, which the caller hands back to broadcast with the lifted
    /// arguments (`Model::lift_call_beside`; statistics.md "RANK of a non-reference array",
    /// probes/stat-test-lift P28-P31; T-1999). Static analysis gives it that array's shape.
    ReferenceByCaller,
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
    /// The kernel builds an array (SPEC-0011 "Array-valued functions" rule 3): a lifted element
    /// is charged one unit before the call and the cells of its own result after it, times
    /// `weight` (MAKEARRAY's 128 for its LAMBDA call per cell).
    Output { weight: u32 },
    /// A loop whose length is the value of argument `arg`: the Bessel recurrences run about once
    /// or twice per order, up to the kernel's own per-call cap `cap` (T-1819). An element is
    /// charged by that argument's value before anything runs, summed over the broadcast
    /// (`lift/driver.rs` `kernel_units`): `1 + trunc(n) * weight / cap` units for a number with
    /// `0 <= trunc(n) <= cap`, so `1 + weight` at the cap; a text is charged as at the cap (the
    /// kernel may read it as a number), and anything else (an order past the cap or negative, an
    /// error, a logical, a blank) 1, as the kernel refuses it before looping or runs order 0.
    ByArg { arg: usize, cap: u32, weight: u32 },
}

impl KernelWork {
    /// The units one kernel call is charged before it runs (for `ByArg`, the most it can be).
    pub(crate) fn weight(self) -> u64 {
        match self {
            KernelWork::Constant | KernelWork::Output { .. } => 1,
            KernelWork::Weighted(weight) => u64::from(weight),
            // The most an element is charged: its argument at the cap.
            KernelWork::ByArg { weight, .. } => u64::from(weight).saturating_add(1),
        }
    }

    /// The units a lifted element is charged after its call, for a result of `cells` cells: 0
    /// unless the kernel builds an array.
    pub(crate) fn output_charge(self, cells: u64) -> u64 {
        match self {
            KernelWork::Output { weight } => cells.saturating_mul(u64::from(weight)),
            KernelWork::Constant | KernelWork::Weighted(_) | KernelWork::ByArg { .. } => 0,
        }
    }
}

/// What one unlifted call returns (SPEC-0011 "Array-valued functions" rule 1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CallResult {
    /// One value: every entry before T-1834.
    Value,
    /// An array (SEQUENCE, MAKEARRAY): unlifted, the static shape is `Unknown`, and a lifted
    /// element is the top-left value of its call.
    Array,
}

/// The parameters of one function: `params[i]` is the kind of argument `i`. Arguments past
/// `params` take `repeat`, and the call is valid with `required..=max` arguments.
pub(crate) struct ArgKinds {
    pub(crate) required: usize,
    pub(crate) params: &'static [ArgKind],
    pub(crate) repeat: Option<ArgKind>,
    pub(crate) max: usize,
    pub(crate) work: KernelWork,
    pub(crate) result: CallResult,
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
            result: CallResult::Value,
        }
    }

    /// The same kinds for a function whose unlifted call returns an array.
    pub(crate) const fn array(self) -> Self {
        Self {
            result: CallResult::Array,
            ..self
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
    }
}

#[cfg(test)]
mod tests {
    use super::{engineering, financial, general, statistical};
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
            ];
            let entered = families.iter().filter(|kinds| kinds.is_some()).count();
            assert!(entered <= 1, "{function:?} is in {entered} families");
        }
    }
}
