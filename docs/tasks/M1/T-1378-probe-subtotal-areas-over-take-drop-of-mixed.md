---
id: T-1378
title: "Probe SUBTOTAL/AREAS over TAKE/DROP of mixed-kind IF, nested or shadowed LET, and CHOOSE/SWITCH/IFS (follow-up of T-1365)"
milestone: M1
lane: functions-math
status: done
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
- [x] Case-insensitive LET names in `yields_cells` (`LET(x,SEQUENCE(4),X)`) are handled.
- [x] A probe case with oracle values for the forms above; the engine matches it; no debug_assert trips.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probes `subtotal-reference-operands-f` (97 cells) and `let-lambda-name-case` (14 cells), both fully passing (eval: probes +2 cases, 111 new bit-exact cells, known failures unchanged). `yields_cells` (static_analysis/aggregate.rs) gained CHOOSE, SWITCH and IFS arms, so a computed array in any arm spills `#VALUE!` instead of tripping the debug_assert. LET names and LAMBDA parameters now match case-insensitively through one helper, `same_variable` (functions/logical/let.rs), used for binding, static analysis and the `_xlpm.` export renaming. Excel for the web refused the duplicate-name-by-case and non-ASCII forms; follow-up T-1387. Opus resumed the sonnet handoff: the probe showed LAMBDA parameters are case-insensitive too, which resolved the open `LET(x,1,LAMBDA(X,x*10)(2))` finding (20).
