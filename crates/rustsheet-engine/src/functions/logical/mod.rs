mod and_or_xor_not;
mod bycol_byrow;
mod lambda;
mod r#let;
mod map_reduce;
mod read_once;
mod scan;
mod switch;

pub(crate) use bycol_byrow::as_lambda;
pub(crate) use lambda::{
    Captures, FunctionValue, LambdaCaller, NamedLambdas, ScopeResult, SharedLambda,
};
pub(crate) use r#let::{has_duplicate_parameter, same_variable};

use crate::cast::calc_result_to_array_node;
use crate::{
    arithmetic::bcast_idx,
    calc_result::CalcResult,
    expressions::{
        parser::{ArrayNode, Node},
        token::Error,
        types::CellReferenceIndex,
    },
    functions::{
        lift::{BroadcastArg, LiftCost, LiftOptions, LiftedArg},
        Function,
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
        ArrayNode::Lambda(_) | ArrayNode::Nested(_) => Err(ArrayNode::Error(Error::VALUE)),
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
    /// once, then the chosen branch once (`read_once` in `functions/logical/read_once.rs` relies on it).
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
            IfBranchMode::Value => self.if_branch_value(node, cell).into(),
        }
    }

    /// The branch an ordinary IF picked, as a value. A reference into another workbook is read
    /// as INDEX reads the cell it picks: `IF(TRUE,…!A1)` is 0, `&"x"` "x", ERROR.TYPE `#N/A`,
    /// where the written `…!A1` is `#BLOCKED!` (probes/external-ref-no-link-index A26,
    /// external-ref-no-link-ifchoose).
    pub(crate) fn if_branch_value(&mut self, node: &Node, cell: CellReferenceIndex) -> CalcResult {
        self.external_reference_picked_value(node, cell)
            .unwrap_or_else(|| self.evaluate_node_in_context(node, cell))
    }

    /// The pick step of a function that stands for one of its arguments once its condition,
    /// index or expression is read: IF, CHOOSE, IFS and SWITCH ([`Self::if_pick`],
    /// [`Self::choose_pick`], [`Self::ifs_pick`], [`Self::switch_pick`]). `None` for any other
    /// function, unread. Reference context follows the argument picked for all four alike
    /// (probes/subtotal-ifs-switch-reference-picks, take-drop-over-ifs-switch-picks; T-2014,
    /// T-2071).
    pub(crate) fn pick_step<'a>(
        &mut self,
        kind: &Function,
        args: &'a [Node],
        cell: CellReferenceIndex,
    ) -> Option<Picked<'a>> {
        Some(match kind {
            Function::If => self.if_pick(args, cell),
            Function::Choose => self.choose_pick(args, cell),
            Function::Ifs => self.ifs_pick(args, cell),
            Function::Switch => self.switch_pick(args, cell),
            _ => return None,
        })
    }

    /// What IF's condition picks: with a scalar condition the branch it chooses, unevaluated;
    /// otherwise the value IF gives (the condition's error, FALSE for a missing branch, or IF
    /// over an array condition, its branches evaluated). IF with no branch at all, as an
    /// eta-reduced IF called with one argument is (`BYROW(A1:B3,_xleta.IF)`), gives `#VALUE!`
    /// for each element of its condition that is not an error, passing an error on
    /// (`LET(f,_xleta.IF,f({1,#N/A}))` is `{#VALUE!,#N/A}`, probes/eta-if-one-argument;
    /// docs/excel-semantics/arrays.md "Functions passed by name").
    pub(crate) fn if_pick<'a>(&mut self, args: &'a [Node], cell: CellReferenceIndex) -> Picked<'a> {
        if !(1..=3).contains(&args.len()) {
            return Picked::Value(CalcResult::new_args_number_error(cell));
        }
        // A one-cell range (`INDIRECT("K1")`) is a scalar condition: it picks a branch, it does
        // not lift (`SUBTOTAL(9,TAKE(IF(INDIRECT("K1"),H1:H2,I1:I2),2))` is 7,
        // probes/subtotal-take-array-selector-b A130, T-2012).
        let cond = self.evaluate_node_in_context(&args[0], cell);
        let cond_result = self.collapse_single_cell(cond);

        // Normalize cond to a 2-D array, or take the scalar early-return path.
        let cond_array: Vec<Vec<ArrayNode>> = match cond_result {
            error @ CalcResult::Error { .. } => return Picked::Value(error),
            CalcResult::Range { left, right } => match self.evaluate_range(left, right, cell) {
                Ok(array) => array,
                Err(error) => return Picked::Value(error),
            },
            CalcResult::Array(a) => a,
            _ if args.len() == 1 => {
                return Picked::Value(CalcResult::new_error(
                    Error::VALUE,
                    cell,
                    "IF needs a value_if_true".to_string(),
                ))
            }
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
        if args.len() == 1 {
            let values = cond_array
                .into_iter()
                .map(|row| {
                    row.into_iter()
                        .map(|value| match value {
                            error @ ArrayNode::Error(_) => error,
                            _ => ArrayNode::Error(Error::VALUE),
                        })
                        .collect()
                })
                .collect();
            return Picked::Value(CalcResult::Array(values));
        }
        Picked::Value(self.if_array(args, cond_array, cell))
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
        // Evaluate branches once and normalise each to a scalar or 2-D array. A branch no
        // element of the condition takes is only sized, not copied (T-1564).
        let picks = |want: bool| {
            cond_array
                .iter()
                .flatten()
                .any(|node| matches!(array_node_to_bool(node), Ok(taken) if taken == want))
        };
        let (true_picked, false_picked) = (picks(true), picks(false));
        let true_arg = match self.evaluate_broadcast_arm(&args[1], cell, true_picked) {
            Ok(arg) => arg,
            Err(error) => return error,
        };
        let false_arg: Option<BroadcastArg> = if has_else {
            match self.evaluate_broadcast_arm(&args[2], cell, false_picked) {
                Ok(arg) => Some(arg),
                Err(error) => return error,
            }
        } else {
            None
        };

        // Output size = largest extent across all array arguments.
        let cond_rows = cond_array.len();
        let cond_cols = cond_array.first().map_or(0, |r| r.len());
        let (true_rows, true_cols) = true_arg.dims();
        let (false_rows, false_cols) = false_arg.as_ref().map_or((1, 1), BroadcastArg::dims);
        let max_rows = cond_rows.max(true_rows).max(false_rows);
        let max_cols = cond_cols.max(true_cols).max(false_cols);

        if let Some((error, message)) = check_array_size(max_rows, max_cols) {
            return CalcResult::new_error(error, cell, message);
        }
        // The size is checked, so a picked range branch is copied now (T-1564).
        let true_arg = match self.read_broadcast_arm(true_arg, cell) {
            Ok(arg) => arg,
            Err(error) => return error,
        };
        let false_arg = match false_arg.map(|arg| self.read_broadcast_arm(arg, cell)) {
            Some(Err(error)) => return error,
            Some(Ok(arg)) => Some(arg),
            None => None,
        };

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
                        Ok(true) => true_arg.at(r, c),
                        Ok(false) => match &false_arg {
                            Some(fa) => fa.at(r, c),
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
        // TRIMRANGE of a range is the range it keeps, so its blank cells are 0 as below
        // (`AVEDEV(IFERROR(TRIMRANGE(D!A1:B6),-1))` is 3.5; probes/t2579-array-value-edges B74, B84).
        let value = if let Node::FunctionKind {
            kind: Function::Trimrange,
            ..
        } = args[0].ungrouped()
        {
            self.evaluate_picked_reference_arg(&args[0], cell)
                .into_single_area(cell)
        } else {
            self.evaluate_node_in_context(&args[0], cell)
        };
        // A name bound to a 3-D span is a reference, not an error, so the fallback is not taken,
        // and its value is `#VALUE!`: `IFERROR(span3,2)` is `#VALUE!` while `ISERROR(span3)` is
        // TRUE and the inline `IFERROR(Feb:Apr!A7:B12,0.5)` 0.5 (gen-m8a b221d72cc501675a; T-1640).
        if self.is_sheet_span_name(args[0].ungrouped()) {
            return value;
        }

        // Normalize value to a 2-D array, or take the scalar early-return path.
        let value_array: Vec<Vec<ArrayNode>> = match value {
            // A multi-cell range gives its values, and a blank cell's is 0: COUNT, AVERAGE and
            // AVEDEV count it as a number and `&` reads "0" (`COUNT(IFERROR(D!A1:A6,-1))` is 4 over
            // 1, blank, TRUE, "x", 5, 1/0; IFNA alike; probes/t2579-array-value-edges B70-B88,
            // excel-graph; T-2579).
            CalcResult::Range { left, right } => match self.evaluate_range(left, right, cell) {
                Ok(array) if left != right => blanks_as_zero(array),
                Ok(array) => array,
                Err(error) => return error,
            },
            // So is a blank element of an array, such as GROUPBY's blank key (true blank to
            // INDEX and COUNT): `TEXTJOIN("|",FALSE,IFERROR(GROUPBY(..),"#"))` reads it "0"
            // (probes/groupby-sort-filter-args G!B15, B55; T-2854).
            CalcResult::Array(a) => blanks_as_zero(a),
            CalcResult::Error { ref error, .. } if is_handled(error) => {
                // Scalar handled error: evaluate the fallback branch directly.
                // The branch result may itself be an array (it will spill). A blank cell there
                // is the empty value too, which STDEV.P counts as 0
                // (`STDEV.P(A9:A12,IFERROR(1/0,E6),{1,..,10})` with E6 blank;
                // probes/t2475-stat-subexpressions P!A158).
                return self
                    .evaluate_node_in_context(&args[1], cell)
                    .blank_as_empty_value();
            }
            // A blank cell gives its value, not the reference: an empty value that OR reads as
            // FALSE and a lookup as 0 (T-0291; gen-m2 7469c3e920e1ade4 A351, ef5c55a81783b7d5 A291).
            blank @ CalcResult::EmptyCell => return blank.blank_as_empty_value(),
            // A non-error scalar (or an error this function does not handle)
            // passes through unchanged.
            other => return other,
        };

        // value is an array: replace each handled error element with the
        // fallback, broadcasting the fallback across positions.
        let fallback_arg = match self.evaluate_broadcast_arg(&args[1], cell) {
            Ok(arg) => arg,
            Err(error) => return error,
        };

        // Output size = largest extent across value and the fallback.
        let value_rows = value_array.len();
        let value_cols = value_array.first().map_or(0, |r| r.len());
        let (fb_rows, fb_cols) = fallback_arg.dims();
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
                    Some(ArrayNode::Error(e)) if is_handled(e) => fallback_arg.at(r, c),
                    // Anything else (including an unhandled error) passes through.
                    Some(n) => n.clone(),
                    // Out of range → #N/A. Both IFERROR and IFNA handle #N/A, so
                    // it takes the fallback; guard with the predicate for clarity.
                    None if is_handled(&Error::NA) => fallback_arg.at(r, c),
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
        match self.ifs_pick(args, cell) {
            Picked::Node(value) => self
                .evaluate_node_as(value, cell, as_reference)
                .empty_value_as_zero(),
            Picked::Value(value) => value,
        }
    }

    /// What IFS's conditions pick: the value of the first TRUE scalar condition, unevaluated;
    /// otherwise the value IFS gives (an argument-count error, a condition's error, `#N/A`, or
    /// IFS lifted over an array condition). Reference context follows the picked value as it
    /// follows IF's branch: `SUBTOTAL(9,IFS(TRUE,(H1,I2)))` is 11 and
    /// `ISREF(IFS(TRUE,XLOOKUP(..)))` TRUE (probes/subtotal-ifs-switch-reference-picks, T-2014).
    pub(crate) fn ifs_pick<'a>(
        &mut self,
        args: &'a [Node],
        cell: CellReferenceIndex,
    ) -> Picked<'a> {
        let args_count = args.len();
        if args_count < 2 {
            return Picked::Value(CalcResult::new_args_number_error(cell));
        }
        if !args_count.is_multiple_of(2) {
            // Missing value for last condition
            return Picked::Value(CalcResult::new_args_number_error(cell));
        }
        let case_count = args_count / 2;
        for case_index in 0..case_count {
            // A one-cell range condition is a scalar, as IF's is: `ROWS(IFS(A3:A3,5))` with A3
            // FALSE is `#N/A` (probes/t2280-ifs-one-cell-range-condition A30, T-2280).
            let condition = self.evaluate_node_in_context(&args[2 * case_index], cell);
            let condition = self.collapse_single_cell(condition);
            if LiftedArg::is_grid(&condition) {
                // The earlier conditions were all FALSE, so the result is decided element by
                // element from here (`IFS({TRUE,FALSE},1,TRUE,2)` is {1,2}).
                let mut lifted = vec![condition];
                lifted.push(self.evaluate_node_in_context(&args[2 * case_index + 1], cell));
                for node in &args[2 * case_index + 2..] {
                    lifted.push(self.evaluate_node_in_context(node, cell));
                }
                // Each element reads its condition and value pairs: one unit per argument.
                let units = u64::try_from(lifted.len()).unwrap_or(u64::MAX);
                return Picked::Value(self.lift_values(
                    lifted,
                    LiftCost::Cells(units),
                    &LiftOptions::REUSE_EQUAL_RUNS,
                    cell,
                    &mut |model, pairs| model.ifs_element(pairs, cell),
                ));
            }
            // A text condition that does not read as a boolean ends IFS in a 1x1 array of
            // `#VALUE!`, not the scalar error IF gives: `TYPE(IFS("a*",5))` is 64 and
            // `ROWS(IFS(A1,5)*2)` 1 with A1 = "a*", whereas `ROWS(IF(A1,5)*2)` is `#VALUE!`; an
            // error condition stays a scalar error (probes/t2280-ifs-one-cell-range-condition, T-2280).
            let is_text = matches!(condition, CalcResult::String(_));
            match self.cast_to_bool(condition, cell) {
                Ok(true) => return Picked::Node(&args[2 * case_index + 1]),
                Ok(false) => {}
                Err(error) if is_text => {
                    return Picked::Value(self.ifs_text_condition_error(
                        &args[2 * case_index + 1..],
                        error,
                        cell,
                    ))
                }
                Err(error) => return Picked::Value(error),
            }
        }
        Picked::Value(no_match(cell))
    }

    /// The result of an IFS text condition that does not read as a boolean: `error` as an array
    /// of the broadcast shape of the pair's value and every later argument, at least 1x1. The
    /// earlier arguments do not count (`IFS(FALSE,{1,2},"x",5)` is 1x1, `IFS("x",5,TRUE,{1,2})`
    /// 1x2, `IFS("x",5,{TRUE,FALSE,TRUE},1)` 1x3; probes/t2653-ifs-text-condition-array-value
    /// and -shape-sources, T-2653).
    fn ifs_text_condition_error(
        &mut self,
        rest: &[Node],
        error: CalcResult,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        let lifted: Vec<CalcResult> = rest
            .iter()
            .map(|node| self.evaluate_node_in_context(node, cell))
            .collect();
        let units = u64::try_from(lifted.len()).unwrap_or(u64::MAX);
        let shaped = self.lift_values(
            lifted,
            LiftCost::Cells(units),
            &LiftOptions::REUSE_EQUAL_RUNS,
            cell,
            &mut |_, _| error.clone(),
        );
        match shaped {
            CalcResult::Array(_) => shaped,
            other => CalcResult::Array(vec![vec![calc_result_to_array_node(other)]]),
        }
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

/// The values of a multi-cell range read by IFERROR or IFNA, each blank cell as 0 (see
/// `if_error_like`).
fn blanks_as_zero(rows: Vec<Vec<ArrayNode>>) -> Vec<Vec<ArrayNode>> {
    rows.into_iter()
        .map(|row| {
            row.into_iter()
                .map(|node| match node {
                    ArrayNode::Empty => ArrayNode::Number(0.0),
                    other => other,
                })
                .collect()
        })
        .collect()
}
