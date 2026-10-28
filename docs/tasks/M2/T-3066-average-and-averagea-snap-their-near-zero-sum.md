---
id: T-3066
title: "AVERAGE and AVERAGEA snap their near-zero sum's final addition as SUM does"
milestone: M2
lane: functions-stat
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
AVERAGE and AVERAGEA give Excel's 0 where their sum's final addition cancels to within SUM's snap bound, and keep
the residue wherever SUM does.

## Context
- T-3048's probe probes/t3048-snap-conditional-sums (excel-graph, 2026-10-28) shows AVERAGE and AVERAGEA snap their
  sum exactly as SUM does, and then divide: over 0.1, 0.2, -0.3 both are 0 (S!D5, D6, and `*1` of the call, P5, P6);
  over 1+7*2^-52 and -1 both are 0 (G5, G6) while k = 8 keeps 8.881784197001252E-16 (H5, H6); a trailing 0 addend
  stops the snap (E5, E6: 1.3877787807814457E-17, the residue over 4). A text cell after the cancel does not stop it
  in either function (I5, I6 are 0): AVERAGEA counts the text as a 0 in its count but adds no 0 addend, so the snap
  is on the last numeric addition. docs/excel-semantics/numbers.md "Near-zero snapping", "Summing functions".
- AVERAGEIF, AVERAGEIFS, SUMIF, SUMIFS and SUMPRODUCT do not snap (same probe), so the change is AVERAGE's and
  AVERAGEA's only. Code: `fn_average` / `fn_averagea` (functions/statistical/count_and_average.rs); SUM's snap is
  `SumParts` (functions/math_and_trigonometry/sum.rs) and `snap_near_zero` (arithmetic.rs): reuse them, do not copy.
- The probe has range arguments only. AGGREGATE(1) goes through AVERAGE (numbers.md "AGGREGATE"), so a change in
  `fn_average` changes it too: probe AVERAGE/AVERAGEA over scalars and array constants and AGGREGATE(1) over the same
  cancel and k = 7/8 data first (excel-probe skill) and match them. SUBTOTAL(1/101) uses `subtotal_average` and stays.
- SPEC-0009 (docs/specs/SPEC-0009-aggregate-argument-walker.md, the row that says only SUM tracks its last addition)
  must name AVERAGE and AVERAGEA too, in this task's PR.
- Failing cells (known failures in eval/baseline/probes.txt): probes/t3048-snap-conditional-sums.case.json S!D5, S!D6,
  S!G5, S!G6, S!I5, S!I6, S!P5, S!P6.

## Acceptance criteria
- [ ] The scalar, array-constant and AGGREGATE(1) forms are probed and match Excel.
- [ ] The 8 cells above pass; the other 78 cells of the probe still pass (AVERAGEIF(S) keep their residue).
- [ ] No regression in gen-m*/poi/calipers AVERAGE cells (`make eval-check`).
- [ ] `make check` green; reviewers approve.

## Out of scope
- AVERAGEIF, AVERAGEIFS, SUMIF, SUMIFS, SUMPRODUCT (Excel does not snap them).
- SUBTOTAL(1/101) (its own `subtotal_average`; unprobed).
