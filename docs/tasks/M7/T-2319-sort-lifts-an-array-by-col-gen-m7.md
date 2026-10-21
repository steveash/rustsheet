---
id: T-2319
title: "SORT lifts an array by_col (gen-m7 7307132bb8820f17 A1076 is a 2x4 #VALUE!)"
milestone: M7
lane: functions-lookup
status: todo
priority: P2
size: S
tier: opus
campaign: visible
depends_on: []
---

## Goal
SORT lifts an array by_col (one call per element), so `SORT(Data!D6:E7,3,1E-15,<2x4 boolean array>)` is a 2x4 #VALUE! grid, as Excel gives, instead of #N/IMPL!.

## Context
Found by the T-2225 triage of eval/suites/gen-m7 (38 cases, 10,991 Excel-scored cells, excel-graph). Quirk entry: docs/excel-semantics/arrays.md "TOROW, TOCOL, SORT and TEXTJOIN lifted arguments (gen-m7)".

Evidence: `SORT(Data!D6:E7,3,1e-15,(HSTACK(Data!B5:D6,Data!E2)>Data!D$11))` spills 8 `#VALUE!` (sort_index 3 is past the 2 columns; 7307132bb8820f17 A1076); ours `#N/IMPL!`. T-1837 made SORT read array sort_index/sort_order lists; by_col is the remaining lifted argument. Code: `fn_sort` in functions/spill_functions.rs.

Visibly-off cells (gap triage 2026-10-21, decision 0083): 8 cells, every one with a wrong type, error, text or value (none is a last-digit difference).

Owned gen-m7 cells (T-2225 triage, excel-graph, 2026-10-21): 8 known-failure lines in eval/baseline/gen-m7.txt, 1 formulas. One line per formula: case, anchor, the anchor formula, Excel vs ours at the first failing cell, then every failing cell (sheet Calc; a range is consecutive cells of one row).

- gen-m7/7307132bb8820f17.case.json A1076 `=SORT(Data!D6:E7,3,1e-15,(HSTACK(Data!B5:D6,Data!E2)>Data!D$11))`: A1076 Excel #VALUE!, ours #N/IMPL!; cells (8): A1076:D1076, A1077:D1077

## Acceptance criteria
- [ ] Visible parity: the 8 gen-m7 cells listed under Context have Excel's value, type, error and text: these 8 known-failure lines disappear from eval/baseline/gen-m7.txt (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] The docs/excel-semantics entry cited above states the rule and its "Ours:" line names this task's change.
- [ ] Unit tests cover the rule on a small hand-written input (one error or edge case and one normal case that must keep passing).
- [ ] `make check` green; reviewers approve.

## Out of scope
- SORTBY (T-1839, T-2310).

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off (a wrong type, error, text or value), so this task is in the campaign. Its target is that every cell it owns has Excel's type, error and text, or is within half a unit of the 15th significant digit of Excel's value (an expected 0 exactly 0); the visible-parity criterion under Acceptance criteria states it.
