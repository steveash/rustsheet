---
id: T-2374
title: "BASE of zero with min_length 0 is empty text (BASE(0.2,29,0.3) is '')"
milestone: M5
lane: functions-math
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
campaign: visible
---

## Goal
BASE of a number that truncates to 0 with an explicit `min_length` that truncates to 0 is empty text, as in Excel; without `min_length` it stays "0".

## Context
Gap triage of gen-m5 round 2 (T-2252, excel-graph oracle, 2026-10-22). `base_text` (mathematical.rs) always emits at least one digit, so `BASE(0.2,29,0.3)` is "0" where Excel gives "" (`LENB`
of it is 0). Excel pads the digits to `min_length` and a zero has none, so with no `min_length` it is "0" (a default of 1:
gen-m5/cd883013bea3b305 Calc!A22, `MID(BASE(0,22),..)` = "0") and with 9 or 10 it is that many zeros (gen-m5/9a7a94353845ea12
Calc!A45, gen-m5/90bf45bea539723d Calc!A251, passing). Rule: docs/excel-semantics/numbers.md "BASE of zero with min_length 0 is
empty; BASE past 2^53 is #NUM!" (added by this triage). Only the zero number is affected: a non-zero number always has digits.

Failing cells (`make eval-show CASE=gen-m5/<case>.case.json`), 1 in eval/baseline/gen-m5.txt:

| case / cell | formula | Excel | ours |
|---|---|---|---|
| gen-m5/dde5ea295a38fb16 Calc!A254 | `=LENB(BASE(Data!B8,29,Data!A$6))` | 0 | 1 |

## Acceptance criteria
- [x] The three passing zero cells named above still pass.
- [x] This 1 known-failure line disappears from eval/baseline/gen-m5.txt (`make eval-baseline`), with no new known failure in any suite.
- [x] `make check` green; reviewers approve.

## Out of scope
- BASE's upper bound (T-2373).

## Visible-parity stage
Decision 0083 (gap triage 2026-10-22, T-2252): every cell this task owns has the wrong type, error, text or boolean, or is a
number further than half a unit in Excel's 15th significant digit, so the task is in the campaign. Its acceptance
criteria above are already the visible-parity stage.

## Outcome
`base_text` emits no digits for a zero number, so an explicit `min_length` that truncates to 0 gives empty text; an omitted `min_length` now defaults to 1 ("0"). Eval: 1 gen-m5 known failure fixed (dde5ea295a38fb16 Calc!A254), no regressions; the three passing zero cells still pass.
