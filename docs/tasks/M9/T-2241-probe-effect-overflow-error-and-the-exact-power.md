---
id: T-2241
title: "Probe EFFECT overflow error and the exact-power-of-ten base (EFFECT(1E300,2), EFFECT(270,30))"
milestone: M9
lane: functions-financial
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
docs/excel-semantics/financial.md "EFFECT and NOMINAL raise to their power with `^`" records Excel's value for the two
edge cases that T-2034 left unprobed. The engine then matches Excel on both.

## Context
T-2034 moved EFFECT and NOMINAL onto `arithmetic::power` (Excel's `^`). The Excel-fidelity review raised two
behaviours that no oracle cell covers:
- Overflow: `rate_power` (functions/financial/rate_conversions.rs) maps an overflowing EFFECT to `#DIV/0!`, as the old
  infinite-rate path did. Excel's `^` itself is `#NUM!` on overflow. Excel's EFFECT could be either.
- Exact powers of ten: `power` returns 1E{n} exactly for a base of exactly 10 and n from 23 to 308
  (`decimal_power_of_ten`), so `EFFECT(270,30)` is now 1E30 - 1. Excel may instead square-and-multiply there.
Write a probes case per the `excel-probe` skill. Suggested cells: `EFFECT(1E300,2)`, `EFFECT(1E200,3)`,
`EFFECT(1E10,100)`, `EFFECT(270,30)`, `EFFECT(9,1)`, `EFFECT(18,2)`, `EFFECT(990,99)`, `NOMINAL(1E300,0.5)`. Get the
oracle values and record them in the entry.

## Acceptance criteria
- [ ] A probes case with the cells above has excel-graph values, and its known failures, if any, are in the baseline.
- [ ] The financial.md entry states Excel's overflow error and its exact-power-of-ten behaviour, citing the cells, and
      drops "Not probed".
- [ ] If the engine differs, `rate_power` is changed to match (a one-line error mapping or a guard), with a unit test.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other financial functions' overflow errors.
