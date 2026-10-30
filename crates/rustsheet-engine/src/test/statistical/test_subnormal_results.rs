use crate::test::util::check_cells;

// T-2737: whether Excel keeps a subnormal result or returns a true 0 follows each function's path,
// not the size of the result (docs/excel-semantics/numbers.md "Distribution densities"). Each row
// is `(x)=0` of a call whose true value is subnormal: FALSE where Excel keeps it, TRUE where it
// flushes it (probes/t2706-subnormal-density-paths, probes/t2737-subnormal-cdf-shapes).
#[test]
fn subnormal_results_are_kept_or_flushed_as_in_excel() {
    let rows: [(&str, &str); 21] = [
        // Kept: the closed forms at the ends and the beta = 1 cdf.
        ("BINOM.DIST(0,1000,0.51,TRUE)", "FALSE"),
        ("NEGBINOM.DIST(0,1000,0.49,TRUE)", "FALSE"),
        ("NEGBINOM.DIST(0,1000,0.49,FALSE)", "FALSE"),
        ("NEGBINOMDIST(0,1000,0.49)", "FALSE"),
        ("BETA.DIST(1E-155,2,1,TRUE)", "FALSE"),
        ("BETA.DIST(2E-104,3,1,TRUE)", "FALSE"),
        ("BETA.DIST(1E-155,2,1,TRUE,0,1)", "FALSE"),
        ("WEIBULL.DIST(1E-155,2,1,TRUE)", "FALSE"),
        // Flushed.
        ("BINOM.DIST(1,1000,0.52,TRUE)", "TRUE"),
        ("BINOM.DIST(3,1000,0.53,TRUE)", "TRUE"),
        ("BINOMDIST(1,1000,0.52,TRUE)", "TRUE"),
        ("NEGBINOM.DIST(1,1000,0.48,TRUE)", "TRUE"),
        ("BETA.DIST(1E-155,2,2,TRUE)", "TRUE"),
        ("BETA.DIST(1E-155,2,1.0000001,TRUE)", "TRUE"),
        ("BETA.DIST(1E-31,10,2,TRUE)", "TRUE"),
        ("GAMMA.DIST(1E-155,2,1,TRUE)", "TRUE"),
        ("GAMMA.DIST(1E-62,5,1,TRUE)", "TRUE"),
        ("EXPON.DIST(1E-150,1E-160,TRUE)", "TRUE"),
        ("CHISQ.DIST(2E-155,4,TRUE)", "TRUE"),
        ("CHISQ.DIST(1430,2,FALSE)", "TRUE"),
        ("CHISQ.DIST(1500,6,FALSE)", "TRUE"),
    ];
    let cells: Vec<(String, String, &str)> = rows
        .iter()
        .enumerate()
        .map(|(i, (call, zero))| (format!("A{}", i + 1), format!("=({call})=0"), *zero))
        .collect();
    let cells: Vec<(&str, &str, &str)> = cells
        .iter()
        .map(|(a, f, z)| (a.as_str(), f.as_str(), *z))
        .collect();
    check_cells(&[], &cells);
}

// T-2738: GAMMA.DIST's density is #NUM! where x / beta is subnormal; the cdf there is 0
// (probes/t2738-gamma-dist-tiny-ratio).
#[test]
fn gamma_density_of_a_subnormal_ratio_is_num() {
    check_cells(
        &[],
        &[
            ("A1", "=GAMMA.DIST(4.4E-308,1,2,FALSE)", "#NUM!"),
            ("A2", "=GAMMA.DIST(1E-10,0.5,1E+300,FALSE)", "#NUM!"),
            ("A3", "=GAMMADIST(1E-10,1,1E+300,FALSE)", "#NUM!"),
            ("A4", "=GAMMA.DIST(1E-300,1,1E+10,TRUE)", "0"),
            ("A5", "=GAMMA.DIST(4.5E-308,1,2,FALSE)>0", "TRUE"),
        ],
    );
}

// T-2753: below x / beta = 1E-15 GAMMA.DIST's density is Excel's log-space form, not the exact
// density (probes/t2753-gamma-density-ratio-sweep B1, A3; probes/t2738-gamma-dist-tiny-ratio A2), where
// the engine is bit-exact; elsewhere in the regime it is 0 to 3 ulp off (T-2852).
#[test]
fn gamma_density_at_a_tiny_ratio_is_the_log_form() {
    check_cells(
        &[],
        &[
            (
                "A1",
                "=GAMMA.DIST(1,1,1E+300,FALSE)",
                "1.0000000000000237e-300",
            ),
            ("A2", "=GAMMA.DIST(1E-200,1,1,FALSE)", "0.9999999999999779"),
            (
                "A3",
                "=GAMMA.DIST(1,1,1E+307,FALSE)",
                "9.999999999999768e-308",
            ),
            ("A4", "=GAMMA.DIST(0,9,2,FALSE)", "0"),
        ],
    );
}

// T-2852: from shape 0.5 to 3 and x / beta 1E-15 to 16 the density is EXP(a LN(y)) / GAMMA(a) / x *
// EXP(-y), so its error is a times LN(y)'s: Excel's own doubles, bit for bit (probes/
// t2753-gamma-density-switch A6, A8, -ratio-sweep A20; probes-numeric m9-gamma-dist-a1 A2,
// t1049-chisqdist A6), where the direct form was 2 to 11 ulp off.
#[test]
fn gamma_density_below_shape_three_is_the_log_form() {
    check_cells(
        &[],
        &[
            ("A1", "=GAMMA.DIST(1E-14,1,1,FALSE)", "0.9999999999999888"),
            ("A2", "=GAMMA.DIST(1E-12,1,1,FALSE)", "0.9999999999990009"),
            (
                "A3",
                "=GAMMA.DIST(3E-16,2,1,FALSE)",
                "3.000000000000007e-16",
            ),
            ("A4", "=GAMMA.DIST(0.1,1,1,FALSE)", "0.9048374180359597"),
            ("A5", "=CHISQ.DIST(0.1,5,FALSE)", "0.004000129828100455"),
        ],
    );
}
