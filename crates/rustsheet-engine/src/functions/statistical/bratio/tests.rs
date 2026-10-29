use super::{beta_dist_lower_tail, closed_or_ported_tails, dispatch, half_complement, Branch};

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

/// BETA.DIST(x, a, b, TRUE) through [`beta_dist_lower_tail`] with y = fl(1 - x), as BETA.DIST forms it.
fn cdf(x: f64, a: f64, b: f64) -> f64 {
    beta_dist_lower_tail(x, 1.0 - x, a, b)
}

// Excel's values (probes-numeric, excel-graph), bit-exact through the bpser port and 1 to 9 ulp
// off on the Cephes kernel before it: w = bpser(a0, b0, x0), w1 = bpser(b0, a0, y0) on swapped
// and unswapped points, GL53's ln B on both sides of b0 = 8, and bpser for min(a, b) > 1.
#[test]
fn bpser_branches_match_excel() {
    let cells: [(f64, f64, f64, f64); 9] = [
        // t2464-r2-bpser-small-shapes A7639, A8989: both shapes below 1.
        (0.36612, 0.8696, 0.7992, 0.353_647_781_486_548_5),
        (0.30847, 0.854, 0.4308, 0.185_638_762_228_111_07),
        // A4703, A11115, A1863: swapped, one shape above 1.
        (0.67491, 1.443, 0.9544, 0.548_639_083_091_801_5),
        (0.7605, 0.6488, 1.224, 0.887_422_334_623_970_9),
        (0.76893, 0.6409, 1.109, 0.871_169_777_428_695_6),
        // A3296 (b0 < 8) and A6107 (b0 >= 8: algdiv): w1 = bpser(b0, a0, y0).
        (0.058802, 7.524, 0.3655, 6.403_455_904_329_501e-11),
        (0.0492, 27.67, 0.5934, 1.121_119_221_077_011e-37),
        // m9-beta-dist-a0p0001 A21: a tiny shape.
        (0.5, 0.0001, 2.0, 0.999_980_684_218_537_7),
        // m9-beta-dist-a2 A25: bpser for min(a, b) > 1.
        (0.1, 2.0, 2.5, 0.039_458_160_723_854_8),
    ];
    for (x, a, b, excel) in cells {
        assert_eq!(
            cdf(x, a, b).to_bits(),
            excel.to_bits(),
            "BETA.DIST({x},{a},{b},TRUE)"
        );
    }
}

// Branches not ported yet, and a shape of 1 outside the closed forms, keep today's kernel.
#[test]
fn unported_points_keep_todays_kernel() {
    use crate::functions::statistical::incomplete_beta::beta_reg_complement;
    for (x, a, b) in [
        (0.295, 0.5, 5.0),                   // bup + bgrat
        (0.5, 50.0, 60.0),                   // bfrac
        (0.916_913_946_587_537_1, 3.5, 1.0), // a0 = 1, y0 > 0.9
        (0.1, 1e308, 0.5),                   // fl(a·LN x) = -inf
        (0.1, f64::MAX, 0.5),
    ] {
        assert_eq!(
            cdf(x, a, b).to_bits(),
            beta_reg_complement(x, 1.0 - x, a, b).to_bits()
        );
    }
}

// L140 at a whole b0 and a0 > 8 runs bpser(a0, 1, x0) with ln B(a0, 1) = -LN(a0): Excel's
// values (probes-numeric, excel-graph), each on today's kernel and off before T-2506.
#[test]
fn l140_whole_b0_takes_ln_beta_of_one() {
    let cells: [(f64, f64, f64, f64); 3] = [
        // m9-beta-dist-a100 A37, A39; t0969-beta-prefactor Pref!D140.
        (0.1, 100.0, 10.0, 1.668_234_277_029_307e-88),
        (0.5, 100.0, 10.0, 7.203_401_329_871_238e-21),
        (0.2, 15.0, 8.0, 1.312_292_337_614_844e-6),
    ];
    for (x, a, b, excel) in cells {
        assert_eq!(
            dispatch(a, b, x, 1.0 - x).map(|d| d.branch),
            Some(Branch::L140BupBpser)
        );
        assert_eq!(
            cdf(x, a, b).to_bits(),
            excel.to_bits(),
            "BETA.DIST({x},{a},{b},TRUE)"
        );
    }
}

// BINOM.DIST's cdf is w1 of K(k + 1, n - k, p, fl(1 - p)), whole shapes, so L140 at a whole b0
// runs bpser(a0, 1, x0) with ln B(a0, 1) = -LN(a0) at a0 <= 8 too: exact dyadic values stay exact
// (`BINOM.DIST(1,6,0.5,TRUE)` = 7/64; the GAMMALN sum left Excel's 0.0546875 in ironcalc-calc
// BINOM.xlsx E5 2 ulp high, T-2847).
#[test]
fn l140_whole_b0_is_exact_at_a_small_a0() {
    let (a, b, p) = (2.0, 5.0, 0.5);
    assert_eq!(
        dispatch(a, b, p, 1.0 - p).map(|d| d.branch),
        Some(Branch::L140BupBpser)
    );
    let (_, upper) = closed_or_ported_tails(p, 1.0 - p, a, b).expect("ported");
    assert_eq!(upper.to_bits(), (7.0_f64 / 64.0).to_bits());
}

// bpser with both shapes above 1 and one above 8 takes the accurate ln B at half-integer shapes
// and TOMS `betaln` elsewhere (T-3007): Excel's values (probes-numeric m9-beta-dist-a10 A25,
// m9-beta-dist-a50 A25, m9-beta-dist-a2 A41; excel-graph), each off on today's kernel.
#[test]
fn bpser_above_eight_takes_a_ln_beta() {
    for (x, a, b, excel) in [
        (0.1, 10.0, 2.5, 2.458_946_199_580_436_7e-9_f64),
        (0.1, 50.0, 2.5, 2.364_143_913_947_642_6e-48),
        (0.9, 2.0, 10.0, 0.999_999_998_999_999_9),
    ] {
        assert_eq!(
            cdf(x, a, b).to_bits(),
            excel.to_bits(),
            "BETA.DIST({x},{a},{b},TRUE)"
        );
    }
}

/// One row of `test/fixtures/bratio_beta_dist.fixture`: Excel's BETA.DIST(x, a, b, TRUE) with
/// y = fl(1 - x), and the branch and swap r2-bratio's model names for the point (the model's
/// branch names, spaces written as underscores).
struct FixtureRow {
    a: f64,
    b: f64,
    x: f64,
    y: f64,
    excel: f64,
    branch: Branch,
    swapped: bool,
}

/// The rows of the bratio fixture (SPEC-0025 "Test fixtures"; written by
/// `docs/research/t3007/make_bratio_fixture.py`). A malformed line is a test failure.
fn fixture_rows() -> Vec<FixtureRow> {
    let bits = |field: &str| f64::from_bits(u64::from_str_radix(field, 16).expect("hex bits"));
    let branch = |name: &str| match name {
        "x=0" => Branch::Zero,
        "y=0" => Branch::One,
        "tiny" => Branch::Tiny,
        "fpser" => Branch::Fpser,
        "apser" => Branch::Apser,
        "w_bpser" => Branch::WBpser,
        "w1_bpser" => Branch::W1Bpser,
        "bup+bgrat" => Branch::BupBgrat,
        "bgrat" => Branch::Bgrat,
        "bpser(a,b>1)" => Branch::BpserLarge,
        "L140_bup+bpser" => Branch::L140BupBpser,
        "L140_bup+bgrat" => Branch::L140BupBgrat,
        "L140_bup+bup+bgrat" => Branch::L140BupBupBgrat,
        "bfrac" => Branch::Bfrac,
        "basym" => Branch::Basym,
        _ => panic!("unknown branch {name:?}"),
    };
    include_str!("../../../test/fixtures/bratio_beta_dist.fixture")
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(
            |line| match line.split_whitespace().collect::<Vec<_>>().as_slice() {
                [a, b, x, y, excel, name, swapped] => FixtureRow {
                    a: bits(a),
                    b: bits(b),
                    x: bits(x),
                    y: bits(y),
                    excel: bits(excel),
                    branch: branch(name),
                    swapped: *swapped == "1",
                },
                _ => panic!("malformed fixture line {line:?}"),
            },
        )
        .collect()
}

// SPEC-0025's dispatch test: on every fixture row the engine's dispatch names the branch and
// swap r2-bratio's model does (every branch region the BETA.DIST cells reach: the x0 = 0.3 cut,
// both swaps, b0 = 15, 40 and 100).
#[test]
fn fixture_dispatch_matches_the_model() {
    let rows = fixture_rows();
    assert_eq!(rows.len(), 857);
    for r in &rows {
        let d = dispatch(r.a, r.b, r.x, r.y).expect("in domain");
        assert_eq!(
            (d.branch, d.swapped),
            (r.branch, r.swapped),
            "K({}, {}, {}, {})",
            r.a,
            r.b,
            r.x,
            r.y
        );
    }
}

/// The fixture rows on a ported branch, with the engine's (w, w1) there.
fn ported_rows() -> Vec<(FixtureRow, (f64, f64))> {
    fixture_rows()
        .into_iter()
        .filter_map(|r| closed_or_ported_tails(r.x, r.y, r.a, r.b).map(|tails| (r, tails)))
        .collect()
}

// Bit-exact cells per ported branch on the fixture (T-3007's count; a ratchet: raise it when a
// change makes more exact, never lower it). The fixture's other 84 WBpser rows have a caller's
// shape of 1, which `ported_tails` leaves on today's kernel.
#[test]
fn fixture_ported_branches_score() {
    let mut scores = [
        (Branch::WBpser, 225, 122),
        (Branch::W1Bpser, 19, 16),
        (Branch::BpserLarge, 205, 123),
        (Branch::L140BupBpser, 122, 91),
    ]
    .map(|(branch, rows, exact)| (branch, rows, exact, 0, 0));
    for (r, (w, _)) in ported_rows() {
        let score = scores
            .iter_mut()
            .find(|score| score.0 == r.branch)
            .expect("a ported branch");
        score.3 += 1;
        score.4 += usize::from(w.to_bits() == r.excel.to_bits());
    }
    for (branch, rows, exact, got_rows, got_exact) in scores {
        assert_eq!(got_rows, rows, "{branch:?} rows");
        assert!(got_exact >= exact, "{branch:?}: {got_exact} bit-exact");
    }
}

// SPEC-0025's properties on every ported branch: one tail computed and the other its stored
// complement (0.5 - w) + 0.5.
#[test]
fn ported_tails_are_stored_complements() {
    for (r, (w, w1)) in ported_rows() {
        assert!(
            w1 == half_complement(w) || w == half_complement(w1),
            "K({}, {}, {}, {}) = ({w}, {w1})",
            r.a,
            r.b,
            r.x,
            r.y
        );
    }
}

// K(a, b, x, y) = swap(K(b, a, y, x)) wherever the two calls swap differently and so compute the
// same oriented point; where both swap or neither does (x or y = 0.5 at min(a, b) <= 1, λ = 0
// at a = b) they are two different evaluations, a few ulp apart.
#[test]
fn ported_tails_are_swap_symmetric() {
    let mut compared = 0;
    for (r, (w, w1)) in ported_rows() {
        let swaps = |a, b, x, y| dispatch(a, b, x, y).map(|d| d.swapped);
        if swaps(r.a, r.b, r.x, r.y) == swaps(r.b, r.a, r.y, r.x) {
            continue;
        }
        compared += 1;
        let mirrored = closed_or_ported_tails(r.y, r.x, r.b, r.a).map(|(p, q)| (q, p));
        assert_eq!(
            mirrored.map(|(p, q)| (p.to_bits(), q.to_bits())),
            Some((w.to_bits(), w1.to_bits())),
            "K({}, {}, {}, {})",
            r.a,
            r.b,
            r.x,
            r.y
        );
    }
    assert_eq!(compared, 536);
}

// Monotone in x on each shape pair's sorted ported rows, except where Excel's own value falls
// between two x an ulp apart: BETA.DIST(x, 2.5, 2.5, TRUE) is 0.05000000000000001 at
// 0.16528026665565798 and 0.05 at the next double (probes-numeric t1745-beta-inv-roundtrip G84,
// H84; statistics.md "BETA.DIST's cdf is not monotone in its last bit").
#[test]
fn ported_tails_rise_with_x_where_excel_does() {
    let mut rows = ported_rows();
    rows.sort_by(|(p, _), (q, _)| {
        (p.a, p.b, p.x)
            .partial_cmp(&(q.a, q.b, q.x))
            .expect("finite")
    });
    let mut pairs = 0;
    for pair in rows.windows(2) {
        let [(p, (wp, _)), (q, (wq, _))] = pair else {
            continue;
        };
        if (p.a, p.b) != (q.a, q.b) {
            continue;
        }
        pairs += 1;
        assert!(
            wq >= wp || q.excel < p.excel,
            "({}, {}): {wp} at {}, {wq} at {}",
            p.a,
            p.b,
            p.x,
            q.x
        );
    }
    assert_eq!(pairs, 422);
}
