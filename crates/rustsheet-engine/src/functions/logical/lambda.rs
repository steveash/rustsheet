use crate::{
    calc_result::CalcResult,
    cast::calc_result_to_array_node,
    expressions::{
        parser::{ArrayNode, NamedVariable, Node},
        token::Error,
        types::CellReferenceIndex,
    },
    functions::{util::array_node_to_calc_result, Function},
    language::get_default_language,
    model::Model,
    reference_operators::RefArg,
};

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use super::r#let::assign_variable_ids;
use crate::evaluation::stack_address;
use crate::variable_stack::Binding;

/// A LAMBDA's parameters and body as `Model::lambdas` stores them, shared with the defined name
/// it comes from, so naming a defined-name LAMBDA copies no body (T-0440).
pub(crate) type SharedLambda = Arc<(Vec<NamedVariable>, Node)>;

/// The LET variables and LAMBDA parameters, by id, that a LAMBDA's body reads, bound when the
/// LAMBDA was made, and kept when the scope that bound them ended, with what they were bound to (`Model::close_over`).
pub(crate) type Captures = Vec<(usize, Arc<Binding>)>;

/// The defined-name LAMBDAs called in one evaluation (cleared with `Model::lambdas`). Each gets
/// one lambda id however often it is named (`Model::defined_name_lambda`), and the body a call
/// patched is kept for the next call (`Model::call_lambda`), so `MAP(r,LAMBDA(x,MyFn(x)))`
/// neither grows `lambdas` nor copies `MyFn`'s body per element (T-0440).
#[derive(Default)]
pub(crate) struct NamedLambdas {
    /// The lambda id of each definition, by the address of its [`SharedLambda`], which the entry
    /// holds so that no other definition can take that address while the entry lives.
    ids: HashMap<usize, (SharedLambda, usize)>,
    /// The lambda ids in `ids`.
    named: HashSet<usize>,
    /// A patched body by lambda id and number of arguments, for the next call. Only a body is
    /// kept: an error (a wrong count) is made again with the calling cell's origin.
    prepared: HashMap<(usize, usize), PreparedLambda>,
}

/// What a LET or a LAMBDA call gives, for [`Model::close_over`]: the LAMBDA it is, if it is one.
pub(crate) trait ScopeResult {
    fn lambda(&self) -> Option<usize>;
}

impl ScopeResult for CalcResult {
    fn lambda(&self) -> Option<usize> {
        match self {
            CalcResult::Lambda(id) => Some(*id),
            _ => None,
        }
    }
}

impl ScopeResult for RefArg {
    fn lambda(&self) -> Option<usize> {
        match self {
            RefArg::Value(value) => value.lambda(),
            RefArg::Ranges(_) => None,
        }
    }
}

impl ScopeResult for Binding {
    fn lambda(&self) -> Option<usize> {
        match self {
            Binding::Value(value) | Binding::ReadOnce(value) => value.lambda(),
            Binding::Areas(_) | Binding::Span(_) => None,
        }
    }
}

/// One LAMBDA called repeatedly with the same number of arguments (`Model::lambda_caller`).
pub(crate) struct LambdaCaller {
    lambda: CalcResult,
    provided: usize,
    prepared: Option<PreparedLambda>,
    /// Whether a call returns a reference the body gives as one (`LambdaCaller::keeping_references`).
    references: bool,
}

impl LambdaCaller {
    /// This caller with the body read in reference context: a body that is a reference, also
    /// one picked by IF or returned by INDEX, comes back as a `CalcResult::Range` instead of the
    /// cell's value, so GROUPBY and PIVOTBY can make it `#VALUE!` as Excel does
    /// (probes/groupby-lambda-forms G!J193, probes/groupby-pivotby-functions G!A541-A581).
    pub(crate) fn keeping_references(mut self) -> Self {
        self.references = true;
        self
    }
}

/// What a function value is, for the callers that choose how many arguments to pass (GROUPBY
/// and PIVOTBY): a function passed by name, or a LAMBDA with its required and total parameters.
pub(crate) enum FunctionValue {
    Eta(Function),
    Lambda { required: usize, total: usize },
}

/// A LAMBDA's parameters (each id, and whether the body reads it once: [`read_once`]) and
/// patched body, or the error calling it gives, and the `last_lambda_id` below which every LAMBDA
/// created is known not to read the parameters (`Model::call_prepared`).
struct PreparedLambda {
    definition: Result<(Vec<(usize, bool)>, Node), CalcResult>,
    lambda_mark: usize,
}

/// How many times the evaluation's stack budget (`Model::set_max_evaluation_stack`, a quarter of
/// a 1 MB stack by default) nested LAMBDA calls may use. A call made by a call can nest without
/// the formula's own depth bounding it (`Model::lambda_stack_base`), and each takes some 5 KB of
/// stack in a release build and 20 KB in a debug build (a LAMBDA calling itself through a
/// parameter), so the default, 1 MB, half the smallest common thread stack (2 MB), holds about
/// 200 calls in a release build and 50 in a debug build.
const LAMBDA_STACK_SHARE: usize = 4;

/// Most cells the LAMBDAs that outlive their scope may capture in one cell's formula
/// (`Model::close_over`): a REDUCE whose steps each return a closure over a fresh array keeps
/// every step's array. Past it a capture is `#NUM!` (DEVIATION, decision 0014: arrays.md "LAMBDAs
/// that outlive their call"). Four whole columns.
const MAX_CAPTURED_CELLS: usize = 1 << 22;

/// One read count of [`read_once`]: more than one read, or reads it cannot count.
const MANY: u8 = 2;

/// Whether the function `kind` evaluates its argument `index` exactly once per evaluation of the
/// call (each audited): VSTACK and HSTACK ([`Model::eval_to_array`]), LET (each value bound once,
/// `bind_argument`, then the body), IF (the condition, then one branch, or with an array condition
/// both: [`crate::variable_stack::VariableStack::copy_read_once`]), and the array of SORT, TAKE and DROP.
fn evaluates_once(kind: &Function, index: usize) -> bool {
    match kind {
        Function::Vstack | Function::Hstack | Function::Let | Function::If => true,
        Function::Sort | Function::Take | Function::Drop => index == 0,
        _ => false,
    }
}

/// For each of `ids`, whether `body` reads that variable exactly once per evaluation. A read
/// counts once when it is the body itself or reached from it only through groups and arguments
/// that [`evaluates_once`]; anywhere else a function might evaluate its argument twice, or a nested
/// LAMBDA might read it on each call, so the read counts as [`MANY`]. Reads add up, except that the
/// branches of an IF are exclusive, so they count as the most any one of them reads:
/// `IF(x>0,VSTACK(a,x),a)` reads `a` once. One walk serves every parameter, and it is not
/// recursive: nodes are listed parents first, then counted children first.
fn read_once(body: &Node, ids: &[usize]) -> Vec<bool> {
    /// A node, the index of its parent, whether it is a branch of an IF parent, and whether it is
    /// evaluated once per evaluation of the body.
    struct Visit<'a> {
        node: &'a Node,
        parent: usize,
        branch: bool,
        once: bool,
    }
    let position = |read: u32| ids.iter().position(|&id| id == read as usize);
    let mut visits = vec![Visit {
        node: body,
        parent: 0,
        branch: false,
        once: true,
    }];
    let mut children = Vec::new();
    let mut next = 0;
    while let Some(visit) = visits.get(next) {
        let (node, once) = (visit.node, visit.once);
        node.push_children(&mut children);
        for (index, child) in children.drain(..).enumerate() {
            let (passes_once, branch) = match node {
                Node::Group { .. } => (true, false),
                Node::FunctionKind { kind, .. } => (
                    evaluates_once(kind, index),
                    *kind == Function::If && index > 0,
                ),
                _ => (false, false),
            };
            visits.push(Visit {
                node: child,
                parent: next,
                branch,
                once: once && passes_once,
            });
        }
        next += 1;
    }
    // Per node and parameter: the reads of its own subtree outside IF branches, and the most
    // reads of any one of its IF branches.
    let width = ids.len();
    let mut sums = vec![0u8; visits.len() * width];
    let mut branches = vec![0u8; visits.len() * width];
    for (i, visit) in visits.iter().enumerate().rev() {
        let read = match visit.node {
            Node::NamedVariableKind { id: Some(read), .. } => {
                position(*read).map(|p| (p, if visit.once { 1 } else { MANY }))
            }
            Node::NamedFunctionKind { id: Some(read), .. } => position(*read).map(|p| (p, MANY)),
            _ => None,
        };
        if let Some((p, count)) = read {
            sums[i * width + p] = sums[i * width + p].saturating_add(count).min(MANY);
        }
        if i == 0 {
            break;
        }
        for p in 0..width {
            let total = sums[i * width + p]
                .saturating_add(branches[i * width + p])
                .min(MANY);
            let parent = visit.parent * width + p;
            if visit.branch {
                branches[parent] = branches[parent].max(total);
            } else {
                sums[parent] = sums[parent].saturating_add(total).min(MANY);
            }
        }
    }
    (0..width)
        .map(|p| sums[p].saturating_add(branches[p]) == 1)
        .collect()
}

/// `#REF!` for a reference that is called like a function.
fn reference_called(cell: CellReferenceIndex) -> CalcResult {
    CalcResult::new_error(Error::REF, cell, "A reference cannot be called".to_string())
}

impl Model {
    /// [`Self::evaluate_node_with_reference`] of a node that is no plain reference: a LET or
    /// LAMBDA call whose body gives a reference, a single cell too, is that reference for OFFSET
    /// and COUNTIF (`OFFSET(LET(x,A1,x),1,0)`, `COUNTIF(LAMBDA(x,x)(A1),">0")` read A1;
    /// probes/lambda-union-reference-callers D1, D2, D5, excel-graph; T-1951).
    pub(crate) fn evaluate_call_with_reference(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        match node {
            Node::FunctionKind {
                kind: Function::Let,
                ..
            }
            | Node::LambdaCallKind { .. } => self
                .evaluate_reference_arg(node, cell)
                .into_single_area(cell),
            _ => self.evaluate_node_in_context(node, cell),
        }
    }

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

    /// [`Self::call_callee`] of a call written where a reference is wanted (an aggregate's
    /// argument, ISREF): a body that gives a reference keeps it, a single cell too, so
    /// `SUM(LAMBDA(x,x)(K1))` and `MAX(LAMBDA(x,x)(K1))` skip the text in K1 (0) and
    /// `ISREF(LAMBDA(x,x)(A1))` is TRUE (probes/let-lambda-reference-results C1, C2, D1, D6,
    /// excel-graph; T-1106).
    pub(crate) fn call_callee_as_reference(
        &mut self,
        lambda: &Node,
        call_args: &[Node],
        cell: CellReferenceIndex,
    ) -> RefArg {
        let callee = self
            .evaluate_reference_arg(lambda, cell)
            .into_single_area(cell);
        // The same guard as `call_callee`'s, kept in step with it.
        if let CalcResult::Range { .. } = callee {
            return RefArg::Value(reference_called(cell));
        }
        let mut caller = self
            .lambda_caller(&callee, call_args.len())
            .keeping_references();
        self.call_lambda_caller_with_nodes_arg(&mut caller, call_args, cell)
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
                let called = called_value(&calc_result_to_array_node(other));
                if let ArrayNode::Error(error) = &called {
                    // Array elements carry no message, so only the scalar call can give one.
                    return CalcResult::new_error(
                        error.clone(),
                        cell,
                        "A value cannot be called".to_string(),
                    );
                }
                array_node_to_calc_result(&called, cell)
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
        // A reference is passed as a reference, a single cell too, so that
        // `LAMBDA(x, MAX(x))(A2)` ignores the text in A2 (upstream MAX_MIN.xlsx H54).
        let provided = call_args.len();
        let mut caller = self.lambda_caller(&lambda_result, provided);
        // A defined-name LAMBDA starts from the body its last call patched (`NamedLambdas`). It
        // is taken out while this call runs, so a call inside it (recursion, `MyFn(MyFn(x))`)
        // patches its own. No LAMBDA created since it was put back reads its ids: only an
        // evaluation of the body binds them, and `call_prepared` dropped it if one did.
        let named = match lambda_result {
            CalcResult::Lambda(id) if self.named_lambdas.named.contains(&id) => {
                Some((id, provided))
            }
            _ => None,
        };
        if let Some(key) = named {
            caller.prepared = self
                .named_lambdas
                .prepared
                .remove(&key)
                .map(|mut prepared| {
                    prepared.lambda_mark = self.last_lambda_id;
                    prepared
                });
        }
        let result = self.call_lambda_caller_with_nodes(&mut caller, call_args, cell);
        if let (Some(key), Some(prepared)) = (named, caller.prepared) {
            if prepared.definition.is_ok() {
                self.named_lambdas.prepared.insert(key, prepared);
            }
        }
        result
    }

    /// A caller for `lambda_result` with `provided` arguments per call, for the LAMBDA helpers
    /// (MAP, REDUCE, SCAN, BYROW, BYCOL, MAKEARRAY) that call one LAMBDA once per element: the
    /// definition is looked up and its body cloned and patched with variable ids on the first
    /// call, then reused (T-1560).
    pub(crate) fn lambda_caller(
        &self,
        lambda_result: &CalcResult,
        provided: usize,
    ) -> LambdaCaller {
        LambdaCaller {
            lambda: lambda_result.clone(),
            provided,
            prepared: None,
            references: false,
        }
    }

    /// What the LAMBDA `lambda_result` is ([`FunctionValue`]); `None` for anything else.
    pub(crate) fn function_value(&self, lambda_result: &CalcResult) -> Option<FunctionValue> {
        let CalcResult::Lambda(id) = lambda_result else {
            return None;
        };
        if let Some(kind) = self.eta_lambdas.get(id) {
            return Some(FunctionValue::Eta(kind.clone()));
        }
        self.lambdas.get(id).map(|definition| {
            let parameters = &definition.0;
            FunctionValue::Lambda {
                required: parameters.iter().filter(|p| !p.is_optional).count(),
                total: parameters.len(),
            }
        })
    }

    /// One call of `caller` with `values` (`provided` of them; optional parameters not covered
    /// receive `EmptyArg`). The values are moved into the parameters, not copied, so REDUCE and
    /// SCAN hand their accumulator (possibly a large array) to each step without a deep copy.
    pub(crate) fn call_lambda_caller(
        &mut self,
        caller: &mut LambdaCaller,
        values: impl IntoIterator<Item = CalcResult>,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        // `call_prepared` binds the parameters in order, so the i-th call takes the i-th value.
        let mut values = values.into_iter();
        self.call_prepared(caller, cell, |_, _| {
            Binding::Value(values.next().unwrap_or(CalcResult::EmptyArg))
        })
        .into_single_area(cell)
    }

    /// One call of `caller` with argument nodes evaluated in the calling cell, as `call_lambda`
    /// binds them.
    pub(crate) fn call_lambda_caller_with_nodes(
        &mut self,
        caller: &mut LambdaCaller,
        call_args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        self.call_lambda_caller_with_nodes_arg(caller, call_args, cell)
            .into_single_area(cell)
    }

    /// [`Self::call_lambda_caller_with_nodes`] keeping the areas of a union a reference-keeping
    /// `caller` returns.
    fn call_lambda_caller_with_nodes_arg(
        &mut self,
        caller: &mut LambdaCaller,
        call_args: &[Node],
        cell: CellReferenceIndex,
    ) -> RefArg {
        self.call_prepared(caller, cell, |model, i| match call_args.get(i) {
            Some(arg) => model.bind_argument(arg, cell),
            None => Binding::Value(CalcResult::EmptyArg),
        })
    }

    /// Binds parameter `i` to `bind(i)`, once for each `i` in order (`call_lambda_caller` relies
    /// on that), and evaluates the patched body. The body is reused while no LAMBDA created since
    /// it was patched reads its parameter ids: one made by the body (`LAMBDA(x,LAMBDA(y,x+y))`)
    /// holds them, so the next call patches a fresh copy with fresh ids, as every call did before
    /// T-1560. A defined-name LAMBDA, an eta-reduced function or a LAMBDA a called one makes holds
    /// none of them, so a body calling `MyFn(x)` keeps its patch (T-0425). (`last_lambda_id` only
    /// grows during an evaluation; `clear_lambdas` resets it between evaluations, when no caller
    /// is alive.) The fresh ids are also what lets `bind_captures` take a bound id for the one a
    /// closure captured: reusing the patch here would break closures
    /// (`a_closure_called_many_times_and_recursively_keeps_its_captures`).
    fn call_prepared(
        &mut self,
        caller: &mut LambdaCaller,
        cell: CellReferenceIndex,
        bind: impl FnMut(&mut Self, usize) -> Binding,
    ) -> RefArg {
        let Some(stack_base) = self.lambda_stack_base() else {
            return RefArg::Value(CalcResult::new_error(
                Error::NUM,
                cell,
                "LAMBDA calls nest too deep".to_string(),
            ));
        };
        let outer_base = self.lambda_calls_base.replace(stack_base);
        let result = self.call_prepared_within_budget(caller, cell, bind);
        self.lambda_calls_base = outer_base;
        result
    }

    /// Where the stack was when the outermost LAMBDA call running now started: the cell's base
    /// inside a cell formula, so cells and calls share one stack budget (evaluation.rs), or this
    /// call's own position for the outermost call outside one. `None` once the calls have used
    /// [`LAMBDA_STACK_SHARE`] times [`Model::get_max_evaluation_stack`] bytes: a LAMBDA calling itself through a parameter,
    /// `LET(f,LAMBDA(g,n,IF(n=0,0,g(g,n-1)+1)),f(f,20000))`, or a chain of 20,000 closures made by
    /// REDUCE, would otherwise overflow the stack. Excel evaluates such calls 300 deep
    /// (probes/lambda-call-depth); the budget stops them sooner, as `#NUM!` (DEVIATION, decision
    /// 0014: docs/excel-semantics/arrays.md "How deep LAMBDA calls nest").
    fn lambda_stack_base(&self) -> Option<usize> {
        let here = stack_address();
        let base = match self.lambda_calls_base {
            Some(base) => base,
            None if self.evaluation.depth > 0 => self.evaluation.stack_base,
            None => here,
        };
        let budget = self.evaluation.max_stack.saturating_mul(LAMBDA_STACK_SHARE);
        (base.abs_diff(here) < budget).then_some(base)
    }

    /// [`Self::call_prepared`] once the stack budget allows the call.
    fn call_prepared_within_budget(
        &mut self,
        caller: &mut LambdaCaller,
        cell: CellReferenceIndex,
        mut bind: impl FnMut(&mut Self, usize) -> Binding,
    ) -> RefArg {
        let mut prepared = match caller.prepared.take() {
            Some(prepared) => prepared,
            None => self.prepare_lambda(&caller.lambda, caller.provided, cell),
        };
        let result = match &prepared.definition {
            Ok((ids, body)) => {
                let mark = self.last_lambda_id;
                let mut captured = self.bind_captures(&caller.lambda, cell);
                for (i, &(raw_id, once)) in ids.iter().enumerate() {
                    let binding = match bind(self, i) {
                        Binding::Value(value) if once => Binding::ReadOnce(value),
                        binding => binding,
                    };
                    self.variable_stack.insert(raw_id, binding, cell);
                }
                let result = if caller.references {
                    self.evaluate_reference_arg(body, cell)
                } else {
                    RefArg::Value(self.evaluate_node_in_context(body, cell))
                };
                // A LAMBDA the body made may read the parameters and the captures bound here.
                captured.extend(ids.iter().map(|&(raw_id, _)| raw_id));
                self.close_over(result.lambda(), &captured, mark, cell);
                self.unbind_captures(captured);
                result
            }
            Err(error) => RefArg::Value(error.clone()),
        };
        if !self.lambdas_since_read(&prepared) {
            prepared.lambda_mark = self.last_lambda_id;
            caller.prepared = Some(prepared);
        }
        result
    }

    /// Whether a LAMBDA created since `prepared.lambda_mark` reads one of its parameter ids. Eta
    /// lambdas hold no body, so only `lambdas` is looked at; the walk costs no more than the
    /// clone that created each LAMBDA.
    fn lambdas_since_read(&self, prepared: &PreparedLambda) -> bool {
        let Ok((ids, _)) = &prepared.definition else {
            return false;
        };
        (prepared.lambda_mark..self.last_lambda_id).any(|lambda| {
            self.lambdas.get(&lambda).is_some_and(|definition| {
                definition
                    .1
                    .reads_variable(|read| ids.iter().any(|&(id, _)| id == read))
            })
        })
    }

    /// The definition of `lambda_result` for `provided` arguments with its body patched with
    /// fresh variable ids, one per parameter (or the error a call gives).
    fn prepare_lambda(
        &mut self,
        lambda_result: &CalcResult,
        provided: usize,
        cell: CellReferenceIndex,
    ) -> PreparedLambda {
        let definition =
            self.lambda_definition(lambda_result, provided, cell)
                .map(|(parameters, mut body)| {
                    let ids: Vec<usize> = parameters
                        .iter()
                        .map(|param| {
                            let raw_id = self.get_next_variable_id();
                            assign_variable_ids(&mut body, &param.name, raw_id as u32);
                            raw_id
                        })
                        .collect();
                    // After every id is patched in: a read is found by its id.
                    let once = read_once(&body, &ids);
                    let ids = ids.into_iter().zip(once).collect();
                    (ids, body)
                });
        PreparedLambda {
            definition,
            lambda_mark: self.last_lambda_id,
        }
    }

    /// The parameters and body of the LAMBDA `lambda_result` when called with `provided`
    /// arguments, `#VALUE!` for a wrong count or for a value that is not a LAMBDA.
    fn lambda_definition(
        &self,
        lambda_result: &CalcResult,
        provided: usize,
        cell: CellReferenceIndex,
    ) -> Result<(Vec<NamedVariable>, Node), CalcResult> {
        let id = match lambda_result {
            CalcResult::Lambda(id) => *id,
            other if other.is_error() => return Err(other.clone()),
            _ => {
                return Err(CalcResult::new_error(
                    Error::VALUE,
                    cell,
                    "Expected a LAMBDA".to_string(),
                ))
            }
        };
        if let Some(kind) = self.eta_lambdas.get(&id) {
            return Ok(eta_definition(kind.clone(), provided));
        }
        let Some(definition) = self.lambdas.get(&id) else {
            return Err(CalcResult::new_error(
                Error::NAME,
                cell,
                "Lambda not found in store".to_string(),
            ));
        };
        // Checked before the copy: a wrong count copies nothing.
        check_arg_count(&definition.0, provided, cell)?;
        Ok(definition.as_ref().clone())
    }

    /// An unbound name: a function passed by name (eta-reduced, stored `_xleta.SUM`, any case,
    /// `_xlfn.` allowed after the prefix) is a LAMBDA, `#NAME?` for anything else.
    /// Excel: see docs/excel-semantics/arrays.md "Functions passed by name".
    pub(crate) fn unbound_name(&mut self, name: &str, cell: CellReferenceIndex) -> CalcResult {
        let function = name
            .get(..ETA_PREFIX.len())
            .filter(|prefix| prefix.eq_ignore_ascii_case(ETA_PREFIX))
            .and_then(|_| name.get(ETA_PREFIX.len()..))
            // Stored formulas use the English function names, whatever the UI language.
            .and_then(|function| get_default_language().functions.lookup_stored(function));
        match function {
            Some(kind) => self.eta_lambda(kind),
            None => CalcResult::new_error(
                Error::NAME,
                cell,
                format!("Variable name \"{name}\" not found."),
            ),
        }
    }

    /// The function `kind` passed by name, as a LAMBDA value (what `_xleta.NAME` evaluates to).
    pub(crate) fn eta_lambda(&mut self, kind: Function) -> CalcResult {
        let id = self.get_next_lambda_id();
        self.eta_lambdas.insert(id, kind);
        CalcResult::Lambda(id)
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

        CalcResult::Lambda(self.store_lambda(Arc::new((parameters, body))))
    }

    /// Gives the LAMBDAs a LET or a LAMBDA call made, and that outlive it, what they read of its
    /// variables `ids` (`mark` is the first lambda id it could have made). Called at the end of
    /// that scope, while `ids` are still bound: a LAMBDA leaves a scope only through its result
    /// (a REDUCE or SCAN accumulator, a LET-bound `mk(5)`), so the LAMBDAs reachable from it, the
    /// result itself and those its captures hold, capture each variable of `ids` their bodies
    /// read, as it is bound now. Excel's closures read those values:
    /// `LET(mk,LAMBDA(a,LAMBDA(b,a+b)),f,mk(1),a,50,f(0))` is 1
    /// (docs/excel-semantics/arrays.md "LAMBDAs that outlive their call",
    /// probes/lambda-escaping-closures, T-0439). A LAMBDA used only inside the scope captures
    /// nothing, so a LAMBDA made on each REDUCE step copies nothing.
    pub(crate) fn close_over(
        &mut self,
        result: Option<usize>,
        ids: &[usize],
        mark: usize,
        cell: CellReferenceIndex,
    ) {
        let Some(result) = result.filter(|_| !ids.is_empty()) else {
            return;
        };
        let mut pending = vec![result];
        let mut seen = HashSet::new();
        while let Some(lambda) = pending.pop() {
            if lambda < mark || !seen.insert(lambda) {
                continue;
            }
            let held = self.lambda_captures.get(&lambda).into_iter().flatten();
            pending.extend(held.filter_map(|(_, binding)| binding.lambda()));
            let Some(definition) = self.lambdas.get(&lambda) else {
                continue;
            };
            let mut read = Vec::new();
            definition.1.reads_variable(|id| {
                if ids.contains(&id) {
                    read.push(id);
                }
                false
            });
            read.sort_unstable();
            read.dedup();
            for id in read {
                let cells = self.variable_stack.cells_to_share(id);
                self.captured_cells = self.captured_cells.saturating_add(cells);
                let binding = if self.captured_cells > MAX_CAPTURED_CELLS {
                    // DEVIATION (decision 0014): arrays.md "LAMBDAs that outlive their call".
                    Some(Arc::new(Binding::Value(CalcResult::new_error(
                        Error::NUM,
                        cell,
                        "LAMBDAs capture too many cells".to_string(),
                    ))))
                } else {
                    self.variable_stack.shared(id)
                };
                if let Some(binding) = binding {
                    pending.extend(binding.lambda());
                    self.lambda_captures
                        .entry(lambda)
                        .or_default()
                        .push((id, binding));
                }
            }
        }
    }

    /// Binds what the LAMBDA `lambda` captured (`close_over`) for one call of it, each
    /// variable that is not bound already, and gives back the ids it bound, for
    /// [`Self::unbind_captures`]. A bound one is in scope still, bound to what was captured: the
    /// same LET or call made the LAMBDA, since every evaluation of a LET, and every call of a
    /// body that made a LAMBDA reading its parameters, binds fresh ids (`call_prepared`). So a
    /// LAMBDA called where it was made copies nothing.
    fn bind_captures(&mut self, lambda: &CalcResult, cell: CellReferenceIndex) -> Vec<usize> {
        let CalcResult::Lambda(id) = lambda else {
            return Vec::new();
        };
        let Some(captures) = self.lambda_captures.get(id) else {
            return Vec::new();
        };
        let mut bound = Vec::new();
        for (variable, binding) in captures {
            if self.variable_stack.get(*variable).is_none() {
                self.variable_stack
                    .insert(*variable, binding.as_ref().clone(), cell);
                bound.push(*variable);
            }
        }
        bound
    }

    /// Unbinds what [`Self::bind_captures`] bound.
    fn unbind_captures(&mut self, bound: Vec<usize>) {
        for variable in bound {
            self.variable_stack.remove(variable);
        }
    }

    /// Stores `definition` under a new lambda id, which it returns.
    pub(crate) fn store_lambda(&mut self, definition: SharedLambda) -> usize {
        let id = self.get_next_lambda_id();
        self.lambdas.insert(id, definition);
        id
    }

    /// The LAMBDA a defined name holds, as a value: the same lambda id each time the name is
    /// used in one evaluation (`NamedLambdas`).
    pub(crate) fn defined_name_lambda(&mut self, definition: SharedLambda) -> CalcResult {
        let address = Arc::as_ptr(&definition).addr();
        if let Some(&(_, id)) = self.named_lambdas.ids.get(&address) {
            return CalcResult::Lambda(id);
        }
        let id = self.store_lambda(Arc::clone(&definition));
        self.named_lambdas.ids.insert(address, (definition, id));
        self.named_lambdas.named.insert(id);
        CalcResult::Lambda(id)
    }
}

/// How a function passed by name is stored in a file (docs/excel-semantics/xlsx-storage.md).
const ETA_PREFIX: &str = "_xleta.";

/// The LAMBDA a function passed by name stands for when called with `provided` arguments: it
/// takes as many parameters as it is given and passes them all on, so `BYROW(r,SUM)` is
/// `BYROW(r,LAMBDA(a,SUM(a)))` and `MAKEARRAY(2,2,SUM)` sums row and column
/// (probes/m7-argument-modes F28:H51, probes/eta-reduced-names, T-1466).
fn eta_definition(kind: Function, provided: usize) -> (Vec<NamedVariable>, Node) {
    let parameters: Vec<NamedVariable> = (1..=provided)
        .map(|i| NamedVariable {
            name: format!("{ETA_PREFIX}{i}"),
            id: None,
            is_optional: false,
        })
        .collect();
    let args = parameters
        .iter()
        .map(|parameter| Node::NamedVariableKind {
            name: parameter.name.clone(),
            id: None,
        })
        .collect();
    (parameters, Node::FunctionKind { kind, args })
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

#[cfg(test)]
mod tests {
    #![allow(clippy::panic, reason = "tests")]

    use std::collections::HashMap;

    use super::{assign_variable_ids, read_once};
    use crate::expressions::{
        parser::{new_parser_english, Node},
        types::CellReferenceRC,
    };

    /// `read_once` for the parameters `a` (id 1) and `x` (id 2) of `LAMBDA(a,x,body)`.
    fn read_once_of(body: &str) -> Vec<bool> {
        let mut parser = new_parser_english(vec!["Sheet1".to_string()], vec![], HashMap::new());
        let context = CellReferenceRC {
            sheet: "Sheet1".to_string(),
            row: 1,
            column: 1,
        };
        let Node::LambdaDefKind { mut body, .. } =
            parser.parse(&format!("LAMBDA(a,x,{body})"), &context)
        else {
            panic!("not a LAMBDA: {body}");
        };
        assign_variable_ids(&mut body, "a", 1);
        assign_variable_ids(&mut body, "x", 2);
        read_once(&body, &[1, 2])
    }

    #[test]
    fn reads_through_once_positions_count_once() {
        // T-1588's positions: the body itself and VSTACK/HSTACK arguments, through groups.
        assert_eq!(read_once_of("a"), [true, false]);
        assert_eq!(read_once_of("(VSTACK(HSTACK((a),1),x))"), [true, true]);
        // T-1593: the branches of IF are exclusive, and its condition is read once.
        assert_eq!(read_once_of("IF(x>0,VSTACK(a,x),a)"), [true, false]);
        assert_eq!(read_once_of("IF(a,1,IF(x,2,3))"), [true, true]);
        // LET's values and body.
        assert_eq!(read_once_of("LET(y,x*2,VSTACK(a,y))"), [true, false]);
        assert_eq!(read_once_of("LET(b,a,c,x,VSTACK(b,c))"), [true, true]);
        // The array of SORT, TAKE and DROP.
        assert_eq!(read_once_of("SORT(VSTACK(a,x))"), [true, true]);
        assert_eq!(read_once_of("TAKE(a,1)"), [true, false]);
        assert_eq!(read_once_of("DROP(a,x)"), [true, false]);
    }

    #[test]
    fn reads_more_than_once_or_elsewhere_do_not_count_once() {
        // An IF condition and a branch both read; a branch and a read outside the IF.
        assert_eq!(read_once_of("IF(SUM(a)>0,a,0)"), [false, false]);
        assert_eq!(read_once_of("IF(a,a,0)"), [false, false]);
        assert_eq!(read_once_of("VSTACK(IF(x,a,0),a)"), [false, true]);
        // One branch reading twice.
        assert_eq!(read_once_of("IF(x,VSTACK(a,a),a)"), [false, true]);
        // LET reading in a value and the body.
        assert_eq!(read_once_of("LET(b,a,VSTACK(a,b))"), [false, false]);
        // SORT, TAKE and DROP read their other arguments in ways not audited.
        assert_eq!(read_once_of("SORT(a,COLUMNS(a))"), [false, false]);
        assert_eq!(read_once_of("TAKE(x,a)"), [false, true]);
        assert_eq!(read_once_of("DROP(a,ROWS(a)-1)"), [false, false]);
        // A function outside the audited ones, and a nested LAMBDA.
        assert_eq!(read_once_of("SUM(a)"), [false, false]);
        assert_eq!(read_once_of("IF(x,MAP(x,LAMBDA(y,a)),0)"), [false, false]);
    }
}
