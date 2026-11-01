---
id: T-3640
title: "Excel's far-tail Q above shape 11 and past x = 600: the 37 t3269-fresh-sweep cells T-3269's band leaves"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-3639]
defer: phase-1b
---

## Goal
Excel's Q(a, x) (CHISQ.DIST.RT, CHIDIST, CHISQ.TEST and POISSON's cdf) is reproduced where T-3269's far-tail form is not
applied today, above shape 11 at 30 < x <= 600 and at every shape past x = 600, by placing Excel's switch to the form
there (or finding the form Excel uses instead), so the 37 cells below are within 15 digits of Excel.

## Context
- Gap triage 2026-11-01: these cells of probes-numeric/t3269-fresh-sweep (T-3269's first fresh sweep, 240 cells,
  excel-graph) are visibly off and no open task named them. T-3269's rework (decision 0203, decision 0188 R4.3) applies
  the Stirling-ratio form only at shapes 3/2 to 11 and 30 < x <= 600 (`incomplete_gamma/excel_tail.rs`
  `FAR_TAIL_MAX_SHAPE`, `FAR_TAIL_MAX_X`, the latter UNVERIFIED), and outside it the callers keep the accurate kernel.
  Ulps below are ours minus Excel's (`make eval-show CASE=probes-numeric/t3269-fresh-sweep`), and Excel's and ours
  against 50-digit mpmath Q at the double arguments (gap triage, mpmath 50 digits):
  - above shape 11, 30 < x <= 600 (30 cells): A146 `CHISQ.DIST.RT(140.208,23)` +45 (Excel -45 from the math, ours 0),
    A188 `(1014.81,23)` +226, A16 `(1106.31,23)` +208, A67 `POISSON(11,105.561,TRUE)` +49, A231 `POISSON(11,148.398,TRUE)`
    +51, A20 `(362.23,29)` +33, A130 `(1178.5,29)` -335, A47 `POISSON(14,450.407,TRUE)` -41, A151 +7, A138 +30, A62 -37,
    A148 +20, A117 -25, A5 -140, A55 -57, A229 -56, A18 -161, A94 -147, A157 `POISSON(17,31.4903,TRUE)` -28, A109 +25,
    A34 -148, A192 +33, A3 -11, A91 -50, A224 -28, A132 -109, A95 `POISSON(19,259.599,TRUE)` +148, A206 +39, A71 -185,
    A97 `POISSON(21,545.317,TRUE)` -437;
  - past x = 600 (7 cells): A185 `POISSON(1,626.167,TRUE)` +315, A84 `CHISQ.DIST.RT(1333.98,7)` -158, A184
    `(1268.62,17)` -380, A21 `POISSON(10,712.736,TRUE)` +26, A39 `POISSON(12,671.582,TRUE)` -953, A158 `(1358.54,35)`
    -189, A6 `(1308.88,39)` -163.
- Excel is 7 to 955 ulp from the math at every one of them (relative up to about 1E-13), so this is a fit of Excel's
  formulation, not an accuracy fix; ours is within 2 ulp of the math at 30 of the 37, and 121 to 312 ulp off it at four
  POISSON cells (A21, A71, A95, A97: POISSON's `poisson_upper` path at a >= 11, where CHISQ.DIST.RT's twin shapes are
  correctly rounded). All 37 are inside Gate A's envelope (ours within about 4.1E-14 relative of the math), hence Phase 1b
  under decision 0188 R5.
- T-3269's review notes (carried in T-3469) say above shape 11 at 45 < x <= 600 the form beats the accurate kernel
  (hold-out train 88 / 16, test 35 / 2), and the cut next to x = 30 is where it fails (`FAR_TAIL_MAX_SHAPE`'s doc: 1
  closer, 3 farther). So the lead is a switch in x that grows with the shape above 11, not a shape cut; T-3469 keeps
  shape 6 and x/a = 10 and hands this placement here.
- Needs an oracle sweep: past x = 600 the cached workbooks have 12 nonzero points (docs/research/t3269/holdout-split.tsv),
  7 of them these cells; T-3639 sweeps both regions with a hold-out.
- The subnormal end of the tail (A92 `CHISQ.DIST.RT(1529.95,21)`) is T-3638's, a semantic rule. The half-integer cells
  of the same sweep at x <= 30 are T-3461's.
- Method: docs/research/excel-kernel-playbook.md.

Cells this task owns:
- probes-numeric/t3269-fresh-sweep.case.json (37): Calc!A3, A5, A6, A16, A18, A20, A21, A34, A39, A47, A55, A62, A67,
  A71, A84, A91, A94, A95, A97, A109, A117, A130, A132, A138, A146, A148, A151, A157, A158, A184, A185, A188, A192, A206,
  A224, A229, A231

## Acceptance criteria
- [ ] A model of Excel's switch (or form) above shape 11 and past x = 600 fitted on T-3639's train case and the
      t3269 hold-out's train split, then scored once on T-3639's hold-out case, with exact / within-4-ulp /
      within-15-digit counts against today's head in statistics.md "Excel's far upper tail (T-3269)", or the
      hypotheses ruled out with their counts.
- [ ] The listed cells are within 15 digits of Excel, or each one left is named with the hypothesis that fails it.
- [ ] Decision 0188 R4 guards (reviewer-excel-fidelity check 7): hold-out counts reported (train and hold-out
      separately, the hold-out scored once); a fresh log-uniform sweep generated and oracle-filled after the freeze,
      with its counts; the Gate A envelope result on every touched point (no point outside 1e-9 of the math or 15
      digits of Excel); each fitted constant tagged `FITTED(T-3640, n_cells, range)` and not used outside that range
      (`FAR_TAIL_MAX_X` and `FAR_TAIL_MAX_SHAPE` re-tagged or replaced); no fitting below the oracle's reproducibility
      floor (decision 0188 R4.4, 4 ulp); no branch that only keeps already-matched cells.
- [ ] No gated cell regresses (main-vs-HEAD per cell); `make check` green; reviewers approve.

## Out of scope
- Shape 6 and x/a = 10 (T-3469); half-integer shapes on [a + 1, 30] (T-3461); a subnormal Q (T-3638); the near-mode
  window at large shapes (T-3286).
