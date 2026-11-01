---
id: T-1129
title: "Probe and fix the sign of negative numbers that round to or past one unit (TEXT(-0.5,0), TEXT(-0.4,0))"
milestone: M10
lane: functions-text
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
`TEXT` of a negative number keeps or drops its sign as Excel does when the value rounds to or past one unit at the last shown place or rounds to zero.

## Context
T-0952 changed the sign test in `formatter/format.rs` to `<=`, so a value exactly one unit keeps its sign. The sign test still runs on the unrounded value: the T-0952 fidelity review found -0.5 under `0` gives "1", -0.06 under `0.0` gives "0.1", -0.005 under `0.00` gives "0.01", -500 under `#,` gives "1" (Excel should show a "-"), and values that round to zero (-0.4 under `0`, -1 under `0.00,`) are unprobed (Excel may show "-0"). Probe first (excel-probe skill): TEXT(-0.5,"0"), TEXT(-0.4,"0"), TEXT(-0.06,"0.0"), TEXT(-0.005,"0.00"), TEXT(-500,"#,"), TEXT(-1,"0.00,"), TEXT(-1,"0"), TEXT(-0.01,"0.00"). Record in docs/excel-semantics/number-formats.md.

Update (T-2206): rounding now runs before the sign test, so -0.5 under `0` ("-1"), -0.06 under `0.0`
and -0.005 under `0.00` keep their minus; what is still open is values that round to zero and `#,` on -500.

## Acceptance criteria
- [ ] A probe case with Excel values for the cells above passes after the fix, and number-formats.md records the sign rule.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probed 30 cells (probes/text-negative-round-sign, excel-graph). Only literal-only formats without a decimal point were wrong (`TEXT(-0.4,"x")` gave "x", Excel "-x"); fixed in formatter/format.rs. Rule recorded in number-formats.md. Eval delta: new probe case, all 30 cells pass.
