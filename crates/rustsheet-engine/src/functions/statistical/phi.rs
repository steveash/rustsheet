use crate::arithmetic::flush_subnormal;
use crate::expressions::types::CellReferenceIndex;
use crate::functions::statistical::special::standard_normal_pdf;
use crate::{calc_result::CalcResult, expressions::parser::Node, model::Model};

impl Model {
    // PHI(x) = standard normal PDF at x
    pub(crate) fn fn_phi(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }

        let x = match self.get_number_no_bools(&args[0], cell) {
            Ok(f) => f,
            Err(e) => return e,
        };

        let result = standard_normal_pdf(x);

        CalcResult::Number(flush_subnormal(result))
    }
}
