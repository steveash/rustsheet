---
id: T-1960
title: "Probe lifted TAKE/DROP over an array value, and the unprobed M7 lift parameters"
milestone: M7
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel oracle values settle the questions SPEC-0011 "Array-valued functions" and "Elements that are references" leave open, so T-1834 to T-1839 implement evidenced rules.

## Context
From T-1833. The spec reads a lifted TAKE's or DROP's whole element (probes/m7-argument-modes H4:I6, K1-K4) as a reference, the same element OFFSET gives (lookup.md "OFFSET, INDIRECT and TRIMRANGE lift into arrays of references"), and enters the unprobed M7 parameters as `Lift` by the general rule. Probe, with the excel-probe skill, each formula on a sheet with A1:B3 = 1,10;2,20;3,30, as `ROWS`, `COLUMNS` and `INDEX(..,i,1)` of the call unless noted:
1. TAKE and DROP over an array value: `TAKE({1,10;2,20;3,30},{1;2})`, `TAKE(SEQUENCE(3,2),{1;2})`, `DROP({1,10;2,20;3,30},{1;2})`. Is the element the top-left value (as EXPAND's) or the whole sub-array? Also `ROWS(INDEX(..,1,1))`, `COLUMNS(INDEX(..,1,1))`.
2. A lifted TAKE spilled directly: `=TAKE(A1:B3,{1;2})` (each element `#VALUE!` as OFFSET's?), and `SUM(TAKE(A1:B3,{1;2}))`, `TAKE(A1:B3,{1;2})+0`, `ISREF(TAKE(A1:B3,{1;2}))`.
3. A 1x1 array in a lifted parameter: `SEQUENCE({3})`, `TAKE(A1:B3,{2})`: one element, or the scalar call?
4. The unprobed parameters: SEQUENCE columns and step (`SEQUENCE(2,{1;3})`, `SEQUENCE(3,1,1,{1;2})`), MAKEARRAY cols (`MAKEARRAY(2,{1;2},LAMBDA(r,c,r+c))`), EXPAND columns and pad_with (`EXPAND(A1:A2,3,{1;2})`, `EXPAND(A1:A2,3,1,{0;1})`), WRAPROWS pad_with (`WRAPROWS(A1:A3,2,{0;1})`), WRAPCOLS wrap_count over an array literal (`WRAPCOLS(A1:A3,{1;2})`), TOCOL scan_by_column (`TOCOL(A1:B3,0,{FALSE;TRUE})`), UNIQUE exactly_once (`UNIQUE(D1:D3,FALSE,{FALSE;TRUE})` with D1:D3 = x,y,x), TAKE/DROP columns (`TAKE(A1:B3,2,{1;2})`).
5. SORTBY's lifted element: with A1:A3 = 3,1,2 and C1:C3 = 2,1,3, `SORTBY(A1:A3,C1:C3,{1;-1;1})` is {1;0;0} (probes/sort-error-args H97-H99), not the top-left of each sort ({1;2;1}). Probe `INDEX(..,i,1)` and `ROWS`/`COLUMNS` of `INDEX(..,i,1)` for `{1;-1}`, `{-1;1}`, `{1;1}` and `{-1;-1;-1}` on that data and on data where the descending top-left is not 2 or 0, and the later sort_order (`SORTBY(A1:A3,C1:C3,1,A1:A3,{1;-1})`), until a rule gives {1;0;0}.
6. UNIQUE's by_col element: `INDEX(UNIQUE(D1:D3,{FALSE;TRUE}),2,1)` with D1:D3 = x,y,x ("x": it lifts; "y": one element only).

## Acceptance criteria
- [ ] A probe case with excel-graph oracle values answers 1-6; arrays.md "Dynamic-array and LAMBDA function argument modes" records each answer.
- [ ] Where an answer differs from SPEC-0011's choice (the per-parameter list, the reference reading of TAKE/DROP, the 1x1 rule), the spec and the owning task (T-1834 to T-1839) are updated in the same PR.
- [ ] Every new failing cell is in eval/baseline with an owning task.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Any implementation (T-1834 to T-1839).
