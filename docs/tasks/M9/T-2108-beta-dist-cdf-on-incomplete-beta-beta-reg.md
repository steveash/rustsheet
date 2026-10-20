---
id: T-2108
title: "BETA.DIST cdf on incomplete_beta::beta_reg; decide the nine chained BETA.INV cells it moves"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-0970]
---

## Goal
BETA.DIST and BETADIST's cdf use the engine's incomplete beta (`statistical/incomplete_beta.rs`,
T-0970) instead of statrs' `Beta::cdf`, and the BETA.INV cells that move with it are settled.

## Context
T-0970 ported the T-1744 best combination (plus a TOMS 708 prefactor past MAXGAM) and moved
F.DIST, F.DIST.RT, F.TEST and the BETA.DIST density to it. It left BETA.DIST's cdf on statrs
because the switch, measured on that branch, fixes 16 more ironcalc-calc cells but moves nine
BETA.INV(BETA.DIST(...)) cells in `ironcalc-calc/STATISTICAL/BETADIST_BETAINV.xlsx` (Sheet1!H2,
H14, H42, H44, H46, H47, H48, H49, H67) from bit-exact to 1-2 ulp. That is so whether BETA.INV
inverts statrs' cdf or the new one, and also with a bisection or a 64-ulp polish on the new cdf.
These cells passed only because statrs' cdf and inverse are wrong in the same way.
The H42 group follows G42, which stays 4 ulp from Excel. See docs/excel-semantics/statistics.md
"Incomplete beta in the ironcalc-calc workbooks" and "BETA.INV solves Excel's own CDF to the
crossing (T-1745)".
Recording the regressions needs `update-baseline --allow-regressions` under a decision record
(directive 2; precedent decision 0064). An autonomous run was not permitted to run that command,
so a person, or a reviewer-approved decision, has to settle it first.
Code: `functions/statistical/beta.rs` `beta_dist_of` (TODO(T-2108)), `inverse_beta.rs`
(`beta_inverse_cdf` should invert the same cdf), the lift weights' comments.
F.INV and F.INV.RT have the same mismatch: they still invert statrs' F cdf while F.DIST uses
`incomplete_beta` (`F_DIST.xlsx` J2, J3, L2, L3 stay NumericClose). Move them in the same
change, or measure them and file a separate task.

## Acceptance criteria
- [ ] A decision record allows exactly the chained BETA.INV lines (or they are avoided), and the
      baseline diff adds no other line.
- [ ] BETA.DIST's cdf is `incomplete_beta`'s; BETA.INV (and F.INV, or a filed task) inverts the same cdf.
- [ ] `make check` green; reviewers approve.

## Out of scope
- BETADIST_BETAINV G5 and G26 (Cephes' fraction 160 ulp off where Excel is correctly rounded;
  Excel 34 ulp from correct): the T-1044 sweep work.
