//! The `#SPILL!` of a dynamic-array anchor whose formula is a lifted call (`=DATE(11:1899,12,1)`),
//! a unary operator or `%` over a range (`=-H:J`), decided from the formula's reference shapes
//! before anything is lifted (T-0483). Excel builds such an array and cannot spill it past the
//! grid, where the engine would refuse to build it (`#ERROR!`, the array bound of decision 0014):
//! probes/lifted-range-rows C3; docs/excel-semantics/arrays.md "Array results above 1,048,576
//! cells are #ERROR!". The operator broadcasts of the same rule are
//! `Model::evaluate_spilling_chain` (arithmetic.rs).

use crate::{
    arithmetic::MAX_SPILL_CHECK_DEPTH,
    calc_result::{range_dims, CalcResult},
    expressions::{parser::Node, types::CellReferenceIndex},
    functions::lift::{ArgKind, CallResult},
    limits::unspillable_shape,
    model::Model,
};

impl Model {
    /// `#SPILL!` when `node`, the root or an operator operand of the formula of the dynamic-array
    /// anchor `anchor`, is a lifted call or a unary operator whose result has at least the shape
    /// [`Model::lifted_shape_floor`] reads, more cells than [`check_array_cells`](crate::limits::check_array_cells) allows and a
    /// shape that runs past the grid from `anchor`. A lift never shrinks its arguments' shapes,
    /// so the whole result could not spill either. Within the bound the array is built and the
    /// spill itself decides.
    pub(crate) fn unspillable_lift(
        &mut self,
        node: &Node,
        anchor: CellReferenceIndex,
    ) -> Option<CalcResult> {
        let (rows, columns) = self.lifted_shape_floor(node, anchor, 0)?;
        unspillable_shape(anchor, rows, columns)
    }

    /// A shape that the lifted result of `node` is at least as tall and as wide as, read from its
    /// references alone, with no cell read: a range is its own shape, a unary operator or `%`
    /// lifts its operand, and a call in the lifting table of one value per element broadcasts its
    /// `Lift` arguments that have a shape (a valid argument count only). Any other node is `None`,
    /// as are nodes more than [`MAX_SPILL_CHECK_DEPTH`] levels down (CLAUDE.md directive 5).
    fn lifted_shape_floor(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
        depth: usize,
    ) -> Option<(usize, usize)> {
        if depth > MAX_SPILL_CHECK_DEPTH {
            return None;
        }
        match node.ungrouped() {
            Node::UnaryKind { right, .. } => self.lifted_shape_floor(right, cell, depth + 1),
            Node::FunctionKind { kind, args } => {
                let kinds = kind.arg_kinds()?;
                // An array-valued entry (SORTBY, UNIQUE) may answer before its lift (SORTBY gives
                // its array's error first), so only a call of one value per element counts.
                if kinds.result != CallResult::Value || !kinds.accepts(args.len()) {
                    return None;
                }
                let mut shape = None;
                for (index, arg) in args.iter().enumerate() {
                    if kinds.kind(index) != Some(ArgKind::Lift) {
                        continue;
                    }
                    if let Some((rows, columns)) = self.lifted_shape_floor(arg, cell, depth + 1) {
                        let (most_rows, most_columns) = shape.unwrap_or((1, 1));
                        shape = Some((most_rows.max(rows), most_columns.max(columns)));
                    }
                }
                shape
            }
            range @ Node::RangeKind { .. } => {
                match self.evaluate_node_with_reference(range, cell) {
                    CalcResult::Range { left, right } if left.sheet == right.sheet => {
                        Some(range_dims(left, right))
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }
}
