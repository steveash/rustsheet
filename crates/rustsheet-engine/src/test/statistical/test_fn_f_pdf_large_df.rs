#![allow(clippy::unwrap_used)]

use crate::test::util::{check_cells, new_empty_model, ulps};

// T-1864: F.DIST's pdf where statrs' direct form overflows (a degrees of freedom above about
// 143) is Loader's saddle-point form. Excel returns a number on every one of these cells
// (probes-numeric t2220-f-dist-sweep-pdf A994, A897; t2220-f-dist-sweep-d2-1 A776, A710, A719;
// t1530-f-df-cut A9, A10; m9-f-dist A30; gen-m9 285d87cea1aaa73c A64). Excel's own value differs
// from the exact density by up to about 3e-14 relative on some cells, so the expected values are
// the exact densities (mpmath, 50 digits) rounded to doubles, and ours must be within 8 ulp
// (more in the deep tails noted below, where the error grows with |ln density|).
#[test]
fn f_pdf_large_df_is_close_to_the_exact_density() {
    let cases: [(&str, f64, u64); 18] = [
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
        // positive (#NUM! before T-1864). Excel returns a number here: F.DIST(20,100,3,FALSE) is
        // probes-numeric t1864-f-pdf-moderate-x A1, 0.0010807581083264741 (the exact density's
        // double is the expected value below, 2 ulp from Excel's).
        ("=F.DIST(20,100,3,FALSE)", 0.001_080_758_108_326_473_7, 8),
        ("=F.DIST(2,140,2,FALSE)", 0.150_825_072_689_534_8, 8),
        // A tail of exponent about -78: about 40 ulp, as for the deep tail above. Excel's is
        // 8.379700461247534E-35 (probes-numeric t1864-f-pdf-edges A1), 26 ulp below the exact one.
        ("=F.DIST(1E10,30,5,FALSE)", 8.379_700_461_247_562e-35, 64),
        // A huge x where statrs' direct form underflows to 0 although the density is a double:
        // Excel's are 2.7556759606310174E-250 and 3.1830988618378795E-301 (probes-numeric
        // t1864-f-pdf-edges A6, A7, about 2E-14 and 9E-15 relative below the exact densities,
        // which are the expected values here). The error grows with |ln density| (about 692 at
        // 3E-301, where its rounding alone is about 7.7E-14 relative, 350 ulp): 171 ulp there.
        ("=F.DIST(1E100,2,3,FALSE)", 2.755_675_960_631_075_4e-250, 64),
        ("=F.DIST(1E200,1,1,FALSE)", 3.183_098_861_837_907e-301, 256),
        // The x-alone overflow keeps its 0 (probes/stat-overflow-arguments E11).
        ("=F.DIST(B1,1,3,FALSE)", 0.0, 8),
        // Underflows in the log form too (t1864-f-pdf-edges A11: Excel 0).
        ("=F.DIST(1E300,1,1,FALSE)", 0.0, 8),
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

// Excel stores Y = density·x, so its density is 0 once Y underflows, though the exact density is a
// normal double (probes-numeric t1864-f-pdf-tiny-x A1, A2, A7, A8, A11, A14; t1864-f-pdf-edges A8),
// and positive just above that (t1864-f-pdf-tiny-x A6, A10, A17; t1864-f-pdf-edges A2, A3).
#[test]
fn f_pdf_is_zero_once_density_times_x_underflows() {
    let mut model = new_empty_model();
    let zero = [
        "=F.DIST(1E-300,3,5,FALSE)",
        "=F.DIST(1E-300,3,100,FALSE)",
        "=F.DIST(1E-220,3,1000,FALSE)",
        "=F.DIST(1E-300,4,1000,FALSE)",
        "=F.DIST(1E-170,4,1000,FALSE)",
        "=F.DIST(1E-300,3,1,FALSE)",
        "=F.DIST(1E-300,3,1000,FALSE)",
    ];
    let positive = [
        "=F.DIST(1E-215,3,1000,FALSE)",
        "=F.DIST(1E-160,4,1000,FALSE)",
        "=F.DIST(1E-65,10,1000,FALSE)",
        "=F.DIST(1E-300,1,1000,FALSE)",
        "=F.DIST(1E-300,2,1000,FALSE)",
    ];
    for (i, formula) in zero.iter().chain(&positive).enumerate() {
        model._set(&format!("A{}", i + 1), formula);
    }
    model.evaluate();
    for (i, formula) in zero.iter().chain(&positive).enumerate() {
        let value = model
            .get_cell_value_by_ref(&format!("Sheet1!A{}", i + 1))
            .unwrap();
        let crate::cell::CellValue::Number(n) = value else {
            panic!("{formula}: expected a number, got {value:?}");
        };
        assert_eq!(n == 0.0, i < zero.len(), "{formula}: {n}");
    }
}

// Excel's range steps on the density, bit for bit (docs/excel-semantics/statistics.md, the F
// density entry's round 5): a subnormal Y = density·x keeps only its own bits (t1864-f-pdf-tiny-x
// A6, A17: 13 and 3 units of 2^-1074, to Excel's division noise), an F below 2^-1022 is 0 (t2586-r5-f-edges-a B1296, Excel 0
// where the density is about 1.8E-310), a subnormal x is #DIV/0! (B931), and a df1·x near f64::MAX
// is 0, not inf/inf (B1437; t2586-verify-r5-f-edges-hold B91). The beta path stays where statrs'
// own form is a finite 0 (t2586-r5-f-bigconst-a B889).
#[test]
fn f_pdf_excel_range_steps() {
    check_cells(
        &[],
        &[
            (
                "A3",
                "=F.DIST((103934770*2^26+66712675)*2^649,1,1,FALSE)",
                "0.0",
            ),
            ("A4", "=F.DIST(EXP(-720),3,5,FALSE)", "#DIV/0!"),
            (
                "A5",
                "=F.DIST((134217727*2^26+67108863)*2^971,1,10000000000,FALSE)",
                "0.0",
            ),
            (
                "A6",
                "=F.DIST((68719476*2^26+49392122)*2^962,1000,3,FALSE)",
                "0.0",
            ),
            ("A8", "=F.DIST(1E307,1,1E10,FALSE)", "0.0"),
            ("A9", "=F.DIST(1E305,1000,3,FALSE)", "0.0"),
            ("A7", "=F.DIST(1+2^-51,20,127,FALSE)", "1.1626735695122385"),
        ],
    );
    // The subnormal Y's quantisation (2.1% and 11.6% from the exact density) is Excel's; its
    // value then differs from fl(Y/x) by about 7E-15 and 3.5E-14 relative (T-2647).
    let mut model = new_empty_model();
    model._set("A1", "=F.DIST(1E-215,3,1000,FALSE)");
    model._set("A2", "=F.DIST(1E-65,10,1000,FALSE)");
    model.evaluate();
    for (cell, excel) in [
        ("A1", 6.422_853_395_936_16e-108),
        ("A2", 1.482_196_937_523_687_3e-258),
    ] {
        let value = model
            .get_cell_value_by_ref(&format!("Sheet1!{cell}"))
            .unwrap();
        let crate::cell::CellValue::Number(n) = value else {
            panic!("{cell}: expected a number, got {value:?}");
        };
        assert!(
            ((n - excel) / excel).abs() < 1e-13,
            "{cell}: {n} vs {excel}"
        );
    }
}
