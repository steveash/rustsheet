use crate::{
    calc_result::CalcResult,
    cast::StringOrArray,
    expressions::{
        parser::{ArrayNode, Node},
        token::Error,
        types::CellReferenceIndex,
    },
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
        match self.get_string_or_array(&args[0], cell) {
            Ok(StringOrArray::String(_)) => {
                CalcResult::new_error(Error::VALUE, cell, String::new())
            }
            Ok(StringOrArray::Array(arr)) => CalcResult::Array(
                arr.iter()
                    .map(|row| {
                        row.iter()
                            .map(|n| match n {
                                ArrayNode::Error(e) => ArrayNode::Error(e.clone()),
                                _ => ArrayNode::Error(Error::VALUE),
                            })
                            .collect()
                    })
                    .collect(),
            ),
            Err(e) => e,
        }
    }
}
