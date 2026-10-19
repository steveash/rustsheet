---
id: T-1988
title: "REGEX* reach scan charges a Sparse state visit by the transitions it walks"
milestone: M7
lane: functions-text
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-1829]
---

## Goal
A REGEX* pattern made of large Unicode classes on the `PikeVM` fallback runs out of work in about 1 s (the cap in
docs/excel-semantics/text.md "Lifted TEXTSPLIT, TEXTBEFORE, TEXTAFTER and REGEX* calls are capped"), as look-around
chains do since T-1829.

## Context
T-1829 performance review, round 2 (MINOR). `Reach::scan` (functions/text/regex_reach.rs) charges every state visit a
flat 21 units, but `step` on a `Sparse` state walks its transitions (`matches_byte`), as the `PikeVM` does, so a
big class costs more per visit than the calibration assumed. Release, time to `#ERROR!`:
`=SUM(--REGEXTEST(REPT("日a ",8000)&SEQUENCE(80),"\b[\w\s\d]{30}q"))` 1.57-1.86 s (against 0.91 s for the `\B` chain the
look charge was calibrated on). Count the transitions `step` walks in a third `Visits` counter with a small
per-transition cost, calibrated in release, or recalibrate `SCAN_VISIT` on this pattern (the rule is "the worst is
charged").

## Acceptance criteria
- [ ] The formula above is `#ERROR!` within about 1 s in a release build; times before/after in the Outcome.
- [ ] The T-1829 formulas (`pike_iteration_is_charged_by_work_done`) still return values and stay fast.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Priority-order closure (T-1985).
