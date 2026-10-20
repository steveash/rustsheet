---
id: T-2083
title: "GETPIVOTDATA resolves data_field and field/item pairs against a loaded pivot table (SPEC-0015)"
milestone: M6
lane: functions-lookup
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-2081]
spec: docs/specs/SPEC-0015-pivot-tables-and-getpivotdata.md
---

## Goal
GETPIVOTDATA answers from a loaded pivot table as Excel does on every cell of probes/getpivotdata-real-pivot.

## Context
SPEC-0015 "GETPIVOTDATA": pick the pivot by the reference, the data field by caption or source name (case-insensitive, untrimmed, blank = the only data field), the items on the row/column axes, then aggregate the cache records (Sum/Count over none is 0). Arrays lift (SPEC-0011). Unknowns stay `#REF!` until T-2085 has evidence. Replaces the T-1703 stub in functions/lookup_and_reference/getpivotdata.rs. Depends on T-2081 (tests can build the model directly); scores in eval once T-2084 lands.

## Acceptance criteria
- [ ] Unit tests against a model built in code cover every rule row of SPEC-0015's table.
- [ ] probes/getpivotdata-evidence-floor still passes.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Pivot refresh, creation, layout in the engine, export (SPEC-0015 non-goals).
