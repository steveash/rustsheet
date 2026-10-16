//! GROUPBY on the grouping core that PIVOTBY shares (grouping.rs; SPEC-0006 Design steps 1-7,
//! T-1322; `sort_order`, `filter_array` and the argument errors, T-1323; subtotals and an array
//! `field_relationship`, T-1324).
//! Excel: docs/excel-semantics/arrays.md "GROUPBY and PIVOTBY", evidence probes/groupby-pivotby
//! sheet G (excel-graph, T-1276) and its follow-ups (T-1328).

use super::group_functions::{Aggregate, Element, Functions, Relative, ALL_ROWS};
use super::grouping::{
    effective_depth, empty_text, subtotal_levels, Grouping, ParentOrder, SortOrder,
};
use crate::{
    calc_result::CalcResult,
    cast::calc_result_to_array_node,
    expressions::{
        parser::{ArrayNode, Node},
        token::Error,
        types::CellReferenceIndex,
    },
    functions::util::array_node_to_calc_result,
    limits::check_array_size,
    model::Model,
};

/// The GROUPBY arguments as evaluated, before any is checked: kept apart so that an array
/// `field_relationship` can be lifted over them ([`Model::lifted_groupby`]).
struct GroupbyInputs {
    keys: Vec<Vec<ArrayNode>>,
    values: Vec<Vec<ArrayNode>>,
    /// The `function` argument as [`Model::function_elements`] reads it.
    function: Vec<Vec<Element>>,
    /// `field_headers`, `total_depth`, `sort_order` and `filter_array`, `None` when omitted.
    options: Vec<Option<CalcResult>>,
    relationship: Option<CalcResult>,
}

/// The GROUPBY options after the checks ([`Model::groupby_args`]).
struct GroupbyArgs {
    functions: Functions,
    headers: FieldHeaders,
    /// The total depth laid out ([`effective_depth`]): 0 none, ±1 the grand total below or
    /// above, ±n with the subtotals of the first n - 1 key fields.
    depth: isize,
    sort: SortOrder,
    /// How a value sort orders the parents (flat for a table relationship).
    order: ParentOrder,
    /// The record rows kept by the filter, in source order.
    rows: Vec<usize>,
}

/// The `field_relationship` argument of GROUPBY.
#[derive(Clone, Copy, PartialEq)]
enum Relationship {
    /// 0 or omitted.
    Hierarchy,
    /// 1.
    Table,
}

/// The `field_headers` argument of GROUPBY (and PIVOTBY): whether the first row of the fields
/// and values holds headers, and which header row the result shows (G!J1, S1, AB1, A25, J25).
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum FieldHeaders {
    /// Omitted: detected from the values ([`values_have_header`]), never shown.
    Detect,
    /// 0: no headers.
    No,
    /// 1: headers, not shown.
    Hidden,
    /// 2: no headers, generated ones shown ("Row Field 1", "Value 1").
    Generated,
    /// 3: headers, shown.
    Shown,
}

/// Whether the values' first row is a header row when `field_headers` is omitted. Only the first
/// value cell decides (the keys are not looked at): text over rows whose first cells are none of
/// them text is a header, text over all-text rows is data, and text over rows that mix text with
/// other values is `#VALUE!` (`None`); anything but text is data (probes/groupby-pivotby-followup3
/// G!A145-A421, -followup2 G!A341-A521). One row of text alone is data (unprobed).
pub(crate) fn values_have_header(values: &[Vec<ArrayNode>]) -> Option<bool> {
    let is_text = |row: &Vec<ArrayNode>| matches!(row.first(), Some(ArrayNode::String(_)));
    let Some((first, below)) = values.split_first() else {
        return Some(false);
    };
    if !is_text(first) || below.is_empty() {
        return Some(false);
    }
    let text_rows = below.iter().filter(|row| is_text(row)).count();
    if text_rows == below.len() {
        Some(false)
    } else if text_rows == 0 {
        Some(true)
    } else {
        None
    }
}

/// `#VALUE!` for an invalid GROUPBY or PIVOTBY argument.
pub(crate) fn value_error(cell: CellReferenceIndex, message: &str) -> CalcResult {
    CalcResult::new_error(Error::VALUE, cell, message.to_string())
}

/// `#N/IMPL!` for a GROUPBY or PIVOTBY feature a later SPEC-0006 step adds.
pub(crate) fn not_implemented(cell: CellReferenceIndex, feature: &str) -> CalcResult {
    CalcResult::new_error(
        Error::NIMPL,
        cell,
        format!("{feature} are not implemented yet"),
    )
}

/// An optional argument, `None` when it is missing or omitted (`GROUPBY(k,v,f,,0)`).
fn given(arg: &Node) -> Option<&Node> {
    Some(arg).filter(|node| !matches!(node, Node::EmptyArgKind))
}

/// A cell of a 2-D array, blank when outside it (arrays from `eval_to_array` are rectangular, so
/// this only guards against a panic).
pub(crate) fn at(data: &[Vec<ArrayNode>], row: usize, column: usize) -> ArrayNode {
    data.get(row)
        .and_then(|cells| cells.get(column))
        .cloned()
        .unwrap_or(ArrayNode::Empty)
}

impl Model {
    /// The arguments from `first` on, evaluated before any is validated, with a one-cell
    /// reference read as its value (`None` for an omitted one). Excel reports the first error
    /// value among them in argument order, ahead of any invalid argument before or after it:
    /// `GROUPBY(k,v,SUM,,3,NA())` is `#N/A`, `GROUPBY(k,v,SUM,1/0,,NA())` `#DIV/0!` and
    /// `GROUPBY(k,v,5,,,NA())` `#N/A` (probes/groupby-sort-filter-args B61-B78). An error inside an
    /// array is not an error value here: the argument is invalid (#VALUE!, B24, B45).
    pub(crate) fn evaluate_options(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> Result<Vec<Option<CalcResult>>, CalcResult> {
        let mut options = Vec::with_capacity(args.len());
        for arg in args {
            let Some(node) = given(arg) else {
                options.push(None);
                continue;
            };
            let result = match self.evaluate_node_in_context(node, cell) {
                range @ CalcResult::Range { left, right } if left == right => {
                    self.dereference(range, cell)
                }
                other => other,
            };
            if result.is_error() {
                return Err(result);
            }
            options.push(Some(result));
        }
        Ok(options)
    }

    /// The `sort_order` argument over `key_columns` key fields and `value_columns` value columns:
    /// a column index, 1-based over the keys then the values, negative for descending (G!A145,
    /// G!J145, G!S145). It must be a whole number between 1 and the last column, never text,
    /// TRUE or a blank cell (groupby-sort-filter-args B1-B14; G!S217). An array (a row or a
    /// column, B15) names key columns, each at most once, in any order: the priority is the
    /// column order, so `{2,-1}` and `{-1,2}` are Region descending then Product ascending (G!AB145,
    /// probes/groupby-pivotby-followup G!A341-A541). An array may name a value column only as its
    /// one element (`{-3}`, B21); a value column next to a key column or a second value column is
    /// `#VALUE!` (B16-B20, B29-B30), and so is any element that is not a whole number (B23-B25),
    /// `{#N/A}` included (groupby-sort-filter-args2 B11, B13). A one-cell reference to an error
    /// is that error, ahead of other invalid arguments (args2 B16, B18).
    pub(crate) fn sort_order(
        &mut self,
        arg: Option<CalcResult>,
        key_columns: usize,
        value_columns: usize,
        cell: CellReferenceIndex,
    ) -> Result<SortOrder, CalcResult> {
        let invalid = || value_error(cell, "sort_order is invalid");
        let indices: Vec<ArrayNode> = match arg {
            None => return Ok(SortOrder::Keys(Vec::new())),
            Some(CalcResult::Number(n)) => vec![ArrayNode::Number(n)],
            Some(result @ (CalcResult::Array(_) | CalcResult::Range { .. })) => {
                self.result_to_array(result, cell)?.concat()
            }
            Some(_) => return Err(invalid()),
        };
        let columns = key_columns + value_columns;
        let mut ascending = vec![true; key_columns];
        let mut named = vec![false; columns];
        let mut value = None;
        for index in &indices {
            let ArrayNode::Number(n) = index else {
                return Err(invalid());
            };
            let column = n.abs();
            if n.fract() != 0.0 || column < 1.0 || column > columns as f64 {
                return Err(invalid());
            }
            // A whole number in 1..=columns, so the cast is exact.
            let column = column as usize - 1;
            match named.get_mut(column) {
                Some(seen) if !*seen => *seen = true,
                _ => return Err(invalid()),
            }
            if let Some(direction) = ascending.get_mut(column) {
                *direction = *n > 0.0;
            } else {
                value = Some(SortOrder::Value {
                    column: column - key_columns,
                    ascending: *n > 0.0,
                });
            }
        }
        match value {
            None => Ok(SortOrder::Keys(ascending)),
            Some(value) if indices.len() == 1 => Ok(value),
            Some(_) => Err(invalid()),
        }
    }

    /// The source rows `first..height` that `filter_array` keeps (all of them when it is
    /// omitted), in source order. The filter is a column as tall as the fields, or as tall as the
    /// records when `first` rows are headers (groupby-sort-filter-args B56-B59), or one value that
    /// keeps or drops every row (TRUE, 2; FALSE, 0, `{FALSE}`: followup3 G!A25-A85, B55). Each
    /// element counts like a condition: a number is TRUE unless 0, a blank is FALSE, text must be
    /// "TRUE" or "FALSE" in any case (B42, B44, B52; groupby-sort-filter-args2 B6-B7, B10).
    /// Anything else is `#VALUE!`: another height (G!AB217), a row or several columns (B48-B50),
    /// other text, numeric text "1" included, `""`, an error element even on a dropped row
    /// (B43-B46, B53-B54; args2 B8-B9, B31-B35), a 1x1 array holding an error (args2 B12, B14),
    /// and a filter that keeps no record, `#VALUE!` rather than FILTER's `#CALC!` (G!A241,
    /// followup3 G!A1-A109). A one-cell reference to an error is that error (args2 B17).
    pub(crate) fn filter_rows(
        &mut self,
        filter: Option<CalcResult>,
        height: usize,
        first: usize,
        cell: CellReferenceIndex,
    ) -> Result<Vec<usize>, CalcResult> {
        let invalid = |message: &str| value_error(cell, message);
        let Some(filter) = filter else {
            return Ok((first..height).collect());
        };
        let keep: Vec<bool> = match filter {
            result @ (CalcResult::Array(_) | CalcResult::Range { .. }) => {
                let array = self.result_to_array(result, cell)?;
                if array.iter().any(|row| row.len() != 1) {
                    return Err(invalid("filter_array must be one column"));
                }
                // A filter as tall as the fields has elements beside the header rows, which
                // are never read: text, `""` or an error there is ignored
                // (groupby-sort-filter-args2 B1, B3, B5, B29-B30).
                let beside_headers = if array.len() == height { first } else { 0 };
                let mut keep = Vec::with_capacity(array.len());
                for (index, node) in array.iter().flatten().enumerate() {
                    if index < beside_headers {
                        keep.push(false);
                        continue;
                    }
                    // An error element is an Err here too: #VALUE!, not the error (B45).
                    let condition = array_node_to_calc_result(node, cell);
                    match self.cast_to_bool(condition, cell) {
                        Ok(value) => keep.push(value),
                        Err(_) => return Err(invalid("filter_array must hold TRUE or FALSE")),
                    }
                }
                keep
            }
            scalar => vec![self
                .cast_to_bool(scalar, cell)
                .map_err(|_| invalid("filter_array must hold TRUE or FALSE"))?],
        };
        // The element for source row `row`: the one value, or the row's, counted from the top
        // of the fields or of the records.
        let offset = match keep.len() {
            1 => None,
            n if n == height => Some(0),
            n if first > 0 && n + first == height => Some(first),
            _ => return Err(invalid("filter_array must be as tall as the fields")),
        };
        let rows: Vec<usize> = (first..height)
            .filter(|&row| match offset {
                None => keep.first().copied().unwrap_or(false),
                Some(offset) => keep.get(row - offset).copied().unwrap_or(false),
            })
            .collect();
        if rows.is_empty() {
            return Err(invalid("filter_array keeps no rows"));
        }
        Ok(rows)
    }

    /// The `field_relationship` argument: omitted, 0 or 1 (text and booleans count as numbers:
    /// `"1"`, groupby-sort-filter-args B83; FALSE, followup G!A941); anything else is `#VALUE!`
    /// (2: G!J241; 3, -1, 0.5: B79-B81). An array is lifted per element before this
    /// ([`Model::lifted_groupby`]).
    fn relationship(
        &mut self,
        arg: Option<CalcResult>,
        cell: CellReferenceIndex,
    ) -> Result<Relationship, CalcResult> {
        let Some(result) = arg else {
            return Ok(Relationship::Hierarchy);
        };
        match self.cast_to_number(result, cell) {
            Ok(0.0) => Ok(Relationship::Hierarchy),
            Ok(1.0) => Ok(Relationship::Table),
            _ => Err(value_error(cell, "field_relationship must be 0 or 1")),
        }
    }

    /// `=GROUPBY(row_fields, values, function, [field_headers], [total_depth], [sort_order],
    /// [filter_array], [field_relationship])`
    pub(crate) fn fn_groupby(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let inputs = match self.groupby_inputs(args, cell) {
            Ok(inputs) => inputs,
            Err(error) => return error,
        };
        match inputs.relationship.clone() {
            Some(array @ (CalcResult::Array(_) | CalcResult::Range { .. })) => {
                self.lifted_groupby(&inputs, array, cell)
            }
            relationship => match self.groupby_with(&inputs, relationship, cell) {
                Ok(array) => CalcResult::Array(array),
                Err(error) => error,
            },
        }
    }

    /// The evaluated GROUPBY arguments, every error value first, in argument order (B61-B78).
    fn groupby_inputs(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> Result<GroupbyInputs, CalcResult> {
        if !(3..=8).contains(&args.len()) {
            return Err(CalcResult::new_args_number_error(cell));
        }
        let keys = self.eval_to_array(&args[0], cell)?;
        let values = self.eval_to_array(&args[1], cell)?;
        let options = self
            .function_elements(&args[2], cell)
            .and_then(|function| Ok((function, self.evaluate_options(&args[3..], cell)?)));
        let (function, mut options) = match options {
            Ok(options) => options,
            Err(error) => return Err(self.lifted_error(args.get(7), error, cell)),
        };
        options.resize(5, None);
        let relationship = options.pop().flatten();
        Ok(GroupbyInputs {
            keys,
            values,
            function,
            options,
            relationship,
        })
    }

    /// GROUPBY with an array `field_relationship`, lifted element by element as Excel does: each
    /// element gives the top-left cell of GROUPBY with that element, or its error. A filter-shaped
    /// column gives a column of "East" (one per element), a numeric one `#VALUE!` per cell, an
    /// error element that error, and an invalid other argument `#VALUE!` per cell (followup
    /// G!A861-A981, followup2 G!A1-A161, probes/groupby-subtotals G!A511-A631). The result only
    /// depends on whether an element reads as 0, 1 or neither, so GROUPBY runs at most three
    /// times, whatever the size of the array.
    fn lifted_groupby(
        &mut self,
        inputs: &GroupbyInputs,
        relationship: CalcResult,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        let elements = match self.result_to_array(relationship, cell) {
            Ok(elements) => elements,
            Err(error) => return error,
        };
        let mut results: [Option<ArrayNode>; 3] = [None, None, None];
        let mut lifted = Vec::with_capacity(elements.len());
        for row in &elements {
            let mut cells = Vec::with_capacity(row.len());
            for element in row {
                if matches!(element, ArrayNode::Error(_)) {
                    cells.push(element.clone());
                    continue;
                }
                let scalar = array_node_to_calc_result(element, cell);
                let kind = match self.relationship(Some(scalar.clone()), cell) {
                    Ok(Relationship::Hierarchy) => 0,
                    Ok(Relationship::Table) => 1,
                    Err(_) => 2,
                };
                if let Some(Some(value)) = results.get(kind) {
                    cells.push(value.clone());
                    continue;
                }
                let value = match self.groupby_with(inputs, Some(scalar), cell) {
                    Ok(output) => at(&output, 0, 0),
                    Err(error) => calc_result_to_array_node(error),
                };
                if let Some(result) = results.get_mut(kind) {
                    *result = Some(value.clone());
                }
                cells.push(value);
            }
            lifted.push(cells);
        }
        CalcResult::Array(lifted)
    }

    /// The error value of an option before `field_relationship`, lifted over it when it is an
    /// array: that error in every element, since each element's GROUPBY meets it first
    /// (`GROUPBY(k,v,SUM,,,NA(),,{0;1})` is two `#N/A`, probes/groupby-subtotals G!A1351).
    fn lifted_error(
        &mut self,
        relationship: Option<&Node>,
        error: CalcResult,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        let Some(node) = relationship.and_then(given) else {
            return error;
        };
        let array = self.evaluate_node_in_context(node, cell);
        if !array.is_multi_value() {
            return error;
        }
        match self.result_to_array(array, cell) {
            Ok(elements) => {
                let error = calc_result_to_array_node(error);
                let lifted = elements.iter().map(|row| vec![error.clone(); row.len()]);
                CalcResult::Array(lifted.collect())
            }
            Err(_) => error,
        }
    }

    /// GROUPBY with a scalar (or omitted) `field_relationship`.
    fn groupby_with(
        &mut self,
        inputs: &GroupbyInputs,
        relationship: Option<CalcResult>,
        cell: CellReferenceIndex,
    ) -> Result<Vec<Vec<ArrayNode>>, CalcResult> {
        let args = self.groupby_args(inputs, relationship, cell)?;
        self.groupby(inputs, args, cell)
    }

    /// Checks the GROUPBY options in Excel's order (each check's evidence is beside it).
    fn groupby_args(
        &mut self,
        inputs: &GroupbyInputs,
        relationship: Option<CalcResult>,
        cell: CellReferenceIndex,
    ) -> Result<GroupbyArgs, CalcResult> {
        let (keys, values) = (&inputs.keys, &inputs.values);
        let mut options = inputs.options.iter().cloned();
        let field_headers = options.next().flatten();
        let total_depth = options.next().flatten();
        let sort_order = options.next().flatten();
        let filter = options.next().flatten();

        let mut functions = self.function_argument(&inputs.function, "GROUPBY", cell)?;
        let headers = self.field_headers(field_headers, cell)?;
        let key_columns = width(keys);
        let depth_given = total_depth.is_some();
        let total_depth = self.total_depth(total_depth, key_columns, cell)?;
        // Values of another height than the keys are #VALUE! (G!AB193, followup G!A1-A41).
        if keys.len() != values.len() || key_columns == 0 {
            return Err(value_error(
                cell,
                "row_fields and values must have the same height",
            ));
        }
        functions.pair_with(width(values), cell)?;
        // A sort_order past the keys names an output value column (one per function of a row of
        // functions: functions2 G!A221; the first function's of a column of them: G!A201, G!A261).
        let sort = self.sort_order(sort_order, key_columns, functions.shape().columns, cell)?;
        let header_row = match headers {
            FieldHeaders::Detect => values_have_header(values).ok_or_else(|| {
                value_error(cell, "GROUPBY cannot tell whether there are headers")
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
        if first >= keys.len() {
            // A header row and no records: probes/groupby-lambda-forms G!S193 (ERROR.TYPE 3).
            return Err(value_error(cell, "GROUPBY has no records"));
        }
        let rows = self.filter_rows(filter, keys.len(), first, cell)?;
        let relationship = self.relationship(relationship, cell)?;
        // A table relationship cannot have subtotals (followup G!A181, G!A261: ERROR.TYPE 3).
        if relationship == Relationship::Table && subtotal_levels(total_depth) > 0 {
            return Err(value_error(cell, "field_relationship 1 has no subtotals"));
        }
        let (depth, order) = match relationship {
            // A table keeps the hierarchy's layout sorted by keys (followup G!A221, G!A241), and
            // sorted by a value it orders every group by its own value, with no subtotals even
            // when the depth is omitted (probes/groupby-subtotals G!A391-A481).
            Relationship::Table => (total_depth, ParentOrder::Flat),
            Relationship::Hierarchy => (
                effective_depth(total_depth, depth_given, &sort, key_columns),
                ParentOrder::ByValue,
            ),
        };
        Ok(GroupbyArgs {
            functions,
            headers,
            depth,
            sort,
            order,
            rows,
        })
    }

    /// GROUPBY's output: the header row, the total row on top (negative depth), the groups with
    /// their subtotals in key or value order, and the total row at the bottom. The headers and the
    /// total stay in place whatever the sort (B38-B39).
    fn groupby(
        &mut self,
        inputs: &GroupbyInputs,
        mut args: GroupbyArgs,
        cell: CellReferenceIndex,
    ) -> Result<Vec<Vec<ArrayNode>>, CalcResult> {
        let (keys, values) = (&inputs.keys, &inputs.values);
        let key_columns = width(keys);
        let shape = args.functions.shape();
        let per_line = shape.rows_per_group;
        let columns = key_columns + usize::from(shape.label_column) + shape.columns;
        let shows_header = matches!(args.headers, FieldHeaders::Generated | FieldHeaders::Shown);
        let extra_rows = usize::from(shape.label_row)
            + usize::from(shows_header)
            + usize::from(args.depth != 0) * per_line;
        let tall = |lines: usize| lines.saturating_mul(per_line).saturating_add(extra_rows);
        let to_error = |(error, message)| CalcResult::new_error(error, cell, message);
        // The output size is checked while grouping, before the sort, any LAMBDA call or the
        // output allocation (SPEC-0006 step 6).
        let limit = |groups: usize| check_array_size(tall(groups), columns);
        let mut grouping =
            Grouping::new(keys, args.rows.iter().copied(), limit).map_err(to_error)?;
        // The subtotal rows count too, still before any call (CLAUDE.md directive 5): counted in
        // key order, they bound the subtotals after a value sort, which keeps each parent's
        // groups together.
        let (levels, before) = (subtotal_levels(args.depth), args.depth < 0);
        grouping.sort_by_key_order(&args.sort, cell);
        let mut lines = grouping.lines(levels, before);
        if let Some(error) = check_array_size(tall(lines.len()), columns) {
            return Err(to_error(error));
        }

        // A two-argument function gets all (kept) rows as its relative set: PERCENTOF is each
        // group's share of the total, and 1 in the total row (G!S97, functions G!A301-A321).
        let functions = &mut args.functions;
        let all = Relative {
            id: Some(ALL_ROWS),
            rows: &args.rows,
        };
        // A value sort keeps the values it sorted by, groups' and subtotals', so that column is
        // not computed twice.
        let sorted = match args.sort {
            SortOrder::Keys(_) => None,
            SortOrder::Value { column, ascending } => {
                let pair = functions.pair_at(0, column);
                let by =
                    grouping.sort_by_aggregate(ascending, args.order, cell, |group_rows, _| {
                        let at = Aggregate {
                            rows: group_rows,
                            relative: all,
                        };
                        self.aggregate(functions, pair, values, at, cell)
                    });
                let checked = lines.len();
                lines = grouping.lines(levels, before);
                debug_assert!(lines.len() <= checked, "a value sort added subtotals");
                Some((column, by))
            }
        };
        let mut output = Vec::with_capacity(tall(lines.len()));
        if shape.label_row {
            let mut row = vec![empty_text(); key_columns];
            row.extend(functions.label_cells(0..shape.columns));
            output.push(row);
        }
        if shows_header {
            output.push(header_cells(args.headers, keys, values, functions));
        }
        let mut total = (args.depth != 0).then(|| {
            let label = total_label_cells(key_columns, levels);
            self.line_row(functions, values, label, (&args.rows, all), None, cell)
        });
        if before {
            output.extend(total.take().into_iter().flatten());
        }
        for line in lines {
            // A subtotal is the function of all its groups' rows (G!J49, G!A73).
            let Some((label, rows)) = grouping.line_parts(line, key_columns) else {
                continue;
            };
            let known = sorted
                .as_ref()
                .and_then(|(column, by)| Some((*column, by.of_line(line)?)));
            let row = self.line_row(functions, values, label, (&rows, all), known, cell);
            output.extend(row);
        }
        output.extend(total.into_iter().flatten());
        if let Some(error) = functions.refused() {
            return Err(error);
        }
        Ok(output)
    }

    /// The `field_headers` argument: omitted, or 0 to 3; anything else is `#VALUE!` (4: G!A217),
    /// a fraction too (2.5: probes/groupby-lambda-forms G!S241).
    pub(crate) fn field_headers(
        &mut self,
        arg: Option<CalcResult>,
        cell: CellReferenceIndex,
    ) -> Result<FieldHeaders, CalcResult> {
        let Some(result) = arg else {
            return Ok(FieldHeaders::Detect);
        };
        match self.cast_to_number(result, cell)? {
            0.0 => Ok(FieldHeaders::No),
            1.0 => Ok(FieldHeaders::Hidden),
            2.0 => Ok(FieldHeaders::Generated),
            3.0 => Ok(FieldHeaders::Shown),
            _ => Err(value_error(cell, "field_headers must be 0, 1, 2 or 3")),
        }
    }

    /// The `total_depth` argument: omitted is 1, a grand total at the bottom; 0 none; -1 the
    /// grand total on top (G!A49, G!AB49); n adds the subtotals of the first n - 1 key fields
    /// below their groups, -n above them (G!J49, G!S49; probes/groupby-subtotals G!A1-A91). It
    /// may name at most as many levels as there are key fields: 3 with two keys and ±2 with one
    /// are `#VALUE!` (probes/groupby-pivotby-followup G!A101-A161), 3 with three keys is not
    /// (groupby-subtotals G!A61). A fraction is `#VALUE!` (0.5: probes/groupby-lambda-forms G!J241).
    pub(crate) fn total_depth(
        &mut self,
        arg: Option<CalcResult>,
        key_columns: usize,
        cell: CellReferenceIndex,
    ) -> Result<isize, CalcResult> {
        let Some(result) = arg else {
            return Ok(1);
        };
        let depth = self.cast_to_number(result, cell)?;
        if depth.fract() != 0.0 || depth.abs() > key_columns.max(1) as f64 {
            return Err(value_error(cell, "total_depth is out of range"));
        }
        // A whole number no larger than the key fields, so the cast is exact.
        Ok(depth as isize)
    }

    /// The GROUPBY rows of one line: `label` (the key cells), then the function of each output
    /// value column over `rows` (with `relative` for two-argument functions), except for the
    /// column `known` names, whose value a value sort already computed; one row per function when
    /// the functions are laid out as rows (G!AB121). The total row is "Total" ("Grand Total" with
    /// subtotals), `""` under the other key fields (empty text, not blank cells: G!AC33), then the
    /// function of every value of each column in source order, not of the group results
    /// (AVERAGE's total is 13.05 = 130.5/10, G!S73).
    fn line_row(
        &mut self,
        functions: &mut Functions,
        values: &[Vec<ArrayNode>],
        label: Vec<ArrayNode>,
        (rows, relative): (&[usize], Relative<'_>),
        known: Option<(usize, &ArrayNode)>,
        cell: CellReferenceIndex,
    ) -> Vec<Vec<ArrayNode>> {
        let at = Aggregate { rows, relative };
        let reuse = known.map(|(column, value)| (column, Some(value)));
        self.value_rows(functions, values, at, reuse, cell)
            .into_iter()
            .map(|cells| {
                let mut row = label.clone();
                row.extend(cells);
                row
            })
            .collect()
    }
}

/// The header row GROUPBY shows: the key field names, `""` over a label column, then the value
/// field name of each output column ([`key_names`], [`value_name`]; G!J25, G!A25; functions
/// G!A61, G!A41).
fn header_cells(
    headers: FieldHeaders,
    keys: &[Vec<ArrayNode>],
    values: &[Vec<ArrayNode>],
    functions: &Functions,
) -> Vec<ArrayNode> {
    let shape = functions.shape();
    let mut row = key_names(headers, keys);
    if shape.label_column {
        row.push(empty_text());
    }
    row.extend(
        (0..shape.columns)
            .map(|column| value_name(headers, values, functions.value_column(column))),
    );
    row
}

/// The number of columns of a (rectangular) array.
pub(crate) fn width(data: &[Vec<ArrayNode>]) -> usize {
    data.first().map_or(0, Vec::len)
}

/// The "Total" label of a total row or column.
pub(crate) fn total_label() -> ArrayNode {
    ArrayNode::String("Total".into())
}

/// The label of a grand total: "Total", or "Grand Total" when there are subtotal `levels`
/// (G!J49; PIVOTBY's rows and columns each by their own depth, P!A97, P!M97).
pub(crate) fn grand_total_label(levels: usize) -> ArrayNode {
    if levels == 0 {
        total_label()
    } else {
        ArrayNode::String("Grand Total".into())
    }
}

/// The key cells of a total row: its [`grand_total_label`], then `""` under the other key fields
/// (G!AC33, P!A97).
pub(crate) fn total_label_cells(key_columns: usize, levels: usize) -> Vec<ArrayNode> {
    let mut cells = vec![grand_total_label(levels)];
    cells.extend((1..key_columns).map(|_| empty_text()));
    cells
}

/// The names of the key fields in a shown header: their own first row (3, G!J25), or
/// "Row Field n" (2, G!A25; probes/groupby-lambda-forms, probes/pivotby-followup P!A81).
pub(crate) fn key_names(headers: FieldHeaders, keys: &[Vec<ArrayNode>]) -> Vec<ArrayNode> {
    (0..width(keys))
        .map(|column| {
            if headers == FieldHeaders::Shown {
                at(keys, 0, column)
            } else {
                ArrayNode::String(format!("Row Field {}", column + 1).into())
            }
        })
        .collect()
}

/// The name of value column `column` in a shown header: its own first row (3), or "Value n" (2).
pub(crate) fn value_name(
    headers: FieldHeaders,
    values: &[Vec<ArrayNode>],
    column: usize,
) -> ArrayNode {
    if headers == FieldHeaders::Shown {
        at(values, 0, column)
    } else {
        ArrayNode::String(format!("Value {}", column + 1).into())
    }
}
