use crate::expressions::types::CellReferenceIndex;
use crate::number_format::to_excel_precision;
use crate::{
    calc_result::CalcResult, expressions::parser::Node, expressions::token::Error, model::Model,
};

impl Model {
    /// GEOMEAN. In range, the root `exp(ln(P)/n)` snaps to its 15-digit value when that value,
    /// multiplied n times, gives P back (numbers.md "GEOMEAN in-range root").
    /// FITTED(T-1123, 170 cells, GEOMEAN of 1..1100 values with a normal product)
    pub(crate) fn fn_geomean(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }

        let mut values: Vec<f64> = Vec::new();
        if let Err(e) = self.for_each_value(args, cell, |f| values.push(f)) {
            return e;
        }

        if values.is_empty() {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "GEOMEAN requires at least one numeric value".to_string(),
            );
        }

        if values.iter().any(|&v| v <= 0.0) {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "GEOMEAN requires all values to be positive".to_string(),
            );
        }

        // Excel multiplies the values' binary significands in [1, 2) into one double, keeping their
        // powers of two apart, and is #NUM! when that product overflows: n * log2(significand)
        // reaching 1024. 1028 copies of 5E+204 are finite and 1029 are #NUM!, and so are 1752
        // alternating 1.5 and 0.75 although their plain product is in range (numbers.md, "GEOMEAN
        // is #NUM! when the product of significands overflows"; probes/geomean-overflow-cut*).
        let significands: f64 = values.iter().map(|&v| split_pow2(v).0).product();
        if significands.is_infinite() {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "GEOMEAN product of significands overflows".to_string(),
            );
        }

        // Excel gives exactly 9E+307 for two 9E+307 values (numbers.md, "Aggregate overflow"), which
        // a log-sum evaluation misses by ulps. When the plain product leaves the double range, take
        // the n-th root of a product kept in range by moving powers of two into a separate
        // exponent. In range the root is `exp(ln(product) / n)`, and one value is returned as is:
        // that matches 67 of 72 many-value sets (probes/geomean-in-range-many) and all of
        // ironcalc-calc GEOMEAN, where a log-sum `exp(mean(ln v))` misses `GEOMEAN("47")` and 1101
        // x 1.5 (probes/geomean-overflow-count H1) and `powf` rounds sqrt(3) down (C23). An
        // exact root is returned exactly: GEOMEAN(3,3), GEOMEAN(1,9) and 50 copies of 7 are 3, 3 and
        // 7, where `exp(ln/n)` is one ulp off, so a root whose 15-digit value, raised to the n-th
        // power by the same left-to-right product, gives the product back is that value
        // (numbers.md, "GEOMEAN in-range root"; probes/geomean-single-and-copies,
        // probes/geomean-root-refinement; T-1123).
        let n = values.len() as f64;
        let plain: f64 = values.iter().product();
        if plain.is_normal() {
            if values.len() == 1 {
                return CalcResult::Number(plain);
            }
            let root = (plain.ln() / n).exp();
            let short = to_excel_precision(root, 15);
            // `short^n` as n left-to-right multiplications, the way `plain` was formed.
            let exact = short != root && values.iter().fold(1.0, |p, _| p * short) == plain;
            return CalcResult::Number(if exact { short } else { root });
        }
        // Keep the product as a significand in [1, 2) and a separate power of two: every factor and
        // every partial product is split, so no step can overflow or underflow whatever the
        // magnitudes mix, and the significand's bits are those of the plain left-to-right product.
        let mut product = 1.0_f64;
        let mut exp2: i64 = 0;
        for &v in &values {
            let (m, e) = split_pow2(v);
            let (p, pe) = split_pow2(product * m);
            product = p;
            exp2 += i64::from(e) + i64::from(pe);
        }
        // The root's power of two is `exp2 / n`: its whole part `q` scales the result exactly and
        // the remainder `r` (0 <= r < n) goes under the root, `(product * 2^r)^(1/n) * 2^q`. That
        // matches 21 of the 29 probed cells bit for bit (probes/geomean-mixed-magnitude,
        // probes/geomean-overflow-many-values, probes/geomean-product-order); the 8 misses are
        // one-ulp, near-halfway roundings of the root that no tried formula reproduces
        // (numbers.md, "GEOMEAN root rounding", T-1099). Taking `2^(exp2 / n)` as a float power is
        // one ulp high on GEOMEAN(1E+100,9E+307) (probes/geomean-mixed-magnitude A1, T-1076).
        // `product * 2^r` stays finite up to r = 1000; past that (over 1000 values) the remainder
        // gets its own root (unprobed).
        let count = i64::try_from(values.len()).unwrap_or(i64::MAX);
        exp2 += exponent_wrap_excess(significands, count, exp2);
        let (q, r) = (exp2.div_euclid(count), exp2.rem_euclid(count));
        let root = match i32::try_from(r) {
            Ok(r) if r <= 1000 => (product * 2f64.powi(r)).powf(1.0 / n),
            _ => product.powf(1.0 / n) * 2f64.powf(r as f64 / n),
        };
        // `2^q` in two halves: `powi` of an exponent below -1023 is 0, not the subnormal. Inputs
        // are normal (subnormal results become 0, numbers.md), so `q >= -1022` today; the split
        // keeps the scaling exact should a subnormal input ever reach here.
        let q = i32::try_from(q).unwrap_or(i32::MAX);
        let result = root * 2f64.powi(q / 2) * 2f64.powi(q - q / 2);
        // Defensive only: a geometric mean of finite positives never exceeds the largest value.
        if result.is_finite() {
            CalcResult::Number(result)
        } else {
            CalcResult::new_error(Error::NUM, cell, "GEOMEAN overflow".to_string())
        }
    }
}

/// Span of the 11-bit double exponent field: Excel's power-of-two sum gains one of these per wrap.
const EXPONENT_FIELD_SPAN: i64 = 2048;

/// The amount Excel's power-of-two sum `exp2` is too high for `count` values whose [1, 2)
/// significand product is `significands` (finite, at least 1).
///
/// It is `2048 * k`, where k counts the wraps of the exponent field below the double range by the
/// [0.5, 1) significand product `significands * 2^-count`, as if Excel formed that product by
/// exponent-field arithmetic modulo 2048. It matches all 41 probed cells of T-1120: 2400 copies of
/// 1E-300 give 1E-300 * 2^(2048/2400), 1700 copies give 1E-300, and 1E-307 steps to k = 2 between
/// 3500 and 3900 copies. Only a product below the double range (negative `exp2`) whose significand
/// product has reached 2 wraps: 2^-1000 * 1.001 (significand product exponent 1) gains k = 1 from
/// 1100 copies, while 2^-1000 * 1.0001 (product below 2), powers of two (product 1) and 2^1000 times
/// 1.001 or 1.1 (positive sum) never gain, up to 3000 or 8200 copies (numbers.md, "GEOMEAN is #NUM!
/// when the product of significands overflows"; probes/geomean-exponent-sum, -sum-2,
/// geomean-overflow-cut-3 C7, geomean-pow2-count; T-1120, T-1156).
fn exponent_wrap_excess(significands: f64, count: i64, exp2: i64) -> i64 {
    // `significands >= 1` puts its exponent in [0, 1023], and `count` is a vector length, so this
    // cannot overflow.
    let below = count - i64::from(split_pow2(significands).1) - 1022;
    if below > 0 && exp2 < 0 && significands >= 2.0 {
        EXPONENT_FIELD_SPAN * ((below + EXPONENT_FIELD_SPAN - 1) / EXPONENT_FIELD_SPAN)
    } else {
        0
    }
}

/// Splits a positive finite `v` into `(m, e)` with `v = m * 2^e` and `m` in `[1, 2)`.
fn split_pow2(v: f64) -> (f64, i32) {
    // Scale subnormals into the normal range first so the exponent field is meaningful.
    let (v, bias) = if v < f64::MIN_POSITIVE {
        (v * 2f64.powi(64), -64)
    } else {
        (v, 0)
    };
    let e = ((v.to_bits() >> 52) & 0x7ff) as i32 - 1023;
    (
        f64::from_bits((v.to_bits() & !(0x7ffu64 << 52)) | (1023u64 << 52)),
        e + bias,
    )
}

#[cfg(test)]
mod tests {
    use super::split_pow2;

    #[test]
    fn split_roundtrips() {
        for v in [5e-324, 1e-310, 1e-154, 1.0, 1.5, 9e307, f64::MAX] {
            let (m, e) = split_pow2(v);
            assert!((1.0..2.0).contains(&m));
            assert_eq!(
                m * 2f64.powi(e.clamp(-1000, 1000)) * 2f64.powi(e - e.clamp(-1000, 1000)),
                v
            );
        }
    }
}
