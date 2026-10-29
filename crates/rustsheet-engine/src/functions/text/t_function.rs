// T(value): text and errors pass through, anything else is "".

use crate::{
    calc_result::CalcResult,
    expressions::{
        parser::{ArrayNode, Node},
        token::Error,
        types::CellReferenceIndex,
    },
    functions::{information::map_elements, text::common::reject_over_short_limit},
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
        // Text over 255 units read from a cell is "" however the cell is reached: a range, name,
        // INDIRECT, OFFSET, INDEX, IF, CHOOSE, LET, spill reference, sheet-qualified, parenthesised
        // or legacy reference, or structured reference. A computed value stays `#VALUE!`, and so
        // does `+D1` (probes/text-255-units-reference-shapes A1:A19; T-3107).
        let from_cell = self.reads_cell_text(&args[0], cell);
        match result {
            // Text over 255 UTF-16 units is #VALUE!, as in REPT, CODE and UNICODE:
            // `T(REPT("x",256))` (docs/excel-semantics/text.md "REPT, CODE, UNICODE and T refuse a
            // text argument over 255 UTF-16 units", probes/t2199-text-length-and-combin-cuts A12).
            // Read from a cell, such text is "" instead, and an array element over the limit is
            // #VALUE! (probes/text-255-units-from-cell A4, A12; -array-element A4; T-2627).
            CalcResult::String(ref text) => match reject_over_short_limit(text) {
                Ok(()) => result,
                Err(_) if from_cell => CalcResult::String(String::new().into()),
                Err(error) => CalcResult::new_error(error, cell, String::new()),
            },
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
                    ArrayNode::String(text) if reject_over_short_limit(text).is_err() => {
                        ArrayNode::Error(Error::VALUE)
                    }
                    ArrayNode::String(_) | ArrayNode::Error(_) => value.clone(),
                    _ => ArrayNode::String("".into()),
                }))
            }
            _ => CalcResult::String("".into()),
        }
    }

    /// Whether `node` hands T a cell's text rather than a computed value. A legacy formula's
    /// automatic `@` and a spill reference still stand for cells; a unary plus makes a value
    /// (probes/text-255-units-reference-shapes A9, A13, A18; T-3107).
    fn reads_cell_text(&mut self, node: &Node, cell: CellReferenceIndex) -> bool {
        let mut node = node;
        loop {
            match node {
                Node::UnaryPlus { .. } => return false,
                Node::Group { child, .. } | Node::Spaced { child, .. } => node = child,
                Node::ImplicitIntersection { child, .. } => node = child,
                Node::SpillRangeOperator { .. } => return true,
                _ => return self.is_reference(node, cell),
            }
        }
    }
}
