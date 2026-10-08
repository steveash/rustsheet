use super::variance::VarianceKind;
use crate::expressions::types::CellReferenceIndex;
use crate::{calc_result::CalcResult, expressions::parser::Node, model::Model};

// STDEV.* is the square root of the matching VAR.* (`excel_variance`).
impl Model {
    pub(crate) fn fn_stdev_p(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.variance_of_values(args, cell, VarianceKind::Population, true, "STDEV.P")
    }

    pub(crate) fn fn_stdev_s(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.variance_of_values(args, cell, VarianceKind::Sample, true, "STDEV.S")
    }

    pub(crate) fn fn_stdeva(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.variance_of_values_a(args, cell, VarianceKind::Sample, true, "STDEVA")
    }

    pub(crate) fn fn_stdevpa(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.variance_of_values_a(args, cell, VarianceKind::Population, true, "STDEVPA")
    }
}
