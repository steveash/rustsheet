#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn test_fn_hyp_geom_dist_smoke() {
    let mut model = new_empty_model();

    // Valid: PDF (non-cumulative)
    model._set("A1", "=HYPGEOM.DIST(1, 4, 12, 20, FALSE)");

    // Valid: CDF (cumulative)
    model._set("A2", "=HYPGEOM.DIST(1, 4, 12, 20, TRUE)");

    // Wrong number of arguments -> #ERROR!
    model._set("A3", "=HYPGEOM.DIST(1, 4, 12, 20)");
    model._set("A4", "=HYPGEOM.DIST(1, 4, 12, 20, TRUE, FALSE)");

    // sample_s > number_sample -> 1 for the cdf (t1049-hypgeomdist A35)
    model._set("A5", "=HYPGEOM.DIST(5, 4, 12, 20, TRUE)");

    // Domain errors:

    // population_s > number_pop -> #NUM!
    model._set("A6", "=HYPGEOM.DIST(1, 4, 25, 20, TRUE)");

    // number_sample > number_pop -> #NUM!
    model._set("A7", "=HYPGEOM.DIST(1, 25, 12, 20, TRUE)");

    model.evaluate();

    // PDF: P(X = 1)
    assert_eq!(model._get_text("A1"), *"0.13869969");

    // CDF: P(X <= 1)
    assert_eq!(model._get_text("A2"), *"0.153147575");

    assert_eq!(model._get_text("A3"), *"#ERROR!");
    assert_eq!(model._get_text("A4"), *"#ERROR!");
    assert_eq!(model._get_text("A5"), *"1");
    assert_eq!(model._get_text("A6"), *"#NUM!");
    assert_eq!(model._get_text("A7"), *"#NUM!");
}

#[test]
fn hyp_geom_dist_domain_edges_and_boolean_sample() {
    let mut model = new_empty_model();
    // probes-numeric/t1048-hypgeomdist and t1049-hypgeomdist cells.
    model._set("A1", "=HYPGEOM.DIST(5,4,8,20,FALSE)");
    model._set("A2", "=HYPGEOM.DIST(1,0,8,20,FALSE)");
    model._set("A3", "=HYPGEOM.DIST(0,0,8,20,FALSE)");
    model._set("A4", "=HYPGEOM.DIST(0,4,0,20,FALSE)");
    model._set("A5", "=HYPGEOM.DIST(-0.5,4,8,20,TRUE)");
    model._set("A6", "=HYPGEOM.DIST(TRUE,4,8,20,TRUE)");
    model._set("A7", "=HYPGEOMDIST(TRUE,4,8,20)");
    model._set("A8", "=HYPGEOM.DIST(1,TRUE,8,20,TRUE)");
    model.evaluate();
    let expected = [
        ("A1", "0"),
        ("A2", "0"),
        ("A3", "1"),
        ("A4", "1"),
        ("A5", "#NUM!"),
        ("A6", "0.465428277"),
        ("A7", "0.363261094"),
        // A TRUE count reads as 1 (probes-numeric/t1049-hypgeomdist A53, T-1849).
        ("A8", "1"),
    ];
    for (cell, value) in expected {
        assert_eq!(model._get_text(cell), *value, "{cell}");
    }
}
