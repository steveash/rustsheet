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
//! leaves the argument to the ordinary reading (SPEC-0020 Amendment 1, T-0491). VLOOKUP, HLOOKUP
//! and MATCH over such a key, their other arguments fixed (ranges, one-cell references, literals),
//! are interior nodes too (SPEC-0020 Amendment 2, T-3221): the head's lift leaves a
//! [`LiftRecord`] that the tail's lift continues, so the two charge and check what the one lift
//! over the whole key would (`Model::lift_values_keyed`). XLOOKUP and XMATCH are keyed lookups
//! the same way (T-3223), and their tail also continues the head's regex state
//! (`Model::keyed_regexes`). A range operand
//! reads what `evaluate_range` reads: its used rows, the rest on record as read empty through
//! `clip_block_to_used_area`. The tail reads the one-cell references again, which finds them
//! evaluated and records nothing new. Should the used area grow while the head is evaluated (a
//! spill evaluated on the way), the head is too short: the argument is then evaluated the
//! ordinary way, its cells already evaluated.

use crate::calc_result::CalcResult;
use crate::expressions::parser::{ArrayNode, Node};
use crate::expressions::token::{Error, OpUnary};
use crate::expressions::types::CellReferenceIndex;
use crate::functions::text::LookupRegexes;
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
    /// The keyed lookups (VLOOKUP, HLOOKUP, MATCH over a tailed key), by argument slice address.
    lookups: Vec<usize>,
    /// What each keyed lookup's head lift charged and gave last, by argument slice address
    /// (SPEC-0020 Amendment 2, section 2).
    lifts: Vec<(usize, LiftRecord)>,
    /// Each XLOOKUP or XMATCH head's regex state, by argument slice address, which its tail
    /// continues (SPEC-0020 Amendment 2, section 3).
    regexes: Vec<(usize, LookupRegexes)>,
}

/// What a keyed lookup's head lift charged and gave last: the tail lift continues it, as the
/// one lift over the logical key would (SPEC-0020 Amendment 2, section 2).
#[derive(Clone)]
pub(crate) struct LiftRecord {
    /// The visits the head lift charged up front (its walks times the search's scan).
    pub(crate) visits: u64,
    /// The head's last element's lifted slots (`LiftedArg::element`; `None` for a scalar).
    pub(crate) last: Vec<Option<ArrayNode>>,
    /// The head's last element's result.
    pub(crate) result: ArrayNode,
}

/// Where a keyed lookup's lift stands while its argument is read as a head and a tail.
pub(crate) enum KeyedLift {
    /// The head is being evaluated: the lift runs as usual and leaves its [`LiftRecord`].
    Head,
    /// The tail is being evaluated: the head's record, if its lift left one.
    Tail(Option<LiftRecord>),
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
    lookups: Vec<usize>,
    sheets: Vec<u32>,
    top: i32,
    bottom: i32,
}

/// IF with its value arguments.
fn is_if(node: &Node) -> bool {
    matches!(node, Node::FunctionKind { kind: Function::If, args } if (2..=3).contains(&args.len()))
}

/// The arguments of a keyed lookup and how many of them past the key are tables: VLOOKUP or
/// HLOOKUP with 3 or 4 arguments, MATCH with 2 or 3 (SPEC-0020 Amendment 2, section 1; T-3221),
/// XMATCH with 2 to 4 and XLOOKUP with 3 to 6, whose `return_array` is a second table (section 3;
/// T-3223). Its one operand is its key, `args[0]`.
fn keyed_lookup(node: &Node) -> Option<(&[Node], usize)> {
    let Node::FunctionKind { kind, args } = node else {
        return None;
    };
    let (counts, tables) = match kind {
        Function::Vlookup | Function::Hlookup => (3..=4, 1),
        Function::Match => (2..=3, 1),
        Function::Xmatch => (2..=4, 1),
        Function::Xlookup => (3..=6, 2),
        _ => return None,
    };
    counts
        .contains(&args.len())
        .then_some((args.as_slice(), tables))
}

/// A keyed lookup's identity while its formula is evaluated: the address of its argument slice,
/// which the lookup function is called with. As for [`address`], it is only compared, never
/// dereferenced, and only while a view of the formula's cell holds it (`read_tailed` borrows the
/// parsed tree meanwhile, so it cannot move or be freed).
pub(crate) fn call_address(args: &[Node]) -> usize {
    args.as_ptr() as usize
}

/// Whether a keyed lookup's arguments past the key read the same in the head and the tail, and
/// as scalars where they could lift (SPEC-0020 Amendment 2, section 1): its `tables` (the first
/// arguments past the key) each a range, a one-cell reference or an array literal; every other
/// argument a literal (a negative number included) or a one-cell reference. A function call, a
/// name or an operator is refused, so an array `range_lookup`, `match_type`, `match_mode` or
/// `search_mode` never lifts with the key.
fn fixed_arguments(args: &[Node], tables: usize) -> bool {
    let scalar = |node: &Node| match ungroup(node) {
        Node::NumberKind(_)
        | Node::BooleanKind(_)
        | Node::StringKind(_)
        | Node::ReferenceKind { .. } => true,
        Node::UnaryKind {
            kind: OpUnary::Minus,
            right,
        } => matches!(ungroup(right), Node::NumberKind(_)),
        _ => false,
    };
    let table = |node: &Node| {
        matches!(
            ungroup(node),
            Node::RangeKind { .. } | Node::ReferenceKind { .. } | Node::ArrayKind(_)
        )
    };
    match args.get(1..) {
        Some(fixed) if fixed.len() >= tables => {
            let (tables, rest) = fixed.split_at(tables);
            tables.iter().all(table) && rest.iter().all(scalar)
        }
        _ => false,
    }
}

/// Pushes the operands of `node` onto `pending`, left first on top, when it is a value
/// operator, IF or a keyed lookup (its key only); `false` for any other node.
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
        _ => match keyed_lookup(node) {
            Some(([key, ..], _)) => {
                pending.push(key);
                true
            }
            _ => false,
        },
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
    /// whose evaluation is an array today: a value operator, IF or a keyed lookup at the top (never
    /// a bare range, which aggregates walk as a reference), every IF over a condition with a range
    /// or `ROW()` operand (a scalar condition picks one branch unevaluated), every keyed lookup
    /// over a key with one (a scalar key does not lift) and with [`fixed_arguments`], operands that
    /// are literals, one-cell references, single-column ranges all of one span of rows, or `ROW()`
    /// of such a range. Read from the formula alone; an explicit stack walks operator chains of any
    /// length, and never enters a keyed lookup's fixed arguments.
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
            lookups: Vec::new(),
            sheets: Vec::new(),
            top: 0,
            bottom: 0,
        };
        // The IF conditions and the lookup keys, each of which must reach a leaf.
        let mut conditions = Vec::new();
        while let Some(node) = pending.pop() {
            let node = ungroup(node);
            if let Some((args, tables)) = keyed_lookup(node) {
                if !fixed_arguments(args, tables) {
                    return None;
                }
                leaves.lookups.push(call_address(args));
            }
            if push_operands(node, &mut pending) {
                // `push_operands` took this function only as IF or a keyed lookup: its first
                // argument is the condition or the key.
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
            lookups: leaves.lookups,
            lifts: Vec::new(),
            regexes: Vec::new(),
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

impl Model {
    /// The innermost view whose argument holds the keyed lookup `call` of `cell`.
    fn keyed_view(&self, call: &[Node], cell: CellReferenceIndex) -> Option<usize> {
        if self.tail_views.is_empty() {
            return None;
        }
        let at = call_address(call);
        self.tail_views
            .iter()
            .rposition(|view| view.cell == cell && view.lookups.contains(&at))
    }

    /// Where the lift of the keyed lookup `call` (its arguments) of `cell` stands: `None` unless
    /// its argument's head or tail is being evaluated.
    pub(crate) fn keyed_lift(&self, call: &[Node], cell: CellReferenceIndex) -> Option<KeyedLift> {
        let view = self.tail_views.get(self.keyed_view(call, cell)?)?;
        let at = call_address(call);
        Some(match view.head_end {
            Some(_) => KeyedLift::Head,
            None => KeyedLift::Tail(
                view.lifts
                    .iter()
                    .find(|(key, _)| *key == at)
                    .map(|(_, record)| record.clone()),
            ),
        })
    }

    /// Keeps the head lift's `record` of the keyed lookup `call` of `cell` for its tail lift.
    pub(crate) fn record_keyed_lift(
        &mut self,
        call: &[Node],
        cell: CellReferenceIndex,
        record: LiftRecord,
    ) {
        let at = call_address(call);
        let Some(view) = self
            .keyed_view(call, cell)
            .and_then(|index| self.tail_views.get_mut(index))
        else {
            return;
        };
        let lifts = &mut view.lifts;
        lifts.retain(|(key, _)| *key != at);
        lifts.push((at, record));
    }

    /// The regex state an XLOOKUP or XMATCH call `call` of `cell` starts from: while its tailed
    /// argument's tail is evaluated, the head's (taken; once per tail), so the call's patterns,
    /// budgets and charges run on as in its one call over the logical key; otherwise a new one
    /// (SPEC-0020 Amendment 2, section 3).
    pub(crate) fn keyed_regexes(
        &mut self,
        call: &[Node],
        cell: CellReferenceIndex,
    ) -> LookupRegexes {
        let at = call_address(call);
        let view = self
            .keyed_view(call, cell)
            .and_then(|index| self.tail_views.get_mut(index))
            .filter(|view| view.head_end.is_none());
        view.and_then(|view| {
            let index = view.regexes.iter().position(|(key, _)| *key == at)?;
            Some(view.regexes.swap_remove(index).1)
        })
        .unwrap_or_else(LookupRegexes::new)
    }

    /// Keeps the regex state `regexes` of the XLOOKUP or XMATCH call `call` of `cell`, already
    /// finished, for its tail when its head is being evaluated; drops it otherwise.
    pub(crate) fn keep_keyed_regexes(
        &mut self,
        call: &[Node],
        cell: CellReferenceIndex,
        regexes: LookupRegexes,
    ) {
        let at = call_address(call);
        let Some(view) = self
            .keyed_view(call, cell)
            .and_then(|index| self.tail_views.get_mut(index))
            .filter(|view| view.head_end.is_some())
        else {
            return;
        };
        view.regexes.retain(|(key, _)| *key != at);
        view.regexes.push((at, regexes));
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
