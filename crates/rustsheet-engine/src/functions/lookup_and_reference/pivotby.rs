//! PIVOTBY on the grouping core GROUPBY shares (SPEC-0006 step 5, T-1326): the row groups and the
//! column groups are two [`Grouping`]s of the same records, laid out as a grid. Excel:
//! docs/excel-semantics/arrays.md "GROUPBY and PIVOTBY", evidence probes/groupby-pivotby sheet P,
//! its follow-ups (T-1328) and probes/pivotby-followup (excel-graph, 2026-10-16, T-1326).

use super::groupby::{
    at, function_argument, is_function_array, not_implemented, shown_key, value_error,
    values_have_header, FieldHeaders, Grouping, SortOrder,
};
use crate::{
    calc_result::CalcResult,
    cast::{array_node_to_string, MAX_TEXT_UNITS},
    expressions::{
        parser::{ArrayNode, Node},
        types::CellReferenceIndex,
    },
    functions::{logical::LambdaCaller, Function},
    limits::check_array_size,
    model::Model,
};

/// One column of the PIVOTBY grid right of the row keys: value column `value` of column group
/// `group`, or the total of value column `value`.
#[derive(Clone, Copy)]
enum Slot {
    Group { group: usize, value: usize },
    Total { value: usize },
}

/// The arguments of one PIVOTBY call after evaluation and checks.
struct Pivot {
    row_keys: Vec<Vec<ArrayNode>>,
    col_keys: Vec<Vec<ArrayNode>>,
    values: Vec<Vec<ArrayNode>>,
    headers: FieldHeaders,
    row_depth: i8,
    col_depth: i8,
    row_sort: SortOrder,
    col_sort: SortOrder,
    /// The record rows (indices into the three arrays), headers and filtered rows left out.
    records: Vec<usize>,
}

impl Model {
    /// `=PIVOTBY(row_fields, col_fields, values, function, [field_headers], [row_total_depth],
    /// [row_sort_order], [col_total_depth], [col_sort_order], [filter_array], [relative_to])`
    pub(crate) fn fn_pivotby(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        match self.pivotby(args, cell) {
            Ok(array) => CalcResult::Array(array),
            Err(error) => error,
        }
    }

    fn pivotby(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> Result<Vec<Vec<ArrayNode>>, CalcResult> {
        let (pivot, function) = self.pivotby_args(args, cell)?;
        let row_fields = width(&pivot.row_keys);
        let col_fields = width(&pivot.col_keys);
        let value_columns = width(&pivot.values);
        let shows_header = matches!(pivot.headers, FieldHeaders::Generated | FieldHeaders::Shown);
        // Header rows: the column field names (shown headers only), one row per column key
        // field, then the row field and value names (shown headers only) (P!M1, P!A1).
        let header_rows = col_fields + 2 * usize::from(shows_header);
        let extra_rows = header_rows + usize::from(pivot.row_depth != 0);
        let total_columns = if pivot.col_depth == 0 {
            0
        } else {
            value_columns
        };
        // The output size is checked while grouping, before any sort, LAMBDA call or the output
        // allocation (SPEC-0006 step 6): first with one column group, then with the real count.
        let min_width = row_fields
            .saturating_add(value_columns)
            .saturating_add(total_columns);
        let row_limit =
            |groups: usize| check_array_size(groups.saturating_add(extra_rows), min_width);
        let mut rows = Grouping::new(&pivot.row_keys, pivot.records.iter().copied(), row_limit)
            .map_err(|(error, message)| CalcResult::new_error(error, cell, message))?;
        let height = rows.groups.len().saturating_add(extra_rows);
        let grid_width = |groups: usize| {
            groups
                .saturating_mul(value_columns)
                .saturating_add(row_fields)
                .saturating_add(total_columns)
        };
        let col_limit = |groups: usize| check_array_size(height, grid_width(groups));
        let mut cols = Grouping::new(&pivot.col_keys, pivot.records.iter().copied(), col_limit)
            .map_err(|(error, message)| CalcResult::new_error(error, cell, message))?;
        if let Some((error, message)) = check_array_size(height, grid_width(cols.groups.len())) {
            return Err(CalcResult::new_error(error, cell, message));
        }

        let mut caller = self.lambda_caller(&function, 1);
        // Rows sort by a value only when there is a Total column: with col_total_depth 0 they keep
        // the key order (probes/pivotby-followup P!A781, P!A1221; with it, P!M121, P!A801).
        // Columns sort by a value with or without the Total row (P!A821, P!A841).
        let key_order = SortOrder::Keys(Vec::new());
        let row_sort = match pivot.row_sort {
            SortOrder::Value { .. } if pivot.col_depth == 0 => &key_order,
            ref order => order,
        };
        let values = &pivot.values;
        self.sort_pivot_groups(&mut caller, &mut rows, values, row_sort, true, cell);
        let col_sort = &pivot.col_sort;
        self.sort_pivot_groups(&mut caller, &mut cols, values, col_sort, false, cell);

        let slots = slots(cols.groups.len(), value_columns, pivot.col_depth);
        let mut output = Vec::with_capacity(height);
        if shows_header {
            output.push(column_names_row(&pivot, &slots));
        }
        for field in 0..col_fields {
            let mut row = vec![ArrayNode::String("".into()); row_fields];
            row.extend(slots.iter().map(|slot| {
                match slot {
                    Slot::Group { group, .. } => cols
                        .groups
                        .get(*group)
                        .and_then(|g| g.keys.get(field))
                        .map_or(ArrayNode::Empty, shown_key),
                    Slot::Total { .. } if field == 0 => ArrayNode::String("Total".into()),
                    Slot::Total { .. } => ArrayNode::String("".into()),
                }
            }));
            output.push(row);
        }
        if shows_header {
            output.push(value_names_row(&pivot, &slots));
        }
        let mut column_of = vec![usize::MAX; pivot.values.len()];
        for (index, group) in cols.groups.iter().enumerate() {
            for &row in &group.rows {
                if let Some(entry) = column_of.get_mut(row) {
                    *entry = index;
                }
            }
        }
        if pivot.row_depth < 0 {
            let row = self.pivot_total_row(&mut caller, &pivot, &cols, &slots, cell);
            output.push(row);
        }
        let mut buckets: Vec<Vec<usize>> = vec![Vec::new(); cols.groups.len()];
        for group in &rows.groups {
            for bucket in &mut buckets {
                bucket.clear();
            }
            for &row in &group.rows {
                if let Some(bucket) = column_of.get(row).and_then(|&c| buckets.get_mut(c)) {
                    bucket.push(row);
                }
            }
            let mut row: Vec<ArrayNode> = group.keys.iter().map(shown_key).collect();
            for slot in &slots {
                let cell_rows = match slot {
                    Slot::Group { group, .. } => buckets.get(*group).map_or(&[][..], Vec::as_slice),
                    Slot::Total { .. } => &group.rows[..],
                };
                row.push(self.pivot_cell(&mut caller, &pivot.values, *slot, cell_rows, cell));
            }
            output.push(row);
        }
        if pivot.row_depth > 0 {
            let row = self.pivot_total_row(&mut caller, &pivot, &cols, &slots, cell);
            output.push(row);
        }
        Ok(output)
    }

    /// Evaluates and checks the PIVOTBY arguments. Function arrays (T-1325), `relative_to`
    /// (T-1327) and the subtotals of depth ±2 (T-1324) are `#N/IMPL!`, never a wrong value.
    fn pivotby_args(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> Result<(Pivot, CalcResult), CalcResult> {
        if !(4..=11).contains(&args.len()) {
            return Err(CalcResult::new_args_number_error(cell));
        }
        let row_keys = self.eval_to_array(&args[0], cell)?;
        let col_keys = self.eval_to_array(&args[1], cell)?;
        let values = self.eval_to_array(&args[2], cell)?;
        // TODO(T-1325): function arrays (`HSTACK(SUM,MAX)`, P!A121).
        if is_function_array(&args[3]) {
            return Err(not_implemented(cell, "PIVOTBY function arrays"));
        }
        // Every error value first, in argument order, then the validation, as in GROUPBY: an error
        // option beats an invalid one before or after it and a non-function
        // (probes/pivotby-followup P!A1401-A1461, P!A1261, P!A1281).
        let mut options = self.evaluate_options(&args[3..], cell)?.into_iter();
        let function = options.next().flatten().unwrap_or(CalcResult::EmptyArg);
        let field_headers = options.next().flatten();
        let row_depth = options.next().flatten();
        let row_sort = options.next().flatten();
        let col_depth = options.next().flatten();
        let col_sort = options.next().flatten();
        let filter = options.next().flatten();
        let relative_to = options.next().flatten();
        let row_depth_given = row_depth.is_some();
        let col_depth_given = col_depth.is_some();

        let (function, function_array) = function_argument(function, "PIVOTBY", cell)?;
        let headers = self.field_headers(field_headers, cell)?;
        let (row_fields, col_fields, value_columns) =
            (width(&row_keys), width(&col_keys), width(&values));
        // ±2 with one field is #VALUE! like GROUPBY's (probes/pivotby-followup P!A401, P!A701);
        // 3 too (P!A381).
        let row_depth = self.total_depth(row_depth, row_fields, cell)?;
        let col_depth = self.total_depth(col_depth, col_fields, cell)?;
        // Fields of another height than the values are #VALUE! (probes/groupby-pivotby-followup
        // G!A81, probes/pivotby-followup P!A301).
        let height = values.len();
        if row_keys.len() != height
            || col_keys.len() != height
            || row_fields == 0
            || col_fields == 0
        {
            return Err(value_error(
                cell,
                "row_fields, col_fields and values must have the same height",
            ));
        }
        // GROUPBY's sort_order rules hold on both axes (P!A141, P!A161, P!A721, P!A1001, P!A1321-A1381).
        let row_sort = self.sort_order(row_sort, row_fields, value_columns, cell)?;
        let col_sort = self.sort_order(col_sort, col_fields, value_columns, cell)?;
        let header_row = match headers {
            // Only the values decide, as in GROUPBY (probes/pivotby-followup P!A1, P!A1021).
            FieldHeaders::Detect => values_have_header(&values).ok_or_else(|| {
                value_error(cell, "PIVOTBY cannot tell whether there are headers")
            })?,
            FieldHeaders::No | FieldHeaders::Generated => false,
            FieldHeaders::Hidden | FieldHeaders::Shown => true,
        };
        let first = usize::from(header_row);
        if first >= height {
            return Err(value_error(cell, "PIVOTBY has no records"));
        }
        // GROUPBY's filter rules hold too: another height, errors and text are #VALUE!, a header
        // row may be left out of the filter, and keeping no record is #VALUE! (P!A281,
        // P!A1481-A1601, probes/groupby-pivotby-followup G!A881, -followup3 G!A133).
        let records = self.filter_rows(filter, height, first, cell)?;
        if function_array {
            // TODO(T-1325): function arrays (`HSTACK(SUM,MAX)`, P!A121).
            return Err(not_implemented(cell, "PIVOTBY function arrays"));
        }
        // TODO(T-1327): relative_to. An array there is lifted per element in Excel
        // (probes/pivotby-followup P!A681).
        if relative_to.is_some() {
            return Err(not_implemented(cell, "PIVOTBY relative_to"));
        }
        // TODO(T-1327): PERCENTOF gets each cell's values and its relative set (the column total
        // when relative_to is omitted: P!Y25); called with one argument it would be wrong.
        if matches!(&function, CalcResult::Lambda(id)
            if self.eta_lambdas.get(id) == Some(&Function::Percentof))
        {
            return Err(not_implemented(cell, "PIVOTBY with PERCENTOF"));
        }
        // TODO(T-1324): subtotals (P!A97, P!M97).
        if row_depth.abs() == 2 || col_depth.abs() == 2 {
            return Err(not_implemented(cell, "PIVOTBY subtotals"));
        }
        // TODO(T-1324): sorting by a value with several fields and the depth omitted shows
        // subtotals, as total depth 2 does (rows: probes/pivotby-followup P!A201, P!A901; columns:
        // P!A941; GROUPBY too: P!A921). An explicit depth 0 or 1 gives none (P!A861, P!A1141).
        let value_sort_subtotals = |sort: &SortOrder, fields: usize, depth_given: bool| {
            matches!(sort, SortOrder::Value { .. }) && fields > 1 && !depth_given
        };
        if value_sort_subtotals(&row_sort, row_fields, row_depth_given)
            || value_sort_subtotals(&col_sort, col_fields, col_depth_given)
        {
            return Err(not_implemented(cell, "PIVOTBY subtotals"));
        }
        let pivot = Pivot {
            row_keys,
            col_keys,
            values,
            headers,
            row_depth,
            col_depth,
            row_sort,
            col_sort,
            records,
        };
        Ok((pivot, function))
    }

    /// Sorts the row or column groups of a pivot by `order`. A key order sorts field by field
    /// (blank last both ways: P!M25, probes/pivotby-followup P!A981). A value order sorts by the
    /// aggregate of that value column (ascending P!M121, descending P!A481; ties in key order,
    /// P!A741; the blank key not pinned). With several
    /// fields the rows sort every level by its own aggregate (P!A861, P!A881), but the columns
    /// only the last, their parents keeping the key order (`parents_by_value`; P!A1141, P!A1161).
    fn sort_pivot_groups(
        &mut self,
        caller: &mut LambdaCaller,
        grouping: &mut Grouping,
        values: &[Vec<ArrayNode>],
        order: &SortOrder,
        parents_by_value: bool,
        cell: CellReferenceIndex,
    ) {
        match order {
            SortOrder::Keys(ascending) => grouping.sort_by_keys(ascending, cell),
            SortOrder::Value { column, ascending } => {
                // The returned aggregates are the group totals, not grid cells; they are not
                // reused for the Total column (one call per group more, within the size check).
                grouping.sort_by_aggregate(*ascending, parents_by_value, cell, |rows| {
                    self.aggregate(caller, values, *column, rows, cell)
                });
            }
        }
    }

    /// One grid cell: `""` when no record falls in it (P!A1, East/2025), the function of the
    /// records otherwise (0 when they sum to 0, P!A97). A total column with several value columns
    /// is `""` in every row, the Total row too (P!Y97, probes/groupby-pivotby-followup G!A281).
    fn pivot_cell(
        &mut self,
        caller: &mut LambdaCaller,
        values: &[Vec<ArrayNode>],
        slot: Slot,
        rows: &[usize],
        cell: CellReferenceIndex,
    ) -> ArrayNode {
        match slot {
            Slot::Total { .. } if width(values) > 1 => ArrayNode::String("".into()),
            _ if rows.is_empty() => ArrayNode::String("".into()),
            Slot::Group { value, .. } | Slot::Total { value } => {
                self.aggregate(caller, values, value, rows, cell)
            }
        }
    }

    /// The Total row: "Total", `""` under the other row fields, then the function of each column
    /// group's records, and of all records under the total column (P!A1, P!A97).
    fn pivot_total_row(
        &mut self,
        caller: &mut LambdaCaller,
        pivot: &Pivot,
        cols: &Grouping,
        slots: &[Slot],
        cell: CellReferenceIndex,
    ) -> Vec<ArrayNode> {
        let mut row = vec![ArrayNode::String("Total".into())];
        row.extend((1..width(&pivot.row_keys)).map(|_| ArrayNode::String("".into())));
        for slot in slots {
            let rows = match slot {
                Slot::Group { group, .. } => {
                    cols.groups.get(*group).map_or(&[][..], |g| &g.rows[..])
                }
                Slot::Total { .. } => &pivot.records[..],
            };
            row.push(self.pivot_cell(caller, &pivot.values, *slot, rows, cell));
        }
        row
    }
}

/// The number of columns of a (rectangular) array.
fn width(data: &[Vec<ArrayNode>]) -> usize {
    data.first().map_or(0, Vec::len)
}

/// The grid columns right of the row keys: each column group's value columns in turn, and the
/// total columns after them (`col_total_depth` 1) or before them (-1: probes/pivotby-followup
/// P!A101, P!A121).
fn slots(groups: usize, value_columns: usize, col_depth: i8) -> Vec<Slot> {
    let grid = (0..groups)
        .flat_map(|group| (0..value_columns).map(move |value| Slot::Group { group, value }));
    let totals = (0..value_columns).map(|value| Slot::Total { value });
    match col_depth {
        0 => grid.collect(),
        depth if depth < 0 => totals.chain(grid).collect(),
        _ => grid.chain(totals).collect(),
    }
}

/// The first header row of shown headers: the column field names, joined with ", " as text, above
/// the first column group, also when the total column comes first; `""` elsewhere (P!M1;
/// probes/pivotby-followup P!A61 "Product, Year", P!A421 "2023" as text, P!A1181). Generated
/// names are "Column Field 1, Column Field 2" (P!A81).
fn column_names_row(pivot: &Pivot, slots: &[Slot]) -> Vec<ArrayNode> {
    let col_fields = width(&pivot.col_keys);
    let name = if pivot.headers == FieldHeaders::Shown {
        joined_names((0..col_fields).map(|field| at(&pivot.col_keys, 0, field)))
    } else {
        joined_names(
            (1..=col_fields).map(|n| ArrayNode::String(format!("Column Field {n}").into())),
        )
    };
    let mut row = vec![ArrayNode::String("".into()); width(&pivot.row_keys)];
    let mut name = Some(name);
    for slot in slots {
        row.push(match slot {
            Slot::Group { .. } => name.take().unwrap_or(ArrayNode::String("".into())),
            Slot::Total { .. } => ArrayNode::String("".into()),
        });
    }
    row
}

/// Header cells joined with ", " as text. An error cell makes the name that error, and a name
/// longer than a cell can hold is `#VALUE!` (both unprobed; the length bound is directive 5).
fn joined_names(names: impl Iterator<Item = ArrayNode>) -> ArrayNode {
    let mut parts = Vec::new();
    let mut units = 0usize;
    for name in names {
        match array_node_to_string(&name) {
            Ok(text) => {
                units = units.saturating_add(text.encode_utf16().count() + 2);
                if units > MAX_TEXT_UNITS + 2 {
                    return ArrayNode::Error(crate::expressions::token::Error::VALUE);
                }
                parts.push(text);
            }
            Err(error) => return ArrayNode::Error(error),
        }
    }
    ArrayNode::String(parts.join(", ").into())
}

/// The last header row of shown headers: the row field names, then the value field name of each
/// grid column, total columns included (P!M1, probes/pivotby-followup P!A521); generated names
/// are "Row Field n" and "Value n" (P!A21, P!A81).
fn value_names_row(pivot: &Pivot, slots: &[Slot]) -> Vec<ArrayNode> {
    let shown = pivot.headers == FieldHeaders::Shown;
    let mut row: Vec<ArrayNode> = (0..width(&pivot.row_keys))
        .map(|field| {
            if shown {
                at(&pivot.row_keys, 0, field)
            } else {
                ArrayNode::String(format!("Row Field {}", field + 1).into())
            }
        })
        .collect();
    row.extend(slots.iter().map(|slot| {
        let (Slot::Group { value, .. } | Slot::Total { value }) = *slot;
        if shown {
            at(&pivot.values, 0, value)
        } else {
            ArrayNode::String(format!("Value {}", value + 1).into())
        }
    }));
    row
}
