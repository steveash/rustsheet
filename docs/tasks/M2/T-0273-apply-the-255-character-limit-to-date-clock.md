---
id: T-0273
title: "Apply the 255-character limit to date-clock text (VALUE and coercion)"
milestone: M2
lane: functions-date
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Date-clock text longer than 255 characters is #VALUE! in VALUE and `0+text`, as in Excel.

## Context
T-0266 probe `probes/date-clock-text-length-limit` (excel-graph): 255 characters parse, 256+ are
#VALUE! (fraction digits, padding, 301-digit `H:M.f:S` fraction). Leading spaces count toward the limit in VALUE
(`0+text` already rejects a leading space at any length). The engine's `parse_date_clock_serial`
(crates/rustsheet-engine/src/functions/date_and_time/time_text.rs) applies no limit; see
docs/excel-semantics/dates.md ("Clock text of more than 255 characters").

## Acceptance criteria
- [ ] Every cell of probes/date-clock-text-length-limit passes; baseline entries removed via `make eval-baseline`.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Number text and date-only text limits (unprobed).
