---
id: T-1107
title: "SUM and AVERAGE of number text over 255 characters is #N/A, not #VALUE!"
milestone: M1
lane: functions-math
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`=SUM(text,1)` and `=AVERAGE(text,2)` with number text of 256 characters evaluate to `#N/A` as in Excel (the engine gives `#VALUE!` since T-1101).

## Context
probes/text-number-long-paths A2 and A4 (oracle: excel-graph, 2026-10-07): at LEN 255 SUM and AVERAGE read the text (1), at 256 they give #N/A, while ROUND, ABS, unary minus, percent, NUMBERVALUE and VALUE give #VALUE! and COUNT skips the text. Probe whether other text (non-numeric, 256 characters) also gives #N/A in SUM and AVERAGE direct arguments before choosing the rule. Code: `for_each_value` in functions/statistical/count_and_average.rs and fn_sum in functions/math_and_trigonometry/mathematical.rs.

## Acceptance criteria
- [ ] probes/text-number-long-paths A2 and A4 pass; baseline shrinks; no regressions.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
