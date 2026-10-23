---
id: T-2465
title: "F.INV and F.INV.RT: the last visibly-off cells after T-0974 (df near 1E5, near-tie crossings)"
milestone: M9
lane: functions-stat
status: done
priority: P1
size: S
tier: opus
campaign: visible
depends_on: [T-0974]
---

## Goal
The F.INV, F.INV.RT and FINV cells T-0974 left visibly off (more than half a unit in the 15th significant digit
from Excel) reach 15 digits, with no new known-failure lines.

## Context
T-0974 refines statrs' F quantile to the crossing of the T-0970 incomplete beta (`fisher.rs` `f_quantile`,
`special::refine_increasing`) and keeps statrs' estimate below p = 1E-15 (`F_QUANTILE_REFINE_P_MIN`). On its
branch these gen-m9 cells (excel-graph) stay visibly off (units of the 15th digit in brackets):
- df near 1E5, where statrs still gives the start and the crossing of our cdf is off:
  27601819397dd56e A81 `F.INV.RT(0.05,99999,9)` Excel 2.7067247920355353, ours 2.706724792036013 (48);
  c18dbf21f11093f3 A190 `F.INV.RT(0.001,99999,1)` Excel 636616.2559081278, ours 636616.2559077828 (345).
  The forward kernel at a huge shape (TOMS 708 `brcomp`-style prefactor) is the first suspect; compare
  F.DIST.RT at those x against Excel first (excel-probe).
- 1 unit or less, a crossing of a cdf a few ulp from Excel's: 320d51765af9b6b7 A379 `F.INV(0.5,3,9)`,
  4ab2ba98d26c8a5e A390 `F.INV(0.999999,3,1.999)`, 589f42e67f4b4b8d A365 `F.INV(0.5,1,3)`, 60b0b6e69ad35a45 A197
  `F.INV(0.975,6,1)`, a61e547a1cab994c A297 `F.INV.RT(Data!D$2,7,1)`, c2974c061384fdd9 A267, eced4bb6a0234c9e A289.
- F.INV(1E-50,6000,4) (probes/stat-quantile-lift E6): Excel 0.0004924127886173762, ours 0.016348828 (bisection of our
  cdf with no statrs start).
The BETAINV cells of T-0974's list (589f42e67f4b4b8d A221, c18dbf21f11093f3 A400, eced4bb6a0234c9e A291) are T-2119's:
BETA.INV inverts BETA.DIST's cdf once that moves to `incomplete_beta`.

Cleanups from the T-0974 review (MINOR, carried here because they touch this code):
- `fisher.rs` `F_QUANTILE_BISECT_DF` only means "no statrs start" now; rename it (e.g. `F_QUANTILE_STATRS_DF_MAX`).
- `incomplete_beta.rs` `A1_CLOSED_FORM_MIN_X` and `student_t.rs` `CLOSED_FORM_MIN_Y` are the same 0.1 cut: keep one
  `pub(crate)` constant.
- `special.rs` has three monotone-cdf bisections (`bisect_increasing`, `bisect_unit_interval`, `refine_increasing`):
  delete the first two once T.INV (t_dist.rs) and BETA.INV (beta.rs, T-2119) call `refine_increasing`.
- T-0974's unmet criteria: "15 of 28 ironcalc lines" and "no statrs inverse_cdf for Beta" moved here and to
  T-2119.

## Acceptance criteria
- [ ] The F-family cells above are within half a unit of the 15th significant digit of Excel's value.
- [ ] No new known-failure lines in any `eval/baseline/` file.
- [ ] `make check` green; reviewers approve.

## Out of scope
- BETA.INV / BETAINV (T-2119); bit-exact F quantiles.

## Outcome
- Root cause of the two df-near-1E5 cells: `incomplete_beta.rs` `large_prefactor` (one shape below 8, a + b of MAXGAM
  or more) took `a * ln x` with x = 1 - 3E-5 rounded, so ln x kept only about 5 digits' worth of y. It now takes
  ln(1 - y) from the given complement above x = 1/2 (`ln_near_one`, TOMS 708 `brcmp1`'s `alnrel(-y)`).
  `F.INV.RT(0.05,99999,9)` went from 48 units of the 15th digit off Excel to 0.27 (and is now the correctly rounded
  root; Excel is 5 ulp high). `F.INV.RT(0.001,99999,1)` went from 345 units to 1.05.
- Eval: 0 known-failure lines added or removed (the cells still differ in the last ulps). Three gen-m9 F cells now display Excel's 15 digits: 4bad303d32cc3f46 A141 (F.DIST.RT,
  df1 1E10), 7c20351562a2d71d A254 (FINV, df1 1.2E8) and 8dd2312c96effb1b A395 (F.DIST). No cell in any suite,
  probes-numeric included, newly displays differently.
- Cleanups from the T-0974 review: `F_QUANTILE_BISECT_DF` is renamed `F_QUANTILE_STATRS_DF_MAX`, and student_t.rs
  shares `incomplete_beta::A1_CLOSED_FORM_MIN_X` instead of its own copy of the 0.1 cut. `bisect_unit_interval` goes
  with T-2119 (its only caller). `bisect_increasing` stays until T.INV moves to `refine_increasing`.
- Unmet: the near-tie cells (0.7 to 1.1 units, crossings of a cdf a few ulp from Excel's), A81 and A190's last unit,
  and E6 move to **T-2500**.
