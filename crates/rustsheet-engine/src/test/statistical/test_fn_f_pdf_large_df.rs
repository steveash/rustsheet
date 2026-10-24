#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

/// The ulps between two positive doubles.
fn ulps(a: f64, b: f64) -> u64 {
    a.to_bits().abs_diff(b.to_bits())
}

// T-1864: F.DIST's pdf where statrs' direct form overflows (a degrees of freedom above about
// 143) is Loader's saddle-point form. Excel returns a number on every one of these cells
// (probes-numeric t2220-f-dist-sweep-pdf A994, A897; t2220-f-dist-sweep-d2-1 A776, A710, A719;
// t1530-f-df-cut A9, A10; m9-f-dist A30; gen-m9 285d87cea1aaa73c A64). Excel's own value differs
// from the exact density by up to about 3e-14 relative on some cells, so the expected values are
// the exact densities (mpmath, 50 digits) rounded to doubles, and ours must be within 8 ulp
// (more in the deep tails noted below, where the error grows with |ln density|).
#[test]
fn f_pdf_large_df_is_close_to_the_exact_density() {
    let cases: [(&str, f64, u64); 15] = [
        ("=F.DIST(1,100,100,FALSE)", 1.989_730_934_679_469, 8),
        ("=F.DIST(3,50,100,FALSE)", 1.052_929_985_222_399_2e-5, 8),
        ("=F.DIST(28.1,100,1,FALSE)", 0.002_623_984_829_706_269_6, 8),
        ("=F.DIST(2,5,1E10,FALSE)", 0.141_672_776_761_799_65, 8),
        ("=F.DIST(2,1E10,5,FALSE)", 0.188_252_492_345_040_16, 8),
        (
            "=F.DIST(0.1,123456792.26459,1,FALSE)",
            0.085_003_679_623_665,
            8,
        ),
        ("=F.DIST(1,1000,1000,FALSE)", 6.306_254_544_590_200_5, 8),
        // A deep tail: the exponent is about -153, so its rounding (about 153 eps absolute) is
        // about 100 ulp of the density (Excel's own value is 3e-14 relative off the exact one).
        (
            "=F.DIST(100,50,100,FALSE)",
            2.234_739_078_213_558_4e-67,
            128,
        ),
        ("=F.DIST(57400,50,1,FALSE)", 2.886_469_951_474_940_4e-8, 8),
        (
            "=F.DIST(1000000,50,1,FALSE)",
            3.969_524_772_853_011_7e-10,
            8,
        ),
        // A moderate x with df1 up to 140, whose direct form overflows although the density is
        // positive (#NUM! before T-1864; no oracle cell here, Excel gives a number at every probed
        // positive density).
        ("=F.DIST(20,100,3,FALSE)", 0.001_080_758_108_326_473_7, 8),
        ("=F.DIST(2,140,2,FALSE)", 0.150_825_072_689_534_8, 8),
        // A tail of exponent about -78: about 40 ulp, as for the deep tail above.
        ("=F.DIST(1E10,30,5,FALSE)", 8.379_700_461_247_562e-35, 64),
        // The x-alone overflow keeps its 0 (probes/stat-overflow-arguments E11).
        ("=F.DIST(B1,1,3,FALSE)", 0.0, 8),
        // A small df keeps the beta-density form (`F_DIST.xlsx` F3 is 0.40509259259259256).
        ("=F.DIST(0.7,4,2,FALSE)", 0.405_092_592_592_592_56, 8),
    ];
    let mut model = new_empty_model();
    // x = 1E308, made by a product as in the probe (where it is a cell value).
    model._set("B1", "=1E300*1E8");
    for (i, (formula, _, _)) in cases.iter().enumerate() {
        model._set(&format!("A{}", i + 1), formula);
    }
    model.evaluate();
    for (i, (formula, expected, max_ulp)) in cases.iter().enumerate() {
        let actual = match model
            .get_cell_value_by_ref(&format!("Sheet1!A{}", i + 1))
            .unwrap()
        {
            crate::cell::CellValue::Number(n) => n,
            other => panic!("{formula}: expected a number, got {other:?}"),
        };
        let diff = ulps(actual, *expected);
        assert!(
            diff <= *max_ulp,
            "{formula}: {actual} vs {expected} ({diff} ulp)"
        );
    }
}

// x <= 0 keeps its value, and df above 1E10 stays #NUM! (probes-numeric t1530-f-df-cut).
#[test]
fn f_pdf_large_df_edges() {
    let mut model = new_empty_model();
    model._set("A1", "=F.DIST(0,500,500,FALSE)");
    model._set("A2", "=F.DIST(-1,500,500,FALSE)");
    model._set("A3", "=F.DIST(2,5,10000000001,FALSE)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"0");
    assert_eq!(model._get_text("A2"), *"#NUM!");
    assert_eq!(model._get_text("A3"), *"#NUM!");
}
