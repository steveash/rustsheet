---
id: T-2316
title: "SEQUENCE and TAKE: an argument error beats a zero rows or columns #CALC! (gen-m7)"
milestone: M7
lane: functions-math
status: todo
priority: P2
size: S
tier: opus
campaign: visible
depends_on: []
---

## Goal
SEQUENCE with 0 rows or columns, and TAKE with 0 rows, return an error in another argument (start, step, columns) instead of #CALC!, as Excel gives, also per lifted element.

## Context
Found by the T-2225 triage of eval/suites/gen-m7 (38 cases, 10,991 Excel-scored cells, excel-graph). Quirk entry: docs/excel-semantics/arrays.md "SEQUENCE and TAKE: an argument error before zero rows".

Evidence: `SEQUENCE(0,4,UNIQUE(..),FLOOR.MATH(..)-..)` is `#VALUE!` (07d27aa0add0247f A817), `SEQUENCE(0,1,LOOKUP(NETWORKDAYS(..)..),TOCOL(Data!E7:E11))` spills 5 `#NUM!` (36809e9faa59553f A334), `SEQUENCE(WRAPROWS(..)+..,0,..)` is `#VALUE!` (611ce3bbfab27cef A1065), `SEQUENCE(0,1,ROUNDDOWN(SORT(..),-6),COLUMNS(COTH(..)))` has `#VALUE!` elements (611ce3bbfab27cef A1633) and `TAKE(Data!B9:C10,0,LN(ROUND(..)))` is `#VALUE!` (7d67c559ce7536fa A1296); ours `#CALC!` everywhere. arrays.md has `SEQUENCE(0)` as `#CALC!` (keep it). Code: functions/math_and_trigonometry/sequence.rs, functions/lookup_and_reference/drop_take.rs (the lane of the main module, SEQUENCE).

Visibly-off cells (gap triage 2026-10-21, decision 0083): 10 cells, every one with a wrong type, error, text or value (none is a last-digit difference).

Owned gen-m7 cells (T-2225 triage, excel-graph, 2026-10-21): 10 known-failure lines in eval/baseline/gen-m7.txt, 5 formulas. One line per formula: case, anchor, the anchor formula, Excel vs ours at the first failing cell, then every failing cell (sheet Calc; a range is consecutive cells of one row).

- gen-m7/07d27aa0add0247f.case.json A817 `=SEQUENCE(0,4,UNIQUE((Data!B9-Data!B3),XLOOKUP(Data!C2,Data!B2:B11,Data!A3:D7,"TRUE"),Data!C2),(FLOOR.MATH(Data!E$4,1000000000000000.0,123456789.123)-(Data!D$5^(-0.0001))))`: A817 Excel #VALUE!, ours #CALC!; cells (1): A817
- gen-m7/36809e9faa59553f.case.json A334 `=SEQUENCE(0,1,LOOKUP(NETWORKDAYS(Data!E7,28565),Data!C10:C10),TOCOL(Data!E7:E11))`: A334 Excel #NUM!, ours #CALC!; cells (5): A334, A335, A336, A337, A338
- gen-m7/611ce3bbfab27cef.case.json A1065 `=SEQUENCE((WRAPROWS(Data!C2,3,Data!$A$3)+(Data!$B2/Data!C12)),0,(ROUNDDOWN(2958465,4)+(Data!E8/2.5)))`: A1065 Excel #VALUE!, ours #CALC!; cells (1): A1065
- gen-m7/611ce3bbfab27cef.case.json A1633 `=SEQUENCE(0,1,ROUNDDOWN(SORT(Data!B6:C12,1,1),-6),COLUMNS(COTH(Data!$A$1)))`: B1634 Excel #VALUE!, ours #CALC!; cells (2): B1634, B1636
- gen-m7/7d67c559ce7536fa.case.json A1296 `=TAKE(Data!B9:C10,0,LN(ROUND(Data!B$9,Data!B3)))`: A1296 Excel #VALUE!, ours #CALC!; cells (1): A1296

## Acceptance criteria
- [ ] Visible parity: the 10 gen-m7 cells listed under Context have Excel's value, type, error and text: these 10 known-failure lines disappear from eval/baseline/gen-m7.txt (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] The docs/excel-semantics entry cited above states the rule and its "Ours:" line names this task's change.
- [ ] Unit tests cover the rule on a small hand-written input (one error or edge case and one normal case that must keep passing).
- [ ] `make check` green; reviewers approve.

## Out of scope
- 1x1 array arguments (T-2298).

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off (a wrong type, error, text or value), so this task is in the campaign. Its target is that every cell it owns has Excel's type, error and text, or is within half a unit of the 15th significant digit of Excel's value (an expected 0 exactly 0); the visible-parity criterion under Acceptance criteria states it.
