use crate::{
    calc_result::CalcResult,
    expressions::{parser::Node, types::CellReferenceIndex},
    model::Model,
};

impl Model {
    pub(crate) fn fn_isomitted(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        // A built-in's empty value (`LOOKUP` hitting a blank cell) is also `EmptyArg` but is not an
        // omitted parameter: Excel gives FALSE (arrays.md "ISOMITTED of a function result"). The
        // argument is still evaluated so its errors and dependencies are tracked as before.
        let from_builtin = matches!(args[0].ungrouped(), Node::FunctionKind { .. });
        let value = self.evaluate_node_in_context(&args[0], cell);
        CalcResult::Boolean(matches!(value, CalcResult::EmptyArg) && !from_builtin)
    }
}
