//! PIVOTBY on the grouping core GROUPBY shares (SPEC-0006 step 5, T-1326; subtotals, T-1324): the
//! row groups and the column groups are two [`Grouping`]s of the same records, laid out as a grid,
//! each axis with its own subtotals. Excel:
//! docs/excel-semantics/arrays.md "GROUPBY and PIVOTBY", evidence probes/groupby-pivotby sheet P,
//! its follow-ups (T-1328) and probes/pivotby-followup (excel-graph, 2026-10-16, T-1326).

use std::borrow::Cow;

use super::groupby::{
    at, empty_text, function_argument, function_arrays_error, grand_total_label, is_function_array,
    key_names, not_implemented, shown_key, total_label_cells, value_error, value_name,
    values_have_header, width, FieldHeaders, SortOrder,
};
use super::grouping::{effective_depth, subtotal_levels, Grouping, Line, ParentOrder};
use crate::{
    calc_result::CalcResult,
    cast::{array_node_to_string, MAX_TEXT_UNITS},
    expressions::{
        parser::{ArrayNode, Node},
        token::Error,
        types::CellReferenceIndex,
    },
    functions::{logical::LambdaCaller, Function},
    limits::check_array_size,
    model::Model,
};

/// One column of the PIVOTBY grid right of the row keys: value column `value` of column group
/// `group`, of the subtotal of the column groups `start..end` (which share their first `level`
/// keys), or the total of value column `value`.
#[derive(Clone, Copy)]
enum Slot {
    Group {
        group: usize,
        value: usize,
    },
    Subtotal {
        level: usize,
        start: usize,
        end: usize,
        value: usize,
    },
    Total {
        value: usize,
    },
}

/// The arguments of one PIVOTBY call after evaluation and checks.
struct Pivot {
    row_keys: Vec<Vec<ArrayNode>>,
    col_keys: Vec<Vec<ArrayNode>>,
    values: Vec<Vec<ArrayNode>>,
    headers: FieldHeaders,
    /// The total depths laid out ([`effective_depth`]).
    row_depth: isize,
    col_depth: isize,
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
        let (mut rows, mut cols, height) = pivot_groupings(&pivot, cell)?;
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
        let row_totals = self.sort_pivot_groups(
            &mut caller,
            &mut rows,
            values,
            row_sort,
            ParentOrder::ByValue,
            cell,
        );
        // The column parents sort by their own aggregate only with subtotals (probes/pivotby-
        // followup P!A941, probes/groupby-subtotals G!A961, G!A991; without: P!A1141, P!A1161).
        let col_levels = subtotal_levels(pivot.col_depth);
        let col_parents = if col_levels > 0 {
            ParentOrder::ByValue
        } else {
            ParentOrder::ByKey
        };
        let col_sort = &pivot.col_sort;
        let col_totals =
            self.sort_pivot_groups(&mut caller, &mut cols, values, col_sort, col_parents, cell);

        let col_lines = cols.lines(col_levels, pivot.col_depth < 0);
        let slots = slots(&col_lines, width(values), pivot.col_depth);
        let mut output = Vec::with_capacity(height);
        output.extend(header_rows(&pivot, &cols, &slots));
        let totals = Totals {
            rows: row_totals,
            cols: col_totals,
        };
        let grid = self.grid_rows(&mut caller, &pivot, (&rows, &cols), &slots, &totals, cell);
        output.extend(grid);
        Ok(output)
    }

    /// The row groups' rows with their subtotals, and the Total row (before them for
    /// `row_total_depth` -1, after them for 1, none for 0: P!A1, P!Y1, probes/pivotby-followup
    /// P!A101; "Grand Total" with subtotals, P!A97, probes/groupby-subtotals G!A661).
    fn grid_rows(
        &mut self,
        caller: &mut LambdaCaller,
        pivot: &Pivot,
        (rows, cols): (&Grouping, &Grouping),
        slots: &[Slot],
        totals: &Totals,
        cell: CellReferenceIndex,
    ) -> Vec<Vec<ArrayNode>> {
        let levels = subtotal_levels(pivot.row_depth);
        let lines = rows.lines(levels, pivot.row_depth < 0);
        let row_fields = width(&pivot.row_keys);
        let mut column_of = vec![usize::MAX; pivot.values.len()];
        for (index, group) in cols.groups.iter().enumerate() {
            for &row in &group.rows {
                if let Some(entry) = column_of.get_mut(row) {
                    *entry = index;
                }
            }
        }
        let mut grid = Grid {
            values: &pivot.values,
            slots,
            column_of,
            buckets: vec![Vec::new(); cols.groups.len()],
            merged: Vec::new(),
        };
        let mut output = Vec::with_capacity(lines.len() + 1);
        let mut total = (pivot.row_depth != 0).then(|| {
            let label = total_label_cells(row_fields, levels);
            let reuse = |slot| totals.total_row_cell(slot);
            self.grid_row(caller, &mut grid, label, &pivot.records, reuse, cell)
        });
        if pivot.row_depth < 0 {
            output.extend(total.take());
        }
        for line in lines {
            let (label, records, group) = match line {
                Line::Group(index) => {
                    let Some(group) = rows.groups.get(index) else {
                        continue;
                    };
                    let label = group.keys.iter().map(shown_key).collect();
                    (label, Cow::Borrowed(&group.rows[..]), Some(index))
                }
                // A subtotal row is the function of all its groups' records (P!A97).
                Line::Subtotal { level, start, end } => (
                    rows.subtotal_label_cells(level, start, row_fields),
                    Cow::Owned(rows.rows_of(start, end)),
                    None,
                ),
            };
            let values = &pivot.values;
            let reuse = |slot| group.and_then(|index| totals.group_row_cell(index, slot, values));
            output.push(self.grid_row(caller, &mut grid, label, &records, reuse, cell));
        }
        output.extend(total);
        output
    }

    /// One grid row: `label` (the row keys), then the cell of each slot over `records` (the row's
    /// records, in source order), or the value `reuse` has for it (an aggregate a value sort
    /// computed).
    fn grid_row(
        &mut self,
        caller: &mut LambdaCaller,
        grid: &mut Grid<'_>,
        mut row: Vec<ArrayNode>,
        records: &[usize],
        reuse: impl Fn(Slot) -> Option<ArrayNode>,
        cell: CellReferenceIndex,
    ) -> Vec<ArrayNode> {
        for bucket in &mut grid.buckets {
            bucket.clear();
        }
        for &record in records {
            let column = grid.column_of.get(record).copied().unwrap_or(usize::MAX);
            if let Some(bucket) = grid.buckets.get_mut(column) {
                bucket.push(record);
            }
        }
        for &slot in grid.slots {
            if let Some(value) = reuse(slot) {
                row.push(value);
                continue;
            }
            let cell_rows: &[usize] = match slot {
                Slot::Group { group, .. } => grid.buckets.get(group).map_or(&[][..], Vec::as_slice),
                // A subtotal column's records are its groups', in source order
                // (`LAMBDA(x,ARRAYTOTEXT(x))`, probes/groupby-subtotals G!A1081).
                Slot::Subtotal { start, end, .. } => {
                    grid.merged.clear();
                    for bucket in grid.buckets.get(start..end).unwrap_or_default() {
                        grid.merged.extend_from_slice(bucket);
                    }
                    grid.merged.sort_unstable();
                    &grid.merged
                }
                Slot::Total { .. } => records,
            };
            row.push(self.pivot_cell(caller, grid.values, slot, cell_rows, cell));
        }
        row
    }

    /// Evaluates and checks the PIVOTBY arguments. Function arrays (T-1325), `relative_to`
    /// (T-1327) and column subtotals over several value columns are `#N/IMPL!`, never a wrong
    /// value.
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
        // Function arrays (`HSTACK(SUM,MAX)`, P!A121) are T-1325's.
        if is_function_array(&args[3]) {
            return Err(function_arrays_error(cell, "PIVOTBY"));
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
        // 3 with one or two too (P!A381).
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
            return Err(function_arrays_error(cell, "PIVOTBY"));
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
        let row_depth = effective_depth(row_depth, row_depth_given, &row_sort, row_fields);
        let col_depth = effective_depth(col_depth, col_depth_given, &col_sort, col_fields);
        // TODO(T-1699): column subtotals over several value columns. Excel shows most of their
        // cells as "", like the Total columns, but fills a few with other columns' subtotals
        // (probes/groupby-subtotals G!A781), a layout not yet understood.
        if subtotal_levels(col_depth) > 0 && value_columns > 1 {
            return Err(not_implemented(
                cell,
                "PIVOTBY column subtotals over several value columns",
            ));
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

    /// Sorts the row or column groups of a pivot by `order`. A key order is already applied
    /// ([`pivot_groupings`]: blank last both ways, P!M25, probes/pivotby-followup P!A981). A value
    /// order sorts by the
    /// aggregate of that value column (ascending P!M121, descending P!A481; ties in key order,
    /// P!A741; the blank key not pinned). With several
    /// fields the rows sort every level by its own aggregate (P!A861, P!A881), but the columns
    /// only the last unless they have subtotals, their parents keeping the key order (`parents`;
    /// P!A1141, P!A1161).
    /// Returns, for a value sort, the value column and each group's aggregate over all its records
    /// in the new order: the group totals (the Total column for rows, the Total row for columns),
    /// never grid cells, which are row x column intersections.
    fn sort_pivot_groups(
        &mut self,
        caller: &mut LambdaCaller,
        grouping: &mut Grouping,
        values: &[Vec<ArrayNode>],
        order: &SortOrder,
        parents: ParentOrder,
        cell: CellReferenceIndex,
    ) -> Option<(usize, Vec<ArrayNode>)> {
        let SortOrder::Value { column, ascending } = *order else {
            return None;
        };
        let totals = grouping.sort_by_aggregate(ascending, parents, cell, |rows| {
            self.aggregate(caller, values, column, rows, cell)
        });
        Some((column, totals))
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
            Slot::Total { .. } if width(values) > 1 => empty_text(),
            _ if rows.is_empty() => empty_text(),
            Slot::Group { value, .. } | Slot::Subtotal { value, .. } | Slot::Total { value } => {
                self.aggregate(caller, values, value, rows, cell)
            }
        }
    }
}

/// The aggregates value sorts computed, reused instead of calling the function again (GROUPBY
/// does the same): per row group for the Total column, per column group for the Total row.
struct Totals {
    rows: Option<(usize, Vec<ArrayNode>)>,
    cols: Option<(usize, Vec<ArrayNode>)>,
}

impl Totals {
    /// A group row's Total cell: the row group's aggregate, with one value column (several have
    /// `""` Total columns, P!Y97).
    fn group_row_cell(
        &self,
        group: usize,
        slot: Slot,
        values: &[Vec<ArrayNode>],
    ) -> Option<ArrayNode> {
        match slot {
            Slot::Total { value } if width(values) == 1 => {
                Self::get(self.rows.as_ref(), group, value)
            }
            _ => None,
        }
    }

    /// A Total row cell under a column group: that group's aggregate.
    fn total_row_cell(&self, slot: Slot) -> Option<ArrayNode> {
        match slot {
            Slot::Group { group, value } => Self::get(self.cols.as_ref(), group, value),
            _ => None,
        }
    }

    fn get(
        sorted: Option<&(usize, Vec<ArrayNode>)>,
        group: usize,
        value: usize,
    ) -> Option<ArrayNode> {
        sorted
            .filter(|(column, _)| *column == value)
            .and_then(|(_, totals)| totals.get(group).cloned())
    }
}

/// Groups the records by the row keys and by the column keys, sorted by the keys their sort order
/// names. The output size is checked while grouping, before any value sort, LAMBDA call or the
/// output allocation (SPEC-0006 step 6): the rows first with one column group, then the columns
/// with the real row count, then with the subtotal rows and columns, counted in key order, which
/// bounds them after a value sort (it keeps each parent's groups together; CLAUDE.md directive
/// 5). Returns the height.
fn pivot_groupings(
    pivot: &Pivot,
    cell: CellReferenceIndex,
) -> Result<(Grouping, Grouping, usize), CalcResult> {
    let row_fields = width(&pivot.row_keys);
    let value_columns = width(&pivot.values);
    let extra_rows = header_height(pivot) + usize::from(pivot.row_depth != 0);
    let total_columns = if pivot.col_depth == 0 {
        0
    } else {
        value_columns
    };
    let to_error = |(error, message)| CalcResult::new_error(error, cell, message);
    let min_width = row_fields
        .saturating_add(value_columns)
        .saturating_add(total_columns);
    let row_limit = |groups: usize| check_array_size(groups.saturating_add(extra_rows), min_width);
    let mut rows = Grouping::new(&pivot.row_keys, pivot.records.iter().copied(), row_limit)
        .map_err(to_error)?;
    let height = rows.groups.len().saturating_add(extra_rows);
    let grid_width = |groups: usize| {
        groups
            .saturating_mul(value_columns)
            .saturating_add(row_fields)
            .saturating_add(total_columns)
    };
    let col_limit = |groups: usize| check_array_size(height, grid_width(groups));
    let mut cols = Grouping::new(&pivot.col_keys, pivot.records.iter().copied(), col_limit)
        .map_err(to_error)?;
    rows.sort_by_key_order(&pivot.row_sort, cell);
    cols.sort_by_key_order(&pivot.col_sort, cell);
    let height = rows
        .lines(subtotal_levels(pivot.row_depth), false)
        .len()
        .saturating_add(extra_rows);
    let col_lines = cols.lines(subtotal_levels(pivot.col_depth), false).len();
    if let Some(error) = check_array_size(height, grid_width(col_lines)) {
        return Err(to_error(error));
    }
    Ok((rows, cols, height))
}

/// Whether the headers are shown (`field_headers` 2 or 3).
fn shows_header(pivot: &Pivot) -> bool {
    matches!(pivot.headers, FieldHeaders::Generated | FieldHeaders::Shown)
}

/// The number of header rows: the column field names (shown headers only), one row per column
/// key field, then the row field and value names (shown headers only) (P!M1, P!A1).
fn header_height(pivot: &Pivot) -> usize {
    width(&pivot.col_keys) + 2 * usize::from(shows_header(pivot))
}

/// The header rows: [`column_names_row`], the column keys (blank as 0; a subtotal column shows
/// its keys over `""`; "Total", or "Grand Total" with subtotals, over the total columns, `""`
/// below it: P!A1, P!M97, probes/groupby-subtotals G!A691) and [`value_names_row`].
fn header_rows(pivot: &Pivot, cols: &Grouping, slots: &[Slot]) -> Vec<Vec<ArrayNode>> {
    let mut output = Vec::with_capacity(header_height(pivot));
    let levels = subtotal_levels(pivot.col_depth);
    if shows_header(pivot) {
        output.push(column_names_row(pivot, slots));
    }
    for field in 0..width(&pivot.col_keys) {
        let mut row = vec![empty_text(); width(&pivot.row_keys)];
        row.extend(slots.iter().map(|slot| {
            match *slot {
                Slot::Group { group, .. } => cols
                    .groups
                    .get(group)
                    .and_then(|g| g.keys.get(field))
                    .map_or(ArrayNode::Empty, shown_key),
                Slot::Subtotal { level, start, .. } if field < level => cols
                    .groups
                    .get(start)
                    .and_then(|g| g.keys.get(field))
                    .map_or_else(empty_text, shown_key),
                Slot::Total { .. } if field == 0 => grand_total_label(levels),
                Slot::Subtotal { .. } | Slot::Total { .. } => empty_text(),
            }
        }));
        output.push(row);
    }
    if shows_header(pivot) {
        output.push(value_names_row(pivot, slots));
    }
    output
}

/// The grid columns right of the row keys: the value columns of each column group and subtotal
/// in turn (`lines`), and the total columns after them (`col_total_depth` 1 or more) or before
/// them (negative: probes/pivotby-followup P!A101, P!A121; groupby-subtotals G!A691).
fn slots(lines: &[Line], value_columns: usize, col_depth: isize) -> Vec<Slot> {
    let grid = lines.iter().flat_map(|&line| {
        (0..value_columns).map(move |value| match line {
            Line::Group(group) => Slot::Group { group, value },
            Line::Subtotal { level, start, end } => Slot::Subtotal {
                level,
                start,
                end,
                value,
            },
        })
    });
    let totals = (0..value_columns).map(|value| Slot::Total { value });
    match col_depth {
        0 => grid.collect(),
        depth if depth < 0 => totals.chain(grid).collect(),
        _ => grid.chain(totals).collect(),
    }
}

/// The first header row of shown headers: the column field names, joined with ", " as text, above
/// the first column group, also when the total or a subtotal column comes first; `""` elsewhere
/// (P!M1; probes/pivotby-followup P!A61 "Product, Year", P!A421 "2023" as text, P!A1181;
/// groupby-subtotals G!A871). Generated
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
    let mut row = vec![empty_text(); width(&pivot.row_keys)];
    let mut name = Some(name);
    for slot in slots {
        row.push(match slot {
            Slot::Group { .. } => name.take().unwrap_or_else(empty_text),
            Slot::Subtotal { .. } | Slot::Total { .. } => empty_text(),
        });
    }
    row
}

/// Header cells joined with ", " as text. An error cell makes the name that error (unprobed). A
/// name longer than a cell can hold, 32,767 characters (docs/excel-semantics/coercion.md "Text
/// longer than 32,767 characters"), is `#VALUE!` (unprobed for PIVOTBY).
fn joined_names(names: impl Iterator<Item = ArrayNode>) -> ArrayNode {
    let mut parts = Vec::new();
    let mut units = 0usize;
    for name in names {
        match array_node_to_string(&name) {
            Ok(text) => {
                units = units.saturating_add(text.encode_utf16().count() + 2);
                if units > MAX_TEXT_UNITS + 2 {
                    return ArrayNode::Error(Error::VALUE);
                }
                parts.push(text);
            }
            Err(error) => return ArrayNode::Error(error),
        }
    }
    ArrayNode::String(parts.join(", ").into())
}

/// The last header row of shown headers: the row field names, then the value field name of each
/// grid column, subtotal and total columns included (P!M1, probes/pivotby-followup P!A521,
/// groupby-subtotals G!A841); generated names
/// are "Row Field n" and "Value n" (P!A21, P!A81). The naming rule is GROUPBY's.
fn value_names_row(pivot: &Pivot, slots: &[Slot]) -> Vec<ArrayNode> {
    let mut row = key_names(pivot.headers, &pivot.row_keys);
    row.extend(slots.iter().map(|slot| {
        let (Slot::Group { value, .. } | Slot::Subtotal { value, .. } | Slot::Total { value }) =
            *slot;
        value_name(pivot.headers, &pivot.values, value)
    }));
    row
}

/// The grid's columns and the scratch space of its rows: the column group of each record, one
/// bucket of a row's records per column group, and the merged buckets of a subtotal column.
struct Grid<'a> {
    values: &'a [Vec<ArrayNode>],
    slots: &'a [Slot],
    column_of: Vec<usize>,
    buckets: Vec<Vec<usize>>,
    merged: Vec<usize>,
}
