---
id: T-1287
title: "Probe PRODUCT error order in a 2D array and a no-number array with a scalar"
milestone: M5
lane: functions-math
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
PRODUCT error order over a 2D array and a no-number array combined with a scalar match Excel.

## Context
Left open by the T-1165 fidelity review: probes/product-array-elements only has one-row arrays. Use the `excel-probe` skill.

## Acceptance criteria
- [ ] Oracle values for `PRODUCT({1,#N/A;#DIV/0!,2})` and `PRODUCT({"a"},3)`; engine matches or is fixed.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
