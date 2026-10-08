---
id: T-0267
title: "Clock-text H:M.f:S fraction must trim only ASCII spaces"
milestone: M2
lane: functions-date
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`H:M.f:S` clock text rejects a fraction with Unicode whitespace like any other clock text (#VALUE!).

## Context
From the T-0253 review: `parse_clock_text` uses `fraction.trim()` (Unicode whitespace) so `"10:11. 5"` with a no-break or ideographic space parses, while dates.md says those are #VALUE! (rule from probes/value-clock-text rows 38, 42-45; interior-fraction cases are inferred, unprobed). Use `trim_spaces`.

## Acceptance criteria
- [ ] A unit test or probe shows the Unicode-space cases are #VALUE!.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
`parse_clock_text` now trims the `H:M.f:S` fraction with `trim_spaces`, so a no-break or ideographic space there is #VALUE! (rule from probes/value-clock-text rows 38, 42-45; interior-fraction cases are inferred, unprobed). Unit cases added to `value_and_coercion_of_clock_text`. No eval delta (no oracle case covered it).
