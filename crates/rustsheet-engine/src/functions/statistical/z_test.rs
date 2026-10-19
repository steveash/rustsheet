use crate::arithmetic::{extended_product, extended_quotient, extended_sqrt, extended_sum};
use crate::functions::statistical::special::normal_cdf;

use crate::expressions::token::Error;
use crate::expressions::types::CellReferenceIndex;
use crate::{calc_result::CalcResult, expressions::parser::Node, model::Model};

/// Z.TEST's p-value: NORM.S.DIST(-z,TRUE), not 1 - NORM.S.DIST(z,TRUE), with
/// z = fl(fl(mean - x)*fl(sqrt(n)))/sigma, every operation rounded twice through x87 extended
/// precision (`extended_*`). Each choice is Excel's: the NORM.S.DIST(-z) form matches 120 of 120
/// cells of probes-numeric/t1916-erf-wrappers column I (19 for 1 - NORM.S.DIST(z)), and this z,
/// against (mean - x)/(sigma/sqrt(n)), matches 240 of 240 there (columns I and J, 141 for the
/// plain-double `(mean - x)*sqrt(n)/sigma`) and the 30 numeric cells of t1048-ztest and t1049-ztest
/// (24 for the quotient form). docs/excel-semantics/statistics.md, T-1778 section, "Engine (T-1929)".
fn z_test_p(mean: f64, x: f64, n: f64, sigma: f64) -> f64 {
    let z = extended_quotient(
        extended_product(extended_sum(mean, -x), extended_sqrt(n)),
        sigma,
    );
    normal_cdf(-z, 0.0, 1.0)
}

impl Model {
    // Z.TEST(array, x, [sigma])
    pub(crate) fn fn_z_test(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        // 2 or 3 arguments
        if args.len() < 2 || args.len() > 3 {
            return CalcResult::new_args_number_error(cell);
        }

        let array_arg = self.evaluate_node_in_context(&args[0], cell);

        // Flatten first argument into Vec<Option<f64>> (numeric / non-numeric)
        let values = match array_arg {
            CalcResult::Range { left, right } => match self.values_from_range(left, right) {
                Ok(v) => v,
                Err(error) => return error,
            },
            CalcResult::Array(array) => match self.values_from_array(array) {
                Ok(v) => v,
                Err(error) => {
                    return CalcResult::new_error(
                        Error::VALUE,
                        cell,
                        format!("Error in array argument: {error:?}"),
                    );
                }
            },
            CalcResult::Number(v) => vec![Some(v)],
            error @ CalcResult::Error { .. } => return error,
            _ => {
                return CalcResult::new_error(
                    Error::VALUE,
                    cell,
                    "Z.TEST first argument must be a range or array".to_string(),
                );
            }
        };

        // Collect basic stats on numeric entries
        let mut sum = 0.0;
        let mut count: u64 = 0;

        for x in values.iter().flatten() {
            sum += x;
            count += 1;
        }

        // Excel: if array has no numeric values -> #N/A
        if count == 0 {
            return CalcResult::new_error(
                Error::NA,
                cell,
                "Z.TEST array has no numeric data".to_string(),
            );
        }

        let n = count as f64;
        let mean = sum / n;

        // x argument (hypothesized population mean)
        let x_value = match self.evaluate_node_in_context(&args[1], cell) {
            CalcResult::Number(v) => v,
            error @ CalcResult::Error { .. } => return error,
            _ => {
                return CalcResult::new_error(
                    Error::VALUE,
                    cell,
                    "Z.TEST second argument (x) must be numeric".to_string(),
                );
            }
        };

        // Optional sigma
        let mut sigma: Option<f64> = None;
        if args.len() == 3 {
            match self.evaluate_node_in_context(&args[2], cell) {
                CalcResult::Number(v) => {
                    // A sigma of 0 or below is #NUM! (`Z.TEST(B1:B10,5,-1)`,
                    // probes-numeric/t1049-ztest A26, t1048-ztest A26).
                    if v <= 0.0 {
                        return CalcResult::new_error(
                            Error::NUM,
                            cell,
                            "Z.TEST sigma must be positive".to_string(),
                        );
                    }
                    sigma = Some(v);
                }
                error @ CalcResult::Error { .. } => return error,
                _ => {
                    return CalcResult::new_error(
                        Error::VALUE,
                        cell,
                        "Z.TEST sigma (third argument) must be numeric".to_string(),
                    );
                }
            }
        }

        // One numeric value is #DIV/0!, with or without sigma (`Z.TEST(D1:D1,4,1)`,
        // `Z.TEST(7,4,1)`: probes-numeric/t1049-ztest A40, A42; t1916-erf-wrappers column T).
        if count <= 1 {
            return CalcResult::new_error(
                Error::DIV,
                cell,
                "Z.TEST requires at least two values".to_string(),
            );
        }

        // If sigma omitted, use sample standard deviation STDEV(array)
        let sigma_value = if let Some(s) = sigma {
            s
        } else {
            // Compute sum of squared deviations
            let mut sumsq_dev = 0.0;
            for x in values.iter().flatten() {
                let d = x - mean;
                sumsq_dev += d * d;
            }

            let var = sumsq_dev / (n - 1.0);
            if var <= 0.0 {
                return CalcResult::new_error(
                    Error::DIV,
                    cell,
                    "Z.TEST standard deviation is zero".to_string(),
                );
            }

            var.sqrt()
        };

        // A sigma/sqrt(n) that underflows is #DIV/0! (not probed; kept from the fork).
        let denom = sigma_value / n.sqrt();
        if denom == 0.0 {
            return CalcResult::new_error(
                Error::DIV,
                cell,
                "Z.TEST denominator is zero".to_string(),
            );
        }

        CalcResult::Number(z_test_p(mean, x_value, n, sigma_value))
    }
}
