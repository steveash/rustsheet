---
id: T-2366
title: "Probe SORT's lifted by_col elements: each call's top-left or only the first (SORTBY's rule)"
milestone: M7
lane: functions-lookup
status: done
priority: P2
size: S
tier: sonnet
depends_on: [T-2319]
---

## Goal
Excel evidence decides what SORT's lifted by_col elements hold when they are not all errors, and ours matches it.

## Context
T-2319 lifted SORT's by_col (lifting-table entry SORT in lift/kinds/general.rs, `CallResult::Array`: each element is its
call's top-left). Its only evidence, gen-m7/7307132bb8820f17 A1076, is an all-`#VALUE!` 2x4 grid, which cannot tell
`Array` from `ArrayFirst` (SORTBY's and UNIQUE's rule: only the first non-error element keeps its top-left, the rest are
empty; arrays.md "UNIQUE's and SORTBY's lifted elements", probes/sort-error-args H97-H99). arrays.md "TOROW, TOCOL, SORT and
TEXTJOIN lifted arguments (gen-m7)" marks it UNVERIFIED; so are the hand-written expectations in test/test_fn_sort_lift.rs.
Probe (excel-probe skill), with A1:A3 = 3,1,2:
- `SORT(A1:A3,1,1,{FALSE,TRUE})` spilled, and `ISBLANK(INDEX(..,1,2))`, `INDEX(..,1,2)&"|"`
- `SORT(A1:A3,1,1,{TRUE,TRUE})` (uniform entries) and `SORT(A1:A3,3,1,{FALSE,TRUE})` (an error first element)
- `ROWS(SORT(A1:A3,1,1,E1:E2))` with E1:E2 = FALSE,TRUE (a range by_col)
Then set the entry to `.array()` or `.array_first()` and fix the tests and arrays.md.

## Acceptance criteria
- [ ] The probe case lands with oracle values; arrays.md's SORT clause is VERIFIED or corrected.
- [ ] Every probed cell passes, or its known failure is owned by a follow-up task.
- [ ] `make check` green; reviewers approve.

## Out of scope
- SORTBY (T-1839, T-2310) and TEXTJOIN (T-2318).

## Outcome
Probe `probes/t2366-sort-lifted-by-col` filled from excel-graph. SORT's lifted by_col elements are never empty
(not SORTBY's `ArrayFirst`; `.array()` kept); arrays.md and the tests cite it. Four probe cells fail (a TRUE by_col element
past the first column with sort_index 1 is `#VALUE!` in Excel): baselined, owned by T-2922. Eval delta: 8 of 12 probe cells pass; 4 new known failures.
