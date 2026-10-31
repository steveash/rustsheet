#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn empty_argument_cumulative() {
    let mut model = new_empty_model();

    model._set("A1", "=BETA.DIST(0.234, 2, 2.5, , 0.15, 1.2)");
    model._set("A2", "=BETA.DIST(0.234, 2, 2.5, C15 , 0.15, 1.2)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"0.588288667");
    assert_eq!(model._get_text("A2"), *"0.588288667");
}

#[test]
fn arguments() {
    let mut model = new_empty_model();

    model._set("A1", "=BETA.DIST()");
    model._set("A2", "=BETA.DIST(3)");
    model._set("A3", "=BETA.DIST(3, 2)");
    model._set("A4", "=BETA.DIST(3, 2, 3)");
    model._set("A5", "=BETA.DIST(3, 2, 3, 1)");
    model._set("A6", "=BETA.DIST(3, 2, 3, 1, 1)");
    model._set("A7", "=BETA.DIST(3, 2, 3, 1, 1, 10)");
    model._set("A8", "=BETA.DIST(3, 2, 3, 1, 1, 10, 1)");

    model._set("A9", "=BETA.DIST(0.23, 2, 3, 1)"); // Missing interval parameters
    model._set("A10", "=BETA.DIST(0.23, 2, 3, 1, 0)"); // Missing interval end
    model._set("A11", "=BETA.DIST(0.23, 2, 3, 1, , 1)"); // Empty interval start

    model._set("B1", "=BETA.INV()");
    model._set("B2", "=BETA.INV(0.5)");
    model._set("B3", "=BETA.INV(0.5, 2)");
    model._set("B4", "=BETA.INV(0.5, 2, 2)"); // Missing interval parameters
    model._set("B5", "=BETA.INV(0.5, 2, 2, 0)"); // Missing interval end
    model._set("B6", "=BETA.INV(0.5, 2, 2, 0, 1)");
    model._set("B7", "=BETA.INV(0.5, 2, 2, 0, 1, 10)");

    model._set("B8", "=BETA.INV(0.5, 2, 2, , 1)"); // Empty interval start

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#ERROR!");
    assert_eq!(model._get_text("A2"), *"#ERROR!");
    assert_eq!(model._get_text("A3"), *"#ERROR!");
    assert_eq!(model._get_text("A4"), *"#ERROR!");
    assert_eq!(model._get_text("A5"), *"#NUM!");
    assert_eq!(model._get_text("A6"), *"#NUM!");
    assert_eq!(model._get_text("A7"), *"0.215820759");
    assert_eq!(model._get_text("A8"), *"#ERROR!");

    assert_eq!(model._get_text("A9"), *"0.22845923");
    assert_eq!(model._get_text("A10"), *"0.22845923");
    assert_eq!(model._get_text("A11"), *"0.22845923");

    assert_eq!(model._get_text("B1"), *"#ERROR!");
    assert_eq!(model._get_text("B2"), *"#ERROR!");
    assert_eq!(model._get_text("B3"), *"#ERROR!");
    assert_eq!(model._get_text("B4"), *"0.5");
    assert_eq!(model._get_text("B5"), *"0.5");
    assert_eq!(model._get_text("B6"), *"0.5");
    assert_eq!(model._get_text("B7"), *"#ERROR!");
}

// T-1543: `statrs`' Beta inverse_cdf panics from p about 1E-200 and does not finish for
// BETA.INV(1E-100,1000,1); the quantile is found by bounded bisection on the cdf.
#[test]
fn test_fn_beta_inv_tiny_probability_is_bounded() {
    let mut model = new_empty_model();
    model._set("A1", "=BETA.INV(1E-300,5,2)");
    model._set("A2", "=BETA.INV(1E-200,1,1)");
    model._set("A3", "=BETA.INV(1E-100,1000,1)");
    model._set("A4", "=BETA.INV(1E-300,1E5,1E5)");
    model._set("A5", "=BETA.INV(1E-100,0.5,100)");
    model.evaluate();
    // Excel (probes-numeric t1543-inverse-cdf-tiny-p) agrees to 6 digits at A2-A5; at A1 it gives
    // 6.988271195E-61 (an Excel quirk, see statistics.md).
    assert_eq!(model._get_text("A1"), "6.98827E-61");
    assert_eq!(model._get_text("A2"), "1E-200");
    assert_eq!(model._get_text("A3"), "0.794328235");
    assert_eq!(model._get_text("A4"), "0.458650995");
    assert_eq!(model._get_text("A5"), "7.8736E-203");
}

// BETA.INV whose quantile rounds to 1 (T-3351, probes/t3351-beta-inv-p-near-1, excel-graph):
// exactly 1 while the flipped tail's quantile y is a normal double, #NUM! once it underflows.
#[test]
fn beta_inv_quantile_rounding_to_one() {
    let cases = [
        // y = 3.9E-81, 3.5E-60 and 5E-17: 1.
        ("=BETA.INV(1-1E-8,2,0.1)=1", "TRUE"),    // A102
        ("=BETA.INV(1-1E-6,0.5,0.3)=1", "TRUE"),  // A46
        ("=BETA.INV(1-1E-16,2,1)=1", "TRUE"),     // A126
        ("=BETAINV(1-1E-12,2,0.1)=1", "TRUE"),    // A195
        ("=BETA.INV(1-7E-10,10,0.03)=1", "TRUE"), // C198: y = 4.1E-307
        ("=BETA.INV(1-1E-9,10,0.03,3,5)", "5"),
        // y underflows (3.7E-801, 1.7E-334, 3.4E-311): #NUM!.
        ("=BETA.INV(1-1E-8,2,0.01)", "#NUM!"),  // A84
        ("=BETA.INV(1-1E-10,2,0.03)", "#NUM!"), // A95
        ("=BETA.INV(1-5E-10,2,0.03)", "#NUM!"), // A200
        ("=BETAINV(1-1E-16,2,0.01)", "#NUM!"),
        ("=BETA.INV(1-1E-8,2,0.01,3,5)", "#NUM!"), // A196
        // Away from 1 the quantile is unchanged: 0.99999999 (A57).
        ("=BETA.INV(1-1E-8,0.5,1)=0.99999998", "TRUE"),
    ];
    let mut model = new_empty_model();
    for (row, (formula, _)) in cases.iter().enumerate() {
        model._set(&format!("A{}", row + 1), formula);
    }
    model.evaluate();
    for (row, (formula, expected)) in cases.iter().enumerate() {
        assert_eq!(
            model._get_text(&format!("A{}", row + 1)),
            *expected,
            "{formula}"
        );
    }
}
