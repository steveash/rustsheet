---
id: T-3398
title: "BETA.INV and the F quantiles on the ported CDF (T-2511 remainder)"
milestone: M9
lane: functions-stat
status: todo
priority: P1
size: M
tier: opus
depends_on: [T-2511]
campaign: visible
---

## Goal
BETA.INV, BETAINV, F.INV, F.INV.RT and FINV reach the Phase 1 rate: their visibly-off attributed cells
(after T-2511: BETA.INV 66 of 519, BETAINV 12 of 119, F.INV 81 of 389, F.INV.RT 83 of 434, FINV 32 of 277) come within
half a unit of Excel's 15th significant digit, or each one still off is named with an owner.

## Context
- T-2511 (2026-10-31) moved the Student-t quantiles onto a rounded-y crossing and left these five untouched. They are
  the part of its scope that remains.
- BETA.INV already solves BETA.DIST's own cdf to the crossing (`beta.rs` `beta_quantile`, T-2119), starting from
  statrs' AS 109 search where that is reliable; a huge shape takes `inverse_beta::beta_quantile_crossing` (T-1538).
  The crossing is of `beta_reg_complement`/`beta_reg_upper`; check that this is the cdf BETA.DIST now returns
  after the bratio port (T-2505..T-2508) before changing the search.
- F.INV, F.INV.RT and FINV search the crossing of `fisher.rs::f_quantile_tail` (today's kernel, unoriented). T-2511
  measured the switch to F.DIST's ported tail (`f_tail`) on gen-m9, ironcalc-calc, probes and probes-numeric: 17
  visibly-off cells fixed (F.INV 5, F.INV.RT 5, FINV 3, 4 unattributed) and 7 newly visibly off, so it was not taken:
  - gen-m9/4ab2ba98d26c8a5e A110 `FINV(1E-10,6,8)` and ee2497a5e683c5d9 A100 `F.INV.RT(1E-10,6,8)`: Excel
    827.9101356457076, ported tail 827.9101356457081
  - gen-m9/4e55500bb8ad3cd8 A332 `F.INV(0.001,3,7)`: Excel 0.007599774174264007, ported 0.007599774174264013
  - probes-numeric/m9-f-inv A31 `F.INV(0.05,50,100)`, A32 and m9-f-inv-rt A32 `F.INV.RT(0.5,50,100)`
  - probes-numeric/t1049-finvrt A4 `F.INV.RT(0.25,1,1)`: Excel 5.828427124746192, ported 5.8284271247461845
  T-2977 had already seen the same direction (9 gated cells 2 to 8 ulp off).
- The Student-t result suggests a lead: Excel's T.INV takes y as the nearest crossing of the tail at a rounded y
  (T.TEST's tail), not of T.DIST's ported kernel, then back-transforms (statistics.md "Student-t quantile is
  SQRT(df*(1/y-1)) of the rounded beta quantile"). F.INV may likewise solve for the beta argument and map it,
  x = df2 y / (df1 (1 - y)), as `f_quantile_through_beta` does above a df of 1E5.
- Split: T-1871 (statrs saturation and panics) also lists gen-m9/4ab2ba98d26c8a5e A110, T-3225 lists
  probes-numeric/m9-f-inv A31, and T-3233 lists gen-m9/4e55500bb8ad3cd8 A332 and m9-f-inv A31; they keep those cells,
  which are named above only as the measurement's evidence.
- Rule: statistics.md "BETA.INV solves Excel's own CDF to the crossing (T-1745)" (155 of 188 rows).

## Cells added by the gap triage (2026-10-31, filed against T-2511 before it closed)
Visibly-off m9-beta-inv cells no open task owned (`make exit-owners MILESTONE=M9`, `rate:BETA.INV`, eval on a64f963e).
Each is the same point as a t1745-beta-inv-roundtrip row this task took over from T-2511, where Excel's x is a crossing of its
own cdf (BETA.DIST at x is p, or p lies between x and a neighbour) but our cdf there is a few ulps off Excel's and our
search does not land on our own crossing (A31: ours is 9.5E-15 off the 50-digit root, Excel 1E-15):
- probes-numeric/m9-beta-inv: Calc!A31, Calc!A102, Calc!A103 (t1745 rows 30, 99, 100)
The other 12 visibly-off m9-beta-inv cells at small shapes, where Excel's cdf is flat or its search stops off the
crossing, are T-3388's; the 5 `#NUM!` cells at p = 0.9999999999 are T-3351's.

## Cells added by the gap triage (2026-10-31i)
`make eval` on origin/main 0c2fbf706 (excel-graph), `scripts/exit_owners.py --all-off` and `--milestone M9`
(`rate:F.INV`, `rate:F.INV.RT`, `rate:FINV`, `rate:BETAINV`, and four unattributed gen-m9 cells whose only non-glue
call is an F quantile): 36 visibly-off cells of this task's five functions that no open task names cell by cell. The
Goal already covers them; these lines make the ownership checkable. Examples, Excel / ours:
`F.INV(0.5,1,1)` (m9-f-inv A4, t1049-finv A7) 0.9999999999999999 / 1.0000000000000009 (the root is exactly 1);
`F.INV.RT(0.5,10,10)` (m9-f-inv-rt A25, m9-legacy-finv A38, FINV A37) 0.9999999999999999 / 0.9999999999999991;
`FINV(0.5,5,50)` (m9-legacy-finv A39, F.INV.RT A40) 0.8821595104705021 / 0.8821595104704987;
`F.INV.RT(1e-10,5,2)` (m9-f-inv-rt A15) 9999999999.299988 / 9999999999.3;
`BETAINV(0.1,0.999,30,-0.0001)` (gen-m9/eced4bb6a0234c9e A291) 0.0033963068798590146 / 0.003396306879859034.
The gen-m9 cell `F.INV.RT(0.5,2,123456789.1229)` (6E-9 relative off the root in Excel) stays T-2555's.
- gen-m9/320d51765af9b6b7.case.json (2): Calc!A15, A379
- gen-m9/40425341c5034eed.case.json (1): Calc!A108
- gen-m9/4ab2ba98d26c8a5e.case.json (1): Calc!A390
- gen-m9/589f42e67f4b4b8d.case.json (1): Calc!A365
- gen-m9/60b0b6e69ad35a45.case.json (2): Calc!A150, A197
- gen-m9/7674c89c824aca43.case.json (1): Calc!A212
- gen-m9/a63d1c7d6f96c56d.case.json (1): Calc!A320
- gen-m9/c18dbf21f11093f3.case.json (1): Calc!A92
- gen-m9/c6cfda6bd10cae93.case.json (1): Calc!A183
- gen-m9/e6cf3b20a014e6a3.case.json (1): Calc!A135
- gen-m9/eced4bb6a0234c9e.case.json (1): Calc!A291
- probes-numeric/m9-f-inv.case.json (5): Calc!A1, A4, A7, A21, A25
- probes-numeric/m9-f-inv-rt.case.json (4): Calc!A1, A7, A15, A25
- probes-numeric/m9-legacy-finv.case.json (12): Calc!A1, A2, A3, A4, A9, A10, A37, A38, A39, A40, A47, A48
- probes-numeric/t1049-finv.case.json (1): Calc!A7
- probes-numeric/t1049-finvrt.case.json (1): Calc!A14

## Acceptance criteria
- [ ] Each of the five functions meets the Phase 1 rate, or the Outcome names every cell still visibly off with an
      owner.
- [ ] No gated line is added without decision 0083's allowance.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The Student-t quantiles (T-2511, T-3061, T-3399).
