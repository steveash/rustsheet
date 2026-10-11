---
id: T-1378
title: "Probe SUBTOTAL/AREAS over TAKE/DROP of mixed-kind IF, nested or shadowed LET, and CHOOSE/SWITCH/IFS (follow-up of T-1365)"
milestone: M1
lane: functions-math
status: in-progress
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Oracle evidence (excel-probe skill) for what SUBTOTAL/AREAS do with `TAKE(IF(TRUE,A1:A4,SEQUENCE(4)),2)`, `IF(FALSE,...)`, `IF(TRUE,A1:A4,5)`, nested/shadowed LET (`LET(x,SEQUENCE(4),LET(x,A1:A4,x))`), and TAKE/DROP of CHOOSE, SWITCH, IFS holding a computed array; then make the engine match.

## Context
T-1365 added `yields_cells` (static_analysis/aggregate.rs) with unprobed rules: an IF is cells only when every branch is; a LET variable sees earlier bindings only. CHOOSE/SWITCH/IFS still count as cells via `is_reference_operand`, and `SUBTOTAL(9,TAKE(CHOOSE(1,SEQUENCE(4)),2))` trips the debug_assert in model.rs (~995) on main.

## Acceptance criteria
- [ ] Case-insensitive LET names in `yields_cells` (`LET(x,SEQUENCE(4),X)`) are handled.
- [ ] A probe case with oracle values for the forms above; the engine matches it; no debug_assert trips.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Handoff (resumed)
Done and pushed: probe `subtotal-reference-operands-f` (97 oracle cells, all passing); `yields_cells` CHOOSE/SWITCH/IFS arms (aggregate.rs) fix the debug_assert; LET names case-insensitive (`same_variable`, `assign_variable_ids(.., ci)` in functions/logical/let.rs; `ci` true for LET, false for LAMBDA binding in lambda.rs:118/173); docs entry in references.md; follow-up T-1387. `make check` was green on the last commit.

Open (reviewer-excel-fidelity, REQUEST_CHANGES twice): LET case folding interacts badly with LAMBDA parameters of a different case. With `ci` true the LambdaDefKind arm treats a param `X` as shadowing LET name `x` and stops, while LAMBDA binding (exact match) does not bind a lowercase `x` in the body, so `=LET(x,1,LAMBDA(X,x*10)(2))` gives `#NAME?` (main: 10). `LET(x,1,LAMBDA(X,X*10)(2))` is 20 (covered by `test_lambda_parameter_case_is_unchanged_by_let_case_folding`).

Next: probe the LAMBDA forms (`LET(x,1,LAMBDA(X,x*10)(2))`, `LAMBDA(x,X+1)(2)`, `LET(x,1,LET(X,5,x))`, duplicate `LET(x,1,X,2,x)`) on excel-graph, implement what the oracle shows (this is T-1387's scope; fold it in or do it first), or in the interim keep main's behavior in the LAMBDA body by skipping nodes whose name exactly equals a parameter when `ci` is true. Minor: use `saturating_sub` for `args.len() - 2` in aggregate.rs's SWITCH arm; say "same-case" in the nested LET shadowing note.
