use super::text_arg::{broadcast_text, TextArg};
use crate::{
    calc_result::CalcResult,
    expressions::{parser::Node, token::Error, types::CellReferenceIndex},
    model::Model,
};

impl Model {
    // FILTERXML(xml, xpath)
    // Excel for the web (the excel-graph golden, decision 0039) gives `#VALUE!` for every valid
    // call (docs/excel-semantics/web.md "FILTERXML is `#VALUE!` in Excel for the web"). The
    // first argument's error propagates (gen-m6/abcfd9979f316267 A9 is `#N/A`,
    // gen-m7/4629e931955aa4b8 A34 is DROP's `#CALC!`). An error only in the second argument, and
    // the winner when both err, are unprobed: we go left to right, first error wins. An
    // array argument lifts, element by element (T-1704). No XML
    // is parsed.
    pub(crate) fn fn_filterxml(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let mut operands = Vec::with_capacity(2);
        for arg in args {
            match self.text_arg(arg, cell) {
                Ok(TextArg::Error(_, error)) => return error,
                Ok(operand) => operands.push(operand),
                Err(e) => return e,
            }
        }
        // An array or range argument lifts: every element is `#VALUE!` (probes/filterxml-evidence-floor
        // D28, `FILTERXML({"<a>1</a>";"<a>2</a>"},"//a")`, T-1704).
        if operands
            .iter()
            .any(|o| o.dims() != (1, 1) || matches!(o, TextArg::Array(_)))
        {
            let refs: Vec<&TextArg> = operands.iter().collect();
            return broadcast_text(cell, &refs, |_, _| Err(Error::VALUE));
        }
        CalcResult::new_error(Error::VALUE, cell, String::new())
    }
}
