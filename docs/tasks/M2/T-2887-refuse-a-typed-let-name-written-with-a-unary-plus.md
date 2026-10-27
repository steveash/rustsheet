---
id: T-2887
title: "Refuse a typed LET name written with a unary plus: =LET(+x,1,x)"
milestone: M2
lane: engine
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
A typed `=LET(+x,1,x)` is refused (`#ERROR!`, as a duplicate LET name is), as Excel refuses it, instead of
evaluating to `#VALUE!`.

## Context
Evidence: T-2772 typed `=LET(+x,1,x)` and `=LAMBDA(+x,x)(1)` on excel-graph, and Excel refused both with a Graph PATCH 400
`InvalidArgument` (docs/research/t2772-let-lambda-plus-refusal-2026-10-27.md; docs/excel-semantics/syntax.md "A typed
unary plus stays in the formula text"). We already refuse the LAMBDA form (`#ERROR!`,
`test_lambda_parameter_under_a_unary_plus_is_refused` in test_fn_let.rs). The LET form gives `#VALUE!`: LET's evaluator
(functions/logical/let.rs, the `_ =>` arm of the name match) reads a declaration that is not a name as a run-time `#VALUE!`.
Find where the LAMBDA form is refused and how a duplicate LET name is refused (`#ERROR!`), and refuse a unary plus over a
LET name the same way. Other declarations that are not names (`LET(1,1,1)`) are unprobed: probe them before widening the
refusal beyond a plus.

## Acceptance criteria
- [ ] Typed `=LET(+x,1,x)` gives `#ERROR!`, pinned by a unit test; the TODO(T-2887) in test_fn_let.rs is resolved.
- [ ] syntax.md updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
- A plus inside a LET value or body (`LET(x,1,+x)` is valid).
