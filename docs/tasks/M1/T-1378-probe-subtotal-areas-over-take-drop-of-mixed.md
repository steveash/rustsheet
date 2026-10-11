---
id: T-1378
title: "Probe SUBTOTAL/AREAS over TAKE/DROP of mixed-kind IF, nested or shadowed LET, and CHOOSE/SWITCH/IFS (follow-up of T-1365)"
milestone: M1
lane: functions-math
status: todo
priority: P3
size: S
tier: sonnet
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
