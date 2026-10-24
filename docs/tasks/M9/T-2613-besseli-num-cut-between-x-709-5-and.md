---
id: T-2613
title: "BESSELI #NUM! cut between x = 709.5 and 709.78 (T-2550 sweep)"
milestone: M9
lane: functions-engineering
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
`BESSELI(x,n)` is `#NUM!` at the same x as Excel (the cut between 709.5 and 709.78).

## Context
T-2550 sweep (probes-numeric/m9-besselk-underflow): `BESSELI(709.5,0)` is 2.02976276636543E+306 and `BESSELI(709.78,0)` is
`#NUM!` in Excel, with `#NUM!` at every probed x to 713; ours returns 2.685109662266049E+306 at 709.78. T-2421 covers x from
710 on. Locate the cut by a finer oracle probe (709.5 to 709.78), then match it. docs/excel-semantics/engineering.md "BESSELK
near the bottom of the double range, BESSELI overflow point (T-2550)".

## Acceptance criteria
- [ ] The cut is probed to the last digit of x and `BESSELI(709.78,0)` is `#NUM!` in ours; the probes-numeric cell passes.
- [ ] `make check` green; reviewers approve.

## Out of scope
- BESSELK kernel (T-2549).
