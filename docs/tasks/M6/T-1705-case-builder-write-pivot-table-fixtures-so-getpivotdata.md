---
id: T-1705
title: "Case builder: write pivot table fixtures so GETPIVOTDATA can be probed against a real pivot"
milestone: M6
lane: harness
status: done
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
The case builder can write a pivot table into a probe workbook so GETPIVOTDATA can be probed against a real pivot.

## Context
T-1628 probed GETPIVOTDATA only without a pivot (builder cannot write pivots); field/item pairs, grand totals and missing items stay unprobed. Needs a design note (tools/ spec->xlsx builder, excel-graph acceptance of pivots).

## Acceptance criteria
- [x] A probe with a pivot fixture is oracle-filled and covers field/item pairs, grand totals and a missing item; findings in docs/excel-semantics/.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
- Design: pivots are a spec-case feature, like `tables` and `data_tables`, not a hand-built fixture.
  A sheet's `pivot_tables` (one row field, an optional column field, one data field: sum, count,
  average, max or min) is written by `tools/src/rustsheet_tools/pivot.py`, called from `build_xlsx`.
  It renders the compact-form cells and adds the pivot cache definition, the records and the table
  definition, with their relationships and content types. The cache is marked `refreshOnLoad`.
  `patch_parts` can now add parts byte-stably. The Rust `SheetSpec` declares `pivot_tables`
  (`PivotTableSpec`), and the engine adapter ignores it.
- Excel for the web loads these workbooks. A one-off check of every rendered cell (labels, values,
  a blank intersection, grand totals) matched the builder's layout exactly.
- Probe `probes/getpivotdata-real-pivot` (excel-graph, 33 cells, tag M6) covers:
  - field/item pairs in both orders, grand and row/column totals, and case-insensitive names;
  - a missing item, a missing field and a missing data field, and `#REF!` arguments;
  - an empty intersection (0), a reference inside, outside and spanning the pivot, and array items.
- Findings: docs/excel-semantics/web.md, "GETPIVOTDATA against a pivot table".
- Eval delta: probes known failures +33. The engine has no GETPIVOTDATA (`#NAME?`); 0 regressions.
- Follow-up: T-1708 (spec for pivot tables in the engine).
