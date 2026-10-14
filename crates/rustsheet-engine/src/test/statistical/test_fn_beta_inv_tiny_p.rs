use crate::test::util::new_empty_model;

// Excel (probes-numeric t1543-beta-inv-tiny-p): BETA.INV is finite for a tiny p, and statrs'
// Beta::inverse_cdf panics from p about 1E-200. Expected values are Excel's cached values.
fn value(model: &crate::Model, cell: &str) -> f64 {
    model._get_text(cell).parse().unwrap_or(f64::NAN)
}

#[test]
fn beta_inv_tiny_p_does_not_panic() {
    let mut model = new_empty_model();
    model._set("A1", "=BETA.INV(1E-300,1,1)");
    model._set("A2", "=BETA.INV(1E-200,2,5)");
    model._set("A3", "=BETA.INV(1E-300,100,100)");
    model._set("A4", "=BETA.INV(1E-200,10000,1)");
    model._set("A5", "=BETA.INV(1E-300,0.001,0.001)");
    model._set("A6", "=BETAINV(1E-200,2,5)");
    model.evaluate();
    // _get_text rounds to 10 significant digits.
    let close = |cell: &str, expected: f64| {
        let v = value(&model, cell);
        assert!(
            (v / expected - 1.0).abs() < 1e-8,
            "{cell}: {v:e} vs {expected:e}"
        );
    };
    close("A1", 1.0e-300);
    close("A2", 2.581_988_897_471_588_7e-101);
    close("A6", 2.581_988_897_471_588_7e-101);
    // x is not small: the bisection branch; Excel gives 0.00025915036184925405.
    close("A3", 0.000_259_150_361_849_254);
    close("A4", 0.876_559_598_543_133_4);
    // Excel is #NUM! where the result underflows.
    assert_eq!(model._get_text("A5"), *"#NUM!");
}
