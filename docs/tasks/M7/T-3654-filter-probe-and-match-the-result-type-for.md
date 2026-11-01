---
id: T-3654
title: "FILTER: probe and match the result type for each kind of include, array element and if_empty"
milestone: M7
lane: functions-lookup
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
FILTER returns values of Excel's type (number, text, boolean, blank-as-0, error, `#CALC!`) for every kind of `include` element, array element and `if_empty` value.

## Context
Lead: the 2026-11-01 holdout themes note (docs/research/holdout-themes-2026-11-01.md). It names a class of input that fails, never a case, and it is not Excel evidence: every behaviour change here needs fresh oracle cells from the excel-probe skill, and the probe cases must not copy the note's wording. If the probes show the engine already matches Excel across the class, record that in the Outcome and close the task.

The holdout themes note reports a few FILTER results of the wrong type. Known rules: arrays.md "FILTER: an array
`if_empty` spills as an array", references.md "FILTER's scalar if_empty is a value", T-3497 (a one-cell error
reference is a 1x1 array). Open: T-3506 (one-cell error reference in an array argument). Not swept for the result
type: blank cells in `array` (0 versus empty text in the spill), empty text, booleans and errors in `array`;
`include` elements that are text, numeric text, booleans, blanks, errors, fractions and negative numbers; an
`if_empty` that is a blank cell, empty text, a reference, an error, a boolean or omitted with a comma; and FILTER
results passed to TYPE, ISTEXT, ISBLANK and COUNTBLANK so the element types are visible. Code:
functions/lookup_and_reference/filter.rs.

## Acceptance criteria
- [ ] A probe case (excel-graph) covers the kinds above, with TYPE/ISBLANK wrappers showing each element's type.
- [ ] The engine matches every probed cell; arrays.md states the type rules.
- [ ] Every quirk found is written up in docs/excel-semantics/arrays.md in the same PR, with its probe cells cited.
- [ ] `make check` green; eval known failures only shrink (baseline updated); reviewers approve.

## Out of scope
- T-3506's one-cell error reference.
