---
id: T-3519
title: "GAMMA.INV at the median (p = 0.5) below shape 0.5: 34 visibly-off cells of the T-3509 held-out probe"
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
`GAMMA.INV(0.5, a, beta)` (the median) at a shape below 0.5 returns Excel's value within half a unit of the 15th
significant digit, or the Outcome lists the forms tried with their counts (decision 0056).

## Context
- Found by the T-3509 held-out probe (docs/research/t3509/README.md, score-main-744815d06.out): of the 57 GAMMA.INV
  cells at shape below 0.5 with p in the upper half, 34 are visibly off, and all 34 are p = 0.5 exactly (the first
  draw wrote p = 0.5 where 1 - t rounds to 1). Shapes 0.0154 to 0.48; the median there is tiny (1.7E-20 at shape
  0.0154, 0.063 at 0.284), so this is the lower-tail solver at P = 1/2, not the upper tail.
- Excel is 21 ulp (median) from the 50-digit math and at most 1.5E-14 relative; ours 19 ulp (median), at most 2.1E-14;
  ours is closer to the math at 20 of 34. Excel is not near correctly rounded, so this is `visible`, not `accuracy`.
  All 34 are inside Gate A (within 1E-9 of the math).
- Examples: `GAMMA.INV(0.5,0.0513890707246318,1)` Excel 8.116524370070504E-07, ours 8.116524370070466E-07, math
  8.116524370070528E-07; `GAMMA.INV(0.5,0.283756718398782,1)` Excel 0.06296108930404444, ours 0.06296108930404459.
- Code: `incomplete_gamma` quantile paths (`fn_gamma_inv` in `statistical/gamma.rs`; `statistical/incomplete_gamma/mod.rs`, T-2561 `excel_tiny_quantile` and the search).
  Related: T-3311 and T-3403 (deep lower tail above shape 1), T-3060 and T-3242 (the cdf below shape 1).
- Cells (excel-graph, 2026-10-31):
  - probes-numeric/t3509-blind-gamma-inv.case.json (34): S!A13, A17, A40, A43, A55, A60, A66, A82, A85, A101, A104, A120, A123, A132, A166, A170, A190, A191, A215, A217, A255, A261, A278, A293, A302, A341, A354, A380, A387, A434, A439, A467, A482, A496

## Visible-parity stage
Decision 0083: the cells this task owns are visibly off, so it is in the campaign. Its target is that every visibly-off
cell it owns is within half a unit of the 15th significant digit of Excel's value; bit-level misses left then go to a
`defer: phase-1b` follow-up. The probe cells are evidence for the region, not a fitting set: the fit is chosen on its
own probe (the `excel-probe` skill: `GAMMA.INV(0.5,a,1)` and `GAMMA.DIST` at Excel's quantile over shape 1E-2 to 0.5).

## Acceptance criteria
- [ ] The 34 listed cells are within half a unit of Excel's 15th digit, or the Outcome lists the forms tried with
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
- Other p at shape below 0.5 (two lower-tail cells in the probe report, not owned here); shapes 0.5 and up.
