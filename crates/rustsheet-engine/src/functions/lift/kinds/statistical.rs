//! The lifting table's statistical entries (SPEC-0011 "The argument-kind table"): LARGE and the
//! other functions over a data argument, the scalar distributions, and the weights of the
//! quantile searches, the incomplete gamma and HYPGEOM.DIST's cdf sum.

use super::ArgKind::{Lift, LiftArrays, LiftArraysRangeWhole, Whole, WholeByCaller};
use super::KernelWork::{Constant, Weighted};
use super::{ArgKinds, LIFT_1, LIFT_2, LIFT_3, LIFT_3_OF_5, LIFT_4, LIFT_4_OF_6};
use crate::functions::Function;

/// A data argument its function reads itself, then one lifted worksheet argument:
/// `LARGE(array, k)`, `PERCENTILE(array, k)`, `QUARTILE(array, quart)`, `TRIMMEAN(array, percent)`.
const DATA_THEN_1: ArgKinds = ArgKinds::fixed(2, &[WholeByCaller, Lift], Constant);
/// The same with an optional second lifted argument: `PERCENTRANK(array, x, [significance])`.
const DATA_THEN_1_OF_2: ArgKinds = ArgKinds::fixed(2, &[WholeByCaller, Lift, Lift], Constant);
/// Two data arguments read by their function, then one or two lifted limits:
/// `PROB(x_range, prob_range, lower_limit, [upper_limit])`.
const TWO_DATA_THEN_1_OF_2: ArgKinds =
    ArgKinds::fixed(3, &[WholeByCaller, WholeByCaller, Lift, Lift], Constant);
/// Two data arguments read by their function, then two lifted arguments:
/// `T.TEST(array1, array2, tails, type)`.
const TWO_DATA_THEN_2: ArgKinds =
    ArgKinds::fixed(4, &[WholeByCaller, WholeByCaller, Lift, Lift], Constant);
/// One lifted argument, then two data arguments read by their function:
/// `FORECAST(x, known_y's, known_x's)`.
const ONE_THEN_TWO_DATA: ArgKinds =
    ArgKinds::fixed(3, &[Lift, WholeByCaller, WholeByCaller], Constant);

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

/// `DSUM(database, field, criteria)` and the other D-functions: the database is read whole, `field`
/// lifts over an array and a two-cell range is `#VALUE!`, `criteria` lifts over an array and a range
/// is the criteria table, read whole (probes/db-argument-modes D11:D22, F11:F23). Each element
/// scans the database, so `Model::db_call` charges the database's cells per element
/// (`LiftCost::Cells`).
const DATABASE: ArgKinds = ArgKinds::fixed(3, &[Whole, LiftArrays, LiftArraysRangeWhole], Constant);

/// This family's part of the lifting table (`Function::arg_kinds`).
pub(super) fn arg_kinds(function: &Function) -> Option<&'static ArgKinds> {
    match function {
        // The scalar arguments of these worksheet functions lift over an array and over a
        // multi-cell range, also PERCENTRANK's significance; each element is the scalar call
        // (probes/stat-argument-modes H345-H463, H549-H550; probes/stat-whole-lift W1-W48;
        // statistics.md "Statistical function argument modes"). The data argument is read by
        // the function's own reader (`statistical/sorted_data.rs`) and sorted once per call.
        // A kernel looks up its sorted numbers in O(1) or by binary search (PERCENTRANK); the
        // call pays for the numbers through its `LiftCost`.
        // The D-functions: see `DATABASE`.
        Function::Daverage
        | Function::Dcount
        | Function::Dcounta
        | Function::Dget
        | Function::Dmax
        | Function::Dmin
        | Function::Dproduct
        | Function::Dstdev
        | Function::Dstdevp
        | Function::Dsum
        | Function::Dvar
        | Function::Dvarp => Some(&DATABASE),
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
        // Z.TEST's `x` and `sigma` and PROB's limits lift over an array and a range, the data is
        // read once by the function (probes/stat-argument-modes H488, H489, H566, H567, H571,
        // H572; probes/stat-test-lift; T-1813). A Z.TEST element is O(1) over the data's
        // count, mean and deviation; a PROB element walks the data (`LiftCost::Cells`).
        Function::ZTest | Function::ZTestCompat => Some(&DATA_THEN_1_OF_2),
        Function::Prob => Some(&TWO_DATA_THEN_1_OF_2),
        // FORECAST's `x` and T.TEST's `tails` and `type` lift likewise, the data is read once
        // by the function (probes/stat-test-lift P1-P9, P60-P67, P88; probes/stat-argument-modes
        // H260, H261, H264, H265, H492-H499, H537-H545; T-1982). A FORECAST element is O(1) over
        // the regression line; a T.TEST element walks the samples (`LiftCost::Cells`).
        Function::Forecast | Function::ForecastLinear => Some(&ONE_THEN_TWO_DATA),
        Function::TTest | Function::TTestCompat => Some(&TWO_DATA_THEN_2),
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
