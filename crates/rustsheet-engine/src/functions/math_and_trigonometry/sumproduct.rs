use crate::arithmetic::{extended_product, extended_sum, flush_subnormal};
use crate::{
    calc_result::CalcResult,
    expressions::{
        parser::{ArrayNode, Node},
        token::Error,
        types::CellReferenceIndex,
    },
    model::Model,
    reference_operators::RefArg,
};

impl Model {
    // ── SUMPRODUCT ────────────────────────────────────────────────────────────

    /// `=SUMPRODUCT(array1, [array2], ...)`
    ///
    /// Multiplies corresponding elements across all arrays and returns the sum.
    /// Non-numeric values (text, booleans, empty) are treated as 0, but a blank one-cell
    /// reference is `#VALUE!`. All arrays must have the same dimensions; a mismatch yields
    /// `#VALUE!` unless a later argument is itself an error or blank.
    pub(crate) fn fn_sumproduct(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }

        // Read every argument in order: a whole-argument error or a blank one-cell argument
        // wins even over a shape mismatch between earlier arguments
        // (`SUMPRODUCT(NA(),D1:E2,blank)` is `#N/A`, `SUMPRODUCT(blank,D1:E2,NA())` `#VALUE!`;
        // probes/t2200-blank-and-padded-m5-args A22, A23; coercion.md, T-2150). Once the shapes
        // disagree, the elements of that and later arguments are not read, so an error element
        // there does not win: unprobed, TODO(T-2450).
        let mut arrays: Vec<Vec<Vec<f64>>> = Vec::with_capacity(args.len());
        let mut shape: Option<(usize, usize)> = None;
        let mut mismatch = false;
        for arg in args {
            let arr = match self.sumproduct_array(arg, cell) {
                Ok(a) => a,
                Err(e) => return e,
            };
            let arg_shape = (arr.len(), arr.first().map_or(0, Vec::len));
            mismatch = mismatch || *shape.get_or_insert(arg_shape) != arg_shape;
            if mismatch {
                continue;
            }

            // Coerce to f64: non-numeric → 0.0, errors propagate.
            let mut num_arr: Vec<Vec<f64>> = Vec::with_capacity(arr.len());
            for row in &arr {
                let mut num_row: Vec<f64> = Vec::with_capacity(row.len());
                for node in row {
                    let v = match node {
                        ArrayNode::Number(v) => *v,
                        ArrayNode::Boolean(_) | ArrayNode::String(_) | ArrayNode::Empty => 0.0,
                        ArrayNode::Lambda(_) => {
                            return CalcResult::new_error(Error::VALUE, cell, String::new())
                        }
                        ArrayNode::Error(e) => {
                            return CalcResult::Error {
                                error: e.clone(),
                                origin: cell,
                                message: String::new(),
                            }
                        }
                    };
                    num_row.push(v);
                }
                num_arr.push(num_row);
            }
            arrays.push(num_arr);
        }

        if mismatch {
            return CalcResult::new_error(
                Error::VALUE,
                cell,
                "SUMPRODUCT: arrays must have the same dimensions".to_string(),
            );
        }
        let (rows, cols) = shape.unwrap_or((0, 0));
        let mut total = 0.0f64;

        for i in 0..rows {
            for j in 0..cols {
                // Each multiplication rounds twice, as `*` does (probes-numeric/
                // double-rounding-sumproduct-multi A1-A14, T-1697).
                let product = arrays.iter().fold(1.0, |acc, a| {
                    flush_subnormal(extended_product(acc, a[i][j]))
                });
                // Excel rounds each addition of the running total twice, as SUM does (numbers.md
                // "double rounding", probes-numeric/double-rounding-avedev-deviations, T-1665).
                total = extended_sum(total, product);
            }
        }

        CalcResult::finite_number_or_num(flush_subnormal(total), cell)
    }

    /// One SUMPRODUCT argument as an array; a blank one-cell reference is `#VALUE!`. An array of
    /// references reads each reference element as a non-number (0) and an error element as its
    /// error (SPEC-0014 rule 5:
    /// `SUMPRODUCT(OFFSET(A1,{0;1},0))` is 0, `SUMPRODUCT(OFFSET(A1,{0;-1},0))` `#REF!`;
    /// probes/reference-lift-consumers E29, -b2 E189).
    fn sumproduct_array(
        &mut self,
        arg: &Node,
        cell: CellReferenceIndex,
    ) -> Result<Vec<Vec<ArrayNode>>, CalcResult> {
        match self.evaluate_value_or_grid(arg, cell) {
            RefArg::Grid(grid) => Ok(grid.to_array(ArrayNode::Empty)),
            read => {
                // A blank cell given as a cell or one-cell range reference is `#VALUE!`, alone or
                // beside other arguments, while a one-cell text or boolean is a 0 factor
                // (probes/t2200-blank-and-padded-m5-args A1-A19; coercion.md, T-2150).
                let result = self.collapse_single_cell(read.into_single_area(cell));
                if matches!(result, CalcResult::EmptyCell) {
                    return Err(CalcResult::new_error(
                        Error::VALUE,
                        cell,
                        "SUMPRODUCT: a one-cell argument is blank".to_string(),
                    ));
                }
                self.result_to_array(result, cell)
            }
        }
    }
}
