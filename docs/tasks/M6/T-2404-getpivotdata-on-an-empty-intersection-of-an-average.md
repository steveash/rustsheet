---
id: T-2404
title: "GETPIVOTDATA on an empty intersection of an Average or Count pivot (any non-Sum aggregate) is 0, not #REF!"
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
Also owns (T-2623): gen-m9/0620505280d2ea6e.case.json Calc!A4 `=SKEW(GETPIVOTDATA("Sales",Pivot!$H$3,"region","West","Rep","Bo"))`
(an Average pivot, Bo has no West record: Excel #DIV/0!, as SKEW of the single 0; ours #REF!) and gen-m9/a63d1c7d6f96c56d.case.json
Calc!A216 (`VLOOKUP(GETPIVOTDATA("UNITS",Pivot!$H$4,"Product",Pivot!$J$4,"REP","Bo"),..)` inside NORMDIST, a Count pivot, Bo has no
B record: Excel #N/A, ours #REF!), and (T-2524) gen-m9/d8266bc15533702a.case.json Calc!A343 `GAMMA.DIST(..,IF(GETPIVOTDATA("count of sales",
Pivot!$H$3:$L$8,"Product",Pivot!$K$4,"Rep",Pivot!$H$6),ACOS(1.5),IMSUB("5",Data!D7)))` (a Count pivot, Bo has no C record: Excel's
IF takes the FALSE branch, "3.995", ours #REF!).
Also owns (T-2674): gen-m6 case 8641fa604c5b8e14 Calc!A19 `=GETPIVOTDATA("Units",Pivot!$H$3,"Rep",Pivot!$K$4,"Product","B")`,
A25 `=GETPIVOTDATA("Count of Units",Pivot!$H$3,"rep","Bo","Product",Pivot!$H$6)` and A39
`=GETPIVOTDATA("Count of Units",Pivot!$H$3,"Rep","Cy","Product","B")` (a Count of Units pivot, empty intersections:
Excel 0, ours #REF!).
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
