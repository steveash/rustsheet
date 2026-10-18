---
id: T-1920
title: "Probe CEILING/FLOOR results past the double range (CEILING(1e300,1e-300))"
milestone: M9
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for what Excel returns when a CEILING/FLOOR-family result passes the double range,
so the engine matches it.

## Context
T-1773's security review noted that `snap_ratio(value, significance)` and the `.ceil()/.floor() *
significance` product in functions/math_and_trigonometry/ceiling_floor.rs can overflow to an
infinity (e.g. `CEILING(1e300,1e-300)`, `FLOOR.MATH(-1e308,1e-10)`), and `number_result` does not
map a non-finite result to `#NUM!`. The behavior predates T-1773 (the code moved unchanged).

## Acceptance criteria
- [ ] A probe case (excel-probe skill) with oracle values for `CEILING(1e300,1e-300)`,
      `FLOOR(1e300,1e-300)`, `CEILING.MATH(1e308,1e-10)`, `FLOOR.MATH(-1e308,1e-10)`,
      `CEILING.PRECISE(1e300,1e-300)` and `ISO.CEILING(1e300,1e-300)`.
- [ ] The finding is in docs/excel-semantics/numbers.md; a mismatch is fixed here (size S, likely
      `#NUM!` for a non-finite result in the family's kernel) or filed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- MROUND (T-1785).
