use super::common::{broadcast_text, TextArg};
use crate::{
    calc_result::CalcResult,
    cast::array_node_to_string,
    expressions::{
        parser::{ArrayNode, Node},
        token::Error,
        types::CellReferenceIndex,
    },
    model::Model,
};

/// Longest text a cell can hold. Excel returns `#VALUE!` for a longer CONCATENATE result; it also
/// bounds the allocation a broadcast over a large range can make (CLAUDE.md directive 5).
const MAX_TEXT_CHARS: usize = 32767;

impl Model {
    /// CONCATENATE(text1, [text2], ...): joins text; array arguments lift element-wise.
    pub(crate) fn fn_concatenate(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }
        // Array arguments lift element-wise (arrays.md "NOT and CONCATENATE lift over array
        // arguments"); with only scalars the result is the plain joined string.
        let mut operands = Vec::with_capacity(args.len());
        for arg in args {
            match self.text_arg(arg, cell) {
                Ok(operand) => operands.push(operand),
                Err(error) => return error,
            }
        }
        let operand_refs: Vec<&TextArg> = operands.iter().collect();
        broadcast_text(cell, &operand_refs, |i, j| {
            let mut joined = String::new();
            for operand in &operands {
                joined.push_str(&array_node_to_string(operand.elem(i, j))?);
                if joined.len() > MAX_TEXT_CHARS && joined.chars().count() > MAX_TEXT_CHARS {
                    return Err(Error::VALUE);
                }
            }
            Ok(ArrayNode::String(joined))
        })
    }
}
