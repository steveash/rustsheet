---
id: T-2415
title: "EXPAND: a multi-cell range pad_with is one #VALUE!, as an array pad_with is (gen-m7, 312 cells)"
milestone: M7
lane: functions-lookup
status: todo
priority: P1
size: S
tier: sonnet
depends_on: []
campaign: visible
---

## Goal
EXPAND whose pad_with is a multi-cell range is one `#VALUE!` (no spill), as it already is for a multi-cell array pad_with, whatever
the array, rows and columns; a one-cell range pad_with keeps working as its value.

## Context
Gap triage of the T-2254 gen-m7 sweep (excel-graph, 2026-10-22). arrays.md "EXPAND, WRAPROWS and WRAPCOLS: scalar arguments lift,
EXPAND's pad_with does not" says an array pad_with makes the whole call `#VALUE!` (`ROWS(EXPAND(A1:A2,3,1,{0;1}))`), and ours does
that for an array value. A multi-cell *range* there was unprobed (T-2365 lists it: "ours intersects"); the 17 gen-m7 formulas below
answer it: Excel gives one `#VALUE!` in the anchor and nothing spilled, for a row range (`Data!A7:D7`), a column range
(`Data!E1:E3`, `Data!C7:C11`), a 2-D range (`Data!B5:C11`, `Data!A3:D10`), with or without padding actually needed
(`EXPAND(TAN(-Data!A9),(Data!$B4+Data!B10),1,Data!B2:B6)` returns a 1x1 that pads nothing and is still `#VALUE!`, 2d9b48c6363e7a16 A830),
and before a spill collision (`EXPAND(Data!$E8,YEAR(..),2,Data!C1:E3)` is `#VALUE!`, ours `#SPILL!`, gen-m7/2c156ba98ef0220a.case.json A1577).
Ours reads the range by implicit intersection (or its top-left) and pads with that value, so the anchor holds the array's first
value and the footprint spills (most of the 312 cells are footprint cells Excel leaves empty). Two formulas use the EXPAND under
another call whose own error rule then applies unchanged: `TEXTAFTER((VLOOKUP(..)&EXPAND("x~*",1,5,Data!D2:E2)),"-")`
(4dac39f89709e2fc A2174) and TEXTSPLIT's ignore_empty `EXPAND(Data!A6:C10,5,3,Data!A4:B5)` (f7e94a817e5385ac A330).
Code: `fn_expand` in crates/rustsheet-engine/src/functions/lookup_and_reference/expand.rs (and its lifting-table entry: pad_with's
kind must refuse a multi-cell reference as it refuses a multi-cell array). Quirk entry: docs/excel-semantics/arrays.md (the EXPAND
section, updated by this triage).

Owned gen-m7 cells (T-2254 triage, excel-graph, 2026-10-22): 312 known-failure lines in eval/baseline/gen-m7.txt, 17 formulas.
After the fix, re-fill gen-m7/0624b684889e9228 (`rustsheet-oracle fill -b excel-graph --suite gen-m7 --force --case
gen-m7/0624b684889e9228.case.json`): T-2417 decodes its A2004 anchor (a spilled REGEXEXTRACT `#VALUE!`), which makes
the rest of A2004's footprint, AH2004:CV2008 (335 cells), scored; T-2417 left that re-fill to this task because those
cells are visibly off until EXPAND is fixed and the visible-parity lock cannot take them as new coverage.
- gen-m7/0624b684889e9228.case.json A2004 `=EXPAND({1;2;3},5,100,Data!A7:D7)`: A2004 Excel #VALUE!, ours 1.0; cells (165): A2004:AG2004, A2005:AG2005, A2006:AG2006, A2007:AG2007, A2008:AG2008
- gen-m7/2c156ba98ef0220a.case.json A185 `=EXPAND(Data!C11:D12,2,2,Data!E1:E3)`: A185 Excel #VALUE!, ours "x"; cells (4): A185:B185, A186:B186
- gen-m7/2c156ba98ef0220a.case.json A1577 `=EXPAND(Data!$E8,YEAR(ABS(Data!B$4)),2,Data!C1:E3)`: A1577 Excel #VALUE!, ours #SPILL!; cells (1): A1577
- gen-m7/2d9b48c6363e7a16.case.json A366 `=EXPAND("a",4,4,Data!C7:C11)`: A366 Excel #VALUE!, ours "a"; cells (16): A366:D366, A367:D367, A368:D368, A369:D369
- gen-m7/2d9b48c6363e7a16.case.json A830 `=EXPAND(TAN(-Data!A9),(Data!$B4+Data!B10),1,Data!B2:B6)`: A830 Excel #VALUE!, ours 0.0; cells (1): A830
- gen-m7/2d9b48c6363e7a16.case.json A2686 `=EXPAND({1,2,3},5,4,Data!B5:C11)`: A2686 Excel #VALUE!, ours 1.0; cells (20): A2686:D2686, A2687:D2687, A2688:D2688, A2689:D2689, A2690:D2690
- gen-m7/4dac39f89709e2fc.case.json A2174 `=TEXTAFTER((VLOOKUP(Data!E9,Data!B4:E12,0.3)&EXPAND("x~*",1,5,Data!D2:E2)),"-")`: B2174 Excel <empty>, ours #VALUE!; cells (4): B2174:E2174
- gen-m7/8c4570978f18f4c2.case.json A526 `=EXPAND(SEQUENCE(1,1),5,1,Data!A11:A12)`: A526 Excel #VALUE!, ours 1.0; cells (5): A526, A527, A528, A529, A530
- gen-m7/8c4570978f18f4c2.case.json A1222 `=EXPAND(LET(v,(Data!E$10+Data!B11),x,(v*v),x%),4,4,Data!C6:D11)`: A1222 Excel #VALUE!, ours 0.01; cells (16): A1222:D1222, A1223:D1223, A1224:D1224, A1225:D1225
- gen-m7/8c4570978f18f4c2.case.json A1686 `=EXPAND(Data!B$9,2,5,Data!B9:C12)`: A1686 Excel #VALUE!, ours "x~*"; cells (10): A1686:E1686, A1687:E1687
- gen-m7/b2078b5d702c691a.case.json A2602 `=EXPAND(Data!B6:C7,4,2,Data!A3:D10)`: A2602 Excel #VALUE!, ours 7.0; cells (8): A2602:B2602, A2603:B2603, A2604:B2604, A2605:B2605
- gen-m7/bc4a69b0b512bc04.case.json A502 `=EXPAND(ROUND(MAX(Data!E10:E10,Data!E9,Data!A9:A9),Data!C10),1,2,Data!A6:C7)`: A502 Excel #VALUE!, ours 1.0; cells (2): A502:B502
- gen-m7/bc4a69b0b512bc04.case.json A3054 `=EXPAND(ROW(),4,COUNT(Data!D4:D11,(0.5-0.1)),Data!E3:E4)`: A3054 Excel #VALUE!, ours 3054.0; cells (20): A3054:E3054, A3055:E3055, A3056:E3056, A3057:E3057
- gen-m7/bc4a69b0b512bc04.case.json A3286 `=EXPAND(MAX(Data!E3),4,3,Data!A11:D11)`: A3286 Excel #VALUE!, ours 0.0; cells (12): A3286:C3286, A3287:C3287, A3288:C3288, A3289:C3289
- gen-m7/f19581ac44da1653.case.json A3077 `=EXPAND(Data!B1:B5,5,2,Data!A11:B12)`: A3077 Excel #VALUE!, ours 1.0; cells (10): A3077:B3077, A3078:B3078, A3079:B3079, A3080:B3080, A3081:B3081
- gen-m7/f7e94a817e5385ac.case.json A2077 `=EXPAND(Data!A11:A12,4,1,Data!A6:C6)`: A2077 Excel #VALUE!, ours FALSE; cells (4): A2077, A2078, A2079, A2080
- gen-m7/f7e94a817e5385ac.case.json A330 `=TEXTSPLIT((LEFT("1")&MAKEARRAY(Data!B12,1,LAMBDA(z,v,z*v))),"a",Data!D5:D12,EXPAND(Data!A6:C10,5,3,Data!A4:B5))`: B330 Excel <empty>, ours #VALUE!; cells (14): B330:C330, A331:C331, A332:C332, A333:C333, A334:C334

## Acceptance criteria
- [ ] Visible parity: these 312 known-failure lines disappear from eval/baseline/gen-m7.txt (`make eval-baseline`), with no new
      known-failure lines in any baseline.
- [ ] A unit test covers a multi-cell range pad_with (`#VALUE!`, one cell) and a one-cell range pad_with (its value pads).
- [ ] arrays.md's EXPAND entry has an "Ours:" line naming this task.
- [ ] `make check` green; reviewers approve.

## Out of scope
- EXPAND sizes past the grid (T-2408); the rest of T-2365's probe questions (an error array beside a lifted argument).

## Visible-parity stage
Decision 0083 (gap triage 2026-10-22, T-2254): every cell this task owns has the wrong type, error, text or value (none is a
last-digit difference), so the task is in the campaign. Its target is that every cell it owns has Excel's type, error and text,
or is within half a unit of the 15th significant digit of Excel's value; the visible-parity criterion under Acceptance criteria
states it.
