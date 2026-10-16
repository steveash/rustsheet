use crate::{
    calc_result::CalcResult,
    expressions::{
        parser::{ArrayNode, Node},
        token::Error,
        types::CellReferenceIndex,
    },
    model::Model,
};

impl Model {
    // ── SERIESSUM ─────────────────────────────────────────────────────────────

    /// `=SERIESSUM(x, n, m, coefficients)`
    ///
    /// Returns the sum of a power series:
    ///   coeff[0]*x^n + coeff[1]*x^(n+m) + coeff[2]*x^(n+2m) + ...
    pub(crate) fn fn_seriessum(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 4 {
            return CalcResult::new_args_number_error(cell);
        }

        let x = match self.get_number_no_bools(&args[0], cell) {
            Ok(v) => v,
            Err(e) => return e,
        };
        let n = match self.get_number_no_bools(&args[1], cell) {
            Ok(v) => v,
            Err(e) => return e,
        };
        let m = match self.get_number_no_bools(&args[2], cell) {
            Ok(v) => v,
            Err(e) => return e,
        };

        let result = self.evaluate_node_in_context(&args[3], cell);
        // Excel: a single value (typed text, or a one-cell reference) is read as a number, so
        // `SERIESSUM(1,0,1,"5")` and a one-cell text reference are 5; text inside an array
        // literal or a multi-cell range is `#VALUE!` (probes/seriessum-boolean-coefficients A8:A11).
        let is_single = match &result {
            CalcResult::String(_) => true,
            CalcResult::Range { left, right } => left == right,
            _ => false,
        };
        let mut coeffs = match self.result_to_array(result, cell) {
            Ok(a) => a,
            Err(e) => return e,
        };
        if is_single {
            if let Some(ArrayNode::String(text)) = coeffs.first().and_then(|row| row.first()) {
                match self.cast_to_number(CalcResult::String(text.clone()), cell) {
                    Ok(v) => coeffs = vec![vec![ArrayNode::Number(v)]],
                    Err(_) => {
                        return CalcResult::new_error(
                            Error::VALUE,
                            cell,
                            "SERIESSUM: coefficients must be numeric".to_string(),
                        )
                    }
                }
            }
        }

        let mut sum = 0.0f64;
        let mut i = 0u64;
        for row in &coeffs {
            for node in row {
                let coeff = match node {
                    ArrayNode::Number(v) => *v,
                    // Excel: a boolean coefficient is `#VALUE!` in every form (cell, typed, array
                    // literal); probes/seriessum-boolean-coefficients, math-argument-modes C44.
                    ArrayNode::Boolean(_) => {
                        return CalcResult::new_error(
                            Error::VALUE,
                            cell,
                            "SERIESSUM: coefficients must be numeric".to_string(),
                        )
                    }
                    ArrayNode::Empty => 0.0,
                    ArrayNode::String(_) => {
                        return CalcResult::new_error(
                            Error::VALUE,
                            cell,
                            "SERIESSUM: coefficients must be numeric".to_string(),
                        )
                    }
                    ArrayNode::Error(e) => {
                        return CalcResult::new_error(
                            e.clone(),
                            cell,
                            "SERIESSUM received an error value".to_string(),
                        )
                    }
                };
                let exponent = n + (i as f64) * m;
                // Excel: with x = 0 a term whose power is zero or negative is #NUM! whatever its
                // coefficient (docs/excel-semantics/numbers.md "SERIESSUM with x = 0").
                if x == 0.0 && exponent <= 0.0 {
                    return CalcResult::new_error(
                        Error::NUM,
                        cell,
                        "SERIESSUM: x=0 with a power of zero or less".to_string(),
                    );
                }
                sum += coeff * x.powf(exponent);
                i += 1;
            }
        }

        if !sum.is_finite() {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "SERIESSUM: result is not finite".to_string(),
            );
        }
        CalcResult::Number(sum)
    }
}
