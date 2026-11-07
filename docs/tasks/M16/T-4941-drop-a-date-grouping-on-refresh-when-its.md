---
id: T-4941
title: "Drop a date grouping on refresh when its field's sharedItems lack containsNonDate=\"0\", as Excel does"
milestone: M16
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
A pivot cache field grouped in place by dates whose `<sharedItems>` lack `containsNonDate="0"` (and `minDate`/`maxDate`) loses its date grouping on a refresh, as Excel's refresh on open drops it: the field lists each date, and GETPIVOTDATA on a month or level is `#REF!`.

## Context
T-4177's pivot-export suite (eval/suites/pivot-export) committed T-4734's first run: pivot-export/t4177-date-flags-needed.export.json is probes/t4170-pivot-date-levels exported with those three attributes removed from Date's sharedItems in each of its seven refreshOnLoad caches. Excel for the web (excel-graph, 2026-11-07) refreshed each pivot without the months: DC, DO, DT, DS, DY and DM list the nine dates where the month labels belong, DX keeps only its data header and grand total (511), and the GETPIVOTDATA cells on a month or level are `#REF!` (B16, on the date 2019-01-10, is 2). The engine keeps the grouping (pivots.md "Excel for the web opens engine-exported grouped pivot caches"). Which of the attributes Excel needs, and whether the fieldGroup survives in its saved cache, is open: probe it (a one-attribute-at-a-time variant of the case) before implementing. Code: the importer's shared-item flags (crates/rustsheet-xlsx/src/import/pivots/) and the refresh's regroup (crates/rustsheet-engine/src/pivot/).

Owner of these visibly-off cells (decision 0245; T-4177, Excel's refresh dropped the months, the engine kept them):
- pivot-export/t4177-date-flags-needed.export.json DC!A4:B13, DO!A3:B13, DT!A3:B13, DS!A3:B13, DY!A4:B13, DM!A4:B13, DX!A3:A4, Calc!B16, Calc!B18, Calc!B20

## Acceptance criteria
- [ ] A probe says which of `containsNonDate`, `minDate` and `maxDate` Excel's refresh needs to keep a date grouping, and pivots.md records it.
- [ ] pivot-export/t4177-date-flags-needed.export.json passes, and t4177-date-levels.export.json still renders its months.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
