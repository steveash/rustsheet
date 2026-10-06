//! Reading and validating the VLOOKUP/HLOOKUP index argument.

use crate::{
    calc_result::CalcResult, expressions::parser::Node, expressions::token::Error,
    expressions::types::CellReferenceIndex, model::Model,
};

/// The row/column index argument of VLOOKUP/HLOOKUP, read before the search but validated only
/// after a match (docs/excel-semantics/lookup.md "VLOOKUP/HLOOKUP search before validating the
/// index").
pub(super) enum LookupIndexArg {
    /// A value argument, already coerced to a number (a failed coercion fails at once).
    Number(f64),
    /// A reference argument; its cell is read only once the lookup value is found.
    Reference(CalcResult),
}

impl Model {
    /// Reads the index argument of VLOOKUP/HLOOKUP. A value argument is coerced now, so literal
    /// text or an error fails before the search (probes/lookup-index-order H9, H13, H28); a
    /// reference is kept for [`Model::resolve_lookup_index`].
    pub(super) fn lookup_index_arg(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> Result<LookupIndexArg, CalcResult> {
        match self.evaluate_node_with_reference(node, cell) {
            range @ CalcResult::Range { .. } => Ok(LookupIndexArg::Reference(range)),
            value => self.cast_to_number(value, cell).map(LookupIndexArg::Number),
        }
    }

    /// Validates the index once the lookup value is found and returns it 0-based. A referenced
    /// cell that is not a number (text, an error) is `#REF!`; an index below 1 is `#VALUE!`, past
    /// `size` or at or below the 32-bit integer minimum is `#REF!` (probes/lookup-index-order, excel-graph).
    pub(super) fn resolve_lookup_index(
        &mut self,
        index: LookupIndexArg,
        size: i32,
        cell: CellReferenceIndex,
    ) -> Result<i32, CalcResult> {
        let invalid_reference = || CalcResult::Error {
            error: Error::REF,
            origin: cell,
            message: "Invalid reference".to_string(),
        };
        let number = match index {
            LookupIndexArg::Number(number) => number,
            LookupIndexArg::Reference(range) => self
                .cast_to_number(range, cell)
                .map_err(|_| invalid_reference())?,
        };
        // -2147483648 is already #REF! while -2147483647 is #VALUE! (probes/lookup-index-shapes
        // H19-H20, H31-H32); a fractional index near the cut-off is unprobed.
        if number <= f64::from(i32::MIN) {
            return Err(invalid_reference());
        }
        // In range now, so the cast cannot saturate on the low side; a huge positive index
        // saturates to i32::MAX, which is past any table.
        let index = number.floor() as i32;
        if index < 1 {
            return Err(CalcResult::Error {
                error: Error::VALUE,
                origin: cell,
                message: "Index must be at least 1".to_string(),
            });
        }
        if index > size {
            return Err(invalid_reference());
        }
        Ok(index - 1)
    }
}
