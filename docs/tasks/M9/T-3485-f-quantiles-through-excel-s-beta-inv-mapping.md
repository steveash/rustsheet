---
id: T-3485
title: "F quantiles through Excel's BETA.INV mapping: the p = 0.5 cells where Excel's y is 2+ ulps from any crossing (T-3398 remainder)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-3398]
campaign: visible
---

## Goal
F.INV, F.INV.RT and FINV reach the Phase 1 rate (after T-3398: F.INV 81 of 389 attributed cells visibly off,
F.INV.RT 83 of 434, FINV 32 of 277), with no cell newly visibly off.

## Context
- T-3398 (2026-10-31) measured F through Excel's BETA.INV mapping, x = (df2/df1) y/(1 - y) with y Excel's beta root
  (`fisher::f_quantile_through_beta`, today used only from a df of 1E5), at every df. Scored over gen-m9, ironcalc-calc,
  probes and probes-numeric against main (visibly-off cells fixed / newly visibly off, F functions only):
  - nearest crossing of BETA.DIST's ported kernel (`inverse_beta::beta_lower_crossing`), keeping the deep-lower-tail
    rule (`F_QUANTILE_REFINE_P_MIN`): about 29 / 7 (the best);
  - the even-significand end below a df of 1E5, nearest above: about 24 / 10;
  - the even end everywhere: 24 / 30 (the huge-df cells of t1538 break);
  - the older Cephes-based search tails, nearest end: 22 / 21.
  The fixed cells include many at p = 0.5, e.g. `F.INV(0.5,1,3)` = 0.5850602740518204 (gen-m9/589f42e67f4b4b8d), and
  `F.INV(0.5,10,10)` = 0.9999999999999999 (probes-numeric/m9-f-inv), which is the y/(1 - y) mapping's signature. So
  the mapping is Excel's structure.
- The 7 new cells are two inputs at p = 0.5 where no variant gives Excel's y. It is 2 or more ulps from every crossing
  tried, which looks like the stop of Excel's search:
  - `F.INV.RT(0.5,2,5)` = `FINV(0.5,2,5)` = 0.7987697769322353 (probes-numeric/m9-f-inv-rt A11; m9-legacy-finv A35,
    A36, A69, A72); the mapping gives 0.7987697769322356 to ...359;
  - `F.INV(0.5,50,100)` = `F.INV.RT(0.5,50,100)` = 0.9933156112563186 (m9-f-inv A32, m9-f-inv-rt A32); the mapping gives
    ...177 to ...181.
  The order of the mapping's arithmetic (four algebraically equal forms) does not decide them either.
- BETA.INV cells T-3398 moved within 2 ulp (decision 0083's allowance; bit-level, not this task's visible target):
  ironcalc-calc STATISTICAL/BETADIST_BETAINV.xlsx Sheet1!H2 (2 ulp), H25 (1), H63 (2), H64 (2), H67 (1). The same
  even-end choice applied to F is measured above.
- Split: T-3225 and T-3233 list probes-numeric/m9-f-inv A31; T-1871 lists gen-m9/4ab2ba98d26c8a5e A110. They keep those
  cells.

## Acceptance criteria
- [ ] F.INV, F.INV.RT and FINV meet the Phase 1 rate, or the Outcome names every cell still off with an owner.
- [ ] No cell becomes visibly off without decision 0083's allowance or an owner ruling.
- [ ] `make check` green; reviewers approve.

## Out of scope
- BETA.INV (T-3398).
