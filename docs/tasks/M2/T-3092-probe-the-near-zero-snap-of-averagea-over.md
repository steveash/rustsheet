---
id: T-3092
title: "Probe the near-zero snap of AVERAGEA over array-constant text, FALSE cells and numeric text, and of SUBTOTAL 1 and 101"
milestone: M2
lane: functions-stat
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
AVERAGEA, SUBTOTAL(1) and SUBTOTAL(101) match Excel's near-zero snapping on the forms T-3066 left unprobed, backed by an oracle probe.

## Context
T-3066 made AVERAGE and AVERAGEA snap their sum as SUM does (docs/excel-semantics/numbers.md "Summing functions"). Its
AVERAGEA rule that a text cell or array element counts but adds no 0 addend is evidenced only for a text cell in a range
(probes/t3048-snap-conditional-sums I5, I6). Unprobed: AVERAGEA over a text element of an array constant, a FALSE cell and a
direct numeric text such as "0" after a cancel (0.1, 0.2, -0.3); SUBTOTAL(1/101) (`subtotal_average` in
functions/subtotal.rs does not snap). Write a probe case, fill it with the excel-probe skill, then match Excel in
`fn_averagea` (`for_each_value_a`) and `subtotal_average`.

## Acceptance criteria
- [x] A probe case with an oracle sidecar covers each unprobed form; numbers.md records the result.
- [x] The engine matches every cell of it (or the mismatch is a documented, task-owned known failure).
- [x] `make check` green; reviewers approve.

## Out of scope
- SUBTOTAL(1/101) over a hidden-row or filtered mix beyond the probed forms.

## Outcome
Probed first: probes/t3092-averagea-subtotal-snap (29 cells, excel-graph 2026-10-29) with its oracle sidecar. Excel: AVERAGEA adds no 0
addend for a text or FALSE array element or a FALSE cell (the snap holds), but a direct `"0"`/FALSE, a 0 element or a 0 cell is an addend;
SUBTOTAL(1/101) snap as SUM does. `for_each_value_a` now treats a non-direct FALSE like text; `subtotal_average` sums through `SumParts`.
numbers.md and SPEC-0009 updated. Eval: new case 29/29 cells pass, baseline gains it, 0 regressions; `make check` green.
