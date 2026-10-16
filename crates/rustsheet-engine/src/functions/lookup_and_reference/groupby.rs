//! GROUPBY on the grouping core that PIVOTBY shares (SPEC-0006 Design steps 1-7, T-1322; `sort_order`,
//! `filter_array` and the argument errors, T-1323).
//! Excel: docs/excel-semantics/arrays.md "GROUPBY and PIVOTBY", evidence probes/groupby-pivotby
//! sheet G (excel-graph, T-1276) and its follow-ups (T-1328).

use std::cmp::Ordering;
use std::collections::hash_map::Entry;
use std::collections::HashMap;

use crate::{
    calc_result::CalcResult,
    cast::calc_result_to_array_node,
    expressions::{
        parser::{ArrayNode, Node},
        token::Error,
        types::CellReferenceIndex,
    },
    functions::logical::{as_lambda, LambdaCaller},
    functions::spill_functions::{cell_key, sort_key_cmp, CellKey},
    functions::util::{array_node_to_calc_result, collation::cmp_text},
    functions::Function,
    limits::check_array_size,
    model::Model,
};

/// One group: its key tuple as first met in the source (the first spelling, `East` for `East`
/// and `east`, G!A1) and the indices of its source rows, in source order.
pub(crate) struct Group {
    pub(crate) keys: Vec<ArrayNode>,
    pub(crate) rows: Vec<usize>,
}

/// Rows grouped by a key tuple: the unit GROUPBY lays out as rows and PIVOTBY as a grid.
pub(crate) struct Grouping {
    pub(crate) groups: Vec<Group>,
}

impl Grouping {
    /// Groups `rows` (indices into `keys`, one key tuple per row) like UNIQUE: text is
    /// case-insensitive and numbers compare exactly ([`group_key`]). The key is built once per row
    /// and looked up in a hash map, so grouping is O(rows x key fields). The groups are in order
    /// of first appearance until [`Grouping::sort_by_keys`]. `limit(n)` is the error `n` groups
    /// would give (the output size check, SPEC-0006 step 6); grouping stops at the first group
    /// that does not fit, so no more groups are built than the output can hold.
    pub(crate) fn new(
        keys: &[Vec<ArrayNode>],
        rows: impl IntoIterator<Item = usize>,
        limit: impl Fn(usize) -> Option<(Error, String)>,
    ) -> Result<Self, (Error, String)> {
        let mut index: HashMap<Vec<CellKey>, usize> = HashMap::new();
        let mut groups: Vec<Group> = Vec::new();
        for row in rows {
            let Some(tuple) = keys.get(row) else {
                continue;
            };
            match index.entry(tuple.iter().map(group_key).collect()) {
                Entry::Occupied(entry) => {
                    if let Some(group) = groups.get_mut(*entry.get()) {
                        group.rows.push(row);
                    }
                }
                Entry::Vacant(entry) => {
                    if let Some(error) = limit(groups.len() + 1) {
                        return Err(error);
                    }
                    entry.insert(groups.len());
                    groups.push(Group {
                        keys: tuple.iter().map(positive_zero).collect(),
                        rows: vec![row],
                    });
                }
            }
        }
        Ok(Grouping { groups })
    }

    /// Sorts the groups by their keys, field by field, `ascending[i]` for field `i` (ascending
    /// when missing). Mixed types sort numbers < text < FALSE < TRUE (G!A193) and the blank key
    /// sorts last in both directions (G!A1, G!A145): that is `sort_key_cmp`. Numbers compare
    /// exactly, not to 15 digits: 0.3 sorts before 0.1+0.2 (probes/groupby-pivotby-followup
    /// G!A761), matching the exact `cell_key` grouping. (Errors against text, booleans and the
    /// blank key: unprobed, SPEC-0006.)
    pub(crate) fn sort_by_keys(&mut self, ascending: &[bool], cell: CellReferenceIndex) {
        self.groups.sort_by(|a, b| {
            a.keys
                .iter()
                .zip(&b.keys)
                .enumerate()
                .map(|(field, (x, y))| {
                    key_cmp(x, y, ascending.get(field).copied().unwrap_or(true), cell)
                })
                .find(|ordering| ordering.is_ne())
                .unwrap_or(Ordering::Equal)
        });
    }

    /// Sorts the groups by an aggregate (GROUPBY's `sort_order` naming a value column; PIVOTBY's
    /// sort by a total), and returns each group's aggregate in the new order so the caller shows
    /// the values it sorted by instead of calling the function again. `aggregate` gets source rows
    /// in source order.
    ///
    /// With several key fields the sort is hierarchical: the parents (key prefixes) are ordered
    /// by the aggregate of all their rows, and the children by their own value inside each parent
    /// (probes/groupby-pivotby-followup2 G!A261; groupby-sort-filter-args B21-B22 descending).
    /// Without `parents_by_value` the parents keep their key order and only the last level sorts
    /// by value, as PIVOTBY's columns do (probes/pivotby-followup P!A1141, P!A1161).
    /// Equal values keep the ascending key order in both directions (groupby-sort-filter-args
    /// B32-B33, B27-B28), and the blank key is not pinned last (G!J145, G!S145). Values compare like
    /// key cells: numbers < text < FALSE < TRUE < errors, errors last ascending and first
    /// descending (B34-B37; groupby-sort-filter-args2 B27-B28, B37). A parent's value is the
    /// function of all its rows, not of its children's values (AVERAGE and `100-SUM(x)` over two
    /// keys, args2 B22-B25), at every level (three keys, args2 B19-B21).
    ///
    /// Work: one `aggregate` call per group and per parent, at most groups x key fields, which the
    /// output size bounds; each level's parent rows are gathered once (O(rows log rows) per level).
    pub(crate) fn sort_by_aggregate(
        &mut self,
        ascending: bool,
        parents_by_value: bool,
        cell: CellReferenceIndex,
        mut aggregate: impl FnMut(&[usize]) -> ArrayNode,
    ) -> Vec<ArrayNode> {
        self.sort_by_keys(&[], cell);
        let fields = self.groups.first().map_or(0, |group| group.keys.len());
        // `levels[l][g]`: the parent of group g at depth l + 1 (an index in key order of first
        // appearance, so equal parent values keep key order), with the parents' aggregates.
        let mut levels: Vec<(Vec<usize>, Vec<ArrayNode>)> = Vec::new();
        let mut previous = vec![0; self.groups.len()];
        for level in 0..fields.saturating_sub(1) {
            let mut index: HashMap<(usize, CellKey), usize> = HashMap::new();
            let mut parent_rows: Vec<Vec<usize>> = Vec::new();
            let mut parents = Vec::with_capacity(self.groups.len());
            for (group, parent) in self.groups.iter().zip(&previous) {
                let key = group.keys.get(level).map_or(CellKey::Empty, group_key);
                let id = *index.entry((*parent, key)).or_insert_with(|| {
                    parent_rows.push(Vec::new());
                    parent_rows.len() - 1
                });
                // Parent rows are only gathered when the parents sort by their aggregate.
                if let Some(rows) = parent_rows.get_mut(id).filter(|_| parents_by_value) {
                    rows.extend_from_slice(&group.rows);
                }
                parents.push(id);
            }
            let values = parent_rows
                .iter_mut()
                .map(|rows| {
                    if !parents_by_value {
                        return ArrayNode::Empty;
                    }
                    rows.sort_unstable();
                    aggregate(rows)
                })
                .collect();
            previous.clone_from(&parents);
            levels.push((parents, values));
        }
        let leaves: Vec<ArrayNode> = self
            .groups
            .iter()
            .map(|group| aggregate(&group.rows))
            .collect();
        let mut order: Vec<usize> = (0..self.groups.len()).collect();
        // A stable sort: groups that compare equal stay in key order.
        order.sort_by(|&a, &b| {
            for (parents, values) in &levels {
                let (x, y) = (parents.get(a), parents.get(b));
                if x != y {
                    let (x, y) = (x.copied().unwrap_or(0), y.copied().unwrap_or(0));
                    if !parents_by_value {
                        return x.cmp(&y);
                    }
                    return compare_aggregates(values.get(x), values.get(y), ascending, cell)
                        .then(x.cmp(&y));
                }
            }
            compare_aggregates(leaves.get(a), leaves.get(b), ascending, cell)
        });
        let mut groups: Vec<Option<Group>> = std::mem::take(&mut self.groups)
            .into_iter()
            .map(Some)
            .collect();
        self.groups = order
            .iter()
            .filter_map(|&at| groups.get_mut(at).and_then(Option::take))
            .collect();
        order
            .iter()
            .map(|&at| leaves.get(at).cloned().unwrap_or(ArrayNode::Empty))
            .collect()
    }
}

/// [`key_cmp`] for two aggregates that may be missing (they never are: the indices come from
/// the same groups).
fn compare_aggregates(
    a: Option<&ArrayNode>,
    b: Option<&ArrayNode>,
    ascending: bool,
    cell: CellReferenceIndex,
) -> Ordering {
    match (a, b) {
        (Some(a), Some(b)) => key_cmp(a, b, ascending, cell),
        _ => Ordering::Equal,
    }
}

/// A key cell as it groups: `cell_key`, except that -0 is 0. Excel stores a cell `=-0` as 0, so
/// 0, `=-0` and `=0` are one group (probes/groupby-pivotby-followup G!A781); the engine keeps the
/// sign of a computed -0.
fn group_key(node: &ArrayNode) -> CellKey {
    match node {
        ArrayNode::Number(n) if *n == 0.0 => cell_key(&ArrayNode::Number(0.0)),
        other => cell_key(other),
    }
}

/// A key cell as a group shows it, -0 as 0 (see [`group_key`]).
fn positive_zero(node: &ArrayNode) -> ArrayNode {
    match node {
        ArrayNode::Number(n) if *n == 0.0 => ArrayNode::Number(0.0),
        other => other.clone(),
    }
}

/// [`sort_key_cmp`] with numbers compared exactly (see [`Grouping::sort_by_keys`]), and text
/// compared in place with the comparator `sort_key_cmp` reaches (`cmp_text`, through
/// `compare_values`) instead of converting and cloning both strings on every comparison.
fn key_cmp(a: &ArrayNode, b: &ArrayNode, ascending: bool, cell: CellReferenceIndex) -> Ordering {
    let ordering = match (a, b) {
        (ArrayNode::Number(x), ArrayNode::Number(y)) => x.total_cmp(y),
        (ArrayNode::String(x), ArrayNode::String(y)) => cmp_text(x, y),
        _ => return sort_key_cmp(a, b, ascending, cell),
    };
    if ascending {
        ordering
    } else {
        ordering.reverse()
    }
}

/// How the groups are ordered: GROUPBY's `sort_order` (PIVOTBY's `row_sort_order` and
/// `col_sort_order` too), read by [`Model::sort_order`].
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum SortOrder {
    /// By the key fields, `ascending[i]` for field `i`; fields not named ascend
    /// ([`Grouping::sort_by_keys`]).
    Keys(Vec<bool>),
    /// By the aggregate of value column `column` (0-based among the value columns), with
    /// [`Grouping::sort_by_aggregate`].
    Value { column: usize, ascending: bool },
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

/// Whether the `function` argument is an array of functions (`HSTACK(SUM,AVERAGE)`, G!S121).
// TODO(T-1325): function arrays.
pub(crate) fn is_function_array(arg: &Node) -> bool {
    matches!(
        arg.ungrouped(),
        Node::FunctionKind {
            kind: Function::Hstack | Function::Vstack,
            ..
        }
    )
}

/// The evaluated `function` argument of GROUPBY or PIVOTBY (`name`), with whether it is an array
/// of functions (T-1325): a LAMBDA or an eta name ([`as_lambda`], the check BYROW and BYCOL
/// share), an array or range; anything else is `#VALUE!` (`5`: probes/groupby-lambda-forms
/// G!A217, probes/pivotby-followup P!A761). Error values are reported before this
/// ([`Model::evaluate_options`]).
pub(crate) fn function_argument(
    function: CalcResult,
    name: &str,
    cell: CellReferenceIndex,
) -> Result<(CalcResult, bool), CalcResult> {
    match as_lambda(function) {
        Ok(lambda) => Ok((lambda, false)),
        Err(array @ (CalcResult::Array(_) | CalcResult::Range { .. })) => Ok((array, true)),
        Err(_) => Err(value_error(cell, &format!("{name} expects a function"))),
    }
}

/// A key as its group shows it: the blank key as the number 0 (G!A1, PIVOTBY's column keys too:
/// probes/groupby-pivotby-followup G!A821), every other key as first spelled.
pub(crate) fn shown_key(key: &ArrayNode) -> ArrayNode {
    match key {
        ArrayNode::Empty => ArrayNode::Number(0.0),
        other => other.clone(),
    }
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
    /// (2: G!J241; 3, -1, 0.5: B79-B81). An array is lifted per element in Excel
    /// (followup G!A861-A981): not implemented.
    fn relationship(
        &mut self,
        arg: Option<CalcResult>,
        cell: CellReferenceIndex,
    ) -> Result<Relationship, CalcResult> {
        match arg {
            None => Ok(Relationship::Hierarchy),
            // TODO(T-1324): an array field_relationship is lifted per element.
            Some(CalcResult::Array(_) | CalcResult::Range { .. }) => {
                Err(not_implemented(cell, "GROUPBY field_relationship arrays"))
            }
            Some(result) => match self.cast_to_number(result, cell) {
                Ok(0.0) => Ok(Relationship::Hierarchy),
                Ok(1.0) => Ok(Relationship::Table),
                _ => Err(value_error(cell, "field_relationship must be 0 or 1")),
            },
        }
    }

    /// `=GROUPBY(row_fields, values, function, [field_headers], [total_depth], [sort_order],
    /// [filter_array], [field_relationship])`
    pub(crate) fn fn_groupby(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        match self.groupby(args, cell) {
            Ok(array) => CalcResult::Array(array),
            Err(error) => error,
        }
    }

    fn groupby(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> Result<Vec<Vec<ArrayNode>>, CalcResult> {
        if !(3..=8).contains(&args.len()) {
            return Err(CalcResult::new_args_number_error(cell));
        }
        let keys = self.eval_to_array(&args[0], cell)?;
        let values = self.eval_to_array(&args[1], cell)?;
        // TODO(T-1325): function arrays (`HSTACK(SUM,AVERAGE)`, G!S121) are not implemented
        // rather than a wrong #VALUE!. PERCENTOF (G!S97) still runs with one argument: its keys
        // column already matches Excel.
        if is_function_array(&args[2]) {
            return Err(function_arrays_error(cell, "GROUPBY"));
        }
        // Every error value first, in argument order; then the validation (B61-B78).
        let mut options = self.evaluate_options(&args[2..], cell)?.into_iter();
        let function = options.next().flatten().unwrap_or(CalcResult::EmptyArg);
        let field_headers = options.next().flatten();
        let total_depth = options.next().flatten();
        let sort_order = options.next().flatten();
        let filter = options.next().flatten();
        let relationship = options.next().flatten();

        let (function, function_array) = function_argument(function, "GROUPBY", cell)?;
        let headers = self.field_headers(field_headers, cell)?;
        let key_columns = width(&keys);
        let depth_given = total_depth.is_some();
        let total_depth = self.total_depth(total_depth, key_columns, cell)?;
        // Values of another height than the keys are #VALUE! (G!AB193, followup G!A1-A41).
        if keys.len() != values.len() || key_columns == 0 {
            return Err(value_error(
                cell,
                "row_fields and values must have the same height",
            ));
        }
        let value_columns = width(&values);
        let sort = self.sort_order(sort_order, key_columns, value_columns, cell)?;
        let header_row = match headers {
            FieldHeaders::Detect => values_have_header(&values).ok_or_else(|| {
                value_error(cell, "GROUPBY cannot tell whether there are headers")
            })?,
            FieldHeaders::No | FieldHeaders::Generated => false,
            FieldHeaders::Hidden | FieldHeaders::Shown => true,
        };
        let first = usize::from(header_row);
        if first >= keys.len() {
            // A header row and no records: probes/groupby-lambda-forms G!S193 (ERROR.TYPE 3).
            return Err(value_error(cell, "GROUPBY has no records"));
        }
        let rows = self.filter_rows(filter, keys.len(), first, cell)?;
        let relationship = self.relationship(relationship, cell)?;
        // A table relationship cannot have subtotals (followup G!A181, G!A261: ERROR.TYPE 3).
        if relationship == Relationship::Table && total_depth.abs() == 2 {
            return Err(value_error(cell, "field_relationship 1 has no subtotals"));
        }
        if function_array {
            return Err(function_arrays_error(cell, "GROUPBY"));
        }
        // TODO(T-1324): subtotals.
        if total_depth.abs() == 2 {
            return Err(not_implemented(cell, "GROUPBY subtotals"));
        }
        if value_sort_shows_subtotals(&sort, key_columns, depth_given) {
            return Err(not_implemented(cell, "GROUPBY subtotals"));
        }
        // With one total row (or none) and keys sorted, a table gives the hierarchy's layout
        // (followup G!A221, G!A241: probed with total_depth 1 and default sorting only).
        // TODO(T-1324): a table relationship sorted by a value column.
        if relationship == Relationship::Table && matches!(sort, SortOrder::Value { .. }) {
            return Err(not_implemented(cell, "GROUPBY tables sorted by value"));
        }

        let shows_header = matches!(headers, FieldHeaders::Generated | FieldHeaders::Shown);
        let extra_rows = usize::from(shows_header) + usize::from(total_depth != 0);
        // The output size is checked while grouping, before the sort, any LAMBDA call or the
        // output allocation (SPEC-0006 step 6).
        let limit =
            |groups: usize| check_array_size(groups + extra_rows, key_columns + value_columns);
        let mut grouping = Grouping::new(&keys, rows.iter().copied(), limit)
            .map_err(|(error, message)| CalcResult::new_error(error, cell, message))?;

        let mut caller = self.lambda_caller(&function, 1);
        // A value sort keeps the values it sorted by, so that column is not computed twice.
        let sorted = match sort {
            SortOrder::Keys(ascending) => {
                grouping.sort_by_keys(&ascending, cell);
                None
            }
            SortOrder::Value { column, ascending } => {
                let by = grouping.sort_by_aggregate(ascending, true, cell, |group_rows| {
                    self.aggregate(&mut caller, &values, column, group_rows, cell)
                });
                Some((column, by))
            }
        };
        let mut output = Vec::with_capacity(grouping.groups.len() + extra_rows);
        if shows_header {
            output.push(header_cells(headers, &keys, &values));
        }
        // The headers and the total stay in place whatever the sort (B38-B39).
        if total_depth < 0 {
            let row = self.total_row(&mut caller, &values, &rows, key_columns, cell);
            output.push(row);
        }
        for (index, group) in grouping.groups.iter().enumerate() {
            let mut row: Vec<ArrayNode> = group.keys.iter().map(shown_key).collect();
            for column in 0..value_columns {
                let value = match &sorted {
                    Some((by_column, by)) if *by_column == column => {
                        by.get(index).cloned().unwrap_or(ArrayNode::Empty)
                    }
                    _ => self.aggregate(&mut caller, &values, column, &group.rows, cell),
                };
                row.push(value);
            }
            output.push(row);
        }
        if total_depth > 0 {
            let row = self.total_row(&mut caller, &values, &rows, key_columns, cell);
            output.push(row);
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
    /// grand total on top (G!A49, G!AB49). 3 and -3 are `#VALUE!`, and so is ±2 with one key field
    /// (probes/groupby-pivotby-followup G!A101-A161), and so is a fraction (0.5:
    /// probes/groupby-lambda-forms G!J241).
    pub(crate) fn total_depth(
        &mut self,
        arg: Option<CalcResult>,
        key_columns: usize,
        cell: CellReferenceIndex,
    ) -> Result<i8, CalcResult> {
        let Some(result) = arg else {
            return Ok(1);
        };
        match self.cast_to_number(result, cell)? {
            0.0 => Ok(0),
            1.0 => Ok(1),
            -1.0 => Ok(-1),
            // Subtotals (±2) need two key fields; the caller reports them as not implemented
            // after validating the other arguments.
            2.0 if key_columns > 1 => Ok(2),
            -2.0 if key_columns > 1 => Ok(-2),
            _ => Err(value_error(cell, "total_depth is out of range")),
        }
    }

    /// The total row: "Total", `""` under the other key fields (empty text, not blank cells:
    /// G!AC33), then the function of every value of each column in source order, not of the
    /// group results (AVERAGE's total is 13.05 = 130.5/10, G!S73).
    fn total_row(
        &mut self,
        caller: &mut LambdaCaller,
        values: &[Vec<ArrayNode>],
        rows: &[usize],
        key_columns: usize,
        cell: CellReferenceIndex,
    ) -> Vec<ArrayNode> {
        let mut row = total_label_cells(key_columns);
        for column in 0..width(values) {
            row.push(self.aggregate(caller, values, column, rows, cell));
        }
        row
    }

    /// The function of the values of `column` in `rows`, passed as one column array in source
    /// order (ARRAYTOTEXT gives "Apple, Pear, Apple", G!AB97), as values, not a reference (ISREF is
    /// FALSE), with blank cells as blanks (ISBLANK is TRUE, COUNTA counts them:
    /// probes/groupby-lambda-forms G!S217, G!A241, probes/groupby-blank-values). An array result is
    /// `#VALUE!`, a 1x1 array too (G!J121, one-record groups included). So is a LAMBDA of another
    /// arity (groupby-lambda-forms G!J217): each call fails on its own, not the whole GROUPBY.
    /// Excel also gives `#VALUE!` for a body that is a one-cell reference (`LAMBDA(x,D2)`,
    /// groupby-lambda-forms G!J193), but a LAMBDA call here returns that cell's value.
    // TODO(T-1325): a reference result is #VALUE! (needs the LAMBDA call to keep references).
    pub(crate) fn aggregate(
        &mut self,
        caller: &mut LambdaCaller,
        values: &[Vec<ArrayNode>],
        column: usize,
        rows: &[usize],
        cell: CellReferenceIndex,
    ) -> ArrayNode {
        let input = rows
            .iter()
            .map(|&row| vec![at(values, row, column)])
            .collect();
        calc_result_to_array_node(self.call_lambda_caller(caller, [CalcResult::Array(input)], cell))
    }
}

/// The header row GROUPBY shows: the key field names, then the value field names
/// ([`key_names`], [`value_name`]; G!J25, G!A25).
fn header_cells(
    headers: FieldHeaders,
    keys: &[Vec<ArrayNode>],
    values: &[Vec<ArrayNode>],
) -> Vec<ArrayNode> {
    let mut row = key_names(headers, keys);
    row.extend((0..width(values)).map(|column| value_name(headers, values, column)));
    row
}

/// The number of columns of a (rectangular) array.
pub(crate) fn width(data: &[Vec<ArrayNode>]) -> usize {
    data.first().map_or(0, Vec::len)
}

/// Empty text, the filler of total rows, corners and empty PIVOTBY cells (not a blank: G!AC33).
pub(crate) fn empty_text() -> ArrayNode {
    ArrayNode::String("".into())
}

/// The "Total" label of a total row or column.
pub(crate) fn total_label() -> ArrayNode {
    ArrayNode::String("Total".into())
}

/// The key cells of a total row: "Total", then `""` under the other key fields (G!AC33, P!A97).
pub(crate) fn total_label_cells(key_columns: usize) -> Vec<ArrayNode> {
    let mut cells = vec![total_label()];
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

/// `#N/IMPL!` for an array of functions (`HSTACK(SUM,AVERAGE)`) in GROUPBY or PIVOTBY (`name`).
// TODO(T-1325): function arrays.
pub(crate) fn function_arrays_error(cell: CellReferenceIndex, name: &str) -> CalcResult {
    not_implemented(cell, &format!("{name} function arrays"))
}

/// Whether a value sort over several fields with the total depth omitted would show subtotals,
/// as depth 2 does (GROUPBY: probes/pivotby-followup P!A921, unlike explicit 0, followup2
/// G!A261; PIVOTBY rows P!A201, P!A901, columns P!A941; an explicit 0 or 1: P!A861, P!A1141).
// TODO(T-1324): subtotals.
pub(crate) fn value_sort_shows_subtotals(
    sort: &SortOrder,
    fields: usize,
    depth_given: bool,
) -> bool {
    matches!(sort, SortOrder::Value { .. }) && fields > 1 && !depth_given
}

#[cfg(test)]
mod tests {
    use super::Grouping;
    use crate::expressions::{parser::ArrayNode, types::CellReferenceIndex};

    const CELL: CellReferenceIndex = CellReferenceIndex {
        sheet: 0,
        row: 1,
        column: 1,
    };

    fn text(s: &str) -> ArrayNode {
        ArrayNode::String(s.into())
    }

    /// The first key of each group and its rows, after sorting with `ascending`.
    fn sorted(keys: &[ArrayNode], ascending: bool) -> Vec<(ArrayNode, Vec<usize>)> {
        let keys: Vec<Vec<ArrayNode>> = keys.iter().map(|key| vec![key.clone()]).collect();
        let mut grouping = Grouping::new(&keys, 0..keys.len(), |_| None)
            .unwrap_or(Grouping { groups: Vec::new() });
        grouping.sort_by_keys(&[ascending], CELL);
        grouping
            .groups
            .into_iter()
            .map(|group| (group.keys[0].clone(), group.rows))
            .collect()
    }

    #[test]
    fn case_folded_first_spelling_and_blank_last_both_ways() {
        let keys = [
            text("West"),
            ArrayNode::Empty,
            text("east"),
            ArrayNode::Number(2.0),
            text("East"),
            ArrayNode::Boolean(true),
        ];
        let ascending = sorted(&keys, true);
        assert_eq!(
            ascending,
            [
                (ArrayNode::Number(2.0), vec![3]),
                (text("east"), vec![2, 4]),
                (text("West"), vec![0]),
                (ArrayNode::Boolean(true), vec![5]),
                (ArrayNode::Empty, vec![1]),
            ]
        );
        // Descending reverses every type but keeps the blank key last (G!A145).
        let descending = sorted(&keys, false);
        assert_eq!(
            descending
                .iter()
                .map(|(key, _)| key.clone())
                .collect::<Vec<_>>(),
            [
                ArrayNode::Boolean(true),
                text("West"),
                text("east"),
                ArrayNode::Number(2.0),
                ArrayNode::Empty,
            ]
        );
    }

    #[test]
    fn two_key_fields_sort_lexicographically() {
        let keys = vec![
            vec![text("b"), ArrayNode::Number(2.0)],
            vec![text("a"), ArrayNode::Number(3.0)],
            vec![text("b"), ArrayNode::Number(1.0)],
            vec![text("a"), ArrayNode::Number(3.0)],
        ];
        let mut grouping = Grouping::new(&keys, 0..keys.len(), |_| None)
            .unwrap_or(Grouping { groups: Vec::new() });
        grouping.sort_by_keys(&[true, false], CELL);
        let rows: Vec<Vec<usize>> = grouping.groups.into_iter().map(|g| g.rows).collect();
        assert_eq!(rows, [vec![1, 3], vec![0], vec![2]]);
    }

    #[test]
    fn value_sort_keeps_parents_with_equal_aggregates_together() {
        // Parents a and b both sum to 4: they keep key order and their children do not
        // interleave; b's children tie at 2 and keep key order (w before y).
        let keys = vec![
            vec![text("a"), text("x")],
            vec![text("b"), text("y")],
            vec![text("a"), text("z")],
            vec![text("b"), text("w")],
        ];
        let values = [1.0, 2.0, 3.0, 2.0];
        let mut grouping = Grouping::new(&keys, 0..keys.len(), |_| None)
            .unwrap_or(Grouping { groups: Vec::new() });
        let mut calls = 0;
        let sums = grouping.sort_by_aggregate(true, true, CELL, |rows| {
            calls += 1;
            ArrayNode::Number(rows.iter().map(|&row| values[row]).sum())
        });
        let rows: Vec<Vec<usize>> = grouping.groups.into_iter().map(|g| g.rows).collect();
        assert_eq!(rows, [vec![0], vec![2], vec![3], vec![1]]);
        assert_eq!(sums, [1.0, 3.0, 2.0, 2.0].map(ArrayNode::Number).to_vec());
        // One call per group and one per parent.
        assert_eq!(calls, 6);
    }
}
