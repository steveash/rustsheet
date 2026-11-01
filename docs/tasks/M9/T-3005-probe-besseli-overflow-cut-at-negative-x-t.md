---
id: T-3005
title: "Probe BESSELI overflow cut at negative x (T-2613 follow-up)"
milestone: M9
lane: functions-engineering
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`BESSELI(x,n)` for negative x near -709.7 is `#NUM!` or finite exactly as in Excel.

## Context
T-2613 probed the BESSELI cut only for positive x (finite at 709.7, `#NUM!` at 709.70001) and applies it to |x| (`BESSELI_OVERFLOW_X` in bessel_i.rs), unprobed for x < 0. Probe `BESSELI(-709.7,0..5)`, `(-709.70001,0..5)` and `(-709.75,1)`, then match them and update engineering.md "BESSELI overflow point (T-2613)".

## Acceptance criteria
- [ ] The negative-x cells above are probed and ours matches them.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probed `BESSELI(-709.7,0..5)`, `(-709.70001,0..5)` and `(-709.75,1)` (probes/besseli-overflow-cut-negative, excel-graph): the cut is symmetric in x and ours already matched all 13 cells, so no engine change. Recorded in engineering.md "BESSELI overflow point (T-2613)". Eval: +13 passing probe cells (baseline updated), no regressions.
