use crate::{
    calc_result::CalcResult,
    expressions::{
        parser::{ArrayNode, Node},
        types::CellReferenceIndex,
    },
    functions::util::array_element_value,
    model::Model,
};

impl Model {
    // ── TRANSPOSE ─────────────────────────────────────────────────────────────

    /// `=TRANSPOSE(array)`
    ///
    /// Returns the transpose of the array: rows become columns and vice versa.
    pub(crate) fn fn_transpose(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }

        let data = match self.eval_to_array(&args[0], cell) {
            Ok(d) => d,
            Err(e) => return e,
        };

        if data.is_empty() {
            return CalcResult::Array(vec![]);
        }

        let num_rows = data.len();
        let num_cols = data[0].len();

        // A single value (a scalar or one cell) transposes to that value, not a 1x1 array, so a
        // lifting function over it does not lift: `COLUMNS(TEXTSPLIT(TRANSPOSE("a,b"),","))` is 2
        // and `ISREF(TRANSPOSE(A1))` is FALSE (docs/excel-semantics/text.md, T-2455). An array
        // constant stays an array: `COLUMNS(TEXTSPLIT(TRANSPOSE({"a,b"}),","))` is 1
        // (probes/t2455-transpose-single-value A12).
        if num_rows == 1 && num_cols == 1 && !matches!(args[0], Node::ArrayKind(_)) {
            return array_element_value(&data[0][0], cell);
        }

        let result: Vec<Vec<ArrayNode>> = (0..num_cols)
            .map(|j| (0..num_rows).map(|i| data[i][j].clone()).collect())
            .collect();

        CalcResult::Array(result)
    }
}
