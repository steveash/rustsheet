---
id: T-3520
title: "GAMMA.INV deep lower tail (p below 1E-10) at shape 0.5 to 20 outside T-3311's model: 16 visibly-off cells of the T-3509 held-out probe"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-3311]
campaign: visible
---

## Goal
GAMMA.INV's deep lower tail (p below 1E-10) at shapes 0.5 to 20 returns Excel's value within half a unit of the 15th
significant digit outside the region T-3311's model covers, or the Outcome lists the forms tried with their counts
(decision 0056).

## Context
- Found by the T-3509 held-out probe (docs/research/t3509/README.md, score-main-744815d06.out): 16 visibly-off cells
  at shape 0.5 to 20 with p from 1E-295 to 7E-25, on main 744815d06 (before T-3311 lands).
- Coverage checked first: T-3311 (on its branch, not yet merged) applies Newton steps from the leading-factor root
  only at shape 1.5 to 7.5 with the cdf at the root within 1E-6 of p; T-3403 (filed on T-3311's branch) owns which
  iterate Excel returns inside that region; T-3294 owns shape 20 and above. So: first score this probe on T-3311's
  HEAD (it is a fresh R4.2 sweep for T-3311 too); cells T-3311 fixes are its own; cells left inside its region move to
  T-3403 (name this task there); this task keeps the rest, which on main are 7 of the 16 cells: shape 0.67 to 1.03
  (A206, A242, A284) and 14.3 to 17.2 (A93, A289, A425, A431), plus any of the others T-3311 does not move.
- Excel vs the 50-digit math: 6 to 63 ulp (median by bucket), at most 1.8E-14 relative; ours 1 to 110 ulp. Excel is
  not near correctly rounded, so this is `visible`. All 16 are inside Gate A.
- Examples: `GAMMA.INV(1.58505193907599E-209,1.60237146365436,1)` Excel 6.1732895874859E-131, ours
  6.173289587486183E-131, math 6.17328958748601E-131; `GAMMA.INV(6.78919535451104E-179,17.1898391112751,1)`
  Excel 3.1297419132650086E-10, ours 3.129741913265001E-10.
- Code: `statistical/incomplete_gamma/mod.rs` (`excel_tiny_quantile`, T-2561's root check), `fn_gamma_inv`.
- Cells (excel-graph, 2026-10-31):
  - probes-numeric/t3509-blind-gamma-inv.case.json (16): S!A79, A93, A115, A133, A164, A199, A206, A242, A284, A289, A425, A431, A433, A443, A473, A489

## Visible-parity stage
Decision 0083: the cells this task owns are visibly off, so it is in the campaign. Its target is that every visibly-off
cell it owns is within half a unit of the 15th significant digit of Excel's value; bit-level misses left then go to a
`defer: phase-1b` follow-up. The probe cells are evidence for the region, not a fitting set.

## Acceptance criteria
- [ ] The probe is scored on T-3311's HEAD (or main after it lands) and the Outcome lists which of the 16 cells each of
      T-3311, T-3403 and this task owns.
- [ ] This task's cells are within half a unit of Excel's 15th digit, or the Outcome lists the forms tried with their
      counts on a hold-out split.
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
- Shape 1.5 to 7.5 inside T-3311's region (T-3311, T-3403); shape 20 and above (T-3294); the median and upper half
  (T-3519).
