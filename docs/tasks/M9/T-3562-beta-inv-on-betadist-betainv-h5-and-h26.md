---
id: T-3562
title: "BETA.INV on BETADIST_BETAINV H5 and H26: Excel's landing point off its own crossing (H26 at shapes 1E-4 is 1.3E-11 off the math)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-3561, T-3566, T-3388]
campaign: visible
---

## Goal
BETA.INV returns Excel's value, within half a unit of the 15th significant digit, on the two gated cells below where
Excel's quantile sits off its own BETA.DIST crossing. The value comes from a model of Excel's search that holds on
held-out and fresh sweep points, or the task records the hypotheses it ruled out.

## Context
- Gap triage 2026-11-01 (`make eval` on b98b3c00d). These two cells count against the M9 exit row's ironcalc-calc
  measure (44 R1(c) visibly-off cells, allowance 36). The only task that names them is a `defer:` one, in an Out of
  scope line (T-3374), so they have no real owner. T-3561 owns G5, the BETA.DIST input of H5.
- The cells:
  - ironcalc-calc/STATISTICAL/BETADIST_BETAINV.xlsx (2): Sheet1!H5, H26
  - H5 `_xlfn.BETA.INV(G5,3,600,2000,5489582)`: Excel 33587.000000000226, ours 33586.99999999992. At Excel's own G5
    (0.6734553797493481, correctly rounded), the 50-digit root is 33587.000000000000409, so Excel is 6.6E-15 above
    it, which is 2.3 units of the 15th digit. Ours stays at 33586.99999999992 even from Excel's G5 (scratch case),
    because it inverts our cdf, which is 2.5E-14 high there (T-3561). So H5 needs T-3561's kernel and then Excel's
    landing point.
  - H26 `_xlfn.BETA.INV(G26,0.0001,0.0001,0.15,1.2)`: Excel 0.2339999999999406, ours 0.23399999999995197. Excel's
    G26 is 0.4998779049300665. The 50-digit root at that p is 0.23400000000293818, so Excel is 1.3E-11 relative
    below it: far outside the 15th digit, but inside Gate A's 1E-9. At shapes 1E-4 the cdf is almost flat around
    1/2, so one ulp of p moves the quantile by about 8E-13. Our BETA.INV at Excel's G26 gives 0.23399999999915488,
    and at our G26, one ulp away, 0.23399999999995197. So H26 also needs G26 bit-exact (T-3374, deferred) or a
    model that does not depend on the last bit of p.
- The rule and earlier findings: docs/excel-semantics/statistics.md "BETA.INV solves Excel's own CDF to the crossing
  (T-1745)", which now has these two cells (this triage). T-3388 fits Excel's landing point at small and moderate
  shapes (flat cdf, off-crossing stops) from T-3387's sweep. Share its start and stop model; do not write a second
  search. T-3387 sweeps shapes from 0.5 up, and neither shapes below 0.01 nor a scaled range [A, B] are probed. That
  is T-3566's sweep, so this task waits for it.
- Code: `fn_beta_inv` in crates/rustsheet-engine/src/functions/statistical/beta.rs, `beta_quantile` and
  inverse_beta.rs `beta_quantile_crossing`.

## Acceptance criteria
- [ ] These 2 known-failure lines disappear from eval/baseline/ironcalc-calc.txt with no new ones in any baseline:
      `F	ironcalc-calc/STATISTICAL/BETADIST_BETAINV.xlsx	Sheet1!H5` and `...	Sheet1!H26`. Or each cell left off is an
      UNVERIFIED entry in statistics.md with the hypotheses it rules out and their counts (decision 0056).
- [ ] Decision 0188 R4 guards (reviewer-excel-fidelity check 7), in the Outcome: a hold-out split of T-3566's and
      T-3387's points fixed before scoring, with its margin, and hold-out counts (bit-exact, within 4 ulp, within 15
      digits) for the chosen model and the correctly rounded quantile; a fresh probes-numeric sweep filled after the
      freeze, with its result; the Gate A envelope result on that sweep, with every point outside it listed; each
      fitted piece tagged `FITTED(T-3562, n_cells, range)` and applied only inside that range, with accurate math
      outside it; no target below the reproducibility floor (4 ulp web/desktop); and no keep-matched branch.
- [ ] No cell of any suite goes from within 15 digits to visibly off (main-against-HEAD `make eval` deltas in the
      Outcome).
- [ ] `make check` green; reviewers approve.

## Out of scope
- G5 (T-3561); G26 and the other BETADIST_BETAINV last bits (T-3374); the m9-beta-inv cells (T-3388).

## Visible-parity stage
Decisions 0083 and 0114 (`visible`): done when H5 and H26 are within half a unit of the 15th significant digit of
Excel's value, or are UNVERIFIED entries with their ruled-out hypotheses. Bit-level residue moves to a follow-up with
`defer: phase-1b`.
