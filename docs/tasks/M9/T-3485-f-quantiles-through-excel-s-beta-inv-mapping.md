---
id: T-3485
title: "F quantiles through Excel's BETA.INV mapping: the p = 0.5 cells where Excel's y is 2+ ulps from any crossing (T-3398 remainder)"
milestone: M9
lane: functions-stat
status: todo
priority: P1
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
- Split: T-3225 and T-3233 list probes-numeric/m9-f-inv A31; T-1871 lists gen-m9/4ab2ba98d26c8a5e A110. They keep those
  cells.
- Visibly-off F quantile cells no other open task lists (gap triage 2026-10-31k, `make eval` on origin/main after
  T-3398, `make exit-owners MILESTONE=M9` rows `rate:F.INV`, `rate:F.INV.RT`, `rate:FINV`, plus the four unattributed
  gen-m9 cells whose only non-glue call is an F quantile). This task owns them. Examples, Excel vs ours:
  `F.INV(0.5,3,9)` 0.8516839503845974 vs ...963; `FINV(1e-09,1,1)` 4.052847345693515e17 vs 4.0528473456935e17;
  `F.INV.RT(0.5,10,10)` 0.9999999999999999 vs 0.9999999999999991.
  - gen-m9/320d51765af9b6b7.case.json (2): Calc!A15, A379
  - gen-m9/40425341c5034eed.case.json (1): Calc!A108
  - gen-m9/4ab2ba98d26c8a5e.case.json (1): Calc!A390
  - gen-m9/60b0b6e69ad35a45.case.json (2): Calc!A150, A197
  - gen-m9/7674c89c824aca43.case.json (1): Calc!A212
  - gen-m9/a63d1c7d6f96c56d.case.json (1): Calc!A320
  - gen-m9/c18dbf21f11093f3.case.json (1): Calc!A92
  - gen-m9/c6cfda6bd10cae93.case.json (1): Calc!A183
  - gen-m9/e6cf3b20a014e6a3.case.json (1): Calc!A135
  - probes-numeric/m9-f-inv-rt.case.json (4): Calc!A1, A7, A15, A25
  - probes-numeric/m9-legacy-finv.case.json (12): Calc!A1, A2, A3, A4, A9, A10, A37, A38, A39, A40, A47, A48
  - probes-numeric/t1049-finv.case.json (1): Calc!A7
  - probes-numeric/t1049-finvrt.case.json (1): Calc!A14

## Acceptance criteria
- [ ] F.INV, F.INV.RT and FINV meet the Phase 1 rate, or the Outcome names every cell still off with an owner.
- [ ] No cell becomes visibly off without decision 0083's allowance or an owner ruling.
- [ ] `make check` green; reviewers approve.

## Out of scope
- BETA.INV (T-3398).
