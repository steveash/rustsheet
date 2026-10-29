#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn test_fn_weibull_dist_smoke() {
    let mut model = new_empty_model();

    // Valid: CDF and PDF for x = 1, alpha = 2, beta = 1
    model._set("A1", "=WEIBULL.DIST(1, 2, 1, TRUE)");
    model._set("A2", "=WEIBULL.DIST(1, 2, 1, FALSE)");

    // Wrong number of arguments -> #ERROR!
    model._set("A3", "=WEIBULL.DIST(1, 2, 1)");
    model._set("A4", "=WEIBULL.DIST(1, 2, 1, TRUE, FALSE)");

    // Domain errors:
    // x < 0  -> #NUM!
    model._set("A5", "=WEIBULL.DIST(-1, 2, 1, TRUE)");
    // alpha <= 0 -> #NUM!
    model._set("A6", "=WEIBULL.DIST(1, 0, 1, TRUE)");
    model._set("A7", "=WEIBULL.DIST(1, -1, 1, TRUE)");
    // beta <= 0 -> #NUM!
    model._set("A8", "=WEIBULL.DIST(1, 2, 0, TRUE)");
    model._set("A9", "=WEIBULL.DIST(1, 2, -1, TRUE)");

    model.evaluate();

    // 1 - e^-1
    assert_eq!(model._get_text("A1"), *"0.632120559");
    // 2 * e^-1
    assert_eq!(model._get_text("A2"), *"0.735758882");

    assert_eq!(model._get_text("A3"), *"#ERROR!");
    assert_eq!(model._get_text("A4"), *"#ERROR!");
    assert_eq!(model._get_text("A5"), *"#NUM!");
    assert_eq!(model._get_text("A6"), *"#NUM!");
    assert_eq!(model._get_text("A7"), *"#NUM!");
    assert_eq!(model._get_text("A8"), *"#NUM!");
    assert_eq!(model._get_text("A9"), *"#NUM!");
}

#[test]
fn weibull_density_at_zero_and_boolean_x() {
    let mut model = new_empty_model();
    // probes-numeric/t1048-weibull A16, A17, A25 and t1049-weibulldist A27, A33.
    model._set("A1", "=WEIBULL.DIST(0,1,1,FALSE)");
    model._set("A2", "=WEIBULL.DIST(0,0.5,1,FALSE)");
    model._set("A3", "=WEIBULL.DIST(0,1,3,FALSE)");
    model._set("A4", "=WEIBULL(TRUE,1,1,TRUE)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"0");
    assert_eq!(model._get_text("A2"), *"0");
    assert_eq!(model._get_text("A3"), *"0");
    assert_eq!(model._get_text("A4"), *"0.632120559");
}

/// Reads `cell` of Sheet1 as a number, panicking on anything else.
fn number(model: &crate::Model, cell: &str) -> f64 {
    match model
        .get_cell_value_by_ref(&format!("Sheet1!{cell}"))
        .unwrap()
    {
        crate::cell::CellValue::Number(n) => n,
        other => panic!("expected a number in {cell}, got {other:?}"),
    }
}

#[test]
fn weibull_matches_excel_digits_through_exp_and_ln() {
    // Excel's values (excel-graph), T-2239: t = EXP(k*LN(x/l)), the density
    // k/EXP(k*LN(l))*EXP((k-1)*LN(x))*EXP(-t) and the cdf -expm1(-t) below t = 1, 1-EXP(-t) above
    // (docs/excel-semantics/statistics.md "The far-tail density is less accurate than exp(-t)").
    let cases = [
        // probes-numeric/m9-weibull-dist A22, A64, A88, A146, A159, A172, A127.
        (
            "=WEIBULL.DIST(1000,0.5,1,FALSE)",
            2.919_931_234_487_755_5e-16,
        ),
        ("=WEIBULL.DIST(100,1,1,FALSE)", 3.720_075_976_020_677_3e-44),
        ("=WEIBULL.DIST(1000,1,9,FALSE)", 6.177_532_755_504_216e-50),
        ("=WEIBULL.DIST(5,3.7,1,FALSE)", 9.363_099_887_555_361e-166),
        ("=WEIBULL.DIST(0.1,3.7,9,TRUE)", 5.878_989_740_656_108e-8),
        ("=WEIBULL.DIST(30,3.7,9,FALSE)", 4.605_259_708_338_221e-37),
        ("=WEIBULL.DIST(30,2,9,TRUE)", 0.999_985_054_661_475_2),
        // probes-numeric/t1048-weibull A8.
        ("=WEIBULL(10,3,8,FALSE)", 0.083_103_608_840_239_87),
        // gen-m9/5389dd4d5b5739d2 A378 and 5919660b7bb4d195 A296 (alpha 2.5 there).
        (
            "=WEIBULL.DIST(100,2.5,1.5E+10,TRUE)",
            3.628_873_693_012_099_4e-21,
        ),
        ("=WEIBULL(3.14159,2.5,0.999,TRUE)", 0.999_999_975_806_733_6),
    ];
    let mut model = new_empty_model();
    for (row, (formula, _)) in cases.iter().enumerate() {
        model._set(&format!("A{}", row + 1), formula);
    }
    model.evaluate();
    for (row, (formula, excel)) in cases.iter().enumerate() {
        assert_eq!(
            number(&model, &format!("A{}", row + 1)),
            *excel,
            "{formula}"
        );
    }
}

#[test]
fn weibull_cdf_switches_from_expm1_at_t_one() {
    // probes-numeric/t2239-weibull-cdf-switch (excel-graph): A1 (t = 0.1) and A113 (t = 0.9917)
    // take Excel's expm1, A114 (t = 1.3506) is 1-EXP(-t); the two forms differ in the last bit there.
    let mut model = new_empty_model();
    model._set("A1", "=WEIBULL.DIST(0.1,1,1,TRUE)");
    model._set("A2", "=WEIBULL.DIST(0.9917,1,1,TRUE)");
    model._set("A3", "=WEIBULL.DIST(1.3506,1,1,TRUE)");
    model.evaluate();
    assert_eq!(number(&model, "A1"), 0.095_162_581_964_040_45);
    assert_eq!(number(&model, "A2"), 0.629_054_452_728_503_2);
    assert_eq!(number(&model, "A3"), 0.740_915_236_856_580_3);
}

#[test]
fn weibull_overflow_and_underflow_edges() {
    let mut model = new_empty_model();
    // (x/beta)^alpha overflows: #NUM! (gen-m9/0bbab6206b5c9a07 A340, 329b177b5d5165ad A101).
    model._set("A1", "=WEIBULL.DIST(100,1E+10,0.01,1)");
    model._set("A2", "=WEIBULL(1E+300,2.5,100,0.5)");
    model._set("A3", "=WEIBULL(2,1E+5,0.5,TRUE)");
    // x = 0 is 0 before any power (gen-m9 T-2256 sweep).
    model._set("A4", "=WEIBULL(0,1.5E+10,0.01,1)");
    // The density's beta^alpha underflows to 0: #DIV/0! (gen-m9/941c409532603d32 A142,
    // 0620505280d2ea6e A61).
    model._set("A5", "=WEIBULL.DIST(0.1,10000000000.5,0.2,FALSE)");
    // The density's beta^alpha overflows: #NUM!, though the density's limit is 0
    // (gen-m9/a63d1c7d6f96c56d A352, 7fd0a98183c6babf A390, f82473275380882e A356).
    model._set("A6", "=WEIBULL.DIST(0.3,99999,99999,\"FALSE\")");
    model._set("A7", "=WEIBULL(1,1E+8,1E+10,FALSE)");
    model._set("A8", "=WEIBULL.DIST(1E-10,29178,1.5E+10,\"FALSE\")");
    // The cdf takes x/beta and stays a number where beta^alpha overflows
    // (gen-m9/7f7ca726c3f68554 A308).
    model._set("A9", "=WEIBULL(1E+15,42,10000000001,TRUE)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"#NUM!");
    assert_eq!(model._get_text("A2"), *"#NUM!");
    assert_eq!(model._get_text("A3"), *"#NUM!");
    assert_eq!(model._get_text("A4"), *"0");
    assert_eq!(model._get_text("A5"), *"#DIV/0!");
    assert_eq!(model._get_text("A6"), *"#NUM!");
    assert_eq!(model._get_text("A7"), *"#NUM!");
    assert_eq!(model._get_text("A8"), *"#NUM!");
    assert_eq!(model._get_text("A9"), *"1");
}
