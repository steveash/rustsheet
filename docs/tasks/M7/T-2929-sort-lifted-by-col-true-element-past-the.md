---
id: T-2929
title: "SORT lifted by_col: TRUE element past the first column with sort_index 1 is #VALUE! in Excel"
milestone: M7
lane: functions-lookup
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Excel's lifted SORT by_col elements past the first column match ours, or the rule behind the divergence is probed and recorded.

## Context
T-2366's probe (probes/t2366-sort-lifted-by-col, A1:A3 = 3,1,2) shows Excel returns `#VALUE!` for the second (TRUE) element of
`SORT(A1:A3,1,1,{FALSE,TRUE})` (I1, H6), `{TRUE,TRUE}` (I7) and a range by_col E1:E2 (H18), where ours returns 3. With sort_index 3
(`{FALSE,TRUE}`, I11) the second element is 3 as ours. A TRUE first element with sort_index 1 is 3 (H7). Hypotheses (unprobed):
the error depends on the element's column index versus sort_index. Probe more combinations (excel-probe skill), then implement.
Code: `SORT` in functions/lift/kinds/general.rs; arrays.md "TOROW, TOCOL, SORT and TEXTJOIN lifted arguments".

## Acceptance criteria
- [x] Follow-up probes pin the rule; arrays.md records it.
- [x] The four failing T-2366 probe cells pass and the baseline shrinks.
- [x] `make check` green; reviewers approve.

## Out of scope
- SORTBY and TEXTJOIN.

## Outcome
Five probe cases (probes/t2929-sort-lifted-by-col-index, -shapes, -errors, -range, -column; excel-graph) pinned the rule:
failed calls keep their error and are skipped; each row's first sorting call leads it, and the leads keep their call's
top-left when every lead's by_col is equal (else only the first); every other sorting element is empty when sort_index is
the single key 2 and it is the first broken element or sits below an empty-able broken one with the same by_col, else
`#VALUE!`. `sort_lifted_elements` (spill_functions.rs) applies it after the lift; arrays.md records it. The four failing
T-2366 cells pass, and the new cases pass in full (379 cells). Unprobed: a sort-invalid call between two that sort
(by analogy with an invalid by_col) and a number by_col beside a boolean in a column.
