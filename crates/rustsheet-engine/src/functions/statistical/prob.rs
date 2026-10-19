use crate::expressions::types::CellReferenceIndex;
use crate::functions::lift::LiftCost;
use crate::functions::Function;
use crate::{
    calc_result::CalcResult, expressions::parser::Node, expressions::token::Error, model::Model,
};

impl Model {
    /// PROB's two data arguments read once: the x values and their probabilities, paired, or
    /// the error the data gives (unequal sizes `#N/A`, no values `#DIV/0!`, a probability out of
    /// [0, 1] or a sum other than 1 `#NUM!`).
    fn prob_data(
        &mut self,
        x_range: &Node,
        prob_range: &Node,
        cell: CellReferenceIndex,
    ) -> Result<(Vec<f64>, Vec<f64>), CalcResult> {
        let error = |error| CalcResult::new_error(error, cell, String::new());
        let x_vals = self.collect_numeric_flat(x_range, cell)?;
        let prob_vals = self.collect_numeric_flat(prob_range, cell)?;
        if x_vals.len() != prob_vals.len() {
            return Err(error(Error::NA));
        }
        if x_vals.is_empty() {
            return Err(error(Error::DIV));
        }
        if prob_vals.iter().any(|p| !(0.0..=1.0).contains(p)) {
            return Err(error(Error::NUM));
        }
        // Allow a small tolerance for floating-point imprecision.
        let prob_sum: f64 = prob_vals.iter().sum();
        if (prob_sum - 1.0).abs() > 1e-14 {
            return Err(error(Error::NUM));
        }
        Ok((x_vals, prob_vals))
    }

    /// `PROB(x_range, prob_range, lower_limit, [upper_limit])`: the probability that a value
    /// of `x_range` lies in [`lower_limit`, `upper_limit`]. The limits lift over an array and a
    /// multi-cell range, the data is read once (SPEC-0011, T-1813). An element checks its limits,
    /// then the data: `PROB(D1:D3,F1:F3,"a")` with D2 = `#N/A` is `#VALUE!`, and a lifted element
    /// with good limits is the data's `#N/A` (probes/stat-test-lift P14, P87). Each element walks
    /// the data, so the call is charged its cells per element.
    pub(crate) fn fn_prob(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let kind = Function::Prob;
        if !kind
            .arg_kinds()
            .is_some_and(|kinds| kinds.accepts(args.len()))
        {
            return CalcResult::new_args_number_error(cell);
        }
        let [x_range, prob_range, ..] = args else {
            return CalcResult::new_args_number_error(cell);
        };
        let data = self
            .prob_data(x_range, prob_range, cell)
            .map_err(CalcResult::without_message);
        let cells = data.as_ref().map_or(0, |(x_vals, _)| {
            u64::try_from(x_vals.len()).unwrap_or(u64::MAX)
        });
        self.lift_call_costed(
            kind,
            args,
            // At least one unit an element, so a data error copied per element is charged too.
            LiftCost::Cells(cells.max(1)),
            cell,
            &mut |model, values| prob_element(model, &data, values, cell),
        )
    }

    pub(crate) fn collect_numeric_flat(
        &mut self,
        arg: &Node,
        cell: CellReferenceIndex,
    ) -> Result<Vec<f64>, CalcResult> {
        let value = self.evaluate_node_in_context(arg, cell);
        self.numeric_flat_of(value, cell)
    }

    /// The numbers of `value` as [`Self::collect_numeric_flat`] reads an argument: a range's or
    /// an array's numbers, or a number; anything else, an error value too, is `#VALUE!`.
    pub(crate) fn numeric_flat_of(
        &mut self,
        value: CalcResult,
        cell: CellReferenceIndex,
    ) -> Result<Vec<f64>, CalcResult> {
        match value {
            CalcResult::Range { left, right } => match self.values_from_range(left, right) {
                Ok(v) => Ok(v.into_iter().flatten().collect()),
                Err(e) => Err(e),
            },
            CalcResult::Array(arr) => match self.values_from_array(arr) {
                Ok(v) => Ok(v.into_iter().flatten().collect()),
                Err(e) => Err(CalcResult::new_error(Error::VALUE, cell, format!("{e:?}"))),
            },
            CalcResult::Number(n) => Ok(vec![n]),
            _ => Err(CalcResult::new_error(
                Error::VALUE,
                cell,
                "Expected numeric range".to_string(),
            )),
        }
    }
}

/// One PROB element over the data read once.
fn prob_element(
    model: &Model,
    data: &Result<(Vec<f64>, Vec<f64>), CalcResult>,
    values: &[CalcResult],
    cell: CellReferenceIndex,
) -> CalcResult {
    let lower = match values.get(2).map(|lower| model.kernel_number(lower, cell)) {
        Some(Ok(lower)) => lower,
        Some(Err(error)) => return error,
        None => return CalcResult::new_args_number_error(cell),
    };
    let upper = match values.get(3).map(|upper| model.kernel_number(upper, cell)) {
        Some(Ok(upper)) => upper,
        Some(Err(error)) => return error,
        None => lower,
    };
    let (x_vals, prob_vals) = match data {
        Ok(data) => data,
        Err(error) => return error.clone(),
    };
    if lower > upper {
        return CalcResult::Number(0.0);
    }
    CalcResult::Number(
        x_vals
            .iter()
            .zip(prob_vals)
            .filter(|(&x, _)| x >= lower && x <= upper)
            .map(|(_, &p)| p)
            .sum(),
    )
}
