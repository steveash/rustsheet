---
id: T-3542
title: "XMATCH: an error-valued search_mode cell wins over the match_mode and range errors (gen-m6 arg-pairs cells)"
milestone: M6
lane: functions-lookup
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
XMATCH returns Excel's #DIV/0! when search_mode is a #DIV/0! cell, ahead of the engine's #VALUE!.

## Context
Found by the T-3496 arg-pairs batch (excel-graph): Excel returns #DIV/0!, the engine #VALUE!, with a bad lookup range or match_mode as well.

## Acceptance criteria
- [ ] The cells below match Excel (probe the rule with the excel-probe skill first; record it in docs/excel-semantics/lookup.md).
- [ ] `make eval-baseline` shrinks gen-m6; `make check` green; reviewers approve.

## Out of scope
- Other functions and the unrelated cells of these cases.

## Visible-parity stage
Decision 0083: every cell below returns a wrong error (visibly off). The acceptance criteria are the stage.
Cells:
- 315644f84093cf49 A292
- 75b38d80dd3152b2 A388 A448 A544
