//! Tests of `x87_exp`: bit identity of the fast paths with the series and with the u128 core
//! before T-2102, the estimates' error bounds, the extended-argument fixture and the per-call
//! timing.

use super::*;

/// The integer core before T-2102 (u128 throughout, a u128 division for the reciprocal and
/// the general roundings), kept verbatim as the reference [`exp_with`] is compared with: do
/// not edit or reuse it.
fn exp_with_reference(a: Register, two_to_the_minus_one: fn(u128, u32) -> (u128, i32)) -> f64 {
    let (negative, a_significand, a_exponent) = a.parts();
    // A zero (no top bit) gives 1. Otherwise 2^top <= |a| < 2^(top + 1); a significand has at
    // most 64 bits, so its bit index fits an i32.
    let Some(top_bit) = a_significand.checked_ilog2() else {
        return 1.0;
    };
    let top = a_exponent + i32::try_from(top_bit).unwrap_or(0);
    // |a| < 2^-65 gives exactly 1: |u| < 2^-64 then, under half a 64-bit ulp of 1, so w = 1.
    if top < -65 {
        return 1.0;
    }
    // |a| * log2 e beyond T_LIMIT: certain from the exponent outside [1024, 2048), and for a
    // double the same f64 test as before on the stored value (exact for a loaded double).
    if top >= 11 || (top == 10 && a.store().abs() * std::f64::consts::LOG2_E > T_LIMIT) {
        return if negative { 0.0 } else { f64::INFINITY };
    }
    // Step 1: t = T * 2^t_exponent, T a 64-bit integer (|a| >= 2^-65, and a significand of at
    // least 53 bits times the 64-bit constant has more than 64 bits to round).
    let (t, shift) = round_to_bits(a_significand * u128::from(LOG2_E_64), 64, false);
    let t_exponent = a_exponent - 63 + shift;
    // Step 2: |t| <= T_LIMIT < 2^11 and |t| >= 2^-66, so `drop` is in 53..=130. The guards above
    // make every `try_from` fallback below unreachable: t < 2^64, n <= 2^11 and |f| < 2^64.
    let Ok(drop) = u32::try_from(-t_exponent) else {
        return f64::NAN;
    };
    let (n, fraction) = if drop >= 65 {
        // |t| < 2^(64 - drop) <= 1/2: n = 0 and f = t.
        (0_i32, i128::try_from(t).unwrap_or(0))
    } else {
        let integer = t >> drop;
        let rest = t & ((1 << drop) - 1);
        let half = 1 << (drop - 1);
        // Integer bit operations, not `||`: a data-dependent branch here mispredicts half the time.
        let n = integer + (u128::from(rest > half) | (u128::from(rest == half) & (integer & 1)));
        let f = i128::try_from(t).unwrap_or(0) - i128::try_from(n << drop).unwrap_or(0);
        (i32::try_from(n).unwrap_or(0), f)
    };
    // f * sign(a) is the true fraction: `fraction` is the magnitude-side remainder of |t|.
    let negative_f = (fraction < 0) != negative;
    let n = if negative { -n } else { n };
    // Step 3: w = W * 2^w_exponent, W a 64-bit integer.
    let (mut w, mut w_exponent) = (1_u128 << 63, -63);
    if fraction != 0 {
        let (u, u_exponent) = two_to_the_minus_one(fraction.unsigned_abs(), drop);
        // u < 0.415, so u_exponent <= -65 and 1 = 2^gap units of u.
        let gap = u32::try_from(-u_exponent).unwrap_or(128);
        if gap < 128 {
            let (sum, sum_shift) = round_to_bits((1 << gap) + u, 64, false);
            (w, w_exponent) = (sum, u_exponent + sum_shift);
        }
        if negative_f {
            // w is in [1, 2), so 2^127 / W is in (2^63, 2^64]: the integer quotient already has
            // the 64 bits, and the remainder rounds it (a tie needs W = 2^63, an exact quotient).
            let numerator = 1_u128 << 127;
            let (quotient, remainder) = (numerator / w, numerator % w);
            let rounded = quotient + u128::from(2 * remainder > w);
            let (q, q_shift) = round_to_bits(rounded, 64, false);
            (w, w_exponent) = (q, -127 - w_exponent + q_shift);
        }
    }
    // Step 4: one rounding of the 64-bit w * 2^n to a double, or to the subnormal grid.
    let exponent = w_exponent + n;
    round_extended_to_double(w, exponent, false).unwrap_or_else(|| {
        if exponent > 0 {
            f64::INFINITY
        } else {
            round_extended_to_subnormal(w, exponent, false).unwrap_or(0.0)
        }
    })
}

fn excel_exp_series(a: f64) -> f64 {
    exp_with::<false>(Register::load(a), two_to_the_minus_one_series)
}

#[test]
fn dd_rounded_twice_ties_after_the_64_bit_rounding() {
    let one_up = 1.0 + f64::EPSILON;
    // 1 + 2^-53 + 2^-80: once rounded it is 1 + 2^-52, but RN64 drops the 2^-80 and leaves
    // the 53-bit midpoint, which RN53 ties to the even 1.0.
    let tail = -power_of_two(-53) + power_of_two(-80);
    assert_eq!(one_up + tail, one_up);
    assert_eq!(dd_rounded_twice((one_up, tail)), 1.0);
    assert_eq!(dd_rounded_twice((-one_up, -tail)), -1.0);
    // Away from a midpoint both roundings agree with the head.
    assert_eq!(dd_rounded_twice((one_up, -power_of_two(-55))), one_up);
    assert_eq!(dd_rounded_twice((0.0, 0.0)), 0.0);
}

#[test]
fn exact_points() {
    assert_eq!(excel_exp(0.0), 1.0);
    assert_eq!(excel_exp(1e-30), 1.0);
    assert_eq!(excel_exp(-1e-30), 1.0);
    assert_eq!(excel_exp(-1000.0), 0.0);
    assert_eq!(excel_exp(1000.0), f64::INFINITY);
    assert!(excel_exp(f64::NAN).is_nan());
}

/// A deterministic xorshift64* stream, so the sweep is reproducible.
fn next(state: &mut u64) -> u64 {
    *state ^= *state >> 12;
    *state ^= *state << 25;
    *state ^= *state >> 27;
    state.wrapping_mul(0x2545_F491_4F6C_DD1D)
}

// The fast path returns the series' bits for every f = m * 2^-drop excel_exp can produce:
// drop in 53..=130, m up to 64 bits and at most 2^(drop - 1), plus each table node j/64 and
// its neighbours, where r is 0 or one unit.
#[test]
fn fast_path_matches_series() {
    let mut state = 0x9E37_79B9_7F4A_7C15_u64;
    let mut checked = 0;
    for drop in 53..=130_u32 {
        let limit = 1_u128
            .checked_shl(drop - 1)
            .map_or(u128::from(u64::MAX), |l| l.min(u128::from(u64::MAX)));
        let mut magnitudes: Vec<u128> = (0..4_000)
            .map(|i| {
                // Random bit lengths, so tiny f (j = 0, r tiny) is as common as large f.
                let bits = 1 + u32::try_from(next(&mut state) % 64).unwrap_or(0);
                let m = u128::from(next(&mut state)) >> (64 - bits.min(64));
                if i % 2 == 0 {
                    m.min(limit)
                } else {
                    u128::from(next(&mut state)) % limit + 1
                }
            })
            .collect();
        for j in 0..=32_u128 {
            let node = j << (drop - 6);
            magnitudes.extend(
                [node.saturating_sub(1), node, node + 1]
                    .into_iter()
                    .filter(|&m| m <= limit),
            );
        }
        for m in magnitudes.into_iter().filter(|&m| m > 0) {
            assert_eq!(
                two_to_the_minus_one(m, drop),
                two_to_the_minus_one_series(m, drop),
                "m = {m:#x}, drop = {drop}"
            );
            checked += 1;
        }
    }
    assert!(checked > 300_000);
}

// The estimate is within 2^-80 of the series sum (itself good to about 2^-100), far inside
// the 2^-74 that MIDPOINT_MARGIN allows for.
#[test]
fn estimate_error_bound() {
    let mut state = 0x6A09_E667_F3BC_C908_u64;
    for drop in 53..=130_u32 {
        let limit = 1_u128
            .checked_shl(drop - 1)
            .map_or(u128::from(u64::MAX), |l| l.min(u128::from(u64::MAX)));
        for _ in 0..2_000 {
            let bits = 1 + u32::try_from(next(&mut state) % 64).unwrap_or(0);
            let m = (u128::from(next(&mut state)) >> (64 - bits)).clamp(1, limit);
            let (hi, lo) = two_to_the_minus_one_estimate(m, drop);
            let (series_hi, series_lo) = two_to_the_minus_one_series_sum(m, drop);
            let error = ((hi - series_hi) + (lo - series_lo)).abs();
            assert!(
                error <= series_hi * 2.0_f64.powi(-80),
                "m = {m:#x}, drop = {drop}"
            );
        }
    }
}

// two_to_the_fraction's estimate of 2^f, and of 2^-f through its reciprocal, is within
// 2^-64 of the series (itself good to about 2^-100), well inside DOUBLE_MIDPOINT_MARGIN's
// 2^-60 less w's 2^-62.5.
#[test]
fn fraction_estimate_error_bound() {
    let mut state = 0xA409_3822_299F_31D0_u64;
    for drop in 53..=130_u32 {
        let limit = 1_u128
            .checked_shl(drop - 1)
            .map_or(u128::from(u64::MAX), |l| l.min(u128::from(u64::MAX)));
        for _ in 0..2_000 {
            let bits = 1 + u32::try_from(next(&mut state) % 64).unwrap_or(0);
            let m = (u128::from(next(&mut state)) >> (64 - bits)).clamp(1, limit);
            let (series_hi, series_lo) = two_to_the_minus_one_series_sum(m, drop);
            let exact = dd_add_one((series_hi, series_lo));
            let (hi, lo) = two_to_the_fraction_estimate(m, drop);
            let error = ((hi - exact.0) + (lo - exact.1)).abs();
            assert!(error <= power_of_two(-64), "m = {m:#x}, drop = {drop}");
            let inverse = dd_div((1.0, 0.0), exact);
            let (head, tail) = reciprocal_near_one(hi, lo);
            let error = ((head - inverse.0) + (tail - inverse.1)).abs();
            assert!(error <= power_of_two(-64), "1/: m = {m:#x}, drop = {drop}");
        }
    }
}

// The same through excel_exp over a wide range of a: random doubles in [-750, 750] and
// random significands at every binade from 2^-65 up.
#[test]
fn excel_exp_fast_path_matches_series() {
    let mut state = 0xD1B5_4A32_D192_ED03_u64;
    for _ in 0..200_000 {
        let unit = (next(&mut state) >> 11) as f64 / 9_007_199_254_740_992.0;
        let wide = (unit - 0.5) * 1500.0;
        let binade =
            f64::from_bits(((next(&mut state) % 75 + 958) << 52) | (next(&mut state) >> 12));
        for a in [wide, binade, -binade] {
            assert_eq!(
                excel_exp(a).to_bits(),
                excel_exp_series(a).to_bits(),
                "a = {a:e}"
            );
        }
    }
}

/// The 64-bit integer core (T-2102) returns the earlier u128 core's bits, with either step 3:
/// random doubles over the whole finite range of EXP, near its underflow and overflow ends
/// (where step 4 leaves the fast normal path), at every binade from 2^-70 up, and as
/// full-width extended registers (64-bit significands).
#[test]
fn integer_core_matches_reference() {
    let mut state = 0x243F_6A88_85A3_08D3_u64;
    let check = |register: Register| {
        assert_eq!(
            exp_with::<true>(register, two_to_the_minus_one).to_bits(),
            exp_with_reference(register, two_to_the_minus_one).to_bits(),
            "{:?}",
            register.parts()
        );
        assert_eq!(
            exp_with::<false>(register, two_to_the_minus_one).to_bits(),
            exp_with_reference(register, two_to_the_minus_one).to_bits(),
            "{:?}",
            register.parts()
        );
    };
    for _ in 0..300_000 {
        let unit = (next(&mut state) >> 11) as f64 / 9_007_199_254_740_992.0;
        let binade =
            f64::from_bits(((next(&mut state) % 85 + 953) << 52) | (next(&mut state) >> 12));
        for a in [
            (unit - 0.5) * 1500.0,
            -745.2 + unit * 37.0,
            700.0 + unit * 10.0,
            binade,
            -binade,
        ] {
            check(Register::load(a));
            check(full_width(a));
        }
    }
    // Both cores on the series step 3, on fewer points (the series is slow).
    for _ in 0..20_000 {
        let unit = (next(&mut state) >> 11) as f64 / 9_007_199_254_740_992.0;
        let a = Register::load((unit - 0.5) * 1500.0);
        assert_eq!(
            exp_with::<false>(a, two_to_the_minus_one_series).to_bits(),
            exp_with_reference(a, two_to_the_minus_one_series).to_bits()
        );
    }
}

/// [`reciprocal`] is RN64(2^127 / W) as the u128 division gives it, at random W and at the
/// ends of [2^63, 2^64).
#[test]
fn reciprocal_matches_division() {
    let divide = |w: u64| {
        let (numerator, w) = (1_u128 << 127, u128::from(w));
        let rounded = numerator / w + u128::from(2 * (numerator % w) > w);
        let (q, shift) = round_to_bits(rounded, 64, false);
        (u64::try_from(q).unwrap(), -64 + shift)
    };
    let mut state = 0x1319_8A2E_0370_7344_u64;
    let mut divisors: Vec<u64> = (0..1_000_000).map(|_| next(&mut state) | 1 << 63).collect();
    for k in 0..2_000 {
        divisors.extend([(1 << 63) + k, u64::MAX - k]);
    }
    for w in divisors {
        assert_eq!(reciprocal(w), divide(w), "w = {w:#x}");
    }
}

/// EXP of an extended argument equals w1-density's mpmath model V6 (`lib.py` `exp_v6`) on the
/// 5,200 random 64-bit arguments of `test/fixtures/exp_extended.fixture` (lines
/// `±significand exponent model`, the significand and the model's bits in hex).
#[test]
fn extended_arguments_match_the_mpmath_model() {
    let mut rows = 0;
    for line in include_str!("../test/fixtures/exp_extended.fixture").lines() {
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        let fields: Vec<&str> = line.split_whitespace().collect();
        let (negative, significand) = fields[0].split_at(1);
        let significand = u128::from_str_radix(significand, 16).unwrap();
        let exponent: i32 = fields[1].parse().unwrap();
        let model = u64::from_str_radix(fields[2], 16).unwrap();
        let a = Register::from_parts(negative == "-", significand, exponent);
        assert_eq!(
            a.parts(),
            (negative == "-", significand, exponent),
            "{line}"
        );
        assert_eq!(excel_exp_extended(a).to_bits(), model, "{line}");
        rows += 1;
    }
    assert_eq!(rows, 5200);
}

/// On a double, EXP of the double as a full-width register (its significand shifted up to 64
/// bits, as an operation leaves it, not the 53 bits of a load) is EXP of the double, bit for
/// bit: random doubles in [-750, 750], at every binade from 2^-70 up, and at the range guards.
fn full_width(a: f64) -> Register {
    let register = Register::load(a).mul(Register::load(1.0));
    assert!(a == 0.0 || register.parts().1 >> 63 == 1, "{a:e}");
    register
}

#[test]
fn extended_argument_of_a_double_is_excel_exp() {
    let mut state = 0x2633_0000_9E37_79B9_u64;
    for _ in 0..100_000 {
        let unit = (next(&mut state) >> 11) as f64 / 9_007_199_254_740_992.0;
        let wide = (unit - 0.5) * 1500.0;
        let binade =
            f64::from_bits(((next(&mut state) % 80 + 953) << 52) | (next(&mut state) >> 12));
        for a in [wide, binade, -binade] {
            let extended = excel_exp_extended(full_width(a));
            assert_eq!(extended.to_bits(), excel_exp(a).to_bits(), "a = {a:e}");
        }
    }
    // Around the T_LIMIT guard (|a| log2 e = 2000 at |a| about 1386.29, inside the 2^10
    // binade that takes the stored test) and the binade edges 1024 and 2048.
    let guard = 2000.0 / std::f64::consts::LOG2_E;
    let mut edges = vec![0.0, 1386.0, 1387.0, f64::MAX];
    for centre in [guard, 1024.0, 2048.0] {
        let bits = centre.to_bits();
        edges.extend([f64::from_bits(bits - 1), centre, f64::from_bits(bits + 1)]);
    }
    let negated: Vec<f64> = edges.iter().map(|a| -a).collect();
    for a in edges.into_iter().chain(negated) {
        assert_eq!(
            excel_exp_extended(full_width(a)).to_bits(),
            excel_exp(a).to_bits(),
            "a = {a:e}"
        );
    }
}

// Excel's own EXP cells (kmyst-erfc-r5expmap1!B1, B21, B61, B25061 and B10288, excel-graph).
#[test]
fn excel_cells() {
    assert_eq!(
        excel_exp(-693.144_335_489_521_8),
        9.359_225_999_226_452e-302
    );
    assert_eq!(excel_exp(-17.325_834_443_610_21), 2.988_723_282_343_492e-8);
    assert_eq!(
        excel_exp(693.150_025_630_557_6),
        1.074_561_465_574_963_2e301
    );
    assert_eq!(excel_exp(2.000_755_701_729_514_3), 7.394_642_131_825_609);
    // f < 0 takes the rounded reciprocal: its 64-bit value sits just above a double midpoint.
    assert_eq!(excel_exp(-0.345_024_546_371_140_6), 0.708_202_969_441_512_7);
}

/// The per-call microbenchmark of T-1945's Outcome (not a check; run it with
/// `cargo test --release -p rustsheet_engine --lib x87_exp::tests::per_call_cost -- --ignored
/// --nocapture`): 200,000 arguments in [-30, 30], the model against the series path and libm.
#[test]
#[ignore = "a timing, not a check"]
fn per_call_cost() {
    let arguments: Vec<f64> = (0..200_000)
        .map(|i| (f64::from(i) * 0.000_731_7).sin() * 30.0)
        .collect();
    let time = |exp: &dyn Fn(f64) -> f64| {
        let start = std::time::Instant::now();
        let sum: f64 = arguments
            .iter()
            .map(|&a| exp(std::hint::black_box(a)))
            .sum();
        (start.elapsed() / 200_000, sum)
    };
    let (fast, fast_sum) = time(&excel_exp);
    let (series, series_sum) = time(&excel_exp_series);
    let (reference, reference_sum) =
        time(&|a| exp_with_reference(Register::load(a), two_to_the_minus_one));
    let (libm, _) = time(&f64::exp);
    assert_eq!(fast_sum.to_bits(), series_sum.to_bits());
    assert_eq!(fast_sum.to_bits(), reference_sum.to_bits());
    println!(
        "excel_exp {fast:?}, u128 core {reference:?}, series path {series:?}, libm {libm:?} per call"
    );
}

/// `dd_exp` against known double-doubles: e (the head and the 2^-106-scale tail of 200-bit
/// mpmath's e), 2 = e^(ln 2), and e^a e^-a = 1, each to about 2^-98.
#[test]
fn dd_exp_is_double_double_accurate() {
    let e = dd_exp((1.0, 0.0));
    assert_eq!(e.0, std::f64::consts::E);
    assert!((e.1 - 1.445_646_891_729_250_2e-16).abs() < 1e-28, "{e:?}");
    let two = dd_exp((LN_2_HI, LN_2_LO));
    assert_eq!(two.0, 2.0);
    assert!(two.1.abs() < 1e-28, "{two:?}");
    for a in [-650.0, -30.25, -0.001, 0.37, 12.0, 650.0] {
        let one = dd_mul(dd_exp((a, 0.0)), dd_exp((-a, 0.0)));
        assert!((one.0 - 1.0 + one.1).abs() < 1e-28, "{a}: {one:?}");
    }
}
