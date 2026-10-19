use crate::{
    calc_result::CalcResult,
    expressions::{parser::Node, token::Error, types::CellReferenceIndex},
    functions::Function,
    model::Model,
};

impl Model {
    // FILTERXML(xml, xpath)
    // Excel for the web (the excel-graph golden, decision 0039) gives `#VALUE!` for every valid
    // call (docs/excel-semantics/web.md "FILTERXML is `#VALUE!` in Excel for the web"). The
    // first argument's error propagates (gen-m6/abcfd9979f316267 A9 is `#N/A`,
    // gen-m7/4629e931955aa4b8 A34 is DROP's `#CALC!`); with errors in both, the first wins left to
    // right (probes/filterxml-error-arguments A1:A6). An array argument lifts, `#VALUE!` per
    // element: a first-argument array literal is probed (probes/filterxml-evidence-floor
    // D27:D28, T-1704), the other array forms in probes/filterxml-array-forms (T-1722). No XML
    // is parsed.
    pub(crate) fn fn_filterxml(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        // An array or range argument lifts: every element is `#VALUE!`
        // (probes/filterxml-evidence-floor D28, `FILTERXML({"<a>1</a>";"<a>2</a>"},"//a")`,
        // T-1704); all-scalar operands give one `#VALUE!`. An error element passes through left
        // to right, as the scalar call does.
        self.lift_text(Function::Filterxml, args, cell, &mut |_, values| {
            for value in values {
                if let CalcResult::Error { error, .. } = value {
                    return Err(error.clone());
                }
            }
            Err(Error::VALUE)
        })
    }
}
