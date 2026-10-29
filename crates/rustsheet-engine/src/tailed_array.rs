//! Whole-column array operands as a used head and one repeated tail row (SPEC-0020, T-0469).
//!
//! `MAX(IF(F:F=A1,G:G,""))` used to build several 1,048,576-row arrays per call, nearly all of
//! them past the sheet's used area. An aggregate argument that is an element-wise expression of
//! whole columns is evaluated twice instead, by the ordinary evaluator: once with every
//! whole-column operand read down to the last used row of the operands' sheets (the head), and
//! once with every such operand an empty one-element array (the tail: the value of every row
//! below the head, where each operand is empty and every other operand is what it is in every
//! row). The aggregate walk then visits the head's elements and the tail element once per row
//! past the head: the sequence the 1,048,576-row array gives, so results and errors are the
//! walk's. SUMPRODUCT reads its arguments the same way (T-0471): each tailed argument keeps its
//! logical 1,048,576-row shape, and the rows past every head share one product.
//!
//! Only expressions whose rows are independent of one another are taken
//! ([`Model::tailed_leaves`]): literals, one-cell references, single-column ranges of one common
//! span (whole columns, or bounded ranges running past the used area), the value operators and IF
//! over a condition that is an array. `ROW()` of such a range is a position leaf: its head is its
//! row numbers and its tail the sentinel `#N/IMPL`, and a tail element that is the sentinel
//! leaves the argument to the ordinary reading (SPEC-0020 Amendment 1, T-0491). A range operand
//! reads what `evaluate_range` reads: its used rows, the rest on record as read empty through
//! `clip_block_to_used_area`. The tail reads the one-cell references again, which finds them
//! evaluated and records nothing new. Should the used area grow while the head is evaluated (a
//! spill evaluated on the way), the head is too short: the argument is then evaluated the
//! ordinary way, its cells already evaluated.

use crate::calc_result::CalcResult;
use crate::expressions::parser::{ArrayNode, Node};
use crate::expressions::token::Error;
use crate::expressions::types::CellReferenceIndex;
use crate::functions::Function;
use crate::model::Model;
use crate::reference_operators::RefArg;

#[cfg(test)]
thread_local! {
    /// Tests turn the tailed reading off to compare it with the ordinary one.
    pub(crate) static TAILED_OFF: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    /// How many arguments were walked as a head and a tail.
    pub(crate) static TAILED_WALKS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// Fewest rows past the used area for which the two evaluations beat one ordinary evaluation.
const MIN_TAILED_ROWS: i32 = 4096;

/// The tail of a `ROW()` position leaf: an error no tailed operand row can hide, so a tail
/// element that depends on the row numbers is this error (SPEC-0020 Amendment 1, section 2). A
/// real `#N/IMPL` in the tail only sends the argument to the ordinary reading.
const POSITION_SENTINEL: Error = Error::NIMPL;

/// The whole-column operands of an argument whose head or tail is being evaluated.
pub(crate) struct TailView {
    /// The formula's cell: formulas evaluated meanwhile may share its parsed nodes.
    cell: CellReferenceIndex,
    /// The range operand nodes, by address (compared, never dereferenced).
    leaves: Vec<usize>,
    /// The `ROW()` position leaves, by address.
    positions: Vec<usize>,
    /// The first row of the operands' common span.
    top: i32,
    /// The head's last row while the head is evaluated, `None` for the tail.
    head_end: Option<i32>,
    /// Set when an operand's used rows went past `head_end`.
    stale: bool,
}

/// An aggregate argument read as a head and a tail: the head's elements, then `tail_len` copies
/// of the tail's one element. No tail when the pass is being abandoned (its value is discarded).
pub(crate) struct Tailed<T> {
    pub(crate) head: T,
    pub(crate) tail: Option<T>,
    pub(crate) tail_len: usize,
    /// The argument has `ROW()` position leaves: a sentinel tail element has no single value.
    pub(crate) positions: bool,
}

impl Tailed<RefArg> {
    /// The elements of a tailed aggregate or SUMPRODUCT argument ([`Tailed::elements_by`] of
    /// its arrays).
    pub(crate) fn elements(
        self,
    ) -> Option<(impl Iterator<Item = ArrayNode>, Option<(ArrayNode, usize)>)> {
        self.elements_by(|arg| match arg.into_ranges_or_value() {
            Err(CalcResult::Array(array)) => Some(array),
            _ => None,
        })
    }
}

impl<T> Tailed<T> {
    /// The elements of a tailed argument, each evaluation read as an array by `as_array`: its
    /// head's, and its tail element with the number of rows it stands for; `None` unless the
    /// head is one column of values and the tail one element, as the two evaluations of an
    /// array argument give. Tests count the arguments read this way (`TAILED_WALKS`).
    pub(crate) fn elements_by(
        self,
        as_array: impl Fn(T) -> Option<Vec<Vec<ArrayNode>>>,
    ) -> Option<(impl Iterator<Item = ArrayNode>, Option<(ArrayNode, usize)>)> {
        let head = as_array(self.head)?;
        if head.iter().any(|row| row.len() != 1) {
            return None;
        }
        let tail = match self.tail {
            None => None,
            Some(tail) => {
                let [row] = <[Vec<ArrayNode>; 1]>::try_from(as_array(tail)?).ok()?;
                let [element] = <[ArrayNode; 1]>::try_from(row).ok()?;
                if self.positions && element == ArrayNode::Error(POSITION_SENTINEL) {
                    return None;
                }
                Some((element, self.tail_len))
            }
        };
        #[cfg(test)]
        TAILED_WALKS.with(|walks| walks.set(walks.get() + 1));
        Some((head.into_iter().flatten(), tail))
    }
}

/// The node under its parentheses and unary pluses. Unlike `Node::ungrouped`, a structured
/// reference stays itself, so a table column is never taken for a whole column (it keeps the
/// ordinary reading).
fn ungroup(mut node: &Node) -> &Node {
    while let Node::Group { child, .. } | Node::UnaryPlus { child } | Node::Spaced { child, .. } =
        node
    {
        node = child;
    }
    node
}

/// A node's identity while its formula is evaluated: the address is only compared, never
/// dereferenced, and only while the argument it was taken from is borrowed by `read_tailed`, so
/// the parsed tree cannot move or be freed meanwhile.
fn address(node: &Node) -> usize {
    std::ptr::from_ref(node) as usize
}

/// `ROW(range)`, and the range node.
fn row_of(node: &Node) -> Option<&Node> {
    match node {
        Node::FunctionKind {
            kind: Function::Row,
            args,
        } if args.len() == 1 => Some(ungroup(&args[0])),
        _ => None,
    }
}

/// The leaves of a tailed argument: range and position nodes by address, their sheets and their
/// common span of rows.
struct Leaves {
    ranges: Vec<usize>,
    positions: Vec<usize>,
    sheets: Vec<u32>,
    top: i32,
    bottom: i32,
}

/// IF with its value arguments.
fn is_if(node: &Node) -> bool {
    matches!(node, Node::FunctionKind { kind: Function::If, args } if (2..=3).contains(&args.len()))
}

/// Pushes the operands of `node` onto `pending`, left first on top, when it is a value
/// operator or IF; `false` for any other node.
fn push_operands<'a>(node: &'a Node, pending: &mut Vec<&'a Node>) -> bool {
    match node {
        Node::OpSumKind { left, right, .. }
        | Node::OpProductKind { left, right, .. }
        | Node::OpPowerKind { left, right, .. }
        | Node::OpConcatenateKind { left, right }
        | Node::CompareKind { left, right, .. } => {
            pending.push(right);
            pending.push(left);
            true
        }
        Node::UnaryKind { right, .. } => {
            pending.push(right);
            true
        }
        Node::FunctionKind { args, .. } if is_if(node) => {
            pending.extend(args.iter().rev());
            true
        }
        _ => false,
    }
}

/// Whether `node`'s operands, at any depth, include one of `leaves`.
fn reaches_leaf(node: &Node, leaves: &[usize]) -> bool {
    let mut pending = vec![node];
    while let Some(node) = pending.pop() {
        let node = ungroup(node);
        if leaves.contains(&address(node)) {
            return true;
        }
        push_operands(node, &mut pending);
    }
    false
}

impl Model {
    /// The range and `ROW()` operands of `arg`, when `arg` is an element-wise expression of them
    /// whose evaluation is an array today: a value operator or IF at the top (never a bare range,
    /// which aggregates walk as a reference), every IF over a condition with a range or `ROW()`
    /// operand (a scalar condition picks one branch unevaluated), operands that are literals,
    /// one-cell references, single-column ranges all of one span of rows, or `ROW()` of such a
    /// range. Read from the formula alone; an explicit stack walks operator chains of any length.
    fn tailed_leaves(&self, arg: &Node, cell: CellReferenceIndex) -> Option<Leaves> {
        let top = ungroup(arg);
        let mut pending = Vec::new();
        if !push_operands(top, &mut pending) {
            return None;
        }
        pending.clear();
        pending.push(top);
        let mut leaves = Leaves {
            ranges: Vec::new(),
            positions: Vec::new(),
            sheets: Vec::new(),
            top: 0,
            bottom: 0,
        };
        let mut conditions = Vec::new();
        while let Some(node) = pending.pop() {
            let node = ungroup(node);
            if push_operands(node, &mut pending) {
                // `push_operands` took this function only as IF with 2 or 3 arguments.
                if let Node::FunctionKind { args, .. } = node {
                    conditions.push(&args[0]);
                }
                continue;
            }
            let range = row_of(node).unwrap_or(node);
            match range {
                Node::NumberKind(_)
                | Node::BooleanKind(_)
                | Node::StringKind(_)
                | Node::ReferenceKind { .. }
                    if std::ptr::eq(range, node) => {}
                Node::RangeKind { .. } => {
                    let (left, right) = range_corners(range, cell)?;
                    if left.column != right.column {
                        return None;
                    }
                    let first = leaves.ranges.is_empty() && leaves.positions.is_empty();
                    if first {
                        (leaves.top, leaves.bottom) = (left.row, right.row);
                    } else if (leaves.top, leaves.bottom) != (left.row, right.row) {
                        // Ragged operands broadcast and pad with #N/A: one head cannot say that.
                        return None;
                    }
                    if std::ptr::eq(range, node) {
                        leaves.ranges.push(address(node));
                    } else {
                        leaves.positions.push(address(node));
                    }
                    if !leaves.sheets.contains(&left.sheet) {
                        leaves.sheets.push(left.sheet);
                    }
                }
                _ => return None,
            }
        }
        let arrays = conditions.into_iter().all(|condition| {
            reaches_leaf(condition, &leaves.ranges) || reaches_leaf(condition, &leaves.positions)
        });
        (!leaves.ranges.is_empty() && arrays).then_some(leaves)
    }

    /// `arg` read by `read` (the aggregate walk's own reading of it) as a head and a tail, or
    /// `None` when it is not tailed: then nothing was read and the walk reads it as usual.
    pub(crate) fn read_tailed<T>(
        &mut self,
        arg: &Node,
        cell: CellReferenceIndex,
        mut read: impl FnMut(&mut Self) -> T,
    ) -> Option<Tailed<T>> {
        // Cycles under iterative calculation and Excel-order stops read a cell twice
        // differently.
        if self.iterating() || self.evaluation.excel_order || self.evaluation.blocking {
            return None;
        }
        #[cfg(test)]
        if TAILED_OFF.with(std::cell::Cell::get) {
            return None;
        }
        let leaves = self.tailed_leaves(arg, cell)?;
        let mut used_end = 1;
        for sheet in &leaves.sheets {
            used_end = used_end.max(self.used_area_end_of(*sheet).ok()?.0);
        }
        // The head is at least the span's first row (`max` then `min`: never a panic, whatever
        // the span).
        let head_end = used_end.max(leaves.top).min(leaves.bottom);
        if leaves.bottom - head_end < MIN_TAILED_ROWS {
            return None;
        }
        let positions = !leaves.positions.is_empty();
        self.tail_views.push(TailView {
            cell,
            leaves: leaves.ranges,
            positions: leaves.positions,
            top: leaves.top,
            head_end: Some(head_end),
            stale: false,
        });
        let head = read(self);
        let stale = self.tail_views.last().is_none_or(|view| view.stale);
        // A pass being abandoned discards the value; the tail would read in vain.
        let abandoned = self.evaluation.abandoned() || self.evaluation.unwinding;
        if stale || abandoned {
            self.tail_views.pop();
            return (!stale).then_some(Tailed {
                head,
                tail: None,
                tail_len: 0,
                positions,
            });
        }
        if let Some(view) = self.tail_views.last_mut() {
            view.head_end = None;
        }
        let tail = read(self);
        self.tail_views.pop();
        Some(Tailed {
            head,
            tail: Some(tail),
            tail_len: usize::try_from(leaves.bottom - head_end).unwrap_or(0),
            positions,
        })
    }

    /// What the whole-column operand `node` evaluates to (`range` at `cell`) while the head or
    /// the tail of its argument is evaluated; `None` when neither is.
    pub(crate) fn tailed_operand(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
        range: &CalcResult,
    ) -> Option<CalcResult> {
        if self.tail_views.is_empty() {
            return None;
        }
        let at = address(node);
        let index = self
            .tail_views
            .iter()
            .rposition(|view| view.cell == cell && view.leaves.contains(&at))?;
        let CalcResult::Range { left, right } = range else {
            return None;
        };
        let Some(head_end) = self.tail_views[index].head_end else {
            return Some(CalcResult::Array(vec![vec![ArrayNode::Empty]]));
        };
        // Past the used rows, on record as read empty: what `evaluate_range` records for a whole
        // column, and the union of the empty cells it reads one by one for a bounded range. A
        // block the seen-range budget cannot record is not cut, so the head goes stale.
        let (last_row, _) = self
            .clip_block_to_used_area(left.sheet, left.row, left.column, right.row, right.column)
            .ok()?;
        if last_row > head_end {
            self.tail_views[index].stale = true;
            return None;
        }
        let head_right = CellReferenceIndex {
            row: head_end,
            ..*right
        };
        Some(match self.evaluate_range(*left, head_right, cell) {
            Ok(array) => CalcResult::Array(array),
            Err(error) => error,
        })
    }
}

impl Model {
    /// What the `ROW()` position leaf `node` evaluates to while the head or the tail of its
    /// argument is evaluated: the head's row numbers, or the sentinel; `None` when neither is.
    pub(crate) fn tailed_position(
        &self,
        node: &Node,
        kind: &Function,
        cell: CellReferenceIndex,
    ) -> Option<CalcResult> {
        if *kind != Function::Row || self.tail_views.is_empty() {
            return None;
        }
        let at = address(node);
        let view = self
            .tail_views
            .iter()
            .rev()
            .find(|view| view.cell == cell && view.positions.contains(&at))?;
        Some(CalcResult::Array(match view.head_end {
            Some(head_end) => (view.top..=head_end)
                .map(|row| vec![ArrayNode::Number(f64::from(row))])
                .collect(),
            None => vec![vec![ArrayNode::Error(POSITION_SENTINEL)]],
        }))
    }
}

/// The corners of the range `node` (a `Node::RangeKind`) written in the formula of `cell`:
/// relative corners are offsets from the cell, and the corners are put in order.
pub(crate) fn range_corners(
    node: &Node,
    cell: CellReferenceIndex,
) -> Option<(CellReferenceIndex, CellReferenceIndex)> {
    let Node::RangeKind {
        sheet_index,
        row1,
        column1,
        row2,
        column2,
        absolute_column1,
        absolute_row2,
        absolute_row1,
        absolute_column2,
        sheet_name: _,
    } = node
    else {
        return None;
    };
    let r1 = *row1 + if *absolute_row1 { 0 } else { cell.row };
    let r2 = *row2 + if *absolute_row2 { 0 } else { cell.row };
    let c1 = *column1 + if *absolute_column1 { 0 } else { cell.column };
    let c2 = *column2 + if *absolute_column2 { 0 } else { cell.column };
    Some((
        CellReferenceIndex {
            sheet: *sheet_index,
            row: r1.min(r2),
            column: c1.min(c2),
        },
        CellReferenceIndex {
            sheet: *sheet_index,
            row: r1.max(r2),
            column: c1.max(c2),
        },
    ))
}
