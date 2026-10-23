---
id: T-2315
title: "TEXTSPLIT: argument errors before the empty-text and empty-delimiter #VALUE!, an all-empty result is #CALC!, lifted arguments (gen-m7)"
milestone: M7
lane: functions-text
status: done
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

More gen-m7 cells (T-2254 triage, excel-graph, 2026-10-22), the same rules: an error ignore_empty or match_mode before the blank-delimiter
`#VALUE!` (`INDIRECT(..)` `#REF!` in 21b4eef18424ce94 A733; `(1.005/Data!C6)>..` `#DIV/0!` in 2d9b48c6363e7a16 A2333; a LOOKUP or HLOOKUP
`#N/A` in 4dac39f89709e2fc A3213, which lifts over a 2-row text, and b2078b5d702c691a A356; ours `#VALUE!` from the blank delimiter cells,
checked with `NA()` in place of the error), an error text before a pad_with array's `#VALUE!` (`Data!D$12&BYCOL(Data!E5:E6,LAMBDA(x,x))` is
`#CALC!`, the BYCOL of a column slice, while pad_with is a 1x3 array: f7e94a817e5385ac A2186; text.md marks that order UNVERIFIED, this
cell settles it), and lifted arguments: an array match_mode `TOCOL(Data!A11:D12)` lifts to 8 rows of `#VALUE!`
(2d9b48c6363e7a16 A3261; ours one `#VALUE!`), and a 1x3 text against an 11x2 ignore_empty broadcasts with `#N/A` in the third column
(bc4a69b0b512bc04 A538:C548; ours `#VALUE!` there, arrays.md "Scalar arguments lift": a position past a shorter argument is `#N/A`).
Owned: 24 known-failure lines in eval/baseline/gen-m7.txt, 7 formulas.
- gen-m7/21b4eef18424ce94.case.json A733 `=TEXTSPLIT(("?b"&Data!C5),Data!B5:C11,Data!B6,INDIRECT((Data!C5&Data!A6),(42=Data!B3)),1,0.5)`: A733 Excel #REF!, ours #VALUE!; cells (1): A733
- gen-m7/2d9b48c6363e7a16.case.json A2333 `=TEXTSPLIT((REPLACE("abc",2,Data!A$1,"")&(Data!C8&Data!B6)),Data!A1:B2,Data!D2:D8,((1.005/Data!C6)>(Data!A7<"1.5")),COUNTIFS(Data!C6:C7,"<5"),OFFSET(Data!C1:C12,0,(Data!$E$11/1),3))`: A2333 Excel #DIV/0!, ours #VALUE!; cells (1): A2333
- gen-m7/2d9b48c6363e7a16.case.json A3261 `=TEXTSPLIT(((Data!B9&Data!A9)&("?b"&Data!A10)),Data!A3:D7,",",(Data!D7<=ISNUMBER(VALUE("x"))),TOCOL(Data!A11:D12),SEQUENCE(4,((-1)+Data!$C5),1e-15,FILTER(Data!C1:E12,(Data!C1:C12=(-2.5)))))`: A3262 Excel #VALUE!, ours <empty>; cells (7): A3262, A3263, A3264, A3265, A3266, A3267, A3268
- gen-m7/4dac39f89709e2fc.case.json A3213 `=TEXTSPLIT(REDUCE((Data!D1^7),(Data!D3<Data!B$11),LAMBDA(x,acc,VSTACK(x,acc))),Data!A6:C12,Data!D8:E9,LOOKUP(0,{1;2;3;4;5},Data!A2:A11),1,FACT(15))`: A3213 Excel #N/A, ours #VALUE!; cells (2): A3213, A3214
- gen-m7/b2078b5d702c691a.case.json A356 `=TEXTSPLIT("ü",Data!B3:E6,OFFSET(Data!C3:D8,UNIQUE(Data!C6:D9,FALSE,Data!E4),0),HLOOKUP(Data!A12,Data!D7:E7,4,TRUE),1,(-0.0001))`: A356 Excel #N/A, ours #VALUE!; cells (1): A356
- gen-m7/bc4a69b0b512bc04.case.json A538 `=TEXTSPLIT((MAP({1,#N/A,3},LAMBDA(x,x&""))&(" x "&Data!B5)),",",Data!B4:E6,FILTER(Data!A1:B12,(Data!C1:C12<>0.1)),1)`: C538 Excel #N/A, ours #VALUE!; cells (11): C538, C539, C540, C541, C542, C543, C544, C545, C546, C547, C548
- gen-m7/f7e94a817e5385ac.case.json A2186 `=TEXTSPLIT((Data!D$12&BYCOL(Data!E5:E6,LAMBDA(x,x))),Data!C1:C5,Data!B10:C11,XOR(INDEX(Data!A5:B6,0.2),(Data!$E4<Data!$B1),Data!B1:B10),Data!E6,(COUNT(Data!A11:A11,Data!D5,Data!B11:B12)/BYCOL(Data!B9:D12,LAMBDA(x,NA()))))`: A2186 Excel #CALC!, ours #VALUE!; cells (1): A2186

## Acceptance criteria
- [x] Visible parity: the 41 gen-m7 cells listed under Context (17 from T-2225, 24 from T-2254) have Excel's value, type, error and text: these 41 known-failure lines disappear from eval/baseline/gen-m7.txt (`make eval-baseline`), with no new known-failure lines in any baseline.
- [x] The docs/excel-semantics entry cited above states the rule and its "Ours:" line names this task's change.
- [x] Unit tests cover the rule on a small hand-written input (one error or edge case and one normal case that must keep passing).
- [x] `make check` green; reviewers approve.

## Out of scope
- TEXTBEFORE/TEXTAFTER (T-2314).

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off (a wrong type, error, text or value), so this task is in the campaign. Its target is that every cell it owns has Excel's type, error and text, or is within half a unit of the 15th significant digit of Excel's value (an expected 0 exactly 0); the visible-parity criterion under Acceptance criteria states it.

## Outcome
TEXTSPLIT now checks its arguments in Excel's order: the text, a delimiter that is itself an error, ignore_empty, match_mode, an array
pad_with (`#VALUE!` per element, no longer lifted with the call), an error element of a delimiter list, then its own empty-delimiter,
no-delimiter and empty-text `#VALUE!`; a split that ignore_empty empties is `#CALC!`. The order was settled with a new probe,
probes/textsplit-error-order A1:A18 (excel-graph, 2026-10-23): a scalar error delimiter beats ignore_empty, but an error element of a
list loses to it (A1-A3, A13, A14), which the gen-m7 cells alone could not separate. text.md "TEXTSPLIT: argument errors first, an
all-empty result" states the order; the pad_with order the lift entry left UNVERIFIED is settled.
Eval delta: gen-m7 fixed 27 (every owned line, 637 -> 610), gen-m9 fixed 4 (TEXTSPLIT cells in 530c636749a85209, 68d8ffb69ff739d8,
8dd2312c96effb1b, f34f66fb7f81e673), probes new case 18/18, regressions 0.
