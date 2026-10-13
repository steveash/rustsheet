//! Reading and validating the VLOOKUP/HLOOKUP index argument.

use crate::{
    calc_result::CalcResult,
    expressions::{
        parser::{ArrayNode, Node},
        token::Error,
        types::CellReferenceIndex,
    },
    functions::{util::array_node_to_calc_result, Function},
    model::Model,
    reference_operators::{several_areas_ref, Ranges, RefArg},
};

/// The row/column index argument of VLOOKUP/HLOOKUP, read before the search but validated only
/// after a match (docs/excel-semantics/lookup.md "VLOOKUP/HLOOKUP search before validating the
/// index").
#[derive(Clone)]
pub(super) enum LookupIndexArg {
    /// A value argument, already coerced to a number (a failed coercion fails at once).
    Number(f64),
    /// A reference argument; its cell is read only once the lookup value is found.
    Reference(CalcResult),
    /// An array argument (`{1,2}`): with a scalar `lookup_value` and `range_lookup` it lifts, once
    /// the value is found; otherwise only its first element is used. Elements validate like a
    /// referenced cell (probes/vlookup-array-index-range-lookup, T-1269).
    Array(Vec<Vec<ArrayNode>>),
    /// A multi-cell range argument, kept unread like a reference: it lifts like an array of its
    /// values once the value is found, so a value that is not found is `#N/A` however large the
    /// range is (probes/lookup-index-multicell-if, T-1426); a lifted lookup reads only its
    /// first cell.
    Range {
        left: CellReferenceIndex,
        right: CellReferenceIndex,
    },
}

impl LookupIndexArg {
    /// The index as one value: an array argument gives its first element, read like a referenced
    /// cell (probes/vlookup-array-index-range-lookup A1-A3, A137-A149).
    pub(super) fn first_element(&self, cell: CellReferenceIndex) -> LookupIndexArg {
        match self {
            LookupIndexArg::Array(grid) => grid
                .first()
                .and_then(|row| row.first())
                .map_or(LookupIndexArg::Number(f64::NAN), |node| {
                    LookupIndexArg::Reference(array_node_to_calc_result(node, cell))
                }),
            LookupIndexArg::Range { left, .. } => LookupIndexArg::Reference(CalcResult::Range {
                left: *left,
                right: *left,
            }),
            other => other.clone(),
        }
    }
}

impl Model {
    /// Reads the index argument of VLOOKUP/HLOOKUP. A value argument is coerced now, so literal
    /// text or an error fails before the search (probes/lookup-index-order H9, H13, H28); a
    /// reference is kept for [`Model::resolve_lookup_index`]. `INDEX(...)` is a reference too: its
    /// selected cell is read only after a match (probes/lookup-index-shapes H2, H3, H11, H12).
    pub(super) fn lookup_index_arg(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> Result<LookupIndexArg, CalcResult> {
        // An `IF` index is the cell it picks, so the lookup pick stays a reference
        // (probes/lookup-pick-index-shapes A3, A9, B2; T-1377).
        let index = match node.ungrouped() {
            Node::FunctionKind {
                kind: Function::If, ..
            } => match self.evaluate_reference_arg(node, cell) {
                // A union picked by `IF` is `#REF!` like a non-numeric referenced cell, so only
                // after a match: a value not found is `#N/A` (probes/lookup-index-multicell-if
                // M1, lookup-index-multicell-lifted M1; T-1426).
                RefArg::Ranges(Ranges::Areas(_)) => {
                    return Ok(LookupIndexArg::Reference(several_areas_ref(cell)))
                }
                picked => picked.into_single_area(cell),
            },
            _ => self.eval_ref_or_index(node, cell),
        };
        match index {
            // A multi-cell range index lifts like an array of its values, never intersecting
            // (probes/lookup-index-multicell-if A1-D2, J1-L2, O1; T-1426).
            CalcResult::Range { left, right }
                if left.row != right.row || left.column != right.column =>
            {
                Ok(LookupIndexArg::Range { left, right })
            }
            range @ CalcResult::Range { .. } => Ok(LookupIndexArg::Reference(range)),
            CalcResult::Array(grid) => Ok(LookupIndexArg::Array(grid)),
            value => self.cast_to_number(value, cell).map(LookupIndexArg::Number),
        }
    }

    /// Reads `range_lookup` (default TRUE): a scalar is coerced to a boolean now, so a bad one
    /// fails before the search; an array stays as it is and lifts with the lookup value.
    pub(super) fn lookup_sorted_arg(
        &mut self,
        node: Option<&Node>,
        cell: CellReferenceIndex,
    ) -> Result<CalcResult, CalcResult> {
        let Some(node) = node else {
            return Ok(CalcResult::Boolean(true));
        };
        match self.evaluate_node_in_context(node, cell) {
            array @ CalcResult::Array(_) => Ok(array),
            value => self.cast_to_bool(value, cell).map(CalcResult::Boolean),
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
            // Callers narrow an array or range with `first_element`; one that does not gets `#REF!`.
            LookupIndexArg::Array(_) | LookupIndexArg::Range { .. } => {
                return Err(invalid_reference())
            }
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
