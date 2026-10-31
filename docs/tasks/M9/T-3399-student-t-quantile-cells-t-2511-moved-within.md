---
id: T-3399
title: "Student-t quantile cells T-2511 moved within 4 ulp, and Excel's y one or two ulps from the nearest"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
The Student-t quantile cells that T-2511 left a few ulp from Excel return Excel's bits: the gated cells it moved by
1 or 2 ulp (decision 0083's standing allowance, condition 4), and the cells where Excel's y is one or two ulps from
the nearest crossing.

## Context
- T-2511 (2026-10-31) computes T.INV, T.INV.2T, TINV and CONFIDENCE.T from df 2, away from the centre, as
  t = fl(sqrt(fl(df * fl(fl(1/y) - 1)))) with y the double nearest the crossing of `student_t::student_t_tail_at`
  (`t_dist::t_rounded_y_quantile`). These known failures were within 15 digits before and still are, but moved:
  - gen-m9/46ac3ef5e61583ca Calc!A231 `TINV(0.05,5)`: Excel 2.570581835636315, ours 2.5705818356363155, 1 ulp
  - gen-m9/5cebac1f02bf30f2 Calc!A60 `TINV(LN(Data!C8),5)`: Excel 4.776160567669916, ours 4.776160567669917, 1 ulp
  - gen-m9/6e18550e9fc9c1b9 Calc!A3 `TINV(0.05,7)` and A4 `T.INV.2T(0.05,7)`: Excel 2.364624251592785, ours
    2.3646242515927853, 1 ulp
  - gen-m9/c2684066d8df263e Calc!A399 `T.INV(0.975,5)`: Excel 2.570581835636315, ours 2.5705818356363146, 1 ulp
  - gen-m9/d910f000b60e2eac Calc!A109 `T.INV.2T(0.05,7)`: as A4 above, 1 ulp
  - ironcalc-calc/STATISTICAL/T_DIST.xlsx `T.DIST T.INV T.DIST.RT`!E66 `T.INV(0.9999999,10)`: Excel
    12.492209688254745, ours 12.492209688254748, 2 ulp
  - ironcalc-calc/STATISTICAL/T_DIST.xlsx `T.DIST.2T T.INV.2T`!D50: Excel 2.2281388519862744, ours
    2.228138851986275, 1 ulp
- Ungated (probes-numeric) cells newly failing, all within 15 digits: m9-t-inv-2t A56 `T.INV.2T(0.01,5)` (1 ulp),
  A57 `T.INV.2T(0.1,5)` (3 ulp); m9-t-inv A55 `T.INV(0.001,5)` (1), A57 `T.INV(0.1,5)` (1), A61 `T.INV(0.9,5)` (2);
  t1048-tinv A2 `TINV(0.1,5)` and B2 `T.INV.2T(0.1,5)` (3 each). Known failures that moved further, still within
  15 digits: t1531-t-tiny-p A54 `T.INV(1E-15,5)` 1 -> 11 ulp (q below 1E-5 with x about 5E-6, so the crossing
  rather than the closed form), A64 and A65 and gen-m9/cb61169f4c033d4b A45, 8fec7b207f4dd753 A2 (df 5 at 1E-20)
  4 -> 5 ulp, `CONFIDENCE.T(0.1,1,3)` 5 -> 6 ulp.
- Variants T-2511 measured on gen-m9, ironcalc-calc, probes and probes-numeric (visibly-off cells against main):
  - the crossing of the ported kernel's tail at y (`bratio::t_kernel_tails`, as T.DIST takes it): 298 fixed, 3 new
    (`T.INV(0.2,5)` = -0.9195437802408258, ours -0.9195437802408264, gen-m9/0bbab6206b5c9a07 A67; probes
    stat-argument-modes H522, H526), while it fixes `T.INV(0.25,30)` (probes-numeric m9-t-inv A84, A86; m9-t-inv-2t
    A85), which the shipped form leaves 17 ulp off;
  - the other end of the final bracket (`refine_increasing`) instead of the nearest: 285 fixed, 6 new;
  - AS 109's y (statrs' `inv_beta_reg`) with Excel's back-transform: 137 fixed;
  - the shipped form: 294 fixed, 0 new.
- Split: T-0967 (Student-t quantile parity) also lists gen-m9/0bbab6206b5c9a07 A67; it keeps the cell's visible
  target, and this task the bit-level work on it.
- At q = 0.05 Excel's y is about 2 ulps from the nearest crossing at df 999 and 1E10 alike (T-3061's 10 cells left,
  e.g. `T.INV.2T(0.1,1E10)` = 1.6448533237762966, ours 1.6448539987439552): a stop rule of Excel's search, not a
  tail accuracy limit. T-3061 keeps those cells; this task is the bit-level remainder.

## Acceptance criteria
- [ ] The cells above return Excel's bits, or the Outcome shows the rule that would and why it is not taken.
- [ ] No cell becomes visibly off.
- [ ] `make check` green; reviewers approve.

## Out of scope
- df 1 (T-2858), the centre (T-2724), df of 99999 or more at p of 1E-200 or below (T-1542).
