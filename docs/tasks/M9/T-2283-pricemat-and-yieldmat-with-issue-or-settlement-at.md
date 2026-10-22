---
id: T-2283
title: "PRICEMAT and YIELDMAT with issue or settlement at serial 0-60: probe the shared YEARFRAC day counts"
milestone: M9
lane: functions-financial
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
PRICEMAT and YIELDMAT give Excel's value when the issue or settlement date is a serial from 0 to 60 (1900-01-00 to the fictitious 1900-02-29), on every basis, backed by oracle evidence.

## Context
Found in review of T-2237. That task made `Model::year_fraction_parts` (functions/date_and_time/yearfrac.rs) take serial 0 on
every basis, read basis 0 on Excel's own calendar (`YEARFRAC(60,59,0)` = -1/360), and read basis 1's year from 1900
rather than 1899 for serials 1-59 (dates.md "YEARFRAC accepts serial 0"). PRICEMAT and YIELDMAT read the same parts
through `get_mat_factors` (functions/financial/mod.rs), so their day counts moved with it. Only one cell covers them:
probes/discount-serial-zero A10:E10 (`PRICEMAT(366,400,0,0.05,0.06,b)`, issue 0), which passes. Probe (excel-probe
skill, excel-graph; about 40 cells) `PRICEMAT(s,m,i,0.05,0.06,b)` and `YIELDMAT(s,m,i,0.05,99,b)` for b in 0..4 with
issue i in {0, 1, 59, 60} and settlement s in {59, 60, 61, 366}, maturity 400. Every cell is expected to pass already.
Record any miss in financial.md and fix it in `get_mat_factors`.

## Acceptance criteria
- [ ] A new probes case (tag M9), oracle-filled, covers the grid. Every cell passes, or each miss is fixed or has a filed task.
- [ ] financial.md "PRICEMAT and YIELDMAT count the settlement-to-maturity days" cites the new case.
- [ ] `make check` green; reviewers approve.

## Out of scope
- YEARFRAC and the other discount securities (probed by T-2237).

## Outcome
Added probes/matdisc-serial-0-60 (tag M9, 32 rows x bases 0-4 = 160 cells: PRICEMAT and YIELDMAT, issue in {0,1,59,60}, settlement in {59,60,61,366}, maturity 400), oracle-filled on excel-graph. Every cell passes already, so `get_mat_factors` needed no change. financial.md cites the case. Eval delta: +160 passing cells in probes, the only baseline change is the new case's clean line, recorded by `make eval-baseline`.
The grid includes settlement-before-issue cells (e.g. settlement 59, issue 60); they match Excel too, so the grid needed no code change.
