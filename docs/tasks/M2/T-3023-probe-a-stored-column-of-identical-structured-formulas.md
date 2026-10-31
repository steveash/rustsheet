---
id: T-3023
title: "Probe a stored column of identical structured formulas without calculatedColumnFormula (T-2983 follow-up)"
milestone: M2
lane: harness
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
references.md (rule 8) states what Excel does when `=[@Col]*3` is typed into one row of a table column whose other cells hold one identical stored structured formula (`=[@Col]*2`) and no `calculatedColumnFormula`.

## Context
T-2983 probed A1 formulas only: Excel does not re-fill them. Since T-3024 the engine (`Model::calculated_column_fill`, calculated_columns.rs) re-fills a column of one formula only when it is a calculated column (`TableColumn::calculated_column_formula`, set by a typed fill), so identical stored structured formulas are not re-filled either; unprobed. Model on probes/t2983-identical-stored-refill with `[@Col]*2` stored in C2:C4.

## Acceptance criteria
- [x] Probe case filled on excel-graph and in the probes baseline; references.md states the rule; an engine task is filed if ours differs.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probe `probes/t3023-identical-stored-structured-refill` filled on excel-graph (2026-10-30): with stored
`T[[#This Row],[Col]]*2` in C2:C4 and no `calculatedColumnFormula`, typing `=[@Col]*3` into C5 changes only C5
(C2:C4 stay 2, 4, 8). A bare `[@Col]` in a stored cell loads as `#REF!`, so the case stores the file form.
references.md rule 8(d) records it. The engine already matches (case passes 8/8); no engine task needed.
