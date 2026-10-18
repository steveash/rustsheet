use crate::{
    calc_result::CalcResult,
    expressions::{parser::Node, token::Error, types::CellReferenceIndex},
    functions::Function,
    model::Model,
};

impl Model {
    // ENCODEURL(text)
    // Excel for the web (the excel-graph golden, decision 0039) gives `#VALUE!` for every
    // argument probed (docs/excel-semantics/web.md "ENCODEURL is `#VALUE!` in Excel for the web"),
    // lifted over a range or array one element at a time. An error argument propagates as it
    // came (gen-m9/fcf11f2ec6f7a59d A7 is `#VALUE!` through CODE; an error argument is unprobed
    // and follows FILTERXML). No text is encoded; desktop confirmation is T-1081.
    pub(crate) fn fn_encodeurl(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(
            Function::Encodeurl,
            args,
            cell,
            &mut |_, values| match values {
                [error @ CalcResult::Error { .. }] => error.clone(),
                [_] => CalcResult::new_error(Error::VALUE, cell, String::new()),
                _ => CalcResult::new_args_number_error(cell),
            },
        )
    }
}
