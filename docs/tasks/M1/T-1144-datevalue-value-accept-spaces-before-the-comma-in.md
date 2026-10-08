---
id: T-1144
title: "DATEVALUE/VALUE accept spaces before the comma in month-day-year text (July 1 , 2026)"
milestone: M1
lane: functions-date
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
DATEVALUE and VALUE read `"July 1 , 2026"` (a space before the comma) as 1 July 2026, as Excel does.

## Context
Found in T-1136: probes/datevalue-separator-spaces B15 and C15 (excel-graph, 2026-10-08) are 46204 and the engine gives `#VALUE!` (baselined). `parse_date` in `crates/rustsheet-engine/src/formatter/date_text.rs` requires the comma right after the day; `"July 1,2026"` and `"July 1 2026"` stay `#VALUE!` (docs/excel-semantics/dates.md). Probe the neighbours first (`"July 1 ,2026"`, `"July 1  ,  2026"`, `"1 July , 2026"`) with the `excel-probe` skill.

## Acceptance criteria
- [ ] A probe records Excel's answer for the neighbouring comma-spacing forms.
- [ ] B15 and C15 of probes/datevalue-separator-spaces pass (baseline shrinks); the dates.md "Unimplemented" note is replaced by the rule.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
