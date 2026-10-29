---
id: T-3050
title: "Probe TEXT of a large positive number under a lone calendar prefix (TEXT(3000000,'b1'))"
milestone: M5
lane: functions-text
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`TEXT(3000000,"b1")` (a positive value past the maximum date serial under a calendar-prefix-only section) matches Excel.

## Context
T-2388 showed `TEXT(-3000000,"b1")` is "-" (probes/t2388-b1-negative A13). The positive case is unprobed and `format_number` (formatter/format.rs) still refuses it with #VALUE! through the maximum-serial check.

## Acceptance criteria
- [ ] Probe `"b1"`, `"b1 x"` and `"b2"` on 3000000 (excel-graph); record in number-formats.md; make our output match, baseline updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
