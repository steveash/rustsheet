//! Excel's far upper tail of Q(a, x) at a shape below [`DIRECT_SHAPE_MAX`] (T-3269): POISSON's
//! cdf and CHISQ.DIST.RT (CHIDIST, CHISQ.TEST) past x = [`FAR_TAIL_MIN_X`].
//!
//! There Excel is 10 to 400 ulp from the true Q (probes-numeric/t3284-gamma-deep-upper-tail,
//! docs/excel-semantics/statistics.md "Incomplete gamma: accurate kernel (T-0971)"), and its error
//! is the one T-2638 found above shape 20 away from the mode: the prefactor is Stirling's ratio
//! form sqrt(a / 2 pi) exp(a ln(x/a) + (a - x) - correction(a)) ([`excel_ratio_exponent`],
//! [`stirling_prefactor`]), whose exponent, rounded to a double, carries an ulp of a number near
//! ln Q into the result. Over the 32 adjacent doubles of `POISSON(2,100+j*2^-46,TRUE)` Excel jumps
//! by 64 ulp exactly where that exponent's rounding does. The continued fraction it multiplies is
//! taken accurately ([`accurate_fraction`]); the correction is the exact one at each shape, a
//! multiple of 1/2 here ([`STIRLING_CORRECTION`]), since TOMS's series is only good from 8.
//!
//! At and below [`FAR_TAIL_MIN_X`] Excel is the accurate value at integer shapes (where this form
//! is up to 22 ulp off), so the callers keep their accurate kernels there, and
//! past [`FAR_TAIL_MAX_X`] (unmeasured) too.

use super::fraction::accurate_fraction;
use super::{excel_ratio_exponent, stirling_prefactor, Terms, DIRECT_SHAPE_MAX};

/// The x above which Excel takes the ratio form at a shape below 20. Bracketed by the T-3269
/// boundary probe (probes-numeric/t3269-gamma-tail-switch, x from 26 to 40 by 0.5 at shapes 2 to
/// 4): at shapes 2, 3 and 4 Excel is the correctly rounded Q at every x up to 30 and the ratio
/// form from 30.5 on (`POISSON(2,30,TRUE)` exact, `POISSON(2,30.5,TRUE)` 32 ulp from exact and 3
/// from the form).
/// FITTED(T-3269, 145 cells, x 26..40)
const FAR_TAIL_MIN_X: f64 = 30.0;

/// The largest x the form was measured at (`POISSON(19,600,TRUE)`, t3284-gamma-deep-upper-tail
/// A111); beyond it the callers keep the accurate kernel (decision 0188 R4).
const FAR_TAIL_MAX_X: f64 = 600.0;

/// ln Gamma(a) - ((a - 1/2) ln a - a + ln(2 pi) / 2) at a = k/2 for k = 1..39, each the double
/// nearest the 50-digit value (`docs/research/t3269/stirling_correction.py` prints this table).
#[rustfmt::skip]
const STIRLING_CORRECTION: [f64; 39] = [
    f64::from_bits(0x3FC3_A37A_020B_8C22), // a = 0.5: 0.15342640972002736
    f64::from_bits(0x3FB4_C071_BCDA_0A5B), // a = 1.0: 0.08106146679532726
    f64::from_bits(0x3FAC_1098_B28D_CF33), // a = 1.5: 0.05481412105191765
    f64::from_bits(0x3FA5_2A9B_923E_A649), // a = 2.0: 0.0413406959554093
    f64::from_bits(0x3FA0_FAB9_626B_44FF), // a = 2.5: 0.03316287351993629
    f64::from_bits(0x3F9C_579A_268D_80B3), // a = 3.0: 0.02767792568499834
    f64::from_bits(0x3F98_50EA_113C_AF0E), // a = 3.5: 0.023746163656297496
    f64::from_bits(0x3F95_4A26_62FD_78A9), // a = 4.0: 0.020790672103765093
    f64::from_bits(0x3F92_EEA2_E990_F134), // a = 4.5: 0.018488450532673187
    f64::from_bits(0x3F91_0B4E_513F_CBED), // a = 5.0: 0.016644691189821193
    f64::from_bits(0x3F8E_FF15_B81C_9CC5), // a = 5.5: 0.015134973221917378
    f64::from_bits(0x3F8C_6B16_7BEB_DF36), // a = 6.0: 0.013876128823070748
    f64::from_bits(0x3F8A_3C5F_8A1E_7D1D), // a = 6.5: 0.012810465242920227
    f64::from_bits(0x3F88_5D4D_612E_4A86), // a = 7.0: 0.01189670994589177
    f64::from_bits(0x3F86_BDFC_C7FB_DB0A), // a = 7.5: 0.011104559758206917
    f64::from_bits(0x3F85_5280_5E7B_3076), // a = 8.0: 0.010411265261972096
    f64::from_bits(0x3F84_11B7_5E41_049C), // a = 8.5: 0.009799416126158804
    f64::from_bits(0x3F82_F487_1B12_AB64), // a = 9.0: 0.009255462182712733
    f64::from_bits(0x3F81_F553_026F_BCE1), // a = 9.5: 0.008768700134139386
    f64::from_bits(0x3F81_0F9D_4C07_43A7), // a = 10.0: 0.00833056343336287
    f64::from_bits(0x3F80_3FC2_D49C_8FB9), // a = 10.5: 0.00793411456431402
    f64::from_bits(0x3F7F_0593_0880_14F8), // a = 11.0: 0.007573675487951841
    f64::from_bits(0x3F7D_AC77_3CC5_B3CD), // a = 11.5: 0.007244554301320383
    f64::from_bits(0x3F7C_7018_733A_A9C6), // a = 12.0: 0.00694284010720953
    f64::from_bits(0x3F7B_4D04_A067_629F), // a = 12.5: 0.006665247032707682
    f64::from_bits(0x3F7A_4051_4700_F36C), // a = 13.0: 0.006408994188004207
    f64::from_bits(0x3F79_4782_6547_FE6C), // a = 13.5: 0.006171712263039458
    f64::from_bits(0x3F78_6076_C002_D4A7), // a = 14.0: 0.0059513701127588475
    f64::from_bits(0x3F77_8958_3FFC_5E8C), // a = 14.5: 0.0057462165130101155
    f64::from_bits(0x3F76_C08F_6F19_4A10), // a = 15.0: 0.005554733551962801
    f64::from_bits(0x3F76_04B9_60AF_6E7E), // a = 15.5: 0.005375599032926835
    f64::from_bits(0x3F75_549F_7DD1_13BC), // a = 16.0: 0.0052076559196096404
    f64::from_bits(0x3F74_AF30_CF00_8C32), // a = 16.5: 0.005049887331583002
    f64::from_bits(0x3F74_137C_74DA_35F2), // a = 17.0: 0.004901395948434738
    f64::from_bits(0x3F73_80AD_132C_17F1), // a = 17.5: 0.00476138694171445
    f64::from_bits(0x3F72_F604_FF62_7D77), // a = 18.0: 0.004629153749334028
    f64::from_bits(0x3F72_72DB_0D5D_3D95), // a = 18.5: 0.0045040661551206855
    f64::from_bits(0x3F71_F697_DD85_7D8E), // a = 19.0: 0.004385560249232324
    f64::from_bits(0x3F71_80B3_94F5_84D1), // a = 19.5: 0.004273129932103007
];

/// The exact Stirling correction at a shape `a` that is a multiple of 1/2 from 1/2 to 19.5.
fn stirling_correction_at(a: f64) -> Option<f64> {
    let twice = 2.0 * a;
    if !((1.0..2.0 * DIRECT_SHAPE_MAX).contains(&twice) && twice.fract() == 0.0) {
        return None;
    }
    // 1 <= 2a <= 39 and whole, so the index is exact and in range.
    STIRLING_CORRECTION.get(twice as usize - 1).copied()
}

/// Excel's Q(a, x) for a shape that is a multiple of 1/2 from 3/2 to 19.5 and
/// [`FAR_TAIL_MIN_X`] < x <= [`FAR_TAIL_MAX_X`]; `None` elsewhere (a = 1 is exp(-x), which
/// Excel has correctly rounded), where the result is not a normal double, or past `terms`.
pub(super) fn excel_far_upper(a: f64, x: f64, terms: &mut Terms) -> Option<f64> {
    if a <= 1.0 || !(x > FAR_TAIL_MIN_X && x <= FAR_TAIL_MAX_X) {
        return None;
    }
    let correction = stirling_correction_at(a)?;
    let fraction = accurate_fraction(a, x, terms)?;
    let prefactor = stirling_prefactor(a, excel_ratio_exponent(a, x), correction);
    let upper = prefactor * (fraction.0 + fraction.1);
    upper.is_normal().then_some(upper)
}
