---
id: T-3735
title: "BETA.DIST cdf with a shape within 16 ulp of 1 (t2464 w2f unit-shape): 127 bit-level cells within 15 digits that no task owns (Phase 1b)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-3592]
defer: phase-1b
---

## Goal
The BETA.DIST cdf cells of probes-numeric/t2464-w2f-unit-shape listed below get Excel's double, or within 4 ulp of it.
Each is already within half a unit of Excel's 15th significant digit and more than 0 ulp from Excel's double. Shapes
here are a = 1 ± k ulp or b = 1 ± k ulp (k up to 16), with the other shape 0.5, 2.5 or 5 and x 0.1, 0.5 or 0.9. The 21
cells beyond 4 ulp are the targets. Cells within 4 ulp are fixed only if the same structural model fixes them.

## Context
- Gap triage 2026-11-01 (`make eval` on origin/main 8c8cfd396, excel-graph). The scan applied `scripts/exit_owners.py`'s
  `owned` rule to every failing cell, against the task files of origin/main and the open task-filing branches. 133 of
  the case's 594 cells fail. T-3592 (deferred) owns the 6 visibly-off ones (Calc!A211, A407, A409, A411, A421, A423).
  T-2510 and T-2512 named the case and are done, so no open task owns the other 127. probes-numeric is ungated, so
  these cells are in no baseline.
- All 127 are numeric and none is visibly off: 86 at 1 ulp, 15 at 2, 5 at 3 or 4, 21 beyond 4 (max 18).
  mpmath at 50 digits on the double inputs puts Excel within 2.4e-15 relative of the math on every cell, so this
  is inside Gate A. Decision 0188 R5 makes it Phase 1b, hence `defer: phase-1b`. Excel is the closer of the two on 58
  of the 127.
- The 21 beyond 4 ulp are all at x = 0.1. 20 have a just below 1 with b = 2.5 or 5, and one (A40) has a = 0.5 with
  b = 1 + 3 ulp. Excel there is noisy against k: up to 20 ulp from the math, with the sign flipping between
  neighbouring a. Ours stays within 7 ulp of the math. Examples, Excel vs ours (signed ulps from the math, Excel /
  ours):
  - A213 `BETA.DIST(0.1,0.999999999999999,2.5,TRUE)`: 0.23156652857908366 vs 0.23156652857908414 (-20 / -3)
  - A227 `BETA.DIST(0.1,(67108863*2^-26+67108863*2^-52),2.5,TRUE)` (a = 1 - 2^-52): 0.23156652857908344 vs
    0.23156652857908394 (-17 / +1)
  - A207 `BETA.DIST(0.1,(33554431*2^-25+67108861*2^-51),2.5,TRUE)`: 0.23156652857908466 vs 0.23156652857908427
    (+11 / -3)
  - A425 `BETA.DIST(0.1,(67108863*2^-26+67108863*2^-52),5.0,TRUE)`: 0.40950999999999993 vs 0.40951000000000043
    (-3 / +6)
  - A40 `BETA.DIST(0.1,0.5,(67108864*2^-26+3*2^-52),TRUE)`: 0.316227766016838 vs 0.31622776601683766 (-1 / -7)
- Suspected root cause: Excel's error swings with k, which points at a cancelling formulation on the a0 < 1, b0 > 1
  branch of bratio's dispatch, not at a constant. x0 = 0.1 sits exactly on TOMS 708's `x0 < 0.1` test (bpser when
  `(x0·b0)^a0 <= 0.7`, otherwise bup + bgrat, or bgrat when b0 > 15). So Excel's branch at these points is the first
  thing to settle. Ours still sends a shape of 1 outside the closed forms to `incomplete_beta.rs` (statistics.md
  "Incomplete beta is TOMS 708 `bratio`", "Ours" paragraph; T-2847). Score every failing and passing cell of the
  case against docs/research/t2464/r2-bratio/bratio_excel.py, the line-by-line model, first. A cell the model gets
  and the port does not is a port bug, not a fit. Related: T-3665 (bup + bgrat in [0.29, 0.3)), T-3382 (bgrat's
  accurate increment), T-3035 (density at min(a, b) < 1).
- Code: crates/rustsheet-engine/src/functions/statistical/bratio/ (`mod.rs` `dispatch`, `bup.rs`, `bgrat.rs`,
  `bpser`), `incomplete_beta.rs` `shape_one_tails`.
- Evidence: the case's 594 oracle cells. If the model has to choose between formulations that the three x values
  cannot tell apart, the fresh sweep below is drawn before scoring, not after.
- Cells owned (all BETA.DIST cdf):
  - probes-numeric/t2464-w2f-unit-shape.case.json (127): Calc!A8, A9, A13, A20, A21, A23, A25, A29, A30, A36, A38, A39, A40, A42, A46, A49, A50, A53, A59, A60, A61, A64, A65, A66, A67, A69, A71, A73, A77, A78, A80, A83, A86, A90, A92, A94, A95, A96, A141, A142, A148, A161, A197, A199, A200, A202, A203, A205, A207, A208, A209, A213, A215, A217, A219, A220, A221, A222, A223, A225, A227, A228, A229, A230, A233, A234, A237, A239, A240, A243, A247, A248, A249, A252, A254, A257, A258, A259, A261, A263, A264, A302, A306, A319, A320, A362, A366, A380, A394, A397, A398, A399, A401, A403, A405, A413, A414, A415, A416, A417, A418, A419, A425, A427, A428, A431, A432, A433, A440, A442, A444, A445, A448, A451, A455, A458, A459, A498, A502, A508, A516, A522, A524, A526, A570, A578, A592
  - Beyond 4 ulp (the targets, 21): Calc!A40, A199, A203, A205, A207, A209, A213, A215, A217, A221, A223, A225, A227, A229, A397, A399, A403, A405, A413, A415, A425
- Depends on T-3592, which owns the 6 visibly-off cells of the same case on the same kernel. Re-measure after it lands.

## Acceptance criteria
- [ ] Each of the 21 cells beyond 4 ulp is within 4 ulp of Excel's double. Otherwise the Outcome lists it with Excel,
      new and 50-digit values and the reason, for example that no structural model reproduces Excel's swing.
- [ ] None of the 127 cells, and no other cell of the case or of the t2464-w2f-*, t2464-r2-bratio-* and m9-beta-dist-*
      cases, goes from within 15 digits to visibly off. The bit-exact counts of those cases before and after are in
      the Outcome.
- [ ] No known-failure line is added to any baseline in eval/baseline/ (`make eval-baseline` refuses regressions).
- [ ] Any fitted or newly chosen formulation meets decision 0188 R4 (reviewer-excel-fidelity check 7), with each guard
      shown in the Outcome:
      - hold-out counts (train and hold-out side by side, bit-exact / 4 ulp / 15 digits, the chosen candidate against
        the correctly rounded one), with the margin stated before scoring;
      - a fresh sweep oracle-filled after the freeze (a within ±64 ulp of 1, b from 1.5 to 15, x from 0.05 to 0.3),
        with its counts;
      - the Gate A envelope result on that sweep, with every point outside it listed;
      - each fitted piece tagged `FITTED(T-3735, n_cells, range)` and applied only inside its range;
      - no step fits differences below the reproducibility floor (4 ulp web against desktop,
        docs/research/oracle-calibration.md);
      - no keep-matched branch.
- [ ] statistics.md's bratio entry records which branch Excel takes at x0 = 0.1 with a0 < 1, with the cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- T-3592's 6 visibly-off cells of the same case.
- Shapes away from 1 (T-3665, T-3443, T-3323, T-3035).
