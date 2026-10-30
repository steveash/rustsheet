use super::{compute_payment, growth_log, period_payments, SplitForm};
use crate::expressions::token::Error;

/// IPMT and PPMT past `(1+rate)^nper`'s range and where IPMT's balance cancels on fv
/// (probes/fin-ipmt-ppmt-huge-rate, gen-m9; financial.md "IPMT cancels on fv").
#[test]
fn period_payments_at_huge_rates() {
    let ipmt =
        |r, per, n, pv, fv, t| period_payments(r, per, n, pv, fv, t, SplitForm::Discounted).0;
    let ppmt =
        |r, per, n, pv, fv, t| period_payments(r, per, n, pv, fv, t, SplitForm::Discounted).1;
    assert_eq!(ppmt(42.0, 10.0, 303.0, 8292.0, 1.0, true), Ok(0.0));
    assert_eq!(ppmt(10.0, 1.0, 296.0, 1.0, 0.0, false), Ok(0.0));
    assert_eq!(
        ppmt(10.0, 11.0, 304.0, 44626.0, 826.0, false),
        Ok(-3.076_830_425_599_131_3e-301)
    );
    assert_eq!(ppmt(1e300, 1.5, 180.0, 194_733.0, -1e300, false), Ok(0.0));
    assert_eq!(ppmt(1e307, 7.0, 292.0, 176_644.0, 89.0, false), Ok(0.0));
    assert_eq!(ppmt(2.0, 1100.0, 1100.0, 1.0, 5.0, false), Ok(-4.0));
    assert_eq!(
        ipmt(1e300, 3.0, 2000.0, 452_017.0, 905.0, false),
        Ok(-4.520_170_000_000_000_4e305)
    );
    assert_eq!(
        ipmt(1e10, 8.0, 226.0, 439_138.0, 43.0, false),
        Ok(-4.391_38e15)
    );
    assert_eq!(ipmt(2.0, 1100.0, 1100.0, 1.0, 5.0, false), Ok(2.0));
    assert_eq!(ipmt(0.5, 2.0, 124.0, 0.0, 315.0, false), Ok(0.0));
    assert_eq!(
        ipmt(50.0, 2.0, 10.0, 0.0, 315.0, false),
        Ok(25.0 * 2f64.powi(-42))
    );
    assert_eq!(
        ipmt(1e10, 10.0, 10.0, 0.0, 315.0, false),
        Ok(315.000_079_353_922_05)
    );
    // Period 1 of type 1 is 0 even where PMT is #NUM!; PPMT is PMT there.
    assert_eq!(ipmt(-0.5, 1.0, 2000.0, 1.0, 0.0, true), Ok(0.0));
    assert_eq!(ppmt(-0.5, 1.0, 2000.0, 1.0, 0.0, true), Err(Error::NUM));
}

/// PPMT at negative rates down to -1 is the discounted form, and where `pv+fv` times the
/// discount passes the double range it still has Excel's value (gen-m9 79d87fc527927626 A46,
/// 5422bdbf173fc233 A383, d0854366830dcdee A335, 762cd0e4ec9a7aa9 A5, 67a4c03afd63f988 A180;
/// financial.md "PPMT discounts with an atanh-series log").
#[test]
fn principal_at_negative_rates() {
    let ppmt =
        |r, per, n, pv, fv| period_payments(r, per, n, pv, fv, false, SplitForm::Discounted).1;
    assert_eq!(
        ppmt(-0.5, 5.0, 294.0, 171_132.0, 0.0),
        Ok(-5_347.875_000_000_112)
    );
    assert_eq!(
        ppmt(-0.5, 13.0, 246.0, 37_796.0, 36.0),
        Ok(-4.618_164_062_499_897)
    );
    // A fractional period is an ulp off Excel's -56630.48966324223 (bits are T-2243's).
    let fractional = ppmt(-0.5, 1.999, 97.0, 226_365.0, 0.0).unwrap_or(f64::NAN);
    assert!((fractional / -56_630.489_663_242_23 - 1.0).abs() < 5e-16);
    assert_eq!(ppmt(-0.5, 1.0, 277.0, 73_630.0, 609.0), Ok(-37_119.5));
    assert_eq!(
        ppmt(-0.5, 4.0, 230.0, 1e300, 0.0),
        Ok(-6.249_999_999_999_966e298)
    );
}

/// IPMT takes Excel's own discount where it moves the value past a few ulp: long terms at
/// negative rates, a fractional period (gen-m9 e168be7f6d3f84c2 A385, c0681a8b150ce301 A104,
/// b6a02893a8d6b151 A35, 1cd3096e3e87c965 A227; financial.md "IPMT follows Excel's own
/// discount where it moves the value").
#[test]
fn interest_on_excels_discount() {
    let ipmt =
        |r, per, n, pv, fv, t| period_payments(r, per, n, pv, fv, t, SplitForm::Discounted).0;
    assert_eq!(
        ipmt(-0.05, 10.0, 336.0, 1.0, 0.0, false),
        Ok(0.031_512_469_880_875_22)
    );
    assert_eq!(
        ipmt(-0.5, 3.0, 232.0, 259_859.0, 333.0, true),
        Ok(64_714.999_999_999_75)
    );
    assert_eq!(
        ipmt(-0.05, 4.0, 341.0, 208_149.0, 115.0, true),
        Ok(9_391.860_328_810_97)
    );
    assert_eq!(
        // 3.14159 (Data!A6), written as a quotient so clippy does not take it for PI.
        ipmt(314_159.0 / 100_000.0, 1.999, 167.0, 0.375, 0.0, false),
        Ok(-1.178_096_249_999_999_8)
    );
    // Within a few ulp the exact balance stays: it has Excel's bits (m9-fin-pmt A101).
    assert_eq!(
        ipmt(0.1, 2.0, 3.0, 10_000.0, 0.0, true),
        Ok(-634.441_087_613_293_1)
    );
}

/// PPMT's log is the series below 0.375 and Excel's LN above it, an ulp or two from
/// `ln_1p` in places (probes-numeric/ppmt-first-period-sweep, ppmt-log-rate-sweep).
#[test]
fn growth_log_is_excels() {
    assert_eq!(growth_log(0.1), 0.095_310_179_804_324_85); // ln_1p: ...87
    assert_eq!(growth_log(0.05), 0.048_790_164_169_432_014); // ln_1p: ...01
    assert_eq!(growth_log(-0.02), -0.020_202_707_317_519_45);
    assert_eq!(growth_log(0.37), 0.314_810_739_840_033_5);
    assert_eq!(growth_log(0.38), 0.322_083_499_169_113_2); // LN(1.38)
    assert_eq!(growth_log(0.6), 0.470_003_629_245_735_63);
}

/// PMT is Excel's discounted payment, not the exact one (financial.md "PMT is a discounted
/// payment", T-3014): each value below is Excel's, and the exact payment misses each by an ulp
/// or more.
#[test]
fn payment_is_excels_discounted_form() {
    let pmt = |rate, nper, pv, fv, period_start| {
        compute_payment(rate, nper, pv, fv, period_start).unwrap_or(f64::NAN)
    };
    // probes-numeric/m9-fin-pmt A169, A85: the exact -10500 and -11000 at nper 1.
    assert_eq!(pmt(0.05, 1.0, 10000.0, 0.0, false), -10_499.999_999_999_998);
    assert_eq!(pmt(0.1, 1.0, 10000.0, 0.0, false), -11_000.000_000_000_002);
    // ironcalc-calc FINANCIAL/PMT.xlsx F10, G18: rate -0.9 with fv and type 1, and alone.
    assert_eq!(pmt(-0.9, 3.0, 120.0, 300.0, true), -2_703.783_783_783_783);
    assert_eq!(pmt(-0.9, 1.0, 100.0, 0.0, false), -9.999_999_999_999_998);
    // probes/t3014-pmt-fv-type-alone A7 (fv alone), C41 (|nper*L| 0.9896, expm1) and C42
    // (|nper*L| 1.0648, 1-Q).
    assert_eq!(
        pmt(0.2, 8.0, 10000.0, -500.0, false),
        -2_575.789_512_882_557
    );
    assert_eq!(
        pmt(-0.05, 19.293, 10000.0, 0.0, false),
        -295.829_576_962_597_2
    );
    assert_eq!(
        pmt(-0.1, 10.106, 10000.0, 0.0, false),
        -526.265_477_108_976_1
    );
}

/// IPMT and PPMT split Excel's discounted payment (financial.md "IPMT and PPMT split Excel's
/// discounted payment", T-2243): ironcalc-calc `FINANCIAL/IPMT_PPMT.xlsx` and the cells the earlier
/// forms left open (T-2667, T-3086); CUMIPMT's type 1 sum keeps the exact split.
#[test]
fn period_payments_split_the_discounted_payment() {
    let split = |r, per, n, pv, fv, t| period_payments(r, per, n, pv, fv, t, SplitForm::Discounted);
    // IPMT_PPMT.xlsx G12, H4, H13 and I2 (PPMT in period 1 of type 1 is Excel's PMT).
    assert_eq!(
        split(0.1, 3.0, 30.0, 8000.0, 10.0, true).0,
        Ok(-717.976_451_376_585_8)
    );
    assert_eq!(
        split(0.3, 5.0, 36.0, 8000.0, 0.0, false).0,
        Ok(-2_399.647_721_391_178)
    );
    assert_eq!(
        split(0.1, 3.3, 30.0, 8000.0, 0.0, false).0,
        Ok(-788.079_957_440_733_7)
    );
    assert_eq!(
        split(0.0083, 1.0, 36.0, 8000.0, 0.0, true).1,
        Ok(-255.863_617_441_602_34)
    );
    // probes/fin-ipmt-ppmt-huge-rate (T-2667) and t2665-ipmt-fractional-nper-past-range (T-3086).
    assert_eq!(
        split(10.0, 2.0, 10.0, 0.0, 315.0, false).0,
        Ok(1.214_460_780_829_540_5e-6)
    );
    assert_eq!(
        split(1000.0, 7.0, 1e5, 1000.0, 0.0, true).0,
        Ok(-999.000_999_000_999_2)
    );
    // A discount below the normal range is a PPMT of 0 (probes/fin-ppmt-discount-underflow F1).
    assert_eq!(split(7.0, 1.0, 341.0, 1.0, 0.0, false).1, Ok(0.0));
    // CUMIPMT(0.1,10,345,2,2,1) is the exact split's -29.395712615048502, the discounted one's
    // ...485 (probes/fin-cumipmt-overflow-and-forms A8).
    assert_eq!(
        period_payments(0.1, 2.0, 10.0, 345.0, 0.0, true, SplitForm::Exact).0,
        Ok(-29.395_712_615_048_502)
    );
}
