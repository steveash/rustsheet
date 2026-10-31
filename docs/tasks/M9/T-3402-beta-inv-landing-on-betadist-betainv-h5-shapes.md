---
id: T-3402
title: "BETA.INV landing on BETADIST_BETAINV H5 (shapes 3 and 600, a miss off Excel's crossing) and H26 (shapes 1E-4, a flat cdf): match Excel to 15 digits"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-3400, T-3401, T-2511]
campaign: visible
---

## Goal
BETA.INV and BETAINV return Excel's value, to 15 significant digits, at the two ironcalc-calc points where Excel's
landing differs visibly from the crossing of a correctly rounded cdf. Needs an oracle sweep: the model of Excel's stop
is fitted on T-3400's landing rows under decision 0188 R4. If no model reproduces Excel there, the task records the
hypotheses it ruled out.

## Context
- Owned cells (gap triage 2026-10-31, `make eval` on main 83d35563). Both are visibly off in ironcalc-calc, which
  counts for M9's exit row and for Gate B. Deferred T-3374 only names the case in its out-of-scope line.
  - ironcalc-calc/STATISTICAL/BETADIST_BETAINV.xlsx (2): Sheet1!H5, H26
  - H5 `BETA.INV(G5,3,600,2000,5489582)`: Excel 33587.000000000226, ours 33586.99999999992. The engine gives the same
    value when fed Excel's own G5 (0.6734553797493481), so this is not only G5's error (T-3401). The math quantile of
    Excel's p is 33587.0000000000004. At Excel's x the math cdf is 42 ulps of p above p, and at ours 15 below. Excel
    is correctly rounded at G5, so its landing here is a "miss" in T-1745's sense, like
    `BETA.INV(0.05,2.5,100)` (probes-numeric/m9-beta-inv A101).
  - H26 `BETA.INV(G26,1E-4,1E-4,0.15,1.2)`: Excel 0.2339999999999406, ours 0.23400000000265136. The math inverse of
    Excel's p (0.4998779049300665) is 0.234000000002938, so Excel is 1.3E-11 relative off the math. G26
    (`BETA.DIST(0.234,1E-4,1E-4,TRUE,0.15,1.2)`) is itself 34 ulps above the math in Excel (ours one more) and is
    within 15 digits. At this shape the density is about 6.5E-4, so one ulp of p spans about 8.6E-14 of x: Excel
    lands on its own flat crossing, near the input 0.234. Ours stops 2E-12 above its own crossing (T-2511's crossing
    search, in flight).
- The rule and its open part: docs/excel-semantics/statistics.md "BETA.INV solves Excel's own CDF to the crossing
  (T-1745)" (155 of 188 crossings; flat and miss landings; "the solver's start, step and stop are UNVERIFIED").
  T-3387 and T-3388 (branch agent/functions/tasks-gap-triage-opus-2026-10-31b) sweep and fit the landing for a in
  {0.5, 1, 2.5, 10} and b up to 1000, unbounded. They cover neither shape 1E-4 nor (3, 600) with bounds. T-3400 adds
  those rows. Reuse T-3388's model if it has landed; do not fit a second one.
- Code: `functions/statistical/inverse_beta.rs`, `special::refine_increasing`, beta.rs `beta_inv_of`.

## Acceptance criteria
- [ ] The known-failure lines `Sheet1!H5` and `Sheet1!H26` of `ironcalc-calc/STATISTICAL/BETADIST_BETAINV.xlsx`
      leave eval/baseline/ironcalc-calc.txt, and no new known-failure line appears in any baseline. Otherwise the
      Outcome records each model tried, with its counts on T-3400's landing rows, and names the cells' next owner.
- [ ] R4 guards (decision 0188, reviewer-excel-fidelity check 7), shown in the Outcome for the chosen model:
      - hold-out: train and hold-out counts side by side (bit-exact, within 4 ulp, within 15 digits) against the
        correctly rounded crossing, on T-3400's fixed hold-out, with the margin stated before scoring;
      - fresh sweep: a probes-numeric case generated and oracle-filled after the model was frozen;
      - Gate A envelope: that sweep's count within 15 digits of Excel or 1E-9 of the 50-digit math, with every
        point outside it listed;
      - `FITTED(T-3402, n_cells, range)` on each fitted constant, cut or stop rule, applied only inside its range,
        with accurate math (the crossing) outside it;
      - reproducibility floor: no step fits differences below 4 ulp web against desktop (decision 0112);
      - no keep-matched branch: nothing whose domain is the set of cells that already matched.
- [ ] No probes-numeric BETA.INV, BETAINV, F.INV or T.INV cell that is within 15 digits of Excel today goes visibly
      off.
- [ ] statistics.md's T-1745 entry records the landing rule found, or the negatives.
- [ ] `make check` green; reviewers approve.

## Out of scope
- G5 and the cdf kernel (T-3401). The crossing search itself (T-2511). Huge shapes (T-2555) and p near 1 (T-3351).

## Visible-parity stage
Decision 0083: done when H5 and H26 are within half a unit of the 15th significant digit of Excel's value, or are
named in the Outcome with a follow-up owner. Bit-level work moves to a `defer: phase-1b` follow-up without the
campaign key.
