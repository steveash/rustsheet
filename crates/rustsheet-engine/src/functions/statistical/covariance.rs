use crate::expressions::types::CellReferenceIndex;
use crate::{
    calc_result::CalcResult, expressions::parser::Node, expressions::token::Error, model::Model,
};

impl Model {
    pub(crate) fn fn_covariance_p(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }

        let (values1_opts, values2_opts) = match self.paired_samples(args, cell) {
            Ok(paired) if paired.same_size => (paired.first, paired.second),
            Ok(_) => {
                return CalcResult::new_error(
                    Error::NA,
                    cell,
                    "COVARIANCE requires arrays of the same size".to_string(),
                )
            }
            Err(error) => return error,
        };

        // Count numeric data points in each array (ignoring text/booleans/empty)
        let count1 = values1_opts.iter().filter(|v| v.is_some()).count();
        let count2 = values2_opts.iter().filter(|v| v.is_some()).count();

        if count1 == 0 || count2 == 0 {
            return CalcResult::new_error(
                Error::DIV,
                cell,
                "COVARIANCE.P requires at least one numeric value in each array".to_string(),
            );
        }

        if count1 != count2 {
            return CalcResult::new_error(
                Error::NA,
                cell,
                "COVARIANCE.P arrays must have the same number of numeric data points".to_string(),
            );
        }

        // Build paired numeric vectors, position by position
        let mut xs: Vec<f64> = Vec::with_capacity(count1);
        let mut ys: Vec<f64> = Vec::with_capacity(count2);

        for (v1_opt, v2_opt) in values1_opts.into_iter().zip(values2_opts) {
            if let (Some(x), Some(y)) = (v1_opt, v2_opt) {
                xs.push(x);
                ys.push(y);
            }
        }

        let n = xs.len();
        if n == 0 {
            // Should be impossible given the checks above, but guard anyway
            return CalcResult::new_error(
                Error::DIV,
                cell,
                "COVARIANCE.P has no paired numeric data points".to_string(),
            );
        }

        let n_f = n as f64;

        let mut sum_x = 0.0;
        let mut sum_y = 0.0;
        for i in 0..n {
            sum_x += xs[i];
            sum_y += ys[i];
        }
        let mean_x = sum_x / n_f;
        let mean_y = sum_y / n_f;

        let mut sum_prod = 0.0;
        for i in 0..n {
            let dx = xs[i] - mean_x;
            let dy = ys[i] - mean_y;
            sum_prod += dx * dy;
        }

        let cov = sum_prod / n_f;
        CalcResult::Number(cov)
    }

    pub(crate) fn fn_covariance_s(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }

        let (values1_opts, values2_opts) = match self.paired_samples(args, cell) {
            Ok(paired) if paired.same_size => (paired.first, paired.second),
            Ok(_) => {
                return CalcResult::new_error(
                    Error::NA,
                    cell,
                    "COVARIANCE requires arrays of the same size".to_string(),
                )
            }
            Err(error) => return error,
        };

        // Count numeric data points in each array (ignoring text/booleans/empty)
        let count1 = values1_opts.iter().filter(|v| v.is_some()).count();
        let count2 = values2_opts.iter().filter(|v| v.is_some()).count();

        if count1 == 0 || count2 == 0 {
            return CalcResult::new_error(
                Error::DIV,
                cell,
                "COVARIANCE.S requires numeric values in each array".to_string(),
            );
        }

        if count1 != count2 {
            return CalcResult::new_error(
                Error::NA,
                cell,
                "COVARIANCE.S arrays must have the same number of numeric data points".to_string(),
            );
        }

        // Build paired numeric vectors
        let mut xs: Vec<f64> = Vec::with_capacity(count1);
        let mut ys: Vec<f64> = Vec::with_capacity(count2);

        for (v1_opt, v2_opt) in values1_opts.into_iter().zip(values2_opts) {
            if let (Some(x), Some(y)) = (v1_opt, v2_opt) {
                xs.push(x);
                ys.push(y);
            }
        }

        let n = xs.len();
        if n < 2 {
            return CalcResult::new_error(
                Error::DIV,
                cell,
                "COVARIANCE.S requires at least two paired data points".to_string(),
            );
        }

        let n_f = n as f64;

        let mut sum_x = 0.0;
        let mut sum_y = 0.0;
        for i in 0..n {
            sum_x += xs[i];
            sum_y += ys[i];
        }
        let mean_x = sum_x / n_f;
        let mean_y = sum_y / n_f;

        let mut sum_prod = 0.0;
        for i in 0..n {
            let dx = xs[i] - mean_x;
            let dy = ys[i] - mean_y;
            sum_prod += dx * dy;
        }

        let cov = sum_prod / (n_f - 1.0);

        CalcResult::Number(cov)
    }
}
