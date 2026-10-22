---
id: T-2310
title: "SORTBY: a single-value by_array keeps the array's order, and a fractional sort_order reads its sign (gen-m7)"
milestone: M7
lane: functions-lookup
status: todo
priority: P2
size: S
tier: sonnet
campaign: visible
depends_on: []
---

## Goal
SORTBY with a single value as by_array (a number from MAX(..)% or TIMEVALUE) returns the array in its own order, and a fractional sort_order such as -0.0001 sorts by its sign (descending), as Excel gives; ours is #VALUE! for both.

## Context
Found by the T-2225 triage of eval/suites/gen-m7 (38 cases, 10,991 Excel-scored cells, excel-graph). Quirk entry: docs/excel-semantics/arrays.md "SORTBY with a single-value by_array or a fractional sort_order".

Evidence: `SORTBY(Data!E1:E12,MAX(..)%,-1)` is Data!E1:E12 unchanged, 12 rows (18b80d3fd292d63a A123:A134; ours `#VALUE!`, and `SORTBY(Data!E1:E12,5,-1)` is `#VALUE!` too), `SORTBY(Data!C1:C12,IFNA(TIMEVALUE(..),..),1)` is Data!C1:C12 unchanged with its blanks as 0 (7307132bb8820f17 A966), `SORTBY(Data!B1:E12,Data!E1:E12,-0.0001,Data!B1:B12,-1)` equals the same call with -1 (its first cell "x~*"; ours `#VALUE!`; 07d27aa0add0247f A658, under a TEXTSPLIT whose 48 cells follow), and 90de8755a2e565e6 A717 has both in a REGEXEXTRACT return_mode (`SORTBY(Data!E1:E12,Data!D3,(-0.0001)/Data!A3)`). arrays.md line "Invalid values" says `SORTBY(arr,key,0)` and a row-count mismatch are `#VALUE!`: a one-cell by_array is not a mismatch in Excel, and a non-integer order is not invalid. Whether 0.5 sorts ascending and whether a one-cell range (not a value) behaves the same are unprobed: they are out of scope, and the code keeps today's answer for them. Code: `fn_sortby` in functions/spill_functions.rs.

Visibly-off cells (gap triage 2026-10-21, decision 0083): 83 cells, every one with a wrong type, error, text or value (none is a last-digit difference).

Owned gen-m7 cells (T-2225 triage, excel-graph, 2026-10-21): 83 known-failure lines in eval/baseline/gen-m7.txt, 4 formulas. One line per formula: case, anchor, the anchor formula, Excel vs ours at the first failing cell, then every failing cell (sheet Calc; a range is consecutive cells of one row).

- gen-m7/07d27aa0add0247f.case.json A658 `=TEXTSPLIT((Data!B6&SORTBY(Data!B1:E12,Data!E1:E12,(-0.0001),Data!B1:B12,-1)),Data!C11:D11)`: A658 Excel "x~*", ours #VALUE!; cells (48): A658:D658, A659:D659, A660:D660, A661:D661, A662:D662, A663:D663, A664:D664, A665:D665, A666:D666, A667:D667, A668:D668, A669:D669
- gen-m7/18b80d3fd292d63a.case.json A123 `=SORTBY(Data!E1:E12,MAX(Data!C1:C12,Data!C2:C10,Data!A1:A11)%,-1)`: A123 Excel 2.5, ours #VALUE!; cells (12): A123, A124, A125, A126, A127, A128, A129, A130, A131, A132, A133, A134
- gen-m7/7307132bb8820f17.case.json A966 `=SORTBY(Data!C1:C12,IFNA(TIMEVALUE("2024-02-29 07:05:09"),LOOKUP("Hello World",Data!C2:C6)),1)`: A966 Excel -0.5, ours #VALUE!; cells (12): A966, A967, A968, A969, A970, A971, A972, A973, A974, A975, A976, A977
- gen-m7/90de8755a2e565e6.case.json A717 `=REGEXEXTRACT(SORT(Data!B11:B11),REGEXREPLACE(SWITCH(Data!E4,0.1,Data!A7),TEXTBEFORE("x~*","a"),"-",0),SORTBY(Data!E1:E12,Data!D3,((-0.0001)/Data!A3)))`: A718 Excel #N/A, ours <empty>; cells (11): A718, A719, A720, A721, A722, A723, A724, A725, A726, A727, A728

## Acceptance criteria
- [ ] Visible parity: the 83 gen-m7 cells listed under Context have Excel's value, type, error and text: these 83 known-failure lines disappear from eval/baseline/gen-m7.txt (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] The docs/excel-semantics entry cited above states the rule and its "Ours:" line names this task's change.
- [ ] Unit tests cover the rule on a small hand-written input (one error or edge case and one normal case that must keep passing).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Lifted sort_order arrays (T-1839); unprobed variants (a positive fraction, a one-cell range by_array).

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off (a wrong type, error, text or value), so this task is in the campaign. Its target is that every cell it owns has Excel's type, error and text, or is within half a unit of the 15th significant digit of Excel's value (an expected 0 exactly 0); the visible-parity criterion under Acceptance criteria states it.
