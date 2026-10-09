---
id: T-0291
title: "IFERROR/IFNA of a blank cell is an empty value (FALSE in OR, 0 as a lookup value)"
milestone: M2
lane: functions-logic
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
IFERROR/IFNA of a blank cell gives Excel's empty value: FALSE in OR, "" in `&`, and still a match for 0 when it is a lookup value.

## Context
Split from T-0235 (item 1). Evidence, all excel-graph gen-m2: 7469c3e920e1ade4 Calc!A351 `=IF(Data!E12,OR(IFERROR(Data!D9,Data!A9)))` with D9 blank is FALSE; the engine gives `#VALUE!` because `if_error_like` (functions/logical/mod.rs) passes the blank through as `CalcResult::EmptyCell`, which OR ignores. Returning `CalcResult::EmptyArg` there fixes A351 but regresses ef5c55a81783b7d5 Calc!A291 `=NOT(VLOOKUP(IFERROR(Data!B8,Data!D1),Data!C8:D10,1,Data!E1))` (B8 blank; Excel FALSE, i.e. the approximate lookup finds -1 as for 0). `lookup_index` (functions/lookup_and_reference/mod.rs) does not match `EmptyArg`. Decide how an empty value reaches each consumer (OR/AND, lookups, `&`) and keep dd99be1f2454159f A110 and d0c29dd5c530e28c A112 passing.

## Acceptance criteria
- [x] 7469c3e920e1ade4 A351 leaves eval/baseline/gen-m2.txt with no regression (ef5c55a81783b7d5 A291 keeps passing).
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
`if_error_like` (IFERROR and IFNA) now returns `CalcResult::EmptyArg` for a blank value, so OR and AND read it as FALSE
and `&` as ""; `lookup_key` reads an empty value as 0 like a blank cell (the same rule as an omitted XLOOKUP
lookup_value), and `compare_values` reads it as a blank, so comparisons and SWITCH are unchanged from the blank-cell
behavior; an omitted argument now compares as a blank too, which probes/omitted-arg-compare (10 cells, excel-graph)
confirms (six of its cells were wrong before). The new probe probes/iferror-blank-value (23 cells, excel-graph) checks every consumer the change reaches
(comparisons, SWITCH, COUNT, COUNTA, AND, OR, `&`, IS*, TYPE, N, MATCH, VLOOKUP, IFNA): all pass but COUNTA (D8, 1 in Excel,
0 here and on main; T-1294). A351 now passes; A291, A110 and A112 still pass. Recorded in logic.md, "IFERROR / IFNA of a
blank cell is an empty value, not the reference"; unit test `iferror_of_a_blank_cell_is_an_empty_value`.
Eval delta: gen-m2 known failures 46 -> 45 (fixed 7469c3e920e1ade4 A351); probes +2 cases (iferror-blank-value 23 cells, 22 pass, 1 known
failure T-1294; omitted-arg-compare 10 cells, all pass). Regressions 0. Follow-ups: T-1294.
