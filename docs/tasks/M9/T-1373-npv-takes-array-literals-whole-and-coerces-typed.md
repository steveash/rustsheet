---
id: T-1373
title: "NPV takes array literals whole and coerces typed text and booleans; FVSCHEDULE is #VALUE! for text"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
NPV and FVSCHEDULE treat their non-scalar arguments as Excel does.

## Context
probes/fin-argument-modes rows H368, H370, H371 and H382-H384 (excel-graph, 2026-10-11, T-1084; financial.md "Financial
function argument modes"):
- `NPV(0.1,{-10000,2750,4250,3250,2750})` is 302.233453999042 in Excel, but the engine gives 0.
- `NPV(0.1,-10000,"2750",TRUE,4250)` coerces the typed "2750" and TRUE (Excel -3914.6233180793665); the engine skips them.
- `NPV(0.1,-10000,"x",4250)` is `#VALUE!` in Excel; the engine skips the text.
- FVSCHEDULE is `#VALUE!` for a text cell in the schedule range and for text in a schedule literal; the engine skips the text.
NPV's ranges already skip text and booleans correctly (H369). Check whether the aggregate argument walker the
statistical functions use (`aggregate` mode) already does this, and reuse it.

## Acceptance criteria
- [ ] H368, H370, H371, H382, H383 and H384 of probes/fin-argument-modes pass. The ulp miss on H368's value, if any, is T-0901's.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The ulp misses of NPV's value (T-0901).
