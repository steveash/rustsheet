use super::no_match;
use crate::functions::lift::{LiftCost, LiftOptions, LiftedArg};
use crate::{
    calc_result::CalcResult,
    expressions::{parser::Node, types::CellReferenceIndex},
    functions::util::compare_values,
    model::Model,
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
        let args_count = args.len();
        if args_count < 3 {
            return CalcResult::new_args_number_error(cell);
        }
        let expr = self.evaluate_node_in_context(&args[0], cell);
        if LiftedArg::is_grid(&expr) {
            let mut lifted = vec![expr];
            for node in &args[1..] {
                lifted.push(self.evaluate_node_in_context(node, cell));
            }
            return self.lift_values(
                lifted,
                LiftCost::Cells(1),
                &LiftOptions::REUSE_EQUAL_RUNS,
                cell,
                &mut |_, element| switch_element(element, cell),
            );
        }
        if expr.is_error() {
            return expr;
        }

        // How many cases we have?
        // 3, 4 args -> 1 case
        let case_count = (args_count - 1) / 2;
        for case_index in 0..case_count {
            let case = self.evaluate_node_in_context(&args[2 * case_index + 1], cell);
            if case.is_error() {
                return case;
            }
            if compare_values(&expr, &case) == 0 {
                return self.evaluate_node_as(&args[2 * case_index + 2], cell, as_reference);
            }
        }
        // None of the cases matched so we return the default
        // If there is an even number of args is the last one otherwise is #N/A
        if args_count.is_multiple_of(2) {
            return self.evaluate_node_as(&args[args_count - 1], cell, as_reference);
        }
        no_match(cell)
    }
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
        if compare_values(expr, &pair[0]) == 0 {
            return pair[1].clone();
        }
    }
    match pairs.remainder() {
        [default] => default.clone(),
        _ => no_match(cell),
    }
}
