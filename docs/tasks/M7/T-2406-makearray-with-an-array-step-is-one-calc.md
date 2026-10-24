---
id: T-2406
title: "MAKEARRAY with an array step is one #CALC!; BYROW, BYCOL and MAP with a LAMBDA step are #CALC! per element (gen-m7)"
milestone: M7
lane: functions-logic
status: done
priority: P1
size: S
tier: opus
depends_on: []
campaign: visible
---

## Goal
Two LAMBDA-helper result rules, as Excel gives them: MAKEARRAY whose LAMBDA returns a multi-cell array (or a 1x1 array literal such
as `{1,2}` for a 1x1 call) for any cell is one `#CALC!` with nothing spilled; and BYROW, BYCOL and MAP whose LAMBDA returns a LAMBDA
(an uncalled function value) keep their shape with `#CALC!` in each such element.

## Context
Gap triage of the T-2254 gen-m7 sweep (excel-graph, 2026-10-22); 164 cells, 38 formulas.
- MAKEARRAY (20 formulas, 122 cells). `MAKEARRAY(2,2,LAMBDA(acc,z,{1,2}))` is one `#CALC!` in A39 with A40:B40 empty
  (0624b684889e9228); so are `VSTACK(r,c)`, `HSTACK({1,2,3})+..` and `HLOOKUP(..)*SORT(..)` steps, and `MAKEARRAY(1,1,LAMBDA(x,acc,{1,2}))`
  (23b5367c18059af2 A3493, one cell: the step is the problem, not the result's size). Ours keeps the grid and puts `#VALUE!` in every
  element. BYROW and BYCOL already make an array step one `#CALC!` (arrays.md "BYROW and BYCOL: a LAMBDA result that is an array",
  T-1664/T-1670: `CalcResult::is_multi_value`, `makes_calc_error` in functions/logical/bycol_byrow.rs), and MAP and SCAN do for a
  multi-cell step (T-2306/T-2309, `lambda_result_value`); `fn_makearray` (same file) does not. Whether MAKEARRAY unwraps a 1x1 array
  step as BYROW does is not shown by these cells (none returns a 1x1 array); follow BYROW's rule and say so in the "Ours:" line.
  Two formulas see the `#CALC!` through another call: `REGEXEXTRACT(HYPERLINK(SORT(..),..),MAKEARRAY(1,2,LAMBDA(y,x,{1,2})),2)` spills 18
  `#CALC!` (the text lifts, f19581ac44da1653 A465; ours `#VALUE!`/`#N/A`) and TEXTSPLIT's ignore_empty `MAKEARRAY(4,3,LAMBDA(x,w,VSTACK(x,w)))`
  is one `#CALC!` (21b4eef18424ce94 A1661; ours lifts over the 4x3 grid). Both pass once MAKEARRAY gives a scalar `#CALC!` (checked by
  hand with `NA()` in its place). 2d9b48c6363e7a16 A3993 is a MAKEARRAY over `LN(BYROW(..,LAMBDA(z,LAMBDA(v,v))))`, an array step either way.
- BYROW, BYCOL, MAP with a LAMBDA step (18 formulas, 42 cells). `BYCOL(Data!A11:D11,LAMBDA(x,LAMBDA(v,v)))` is four `#CALC!`
  (bc4a69b0b512bc04 A294), `MAP(Data!D1:E4,LAMBDA(acc,LAMBDA(w,w)))` eight (bc4a69b0b512bc04 A2102), `BYROW({"",0,FALSE},LAMBDA(acc,LAMBDA(x,x)))`
  one, over ranges, arrays, a single cell and an empty cell alike. Ours gives `#VALUE!` per element, since an array element cannot hold a
  LAMBDA (T-2039 is the general "arrays of LAMBDA values" work; a cell shows an uncalled LAMBDA as `#CALC!`, arrays.md "An uncalled LAMBDA
  is one value"). Making the element `#CALC!` matches every cell here without T-2039; keep the change compatible with it.
Quirk entries: docs/excel-semantics/arrays.md "MAKEARRAY with an array step; BYROW, BYCOL and MAP with a LAMBDA step" (added by this triage).

Owned gen-m7 cells (T-2254 triage, excel-graph, 2026-10-22): 164 known-failure lines in eval/baseline/gen-m7.txt, 38 formulas.
- gen-m7/0624b684889e9228.case.json A39 `=MAKEARRAY(2,2,LAMBDA(acc,z,{1,2}))`: A39 Excel #CALC!, ours #VALUE!; cells (4): A39:B39, A40:B40
- gen-m7/21b4eef18424ce94.case.json A1661 `=TEXTSPLIT((TRIM(Data!D$3)&Data!B1),Data!A9:A11,Data!D8:E8,MAKEARRAY(4,3,LAMBDA(x,w,VSTACK(x,w))),0)`: A1661 Excel #CALC!, ours #VALUE!; cells (12): A1661:C1661, A1662:C1662, A1663:C1663, A1664:C1664
- gen-m7/23b5367c18059af2.case.json A245 `=MAKEARRAY(2,3,LAMBDA(y,v,VSTACK(y,v)))`: A245 Excel #CALC!, ours #VALUE!; cells (6): A245:C245, A246:C246
- gen-m7/23b5367c18059af2.case.json A1869 `=MAKEARRAY(1,2,LAMBDA(z,v,VSTACK(z,v)))`: A1869 Excel #CALC!, ours #VALUE!; cells (2): A1869:B1869
- gen-m7/23b5367c18059af2.case.json A3029 `=MAKEARRAY(2,3,LAMBDA(v,x,VSTACK(v,x)))`: A3029 Excel #CALC!, ours #VALUE!; cells (6): A3029:C3029, A3030:C3030
- gen-m7/23b5367c18059af2.case.json A3493 `=MAKEARRAY(1,1,LAMBDA(x,acc,{1,2}))`: A3493 Excel #CALC!, ours #VALUE!; cells (1): A3493
- gen-m7/2c156ba98ef0220a.case.json A1429 `=MAKEARRAY(3,3,LAMBDA(w,z,{1,2}))`: A1429 Excel #CALC!, ours #VALUE!; cells (9): A1429:C1429, A1430:C1430, A1431:C1431
- gen-m7/2d9b48c6363e7a16.case.json A3993 `=MAKEARRAY(3,1,LAMBDA(w,acc,LN(BYROW(EXPAND(1,2,2,""),LAMBDA(z,LAMBDA(v,v))))))`: A3993 Excel #CALC!, ours #VALUE!; cells (3): A3993, A3994, A3995
- gen-m7/7ea6ef6ceabacafd.case.json A1079 `=MAKEARRAY(FACT(1/3),2,LAMBDA(w,y,VSTACK(w,Data!E9:E11,Data!B11:C12)))`: A1079 Excel #CALC!, ours #VALUE!; cells (2): A1079:B1079
- gen-m7/7ea6ef6ceabacafd.case.json A1128 `=MAKEARRAY(3,1,LAMBDA(x,w,{1,2}))`: A1128 Excel #CALC!, ours #VALUE!; cells (3): A1128, A1129, A1130
- gen-m7/7ea6ef6ceabacafd.case.json A2010 `=MAKEARRAY(4,3,LAMBDA(y,z,{1,2}))`: A2010 Excel #CALC!, ours #VALUE!; cells (12): A2010:C2010, A2011:C2011, A2012:C2012, A2013:C2013
- gen-m7/7ea6ef6ceabacafd.case.json A2402 `=MAKEARRAY(1,1,LAMBDA(v,z,{1,2}))`: A2402 Excel #CALC!, ours #VALUE!; cells (1): A2402
- gen-m7/7ea6ef6ceabacafd.case.json A3235 `=MAKEARRAY(3,3,LAMBDA(y,acc,VSTACK(y,acc)))`: A3235 Excel #CALC!, ours #VALUE!; cells (9): A3235:C3235, A3236:C3236, A3237:C3237
- gen-m7/7ea6ef6ceabacafd.case.json A3578 `=MAKEARRAY(3,1,LAMBDA(y,w,VSTACK(y,w)))`: A3578 Excel #CALC!, ours #VALUE!; cells (3): A3578, A3579, A3580
- gen-m7/b2078b5d702c691a.case.json A1625 `=MAKEARRAY(3,4,LAMBDA(z,y,VSTACK(z,y)))`: A1625 Excel #CALC!, ours #VALUE!; cells (12): A1625:D1625, A1626:D1626, A1627:D1627
- gen-m7/b2078b5d702c691a.case.json A2785 `=MAKEARRAY(3,1,LAMBDA(z,x,(HLOOKUP("#N/A",Data!A1:D4,4,z)*SORT(Data!D6:D12))))`: A2785 Excel #CALC!, ours #VALUE!; cells (3): A2785, A2786, A2787
- gen-m7/bc4a69b0b512bc04.case.json A1956 `=MAKEARRAY(2,1,LAMBDA(v,x,VSTACK(v,x)))`: A1956 Excel #CALC!, ours #VALUE!; cells (2): A1956, A1957
- gen-m7/f19581ac44da1653.case.json A465 `=REGEXEXTRACT(HYPERLINK(SORT(Data!B4:D9),SQRTPI(Data!$C$5)),MAKEARRAY(1,COLUMNS(Data!B9:C11),LAMBDA(y,x,{1,2})),2)`: A465 Excel #CALC!, ours #VALUE!; cells (18): A465:C465, A466:C466, A467:C467, A468:C468, A469:C469, A470:C470
- gen-m7/f19581ac44da1653.case.json A2417 `=MAKEARRAY(3,2,LAMBDA(v,x,{1,2}))`: A2417 Excel #CALC!, ours #VALUE!; cells (6): A2417:B2417, A2418:B2418, A2419:B2419
- gen-m7/f7e94a817e5385ac.case.json A208 `=MAKEARRAY(4,2,LAMBDA(w,x,(HSTACK({1,2,3})+(x*Data!B6))))`: A208 Excel #CALC!, ours #VALUE!; cells (8): A208:B208, A209:B209, A210:B210, A211:B211
- gen-m7/0624b684889e9228.case.json A3469 `=BYCOL(ROUNDUP(2^31,(2/Data!D5)),LAMBDA(x,LAMBDA(w,w)))`: A3469 Excel #CALC!, ours #VALUE!; cells (1): A3469
- gen-m7/21b4eef18424ce94.case.json A465 `=MAP(EXPAND(1,2,2,""),LAMBDA(v,LAMBDA(z,z)))`: A465 Excel #CALC!, ours #VALUE!; cells (4): A465:B465, A466:B466
- gen-m7/21b4eef18424ce94.case.json A3078 `=BYROW({"",0,FALSE},LAMBDA(acc,LAMBDA(x,x)))`: A3078 Excel #CALC!, ours #VALUE!; cells (1): A3078
- gen-m7/2c156ba98ef0220a.case.json A1149 `=BYROW(EXPAND(1,2,2,""),LAMBDA(y,LAMBDA(v,v)))`: A1149 Excel #CALC!, ours #VALUE!; cells (2): A1149, A1150
- gen-m7/2c156ba98ef0220a.case.json A2297 `=BYCOL(Data!A3:A8,LAMBDA(w,LAMBDA(y,y)))`: A2297 Excel #CALC!, ours #VALUE!; cells (1): A2297
- gen-m7/2d9b48c6363e7a16.case.json A1917 `=BYROW(Data!E8:E12,LAMBDA(x,LAMBDA(w,w)))`: A1917 Excel #CALC!, ours #VALUE!; cells (5): A1917, A1918, A1919, A1920, A1921
- gen-m7/2d9b48c6363e7a16.case.json A3309 `=BYROW(IF({1,0,1},1,NA()),LAMBDA(w,LAMBDA(x,x)))`: A3309 Excel #CALC!, ours #VALUE!; cells (1): A3309
- gen-m7/9bb61f6ae34c7e20.case.json A161 `=BYCOL(Data!C8:C11,LAMBDA(v,LAMBDA(y,y)))`: A161 Excel #CALC!, ours #VALUE!; cells (1): A161
- gen-m7/9bb61f6ae34c7e20.case.json A3323 `=BYROW(Data!Z1,LAMBDA(y,LAMBDA(w,w)))`: A3323 Excel #CALC!, ours #VALUE!; cells (1): A3323
- gen-m7/b2078b5d702c691a.case.json A1454 `=BYROW(EXPAND(1,2,2,""),LAMBDA(x,LAMBDA(v,v)))`: A1454 Excel #CALC!, ours #VALUE!; cells (2): A1454, A1455
- gen-m7/b2078b5d702c691a.case.json A3921 `=BYCOL(Data!B6:C8,LAMBDA(x,LAMBDA(acc,acc)))`: A3921 Excel #CALC!, ours #VALUE!; cells (2): A3921:B3921
- gen-m7/bc4a69b0b512bc04.case.json A294 `=BYCOL(Data!A11:D11,LAMBDA(x,LAMBDA(v,v)))`: A294 Excel #CALC!, ours #VALUE!; cells (4): A294:D294
- gen-m7/bc4a69b0b512bc04.case.json A942 `=MAP(Data!Z1,LAMBDA(x,LAMBDA(y,y)))`: A942 Excel #CALC!, ours #VALUE!; cells (1): A942
- gen-m7/bc4a69b0b512bc04.case.json A2102 `=MAP(Data!D1:E4,LAMBDA(acc,LAMBDA(w,w)))`: A2102 Excel #CALC!, ours #VALUE!; cells (8): A2102:B2102, A2103:B2103, A2104:B2104, A2105:B2105
- gen-m7/f19581ac44da1653.case.json A671 `=BYROW(Data!C8:E10,LAMBDA(z,LAMBDA(y,y)))`: A671 Excel #CALC!, ours #VALUE!; cells (3): A671, A672, A673
- gen-m7/f19581ac44da1653.case.json A1599 `=BYROW(Data!C11:E11,LAMBDA(z,LAMBDA(x,x)))`: A1599 Excel #CALC!, ours #VALUE!; cells (1): A1599
- gen-m7/f7e94a817e5385ac.case.json A697 `=BYCOL({"",0,FALSE},LAMBDA(z,LAMBDA(w,w)))`: A697 Excel #CALC!, ours #VALUE!; cells (3): A697:C697
- gen-m7/7ea6ef6ceabacafd.case.json A3260 `=BYCOL(Data!E8:E8,LAMBDA(acc,LAMBDA(z,z)))`: A3260 Excel #CALC!, ours #VALUE!; cells (1): A3260

## Acceptance criteria
- [x] Visible parity: these 164 known-failure lines disappear from eval/baseline/gen-m7.txt (`make eval-baseline`), with no new
      known-failure lines in any baseline.
- [x] Unit tests: a MAKEARRAY array step (one `#CALC!`, ROWS of it `#CALC!`), a scalar step (unchanged), and BYCOL/MAP with a LAMBDA step
      (`#CALC!` per element, shape kept).
- [x] The arrays.md entry's "Ours:" line names this task.
- [x] `make check` green; reviewers approve.

## Out of scope
- Arrays that hold LAMBDA values MAP can call (T-2039).

## Visible-parity stage
Decision 0083 (gap triage 2026-10-22, T-2254): every cell this task owns has the wrong type, error, text or value (none is a
last-digit difference), so the task is in the campaign. Its target is that every cell it owns has Excel's type, error and text,
or is within half a unit of the 15th significant digit of Excel's value; the visible-parity criterion under Acceptance criteria
states it.

## Handoff (resumed)
Superseded by the Outcome below: the LAMBDA-element half is done.
Done on the branch: MAKEARRAY with a multi-cell step is one `#CALC!` (`fn_makearray` uses `makes_calc_error`); 122 gen-m7 and
2 gen-m9 cells fixed, 0 regressions, baseline recorded. Not done: BYROW/BYCOL/MAP with a LAMBDA step. Making the element a plain
`#CALC!` error (in `lambda_result_to_array_node`/MAP) fixed the 42 cells but regressed gen-m7/23b5367c18059af2 A2578
(`TEXTSPLIT(..&MAP(Data!Z1,LAMBDA(acc,LAMBDA(v,v))),..)` is `#VALUE!`), gen-m7/f7e94a817e5385ac A942 and gen-m9/165db58d1540f014
A273: the element is a LAMBDA value, `#CALC!` as a cell result but `#VALUE!`/other when consumed. Needs a LAMBDA-valued array
element (T-2039) or a result-position-only rule. Keep this task open for that half (acceptance 1 is the 164 lines; 122 are fixed).

## Outcome
MAKEARRAY with a multi-cell LAMBDA step is one `#CALC!` (`makes_calc_error`, as BYROW/BYCOL). A LAMBDA element of BYROW, BYCOL and
MAP is `#CALC!` when the call is the cell formula's outermost node (superseded by T-2039: the element now holds the LAMBDA itself; `Model::lambda_elements_shown`, set by `run_cell_formula`, taken by
the call) and stays `#VALUE!` where consumed, which keeps the three cells the handoff found regressing. Eval: gen-m7 known failures
-164 (122 MAKEARRAY + 42 LAMBDA-element), gen-m9 -2, 0 regressions. `tier: opus` is from the sonnet handoff.
Follow-up: T-2463 (probe the element under ISERROR/IFERROR/ERROR.TYPE/TYPE, non-root positions, MAKEARRAY 1x1 and LAMBDA steps).
