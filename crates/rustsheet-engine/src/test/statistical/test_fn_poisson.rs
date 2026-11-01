#![allow(clippy::unwrap_used)]

use crate::test::util::{new_empty_model, round_trip_value};

#[test]
fn test_fn_poisson_dist_smoke() {
    let mut model = new_empty_model();

    // λ = 2, x = 3
    // P(X = 3)  ≈ 0.180447045
    // P(X <= 3) ≈ 0.857123461
    model._set("A1", "=POISSON.DIST(3, 2, FALSE)");
    model._set("A2", "=POISSON.DIST(3, 2, TRUE)");

    // Wrong arg count
    model._set("A3", "=POISSON.DIST(3, 2)");
    model._set("A4", "=POISSON.DIST(3, 2, TRUE, FALSE)");

    // Domain errors
    model._set("A5", "=POISSON.DIST(-1, 2, TRUE)"); // x < 0
    model._set("A6", "=POISSON.DIST(3, -2, TRUE)"); // mean < 0

    // λ = 0 special cases
    model._set("A7", "=POISSON.DIST(0, 0, FALSE)"); // 1
    model._set("A8", "=POISSON.DIST(1, 0, FALSE)"); // 0
    model._set("A9", "=POISSON.DIST(5, 0, TRUE)"); // 1

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"0.180447044");
    assert_eq!(model._get_text("A2"), *"0.85712346");

    assert_eq!(model._get_text("A3"), *"#ERROR!");
    assert_eq!(model._get_text("A4"), *"#ERROR!");
    assert_eq!(model._get_text("A5"), *"#NUM!");
    assert_eq!(model._get_text("A6"), *"#NUM!");

    assert_eq!(model._get_text("A7"), *"1");
    assert_eq!(model._get_text("A8"), *"0");
    assert_eq!(model._get_text("A9"), *"1");
}

#[test]
fn test_fn_poisson_dist_arguments() {
    let mut model = new_empty_model();

    model._set("A1", "=POISSON.DIST()");
    model._set("A2", "=POISSON.DIST(2)");
    model._set("A3", "=POISSON.DIST(2, 5)");
    model._set("A4", "=POISSON.DIST(2, 5, TRUE)");
    model._set("A5", "=POISSON.DIST(2, 5, TRUE, 1)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#ERROR!");
    assert_eq!(model._get_text("A2"), *"#ERROR!");
    assert_eq!(model._get_text("A3"), *"#ERROR!");
    assert_eq!(model._get_text("A4"), *"0.124652019");
    assert_eq!(model._get_text("A5"), *"#ERROR!");
}

/// Excel's pmf to the bit, one cell per branch of `poisson.rs` `excel_poisson_pmf`
/// (docs/excel-semantics/statistics.md "POISSON.DIST pmf: two closed forms", T-2055).
#[test]
fn test_fn_poisson_pmf_excel_bits() {
    let cases: [(&str, f64); 10] = [
        // mean <= 1.1: EXP(k LN(mean) - GAMMALN(k+1)) * EXP(-mean) (gen-m9 7f7ca726c3f68554
        // A298, 92d4fa29e5510f1e A186, 139684e68fb2fc3b A69; probes-numeric/
        // t2055-poisson-pmf-switch A22).
        ("=POISSON(7,0.01,FALSE)", 1.964_384_590_772_173_2e-18),
        ("=POISSON(7,0.5,FALSE)", 9.401_826_942_470_152e-7),
        ("=POISSON(1,1E-10,FALSE)", 9.999_999_998_999_996e-11),
        ("=POISSON.DIST(7,1.1,FALSE)", 0.000_128_704_677_162_297_64),
        // k = 1 at any mean (gen-m9 afc00c8b70e78baf A193).
        ("=POISSON(1,100,FALSE)", 3.720_075_976_020_837_6e-42),
        // The saddle point: the log form, the series at the 0.2 cut, a fitted small-k stirlerr
        // (probes-numeric/t1048-poisson A4, A8, A29; gen-m9 c4635963fc926284 A125,
        // 7fd0a98183c6babf A107).
        ("=POISSON(2,3,FALSE)", 0.224_041_807_655_387_75),
        ("=POISSON.DIST(10,4,FALSE)", 0.005_292_476_676_420_117),
        ("=POISSON(1000,1000,FALSE)", 0.012_614_611_348_721_503),
        ("=POISSON(92,1.5,FALSE)", 2.845_699_912_954_148_4e-127),
        ("=POISSON(4,3,FALSE)", 0.168_031_355_741_540_85),
    ];
    let mut model = new_empty_model();
    for (row, (formula, _)) in cases.iter().enumerate() {
        model._set(&format!("A{}", row + 1), formula);
    }
    model.evaluate();
    for (row, (formula, excel)) in cases.iter().enumerate() {
        let value = round_trip_value(&model, &format!("A{}", row + 1));
        assert_eq!(
            value.parse::<f64>().unwrap().to_bits(),
            excel.to_bits(),
            "{formula}: {value}"
        );
    }
}
