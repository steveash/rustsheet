---
id: T-0162
title: "Probe VALUE of plain clock text (VALUE(10:00))"
milestone: M1
lane: research
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Excel-evidenced result of `VALUE` on plain clock text (`VALUE("10:00")`, `VALUE("6 PM")`), implemented if it differs from today's `#VALUE!`.

## Context
T-0159 made VALUE read date text with a clock suffix. Plain clock text goes through `parse_time_serial`
for number coercion (`0+"10:00"`) but `fn_value` does not call it; no oracle evidence for VALUE itself.
Code: `fn_value` in crates/rustsheet-engine/src/functions/text/common.rs.

## Acceptance criteria
- [x] Probe case with oracle values for VALUE of clock texts; finding in docs/excel-semantics/dates.md.
- [x] `fn_value` matches the oracle; tests added.
- [x] `make check` green; reviewers approve.

## Out of scope
- VALUE percent rounding (T-0929).

## Outcome
Probe `probes/value-clock-text` (excel-graph, 39 texts × VALUE / `0+text` / TIMEVALUE): VALUE of
clock text is the coercion serial of the trimmed text (`VALUE("25:00")` = 1.0416666666666667).
`fn_value` now calls `parse_time_serial` on the trimmed text. Found on the way and fixed in the
shared parser: coercion allows trailing (not leading) spaces; a no-break space is never trimmed;
hour-only AM/PM takes hour 0-12 and `A`/`P` with an optional `.`. Recorded in
docs/excel-semantics/dates.md#time-text. Eval: new case 117/117 cells pass; 0 regressions, 0
baseline changes in every suite (poi included).
