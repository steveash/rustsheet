use super::variance::VarianceKind;
use crate::expressions::types::CellReferenceIndex;
use crate::functions::util::aggregate_args::GridReading;
use crate::{calc_result::CalcResult, expressions::parser::Node, model::Model};

// STDEV.* is the square root of the matching VAR.* (`excel_variance`).
impl Model {
    pub(crate) fn fn_stdev_p(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        // Unprobed over an array of references: the value view.
        self.variance_of_values(
            args,
            GridReading::Values,
            cell,
            VarianceKind::Population,
            true,
            "STDEV.P",
        )
    }

    pub(crate) fn fn_stdev_s(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        // probes/t2065-lift-stdevs, probes/t2816-lift-stdevs (T-2824); legacy STDEV too
        // (probes/t2577-indirect-ref-array Calc!A157).
        self.variance_of_values(
            args,
            GridReading::TopLeft,
            cell,
            VarianceKind::Sample,
            true,
            "STDEV.S",
        )
    }

    pub(crate) fn fn_stdeva(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.variance_of_values_a(args, cell, VarianceKind::Sample, true, "STDEVA")
    }

    pub(crate) fn fn_stdevpa(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.variance_of_values_a(args, cell, VarianceKind::Population, true, "STDEVPA")
    }
}
