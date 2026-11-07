---
id: T-4154
title: "GETPIVOTDATA over PivotResult for every PV1 shape"
milestone: M16
lane: functions-lookup
status: todo
priority: P1
size: M
tier: opus
depends_on: [T-4142, T-2085]
---

## Goal
GETPIVOTDATA reads the `PivotResult`: several data fields, page fields, hidden items, two or more fields per axis and their subtotals, number/date/boolean items, as T-2085's probes evidence.

## Context
SPEC-0015's unknowns answered `#REF!` until evidenced; T-2085 probes them. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

Owner of these visibly-off cells (decision 0245; T-4130, GETPIVOTDATA against a pivot collapsed to #SPILL!: Excel gives #REF! except for a field and item pair that names an item, which gives #SPILL!. The engine gives #SPILL! for the grand total by either name (B1, B2) and for an unknown item (B4), and 27 for a range holding the anchor (B6); B3 and B5 already agree; pivots.md "GETPIVOTDATA against a pivot collapsed to #SPILL!"):
- probes/t4130-pivot-spill-getpivotdata Q!B1:B2, Q!B4, Q!B6

Owner of these visibly-off cells (decision 0245; T-4170, pivots.md "Manual groups and date levels as Excel for the web
renders them"). Cause: the engine answers `#REF!` for an item of a field on an axis that holds two or more fields
(SPEC-0015 unknown 10), here DC's rows Years, Quarters and Date, where Excel answers the total (the grand total B17 and
DX's pair B20, one field per axis, already agree):
- probes/t4170-pivot-date-levels Calc!B1:B3, Calc!B5:B6, Calc!B8:B9, Calc!B11:B14

Owner of these visibly-off cells (decision 0245; moved from T-4171, whose fix made the manual-group pivots load and
render). Cause: the same SPEC-0015 unknown 10, on MC, MT and MS, whose rows are Product2 and Product: the engine
answers `#REF!` for a group, an ungrouped item or a pair where Excel answers the total (MG and MB, one row field, and
the grand total already agree; web.md "GETPIVOTDATA on manual groups and date levels"):
- probes/t4170-pivot-manual-groups Calc!B1:B5, Calc!B8:B10, Calc!B12, Calc!B14

Owner of these visibly-off cells (decision 0245; T-4177, the same causes on the engine's exports of the probes above
and on API-built pivots, pivots.md "Excel for the web opens engine-exported pivot parts"): an item of a field on an
axis that holds two or more fields (SPEC-0015 unknown 10); a boolean or number item (`TRUE`, 7, 2.5: Excel 8, 32 and
256, the engine `#REF!`); and the grand total of a pivot collapsed to `#SPILL!` (Excel `#REF!`, the engine
`#SPILL!`):
- pivot-export/t4177-date-levels.export.json Calc!B1:B3, Calc!B5:B6, Calc!B8:B9, Calc!B11:B14
- pivot-export/t4177-manual-groups.export.json Calc!B1:B5, Calc!B8:B10, Calc!B12, Calc!B14
- pivot-export/t4177-grouped-cache.export.json Calc!B1:B3, Calc!B7:B9
- pivot-export/t4177-api-items.export.json Calc!B3, Calc!B5:B6
- pivot-export/t4177-api-items-refresh.export.json Calc!B3, Calc!B5:B6
- pivot-export/t4177-api-spill-refresh.export.json Calc!B2

## Acceptance criteria
- [ ] probes/getpivotdata-* pass; the SPEC-0015 unknowns T-2085 resolved are implemented.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Show-as values in GETPIVOTDATA (PV2).
