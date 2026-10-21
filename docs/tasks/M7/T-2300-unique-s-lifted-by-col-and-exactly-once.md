---
id: T-2300
title: "UNIQUE's lifted by_col and exactly_once: later elements are empty unless every entry is equal"
milestone: M7
lane: functions-lookup
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-1960]
spec: docs/specs/SPEC-0011-scalar-argument-lifting.md
---

## Goal
UNIQUE's lifted by_col and exactly_once give Excel's elements: when every entry of the lifted argument is equal each element is the top-left of that call, and otherwise the first element is the top-left of the call with the first entry and every later element is empty (it spills as 0; ISBLANK TRUE, SUM 0, ROWS, COLUMNS and INDEX of it `#VALUE!`).

## Context
From T-1960. T-1836 lifted by_col with the top-left rule (SPEC-0011 rule 2) and exactly_once by the per-parameter rule. Excel: `UNIQUE(D1:D3,FALSE,{FALSE;TRUE})` is 2x1 with element 1 "x" and element 2 empty (not "y"); `{TRUE;FALSE}` gives "y" then empty; `{FALSE;FALSE;TRUE}` gives "x", empty, empty (element 2 is empty although its entry equals the first); `UNIQUE(D1:D3,{FALSE;TRUE})` and `{TRUE;FALSE}` give "x" then empty; the uniform `{FALSE;FALSE}` and `FALSE,{TRUE;TRUE}` give two equal elements ("x", "x" and "y", "y"; pass today). SORTBY's sort_order shows the same rule (T-1839), so build it once in the lift driver as an element rule an entry opts into (directive 3), not in `fn_unique`; T-1839 reuses it. An empty element is a value the driver can already place (`CalcResult::EmptyCell` or the blank a spill writes as 0); check `INDEX(INDEX(..,2,1),1,1)` is `#VALUE!` and `SUM(INDEX(..,2,1))` 0. UNIQUE's empty element is evidenced only by SUM 0 and ROWS, COLUMNS and INDEX `#VALUE!`; the spilled 0, ISBLANK TRUE and `&"|"` = "|" are SORTBY's cells (Sort!U5:V13, X45:X47), so first add probe cells `ISBLANK(INDEX(UNIQUE(D1:D3,FALSE,{FALSE;TRUE}),2,1))`, its `&"|"` and a direct spill (excel-probe skill) and confirm them. TOCOL's scan_by_column does not show it (`TOCOL(A1:B3,0,{FALSE;TRUE})` element 2 is 1, ROWS 1, SUM 1; passes).

Owned cells, probes/m7-lift-take-drop-sortby (T-1960, excel-graph, 2026-10-21; Sheet1 A1:B3 = 1,10;2,20;3,30, D1:D3 = x,y,x; column E holds each row's formula as text, F:S its ROWS, COLUMNS and per element INDEX(INDEX(..,i,1),1,1), ROWS, COLUMNS and SUM of INDEX(..,i,1); arrays.md "Dynamic-array and LAMBDA function argument modes"):

| case / cell | formula | Excel | ours |
|---|---|---|---|
| probes/m7-lift-take-drop-sortby Sheet1!L25 | `=INDEX(INDEX(UNIQUE(D1:D3,FALSE,{FALSE;TRUE}),2,1),1,1)` | #VALUE! | "y" |
| probes/m7-lift-take-drop-sortby Sheet1!L26 | `=INDEX(INDEX(UNIQUE(D1:D3,FALSE,{TRUE;FALSE}),2,1),1,1)` | #VALUE! | "x" |
| probes/m7-lift-take-drop-sortby Sheet1!L29 | `=INDEX(INDEX(UNIQUE(D1:D3,{FALSE;TRUE}),2,1),1,1)` | #VALUE! | "x" |
| probes/m7-lift-take-drop-sortby Sheet1!L30 | `=INDEX(INDEX(UNIQUE(D1:D3,{TRUE;FALSE}),2,1),1,1)` | #VALUE! | "x" |
| probes/m7-lift-take-drop-sortby Sheet1!L37 | `=INDEX(INDEX(UNIQUE(D1:D3,FALSE,{FALSE;FALSE;TRUE}),2,1),1,1)` | #VALUE! | "x" |
| probes/m7-lift-take-drop-sortby Sheet1!M25 | `=ROWS(INDEX(UNIQUE(D1:D3,FALSE,{FALSE;TRUE}),2,1))` | #VALUE! | 1.0 |
| probes/m7-lift-take-drop-sortby Sheet1!M26 | `=ROWS(INDEX(UNIQUE(D1:D3,FALSE,{TRUE;FALSE}),2,1))` | #VALUE! | 1.0 |
| probes/m7-lift-take-drop-sortby Sheet1!M29 | `=ROWS(INDEX(UNIQUE(D1:D3,{FALSE;TRUE}),2,1))` | #VALUE! | 1.0 |
| probes/m7-lift-take-drop-sortby Sheet1!M30 | `=ROWS(INDEX(UNIQUE(D1:D3,{TRUE;FALSE}),2,1))` | #VALUE! | 1.0 |
| probes/m7-lift-take-drop-sortby Sheet1!M37 | `=ROWS(INDEX(UNIQUE(D1:D3,FALSE,{FALSE;FALSE;TRUE}),2,1))` | #VALUE! | 1.0 |
| probes/m7-lift-take-drop-sortby Sheet1!N25 | `=COLUMNS(INDEX(UNIQUE(D1:D3,FALSE,{FALSE;TRUE}),2,1))` | #VALUE! | 1.0 |
| probes/m7-lift-take-drop-sortby Sheet1!N26 | `=COLUMNS(INDEX(UNIQUE(D1:D3,FALSE,{TRUE;FALSE}),2,1))` | #VALUE! | 1.0 |
| probes/m7-lift-take-drop-sortby Sheet1!N29 | `=COLUMNS(INDEX(UNIQUE(D1:D3,{FALSE;TRUE}),2,1))` | #VALUE! | 1.0 |
| probes/m7-lift-take-drop-sortby Sheet1!N30 | `=COLUMNS(INDEX(UNIQUE(D1:D3,{TRUE;FALSE}),2,1))` | #VALUE! | 1.0 |
| probes/m7-lift-take-drop-sortby Sheet1!N37 | `=COLUMNS(INDEX(UNIQUE(D1:D3,FALSE,{FALSE;FALSE;TRUE}),2,1))` | #VALUE! | 1.0 |
| probes/m7-lift-take-drop-sortby Sheet1!O25 | `=SUM(INDEX(UNIQUE(D1:D3,FALSE,{FALSE;TRUE}),2,1))` | 0.0 | #VALUE! |
| probes/m7-lift-take-drop-sortby Sheet1!O26 | `=SUM(INDEX(UNIQUE(D1:D3,FALSE,{TRUE;FALSE}),2,1))` | 0.0 | #VALUE! |
| probes/m7-lift-take-drop-sortby Sheet1!O29 | `=SUM(INDEX(UNIQUE(D1:D3,{FALSE;TRUE}),2,1))` | 0.0 | #VALUE! |
| probes/m7-lift-take-drop-sortby Sheet1!O30 | `=SUM(INDEX(UNIQUE(D1:D3,{TRUE;FALSE}),2,1))` | 0.0 | #VALUE! |
| probes/m7-lift-take-drop-sortby Sheet1!O37 | `=SUM(INDEX(UNIQUE(D1:D3,FALSE,{FALSE;FALSE;TRUE}),2,1))` | 0.0 | #VALUE! |
| probes/m7-lift-take-drop-sortby Sheet1!P37 | `=INDEX(INDEX(UNIQUE(D1:D3,FALSE,{FALSE;FALSE;TRUE}),3,1),1,1)` | #VALUE! | "y" |
| probes/m7-lift-take-drop-sortby Sheet1!Q37 | `=ROWS(INDEX(UNIQUE(D1:D3,FALSE,{FALSE;FALSE;TRUE}),3,1))` | #VALUE! | 1.0 |
| probes/m7-lift-take-drop-sortby Sheet1!R37 | `=COLUMNS(INDEX(UNIQUE(D1:D3,FALSE,{FALSE;FALSE;TRUE}),3,1))` | #VALUE! | 1.0 |
| probes/m7-lift-take-drop-sortby Sheet1!S37 | `=SUM(INDEX(UNIQUE(D1:D3,FALSE,{FALSE;FALSE;TRUE}),3,1))` | 0.0 | #VALUE! |

gen-m7 cells (T-2225 triage, 2026-10-21): UNIQUE with an array by_col or exactly_once (UNIQUE, SORT, XOR, FILTER or HSTACK results): Excel's later elements are empty (or `#VALUE!` for the 1x2 HSTACK by_col of 8044dce50616cfe9 A1434), ours each call's top-left.

Owned gen-m7 cells (T-2225 triage, excel-graph, 2026-10-21): 34 known-failure lines in eval/baseline/gen-m7.txt, 4 formulas. One line per formula: case, anchor, the anchor formula, Excel vs ours at the first failing cell, then every failing cell (sheet Calc; a range is consecutive cells of one row).

- gen-m7/2c5ea078f25443a2.case.json A483 `=UNIQUE(Data!D8:D9,UNIQUE(Data!B10:C11,VLOOKUP(Data!$B6,Data!A2:D4,2),("x~*"=3.14159)),(0.3=123456789.123))`: B483 Excel 0.0, ours 0.5; cells (3): B483, A484:B484
- gen-m7/7307132bb8820f17.case.json A1173 `=UNIQUE(Data!B6:D9,SORT(-Data!A$12),XOR(Data!C11,AND(Data!E1:E12,TRUE),Data!B5:B8))`: B1173 Excel <empty>, ours 0.0; cells (11): B1173:C1173, A1174:C1174, A1175:C1175, A1176:C1176
- gen-m7/7d67c559ce7536fa.case.json A185 `=UNIQUE(Data!B4:D6,Data!C$8,FILTER(Data!D1:E12,(Data!A1:A12<>(-2.5)),Data!B7:D9))`: B186 Excel 0.0, ours 123456789.123; cells (19): B186, A187:B187, A188:B188, A190:B190, A191:B191, A192:B192, A193:B193, A194:B194, A195:B195, A196:B196
- gen-m7/8044dce50616cfe9.case.json A1434 `=UNIQUE(SUMPRODUCT((100+42),Data!C8:E8,Data!A8:D12),HSTACK(0.2,1000000000000000.0),(Data!B2<=((-1)/(-0.5))))`: B1434 Excel #VALUE!, ours <empty>; cells (1): B1434

## Acceptance criteria
- [ ] The 34 gen-m7 cells listed under Context (T-2225 triage) match Excel: their 34 known-failure lines leave eval/baseline/gen-m7.txt (`make eval-baseline`), with no new known failures.
- [ ] The cells above pass and leave eval/baseline/probes.txt (`make eval-baseline`), with no new known failures; rows 24, 35 and 36 still pass.
- [ ] The rule is one driver option with a unit test (uniform entries, a differing later entry, a differing entry equal to the first after a differing one), documented in SPEC-0011 "Array-valued functions" rule 2.
- [ ] `make check` green; reviewers approve.

## Out of scope
- SORTBY (T-1839), which reuses the rule.
