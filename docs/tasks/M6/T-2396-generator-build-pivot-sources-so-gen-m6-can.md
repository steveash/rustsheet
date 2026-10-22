---
id: T-2396
title: "Generator: build pivot sources so gen-m6 can emit GETPIVOTDATA calls (400-cell bar, decision 0094)"
milestone: M6
lane: harness
status: in-progress
priority: P2
size: M
tier: opus
depends_on: []
campaign: visible
---

## Goal
`rustsheet-gen --milestone M6 --focus` can make GETPIVOTDATA the outer call, so it reaches decision 0094's 400 Excel cells like the other M6 functions.

## Context
- GETPIVOTDATA has 54 Excel cells (probes only) and no generated cell. Every other generatable M6 function has 575 or more
  (T-2180, T-2253). It needs a pivot table to query, and generated cases have none, so the generator skips it.
- Spec cases can already hold pivot tables: a sheet's `"pivot_tables"` in tools/src/rustsheet_tools/spec2xlsx.py, rendered by
  rustsheet_tools/pivot.py (T-1705), with the engine adapter registering them (T-2084). See SPEC-0015 for what the engine reads.
- The work: when GETPIVOTDATA is in focus, the generator adds a pivot table over its Data sheet (row/column/value fields drawn from the
  data columns, function sum/count/average), and draws `data_field`, `pivot_table` and field/item pairs from that pivot's fields and
  items, with some misses (an unknown field or item, a wrong reference) so the `#REF!` paths are covered. Probed shapes:
  docs/excel-semantics/ (GETPIVOTDATA entries) and T-2085.

## Acceptance criteria
- [ ] Tools tests: a focus case for GETPIVOTDATA builds through spec2xlsx with its pivot, and its calls name only fields of that pivot
      (plus the deliberate misses).
- [ ] A scratch or committed sample of such cases fills through excel-graph with a value for every cell (the excel-graph oracle
      recalculates pivots; if it does not, record that here and block on it).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The sweep itself (a gen-m6 sweep task runs it once this lands).
