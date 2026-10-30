use super::no_match;
use crate::functions::lift::{LiftCost, LiftOptions, LiftedArg};
use crate::{
    calc_result::CalcResult,
    expressions::{parser::Node, types::CellReferenceIndex},
    functions::util::compare_values_pairwise,
    model::Model,
    reference_operators::Picked,
};

impl Model {
    /// =SWITCH(expression, case1, value1, [case, value]*, [default]). An array or range
    /// expression lifts SWITCH element by element over it and the other arguments,
    /// broadcast as the element-wise operators do: `SWITCH(A1:B2,1,"a","z")` is
    /// `{"a","z";"z","z"}` (probes/ifs-switch-array-values M6, M8; docs/excel-semantics/logic.md).
    pub(crate) fn fn_switch(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.switch_impl(args, cell, false)
    }

    /// SWITCH; `as_reference` hands the chosen result back as the reference it is, for an
    /// argument that reads one (MATCH's `match_type`: probes/match-type-text-passthrough K4).
    pub(crate) fn switch_impl(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
        as_reference: bool,
    ) -> CalcResult {
        match self.switch_pick(args, cell) {
            Picked::Node(value) => self
                .evaluate_node_as(value, cell, as_reference)
                .empty_value_as_zero(),
            Picked::Value(value) => value,
        }
    }

    /// SWITCH lifted over an array `expr` or case value: every argument is read, and each element
    /// compares its value with the cases. `cases` are the case values `switch_pick` has already
    /// read, in order; they are reused, not read again, so nested array-case `SWITCH`es stay linear.
    fn switch_lifted(
        &mut self,
        expr: CalcResult,
        cases: Vec<CalcResult>,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        let mut lifted = vec![expr];
        let mut read = cases.into_iter();
        for (index, node) in args.iter().enumerate().skip(1) {
            // Odd positions are the case values, the first `read.len()` of which are in hand.
            let value = (index % 2 == 1).then(|| read.next()).flatten();
            lifted.push(value.unwrap_or_else(|| self.evaluate_node_in_context(node, cell)));
        }
        // One unit per argument.
        let units = u64::try_from(lifted.len()).unwrap_or(u64::MAX);
        self.lift_values(
            lifted,
            LiftCost::Cells(units),
            &LiftOptions::REUSE_EQUAL_RUNS,
            cell,
            &mut |_, element| switch_element(element, cell),
        )
    }

    /// What SWITCH's scalar expression picks: the value of the first matching case or the
    /// default, unevaluated; otherwise the value SWITCH gives (an argument-count error, the
    /// expression's or a case's error, `#N/A`, or SWITCH lifted over an array expression). An
    /// empty array's `#CALC!` in the expression or a case is `#VALUE!`, in a result it stays
    /// (probes/empty-array-args-sequence A1:A4).
    /// Reference context follows the picked value as it follows IF's branch:
    /// `SUBTOTAL(9,SWITCH(1,1,(H1,I2)))` is 11 (probes/subtotal-ifs-switch-reference-picks, T-2014).
    pub(crate) fn switch_pick<'a>(
        &mut self,
        args: &'a [Node],
        cell: CellReferenceIndex,
    ) -> Picked<'a> {
        let args_count = args.len();
        if args_count < 3 {
            return Picked::Value(CalcResult::new_args_number_error(cell));
        }
        let expr = self.evaluate_node_in_context(&args[0], cell);
        if LiftedArg::is_grid(&expr) {
            return Picked::Value(self.switch_lifted(expr, Vec::new(), args, cell));
        }
        if expr.is_error() {
            return Picked::Value(expr.empty_array_as_value_error(cell));
        }

        // How many cases we have?
        // 3, 4 args -> 1 case
        let case_count = (args_count - 1) / 2;
        let mut cases = Vec::new();
        for case_index in 0..case_count {
            let case = self.evaluate_node_in_context(&args[2 * case_index + 1], cell);
            // An array case value lifts the call like an array expression does, though the
            // expression is a scalar (`SUBTOTAL(9,TAKE(SWITCH(1,{1,2},H1:H2,I1:I2),2))` is a 2x2
            // of `#VALUE!`, probes/subtotal-take-array-selector-b A10, T-2012).
            if LiftedArg::is_grid(&case) {
                cases.push(case);
                return Picked::Value(self.switch_lifted(expr, cases, args, cell));
            }
            if case.is_error() {
                return Picked::Value(case.empty_array_as_value_error(cell));
            }
            if switch_matches(&expr, &case) {
                return Picked::Node(&args[2 * case_index + 2]);
            }
            cases.push(case);
        }
        // None of the cases matched so we return the default
        // If there is an even number of args is the last one otherwise is #N/A
        if args_count.is_multiple_of(2) {
            return Picked::Node(&args[args_count - 1]);
        }
        Picked::Value(no_match(cell))
    }
}

/// Whether SWITCH's expression value matches a case value. A blank cell on either side reads as
/// the number 0, not as the blank `compare_values` equates with FALSE and "": `SWITCH(FALSE,A1,..)`
/// and `SWITCH(A1,"",..)` with A1 blank are `#N/A`, while `SWITCH(0,A1,..)` and `SWITCH(A1,A1,..)`
/// match (probes/t2200-blank-and-padded-m5-args A28:A46, T-2166). An omitted argument
/// (`SWITCH(0,,"a")`) keeps `compare_values`' empty-value rule.
fn switch_matches(expr: &CalcResult, case: &CalcResult) -> bool {
    fn blank_as_zero(value: &CalcResult) -> &CalcResult {
        static ZERO: CalcResult = CalcResult::Number(0.0);
        if matches!(value, CalcResult::EmptyCell) {
            &ZERO
        } else {
            value
        }
    }
    compare_values_pairwise(blank_as_zero(expr), blank_as_zero(case)) == 0
}

/// SWITCH on one element of lifted `[expression, case1, value1, ..., default?]`: the scalar path's
/// rules on the element values.
fn switch_element(element: &[CalcResult], cell: CellReferenceIndex) -> CalcResult {
    let Some((expr, rest)) = element.split_first() else {
        return no_match(cell);
    };
    if expr.is_error() {
        return expr.clone();
    }
    let mut pairs = rest.chunks_exact(2);
    for pair in pairs.by_ref() {
        if pair[0].is_error() {
            return pair[0].clone();
        }
        if switch_matches(expr, &pair[0]) {
            return pair[1].clone();
        }
    }
    match pairs.remainder() {
        [default] => default.clone(),
        _ => no_match(cell),
    }
}
