use crate::test::util::new_empty_model;

// Excel (probes-numeric t1531-t-tiny-p): the Student-t quantile is finite for a tiny p, never an
// error, and statrs' own inverse_cdf panics there.
#[test]
fn t_inv_tiny_p_does_not_panic() {
    let mut model = new_empty_model();
    model._set("A1", "=T.INV(1E-300,1)");
    model._set("A2", "=T.INV(1E-300,1.5)");
    model._set("A3", "=T.INV(1E-300,99999)");
    model._set("A4", "=T.INV.2T(1E-300,5)");
    model._set("A5", "=CONFIDENCE.T(1E-300,1,2)");
    model._set("A7", "=T.INV(1E-10,1)");
    model.evaluate();
        assert!(
            model._get_text(c).parse::<f64>().is_ok(),
            "{c}: {}",
            model._get_text(c)
        );
    }
    let a1: f64 = model._get_text("A1").parse().unwrap_or(0.0);
    assert!((a1 / -3.183_098_861_837_907e299 - 1.0).abs() < 1e-9, "{a1}");
    let a7: f64 = model._get_text("A7").parse().unwrap_or(0.0);
    assert!((a7 / -3_183_098_861.837_906_4 - 1.0).abs() < 1e-9, "{a7}");
}
