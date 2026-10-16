---
id: T-1663
title: "SQRTPI of a multi-cell range is one #VALUE!, not a spill"
milestone: M2
lane: functions-math
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
`SQRTPI(D1:E1)` gives one `#VALUE!` as Excel does, with nothing spilled into B80.

## Context
Found by the T-1660 spill-extent check: probes/radians-sqrtpi-munit A80 `=SQRTPI(D1:E1)` (D1 TRUE, E1 empty) is a single `#VALUE!` in Excel (excel-graph records no B80), but the engine spills `{#VALUE!, 0}`; known failure `Sheet1!B80`. docs/excel-semantics/numbers.md "RADIANS, SQRTPI and MUNIT". Code: `fn_sqrtpi` (functions/math_and_trigonometry/mathematical.rs) lifts a range operand. Check with excel-probe whether every multi-cell range is one `#VALUE!` or only this boolean one, before changing the lift (`SQRTPI({1,TRUE})` does lift, so arrays stay as they are).

## Acceptance criteria
- [ ] probes/radians-sqrtpi-munit Sheet1!B80 passes; baseline shrinks; no regressions.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
