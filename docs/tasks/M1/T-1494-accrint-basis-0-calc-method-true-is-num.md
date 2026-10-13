---
id: T-1494
title: "ACCRINT basis 0 calc_method TRUE is #NUM! in a 1904 workbook for a settlement near 9995-12-30 (unexplained)"
milestone: M1
lane: functions-financial
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Establish and match ACCRINT in a 1904 workbook for settlements from 9995-12-30 on when not basis 1 / calc_method FALSE: calc_method TRUE is `#NUM!` (bases 0 and 1), and basis 0 / FALSE gives 410415.83 for every settlement from 9995-12-31 (probes/date-system-1904-financial-settlement-bound columns D-F).

## Context
T-1492 probed ACCRINT settlements near the 1904 upper bound (probes/date-system-1904-financial-settlement-bound, dates.md); the basis 1 / calc_method FALSE column matched, but a basis 0 / calc_method TRUE column was `#NUM!` for every settlement from 9995-12-30 (where YEARFRAC still works) while the engine gives a value; column F (basis 0, calc FALSE) gives 410415.83. The failing cells are baselined. Needs a probe (vary basis, calc_method, first_interest, and the 1900 twin) to find the rule. Code: `Model::fn_accrint` (accrint.rs).

## Acceptance criteria
- [ ] A probe with its 1900 twin pins the rule; dates.md and the engine match it.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
