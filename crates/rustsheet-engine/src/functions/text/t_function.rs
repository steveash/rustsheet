// T(value): text and errors pass through, anything else is "".

use crate::{
    calc_result::CalcResult,
    expressions::{
        parser::{ArrayNode, Node},
        token::Error,
        types::CellReferenceIndex,
    },
    functions::information::map_elements,
    model::Model,
};

impl Model {
    pub(crate) fn fn_t(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.t_of_argument(args, cell, false)
    }

    /// T in a legacy formula's value position (SPEC-0012 rule 3): an array argument is `#VALUE!`.
    pub(crate) fn fn_t_legacy(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.t_of_argument(args, cell, true)
    }

    /// T's kernel; `legacy` makes an array argument `#VALUE!`, whatever its size.
    fn t_of_argument(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
        legacy: bool,
    ) -> CalcResult {
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        // A range is its top-left cell, with no lift and no implicit intersection, legacy or
        // dynamic: `T(A1:A3)` with A1 "x" is "x" from any row, and with A1 5 "" even from A2's
        // row (probes/t-n-range-top-left F2, F10, F12; t-array-lift S!E7; T-0985).
        let result = self.evaluate_node_top_left(&args[0], cell);
        match result {
            CalcResult::String(_) => result,
            error @ CalcResult::Error { .. } => error,
            // An array lifts element-wise, text and error elements kept and anything else "":
            // `T({"a",1,#N/A,TRUE})` spills "a", "", #N/A, "" (probes/t-array-lift,
            // legacy-array-result S!G1:G2; T-1729).
            // In a legacy formula's value position it is `#VALUE!` (legacy-n-t-reference-args;
            // SPEC-0012).
            CalcResult::Array(_) if legacy => {
                CalcResult::new_error(Error::VALUE, cell, String::new())
            }
            CalcResult::Array(values) => {
                CalcResult::Array(map_elements(&values, |value| match value {
                    ArrayNode::String(_) | ArrayNode::Error(_) => value.clone(),
                    _ => ArrayNode::String("".into()),
                }))
            }
            _ => CalcResult::String("".into()),
        }
    }
}
