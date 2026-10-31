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

## Acceptance criteria
- [ ] Each of the five functions meets the Phase 1 rate, or the Outcome names every cell still visibly off with an
      owner.
- [ ] No gated line is added without decision 0083's allowance.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The Student-t quantiles (T-2511, T-3061, T-3399).
