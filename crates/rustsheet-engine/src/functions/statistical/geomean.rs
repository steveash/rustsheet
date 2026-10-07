use crate::expressions::types::CellReferenceIndex;
use crate::{
    calc_result::CalcResult, expressions::parser::Node, expressions::token::Error, model::Model,
};

impl Model {
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

        // Excel gives exactly 9E+307 for two 9E+307 values (numbers.md, "Aggregate overflow"), which
        // a log-sum evaluation misses by ulps. When the plain product leaves the double range, take
        // the n-th root of a product kept in range by moving powers of two into a separate
        // exponent. In range the log-sum is kept: it matches the oracle on the existing cases
        // (ironcalc-calc GEOMEAN C23) where the n-th root of the product does not.
        let n = values.len() as f64;
        let plain: f64 = values.iter().product();
        if plain.is_normal() {
            let log_sum: f64 = values.iter().map(|v| v.ln()).sum();
            return CalcResult::Number((log_sum / n).exp());
        }
        // Split every factor into mantissa in [1, 2) and a power of two, so no step can overflow
        // or underflow whatever the magnitudes mix.
        let mut product = 1.0_f64;
        let mut exp2: i64 = 0;
        for &v in &values {
            let (m, e) = split_pow2(v);
            product *= m;
            exp2 += i64::from(e);
            if product >= 2f64.powi(500) {
                product *= 2f64.powi(-500);
                exp2 += 500;
            }
        }
        // The root's power of two is `exp2 / n`: its whole part `q` scales the result exactly and
        // the remainder `r` (0 <= r < n) goes under the root, `(product * 2^r)^(1/n) * 2^q`. That
        // matches every probed overflow case bit for bit, where taking `2^(exp2 / n)` as a float
        // power is one ulp high on GEOMEAN(1E+100,9E+307) (probes/geomean-mixed-magnitude A1,
        // T-1076). `product` is below 2^501, so the remainder fits under the root up to 500;
        // past that (over 500 values) it gets its own root (rounding probed: probes/geomean-overflow-many-values
        // F6, F7 match Excel with a remainder of 599).
        let count = i64::try_from(values.len()).unwrap_or(i64::MAX);
        let (q, r) = (exp2.div_euclid(count), exp2.rem_euclid(count));
        let root = match i32::try_from(r) {
            Ok(r) if r <= 500 => (product * 2f64.powi(r)).powf(1.0 / n),
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
