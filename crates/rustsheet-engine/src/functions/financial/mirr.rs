use crate::{
    arithmetic::power_with_libm_exp,
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
        let mut positive_values = Vec::new();
        let mut negative_values = Vec::new();
        let mut last_negative_index = -1;
        for (index, &value) in values.iter().enumerate() {
            let (p, n) = if value >= 0.0 {
                (value, 0.0)
            } else {
                last_negative_index = index as i32;
                (0.0, value)
            };
            positive_values.push(p);
            negative_values.push(n);
        }
        if last_negative_index == -1 {
            return CalcResult::new_error(
                Error::DIV,
                cell,
                "Invalid data for MIRR function".to_string(),
            );
        }
        // Excel discounts each flow to period 0 with a running power, `x / (1+r)^i` with
        // `(1+r)^i = (1+r)^(i-1) * (1+r)`, and compounds the positive sum forward as
        // `P * (1+rr)^n / (1+rr)`; the root is `^` with libm's `exp` (Mac desktop Excel's cached
        // MIRR.xlsx Sheet2!N16; see [`power_with_libm_exp`]). Every order and power source matters
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
                positive_sum += value / reinvest_power;
            } else if value < 0.0 {
                negative_sum += value / finance_power;
            }
            reinvest_power *= reinvest_growth;
            finance_power *= finance_growth;
        }
        // A rate of -1 makes the powers 0, so its limits are taken by hand (the pre-T-1687
        // behaviour, unfitted; `fn_mirr_div_0` covers a finance rate of -1).
        let top = if reinvest_rate == -1.0 {
            // This is finite
            match positive_values.last() {
                Some(f) => *f,
                None => 0.0,
            }
        } else {
            -(positive_sum * reinvest_power / reinvest_growth)
        };
        let bottom = if finance_rate == -1.0 {
            if last_negative_index == 0 {
                // This is still finite
                negative_values[last_negative_index as usize]
            } else {
                // or -Infinity depending of the sign in the last_negative_index coef.
                // But it is irrelevant for the calculation
                f64::INFINITY
            }
        } else {
            negative_sum
        };

        let ratio = top / bottom;
        let exponent = 1.0 / (years - 1.0);
        // Only the ordinary case is fitted; a zero, negative or infinite ratio and a single flow
        // keep `powf` and the errors below.
        let root = match power_with_libm_exp(ratio, exponent) {
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
