//! PIVOTBY on the grouping core GROUPBY shares (SPEC-0006 step 5, T-1326; subtotals, T-1324): the
//! row groups and the column groups are two [`Grouping`]s of the same records, laid out as a grid,
//! each axis with its own subtotals. Excel:
//! docs/excel-semantics/arrays.md "GROUPBY and PIVOTBY", evidence probes/groupby-pivotby sheet P,
//! its follow-ups (T-1328) and probes/pivotby-followup (excel-graph, 2026-10-16, T-1326).

use super::group_functions::{Aggregate, Functions, Shape};
use super::groupby::{
    grand_total_label, key_names, not_implemented, total_label_cells, value_name,
    values_have_header, width, FieldHeaders,
};
use super::grouping::{
    at, effective_depth, empty_text, shown_key, subtotal_levels, value_error, Aggregates, Axis,
    Grouping, Line, ParentOrder, Relative, SortOrder,
};
use super::relative_to::{Item, RelativeSets, RelativeTo};
use crate::{
    calc_result::CalcResult,
    cast::{array_node_to_string, MAX_TEXT_UNITS},
    expressions::{
        parser::{ArrayNode, Node},
        token::Error,
        types::CellReferenceIndex,
    },
    limits::check_array_size,
    model::Model,
};

/// One column of the PIVOTBY grid right of the row keys (and the label column): output value
/// column `value` ([`Functions::shape`]) of column group `group`, of the subtotal of the column
/// groups `start..end` (which share their first `level` keys), or the total of it.
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
    /// The output shape of the functions.
    shape: Shape,
    /// The `relative_to` argument, or its elements when it is an array (lifted).
    relative_to: RelativeTo,
    lifted: Option<Vec<Vec<ArrayNode>>>,
}

/// The relative set of a call that has none: PIVOTBY's one-argument functions.
const NO_RELATIVE: Relative<'static> = Relative {
    id: None,
    rows: &[],
};

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
        let (mut pivot, mut functions) = self.pivotby_args(args, cell)?;
        let (mut rows, mut cols, height) = pivot_groupings(&pivot, cell)?;
        if let Some(elements) = pivot.lifted.take() {
            return Ok(self.lifted_relative_to(elements, cell));
        }
        let functions = &mut functions;
        // Rows sort by a value only when there is a Total column: with col_total_depth 0 they keep
        // the key order (probes/pivotby-followup P!A781, P!A1221; with it, P!M121, P!A801).
        // Columns sort by a value with or without the Total row (P!A821, P!A841).
        let key_order = SortOrder::Keys(Vec::new());
        let row_sort = match pivot.row_sort {
            SortOrder::Value { .. } if pivot.col_depth == 0 => &key_order,
            ref order => order,
        };
        let row_totals = self.sort_pivot_groups(
            functions,
            &mut rows,
            &pivot,
            (row_sort, ParentOrder::ByValue, Axis::Rows),
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
        let col_totals = self.sort_pivot_groups(
            functions,
            &mut cols,
            &pivot,
            (col_sort, col_parents, Axis::Columns),
            cell,
        );

        let col_lines = cols.lines(col_levels, pivot.col_depth < 0);
        let row_lines = rows.lines(subtotal_levels(pivot.row_depth), pivot.row_depth < 0);
        // The sets name groups by index, so they are built once the groups are sorted.
        let sets = if functions.needs_relative() {
            let lines = (&row_lines[..], &col_lines[..]);
            let sets = RelativeSets::new(
                pivot.relative_to,
                (&rows, &cols),
                lines,
                &pivot.records,
                cell,
            )?;
            Some(sets)
        } else {
            None
        };
        let slots = slots(&col_lines, pivot.shape.columns, pivot.col_depth);
        let mut output = Vec::with_capacity(height);
        output.extend(header_rows(&pivot, functions, &cols, &slots));
        let totals = Totals {
            rows: row_totals,
            cols: col_totals,
        };
        let lines = (&rows, &cols, &row_lines[..]);
        let grid = self.grid_rows(
            functions,
            &pivot,
            lines,
            (&slots, sets.as_ref()),
            &totals,
            cell,
        );
        output.extend(grid);
        if let Some(error) = functions.refused() {
            return Err(error);
        }
        Ok(output)
    }

    /// The row groups' rows with their subtotals, and the Total row (before them for
    /// `row_total_depth` -1, after them for 1, none for 0: P!A1, P!Y1, probes/pivotby-followup
    /// P!A101; "Grand Total" with subtotals, P!A97, probes/groupby-subtotals G!A661), each
    /// repeated per function when the functions are laid out as rows (functions2 P!A21, P!A61).
    fn grid_rows(
        &mut self,
        functions: &mut Functions,
        pivot: &Pivot,
        (rows, cols, lines): (&Grouping, &Grouping, &[Line]),
        (slots, sets): (&[Slot], Option<&RelativeSets>),
        totals: &Totals,
        cell: CellReferenceIndex,
    ) -> Vec<Vec<ArrayNode>> {
        let levels = subtotal_levels(pivot.row_depth);
        let row_fields = width(&pivot.row_keys);
        let column_of = cols.group_of(pivot.values.len());
        let mut grid = Grid {
            values: &pivot.values,
            slots,
            column_of,
            buckets: vec![Vec::new(); cols.groups.len()],
            merged: Vec::new(),
            sets,
            row: None,
        };
        let mut output = Vec::with_capacity((lines.len() + 1) * pivot.shape.rows_per_group);
        let blank = functions.blank_totals(width(&pivot.values));
        let mut total = (pivot.row_depth != 0).then(|| {
            let label = total_label_cells(row_fields, levels);
            let reuse = |slot| totals.total_row_cell(slot);
            self.grid_row(functions, &mut grid, label, &pivot.records, reuse, cell)
        });
        if pivot.row_depth < 0 {
            output.extend(total.take().into_iter().flatten());
        }
        for &line in lines {
            // A subtotal row is the function of all its groups' records (P!A97).
            let Some((label, records)) = rows.line_parts(line, row_fields) else {
                continue;
            };
            grid.row = Some(line);
            let reuse = |slot| totals.line_total_cell(line, slot, blank);
            output.extend(self.grid_row(functions, &mut grid, label, &records, reuse, cell));
        }
        output.extend(total.into_iter().flatten());
        output
    }

    /// One grid line's rows: `label` (the row keys) and the label of the function (functions as
    /// rows, one row each), then the cell of each slot over `records` (the line's records, in
    /// source order), or, in the first row, the value `reuse` has for it (an aggregate a value
    /// sort computed).
    fn grid_row(
        &mut self,
        functions: &mut Functions,
        grid: &mut Grid<'_>,
        label: Vec<ArrayNode>,
        records: &[usize],
        reuse: impl Fn(Slot) -> Option<ArrayNode>,
        cell: CellReferenceIndex,
    ) -> Vec<Vec<ArrayNode>> {
        for bucket in &mut grid.buckets {
            bucket.clear();
        }
        for &record in records {
            let column = grid.column_of.get(record).copied().unwrap_or(usize::MAX);
            if let Some(bucket) = grid.buckets.get_mut(column) {
                bucket.push(record);
            }
        }
        let shape = functions.shape();
        let mut output = Vec::with_capacity(shape.rows_per_group);
        for sub_row in 0..shape.rows_per_group {
            let mut row = label.clone();
            if shape.label_column {
                row.push(functions.label(functions.pair_at(sub_row, 0)));
            }
            for &slot in grid.slots {
                if let Some(value) = reuse(slot).filter(|_| sub_row == 0) {
                    row.push(value);
                    continue;
                }
                let cell_rows: &[usize] = match slot {
                    Slot::Group { group, .. } => {
                        grid.buckets.get(group).map_or(&[][..], Vec::as_slice)
                    }
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
                let relative = grid
                    .sets
                    .map_or(NO_RELATIVE, |sets| sets.of_cell(grid.row, slot_item(slot)));
                let at = Aggregate {
                    rows: cell_rows,
                    relative,
                };
                row.push(self.pivot_cell(functions, grid.values, slot, sub_row, at, cell));
            }
            output.push(row);
        }
        output
    }

    /// Evaluates and checks the PIVOTBY arguments. Column subtotals over several value columns are
    /// `#N/IMPL!`, never a wrong value.
    fn pivotby_args(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> Result<(Pivot, Functions), CalcResult> {
        if !(4..=11).contains(&args.len()) {
            return Err(CalcResult::new_args_number_error(cell));
        }
        let row_keys = self.eval_to_array(&args[0], cell)?;
        let col_keys = self.eval_to_array(&args[1], cell)?;
        let values = self.eval_to_array(&args[2], cell)?;
        // Every error value first, in argument order, then the validation, as in GROUPBY: an error
        // option beats an invalid one before or after it and a non-function
        // (probes/pivotby-followup P!A1401-A1461, P!A1261, P!A1281).
        let function = self.function_elements(&args[3], cell)?;
        let mut options = self.evaluate_options(&args[4..], cell)?.into_iter();
        let field_headers = options.next().flatten();
        let row_depth = options.next().flatten();
        let row_sort = options.next().flatten();
        let col_depth = options.next().flatten();
        let col_sort = options.next().flatten();
        let filter = options.next().flatten();
        let relative_to = options.next().flatten();
        let row_depth_given = row_depth.is_some();
        let col_depth_given = col_depth.is_some();

        let mut functions = self.function_argument(&function, "PIVOTBY", cell)?;
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
        functions.pair_with(value_columns, cell)?;
        let shape = functions.shape();
        // GROUPBY's sort_order rules hold on both axes (P!A141, P!A161, P!A721, P!A1001, P!A1321-A1381),
        // over the output value columns (functions2 P!A181-A221).
        let row_sort = self.sort_order(row_sort, row_fields, shape.columns, cell)?;
        let col_sort = self.sort_order(col_sort, col_fields, shape.columns, cell)?;
        let header_row = match headers {
            // Only the values decide, as in GROUPBY (probes/pivotby-followup P!A1, P!A1021).
            FieldHeaders::Detect => values_have_header(&values).ok_or_else(|| {
                value_error(cell, "PIVOTBY cannot tell whether there are headers")
            })?,
            FieldHeaders::No | FieldHeaders::Generated => false,
            FieldHeaders::Hidden | FieldHeaders::Shown => true,
        };
        let headers = match headers {
            FieldHeaders::Detect if header_row && functions.shows_detected_headers() => {
                FieldHeaders::Shown
            }
            other => other,
        };
        let first = usize::from(header_row);
        if first >= height {
            return Err(value_error(cell, "PIVOTBY has no records"));
        }
        // GROUPBY's filter rules hold too: another height, errors and text are #VALUE!, a header
        // row may be left out of the filter, and keeping no record is #VALUE! (P!A281,
        // P!A1481-A1601, probes/groupby-pivotby-followup G!A881, -followup3 G!A133).
        let records = self.filter_rows(filter, height, first, cell)?;
        let (relative_to, lifted) = self.relative_to_arg(relative_to, cell)?;
        let row_depth = effective_depth(row_depth, row_depth_given, &row_sort, row_fields);
        let col_depth = effective_depth(col_depth, col_depth_given, &col_sort, col_fields);
        // TODO(T-1700): column subtotals over several value columns. Excel shows most of their
        // cells as "", like the Total columns, but fills a few with other columns' subtotals
        // (probes/groupby-subtotals G!A781), a layout not yet understood.
        if subtotal_levels(col_depth) > 0 && (value_columns > 1 || shape.columns > 1) {
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
            shape,
            relative_to,
            lifted,
        };
        Ok((pivot, functions))
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
        functions: &mut Functions,
        grouping: &mut Grouping,
        pivot: &Pivot,
        (order, parents, axis): (&SortOrder, ParentOrder, Axis),
        cell: CellReferenceIndex,
    ) -> Option<(usize, Aggregates)> {
        let values = &pivot.values;
        let SortOrder::Value { column, ascending } = *order else {
            return None;
        };
        let pair = functions.pair_at(0, column);
        let totals = grouping.sort_by_aggregate(ascending, (parents, axis), cell, |rows, up| {
            let relative = pivot.relative_to.of_sort(axis, rows, up, &pivot.records);
            let at = Aggregate { rows, relative };
            self.aggregate(functions, pair, values, at, cell)
        });
        Some((column, totals))
    }

    /// One grid cell in row `sub_row` of its line: `""` when no record falls in it (P!A1,
    /// East/2025), the function of the records otherwise (0 when they sum to 0, P!A97). A total
    /// column of one function over several value columns is `""` in every row, the Total row too
    /// (P!Y97, probes/groupby-pivotby-followup G!A281).
    fn pivot_cell(
        &mut self,
        functions: &mut Functions,
        values: &[Vec<ArrayNode>],
        slot: Slot,
        sub_row: usize,
        at: Aggregate<'_>,
        cell: CellReferenceIndex,
    ) -> ArrayNode {
        match slot {
            Slot::Total { .. } if functions.blank_totals(width(values)) => empty_text(),
            _ if at.rows.is_empty() => empty_text(),
            Slot::Group { value, .. } | Slot::Subtotal { value, .. } | Slot::Total { value } => {
                let pair = functions.pair_at(sub_row, value);
                self.aggregate(functions, pair, values, at, cell)
            }
        }
    }
}

/// The aggregates value sorts computed, reused instead of calling the function again (GROUPBY
/// does the same): per row group and row subtotal for the Total column, per column group and
/// column subtotal for the Total row.
struct Totals {
    rows: Option<(usize, Aggregates)>,
    cols: Option<(usize, Aggregates)>,
}

impl Totals {
    /// The Total cell of a row group or row subtotal `line`: its aggregate, unless the Total
    /// columns are `blank` (one function over several value columns, P!Y97).
    fn line_total_cell(&self, line: Line, slot: Slot, blank: bool) -> Option<ArrayNode> {
        match slot {
            Slot::Total { value } if !blank => Self::get(self.rows.as_ref(), line, value),
            _ => None,
        }
    }

    /// A Total row cell under a column group or column subtotal: its aggregate.
    fn total_row_cell(&self, slot: Slot) -> Option<ArrayNode> {
        let (line, value) = match slot {
            Slot::Group { group, value } => (Line::Group(group), value),
            Slot::Subtotal {
                level,
                start,
                end,
                value,
            } => (Line::Subtotal { level, start, end }, value),
            Slot::Total { .. } => return None,
        };
        Self::get(self.cols.as_ref(), line, value)
    }

    fn get(sorted: Option<&(usize, Aggregates)>, line: Line, value: usize) -> Option<ArrayNode> {
        sorted
            .filter(|(column, _)| *column == value)
            .and_then(|(_, aggregates)| aggregates.of_line(line).cloned())
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
    // The keys and the label column, left of the grid.
    let row_fields = width(&pivot.row_keys) + usize::from(pivot.shape.label_column);
    let value_columns = pivot.shape.columns;
    let per_line = pivot.shape.rows_per_group;
    let extra_rows = header_height(pivot) + usize::from(pivot.row_depth != 0) * per_line;
    let tall = |lines: usize| lines.saturating_mul(per_line).saturating_add(extra_rows);
    let total_columns = if pivot.col_depth == 0 {
        0
    } else {
        value_columns
    };
    let to_error = |(error, message)| CalcResult::new_error(error, cell, message);
    let min_width = row_fields
        .saturating_add(value_columns)
        .saturating_add(total_columns);
    let row_limit = |groups: usize| check_array_size(tall(groups), min_width);
    let mut rows = Grouping::new(&pivot.row_keys, pivot.records.iter().copied(), row_limit)
        .map_err(to_error)?;
    let height = tall(rows.groups.len());
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
    let height = tall(rows.lines(subtotal_levels(pivot.row_depth), false).len());
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
/// key field, the function labels (functions laid out as columns), then the row field and value
/// names (shown headers only) (P!M1, P!A1, P!A121).
fn header_height(pivot: &Pivot) -> usize {
    width(&pivot.col_keys)
        + usize::from(pivot.shape.label_row)
        + 2 * usize::from(shows_header(pivot))
}

/// The `""` cells of a header row left of the grid: under the row keys and the label column.
fn header_corner(pivot: &Pivot) -> Vec<ArrayNode> {
    vec![empty_text(); width(&pivot.row_keys) + usize::from(pivot.shape.label_column)]
}

/// The header rows: [`column_names_row`], the column keys (blank as 0; a subtotal column shows
/// its keys over `""`; "Total", or "Grand Total" with subtotals, over the total columns, `""`
/// below it: P!A1, P!M97, probes/groupby-subtotals G!A691), the function labels of each column
/// (P!A121; with shown headers after the keys, functions P!A621) and [`value_names_row`].
fn header_rows(
    pivot: &Pivot,
    functions: &Functions,
    cols: &Grouping,
    slots: &[Slot],
) -> Vec<Vec<ArrayNode>> {
    let mut output = Vec::with_capacity(header_height(pivot));
    let levels = subtotal_levels(pivot.col_depth);
    if shows_header(pivot) {
        output.push(column_names_row(pivot, slots));
    }
    for field in 0..width(&pivot.col_keys) {
        let mut row = header_corner(pivot);
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
    if pivot.shape.label_row {
        let mut row = header_corner(pivot);
        row.extend(functions.label_cells(slots.iter().map(|slot| slot_value(*slot))));
        output.push(row);
    }
    if shows_header(pivot) {
        output.push(value_names_row(pivot, functions, slots));
    }
    output
}

/// The output value column of a slot.
fn slot_value(slot: Slot) -> usize {
    let (Slot::Group { value, .. } | Slot::Subtotal { value, .. } | Slot::Total { value }) = slot;
    value
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
    let mut row = header_corner(pivot);
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
/// are "Row Field n" and "Value n" (P!A21, P!A81), `""` over a label column (functions2 P!A41).
/// The naming rule is GROUPBY's.
fn value_names_row(pivot: &Pivot, functions: &Functions, slots: &[Slot]) -> Vec<ArrayNode> {
    let mut row = key_names(pivot.headers, &pivot.row_keys);
    if pivot.shape.label_column {
        row.push(empty_text());
    }
    row.extend(slots.iter().map(|slot| {
        let column = functions.value_column(slot_value(*slot));
        value_name(pivot.headers, &pivot.values, column)
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
    /// The relative sets, when a function takes them, and the row item being laid out (`None`
    /// for the Total row).
    sets: Option<&'a RelativeSets>,
    row: Item,
}

/// The column item of a slot ([`RelativeSets::of_cell`]).
fn slot_item(slot: Slot) -> Item {
    match slot {
        Slot::Group { group, .. } => Some(Line::Group(group)),
        Slot::Subtotal {
            level, start, end, ..
        } => Some(Line::Subtotal { level, start, end }),
        Slot::Total { .. } => None,
    }
}
