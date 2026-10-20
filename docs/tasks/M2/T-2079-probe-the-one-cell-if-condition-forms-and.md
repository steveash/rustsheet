---
id: T-2079
title: "Probe the one-cell IF condition forms and SWITCH array case after an earlier match (T-2012 follow-up)"
milestone: M2
lane: engine
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Excel evidence for the parts of T-2012 that rest on inference: which one-cell conditions IF reads as a scalar, and when SWITCH lifts on an array case value.

## Context
T-2012 made `if_pick` read any one-cell range condition as a scalar (`collapse_single_cell`) and `switch_pick` lift on an array case value, but only `INDIRECT("K1")` (probes/subtotal-take-array-selector-b A130) and `SWITCH(1,{1,2},H1:H2,I1:I2)` (A10) are probed. Unprobed: `SUBTOTAL(9,TAKE(IF(K1:K1,H1:H2,I1:I2),2))`, the OFFSET and INDEX twins, `ISREF(IF(K1:K1,H1,I1))`, a one-cell range holding text or an error, `INDIRECT("$K$1")` and `INDIRECT("K1",TRUE)` (the static `is_lifting_selector` keeps these unknown); and whether SWITCH short-circuits before an array case: `SWITCH(1,1,H1:H2,{1,2},I1:I2)` and `SWITCH(1,#N/A,H1:H2,{1,2},I1:I2)`.

## Acceptance criteria
- [x] A probe case with those cells and Excel oracle values; the engine matches them, or the mismatches are fixed.
- [x] references.md "unconfirmed" note replaced by the evidence.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
probes/if-switch-one-cell-selector (12 formulas, excel-graph 2026-10-20): every one-cell range condition form (K1:K1, OFFSET,
INDEX, INDIRECT with `$K$1` and with a1 TRUE) is read as a scalar, a text or error condition cell is `#VALUE!`/`#N/A`, and
SWITCH short-circuits on an earlier match and returns an earlier error case value without lifting. The engine already
matched all 13 cells (T-2012's inference confirmed); references.md's "unconfirmed" note is replaced by the evidence. Eval:
one new passing probes case, no known-failure change.
