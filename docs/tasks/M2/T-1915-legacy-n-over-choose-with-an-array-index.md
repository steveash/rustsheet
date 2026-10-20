---
id: T-1915
title: "Legacy N over CHOOSE with an array index: no @ and #VALUE!"
milestone: M2
lane: engine
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
A legacy `=N(CHOOSE({1,2},H1:H2,I1:I2))` shows no `@` and is #VALUE! on the arms' row (D1; off their rows already matches).

## Context
probes/subtotal-take-array-selector D1, D3, D7 (D5 passes; ours 3 and `=N(@CHOOSE(@{1,2},...))`). ABS and LEN match. Relates to the legacy N/T marker (T-1713, T-1851) and `is_choose_index` in static_analysis/legacy.rs.

## Acceptance criteria
- [x] The listed probes/subtotal-take-array-selector cells pass; the baseline shrinks.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
N over a CHOOSE with an array-constant index over plain references now leaves the call whole (no `@`) and reads `#VALUE!` (`chooses_references_by_an_array_index`). probes/subtotal-take-array-selector D1, D3, D7 pass; baseline shrank by those 3 cells.
