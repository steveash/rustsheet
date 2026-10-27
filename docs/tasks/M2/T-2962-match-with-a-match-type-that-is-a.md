---
id: T-2962
title: "MATCH with a match_type that is a reference to a #N/A cell gives #REF! in Excel (probes/m2-argument-kind-gaps Calc!A25)"
milestone: M2
lane: functions-lookup
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`MATCH(30,A1:A5,<ref to a cell holding NA()>)` gives the same result as Excel, `#REF!`.

## Context
Found by T-2861: probes/m2-argument-kind-gaps Calc!A25 `=IF(ISERROR(MATCH(30,A1:A5,M2)),ERROR.TYPE(MATCH(30,A1:A5,M2)),MATCH(30,A1:A5,M2))`
with M2 `=NA()`: Excel (excel-graph, 2026-10-27) ERROR.TYPE 4 (#REF!), ours 7 (#N/A). The literal `NA()` as match_type (A23; the `#DIV/0!` literal, A24, gives 2) and the same
reference as VLOOKUP/HLOOKUP range_lookup pass with Excel's propagated #N/A, so only the MATCH match_type reference differs.
Probe it further (a reference to other error cells, e.g. #DIV/0!, as match_type) with the excel-probe skill before fixing; record the rule in
docs/excel-semantics/lookup.md. The cell is a known failure owned by this task.

## Visible-parity stage
Stage 1: the cell is an error-type mismatch (visibly off); the target is the exact error type Excel gives.

## Acceptance criteria
- [ ] probes/m2-argument-kind-gaps Calc!A25 passes and its baseline entry is removed (`make eval-baseline`).
- [ ] docs/excel-semantics/lookup.md records the rule with the probe cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other error kinds as the match_type reference until probed.
