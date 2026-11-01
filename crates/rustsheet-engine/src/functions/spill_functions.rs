use crate::area_budget::text_units;
use crate::functions::util::array_node_to_calc_result;
use crate::functions::util::near_integer::near_integer_whole;
use std::cmp::Ordering;
use std::collections::HashMap;

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

use super::util::collation::{cmp_text_pairwise, collation_key, CollationKey};
use super::util::{compare_values_pairwise, sort_indices_pairwise};

/// Compare two sort keys following Excel's rules:
///   Numbers < Strings < Booleans < Errors < Empty cells
/// Empty cells always sort last regardless of ascending/descending.
///
/// Texts compare as Excel's `<`, `=`, `>` do (`compare_values_pairwise`), which is not a total
/// order on texts Excel orders intransitively, so SORT and SORTBY sort with
/// [`sort_indices_pairwise`], which cannot panic (T-3211); GROUPBY's `key_cmp` compares texts
/// itself. On everything else it is a total order (grouping.rs
/// `sort_comparators_are_total_orders_on_mixed_types`, T-1694). The blank check must stay here:
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
            let cmp = compare_values_pairwise(&key_a, &key_b);
            if ascending {
                cmp.cmp(&0)
            } else {
                0.cmp(&cmp)
            }
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

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
    /// A text under UNIQUE ([`unique_cell_key`]).
    Collated(CollationKey),
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
///
/// Two texts are one value when Excel's `=` finds them equal ([`cmp_text_pairwise`]): case is
/// folded, `"ß"` is `"ss"` and `"á"` is `"a"&UNICHAR(769)`, but `"a"` and `"a"` plus 16 U+0302
/// (equal wrapped accent sums) are two (probes/t3211-collation-sort-cycles G1:J15,
/// t3181-collation-prefix-arrays B1, B2; docs/excel-semantics/coercion.md "Not a total order").
/// Lines are bucketed by their [`CellKey`]s, texts by [`collation_key`], which every pair the
/// rule finds equal shares, and a line joins the first kept line of its bucket the rule finds
/// equal to it. On an intransitive cycle Excel's grouping follows its sort order instead (G4);
/// that is unmatched (T-3211 Outcome).
///
/// Texts can share a key and still be unequal (`"a"` plus 16, 32, 48 ... U+0302), so a bucket may
/// hold many kept lines and the walk may compare each line with all of them: every comparison is
/// charged to the formula's work budget ([`text_units`] of the texts compared, at least one unit),
/// so a crafted column ends as the budget's `#VALUE!` instead of running quadratically
/// (directive 5; SPEC-0024).
fn unique_of(
    model: &mut Model,
    data: &[Vec<ArrayNode>],
    by_col: bool,
    exactly_once: bool,
    cell: CellReferenceIndex,
) -> CalcResult {
    let num_rows = data.len();
    let num_cols = data.first().map_or(0, Vec::len);
    let (count, length) = if by_col {
        (num_cols, num_rows)
    } else {
        (num_rows, num_cols)
    };
    let at = |line: usize, k: usize| {
        if by_col {
            &data[k][line]
        } else {
            &data[line][k]
        }
    };
    let same_texts = |a: usize, b: usize| {
        (0..length).all(|k| match (at(a, k), at(b, k)) {
            (ArrayNode::String(x), ArrayNode::String(y)) => {
                cmp_text_pairwise(x, y) == Ordering::Equal
            }
            _ => true,
        })
    };
    let compare_units = |a: usize, b: usize| {
        (0..length)
            .map(|k| match (at(a, k), at(b, k)) {
                (ArrayNode::String(x), ArrayNode::String(y)) => text_units(x.len() + y.len()),
                _ => 0,
            })
            .fold(1, u64::saturating_add)
    };
    // The kept lines, in order, with how many lines each stands for.
    let mut kept: Vec<(usize, usize)> = Vec::new();
    let mut buckets: HashMap<Vec<CellKey>, Vec<usize>> = HashMap::with_capacity(count);
    for line in 0..count {
        let key: Vec<CellKey> = (0..length).map(|k| unique_cell_key(at(line, k))).collect();
        let bucket = buckets.entry(key).or_default();
        let mut found = None;
        for &slot in bucket.iter() {
            let first = kept[slot].0;
            if let Err(error) = model.spend_work(compare_units(first, line), cell) {
                return error;
            }
            if same_texts(first, line) {
                found = Some(slot);
                break;
            }
        }
        match found {
            Some(slot) => kept[slot].1 += 1,
            None => {
                bucket.push(kept.len());
                kept.push((line, 1));
            }
        }
    }
    let lines: Vec<usize> = kept
        .into_iter()
        .filter(|&(_, n)| !exactly_once || n == 1)
        .map(|(line, _)| line)
        .collect();
    if lines.is_empty() {
        let what = if by_col { "columns" } else { "rows" };
        return CalcResult::new_error(
            Error::CALC,
            cell,
            format!("UNIQUE found no matching {what}"),
        );
    }
    if by_col {
        CalcResult::Array(
            data.iter()
                .map(|row| lines.iter().map(|&c| row[c].clone()).collect())
                .collect(),
        )
    } else {
        CalcResult::Array(lines.iter().map(|&r| data[r].clone()).collect())
    }
}

/// UNIQUE's [`cell_key`]: a text is keyed by its [`collation_key`] (see [`unique_of`]).
fn unique_cell_key(node: &ArrayNode) -> CellKey {
    match node {
        ArrayNode::String(text) => CellKey::Collated(collation_key(text)),
        other => cell_key(other),
    }
}

/// The first of `orderings` that is not `Equal`: the comparison of SORT's and SORTBY's keys,
/// each tie broken by the next key.
fn first_unequal(mut orderings: impl Iterator<Item = Ordering>) -> Ordering {
    orderings
        .find(|ordering| *ordering != Ordering::Equal)
        .unwrap_or(Ordering::Equal)
}

/// The elements of a SORT lifted over its `by_col`, `result`, given whether each call counts as a
/// sorting call, row-major. Calls that do not count keep their error; the first sorting call keeps
/// its top-left, the next is empty when `only_key_two` and `#VALUE!` otherwise, and every later
/// one is `#VALUE!`, whatever the `by_col` values and the shape (probes/t3090-sort-lifted-by-col-
/// broken-lead and the t2929 and t2947 probes refilled on 2026-10-29, T-3090).
// Excel: docs/excel-semantics/arrays.md "TOROW, TOCOL, SORT and TEXTJOIN lifted arguments".
fn sort_lifted_elements(result: CalcResult, sorted: &[bool], only_key_two: bool) -> CalcResult {
    let CalcResult::Array(mut rows) = result else {
        return result;
    };
    let columns = rows.first().map_or(0, Vec::len);
    if columns == 0 || rows.len().saturating_mul(columns) != sorted.len() {
        return CalcResult::Array(rows);
    }
    // Every sorting call but the first, which keeps its top-left.
    let mut broken = sorted
        .iter()
        .zip(rows.iter_mut().flatten())
        .filter_map(|(&sorted, node)| sorted.then_some(node))
        .skip(1);
    if let Some(node) = broken.next() {
        *node = if only_key_two {
            ArrayNode::Empty
        } else {
            ArrayNode::Error(Error::VALUE)
        };
    }
    for node in broken {
        *node = ArrayNode::Error(Error::VALUE);
    }
    CalcResult::Array(rows)
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

/// An argument that must be an array rejects the empty value a value-returning call gives for a
/// blank cell (`EmptyArg`: `CELL("contents",<blank>)`, `IFERROR(<blank>,5)`, `IFNA(..)`), through
/// IF, LET, CHOOSE, `@` and parentheses too, with `#VALUE!`, while a blank reference read through
/// those wrappers is a blank cell that stacks as 0 (`VSTACK(IFERROR(Data!C11,5),1)` and
/// `TOCOL(CELL("contents",Data!C11))` `#VALUE!`, `VSTACK(IF(TRUE,Data!C11),1)` 0;1;
/// probes/t3030-cell-contents-blank-shapers, probes/t3053-blank-reference-wrappers, excel-graph,
/// T-3053; information.md). An argument left out of the formula is not checked (unprobed).
pub(crate) fn reject_empty_value(
    arg: &Node,
    result: CalcResult,
    cell: CellReferenceIndex,
) -> Result<CalcResult, CalcResult> {
    if matches!(result, CalcResult::EmptyArg) && !matches!(arg, Node::EmptyArgKind) {
        return Err(CalcResult::new_error(
            Error::VALUE,
            cell,
            "an empty value is not an array".to_string(),
        ));
    }
    Ok(result)
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

    /// [`Model::eval_to_array`] for an argument that must be an array ([`reject_empty_value`]).
    pub(crate) fn eval_array_arg(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> Result<Vec<Vec<ArrayNode>>, CalcResult> {
        let result = self.evaluate_node_in_context(node, cell);
        let result = reject_empty_value(node, result, cell)?;
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

    /// [`Model::eval_array_arg`] that keeps a lone blank cell blank, as
    /// [`Model::eval_to_array_keep_blank`] does.
    pub(crate) fn eval_array_arg_keep_blank(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> Result<Vec<Vec<ArrayNode>>, CalcResult> {
        let result = self.evaluate_node_in_context(node, cell);
        let result = reject_empty_value(node, result, cell)?;
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
        // Whether each call counts as a sorting call, in row-major order (one entry per element:
        // every element runs, `Reuse::Never`).
        let mut sorted: Vec<bool> = Vec::new();
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
                let (result, read) = match (by_col, &raw_order) {
                    (Err(error), _) => (error, false),
                    (Ok(_), Err(error)) => (error.clone(), false),
                    (Ok(by_col), Ok(raw_order)) => (
                        model.sort_by_keys(&data, &raw_index, raw_order, by_col, cell),
                        true,
                    ),
                };
                // A call failing on its sort_index counts as a sorting call unless no call has
                // sorted yet, while an invalid by_col is always skipped (arrays.md, probes/
                // t2947-sort-lifted-by-col-corners).
                let failed = matches!(result, CalcResult::Error { .. });
                let sorted_before = sorted.contains(&true);
                sorted.push(read && (!failed || sorted_before));
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
            sort_indices_pairwise(&mut row_indices, |a, b| {
                first_unequal(keys.iter().map(|&(key, ascending)| {
                    sort_key_cmp(&data[a][key], &data[b][key], ascending, cell)
                }))
            });

            CalcResult::Array(row_indices.iter().map(|&r| data[r].clone()).collect())
        } else {
            // Sort columns by the key rows.
            let mut col_indices: Vec<usize> = (0..num_cols).collect();
            sort_indices_pairwise(&mut col_indices, |a, b| {
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
            sort_indices_pairwise(&mut row_indices, |a, b| {
                first_unequal(
                    sort_keys.iter().map(|(key, ascending)| {
                        sort_key_cmp(&key[a][0], &key[b][0], *ascending, cell)
                    }),
                )
            });
            CalcResult::Array(row_indices.iter().map(|&r| data[r].clone()).collect())
        } else {
            let mut col_indices: Vec<usize> = (0..num_cols).collect();
            sort_indices_pairwise(&mut col_indices, |a, b| {
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
            &mut |model, values| {
                let data = match &data {
                    Ok(data) => data,
                    Err(error) => return error.clone(),
                };
                let flag = |index: usize| match values.get(index) {
                    None | Some(CalcResult::EmptyArg) => Ok(false),
                    Some(value) => Self::cast_bool_ref(value, cell),
                };
                match (flag(1), flag(2)) {
                    (Ok(by_col), Ok(exactly_once)) => {
                        unique_of(model, data, by_col, exactly_once, cell)
                    }
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
}

#[cfg(test)]
mod tests {
    use super::unique_of;
    use crate::{
        calc_result::CalcResult, expressions::parser::ArrayNode,
        expressions::types::CellReferenceIndex, test::util::new_empty_model,
    };

    const CELL: CellReferenceIndex = CellReferenceIndex {
        sheet: 0,
        row: 1,
        column: 1,
    };

    /// `"a"` plus 16·k U+0302 all share one collation key (the marks' wrapped sum is 0) and are
    /// pairwise unequal (the prefix rule), so UNIQUE keeps every one and compares each with all the
    /// earlier ones: those comparisons are charged to the formula's work budget (T-3211), and a
    /// budget too small for them ends the call with its error.
    #[test]
    fn unique_charges_the_pairwise_walk_of_a_shared_key_bucket() {
        let n = 100;
        let data: Vec<Vec<ArrayNode>> = (0..n)
            .map(|k| {
                vec![ArrayNode::String(
                    format!("a{}", "\u{302}".repeat(16 * k)).into(),
                )]
            })
            .collect();
        let mut model = new_empty_model();
        let before = model.area_budget.work;
        let CalcResult::Array(kept) = unique_of(&mut model, &data, false, false, CELL) else {
            panic!("UNIQUE of distinct texts is an array");
        };
        assert_eq!(kept.len(), n);
        let comparisons = (n * (n - 1) / 2) as u64;
        assert!(before - model.area_budget.work >= comparisons);

        let mut model = new_empty_model();
        model.area_budget.work = comparisons / 2;
        let result = unique_of(&mut model, &data, false, false, CELL);
        assert!(matches!(result, CalcResult::Error { .. }));
        assert!(model.work_exceeded());
    }
}
