---
id: T-0162
title: "Probe VALUE of plain clock text (VALUE(10:00))"
milestone: M1
lane: research
status: todo
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
- [ ] Probe case with oracle values for VALUE of clock texts; finding in docs/excel-semantics/dates.md.
- [ ] `fn_value` matches the oracle; tests added.
- [ ] `make check` green; reviewers approve.

## Out of scope
- VALUE percent rounding (T-0929).
