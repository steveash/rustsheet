mod and_or_xor_not;
mod bycol_byrow;
mod lambda;
mod r#let;
mod map_reduce;
mod scan;
mod switch;

pub(crate) use r#let::{has_duplicate_parameter, same_variable};

use crate::{
    arithmetic::bcast_idx,
    calc_result::CalcResult,
    expressions::{
        parser::{ArrayNode, Node},
        token::Error,
        types::CellReferenceIndex,
    },
    limits::check_array_size,
    model::Model,
    reference_operators::{Picked, RefArg},
};

/// How IF hands back the branch a scalar condition chose.
#[derive(Clone, Copy)]
pub(crate) enum IfBranchMode {
    /// As a value (an ordinary IF).
    Value,
    /// As an aggregate argument sees it: a reference stays a range.
    Aggregate,
    /// As an argument that needs one sheet's reference (INDEX, MATCH): a 3-D span is `#VALUE!`.
    SingleSheet,
    /// As a cell-reading argument sees it ([`Model::evaluate_chosen_reference`]): a nested IFS,
    /// SWITCH, LET or CHOOSE branch stays the cell it picks.
    Chosen,
}

// ── IF array helpers ──────────────────────────────────────────────────────────

/// A branch argument to IF/IFERROR/IFNA (and an arm of CHOOSE over an index array): either a
/// scalar (broadcast to all positions) or a 2-D array (indexed by position with size-1
/// broadcasting; a ragged out-of-range position → #N/A). See [`if_arg_at`].
pub(crate) enum IfArg {
    Scalar(ArrayNode),
    Array(Vec<Vec<ArrayNode>>),
}

pub(crate) fn if_arg_dims(arg: &IfArg) -> (usize, usize) {
    match arg {
        IfArg::Scalar(_) => (1, 1),
        IfArg::Array(arr) => (arr.len(), arr.first().map_or(0, |r| r.len())),
    }
}

/// Index a branch argument at output position (row, col), applying Excel's
/// size-1 broadcasting rule: a scalar covers every position, a length-1
/// dimension repeats its only element to cover the whole output extent, and a
/// ragged mismatch (e.g. length 2 against an extent of 3) yields `#N/A`.
pub(crate) fn if_arg_at(arg: &IfArg, row: usize, col: usize) -> ArrayNode {
    match arg {
        IfArg::Scalar(node) => node.clone(),
        IfArg::Array(arr) => {
            let rows = arr.len();
            let cols = arr.first().map_or(0, |r| r.len());
            match bcast_idx(rows, row)
                .and_then(|r| arr.get(r))
                .and_then(|r| bcast_idx(cols, col).and_then(|c| r.get(c)))
            {
                Some(n) => n.clone(),
                None => ArrayNode::Error(Error::NA),
            }
        }
    }
}

fn calc_result_to_array_node(result: CalcResult) -> ArrayNode {
    match result {
        CalcResult::Number(n) => ArrayNode::Number(n),
        CalcResult::Boolean(b) => ArrayNode::Boolean(b),
        CalcResult::String(s) => ArrayNode::String(s),
        CalcResult::Error { error, .. } => ArrayNode::Error(error),
        CalcResult::EmptyCell | CalcResult::EmptyArg => ArrayNode::Empty,
        CalcResult::Lambda(_) | CalcResult::Range { .. } | CalcResult::Array(_) => {
            ArrayNode::Error(Error::VALUE)
        }
    }
}

/// Whether an argument value makes IFS or SWITCH run element by element: an array or a range.
fn is_lifted(value: &CalcResult) -> bool {
    matches!(value, CalcResult::Range { .. } | CalcResult::Array(_))
}

/// IFS's and SWITCH's `#N/A` when nothing matches.
fn no_match(cell: CellReferenceIndex) -> CalcResult {
    CalcResult::Error {
        error: Error::NA,
        origin: cell,
        message: "Did not find a match".to_string(),
    }
}

/// Cast one element of a cond array to bool. Returns Err with the error node to
/// place in the output when the element is itself an error or unparseable string.
fn array_node_to_bool(node: &ArrayNode) -> Result<bool, ArrayNode> {
    match node {
        ArrayNode::Boolean(b) => Ok(*b),
        ArrayNode::Number(n) => Ok(*n != 0.0),
        ArrayNode::String(s) => {
            let lower = s.to_lowercase();
            if lower == "true" {
                Ok(true)
            } else if lower == "false" {
                Ok(false)
            } else {
                Err(ArrayNode::Error(Error::VALUE))
            }
        }
        ArrayNode::Empty => Ok(false),
        ArrayNode::Error(e) => Err(ArrayNode::Error(e.clone())),
    }
}

// ─────────────────────────────────────────────────────────────────────────────

impl Model {
    pub(crate) fn fn_true(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.is_empty() {
            CalcResult::Boolean(true)
        } else {
            CalcResult::new_args_number_error(cell)
        }
    }

    pub(crate) fn fn_false(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.is_empty() {
            CalcResult::Boolean(false)
        } else {
            CalcResult::new_args_number_error(cell)
        }
    }

    pub(crate) fn fn_if(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.if_impl(args, cell, IfBranchMode::Value)
            .into_single_area(cell)
    }

    /// IF; with a scalar condition the chosen branch comes back as `mode` says
    /// (T-0216, T-1026; docs/excel-semantics/references.md). In reference context a union in
    /// the chosen branch stays its areas: `SUM(IF(TRUE,(A1,B2)))` is 6 and
    /// `IF(TRUE,(A1,B2)) A1` is 1 (probes/multiarea-ifchoose H1:H2). The condition is evaluated
    /// once, then the chosen branch once (`read_once` in functions/logical/lambda.rs relies on it).
    pub(crate) fn if_impl(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
        mode: IfBranchMode,
    ) -> RefArg {
        let node = match self.if_pick(args, cell) {
            Picked::Node(node) => node,
            Picked::Value(value) => return RefArg::Value(value),
        };
        match mode {
            IfBranchMode::Aggregate => self.evaluate_reference_arg(node, cell),
            IfBranchMode::SingleSheet => self.evaluate_single_sheet_arg(node, cell).into(),
            IfBranchMode::Chosen => self.evaluate_chosen_reference(node, cell).into(),
            // A reference into another workbook picked is read as INDEX reads the cell it picks:
            // `IF(TRUE,…!A1)` is 0, `&"x"` "x", ERROR.TYPE `#N/A`, where the written `…!A1` is
            // `#BLOCKED!` (probes/external-ref-no-link-index A26, external-ref-no-link-ifchoose).
            IfBranchMode::Value => self
                .external_reference_picked_value(node, cell)
                .unwrap_or_else(|| self.evaluate_node_in_context(node, cell))
                .into(),
        }
    }

    /// What IF's condition picks: with a scalar condition the branch it chooses, unevaluated;
    /// otherwise the value IF gives (the condition's error, FALSE for a missing branch, or IF
    /// over an array condition, its branches evaluated).
    pub(crate) fn if_pick<'a>(&mut self, args: &'a [Node], cell: CellReferenceIndex) -> Picked<'a> {
        if args.len() != 2 && args.len() != 3 {
            return Picked::Value(CalcResult::new_args_number_error(cell));
        }
        let cond_result = self.evaluate_node_in_context(&args[0], cell);

        // Normalize cond to a 2-D array, or take the scalar early-return path.
        let cond_array: Vec<Vec<ArrayNode>> = match cond_result {
            error @ CalcResult::Error { .. } => return Picked::Value(error),
            CalcResult::Range { left, right } => match self.evaluate_range(left, right, cell) {
                Ok(array) => array,
                Err(error) => return Picked::Value(error),
            },
            CalcResult::Array(a) => a,
            other => {
                // Scalar cond: cast to bool and pick the chosen branch.
                // The branch result may itself be an array (it will spill).
                return match self.cast_to_bool(other, cell) {
                    Ok(true) => Picked::Node(&args[1]),
                    Ok(false) => args
                        .get(2)
                        .map_or(Picked::Value(CalcResult::Boolean(false)), Picked::Node),
                    Err(error) => Picked::Value(error),
                };
            }
        };
        Picked::Value(self.if_array(args, cond_array, cell))
    }

    /// Evaluates a branch of IF over an array condition (or an arm of CHOOSE over an index
    /// array) once, as a scalar or a 2-D array; `Err` is a range that cannot be read.
    pub(crate) fn evaluate_if_arg(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> Result<IfArg, CalcResult> {
        Ok(match self.evaluate_node_in_context(node, cell) {
            CalcResult::Range { left, right } => {
                IfArg::Array(self.evaluate_range(left, right, cell)?)
            }
            CalcResult::Array(a) => IfArg::Array(a),
            other => IfArg::Scalar(calc_result_to_array_node(other)),
        })
    }

    /// IF over an array condition: one result per position, the branches broadcast.
    fn if_array(
        &mut self,
        args: &[Node],
        cond_array: Vec<Vec<ArrayNode>>,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        let has_else = args.len() == 3;
        // Both branches are read here, so a parameter read once in each is read twice.
        self.variable_stack.copy_read_once();
        // Evaluate branches once and normalise each to a scalar or 2-D array.
        let true_arg = match self.evaluate_if_arg(&args[1], cell) {
            Ok(arg) => arg,
            Err(error) => return error,
        };
        let false_arg: Option<IfArg> = if has_else {
            match self.evaluate_if_arg(&args[2], cell) {
                Ok(arg) => Some(arg),
                Err(error) => return error,
            }
        } else {
            None
        };

        // Output size = largest extent across all array arguments.
        let cond_rows = cond_array.len();
        let cond_cols = cond_array.first().map_or(0, |r| r.len());
        let (true_rows, true_cols) = if_arg_dims(&true_arg);
        let (false_rows, false_cols) = false_arg.as_ref().map_or((1, 1), if_arg_dims);
        let max_rows = cond_rows.max(true_rows).max(false_rows);
        let max_cols = cond_cols.max(true_cols).max(false_cols);

        if let Some((error, message)) = check_array_size(max_rows, max_cols) {
            return CalcResult::new_error(error, cell, message);
        }

        let mut output: Vec<Vec<ArrayNode>> = Vec::with_capacity(max_rows);
        for r in 0..max_rows {
            let mut row: Vec<ArrayNode> = Vec::with_capacity(max_cols);
            for c in 0..max_cols {
                // Index cond with size-1 broadcasting; a ragged out-of-range
                // position yields #N/A.
                let cond_node = bcast_idx(cond_rows, r)
                    .and_then(|cr| cond_array.get(cr))
                    .and_then(|crow| bcast_idx(cond_cols, c).and_then(|cc| crow.get(cc)));
                let node = match cond_node {
                    None => ArrayNode::Error(Error::NA),
                    Some(cond_node) => match array_node_to_bool(cond_node) {
                        Err(err_node) => err_node,
                        Ok(true) => if_arg_at(&true_arg, r, c),
                        Ok(false) => match &false_arg {
                            Some(fa) => if_arg_at(fa, r, c),
                            None => ArrayNode::Boolean(false),
                        },
                    },
                };
                row.push(node);
            }
            output.push(row);
        }

        CalcResult::Array(output)
    }

    pub(crate) fn fn_iferror(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        // IFERROR replaces every kind of error with the fallback.
        self.if_error_like(args, cell, |_| true)
    }

    pub(crate) fn fn_ifna(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        // IFNA replaces only #N/A; every other error passes through.
        self.if_error_like(args, cell, |error| error == &Error::NA)
    }

    /// Shared implementation for IFERROR and IFNA.
    ///
    /// `is_handled` selects which errors are swapped for the fallback: any error
    /// for IFERROR, only `#N/A` for IFNA. Both functions support dynamic arrays:
    /// when `value` is an array/range, each handled error element is replaced by
    /// the corresponding (broadcast) element of the fallback; everything else
    /// passes through. A position out of bounds in `value` is treated as `#N/A`,
    /// which both functions handle, so it takes the fallback.
    fn if_error_like(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
        is_handled: impl Fn(&Error) -> bool,
    ) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let value = self.evaluate_node_in_context(&args[0], cell);

        // Normalize value to a 2-D array, or take the scalar early-return path.
        let value_array: Vec<Vec<ArrayNode>> = match value {
            CalcResult::Range { left, right } => match self.evaluate_range(left, right, cell) {
                Ok(array) => array,
                Err(error) => return error,
            },
            CalcResult::Array(a) => a,
            CalcResult::Error { ref error, .. } if is_handled(error) => {
                // Scalar handled error: evaluate the fallback branch directly.
                // The branch result may itself be an array (it will spill).
                return self.evaluate_node_in_context(&args[1], cell);
            }
            // A blank cell gives its value, not the reference: an empty value that OR reads as
            // FALSE and a lookup as 0 (T-0291; gen-m2 7469c3e920e1ade4 A351, ef5c55a81783b7d5 A291).
            CalcResult::EmptyCell => return CalcResult::EmptyArg,
            // A non-error scalar (or an error this function does not handle)
            // passes through unchanged.
            other => return other,
        };

        // value is an array: replace each handled error element with the
        // fallback, broadcasting the fallback across positions.
        let fallback_arg = match self.evaluate_if_arg(&args[1], cell) {
            Ok(arg) => arg,
            Err(error) => return error,
        };

        // Output size = largest extent across value and the fallback.
        let value_rows = value_array.len();
        let value_cols = value_array.first().map_or(0, |r| r.len());
        let (fb_rows, fb_cols) = if_arg_dims(&fallback_arg);
        let max_rows = value_rows.max(fb_rows);
        let max_cols = value_cols.max(fb_cols);

        if let Some((error, message)) = check_array_size(max_rows, max_cols) {
            return CalcResult::new_error(error, cell, message);
        }

        let mut output: Vec<Vec<ArrayNode>> = Vec::with_capacity(max_rows);
        for r in 0..max_rows {
            let mut row: Vec<ArrayNode> = Vec::with_capacity(max_cols);
            for c in 0..max_cols {
                // Index value with size-1 broadcasting; a ragged out-of-range
                // position is treated as #N/A.
                let value_node = bcast_idx(value_rows, r)
                    .and_then(|vr| value_array.get(vr))
                    .and_then(|vrow| bcast_idx(value_cols, c).and_then(|vc| vrow.get(vc)));
                let node = match value_node {
                    // A handled error element → use the (broadcast) fallback.
                    Some(ArrayNode::Error(e)) if is_handled(e) => if_arg_at(&fallback_arg, r, c),
                    // Anything else (including an unhandled error) passes through.
                    Some(n) => n.clone(),
                    // Out of range → #N/A. Both IFERROR and IFNA handle #N/A, so
                    // it takes the fallback; guard with the predicate for clarity.
                    None if is_handled(&Error::NA) => if_arg_at(&fallback_arg, r, c),
                    None => ArrayNode::Error(Error::NA),
                };
                row.push(node);
            }
            output.push(row);
        }

        CalcResult::Array(output)
    }

    /// A chosen branch as the reference it is (`as_reference`) or as a value.
    pub(crate) fn evaluate_node_as(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
        as_reference: bool,
    ) -> CalcResult {
        if as_reference {
            self.evaluate_chosen_reference(node, cell)
        } else {
            self.evaluate_node_in_context(node, cell)
        }
    }

    /// =IFS(condition1, value, [condition, value]*). An array or range condition lifts
    /// IFS element by element over it and the arguments after it, broadcast as the element-wise
    /// operators do: `IFS(A1:B2>1,5)` is `{#N/A,5;5,5}` (probes/ifs-switch-array-values M1, M3;
    /// docs/excel-semantics/logic.md). A scalar condition keeps a range value a reference:
    /// `SUM(IFS(TRUE,A1:B2))` is 10.
    pub(crate) fn fn_ifs(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.ifs_impl(args, cell, false)
    }

    /// IFS; `as_reference` hands the chosen value back as the reference it is, for an argument
    /// that reads one (MATCH's `match_type`: probes/match-type-text-passthrough K1, T-1399).
    pub(crate) fn ifs_impl(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
        as_reference: bool,
    ) -> CalcResult {
        let args_count = args.len();
        if args_count < 2 {
            return CalcResult::new_args_number_error(cell);
        }
        if !args_count.is_multiple_of(2) {
            // Missing value for last condition
            return CalcResult::new_args_number_error(cell);
        }
        let case_count = args_count / 2;
        for case_index in 0..case_count {
            let condition = self.evaluate_node_in_context(&args[2 * case_index], cell);
            if is_lifted(&condition) {
                // The earlier conditions were all FALSE, so the result is decided element by
                // element from here (`IFS({TRUE,FALSE},1,TRUE,2)` is {1,2}).
                let mut lifted = vec![condition];
                lifted.push(self.evaluate_node_in_context(&args[2 * case_index + 1], cell));
                for node in &args[2 * case_index + 2..] {
                    lifted.push(self.evaluate_node_in_context(node, cell));
                }
                return self.lift_with_cost(
                    lifted,
                    |_| 1,
                    cell,
                    |model, pairs| model.ifs_element(pairs, cell),
                );
            }
            match self.cast_to_bool(condition, cell) {
                Ok(true) => {
                    return self.evaluate_node_as(&args[2 * case_index + 1], cell, as_reference)
                }
                Ok(false) => {}
                Err(error) => return error,
            }
        }
        no_match(cell)
    }

    /// IFS on one element of lifted (condition, value) pairs: the value of the first TRUE
    /// condition, an error condition's error, or `#N/A`.
    fn ifs_element(&mut self, pairs: &[CalcResult], cell: CellReferenceIndex) -> CalcResult {
        for pair in pairs.chunks_exact(2) {
            match self.cast_to_bool(pair[0].clone(), cell) {
                Ok(true) => return pair[1].clone(),
                Ok(false) => {}
                Err(error) => return error,
            }
        }
        no_match(cell)
    }
}
