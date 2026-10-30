---
id: T-3251
title: "Typed minus-dollar mixed fraction with percent must stay text, not #ERROR!"
milestone: M9
lane: functions-text
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Typed `-$3 1/2%` stays a text cell, as in Excel, instead of evaluating to `#ERROR!`.

## Context
probes/typed-mixed-fraction-number-format A15/B15 (excel-graph): Excel keeps the text `-$3 1/2%` (`=A15+0` is `#VALUE!`); ours shows `#ERROR!`, so the typed text starting with `-` seems to reach the formula path. Found by T-3237; `mixed_fraction` already rejects the input. Excel still gives that cell the number format `0.00%` (A15 ENTERED numberFormat), so the fix sets the format too.

## Acceptance criteria
- [ ] A15/B15 of that probe pass (TF/F lines leave eval/baseline/probes.txt).
- [ ] `make check` green; reviewers approve.

## Out of scope
-
