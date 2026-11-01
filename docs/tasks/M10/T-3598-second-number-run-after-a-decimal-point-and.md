---
id: T-3598
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
- Gap triage 2026-11-01 (`make eval` on origin/main fafb6c6f): the same quirk under the numeral-system digit tags of
  the same probe leaves every row of column C failing (C3 `[$-2000409]` "١٢٣٤٥٦٧.٨٩|٠", ours "١,٢٣٤,٥٦٧.٨٩|٠"; the
  digit set changes, the dropped grouping does not). `scripts/exit_owners.py --milestone M10` read 28 of them as
  unowned, because a hyphen span names only its ends (decision 0136). This task owns the whole column:
  probes/t1925-numeral-digits.case.json S!C1:C32

## Acceptance criteria
- [ ] probes/t1925-numeral-digits S!C1:C32 pass (32 cells, ASCII and numeral-system digits); a probe covers the
      neighbouring forms.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
