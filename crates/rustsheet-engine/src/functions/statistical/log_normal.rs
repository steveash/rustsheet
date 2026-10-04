use crate::expressions::types::CellReferenceIndex;
use crate::functions::statistical::special::{normal_cdf, normal_inv_std, SQRT_2PI};
use crate::{
    calc_result::CalcResult, expressions::parser::Node, expressions::token::Error, model::Model,
};

impl Model {
    pub(crate) fn fn_log_norm_dist(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        if args.len() != 4 {
            return CalcResult::new_args_number_error(cell);
        }

        let x = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        let mean = match self.get_number(&args[1], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        let std_dev = match self.get_number(&args[2], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        let cumulative = match self.get_boolean(&args[3], cell) {
            Ok(b) => b,
            Err(e) => return e,
        };

        // Excel domain checks
        if x <= 0.0 || std_dev <= 0.0 {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Invalid parameter for LOGNORM.DIST".to_string(),
            };
        }

        let result = if cumulative {
            normal_cdf(x.ln(), mean, std_dev)
        } else {
            // Same kernel as `normal_pdf` with the extra `x` factor; this order reproduces
            // the cached last digits in LOGNORM.xlsx.
            let d = (x.ln() - mean) / std_dev;
            (-0.5 * d * d).exp() / (x * SQRT_2PI * std_dev)
        };

        if !result.is_finite() {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Invalid parameter for LOGNORM.DIST".to_string(),
            };
        }

        CalcResult::Number(result)
    }

    pub(crate) fn fn_log_norm_inv(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        if args.len() != 3 {
            return CalcResult::new_args_number_error(cell);
        }

        let p = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        let mean = match self.get_number(&args[1], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        let std_dev = match self.get_number(&args[2], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };

        // Excel domain checks
        if p <= 0.0 || p >= 1.0 || std_dev <= 0.0 {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Invalid parameter for LOGNORM.INV".to_string(),
            };
        }

        let result = (mean + std_dev * normal_inv_std(p)).exp();

        if !result.is_finite() {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Invalid parameter for LOGNORM.INV".to_string(),
            };
        }

        CalcResult::Number(result)
    }

    // LOGNORMDIST(x, mean, standard_dev) — always cumulative=TRUE, 3 args
    pub(crate) fn fn_lognormdist_compat(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        if args.len() != 3 {
            return CalcResult::new_args_number_error(cell);
        }
        let mut new_args = args.to_vec();
        new_args.push(Node::BooleanKind(true));
        self.fn_log_norm_dist(&new_args, cell)
    }
}
