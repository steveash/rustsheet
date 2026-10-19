use crate::{
    arithmetic::power,
    calc_result::CalcResult,
    expressions::{parser::Node, token::Error, types::CellReferenceIndex},
    model::Model,
};

impl Model {
    /// `MIRR(values, finance_rate, reinvest_rate)`: `(FV / -PV)^(1/(n-1)) - 1`, with FV the positive
    /// values compounded at the reinvest rate to the last period and PV the negative values
    /// discounted at the finance rate to the first, in Excel's evaluation order (below).
    pub(crate) fn fn_mirr(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 3 {
            return CalcResult::new_args_number_error(cell);
        }
        let values = match self.get_array_of_numbers(&args[0], &cell) {
            Ok(s) => s,
            Err(error) => return error,
        };
        let finance_rate = match self.get_number(&args[1], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let reinvest_rate = match self.get_number(&args[2], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        if values.len() < 2 || !values.iter().any(|&value| value < 0.0) {
            return CalcResult::new_error(
                Error::DIV,
                cell,
                "Invalid data for MIRR function".to_string(),
            );
        }
        // Excel: a flow divided by a zero power is #DIV/0!, so a reinvest rate of -1 with any
        // positive flow, or a finance rate of -1 with a negative flow after the first, is an
        // error (probes/fin-mirr-edges A7, A12-A14, A19, A20). A single flow is #DIV/0! too (A3).
        let div_error = || CalcResult::new_error(Error::DIV, cell, "Division by 0".to_string());
        // Excel discounts each flow to period 0 with a running power, `x / (1+r)^i` with
        // `(1+r)^i = (1+r)^(i-1) * (1+r)`, and compounds the positive sum forward as
        // `P * (1+rr)^n / (1+rr)`; the root is Excel's `^`. Every order and power source matters
        // in the last digits: this one matches all 432 cells of probes-numeric/m9-fin-mirr
        // (docs/excel-semantics/financial.md "MIRR's evaluation order", T-1687).
        let years = values.len() as f64;
        let reinvest_growth = 1.0 + reinvest_rate;
        let finance_growth = 1.0 + finance_rate;
        let mut positive_sum = 0.0;
        let mut negative_sum = 0.0;
        let mut reinvest_power = 1.0;
        let mut finance_power = 1.0;
        for &value in &values {
            if value > 0.0 {
                if reinvest_power == 0.0 || reinvest_growth == 0.0 {
                    return div_error();
                }
                positive_sum += value / reinvest_power;
            } else if value < 0.0 {
                if finance_power == 0.0 {
                    return div_error();
                }
                negative_sum += value / finance_power;
            }
            reinvest_power *= reinvest_growth;
            finance_power *= finance_growth;
        }
        // A power that overflows: the reinvest one is #DIV/0! (A16, A17, A21), the finance one
        // #NUM! (A15, A24); the reinvest check comes first (A21 is #DIV/0!).
        if reinvest_power.is_infinite() {
            return div_error();
        }
        if finance_power.is_infinite() {
            return CalcResult::new_error(Error::NUM, cell, "Invalid data for MIRR".to_string());
        }
        let top = -(positive_sum * reinvest_power / reinvest_growth);
        let bottom = negative_sum;

        let ratio = top / bottom;
        let exponent = 1.0 / (years - 1.0);
        // Only the ordinary case is fitted; a zero, negative or infinite ratio and a single flow
        // keep `powf` and the errors below.
        let root = match power(ratio, exponent) {
            Ok(root) if ratio.is_finite() && ratio > 0.0 && exponent.is_finite() => root,
            _ => ratio.powf(exponent),
        };
        let result = root - 1.0;
        if result.is_infinite() {
            return CalcResult::new_error(Error::DIV, cell, "Division by 0".to_string());
        }
        if result.is_nan() {
            return CalcResult::new_error(Error::NUM, cell, "Invalid data for MIRR".to_string());
        }
        CalcResult::Number(result)
    }
}
