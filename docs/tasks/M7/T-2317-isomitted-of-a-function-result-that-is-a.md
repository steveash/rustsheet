---
id: T-2317
title: "ISOMITTED of a function result that is a blank cell is FALSE (gen-m7)"
milestone: M7
lane: functions-logic
status: todo
priority: P2
size: S
tier: sonnet
campaign: visible
depends_on: []
---

## Goal
ISOMITTED of a function result that is a reference to a blank cell (LOOKUP returning an empty cell) is FALSE, as Excel gives; only an omitted LAMBDA parameter is TRUE.

## Context
Found by the T-2225 triage of eval/suites/gen-m7 (38 cases, 10,991 Excel-scored cells, excel-graph). Quirk entry: docs/excel-semantics/arrays.md, the ISOMITTED paragraph before "MAP and SCAN with a LAMBDA that returns a multi-cell range".

Evidence: `ISOMITTED(LOOKUP(ROWS(Data!D$3),Data!C9:C12,Data!B10:B12))` is FALSE (2c5ea078f25443a2 A1087); ours TRUE because the LOOKUP result is an empty cell. arrays.md: `ISOMITTED(A1)` outside a LAMBDA is FALSE, and ours already gives FALSE for `ISOMITTED(Data!Z99)`: the empty value coming out of a function is mistaken for an omitted argument. Code: functions/information/isomitted.rs.

Visibly-off cells (gap triage 2026-10-21, decision 0083): 1 cells, every one with a wrong type, error, text or value (none is a last-digit difference).

Owned gen-m7 cells (T-2225 triage, excel-graph, 2026-10-21): 1 known-failure lines in eval/baseline/gen-m7.txt, 1 formulas. One line per formula: case, anchor, the anchor formula, Excel vs ours at the first failing cell, then every failing cell (sheet Calc; a range is consecutive cells of one row).

- gen-m7/2c5ea078f25443a2.case.json A1087 `=ISOMITTED(LOOKUP(ROWS(Data!D$3),Data!C9:C12,Data!B10:B12))`: A1087 Excel FALSE, ours TRUE; cells (1): A1087

## Acceptance criteria
- [ ] Visible parity: the 1 gen-m7 cells listed under Context have Excel's value, type, error and text: these 1 known-failure lines disappear from eval/baseline/gen-m7.txt (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] The docs/excel-semantics entry cited above states the rule and its "Ours:" line names this task's change.
- [ ] Unit tests cover the rule on a small hand-written input (one error or edge case and one normal case that must keep passing).
- [ ] `make check` green; reviewers approve.

## Out of scope
- -

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off (a wrong type, error, text or value), so this task is in the campaign. Its target is that every cell it owns has Excel's type, error and text, or is within half a unit of the 15th significant digit of Excel's value (an expected 0 exactly 0); the visible-parity criterion under Acceptance criteria states it.
