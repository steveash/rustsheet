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
use KernelWork::{Constant, Weighted};

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

/// A data argument its function reads itself, then one lifted worksheet argument:
/// `LARGE(array, k)`, `PERCENTILE(array, k)`, `QUARTILE(array, quart)`, `TRIMMEAN(array, percent)`.
const DATA_THEN_1: ArgKinds = ArgKinds::fixed(2, &[WholeByCaller, Lift], Constant);
/// The same with an optional second lifted argument: `PERCENTRANK(array, x, [significance])`.
const DATA_THEN_1_OF_2: ArgKinds = ArgKinds::fixed(2, &[WholeByCaller, Lift, Lift], Constant);

/// One required argument that lifts over an array and a range: the IS* type tests, ERROR.TYPE,
/// ENCODEURL and the one-number math functions (ABS, SIN, LN, ...).
const LIFT_1: ArgKinds = ArgKinds::fixed(1, &[Lift], Constant);
/// One argument, lifted over arrays only: ISEVEN, ISODD and the one-argument Analysis `ToolPak`
/// functions (BIN2DEC, HEX2DEC, OCT2DEC, ERF.PRECISE, ERFC, ERFC.PRECISE, the one-argument IM*).
const ARRAYS_1: ArgKinds = ArgKinds::fixed(1, &[LiftArrays], Constant);
/// `ADDRESS(row_num, column_num, [abs_num], [a1], [sheet_text])`: every argument lifts.
const ADDRESS: ArgKinds = ArgKinds::fixed(2, &[Lift, Lift, Lift, Lift, Lift], Constant);
/// `HYPERLINK(link_location, [friendly_name])`: both arguments lift. So do TRUNC,
/// CEILING.PRECISE, ISO.CEILING and FLOOR.PRECISE.
const LIFT_1_OF_2: ArgKinds = ArgKinds::fixed(1, &[Lift, Lift], Constant);

// The shapes below are named by kind and (required, total) argument counts: `LIFT_4_OF_6` is four
// required and two optional arguments, every one `Lift`; `ATP_*` are the `LiftArrays` shapes.

/// Three or four worksheet arguments that lift over arrays and ranges: RRI, PDURATION, SLN;
/// ISPMT and SYD.
const LIFT_3: ArgKinds = ArgKinds::fixed(3, &[Lift, Lift, Lift], Constant);
/// `CEILING.MATH(number, [significance], [mode])` and FLOOR.MATH.
const LIFT_1_OF_3: ArgKinds = ArgKinds::fixed(1, &[Lift, Lift, Lift], Constant);
const LIFT_4: ArgKinds = ArgKinds::fixed(4, &[Lift, Lift, Lift, Lift], Constant);
/// `PMT(rate, nper, pv, [fv], [type])` and PV, FV, NPER alike.
const LIFT_3_OF_5: ArgKinds = ArgKinds::fixed(3, &[Lift, Lift, Lift, Lift, Lift], Constant);
/// `IPMT(rate, per, nper, pv, [fv], [type])` and PPMT.
const LIFT_4_OF_6: ArgKinds = ArgKinds::fixed(4, &[Lift, Lift, Lift, Lift, Lift, Lift], Constant);
/// `DDB(cost, salvage, life, period, [factor])`.
const LIFT_4_OF_5: ArgKinds = ArgKinds::fixed(4, &[Lift, Lift, Lift, Lift, Lift], Constant);
/// `RATE(nper, pmt, pv, [fv], [type], [guess])`: Newton's method with a fixed cap of 50 steps
/// (`financial_util::compute_rate`), each element charged that weight (SPEC-0011 security rule).
/// Under the 2^20-element output cap the charge never reaches `MAX_LIFTED_CELL_VISITS`, so it
/// records the work rather than refusing a call: at worst about 2^20 × 50 Newton steps.
const RATE: ArgKinds = ArgKinds::fixed(3, &[Lift, Lift, Lift, Lift, Lift, Lift], Weighted(50));
/// `ACCRINTM(issue, settlement, rate, [par], [basis])`.
const ATP_3_OF_5: ArgKinds = ArgKinds::fixed(
    3,
    &[LiftArrays, LiftArrays, LiftArrays, LiftArrays, LiftArrays],
    Constant,
);
/// `DISC(settlement, maturity, pr, redemption, [basis])` and INTRATE, PRICEDISC, RECEIVED,
/// YIELDDISC alike.
const ATP_4_OF_5: ArgKinds = ArgKinds::fixed(
    4,
    &[LiftArrays, LiftArrays, LiftArrays, LiftArrays, LiftArrays],
    Constant,
);
/// `PRICEMAT(settlement, maturity, issue, rate, yld, [basis])` and YIELDMAT.
const ATP_5_OF_6: ArgKinds = ArgKinds::fixed(
    5,
    &[
        LiftArrays, LiftArrays, LiftArrays, LiftArrays, LiftArrays, LiftArrays,
    ],
    Constant,
);
// --- Engineering (T-1383) -------------------------------------------------------------------
/// Two worksheet arguments that lift over an array and a range: BITAND, BITOR, BITXOR,
/// BITLSHIFT, BITRSHIFT, the ROUND family, MOD, FLOOR and CEILING.
const LIFT_2: ArgKinds = ArgKinds::fixed(2, &[Lift, Lift], Constant);
/// `BESSELI|J|K|Y(x, n)`: the recurrences loop about once (BESSELK, BESSELY) or twice (BESSELI,
/// BESSELJ's Miller recurrence) per order, up to the order cap `transcendental::MAX_ORDER`
/// (100,000), each kernel's own per-call cap. The weight charges a lifted call for that work:
/// at most 2,048 elements, which at the cap run in about 2 s in release (BESSELI, the slowest:
/// 8,192 elements took 8 s; docs/excel-semantics/engineering.md "BESSEL order cap").
const BESSEL: ArgKinds = ArgKinds::fixed(2, &[LiftArrays, LiftArrays], Weighted(BESSEL_WEIGHT));
/// The units one Bessel element is charged against `MAX_LIFTED_CELL_VISITS`.
const BESSEL_WEIGHT: u32 = 1 << 16;
/// `IMSUM(inumber1, ...)` and IMPRODUCT: up to 255 arguments, each read whole by the function's
/// own fold (`Model::fold_complex`), so nothing lifts and the call is one value. The fold never
/// enters the lift driver, so its `KernelWork` is not charged; it caps its own reads.
const COMPLEX_FOLD: ArgKinds = ArgKinds {
    required: 1,
    params: &[],
    repeat: Some(WholeByCaller),
    max: 255,
    work: Constant,
};
// --- End engineering (T-1383) ---------------------------------------------------------------
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

// --- Financial ATP (T-1822) -----------------------------------------------------------------
/// What one walk of a coupon schedule is charged: the walk steps one coupon period at a time
/// from the maturity back to the settlement (or from the first interest date to the issue), at
/// most about 32,400 quarters inside the calendar (a settlement before serial 1 is #NUM! first,
/// `Model::check_settlement`), about 1 ms in a release build (`COUPNUM(1,DATE(9990,6,30),4)`).
const COUPON_WALK_WEIGHT: u32 = 1 << 16;
/// `COUPDAYBS(settlement, maturity, frequency, [basis])` and the other COUP* functions: up to
/// two schedule walks (COUPDAYSNC's basis 0 takes both coupon dates), so a lifted call runs at
/// most 1,024 elements: at the calendar's longest schedule (maturity 9990-06-30, quarterly) that
/// took 1.1 s for COUPNUM and 2.2 s for COUPDAYSNC in a release build (T-1822).
const COUPON: ArgKinds = ArgKinds::fixed(
    3,
    &[LiftArrays, LiftArrays, LiftArrays, LiftArrays],
    Weighted(2 * COUPON_WALK_WEIGHT),
);
/// `ACCRINT(issue, first_interest, settlement, rate, par, frequency, [basis], [calc_method])`:
/// a walk forward from the first interest date to the settlement and one back to the issue
/// (1,024 elements over the longest schedule: 1.0 s).
const ACCRINT: ArgKinds = ArgKinds::fixed(
    6,
    &[
        LiftArrays, LiftArrays, LiftArrays, LiftArrays, LiftArrays, LiftArrays, LiftArrays,
        LiftArrays,
    ],
    Weighted(2 * COUPON_WALK_WEIGHT),
);
/// `DURATION(settlement, maturity, coupon, yld, frequency, [basis])` and MDURATION: three walks
/// (the coupon count and the two day counts) and a sum over every coupon, about 3 ms at the
/// longest schedule, so a lifted call runs at most 512 elements (1.5 s for PRICE or MDURATION).
const BOND_5_OF_6: ArgKinds = ArgKinds::fixed(
    5,
    &[
        LiftArrays, LiftArrays, LiftArrays, LiftArrays, LiftArrays, LiftArrays,
    ],
    Weighted(4 * COUPON_WALK_WEIGHT),
);
/// `PRICE(settlement, maturity, rate, yld, redemption, frequency, [basis])`, as DURATION.
const BOND_6_OF_7: ArgKinds = ArgKinds::fixed(
    6,
    &[
        LiftArrays, LiftArrays, LiftArrays, LiftArrays, LiftArrays, LiftArrays, LiftArrays,
    ],
    Weighted(4 * COUPON_WALK_WEIGHT),
);
// --- End financial ATP (T-1822) -------------------------------------------------------------

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
            | Function::Pmt
            | Function::Pv
            | Function::Fv
            | Function::Nper
            | Function::Rate
            | Function::Ipmt
            | Function::Ppmt
            | Function::Ispmt
            | Function::Rri
            | Function::Pduration
            | Function::Sln
            | Function::Syd
            | Function::Ddb
            | Function::Effect
            | Function::Nominal
            | Function::Dollarde
            | Function::Dollarfr
            | Function::Accrintm
            | Function::Disc
            | Function::Intrate
            | Function::Pricedisc
            | Function::Received
            | Function::Yielddisc
            | Function::Pricemat
            | Function::Yieldmat
            | Function::Tbilleq
            | Function::Tbillprice
            | Function::Tbillyield
            // --- Engineering (T-1383) ---
            | Function::Besseli
            | Function::Besselj
            | Function::Besselk
            | Function::Bessely
            | Function::Erf
            | Function::ErfPrecise
            | Function::Erfc
            | Function::ErfcPrecise
            | Function::Bin2dec
            | Function::Bin2hex
            | Function::Bin2oct
            | Function::Dec2Bin
            | Function::Dec2hex
            | Function::Dec2oct
            | Function::Hex2bin
            | Function::Hex2dec
            | Function::Hex2oct
            | Function::Oct2bin
            | Function::Oct2dec
            | Function::Oct2hex
            | Function::Bitand
            | Function::Bitor
            | Function::Bitxor
            | Function::Bitlshift
            | Function::Bitrshift
            | Function::Complex
            | Function::Convert
            | Function::Delta
            | Function::Gestep
            | Function::Imabs
            | Function::Imaginary
            | Function::Imargument
            | Function::Imconjugate
            | Function::Imcos
            | Function::Imcosh
            | Function::Imcot
            | Function::Imcsc
            | Function::Imcsch
            | Function::Imdiv
            | Function::Imexp
            | Function::Imln
            | Function::Imlog10
            | Function::Imlog2
            | Function::Impower
            | Function::Imreal
            | Function::Imsec
            | Function::Imsech
            | Function::Imsin
            | Function::Imsinh
            | Function::Imsqrt
            | Function::Imsub
            | Function::Imtan
            | Function::Imsum
            | Function::Improduct
            // --- End engineering (T-1383) ---
            // --- Math (T-1802) ---
            | Function::Quotient
            | Function::Factdouble
            | Function::Seriessum
            // --- End math (T-1802) ---
            // --- Math number lifts (T-1773) ---
            // (alphabetical)
            | Function::Abs
            | Function::Acos
            | Function::Acosh
            | Function::Acot
            | Function::Acoth
            | Function::Asin
            | Function::Asinh
            | Function::Atan
            | Function::Atanh
            | Function::Ceiling
            | Function::CeilingMath
            | Function::CeilingPrecise
            | Function::Cos
            | Function::Cosh
            | Function::Cot
            | Function::Coth
            | Function::Csc
            | Function::Csch
            | Function::Degrees
            | Function::EcmaCeiling
            | Function::Even
            | Function::Exp
            | Function::Fact
            | Function::Floor
            | Function::FloorMath
            | Function::FloorPrecise
            | Function::Int
            | Function::IsoCeiling
            | Function::Ln
            | Function::Log10
            | Function::Mod
            | Function::Odd
            | Function::Radians
            | Function::Round
            | Function::Rounddown
            | Function::Roundup
            | Function::Sec
            | Function::Sech
            | Function::Sign
            | Function::Sin
            | Function::Sinh
            | Function::Sqrt
            | Function::Tan
            | Function::Tanh
            | Function::Trunc
            // --- End math number lifts (T-1773) ---
            // --- Financial ATP (T-1822) ---
            | Function::Accrint
            | Function::Coupdaybs
            | Function::Coupdays
            | Function::Coupdaysnc
            | Function::Coupncd
            | Function::Coupnum
            | Function::Couppcd
            | Function::Duration
            | Function::Mduration
            | Function::Price
            // --- End financial ATP (T-1822) ---
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
            // The core worksheet financial functions lift over arrays and over a range in every
            // argument, on the range's own rows too (`ROWS(PMT(C1:C60,10,100))` and
            // `ROWS(PMT(0.1,D1:D60,100))` are 60): probes/fin-argument-modes (every argument over
            // `{v;v}`, the first over a range), probes/fin-later-argument-ranges K1-K25 (the last
            // over a range), probes/scalar-lift-shapes H13-H48 (docs/excel-semantics/financial.md
            // "Financial function argument modes"). Their kernels run in closed form; RATE is
            // weighted by its step cap. DB and VDB loop over the periods without a cap of their
            // own, so they are not entered yet (TODO(T-1820)).
            Function::Pmt | Function::Pv | Function::Fv | Function::Nper => Some(&LIFT_3_OF_5),
            Function::Rate => Some(&RATE),
            Function::Ipmt | Function::Ppmt => Some(&LIFT_4_OF_6),
            Function::Ispmt | Function::Syd => Some(&LIFT_4),
            Function::Rri | Function::Pduration | Function::Sln => Some(&LIFT_3),
            Function::Ddb => Some(&LIFT_4_OF_5),
            // Analysis ToolPak functions whose every argument has been probed over a range on
            // its own rows: all are refused, while an array lifts (fin-argument-modes; EFFECT:
            // scalar-lift-shapes H11, H12; DOLLARDE, DOLLARFR: scalar-lift-open-a Y16-Y18; the
            // rest: fin-later-argument-ranges H1-H82).
            Function::Effect | Function::Nominal | Function::Dollarde | Function::Dollarfr => {
                Some(&ATP_2)
            }
            Function::Accrintm => Some(&ATP_3_OF_5),
            Function::Disc
            | Function::Intrate
            | Function::Pricedisc
            | Function::Received
            | Function::Yielddisc => Some(&ATP_4_OF_5),
            Function::Pricemat | Function::Yieldmat => Some(&ATP_5_OF_6),
            Function::Tbilleq | Function::Tbillprice | Function::Tbillyield => Some(&ATP_3),
            // --- Engineering (T-1383) ---
            // The engineering functions are Analysis ToolPak functions: every argument lifts
            // over an array and refuses a multi-cell range, also on the range's own rows
            // (docs/excel-semantics/engineering.md "Engineering function argument modes"). Each
            // argument is probed over both: the first in probes/eng-argument-modes B:E; the
            // later ones in B:D there and over a range in probes/scalar-lift-shapes H8-H10
            // (BESSELJ's n, DEC2BIN's places, DELTA), probes/scalar-lift-open-a Y8-Y15
            // (CONVERT's units, COMPLEX's suffix, IMSUB's and IMDIV's second, IMPOWER's n) and
            // probes/eng-later-argument-lift H1-H11, H16-H19 (the other orders and places,
            // COMPLEX's i_num, DELTA's number2, GESTEP's step, ERF's upper_limit).
            Function::Besseli | Function::Besselj | Function::Besselk | Function::Bessely => {
                Some(&BESSEL)
            }
            Function::Bin2dec
            | Function::Hex2dec
            | Function::Oct2dec
            | Function::ErfPrecise
            | Function::Erfc
            | Function::ErfcPrecise
            | Function::Imabs
            | Function::Imaginary
            | Function::Imargument
            | Function::Imconjugate
            | Function::Imcos
            | Function::Imcosh
            | Function::Imcot
            | Function::Imcsc
            | Function::Imcsch
            | Function::Imexp
            | Function::Imln
            | Function::Imlog10
            | Function::Imlog2
            | Function::Imreal
            | Function::Imsec
            | Function::Imsech
            | Function::Imsin
            | Function::Imsinh
            | Function::Imsqrt
            | Function::Imtan => Some(&ARRAYS_1),
            // `[places]`, `[number2]`, `[step]`, `[upper_limit]`: one required, one optional.
            Function::Bin2hex
            | Function::Bin2oct
            | Function::Dec2Bin
            | Function::Dec2hex
            | Function::Dec2oct
            | Function::Hex2bin
            | Function::Hex2oct
            | Function::Oct2bin
            | Function::Oct2hex
            | Function::Delta
            | Function::Gestep
            | Function::Erf => Some(&ATP_1_OF_2),
            Function::Imdiv | Function::Imsub | Function::Impower => Some(&ATP_2),
            // `COMPLEX(real_num, i_num, [suffix])`.
            Function::Complex => Some(&ATP_2_OF_3),
            Function::Convert => Some(&ATP_3),
            // BIT* are worksheet functions: both arguments lift over a range too
            // (probes/eng-argument-modes E8:E12, probes/scalar-lift-shapes H7,
            // probes/eng-later-argument-lift H12-H15, H20, K12-K15). The kernels are O(1).
            Function::Bitand
            | Function::Bitor
            | Function::Bitxor
            | Function::Bitlshift
            | Function::Bitrshift => Some(&LIFT_2),
            // IMSUM and IMPRODUCT fold their arguments whole: a range or an array is never
            // lifted, `ROWS(IMSUM({"1+2i";"3"}))` is 1 (probes/eng-argument-modes A60:A83,
            // B/C/E42, B/C/E50; engineering.md "IMSUM and IMPRODUCT take their arguments
            // whole", T-1384).
            Function::Imsum | Function::Improduct => Some(&COMPLEX_FOLD),
            // --- End engineering (T-1383) ---
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
            // --- Financial ATP (T-1822) ---
            // Analysis ToolPak functions: every argument lifts over an array and refuses a
            // multi-cell range, also on the range's own rows. The first arguments over a range:
            // probes/fin-argument-modes H3-H253; the later ones: probes/fin-remaining-later-
            // argument-ranges H1-H172 (each with its intersected control in I), and PRICE's
            // frequency and basis, COUPDAYS's frequency and basis, COUPNUM's frequency and
            // ACCRINT's frequency and basis in probes/scalar-lift-open-a Y1-Y7
            // (docs/excel-semantics/financial.md "Financial function argument modes"). Their
            // coupon schedule walks are bounded by the calendar and weighted per walk.
            Function::Coupdaybs
            | Function::Coupdays
            | Function::Coupdaysnc
            | Function::Coupncd
            | Function::Coupnum
            | Function::Couppcd => Some(&COUPON),
            Function::Accrint => Some(&ACCRINT),
            Function::Duration | Function::Mduration => Some(&BOND_5_OF_6),
            Function::Price => Some(&BOND_6_OF_7),
            // --- End financial ATP (T-1822) ---
            _ => None,
        }
    }
}
