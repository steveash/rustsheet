//! The lifting table's statistical entries (SPEC-0011 "The argument-kind table"): LARGE and the
//! other functions over a data argument, the scalar distributions, and the weights of the
//! quantile searches, the incomplete gamma and HYPGEOM.DIST's cdf sum.

use super::ArgKind::{
    Lift, LiftArrays, LiftArraysRangeWhole, ReferenceByCaller, Whole, WholeByCaller,
};
use super::KernelWork::{ByValue, Constant, Weighted};
use super::ValueWork::{HypergeometricSum, IncompleteGamma};
use super::{
    ArgKinds, GammaShape, KernelWork, LIFT_1, LIFT_2, LIFT_3, LIFT_3_OF_5, LIFT_4, LIFT_4_OF_6,
};
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
/// `FORECAST.ETS(target_date, values, timeline, [seasonality], [data_completion], [aggregation])`:
/// the target date and the scalar settings lift, the series and timeline are read by the function
/// (T-2028); a setting is not read by the placeholder kernel.
const ETS_TARGET: ArgKinds = ArgKinds::fixed(
    3,
    &[Lift, WholeByCaller, WholeByCaller, Lift, Lift, Lift],
    Constant,
);
/// `FORECAST.ETS.CONFINT(target_date, values, timeline, [confidence_level], [seasonality],
/// [data_completion], [aggregation])`.
const ETS_CONFINT_TARGET: ArgKinds = ArgKinds::fixed(
    3,
    &[Lift, WholeByCaller, WholeByCaller, Lift, Lift, Lift, Lift],
    Constant,
);
/// `FORECAST.ETS.STAT(values, timeline, statistic_type, [seasonality], [data_completion],
/// [aggregation])`: the statistic type lifts.
const ETS_STAT_TYPE: ArgKinds = ArgKinds::fixed(
    3,
    &[WholeByCaller, WholeByCaller, Lift, Lift, Lift, Lift],
    Constant,
);
/// `FORECAST.ETS.SEASONALITY(values, timeline, [data_completion], [aggregation])`.
const ETS_SEASONALITY: ArgKinds =
    ArgKinds::fixed(2, &[WholeByCaller, WholeByCaller, Lift, Lift], Constant);
/// A lifted argument, a reference read by its function, then an optional lifted argument:
/// `RANK(number, ref, [order])`.
const ONE_THEN_REFERENCE_THEN_1: ArgKinds =
    ArgKinds::fixed(2, &[Lift, ReferenceByCaller, Lift], Constant);

// --- Statistical (T-1290) -------------------------------------------------------------------
/// What one BINOM.DIST, BINOMDIST, BINOM.DIST.RANGE, NEGBINOM.DIST or NEGBINOMDIST element is
/// charged. Each sums its pmf terms in double-double (`statistical/discrete.rs`, T-0927): at most
/// two walks of `discrete::MAX_STEPS` (2^10) steps at about 32 ns, 66 us, then, past them, statrs'
/// incomplete beta (BINOM.DIST.RANGE: two of them, 6.4 us at 1E18 trials). The weight is
/// BINOM.INV's for the same bound, about 70 us: 32,768 elements a lifted call, about 2.3 s (a
/// DEVIATION, statistics.md "Discrete distributions: within an ulp of the exact sum").
const DISCRETE_SUM_WEIGHT: u32 = 1 << 12;
/// `NEGBINOMDIST(number_f, number_s, probability_s)`.
const DISCRETE_SUM_3: ArgKinds =
    ArgKinds::fixed(3, &[Lift, Lift, Lift], Weighted(DISCRETE_SUM_WEIGHT));
/// `BINOM.DIST(number_s, trials, probability_s, cumulative)`, BINOMDIST and NEGBINOM.DIST.
const DISCRETE_SUM_4: ArgKinds =
    ArgKinds::fixed(4, &[Lift, Lift, Lift, Lift], Weighted(DISCRETE_SUM_WEIGHT));
/// `BINOM.DIST.RANGE(trials, probability_s, number_s, [number_s2])`.
const DISCRETE_SUM_3_OF_4: ArgKinds =
    ArgKinds::fixed(3, &[Lift, Lift, Lift, Lift], Weighted(DISCRETE_SUM_WEIGHT));
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
/// adjustments, its own per-call cap: 1.6 ms at full cost), refined to the crossing of the T-0970
/// incomplete beta (`special::refine_increasing`: at most 129 of its cdfs, each at most two
/// continued fractions of 300 steps, about twice statrs' cost: 0.9 ms more; T-0974, T-2119), or, at
/// a huge shape or df, a crossing of at most 67 of those cdfs (`inverse_beta::beta_quantile_crossing`,
/// T-1538: 0.5 ms), so the full Newton search plus refinement (2.5 ms) is the bound.
/// The weight caps a lifted call at 2^27 / 2^18 = 512 elements: 1.7 s at the bound (a DEVIATION,
/// statistics.md). The cases measured cost much less (over 8,192 elements in release: 167 us for
/// `BETA.INV(1E-300,0.5,1E6)`, a full bisection of fast cdfs; 105 us for the longest Newton search
/// seen before T-0974, 377 steps, `F.INV(3.718810279553989E-162,4532,43)`; 1 to 2 us for
/// `BETA.INV(0.4,2,3)`).
const QUANTILE_WEIGHT: u32 = 1 << 18;
/// `BETA.INV(probability, alpha, beta, [A], [B])` and BETAINV.
const BETA_INV: ArgKinds = ArgKinds::fixed(
    3,
    &[Lift, Lift, Lift, Lift, Lift],
    Weighted(QUANTILE_WEIGHT),
);
/// `F.INV(probability, deg_freedom1, deg_freedom2)`, F.INV.RT and FINV.
const F_INV: ArgKinds = ArgKinds::fixed(3, &[Lift, Lift, Lift], Weighted(QUANTILE_WEIGHT));
/// What one T.INV or CONFIDENCE.T element is charged. From df 2 its quantile is a crossing of the
/// tail at y (`t_dist::t_rounded_y_quantile`, T-2511): about 25 to 30 tails seeded by statrs' search,
/// at most 66 without a seed (q below 1E-8) and about 81 with a poor one, each tail up to about 30 us where it sums the
/// integer-df series (df up to 1000). Measured in release (T-2511): 1.53 ms for `T.INV(1E-12,999)`,
/// the worst seen, about 1.0 ms at df 999 and 1000 otherwise, 0.47 ms at df 500, under 0.2 ms at
/// df 99 and from df 1001. At df 1 it is the capped statrs port (0.21 ms) or a bisection of at most
/// 264 cdfs (0.82 ms). The weight caps a lifted call at 2^27 / 2^17 = 1,024 elements: about 1.6 s
/// at the worst case seen (a DEVIATION, statistics.md). It assumes a good seed: a poor one at df
/// near 1000 would cost about 2.5 ms a call; hoisting the per-df work out of the tail is T-3399's.
const T_QUANTILE_WEIGHT: u32 = 1 << 17;
/// `T.INV(probability, deg_freedom)`, T.INV.2T and TINV.
const T_INV: ArgKinds = ArgKinds::fixed(2, &[Lift, Lift], Weighted(T_QUANTILE_WEIGHT));
/// `CONFIDENCE.T(alpha, standard_dev, size)`.
const CONFIDENCE_T: ArgKinds = ArgKinds::fixed(3, &[Lift, Lift, Lift], Weighted(T_QUANTILE_WEIGHT));
// --- End statistical quantiles (T-1848) -----------------------------------------------------

// --- Incomplete gamma (T-1872) --------------------------------------------------------------
/// What one GAMMA.DIST, GAMMA.INV, CHISQ.* or POISSON element is charged. Each runs one capped
/// capped incomplete gamma (`statistical/incomplete_gamma`): a cdf of at most 2^19 series or
/// continued-fraction terms, or an inversion whose cdfs (at most 129 of them, the refinement from
/// a Wilson-Hilferty start, T-3131; statrs' search ran up to about 2,250 before) share one budget
/// of 2^19 terms. Measured in release (T-1872, load 2 to 5 on 4 cores) over 512 elements at the cap: a
/// continued-fraction term costs 4.1 to 4.7 ns on its NaN path (2.4 ms for 2^19,
/// `GAMMA.DIST(1.0000001E300,1E300,1,TRUE)`, whose terms overflow at once) and 2.4 ns where it
/// converges and rescales (1.3 ms, `GAMMA.DIST(1000000000000001,1E15,1,TRUE)`, which reaches the
/// cap at a shape of 1E15 and rescales on about 65% of its terms), a series term 2.5 ns (1.3 ms,
/// `GAMMA.DIST(1E12,1E12,1,TRUE)`), and a cdf's own work 49 ns (`GAMMA.INV(1E-300,0.001,1E-300)`,
/// 1,060 cdfs: 52 us). The bound of an element is 2^19 terms at the slowest, 4.7 ns, plus 129 cdfs at
/// 49 ns, about 2.5 ms (2.6 ms with statrs' 2,250 cdfs before T-3131), so the 2^18 weight still
/// holds: it caps a lifted call at 2^27 / 2^18 = 512 elements, 1.3 s at the bound (a DEVIATION,
/// statistics.md). Measured cases cost less: 2.4 ms for the
/// continued fraction at the cap, 1.2 ms for `CHISQ.DIST(1E10,1E10,TRUE)` (481,117 terms),
/// 1.6 us for `GAMMA.INV(0.4,2,3)`. It is the most an element is charged; each is charged by
/// its shape below it (`incomplete_gamma`, T-2795, at the 4.7 ns and 49 ns above:
/// `INCOMPLETE_GAMMA_TERM_COST` and `INCOMPLETE_GAMMA_CDF_COST` in mod.rs).
const INCOMPLETE_GAMMA_WEIGHT: u32 = 1 << 18;
/// An incomplete-gamma entry charged by value (T-2795): at most `INCOMPLETE_GAMMA_WEIGHT` an
/// element, in proportion to the bound of its shape (`ValueWork::IncompleteGamma`), so a lift
/// of small shapes runs far more than 512 elements (gen-m9/c2974c061384fdd9 A56, a CHIDIST over
/// a 731-element PIVOTBY, was `#ERROR!` where Excel spills): `CHIDIST(x,5)` is charged 150
/// units, a CHISQ.INV.RT of df 10 the bound unless its probability is outside (0, 1).
const fn incomplete_gamma(
    args: usize,
    shape: usize,
    shape_of: GammaShape,
    probability: Option<usize>,
) -> KernelWork {
    ByValue(IncompleteGamma {
        args,
        shape,
        shape_of,
        probability,
        weight: INCOMPLETE_GAMMA_WEIGHT,
    })
}
/// `CHISQ.DIST.RT(x, deg_freedom)` and CHIDIST.
const CHISQ_DIST_RT: ArgKinds = ArgKinds::fixed(
    2,
    &[Lift, Lift],
    incomplete_gamma(2, 1, GammaShape::HalfDegrees, None),
);
/// `CHISQ.INV(probability, deg_freedom)`, CHISQ.INV.RT and CHIINV.
const CHISQ_INV: ArgKinds = ArgKinds::fixed(
    2,
    &[Lift, Lift],
    incomplete_gamma(2, 1, GammaShape::HalfDegrees, Some(0)),
);
/// `CHISQ.DIST(x, deg_freedom, cumulative)`.
const CHISQ_DIST: ArgKinds = ArgKinds::fixed(
    3,
    &[Lift, Lift, Lift],
    incomplete_gamma(3, 1, GammaShape::HalfDegrees, None),
);
/// `POISSON.DIST(x, mean, cumulative)` and POISSON: Q(trunc(x) + 1, mean).
const POISSON: ArgKinds = ArgKinds::fixed(
    3,
    &[Lift, Lift, Lift],
    incomplete_gamma(3, 0, GammaShape::PoissonCount, None),
);
/// `GAMMA.INV(probability, alpha, beta)` and GAMMAINV.
const GAMMA_INV: ArgKinds = ArgKinds::fixed(
    3,
    &[Lift, Lift, Lift],
    incomplete_gamma(3, 1, GammaShape::Shape, Some(0)),
);
/// `GAMMA.DIST(x, alpha, beta, cumulative)` and GAMMADIST.
const GAMMA_DIST: ArgKinds = ArgKinds::fixed(
    4,
    &[Lift, Lift, Lift, Lift],
    incomplete_gamma(4, 1, GammaShape::Shape, None),
);
// --- End incomplete gamma (T-1872) ----------------------------------------------------------

// --- Hypergeometric (T-1873) ----------------------------------------------------------------
/// The most one HYPGEOM.DIST element is charged: its capped cdf's bound, (2^18 + 131) terms at
/// 250 ns, about 66 ms, so 32 elements at the bound take about 2.1 s (a DEVIATION; measured in
/// release: hypegeom.rs `MAX_TERMS`).
const HYPGEOM_DIST_WEIGHT: u32 = 1 << 22;
/// The units one term of HYPGEOM.DIST's cdf sum is charged: `HYPGEOM_DIST_WEIGHT` over the
/// (2^18 + 131) terms of its bound, rounded up, so about 250 ns a term (T-2433).
const HYPGEOM_TERM_WEIGHT: u32 = 16;
/// What HYPGEOM.DIST's double-double walk is charged: at most 2^12 steps (T-0977), measured in
/// release at about 120 us an element for the cdf's 4,000-step walk (16,384 elements of
/// `HYPGEOM.DIST(0,4000,4000,1E9,TRUE)` took 1.93 to 2.0 s in benches/lift.rs, T-2433; the pmf's
/// walk is cheaper, 78 us), so twice HYPGEOMDIST's weight: 8,192 elements a lifted call, about 1 s.
const HYPGEOM_WALK_WEIGHT: u32 = 1 << 14;
/// The exponents HYPGEOM.DIST's cdf sum may evaluate beside its terms: its window's bisections
/// (hypegeom.rs `nonzero_terms`).
const HYPGEOM_WINDOW_EXPONENTS: u32 = 131;
/// `HYPGEOM.DIST(sample_s, number_sample, population_s, number_pop, cumulative)`. A cdf or pmf
/// takes either the double-double walk (`HYPGEOM_WALK_WEIGHT`) or the capped sum, never both,
/// and the sum's window is at most `min(sample_s, number_sample, population_s) + 1` terms (above
/// `min(number_sample, population_s)` the cdf is 1 before any sum), so each element is charged
/// the larger of the two by its counts (T-2433; before it, every element was charged the cap's
/// bound, and a lift of 33 small calls was `#ERROR!` where Excel spills:
/// gen-m9/7bd219ec0c0c5e22 A159).
const HYPGEOM_DIST: ArgKinds = ArgKinds::fixed(
    5,
    &[Lift; 5],
    ByValue(HypergeometricSum {
        counts: &[0, 1, 2],
        walk: HYPGEOM_WALK_WEIGHT,
        extra: HYPGEOM_WINDOW_EXPONENTS,
        per_term: HYPGEOM_TERM_WEIGHT,
        weight: HYPGEOM_DIST_WEIGHT,
    }),
);
/// One HYPGEOMDIST element: its pmf walk (`statistical/discrete.rs`, T-0977) of at most 2^12
/// double-double steps, 78 us in release at 4,095 steps, twice BINOM.DIST's walk, so twice its
/// weight: 16,384 elements a lifted call, about 1.3 s. Past the walk's budget the pmf is statrs'
/// closed form.
const HYPGEOMDIST_WEIGHT: u32 = 1 << 13;
const HYPGEOMDIST: ArgKinds = ArgKinds::fixed(4, &[Lift; 4], Weighted(HYPGEOMDIST_WEIGHT));
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
        // The FORECAST.ETS family never computes (statistics.md "FORECAST.ETS family"), but its
        // `target_date` and `statistic_type` lift and read as numbers first (probes/stat-argument-modes
        // H268-H280, probes/forecast-ets-longer-series K38; T-2028).
        Function::ForecastEts => Some(&ETS_TARGET),
        Function::ForecastEtsConfint => Some(&ETS_CONFINT_TARGET),
        Function::ForecastEtsStat => Some(&ETS_STAT_TYPE),
        Function::ForecastEtsSeasonality => Some(&ETS_SEASONALITY),
        // RANK's `number` and `order` lift likewise over a `ref` read once; a `ref` that is not a
        // reference broadcasts as its own shape of errors (probes/stat-test-lift P20-P33, P72;
        // probes/stat-argument-modes H492-H499; T-1999). A lifted call sorts the numbers once
        // (`LiftCost::Prepared`).
        Function::RankCompat | Function::RankEq | Function::RankAvg => {
            Some(&ONE_THEN_REFERENCE_THEN_1)
        }
        // --- Statistical (T-1290) ---
        // The statistical functions whose every argument is scalar are worksheet functions:
        // every argument lifts over an array and over a multi-cell range, on the range's own
        // rows too (probes/stat-argument-modes: every argument over `{v;v}`, the first over
        // a range; probes/stat-later-argument-lift H1-H73: the last and the optional ones
        // over a range, K3-K16: the optional ones over an array; scalar-lift-shapes H15,
        // H16; docs/excel-semantics/statistics.md "Statistical function argument modes").
        // Their kernels are closed form, or call statrs' incomplete beta (a continued
        // fraction of at most 140 steps) or, for the F family, `incomplete_beta` (at most 300
        // steps; 1.95 us at its worst measured, shapes 5E9 = df 1E10, T-0970, still under
        // BINOM.DIST's 3.2 us below) a fixed number of times: measured in release over
        // 2^20 elements, BETA.DIST with shapes of 1E6 costs about 0.29 us an element, F.DIST
        // and T.DIST with 1E9 degrees of freedom 0.28 and 0.30 us, NORM.DIST 0.14 us, and the
        // slowest, BINOM.DIST with 1E18 trials, about 3.2 us (3.3 s for 2^20 elements), and
        // BINOM.DIST.RANGE, which runs two cdfs, 6.4 us (6.7 s for 2^20 elements). The binomial
        // and negative binomial ones also walk their pmf terms first and are weighted (T-0927).
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
        | Function::TDist
        | Function::TDistCompat
        | Function::FDistRT
        | Function::FDistCompat => Some(&LIFT_3),
        Function::NormDist
        | Function::NormDistCompat
        | Function::WeibullDist
        | Function::WeibullCompat
        | Function::LogNormDist
        | Function::FDist => Some(&LIFT_4),
        Function::BinomDist | Function::BinomDistCompat | Function::NegbinomDist => {
            Some(&DISCRETE_SUM_4)
        }
        Function::NegbinomDistCompat => Some(&DISCRETE_SUM_3),
        // `BETA.DIST(x, alpha, beta, cumulative, [A], [B])`, `BETADIST(x, alpha, beta, [A], [B])`.
        Function::BetaDist => Some(&LIFT_4_OF_6),
        Function::BetaDistCompat => Some(&LIFT_3_OF_5),
        Function::BinomDistRange => Some(&DISCRETE_SUM_3_OF_4),
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
        Function::ChisqDistRT | Function::ChiDistCompat => Some(&CHISQ_DIST_RT),
        Function::ChisqInv | Function::ChisqInvRT | Function::ChiInvCompat => Some(&CHISQ_INV),
        Function::ChisqDist => Some(&CHISQ_DIST),
        Function::PoissonDist | Function::PoissonCompat => Some(&POISSON),
        Function::GammaInv | Function::GammaInvCompat => Some(&GAMMA_INV),
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
