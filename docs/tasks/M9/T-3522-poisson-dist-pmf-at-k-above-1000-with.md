---
id: T-3522
title: "POISSON.DIST pmf at k above 1000 with the mean near k: 12 visibly-off cells of the T-3509 held-out probe"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: []
campaign: visible
---

## Goal
`POISSON.DIST(k,mean,FALSE)` at k above 1000 with the mean within about 0.5 to 1.4 times k + 1 matches Excel within
half a unit of the 15th significant digit, or the Outcome lists the forms tried with their counts (decision 0056).

## Context
- Found by the T-3509 held-out probe (docs/research/t3509/README.md, score-main-744815d06.out): 12 of the 23 probe
  cells at k + 1 of 1000 and up with mean/(k + 1) from 0.5 to 1.4 are visibly off (k 1234 to 56218, mean/k 0.75 to
  1.18, pmf 3.4E-3 down to 5.9E-241); 2 more at k 20 to 1000 in the same band (B142, B256) are listed below too.
- Excel vs the 50-digit math: 228 ulp (median), at most 2.5E-13 relative; ours 101 ulp (median), at most 2.1E-13;
  ours is closer to the math at 9 of 12. Excel is not near correctly rounded, so this is `visible`. All are inside
  Gate A.
- Examples: `POISSON.DIST(9311,9720.45387932461,FALSE)` Excel 6.568428492688257E-07, ours 6.568428492688176E-07,
  math 6.568428492688279E-07; `POISSON.DIST(21699,17183.5049329935,FALSE)` Excel 5.911582380978422E-241, ours
  5.911582380975733E-241.
- Coverage checked first: T-3295 is the pmf at k 30 and up with the mean well off k, waiting on T-3288's sweep (k 30 to
  1000); T-3247 (`defer: phase-1b`) keeps the pmf's bit-level residue. Neither reaches k above 1000 near the mean,
  where the relative error is 10 to 100 times larger. Both use T-2055's saddle point (`statistical/poisson.rs`,
  `loader.rs` `excel_bd0`, stirlerr); the error growing with k points at how Excel stores stirlerr(k) or
  bd0(k, mean) for a large k. Coordinate with T-3295 before changing `excel_bd0`, which BETA.DIST's density shares.
- Cells (excel-graph, 2026-10-31):
  - probes-numeric/t3509-blind-poisson-dist.case.json (14): S!B2, B47, B108, B111, B142, B187, B256, B290, B355, B364, B389, B399, B406, B418

## Visible-parity stage
Decision 0083: the cells this task owns are visibly off, so it is in the campaign. Its target is that every visibly-off
cell it owns is within half a unit of the 15th significant digit of Excel's value; bit-level misses left then go to
T-3247 (`defer: phase-1b`). The probe cells are evidence for the region, not a fitting set: probe it first (the
`excel-probe` skill: k 1E3 to 1E5, mean 0.5k to 1.5k).

## Acceptance criteria
- [ ] The 14 listed cells are within half a unit of Excel's 15th digit, or the Outcome lists the forms tried with
      their counts on a hold-out split.
- [ ] Decision 0188 R4 guards: the candidate is chosen on a hold-out split of its own evidence (R4.1, train and test
      counts side by side, accepted only if it beats the correctly rounded candidate on test); it is never fitted to
      the T-3509 probe, which is scored before and after and both per-function tables go in the Outcome
      (docs/research/t3509/README.md); a fresh log-uniform sweep is filled after the model is frozen (R4.2); every fitted
      constant, cut-point or table carries `FITTED(T-XXXX, n cells, range)` and falls back to accurate math outside
      that range (R4.3); nothing is fitted below the 4-ulp oracle floor (R4.4); the Gate A envelope result is reported
      (R4.5).
- [ ] No gated cell regresses; no new known-failure line in eval/baseline; `make check` green; reviewers approve.
- [ ] `make check` green; reviewers approve.

## Out of scope
- k up to 1000 with the mean well off k (T-3295); the pmf's last bits (T-3247); the cdf (T-3269, T-3286).
