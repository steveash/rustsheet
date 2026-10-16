---
id: T-1664
title: "BYROW whose function returns an array is one #CALC! cell, not a spill"
milestone: M2
lane: engine
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
`BYROW(A1:B3,_xleta.ABS)` and `BYROW(A1:B3,_xleta.IF)` give one `#CALC!` cell as Excel does, with nothing spilled below.

## Context
Found by the T-1660 spill-extent check: in probes/eta-reduced-names, E49 `=BYROW(A1:B3,_xleta.ABS)` and E101 `=BYROW(A1:B3,_xleta.IF)` are a single `#CALC!` in Excel (no E50, E51, E102, E103 in the sidecar), while the engine spills a 3x1 result of errors (E50/E51 `#VALUE!`, E102/E103 `#ERROR!`). Known failures `Sheet1!E50`, `E51`, `E102`, `E103`. docs/excel-semantics/arrays.md (eta-reduced names): "A function whose result is an array for a row is `#CALC!` in BYROW". gen-m9/b5352ebdb9c5f1a1 Calc!A40 `=ISBLANK(LEN(BYROW(Data!C11:D12,LAMBDA(v,v))))` is one FALSE in Excel while the engine spills into A41 (gen-m9 is not gated). Code: the BYROW/BYCOL implementation (rg `fn_byrow`).

## Acceptance criteria
- [x] probes/eta-reduced-names Sheet1!E49, E50, E51 pass; baseline shrinks; no regressions. E101-E103 (`_xleta.IF` with one argument) are not met here: the engine's IF gives `#ERROR!` per row and never returns an array, so they stay with T-1511.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
BYROW and BYCOL give one `#CALC!` cell, nothing spilled, when the function returns an array or multi-cell reference (`CalcResult::is_multi_value`). Eval: probes known failures 5479 -> 5473 (6 fixed), no regressions. Unit tests `byrow_bycol_array_result_is_one_calc_error`, `byrow_one_by_one_array_result_is_unchanged`. Also passes: probes/m7-argument-modes F45:H45; gen-m9 b5352ebdb9c5f1a1 A40 and 4df1b22d3ed63f58 A35 (not gated). Not fixed: E101-E103 (one-argument `_xleta.IF`, T-1511) and gen-m9/95efb37a9874f82e A19 (`#SPILL!`; FISHERINV/RADIANS probably do not lift over a range argument). BYCOL, array-constant sources, 1x1 results and mixed rows are unprobed: T-1669.
