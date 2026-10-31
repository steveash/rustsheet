---
id: T-3254
title: "T.TEST and TTEST last digits on the t1048 probe and the stat-test-lift INDEX cells, plus four gen-m9 Student t cells (40 cells within 15 digits no task names)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-2930]
defer: phase-1b
---

## Goal
The 40 Student t cells listed below (T.TEST and TTEST on probes-numeric/t1048-ttest and probes/stat-test-lift, and
TDIST, T.DIST.RT, TINV and T.INV.2T on gen-m9), all already within 15 significant digits of Excel, match Excel to the
bit, or the Outcome records the candidate formulations with their bit-exact counts (decision 0056 time box).

## Context
- Gap triage 2026-10-30 (`make eval` on origin/main 6a26b992a, excel-graph). An ownership check over every failing cell
  (`scripts/exit_owners.py`'s `owned` rule over the task files of origin/main and of every pushed branch) found these
  cells named by no open task. Neighbouring owners, none of which lists them: T-0969 (blocked; the accurate Student t
  cdf and its 55 known failures), T-2930 (T.TEST's tail and the T bpser branches through the ported kernel), T-2993
  (TTEST over PIVOTBY slices).
- 34 cells are within 4 ulp and 6 further; none is visibly off, so this is Phase 1b work and stays deferred. It depends
  on T-2930 because its tail and bpser changes move these cells: re-run the list after it before any fit.
- Suspected root cause: T.TEST is a t statistic and degrees of freedom (Welch's for type 3) fed to the two-tailed or
  one-tailed t distribution. The residue sits in the statistic's summation order or in the tail (statistics.md "T.TEST
  rounds y = df/(df+t^2) to a double before the tail"). The t1048 pairs are T.TEST and TTEST on the same arguments
  (columns A and B) and agree with each other, so one fix moves both (B6 and B8 fail with A6 and A8; T-0969's tally
  names them). Type 3 (A15, A18, A27, A35, A36: up to
  4 ulp) misses more than types 1 and 2, which points at the Welch df.
- The stat-test-lift cells take one element of a lifted T.TEST (`INDEX(T.TEST(A1:A8,B1:B8,{1;2},{2;3}),2)`), so they
  are the same scalar T.TEST: check the scalar call at those arguments before the lift.
- Code: `crates/rustsheet-engine/src/functions/statistical/student_t.rs`, `t_dist.rs`.

Examples, Excel vs ours (`make eval-show CASE=...`):
- probes-numeric/t1048-ttest Calc!A15 `=TTEST(D1:D10,F1:F10,1,3)`: 0.2637857766130388 vs 0.2637857766130386
- probes-numeric/t1048-ttest Calc!A43 `=TTEST({1,2,3,4},{2,3,5,9},1,1)`: 0.04893171742678492 vs 0.0489317174267849
- probes/stat-test-lift Sheet1!P65 `=INDEX(T.TEST(A1:A8,B1:B8,1.7,{2;2.9}),2)`: 0.1614920379734857 vs 0.16149203797348596
- gen-m9/d910f000b60e2eac Calc!A357 `=T.INV.2T(SUM(Data!D7:D7,Data!D2:D6),8)`: -0.12957674513356338 vs
  -0.12957674513356363
- gen-m9/44a8841352dc48e9 Calc!A310 `=TDIST(1.5,7,2)`: 0.17729848698997008 vs 0.17729848698997006

Owned cells:
- probes-numeric/t1048-ttest.case.json (32): Calc!A6, A8, A9, A11, A12, A13, A14, A15, A16, A17, A18, A27, A34, A35, A36, A42, A43, B9, B11, B12, B13, B14, B15, B16, B17, B18, B27, B34, B35, B36, B42, B43
- probes/stat-test-lift.case.json (4): Sheet1!P64, P65, P66, P67
- gen-m9/44a8841352dc48e9.case.json (1): Calc!A310
- gen-m9/a612e89321a6df9c.case.json (1): Calc!A191
- gen-m9/d37313406b9f6e55.case.json (1): Calc!A285
- gen-m9/d910f000b60e2eac.case.json (1): Calc!A357

## Acceptance criteria
- [ ] The owned cells match Excel to the bit in `make eval`, and their gated lines (stat-test-lift in
      eval/baseline/probes.txt, the four gen-m9 cells in eval/baseline/gen-m9.txt) leave the baseline with
      `make eval-baseline`, with no new known-failure line anywhere; or the Outcome lists each candidate with its
      bit-exact count over these cells and the cells still open move to a follow-up.
- [ ] docs/excel-semantics/statistics.md records the T.TEST formulation that matches (or the ones ruled out), citing
      t1048-ttest A15 and A43.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Visibly-off Student t cells (T-0969, T-2930); TTEST over PIVOTBY slices (T-2993).

## Deferred (decision 0114)
`defer: phase-1b`: the cells this task owns are already within 15 significant digits of Excel, so it waits until Phase 1
exits. Remove the key only when the steward records the Phase 1 exit, or when an eval shows one of its cells visibly off
(then give it a `campaign:` stage instead).
