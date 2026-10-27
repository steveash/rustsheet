use super::{dispatch, Branch};

fn branch(a: f64, b: f64, x: f64) -> (Branch, bool) {
    let d = dispatch(a, b, x, 1.0 - x).expect("in domain");
    (d.branch, d.swapped)
}

#[test]
fn edges_and_domain() {
    assert_eq!(branch(2.0, 3.0, 0.0), (Branch::Zero, false));
    assert_eq!(branch(2.0, 3.0, 1.0), (Branch::One, false));
    assert_eq!(branch(1e-20, 1e-20, 0.4), (Branch::Tiny, false));
    assert!(dispatch(-1.0, 2.0, 0.5, 0.5).is_none());
    assert!(dispatch(1.0, f64::NAN, 0.5, 0.5).is_none());
    assert!(dispatch(1.0, 2.0, 1.5, -0.5).is_none());
    assert!(dispatch(0.0, 0.0, 0.5, 0.5).is_none());
}

// min(a, b) <= 1: the swap at x > 0.5, and the 1992 Fortran's 0.3 (not R's 0.29) for w1 = bpser.
#[test]
fn small_shapes_follow_the_1992_dispatch() {
    assert_eq!(branch(0.5, 0.8, 0.2), (Branch::WBpser, false));
    assert_eq!(branch(0.5, 0.8, 0.7), (Branch::WBpser, true));
    assert_eq!(branch(2.0, 0.5, 0.2), (Branch::WBpser, false));
    // a0 < 1 < b0: x0 in [0.29, 0.3) is not yet w1 = bpser (R would switch at 0.29).
    assert_eq!(branch(0.5, 5.0, 0.295), (Branch::BupBgrat, false));
    assert_eq!(branch(0.5, 5.0, 0.3), (Branch::W1Bpser, false));
    assert_eq!(branch(0.5, 20.0, 0.2), (Branch::Bgrat, false));
    assert_eq!(branch(0.5, 5.0, 0.05), (Branch::WBpser, false));
    // Both shapes <= 1, a0 below min(0.2, b0) and x0^a0 > 0.9.
    assert_eq!(branch(0.01, 0.5, 0.3), (Branch::W1Bpser, false));
    assert_eq!(branch(0.01, 0.5, 0.2), (Branch::BupBgrat, false));
}

// min(a, b) > 1: the swap at the mean (λ < 0), then b0's ranges.
#[test]
fn large_shapes_follow_the_1992_dispatch() {
    assert_eq!(branch(5.0, 3.0, 0.1), (Branch::BpserLarge, false));
    assert_eq!(branch(5.0, 3.0, 0.5), (Branch::L140BupBpser, false));
    assert_eq!(branch(3.0, 5.0, 0.7), (Branch::L140BupBpser, true));
    assert_eq!(branch(5.0, 30.0, 0.02), (Branch::BpserLarge, false));
    assert_eq!(branch(5.0, 30.0, 0.05), (Branch::L140BupBpser, false));
    assert_eq!(branch(12.0, 2.0, 0.8), (Branch::L140BupBupBgrat, false));
    assert_eq!(branch(30.0, 5.0, 0.8), (Branch::L140BupBgrat, false));
    assert_eq!(branch(30.0, 20.0, 0.75), (Branch::L140BupBpser, true));
    assert_eq!(branch(50.0, 60.0, 0.45), (Branch::Bfrac, false));
    // At the mean of two shapes above 100, λ is within 3% of the smaller one: basym.
    assert_eq!(branch(500.0, 600.0, 500.0 / 1100.0).0, Branch::Basym);
}
