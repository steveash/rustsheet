---
id: T-3603
title: "BETA.DIST cdf on bratio's bup + bgrat at a first shape below 1: 21 t2464-w2f-unit-shape cells 5 to 26 ulp off Excel, once T-3568 and T-3582 model the branch (Phase 1b)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-3568, T-3582]
defer: phase-1b
---

## Goal
After T-3568 (bup's terms and arithmetic) and T-3582 (Excel's bgrat increment at b != 1/2) give a model, BETA.DIST's
cdf takes bratio's bup + bgrat branch at a0 < 1 with that model instead of the Cephes fallback, and the 21 cells below
come within 4 ulp of Excel's value. The 1,046 cells of the same branch that are already within 4 ulp do not move
further from Excel.

## Context
- Gap triage 2026-11-01 (`make eval` on origin/main 2f6dbaff1, excel-graph). An ownership check over every failing
  Excel cell (`scripts/exit_owners.py`'s `owned` rule, open tasks of origin/main) found 940 cells of
  probes-numeric/t2464-r2-bratio-x29 and 127 of probes-numeric/t2464-w2f-unit-shape that no open task names. None is
  visibly off: every one is within half a unit of Excel's 15th significant digit. Decision 0188 R5 makes this Phase 1b,
  hence `defer: phase-1b`. Open triage PRs #2602 and #2607 (unmerged) name only the 6 + 9 visibly-off cells of these two
  cases, which T-3592 (on main) already owns.
- **Reproducibility floor (decision 0188 R4.4).** Differences of 4 ulp or less are not targets. All 940 x29 cells
  (1 to 4 ulp) and 106 of the 127 unit-shape cells are below the floor. This task owns them only as the no-regression
  set of the branch. The targets are the 21 cells more than 4 ulp from Excel.
- **Branch.** bratio's dispatch (statistics.md "Incomplete beta is TOMS 708 `bratio`") sends a0 < 1 < b0 with
  0.1 <= x0 < 0.3 to w1 = bup(b0, a0, y0, x0, 20) and then bgrat from b0 + 20, or straight to bgrat when b0 > 15.
  The engine ports this branch only at a0 = 1/2 (`bgrat_half`). Every other a0 falls back to `beta_reg_tails`
  (incomplete_beta.rs): `ported_tails_with` in crates/rustsheet-engine/src/functions/statistical/bratio/mod.rs
  (`Branch::BupBgrat`, `Branch::Bgrat`), comment "T-3582 owns the rest".
- **What Excel does there (new quirk, statistics.md "Incomplete beta at a first shape a few ulps below 1 jumps between
  neighbouring shapes").** At x = 0.1, b = 2.5 or 5 and a = 1 - k·2^-53 (k 1 to 16), Excel's value is not smooth in
  a. It moves by up to 36 ulp between neighbouring shapes (the exact value by about 1.5) and lies 25.5 ulp below to 11.4 ulp above the exact value
  (mpmath at 40 digits on the double arguments). The engine stays within 7.2 ulp of the exact value. This is the
  noise T-3582 is chasing: along runs of consecutive doubles, Excel's direct bgrat increment minus the exact one is
  white noise of sd 5.7 to 8.5 units of 2^-53 (docs/research/t2464/r5-bgrat-fdf1/RESULTS.md). Excel / ours / exact,
  with the signed ulps of Excel and ours from the exact value:
  - A213 `BETA.DIST(0.1,0.999999999999999,2.5,TRUE)`: 0.23156652857908366 / 0.23156652857908414 /
    0.231566528579084219 (-20.0, -3.0)
  - A227 `BETA.DIST(0.1,(67108863*2^-26+67108863*2^-52),2.5,TRUE)`: 0.23156652857908344 / 0.23156652857908394 /
    0.231566528579083920 (-17.2, +0.8)
  - A209 `BETA.DIST(0.1,(134217727*2^-27+67108853*2^-53),2.5,TRUE)`: 0.23156652857908455 / 0.2315665285790841 /
    0.231566528579084304 (+8.9, -7.1)
  - A40 `BETA.DIST(0.1,0.5,(67108864*2^-26+3*2^-52),TRUE)` (b0 = 1 + 3·2^-52, a0 = 1/2, the ported `bgrat_half` path):
    0.316227766016838 / 0.31622776601683766 (-1.2, -7.2). Here Excel is the accurate one.
- Below-floor examples (x29, a < 1 and x in [0.29, 0.3), Excel / ours): A100 `BETA.DIST(0.2999,0.25,2.5,TRUE)`
  0.902772010636735 / 0.9027720106367351; A1005 `BETA.DIST(0.2914,0.5,7.5,TRUE)` 0.9746909208143628 /
  0.9746909208143629. On the 940, the exact value is within 0.5 ulp of Excel on 264 and of ours on 470. The x29
  case's x0 cut is statistics.md's "x0 >= 0.3, not R's 0.29" (63 of its cells match neither dispatch).
- Not attempted blind: the evidence for the model comes from T-3568 and T-3582, which commit their own probes. Fitting
  the 21 cells alone would fit Excel's noise.
- Cells owned:
  - probes-numeric/t2464-w2f-unit-shape.case.json (21 targets): Calc!A40, A199, A203, A205, A207, A209, A213, A215, A217, A221, A223, A225, A227, A229, A397, A399, A403, A405, A413, A415, A425
  - probes-numeric/t2464-w2f-unit-shape.case.json: the whole case, its other failing cells not named by another open task (106 at this run, 1 to 4 ulp; the no-regression set)
  - probes-numeric/t2464-r2-bratio-x29.case.json: the whole case, every failing cell not named by another open task (940 at this run, 1 to 4 ulp; the no-regression set)

## Acceptance criteria
- [ ] The 21 target cells are each within 4 ulp of Excel's value (`make eval-show
      CASE=probes-numeric/t2464-w2f-unit-shape.case.json`), or the Outcome lists each with Excel, ours, the exact
      value and why the model cannot reach it.
- [ ] On t2464-w2f-unit-shape (594 cells) and t2464-r2-bratio-x29 (3,080 cells), the counts bit-exact, within 4 ulp
      and within 15 digits of Excel do not fall, and no cell moves from within 4 ulp to beyond it. Counts before and
      after are in the Outcome.
- [ ] No new known-failure line in any eval/baseline/*.txt (`make eval-baseline` refuses regressions). Both cases are
      probes-numeric, which has no baseline, so the report.json counts above are the measure.
- [ ] The R4 guards of reviewer-excel-fidelity check 7 (decision 0188), each shown in the Outcome:
      - hold-out: a split stated before scoring, with train and hold-out counts (bit-exact, within 4 ulp, within 15
        digits) for the chosen model and for the correctly rounded value;
      - a fresh sweep after the freeze (a new probes-numeric case at a0 < 1, 0.1 <= x0 < 0.3, b0 1.5 to 50);
      - the Gate A envelope result on that sweep, every point outside it listed;
      - the `FITTED(T-3603, n_cells, range)` tag on each fitted piece, applied only inside its range;
      - the reproducibility floor: no step fits differences of 4 ulp or less;
      - no keep-matched branch (no band tuned to the cells that already match).
- [ ] statistics.md's "Incomplete beta at a first shape a few ulps below 1" entry gets the model's form or the
      negatives with their counts.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The visibly-off cells T-3592 owns in the same cases (unit-shape A211, A407, A409, A411, A421, A423; x29 A1551, A1574,
  A1597, A1609, A1616, A1637, A2327, A2393, A2408). Whichever task lands first scores the other's cells.
- The research itself (T-3568, T-3582) and the other bratio branches (T-3569 to T-3571).
