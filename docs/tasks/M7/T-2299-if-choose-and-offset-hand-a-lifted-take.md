---
id: T-2299
title: "IF, CHOOSE and OFFSET hand a lifted TAKE or DROP grid on, and TAKE over an error source lifts"
milestone: M7
lane: functions-lookup
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: [T-1960]
spec: docs/specs/SPEC-0011-scalar-argument-lifting.md
---

## Goal
A lifted TAKE's or DROP's array of references is handed on by IF and CHOOSE in every consumer (SUMPRODUCT too), lifts again as OFFSET's reference argument, and TAKE or DROP over an error source with a lifted count is one error per element, as in Excel.

## Context
From T-1960. T-1838 joined TAKE and DROP to `reference_producer` but left `may_hand_on_grid` (functions/lookup_and_reference/ref_lift.rs) unprobed for them. Excel (SPEC-0014 rules 3-7 hold for TAKE's grid as for OFFSET's): `INDEX(IF(TRUE,TAKE(A1:B3,{1;2})),1,1)` and the CHOOSE form are the reference A1:B1 (ISREF TRUE, SUM 11; element 2 is A1:B2, SUM 33; these pass today), `SUM(IF(TRUE,TAKE(..)))` is 2 (top-left aggregate, passes), but `SUMPRODUCT(IF(TRUE,TAKE(A1:B3,{1;2})))` and `SUMPRODUCT(CHOOSE(1,..))` are 0 (ours `#VALUE!`; `SUMPRODUCT(TAKE(..))` alone is 0 and passes). `OFFSET(TAKE(A1:B3,{1;2}),1,0)` lifts again: ROWS 2, elements A2:B2 (SUM 22) and A2:B3 (SUM 55) (ours `#VALUE!`). `TAKE(NA(),{1;2})` is two `#N/A` elements (ROWS 2; ours one `#N/A`): an error source does not short-circuit a lifted count.

Owned cells, probes/m7-lift-take-drop-sortby (T-1960, excel-graph, 2026-10-21; Sheet1 A1:B3 = 1,10;2,20;3,30, D1:D3 = x,y,x; column E holds each row's formula as text, F:S its ROWS, COLUMNS and per element INDEX(INDEX(..,i,1),1,1), ROWS, COLUMNS and SUM of INDEX(..,i,1); arrays.md "Dynamic-array and LAMBDA function argument modes"):

| case / cell | formula | Excel | ours |
|---|---|---|---|
| probes/m7-lift-take-drop-sortby Sheet1!F33 | `=ROWS(TAKE(NA(),{1;2}))` | 2.0 | #N/A |
| probes/m7-lift-take-drop-sortby Sheet1!F34 | `=ROWS(OFFSET(TAKE(A1:B3,{1;2}),1,0))` | 2.0 | #VALUE! |
| probes/m7-lift-take-drop-sortby Sheet1!G33 | `=COLUMNS(TAKE(NA(),{1;2}))` | 1.0 | #N/A |
| probes/m7-lift-take-drop-sortby Sheet1!G34 | `=COLUMNS(OFFSET(TAKE(A1:B3,{1;2}),1,0))` | 1.0 | #VALUE! |
| probes/m7-lift-take-drop-sortby Sheet1!H34 | `=INDEX(INDEX(OFFSET(TAKE(A1:B3,{1;2}),1,0),1,1),1,1)` | 2.0 | #VALUE! |
| probes/m7-lift-take-drop-sortby Sheet1!I34 | `=ROWS(INDEX(OFFSET(TAKE(A1:B3,{1;2}),1,0),1,1))` | 1.0 | #VALUE! |
| probes/m7-lift-take-drop-sortby Sheet1!J34 | `=COLUMNS(INDEX(OFFSET(TAKE(A1:B3,{1;2}),1,0),1,1))` | 2.0 | #VALUE! |
| probes/m7-lift-take-drop-sortby Sheet1!K34 | `=SUM(INDEX(OFFSET(TAKE(A1:B3,{1;2}),1,0),1,1))` | 22.0 | #VALUE! |
| probes/m7-lift-take-drop-sortby Sheet1!L34 | `=INDEX(INDEX(OFFSET(TAKE(A1:B3,{1;2}),1,0),2,1),1,1)` | 2.0 | #VALUE! |
| probes/m7-lift-take-drop-sortby Sheet1!M34 | `=ROWS(INDEX(OFFSET(TAKE(A1:B3,{1;2}),1,0),2,1))` | 2.0 | #VALUE! |
| probes/m7-lift-take-drop-sortby Sheet1!N34 | `=COLUMNS(INDEX(OFFSET(TAKE(A1:B3,{1;2}),1,0),2,1))` | 2.0 | #VALUE! |
| probes/m7-lift-take-drop-sortby Sheet1!O34 | `=SUM(INDEX(OFFSET(TAKE(A1:B3,{1;2}),1,0),2,1))` | 55.0 | #VALUE! |
| probes/m7-lift-take-drop-sortby Sheet1!U10 | `=ROWS(TAKE(NA(),{1;2}))` | 2.0 | #N/A |
| probes/m7-lift-take-drop-sortby Sheet1!U17 | `=SUMPRODUCT(CHOOSE(1,TAKE(A1:B3,{1;2})))` | 0.0 | #VALUE! |
| probes/m7-lift-take-drop-sortby Sheet1!U7 | `=SUMPRODUCT(IF(TRUE,TAKE(A1:B3,{1;2})))` | 0.0 | #VALUE! |
| probes/m7-lift-take-drop-sortby Sheet1!U9 | `=ROWS(OFFSET(TAKE(A1:B3,{1;2}),1,0))` | 2.0 | #VALUE! |

## Acceptance criteria
- [ ] The cells above pass and leave eval/baseline/probes.txt (`make eval-baseline`), with no new known failures.
- [ ] The rest of the case's TAKE/DROP rows (5-11, 27, 28, 31, 32 and U1-U18) still pass.
- [ ] lookup.md "OFFSET, INDIRECT and TRIMRANGE lift into arrays of references" (or SPEC-0014 "As built") names TAKE and DROP as handed on.
- [ ] `make check` green; reviewers approve.

## Out of scope
- New producers or consumers beyond the cells above (SPEC-0014 tasks).
