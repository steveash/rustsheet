---
id: T-0962
title: "Probe XLOOKUP row lookup vector with multi-row return range and array return arguments"
milestone: M9
lane: functions-lookup
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Excel's behavior for XLOOKUP with a row lookup vector and a multi-row return range (returns the
matched column?), and for array (non-range) return arguments, is established by oracle probes and implemented.

## Context
T-0948 only implements the column-vector case (poi/xlookup.xlsx Sheet1!C2); the horizontal analog
keeps the same-shape rule (`#VALUE!`) without evidence. Code: `fn_xlookup`, `xlookup_pick` in
`crates/rustsheet-engine/src/functions/xlookup.rs`. Use the `excel-probe` skill.

## Acceptance criteria
- [x] Probe cases in eval/suites/probes with oracle values; lookup.md entry updated.
- [x] Implementation matches the oracle values.
- [x] `make check` green; reviewers approve.

Also probe (reviewer findings on T-0948): ISREF and implicit intersection of a wide result, a blank
matched cell in the row (0 vs empty), `UPPER(XLOOKUP(2,A1:A3,B1:C3))` spilling, `XLOOKUP(x,A1:A1,B1:C1)`,
and a wide return range with a different row count.

## Out of scope
- Binary-search/other XLOOKUP modes.

## Outcome
New probe probes/xlookup-row-column-picks (22 formulas, 37 scored cells, excel-graph) answers every question here, and all of them now pass. Engine changes (functions/xlookup.rs):
- One shape rule for range and array lookup vectors (`xlookup_orientation`): a one-cell vector takes its orientation from the return's shape.
- A range lookup accepts array return arguments.
- An array row lookup accepts a taller array return and gives the matched column (`xlookup_pick_array`).
- `@XLOOKUP(...)` intersects the picked row or column instead of taking its first value (`evaluate_at_operand_reference`).

Recorded in lookup.md "XLOOKUP: return_array may be wider than the lookup vector". Eval delta: +1 case fully passing (37 cells), no regressions.
