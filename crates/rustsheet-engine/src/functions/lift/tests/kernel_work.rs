//! SPEC-0011 "Security rule for kernel work" (T-1893): a kernel whose work could depend on its
//! argument values is `KernelWork::Weighted` (or `ByValue`) in the lifting table, or `Constant`
//! with its fixed cap named below, so no family task can enter one as `Constant` silently. A
//! family task that enters a value-dependent function adds it here. The check does not see a
//! `Weighted` kernel's own per-call cap; its table entry's doc comment cites it.

use super::super::kinds::{KernelWork, ValueWork};
use crate::functions::Function;

/// Where the incomplete beta's cap is: statrs' `checked_beta_reg` runs at most 140
/// continued-fraction steps, and `ln_gamma` is a fixed Lanczos sum.
const BETA_CDF: &str = "one statrs incomplete beta: at most 140 continued-fraction steps";

/// The engine's own incomplete beta (`statistical/incomplete_beta.rs`, T-0970) behind the F
/// family: Cephes' continued fraction capped at 300 steps, fixed-length series.
const ENGINE_BETA_CDF: &str =
    "one incomplete_beta evaluation: at most 300 continued-fraction steps";

/// ERF's and ERFC's cap: `erf::small_erf` and `erf::erfc_kernel` are fixed-degree rationals, and
/// `x87_exp::excel_exp` runs a fixed-length estimate and at most one fixed 26-term series.
const ERF_KERNELS: &str =
    "erf kernels: fixed-degree rationals and one fixed-length excel_exp series";

/// WORKDAY's cap: a galloping search then a bisection over the date range (`nearest_offset`), at
/// most about 44 closed-form counts, each with 14 binary searches of the holidays, prepared once
/// (`LiftCost::Prepared`).
const WORKDAY_SEARCH: &str = "at most about 44 closed-form counts of the prepared holidays";

/// NETWORKDAYS' cap: one closed-form count (T-1808) with 14 binary searches of the holidays.
const NETWORKDAYS_COUNT: &str = "one closed-form count of the prepared holidays";

/// The value-dependent kernels: each with the fixed cap that lets its entry be `Constant`, or
/// `None` when it must be `Weighted` or `ByValue` (a cap that grows with the values, or one not yet
/// read).
/// A function with no table entry yet is checked once it is entered.
const VALUE_DEPENDENT: &[(Function, Option<&str>)] = &[
    // Fixed caps, read in each kernel.
    (Function::BetaDist, Some(BETA_CDF)),
    (Function::BetaDistCompat, Some(BETA_CDF)),
    (Function::FDist, Some(ENGINE_BETA_CDF)),
    (Function::FDistRT, Some(ENGINE_BETA_CDF)),
    (Function::FDistCompat, Some(ENGINE_BETA_CDF)),
    (Function::TDist, Some(BETA_CDF)),
    (Function::TDist2T, Some(BETA_CDF)),
    (Function::TDistRT, Some(BETA_CDF)),
    (Function::TDistCompat, Some(BETA_CDF)),
    // A pmf walk of up to 2^12 double-double steps (T-0977): `Weighted`.
    (Function::HypGeomDistCompat, None),
    (
        Function::Fact,
        Some("descending_product stops at the first infinity: n*(n-1)*... overflows in 170 steps"),
    ),
    (
        Function::Factdouble,
        Some("descending_product stops at the first infinity: n*(n-2)*... overflows in 151 steps"),
    ),
    (
        Function::Permut,
        Some("at most 171 factors: 171 distinct integers pass f64::MAX"),
    ),
    (Function::Permutationa, Some("O(1): one powf")),
    (
        Function::Exp,
        Some("one x87_exp::excel_exp: a fixed estimate, at most one fixed 26-term series"),
    ),
    (Function::Erf, Some(ERF_KERNELS)),
    (Function::ErfPrecise, Some(ERF_KERNELS)),
    (Function::Erfc, Some(ERF_KERNELS)),
    (Function::ErfcPrecise, Some(ERF_KERNELS)),
    (
        Function::Seriessum,
        Some("one step per coefficient, charged per element as their cells (LiftCost::Cells)"),
    ),
    // The searches compare each start position of `within_text` with `find_text`: `ByValue`,
    // charged by their texts' lengths (T-1961).
    (Function::Find, None),
    (Function::Findb, None),
    (Function::Search, None),
    (Function::Searchb, None),
    // The other text kernels (T-1775): linear in their texts, as `broadcast_text` was. A text is
    // at most 32,767 units in a cell, and a lifted call's results are bounded by the driver's
    // 64 MiB text budget.
    (
        Function::Substitute,
        Some("one pass over the text; the result's length is checked before it is built"),
    ),
    (
        Function::Rept,
        Some("the result's length (at most 32,767 units) is checked before it is built"),
    ),
    (
        Function::Npv,
        Some("one pass over the cash flows, charged per element as their count (LiftCost::Cells)"),
    ),
    (
        Function::Mirr,
        Some("one pass over the cash flows, charged per element as their count (LiftCost::Cells)"),
    ),
    (
        Function::Xnpv,
        Some("one pass over the cash flows, charged per element as their count (LiftCost::Cells)"),
    ),
    (
        Function::Fvschedule,
        Some("one pass over the schedule, charged per element as its count (LiftCost::Cells)"),
    ),
    (
        Function::Decimal,
        Some("at most 255 text units: a longer text is #VALUE! before the loop"),
    ),
    (
        Function::Base,
        Some("at most 64 digits of an i64, then at most 255 units of padding"),
    ),
    (Function::Arabic, Some("one pass over the text")),
    (Function::Workday, Some(WORKDAY_SEARCH)),
    (Function::WorkdayIntl, Some(WORKDAY_SEARCH)),
    (Function::Networkdays, Some(NETWORKDAYS_COUNT)),
    (Function::NetworkdaysIntl, Some(NETWORKDAYS_COUNT)),
    // Weighted: solvers, searches, series and walks whose length the values set.
    (Function::Rate, None),
    (Function::Irr, None),
    (Function::Xirr, None),
    (Function::Db, None),
    (Function::Vdb, None),
    (Function::Combin, None),
    (Function::Combina, None),
    (Function::Multinomial, None),
    (Function::BinomDist, None),
    (Function::BinomDistCompat, None),
    (Function::BinomDistRange, None),
    (Function::NegbinomDist, None),
    (Function::NegbinomDistCompat, None),
    (Function::BinomInv, None),
    (Function::CritbinomCompat, None),
    (Function::BetaInv, None),
    (Function::BetaInvCompat, None),
    (Function::FInv, None),
    (Function::FInvRT, None),
    (Function::FInvCompat, None),
    (Function::TInv, None),
    (Function::TInv2T, None),
    (Function::TInvCompat, None),
    (Function::ConfidenceT, None),
    (Function::ChisqDist, None),
    (Function::ChisqDistRT, None),
    (Function::ChiDistCompat, None),
    (Function::ChisqInv, None),
    (Function::ChisqInvRT, None),
    (Function::ChiInvCompat, None),
    (Function::GammaDist, None),
    (Function::GammaDistCompat, None),
    (Function::GammaInv, None),
    (Function::GammaInvCompat, None),
    (Function::PoissonDist, None),
    (Function::PoissonCompat, None),
    (Function::HypGeomDist, None),
    (Function::Besseli, None),
    (Function::Besselj, None),
    (Function::Besselk, None),
    (Function::Bessely, None),
    (Function::Coupdaybs, None),
    (Function::Coupdays, None),
    (Function::Coupdaysnc, None),
    (Function::Coupncd, None),
    (Function::Coupnum, None),
    (Function::Couppcd, None),
    (Function::Accrint, None),
    (Function::Duration, None),
    (Function::Mduration, None),
    (Function::Price, None),
    (Function::Yield, None),
    (Function::Oddfprice, None),
    (Function::Oddfyield, None),
    (Function::Oddlprice, None),
    (Function::Oddlyield, None),
    (Function::Cumipmt, None),
    (Function::Cumprinc, None),
    (Function::Amorlinc, None),
    (Function::Amordegrc, None),
];

/// The entries of `list` that break the rule: `Constant` with no cap named, or a `Weighted`
/// weight of 1 or a `ByValue` estimate of at most 1, which charge no more than `Constant`.
fn violations(list: &[(Function, Option<&str>)]) -> Vec<String> {
    list.iter()
        .filter_map(|(function, cap)| {
            let work = function.arg_kinds()?.work;
            match (work, cap) {
                (KernelWork::Constant, None) => Some(format!(
                    "{function:?} is Constant: make it Weighted or name its fixed cap"
                )),
                (KernelWork::Weighted(weight), _) if weight <= 1 => Some(format!(
                    "{function:?} is Weighted({weight}), no more than Constant"
                )),
                (KernelWork::ByValue(work), _) if work.most() <= 1 => Some(format!(
                    "{function:?} is ByValue charging at most 1, no more than Constant"
                )),
                _ => None,
            }
        })
        .collect()
}

#[test]
fn value_dependent_kernels_are_weighted_or_name_their_cap() {
    assert_eq!(violations(VALUE_DEPENDENT), Vec::<String>::new());
}

// The check catches a `Constant` entry with no cap (PERMUT, seeded) and passes a weighted one.
#[test]
fn a_constant_kernel_without_a_cap_is_caught() {
    let seeded = [(Function::Permut, None), (Function::Rate, None)];
    assert_eq!(
        violations(&seeded),
        ["Permut is Constant: make it Weighted or name its fixed cap"]
    );
}

/// BESSEL's `ByValue` charge (T-1819): an element at the order cap costs `2^16`, so exactly 2,048
/// of them fit under `MAX_LIFTED_CELL_VISITS` (as the flat weight allowed before) and 2,049 do
/// not; an order-0 element costs 1, a text order is charged as at the cap and a past-the-cap
/// order, which the kernel refuses before looping, 1.
#[test]
fn bessel_elements_are_charged_by_order() {
    use super::super::driver::{kernel_units, LiftedArg};
    use crate::calc_result::CalcResult;
    use crate::expressions::parser::ArrayNode;
    use crate::limits::MAX_LIFTED_CELL_VISITS;

    let work = Function::Besselj.arg_kinds().map(|kinds| kinds.work);
    let Some(work @ KernelWork::ByValue(ValueWork::Loop { arg: 1, .. })) = work else {
        panic!("BESSELJ is not charged by its order: {work:?}");
    };
    let column = |orders: &[ArrayNode]| {
        vec![
            LiftedArg::Scalar(CalcResult::Number(2.5)),
            LiftedArg::Grid(orders.iter().map(|n| vec![n.clone()]).collect()),
        ]
    };
    let at_cap = |count: usize| column(&vec![ArrayNode::Number(100_000.0); count]);
    let units = |args: &[LiftedArg], rows: usize| kernel_units(work, args, rows, 1, rows as u64);
    assert_eq!(units(&at_cap(2048), 2048), MAX_LIFTED_CELL_VISITS);
    assert!(units(&at_cap(2049), 2049) > MAX_LIFTED_CELL_VISITS);
    assert_eq!(units(&column(&[ArrayNode::Number(0.0)]), 1), 1);
    assert_eq!(units(&column(&[ArrayNode::Number(1.9)]), 1), 1);
    assert_eq!(units(&column(&[ArrayNode::Number(-5.0)]), 1), 1);
    assert_eq!(units(&column(&[ArrayNode::Number(100_000.9)]), 1), 1 << 16);
    assert_eq!(units(&column(&[ArrayNode::Number(100_001.0)]), 1), 1);
    assert_eq!(units(&column(&[ArrayNode::Number(1E300)]), 1), 1);
    assert_eq!(units(&column(&[ArrayNode::Number(f64::INFINITY)]), 1), 1);
    assert_eq!(units(&column(&[ArrayNode::String("7".into())]), 1), 1 << 16);
    assert_eq!(units(&column(&[ArrayNode::Boolean(true)]), 1), 1);
    // A scalar order over a lifted x: every element is charged that order.
    let scalar = [
        LiftedArg::Grid(vec![vec![ArrayNode::Number(1.0)]; 10]),
        LiftedArg::Scalar(CalcResult::Number(100_000.0)),
    ];
    assert_eq!(kernel_units(work, &scalar, 10, 1, 10), 10 << 16);
}

/// FIND's and SEARCH's `ByValue` charge (T-1961): each element is charged both texts' lengths
/// plus, at each start position of `within_text`, 1 and the bytes compared there over the
/// entry's `per_visit`; a number is charged as its longest text and an error as the empty text.
#[test]
fn text_search_elements_are_charged_by_their_texts() {
    use super::super::driver::{kernel_units, LiftedArg};
    use crate::calc_result::CalcResult;
    use crate::expressions::parser::ArrayNode;
    use crate::expressions::token::Error;
    use crate::limits::MAX_LIFTED_CELL_VISITS;

    for (function, per_visit) in [
        (Function::Find, 256),
        (Function::Findb, 256),
        (Function::Search, 8),
        (Function::Searchb, 8),
    ] {
        let work = function.arg_kinds().map(|kinds| kinds.work);
        let Some(
            work @ KernelWork::ByValue(ValueWork::TextScan {
                find: 0, within: 1, ..
            }),
        ) = work
        else {
            panic!("{function:?} is not charged by its texts: {work:?}");
        };
        let charge = |find: CalcResult, within: ArrayNode| {
            let args = [LiftedArg::Scalar(find), LiftedArg::Grid(vec![vec![within]])];
            kernel_units(work, &args, 1, 1, 1)
        };
        let text = |len: usize| "a".repeat(len);
        let short = charge(
            CalcResult::String("b".into()),
            ArrayNode::String(text(9).into()),
        );
        assert_eq!(short, 2 + 10, "{function:?}");
        let long = charge(
            CalcResult::String(text(32_767).into()),
            ArrayNode::String(text(32_767).into()),
        );
        assert_eq!(
            long,
            32_768 + 32_768 * (1 + 32_767 / per_visit),
            "{function:?}"
        );
        assert!(long * 1000 > MAX_LIFTED_CELL_VISITS, "{function:?}");
        let number = charge(CalcResult::Number(1.5), ArrayNode::String(text(99).into()));
        assert_eq!(number, 33 + 100 * (1 + 32 / per_visit), "{function:?}");
        let error = charge(
            CalcResult::String(text(10).into()),
            ArrayNode::Error(Error::NA),
        );
        assert_eq!(error, 11 + 1, "{function:?}");
    }
}

/// DB's and VDB's `ByValue` charge (T-1820): an element at the period cap costs `2^17`, so exactly
/// 1,024 of them fit under `MAX_LIFTED_CELL_VISITS` and 1,025 do not.
#[test]
fn depreciation_elements_are_charged_by_period() {
    use super::super::driver::{kernel_units, LiftedArg};
    use crate::calc_result::CalcResult;
    use crate::expressions::parser::ArrayNode;
    use crate::limits::MAX_LIFTED_CELL_VISITS;

    for (function, arg) in [(Function::Db, 3), (Function::Vdb, 4)] {
        let work = function.arg_kinds().map(|kinds| kinds.work);
        let Some(work @ KernelWork::ByValue(ValueWork::Loop { arg: charged, .. })) = work else {
            panic!("{function:?} is not charged by its period: {work:?}");
        };
        assert_eq!(charged, arg, "{function:?}");
        let at_cap = |count: usize| {
            let mut args: Vec<LiftedArg> = (0..arg)
                .map(|_| LiftedArg::Scalar(CalcResult::Number(1.0)))
                .collect();
            args.push(LiftedArg::Grid(vec![
                vec![ArrayNode::Number(100_000.0)];
                count
            ]));
            kernel_units(work, &args, count, 1, count as u64)
        };
        assert_eq!(at_cap(1024), MAX_LIFTED_CELL_VISITS, "{function:?}");
        assert!(at_cap(1025) > MAX_LIFTED_CELL_VISITS, "{function:?}");
    }
}

/// The coupon functions' `ByValue` charge (T-1846): an element is charged its walks in proportion
/// to the periods between its earliest and latest date, so exactly 1,024 COUP* elements across
/// the whole calendar quarterly fit under `MAX_LIFTED_CELL_VISITS` (as the flat weight allowed
/// before) and 1,025 do not, while a short schedule costs a few units. A text date is charged as
/// the longest schedule, and an error or an invalid frequency, which the kernel refuses before it
/// walks, 1.
#[test]
fn coupon_elements_are_charged_by_schedule_length() {
    use super::super::driver::{kernel_units, LiftedArg};
    use super::super::kinds::COUPON_SPAN_DAYS;
    use crate::calc_result::CalcResult;
    use crate::expressions::parser::ArrayNode;
    use crate::limits::MAX_LIFTED_CELL_VISITS;

    let work = |function: Function| {
        let work = function.arg_kinds().map(|kinds| kinds.work);
        let Some(work @ KernelWork::ByValue(ValueWork::CouponWalks { .. })) = work else {
            panic!("{function:?} is not charged by its schedule: {work:?}");
        };
        work
    };
    let number = |n: f64| LiftedArg::Scalar(CalcResult::Number(n));
    // COUPNUM over `count` settlements in a column, the maturity and frequency scalars.
    let coupnum = |settlement: ArrayNode, maturity: f64, frequency: LiftedArg, count: usize| {
        let args = [
            LiftedArg::Grid(vec![vec![settlement]; count]),
            number(maturity),
            frequency,
        ];
        kernel_units(work(Function::Coupnum), &args, count, 1, count as u64)
    };
    let longest = COUPON_SPAN_DAYS as f64;
    let quarterly = || number(4.0);
    assert_eq!(
        coupnum(ArrayNode::Number(0.0), longest, quarterly(), 1024),
        MAX_LIFTED_CELL_VISITS
    );
    assert!(coupnum(ArrayNode::Number(0.0), longest, quarterly(), 1025) > MAX_LIFTED_CELL_VISITS);
    // DATE(9990,6,30) from serial 1, the bench's case: 2 walks of 65,460 units.
    assert_eq!(
        coupnum(ArrayNode::Number(1.0), 2_954_994.0, quarterly(), 1),
        2 * 65_460
    );
    // 2007-01-01 to 2020-01-01 semiannually: 2 walks of 55 units (26 periods).
    assert_eq!(
        coupnum(ArrayNode::Number(39_083.0), 43_831.0, number(2.0), 1),
        2 * 55
    );
    // A settlement a day before the maturity: a walk's one period, 3 units.
    assert_eq!(
        coupnum(ArrayNode::Number(43_830.0), 43_831.0, number(1.0), 1),
        2 * 3
    );
    // A settlement after the maturity is #NUM! before any walk, but the estimate does not read
    // the argument order: it is charged its span, an over-estimate.
    assert_eq!(
        coupnum(ArrayNode::Number(50_000.0), 43_831.0, number(1.0), 1),
        2 * 37
    );
    let text = || ArrayNode::String("2020-01-01".into());
    assert_eq!(coupnum(text(), 43_831.0, number(1.0), 1), 2 << 16);
    assert_eq!(
        coupnum(
            ArrayNode::Number(0.0),
            longest,
            LiftedArg::Scalar(CalcResult::String("4".into())),
            1
        ),
        2 << 16
    );
    let error = ArrayNode::Error(crate::expressions::token::Error::NA);
    assert_eq!(coupnum(error, longest, quarterly(), 1), 1);
    assert_eq!(coupnum(ArrayNode::Number(0.0), longest, number(3.0), 1), 1);
    assert_eq!(
        coupnum(ArrayNode::Number(0.0), longest, number(4.9), 1),
        2 << 16
    );
    assert_eq!(
        coupnum(
            ArrayNode::Number(0.0),
            longest,
            LiftedArg::Scalar(CalcResult::EmptyArg),
            1
        ),
        1
    );
    // A blank settlement reads as serial 0.
    assert_eq!(coupnum(ArrayNode::Empty, longest, quarterly(), 1), 2 << 16);

    // PRICE walks four times; ACCRINT's span runs over its three dates.
    let price = [
        LiftedArg::Grid(vec![vec![ArrayNode::Number(39_001.0)]; 513]),
        number(43_831.0),
        number(0.05),
        number(0.04),
        number(100.0),
        number(2.0),
    ];
    let units = kernel_units(work(Function::Price), &price, 513, 1, 513);
    assert_eq!(units, 513 * 4 * 56);
    let accrint = [
        number(39_083.0),
        LiftedArg::Grid(vec![vec![ArrayNode::Number(43_831.0)]]),
        number(39_448.0),
        number(0.1),
        number(1000.0),
        number(2.0),
    ];
    assert_eq!(
        kernel_units(work(Function::Accrint), &accrint, 1, 1, 1),
        2 * 55
    );
}

/// YIELD's and the ODD* functions' `ByValue` charge (T-1869): the solver's walk and price sums are
/// one pass charged `2^24` at the longest schedule, so 8 YIELD elements there fit and 9 do not,
/// while a schedule of one period still costs its fixed part (524 units); a span from a negative
/// date is widened by the 1904 shift, which a negative date does not take.
#[test]
fn yield_and_odd_elements_are_charged_by_schedule_length() {
    use super::super::driver::{kernel_units, LiftedArg};
    use super::super::kinds::COUPON_SPAN_DAYS;
    use crate::calc_result::CalcResult;
    use crate::expressions::parser::ArrayNode;
    use crate::limits::MAX_LIFTED_CELL_VISITS;

    let number = |n: f64| LiftedArg::Scalar(CalcResult::Number(n));
    let yield_units = |settlement: f64, maturity: f64, frequency: f64, count: usize| {
        let args = [
            LiftedArg::Grid(vec![vec![ArrayNode::Number(settlement)]; count]),
            number(maturity),
            number(0.05),
            number(95.0),
            number(100.0),
            number(frequency),
        ];
        let work = Function::Yield.arg_kinds().map(|kinds| kinds.work);
        let Some(work @ KernelWork::ByValue(ValueWork::CouponWalks { .. })) = work else {
            panic!("YIELD is not charged by its schedule: {work:?}");
        };
        kernel_units(work, &args, count, 1, count as u64)
    };
    let longest = COUPON_SPAN_DAYS as f64;
    assert_eq!(yield_units(0.0, longest, 4.0, 8), MAX_LIFTED_CELL_VISITS);
    assert!(yield_units(0.0, longest, 4.0, 9) > MAX_LIFTED_CELL_VISITS);
    assert_eq!(yield_units(43_830.0, 43_831.0, 4.0, 1), 524);
    // The Goal's call: 9 elements of about 13 years semiannually.
    assert_eq!(yield_units(39_001.0, 43_831.0, 2.0, 9), 9 * 14_213);
    // -1 to 10 is 11 days as written, but up to 2,935 in a 1904 workbook.
    assert_eq!(
        yield_units(-1.0, 10.0, 4.0, 1),
        yield_units(0.0, 2_935.0, 4.0, 1)
    );
}

/// HYPGEOM.DIST's `ByValue` charge (T-2433): an element is charged the larger of its walk (2^14)
/// and its cdf sum, 16 units a term over min(sample_s, number_sample, population_s) + 1 terms
/// and 131 window exponents, at most the cap's bound 2^22, so exactly 32 elements at the bound
/// fit under `MAX_LIFTED_CELL_VISITS` (as the flat weight allowed before) and 33 do not. A text
/// count bounds nothing, so the least of the other counts is charged, the bound when every count
/// is a text; an error, a negative count or one of 2^31 - 1 or more,
/// which the kernel refuses before either path, 1.
#[test]
fn hypgeom_dist_elements_are_charged_by_their_counts() {
    use super::super::driver::{kernel_units, LiftedArg};
    use crate::calc_result::CalcResult;
    use crate::expressions::parser::ArrayNode;
    use crate::limits::MAX_LIFTED_CELL_VISITS;

    let work = Function::HypGeomDist.arg_kinds().map(|kinds| kinds.work);
    let Some(work @ KernelWork::ByValue(ValueWork::HypergeometricSum { .. })) = work else {
        panic!("HYPGEOM.DIST is not charged by its counts: {work:?}");
    };
    // sample_s lifted over `samples`, number_sample 1E8, population_s 1E8, number_pop 2E8.
    let column = |samples: &[ArrayNode]| {
        vec![
            LiftedArg::Grid(samples.iter().map(|n| vec![n.clone()]).collect()),
            LiftedArg::Scalar(CalcResult::Number(1E8)),
            LiftedArg::Scalar(CalcResult::Number(1E8)),
            LiftedArg::Scalar(CalcResult::Number(2E8)),
            LiftedArg::Scalar(CalcResult::Boolean(true)),
        ]
    };
    let one = |sample: ArrayNode| kernel_units(work, &column(&[sample]), 1, 1, 1);
    let at_bound = |count: usize| column(&vec![ArrayNode::Number(5E7); count]);
    let units = |args: &[LiftedArg], rows: usize| kernel_units(work, args, rows, 1, rows as u64);
    assert_eq!(units(&at_bound(32), 32), MAX_LIFTED_CELL_VISITS);
    assert!(units(&at_bound(33), 33) > MAX_LIFTED_CELL_VISITS);
    // The walk's floor, then 16 units a term.
    assert_eq!(one(ArrayNode::Number(0.0)), 1 << 14);
    assert_eq!(one(ArrayNode::Empty), 1 << 14);
    assert_eq!(one(ArrayNode::Boolean(true)), 1 << 14);
    assert_eq!(one(ArrayNode::Number(892.0)), 1 << 14);
    assert_eq!(one(ArrayNode::Number(893.0)), (893 + 1 + 131) * 16);
    assert_eq!(one(ArrayNode::Number(1000.9)), (1000 + 1 + 131) * 16);
    assert_eq!(one(ArrayNode::Number(262_144.0)), 1 << 22);
    // Refused before either path, or read as any count.
    assert_eq!(one(ArrayNode::Number(-0.5)), 1);
    assert_eq!(one(ArrayNode::Number(2_147_483_647.0)), 1);
    assert_eq!(one(ArrayNode::Number(2_147_483_646.9)), 1 << 22);
    assert_eq!(
        one(ArrayNode::Error(crate::expressions::token::Error::NA)),
        1
    );
    assert_eq!(one(ArrayNode::String("7".into())), 1 << 22);
    // A text beside a small count: that count bounds the window (gen-m9/a013a45a8fbcab4f A169,
    // `HYPGEOM.DIST(3,PIVOTBY(..),10,3,..)` over the PIVOTBY's header texts).
    let text_counts = |sample: f64| {
        vec![
            LiftedArg::Scalar(CalcResult::Number(sample)),
            LiftedArg::Grid(vec![vec![ArrayNode::String("Total".into())]]),
            LiftedArg::Scalar(CalcResult::String("1E8".into())),
            LiftedArg::Scalar(CalcResult::Number(2E8)),
            LiftedArg::Scalar(CalcResult::Boolean(true)),
        ]
    };
    assert_eq!(kernel_units(work, &text_counts(3.0), 1, 1, 1), 1 << 14);
    assert_eq!(kernel_units(work, &text_counts(1E6), 1, 1, 1), 1 << 22);
    let all_texts = [
        LiftedArg::Scalar(CalcResult::String("3".into())),
        LiftedArg::Grid(vec![vec![ArrayNode::String("3".into())]]),
        LiftedArg::Scalar(CalcResult::String("3".into())),
    ];
    assert_eq!(kernel_units(work, &all_texts, 1, 1, 1), 1 << 22);
    // The least count bounds the window: a small population_s over a huge sample_s.
    let small_successes = [
        LiftedArg::Grid(vec![vec![ArrayNode::Number(1E8)]; 4]),
        LiftedArg::Scalar(CalcResult::Number(1E8)),
        LiftedArg::Scalar(CalcResult::Number(3000.0)),
        LiftedArg::Scalar(CalcResult::Number(2E8)),
        LiftedArg::Scalar(CalcResult::Boolean(true)),
    ];
    assert_eq!(
        kernel_units(work, &small_successes, 4, 1, 4),
        4 * (3000 + 132) * 16
    );
}
