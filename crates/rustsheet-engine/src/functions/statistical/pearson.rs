use crate::expressions::types::CellReferenceIndex;
use crate::{calc_result::CalcResult, expressions::parser::Node, model::Model};

impl Model {
    // PEARSON(array1, array2)
    pub(crate) fn fn_pearson(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.correlation_coefficient(args, cell)
    }

    // RSQ(known_y's, known_x's): the square of the correlation coefficient, which Excel takes
    // over n - 1 where CORREL takes it over n (`RSQ(J1:J6,B1:B6)` is 1.0000000000000004:
    // probes/regression-pairs-a N column).
    pub(crate) fn fn_rsq(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let moments = match self.paired_moments(args, cell) {
            Ok(moments) => moments,
            Err(error) => return error,
        };
        match moments.correlation(moments.n - 1.0) {
            Ok(r) => CalcResult::Number(r * r),
            Err(error) => super::correl::moment_error(error, cell),
        }
    }
}
