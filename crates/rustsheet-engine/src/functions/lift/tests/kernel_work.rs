//! SPEC-0011 "Security rule for kernel work" (T-1893): a kernel whose work could depend on its
//! argument values is `KernelWork::Weighted` (or `ByArg`) in the lifting table, or `Constant`
//! with its fixed cap named below, so no family task can enter one as `Constant` silently. A
//! family task that enters a value-dependent function adds it here. The check does not see a
//! `Weighted` kernel's own per-call cap; its table entry's doc comment cites it.

use super::super::kinds::KernelWork;
use crate::functions::Function;

/// Where the incomplete beta's cap is: statrs' `checked_beta_reg` runs at most 140
/// continued-fraction steps, and `ln_gamma` is a fixed Lanczos sum.
const BETA_CDF: &str = "one statrs incomplete beta: at most 140 continued-fraction steps";

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
/// `None` when it must be `Weighted` or `ByArg` (a cap that grows with the values, or one not yet
/// read).
/// A function with no table entry yet is checked once it is entered.
const VALUE_DEPENDENT: &[(Function, Option<&str>)] = &[
    // Fixed caps, read in each kernel.
    (Function::BetaDist, Some(BETA_CDF)),
    (Function::BetaDistCompat, Some(BETA_CDF)),
    (Function::FDist, Some(BETA_CDF)),
    (Function::FDistRT, Some(BETA_CDF)),
    (Function::FDistCompat, Some(BETA_CDF)),
    (Function::TDist, Some(BETA_CDF)),
    (Function::TDist2T, Some(BETA_CDF)),
    (Function::TDistRT, Some(BETA_CDF)),
    (Function::TDistCompat, Some(BETA_CDF)),
    (
        Function::HypGeomDistCompat,
        Some("the pmf: three statrs binomials, a fixed factorial cache or ln_gamma each"),
    ),
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
/// weight of 1 or a `ByArg` weight of 0, which charge no more than `Constant`.
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
                (KernelWork::ByArg { weight: 0, .. }, _) => Some(format!(
                    "{function:?} is ByArg with weight 0, no more than Constant"
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

/// BESSEL's `ByArg` charge (T-1819): an element at the order cap costs `2^16`, so exactly 2,048
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
    let Some(work @ KernelWork::ByArg { arg: 1, .. }) = work else {
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
