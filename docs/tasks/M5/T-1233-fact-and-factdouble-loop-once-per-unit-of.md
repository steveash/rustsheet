---
id: T-1233
title: "FACT and FACTDOUBLE loop once per unit of the argument: bound the work"
milestone: M5
lane: functions-math
status: done
priority: P0
size: S
tier: sonnet
depends_on: []
---

## Goal
FACT and FACTDOUBLE take bounded time for any argument (CLAUDE.md directive 5) and give Excel's value or error for large arguments.

## Context
Found while fixing T-1217 (MULTINOMIAL). `fn_fact` and `fn_factdouble` (`single_number_fn!` in
crates/rustsheet-engine/src/functions/math_and_trigonometry/mathematical.rs) multiply in a `while k <= x` loop, so
`FACT(1E+15)` runs 10^15 iterations even though the product is infinite after 170 steps; `FACTDOUBLE(1E+15)` likewise.
Probe first (excel-probe skill): `FACT(170)`, `FACT(171)`, `FACT(1E+15)`, `FACT(2^53)`, `FACTDOUBLE(300)`, `FACTDOUBLE(301)`,
`FACTDOUBLE(1E+15)`; Excel is probably `#NUM!` past the overflow point (MULTINOMIAL is `#NUM!` past a sum of 170,
docs/excel-semantics/numbers.md "MULTINOMIAL of a huge argument"). Then exit the loop as soon as the product overflows.

## Acceptance criteria
- [x] A unit test evaluates `FACT(1E+15)` and `FACTDOUBLE(1E+15)` quickly with no panic and Excel's result.
- [x] The probe values are recorded in docs/excel-semantics/numbers.md and match the engine.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
`fn_fact`/`fn_factdouble` return `#NUM!` at the first infinite product, so `FACT(1E+15)` is immediate. Probed on excel-graph (probes/fact-large-arguments), recorded in numbers.md, unit test added. Eval delta: +1 case, 18 cells, 13 pass; 5 new known failures (last-digit differences in FACT(170.9), FACTDOUBLE values near the limit) filed as T-1249.
