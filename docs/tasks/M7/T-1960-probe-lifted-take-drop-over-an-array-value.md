---
id: T-1960
title: "Probe lifted TAKE/DROP over an array value, and the unprobed M7 lift parameters"
milestone: M7
lane: research
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Excel oracle values settle the questions SPEC-0011 "Array-valued functions" and "Elements that are references" leave open, so T-1834 to T-1839 implement evidenced rules.

## Context
From T-1833. The spec reads a lifted TAKE's or DROP's whole element (probes/m7-argument-modes H4:I6, K1-K4) as a reference, the same element OFFSET gives (lookup.md "OFFSET, INDIRECT and TRIMRANGE lift into arrays of references"), and enters the unprobed M7 parameters as `Lift` by the general rule. Probe, with the excel-probe skill, each formula on a sheet with A1:B3 = 1,10;2,20;3,30, as `ROWS`, `COLUMNS` and `INDEX(..,i,1)` of the call unless noted:
1. TAKE and DROP over an array value: `TAKE({1,10;2,20;3,30},{1;2})`, `TAKE(SEQUENCE(3,2),{1;2})`, `DROP({1,10;2,20;3,30},{1;2})`. Is the element the top-left value (as EXPAND's) or the whole sub-array? Also `ROWS(INDEX(..,1,1))`, `COLUMNS(INDEX(..,1,1))`.
2. A lifted TAKE spilled directly: `=TAKE(A1:B3,{1;2})` (each element `#VALUE!` as OFFSET's?), and `SUM(TAKE(A1:B3,{1;2}))`, `TAKE(A1:B3,{1;2})+0`, `ISREF(TAKE(A1:B3,{1;2}))`.
3. A 1x1 array in a lifted parameter: `SEQUENCE({3})`, `TAKE(A1:B3,{2})`: one element, or the scalar call? T-1834 runs the scalar call (`SEQUENCE({3})` spills 3 rows, `SEQUENCE(INDIRECT("C3")*1)` too; the driver's 1x1 rule, SPEC-0011 "As built (T-1834)"); also probe `ROWS(SEQUENCE({3}))` and a lifted element error, `INDEX(SEQUENCE({1;0}),2,1)` (T-1834 gives `#CALC!`).
4. The unprobed parameters: SEQUENCE columns and step (`SEQUENCE(2,{1;3})`, `SEQUENCE(3,1,1,{1;2})`), MAKEARRAY cols (`MAKEARRAY(2,{1;2},LAMBDA(r,c,r+c))`), EXPAND columns and pad_with (`EXPAND(A1:A2,3,{1;2})`, `EXPAND(A1:A2,3,1,{0;1})`), WRAPROWS pad_with (`WRAPROWS(A1:A3,2,{0;1})`), WRAPCOLS wrap_count over an array literal (`WRAPCOLS(A1:A3,{1;2})`), TOCOL scan_by_column (`TOCOL(A1:B3,0,{FALSE;TRUE})`), UNIQUE exactly_once (`UNIQUE(D1:D3,FALSE,{FALSE;TRUE})` with D1:D3 = x,y,x), TAKE/DROP columns (`TAKE(A1:B3,2,{1;2})`).
5. SORTBY's lifted element: with A1:A3 = 3,1,2 and C1:C3 = 2,1,3, `SORTBY(A1:A3,C1:C3,{1;-1;1})` is {1;0;0} (probes/sort-error-args H97-H99), not the top-left of each sort ({1;2;1}). Probe `INDEX(..,i,1)` and `ROWS`/`COLUMNS` of `INDEX(..,i,1)` for `{1;-1}`, `{-1;1}`, `{1;1}` and `{-1;-1;-1}` on that data and on data where the descending top-left is not 2 or 0, and the later sort_order (`SORTBY(A1:A3,C1:C3,1,A1:A3,{1;-1})`), until a rule gives {1;0;0}.
6. UNIQUE's by_col element: `INDEX(UNIQUE(D1:D3,{FALSE;TRUE}),2,1)` with D1:D3 = x,y,x ("x": it lifts; "y": one element only).
7. A lifted TAKE or DROP handed on (T-1838 left these unprobed): `SUMPRODUCT(IF(TRUE,TAKE(A1:B3,{1;2})))`,
   `INDEX(CHOOSE(1,TAKE(A1:B3,{1;2})),1,1)`, `ROWS(OFFSET(TAKE(A1:B3,{1;2}),1,0))`, and an error source with a
   lifted count, `ROWS(TAKE(NA(),{1;2}))` (one `#N/A`, or two?).

## Acceptance criteria
- [x] A probe case with excel-graph oracle values answers 1-7; arrays.md "Dynamic-array and LAMBDA function argument modes" records each answer.
- [x] Where an answer differs from SPEC-0011's choice (the per-parameter list, the reference reading of TAKE/DROP, the 1x1 rule), the spec and the owning task (T-1834 to T-1839) are updated in the same PR.
- [x] Every new failing cell is in eval/baseline with an owning task.
- [x] `make check` green; reviewers approve.

## Out of scope
- Any implementation (T-1834 to T-1839).

## Handoff (resumed)
Handed off by a sonnet run without starting the probes (no cells written, no oracle calls made). Reason:
item 5 asks for a rule fitted to the oracle ("until a rule gives {1;0;0}") and items 1, 3 and 7 decide
whether SPEC-0011's reading stands, which is analysis and a spec decision beyond the sonnet rubric.
Oracle: `rustsheet-oracle status` showed `excel-graph OK` at hand-off time. Suggested order: write one probe
case under eval/suites/probes for items 1-4, 6 and 7 (mechanical `ROWS`/`COLUMNS`/`INDEX(..,i,1)` cells), fill it,
record the answers in arrays.md, then iterate item 5 with extra cells until a rule fits, and update SPEC-0011
and T-1834 to T-1839 where an answer differs.

## Outcome
probes/m7-lift-take-drop-sortby (934 cells, excel-graph, 2026-10-21) answers 1-7; arrays.md "Dynamic-array and LAMBDA
function argument modes" and the new "UNIQUE's and SORTBY's lifted elements" record them, and SPEC-0011 is updated.
1. Over an array value a lifted TAKE or DROP keeps the top-left value (SPEC-0011's reading stands; no nested array).
2. A lifted TAKE over a range behaves as a lifted OFFSET's grid: spilled `#VALUE!`, `+0` `#VALUE!`, SUM 2, ISREF FALSE.
3. A 1x1 array lifts (`SEQUENCE({3})` is the 1x1 1); a single cell does not (`SEQUENCE(A3:A3)` spills 3 rows):
   T-1834's 1x1 rule is too wide (T-2298). `INDEX(SEQUENCE({1;0}),2,1)` is `#CALC!` as built.
4. Every unprobed parameter lifts except EXPAND's pad_with (an array is `#VALUE!`; SPEC-0011 now `Whole`, T-1835).
5. SORTBY: uniform entries give each call's top-left; otherwise element 1 is the first call's top-left and later
   elements are empty ({1;0;0} spilled), or `#VALUE!` with an invalid entry or a later key pair (T-1839).
6. UNIQUE's by_col and exactly_once show the same empty-later-elements rule (T-2300).
7. IF and CHOOSE hand a TAKE grid on, OFFSET over it lifts again, `TAKE(NA(),{1;2})` is two `#N/A` (T-2299).
Eval: one new probe case, 399 of 934 scored cells pass; the failing cells are baselined, owned by T-1835 (26), T-1839 (452),
T-2298 (22), T-2299 (16) and T-2300 (24). Follow-ups: T-2298, T-2299, T-2300.
