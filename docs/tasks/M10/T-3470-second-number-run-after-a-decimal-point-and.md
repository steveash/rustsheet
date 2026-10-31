---
id: T-3470
title: "Second number run after a decimal point and literal prints no grouping"
milestone: M10
lane: functions-text
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`TEXT(1234567.89,"0.00|#,##0")` prints "1234567.89|0" as Excel does.

## Context
Found by T-1925: probes/t1925-numeral-digits C1, C2, C21-C27 (we print "1,234,567.89|0"). Semantics: number-formats.md "Unrelated quirk the probe found". Probe the other forms (`#,##0` before the point, other literals) before generalizing.

## Acceptance criteria
- [ ] C1 and C2 pass; a probe covers the neighbouring forms.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
