---
id: T-3092
title: "Probe the near-zero snap of AVERAGEA over array-constant text, FALSE cells and numeric text, and of SUBTOTAL 1 and 101"
milestone: M2
lane: functions-stat
status: in-progress
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
- [ ] A probe case with an oracle sidecar covers each unprobed form; numbers.md records the result.
- [ ] The engine matches every cell of it (or the mismatch is a documented, task-owned known failure).
- [ ] `make check` green; reviewers approve.

## Out of scope
-
