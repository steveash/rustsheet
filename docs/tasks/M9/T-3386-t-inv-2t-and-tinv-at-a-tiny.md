---
id: T-3386
title: "T.INV.2T and TINV at a tiny p: Excel is over 1e-4 off the math from p 1.7E-110 down (R3 candidate, needs an owner ruling)"
milestone: M9
lane: functions-stat
status: todo
priority: P1
size: S
tier: opus
depends_on: [T-3358]
---

## Goal
Decision 0188 R3 decides, with an owner ruling, whether T.INV.2T and TINV at a tiny p follow Excel or the math, and
the region is evidenced as committed points; until then Excel's value stays the target (directive 1).

## Context
- T-3349's fresh sweep (probes-numeric/t3349-t-inv-2t-sweep, excel-graph 2026-10-31, mpmath 60 digits): 42 of 300
  points have Excel more than 1e-4 relative off the math, every one at p from 1.38E-300 to 1.73E-110 and df from
  2490 to 1.3E8 (A211-A300, extreme and log-uniform strata). Example `T.INV.2T(6.68863396408457E-199,2490)`: Excel
  100.83724584548176, math 33.047211122914894, engine 33.047211122914874. The engine returns the math there, so these
  cells are visibly off Excel but inside Gate A's envelope.
- R3's initial case 1 lists T.INV (left tail), F.INV, BETA.INV and CHISQ.INV.RT below p 1e-150, not T.INV.2T or
  TINV, and these points reach p 1.7E-110, above that bound. A new R3 case or a wider region needs a decision record
  with an owner ruling (decision 0188 R3); no agent adds one alone.
- docs/excel-semantics/statistics.md "Two-tailed Student-t quantile with p near 1 is accurate in Excel (T-3349)".
- A second region for the same ruling (gap triage 2026-10-31): at one degree of freedom with p near 1, Excel's
  `1/TAN(PI()*p/2)` form (statistics.md, T-2858) is far off the math, where the engine is on it today:
  probes/t3349-t-inv-2t-near-one A10 `T.INV.2T(0.999999999999999,1)` Excel 1.6155696572206502E-15, math and engine
  1.5695408241038840E-15 (Excel 2.9E-2 relative off); A9 `T.INV.2T(0.999999999999,1)` Excel 1.570804792662417E-12,
  engine 1.5707615780781872E-12 (2.75E-5 off, under the 1E-4 bar). T-2858 (sonnet, in flight) reproduces Excel's form,
  which directive 1 makes the target until a ruling; A10 is above R3's G = 1E-4, so record it in the same decision
  record (follow the math or match Excel, with the p bound where Excel's error crosses 1E-4 at df 1).

## Acceptance criteria
- [ ] A decision record with the owner's ruling (follow the math, or match Excel) and the region's bounds, from
      committed evidence points (T-3358's tables or a probe on both sides of the bound).
- [ ] If the ruling is the math: a `MATH-OVER-EXCEL` entry and the R3 marks (SPEC-0029); if Excel: a task that
      reproduces Excel's form there.
- [ ] `make check` green; reviewers approve.

## Out of scope
- T.INV's own region (R3 case 1, T-3358).
