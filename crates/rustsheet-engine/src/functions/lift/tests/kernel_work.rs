//! SPEC-0011 "Security rule for kernel work" (T-1893): a kernel whose work could depend on its
//! argument values is `KernelWork::Weighted` in the lifting table, or `Constant` with its fixed
//! cap named below, so no family task can enter one as `Constant` silently.

use super::super::kinds::KernelWork;
use crate::functions::Function;

/// Where the incomplete beta's cap is: statrs' `checked_beta_reg` runs at most 140
/// continued-fraction steps, and `ln_gamma` is a fixed Lanczos sum.
const BETA_CDF: &str = "one statrs incomplete beta: at most 140 continued-fraction steps";

/// The value-dependent kernels: each with the fixed cap that lets its entry be `Constant`, or
/// `None` when it must be `Weighted` (a cap that grows with the values, or one not yet read).
/// A function with no table entry yet is checked once it is entered.
const VALUE_DEPENDENT: &[(Function, Option<&str>)] = &[
    // Fixed caps, read in each kernel.
    (Function::BinomDist, Some(BETA_CDF)),
    (Function::BinomDistCompat, Some(BETA_CDF)),
    (
        Function::BinomDistRange,
        Some("two statrs incomplete betas of at most 140 steps each"),
    ),
    (Function::NegbinomDist, Some(BETA_CDF)),
    (
        Function::NegbinomDistCompat,
        Some("the pmf: ln_gamma's fixed Lanczos sum"),
    ),
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
        Function::Factdouble,
        Some("descending_product stops at the first infinity: n*(n-2)*... overflows in 151 steps"),
    ),
    (
        Function::Permut,
        Some("at most 171 factors: 171 distinct integers pass f64::MAX"),
    ),
    (Function::Permutationa, Some("O(1): one powf")),
    (
        Function::Erf,
        Some("special::erfc_cf: at most 600 steps; the series is fixed"),
    ),
    (
        Function::ErfPrecise,
        Some("special::erfc_cf: at most 600 steps; the series is fixed"),
    ),
    (
        Function::Erfc,
        Some("special::erfc_cf: at most 600 steps; the series is fixed"),
    ),
    (
        Function::ErfcPrecise,
        Some("special::erfc_cf: at most 600 steps; the series is fixed"),
    ),
    (
        Function::Seriessum,
        Some("one step per coefficient, charged per element as their cells (LiftCost::Cells)"),
    ),
    // Weighted: solvers, searches, series and walks whose length the values set.
    (Function::Rate, None),
    (Function::Irr, None),
    (Function::Xirr, None),
    (Function::Db, None),
    (Function::Vdb, None),
    (Function::Fact, None),
    (Function::Combin, None),
    (Function::Combina, None),
    (Function::Multinomial, None),
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
/// weight of 1, which charges no more than `Constant`.
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
