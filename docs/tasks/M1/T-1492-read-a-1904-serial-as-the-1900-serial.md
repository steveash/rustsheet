---
id: T-1492
title: "Read a 1904 serial as the 1900 serial + 2*1462 in every financial date reader"
milestone: M1
lane: functions-financial
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Every 1904 financial date reader (COUP*, PRICE, YIELD, DURATION, MDURATION, YEARFRAC, ACCRINT, DISC and the like) reads a 1904 serial s as the 1900 serial s + 2*1462 and counts days in that calendar, as the odd-period functions already do (`DateSystem::odd_period_1900_serial`), instead of s + 1463 (`DateSystem::financial_1900_serial`).

## Context
T-1489 verified the rule on 165 cells (dates.md "The 1904 date system", probes/date-system-1904-financial-readers). Today 97 of them differ or fail the last digits from Excel, e.g. COUPNCD basis 1 70309 against 70308. Date-returning functions must convert their result back (COUPPCD and COUPNCD differ from the 1900 twin by 2*1462 serials). Mind the bounds: dates read past 9999-12-31 are `#NUM!` (see odd_period_1900_serial). Other readers than the 11 probed are unprobed; switch them only where the probe or an existing case covers them, and list the rest.

## Acceptance criteria
- [ ] The cells of probes/date-system-1904-financial-readers leave eval/baseline via `make eval-baseline`, with no new F line in any suite.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The odd-period functions (done in T-1457).
