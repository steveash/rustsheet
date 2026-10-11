---
id: T-0962
title: "Probe XLOOKUP row lookup vector with multi-row return range and array return arguments"
milestone: M9
lane: functions-lookup
status: in-progress
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
- [ ] Probe cases in eval/suites/probes with oracle values; lookup.md entry updated.
- [ ] Implementation matches the oracle values.
- [ ] `make check` green; reviewers approve.

Also probe (reviewer findings on T-0948): ISREF and implicit intersection of a wide result, a blank
matched cell in the row (0 vs empty), `UPPER(XLOOKUP(2,A1:A3,B1:C3))` spilling, `XLOOKUP(x,A1:A1,B1:C1)`,
and a wide return range with a different row count.

## Out of scope
- Binary-search/other XLOOKUP modes.
