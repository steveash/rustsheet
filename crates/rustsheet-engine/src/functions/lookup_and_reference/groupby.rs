//! GROUPBY on the grouping core that PIVOTBY shares (SPEC-0006 Design steps 1-3 and 5-7, T-1322).
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
    functions::logical::LambdaCaller,
    functions::spill_functions::{cell_key, sort_key_cmp, CellKey},
    functions::util::collation::cmp_text,
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

/// The `field_headers` argument of GROUPBY (and PIVOTBY): whether the first row of the fields
/// and values holds headers, and which header row the result shows (G!J1, S1, AB1, A25, J25).
#[derive(Clone, Copy, PartialEq)]
enum FieldHeaders {
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
fn values_have_header(values: &[Vec<ArrayNode>]) -> Option<bool> {
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

/// `#VALUE!` for an invalid GROUPBY argument.
fn value_error(cell: CellReferenceIndex, message: &str) -> CalcResult {
    CalcResult::new_error(Error::VALUE, cell, message.to_string())
}

/// `#N/IMPL!` for a GROUPBY feature a later SPEC-0006 step adds.
fn not_implemented(cell: CellReferenceIndex, feature: &str) -> CalcResult {
    CalcResult::new_error(
        Error::NIMPL,
        cell,
        format!("{feature} are not implemented yet"),
    )
}

/// An optional argument, `None` when it is missing or omitted (`GROUPBY(k,v,f,,0)`).
fn given(arg: Option<&Node>) -> Option<&Node> {
    arg.filter(|node| !matches!(node, Node::EmptyArgKind))
}

/// A cell of a 2-D array, blank when outside it (arrays from `eval_to_array` are rectangular, so
/// this only guards against a panic).
fn at(data: &[Vec<ArrayNode>], row: usize, column: usize) -> ArrayNode {
    data.get(row)
        .and_then(|cells| cells.get(column))
        .cloned()
        .unwrap_or(ArrayNode::Empty)
}

impl Model {
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
        if matches!(
            args[2].ungrouped(),
            Node::FunctionKind {
                kind: Function::Hstack | Function::Vstack,
                ..
            }
        ) {
            return Err(not_implemented(cell, "GROUPBY function arrays"));
        }
        let function = self.evaluate_node_in_context(&args[2], cell);
        let headers = self.field_headers(args.get(3), cell)?;
        let key_columns = keys.first().map_or(0, Vec::len);
        let total_depth = self.total_depth(args.get(4), key_columns, cell)?;
        // TODO(T-1323): sort_order and filter_array; TODO(T-1324): field_relationship.
        if args.iter().skip(5).any(|arg| given(Some(arg)).is_some()) {
            return Err(not_implemented(
                cell,
                "GROUPBY sort_order, filter_array and field_relationship",
            ));
        }
        match &function {
            CalcResult::Lambda(_) => {}
            CalcResult::Array(_) | CalcResult::Range { .. } => {
                return Err(not_implemented(cell, "GROUPBY function arrays"));
            }
            CalcResult::Error { .. } => return Err(function),
            // A scalar is #VALUE! (probes/groupby-lambda-forms G!A217).
            _ => return Err(value_error(cell, "GROUPBY expects a function")),
        }
        // Values of another height than the keys are #VALUE! (G!AB193, followup G!A1-A41).
        if keys.len() != values.len() || key_columns == 0 {
            return Err(value_error(
                cell,
                "row_fields and values must have the same height",
            ));
        }
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
        let value_columns = values.first().map_or(0, Vec::len);
        let shows_header = matches!(headers, FieldHeaders::Generated | FieldHeaders::Shown);
        let extra_rows = usize::from(shows_header) + usize::from(total_depth != 0);
        // The output size is checked while grouping, before the sort, any LAMBDA call or the
        // output allocation (SPEC-0006 step 6).
        let limit =
            |groups: usize| check_array_size(groups + extra_rows, key_columns + value_columns);
        let mut grouping = Grouping::new(&keys, first..keys.len(), limit)
            .map_err(|(error, message)| CalcResult::new_error(error, cell, message))?;
        grouping.sort_by_keys(&[], cell);
        let rows = grouping.groups.len() + extra_rows;

        let mut caller = self.lambda_caller(&function, 1);
        let mut output = Vec::with_capacity(rows);
        if shows_header {
            output.push(header_cells(headers, &keys, &values));
        }
        let all_rows: Vec<usize> = (first..values.len()).collect();
        if total_depth < 0 {
            let row = self.total_row(&mut caller, &values, &all_rows, key_columns, cell);
            output.push(row);
        }
        for group in &grouping.groups {
            // The blank key shows as the number 0 (G!A1); every other key as first spelled.
            let mut row: Vec<ArrayNode> = group
                .keys
                .iter()
                .map(|key| match key {
                    ArrayNode::Empty => ArrayNode::Number(0.0),
                    other => other.clone(),
                })
                .collect();
            for column in 0..value_columns {
                row.push(self.aggregate(&mut caller, &values, column, &group.rows, cell));
            }
            output.push(row);
        }
        if total_depth > 0 {
            let row = self.total_row(&mut caller, &values, &all_rows, key_columns, cell);
            output.push(row);
        }
        Ok(output)
    }

    /// The `field_headers` argument: omitted, or 0 to 3; anything else is `#VALUE!` (4: G!A217),
    /// a fraction too (2.5: probes/groupby-lambda-forms G!S241).
    fn field_headers(
        &mut self,
        arg: Option<&Node>,
        cell: CellReferenceIndex,
    ) -> Result<FieldHeaders, CalcResult> {
        let Some(node) = given(arg) else {
            return Ok(FieldHeaders::Detect);
        };
        match self.get_number(node, cell)? {
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
    fn total_depth(
        &mut self,
        arg: Option<&Node>,
        key_columns: usize,
        cell: CellReferenceIndex,
    ) -> Result<i8, CalcResult> {
        let Some(node) = given(arg) else {
            return Ok(1);
        };
        match self.get_number(node, cell)? {
            0.0 => Ok(0),
            1.0 => Ok(1),
            -1.0 => Ok(-1),
            // TODO(T-1324): subtotals.
            depth if depth.abs() == 2.0 && key_columns > 1 => {
                Err(not_implemented(cell, "GROUPBY subtotals"))
            }
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
        let mut row = vec![ArrayNode::String("Total".into())];
        row.extend((1..key_columns).map(|_| ArrayNode::String("".into())));
        for column in 0..values.first().map_or(0, Vec::len) {
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
    fn aggregate(
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

/// The header row GROUPBY shows: the fields' and values' own first row (3, G!J25), or
/// "Row Field n" and "Value n" (2, G!A25; numbered per column, unprobed beyond one).
fn header_cells(
    headers: FieldHeaders,
    keys: &[Vec<ArrayNode>],
    values: &[Vec<ArrayNode>],
) -> Vec<ArrayNode> {
    let key_columns = keys.first().map_or(0, Vec::len);
    let value_columns = values.first().map_or(0, Vec::len);
    if headers == FieldHeaders::Shown {
        return (0..key_columns)
            .map(|column| at(keys, 0, column))
            .chain((0..value_columns).map(|column| at(values, 0, column)))
            .collect();
    }
    (1..=key_columns)
        .map(|n| ArrayNode::String(format!("Row Field {n}").into()))
        .chain((1..=value_columns).map(|n| ArrayNode::String(format!("Value {n}").into())))
        .collect()
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
}
