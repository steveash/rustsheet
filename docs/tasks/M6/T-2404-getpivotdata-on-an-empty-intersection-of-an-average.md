---
id: T-2404
title: "GETPIVOTDATA on an empty intersection of an Average or Count pivot is 0, not #REF!"
milestone: M6
lane: functions-lookup
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
GETPIVOTDATA on an intersection with no source records answers what Excel does for every aggregate the case builder
writes (sum, count, average, max, min), not `#REF!`.

## Context
Found by T-2396's generator sample (excel-graph, 2026-10-22, scratch, not committed): over an Average of Units pivot
(rows Product, columns Region; Product A has no East record, B no South, C no West), `=GETPIVOTDATA("units",Pivot!$H$3,"Product","a","Region","East")`,
`("Units",Pivot!$H$3,"PRODUCT",Pivot!$H$7,"Region","South")` and `("Average of Units",Pivot!$L$5,"Product","C","Region","West")`
are 0 in Excel; ours `#REF!`. A Sum's empty intersection is 0 (probes/getpivotdata-real-pivot H13). Ours: `aggregate`
in crates/rustsheet-engine/src/functions/lookup_and_reference/pivot_index.rs refuses any aggregate but Sum over no
values (SPEC-0015 unknown 2). docs/excel-semantics/web.md "GETPIVOTDATA against a pivot table" (the empty-intersection line).
Needs a committed probe first (`excel-probe` skill, excel-graph): a `pivot_tables` probe case with average, count,
max and min pivots (spec2xlsx `function`), each queried on an empty intersection, a non-empty one and the grand total.
Generate a pivot with `cd tools && uv run rustsheet-gen --milestone M6 --focus --functions GETPIVOTDATA --count 1` to
copy the shape, or write it by hand like probes/getpivotdata-real-pivot.

## Acceptance criteria
- [ ] The probe case passes in full; web.md states the rule per aggregate and SPEC-0015 unknown 2 is updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
- StdDev/Var and the other aggregates the case builder cannot write.
