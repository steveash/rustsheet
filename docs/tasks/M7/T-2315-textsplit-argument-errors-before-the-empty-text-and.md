---
id: T-2315
title: "TEXTSPLIT: argument errors before the empty-text and empty-delimiter #VALUE!, an all-empty result is #CALC!, lifted arguments (gen-m7)"
milestone: M7
lane: functions-text
status: todo
priority: P2
size: S
tier: opus
campaign: visible
depends_on: []
---

## Goal
TEXTSPLIT returns an error argument (match_mode, ignore_empty) before its own #VALUE! for empty text or an empty delimiter, is #CALC! when ignore_empty drops every piece, and lifts an array ignore_empty or match_mode, as Excel gives.

## Context
Found by the T-2225 triage of eval/suites/gen-m7 (38 cases, 10,991 Excel-scored cells, excel-graph). Quirk entry: docs/excel-semantics/text.md "TEXTSPLIT: argument errors first, an all-empty result".

Evidence: `TEXTSPLIT("ABC",Data!D4:E7,Data!C5:E9,-1,#DIV/0!)` is `#DIV/0!` (90de8755a2e565e6 A1643, ours `#VALUE!`, the delimiter ranges hold blanks); `TEXTSPLIT("","-",Data!A11:D12,#REF!)` is `#REF!` (ac67253061bba16a A184, ours `#VALUE!`); `TEXTSPLIT(100,{1.005,100},"a",TRUE)` drops both empty pieces and is `#CALC!` in Excel, seen through `REGEXEXTRACT(LOWER(..),..)` (07d27aa0add0247f A768; ours `#VALUE!`); `TEXTSPLIT("x~*",Data!C7:C11," ",UNIQUE(Data!C4:E8),1,(OFFSET(..)<>FALSE))` spills 15 `#VALUE!` (an array ignore_empty lifts; 8044dce50616cfe9 A1064, ours one `#VALUE!`). Code: functions/text/textsplit.rs.

Visibly-off cells (gap triage 2026-10-21, decision 0083): 17 cells, every one with a wrong type, error, text or value (none is a last-digit difference).

Owned gen-m7 cells (T-2225 triage, excel-graph, 2026-10-21): 17 known-failure lines in eval/baseline/gen-m7.txt, 4 formulas. One line per formula: case, anchor, the anchor formula, Excel vs ours at the first failing cell, then every failing cell (sheet Calc; a range is consecutive cells of one row).

- gen-m7/07d27aa0add0247f.case.json A768 `=REGEXEXTRACT(LOWER(TEXTSPLIT(Data!E1,Data!D1:E1,"a",TRUE)),"(\w)(\w)")`: A768 Excel #CALC!, ours #VALUE!; cells (1): A768
- gen-m7/8044dce50616cfe9.case.json A1064 `=TEXTSPLIT("x~*",Data!C7:C11," ",UNIQUE(Data!C4:E8),1,(OFFSET(Data!D2:D2,1,0)<>(2>100)))`: B1064 Excel #VALUE!, ours <empty>; cells (14): B1064:C1064, A1065:C1065, A1066:C1066, A1067:C1067, A1068:C1068
- gen-m7/90de8755a2e565e6.case.json A1643 `=TEXTSPLIT(TEXTSPLIT((""&"ABC"),Data!D4:E7,Data!C5:E9,Data!B2,(Data!C7/Data!$C$2)),Data!B1:E6,",")`: A1643 Excel #DIV/0!, ours #VALUE!; cells (1): A1643
- gen-m7/ac67253061bba16a.case.json A184 `=TEXTSPLIT(IF(Data!$C3,""),"-",Data!A11:D12,INDEX(Data!A4:A7,DAYS(Data!B10,Data!B$5),3))`: A184 Excel #REF!, ours #VALUE!; cells (1): A184

## Acceptance criteria
- [ ] Visible parity: the 17 gen-m7 cells listed under Context have Excel's value, type, error and text: these 17 known-failure lines disappear from eval/baseline/gen-m7.txt (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] The docs/excel-semantics entry cited above states the rule and its "Ours:" line names this task's change.
- [ ] Unit tests cover the rule on a small hand-written input (one error or edge case and one normal case that must keep passing).
- [ ] `make check` green; reviewers approve.

## Out of scope
- TEXTBEFORE/TEXTAFTER (T-2314).

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off (a wrong type, error, text or value), so this task is in the campaign. Its target is that every cell it owns has Excel's type, error and text, or is within half a unit of the 15th significant digit of Excel's value (an expected 0 exactly 0); the visible-parity criterion under Acceptance criteria states it.
