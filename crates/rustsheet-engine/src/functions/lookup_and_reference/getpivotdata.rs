use super::pivot_index::{nested_axis, PivotQuery};
use crate::{
    calc_result::CalcResult,
    expressions::{parser::Node, token::Error, types::CellReferenceIndex},
    functions::lift::{LiftCost, LiftOptions},
    model::Model,
};

/// The `#REF!` of every GETPIVOTDATA that finds no answer.
fn no_answer(cell: CellReferenceIndex) -> CalcResult {
    CalcResult::new_error(Error::REF, cell, "No pivot table answer".to_string())
}

impl Model {
    /// `=GETPIVOTDATA(data_field, pivot_table, [field1, item1, ...])` (SPEC-0015 "GETPIVOTDATA",
    /// docs/excel-semantics/web.md "GETPIVOTDATA against a pivot table", T-2083). An error in
    /// `data_field` or `pivot_table` is the result (`data_field` first; which wins is unprobed,
    /// T-1703). Then the pivot whose location meets the reference, its data field by caption or
    /// source name, and the items of the pairs pick the records the data field's aggregate runs
    /// over. Everything else is `#REF!`: no pivot or several, a pivot shape or argument SPEC-0015
    /// lists as unknown, a name that matches nothing. An array of items lifts (L1, L5); any
    /// other array argument is one `#REF!`.
    pub(crate) fn fn_getpivotdata(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        // Excel for the web refuses the workbook for 5, 7, ... arguments (a second or later field
        // without its item), so, like every refused count, it is `#ERROR!`, before any argument
        // is read (docs/excel-semantics/syntax.md "GETPIVOTDATA with an odd count").
        if args.len() < 2 || (args.len() >= 5 && args.len() % 2 == 1) {
            return CalcResult::new_args_number_error(cell);
        }
        let data_field = self.evaluate_node_in_context(&args[0], cell);
        if data_field.is_error() {
            return data_field;
        }
        let pivot = match self.evaluate_node_with_reference(&args[1], cell) {
            CalcResult::Range { left, right } => {
                if left == right {
                    let value = self.evaluate_cell(left);
                    if value.is_error() {
                        return value;
                    }
                }
                self.pivot_at(left, right)
            }
            value if value.is_error() => return value,
            _ => None,
        };
        let Some(pivot) = pivot else {
            return no_answer(cell);
        };
        match self.getpivotdata_on(pivot, &data_field, &args[2..], cell) {
            Some(result) => result,
            None => no_answer(cell),
        }
    }

    /// The one pivot on the reference's sheet whose location meets the rectangle `left`:`right`
    /// (H11, H16; none is H12, H20); several is SPEC-0015 unknown 9.
    fn pivot_at(&self, left: CellReferenceIndex, right: CellReferenceIndex) -> Option<usize> {
        if left.sheet != right.sheet {
            return None;
        }
        let sheet_id = self.workbook.worksheet(left.sheet).ok()?.sheet_id;
        let (top, bottom) = (left.row.min(right.row), left.row.max(right.row));
        let (first, last) = (left.column.min(right.column), left.column.max(right.column));
        let mut found = self
            .workbook
            .pivot_tables
            .iter()
            .enumerate()
            .filter(|(_, p)| {
                // The model's rows and columns are 1-based, a pivot's location 0-based.
                let area = p.location;
                p.sheet_id == sheet_id
                    && i64::from(area.top) < i64::from(bottom)
                    && i64::from(top) <= i64::from(area.bottom) + 1
                    && i64::from(area.left) < i64::from(last)
                    && i64::from(first) <= i64::from(area.right) + 1
            })
            .map(|(index, _)| index);
        let pivot = found.next()?;
        found.next().is_none().then_some(pivot)
    }

    /// GETPIVOTDATA against pivot `pivot`, `None` for `#REF!`. Names are resolved here, once per
    /// call, never per record.
    fn getpivotdata_on(
        &mut self,
        pivot: usize,
        data_field: &CalcResult,
        pairs: &[Node],
        cell: CellReferenceIndex,
    ) -> Option<CalcResult> {
        let axis = self.pivot_axis(pivot)?.to_vec();
        let table = self.workbook.pivot_tables.get(pivot)?;
        let cache = self.workbook.pivot_caches.get(table.cache as usize)?;
        let field_name = |field: u32| cache.fields.get(field as usize).map(|f| f.name.as_str());
        // Only a pivot with one data field gets here (`pivot_axis`).
        let data = table.data_fields.first()?;
        match data_field {
            // A blank cell is the only data field (J7); `""` and an omitted argument stay
            // `#REF!` (SPEC-0015 unknown 8).
            CalcResult::EmptyCell => {}
            // The caption or the source field name, ignoring case, untrimmed (H1, H2, H10, H19).
            CalcResult::String(name) => {
                let name = name.to_lowercase();
                if data.caption.to_lowercase() != name
                    && field_name(data.field)?.to_lowercase() != name
                {
                    return None;
                }
            }
            _ => return None,
        }
        // A field with no item (H15); a pair on a pivot with nested axis fields (unknown 10).
        if pairs.len() % 2 == 1 || (!pairs.is_empty() && nested_axis(table)) {
            return None;
        }
        let axis_names: Vec<String> = axis
            .iter()
            .map(|&f| field_name(f).map(str::to_lowercase))
            .collect::<Option<_>>()?;
        let mut positions = Vec::with_capacity(pairs.len() / 2);
        let mut items = Vec::with_capacity(pairs.len() / 2);
        let mut omitted = Vec::with_capacity(pairs.len() / 2);
        for pair in pairs.chunks_exact(2) {
            // A field name must be text naming a field on an axis (H8, J3), each field once
            // (H17; the same item twice is SPEC-0015 unknown 5).
            let CalcResult::String(name) = self.evaluate_node_in_context(&pair[0], cell) else {
                return None;
            };
            let name = name.to_lowercase();
            let position = axis_names.iter().position(|n| *n == name)?;
            if positions.contains(&position) {
                return None;
            }
            positions.push(position);
            omitted.push(matches!(pair[1], Node::EmptyArgKind));
            items.push(self.evaluate_node_in_context(&pair[1], cell));
        }
        let query_items = vec![None; axis.len()];
        Some(self.lift_values(
            items,
            LiftCost::Unit,
            &LiftOptions::EVERY_ELEMENT,
            cell,
            &mut |model, items| {
                let mut query = PivotQuery {
                    data_field: 0,
                    items: query_items.clone().into_boxed_slice(),
                };
                for ((&position, item), &omitted) in positions.iter().zip(items).zip(&omitted) {
                    // An item is text, a label cell's value included (J6), or an omitted
                    // argument for the blank item (pv23_041 G5:G7); a number, a date or a
                    // boolean is SPEC-0015 unknown 3, `""`, a blank cell or `"(blank)"`
                    // unprobed, an error in a pair unprobed.
                    let found = match item {
                        _ if omitted => model.pivot_blank_item(pivot, position),
                        CalcResult::String(name) => model.pivot_item(pivot, position, name),
                        _ => None,
                    };
                    let Some(found) = found else {
                        return no_answer(cell);
                    };
                    if let Some(slot) = query.items.get_mut(position) {
                        *slot = Some(found);
                    }
                }
                match model.pivot_answer(pivot, &query) {
                    Ok(value) => CalcResult::Number(value),
                    Err(error) => {
                        CalcResult::new_error(error, cell, "No pivot table answer".into())
                    }
                }
            },
        ))
    }
}
