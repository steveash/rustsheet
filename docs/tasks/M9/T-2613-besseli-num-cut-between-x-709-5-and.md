---
id: T-2613
title: "BESSELI #NUM! cut between x = 709.5 and 709.78 (T-2550 sweep)"
milestone: M9
lane: functions-engineering
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
campaign: semantics
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

## Visible-parity stage
Target (decision 0114): every visibly-off cell this task owns is within half a unit of the 15th significant digit
of Excel's value, or has Excel's type, error and text. Bit-level criteria still unmet then move to a follow-up
filed with `defer: phase-1b`.

Phase 1 triage (T-0090, eval of 2026-10-27 at `main` 7f6b373a, task ownership as of `main` e5bc60c4): this task owns the 1 visibly-off BESSELI cell. Fixing them lifts BESSELI to the Phase 1 rate (≥ 99.9% of attributed cells within 15 digits); no other open task owns enough of those cells.
