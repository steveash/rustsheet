---
id: T-2307
title: "SCAN and REDUCE give the LAMBDA a reference initial_value and range elements as references (gen-m7)"
milestone: M7
lane: functions-logic
status: done
priority: P2
size: S
tier: opus
campaign: visible
depends_on: [T-2306]
---

## Goal
SCAN and REDUCE give the LAMBDA a reference when initial_value is a reference and each element of a range array as a reference to its cell, so functions that skip text and booleans in references (PRODUCT, COUNT, SUM) skip them, as Excel does.

## Context
Found by the T-2225 triage of eval/suites/gen-m7 (38 cases, 10,991 Excel-scored cells, excel-graph). Quirk entry: docs/excel-semantics/arrays.md "SCAN and REDUCE keep errors in the LAMBDA".

Root cause: `fn_scan` evaluates initial_value with `evaluate_node_in_context` (a value) and `lambda_cells(.., false)` reads the array as values; `fn_reduce` likewise. Excel's results show references: `REDUCE(Data!A12,Data!C11,PRODUCT)` with A12 and C11 both TRUE is 0 (PRODUCT of two references to booleans; ours 1, the direct-value PRODUCT; 18b80d3fd292d63a A467), `REDUCE(Data!C7,Data!E12,COUNT)` with C7 the text "1" and E12 -2.5 is 1 (ours 2; 2c5ea078f25443a2 A1767), and `SCAN(Data!A5,Data!A7:C11,PRODUCT)` with A5 the text "ü" starts at 2.5 = PRODUCT(<ref "ü">,2.5) (ours `#VALUE!`; 18b80d3fd292d63a A285, which also needs T-2306 for its later steps). Which of initial_value and the elements is a reference is not split apart by these cells (A467 and A1767 have one-cell arrays): probe `REDUCE(Z1,{1},COUNT)`-style cells (excel-probe) or read b24ce74f08d3f91a A714 (`SUM(0.2,z)+z` over a range, T-2306's list) before choosing, and record the rule in arrays.md.

Visibly-off cells (gap triage 2026-10-21, decision 0083): 17 cells, every one with a wrong type, error, text or value (none is a last-digit difference).

Owned gen-m7 cells (T-2225 triage, excel-graph, 2026-10-21): 17 known-failure lines in eval/baseline/gen-m7.txt, 3 formulas. One line per formula: case, anchor, the anchor formula, Excel vs ours at the first failing cell, then every failing cell (sheet Calc; a range is consecutive cells of one row).

- gen-m7/18b80d3fd292d63a.case.json A285 `=SCAN(Data!A5,Data!A7:C11,_xleta.PRODUCT)`: A285 Excel 2.5, ours #VALUE!; cells (15): A285:C285, A286:C286, A287:C287, A288:C288, A289:C289
- gen-m7/18b80d3fd292d63a.case.json A467 `=REDUCE(Data!A12,Data!C11,_xleta.PRODUCT)`: A467 Excel 0.0, ours 1.0; cells (1): A467
- gen-m7/2c5ea078f25443a2.case.json A1767 `=REDUCE(Data!C7,Data!E12,_xleta.COUNT)`: A1767 Excel 1.0, ours 2.0; cells (1): A1767

## Acceptance criteria
- [x] Visible parity: the 17 gen-m7 cells listed under Context have Excel's value, type, error and text: these 17 known-failure lines disappear from eval/baseline/gen-m7.txt (`make eval-baseline`), with no new known-failure lines in any baseline.
- [x] The docs/excel-semantics entry cited above states the rule and its "Ours:" line names this task's change.
- [x] Unit tests cover the rule on a small hand-written input (one error or edge case and one normal case that must keep passing).
- [x] `make check` green; reviewers approve.

## Out of scope
- Error short-circuiting (T-2306); MAP and BYROW argument kinds.

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off (a wrong type, error, text or value), so this task is in the campaign. Its target is that every cell it owns has Excel's type, error and text, or is within half a unit of the 15th significant digit of Excel's value (an expected 0 exactly 0); the visible-parity criterion under Acceptance criteria states it.

## Outcome
- `fn_scan` and `fn_reduce` now take initial_value through `Model::lambda_initial_value` (map_reduce.rs). It uses `eval_ref_or_index`, as a LAMBDA call binds its argument, so a single-cell reference stays a reference. A union is read as before.
- No new probe was needed to split the rule. A285 and A1767 need a reference initial value, and A467 needs a reference element. Range elements, a one-cell range included, were already references (`lambda_cells`, T-1442). Recorded in arrays.md "SCAN and REDUCE keep errors in the LAMBDA".
- Eval delta:
  - gen-m7: 990 → 973 known failures (the 17 owned lines).
  - Regressions: 0 in every suite.
