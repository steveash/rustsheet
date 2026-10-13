use crate::{
    calc_result::CalcResult,
    expressions::{
        parser::{for_each_chain_operand_mut, NamedVariable, Node},
        token::Error,
        types::CellReferenceIndex,
    },
    functions::Function,
    model::Model,
    reference_operators::{Ranges, RefArg},
    variable_stack::Binding,
};

/// Whether two LET variable or LAMBDA parameter names are the same variable: Excel compares them
/// case-insensitively (`LET(x,A1:A4,X)` reads `x`, probes/subtotal-reference-operands-f C41;
/// `LAMBDA(x,X+1)(2)` is 3 and `LET(x,1,LAMBDA(X,x*10)(2))` is 20, probes/let-lambda-name-case),
/// folding non-ASCII letters too (`LET(é,1,É+1)` is 2, `LET(Ärger,5,äRGER+1)` is 6,
/// probes/let-name-non-ascii).
pub(crate) fn same_variable(a: &str, b: &str) -> bool {
    // No allocation: this runs per node on every LAMBDA call and LET evaluation.
    if a.is_ascii() && b.is_ascii() {
        return a.eq_ignore_ascii_case(b);
    }
    a.chars()
        .flat_map(char::to_lowercase)
        .eq(b.chars().flat_map(char::to_lowercase))
}

/// Whether two LAMBDA parameters are the same variable ([`same_variable`]): Excel refuses
/// `LAMBDA(x,x,x)(1,2)`, `LAMBDA(x,X,x+X)(1,2)` and `LAMBDA(ä,Ä,ä+Ä)(1,2)` as corrupt files
/// (docs/excel-semantics/references.md, T-1387).
pub(crate) fn has_duplicate_parameter(parameters: &[NamedVariable]) -> bool {
    parameters.iter().enumerate().any(|(i, p)| {
        parameters[..i]
            .iter()
            .any(|q| same_variable(&q.name, &p.name))
    })
}

/// Recursively walks `node`, replacing every `NamedVariableKind { name, id: None }` whose
/// name matches `target` with `id: Some(id)`.
///
/// When a nested LET is encountered that binds the same name, the walk stops past that
/// pair's value expression — the inner LET will assign its own id for subsequent uses.
pub(super) fn assign_variable_ids(node: &mut Node, target: &str, id: u32) {
    match node {
        Node::NamedVariableKind { name, id: var_id }
            if var_id.is_none() && same_variable(name, target) =>
        {
            *var_id = Some(id);
        }
        Node::NamedFunctionKind {
            name,
            id: var_id,
            args,
        } => {
            if var_id.is_none() && same_variable(name, target) {
                *var_id = Some(id);
            }
            for arg in args.iter_mut() {
                assign_variable_ids(arg, target, id);
            }
        }
        Node::FunctionKind { kind, args } => {
            if *kind == Function::Let {
                let n_pairs = (args.len().saturating_sub(1)) / 2;
                // Find the first pair index where the inner LET shadows `target`.
                let shadow_pair = (0..n_pairs).find(|&k| {
                    matches!(&args[2 * k], Node::NamedVariableKind { name, .. } if same_variable(name, target))
                });
                for i in 0..args.len() {
                    // Odd indices are value expressions; the last (even) index is the body.
                    // Even indices that are not the last are name declarations — never patch.
                    let is_declaration = i % 2 == 0 && i < args.len() - 1;
                    if is_declaration {
                        continue;
                    }
                    // Beyond the shadow pair's own value expression the name is shadowed.
                    if let Some(k) = shadow_pair {
                        if i > 2 * k + 1 {
                            continue;
                        }
                    }
                    assign_variable_ids(&mut args[i], target, id);
                }
            } else {
                for arg in args.iter_mut() {
                    assign_variable_ids(arg, target, id);
                }
            }
        }
        Node::OpSumKind { .. }
        | Node::OpProductKind { .. }
        | Node::OpPowerKind { .. }
        | Node::CompareKind { .. }
        | Node::OpConcatenateKind { .. } => {
            for_each_chain_operand_mut(node, |operand| {
                assign_variable_ids(operand, target, id);
            });
        }
        Node::UnaryKind { right, .. } => assign_variable_ids(right, target, id),
        Node::ImplicitIntersection { child, .. }
        | Node::SpillRangeOperator { child }
        | Node::Group { child, .. } => {
            assign_variable_ids(child, target, id);
        }
        Node::OpRangeKind { left, right }
        | Node::OpIntersectKind { left, right, .. }
        | Node::OpUnionKind { left, right, .. } => {
            assign_variable_ids(left, target, id);
            assign_variable_ids(right, target, id);
        }
        Node::LambdaDefKind { parameters, body } => {
            // If a LAMBDA parameter shadows `target`, don't recurse into the body.
            let shadowed = parameters.iter().any(|p| same_variable(&p.name, target));
            if !shadowed {
                assign_variable_ids(body, target, id);
            }
        }
        Node::LambdaCallKind { lambda, args } => {
            assign_variable_ids(lambda, target, id);
            for arg in args.iter_mut() {
                assign_variable_ids(arg, target, id);
            }
        }
        _ => {}
    }
}

impl Model {
    pub(crate) fn fn_let(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.let_impl(args, cell, false).into_single_area(cell)
    }

    /// LET; with `as_reference` the body is evaluated in reference context, so a union there
    /// stays its areas: `SUM(LET(x,(A1,B2),x))` is 6 (probes/multiarea-letbind H8).
    pub(crate) fn let_impl(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
        as_reference: bool,
    ) -> RefArg {
        // LET requires an odd number of args >= 3: name1, value1, [name2, value2, ...], body
        if args.len() < 3 || args.len().is_multiple_of(2) {
            return RefArg::Value(CalcResult::new_args_number_error(cell));
        }

        let pair_count = (args.len() - 1) / 2;
        // Clone the args so we can patch NamedVariableKind ids without touching the stored AST.
        let mut cloned: Vec<Node> = args.to_vec();
        let mut bound_ids: Vec<usize> = Vec::with_capacity(pair_count);

        // Excel refuses a formula that binds one name twice, in any case (`LET(x,1,X,2,x)`,
        // docs/excel-semantics/references.md); a LET has few names, so a linear scan is enough.
        let mut seen_names: Vec<String> = Vec::with_capacity(pair_count);

        for i in 0..pair_count {
            // Extract the variable name from the declaration node.
            let name = match &cloned[2 * i] {
                Node::NamedVariableKind { name, .. }
                    if !seen_names.iter().any(|seen| same_variable(seen, name)) =>
                {
                    Ok(name.clone())
                }
                Node::NamedVariableKind { name, .. } => Err(CalcResult::new_error(
                    Error::ERROR,
                    cell,
                    format!("LET: variable name '{name}' is already bound"),
                )),
                _ => Err(CalcResult::new_error(
                    Error::VALUE,
                    cell,
                    format!("LET: argument {} must be a variable name", 2 * i + 1),
                )),
            };
            let name = match name {
                Ok(name) => {
                    seen_names.push(name.clone());
                    name
                }
                Err(error) => {
                    // Remove only the ids introduced by this LET frame.
                    for id in bound_ids {
                        self.variable_stack.remove(id);
                    }
                    return RefArg::Value(error);
                }
            };

            let raw_id = self.get_next_variable_id();
            let id = raw_id as u32;
            bound_ids.push(raw_id);

            // Patch the variable's id into all subsequent nodes (value expr + body and later pairs).
            for item in cloned.iter_mut().skip(2 * i + 1) {
                assign_variable_ids(item, &name, id);
            }

            // Eagerly evaluate the binding value and cache it.
            // CalcResult::Range stores only bounds (no cell data), so range bindings are
            // effectively lazy — values are only read when a consuming function iterates the range.
            let binding = self.bind_argument(&cloned[2 * i + 1], cell);
            self.variable_stack.insert(raw_id, binding, cell);
        }

        // Evaluate the body with all bindings in scope.
        let body = &cloned[args.len() - 1];
        let result = if as_reference {
            self.evaluate_reference_arg(body, cell)
        } else {
            RefArg::Value(self.evaluate_node_in_context(body, cell))
        };

        // Remove only the ids introduced by this LET frame.
        for id in bound_ids {
            self.variable_stack.remove(id);
        }

        result
    }

    /// What a LET value or a LAMBDA argument binds. A reference stays a reference, a single
    /// cell too (upstream `MIN_MAX_extras.xlsx` A16, `MAX_MIN.xlsx` H54), and so does one INDEX,
    /// XLOOKUP, TAKE, DROP or TRIMRANGE picks (T-1232), but VLOOKUP and HLOOKUP bind their value (probes/let-bind-reference-forms; a
    /// called `LET(f,INDEX(A1:A3,2),f(2))` is `#REF!`, let-name-call-refs H5). An argument that
    /// may stand for several areas is read in reference context, so a union stays its areas
    /// wherever it comes from: written out, another variable, IF, CHOOSE or LET
    /// (probes/multiarea-let H1, multiarea-letbind H1:H3 and H7).
    pub(crate) fn bind_argument(&mut self, node: &Node, cell: CellReferenceIndex) -> Binding {
        let node = node.ungrouped();
        let is_span = matches!(node, Node::SheetSpanKind { .. });
        // XLOOKUP, TAKE, DROP and TRIMRANGE bind the reference they pick, so calling the name is
        // `#REF!` as for OFFSET (probes/let-name-call-forms H7:H9, H11; T-1232).
        let is_picked = matches!(node, Node::FunctionKind { kind, .. } if kind.picks_reference());
        if !is_span && !is_picked && !self.is_multi_area(node) {
            // An INDEX that picks one cell stays a reference, `@INDEX(..)` too (probes/let-bind-implicit-index
            // H1:H6, T-1503): `eval_ref_or_index` owns that rule.
            return Binding::Value(self.eval_ref_or_index(node, cell));
        }
        match self.evaluate_picked_reference_arg(node, cell) {
            RefArg::Ranges(Ranges::Areas(areas)) => Binding::Areas(areas),
            // A 3-D span, written or from IF or CHOOSE: an aggregate reads it through the
            // variable (probes/3d-ref-let-values C7, 3d-ref-let-bound-span C1:C3, 111), every
            // value read is `#VALUE!` (3d-ref-let-values C1:C6, 3d-ref-let-bound-span C4, C7).
            RefArg::Ranges(Ranges::One(range)) if range.left.sheet != range.right.sheet => {
                Binding::Span(range)
            }
            other => Binding::Value(other.into_single_area(cell)),
        }
    }

    /// What a LET variable or a LAMBDA parameter is bound to; a reference stays a reference.
    /// Shared by LET, LAMBDA and the evaluator (model.rs).
    pub(crate) fn variable_value(&mut self, node: &Node, cell: CellReferenceIndex) -> CalcResult {
        match node {
            Node::NamedVariableKind { name, id: Some(id) } => {
                match self.variable_stack.get(*id as usize) {
                    Some(binding) => binding.value(cell),
                    None => CalcResult::new_error(
                        Error::NAME,
                        cell,
                        format!("Variable \"{name}\" not found in scope."),
                    ),
                }
            }
            Node::NamedVariableKind { name, id: None } => CalcResult::new_error(
                Error::NAME,
                cell,
                format!("Variable name \"{name}\" not found."),
            ),
            _ => CalcResult::new_error(Error::ERROR, cell, "Not a variable".to_string()),
        }
    }

    /// A LET variable or LAMBDA parameter in reference context: a union binding is its areas, a
    /// 3-D span its range, anything else what [`Model::variable_value`] gives.
    pub(crate) fn variable_reference(&mut self, node: &Node, cell: CellReferenceIndex) -> RefArg {
        if let Node::NamedVariableKind { id: Some(id), .. } = node {
            match self.variable_stack.get(*id as usize) {
                Some(Binding::Areas(areas)) => return RefArg::Ranges(Ranges::Areas(areas.clone())),
                Some(Binding::Span(range)) => return RefArg::Ranges(Ranges::One(range.clone())),
                _ => {}
            }
        }
        RefArg::from(self.variable_value(node, cell))
    }
}
