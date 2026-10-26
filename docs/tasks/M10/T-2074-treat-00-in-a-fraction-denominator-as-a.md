---
id: T-2074
title: "Treat 00 in a fraction denominator as a placeholder denominator and fix the ?/100 and grouped-integer cases"
milestone: M10
lane: functions-text
status: done
priority: P3
size: S
tier: sonnet
campaign: semantics
depends_on: []
---

## Goal
`00` in a fraction denominator, `?/100` on tiny negatives and big values, and grouping in the integer part match Excel.

## Context
Probe rows 9, 10, 13, 14: `?/00` is a two-digit placeholder denominator padded to two digits (1.5 "3/02", 1.99 "197/99", 0 "0/01"), we already pick the right denominator but pad the zero on the wrong side ("3/20", "0/10" for "3/02", "0/01"); `?/100` on -0.001 "0/100" (no minus), on 1234.5 #VALUE!; `#,##0 ?/4` on 1234.5 "1,234 2/4" (grouping applies; we print 1234). Evidence in number-formats.md (T-2061).

T-0982 fixed the zero padding (`0` pads a denominator on the left): rows 13 and 14 pass except F13. Left (2026-10-20): F9 and F13 (no minus on a tiny negative without an integer section), I9 (`?/100` on 1234.5 is #VALUE!) and I10 (grouping in the integer part).

## Acceptance criteria
- [ ] Every mismatching cell named above passes in probes/fraction-format-features, `make eval-baseline` records the fix.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Visible-parity stage
Decision 0083: a wrong TEXT or displayed-text result is visible to every user of the format, so this task is in the campaign. Its acceptance criteria are already the visible-parity stage.

## Outcome
In formatter/fraction.rs a negative value with no integer part and a zero numerator takes no minus (F9, F13), and `use_thousands` groups the integer digits (I10). I9 already passed. Eval: probes/fraction-format-features F9, F13, I10 fixed; rows 5 and 6 (#VALUE!/huge values) are not this task.
