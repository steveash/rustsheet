---
id: T-2947
title: "Probe SORT lifted by_col corners: a sort-invalid call between two that sort, a number by_col beside a boolean"
milestone: M7
lane: functions-lookup
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Two UNVERIFIED corners of SORT's lifted by_col rule (T-2929) are probed and either confirmed or fixed.

## Context
T-2929 pinned which lifted SORT by_col elements keep their call's top-left, are empty or `#VALUE!`
(arrays.md "TOROW, TOCOL, SORT and TEXTJOIN lifted arguments"; `sort_lifted_elements` in functions/spill_functions.rs).
Two corners rest on analogy: (1) a call that fails on its sort_index (past the array) between two calls that sort is
skipped as an invalid by_col ("x") is (probes/t2929-sort-lifted-by-col-errors K12); e.g. `SORT(<3x1>,2,1,{TRUE,FALSE,TRUE})`
and `SORT(<3x1>,2,1,{TRUE;FALSE;TRUE})` (FALSE with sort_index 2 fails on one column); (2) column leads compare by_col as the
booleans they read as, so `{1;TRUE}` counts as equal. Write a probe case (excel-probe skill), fill it, adjust
`sort_lifted_elements` if Excel differs, and update arrays.md.

## Acceptance criteria
- [ ] A probe case covers both corners; arrays.md drops their UNVERIFIED notes or records the rule Excel shows.
- [ ] Any differing cell is fixed and the case passes.
- [ ] `make check` green; reviewers approve.

## Out of scope
- SORTBY, UNIQUE.
