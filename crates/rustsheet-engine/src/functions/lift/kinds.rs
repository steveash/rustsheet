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
/// `DAYS360(start_date, end_date, [method])`.
const LIFT_2_OF_3: ArgKinds = ArgKinds::fixed(2, &[Lift, Lift, Lift], Constant);
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
// --- Statistical (T-1290) -------------------------------------------------------------------
/// `BINOM.DIST.RANGE(trials, probability_s, number_s, [number_s2])`.
const LIFT_3_OF_4: ArgKinds = ArgKinds::fixed(3, &[Lift, Lift, Lift, Lift], Constant);
/// `BINOM.INV(trials, probability_s, alpha)` and CRITBINOM: a search that halves `[0, trials]`
/// (a `u64`) at most 64 times, its own per-call cap (`statistical/binom.rs`), with one statrs
/// incomplete-beta cdf (up to 140 continued-fraction steps) each. Measured in release: up to
/// about 65 us an element (`BINOM.INV(1E12,0.3,alpha)`; 37 us at 1E6 trials, 13 us at a tiny
/// `probability_s`). The weight caps a lifted call at 2^27 / 4096 = 32,768 elements: 2.14 s
/// measured at the worst case, the budget of the Bessel entries (a DEVIATION, statistics.md).
const BINOM_INV: ArgKinds = ArgKinds::fixed(3, &[Lift, Lift, Lift], Weighted(BINOM_INV_WEIGHT));
/// The units one BINOM.INV element is charged against `MAX_LIFTED_CELL_VISITS`.
const BINOM_INV_WEIGHT: u32 = 1 << 12;
// --- End statistical (T-1290) ---------------------------------------------------------------

// --- Statistical quantiles (T-1848) ---------------------------------------------------------
/// The cost one incomplete-beta cdf can reach: statrs' `checked_beta_reg` at its full 140
/// continued-fraction steps plus three `ln_gamma` (shapes near 1E5 at x = 0.5), measured in release
/// at 3.1 to 3.4 us a call (T-1848, load 8 on 4 cores; 0.15 us for `I_0.4(2,3)`). A step adjustment
/// costs about 13 ns. Each quantile weight below charges the bound of its kernel's caps at that cost,
/// not a measured case, so the bound holds wherever a slow cdf and a long search coincide.
///
/// What one BETA.INV or F.INV element is charged. Its quantile is the capped port of statrs'
/// inversion (`statistical/inverse_beta.rs`: at most 512 Newton steps of one cdf and 4,096 step
/// adjustments, its own per-call cap: 1.6 ms at full cost) or a bisection of at most 1,100 cdfs
/// (BETA.INV, `special::bisect_unit_interval`: 3.4 ms) or 1,301 (F.INV at a df of 1E5 or more,
/// `special::bisect_increasing`'s 1,100 doublings and 200 halvings: 4.0 ms, the bound). The weight
/// caps a lifted call at 2^27 / 2^18 = 512 elements: 2.1 s at the bound (a DEVIATION,
/// statistics.md). The cases measured cost much less (over 8,192 elements in release: 167 us for
/// `BETA.INV(1E-300,0.5,1E6)`, a full bisection of fast cdfs; 105 us for the longest Newton search
/// seen, 377 steps, `F.INV(3.718810279553989E-162,4532,43)`; 1 to 2 us for `BETA.INV(0.4,2,3)`).
const QUANTILE_WEIGHT: u32 = 1 << 18;
/// `BETA.INV(probability, alpha, beta, [A], [B])` and BETAINV.
const BETA_INV: ArgKinds = ArgKinds::fixed(
    3,
    &[Lift, Lift, Lift, Lift, Lift],
    Weighted(QUANTILE_WEIGHT),
);
/// `F.INV(probability, deg_freedom1, deg_freedom2)`, F.INV.RT and FINV.
const F_INV: ArgKinds = ArgKinds::fixed(3, &[Lift, Lift, Lift], Weighted(QUANTILE_WEIGHT));
/// What one T.INV or CONFIDENCE.T element is charged. Its quantile is the capped port with at most
/// 64 Newton steps and 512 step adjustments (`inverse_beta::T_MAX_NEWTON_STEPS`,
/// `T_MAX_STEP_ADJUSTMENTS`; 27 and 78 were the most seen: 0.21 ms at full cost), a closed form, or
/// a bisection of at most 264 cdfs (`t_dist::t_quantile`'s 64 doublings and 200 halvings: 0.82 ms,
/// the bound). The weight caps a lifted call at 2^27 / 2^16 = 2,048 elements: 1.7 s at the bound
/// (a DEVIATION, statistics.md). Measured cases cost much less (over 65,536 elements in release:
/// 17.5 us for `T.INV.2T(1E-9,99999)`, the bisection; 1.2 us for `T.INV(0.4,5)`).
const T_QUANTILE_WEIGHT: u32 = 1 << 16;
/// `T.INV(probability, deg_freedom)`, T.INV.2T and TINV.
const T_INV: ArgKinds = ArgKinds::fixed(2, &[Lift, Lift], Weighted(T_QUANTILE_WEIGHT));
/// `CONFIDENCE.T(alpha, standard_dev, size)`.
const CONFIDENCE_T: ArgKinds = ArgKinds::fixed(3, &[Lift, Lift, Lift], Weighted(T_QUANTILE_WEIGHT));
// --- End statistical quantiles (T-1848) -----------------------------------------------------

// --- Incomplete gamma (T-1872) --------------------------------------------------------------
/// What one GAMMA.DIST, GAMMA.INV, CHISQ.* or POISSON element is charged. Each runs one capped
/// port of statrs' incomplete gamma (`statistical/incomplete_gamma.rs`): a cdf of at most 2^19
/// series or continued-fraction terms, or an inversion whose cdfs (at most about 2,120 of them:
/// 1,075 halvings, 1,025 doublings, 8 bisections, 4 Newton steps) share one budget of 2^19
/// terms. Measured in release (T-1872, load 2 to 5 on 4 cores) over 512 elements at the cap: a
/// continued-fraction term costs 4.1 to 4.7 ns on its NaN path (2.4 ms for 2^19,
/// `GAMMA.DIST(1.0000001E300,1E300,1,TRUE)`, whose terms overflow at once) and 2.4 ns where it
/// converges and rescales (1.3 ms, `GAMMA.DIST(1000000000000001,1E15,1,TRUE)`, which reaches the
/// cap at a shape of 1E15 and rescales on about 65% of its terms), a series term 2.5 ns (1.3 ms,
/// `GAMMA.DIST(1E12,1E12,1,TRUE)`), and a cdf's own work 49 ns (`GAMMA.INV(1E-300,0.001,1E-300)`,
/// 1,060 cdfs: 52 us). The bound of an element is 2^19 terms at the slowest, 4.7 ns, plus 2,120 cdfs
/// at 49 ns, about 2.6 ms, so the weight caps a lifted call at 2^27 / 2^18 = 512 elements: 1.3 s
/// at the bound (a DEVIATION, statistics.md). Measured cases cost less: 2.4 ms for the
/// continued fraction at the cap, 1.2 ms for `CHISQ.DIST(1E10,1E10,TRUE)` (481,117 terms),
/// 1.6 us for `GAMMA.INV(0.4,2,3)`.
const INCOMPLETE_GAMMA_WEIGHT: u32 = 1 << 18;
/// `CHISQ.DIST.RT(x, deg_freedom)`, CHIDIST, `CHISQ.INV(probability, deg_freedom)`,
/// CHISQ.INV.RT and CHIINV.
const CHISQ_2: ArgKinds = ArgKinds::fixed(2, &[Lift, Lift], Weighted(INCOMPLETE_GAMMA_WEIGHT));
/// `CHISQ.DIST(x, deg_freedom, cumulative)`, `POISSON.DIST(x, mean, cumulative)`, POISSON,
/// `GAMMA.INV(probability, alpha, beta)` and GAMMAINV.
const INCOMPLETE_GAMMA_3: ArgKinds =
    ArgKinds::fixed(3, &[Lift, Lift, Lift], Weighted(INCOMPLETE_GAMMA_WEIGHT));
/// `GAMMA.DIST(x, alpha, beta, cumulative)` and GAMMADIST.
const GAMMA_DIST: ArgKinds = ArgKinds::fixed(
    4,
    &[Lift, Lift, Lift, Lift],
    Weighted(INCOMPLETE_GAMMA_WEIGHT),
);
// --- End incomplete gamma (T-1872) ----------------------------------------------------------

// --- Hypergeometric (T-1873) ----------------------------------------------------------------
/// One HYPGEOM.DIST element: its capped cdf's bound, (2^18 + 131) terms at 250 ns, about 66 ms, so
/// 32 elements take about 2.1 s (a DEVIATION; measured in release: hypegeom.rs `MAX_TERMS`).
const HYPGEOM_DIST_WEIGHT: u32 = 1 << 22;
const HYPGEOM_DIST: ArgKinds = ArgKinds::fixed(5, &[Lift; 5], Weighted(HYPGEOM_DIST_WEIGHT));
const HYPGEOMDIST: ArgKinds = ArgKinds::fixed(4, &[Lift; 4], Constant);
// --- End hypergeometric (T-1873) ------------------------------------------------------------

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

// --- Financial ATP (T-1842) -----------------------------------------------------------------
/// What one YIELD or ODDFYIELD element is charged: its kernel walks the coupon schedule once
/// (`CouponSchedule`, `OddFirstSchedule`; up to 4.4 and 6.3 ms), then the solver evaluates the
/// price at most 397 times (100 Newton steps of three, 97 bisection prices), each a sum over up to
/// 32,400 quarterly coupons (0.55 ms at the calendar's longest schedule, release build). That
/// bound is about 0.23 s an element, so a lifted call runs at most 2^27 / 2^24 = 8 elements,
/// about 1.8 s at the bound; the slowest inputs found take 54 ms an element (`YIELD(1,2958465,
/// 0.05,1E300,100,4,1)`, about 90 prices: 16 elements took 0.94 s). docs/excel-semantics/
/// financial.md "Yield and odd-coupon work cap" (a DEVIATION; TODO(T-1869)).
const YIELD_SOLVER_WEIGHT: u32 = 1 << 24;
/// `YIELD(settlement, maturity, rate, pr, redemption, frequency, [basis])`.
const YIELD: ArgKinds = ArgKinds::fixed(
    6,
    &[
        LiftArrays, LiftArrays, LiftArrays, LiftArrays, LiftArrays, LiftArrays, LiftArrays,
    ],
    Weighted(YIELD_SOLVER_WEIGHT),
);
/// `ODDFPRICE(settlement, maturity, issue, first_coupon, rate, yld, redemption, frequency,
/// [basis])`: the first coupon's schedule check and the odd first period's walks and sums, up to
/// 6.3 ms an element (a long first period from serial 1 to 9999-09-30), so 8 walks: at most 256
/// elements, 1.7 s measured at that worst case.
const ODDFPRICE: ArgKinds = ArgKinds::fixed(
    8,
    &[
        LiftArrays, LiftArrays, LiftArrays, LiftArrays, LiftArrays, LiftArrays, LiftArrays,
        LiftArrays, LiftArrays,
    ],
    Weighted(8 * COUPON_WALK_WEIGHT),
);
/// `ODDFYIELD(settlement, maturity, issue, first_coupon, rate, pr, redemption, frequency,
/// [basis])`: as YIELD (`YIELD_SOLVER_WEIGHT`).
const ODDFYIELD: ArgKinds = ArgKinds::fixed(
    8,
    &[
        LiftArrays, LiftArrays, LiftArrays, LiftArrays, LiftArrays, LiftArrays, LiftArrays,
        LiftArrays, LiftArrays,
    ],
    Weighted(YIELD_SOLVER_WEIGHT),
);
/// `ODDLPRICE(settlement, maturity, last_interest, rate, yld, redemption, frequency, [basis])`
/// and ODDLYIELD (closed form, no solver): a walk from the last interest date to the maturity and
/// a loop over its quasi-coupon periods, up to 2.2 ms an element from serial 2 to 9999-12-31, so
/// two walks: at most 1,024 elements, 2.2 s measured for ODDLPRICE at that worst case.
const ODD_LAST: ArgKinds = ArgKinds::fixed(
    7,
    &[
        LiftArrays, LiftArrays, LiftArrays, LiftArrays, LiftArrays, LiftArrays, LiftArrays,
        LiftArrays,
    ],
    Weighted(2 * COUPON_WALK_WEIGHT),
);
// --- End financial ATP (T-1842) -------------------------------------------------------------

// --- Financial ATP (T-1845) -----------------------------------------------------------------
/// What one CUMIPMT or CUMPRINC element is charged: the derived bound of its kernel, at most
/// `CUMULATIVE_LOOP_PERIODS` (32) PPMT steps of three powers each and then, when a power
/// overflows on the last step, the closed form (a few `exp`s), about 2.9 us an element in a
/// release build (2^19 elements of `CUMPRINC(..,1024,1E307*15.98,1,32,0)`, whose last step
/// overflows, took 1.53 s; 32 full steps of type 1 1.51 s). 2^8 units run at most 2^19
/// elements, about 1.5 s. docs/excel-semantics/financial.md "CUMIPMT and CUMPRINC over many
/// periods" (a DEVIATION).
const CUMULATIVE_WEIGHT: u32 = 1 << 8;
/// `CUMIPMT(rate, nper, pv, start_period, end_period, type)` and CUMPRINC, whatever the period
/// count (functions/financial/cumulative.rs).
const CUMULATIVE: ArgKinds = ArgKinds::fixed(
    6,
    &[
        LiftArrays, LiftArrays, LiftArrays, LiftArrays, LiftArrays, LiftArrays,
    ],
    Weighted(CUMULATIVE_WEIGHT),
);
// --- End financial ATP (T-1845) -------------------------------------------------------------

// --- Financial ATP (T-1841) -----------------------------------------------------------------
/// What one AMORLINC element is charged, priced from its cap: at most `LINEAR_STEP_CAP` (1,024)
/// single subtractions (functions/financial/amortization.rs `linear_charge`; the longest real walk
/// takes 616). A step costs about 7.7 ns in a release build (2^18 elements of
/// `AMORLINC(..,2400,..,300,9E15,1.08E-16,1)`, 613 steps each, within 1% of the longest, took
/// 1.23 s, 4.7 us an element), so a walk at the cap is about 7.9 us and 2^9 units run at most
/// 2^18 elements, about 2.1 s. docs/excel-semantics/financial.md "AMORLINC and AMORDEGRC work
/// cap" (a DEVIATION).
const AMORLINC_WEIGHT: u32 = 1 << 9;
/// What one AMORDEGRC element is charged: at most `DEGRESSIVE_LOOP_PERIODS` (2^20) periods
/// walked one by one, then a closed form, about 4 ms an element in a release build (512 elements
/// of `AMORDEGRC(1E15,..,1,5E6,1E-7,1)`, whose walk takes no shortcut, took 2.06 s). 2^18 units
/// run at most 512 elements, about 2 s.
const AMORDEGRC_WEIGHT: u32 = 1 << 18;
/// `AMORLINC(cost, date_purchased, first_period, salvage, period, rate, [basis])`.
const AMORLINC: ArgKinds = ArgKinds::fixed(
    6,
    &[
        LiftArrays, LiftArrays, LiftArrays, LiftArrays, LiftArrays, LiftArrays, LiftArrays,
    ],
    Weighted(AMORLINC_WEIGHT),
);
/// `AMORDEGRC(cost, date_purchased, first_period, salvage, period, rate, [basis])`.
const AMORDEGRC: ArgKinds = ArgKinds::fixed(
    6,
    &[
        LiftArrays, LiftArrays, LiftArrays, LiftArrays, LiftArrays, LiftArrays, LiftArrays,
    ],
    Weighted(AMORDEGRC_WEIGHT),
);
// --- End financial ATP (T-1841) -------------------------------------------------------------

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
            // --- Financial ATP (T-1842) ---
            // Analysis ToolPak functions, as the T-1822 ones: every argument lifts over an array
            // (probes/fin-argument-modes H190-H230, H344-H351) and refuses a multi-cell range,
            // also on the range's own rows (H191-H223, H345; fin-remaining-later-argument-ranges
            // H105-H164, H173-H184; fin-yield-odd-lift H5-H14), each element the scalar call
            // (fin-yield-odd-lift A1-E5; docs/excel-semantics/financial.md "Financial function
            // argument modes"). Their solvers and schedule walks are weighted per element
            // ("Yield and odd-coupon work cap").
            Function::Yield => Some(&YIELD),
            Function::Oddfprice => Some(&ODDFPRICE),
            Function::Oddfyield => Some(&ODDFYIELD),
            Function::Oddlprice | Function::Oddlyield => Some(&ODD_LAST),
            // --- End financial ATP (T-1842) ---
            // --- Financial ATP (T-1845) ---
            // Analysis ToolPak functions, as the T-1822 ones: every argument lifts over an array
            // (probes/fin-argument-modes H73-H87; fin-remaining-later-argument-ranges K11) and
            // refuses a multi-cell range, also on the range's own rows (fin-argument-modes H74,
            // H82; fin-remaining-later-argument-ranges H65-H84, each with its intersected control
            // in I; docs/excel-semantics/financial.md "Financial function argument modes"). Their
            // period sums are bounded ("CUMIPMT and CUMPRINC over many periods").
            Function::Cumipmt | Function::Cumprinc => Some(&CUMULATIVE),
            // --- End financial ATP (T-1845) ---
            // --- Financial ATP (T-1841) ---
            // Analysis ToolPak functions, as the T-1822 ones: every argument lifts over an array
            // (probes/fin-argument-modes H19, H21-H26, H28, H30-H35) and refuses a multi-cell
            // range, also on the range's own rows (fin-argument-modes H20, H29;
            // fin-remaining-later-argument-ranges H11-H34, each with its intersected control in
            // I). Their period walks are bounded (docs/excel-semantics/financial.md "AMORLINC and
            // AMORDEGRC over many periods").
            Function::Amorlinc => Some(&AMORLINC),
            Function::Amordegrc => Some(&AMORDEGRC),
            // --- End financial ATP (T-1841) ---
            // --- Statistical (T-1290) ---
            // The statistical functions whose every argument is scalar are worksheet functions:
            // every argument lifts over an array and over a multi-cell range, on the range's own
            // rows too (probes/stat-argument-modes: every argument over `{v;v}`, the first over
            // a range; probes/stat-later-argument-lift H1-H73: the last and the optional ones
            // over a range, K3-K16: the optional ones over an array; scalar-lift-shapes H15,
            // H16; docs/excel-semantics/statistics.md "Statistical function argument modes").
            // Their kernels are closed form, or call statrs' incomplete beta (a continued
            // fraction of at most 140 steps) a fixed number of times: measured in release over
            // 2^20 elements, BETA.DIST with shapes of 1E6 costs about 0.29 us an element, F.DIST
            // and T.DIST with 1E9 degrees of freedom 0.28 and 0.30 us, NORM.DIST 0.14 us, and the
            // slowest, BINOM.DIST with 1E18 trials, about 3.2 us (3.3 s for 2^20 elements), and
            // BINOM.DIST.RANGE, which runs two cdfs, 6.4 us (6.7 s for 2^20 elements).
            // PERMUT multiplies at most 171 factors whatever its arguments (0.22 us). The
            // quantile searches of BETA.INV, F.INV and T.INV (T-1848), the incomplete-gamma
            // functions (T-1872) and HYPGEOM.DIST's capped cdf sum (T-1873) are entered below.
            Function::Gauss
            | Function::Phi
            | Function::Fisher
            | Function::FisherInv
            | Function::Gamma
            | Function::GammaLn
            | Function::GammaLnPrecise
            | Function::NormSInv
            | Function::NormSInvCompat
            | Function::NormSDistCompat => Some(&LIFT_1),
            Function::NormSdist
            | Function::Permut
            | Function::Permutationa
            | Function::TDist2T
            | Function::TDistRT => Some(&LIFT_2),
            Function::NormInv
            | Function::NormInvCompat
            | Function::LogNormInv
            | Function::LoginvCompat
            | Function::ConfidenceNorm
            | Function::ConfidenceCompat
            | Function::Standardize
            | Function::ExponDist
            | Function::ExponDistCompat
            | Function::LogNormDistCompat
            | Function::NegbinomDistCompat
            | Function::TDist
            | Function::TDistCompat
            | Function::FDistRT
            | Function::FDistCompat => Some(&LIFT_3),
            Function::NormDist
            | Function::NormDistCompat
            | Function::WeibullDist
            | Function::WeibullCompat
            | Function::LogNormDist
            | Function::BinomDist
            | Function::BinomDistCompat
            | Function::NegbinomDist
            | Function::FDist => Some(&LIFT_4),
            // `BETA.DIST(x, alpha, beta, cumulative, [A], [B])`, `BETADIST(x, alpha, beta, [A], [B])`.
            Function::BetaDist => Some(&LIFT_4_OF_6),
            Function::BetaDistCompat => Some(&LIFT_3_OF_5),
            Function::BinomDistRange => Some(&LIFT_3_OF_4),
            Function::BinomInv | Function::CritbinomCompat => Some(&BINOM_INV),
            // --- End statistical (T-1290) ---
            // --- Statistical quantiles (T-1848) ---
            // As the statistical functions above: every argument lifts over an array and a range,
            // on the range's own rows (probes/stat-argument-modes H90-H99, H173-H177, H231-H245,
            // H518-H529; probes/stat-quantile-lift H1-H25: the last arguments and BETA.INV's [A]
            // and [B] over a range, K1-K8: [A] and [B] over an array). Each quantile search keeps
            // a per-call cap of its own (`statistical/inverse_beta.rs` and the bisections), so
            // the entries are weighted by their measured cost (statistics.md "Quantile searches:
            // per-call cap and lifting").
            Function::BetaInv | Function::BetaInvCompat => Some(&BETA_INV),
            Function::FInv | Function::FInvRT | Function::FInvCompat => Some(&F_INV),
            Function::TInv | Function::TInv2T | Function::TInvCompat => Some(&T_INV),
            Function::ConfidenceT => Some(&CONFIDENCE_T),
            // --- End statistical quantiles (T-1848) ---
            // --- Incomplete gamma (T-1872) ---
            // As the statistical functions above: every argument lifts over an array and a range,
            // on the range's own rows (probes/stat-argument-modes H132-H156, H293-H314,
            // H476-H485; probes/stat-quantile-lift H27-H50: the last argument over a range). The
            // incomplete gamma and its search keep a per-call cap of their own
            // (`statistical/incomplete_gamma.rs`), so the entries are weighted by its bound
            // (statistics.md "Incomplete gamma: per-call cap and lifting").
            Function::ChisqDistRT
            | Function::ChiDistCompat
            | Function::ChisqInv
            | Function::ChisqInvRT
            | Function::ChiInvCompat => Some(&CHISQ_2),
            Function::ChisqDist
            | Function::PoissonDist
            | Function::PoissonCompat
            | Function::GammaInv
            | Function::GammaInvCompat => Some(&INCOMPLETE_GAMMA_3),
            Function::GammaDist | Function::GammaDistCompat => Some(&GAMMA_DIST),
            // --- End incomplete gamma (T-1872) ---
            // --- Hypergeometric (T-1873) ---
            // As above (stat-argument-modes H330-H342, stat-quantile-lift H51-H54).
            Function::HypGeomDist => Some(&HYPGEOM_DIST),
            Function::HypGeomDistCompat => Some(&HYPGEOMDIST),
            // --- End hypergeometric (T-1873) ---
            _ => None,
        }
    }
}
