---
id: T-1707
title: "CORREL, PEARSON, RSQ: arrays of different sizes are #N/A, and the last digit of their results"
milestone: M9
lane: functions-stat
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
CORREL, PEARSON and RSQ match Excel for arrays of different sizes (`#N/A`, not `#VALUE!`) and in the last digit.

## Context
Found by T-1325: `GROUPBY(Data!A2:A11,Data!D2:D11,_xleta.CORREL)` calls CORREL with each group's values and all
values, which differ in size: Excel gives `#N/A` per group, the engine `#VALUE!` (probes/groupby-pivotby-functions
G!B421:B425, -functions2 G!B281:B285 RSQ, G!B301:B305 PEARSON; PIVOTBY functions2 P!A121). The total row, the same
array twice, is 1.0000000000000002 (CORREL, PEARSON) and 0.9999999999999998 (RSQ) in Excel, exactly 1 in the engine
(G!B426, functions2 G!B286, G!B306): Excel's formula rounds differently. COVARIANCE.P passes. Probe the plain
`CORREL({1;2},{1;2;3})` too.

Also the same last-digit rule (gap triage 2026-10-19): ironcalc-calc/STATISTICAL/CORREL_SLOPE_INTERCEPT_RSQ_STEYX.xlsx
Sheet1!C2 `CORREL(A2:A13,B2:B13)`: Excel (Mac cache, web agrees) -0.13574278487946612, ours -0.1357427848794661.

## Acceptance criteria
- [x] Those cells and ironcalc-calc CORREL_SLOPE_INTERCEPT_RSQ_STEYX.xlsx C2 pass and leave eval/baseline (probes.txt, ironcalc-calc.txt).
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Handoff (resumed)
Not started in code by the sonnet worker (2026-10-19). Two parts: (1) CORREL/PEARSON/RSQ with arrays of different sizes
must be `#N/A` (small, but probe `CORREL({1;2},{1;2;3})` first); (2) the last-digit rule (Excel's total-row
1.0000000000000002 / 0.9999999999999998 and CORREL_SLOPE_INTERCEPT_RSQ_STEYX.xlsx C2) is numeric fitting of Excel's
summation order, which is beyond the sonnet tier (see docs/research/excel-kernel-playbook.md). Next: split (1) into its
own sonnet task if wanted, and fit (2) with the playbook's probe designs.

## Outcome
The `#N/A` for different sizes was already on main (T-1292/T-1620's size check). The last digit: CORREL and PEARSON
are now COVARIANCE.P/(STDEV.P*STDEV.P) and RSQ the square of the sample form, all plain two-pass sums
(`excel_correlation`, pearson.rs), fitted to all 44 numeric oracle cells (statistics.md entry). Eval: probes known
failures 1948 -> 1917 (31 fixed), ironcalc-calc 1057 -> 1056 (C2 fixed); no regressions.
Follow-up: T-2080 (probe inexact constant series and overflow-scale data).
