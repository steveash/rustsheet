---
id: T-3764
title: "BETA.DIST cdf at b = 1 and a shape of 136 to 1414: Excel's w on the a0 = 1 complement is one 2^-53 step from ours on 47 t3568 column-B cells (x^a from 7E-17 to 2E-3)"
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
`BETA.DIST(x, a, 1, TRUE)` at x from 0.9 to 0.9965 and a from 136 to 1414 returns Excel's value on the 47 column-B
cells of the three t3568-r5-bup cases listed below. These cells take the a0 = 1 complement of `shape_one_tails`. The
fix finds where Excel's w = 1 - x^a differs by one unit of 2^-53 from the engine's
`-excel_expm1(b0 · excel_ln_one_minus(x0))`, and does not lose any of the 1,738 column-B cells that match today.

## Context
- Gap triage 2026-11-01k (`make eval` on origin/main 063c2dd9, excel-graph; `scripts/exit_owners.py --all-off`).
  These 47 cells are visibly off. Their only owner is T-3712, which cannot be picked: it depends on T-3382, which is
  still open. T-3712 names the three cases whole but works on column A (the L140 bup+bgrat cdf). It reads column B
  only as Excel's D, and its Context already says column B "is the a0 = 1 complement". Split: this task takes the
  column-B cells below, and T-3712 keeps column A and column C.
- After bratio's swap the cell is a0 = 1, b0 = a, x0 = 1 - x (0.0035 to 0.1), y0 = x > 0.9. That is the
  `shape_one_tails` branch (crates/rustsheet-engine/src/functions/statistical/incomplete_beta.rs). It computes
  w = -excel_expm1(b0 · excel_ln_one_minus(x0)) and returns w1 = (0.5 - w) + 0.5 (statistics.md "Incomplete beta at a
  shape of 1 outside the closed forms (T-3007)"). The rule was fitted on F cells at small b0. Here b0 is 136 to 1414
  on the failing cells (17.8 to 2970 over the whole column), so z = b0 · ln(1 - x0) runs from about -1 to well below
  -36, and w1 is a difference of two doubles near 1.
- On every failing cell, ours and Excel differ by exactly one 2^-53 step of w, and the complement amplifies that
  step:
  - t3568-r5-bup-fresh B744 `BETA.DIST(0.95591070645675,826.0,1.0,TRUE)`: Excel 1.1102230246251565E-16, ours
    2.220446049250313E-16, math x^a = 6.68E-17
  - t3568-r5-bup-tune B856 `BETA.DIST(0.9094554,241.0,1.0,TRUE)`: Excel 1.1649814446457185E-10, ours
    1.164980334422694E-10, math 1.164980992637334E-10
  - t3568-r5-bup-hold B816 `BETA.DIST(0.912187876,136.0,1.0,TRUE)`: Excel 3.7278996465328618E-06, ours
    3.727899646643884E-06, math 3.7278996465580678E-06
  - t3568-r5-bup-hold B1610 `BETA.DIST(0.9965,1414.0,1.0,TRUE)`: Excel 0.007029209602111175, ours
    0.007029209602111286, math 0.007029209602111176
- Evidence already in eval (no new sweep is needed to choose): 1,785 column-B cells, all of this form, across
  t3568-r5-bup-hold, -tune and -fresh, with Excel's values in their `.oracle.json`. The engine matches 1,738 of them.
  Gap-triage scratch scores, with libm stand-ins (not Excel's kernels) and the formula taken as written:
  - Several forms each fit all 47 failing cells and 516 of the 1,785: 1 - (1 - x^a), 1 + expm1(a·ln x),
    1 - (1 - exp(a·log1p(-x0))).
  - The engine's form fits the other 1,738.
  - By z = a·log1p(-x0), the failing cells fall in these bands (cells / engine matches / 1 - (1 - exp(z)) matches):
    - z in [-5, -1): 5 / 4 / 5
    - z in [-20, -5): 146 / 131 / 87
    - z in [-36, -20): 218 / 194 / 107
    - z < -36: 1,416 / 1,409 / 317
  So no single libm form explains the whole column. Two models to try first:
  - Excel's w comes from Kahan's expm1 with a different rounding of `(u - 1) * x / LN(u)`. numbers.md "Excel's expm1
    is Kahan's formula" notes that rounding the product and quotient to 64 bits also fits its sweep.
  - Excel's expm1 switches to EXP(x) - 1 from |x| >= 1, as PMT's 1-Q does (financial.md "PMT is a discounted
    payment", T-3014; numbers.md says whether that switch belongs to Excel's expm1 itself "is not established").
  Also compare excel_ln_one_minus(x0) with an accurate ln(1 - x0): b0 multiplies its last-bit error by up to 1414.
- Excel is far from the math where x^a is tiny: B744 is 66% above it. Matching Excel's value is always right (CLAUDE.md
  directive 1). Do not move these cells to the math: that needs an R3 owner ruling (decision 0188), and this task does
  not ask for one.
- Related:
  - T-3150 (Excel's w1 below 2^-53 at a0 = 1) covers the region where w1 is a multiple of 2^-53. Four cells below are
    in it (B744, B1130, B1609, B1546, with x^a < 2^-53). Whichever task lands first takes them and records it in the
    other's file.
  - The same branch serves F.DIST and F.DIST.RT with a d1 of 2 (a0 = 1) and NEGBINOM/BINOM at a shape of 1. Check
    their eval cells for regressions.
- Code: `shape_one_tails` and `closed_form_tails` in functions/statistical/incomplete_beta.rs; `excel_expm1` in
  x87_exp.rs; `excel_ln_one_minus` and `excel_log1p` in x87_log1p.rs. Read docs/research/excel-kernel-playbook.md
  first. Decision 0056 applies: after two attempts that do not raise the bit-exact count, time-box.

Owned cells (all `BETA.DIST(x, a, 1.0, TRUE)`):
- probes-numeric/t3568-r5-bup-fresh.case.json (9): Calc!B482, B502, B535, B572, B652, B681, B744, B764, B798
- probes-numeric/t3568-r5-bup-hold.case.json (26): Calc!B816, B955, B983, B985, B1002, B1003, B1010, B1043, B1048, B1051, B1055, B1091, B1113, B1130, B1204, B1227, B1420, B1433, B1468, B1546, B1574, B1583, B1590, B1604, B1610, B1616
- probes-numeric/t3568-r5-bup-tune.case.json (12): Calc!B847, B856, B884, B957, B1113, B1214, B1233, B1320, B1363, B1515, B1600, B1609

## Visible-parity stage
Decision 0083 / 0114 (`visible`): met when the 47 cells above are within half a unit of the 15th significant digit
of Excel's value, and no column-B cell of the three cases that is within 15 digits today moves off. Bit-level
criteria still unmet then go to a follow-up with `defer: phase-1b`.

## Acceptance criteria
- [ ] The 47 owned cells are within half a unit of Excel's 15th significant digit (`make eval-show` on the three
      cases). Column B of the three cases has more cells within 15 digits than 1,738 of 1,785, and the Outcome shows
      the before and after counts.
- [ ] No gated cell and no probes-numeric cell goes from within 15 digits to visibly off. In particular, check the
      F.DIST/F.DIST.RT/FDIST cells at a d1 of 2, the shape-of-1 cells of t1745-beta-inv-roundtrip, and
      BETADIST_BETAINV.xlsx. Any that moves is listed with Excel / new / exact values, or the change is narrowed.
- [ ] R4 guards (reviewer-excel-fidelity check 7, decision 0188), since this picks a rounding model or a switch from
      ranked candidates (kind (c), or kind (a) if a cut is fitted):
      - Hold-out counts: train and hold-out side by side (bit-exact, within 4 ulp, within 15 digits) for the chosen
        model and the correctly rounded value. The split is about 30% of the column-B cells by cell-id hash, fixed
        with its margin before any scoring. The chosen model beats correctly rounded on hold-out.
      - Fresh sweep: drawn after the model is frozen, as a new probes-numeric case, log-uniform over x0 in
        [1E-3, 0.1] and b0 in [2, 3000]. It includes points with w1 from 1E-3 down to about 1E-16 and F.DIST.RT
        companions at d1 = 2. The result is recorded in the Outcome.
      - Gate A envelope: the fresh sweep's count within 15 digits of Excel or within 1e-9 of 50-digit math, with
        every point outside listed (mpmath values attached).
      - `FITTED(T-3764, n_cells, range)` tag and range on any fitted cut, with accurate math outside the range.
      - Reproducibility floor: no step fits differences below the oracle's floor.
      - No keep-matched branch: no band defined by the cells that already match.
- [ ] statistics.md "Incomplete beta at a shape of 1 outside the closed forms" records the model for large b0, citing
      these cells. If no model fits, it lists every candidate tried with its counts.
- [ ] A unit test pins at least two owned cells at Excel's value, naming the case and cell.
- [ ] No new known-failure line in any eval/baseline/<suite>.txt (`make eval-baseline` refuses regressions).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Column A, the L140 bup+bgrat cdf (T-3712), and column C, the density (T-3569, T-3392).
- w1 below 2^-53 beyond the four cells named above (T-3150).
- Moving any cell to the math (R3, which needs an owner ruling).
