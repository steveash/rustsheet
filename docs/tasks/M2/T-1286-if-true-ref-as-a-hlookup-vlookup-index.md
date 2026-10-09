---
id: T-1286
title: "IF(TRUE,ref) as a HLOOKUP/VLOOKUP index keeps the pick a reference; multi-cell range index is #VALUE!"
milestone: M2
lane: engine
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
HLOOKUP/VLOOKUP match Excel for the two index shapes probes/lookup-pick-index-shapes shows the engine gets wrong.

## Context
Oracle (excel-graph 2026-10-09, T-0299): an `IF(TRUE,G5)` (single-cell; the range form passes) index keeps the pick a reference to aggregates (`COUNT(HLOOKUP(42,F5:G6,IF(TRUE,G5)))` is 0, A2; `SUM(VLOOKUP(2,I5:J7,IF(TRUE,G5)))` is 0, D2) but the engine gives 1 and 5. A multi-cell range index `HLOOKUP(42,F5:G6,G5:G6)` outside the range's rows is `#VALUE!` (C7), the engine gives `#REF!`. See `lookup_index_arg` (T-0295) in the engine's lookup functions.

## Acceptance criteria
- [ ] probes/lookup-pick-index-shapes cells A2, D2 and C7 pass and are removed from the baseline.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
