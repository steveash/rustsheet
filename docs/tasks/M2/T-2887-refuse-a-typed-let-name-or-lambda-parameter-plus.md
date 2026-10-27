---
id: T-2887
title: "Refuse a typed LET name or LAMBDA parameter written with a unary plus: =LET(+x,1,x), =LAMBDA(+x,x)(1)"
milestone: M2
lane: engine
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Typed `=LET(+x,1,x)` and `=LAMBDA(+x,x)(1)` are refused (`#ERROR!`, as other refused typed formulas), as Excel refuses
them, instead of evaluating to `#VALUE!`.

## Context
Evidence: T-2772 typed `=LAMBDA(+x,x)(1)` and `=LET(+x,1,x)` on excel-graph; both got Graph PATCH 400
`InvalidArgument` (2026-10-27). A refused input cannot be committed as a case, so the log is the evidence
(docs/excel-semantics/syntax.md "A typed unary plus stays in the formula text"). We give `#VALUE!` for both:
LET's evaluator (functions/logical/let.rs, the `_ =>` arm of the name match) reads a non-name declaration as a run-time
`#VALUE!`, and the LAMBDA parameter list in `parse_lambda_items` (expressions/parser/lambda.rs) does not refuse a `+x`
item. Check how a duplicate LET name is refused (`#ERROR!`, test_fn_let.rs) and how other non-name LET/LAMBDA
declarations behave in Excel (`LET(1,1,1)` is unprobed: probe it before widening the refusal past a plus).

## Acceptance criteria
- [ ] `=LET(+x,1,x)` and `=LAMBDA(+x,x)(1)` typed give `#ERROR!`, pinned by unit tests.
- [ ] syntax.md updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
- A plus inside the LAMBDA body (`LAMBDA(x,+x)` is valid).
