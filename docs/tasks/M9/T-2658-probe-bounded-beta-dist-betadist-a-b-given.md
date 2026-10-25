---
id: T-2658
title: "Probe bounded BETA.DIST/BETADIST (A, B given) at shape 1 near the x0 = 0.5 swap and the y0 = 0.9 closed-form cut"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
It is known, from oracle cells, whether Excel's BETA.DIST and BETADIST with bounds A and B given take the incomplete
beta's closed forms on bratio's oriented arguments at t = (x - A)/(B - A) and y = fl(1 - t). That covers a shape of 1,
both sides of the x0 = 0.5 swap and both sides of the y0 = 0.9 cut. The engine either matches those cells or a task
records the gap.

## Context
- T-2502 sends BETA.DIST's cdf through `incomplete_beta.rs` `closed_form_tails` (via `beta_reg_complement`,
  `beta.rs` around line 136), the bounded path included, with t = (x - A)/(B - A) and y = fl(1 - t).
- The evidence for the oriented closed forms (T-2464 w5-triangulation `closed_final.py`, 1,286 of 1,286) covers only
  4-argument BETA.DIST and 3-argument BETADIST, and docs/excel-semantics/statistics.md "Incomplete beta closed forms act
  on the oriented arguments" records no bounded cell. Excel may form t, its complement or the swap test differently
  when A and B are given, e.g. y = (B - x)/(B - A).
- Probe design (excel-probe skill): `BETA.DIST(x,a,1,TRUE,A,B)` and `BETA.DIST(x,1,b,TRUE,A,B)`, with BETADIST
  companions, at a few (A, B) where t is inexact (A = 0.1, B = 1; A = -3, B = 7). Take x so that t runs over
  0.5 ± a few ulp and fl(1 - t) over 0.9 ± a few ulp, and include the unbounded forms at the same t as controls.
  Score each cell against `closed_form_tails` at both y = fl(1 - t) and y = (B - x)/(B - A).

## Acceptance criteria
- [ ] A probes-numeric case with the cells above has Excel values (oracle fill).
- [ ] statistics.md records the bounded rule: which t and y Excel uses, and whether the swap and the cut follow t.
- [ ] If the engine misses any of these cells, the fix lands here (S) or a follow-up task is filed with the cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- bratio's general branches (T-2505).
- BETA.INV with bounds.
