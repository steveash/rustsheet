---
id: T-0439
title: "LAMBDA closures that outlive their call read the parameter values they were made with"
milestone: M3
lane: engine
status: done
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
A LAMBDA made inside a LAMBDA call and used after that call returns reads the values its outer
parameters had when it was made, as Excel does, instead of `#NAME?`:
`=LET(g, REDUCE(LAMBDA(z,z), {1,2,3}, LAMBDA(f, x, LAMBDA(z, f(z)*10+x))), g(0))`.

## Context
Found while doing T-0425. Variables are bound by id on `variable_stack` only for the length of a
call (`Model::call_prepared` and `call_lambda` in functions/logical/lambda.rs remove them after),
and a LAMBDA value stores only its body (`Model::lambdas`), so a closure that escapes (a REDUCE or
SCAN accumulator, a LET-bound result) finds its outer ids unbound: the formula above is `#NAME?` on
main. Excel is expected to give 123 (lexical closures), but there is no oracle evidence yet: probe
it first (`excel-probe`: the formula above, the same through SCAN and through a LET-bound inline
call, and a closure read after its outer parameter is rebound) and record the result in
docs/excel-semantics/arrays.md. The likely fix captures the values of the outer ids a LAMBDA body
reads when the LAMBDA is created (cf. `Node::reads_variable`), which then also lets
`call_prepared` keep its patched body whatever the body creates.

## Acceptance criteria
- [x] Probe cases with oracle values for escaping closures; a semantics entry citing them.
- [x] The probed formulas match Excel; closure tests in test_fn_map_reduce.rs still pass.
- [x] `make check` green; reviewers approve.

## Out of scope
- Recursive LAMBDAs through defined names (unchanged).

## Outcome
A LAMBDA made inside a LET or a LAMBDA call now captures, when it is made, the bound variables its body
reads (`Model::store_closure`; one shared copy per variable, `VariableStack::shared`), and each call
binds the captured ones that are not in scope (`bind_captures` in `call_prepared`); captures are
dropped after each root cell. Probed on excel-graph (probes/lambda-escaping-closures, 28 cells): 20
pass, among them the REDUCE chain (123), LET-bound curried calls, rebinding and shadowing, and captured
references, unions and errors. The other 8 are separate gaps, filed: chained calls `f(..)(..)`
(H3, H8, H10: T-2038), arrays of LAMBDAs from SCAN (H2, I2, J2: T-2039) and LET names spelled like
cells (H4, H5: T-2040). `call_prepared` still patches a fresh body when the body makes a LAMBDA that
reads its parameters: closures rely on those fresh ids, so the Context's idea of keeping the patch
was dropped.
Review found that self-calls through a parameter (`f(f,20000)`, already on main) and long closure
chains overflowed the stack; nested LAMBDA calls now stop at four times the evaluation's stack budget
as `#NUM!` (decision 0080, DEVIATION "How deep LAMBDA calls nest"; probes/lambda-call-depth shows
Excel evaluates 300 levels, ours about 200 in release, 4 cells listed as deviations).
Eval: probes +2 cases (40 cells: 28 pass, 4 deviations, 8 new-coverage known failures), 0 regressions.
