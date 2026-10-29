use super::element_errors::array_element_error;
use super::paired::PairedArg;
use crate::expressions::types::CellReferenceIndex;
use crate::functions::lift::LiftCost;
use crate::functions::Function;
use crate::{
    calc_result::CalcResult, expressions::parser::Node, expressions::token::Error, model::Model,
};

impl Model {
    /// PROB's two data arguments read once: the numbers of `x_range` and `prob_range` paired
    /// by position, a pair kept only when both are numbers, or the error the data gives (an
    /// error cell, unequal cell counts `#N/A`, no pair of numbers `#DIV/0!`, a probability out of
    /// [0, 1] or a sum other than 1 `#NUM!`). Counts are cells, not numbers: `PROB(Data!C7:D7,1E-15,2)`
    /// with one blank is `#N/A`, a single text or boolean against a number is no pair
    /// (`PROB(Data!C$4,13,-2.5)` with C4 TRUE is `#DIV/0!`), and a probability whose x is text
    /// is dropped (statistics.md "PROB: an error data argument first", T-2468).
    fn prob_data(
        &mut self,
        x_value: CalcResult,
        prob_value: CalcResult,
        cell: CellReferenceIndex,
    ) -> Result<(Vec<f64>, Vec<f64>), CalcResult> {
        let error = |error| CalcResult::new_error(error, cell, String::new());
        let as_arg = |value| match PairedArg::from_result(value) {
            Ok(arg) => Ok(arg),
            Err(CalcResult::String(_) | CalcResult::Boolean(_)) => {
                Ok(PairedArg::Values(vec![None]))
            }
            Err(_) => Err(error(Error::VALUE)),
        };
        let (x_arg, prob_arg) = (as_arg(x_value)?, as_arg(prob_value)?);
        // Unequal cell counts are `#N/A` before an error element of either side:
        // `PROB(0,IF({1,1,1,1},1/0),0)` and `PROB(IF({1,1},1/0),{0.5,0.5,0},1)` are `#N/A`
        // (probes/t2707-prob-orders A54, A56), as in the paired functions (`PairedSamplesError`).
        if let (Some(x_cells), Some(prob_cells)) = (x_arg.cell_count(), prob_arg.cell_count()) {
            if x_cells != prob_cells {
                return Err(error(Error::NA));
            }
        }
        // An array's error element is the result (probes/array-error-element-order A12-A14, A18),
        // the other argument's own error winning; the order against a limit's error is inferred
        // from a range error cell (probes/stat-test-lift P14), no probe cell separates them.
        let data =
            self.paired_values(x_arg, prob_arg, cell, |_, e| array_element_error(e, cell))?;
        if !data.same_size {
            return Err(error(Error::NA));
        }
        let (x_vals, prob_vals): (Vec<f64>, Vec<f64>) = data
            .first
            .iter()
            .zip(&data.second)
            .filter_map(|(x, p)| Some(((*x)?, (*p)?)))
            .unzip();
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
    /// multi-cell range, the data is read once (SPEC-0011, T-1813). A data argument that is an
    /// error value (or a blank one-cell reference) is each element's result first; otherwise an element checks its limits,
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
        let x_value = self.evaluate_node_in_context(x_range, cell);
        let prob_value = self.evaluate_node_in_context(prob_range, cell);
        // Before the limits, each data argument in turn, x_range first, is the result when it is
        // an error value, or `#VALUE!` when it is a one-cell reference to an empty cell: a blank
        // x_range beats prob_range's error (`PROB(<blank>,NA(),1)` is `#VALUE!`,
        // probes/t2707-prob-orders A9) and x_range's error a blank prob_range (A11)
        // (statistics.md "PROB: an error data argument first", T-2468, T-2707).
        // It is each lifted element's result, not the call's: with an array limit the errors
        // spill (gen-m9/329b177b5d5165ad A144).
        let direct = [&x_value, &prob_value].into_iter().find_map(|value| {
            if matches!(value, CalcResult::Error { .. }) {
                Some(value.clone())
            } else {
                self.is_blank_one_cell(value)
                    .then(|| CalcResult::new_error(Error::VALUE, cell, String::new()))
            }
        });
        let data = self
            .prob_data(x_value, prob_value, cell)
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
            &mut |model, values| match &direct {
                Some(error) => error.clone(),
                None => prob_element(model, &data, values, cell),
            },
        )
    }

    /// The numbers of `value` as FREQUENCY reads a data argument: a range's or
    /// an array's numbers, or a number; anything else, an error value too, is `#VALUE!`.
    pub(crate) fn numeric_flat_of(
        &mut self,
        value: CalcResult,
        cell: CellReferenceIndex,
    ) -> Result<Vec<f64>, CalcResult> {
        self.numeric_flat(value, cell, false)
    }

    /// [`Self::numeric_flat_of`] where an error element of an array is the result, marked as
    /// [`super::element_errors::is_array_element_error`] reads it (statistics.md "An error element of an array argument
    /// is that error").
    pub(crate) fn numeric_flat_with_element_errors(
        &mut self,
        value: CalcResult,
        cell: CellReferenceIndex,
    ) -> Result<Vec<f64>, CalcResult> {
        self.numeric_flat(value, cell, true)
    }

    fn numeric_flat(
        &mut self,
        value: CalcResult,
        cell: CellReferenceIndex,
        element_errors: bool,
    ) -> Result<Vec<f64>, CalcResult> {
        match value {
            CalcResult::Range { left, right } => match self.values_from_range(left, right) {
                Ok(v) => Ok(v.into_iter().flatten().collect()),
                Err(e) => Err(e),
            },
            CalcResult::Array(arr) => match self.values_from_array(arr) {
                Ok(v) => Ok(v.into_iter().flatten().collect()),
                Err(error) if element_errors => Err(array_element_error(error, cell)),
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
