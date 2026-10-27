---
id: T-2843
title: "Probe 1904 argument-error order of ACCRINT, COUPNUM and TBILL* with late dates, and mid-9999 T-bill settlements"
milestone: M9
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
docs/excel-semantics/dates.md "The 1904 date system: ACCRINT, COUPNUM and the TBILL functions read the date itself"
has no UNVERIFIED line on error order, and financial.md "T-bills settled in 9999" is pinned at mid-year too.

## Context
- Since T-2337 the engine range-checks the dates of ACCRINT, COUPNUM, TBILLPRICE, TBILLYIELD and TBILLEQ in a 1904
  workbook in the date's own reading as they are read (`DateReading::Calendar`, functions/financial/dates.rs), so a
  1904 date in 9996-9999 followed by an error argument now gives that error (`TBILLEQ(DATE(9996,1,1),NA(),0.05)` is
  `#N/A`, `#NUM!` before T-2337). No 1904 probe pairs a late date with a later error argument
  (probes/securities-argument-error-order is 1900 only).
- The 9999 T-bill cut is probed at 9999-01-01 and late 9999 only (probes/fin-1900-tbill-last-day).
- Use the excel-probe skill: one new 1904 case (error order: ACCRINT with a 9996-9999 first_interest or settlement
  before `NA()`, COUPNUM and TBILL* with a 9996 settlement before `NA()`/text, and a 1904 serial past 2957003 before an
  error), and mid-year 9999 settlements (9999-06-30, 9999-07-01) in both systems.

## Acceptance criteria
- [ ] The new probe cases are filled through excel-graph and committed; every cell passes or has a filed task.
- [ ] dates.md and financial.md state the probed behavior and drop the UNVERIFIED line.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
