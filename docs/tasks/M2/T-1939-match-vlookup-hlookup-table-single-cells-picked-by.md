---
id: T-1939
title: "MATCH/VLOOKUP/HLOOKUP table: single cells picked by CHOOSE, OFFSET or a name, and which single text is numeric"
milestone: M2
lane: engine
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
MATCH's `lookup_array` and the table of VLOOKUP and HLOOKUP read every single-cell reference form, and every single text value, as Excel does, backed by a probe.

## Context
Review follow-up of T-1862. `Model::evaluate_single_sheet_arg` (functions/util.rs) keeps a written one-cell reference, and one that IF or INDEX picks, as a one-cell range, so `MATCH("b",A2,0)` is 1 (probes/match-single-value-lookup-array). Other single-cell producers still come back as a value: `MATCH("b",CHOOSE(1,A2),0)`, `OFFSET(A2,0,0)`, `INDIRECT("A2")` and a defined name for one cell are unprobed. Consider reusing `evaluate_picked_reference_arg` (statistical/count.rs callers) rather than adding arms. Also unprobed: which single text is "numeric" (`#N/A`) rather than `#VALUE!`. `Model::lookup_table` (lookup.rs) uses `cast_number`, so dates ("1/2/2020"), currency ("$1"), percents, " 2 ", "TRUE" and "" are inferred (docs/excel-semantics/lookup.md "MATCH, VLOOKUP and HLOOKUP over a single value").

## Acceptance criteria
- [ ] A probe (excel-graph) covers the forms and texts above under MATCH and VLOOKUP, and the engine matches it (0 eval regressions).
- [ ] docs/excel-semantics/lookup.md is updated with the findings.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
