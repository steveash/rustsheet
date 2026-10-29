use crate::functions::util::array_node_to_calc_result;
use crate::functions::util::near_integer::near_integer_whole;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};

use crate::{
    calc_result::CalcResult,
    expressions::{
        parser::{ArrayNode, Node},
        token::Error,
        types::CellReferenceIndex,
    },
    functions::{lift::LiftCost, Function},
    model::Model,
};

use super::util::compare_values;

/// Compare two sort keys following Excel's rules:
///   Numbers < Strings < Booleans < Errors < Empty cells
/// Empty cells always sort last regardless of ascending/descending.
///
/// It is a total order, so `sort_by` cannot panic: grouping.rs tests it with GROUPBY's `key_cmp`
/// (`sort_comparators_are_total_orders_on_mixed_types`, T-1694). The blank check must stay here:
/// `compare_values` alone coerces a blank to 0, "" or FALSE, which is not transitive (-0 = blank =
/// "" but -0 < "").
pub(crate) fn sort_key_cmp(
    a: &ArrayNode,
    b: &ArrayNode,
    ascending: bool,
    cell: CellReferenceIndex,
) -> Ordering {
    let a_empty = matches!(a, ArrayNode::Empty);
    let b_empty = matches!(b, ArrayNode::Empty);
    match (a_empty, b_empty) {
        (true, true) => Ordering::Equal,
        (true, false) => Ordering::Greater,
        (false, true) => Ordering::Less,
        (false, false) => {
            let key_a = array_node_to_calc_result(a, cell);
            let key_b = array_node_to_calc_result(b, cell);
            let cmp = compare_values(&key_a, &key_b);
            if ascending {
                cmp.cmp(&0)
            } else {
                0.cmp(&cmp)
            }
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn array_node_is_truthy(node: &ArrayNode) -> bool {
    match node {
        ArrayNode::Boolean(b) => *b,
        ArrayNode::Number(n) => *n != 0.0,
        _ => false,
    }
}

/// A hashable, equality-normalised representation of a single cell value.
/// Strings are uppercased (matching case-insensitive `array_nodes_equal`).
/// Numbers use bit-level identity (`f64::to_bits`), which is exact for all
/// values that actually appear in spreadsheet cells.
#[derive(Clone, Hash, Eq, PartialEq)]
pub(crate) enum CellKey {
    Number(u64),
    Boolean(bool),
    Str(String),
    Error(u8),
    Empty,
    /// A LAMBDA element, by its lambda id.
    Lambda(usize),
}

pub(crate) fn cell_key(node: &ArrayNode) -> CellKey {
    match node {
        ArrayNode::Number(n) => CellKey::Number(n.to_bits()),
        ArrayNode::Boolean(b) => CellKey::Boolean(*b),
        ArrayNode::String(s) => CellKey::Str(s.to_uppercase()),
        // ERROR.TYPE codes are distinct per kind, so they key errors exactly.
        ArrayNode::Error(e) => CellKey::Error(e.error_type_code()),
        ArrayNode::Empty => CellKey::Empty,
        ArrayNode::Lambda(id) => CellKey::Lambda(*id),
        // Keyed as the `#VALUE!` it shows: UNIQUE over nested cells is unprobed (SPEC-0018).
        ArrayNode::Nested(_) => CellKey::Error(Error::VALUE.error_type_code()),
    }
}

/// Whether UNIQUE's `by_col` and `exactly_once` are each absent, empty or a literal, with
/// `by_col` not TRUE, so its array may be tailed (SPEC-0020 Amendment 1, section 3): by columns
/// a whole column is one key, and a flag that is an array or a reference lifts the call.
fn unique_flags_are_literals(args: &[Node]) -> bool {
    let literal = |arg: Option<&Node>, by_col: bool| match arg {
        None | Some(Node::EmptyArgKind) => true,
        Some(Node::BooleanKind(flag)) => !(by_col && *flag),
        Some(Node::NumberKind(flag)) => !(by_col && *flag != 0.0),
        Some(_) => false,
    };
    literal(args.get(1), true) && literal(args.get(2), false)
}

/// One UNIQUE call over `data`: its distinct rows (or columns), or those that occur once.
fn unique_of(
    data: &[Vec<ArrayNode>],
    by_col: bool,
    exactly_once: bool,
    cell: CellReferenceIndex,
) -> CalcResult {
    if !by_col {
        let num_rows = data.len();

        // First pass: count how many times each row key appears.
        let mut counts: HashMap<Vec<CellKey>, usize> = HashMap::with_capacity(num_rows);
        for row in data {
            *counts.entry(row_key(row)).or_insert(0) += 1;
        }

        // Second pass: collect rows in original order.
        let result: Vec<Vec<ArrayNode>> = if exactly_once {
            data.iter()
                .filter(|row| counts[&row_key(row)] == 1)
                .cloned()
                .collect()
        } else {
            let mut seen: HashSet<Vec<CellKey>> = HashSet::with_capacity(num_rows);
            data.iter()
                .filter(|row| seen.insert(row_key(row)))
                .cloned()
                .collect()
        };

        if result.is_empty() {
            return CalcResult::new_error(
                Error::CALC,
                cell,
                "UNIQUE found no matching rows".to_string(),
            );
        }

        CalcResult::Array(result)
    } else {
        let num_cols = data[0].len();

        // First pass: count how many times each column key appears.
        let mut counts: HashMap<Vec<CellKey>, usize> = HashMap::with_capacity(num_cols);
        for j in 0..num_cols {
            *counts.entry(col_key(data, j)).or_insert(0) += 1;
        }

        // Second pass: collect column indices in original order.
        let result_cols: Vec<usize> = if exactly_once {
            (0..num_cols)
                .filter(|&j| counts[&col_key(data, j)] == 1)
                .collect()
        } else {
            let mut seen: HashSet<Vec<CellKey>> = HashSet::with_capacity(num_cols);
            (0..num_cols)
                .filter(|&j| seen.insert(col_key(data, j)))
                .collect()
        };

        if result_cols.is_empty() {
            return CalcResult::new_error(
                Error::CALC,
                cell,
                "UNIQUE found no matching columns".to_string(),
            );
        }

        CalcResult::Array(
            data.iter()
                .map(|row| result_cols.iter().map(|&c| row[c].clone()).collect())
                .collect(),
        )
    }
}

fn row_key(row: &[ArrayNode]) -> Vec<CellKey> {
    row.iter().map(cell_key).collect()
}

fn col_key(data: &[Vec<ArrayNode>], j: usize) -> Vec<CellKey> {
    data.iter().map(|row| cell_key(&row[j])).collect()
}

/// The first of `orderings` that is not `Equal`: the comparison of SORT's and SORTBY's keys,
/// each tie broken by the next key.
fn first_unequal(mut orderings: impl Iterator<Item = Ordering>) -> Ordering {
    orderings
        .find(|ordering| *ordering != Ordering::Equal)
        .unwrap_or(Ordering::Equal)
}

/// The elements of a SORT lifted over its `by_col`, `result`, given each call's `by_col` when it
/// sorted (`None` when it failed), row-major. Failed calls keep their error; each row's first
/// sorting call leads it, and the leads keep their call's top-left when all their `by_col` values
/// are equal (else only the first does); every other sorting call is empty when `only_key_two`
/// and it is the first such element or sits below an empty-able one with the same `by_col`, and
/// `#VALUE!` otherwise (probes/t2929-sort-lifted-by-col-*, T-2929; the rule and its UNVERIFIED
/// corners are in arrays.md).
// Excel: docs/excel-semantics/arrays.md "TOROW, TOCOL, SORT and TEXTJOIN lifted arguments".
fn sort_lifted_elements(
    result: CalcResult,
    sorted: &[Option<ByCol>],
    only_key_two: bool,
) -> CalcResult {
    let CalcResult::Array(mut rows) = result else {
        return result;
    };
    let columns = rows.first().map_or(0, Vec::len);
    if columns == 0 || rows.len().saturating_mul(columns) != sorted.len() {
        return CalcResult::Array(rows);
    }
    let by_col = |row: usize, column: usize| sorted.get(row * columns + column).copied().flatten();
    let lead = |row: usize| (0..columns).find(|&column| by_col(row, column).is_some());
    let mut leads = (0..rows.len()).filter_map(|row| lead(row).and_then(|c| by_col(row, c)));
    let first_lead = leads.next();
    let leads_equal = leads.all(|value| Some(value) == first_lead);
    let mut first_lead_placed = false;
    let mut broken_seen = false;
    // Whether each element of the row above is a broken element that may be empty.
    let mut above = vec![false; columns];
    for (row, nodes) in rows.iter_mut().enumerate() {
        let row_lead = lead(row);
        let mut here = vec![false; columns];
        for (column, node) in nodes.iter_mut().enumerate() {
            let Some(value) = by_col(row, column) else {
                continue;
            };
            if Some(column) == row_lead && (leads_equal || !first_lead_placed) {
                first_lead_placed = true;
                continue;
            }
            let same_above = row > 0
                && above.get(column).copied().unwrap_or(false)
                && by_col(row - 1, column) == Some(value);
            let may_be_empty = !std::mem::replace(&mut broken_seen, true) || same_above;
            if let Some(slot) = here.get_mut(column) {
                *slot = may_be_empty;
            }
            *node = if may_be_empty && only_key_two {
                ArrayNode::Empty
            } else {
                ArrayNode::Error(Error::VALUE)
            };
        }
        above = here;
    }
    CalcResult::Array(rows)
}

/// The `by_col` an element of a lifted SORT sorted with: its value, and whether it was written as a
/// number (a number is never equal to a boolean).
#[derive(Clone, Copy, PartialEq, Eq)]
struct ByCol {
    value: bool,
    is_number: bool,
}

/// An evaluated SORTBY `by_array` argument.
type SortByArray = Vec<Vec<ArrayNode>>;

/// The ways a SORTBY `by_array` of `key` can sort an array of `rows` x `cols`, as (rows, columns):
/// a column as tall as the array sorts its rows and a row as wide as the array sorts its columns
/// (`SORTBY(A1:B3,{2,1})` swaps the columns; `SORTBY(A1:A3,{3,1,2})` is `#VALUE!`). So one value
/// sorts the columns of a one-column array, which leaves it as it is: `SORTBY(A1:A3,5)` is A1:A3,
/// whatever the value (text, a boolean, a one-cell range), while `SORTBY(A1:B3,5)` is `#VALUE!`
/// (probes/t2310-sortby-single-by-array H1-H81). Anything wider is `#VALUE!` too
/// (probes/sort-error-args H101).
// Excel: docs/excel-semantics/arrays.md "SORTBY with a single-value by_array or a fractional sort_order".
fn by_array_axes(key: &[Vec<ArrayNode>], rows: usize, cols: usize) -> (bool, bool) {
    let width = key.first().map_or(0, Vec::len);
    (
        key.len() == rows && width == 1,
        key.len() == 1 && width == cols,
    )
}

impl Model {
    /// Evaluate a node and convert the result to a 2-D array of `ArrayNodes`.
    /// Handles Range references, inline Arrays, and scalar values.
    pub(crate) fn eval_to_array(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> Result<Vec<Vec<ArrayNode>>, CalcResult> {
        let result = self.evaluate_node_in_context(node, cell);
        self.result_to_array(result, cell)
    }

    /// [`Model::eval_to_array`] that keeps a lone blank cell blank, as a blank cell of a range is:
    /// `TOROW(Z1,3)` is `#CALC!` and `UNIQUE(Z1)&"x"` is "x" (docs/excel-semantics/arrays.md
    /// "TOROW and UNIQUE of a lone blank cell", T-2409).
    pub(crate) fn eval_to_array_keep_blank(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> Result<Vec<Vec<ArrayNode>>, CalcResult> {
        let result = self.evaluate_node_in_context(node, cell);
        self.result_to_array_keep_blank(result, cell)
    }

    /// [`Model::result_to_array`], a blank cell kept as a blank element.
    pub(crate) fn result_to_array_keep_blank(
        &mut self,
        result: CalcResult,
        cell: CellReferenceIndex,
    ) -> Result<Vec<Vec<ArrayNode>>, CalcResult> {
        match result {
            CalcResult::EmptyCell => Ok(vec![vec![ArrayNode::Empty]]),
            result => self.result_to_array(result, cell),
        }
    }

    /// [`Model::eval_to_array`] for an already evaluated `result`.
    pub(crate) fn result_to_array(
        &mut self,
        result: CalcResult,
        cell: CellReferenceIndex,
    ) -> Result<Vec<Vec<ArrayNode>>, CalcResult> {
        match result {
            CalcResult::Range { left, right } => self.evaluate_range(left, right, cell),
            CalcResult::Array(arr) => Ok(arr),
            CalcResult::Number(n) => Ok(vec![vec![ArrayNode::Number(n)]]),
            CalcResult::Boolean(b) => Ok(vec![vec![ArrayNode::Boolean(b)]]),
            CalcResult::String(s) => Ok(vec![vec![ArrayNode::String(s)]]),
            CalcResult::EmptyCell | CalcResult::EmptyArg => Ok(vec![vec![ArrayNode::Number(0.0)]]),
            err @ CalcResult::Error { .. } => Err(err),
            // A LAMBDA is a one-element array of it: `VSTACK(LAMBDA(x,x),LAMBDA(x,x*2))` is two
            // LAMBDAs MAP can call (probes/lambda-arrays F1, F4, T-2039).
            CalcResult::Lambda(id) => Ok(vec![vec![ArrayNode::Lambda(id)]]),
        }
    }

    // ── SORT ──────────────────────────────────────────────────────────────────

    fn validate_sort_order(order: i32, cell: CellReferenceIndex) -> Result<bool, CalcResult> {
        match order {
            1 => Ok(true),
            -1 => Ok(false),
            _ => Err(CalcResult::new_error(
                Error::VALUE,
                cell,
                "sort_order must be 1 or -1".to_string(),
            )),
        }
    }

    /// `=SORT(array, [sort_index], [sort_order], [by_col])`
    ///
    /// Returns the array sorted by the given column (or row when `by_col=TRUE`).
    ///   * `sort_index` – 1-based column/row index to sort by (default 1), or a list of them
    ///     (an array or range), each tie broken by the next
    ///   * `sort_order` – 1 = ascending, -1 = descending (default 1), or a list as long as
    ///     `sort_index`
    ///   * `by_col`     – FALSE = sort rows, TRUE = sort columns (default FALSE)
    ///
    /// `array` is evaluated once (`read_once` in functions/logical/lambda.rs relies on it).
    pub(crate) fn fn_sort(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let kind = Function::Sort;
        if !kind
            .arg_kinds()
            .is_some_and(|kinds| kinds.accepts(args.len()))
        {
            return CalcResult::new_args_number_error(cell);
        }

        let data = match self.eval_to_array(&args[0], cell) {
            Ok(d) => d,
            Err(e) => return e,
        };

        if data.is_empty() || data[0].is_empty() {
            return CalcResult::Array(vec![]);
        }

        let num_rows = data.len();
        let num_cols = data[0].len();

        // An omitted argument (`SORT(arr,,)`) takes its default: probes/counta-empty-value D22-D24.
        // Argument errors come before value validation and are reported left to right: sort_index,
        // then sort_order, then by_col (probes/sort-error-args H29-H53; docs/excel-semantics/arrays.md
        // "SORT argument errors come before validation").
        // A sort_order that cannot be read as numbers is `#VALUE!` only after by_col is read, as a
        // validation error is (a multi-cell sort_order is a list since T-1837, checked against
        // sort_index below: c5d278a8662ef4b2 A39).
        let raw_index = if args.len() >= 2 && !matches!(args[1], Node::EmptyArgKind) {
            self.sort_number_list(&args[1], cell)
        } else {
            Ok(vec![1.0])
        };
        let raw_index = match raw_index {
            Ok(list) => list,
            Err(e) => return e,
        };
        let raw_order = if args.len() >= 3 && !matches!(args[2], Node::EmptyArgKind) {
            self.sort_number_list(&args[2], cell)
        } else {
            Ok(vec![1.0])
        };
        if let Err(e @ CalcResult::Error { .. }) = &raw_order {
            if !matches!(
                e,
                CalcResult::Error {
                    error: Error::VALUE,
                    ..
                }
            ) {
                return e.clone();
            }
        }
        // The array, sort_index and sort_order are read once here; by_col lifts over an array, one
        // SORT per element (`SORT(Data!D6:E7,3,1e-15,<2x4 booleans>)` is a 2x4 `#VALUE!`,
        // gen-m7/7307132bb8820f17 A1076; T-2319). Each element sorts the whole array, and a range
        // by_col lifts too (probes/t2366-sort-lifted-by-col). Which elements keep their call's
        // top-left is `sort_lifted_elements`'s (T-2929).
        let cells = u64::try_from(num_rows.saturating_mul(num_cols))
            .unwrap_or(u64::MAX)
            .max(1);
        // Each call's by_col when it sorted, in row-major order (one entry per element: every
        // element runs, `Reuse::Never`).
        let mut sorted: Vec<Option<ByCol>> = Vec::new();
        let result = self.lift_call_costed(
            kind,
            args,
            LiftCost::Cells(cells),
            cell,
            &mut |model, values| {
                let by_col = match values.get(3) {
                    None | Some(CalcResult::EmptyArg) => Ok(false),
                    Some(value) => Self::cast_bool_ref(value, cell),
                };
                // A number by_col is not the boolean it reads as when leads are compared
                // (probes/t2947-sort-lifted-by-col-corners).
                let is_number = matches!(values.get(3), Some(CalcResult::Number(_)));
                let (result, read) = match (by_col, &raw_order) {
                    (Err(error), _) => (error, None),
                    (Ok(_), Err(error)) => (error.clone(), None),
                    (Ok(by_col), Ok(raw_order)) => (
                        model.sort_by_keys(&data, &raw_index, raw_order, by_col, cell),
                        Some(ByCol {
                            value: by_col,
                            is_number,
                        }),
                    ),
                };
                // A call failing on its sort_index counts as a sorting call unless no call
                // has sorted yet (`{FALSE;TRUE;TRUE}` keeps the later two; `{FALSE,FALSE,TRUE,TRUE}`), while an invalid by_col
                // is always skipped (probes/t2947-sort-lifted-by-col-corners).
                let failed = matches!(result, CalcResult::Error { .. });
                let sorted_before = sorted.iter().any(Option::is_some);
                sorted.push(read.filter(|_| !failed || sorted_before));
                result
            },
        );
        // One call is the unlifted SORT (or a 1x1 by_col), whose result is the sorted array.
        if sorted.len() < 2 {
            return result;
        }
        // sort_index truncates, so 2.7 is key 2 too (probes/t2929-sort-lifted-by-col-errors K18).
        let only_key_two = matches!(raw_index.as_slice(), [index] if (2.0..3.0).contains(index));
        sort_lifted_elements(result, &sorted, only_key_two)
    }

    /// One SORT call over the read `data`: the `sort_index` and `sort_order` lists validated against
    /// `data` (rows, or columns when `by_col`), then the sort.
    fn sort_by_keys(
        &self,
        data: &[Vec<ArrayNode>],
        raw_index: &[f64],
        raw_order: &[f64],
        by_col: bool,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        let num_rows = data.len();
        let num_cols = data.first().map_or(0, Vec::len);
        // An array or range sort_index is a list of keys, applied in order, and a sort_order list
        // must be as long (`SORT(A1:B3,{1;2})` sorts by column 1 then 2; `SORT(A1:B3,1,{1;-1})` is
        // `#VALUE!`: probes/m7-argument-modes F10:H11). Rows tied on a key are ordered by the
        // next, and a single sort_order applies to every key (probes/sort-index-key-ties A1-A5).
        // A key may repeat, but more keys than the array has columns (rows, by_col) is `#VALUE!`
        // (A6, A15-A19), which also bounds the comparison's work by the array's size.
        let extent = if by_col { num_rows } else { num_cols };
        if raw_index.len() > extent || (raw_order.len() > 1 && raw_order.len() != raw_index.len()) {
            return CalcResult::new_error(
                Error::VALUE,
                cell,
                "more sort keys than the array has, or a sort_order of another length".to_string(),
            );
        }
        let mut keys: Vec<(usize, bool)> = Vec::with_capacity(raw_index.len());
        for (k, &index) in raw_index.iter().enumerate() {
            let index = index as i64;
            if index < 1 {
                return CalcResult::new_error(
                    Error::VALUE,
                    cell,
                    "sort_index must be >= 1".to_string(),
                );
            }
            let order = raw_order
                .get(k)
                .or(raw_order.first())
                .copied()
                .unwrap_or(1.0);
            let ascending = match Self::validate_sort_order(order as i32, cell) {
                Ok(a) => a,
                Err(e) => return e,
            };
            if index as usize > extent {
                return CalcResult::new_error(
                    Error::VALUE,
                    cell,
                    "sort_index exceeds the array".to_string(),
                );
            }
            keys.push((index as usize - 1, ascending));
        }

        if !by_col {
            // Sort rows by the key columns.
            let mut row_indices: Vec<usize> = (0..num_rows).collect();
            row_indices.sort_by(|&a, &b| {
                first_unequal(keys.iter().map(|&(key, ascending)| {
                    sort_key_cmp(&data[a][key], &data[b][key], ascending, cell)
                }))
            });

            CalcResult::Array(row_indices.iter().map(|&r| data[r].clone()).collect())
        } else {
            // Sort columns by the key rows.
            let mut col_indices: Vec<usize> = (0..num_cols).collect();
            col_indices.sort_by(|&a, &b| {
                first_unequal(keys.iter().map(|&(key, ascending)| {
                    sort_key_cmp(&data[key][a], &data[key][b], ascending, cell)
                }))
            });

            CalcResult::Array(
                data.iter()
                    .map(|row| col_indices.iter().map(|&c| row[c].clone()).collect())
                    .collect(),
            )
        }
    }

    /// SORT's `sort_index` or `sort_order` as a list of numbers: a one-row or one-column array or
    /// multi-cell range is each of its values, anything else one number. An error element and a
    /// 2-D list are invalid (NaN), so they fail validation as `#VALUE!` after the later
    /// arguments' errors: `SORT(D,IF({TRUE;FALSE},1,NA()),1/0)` is `#DIV/0!` and
    /// `SORT(D,{1,2;2,1})` `#VALUE!` (probes/sort-index-key-ties A8, A11, A20).
    fn sort_number_list(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> Result<Vec<f64>, CalcResult> {
        let value = self.evaluate_node_in_context(node, cell);
        let is_list = match &value {
            CalcResult::Array(_) => true,
            CalcResult::Range { left, right } => left != right,
            _ => false,
        };
        if !is_list {
            return Ok(vec![self.cast_to_number(value, cell)?]);
        }
        let grid = self.result_to_array(value, cell)?;
        if grid.len() > 1 && grid.first().is_some_and(|row| row.len() > 1) {
            return Ok(vec![f64::NAN]);
        }
        Ok(grid
            .iter()
            .flatten()
            .map(|node| {
                self.cast_number_ref(&array_node_to_calc_result(node, cell), cell)
                    .unwrap_or(f64::NAN)
            })
            .collect())
    }

    // ── SORTBY ────────────────────────────────────────────────────────────────

    /// `=SORTBY(array, by_array1, [sort_order1], [by_array2, sort_order2], ...)`
    ///
    /// Sorts the array's rows (or columns) using one or more external key arrays.
    ///   * `sort_order` values: 1 = ascending (default), -1 = descending
    ///   * `by_array` is a column as tall as array (sorting its rows) or a row as wide as it
    ///     (sorting its columns), the same way for every key
    ///   * Valid argument counts: 2, 3, 5, 7, … (additional keys in pairs)
    pub(crate) fn fn_sortby(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let kind = Function::Sortby;
        // Valid: 2, 3, 5, 7, ... arguments (whole by_array, sort_order pairs).
        if !kind
            .arg_kinds()
            .is_some_and(|kinds| kinds.accepts(args.len()))
        {
            return CalcResult::new_args_number_error(cell);
        }

        // An error array is lifted like any other: every element is that error, in the sort_orders'
        // shape (`ROWS(SORTBY(NA(),C1:C3,{1;-1}))` is 2; probes/sortby-lifted-element-corners F7,
        // excel-graph, T-2349).
        let data = self.eval_to_array(&args[0], cell);
        if data.as_ref().is_ok_and(|d| d.is_empty() || d[0].is_empty()) {
            return CalcResult::Array(vec![]);
        }

        // The array and the by_arrays are read once here; the sort_orders lift over an array and
        // a range (SPEC-0011 "Array-valued functions", T-1839). Each element sorts the whole
        // array, so the call is charged its cells and the by_arrays' per element.
        let keys: Vec<Result<SortByArray, CalcResult>> = (1..args.len())
            .step_by(2)
            .map(|i| self.eval_to_array(&args[i], cell))
            .collect();
        let cells = data.iter().flatten().map(Vec::len).sum::<usize>()
            + keys
                .iter()
                .flatten()
                .map(|key| key.iter().map(Vec::len).sum::<usize>())
                .sum::<usize>();
        let cells = u64::try_from(cells).unwrap_or(u64::MAX).max(1);
        self.lift_call_costed(
            kind,
            args,
            LiftCost::Cells(cells),
            cell,
            &mut |model, values| model.sortby_of(&data, &keys, values, cell),
        )
    }

    /// One SORTBY call over the read `data` and `keys`, with the `sort_orders` in `values` (at the
    /// argument positions 2, 4, ...). Every argument is taken left to right and the first error
    /// wins before any value is validated (probes/sort-error-args H73-H105;
    /// docs/excel-semantics/arrays.md).
    fn sortby_of(
        &mut self,
        data: &Result<Vec<Vec<ArrayNode>>, CalcResult>,
        keys: &[Result<SortByArray, CalcResult>],
        values: &[CalcResult],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        let data = match data {
            Ok(data) => data,
            Err(error) => return error.clone(),
        };
        let num_rows = data.len();
        let mut raw_keys: Vec<(&SortByArray, i32)> = Vec::with_capacity(keys.len());
        for (pair, key) in keys.iter().enumerate() {
            let key = match key {
                Ok(key) => key,
                Err(error) => return error.clone(),
            };
            let order = match values.get(2 + 2 * pair) {
                None | Some(CalcResult::EmptyArg) => 1,
                // A fraction reads as its floor, snapped up near an integer: -0.0001 and -0.9999
                // are -1 and 1.5 is 1, while 0.5, -1.5 and -1E-300 are `#VALUE!`
                // (probes/t2310-sortby-single-by-array H97-H165). The snap is `near_integer_whole`'s,
                // half the tolerance below zero: `-1-2^-23` is -1, `-1-2^-22` `#VALUE!`
                // (probes/t2370-sortby-mixed-by-array-snap, T-2370).
                Some(value) => match self.cast_to_number(value.clone(), cell) {
                    Ok(order) => near_integer_whole(order) as i32,
                    Err(error) => return error,
                },
            };
            raw_keys.push((key, order));
        }

        // Every by_array must sort the same way: `SORTBY(A1:A3,5,1,A1:A3,-1)` is `#VALUE!`
        // (probes/t2310-sortby-single-by-array H85-H93).
        let num_cols = data.first().map_or(0, Vec::len);
        let (mut by_rows, mut by_cols) = (true, true);
        let mut sort_keys: Vec<(&SortByArray, bool)> = Vec::with_capacity(raw_keys.len());
        for (key, order) in raw_keys {
            let (rows, cols) = by_array_axes(key, num_rows, num_cols);
            by_rows &= rows;
            by_cols &= cols;
            if !by_rows && !by_cols {
                return CalcResult::new_error(
                    Error::VALUE,
                    cell,
                    "by_array must be one column as tall or one row as wide as array".to_string(),
                );
            }
            let ascending = match Self::validate_sort_order(order, cell) {
                Ok(a) => a,
                Err(e) => return e,
            };
            sort_keys.push((key, ascending));
        }

        if by_rows {
            let mut row_indices: Vec<usize> = (0..num_rows).collect();
            row_indices.sort_by(|&a, &b| {
                first_unequal(
                    sort_keys.iter().map(|(key, ascending)| {
                        sort_key_cmp(&key[a][0], &key[b][0], *ascending, cell)
                    }),
                )
            });
            CalcResult::Array(row_indices.iter().map(|&r| data[r].clone()).collect())
        } else {
            let mut col_indices: Vec<usize> = (0..num_cols).collect();
            col_indices.sort_by(|&a, &b| {
                first_unequal(
                    sort_keys.iter().map(|(key, ascending)| {
                        sort_key_cmp(&key[0][a], &key[0][b], *ascending, cell)
                    }),
                )
            });
            CalcResult::Array(
                data.iter()
                    .map(|row| col_indices.iter().map(|&c| row[c].clone()).collect())
                    .collect(),
            )
        }
    }

    // ── UNIQUE ────────────────────────────────────────────────────────────────

    /// `=UNIQUE(array, [by_col], [exactly_once])`
    ///
    /// Returns the unique rows (or columns) from array.
    ///   * `by_col`       – FALSE = deduplicate rows, TRUE = deduplicate columns (default FALSE)
    ///   * `exactly_once` – FALSE = return one copy of every distinct value (default)
    ///     TRUE  = return only rows/columns that appear exactly once
    pub(crate) fn fn_unique(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let kind = Function::Unique;
        if !kind
            .arg_kinds()
            .is_some_and(|kinds| kinds.accepts(args.len()))
        {
            return CalcResult::new_args_number_error(cell);
        }
        // The array is read once and handed to every element whole; by_col and exactly_once lift
        // over an array and a range (SPEC-0011 "Array-valued functions", T-1836). Each element
        // walks the array, so the call is charged its cells per element.
        // An error array is every element's result: `UNIQUE(<#VALUE!>,HSTACK(0.2,1E+15))` is two
        // `#VALUE!` (gen-m7/8044dce50616cfe9 Calc!A1434:B1434; T-2300).
        let (data, cells) = self.unique_array(args, cell);
        let cells = match &data {
            Ok(d) if d.is_empty() || d[0].is_empty() => return CalcResult::Array(vec![]),
            Ok(_) => cells,
            Err(_) => 1,
        };
        let cells = u64::try_from(cells).unwrap_or(u64::MAX).max(1);
        self.lift_call_costed(
            kind,
            args,
            LiftCost::Cells(cells),
            cell,
            &mut |_, values| {
                let data = match &data {
                    Ok(data) => data,
                    Err(error) => return error.clone(),
                };
                let flag = |index: usize| match values.get(index) {
                    None | Some(CalcResult::EmptyArg) => Ok(false),
                    Some(value) => Self::cast_bool_ref(value, cell),
                };
                match (flag(1), flag(2)) {
                    (Ok(by_col), Ok(exactly_once)) => unique_of(data, by_col, exactly_once, cell),
                    (Err(error), _) | (_, Err(error)) => error,
                }
            },
        )
    }

    /// UNIQUE's array and its cell count. An element-wise expression of whole columns is read
    /// as its used head and two copies of its tail row (SPEC-0020 Amendment 1, T-0493): with
    /// rows as keys, the head followed by any `n >= 2` copies of one row has the same distinct
    /// rows, the same rows counted once and the same order as with two copies, and a tail is at
    /// least 4,096 rows. The count is the logical size, head plus tail, as the materialized
    /// array's.
    fn unique_array(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> (Result<Vec<Vec<ArrayNode>>, CalcResult>, usize) {
        let read = |model: &mut Self| model.eval_to_array_keep_blank(&args[0], cell);
        let tailed = if unique_flags_are_literals(args) {
            // An error evaluation is no head or tail: the ordinary reading below reports it.
            self.read_tailed(&args[0], cell, read)
                .and_then(|tailed| tailed.elements_by(Result::ok))
        } else {
            None
        };
        if let Some((head, tail)) = tailed {
            let mut rows: Vec<Vec<ArrayNode>> = head.map(|element| vec![element]).collect();
            let mut cells = rows.len();
            // No tail when the pass is being abandoned: the value is discarded.
            if let Some((element, tail_len)) = tail {
                cells = cells.saturating_add(tail_len);
                rows.push(vec![element.clone()]);
                rows.push(vec![element]);
            }
            return (Ok(rows), cells);
        }
        let data = read(self);
        let cells = data
            .as_ref()
            .map_or(0, |data| data.iter().map(Vec::len).sum::<usize>());
        (data, cells)
    }

    // ── FILTER ────────────────────────────────────────────────────────────────

    /// `=FILTER(array, include, [if_empty])`
    ///
    /// Returns only the rows of array where the corresponding include value is truthy.
    ///   * include  – a column vector of booleans/numbers with the same height as array
    ///   * `if_empty` – value returned when no rows pass the filter (default: #CALC! error)
    ///
    /// Note: include must be a range reference or inline array containing boolean/numeric
    /// values. For comparison-based filtering (e.g. A1:A5>3) store the comparison result
    /// in a helper column first.
    pub(crate) fn fn_filter(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() < 2 || args.len() > 3 {
            return CalcResult::new_args_number_error(cell);
        }

        let data = match self.eval_to_array(&args[0], cell) {
            Ok(d) => d,
            Err(e) => return e,
        };

        if data.is_empty() || data[0].is_empty() {
            return CalcResult::Array(vec![]);
        }

        let num_rows = data.len();
        let num_cols = data[0].len();

        let include = match self.eval_to_array(&args[1], cell) {
            Ok(d) => d,
            Err(e) => return e,
        };

        // Strings and errors are invalid as include mask elements.
        // Strings → #VALUE!; errors → propagate the error.
        for row in &include {
            for node in row {
                match node {
                    ArrayNode::Boolean(_) | ArrayNode::Number(_) | ArrayNode::Empty => {}
                    ArrayNode::String(_) | ArrayNode::Lambda(_) | ArrayNode::Nested(_) => {
                        return CalcResult::new_error(
                            Error::VALUE,
                            cell,
                            "include array must contain numbers or booleans".to_string(),
                        );
                    }
                    ArrayNode::Error(e) => {
                        return CalcResult::new_error(
                            e.clone(),
                            cell,
                            "include array contains an error".to_string(),
                        );
                    }
                }
            }
        }

        // Determine whether we are filtering rows or columns
        let include_flat: Vec<bool>;
        let filter_rows: bool;

        if include.len() == num_rows && include[0].len() == 1 {
            // Column vector — filter rows
            include_flat = include
                .iter()
                .map(|row| array_node_is_truthy(&row[0]))
                .collect();
            filter_rows = true;
        } else if include.len() == 1 && include[0].len() == num_rows {
            // Row vector with length == num_rows — also filter rows
            include_flat = include[0].iter().map(array_node_is_truthy).collect();
            filter_rows = true;
        } else if include.len() == 1 && include[0].len() == num_cols {
            // Row vector — filter columns
            include_flat = include[0].iter().map(array_node_is_truthy).collect();
            filter_rows = false;
        } else if include.len() == num_cols && include[0].len() == 1 {
            // Column vector with length == num_cols — filter columns
            include_flat = include
                .iter()
                .map(|row| array_node_is_truthy(&row[0]))
                .collect();
            filter_rows = false;
        } else {
            return CalcResult::new_error(
                Error::VALUE,
                cell,
                "include array dimensions must match array rows or columns".to_string(),
            );
        }

        let filtered: Vec<Vec<ArrayNode>> = if filter_rows {
            data.into_iter()
                .enumerate()
                .filter_map(|(i, row)| if include_flat[i] { Some(row) } else { None })
                .collect()
        } else {
            let col_indices: Vec<usize> = include_flat
                .iter()
                .enumerate()
                .filter_map(|(i, &keep)| if keep { Some(i) } else { None })
                .collect();
            data.iter()
                .map(|row| col_indices.iter().map(|&c| row[c].clone()).collect())
                .collect()
        };

        let is_empty_result = filtered.is_empty() || filtered.first().is_none_or(|r| r.is_empty());

        if is_empty_result {
            if args.len() >= 3 {
                let empty_val = self.evaluate_node_in_context(&args[2], cell);
                match empty_val {
                    CalcResult::EmptyArg => CalcResult::new_error(
                        Error::CALC,
                        cell,
                        "No data returned by FILTER".to_string(),
                    ),
                    // A range is its values: FILTER gives values, never a reference, found or not
                    // (Excel-cached: upstream ISREF.xlsx Sheet1!D11, D14, D17, D20; probes/isref-names-filter-if-empty).
                    CalcResult::Range { left, right } => {
                        match self.evaluate_range(left, right, cell) {
                            Ok(array) => CalcResult::Array(array),
                            Err(error) => error,
                        }
                    }
                    // An array if_empty spills as is (Excel: probes/filter-if-empty-array).
                    CalcResult::Array(a) => CalcResult::Array(a),
                    // A scalar stays a scalar, not a 1x1 array: `MAX(FILTER(.., .., ""))` with no
                    // match is #VALUE! because MAX gets the text "" as a value (upstream
                    // MAX_MIN.xlsx H39, H40, H43).
                    v => v,
                }
            } else {
                CalcResult::new_error(Error::CALC, cell, "No data returned by FILTER".to_string())
            }
        } else {
            CalcResult::Array(filtered)
        }
    }
}
