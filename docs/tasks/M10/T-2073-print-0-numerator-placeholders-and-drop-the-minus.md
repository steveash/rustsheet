---
id: T-2073
title: "Print 0 numerator placeholders and drop the minus-space under # and 0 integer in fixed-denominator fractions"
milestone: M10
lane: functions-text
status: done
priority: P3
size: S
tier: sonnet
campaign: visible
depends_on: []
---

## Goal
`0` numerator placeholders and the minus/separator under `#` and `0` integers match Excel in fixed-denominator fractions.

## Context
Probe rows 8, 12, 15: `# 00/16` on 0 "00/16" (we blank to spaces), `# 0/4` on 0 "0/4", 1 "1 0/4"; the integer and separator drop under `#` when zero (`# 0/4` on 0.5 "2/4", -0.5 "-2/4", `# 00/16` on -0.5 "-08/16"); `0 #/4` on 0.5 "0 2/4", -0.5 "-0 2/4" (we drop the space). Evidence in number-formats.md (T-2061).

## Acceptance criteria
- [x] Every mismatching cell named above passes in probes/fraction-format-features, `make eval-baseline` records the fix.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Visible-parity stage
Decision 0083: a wrong TEXT or displayed-text result is visible to every user of the format, so this task is in the campaign. Its acceptance criteria are already the visible-parity stage.

## Outcome
Done by T-0982 (same PR): the zero numerator under a `0` placeholder, the zero integer and its separator, and the separator under a `0` integer follow the poi/NumberFormatTests rules, and every listed cell in rows 8, 12 and 15 passes.
