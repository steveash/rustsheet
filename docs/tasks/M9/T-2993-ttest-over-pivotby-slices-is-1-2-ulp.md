---
id: T-2993
title: "TTEST over PIVOTBY slices is 1-2 ulp off Excel (probes/pivotby-ttest-array2 P!A36, A37, A42, A43)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
`TTEST(Data!C10:D12,<slice of PIVOTBY(..)>,1,2)` in probes/pivotby-ttest-array2 P!A36, A37, A42 and A43 returns Excel's
digits; ours differ in the last 1-2 ulp (A36: Excel 0.030297981348950877, ours 0.030297981348950905).

## Context
Found by T-2854 (excel-graph probe added by its handoff, 2026-10-28). After T-2854 the PIVOTBY grid matches Excel cell for cell
(blank keys are true blanks), so array2 holds Excel's values and the difference is in T.TEST's own arithmetic over these
inputs (type 2, one tail): the summation order or variance formula. Every other TTEST cell of the case passes, so compare
the formula path for these four inputs against the passing ones (`rustsheet-eval show probes/pivotby-ttest-array2.case.json`).

## Acceptance criteria
- [ ] The four known-failure lines leave eval/baseline/probes.txt with no new ones.
- [ ] `make check` green; reviewers approve.

## Out of scope
- PIVOTBY itself (T-2854).
