---
id: T-2085
title: "Probe GETPIVOTDATA on unprobed pivot shapes: several data fields, empty average, number and date items, filters (SPEC-0015)"
milestone: M6
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
spec: docs/specs/SPEC-0015-pivot-tables-and-getpivotdata.md
---

## Goal
Excel evidence for the GETPIVOTDATA cases SPEC-0015 leaves unprobed.

## Context
SPEC-0015 "Excel behavior to match", unknowns: several data fields (blank data_field, the Values field), an empty intersection of Average/Count/Max/Min pivots, number/date/boolean items, page fields and hidden items, two row fields with subtotals. Extend tools/src/rustsheet_tools/pivot.py only as far as each shape needs (filters/hidden items may need the builder to grow; cut a harness task if so). Record findings in docs/excel-semantics/web.md "GETPIVOTDATA against a pivot table".

## Acceptance criteria
- [ ] Probe cases with oracle values for each unknown the builder can express; the rest listed as unprobed with why.
- [ ] Findings recorded in web.md; T-2083 updated if a rule changes.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Pivot refresh, creation, layout in the engine, export (SPEC-0015 non-goals).
