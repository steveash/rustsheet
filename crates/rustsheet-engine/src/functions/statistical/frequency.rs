use crate::expressions::parser::ArrayNode;
use crate::expressions::types::CellReferenceIndex;
use crate::reference_operators::RefArg;
use crate::{calc_result::CalcResult, expressions::parser::Node, model::Model};

/// An error argument of FREQUENCY and whether it came from inside an array or range.
struct FreqError {
    error: CalcResult,
    in_array: bool,
}

impl FreqError {
    fn scalar(error: CalcResult) -> Self {
        Self {
            error,
            in_array: false,
        }
    }

    fn element(error: CalcResult) -> Self {
        Self {
            error,
            in_array: true,
        }
    }
}

impl Model {
    fn collect_freq_values(
        &mut self,
        arg: &Node,
        is_data: bool,
        cell: CellReferenceIndex,
    ) -> Result<Vec<f64>, FreqError> {
        let value = self.evaluate_node_in_context(arg, cell);
        self.freq_values_of(value, is_data, cell)
    }

    /// The numbers FREQUENCY counts in the evaluated argument `value`, `data_array` when
    /// `is_data`, else `bins_array`.
    fn freq_values_of(
        &mut self,
        value: CalcResult,
        is_data: bool,
        cell: CellReferenceIndex,
    ) -> Result<Vec<f64>, FreqError> {
        match value {
            // An error argument is the result, data before bins (probes/frequency-bins A10-A12).
            error @ CalcResult::Error { .. } => Err(FreqError::scalar(error)),
            // A typed or referenced text, boolean or blank is skipped like the text in a range
            // (probes/frequency-bins A9, A16-A25); only an error is the result.
            CalcResult::EmptyCell
            | CalcResult::EmptyArg
            | CalcResult::String(_)
            | CalcResult::Boolean(_) => Ok(vec![]),
            // An error element of the data array is the result, not the `#VALUE!` the shared reader
            // gives (gen-m9/05d44f6dd6cfc567 A126: `{-1,TRUE,0}` through CHISQ.DIST.RT is `#NUM!`),
            // and a scalar bins error beats it (`FREQUENCY({1;#DIV/0!},NA())` is `#N/A`,
            // probes/frequency-bins B1-B3).
            array @ CalcResult::Array(_) if is_data => self
                .numeric_flat_with_element_errors(array, cell)
                .map_err(FreqError::element),
            // An error element of the bins array is skipped like a text one: `ROWS(FREQUENCY(0,
            // {1,#VALUE!,3}))` is 3 (probes/array-error-element-order A1-A2, A4; desktop agrees,
            // corpus-bench/53062 AE6).
            CalcResult::Array(array) => {
                let kept = array
                    .into_iter()
                    .map(|row| {
                        row.into_iter()
                            .filter(|item| !matches!(item, ArrayNode::Error(_)))
                            .collect()
                    })
                    .collect();
                self.numeric_flat_of(CalcResult::Array(kept), cell)
                    .map_err(FreqError::element)
            }
            // Every other data error, a range's error cell included, comes before the bins' as
            // on `main` (the reverse is unprobed); the bins' own lose to the data's.
            value if is_data => self.numeric_flat_of(value, cell).map_err(FreqError::scalar),
            value => self
                .numeric_flat_of(value, cell)
                .map_err(FreqError::element),
        }
    }

    // FREQUENCY(data_array, bins_array)
    // Returns a vertical array: result[i] = count of values in the i-th bin interval.
    // The result has one more element than bins_array.
    pub(crate) fn fn_frequency(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }

        // Only data_array is probed with a union (probes/multiarea-cond); bins_array keeps its
        // single-reference path.
        let data = if self.is_multi_area(&args[0]) {
            match self.evaluate_reference_arg(&args[0], cell) {
                RefArg::Ranges(areas) => self
                    .values_from_areas(&areas, cell)
                    .map(|v| v.into_iter().flatten().collect())
                    .map_err(FreqError::scalar),
                // A LET variable, IF or name that holds an array or a number is counted like
                // any other data_array; only an error is the result (T-1065 fidelity review).
                RefArg::Value(error @ CalcResult::Error { .. }) => Err(FreqError::scalar(error)),
                RefArg::Value(value) => self.freq_values_of(value, true, cell),
                RefArg::Grid(grid) => self.freq_values_of(grid.to_values(), true, cell),
            }
        } else {
            self.collect_freq_values(&args[0], true, cell)
        };
        let bins_raw = self.collect_freq_values(&args[1], false, cell);
        // Excel: a scalar error argument is the result, data before bins, and beats an error
        // element of the data array (`FREQUENCY({1;#DIV/0!},NA())` is `#N/A`,
        // `FREQUENCY(NA(),{1;#DIV/0!})` is `#N/A`; gen-m9/589f42e67f4b4b8d A338;
        // probes/frequency-bins B1-B3, T-2427).
        let (data, bins_raw) = match (data, bins_raw) {
            (Ok(data), Ok(bins)) => (data, bins),
            (Err(e), _) if !e.in_array => return e.error,
            (_, Err(e)) if !e.in_array => return e.error,
            // Otherwise the data's error first, else the bins'.
            (Err(e), _) | (_, Err(e)) => return e.error,
        };

        // Excel: with no numeric bin the result is still two rows, 0 then every data value
        // (probes/frequency-bins A16-A25; docs/excel-semantics/statistics.md, T-2427).
        if bins_raw.is_empty() {
            return CalcResult::Array(vec![
                vec![ArrayNode::Number(0.0)],
                vec![ArrayNode::Number(data.len() as f64)],
            ]);
        }

        let n_bins = bins_raw.len();

        // Sort bins by value but remember their original positions so that output
        // is placed back in the original (unsorted) bin order, matching Excel.
        let mut indexed_bins: Vec<(f64, usize)> = bins_raw
            .into_iter()
            .enumerate()
            .map(|(i, v)| (v, i))
            .collect();
        indexed_bins.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

        // sorted_bins[i] = (value, original_index)
        let sorted_vals: Vec<f64> = indexed_bins.iter().map(|&(v, _)| v).collect();

        // counts[i] is the count for sorted bin i; counts[n_bins] is the overflow bucket.
        let mut sorted_counts = vec![0u64; n_bins + 1];
        for &val in &data {
            let idx = sorted_vals.partition_point(|&b| b < val);
            sorted_counts[idx] += 1;
        }

        // Map sorted counts back to original bin positions.
        let mut result_counts = vec![0u64; n_bins + 1];
        for (sorted_idx, &(_, orig_idx)) in indexed_bins.iter().enumerate() {
            result_counts[orig_idx] = sorted_counts[sorted_idx];
        }
        // The overflow bucket is always last.
        result_counts[n_bins] = sorted_counts[n_bins];

        let result: Vec<Vec<ArrayNode>> = result_counts
            .iter()
            .map(|&c| vec![ArrayNode::Number(c as f64)])
            .collect();

        CalcResult::Array(result)
    }
}
