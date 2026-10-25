//! Excel's GAMMALN and GAMMALN.PRECISE (one routine), as the T-1941 research identified it
//! (docs/excel-semantics/statistics.md "GAMMA, GAMMALN and LN kernels (T-1941)"; T-2128). Every
//! operation is an x87 op stored to a double, `fl53(fl64(.))` (`arithmetic.rs` `extended_*`),
//! except where a kernel keeps a stage in a register ([`Register`]); LN is Excel's FYL2X
//! ([`excel_ln`]). The pieces, by the exact comparisons Excel makes:
//! - x < 0.7 (the double 0x1.6666666666666p-1): `fl(GAMMALN(fl(x + 1)) - LN(x))`, once, since
//!   x + 1 is at least 1;
//! - [0.7, 1.5]: K3, `fl(fl(x - 1) * fl(N/D))`;
//! - (1.5, 2]: `fl(fl(x - 1) * fl(fl(x - 2) * fl(N/D)))`;
//! - (2, 3]: K2, `fl(fl(x - 2) * fl(N/D))`;
//! - (3, 8): t = x - k in [2, 3) and the ascending stored product p = t (t + 1) ... (t + k - 1),
//!   then `fl(K2(t) + LN(p))`.
//!
//! The ends are Excel's: GAMMALN(1.5) is K3's -0.12078223763524523 (the (1.5, 2] kernel gives
//! ...452), GAMMALN(3) is K2's 0.6931471805599454 (the reduction would give LN(2) = ...453), and
//! GAMMALN(4) is the reduction's LN(6) (ironcalc-calc `GAMMALN_GAMMALNPRECISE.xlsx` A5, A9, desktop
//! cache; `kmyst-gam-c304`, `r203`, `r205`, excel-graph);
//! - x >= 8: Stirling's series with exactly 10 Bernoulli terms, every op stored (bit-exact,
//!   `gammaln_big_check.c`).
//!
//! The four rational kernels' coefficients are in `gamma_kernels.rs`: they have Excel's structure
//! and the best known coefficients, and are not yet bit-exact (T-2122).

use super::gamma_kernels::{K2_CAND9_D, K2_CAND9_N, K2_D, K2_N, K3_D, K3_N, R15_D, R15_N};
use crate::arithmetic::{extended_product, extended_quotient, extended_sum};
use crate::x87_log10::excel_ln;
use crate::x87_register::Register;

/// Below this (the double nearest 0.7, compared exactly) GAMMALN recurs once upward.
const RECURSION_BELOW: f64 = f64::from_bits(0x3FE6_6666_6666_6666); // 0x1.6666666666666p-1

/// ln √(2π) as Excel's Stirling sum adds it: one ulp below the correctly rounded double, which
/// `loader.rs`'s `LN_SQRT_2PI_CORRECTLY_ROUNDED` is (the two stay distinct on purpose).
pub(super) const LN_SQRT_2PI: f64 = 0.918_938_533_204_672_7;

/// The Stirling series' Bernoulli terms `B_2k / (2k (2k - 1))`, k = 1..10, each a correctly rounded
/// double quotient (as `gammaln_big_check.c`'s `1.0/12, -1.0/360, ...`). Excel uses exactly ten:
/// eleven or nine miss points from x = 8.
const STIRLING: [f64; 10] = [
    1.0 / 12.0,
    -1.0 / 360.0,
    1.0 / 1260.0,
    -1.0 / 1680.0,
    1.0 / 1188.0,
    -691.0 / 360_360.0,
    1.0 / 156.0,
    -3617.0 / 122_400.0,
    43867.0 / 244_188.0,
    -174_611.0 / 125_400.0,
];

/// x - 2 below this takes reviewer 3's cand9 K2 coefficients (HANDOFF2 §12.1, §12.6), measured
/// on the T-1941 data (statistics.md).
const K2_CAND9_BELOW: f64 = 0.01;

/// Excel's GAMMALN(x) for x > 0; NaN for anything else, and infinity past the double range.
pub(crate) fn excel_gammaln(x: f64) -> f64 {
    if !(x > 0.0 && x.is_finite()) {
        return f64::NAN;
    }
    if x < RECURSION_BELOW {
        // fl(x + 1) is at least 1, so this recurs once.
        let up = gammaln_from_0_7(extended_sum(x, 1.0));
        return extended_sum(up, -excel_ln(x));
    }
    gammaln_from_0_7(x)
}

/// GAMMALN on x >= 0.7.
fn gammaln_from_0_7(x: f64) -> f64 {
    if x <= 1.5 {
        k3(x)
    } else if x <= 2.0 {
        kernel_1_5_to_2(x)
    } else if x <= 3.0 {
        k2(x)
    } else if x < 8.0 {
        reduced_to_k2(x)
    } else {
        stirling(x)
    }
}

/// (3, 8): t = x - k in [2, 3), and `fl(K2(t) + LN(t (t + 1) ... (t + k - 1)))` with the product
/// ascending and stored at each step. Every t + i is exact (it lies between t and x, on x's grid),
/// and at most five steps run.
fn reduced_to_k2(x: f64) -> f64 {
    let mut t = x;
    let mut steps = 0_u8;
    while t >= 3.0 && steps < 5 {
        t -= 1.0;
        steps += 1;
    }
    let mut product = t;
    for i in 1..steps {
        product = extended_product(product, t + f64::from(i));
    }
    extended_sum(k2(t), excel_ln(product))
}

/// K2 on [2, 3] (`gamma_kernels.rs`); K2(2) = 0.
fn k2(x: f64) -> f64 {
    let t = x - 2.0;
    let ratio = if t < K2_CAND9_BELOW {
        extended_quotient(
            stored_horner_leading_first(&K2_CAND9_N, x),
            stored_horner_leading_first(&K2_CAND9_D, x),
        )
    } else {
        extended_quotient(
            register_head_horner(&K2_N, x, 4),
            register_head_horner(&K2_D, x, 3),
        )
    };
    extended_product(t, ratio)
}

/// K3 on [0.7, 1.5] (`gamma_kernels.rs`).
fn k3(x: f64) -> f64 {
    let ratio = extended_quotient(
        stored_horner_constant_first(&K3_N, x),
        stored_horner_constant_first(&K3_D, x),
    );
    extended_product(x - 1.0, ratio)
}

/// GAMMALN on (1.5, 2] (`gamma_kernels.rs`).
fn kernel_1_5_to_2(x: f64) -> f64 {
    let ratio = extended_quotient(
        stored_horner_constant_first(&R15_N, x),
        stored_horner_constant_first(&R15_D, x),
    );
    extended_product(x - 1.0, extended_product(x - 2.0, ratio))
}

/// Stirling's series on x >= 8 (module docs): corr = LN(x); a = x - 0.5; b = a corr; d = b - x;
/// e = d + ln √(2π); z = 1/x; z2 = z z; r = `B_10`; r = r z2 + `B_k` for k = 9..1; GAMMALN = e + r z.
fn stirling(x: f64) -> f64 {
    let corr = excel_ln(x);
    let a = extended_sum(x, -0.5);
    let b = extended_product(a, corr);
    let d = extended_sum(b, -x);
    let e = extended_sum(d, LN_SQRT_2PI);
    extended_sum(e, stirling_series(extended_quotient(1.0, x)))
}

/// The Stirling series in z = 1/x as both GAMMALN and GAMMA's x >= 10 path sum it: z2 = z z;
/// r = `B_10`; r = r z2 + `B_k` for k = 9..1; the series is r z, every op stored.
pub(super) fn stirling_series(z: f64) -> f64 {
    let z2 = extended_product(z, z);
    let mut r = STIRLING[9];
    for &term in STIRLING[..9].iter().rev() {
        r = extended_sum(extended_product(r, z2), term);
    }
    extended_product(r, z)
}

/// Raw-x Horner with every product and sum stored, the leading coefficient first.
fn stored_horner_leading_first(coefficients: &[f64], x: f64) -> f64 {
    let (&leading, rest) = coefficients.split_first().unwrap_or((&0.0, &[]));
    rest.iter()
        .fold(leading, |sum, &c| extended_sum(extended_product(sum, x), c))
}

/// Raw-x Horner with every product and sum stored, the constant term first.
pub(super) fn stored_horner_constant_first(coefficients: &[f64], x: f64) -> f64 {
    let (&leading, rest) = coefficients.split_last().unwrap_or((&0.0, &[]));
    rest.iter()
        .rev()
        .fold(leading, |sum, &c| extended_sum(extended_product(sum, x), c))
}

/// Raw-x Horner, the leading coefficient first, with the sums of stages 1 to `stored_from - 1`
/// kept in an x87 register and every op from stage `stored_from`'s product on stored
/// (`k2_ref.py` `chain`).
fn register_head_horner(coefficients: &[f64], x: f64, stored_from: usize) -> f64 {
    let (&leading, rest) = coefficients.split_first().unwrap_or((&0.0, &[]));
    let (head_terms, stored_terms) = rest.split_at((stored_from - 1).min(rest.len()));
    let register_x = Register::load(x);
    let head = head_terms.iter().fold(Register::load(leading), |sum, &c| {
        sum.mul(register_x).add(Register::load(c))
    });
    let Some((&first, later)) = stored_terms.split_first() else {
        return head.store();
    };
    // The hand-off: the register times x, stored, then the first stored sum.
    let sum = extended_sum(head.mul(register_x).store(), first);
    later
        .iter()
        .fold(sum, |sum, &c| extended_sum(extended_product(sum, x), c))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::kernel_rows::{fixture_rows, full_rows};

    /// The regions the fixture and the full-run test score separately ([`region`]), by the exact
    /// cuts of [`excel_gammaln`]; K2 is split at 2.02 (cand9 covers (2, 2.01)).
    const REGIONS: [(&str, f64, f64); 7] = [
        ("x < 0.7", 0.0, RECURSION_BELOW),
        ("K3 [0.7, 1.5]", RECURSION_BELOW, 1.5),
        ("(1.5, 2]", 1.5, 2.0),
        ("K2 (2, 2.02)", 2.0, 2.02),
        ("K2 [2.02, 3]", 2.02, 3.0),
        ("(3, 8)", 3.0, 8.0),
        ("x >= 8", 8.0, f64::INFINITY),
    ];

    fn region(x: f64) -> usize {
        if x < RECURSION_BELOW {
            0
        } else if x <= 1.5 {
            1
        } else if x <= 2.0 {
            2
        } else if x < 2.02 {
            3
        } else if x <= 3.0 {
            4
        } else if x < 8.0 {
            5
        } else {
            6
        }
    }

    /// The committed sample (`test/fixtures/gammaln_kernel.fixture`: 1,500 random distinct x per
    /// region of the T-1941 GAMMALN.PRECISE cells): the engine equals the research checkers
    /// composed (`glref.py`'s numpy-x87 models of `k2_ref.py`, cand9, `w5/p4/ss_fc.txt`,
    /// `w6/data/xl_chain_d3ls.txt`, `gammaln_big_check.c`) on every point, and Excel at the
    /// research rates.
    #[test]
    fn gammaln_equals_the_research_model_on_the_fixture() {
        let rows = fixture_rows(include_str!("../../test/fixtures/gammaln_kernel.fixture"));
        assert_eq!(rows.len(), 10_500);
        let mut exact = [0_usize; 7];
        for row in &rows {
            let value = excel_gammaln(row.x);
            assert_eq!(
                value.to_bits(),
                row.model.to_bits(),
                "GAMMALN({:e}) vs the model",
                row.x
            );
            if value.to_bits() == row.excel.to_bits() {
                exact[region(row.x)] += 1;
            }
        }
        // Out of 1,500 each (statistics.md "GAMMA, GAMMALN and LN kernels (T-1941)").
        assert_eq!(exact, [1313, 453, 627, 1476, 1427, 1486, 1500]);
    }

    /// Exact values and the region ends (module docs): GAMMALN(1) = GAMMALN(2) = 0, 1.5 takes K3
    /// and 3 takes K2 (Excel's cells), and 4..7 are LN of the factorial through the reduction.
    #[test]
    fn exact_points() {
        assert_eq!(excel_gammaln(1.0), 0.0);
        assert_eq!(excel_gammaln(2.0), 0.0);
        assert_eq!(excel_gammaln(1.5), -0.120_782_237_635_245_23);
        assert_eq!(excel_gammaln(3.0), 0.693_147_180_559_945_4);
        assert_eq!(excel_gammaln(4.0), excel_ln(6.0));
        assert_eq!(excel_gammaln(7.0), excel_ln(720.0));
    }

    /// No input panics or loops: non-positive and non-finite arguments are NaN, the extremes are
    /// finite or infinite, never NaN.
    #[test]
    fn every_double_is_handled() {
        for x in [0.0, -0.0, -1.0, f64::NEG_INFINITY, f64::INFINITY, f64::NAN] {
            assert!(excel_gammaln(x).is_nan(), "{x}");
        }
        for x in [
            f64::from_bits(1),
            f64::MIN_POSITIVE,
            1e-300,
            0.7,
            7.999_999_999_999_999,
            1e300,
        ] {
            assert!(excel_gammaln(x).is_finite(), "{x}");
        }
        assert!(!excel_gammaln(f64::MAX).is_nan());
    }

    /// The full T-1941 GAMMALN.PRECISE runs by region, when `RUSTSHEET_T1941_ROWS` names the
    /// extracted rows (`crate::test::kernel_rows`); the rates are in statistics.md.
    #[test]
    #[ignore = "needs the T-1941 research data (RUSTSHEET_T1941_ROWS)"]
    fn gammaln_matches_the_t1941_runs() {
        let Some(rows) = full_rows("gl") else {
            return;
        };
        let mut seen = std::collections::HashSet::new();
        let mut counts = [(0_usize, 0_usize); 7];
        for &(x, excel) in &rows {
            if !seen.insert(x.to_bits()) {
                continue;
            }
            let entry = &mut counts[region(x)];
            entry.1 += 1;
            if excel_gammaln(x).to_bits() == excel.to_bits() {
                entry.0 += 1;
            }
        }
        for (&(name, _, _), &(exact, total)) in REGIONS.iter().zip(&counts) {
            #[allow(
                clippy::cast_precision_loss,
                reason = "counts below 2^53 convert exactly"
            )]
            let rate = exact as f64 / total as f64;
            eprintln!("{name}: {exact} / {total} = {rate:.4}");
        }
        // Stirling is bit-exact given Excel's LN: its 4 misses (9.612345678912595,
        // 9.890123456796799, 3141590.0000053067, 1.2345670000006508e100) are LN's FYL2X near-ties,
        // where Excel's own LN column took the other neighbour in the same run.
        assert_eq!(counts[6], (292_645, 292_649));
    }
}
