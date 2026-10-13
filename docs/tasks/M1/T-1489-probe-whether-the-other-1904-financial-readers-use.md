---
id: T-1489
title: "Probe whether the other 1904 financial readers use serial + 2*1462 near non-leap century years"
milestone: M1
lane: functions-financial
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
We know whether COUP*, PRICE, YIELD, DURATION, YEARFRAC, ACCRINT and the other 1904 financial date readers read a
1904 serial s as the 1900 serial s + 1463 (`DateSystem::financial_1900_serial`) or as s + 2*1462, as the
odd-period functions do (`DateSystem::odd_period_1900_serial`, T-1457).

## Context
The two readings give the same day counts except for dates within four years before 1 March of a non-leap century
year (2100, 2200, ...), where s + 2*1462 is a day earlier, and around 29 February of the later calendar
(docs/excel-semantics/dates.md, the odd-period paragraph under "The 1904 date system"). The coupon schedule bound
(m + 2*1462 past the last date) hints that every financial reader uses s + 2*1462. Probe (excel-probe) in a 1904
workbook: COUPPCD, COUPNCD, COUPDAYBS, COUPDAYS (bases 0 and 1), PRICE, YEARFRAC (bases 0 and 1) and ACCRINT with
settlement 2096-03-01 or 2100-02-15 and maturity 2100-06-30 or 2104-02-27, and their 1900 twins.

## Acceptance criteria
- [ ] Probe cases with excel-graph oracle values committed; the finding recorded in dates.md.
- [ ] If the readers use s + 2*1462, a follow-up task to switch `financial_1900_serial` is filed (or the switch is
      made here if it is a one-line change with no regressions).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The odd-period functions (done in T-1457).
