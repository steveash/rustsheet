---
id: T-3538
title: "BETA.DIST cdf last digits on the m9-beta-dist a = 0.5, 5 and 10 grids, and NEGBINOM.DIST(5,5,0.4,TRUE): 21 cells within 15 digits that no task names, where Excel is itself up to 23 ulp off the math (Phase 1b)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-3383]
defer: phase-1b
---

## Goal
The 20 BETA.DIST cumulative cells below and one NEGBINOM.DIST cdf on the same incomplete beta, all already within 15 significant digits of Excel, match Excel to the bit
or within 1 ulp once T-3383's models of Excel's incomplete beta (bpser, bgrat, bfrac and bup arithmetic) are ported.
Otherwise the Outcome records each candidate's bit-exact and within-4-ulp counts per bratio branch (decision 0056
time box).

## Context
- Gap triage 2026-10-31m (`make eval` on origin/main 900fbc1bf, excel-graph). An ownership check over every failing
  Excel cell used `scripts/exit_owners.py`'s `owned` rule over the open task files of origin/main and of every
  `origin/agent/*` branch pushed since 2026-10-29. It found these cumulative cells named by no open task. None is
  visibly off, so this task carries no campaign key (decision 0083). The visibly-off cdf cells of the same grids are
  T-3592's (shapes 5 to 100) and T-3475's on their branches; the density cells of the a = 0.5 grid go to T-2821, and
  those of the a = 5 and 10 grids to T-3392 (this triage adds them there).
- Scored in triage against 50-digit mpmath (`mp.betainc(a, b, 0, x, regularized=True)`), in ulps of Excel's value:
  Excel is not correctly rounded on most of them, so the target is Excel's arithmetic, not the math:
  - a = 0.5: `BETA.DIST(0.5,0.5,0.0001,TRUE)` (A3) Excel 15.9 ulp off the math, ours 16.9; `BETA.DIST(0.9,0.5,0.0001,TRUE)`
    (A5) Excel 13.1, ours 10.1; `BETA.DIST(0.5,0.5,0.5,TRUE)` (A9) Excel 0.5000000000000001 (1.0), ours
    0.4999999999999998 (4.0); `BETA.DIST(0.1,0.5,2,TRUE)` (A19) Excel 5.1, ours 3.1.
  - a = 5: `BETA.DIST(0.1,5,0.0001,TRUE)` (A1) Excel 23.2, ours 22.2; `BETA.DIST(0.1,5,2.5,TRUE)` (A25) Excel
    0.00010296055276246569 (22.3), ours ...559 (15.3), 7 ulp apart.
  - a = 10: `BETA.DIST(0.1,10,0.5,TRUE)` (A7) Excel 17.8, ours 18.8; `BETA.DIST(0.1,10,2,TRUE)` (A19) Excel 17.3,
    ours 15.3; `BETA.DIST(0.1,10,10,TRUE)` (A37) Excel 10.5, ours 12.5.
- The b = 0.0001 cells (A1, A3, A5 of each grid) run the small-b branch (apser or bpser by TOMS 708's dispatch). The
  b <= 1 cells run bpser, and the b > 1 cells bfrac or bgrat. T-3383 owns the models of each. statistics.md
  "Incomplete beta is TOMS 708 `bratio`" and "Incomplete beta: the integrated model"; code in
  crates/rustsheet-engine/src/functions/statistical/incomplete_beta.rs and bratio/.
- Cells (eval/suites/probes-numeric, which has no baseline file: a fixed cell leaves the report's failing list):
- probes-numeric/m9-beta-dist-a0p5.case.json (9): Calc!A1, A3, A5, A7, A9, A11, A19, A25, A31
- probes-numeric/m9-beta-dist-a5.case.json (5): Calc!A1, A3, A9, A19, A25
- probes-numeric/m9-beta-dist-a10.case.json (6): Calc!A5, A7, A9, A19, A37, A49
- probes/stat-argument-modes.case.json (1): Sheet1!H374

`NEGBINOM.DIST(5,5,0.4,TRUE)` (H374) is I_0.4(5, 6): Excel 0.3668967424000001, ours 0.36689674240000014, 1 ulp. It is in
probes, so its line leaves eval/baseline/probes.txt when fixed.

## Acceptance criteria
- [ ] The 21 cells above are bit-exact or within 1 ulp of Excel (stated per cell), or the Outcome lists, per bratio
      branch, each candidate tried with its bit-exact and within-4-ulp counts, and the cells stay named here.
- [ ] Any fitted or ranked piece follows the R4 guards of reviewer-excel-fidelity check 7 (decision 0188): train and
      hold-out counts side by side from a split fixed before scoring (R4.1); a fresh log-uniform sweep oracle-filled
      after the freeze (R4.2) with its Gate A envelope result and every point outside listed; a `FITTED(T-3538,
      n_cells, range)` tag and range on each fitted constant, with accurate math outside the range (R4.3); no step
      fitting differences below the oracle's reproducibility floor (R4.4); no keep-matched branch.
- [ ] No cell of the five m9-beta-dist grids, the t2464 bratio probes or eval/baseline/gen-m9.txt regresses
      (`make eval-baseline` refuses regressions).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The visibly-off cdf cells of the same grids (T-3592, T-3475) and every density cell (T-2821, T-3392).
- The incomplete-beta models themselves: T-3383 builds them; this task ports and scores them on these cells.
