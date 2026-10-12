---
id: T-1399
title: "MATCH match_type text from a cell passed through IFS is #REF! (gen-m7 6db767d720c5b55a A5)"
milestone: M2
lane: functions-lookup
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
MATCH(..., IFS(TRUE, <cell holding text>)) is #REF!, like a direct text cell reference.

## Context
T-1313 made a text match_type read directly from a cell #REF! (`match_type_arg` in lookup_and_reference/mod.rs). gen-m7/6db767d720c5b55a Calc!A5 `=(MATCH(SUMPRODUCT(Data!B8,Data!$C3),Data!B3:B4,IFS(TRUE,Data!D$3))=3.14159)` with D3 = "x": Excel `#REF!`, engine `#VALUE!`, because IFS returns the value, not the reference it picks (IF already does in reference context, see `evaluate_reference_arg`). Probe IFS/IF/CHOOSE/SWITCH pass-through for match_type first.

## Acceptance criteria
- [x] gen-m7/6db767d720c5b55a A5 (not committed; reproduced by probes/match-type-text-passthrough K1) passes; baseline updated.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
MATCH's match_type now keeps the reference picked by IF, IFS, SWITCH and LET (CHOOSE already did); probe `match-type-text-passthrough` (9 cells, oracle-backed) passes. IFERROR gives a value (#VALUE!). Recorded in docs/excel-semantics/lookup.md Rule 3.
