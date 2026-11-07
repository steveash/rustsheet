---
id: T-4955
title: "Probe whether an Excel-saved timelinePivotFilter opens in Excel for the web"
milestone: M16
lane: research
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-4767]
---

## Goal
Know whether Excel for the web opens a timeline cache that carries an Excel-saved `<x15:timelinePivotFilter>`, so
the 57 slicers-timelines catalog rows under `x15:timelineCacheDefinition/timelinePivotFilter` (PV-1541 to PV-1604),
now `no` from builder-made shapes only, rest on an Excel-authored file too.

## Context
- T-4767 found every builder-made `timelinePivotFilter` refused (`FileCorruptTryRepair`): customFilters,
  dynamicFilter, filters with dateGroupItem, top10, iconFilter, colorFilter, a sortState, a minimal
  `<timelinePivotFilter fld="2"><x:autoFilter ref="A1"/></timelinePivotFilter>`, with and without a matching pivot
  filter (docs/excel-semantics/pivots.md "Capability matrix 3", input sha256s listed there). The same file without
  the element opens (probes/t4767-pivot-timelines TL).
- Excel for the web never saved the element in those runs (seen in cached downloads: it writes the timeline's filter
  as the pivot's `dateBetween` filter instead; the sidecars do not record it, T-4956), and no Excel-saved file with
  one is in the surveyed corpora.
- So the question needs an Excel-authored workbook with the element: a public corpus file (search GitHub and the
  corpora for `timelinePivotFilter` in an xlsx), uploaded through `rustsheet-oracle recalc -b excel-graph`.
- Desktop Excel is out of scope (decision 0266): if no Excel-saved file can be found, record that and close.

## Acceptance criteria
- [ ] An Excel-saved file with a `timelinePivotFilter` is found and opened on the web, or the search is recorded as
  fruitless; pivots.md "Capability matrix 3" and the catalog rows updated with the answer.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Implementing timelines (T-4188).
