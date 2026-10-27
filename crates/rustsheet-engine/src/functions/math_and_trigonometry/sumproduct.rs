use crate::arithmetic::{extended_product, extended_sum, flush_subnormal};
use crate::nested_array::NestedKind;
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
    /// `#VALUE!` unless an argument holds an error element or a later one is itself an error or blank.
    pub(crate) fn fn_sumproduct(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }

        // Read every argument in order: a whole-argument error or a blank one-cell argument
        // wins even over a shape mismatch between earlier arguments
        // (`SUMPRODUCT(NA(),D1:E2,blank)` is `#N/A`, `SUMPRODUCT(blank,D1:E2,NA())` `#VALUE!`;
        // probes/t2200-blank-and-padded-m5-args A22, A23; coercion.md, T-2150). An error element
        // of any argument beats a shape mismatch in either order, but not a whole-argument
        // error of a later argument (`SUMPRODUCT(R,X,NA())` is `#N/A`); the first element error
        // in argument then element order is kept (probes/t2296-sumproduct-error-element, T-2450). A blank
        // one-cell argument after an element error also wins, inferred and unprobed.
        let mut arrays: Vec<Vec<Vec<f64>>> = Vec::with_capacity(args.len());
        let mut shape: Option<(usize, usize)> = None;
        let mut mismatch = false;
        let mut element_error: Option<Error> = None;
        for arg in args {
            let arr = match self.sumproduct_array(arg, cell) {
                Ok(a) => a,
                Err(e) => return e,
            };
            let arg_shape = (arr.len(), arr.first().map_or(0, Vec::len));
            mismatch = mismatch || *shape.get_or_insert(arg_shape) != arg_shape;

            // Coerce to f64: non-numeric → 0.0.
            let mut num_arr: Vec<Vec<f64>> = Vec::with_capacity(arr.len());
            for row in &arr {
                let mut num_row: Vec<f64> = Vec::with_capacity(row.len());
                for node in row {
                    let v = match node {
                        ArrayNode::Number(v) => *v,
                        ArrayNode::Boolean(_) | ArrayNode::String(_) | ArrayNode::Empty => 0.0,
                        // A nested GROUPBY or PIVOTBY cell is no number: `SUMPRODUCT(GROUPBY(..))`
                        // is 0 (probes/t2594-nested-aggregates A22, T-2625). A reference cell is
                        // unprobed and stays `#VALUE!`.
                        ArrayNode::Nested(nested) if nested.kind() == NestedKind::Array => 0.0,
                        ArrayNode::Lambda(_) | ArrayNode::Nested(_) => {
                            return Model::lambda_value_error(cell)
                        }
                        ArrayNode::Error(e) => {
                            element_error.get_or_insert_with(|| e.clone());
                            0.0
                        }
                    };
                    num_row.push(v);
                }
                num_arr.push(num_row);
            }
            arrays.push(num_arr);
        }

        if let Some(error) = element_error {
            return CalcResult::new_error(error, cell, String::new());
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
