---
id: T-2519
title: "One-cell TAKE/DROP of a range under +0 or N() is a scalar, not a lifting 1x1 array (OFFSET height/width/rows)"
milestone: M7
lane: functions-lookup
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
`ROWS(OFFSET(B9:C12,5,1,DROP(H2:H4,-2,0)+0,2))` and the same with `N(..)` or TAKE are 3 as in Excel (ours: 1, a lifted 1x1).

## Context
Evidence: probes/t2454-offset-one-cell-array-args A5:D5, A9:D9 and probes/t2454-offset-array-height A12:D12 (excel-graph, 2026-10-23). Excel's DROP/TAKE of a range is a reference, so a one-cell result under `+0` or `N()` is a scalar, while `{3}+0` and SEQUENCE(1,1,3)+0 stay lifting 1x1 arrays. Ours evaluates DROP/TAKE in value context to a 1x1 `CalcResult::Array` (drop_take.rs `take_drop_reference`, value path), which OFFSET then lifts. T-2454 fixed the bare form by reading OFFSET's numbers as references (`offset_values`).

T-2436 probe (excel-graph, 2026-10-24; probes/t2436/<case>, each gen-m9 cell's sub-expressions on its Data sheet): a one-cell TAKE is a scalar, so an error through it is not a 1x1 array: `ROWS(Data!E9-TAKE(Data!E7,2,42))` and `ROWS(DECIMAL(TAKE(Data!$B11,-2),MATCH(..)))` are the errors (`#VALUE!`, `#N/A`; ours 1), and `CHITEST(<that #VALUE!>,..)`, `INTERCEPT(Data!B4:B4,<that #N/A>)` and `SUMXMY2(TAKE(<blank>,-1,1),Data!B6:C10)` give `#VALUE!`, `#N/A` and `#VALUE!` (ours `#N/A`, `#VALUE!`, `#N/A`).
Owned gen-m9 cells (T-2436 probe, excel-graph, 2026-10-24): 3 known-failure lines in eval/baseline/gen-m9.txt, 3 formulas.
Examples:
- gen-m9/7674c89c824aca43 Calc!A305 `=CHITEST((Data!E$9-TAKE(Data!E$7,2,42)),Data!C8:C10)`: Excel #VALUE!, ours #N/A; inner `(Data!E$9-TAKE(Data!E$7,2,42))` Excel #VALUE!, ours #VALUE!
- gen-m9/9cca17438a7e6f8a Calc!A171 `=INTERCEPT(Data!B4:B4,DECIMAL(TAKE(Data!$B11,-2),MATCH(Data!E12,Data!A4:C11)))`: Excel #N/A, ours #VALUE!; inner `DECIMAL(TAKE(Data!$B11,-2),MATCH(Data!E12,Data!A4:C11))` Excel #N/A, ours #N/A
- gen-m9/7c20351562a2d71d Calc!A277 `=CONFIDENCE.NORM(SUMXMY2(TAKE(Data!C11,-1,1),Data!B6:C10),1E-300,FISHERINV((0.1/Data!D3)))`: Excel #VALUE!, ours #N/A; inner `SUMXMY2(TAKE(Data!C11,-1,1),Data!B6:C10)` Excel #VALUE!, ours #N/A
Cells:
- gen-m9/7674c89c824aca43.case.json (1): Calc!A305
- gen-m9/9cca17438a7e6f8a.case.json (1): Calc!A171
- gen-m9/7c20351562a2d71d.case.json (1): Calc!A277

Exit owners (decision 0096; the 2026-10-27 planner run names each cell the ranges above hold):
- probes/t2454-offset-one-cell-array-args Sheet1!C5, Sheet1!C9
- probes/t2454-offset-array-height Sheet1!B12

## Acceptance criteria
- [ ] ~~Visible parity (gen-m9, T-2436)~~ moved to T-3072: the 3 gen-m9 cells need the arithmetic path, not OFFSET.
- [x] The cells listed above pass and leave eval/baseline/probes.txt; `{3}+0` and SEQUENCE rows keep passing.
- [x] `make check` green; reviewers approve.

Also probe `CHOOSE(1,DROP(H2:H4,-2,0))` and `LET(x,DROP(H2:H4,-2,0),x)` as OFFSET's height, and find the mechanism of the scalar height 1E-15 giving 2 rows (lookup.md, probes/t2454-offset-array-height A1).

## Out of scope
- Arguments the probes did not cover.

## Outcome
`take_drop_reference` (drop_take.rs) now returns the one-cell reference in value context, so a one-cell TAKE/DROP of a single-sheet range under `+0`/`N()` is a scalar. probes known failures 2564 -> 2551 (13 cells fixed: all of t2454-offset-array-height and t2454-offset-one-cell-array-args), no regressions. The gen-m9 cells are not fixed by this (the binary-operator path still lifts a 1x1 array): moved to T-3072. Not done: CHOOSE/LET as OFFSET's height and the 1E-15 height mechanism probes (need the oracle).
