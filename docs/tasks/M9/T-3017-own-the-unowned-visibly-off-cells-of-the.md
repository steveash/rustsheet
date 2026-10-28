---
id: T-3017
title: "Own the unowned visibly-off cells of the functions 1 to 5 cells short of the Phase 1 rate (cheapest-first triage)"
milestone: M9
lane: research
status: todo
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
Every visibly-off cell of a function that is 1 to 5 cells short of the Phase 1 rate has an owner task with a
root cause, cheapest function first, so each of those functions has a path to the rate.

## Context
T-0098's assessment (full `rustsheet-eval run --desktop-owned` on main, 2026-10-28: Phase 1 412 of 506; 94 below
the rate, 42 of them 5 or fewer cells short) found these cells named by no open task (a task owns a cell when it
names its case and its address). Read each with `make eval-show CASE=<case>`, find the root cause (probe first with
the excel-probe skill when Excel's rule is not recorded), and fold it into an existing task when one already covers
the same root cause, or file one. Give the owner P1 when it lifts a function to the rate.
  - CHITEST (needs 2): `probes-numeric/m9-legacy-chitest.case.json` A23
  - SEARCH (needs 2): `corpus-bench/53062/1_53062_input.xlsx` Q7, Q8
  - ARRAYTOTEXT (needs 3): `probes/arraytotext-error-vs-format.case.json` A14, A2, A3
  - DEVSQ (needs 3): `probes/stat-order-floor-batch.case.json` A294
  - WEIBULL (needs 3): `gen-m9/11eb3ab4f4f8104c.case.json` A320; `gen-m9/5cebac1f02bf30f2.case.json` A336; `probes-numeric/t1048-weibull.case.json` A8
  - GAMMADIST (needs 4): `probes/t2435-dist-extremes.case.json` B102
  - GAMMAINV (needs 4): `gen-m9/0a614cfe974b59c3.case.json` A327; `probes-numeric/t1048-gammainv.case.json` A7
  - LOGNORM.DIST (needs 4): `probes-numeric/m9-normpdf-lognorm-dist.case.json` A29, A41, A73
  - NORM.INV (needs 5): `ironcalc-calc/STATISTICAL/NORM_DIST.xlsx` F8, F9, F39, F70
  - NUMBERVALUE (needs 5): `probes/numbervalue-default-group.case.json` A1, A2, A3, A11; `probes/numbervalue-separator-edges.case.json` A8
The FORECAST.ETS family's cells are T-2425's (blocked on desktop evidence) and stay out of this list.

## Acceptance criteria
- [ ] Each cell above is named by an open owner task (an existing one or a new one) or is shown fixed on main.
- [ ] Owners that lift a function to the rate are P1.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fixing the cells (the owner tasks do), and functions more than 5 cells short.
