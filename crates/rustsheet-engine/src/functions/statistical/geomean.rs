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
        let mut product = 1.0_f64;
        let mut exp2: i32 = 0;
        for &v in &values {
            product *= v;
            if !(f64::MIN_POSITIVE.sqrt()..=f64::MAX.sqrt()).contains(&product) {
                let e = ((product.to_bits() >> 52) & 0x7ff) as i32 - 1023;
                product *= 2f64.powi(-e);
                exp2 += e;
            }
        }
        CalcResult::Number(product.powf(1.0 / n) * 2f64.powf(f64::from(exp2) / n))
    }
}
