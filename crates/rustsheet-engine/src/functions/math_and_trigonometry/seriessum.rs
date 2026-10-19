use crate::{
    calc_result::CalcResult,
    expressions::{
        parser::{ArrayNode, Node},
        types::CellReferenceIndex,
    },
    functions::{
        lift::{num_error, value_error, LiftCost},
        util::is_multi_sheet_span,
        Function,
    },
    model::Model,
};

/// SERIESSUM's coefficients as read once for the whole call: the cells, or the error that reading
/// them gave, which every element reports after its own x, n and m checks.
type Coefficients = Result<Vec<Vec<ArrayNode>>, CalcResult>;

impl Model {
    // ── SERIESSUM ─────────────────────────────────────────────────────────────

    /// `=SERIESSUM(x, n, m, coefficients)`
    ///
    /// Returns the sum of a power series:
    ///   coeff[0]*x^n + coeff[1]*x^(n+m) + coeff[2]*x^(n+2m) + ...
    ///
    /// Excel: x, n and m are Analysis `ToolPak` arguments: they lift over an array and refuse a
    /// multi-cell range (probes/math-atp-array-lift C5-C16, scalar-lift-open-a Y24-Y26), while the
    /// coefficients are read whole, once, and handed to every element (math-atp-array-lift C13,
    /// C40, scalar-lift-open-a Y27; docs/excel-semantics/numbers.md "QUOTIENT, FACTDOUBLE and
    /// SERIESSUM argument modes"). Each element walks the coefficients, so the call is charged
    /// their cells per element.
    pub(crate) fn fn_seriessum(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let Some(node) = args.get(3).filter(|_| args.len() == 4) else {
            return CalcResult::new_args_number_error(cell);
        };
        let coefficients = self.series_coefficients(node, cell);
        let cells = coefficients.as_ref().map_or(1, |rows| {
            let count = rows.iter().map(Vec::len).sum::<usize>().max(1);
            u64::try_from(count).unwrap_or(u64::MAX)
        });
        self.lift_call_costed(
            Function::Seriessum,
            args,
            LiftCost::Cells(cells),
            cell,
            &mut |model, values| model.seriessum_of(values, &coefficients, cell),
        )
    }

    /// Reads SERIESSUM's coefficients. Excel: a single value (typed text, or a one-cell
    /// reference) is read as a number, so `SERIESSUM(1,0,1,"5")` and a one-cell text reference
    /// are 5; text inside an array literal or a multi-cell range is `#VALUE!`
    /// (probes/seriessum-boolean-coefficients A8:A11).
    fn series_coefficients(&mut self, node: &Node, cell: CellReferenceIndex) -> Coefficients {
        // Excel: a 3-D span as the coefficients (`Sheet1:Sheet2!A1`, even one cell wide) is `#VALUE!`, not the `#REF!`
        // that reading it as a value gives (probes/quotient-factdouble-seriessum-3d-legacy C4,
        // T-1832).
        if is_multi_sheet_span(node) {
            return Err(value_error(cell));
        }
        let result = self.evaluate_node_in_context(node, cell);
        let is_single = match &result {
            CalcResult::String(_) => true,
            CalcResult::Range { left, right } => left == right,
            _ => false,
        };
        let coefficients = self.result_to_array(result, cell)?;
        if is_single {
            if let Some(ArrayNode::String(text)) = coefficients.first().and_then(|row| row.first())
            {
                return match self.cast_number(text) {
                    Some(v) => Ok(vec![vec![ArrayNode::Number(v)]]),
                    None => Err(value_error(cell)),
                };
            }
        }
        Ok(coefficients)
    }

    /// One SERIESSUM element: x, n and m are checked in order, then the coefficients.
    fn seriessum_of(
        &self,
        values: &[CalcResult],
        coefficients: &Coefficients,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        // The fourth slot is the coefficients' `EmptyArg` (`WholeByCaller`).
        let [x, n, m, _] = values else {
            return CalcResult::new_args_number_error(cell);
        };
        let x = match self.cast_number_no_bools_ref(x, cell) {
            Ok(v) => v,
            Err(e) => return e,
        };
        let n = match self.cast_number_no_bools_ref(n, cell) {
            Ok(v) => v,
            Err(e) => return e,
        };
        let m = match self.cast_number_no_bools_ref(m, cell) {
            Ok(v) => v,
            Err(e) => return e,
        };
        let coefficients = match coefficients {
            Ok(rows) => rows,
            Err(error) => return error.clone(),
        };

        let mut sum = 0.0f64;
        for (i, node) in coefficients.iter().flatten().enumerate() {
            let coeff = match node {
                ArrayNode::Number(v) => *v,
                ArrayNode::Empty => 0.0,
                // Excel: a boolean coefficient is `#VALUE!` in every form (cell, typed, array
                // literal); probes/seriessum-boolean-coefficients, math-argument-modes C44.
                ArrayNode::Boolean(_) | ArrayNode::String(_) => return value_error(cell),
                ArrayNode::Error(e) => {
                    return CalcResult::new_error(e.clone(), cell, String::new())
                }
            };
            let exponent = n + (i as f64) * m;
            // Excel: with x = 0 a term whose power is zero or negative is #NUM! whatever its
            // coefficient (docs/excel-semantics/numbers.md "SERIESSUM with x = 0").
            if x == 0.0 && exponent <= 0.0 {
                return num_error(cell);
            }
            sum += coeff * x.powf(exponent);
        }

        if !sum.is_finite() {
            return num_error(cell);
        }
        CalcResult::Number(sum)
    }
}
