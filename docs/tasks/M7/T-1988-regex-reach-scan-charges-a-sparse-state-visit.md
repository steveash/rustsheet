---
id: T-1988
title: "REGEX* reach scan charges a Sparse state visit by the transitions it walks"
milestone: M7
lane: functions-text
status: done
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
- [x] The formula above is `#ERROR!` within about 1 s in a release build; times before/after in the Outcome.
- [x] The T-1829 formulas (`pike_iteration_is_charged_by_work_done`) still return values and stay fast.
- [x] `make check` green; reviewers approve.

## Out of scope
- Priority-order closure (T-2015).

## Outcome
`Reach::scan`'s byte step now walks a `Sparse` state range by range, as
`SparseTransitions::matches_byte` does, and counts the ranges in a third `Visits` counter charged
`SPARSE_TRANSITION` = 3 units each (functions/text/regex_engine.rs). Release build, time to `#ERROR!`
for `=SUM(--REGEXTEST(REPT("日a ",8000)&SEQUENCE(80),"\b[\w\s\d]{30}q"))`: 1.60 s before, 0.89-1.02 s
after (0.96-1.16 s with `ж` in place of `日`; 2 units gave 1.0-1.15 s and 4 gave 0.78 s). The T-1829
formulas are unchanged: `naïve`×500 → 500 in 1 ms, `café foo bar`×2500 → 27500 in 6 ms, `\b(\w+)\b`
×250 → 4750 in 2 ms; the `\B`×40 chain is still `#ERROR!` after about 1.1-1.25 s (it walks no sparse
state). Related patterns that do not run out (`\b\w{30}q`, `\b\p{L}{30}q`, `\b[^\W]{30}q`, emoji
text) still return values in 0.33-0.49 s. New unit test `scan_charges_the_sparse_transitions_it_walks`;
DEVIATION entry in docs/excel-semantics/text.md updated. Eval delta: none (0 fixes, 0 regressions).
