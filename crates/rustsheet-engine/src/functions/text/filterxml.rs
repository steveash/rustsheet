use crate::{
    calc_result::CalcResult,
    expressions::{parser::Node, token::Error, types::CellReferenceIndex},
    model::Model,
};

impl Model {
    // FILTERXML(xml, xpath)
    // Excel for the web (the excel-graph golden, decision 0039) gives `#VALUE!` for every valid
    // call (docs/excel-semantics/web.md "FILTERXML is `#VALUE!` in Excel for the web"). The
    // arguments are evaluated left to right and the first error wins (gen-m6/abcfd9979f316267
    // A9 is `#N/A`, gen-m7/4629e931955aa4b8 A34 is DROP's `#CALC!`). Whether an array argument
    // lifts is unprobed, so it does not: the whole call is `#VALUE!`. No XML is parsed.
    pub(crate) fn fn_filterxml(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }
        for arg in args {
            let value = self.evaluate_node_in_context(arg, cell);
            if value.is_error() {
                return value;
            }
        }
        CalcResult::new_error(Error::VALUE, cell, String::new())
    }
}
