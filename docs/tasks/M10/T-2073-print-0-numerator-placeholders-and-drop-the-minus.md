---
id: T-2073
title: "Print 0 numerator placeholders and drop the minus-space under # and 0 integer in fixed-denominator fractions"
milestone: M10
lane: functions-text
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
`0` numerator placeholders and the minus/separator under `#` and `0` integers match Excel in fixed-denominator fractions.

## Context
Probe rows 8, 12, 15: `# 00/16` on 0 "00/16" (we blank to spaces), `# 0/4` on 0 "0/4", 1 "1 0/4"; the integer and separator drop under `#` when zero (`# 0/4` on 0.5 "2/4", -0.5 "-2/4", `# 00/16` on -0.5 "-08/16"); `0 #/4` on 0.5 "0 2/4", -0.5 "-0 2/4" (we drop the space). Evidence in number-formats.md (T-2061).

## Acceptance criteria
- [ ] Every mismatching cell named above passes in probes/fraction-format-features, `make eval-baseline` records the fix.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
