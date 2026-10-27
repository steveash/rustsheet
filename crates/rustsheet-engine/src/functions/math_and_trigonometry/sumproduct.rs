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

/// One SUMPRODUCT argument's elements as numbers.
enum Factors {
    /// Row by row.
    Grid(Vec<Vec<f64>>),
    /// One column: the head's rows, then `tail_len` rows of `tail`.
    Tailed {
        head: Vec<f64>,
        tail: f64,
        tail_len: usize,
    },
}

impl Factors {
    /// (rows, columns).
    fn shape(&self) -> (usize, usize) {
        match self {
            Factors::Grid(grid) => (grid.len(), grid.first().map_or(0, Vec::len)),
            Factors::Tailed { head, tail_len, .. } => (head.len() + tail_len, 1),
        }
    }

    /// The element at row `i`, column `j`, within the shape.
    fn get(&self, i: usize, j: usize) -> f64 {
        match self {
            Factors::Grid(grid) => grid
                .get(i)
                .and_then(|row| row.get(j))
                .copied()
                .unwrap_or(0.0),
            Factors::Tailed { head, tail, .. } => head.get(i).copied().unwrap_or(*tail),
        }
    }
}

/// One element as a SUMPRODUCT factor: non-numbers are 0, an error is 0 and goes to
/// `element_error` unless one is there already; a lambda or a nested reference is `#VALUE!`.
fn sumproduct_factor(
    node: &ArrayNode,
    cell: CellReferenceIndex,
    element_error: &mut Option<Error>,
) -> Result<f64, CalcResult> {
    Ok(match node {
        ArrayNode::Number(v) => *v,
        ArrayNode::Boolean(_) | ArrayNode::String(_) | ArrayNode::Empty => 0.0,
        // A nested GROUPBY or PIVOTBY cell is no number: `SUMPRODUCT(GROUPBY(..))`
        // is 0 (probes/t2594-nested-aggregates A22, T-2625). A reference cell is
        // unprobed and stays `#VALUE!`.
        ArrayNode::Nested(nested) if nested.kind() == NestedKind::Array => 0.0,
        ArrayNode::Lambda(_) | ArrayNode::Nested(_) => {
            return Err(Model::lambda_as_number_error(cell))
        }
        ArrayNode::Error(e) => {
            element_error.get_or_insert_with(|| e.clone());
            0.0
        }
    })
}

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
        let mut factors: Vec<Factors> = Vec::with_capacity(args.len());
        let mut shape: Option<(usize, usize)> = None;
        let mut mismatch = false;
        let mut element_error: Option<Error> = None;
        for arg in args {
            let arg_factors = match self.sumproduct_factors(arg, cell, &mut element_error) {
                Ok(factors) => factors,
                Err(e) => return e,
            };
            let arg_shape = arg_factors.shape();
            mismatch = mismatch || *shape.get_or_insert(arg_shape) != arg_shape;
            factors.push(arg_factors);
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
        // Each multiplication rounds twice, as `*` does (probes-numeric/
        // double-rounding-sumproduct-multi A1-A14, T-1697).
        let product = |i: usize, j: usize| {
            factors.iter().fold(1.0, |acc, factor| {
                flush_subnormal(extended_product(acc, factor.get(i, j)))
            })
        };
        // Past every tailed argument's head, when every argument is tailed, each row has the
        // same product (SPEC-0020, T-0471).
        let heads = factors
            .iter()
            .map(|factor| match factor {
                Factors::Tailed { head, .. } => Some(head.len()),
                Factors::Grid(_) => None,
            })
            .try_fold(0, |longest, head| head.map(|head| longest.max(head)));
        let walked = heads.unwrap_or(rows).min(rows);
        let mut total = 0.0f64;
        for i in 0..walked {
            for j in 0..cols {
                // Excel rounds each addition of the running total twice, as SUM does (numbers.md
                // "double rounding", probes-numeric/double-rounding-avedev-deviations, T-1665).
                total = extended_sum(total, product(i, j));
            }
        }
        if walked < rows {
            // A zero product leaves the total as it is (`extended_sum` adds a zero plainly, and
            // the total is never -0); any other one is added once per row, as the walk does.
            let tail = product(walked, 0);
            if tail != 0.0 {
                for _ in walked..rows {
                    total = extended_sum(total, tail);
                }
            }
        }

        CalcResult::finite_number_or_num(flush_subnormal(total), cell)
    }

    /// One SUMPRODUCT argument's factors, its elements read as numbers: an element-wise
    /// expression of whole columns as a head and a tail (SPEC-0020, T-0471), anything else as a
    /// grid. The first element error in element order goes to `element_error`.
    fn sumproduct_factors(
        &mut self,
        arg: &Node,
        cell: CellReferenceIndex,
        element_error: &mut Option<Error>,
    ) -> Result<Factors, CalcResult> {
        let tailed = self
            .read_tailed(arg, cell, |model| model.evaluate_value_or_grid(arg, cell))
            .and_then(crate::tailed_array::Tailed::elements);
        if let Some((head, tail)) = tailed {
            #[cfg(test)]
            crate::tailed_array::TAILED_WALKS.with(|walks| walks.set(walks.get() + 1));
            let head = head
                .map(|node| sumproduct_factor(&node, cell, element_error))
                .collect::<Result<Vec<f64>, CalcResult>>()?;
            let (tail, tail_len) = match tail {
                Some((node, rows)) => (sumproduct_factor(&node, cell, element_error)?, rows),
                // The pass is being abandoned: the value is discarded.
                None => (0.0, 0),
            };
            return Ok(Factors::Tailed {
                head,
                tail,
                tail_len,
            });
        }
        let array = self.sumproduct_array(arg, cell)?;
        let grid = array
            .iter()
            .map(|row| {
                row.iter()
                    .map(|node| sumproduct_factor(node, cell, element_error))
                    .collect()
            })
            .collect::<Result<Vec<Vec<f64>>, CalcResult>>()?;
        Ok(Factors::Grid(grid))
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
