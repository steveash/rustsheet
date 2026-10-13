use crate::{
    calc_result::CalcResult,
    cast::calc_result_to_array_node,
    expressions::{
        parser::{ArrayNode, NamedVariable, Node},
        token::Error,
        types::CellReferenceIndex,
    },
    functions::util::array_node_to_calc_result,
    model::Model,
};

use super::r#let::assign_variable_ids;
use crate::variable_stack::Binding;

/// `#REF!` for a reference that is called like a function.
fn reference_called(cell: CellReferenceIndex) -> CalcResult {
    CalcResult::new_error(Error::REF, cell, "A reference cannot be called".to_string())
}

impl Model {
    /// A call written in the formula, `callee(args)`, of what its callee evaluates to. A called
    /// reference is `#REF!`, even when its cell holds a LAMBDA (`(A1)(A2)`, `(C1)(2)`;
    /// docs/excel-semantics/references.md "Calling a parenthesized operand", T-0149); anything
    /// else is called as a LAMBDA.
    pub(crate) fn call_callee(
        &mut self,
        callee: CalcResult,
        call_args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        if let CalcResult::Range { .. } = callee {
            return reference_called(cell);
        }
        self.call_lambda(callee, call_args, cell)
    }

    /// What a called LET variable or LAMBDA parameter holds. A union is a reference, so calling
    /// it is `#REF!` like any reference (probes/let-name-call-forms H1, H2, H13, T-1231).
    pub(crate) fn variable_callee(
        &self,
        var_id: u32,
        name: &str,
        cell: CellReferenceIndex,
    ) -> Result<CalcResult, CalcResult> {
        match self.variable_stack.get(var_id as usize) {
            Some(Binding::Areas(_)) => Err(reference_called(cell)),
            Some(binding) => Ok(binding.value(cell)),
            None => Err(CalcResult::new_error(
                Error::NAME,
                cell,
                format!("Variable \"{name}\" not found in scope."),
            )),
        }
    }

    /// A LET variable or LAMBDA parameter called like a function, `f(2)` in `LET(f,..,f(2))` or
    /// `LAMBDA(f,f(2))`.
    /// A LAMBDA is called and a reference is `#REF!` (`call_callee`); a value gives Excel's odd
    /// results without its arguments being read: a number or boolean is `#NAME?`, text is 0, an
    /// error is itself, and an array is called element by element
    /// (docs/excel-semantics/references.md "Calling a parenthesized operand", T-1092).
    pub(crate) fn call_variable(
        &mut self,
        callee: CalcResult,
        call_args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        match callee {
            CalcResult::Lambda(_) | CalcResult::Range { .. } => {
                self.call_callee(callee, call_args, cell)
            }
            CalcResult::Array(array) => CalcResult::Array(
                array
                    .iter()
                    .map(|row| row.iter().map(called_value).collect())
                    .collect(),
            ),
            error @ CalcResult::Error { .. } => error,
            other => {
                array_node_to_calc_result(&called_value(&calc_result_to_array_node(other)), cell)
            }
        }
    }

    /// Evaluates the body of a named lambda with the given call-site argument nodes.
    /// Optional parameters not covered by `call_args` receive `EmptyArg`.
    pub(crate) fn call_lambda(
        &mut self,
        lambda_result: CalcResult,
        call_args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        let (parameters, body) = match lambda_result {
            CalcResult::Lambda(id) => match self.lambdas.get(&id) {
                Some(l) => l.clone(),
                None => {
                    return CalcResult::new_error(
                        Error::NAME,
                        cell,
                        "Lambda not found in store".to_string(),
                    )
                }
            },
            other if other.is_error() => return other,
            _ => return CalcResult::new_error(Error::VALUE, cell, "Expected a LAMBDA".to_string()),
        };

        if let Err(e) = check_arg_count(&parameters, call_args.len(), cell) {
            return e;
        }

        let mut patched_body = body;
        let mut bound_ids: Vec<usize> = Vec::with_capacity(parameters.len());

        for param in &parameters {
            let raw_id = self.get_next_variable_id();
            assign_variable_ids(&mut patched_body, &param.name, raw_id as u32);
            bound_ids.push(raw_id);
        }

        for (i, raw_id) in bound_ids.iter().enumerate() {
            // A reference is passed as a reference, a single cell too, so that
            // `LAMBDA(x, MAX(x))(A2)` ignores the text in A2 (upstream MAX_MIN.xlsx H54).
            let binding = match call_args.get(i) {
                Some(arg) => self.bind_argument(arg, cell),
                None => Binding::Value(CalcResult::EmptyArg),
            };
            self.variable_stack.insert(*raw_id, binding, cell);
        }

        let result = self.evaluate_node_in_context(&patched_body, cell);

        for raw_id in bound_ids {
            self.variable_stack.remove(raw_id);
        }

        result
    }

    /// Like `call_lambda` but accepts already-evaluated `CalcResult` values instead of `Node`s.
    /// Optional parameters not covered by `values` receive `EmptyArg`.
    pub(crate) fn call_lambda_with_values(
        &mut self,
        lambda_result: CalcResult,
        values: Vec<CalcResult>,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        let (parameters, body) = match lambda_result {
            CalcResult::Lambda(id) => match self.lambdas.get(&id) {
                Some(l) => l.clone(),
                None => {
                    return CalcResult::new_error(
                        Error::NAME,
                        cell,
                        "Lambda not found in store".to_string(),
                    )
                }
            },
            other if other.is_error() => return other,
            _ => return CalcResult::new_error(Error::VALUE, cell, "Expected a LAMBDA".to_string()),
        };

        if let Err(e) = check_arg_count(&parameters, values.len(), cell) {
            return e;
        }

        let mut patched_body = body;
        let mut bound_ids: Vec<usize> = Vec::with_capacity(parameters.len());

        for param in &parameters {
            let raw_id = self.get_next_variable_id();
            assign_variable_ids(&mut patched_body, &param.name, raw_id as u32);
            bound_ids.push(raw_id);
        }

        for (i, raw_id) in bound_ids.iter().enumerate() {
            let val = if i < values.len() {
                values[i].clone()
            } else {
                CalcResult::EmptyArg
            };
            self.variable_stack
                .insert(*raw_id, Binding::Value(val), cell);
        }

        let result = self.evaluate_node_in_context(&patched_body, cell);

        for raw_id in bound_ids {
            self.variable_stack.remove(raw_id);
        }

        result
    }

    pub(crate) fn fn_lambda(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        // Reached when LAMBDA is stored as FunctionKind::Lambda (e.g. imported from xlsx).
        if args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }
        let last = args.len() - 1;
        let body = args[last].clone();
        let parameters: Vec<NamedVariable> = args[..last]
            .iter()
            .filter_map(|n| {
                if let Node::NamedVariableKind { name, id } = n {
                    let (clean, is_optional) = if let Some(s) = name.strip_prefix("_xlop.") {
                        (s.to_string(), true)
                    } else if let Some(s) = name.strip_prefix("_xlpm.") {
                        (s.to_string(), false)
                    } else {
                        (name.clone(), false)
                    };
                    Some(NamedVariable {
                        name: clean,
                        id: *id,
                        is_optional,
                    })
                } else {
                    None
                }
            })
            .collect();
        if super::has_duplicate_parameter(&parameters) {
            return CalcResult::new_error(
                Error::ERROR,
                cell,
                "LAMBDA parameter names must be distinct".to_string(),
            );
        }

        let id = self.get_next_lambda_id();
        self.lambdas.insert(id, (parameters, body));
        CalcResult::Lambda(id)
    }
}

fn check_arg_count(
    parameters: &[NamedVariable],
    provided: usize,
    cell: CellReferenceIndex,
) -> Result<(), CalcResult> {
    let required = parameters.iter().filter(|p| !p.is_optional).count();
    let total = parameters.len();
    if provided < required || provided > total {
        let msg = if required == total {
            format!("LAMBDA expected {total} argument(s), got {provided}")
        } else {
            format!("LAMBDA expected between {required} and {total} argument(s), got {provided}")
        };
        Err(CalcResult::new_error(Error::VALUE, cell, msg))
    } else {
        Ok(())
    }
}

/// What calling one value (not a LAMBDA, not a reference) gives (`Model::call_variable`).
fn called_value(value: &ArrayNode) -> ArrayNode {
    match value {
        ArrayNode::String(_) => ArrayNode::Number(0.0),
        ArrayNode::Error(error) => ArrayNode::Error(error.clone()),
        // An empty element is taken as a number (not probed: `LET(f,"",f(2))` makes Excel refuse
        // the workbook).
        ArrayNode::Number(_) | ArrayNode::Boolean(_) | ArrayNode::Empty => {
            ArrayNode::Error(Error::NAME)
        }
    }
}
