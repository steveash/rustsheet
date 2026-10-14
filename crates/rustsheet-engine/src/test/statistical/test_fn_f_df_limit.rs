use crate::test::util::new_empty_model;

// Excel: df above 1E10 is #NUM! for every F-distribution function (probes-numeric t1530-f-df-cut);
// the quantile functions must also return at once.
#[test]
fn f_family_df_above_1e10_is_num() {
    let mut model = new_empty_model();
    let formulas = [
        "=F.INV(0.5,10000000001,5)",
        "=F.INV(0.5,2,10000000001)",
        "=F.INV.RT(0.5,10000000001,5)",
        "=F.INV.RT(0.5,2,10000000001)",
        "=FINV(0.5,10000000001,5)",
        "=FINV(0.5,2,10000000001)",
        "=F.DIST(2,10000000001,5,TRUE)",
        "=F.DIST(2,5,10000000001,FALSE)",
        "=F.DIST.RT(2,5,10000000001)",
        "=FDIST(2,10000000001,5)",
    ];
    for (i, f) in formulas.iter().enumerate() {
        model._set(&format!("A{}", i + 1), f);
    }
    model.evaluate();
    for i in 1..=formulas.len() {
        assert_eq!(model._get_text(&format!("A{i}")), *"#NUM!", "A{i}");
    }
}

// df = 1E10 is accepted (Excel: F.INV(0.5,1E10,5) = 1.1490395521130337) and must not hang.
#[test]
fn f_inv_at_df_limit_is_bounded() {
    let mut model = new_empty_model();
    model._set("A1", "=F.INV(0.5,10000000000.5,5)");
    model._set("A2", "=F.INV.RT(0.5,5,10000000000)");
    model._set("A3", "=F.INV(0.5,1E10,1E10)");
    model.evaluate();
    assert_ne!(model._get_text("A1"), *"#NUM!");
    assert_ne!(model._get_text("A2"), *"#NUM!");
    // Accuracy at huge df is limited by the statrs cdf (about 1E-6); TODO(T-1538): tighten to Excel's exact 1.
    assert_eq!(model._get_text("A3").get(..4), Some("1.00"));
}
