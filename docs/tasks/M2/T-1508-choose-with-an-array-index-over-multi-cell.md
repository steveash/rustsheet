---
id: T-1508
title: "CHOOSE with an array index over multi-cell range arms lifts to the broadcast grid"
milestone: M2
lane: functions-lookup
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
`CHOOSE({1,2},D1:D2,D3:D4)` gives the grid Excel gives (each index element picks its whole arm, broadcast against the index), so `COUNTBLANK(CHOOSE({1,2},D1:D2,D3:D4))` spills a 2x2 of `#VALUE!` as in Excel.

## Context
probes/countblank-array-values H21, I21 and the RANK grid H35, I35 (excel-graph, 2026-10-13, T-1437) fail: the COUNTBLANK result is 2x2 in
Excel, 1x2 in the engine. The engine's standalone `=CHOOSE({1,2},D1:D2,D3:D4)` (D1=10, D3=30, D4="x") is 10,
`#VALUE!` (1x2); Excel's 2x2 COUNTBLANK result says each element lifts to its 2x1 arm. Probe the standalone shapes
first (`CHOOSE({1,2},D1:D2,D3:D4)`, `CHOOSE({1;2},D1:E1,D3:E3)`, mismatched arm sizes, a scalar arm next to a range arm)
with the excel-probe skill, then fix CHOOSE's array-index path (functions/lookup_and_reference). See
docs/excel-semantics/references.md ("COUNTBLANK of a value that is not a reference is #VALUE!") and criteria.md line
about `CHOOSE({1,2},B1:B6,B1:B6)` in SUMPRODUCT.

## Acceptance criteria
- [x] probes/countblank-array-values H21, I21, H35, I35 pass; the probed standalone shapes pass and are recorded in docs/excel-semantics/lookup.md.
- [x] No eval regressions; `make eval-baseline` records the fix.
- [x] `make check` green; reviewers approve.

## Out of scope
- CHOOSE with a scalar index (reference results are unchanged).

## Outcome
CHOOSE over an index array now broadcasts its value arguments like IF over an array condition: each arm is read whole, the result spans the largest extent of the index and every arm (picked or not), ragged positions are `#N/A`, and an index out of range is `#VALUE!`. IF's branch evaluation became `evaluate_if_arg`, which both functions share. New probe probes/choose-array-index-range-arms (66 cells, excel-graph) passes, and the rule is in lookup.md. Eval: probes known failures 5618 → 5611, with 7 fixed: countblank-array-values H21, I21, H35, I35; array-context-legacy-sumproduct H1, H10 (both M0-M2); external-ref-no-link-ifchoose A32. 0 regressions.
