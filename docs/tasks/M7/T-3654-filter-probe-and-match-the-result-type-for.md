---
id: T-3654
title: "FILTER: probe and match the result type for each kind of include, array element and if_empty"
milestone: M7
lane: functions-lookup
status: done
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
- [x] A probe case (excel-graph) covers the kinds above, with TYPE/ISBLANK wrappers showing each element's type.
- [x] The engine matches every probed cell; arrays.md states the type rules.
- [x] Every quirk found is written up in docs/excel-semantics/arrays.md in the same PR, with its probe cells cited.
- [x] `make check` green; eval known failures only shrink (baseline updated); reviewers approve.

## Out of scope
- T-3506's one-cell error reference.

## Outcome
Probe `eval/suites/probes/cases/t3654-filter-result-types` (excel-graph, 2026-11-01; 211 cells): array element kinds
spilled and seen through TYPE/ISBLANK/ISTEXT/MAP, 23 include shapes (cells and constants), 11 if_empty kinds,
lone blank cells. Three engine gaps fixed in `filter.rs`:
- include text that spells TRUE/FALSE in any case is that boolean (was `#VALUE!`); other text stays `#VALUE!`.
- a lone blank cell as `array` stays a blank element (`ISBLANK(FILTER(A2,TRUE))` TRUE; was 0).
- a blank one-cell `if_empty` is 0 (ROWS 1, `&"|"` "0|"; was blank / `#VALUE!`).
Everything else already matched. Written up in arrays.md "FILTER: the type of each include element and of the
result". Eval: probes new case passes 211/211; no other suite changed.
