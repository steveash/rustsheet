//! VLOOKUP/HLOOKUP over an array or multi-cell lookup_value (T-0270).

use crate::{
    calc_result::CalcResult,
    expressions::{parser::Node, token::Error, types::CellReferenceIndex},
    model::Model,
};

use super::{index_arg::LookupIndexArg, lookup_index, lookup_table::SearchVector};

/// What one VLOOKUP/HLOOKUP search needs besides the lookup value: the first column (or row),
/// the index argument with the table size it is checked against, and the match mode.
pub(super) struct LookupSearch<'a> {
    pub(super) search_vector: &'a SearchVector,
    pub(super) index: LookupIndexArg,
    pub(super) size: i32,
    pub(super) is_sorted: bool,
}

impl Model {
    /// The VLOOKUP/HLOOKUP search for `lookup_value`, then `pick` of the found position and the
    /// resolved index. Excel searches first: a value that is not found is #N/A whatever the
    /// index is. An array or multi-cell range `lookup_value` is looked up element by element
    /// into a same-shaped array, each element by the scalar rules (an error element is itself,
    /// probes/vlookup-array-lookup-value, T-0270); it lifts through [`Model::lift_with_cost`],
    /// so a whole column of lookup values is bounded by the shared cell-visit cap.
    pub(super) fn lookup_each(
        &mut self,
        lookup_value: CalcResult,
        table_node: &Node,
        search: &LookupSearch<'_>,
        as_reference: bool,
        cell: CellReferenceIndex,
        pick: impl Fn(&mut Self, usize, i32, bool) -> CalcResult,
    ) -> CalcResult {
        // A lifted element is a value: only a scalar lookup may hand its cell on as a reference.
        let lifts = matches!(
            lookup_value,
            CalcResult::Range { .. } | CalcResult::Array(_)
        );
        let as_reference = as_reference && !lifts;
        self.lift_with_cost(
            vec![lookup_value],
            |model| model.walk_cost(table_node, cell),
            cell,
            |model, values| {
                let value = &values[0];
                if value.is_error() {
                    return value.clone();
                }
                match lookup_index(value, search.search_vector, search.is_sorted) {
                    Some(found) => {
                        match model.resolve_lookup_index(search.index.clone(), search.size, cell) {
                            Ok(index) => pick(model, found, index, as_reference),
                            Err(error) => error,
                        }
                    }
                    None => CalcResult::Error {
                        error: Error::NA,
                        origin: cell,
                        message: "Not found".to_string(),
                    },
                }
            },
        )
    }
}
