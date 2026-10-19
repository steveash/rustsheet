//! The lifting table's financial entries (SPEC-0011 "The argument-kind table"): the core
//! worksheet functions (PMT, RATE, DDB, ...) and the Analysis `ToolPak` ones, with the work
//! weights of their coupon walks, solvers and period loops.

use super::ArgKind::{Lift, LiftArrays};
use super::KernelWork::{Constant, Weighted};
use super::{ArgKinds, ATP_2, ATP_3, LIFT_3, LIFT_3_OF_5, LIFT_4, LIFT_4_OF_6};
use crate::functions::Function;

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

/// This family's part of the lifting table (`Function::arg_kinds`).
pub(super) fn arg_kinds(function: &Function) -> Option<&'static ArgKinds> {
    match function {
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
        _ => None,
    }
}
