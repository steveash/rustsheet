---
id: T-2929
title: "SORT lifted by_col: TRUE element past the first column with sort_index 1 is #VALUE! in Excel"
milestone: M7
lane: functions-lookup
status: in-progress
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
- [ ] Follow-up probes pin the rule; arrays.md records it.
- [ ] The four failing T-2366 probe cells pass and the baseline shrinks.
- [ ] `make check` green; reviewers approve.

## Out of scope
- SORTBY and TEXTJOIN.
