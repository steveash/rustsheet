---
id: T-3521
title: "CHISQ.DIST.RT at shape 20 and above with x/df 1.4 to 10: 8 visibly-off cells of the T-3509 held-out probe"
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
CHISQ.DIST.RT at a shape (df/2) of 20 and above with x/df from 1.4 to 10 matches Excel within half a unit of the 15th
significant digit, or the Outcome lists the forms tried with their counts (decision 0056).

## Context
- Found by the T-3509 held-out probe (docs/research/t3509/README.md, score-main-744815d06.out): 8 of the 28 probe
  cells at df 40 to 2000 with x/df 1.4 to 10 are visibly off (df 47 to 1442, Q from 9.4E-7 down to 2.1E-225); the
  bucket's other 20 are within 4 ulp. Below and above that band (x/df under 1.4, or 10 and up) the shape-20+ cells
  are all within 15 digits.
- Excel vs the 50-digit math: 82 ulp (median), at most 5.2E-14 relative; ours 21 ulp (median), at most 6.2E-14;
  ours is closer to the math at 5 of 8. Excel is not near correctly rounded, so this is `visible`. All 8 are inside
  Gate A.
- Examples: `CHISQ.DIST.RT(361.118428794895,155)` Excel 1.7137399791597373E-18, ours 1.7137399791597622E-18, math
  1.7137399791597533E-18; `CHISQ.DIST.RT(3903.00797150386,1442)` Excel 2.1044439983620602E-225, ours
  2.1044439983618206E-225.
- Not covered: T-3310 is the far right tail at df 3 to 22; T-3286 is near the mode at shapes 21 to 1E8 (the engine's
  `NEAR_MODE` band); T-2638 set Excel's prefactor from shape 20 away from the mode. Check first whether these cells
  are on T-2638's prefactor path or on the near-mode one, and name T-3286 here if they share its fix.
- Code: `fn_chisq_dist_rt` in `statistical/chisq.rs`, `statistical/incomplete_gamma/` (the upper tail and
  `gamma_prefactor`).
- Cells (excel-graph, 2026-10-31):
  - probes-numeric/t3509-blind-chisq-dist-rt.case.json (8): S!A28, A84, A135, A137, A138, A266, A405, A432

## Visible-parity stage
Decision 0083: the cells this task owns are visibly off, so it is in the campaign. Its target is that every visibly-off
cell it owns is within half a unit of the 15th significant digit of Excel's value; bit-level misses left then go to a
`defer: phase-1b` follow-up. The probe cells are evidence for the region, not a fitting set: probe the band first
(the `excel-probe` skill: CHISQ.DIST.RT and GAMMA.DIST at shapes 20 to 1E3, x/a 1.2 to 12).

## Acceptance criteria
- [ ] The 8 listed cells are within half a unit of Excel's 15th digit, or the Outcome lists the forms tried with
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
- The far right tail at df up to 22 (T-3310); near the mode (T-3286); POISSON's deep upper tail (T-3269).
