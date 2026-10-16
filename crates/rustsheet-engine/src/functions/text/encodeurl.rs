use crate::{
    calc_result::CalcResult,
    expressions::{parser::Node, token::Error, types::CellReferenceIndex},
    functions::information::{error_array, Lifted},
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
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        match self.lifted_arg(&args[0], cell) {
            Ok(Lifted::Scalar(error @ CalcResult::Error { .. })) => error,
            Ok(Lifted::Scalar(_)) => CalcResult::new_error(Error::VALUE, cell, String::new()),
            Ok(Lifted::Array(values)) => error_array(&values, &Error::VALUE),
            Err(error) => error,
        }
    }
}
