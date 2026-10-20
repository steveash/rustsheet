---
id: T-1915
title: "Legacy N over CHOOSE with an array index: no @ and #VALUE!"
milestone: M2
lane: engine
status: in-progress
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
- [ ] The listed probes/subtotal-take-array-selector cells pass; the baseline shrinks.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
