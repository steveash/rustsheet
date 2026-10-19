//! The distribution functions Excel builds from its ERF/ERFC kernels obey Excel's identities bit
//! for bit (T-1929; docs/excel-semantics/statistics.md, "One incomplete-gamma kernel under ERF,
//! ERFC, NORM, CHISQ, GAMMA.DIST and POISSON (T-1778)"). Each test puts both sides of one identity
//! on probe inputs, and every expected number is the excel-graph oracle's cell, compared bit for
//! bit (`check_cells`).

use crate::test::util::{check_cells, check_cells_with_numbers};

/// `ERFC.PRECISE(x)` = `CHISQ.DIST.RT(2x^2,1)`, and `CHISQ.DIST.RT(2z,1)` = `ERFC.PRECISE(SQRT(z))`
/// with the root rounded twice: probes-numeric/t1916-erf-wrappers B, C (rows 2258, 2261, 2264)
/// and M, N (rows 1721, 1722), and t1916-erfc-tail B, D at x = 26.5 and past the subnormal cutoff.
#[test]
fn chisq_dist_rt_df_1_is_erfc_of_the_root() {
    check_cells_with_numbers(
        &[
            ("A1", 0.260_000_000_160_716_6),
            ("A2", 0.260_000_000_160_716_8),
            ("A3", 0.260_000_000_160_716_95),
            ("A4", 0.720_458_042_119_282_1),
            ("A5", 0.724_320_673_588_002_5),
            ("A6", 26.5),
            ("A7", 26.599_500_618_033_99),
        ],
        &[],
        &[
            ("B1", "=CHISQ.DIST.RT(2*A1,1)", "0.47084168185351727"),
            ("C1", "=ERFC.PRECISE(SQRT(A1))", "0.47084168185351727"),
            ("B2", "=CHISQ.DIST.RT(2*A2,1)", "0.4708416818535172"),
            ("C2", "=ERFC.PRECISE(SQRT(A2))", "0.4708416818535172"),
            ("B3", "=CHISQ.DIST.RT(2*A3,1)", "0.4708416818535169"),
            ("C3", "=ERFC.PRECISE(SQRT(A3))", "0.4708416818535169"),
            ("B4", "=CHISQ.DIST.RT(A4*A4,1)", "0.4712430243790635"),
            ("C4", "=ERFC.PRECISE(SQRT(A4*A4/2))", "0.4712430243790635"),
            ("B5", "=CHISQ.DIST.RT(A5*A5,1)", "0.4688688877737471"),
            ("C5", "=ERFC.PRECISE(SQRT(A5*A5/2))", "0.4688688877737471"),
            ("B6", "=CHISQ.DIST.RT(2*A6*A6,1)", "2.2109076642637346e-307"),
            ("C6", "=ERFC.PRECISE(A6)", "2.2109076642637346e-307"),
            ("B7", "=CHISQ.DIST.RT(2*A7*A7,1)", "0"),
            ("C7", "=ERFC.PRECISE(A7)", "0"),
        ],
    );
}

/// CHIDIST and CHISQ.TEST at one degree of freedom are the same Q(1/2, x/2):
/// probes-numeric/m9-legacy-chidist A1-A66 and m9-legacy-chitest A7, A8 (a 2x1 test, df 1);
/// CHISQ.TEST's statistic there is 4/12 + 4/13, whose CHISQ.DIST.RT is the same double.
#[test]
fn chidist_and_chisq_test_df_1_are_the_same_wrapper() {
    check_cells(
        &[("B1", "10"), ("B2", "15"), ("B4", "12"), ("B5", "13")],
        &[
            ("A1", "=CHIDIST(0,1)", "1"),
            ("A2", "=CHISQ.DIST.RT(0,1)", "1"),
            ("A9", "=CHIDIST(1e-06,1)", "0.9992021155721779"),
            ("A10", "=CHISQ.DIST.RT(1e-06,1)", "0.9992021155721779"),
            ("A33", "=CHIDIST(3.84,1)", "0.05004352124870511"),
            ("A34", "=CHISQ.DIST.RT(3.84,1)", "0.05004352124870511"),
            ("A57", "=CHIDIST(200,1)", "2.0884875837625446e-45"),
            ("A58", "=CHISQ.DIST.RT(200,1)", "2.0884875837625446e-45"),
            ("A65", "=CHIDIST(1000,1)", "1.7958327848006243e-219"),
            ("A66", "=CHISQ.DIST.RT(1000,1)", "1.7958327848006243e-219"),
            ("C7", "=CHITEST(B1:B2,B4:B5)", "0.4233396415824436"),
            ("C8", "=CHISQ.TEST(B1:B2,B4:B5)", "0.4233396415824436"),
            ("C9", "=CHISQ.DIST.RT(4/12+4/13,1)", "0.4233396415824436"),
        ],
    );
}

/// `ERF.PRECISE(x)` = `GAMMA.DIST(x^2,0.5,1,TRUE)` = `CHISQ.DIST(2x^2,1,TRUE)`, and
/// `GAMMA.DIST(z,0.5,1,TRUE)` = `ERF.PRECISE(SQRT(z))`: probes-numeric/m9-gamma-dist-a0p5 A1-A9
/// and m9-chisq-dist A1-A9 (the ERF.PRECISE values are kmyst-erf-gamma's, erf.rs' tests).
#[test]
fn gamma_dist_shape_half_is_erf_of_the_root() {
    check_cells(
        &[],
        &[
            ("A1", "=GAMMA.DIST(0.05,0.5,1,TRUE)", "0.2481703659541508"),
            ("B1", "=ERF.PRECISE(SQRT(0.05))", "0.2481703659541508"),
            ("C1", "=CHISQ.DIST(0.1,1,TRUE)", "0.2481703659541508"),
            ("A2", "=GAMMA.DIST(0.25,0.5,1,TRUE)", "0.5204998778130465"),
            ("B2", "=ERF.PRECISE(0.5)", "0.5204998778130465"),
            ("C2", "=CHISQ.DIST(2*0.5^2,1,TRUE)", "0.5204998778130465"),
            ("A3", "=GAMMA.DIST(0.5,0.5,1,TRUE)", "0.6826894921370861"),
            ("C3", "=CHISQ.DIST(1,1,TRUE)", "0.6826894921370861"),
            ("A4", "=GAMMA.DIST(1.5,0.5,1,TRUE)", "0.9167354833364496"),
            ("B4", "=ERF.PRECISE(SQRT(1.5))", "0.9167354833364496"),
            ("A5", "=GAMMA.DIST(0.0625,0.5,1,TRUE)", "0.276326390168237"),
            ("B5", "=ERF.PRECISE(0.25)", "0.276326390168237"),
            ("C5", "=CHISQ.DIST(0.125,1,TRUE)", "0.276326390168237"),
        ],
    );
}

/// `GAMMA.DIST(x,a,b,TRUE)` = `GAMMA.DIST(x/b,a,1,TRUE)`, the quotient rounded twice, and
/// `CHISQ.DIST(x,k,TRUE)` = `GAMMA.DIST(x/2,k/2,1,TRUE)`: probes-numeric/m9-gamma-dist-a0p5
/// A11-A29 (scales 2 and 1E6) and m9-gamma-dist-a2 A21 (which statrs' x*(1/b) misses).
#[test]
fn gamma_dist_scale_divides_x() {
    check_cells(
        &[],
        &[
            ("A11", "=GAMMA.DIST(0.1,0.5,2,TRUE)", "0.2481703659541508"),
            ("A19", "=GAMMA.DIST(3,0.5,2,TRUE)", "0.9167354833364496"),
            (
                "A21",
                "=GAMMA.DIST(50000,0.5,1000000,TRUE)",
                "0.2481703659541508",
            ),
            (
                "A29",
                "=GAMMA.DIST(1500000,0.5,1000000,TRUE)",
                "0.9167354833364496",
            ),
            (
                "B29",
                "=GAMMA.DIST(1500000/1000000,0.5,1,TRUE)",
                "0.9167354833364496",
            ),
            (
                "C21",
                "=GAMMA.DIST(200000,2,1000000,TRUE)",
                "0.017523096306421772",
            ),
            (
                "D21",
                "=GAMMA.DIST(200000/1000000,2,1,TRUE)",
                "0.017523096306421772",
            ),
        ],
    );
    // Away from shape 0.5 both sides are the same statrs series (its values are not Excel's yet).
    let pairs = [
        ("=CHISQ.DIST(3,3,TRUE)", "=GAMMA.DIST(3/2,3/2,1,TRUE)"),
        ("=CHISQ.DIST(7.3,10,TRUE)", "=GAMMA.DIST(7.3/2,10/2,1,TRUE)"),
        (
            "=GAMMA.DIST(4.4,2.5,3,TRUE)",
            "=GAMMA.DIST(4.4/3,2.5,1,TRUE)",
        ),
    ];
    assert_same_values(&pairs);
}

/// `POISSON.DIST(k,m,TRUE)` = `CHISQ.DIST.RT(2m,2(k+1))`, an upper Q (T-1778, 60 of 60): both are
/// Q(k + 1, m) of the one capped series, so they agree on every input both take.
#[test]
fn poisson_cdf_is_the_chi_squared_upper_tail() {
    assert_same_values(&[
        ("=POISSON.DIST(2,3.5,TRUE)", "=CHISQ.DIST.RT(2*3.5,2*(2+1))"),
        ("=POISSON.DIST(0,0.25,TRUE)", "=CHISQ.DIST.RT(0.5,2)"),
        ("=POISSON(40,37.3,TRUE)", "=CHISQ.DIST.RT(74.6,82)"),
        ("=POISSON.DIST(5,1E-3,TRUE)", "=CHISQ.DIST.RT(2E-3,12)"),
    ]);
}

/// Z.TEST is NORM.S.DIST(-z,TRUE) with z = (mean - x)*SQRT(n)/sigma: probes-numeric/
/// t1916-erf-wrappers B, G, I and J (rows 1721, 1722), t1049-ztest A20, A49 and t1048-ztest
/// A9, A10. One value is #DIV/0! with or without sigma (t1916-erf-wrappers T1409, t1049-ztest
/// A40, A42), and a negative sigma #NUM! (t1049-ztest A26).
#[test]
fn z_test_is_norm_s_dist_of_minus_z() {
    check_cells_with_numbers(
        &[
            ("A1", 0.720_458_042_119_282_1),
            ("A2", 0.724_320_673_588_002_5),
            ("O1", 0.709_606_953_501_723_4),
        ],
        &[
            ("B1", "3"),
            ("B2", "6"),
            ("B3", "7"),
            ("B4", "8"),
            ("B5", "6"),
            ("B6", "5"),
            ("B7", "4"),
            ("B8", "2"),
            ("B9", "1"),
            ("B10", "9"),
            ("E2", "4"),
            ("E3", "8"),
            ("E4", "abc"),
            ("E5", "6"),
        ],
        &[
            ("C1", "=NORM.S.DIST(-A1,TRUE)", "0.2356215121895317"),
            (
                "D1",
                "=Z.TEST(HSTACK(A1,A1),0,SQRT(2))",
                "0.2356215121895317",
            ),
            ("F1", "=NORM.S.DIST(A1,TRUE)", "0.7643784878104682"),
            (
                "G1",
                "=Z.TEST(HSTACK(-A1,-A1),0,SQRT(2))",
                "0.7643784878104682",
            ),
            ("C2", "=NORM.S.DIST(-A2,TRUE)", "0.23443444388687346"),
            (
                "D2",
                "=Z.TEST(HSTACK(A2,A2),0,SQRT(2))",
                "0.23443444388687346",
            ),
            (
                "G2",
                "=Z.TEST(HSTACK(-A2,-A2),0,SQRT(2))",
                "0.7655655561131265",
            ),
            ("H1", "=Z.TEST(B1:B10,4)", "0.09057419685136381"),
            ("H2", "=ZTEST(B1:B10,4)", "0.09057419685136381"),
            ("H3", "=Z.TEST(B1:B10,0)", "2.825439313895175e-10"),
            ("H4", "=Z.TEST(E2:E5,4)", "0.041632258331775196"),
            (
                "H5",
                "=Z.TEST({1,2,3,4,5,6,7,8,9,10},3)",
                "0.004511719409040159",
            ),
            (
                "H6",
                "=Z.TEST({1,2,3,4,5,6,7,8,9,10},3,2)",
                "3.861339775271687e-05",
            ),
            ("H7", "=Z.TEST(O1,0,1)", "#DIV/0!"),
            ("H8", "=Z.TEST(7,4,1)", "#DIV/0!"),
            ("H9", "=Z.TEST(B1:B10,5,-1)", "#NUM!"),
        ],
    );
}

/// `GAUSS(x)` = `NORM.S.DIST(x,TRUE)-0.5` (T-1778, 1,545 of 1,545).
#[test]
fn gauss_is_norm_s_dist_minus_one_half() {
    assert_same_values(&[
        (
            "=GAUSS(0.7204580421192821)",
            "=NORM.S.DIST(0.7204580421192821,TRUE)-0.5",
        ),
        ("=GAUSS(-2.5)", "=NORM.S.DIST(-2.5,TRUE)-0.5"),
        ("=GAUSS(6)", "=NORM.S.DIST(6,TRUE)-0.5"),
    ]);
}

/// `T.DIST(x,n,TRUE)` for x < 0 = `T.DIST.2T(-x,n)/2` (T-1778, 60 of 60): probes-numeric/
/// m9-t-dist-cdf's negative points.
#[test]
fn t_dist_below_zero_is_half_the_two_tailed() {
    assert_same_values(&[
        ("=T.DIST(-0.5,1,TRUE)", "=T.DIST.2T(0.5,1)/2"),
        ("=T.DIST(-2,2,TRUE)", "=T.DIST.2T(2,2)/2"),
        ("=T.DIST(-10,30,TRUE)", "=T.DIST.2T(10,30)/2"),
        ("=T.DIST(-0.5,1E6,TRUE)", "=T.DIST.2T(0.5,1E6)/2"),
    ]);
}

/// Each pair of formulas evaluates to the same double.
fn assert_same_values(pairs: &[(&str, &str)]) {
    let mut model = crate::test::util::new_empty_model();
    for (row, (left, right)) in (1..).zip(pairs) {
        model._set(&format!("A{row}"), left);
        model._set(&format!("B{row}"), right);
    }
    model.evaluate();
    for (row, (left, right)) in (1..).zip(pairs) {
        let a = crate::test::util::round_trip_value(&model, &format!("A{row}"));
        let b = crate::test::util::round_trip_value(&model, &format!("B{row}"));
        assert!(a.parse::<f64>().is_ok(), "{left}: {a}");
        assert_eq!(a, b, "{left} vs {right}");
    }
}
