//! Excel's GAMMA, as the T-1941 research identified it (docs/excel-semantics/statistics.md "GAMMA
//! is Excel's own x87 routine"; T-2129). Every operation is an x87 op stored to a double,
//! `fl53(fl64(.))` (`arithmetic.rs` `extended_*`); LN is Excel's FYL2X ([`excel_ln`]), EXP its
//! model V6 ([`excel_exp`]) and the reflection's sine an FSIN, RN64 of the true sine. The paths,
//! by the exact comparisons Excel makes:
//! - an integer n <= 88: the downward product (n - 1)(n - 2)...2;
//! - an exact half-integer m + 1/2: the ascending product 1/2 * 3/2 * ... * (x - 1) times √π, and
//!   for a negative one ±fl(fl(π / GAMMA(q)) / q), q = -x;
//! - x >= 10: x^(x/2) by binary powering, twice, times EXP terms (`gamma_big_check.c`);
//! - 1 <= x < 10: the descending stored product (x - 1)(x - 2)... down to y in [1, 2), times G(y);
//! - 0 < x < 1: fl(fl(1/x) * G(fl(x + 1)));
//! - -10 <= x < 0: z = 1/x, y = x + 1, then z = z / y and y = y + 1 while y < 1, and z G(y);
//! - x < -10: the reflection ±fl(fl(π / GAMMA(q)) / fl(q s)), s = |FSIN(fl(π z))|.
//!
//! G on [1, 2) is the kernel K1, `fl(N/D)` (`gamma_kernels.rs`): Excel's structure with the best
//! known coefficients, not yet bit-exact (T-2122).

use super::gamma_kernels::{K1_D, K1_N};
use crate::arithmetic::{extended_product, extended_quotient, extended_sqrt, extended_sum};
use crate::functions::math_and_trigonometry::excel_fsin;
use crate::x87_exp::excel_exp;
use crate::x87_log10::excel_ln;

/// π and √π as doubles.
const PI: f64 = std::f64::consts::PI;
const SQRT_PI: f64 = f64::from_bits(0x3FFC_5BF8_91B4_EF6B); // 0x1.c5bf891b4ef6bp+0

/// √(2π) as Excel's x >= 10 path multiplies by it (the literal 2.50662827463100050242).
const SQRT_2PI: f64 = f64::from_bits(0x4004_0D93_1FF6_2706); // 0x1.40d931ff62706p+1

/// Excel's GAMMA(x); NaN at the poles (0 and the negative integers) and for a non-finite x, an
/// infinity past the double range (the caller's #NUM!).
pub(crate) fn excel_gamma(x: f64) -> f64 {
    if !x.is_finite() {
        return f64::NAN;
    }
    if x == x.floor() {
        if x <= 0.0 {
            return f64::NAN;
        }
        if x <= 88.0 {
            return downward_factorial(x);
        }
    }
    if x == x.floor() + 0.5 {
        return half_integer(x);
    }
    if x >= 10.0 {
        // GAMMA overflows from 171.6243769563027 on; far past it the terms would be inf * 0.
        if x > 172.0 {
            return f64::INFINITY;
        }
        return large(x);
    }
    if x >= 1.0 {
        let mut y = x;
        let mut z = 1.0;
        // At most eight steps: y < 10.
        while y >= 2.0 {
            y -= 1.0;
            z = extended_product(z, y);
        }
        return extended_product(z, k1(y));
    }
    if x > 0.0 {
        return extended_product(extended_quotient(1.0, x), k1(extended_sum(x, 1.0)));
    }
    if -x <= 10.0 {
        let mut z = extended_quotient(1.0, x);
        let mut y = extended_sum(x, 1.0);
        // At most ten steps: x >= -10.
        while y < 1.0 {
            z = extended_quotient(z, y);
            y = extended_sum(y, 1.0);
        }
        return extended_product(z, k1(y));
    }
    reflection(x)
}

/// (n - 1)(n - 2)...2, each product stored, for an integer 1 <= n <= 88 (T-1524).
fn downward_factorial(n: f64) -> f64 {
    let mut product = 1.0;
    let mut k = n - 1.0;
    while k >= 2.0 {
        product = extended_product(product, k);
        k -= 1.0;
    }
    product
}

/// An exact half-integer: 1/2 * 3/2 * ... * (x - 1) ascending, then times √π (172 of 172 points);
/// a negative one is ±fl(fl(π / GAMMA(q)) / q), q = -x (171 of 171), 0 from x = -171.5 on.
fn half_integer(x: f64) -> f64 {
    if x > 0.0 {
        let mut product: f64 = 1.0;
        let mut k = 0.5;
        // At most 172 steps: GAMMA beyond 171.5 overflows, and the product with it.
        while k < x && product.is_finite() {
            product = extended_product(product, k);
            k += 1.0;
        }
        return extended_product(product, SQRT_PI);
    }
    let q = -x;
    let magnitude = flush_subnormal(extended_quotient(extended_quotient(PI, half_integer(q)), q));
    signed(q, magnitude)
}

/// x >= 10 (module docs; 155,714 of 155,964 points): h = x/2, r = x^floor(h) by right-to-left
/// binary powering, v = r EXP((h - floor(h)) LN(x)) = x^(x/2), the Stirling series s in 1/x, and
/// GAMMA = EXP(-x) √(1/x) v v √(2π) EXP(s), every product stored in that order.
fn large(x: f64) -> f64 {
    let half = 0.5 * x;
    let whole = half.floor();
    let fraction = half - whole;
    // 2^63 > whole for every finite x here (x < 2^1024): the powering runs at most 1024 rounds.
    let mut exponent = if whole < 9.0e18 {
        whole as u64
    } else {
        u64::MAX
    };
    let mut power: Option<f64> = None;
    let mut base = x;
    while exponent != 0 {
        if exponent & 1 == 1 {
            power = Some(power.map_or(base, |p| extended_product(p, base)));
        }
        exponent >>= 1;
        if exponent != 0 {
            base = extended_product(base, base);
        }
    }
    let v = extended_product(
        power.unwrap_or(1.0),
        excel_exp(extended_product(fraction, excel_ln(x))),
    );
    let w1 = extended_quotient(1.0, x);
    let series = super::gammaln::stirling_series(w1);
    let w = excel_exp(series);
    let mut t = extended_product(excel_exp(-x), extended_sqrt(w1));
    t = extended_product(t, v);
    t = extended_product(t, v);
    t = extended_product(t, SQRT_2PI);
    extended_product(t, w)
}

/// x < -10: q = -x; p = floor(q); z = q - p, or q - (p + 1) when z > 1/2; s = |FSIN(fl(π z))|;
/// GAMMA = ±fl(fl(π / GAMMA(q)) / fl(q s)) (6,143 of 6,144 points; the miss is an FSIN tie).
fn reflection(x: f64) -> f64 {
    let q = -x;
    let mut p = q.floor();
    let mut z = q - p;
    if z > 0.5 {
        p += 1.0;
        z = q - p;
    }
    let s = excel_fsin(extended_product(PI, z)).abs();
    let gamma_q = excel_gamma(q);
    if !gamma_q.is_finite() {
        return 0.0;
    }
    let magnitude = flush_subnormal(extended_quotient(
        extended_quotient(PI, gamma_q),
        extended_product(q, s),
    ));
    signed(q, magnitude)
}

/// `magnitude` with the sign of GAMMA(-q) for a non-integer q > 0, (-1)^ceil(q); a flushed 0 is
/// +0, as Excel gives it (GAMMA(-171.5), `kmyst-gam-h102`).
fn signed(q: f64, magnitude: f64) -> f64 {
    if magnitude != 0.0 && q.ceil() % 2.0 == 1.0 {
        -magnitude
    } else {
        magnitude
    }
}

/// Excel flushes a subnormal GAMMA to 0 (GAMMA(-171.5) = 0, `kmyst-gam-h102`).
fn flush_subnormal(v: f64) -> f64 {
    if v.abs() < f64::MIN_POSITIVE {
        0.0
    } else {
        v
    }
}

/// G on [1, 2): K1 = fl(N/D), raw-y Horner with every op stored (`gamma_kernels.rs`).
fn k1(y: f64) -> f64 {
    extended_quotient(
        super::gammaln::stored_horner_constant_first(&K1_N, y),
        super::gammaln::stored_horner_constant_first(&K1_D, y),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::kernel_rows::{fixture_rows, full_rows};

    /// The regions the fixture and the full-run test score separately, in [`excel_gamma`]'s
    /// order: integers, half-integers, x >= 10, [1, 10), (0, 1), [-10, 0), x < -10.
    const REGIONS: [&str; 7] = [
        "integers",
        "half-integers",
        "x >= 10",
        "[1, 10)",
        "(0, 1)",
        "[-10, 0)",
        "x < -10",
    ];

    fn region(x: f64) -> usize {
        if x == x.floor() {
            0
        } else if x == x.floor() + 0.5 {
            1
        } else if x >= 10.0 {
            2
        } else if x >= 1.0 {
            3
        } else if x > 0.0 {
            4
        } else if -x <= 10.0 {
            5
        } else {
            6
        }
    }

    /// The committed sample (`test/fixtures/gamma_kernel.fixture`): the engine equals the
    /// research checkers composed (`docs/research/t1941/engine/garef.py`: `gamma_big_check.c`,
    /// `gamma_neg_check.py`, W4's `plain1.txt` K1, EXP model V6, the FYL2X LN, FSIN as RN64 of the
    /// sine) on every point, and Excel at the research rates.
    #[test]
    fn gamma_equals_the_research_model_on_the_fixture() {
        let rows = fixture_rows(include_str!("../../test/fixtures/gamma_kernel.fixture"));
        let mut exact = [0_usize; 7];
        let mut total = [0_usize; 7];
        for row in &rows {
            let value = excel_gamma(row.x);
            assert_eq!(
                value.to_bits(),
                row.model.to_bits(),
                "GAMMA({:e}) vs the model",
                row.x
            );
            total[region(row.x)] += 1;
            if value.to_bits() == row.excel.to_bits() {
                exact[region(row.x)] += 1;
            }
        }
        assert_eq!(total, [16, 314, 600, 600, 600, 600, 600]);
        assert_eq!(exact, [16, 314, 596, 458, 481, 524, 600]);
    }

    /// Excel's cells (statistics.md "GAMMA and GAMMALN sweep (T-1042)"): the overflow edge, the
    /// poles, near zero, small integers exact, half-integers from √π.
    #[test]
    fn excel_cells() {
        assert_eq!(excel_gamma(171.5), 9.483_367_566_824_801e307);
        assert_eq!(excel_gamma(171.6), 1.585_896_909_667_250_8e308);
        for x in [171.7, 171.99, 172.0, 200.0, 1e300] {
            assert!(!excel_gamma(x).is_finite(), "{x}");
        }
        for x in [
            0.0,
            -1.0,
            -2.0,
            -10.0,
            f64::NAN,
            f64::INFINITY,
            f64::NEG_INFINITY,
        ] {
            assert!(excel_gamma(x).is_nan(), "{x}");
        }
        assert_eq!(excel_gamma(0.5), 1.772_453_850_905_516);
        assert_eq!(excel_gamma(-0.5), -3.544_907_701_811_031_8);
        assert_eq!(excel_gamma(1e-300), 9.999_999_999_999_999e299);
        assert_eq!(excel_gamma(-1e-300), -9.999_999_999_999_999e299);
        assert_eq!(excel_gamma(26.0), 1.551_121_004_333_098_4e25);
        assert_eq!(excel_gamma(-171.5).to_bits(), 0.0_f64.to_bits());
        for n in 1..=25_i32 {
            let factorial: f64 = (1..n).map(f64::from).product();
            assert_eq!(excel_gamma(f64::from(n)), factorial, "GAMMA({n})");
        }
    }

    /// No input panics or runs unbounded: huge negatives reflect to 0 or are poles, tiny and
    /// subnormal arguments are finite or infinite.
    #[test]
    fn every_double_is_handled() {
        for x in [
            -1e300,
            -1e15 - 0.5,
            -170.6,
            -10.000_000_000_000_002,
            -9.999_999_999_999_998,
            f64::from_bits(1),
            -f64::from_bits(1),
            f64::MIN_POSITIVE,
            0.999_999_999_999_999_9,
            1.999_999_999_999_999_8,
            9.999_999_999_999_998,
            171.624_376_956_302_7,
            f64::MAX,
            -f64::MAX,
        ] {
            let _ = excel_gamma(x);
        }
    }

    /// The full T-1941 GAMMA runs by region, when `RUSTSHEET_T1941_ROWS` names the extracted rows
    /// (`crate::test::kernel_rows`); the rates are in statistics.md.
    #[test]
    #[ignore = "needs the T-1941 research data (RUSTSHEET_T1941_ROWS)"]
    fn gamma_matches_the_t1941_runs() {
        let Some(rows) = full_rows("ga") else {
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
            if excel_gamma(x).to_bits() == excel.to_bits() {
                entry.0 += 1;
            }
        }
        for (name, &(exact, total)) in REGIONS.iter().zip(&counts) {
            #[allow(
                clippy::cast_precision_loss,
                reason = "counts below 2^53 convert exactly"
            )]
            let rate = exact as f64 / total as f64;
            eprintln!("{name}: {exact} / {total} = {rate:.4}");
        }
    }
}
