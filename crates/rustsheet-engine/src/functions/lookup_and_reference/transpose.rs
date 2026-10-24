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

        let result = self.evaluate_node_in_context(&args[0], cell);
        let is_array = matches!(result, CalcResult::Array(_));
        let data = match self.result_to_array(result, cell) {
            Ok(d) => d,
            Err(e) => return e,
        };

        if data.is_empty() {
            return CalcResult::Array(vec![]);
        }

        let num_rows = data.len();
        let num_cols = data[0].len();

        // A single value (a scalar, one cell or a one-cell range) transposes to that value, not a
        // 1x1 array, so a lifting function over it does not lift:
        // `COLUMNS(TEXTSPLIT(TRANSPOSE("a,b"),","))` is 2 and `ISREF(TRANSPOSE(A1))` is FALSE. A
        // 1x1 array, a constant or computed, stays an array: `TRANSPOSE({"a,b"})` and
        // `TRANSPOSE(IF(TRUE,{"a,b"}))` split to 1 column (probes/t2455-transpose-single-value
        // A12:A16, A70:A72; docs/excel-semantics/text.md, T-2455).
        if num_rows == 1 && num_cols == 1 && !is_array {
            return array_element_value(&data[0][0], cell);
        }

        let result: Vec<Vec<ArrayNode>> = (0..num_cols)
            .map(|j| (0..num_rows).map(|i| data[i][j].clone()).collect())
            .collect();

        CalcResult::Array(result)
    }
}
