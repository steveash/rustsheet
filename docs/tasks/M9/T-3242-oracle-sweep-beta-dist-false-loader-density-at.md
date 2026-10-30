---
id: T-3242
title: "Oracle sweep: BETA.DIST(.,FALSE) Loader density at large shapes (a + b from 128 to 8192, partner 1.5 to a), the fitting and hold-out points the large-exponent residue needs"
milestone: M9
lane: research
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Oracle-filled probe cases give T-3243 (BETA.DIST at a = 1025, b = 1.5) the Excel points it needs to fit the Loader
density's large-shape behaviour, plus a hold-out no fitting choice uses, so the fit is not made blind to one shape
(decision 0056).

## Context
- Gap triage 2026-10-30c (`make eval` on main 40ca52179, excel-graph). The only cached evidence above a + b = 300 is
  the a = 1025, b = 1.5 column of T-2464's log1p probe (5,264 cells, one shape, x in (0.625, 1); T-3243 owns it).
  There, the error between Excel and the engine grows with the size of the log of the density. Table: |ln f| bucket,
  cells, sd of (ours - Excel) in units of 2^-52 relative:
  0-50 273 13.6; 50-100 463 28.3; 100-150 488 51.6; 150-200 499 57.3; 200-250 467 58.4; 250-300 503 110;
  300-350 478 112; 350-400 452 109; 400-450 352 37.8; 450-500 205 17.8.
  Against a 200-bit mpmath value (300-cell sample), Excel is a median 92 ulp (p90 323) from the correctly rounded
  density and the engine 69 (p90 276). So a more accurate kernel does not reach Excel's digits (stage `visible`,
  decision 0114). EXP of the double nearest the exact ln f is not Excel's value either (median 91 ulp).
- One shape cannot tell what depends on n = a + b - 2, on k = a - 1 and on the bd0 arguments. It also cannot show
  where the error collapses above |ln f| = 400. The model of record (docs/excel-semantics/statistics.md, "Beta density
  for min(a,b) > 1 is Loader's saddle point") has an n trend that is only measured to n = 105 (-10 to -14 units at
  n 70-105).
- The probes-numeric cases of T-2513's round 3 stop at n = 256. Their two-large-shape residue is T-3244's.

## Sweep design (adjust with reasons in the Outcome)
- Shapes: a in {128.5, 256.5, 512.5, 1025, 2048.5, 4096.5, 8192.5} crossed with b in {1.5, 2.5, 8.5, 40.5} and the
  symmetric b = a. At a = 1025 use b = 2.5 and up only, because b = 1.5 is the cached shape.
- x: for each shape, 8 runs of 32 consecutive doubles (exact-input literals `(m*2^-p+r*2^-q)`, as in T-2513's
  generators). Put the runs where ln f is about -25, -50, -100, -200, -300, -400, -500 and -650 (skip the levels that
  underflow), and add one run at the mode. That is about 10,000 cells.
- Hold-out: a second case with the same shapes plus a in {384.5, 1536.5}, b in {4.5, 20.5}, and runs at other levels,
  from its own seed. It is never used for a choice.
- Beside each BETA.DIST row, put `F.DIST` at the mapped point x' = b·x/(a·(1 - x)) with d1 = 2a and d2 = 2b, but
  only where 2a and 2b are integers. That is one more column for T-3173, whose power-term error also tracks the size
  of the exponent ("error tracking ulp(P)" in its Context).
- Generator and run notes go in docs/research/t3242/. Fill through `cd tools && uv run rustsheet-oracle fill`, then
  canary-verify (excel-probe skill).

## Acceptance criteria
- [ ] Two probe cases (fit and hold-out) with excel-graph sidecars land under eval/suites/probes-numeric/cases/.
      The generator that rebuilds them byte for byte is in docs/research/t3242/.
- [ ] `make eval` loads both. Their known failures are recorded with `make eval-baseline` and no existing line
      regresses.
- [ ] T-3243's Cells section names both new case stems, so that no visibly-off cell they add is unowned
      (`make unowned-off ALL=1 FUNCTION=BETA.DIST`). T-3173's Cells section names the F.DIST column.
- [ ] statistics.md's Loader entry gets one line on what the sweep shows, without fitting: the error sd against
      |ln f| and n per shape, and where it collapses.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fitting or changing the engine (T-3243, T-3244).
- min(a,b) <= 1 (T-2503, T-2821, T-3035).
