---
id: T-3021
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
Every visibly-off cell of a function that is 1 to 5 cells short of the Phase 1 rate has an owner task that names
it, with a root cause, so each of those functions has a path to the rate.

## Context
T-0098's assessment: a full `rustsheet-eval run --desktop-owned` on main 8d2f9394 (2026-10-28), Phase 1 418 of 506,
88 functions below the rate, 39 of them 1 to 5 cells short. "Short" is decision 0118's rate: a function's
`rate_visible` cells less floor(`rate_cells` / 1000) (report.json `v15_checkpoint.functions`). Of those functions'
visibly-off attributed cells (`attributed_off` entries outside 15 digits), these are named by no open task, which
owns a cell when it names its case and address:
- CHITEST: probes-numeric/m9-legacy-chitest `Calc!A23` (T-2094 names its A28 only).
- GAMMADIST: probes/t2435-dist-extremes `S!B102` (T-2811 names A101 and A102).
- GAMMAINV: gen-m9/0a614cfe974b59c3 `Calc!A327` (eleven tasks name other cells of that case; none names A327).
These are named only at the case level, without the cell, so confirm the task means them and name the cell:
- GAMMAINV: probes-numeric/t1048-gammainv `Calc!A7` (T-0972, "t1048-gammainv.case.json 4").
- LOGNORM.DIST: probes-numeric/m9-normpdf-lognorm-dist `Calc!A29`, `Calc!A41`, `Calc!A73` (T-0975).
- NORM.INV: ironcalc-calc/STATISTICAL/NORM_DIST.xlsx `NORM.DIST NORM.INV!F70` (T-0925, "NORM_DIST.xlsx *.INV cells"; its F8, F9
  and F39 are off too but within 15 digits, so they do not count toward the rate).
Read each with `make eval-show CASE=<case>`, find the root cause (probe first with the excel-probe skill when Excel's
rule is not recorded), and fold it into the task that has the same root cause, or file one. Every other visibly-off
cell of these 39 functions is named by an open task; the FORECAST.ETS family's are T-2425's (blocked on desktop
evidence).

## Acceptance criteria
- [ ] Each cell above is named by an open owner task, or is shown fixed on main.
- [ ] Owners that lift a function to the rate are P1.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fixing the cells (the owner tasks do), and functions more than 5 cells short.
