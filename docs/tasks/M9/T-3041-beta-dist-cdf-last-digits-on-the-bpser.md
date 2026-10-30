---
id: T-3041
title: "BETA.DIST cdf last digits on the bpser runs probe for min(a,b) <= 1 (t2464-r2-bpser-small-runs, 2,493 cells within 15 digits)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-3007, T-3042]
defer: phase-1b
---

## Goal
BETA.DIST's cumulative value in TOMS bpser's region (min(a, b) <= 1) matches Excel bit for bit on the
consecutive-double runs, so the probe's 2,493 one-to-few-ulp cells pass. If no model does, the task records the
candidates and their counts (decision 0056).

## Context
Gap triage 2026-10-28 (`make eval` on origin/main 4b20263be). No open task names this case.
T-2981 routed the other incomplete-beta sweep cases and T-2933 has the tiny shapes, but neither covers it.
- Cells: probes-numeric/t2464-r2-bpser-small-runs, every failing cell of the case: 2,493 of 8,192, all BETA.DIST
  cumulative. 2,492 are NumericUlp and 1 is NumericClose, and none is visibly off. Each is 1 to a few ulp from Excel.
  For example (`make eval-show CASE=probes-numeric/t2464-r2-bpser-small-runs.case.json`):
  - Calc!A10 `BETA.DIST((114729313*2^-29+60022177*2^-55),0.37,0.81,TRUE)`: Excel 0.5149272235236996, ours ...997.
  - Calc!A1004 `BETA.DIST((91778082*2^-28+27273533*2^-54),0.83,0.29,TRUE)`: Excel 0.1511567079953103, ours
    0.15115670799531034.
  - Calc!A1007 `BETA.DIST(96236294537381*2^-48,0.83,0.29,TRUE)`: Excel 0.15115670799531042, ours ...045.
- Also the legacy twins that sit in the same kernel, probes-numeric/m9-legacy-betadist Calc!A7, A8, A11, A12, A13, A14,
  A19, A20, A25, A26 (BETADIST and BETA.DIST pairs). An example is A11 `BETADIST(0.01,10,1.5)`: Excel
  3.6832805604961844E-20, ours ...826.
- The probe is 16 runs of 512 consecutive doubles, at least two per bpser sub-branch (b0 <= 1, 1 < b0 < 8,
  b0 >= 8). The research is docs/research/t2464/r2-bpser-small/RESULTS.md, with the model in its section 2:
  z = fl64(fl(a LN x) - L), I = fl(EXP(z) * s), and L built from Excel's GAMMALN. Its ceilings (section 3, item 3,
  `lcover_runs.out`) say a constant L reaches 100% on five of the 1 < b0 < 8 runs. The b0 <= 1 runs and the a > 1,
  b < 1 runs top out at 55-84%. For b0 >= 8, ln B (GL + algdiv) is unsolved, and `fixwhich.out` shows each miss
  is fixed by moving s or E one ulp.
- The model's L uses Excel's GAMMALN. The engine's GAMMALN is not bit-exact at these shapes (T-3042), so this
  task depends on it. It also depends on T-3007, which changes the bratio port the cells run through.
- Engine: `crates/rustsheet-engine/src/functions/statistical/bratio/` (bpser branch),
  `functions/statistical/gammaln.rs`. Lab: `lib.V(nofpser=True, noapser=True, lg="GL53/glalgdiv")` and `bscore.py` in
  docs/research/t2464/r2-bpser-small/.
- Semantics: docs/excel-semantics/statistics.md, the BETA.DIST incomplete-beta entries (T-2464, T-2505).

## Acceptance criteria
- [ ] A bpser formulation scored in the lab on t2464-r2-bpser-small-runs, on the shape-level hold-out half
      (`lib.shape_hold`), beats the shipped model's bit-exact count there and loses no gated cell. The engine then
      uses it, and `make eval` shows the case's failing count below 2,493 with no visibly-off cell. Otherwise the task
      is blocked time-boxed (decision 0056), with the forms tried and their counts recorded in RESULTS.md.
- [ ] No new F line in any eval/baseline/*.txt. The BETA.DIST, F.DIST and BINOM lines there only shrink.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The tiny-shape cells (T-2933) and the bratio port's open items (T-3007).
- GAMMALN's own kernel (T-3042, T-2122).

## Deferred (decision 0114)
`defer: phase-1b`: every cell this task owns is already within 15 significant digits of Excel, so it waits until
Phase 1 exits. Remove the key only when the steward records the Phase 1 exit, or when an eval shows one of its cells
visibly off (then give it a `campaign:` stage instead).
