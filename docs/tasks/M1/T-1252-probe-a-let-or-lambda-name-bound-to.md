---
id: T-1252
title: "Probe a LET or LAMBDA name bound to XLOOKUP, TAKE, DROP or TRIMRANGE when not called (ISREF, CELL, ROWS, SUM)"
milestone: M1
lane: functions-logic
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Know whether a LET name or LAMBDA parameter bound to XLOOKUP, TAKE, DROP or TRIMRANGE over a range is a reference when it is not called, as Excel has it.

## Context
T-1232 binds these four as the reference they pick (`bind_argument`, functions/logical/let.rs), proven only for the called form (probes/let-name-call-forms H7:H9, H11). Uncalled uses (ISREF, CELL("address"), ROWS, SUM, COUNTIF through the name, in LET and LAMBDA) are unprobed. Probe them (excel-probe skill), record in docs/excel-semantics/references.md, fix any mismatch. Also: the kind list Xlookup|Take|Drop|Trimrange is repeated in `evaluate_picked_reference_arg`, `is_read_reference` and `bind_argument`; share one predicate (directive 3). Reviewer also saw `SUM(XLOOKUP({1;2},A1:A3,B1:B3))` give #N/A with no evidence of Excel's value; probe it too.

## Acceptance criteria
- [ ] Probe case with oracle values committed; engine matches or known failures are filed; the kind list lives in one place.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probe `probes/let-name-uncalled-picked-refs` (41 cells, excel-graph): an uncalled LET name or LAMBDA parameter bound to XLOOKUP, TAKE, DROP or TRIMRANGE is the reference it picks, and the engine matches on all 40 such cells. The call list is now one predicate, `Function::picks_reference`, used by `evaluate_picked_reference_arg`, `bind_argument` and `is_read_reference`. `SUM(XLOOKUP({1;2},A1:A3,B1:B3))` is 30 in Excel and `#N/A` here (one new known failure in the probes baseline); filed T-1264. Recorded in docs/excel-semantics/references.md. Eval delta: probes +1 case, +1 known failure (new coverage), no regressions.
