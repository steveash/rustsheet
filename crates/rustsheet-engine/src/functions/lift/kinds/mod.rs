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

use crate::calc_result::CalcResult;
use crate::constants::MAXIMUM_DATE_SERIAL_NUMBER;
use crate::expressions::parser::ArrayNode;
use crate::formatter::dates::FINANCIAL_EPOCH_1904_IN_1900_SERIALS;
use crate::functions::Function;
use crate::limits::COUNT_LIMIT_EXCLUSIVE;

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
    /// A loop whose length the element's own argument values predict in O(1) (T-1819, T-1846):
    /// each element is charged [`ValueWork::units`] of its values before anything runs, summed
    /// over the broadcast (`lift/driver.rs` `kernel_units`).
    ByValue(ValueWork),
}

/// How a [`KernelWork::ByValue`] kernel's work follows from one element's argument values. Every
/// estimate is at least 1 and at most [`ValueWork::most`], so a lift charged by value never costs
/// more than the flat worst case it replaces; a text search has no such case, as its texts have no
/// fixed length.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ValueWork {
    /// A loop whose length is the value of argument `arg`: the Bessel recurrences run about once
    /// or twice per order, up to the kernel's own per-call cap `cap` (T-1819). An element is
    /// charged `1 + trunc(n) * weight / cap` units for a number with `0 <= trunc(n) <= cap`, so
    /// `1 + weight` at the cap; a text is charged as at the cap (the kernel may read it as a
    /// number), and anything else (an order past the cap or negative, an error, a logical, a
    /// blank) 1, as the kernel refuses it before looping or runs order 0.
    Loop { arg: usize, cap: u32, weight: u32 },
    /// `walks` passes over a coupon schedule that steps one period at a time between the earliest
    /// and the latest of the date arguments `dates`, `frequency` periods a year (T-1846; T-1869
    /// for the solvers, whose one pass is the walk and every price sum). A pass is charged
    /// `weight` units at the longest schedule (quarterly across the whole calendar,
    /// [`COUPON_SPAN_DAYS`]) and in proportion to its period count plus one below that; see
    /// [`coupon_walk_units`].
    CouponWalks {
        dates: &'static [usize],
        frequency: usize,
        walks: u32,
        weight: u32,
    },
    /// A search for argument `find`'s text in argument `within`'s (FIND and SEARCH, T-1961):
    /// both texts are read once, and each start position of `within` compares at most the
    /// shorter text's length with `find`, `per_visit` compared bytes costing one visit beside the
    /// position's own; see [`text_scan_units`].
    TextScan {
        find: usize,
        within: usize,
        per_visit: u32,
    },
    /// HYPGEOM.DIST's cdf (T-2433): either a walk of at most `walk` units, or a sum over a window
    /// of at most the least of the counts `counts` (each truncated) plus one terms, and up to
    /// `extra` more exponents for the window's bounds, at `per_term` units a term; never more
    /// than `weight`, the bound of the sum's own per-call cap. A text count bounds nothing (the
    /// kernel may read it as any count), so the least of the others is taken, `weight` when
    /// every count is a text; an error, a negative count or one the kernel
    /// refuses as too large ([`COUNT_LIMIT_EXCLUSIVE`]) 1, as the kernel returns before either
    /// path. See [`hypergeometric_units`].
    HypergeometricSum {
        counts: &'static [usize],
        walk: u32,
        extra: u32,
        per_term: u32,
        weight: u32,
    },
}

/// The widest span of days a coupon schedule walk can cross: the calendar's last serial, as the
/// kernels read every date in the 1900 financial space from serial 0 to 9999-12-31 (a 1904
/// serial is shifted by the same 2 * 1462 days at both ends, `DateSystem::financial_1900_serial`).
pub(crate) const COUPON_SPAN_DAYS: u64 = MAXIMUM_DATE_SERIAL_NUMBER.unsigned_abs() as u64;

/// What a [`KernelWork::ByValue`] argument's value is, for its charge.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum ArgValue {
    Number(f64),
    Boolean(bool),
    /// A text of this many UTF-8 bytes, at least its UTF-16 units.
    Text(usize),
    /// A blank cell or an omitted argument: the kernels read it as 0.
    Blank,
    Error,
}

impl ArgValue {
    pub(crate) fn of(value: &CalcResult) -> Self {
        match value {
            CalcResult::Number(n) => ArgValue::Number(*n),
            CalcResult::Boolean(b) => ArgValue::Boolean(*b),
            CalcResult::String(s) => ArgValue::Text(s.len()),
            CalcResult::EmptyCell | CalcResult::EmptyArg => ArgValue::Blank,
            _ => ArgValue::Error,
        }
    }

    /// The whole number a coupon kernel reads: a number floored, a logical as 1 or 0 (ACCRINT
    /// reads one; the others refuse it, so this over-charges them).
    fn whole_number(self) -> Option<f64> {
        match self {
            ArgValue::Number(n) => Some(n.floor()),
            ArgValue::Boolean(b) => Some(f64::from(u8::from(b))),
            ArgValue::Text(_) | ArgValue::Blank | ArgValue::Error => None,
        }
    }

    pub(crate) fn of_node(node: &ArrayNode) -> Self {
        match node {
            ArrayNode::Number(n) => ArgValue::Number(*n),
            ArrayNode::Boolean(b) => ArgValue::Boolean(*b),
            ArrayNode::String(s) => ArgValue::Text(s.len()),
            ArrayNode::Empty => ArgValue::Blank,
            ArrayNode::Error(_) | ArrayNode::Lambda(_) | ArrayNode::Nested(_) => ArgValue::Error,
        }
    }
}

impl ValueWork {
    /// The most one element is charged.
    pub(crate) fn most(self) -> u64 {
        match self {
            // An argument at the cap.
            ValueWork::Loop { weight, .. } => u64::from(weight).saturating_add(1),
            // Every walk at the longest schedule.
            ValueWork::CouponWalks { walks, weight, .. } => {
                u64::from(walks).saturating_mul(u64::from(weight)).max(1)
            }
            // A text has no fixed length.
            ValueWork::TextScan { .. } => u64::MAX,
            // The sum at its cap.
            ValueWork::HypergeometricSum { weight, .. } => u64::from(weight),
        }
    }

    /// The units one element is charged, from its argument values (`value(i)` is argument `i`'s,
    /// `None` past the arguments given).
    pub(crate) fn units(self, value: impl Fn(usize) -> Option<ArgValue>) -> u64 {
        match self {
            ValueWork::Loop { arg, cap, weight } => {
                let order = match value(arg) {
                    // Past the cap the kernel refuses the order before it loops.
                    Some(ArgValue::Number(n)) if (0.0..=f64::from(cap)).contains(&n.trunc()) => {
                        n.trunc()
                    }
                    Some(ArgValue::Text(_)) => f64::from(cap),
                    _ => 0.0,
                };
                // `order` is a whole number in [0, cap], so the cast is exact.
                let order = order as u64;
                1 + order.saturating_mul(u64::from(weight)) / u64::from(cap.max(1))
            }
            ValueWork::CouponWalks {
                dates,
                frequency,
                walks,
                weight,
            } => {
                // An invalid frequency is #NUM! and an error argument its error, both before any
                // walk (`Model::coupon_args`, `Model::bond_args`, `Model::accrint_of`). A logical
                // is charged as its number: ACCRINT reads it as one.
                let per_year = match value(frequency) {
                    // A text may read as any frequency.
                    Some(ArgValue::Text(_)) => 4.0,
                    Some(arg) => match arg.whole_number() {
                        Some(n) => n,
                        None => return 1,
                    },
                    None => return 1,
                };
                if !matches!(per_year as u64, 1 | 2 | 4) {
                    return 1;
                }
                let mut earliest = f64::INFINITY;
                let mut latest = f64::NEG_INFINITY;
                for &arg in dates {
                    let date = match value(arg) {
                        Some(ArgValue::Blank) | None => 0.0,
                        // A text may read as any date.
                        Some(ArgValue::Text(_)) => return self.most(),
                        Some(ArgValue::Error) => return 1,
                        Some(arg) => arg.whole_number().unwrap_or(0.0),
                    };
                    earliest = earliest.min(date);
                    latest = latest.max(date);
                }
                // A 1904 workbook reads a date 2 * 1462 days on, but not a negative one
                // (`DateSystem::financial_1900_serial`, `odd_period_1900_serial`): a span from a
                // negative date may be that much wider.
                let shift = if earliest < 0.0 {
                    FINANCIAL_EPOCH_1904_IN_1900_SERIALS as f64
                } else {
                    0.0
                };
                let span = (latest - earliest + shift).clamp(0.0, COUPON_SPAN_DAYS as f64);
                // `span` is a whole number in [0, COUPON_SPAN_DAYS], so the cast is exact.
                let walk = coupon_walk_units(span as u64, per_year as u64, weight);
                u64::from(walks).saturating_mul(walk).max(1)
            }
            ValueWork::TextScan {
                find,
                within,
                per_visit,
            } => text_scan_units(
                text_scan_length(value(find)),
                text_scan_length(value(within)),
                per_visit,
            ),
            ValueWork::HypergeometricSum {
                counts,
                walk,
                extra,
                per_term,
                weight,
            } => {
                let mut least = f64::INFINITY;
                for &arg in counts {
                    let count = match value(arg) {
                        Some(ArgValue::Blank) | None => 0.0,
                        // A text may read as any count, so it bounds nothing: the least of
                        // the other counts still bounds the window.
                        Some(ArgValue::Text(_)) => continue,
                        Some(ArgValue::Error) => return 1,
                        Some(ArgValue::Number(n)) => n,
                        Some(ArgValue::Boolean(b)) => f64::from(u8::from(b)),
                    };
                    // The kernel's #NUM! before either path (hypegeom.rs `hyp_geom_dist_of`).
                    if !(0.0..COUNT_LIMIT_EXCLUSIVE).contains(&count) {
                        return 1;
                    }
                    least = least.min(count.trunc());
                }
                // `least` is a whole number in [0, 2^31), or infinite when every count is a
                // text: the sum at its cap.
                let terms = if least.is_finite() {
                    least as u64
                } else {
                    u64::MAX
                };
                hypergeometric_units(terms, walk, extra, per_term, weight)
            }
        }
    }
}

/// The units one HYPGEOM.DIST element is charged when the least of its counts is `least`: the
/// larger of its two paths, the walk's `walk` units and the sum's `least + 1 + extra` terms at
/// `per_term` units, at most `weight`. Saturating.
fn hypergeometric_units(least: u64, walk: u32, extra: u32, per_term: u32, weight: u32) -> u64 {
    let sum = least
        .saturating_add(1)
        .saturating_add(u64::from(extra))
        .saturating_mul(u64::from(per_term));
    sum.max(u64::from(walk)).min(u64::from(weight))
}

/// The most UTF-8 bytes a number or a logical reads as when a text function takes it as text:
/// the General form of a number is at most 15 digits with a sign, a point and a 4-character
/// exponent, `FALSE` is 5.
const NUMBER_TEXT_BYTES: u64 = 32;

/// The length a [`ValueWork::TextScan`] charges for an argument: a text's bytes, a number's or a
/// logical's longest text, 0 for a blank (the empty text) and for an error, which the kernel
/// returns before it scans.
fn text_scan_length(value: Option<ArgValue>) -> u64 {
    match value {
        Some(ArgValue::Text(bytes)) => u64::try_from(bytes).unwrap_or(u64::MAX),
        Some(ArgValue::Number(_) | ArgValue::Boolean(_)) => NUMBER_TEXT_BYTES,
        Some(ArgValue::Blank | ArgValue::Error) | None => 0,
    }
}

/// The units one search for a `find`-byte text in a `within`-byte one is charged: each text's
/// length plus 1 for reading it, and at each of `within + 1` start positions 1 plus the bytes
/// compared there (at most the shorter length) over `per_visit`. Saturating.
/// TEXTSPLIT, TEXTBEFORE and TEXTAFTER charge a whole call by the same shape (`ScanCost` in
/// functions/text/lifted.rs), but over a delimiter list read whole; this one is per element.
fn text_scan_units(find: u64, within: u64, per_visit: u32) -> u64 {
    let compared = find.min(within) / u64::from(per_visit.max(1));
    find.saturating_add(1).saturating_add(
        within
            .saturating_add(1)
            .saturating_mul(compared.saturating_add(1)),
    )
}

/// The units one coupon schedule walk across `span` days at `per_year` (1, 2 or 4) periods a
/// year is charged: `weight` for the longest walk, a quarterly schedule across
/// [`COUPON_SPAN_DAYS`], and in proportion to the period count `span * per_year / 365` plus one
/// below it, rounded up. The extra period is each pass's fixed cost: a solver price over a
/// one-coupon schedule costs about as much as one more coupon term (T-1869: 20,000 two-coupon
/// YIELDs at the solver's extremes took 12 ms, four-coupon ones 223 ms; a release build).
pub(crate) fn coupon_walk_units(span: u64, per_year: u64, weight: u32) -> u64 {
    let longest = COUPON_SPAN_DAYS * 4 + 365;
    // At most 2^24 * (2958465 * 4 + 365), far inside a u64.
    let scaled = u64::from(weight) * (span.min(COUPON_SPAN_DAYS) * per_year.min(4) + 365);
    scaled.div_ceil(longest)
}

impl KernelWork {
    /// The units one kernel call is charged before it runs (for `ByValue`, the most it can be:
    /// `u64::MAX` for a `TextScan`, which `kernel_units` charges by value instead).
    pub(crate) fn weight(self) -> u64 {
        match self {
            KernelWork::Constant | KernelWork::Output { .. } => 1,
            KernelWork::Weighted(weight) => u64::from(weight),
            KernelWork::ByValue(work) => work.most(),
        }
    }

    /// The units a lifted element is charged after its call, for a result of `cells` cells: 0
    /// unless the kernel builds an array.
    pub(crate) fn output_charge(self, cells: u64) -> u64 {
        match self {
            KernelWork::Output { weight } => cells.saturating_mul(u64::from(weight)),
            KernelWork::Constant | KernelWork::Weighted(_) | KernelWork::ByValue(_) => 0,
        }
    }
}

/// What one unlifted call returns (SPEC-0011 "Array-valued functions" rule 1), and so what a
/// lifted element keeps (rule 2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CallResult {
    /// One value: every entry before T-1834.
    Value,
    /// An array (SEQUENCE, MAKEARRAY): unlifted, the static shape is `Unknown`, and a lifted
    /// element is the top-left value of its call.
    Array,
    /// An array as [`CallResult::Array`], but unless every element's lifted entries are equal,
    /// only the first element whose call is not an error keeps its top-left value: an error
    /// stays, and every other element is empty (UNIQUE; probes/m7-unique-lift-elements,
    /// docs/excel-semantics/arrays.md "UNIQUE's and SORTBY's lifted elements"; T-2300). The
    /// driver applies it to computed elements only, so it needs `Reuse::Never`, which every
    /// table entry's options have (`LiftOptions::for_entry`).
    ArrayFirst,
    /// As [`CallResult::ArrayFirst`], and when every entry is equal and spans several columns the
    /// elements past the first column are empty too (UNIQUE; probes/m7-unique-lift-corners,
    /// T-2348). SORTBY's uniform 1xN `sort_order` is unprobed, so it stays on `ArrayFirst`.
    ArrayFirstColumn,
}

impl CallResult {
    /// Whether one unlifted call returns an array.
    pub(crate) fn is_array(self) -> bool {
        matches!(
            self,
            CallResult::Array | CallResult::ArrayFirst | CallResult::ArrayFirstColumn
        )
    }
}

/// The parameters of one function: `params[i]` is the kind of argument `i`. Arguments past
/// `params` take the kinds of `repeat` in turn, a cycle (SORTBY's `by_array`, `sort_order` pairs;
/// SPEC-0011 "Array-valued functions" rule 4), and the call is valid with `required..=max`
/// arguments that end on a whole cycle.
pub(crate) struct ArgKinds {
    pub(crate) required: usize,
    pub(crate) params: &'static [ArgKind],
    /// Empty when nothing repeats.
    pub(crate) repeat: &'static [ArgKind],
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
            repeat: &[],
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

    /// The same kinds for a function whose lifted elements follow [`CallResult::ArrayFirstColumn`].
    pub(crate) const fn array_first_column(self) -> Self {
        Self {
            result: CallResult::ArrayFirstColumn,
            ..self
        }
    }

    /// The kind of argument `index`, `None` past the last valid argument.
    pub(crate) fn kind(&self, index: usize) -> Option<ArgKind> {
        if index >= self.max {
            return None;
        }
        match index.checked_sub(self.params.len()) {
            None => self.params.get(index).copied(),
            Some(past) => self.repeat.get(past % self.repeat.len().max(1)).copied(),
        }
    }

    /// Whether `count` is between the function's least and most arguments, whole `repeat`
    /// cycles aside (see [`ArgKinds::accepts`]).
    fn takes_count(&self, count: usize) -> bool {
        (self.required..=self.max).contains(&count)
    }

    /// Whether a call with `count` arguments is valid: past `params` only whole cycles of
    /// `repeat` (`SORTBY` takes 2, 3, 5, 7, ... arguments).
    pub(crate) fn accepts(&self, count: usize) -> bool {
        let whole_cycles = match count.checked_sub(self.params.len()) {
            None | Some(0) => true,
            // An empty `repeat` takes no argument past `params`.
            Some(past) => !self.repeat.is_empty() && past % self.repeat.len() == 0,
        };
        self.takes_count(count) && whole_cycles
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
/// CEILING.PRECISE, ISO.CEILING, FLOOR.PRECISE and DOLLAR.
const LIFT_1_OF_2: ArgKinds = ArgKinds::fixed(1, &[Lift, Lift], Constant);
/// `CEILING.MATH(number, [significance], [mode])`, FLOOR.MATH, FIXED and NUMBERVALUE.
const LIFT_1_OF_3: ArgKinds = ArgKinds::fixed(1, &[Lift, Lift, Lift], Constant);

/// Three or four worksheet arguments that lift over arrays and ranges: RRI, PDURATION, SLN;
/// ISPMT and SYD.
const LIFT_3: ArgKinds = ArgKinds::fixed(3, &[Lift, Lift, Lift], Constant);
/// `DAYS360(start_date, end_date, [method])` and BASE.
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
