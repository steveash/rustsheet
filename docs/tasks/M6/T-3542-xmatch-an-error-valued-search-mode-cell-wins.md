---
id: T-3542
title: "XMATCH: an error-valued search_mode cell wins over an out-of-range numeric match_mode, and a multi-cell match_mode is lifted into a spill (non-numeric text match_mode still wins) (gen-m6 arg-pairs cells)"
milestone: M6
lane: functions-lookup
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
XMATCH returns Excel's #DIV/0! when search_mode is a #DIV/0! cell and match_mode is out of range, or a multi-cell range lifted element by element (A292 spills 2x2 over C3:D4; check B292, A293, B293 too); A64/A598 (text match_mode, #VALUE!) must keep passing (see lookup.md).

## Context
Found by the T-3496 arg-pairs batch (excel-graph): Excel returns #DIV/0!, the engine #VALUE!, with an out-of-range or multi-cell match_mode.

## Acceptance criteria
- [ ] The cells below match Excel (probe the rule with the excel-probe skill first; record it in docs/excel-semantics/lookup.md).
- [ ] `make eval-baseline` shrinks gen-m6; `make check` green; reviewers approve.

## Out of scope
- Other functions and the unrelated cells of these cases.

## Visible-parity stage
Decision 0083: every cell below returns a wrong error; only some count in the in-scope visibly-off lock (it rose by 11 for the batch's 22 cells). The acceptance criteria are the stage.
Cells:
- 315644f84093cf49 A292
- 75b38d80dd3152b2 A388 A448 A544
