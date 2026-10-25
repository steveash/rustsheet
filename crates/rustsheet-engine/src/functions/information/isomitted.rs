use crate::{
    calc_result::CalcResult,
    expressions::{parser::Node, types::CellReferenceIndex},
    functions::Function,
    model::Model,
};

impl Model {
    pub(crate) fn fn_isomitted(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        // LOOKUP's empty value for a blank result cell is also `EmptyArg` but is not an
        // omitted parameter: Excel gives FALSE (arrays.md "ISOMITTED of a function result"). The
        // argument is still evaluated so its errors and dependencies are tracked as before.
        let from_lookup = matches!(
            args[0].ungrouped(),
            Node::FunctionKind {
                kind: Function::Lookup,
                ..
            }
        );
        // A parameter is omitted by how it was bound, not by its value: one bound to the empty
        // value (`LAMBDA(x,ISOMITTED(x))(IFERROR(Z1,1))`, a blank element in MAP) is FALSE
        // (probes/t2461-blank-array-sources A12:A14, T-2649).
        if let Node::NamedVariableKind { id: Some(_), .. } = args[0].ungrouped() {
            return CalcResult::Boolean(self.is_omitted_variable(args[0].ungrouped()));
        }
        let value = self.evaluate_node_in_context(&args[0], cell);
        CalcResult::Boolean(matches!(value, CalcResult::EmptyArg) && !from_lookup)
    }
}
