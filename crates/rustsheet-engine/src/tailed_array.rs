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
//! walk's.
//!
//! Only expressions whose rows are independent of one another are taken
//! ([`Model::tailed_leaves`]): literals, one-cell references, single whole columns, the value
//! operators and IF over a condition that is an array. A whole-column operand reads what
//! `evaluate_range` reads: its used rows, the rest on record as read empty through
//! `clip_to_used_area`. The tail reads the one-cell references again, which finds them evaluated
//! and records nothing new. Should the used area grow while the head is evaluated (a spill
//! evaluated on the way), the head is too short: the argument is then evaluated the ordinary
//! way, its cells already evaluated.

use crate::calc_result::CalcResult;
use crate::constants::LAST_ROW;
use crate::expressions::parser::{ArrayNode, Node};
use crate::expressions::types::CellReferenceIndex;
use crate::functions::Function;
use crate::model::Model;

#[cfg(test)]
thread_local! {
    /// Tests turn the tailed reading off to compare it with the ordinary one.
    pub(crate) static TAILED_OFF: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    /// How many arguments were walked as a head and a tail.
    pub(crate) static TAILED_WALKS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// Fewest rows past the used area for which the two evaluations beat one ordinary evaluation.
const MIN_TAILED_ROWS: i32 = 4096;

/// The whole-column operands of an argument whose head or tail is being evaluated.
pub(crate) struct TailView {
    /// The formula's cell: formulas evaluated meanwhile may share its parsed nodes.
    cell: CellReferenceIndex,
    /// The whole-column operand nodes, by address (compared, never dereferenced).
    leaves: Vec<usize>,
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
    /// The whole-column operands of `arg` (node addresses) and their sheets, when `arg` is an
    /// element-wise expression of them whose evaluation is an array today: a value operator or
    /// IF at the top (never a bare range, which aggregates walk as a reference), every IF over
    /// a condition with a whole-column operand (a scalar condition picks one branch
    /// unevaluated), operands that are literals, one-cell references or single whole columns.
    /// Read from the formula alone; an explicit stack walks operator chains of any length.
    fn tailed_leaves(
        &self,
        arg: &Node,
        cell: CellReferenceIndex,
    ) -> Option<(Vec<usize>, Vec<u32>)> {
        let top = ungroup(arg);
        let mut pending = Vec::new();
        if !push_operands(top, &mut pending) {
            return None;
        }
        pending.clear();
        pending.push(top);
        let mut leaves = Vec::new();
        let mut sheets = Vec::new();
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
            match node {
                Node::NumberKind(_)
                | Node::BooleanKind(_)
                | Node::StringKind(_)
                | Node::ReferenceKind { .. } => {}
                Node::RangeKind { .. } => {
                    let (left, right) = range_corners(node, cell)?;
                    if left.row != 1 || right.row != LAST_ROW || left.column != right.column {
                        return None;
                    }
                    leaves.push(address(node));
                    if !sheets.contains(&left.sheet) {
                        sheets.push(left.sheet);
                    }
                }
                _ => return None,
            }
        }
        let arrays = conditions
            .into_iter()
            .all(|condition| reaches_leaf(condition, &leaves));
        (!leaves.is_empty() && arrays).then_some((leaves, sheets))
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
        let (leaves, sheets) = self.tailed_leaves(arg, cell)?;
        let mut head_end = 1;
        for sheet in sheets {
            head_end = head_end.max(self.used_area_end_of(sheet).ok()?.0);
        }
        if LAST_ROW - head_end < MIN_TAILED_ROWS {
            return None;
        }
        self.tail_views.push(TailView {
            cell,
            leaves,
            head_end: Some(head_end),
            stale: false,
        });
        let head = read(self);
        let stale = self.tail_views.last().is_none_or(|view| view.stale);
        // A pass being abandoned discards the value; the tail would read in vain.
        let abandoned = self.evaluation.restart.is_some() || self.evaluation.unwinding;
        if stale || abandoned {
            self.tail_views.pop();
            return (!stale).then_some(Tailed {
                head,
                tail: None,
                tail_len: 0,
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
            tail_len: usize::try_from(LAST_ROW - head_end).unwrap_or(0),
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
        // The record `evaluate_range` makes: past the used rows, read empty.
        let (last_row, _) = self
            .clip_to_used_area(left.sheet, left.row, left.column, right.row, right.column)
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
