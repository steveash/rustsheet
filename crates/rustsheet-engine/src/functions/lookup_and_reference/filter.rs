use crate::{
    calc_result::CalcResult,
    expressions::{
        parser::{ArrayNode, Node},
        token::Error,
        types::CellReferenceIndex,
    },
    model::Model,
};

/// Whether an include element keeps its row or column, or the error it makes FILTER return. A
/// number keeps it unless 0, a blank does not, text is a boolean only when it spells TRUE or FALSE
/// in any case (`"true"` keeps it, `" TRUE"` and `"1"` are `#VALUE!`), an error is itself
/// (docs/excel-semantics/arrays.md "FILTER: the type of each include element and of the result";
/// probes/t3654-filter-result-types P20:P42).
fn include_keeps(node: &ArrayNode, cell: CellReferenceIndex) -> Result<bool, CalcResult> {
    match node {
        ArrayNode::Boolean(b) => Ok(*b),
        ArrayNode::Number(n) => Ok(*n != 0.0),
        ArrayNode::Empty => Ok(false),
        ArrayNode::String(text) => Model::cast_bool_ref(&CalcResult::String(text.clone()), cell)
            .map_err(|_| {
                CalcResult::new_error(
                    Error::VALUE,
                    cell,
                    "include array must contain numbers or booleans".to_string(),
                )
            }),
        ArrayNode::Lambda(_) | ArrayNode::Nested(_) => Err(CalcResult::new_error(
            Error::VALUE,
            cell,
            "include array must contain numbers or booleans".to_string(),
        )),
        ArrayNode::Error(e) => Err(CalcResult::new_error(
            e.clone(),
            cell,
            "include array contains an error".to_string(),
        )),
    }
}

impl Model {
    /// `=FILTER(array, include, [if_empty])`
    ///
    /// Returns only the rows of array where the corresponding include value is truthy.
    ///   * include  – a column vector of booleans/numbers with the same height as array
    ///   * `if_empty` – value returned when no rows pass the filter (default: #CALC! error)
    pub(crate) fn fn_filter(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() < 2 || args.len() > 3 {
            return CalcResult::new_args_number_error(cell);
        }

        // A lone blank cell stays a blank element, as in a range: `ISBLANK(FILTER(A2,TRUE))` is
        // TRUE and `FILTER(A2,TRUE)&"|"` is "|" (probes/t3654-filter-result-types Q63:Q66).
        let data = match self.eval_array_arg_keep_blank(&args[0], cell) {
            Ok(d) => d,
            Err(e) => return e,
        };

        if data.is_empty() || data[0].is_empty() {
            return CalcResult::Array(vec![]);
        }

        let num_rows = data.len();
        let num_cols = data[0].len();

        let include = match self.eval_to_array(&args[1], cell) {
            Ok(d) => d,
            Err(e) => return e,
        };

        // Every element is checked, the excluded ones too: an error anywhere is returned
        // (`FILTER(C1:C3,{0;#N/A;0})` is #N/A; probes/t3654-filter-result-types P35).
        let mut keep: Vec<Vec<bool>> = Vec::with_capacity(include.len());
        for row in &include {
            let mut kept = Vec::with_capacity(row.len());
            for node in row {
                match include_keeps(node, cell) {
                    Ok(k) => kept.push(k),
                    Err(error) => return error,
                }
            }
            keep.push(kept);
        }

        // Determine whether we are filtering rows or columns
        let include_flat: Vec<bool>;
        let filter_rows: bool;

        if keep.len() == num_rows && keep[0].len() == 1 {
            // Column vector — filter rows
            include_flat = keep.iter().map(|row| row[0]).collect();
            filter_rows = true;
        } else if keep.len() == 1 && keep[0].len() == num_rows {
            // Row vector with length == num_rows — also filter rows
            include_flat = keep[0].clone();
            filter_rows = true;
        } else if keep.len() == 1 && keep[0].len() == num_cols {
            // Row vector — filter columns
            include_flat = keep[0].clone();
            filter_rows = false;
        } else if keep.len() == num_cols && keep[0].len() == 1 {
            // Column vector with length == num_cols — filter columns
            include_flat = keep.iter().map(|row| row[0]).collect();
            filter_rows = false;
        } else {
            return CalcResult::new_error(
                Error::VALUE,
                cell,
                "include array dimensions must match array rows or columns".to_string(),
            );
        }

        let filtered: Vec<Vec<ArrayNode>> = if filter_rows {
            data.into_iter()
                .enumerate()
                .filter_map(|(i, row)| if include_flat[i] { Some(row) } else { None })
                .collect()
        } else {
            let col_indices: Vec<usize> = include_flat
                .iter()
                .enumerate()
                .filter_map(|(i, &keep)| if keep { Some(i) } else { None })
                .collect();
            data.iter()
                .map(|row| col_indices.iter().map(|&c| row[c].clone()).collect())
                .collect()
        };

        let is_empty_result = filtered.is_empty() || filtered.first().is_none_or(|r| r.is_empty());

        if is_empty_result {
            if args.len() >= 3 {
                let empty_val = self.evaluate_node_in_context(&args[2], cell);
                match empty_val {
                    CalcResult::EmptyArg => CalcResult::new_error(
                        Error::CALC,
                        cell,
                        "No data returned by FILTER".to_string(),
                    ),
                    // A range is its values: FILTER gives values, never a reference, found or not
                    // (Excel-cached: upstream ISREF.xlsx Sheet1!D11, D14, D17, D20; probes/isref-names-filter-if-empty).
                    // A blank one-cell reference is 0 (`ROWS` 1, `&"|"` "0|"), unlike a blank
                    // inside a larger range, which stays blank (probes/t3654-filter-result-types
                    // P46:X46, P55:X55).
                    CalcResult::EmptyCell => CalcResult::Number(0.0),
                    CalcResult::Range { left, right } => {
                        match self.evaluate_range(left, right, cell) {
                            Ok(array) => CalcResult::Array(array),
                            Err(error) => error,
                        }
                    }
                    // An array if_empty spills as is (Excel: probes/filter-if-empty-array).
                    CalcResult::Array(a) => CalcResult::Array(a),
                    // A scalar stays a scalar, not a 1x1 array: `MAX(FILTER(.., .., ""))` with no
                    // match is #VALUE! because MAX gets the text "" as a value (upstream
                    // MAX_MIN.xlsx H39, H40, H43).
                    v => v,
                }
            } else {
                CalcResult::new_error(Error::CALC, cell, "No data returned by FILTER".to_string())
            }
        } else {
            CalcResult::Array(filtered)
        }
    }
}
