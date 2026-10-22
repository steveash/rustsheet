---
id: T-2314
title: "TEXTBEFORE and TEXTAFTER: instance_num past the text length is #VALUE!, a number if_not_found is returned as text (gen-m7)"
milestone: M7
lane: functions-text
status: in-progress
priority: P2
size: S
tier: sonnet
campaign: visible
depends_on: []
---

## Goal
TEXTBEFORE and TEXTAFTER are #VALUE! when |instance_num| is greater than the text's length, whatever the delimiter, match_end or if_not_found, and return a number if_not_found as its text, as Excel gives.

## Context
Found by the T-2225 triage of eval/suites/gen-m7 (38 cases, 10,991 Excel-scored cells, excel-graph). Quirk entry: docs/excel-semantics/text.md "TEXTBEFORE and TEXTAFTER: instance_num past the text, a number if_not_found".

Evidence: `TEXTBEFORE("b","a",-2,1,1)` (2c5ea078f25443a2 A360), `TEXTBEFORE(<"x~*">,<"1">,WORKDAY(..))` (611ce3bbfab27cef A336), `TEXTBEFORE(<empty>,Data!A6:A11,2,123456789.123,1,FALSE)` (c36ff5f42a1a4d47 A148, ours "") and `TEXTBEFORE(..,"-",DATE(2060,10,19),0,0,6)` with an if_not_found (b24ce74f08d3f91a A767, ours 6) are `#VALUE!`; ours `#N/A` or the if_not_found. `TEXTAFTER(..,"a",..,0,1,Data!$E12=-2.5)` is the text "-2.5" (ac67253061bba16a A776) and `TEXTBEFORE(..,",",2,..,1,INT(Data!C$8)=3)` the text "3" (ac67253061bba16a A1345); ours returns the numbers. text.md already has "Every list entry longer than the text gives ... #VALUE! with more than two arguments"; the instance_num bound is the same kind of rule. Code: functions/text/before_after.rs.

Visibly-off cells (gap triage 2026-10-21, decision 0083): 6 cells, every one with a wrong type, error, text or value (none is a last-digit difference).

Owned gen-m7 cells (T-2225 triage, excel-graph, 2026-10-21): 6 known-failure lines in eval/baseline/gen-m7.txt, 6 formulas. One line per formula: case, anchor, the anchor formula, Excel vs ours at the first failing cell, then every failing cell (sheet Calc; a range is consecutive cells of one row).

- gen-m7/2c5ea078f25443a2.case.json A360 `=TEXTBEFORE(RIGHT((Data!E5&"?b")),"a",-2,1,1)`: A360 Excel #VALUE!, ours #N/A; cells (1): A360
- gen-m7/611ce3bbfab27cef.case.json A336 `=TEXTBEFORE(Data!A8,Data!A11:A11,WORKDAY(Data!B1,(Data!$D1+0.2)))`: A336 Excel #VALUE!, ours #N/A; cells (1): A336
- gen-m7/ac67253061bba16a.case.json A776 `=TEXTAFTER(Data!B9,"a",(LN(0.1)+(Data!A9/Data!E$6)),0,1,Data!$E12)`: A776 Excel "-2.5", ours -2.5; cells (1): A776
- gen-m7/ac67253061bba16a.case.json A1345 `=TEXTBEFORE(Data!A4,",",2,TRUNC(MOD(1+2^-52,Data!$A$7),-2),1,INT(Data!C$8))`: A1345 Excel "3", ours 3.0; cells (1): A1345
- gen-m7/b24ce74f08d3f91a.case.json A767 `=TEXTBEFORE(ASC((Data!C1&Data!C5)),"-",DATE(2060,10,19),0,0,COUNT(Data!B10:B11,Data!B4:B9,SUMIF(Data!A7:A8,"<>1",Data!C6:D6)))`: A767 Excel #VALUE!, ours 6.0; cells (1): A767
- gen-m7/c36ff5f42a1a4d47.case.json A148 `=TEXTBEFORE(Data!A8,Data!A6:A11,2,Data!D5,1,AND(NOT(Data!C4),AND(Data!E5:E10,FALSE,TRUE)))`: A148 Excel #VALUE!, ours ""; cells (1): A148

## Acceptance criteria
- [ ] Visible parity: the 6 gen-m7 cells listed under Context have Excel's value, type, error and text: these 6 known-failure lines disappear from eval/baseline/gen-m7.txt (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] The docs/excel-semantics entry cited above states the rule and its "Ours:" line names this task's change.
- [ ] Unit tests cover the rule on a small hand-written input (one error or edge case and one normal case that must keep passing).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The empty delimiter (text.md says it ignores instance_num's size: keep those cells passing).

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off (a wrong type, error, text or value), so this task is in the campaign. Its target is that every cell it owns has Excel's type, error and text, or is within half a unit of the 15th significant digit of Excel's value (an expected 0 exactly 0); the visible-parity criterion under Acceptance criteria states it.
