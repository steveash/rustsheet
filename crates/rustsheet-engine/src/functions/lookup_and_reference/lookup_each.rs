//! VLOOKUP/HLOOKUP over an array or multi-cell `lookup_value` (T-0270).

use crate::{
    calc_result::CalcResult,
    cast::calc_result_to_array_node,
    expressions::{parser::ArrayNode, token::Error, types::CellReferenceIndex},
    functions::{
        lift::{LiftCost, LiftOptions, LiftedArg},
        util::array_node_to_calc_result,
    },
    limits::check_array_size,
    model::Model,
};

use super::{index_arg::LookupIndexArg, lookup_index, lookup_table::SearchVector};

/// What one VLOOKUP/HLOOKUP search needs besides the lookup value: the first column (or row),
/// the index argument with the table size it is checked against, and the match mode.
pub(super) struct LookupSearch<'a> {
    pub(super) search_vector: &'a SearchVector,
    pub(super) index: LookupIndexArg,
    pub(super) size: i32,
    /// The `range_lookup` argument: a boolean, or an array that lifts with the lookup value.
    pub(super) is_sorted: CalcResult,
}

impl Model {
    /// The VLOOKUP/HLOOKUP search for `lookup_value`, then `pick` of the found position and the
    /// resolved index. Excel searches first: a value that is not found is #N/A whatever the
    /// index is. An array or multi-cell range `lookup_value` is looked up element by element
    /// into a same-shaped array, each element by the scalar rules (an error element is itself,
    /// probes/vlookup-array-lookup-value, T-0270); it lifts through [`Model::lift_values`],
    /// so a whole column of lookup values is bounded by the shared cell-visit cap
    /// (elements times the search vector length).
    pub(super) fn lookup_each(
        &mut self,
        lookup_value: CalcResult,
        search: &LookupSearch<'_>,
        as_reference: bool,
        cell: CellReferenceIndex,
        pick: impl Fn(&mut Self, usize, i32, bool) -> CalcResult,
    ) -> CalcResult {
        // A lifted element is a value: only a scalar lookup may hand its cell on as a reference.
        let is_array = LiftedArg::is_grid;
        let lifts = is_array(&lookup_value) || is_array(&search.is_sorted);
        let as_reference = as_reference && !lifts;
        // Only an array range_lookup is lifted; a boolean one adds no criterion, so it does not
        // multiply the cell-visit estimate.
        let mut criteria = vec![lookup_value];
        if is_array(&search.is_sorted) {
            criteria.push(search.is_sorted.clone());
        }
        // A lifted lookup or range_lookup reads only the first element of an index array.
        let narrowed = lifts.then(|| search.index.first_element(cell));
        let index = narrowed.as_ref().unwrap_or(&search.index);
        // Each element scans the evaluated cells of the first column (or row) once, however the
        // table was given: a computed array table counts too, not just a range. The estimate is
        // charged once per lifted criterion, as T-1784's driver charged it.
        let scan = search.search_vector.scan_cells();
        let criteria_count = u64::try_from(criteria.len()).unwrap_or(u64::MAX);
        self.lift_values(
            criteria,
            LiftCost::Cells(scan.saturating_mul(criteria_count)),
            &LiftOptions::REUSE_EQUAL_RUNS,
            cell,
            &mut |model, values| {
                let value = &values[0];
                if value.is_error() {
                    return value.clone();
                }
                let is_sorted = match model
                    .cast_to_bool(values.get(1).unwrap_or(&search.is_sorted).clone(), cell)
                {
                    Ok(is_sorted) => is_sorted,
                    Err(error) => return error,
                };
                let Some(found) = lookup_index(value, search.search_vector, is_sorted) else {
                    return CalcResult::Error {
                        error: Error::NA,
                        origin: cell,
                        message: "Not found".to_string(),
                    };
                };
                match (lifts, &search.index) {
                    (false, LookupIndexArg::Array(grid)) => {
                        return model.pick_each(grid, found, search.size, cell, &pick);
                    }
                    // Read only now that the value is found, so the range's size or errors never
                    // hide a `#N/A`.
                    (false, LookupIndexArg::Range { left, right }) => {
                        return match model.evaluate_range(*left, *right, cell) {
                            Ok(grid) => model.pick_each(&grid, found, search.size, cell, &pick),
                            Err(error) => error,
                        };
                    }
                    _ => {}
                }
                match model.resolve_lookup_index(index.clone(), search.size, cell) {
                    Ok(index) => pick(model, found, index, as_reference),
                    Err(error) => error,
                }
            },
        )
    }

    /// A scalar lookup with an index array: the index elements are all validated in reading
    /// order, the first bad one giving the error for the whole call, and then each picks its
    /// cell into a same-shaped array (probes/vlookup-array-index-range-lookup, T-1269).
    fn pick_each(
        &mut self,
        grid: &[Vec<ArrayNode>],
        found: usize,
        size: i32,
        cell: CellReferenceIndex,
        pick: &impl Fn(&mut Self, usize, i32, bool) -> CalcResult,
    ) -> CalcResult {
        let rows = grid.len();
        let columns = grid.first().map_or(0, Vec::len);
        if let Some((error, message)) = check_array_size(rows, columns) {
            return CalcResult::new_error(error, cell, message);
        }
        let mut indexes = Vec::with_capacity(rows);
        for row in grid {
            let mut resolved = Vec::with_capacity(columns);
            for node in row {
                let element = LookupIndexArg::Reference(array_node_to_calc_result(node, cell));
                match self.resolve_lookup_index(element, size, cell) {
                    Ok(index) => resolved.push(index),
                    Err(error) => return error,
                }
            }
            indexes.push(resolved);
        }
        CalcResult::Array(
            indexes
                .into_iter()
                .map(|row| {
                    row.into_iter()
                        .map(|index| calc_result_to_array_node(pick(self, found, index, false)))
                        .collect()
                })
                .collect(),
        )
    }
}
