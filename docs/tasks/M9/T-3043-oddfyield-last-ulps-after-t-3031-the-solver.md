---
id: T-3043
title: "ODDFYIELD last ulps after T-3031: the solver sweep and the date-system odd-bond probes (263 cells within 15 digits)"
milestone: M9
lane: functions-financial
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-3031]
defer: phase-1b
---

## Goal
ODDFYIELD matches Excel bit for bit on the sweep and date-system cells that T-2007's fitted Newton leaves 1 to a few
ulp off. If no slope, price or stop form does, the task records the forms tried with their counts (decision 0056).

## Context
Gap triage 2026-10-28 (`make eval` on origin/main 4b20263be). T-2007 fitted ODDFYIELD as YIELD's Newton with the odd
first coupon weighed as a regular one in the slope (financial.md "ODDFYIELD stops short of ODDFPRICE's root").
T-3031 owns that entry's visibly-off cells: the long first periods, and the sweep's near-zero-yield bond and
inversions that it lists. Nobody owns the bit-level rest. Every cell below is NumericUlp, or NumericClose within 15
digits, and none is visibly off. All are known-failure F lines in eval/baseline/probes.txt.
- probes/oddfyield-solver-sweep (201 cells, excluding every cell T-3031 lists):
  - the ODDFYIELD columns, S!A1, A2, A3, A8, A10, A11, A13, A16, A17, A18, A20, A21, A23, A24, A25, A26, A30, A31,
    A34, A35, A36, A37, A38, A41, A46, A47, A48, A53, A58, A61, A62, A63, A64, A65, A66, A68, A69, A70, A71, A72, A74,
    A84, A85, A86, A87, A88, A91, A92, A93, A94, A95, A97, A98, A99, A100, A101;
  - B2, B3, B5, B6, B8, B9, B10, B11, B12, B16, B17, B18, B19, B20, B21, B22, B23, B24, B25, B26, B27, B28, B30, B32,
    B36, B37, B38, B39, B41, B42, B44, B45, B46, B47, B48, B49, B50, B55, B59, B62, B63, B64, B67, B71, B72, B74, B75,
    B76, B78, B79, B80, B83, B85, B87, B89, B91, B92, B97, B99, B100;
  - C1, C5, C9, C12, C19, C20, C27, C29, C30, C31, C40, C55, C60, C61, C66, C68, C74, C78, C82, C83, C86, C87, C90,
    C95, C97, C98, C100;
  - the ODDFPRICE inversions, E2, E3, E4, E5, E6, E7, E11, E12, E13, E14, E16, E17, E19, E21, E22, E25, E26, E27,
    E28, E34, E36, E37, E42, E44, E47, E48, E49, E50, E51, E55, E60;
  - F2, F3, F4, F5, F6, F7, F8, F10, F14, F16, F23, F25, F29, F30, F31, F39, F40, F41, F42, F44, F47, F48, F55,
    F56, F57, F59, F61.
  Example (`make eval-show CASE=probes/oddfyield-solver-sweep.case.json`): S!E2, the ODDFYIELD of an ODDFPRICE at a
  yield of 0.005, is Excel 0.005000000002369702 and ours 0.0050000000023697.
- probes/date-system-1900-odd-years S!A3, B3, C3, D3, E3, F3, G3, H3, I3, J3, K3, L3, M3, N3, O3, P3, Q3, R3, S3 (19).
  These are one bond in different years. For example A3
  `ODDFYIELD(DATE(2095,2,1),DATE(2096,6,30),DATE(2095,1,1),DATE(2095,6,30),0.05,98,100,2,0)`: Excel
  0.06499377053016654, ours ...656.
- probes/date-system-1900-odd-threshold S!A5, B5, C5, D5, E5, F5, G5, H5, I5, J5, K5, L5, M5 (13), with the same value.
- probes/date-system-1904-odd-threshold S!C5, D5, E5, F5, G5, H5, I5, J5, K5, L5, M5 (11).
- probes/date-system-1904-odd-years S!C3, E3, F3, J3, L3, N3, S3 (7).
- probes/date-system-1900-odd-accrint-bound S!A3, A4, B3, B4, B6, C3, C4, D3, D4, D6 (10). For example A3 is Excel
  0.05396550972225409, ours ...408.
- probes/date-system-1904-financial S!A32 and probes/odd-bond-argument-check-order Calc!A258 (1 each).
- Evidence and the fit's history are in financial.md "ODDFYIELD stops short of ODDFPRICE's root" (the residuals,
  both sides of the root, the 1,920 rejected variants). The 43 date-system twins are the same bond at one value,
  so they move together. Expect one formulation change to settle them as a block or not at all.
- Engine: `fn_oddfyield` → `odd_first_yield` in
  `crates/rustsheet-engine/src/functions/financial/financial_bonds/yield_solver.rs` (`excel_yield_newton`,
  `OddFirstSchedule::dprice`). Lab: T-3031 commits an `oddfyield` kernel to fitlab/kernels/solvers.py. Score
  candidates with it.
- T-3031 changes the same slope, so this task starts from its result.

## Acceptance criteria
- [ ] A formulation that beats the shipped model's bit-exact count on the 201 sweep cells and the 62 date-system and
      check-order cells, losing none that pass today. The F lines it fixes leave eval/baseline/probes.txt, with no
      new F line in any baseline. Otherwise the task is blocked time-boxed (decision 0056), with the forms and counts
      recorded in financial.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
- T-3031's visibly-off cells and its `#NUM!` probe; ODDFPRICE's own digits (T-1374); YIELD (T-2788).

## Deferred (decision 0114)
`defer: phase-1b`: every cell this task owns is already within 15 significant digits of Excel, so it waits until
Phase 1 exits. Remove the key only when the steward records the Phase 1 exit, or when an eval shows one of its cells
visibly off (then give it a `campaign:` stage instead).
