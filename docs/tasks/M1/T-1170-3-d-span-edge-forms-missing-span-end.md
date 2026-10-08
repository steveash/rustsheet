---
id: T-1170
title: "3-D span edge forms: missing span end is NAME, explicit @ of a span is REF, HLOOKUP and COUNTBLANK over IF-wrapped spans"
milestone: M1
lane: engine
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
The 3-D span edge forms (six cells) that differ from Excel match it: a span ending on a missing sheet is `#NAME?`, `@` of a multi-sheet span is `#REF!`, and HLOOKUP and COUNTBLANK over an IF-wrapped span are `#VALUE!`.

## Context
Found by T-1139: probes/3d-ref-span-edges (excel-graph, 2026-10-08) cells Calc!A1, A2 (engine `#REF!`, Excel `#NAME?`), B1, B3 (engine `#VALUE!`, Excel `#REF!`), C4 (engine `#REF!`, Excel `#VALUE!`), C5 (engine -4, Excel `#VALUE!`). See references.md 3-D span entry. Code: `get_sheet_span_value` / `get_sheet_span_reference` (reference_operators.rs), `evaluate_single_sheet_arg` (functions/util.rs).

## Acceptance criteria
- [ ] Calc!A1, A2, B1, B3, C4, C5 of probes/3d-ref-span-edges pass and leave eval/baseline/probes.txt via `make eval-baseline`.
- [ ] The references.md "Engine mismatches" sentence is removed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- LET-bound spans (T-1158).
