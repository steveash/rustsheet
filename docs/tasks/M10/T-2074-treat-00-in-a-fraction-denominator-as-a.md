---
id: T-2074
title: "Treat 00 in a fraction denominator as a placeholder denominator and fix the ?/100 and grouped-integer cases"
milestone: M10
lane: functions-text
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
`00` in a fraction denominator, `?/100` on tiny negatives and big values, and grouping in the integer part match Excel.

## Context
Probe rows 9, 10, 13, 14: `?/00` is a two-digit placeholder denominator padded to two digits (1.5 "3/02", 1.99 "197/99", 0 "0/01"), we treat it as 20; `?/100` on -0.001 "0/100" (no minus), on 1234.5 #VALUE!; `#,##0 ?/4` on 1234.5 "1,234 2/4" (grouping applies; we print 1234). Evidence in number-formats.md (T-2061).

## Acceptance criteria
- [ ] Every mismatching cell named above passes in probes/fraction-format-features, `make eval-baseline` records the fix.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
