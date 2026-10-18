use crate::{
    calc_result::CalcResult,
    expressions::{parser::Node, token::Error, types::CellReferenceIndex},
    model::Model,
};

impl Model {
    /// `=GETPIVOTDATA(data_field, pivot_table, [field1, item1, ...])` for a workbook without pivot
    /// tables, which RustSheet does not read (docs/excel-semantics/web.md "GETPIVOTDATA without a
    /// pivot table", probes/getpivotdata-evidence-floor, T-1703): an error in `data_field` or
    /// `pivot_table` is the result, otherwise `#REF!` whatever the references and pairs are. Which
    /// error wins when both are errors is unprobed (`data_field` first here). The field/item pairs
    /// are not evaluated.
    pub(crate) fn fn_getpivotdata(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        if args.len() < 2 {
            return CalcResult::new_args_number_error(cell);
        }
        for arg in &args[..2] {
            let value = self.evaluate_node_in_context(arg, cell);
            if value.is_error() {
                return value;
            }
        }
        CalcResult::new_error(Error::REF, cell, "No pivot table".to_string())
    }
}
