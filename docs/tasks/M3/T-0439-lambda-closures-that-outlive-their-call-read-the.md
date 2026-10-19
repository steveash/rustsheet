---
id: T-0439
title: "LAMBDA closures that outlive their call read the parameter values they were made with"
milestone: M3
lane: engine
status: in-progress
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
- [ ] Probe cases with oracle values for escaping closures; a semantics entry citing them.
- [ ] The probed formulas match Excel; closure tests in test_fn_map_reduce.rs still pass.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Recursive LAMBDAs through defined names (unchanged).
