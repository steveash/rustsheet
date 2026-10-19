---
id: T-0439
title: "LAMBDA closures that outlive their call read the parameter values they were made with"
milestone: M3
lane: engine
status: blocked
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
When a LET or a LAMBDA call ends, the LAMBDAs reachable from its result now capture the variables of
that scope their bodies read (`Model::close_over`; one shared copy per variable,
`VariableStack::shared`), and each call binds the captured ones that are not in scope (`bind_captures`
in `call_prepared`); captures are dropped after each root cell. Capturing when a LAMBDA is made, the
first version, copied a REDUCE accumulator on every step (found in review), so a LAMBDA that never
leaves its scope now captures nothing, and one cell's closures may hold at most 4,194,304 captured
cells (`MAX_CAPTURED_CELLS`; past it a capture is `#NUM!`, DEVIATION). Probed on excel-graph (probes/lambda-escaping-closures, 28 cells): 20
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

## Blocked because
Review round 3 (reviewer-performance, head 6953374d) found one blocking cost: `bind_captures`
deep-copies every captured binding on each call of an escaped closure, so
`=LET(mk,LAMBDA(r,LAMBDA(i,IF(i<0,INDEX(r,1),i))),f,mk(SEQUENCE(n)),SUM(MAP(SEQUENCE(n),f)))` is
O(n^2): 3.6 s at n=10,000 against 7 ms on main (main never reads `r`, so it got the right sum).
Security, architecture and quality approve; the Excel-fidelity round-2 verdict on the scope-exit
design was still pending when this run stopped. Next step: make captured bindings shared, not
copied (a `Binding::Shared(Arc<Binding>)` that `Binding::value`, `variable_reference` and
`read_value` read through, with `bind_captures` cloning only the `Arc` and putting it into
`VariableStack::shared` so an inner capture reuses it); add a test for MAP over an escaped closure
that captured a large array; guard `shared.remove` in `insert`/`remove` with `!shared.is_empty()`;
record bench numbers (`cargo bench -p rustsheet --bench lift -- WORKDAY`: 61.6 ms branch vs
62.7 ms main for the no-capture path); then re-run the performance and fidelity reviewers.
PR #1226 (draft).
Excel-fidelity round 2 (head 56a019f1) also requests changes, docs only: give the
`MAX_CAPTURED_CELLS` cap its own arrays.md entry with `Status: DEVIATION (decision 0014)` (Excel
unprobed) and point the code comment at it; say the capture budget is per root evaluation (cells
evaluated as dependencies share it) or reset it per formula; write the depth entry's Status as
`DEVIATION (decision 0014)` and cite 0080 in "Ours"; note that a debug build stops near 50 levels;
file a follow-up (smaller LAMBDA call frames or a larger default budget) so probes/lambda-call-depth
H3, H6, I3, I6 can leave `eval/suites/probes/deviations`.
Performance round 3b (head 67d3bbb1) confirms the per-call copy blocker and adds: plain LAMBDA calls
are 6-15% slower (`SUM(MAP(SEQUENCE(300000),LAMBDA(x,x*2)))` 153 -> 163 ms, REDUCE sum 123 -> 136 ms,
MAP with a LET inside 264 -> 305 ms, release medians). Causes: `captured.extend(ids)` allocates on
every call (pass the parameter ids to `close_over` separately, or skip when `result.lambda()` is
None), and `shared.remove` runs on every bind and unbind (guard with `!shared.is_empty()`). Add a
MAP/REDUCE/LET bench to crates/rustsheet/benches/lift.rs and record before/after numbers. Also move
`cells_to_share` above `shared`'s doc comment (the two docs merged). A `Binding::Shared(Arc<_>)`
read through by `Binding::value` would also fix main's O(n^2) array reads inside LAMBDA bodies.

