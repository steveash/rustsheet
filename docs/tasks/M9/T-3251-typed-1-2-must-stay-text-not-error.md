---
id: T-3251
title: "Typed \"- 1/2%\" must stay text, not #ERROR!"
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
probes/typed-mixed-fraction-number-format A15/B15 (excel-graph): Excel keeps the text `-$3 1/2%` (`=A15+0` is `#VALUE!`); ours shows `#ERROR!`, so the typed text starting with `-` seems to reach the formula path. Found by T-3237; `mixed_fraction` already rejects the input.

## Acceptance criteria
- [ ] A15/B15 of that probe pass (TF/F lines leave eval/baseline/probes.txt).
- [ ] `make check` green.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
