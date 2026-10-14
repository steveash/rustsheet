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
    model._set("B1", "=F.INV(0.5,10000000000.5,5)");
    model.evaluate();
    for i in 1..=formulas.len() {
        assert_eq!(model._get_text(&format!("A{i}")), *"#NUM!", "A{i}");
    }
    // 1E10 itself is accepted (df truncates first).
    assert_ne!(model._get_text("B1"), *"#NUM!");
}
