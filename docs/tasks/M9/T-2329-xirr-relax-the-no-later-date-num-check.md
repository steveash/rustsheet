---
id: T-2329
title: "XIRR: relax the no-later-date #NUM! check for a zero first value, apply T-2328's stop bound"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
XIRR returns Excel's value, not `#NUM!`, for a zero first value whose date is after every other date, and applies the T-2328 stop bound once it lands.

## Context
probes/xirr-span-zero-first A28-A30 (engine `#NUM!`, Excel 2.98e-9 / 3.73e-9) contradict the engine's no-later-date check (`xirr_dates`, functions/financial/mod.rs). A11, A14, A20 need T-2328's stop bound. See financial.md "XIRR is a bracketing bisection" and "XIRR reads empty cells as 0".

## Acceptance criteria
- [ ] A28-A30 pass; the nonzero-first-value no-later-date cases (xirr-mismatched-empties X9, X12, X13) stay `#NUM!`.
- [ ] A11, A14, A20 pass once T-2328 has fitted the bound.
- [ ] `make check` green; reviewers approve.

## Out of scope
Probing the exact condition (a zero first value with all other dates equal).
