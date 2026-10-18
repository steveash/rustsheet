---
id: T-1494
title: "ACCRINT basis 0 calc_method TRUE is #NUM! in a 1904 workbook for a settlement near 9995-12-30 (unexplained)"
milestone: M1
lane: functions-financial
status: done
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
- [x] A probe with its 1900 twin pins the rule; dates.md and the engine match it.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probes date-system-1904-accrint-settlement-bound and its 1900 twin date-system-1900-accrint-settlement-bound
(56 cells each, excel-graph) pin two rules, now in dates.md and financial.md and matched by `fn_accrint`:
- calc_method TRUE: #NUM! when the coupon date ending the settlement's period is past 9999-12-31, in both date
  systems (so from 9995-12-30 on in 1904 for these bonds); `compute_accrint` checks the walked next coupon.
- calc_method FALSE in 1904: every basis takes a settlement up to the last 1904 date
  (`accrint_settlement_limit` no longer depends on basis); bases 1-3 count actual days to the settlement read past
  9999-12-31, bases 0 and 4 take first_interest's 1904 serial + 1462 as the settlement slice's day count.
Eval delta: probes known failures 5152 -> 5142 (10 fixed in date-system-1904-financial-settlement-bound; the two new
cases pass 112/112), 0 regressions.
