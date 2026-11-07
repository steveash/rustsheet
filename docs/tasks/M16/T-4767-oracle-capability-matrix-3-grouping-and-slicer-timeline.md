---
id: T-4767
title: "Oracle capability matrix 3: grouping and slicer/timeline rows"
milestone: M16
lane: research
status: done
priority: P1
size: M
tier: opus
depends_on: [T-4130]
---

## Goal
Every catalog row of the grouping (125) and slicers-timelines (114) families, and the four record-tuple filtering
rows T-4130 left `unknown`, has `web` filled with yes or no, and each cites a probe and sheet in `evidence`, as T-4129
and T-4130 did for the other families.

## Context
- T-4130 was split to stay size M (its Outcome): it filled filtering, sorting and calculated, and answered the
  three questions. These two families were left `unknown`.
- Grouping needs builder support the case builder lacks: date `rangePr` (`groupBy` days to years, `startDate`,
  `endDate`), discrete (manual) groups (`discretePr`, a `databaseField="0"` group field with `base`/`par`), and
  `groupItems` of each item type (`b`, `d`, `e`, `m`, `n`, `s` and their attributes). T-4122 leaves date grouping to
  T-4170; add only what the rows need, in pivot_definition.py and its Rust mirror
  (crates/rustsheet-eval/src/pivot_definition.rs), with golden parts. The numeric `fieldGroup` is already evidenced
  (probes/t4128-pivot-spike G; web.md, M6 grouping).
- Slicers and timelines need new package parts (slicer cache and slicer parts, the drawing's slicer frame, the
  workbook's `x14:slicerCaches`/`x15:timelineCacheRefs` and defined names, the sheet's `x14:slicerList`): a
  spec2xlsx module like pivot_chart.py (T-4130). The question per row is whether the web opens the file, and whether
  a slicer's or timeline's selection filters the pivot.
- The tuple rows (`tpls`, `tpl@fld`, `tpl@hier`, `tpl@item` under groupItems) are OLAP shapes. T-4130 found tuples
  under a cache shared item refused, and four filtering rows (`pivotCacheRecords/r/{e,m,n,s}/tpls/tpl@hier`,
  PV-0699, PV-0720, PV-0740, PV-0761) opening in scratch files only, so they are `unknown` (pivots.md "Capability
  matrix 2"). This task teaches the builder to write tuples under a record value (Python and the Rust mirror) and
  commits a probe for those four rows too.
- Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

## Acceptance criteria
- [x] `web` filled for every grouping and slicers-timelines row and PV-0699, PV-0720, PV-0740, PV-0761, each citing
  a probe and sheet.
- [x] Findings in docs/excel-semantics/pivots.md; owners for every visibly-off cell (decision 0245).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Implementing grouping (T-4171) or slicers (T-4188).

## Outcome
Not split: every grouping (125) and slicers-timelines (114) row and the four record-tuple filtering rows have `web`
filled, each citing a probe and sheet (or, for a refusal, the scratch runs pivots.md lists by input sha256):
- **grouping**: 89 `yes`, 36 `no`. Refused: a group item's `<x>` child and its `<tpls>` (with or without `c`), for every
  item type. Every ST_GroupBy value, `discretePr`, `rangePr`, and group items of every type with every attribute open;
  a group item renders as its type (`b` as the text TRUE, `d` as a date serial, `e` as the error, `m` as (blank), `n` as
  a number).
- **slicers-timelines**: 56 `yes`, 58 `no`. Refused: every `timelinePivotFilter` shape tried (57 rows) and
  `slicerData="1"` (PV-1277; `0` opens). A slicer's and a timeline's selection both filter the pivot on open, and both
  win over the pivot's own hidden items or date filter. Table slicers open.
- **record tuples** (PV-0699, PV-0720, PV-0740, PV-0761): `yes`; the pivot renders the source.
Findings: pivots.md "Capability matrix 3" (and new lines in "Pivot flags Excel for the web refuses").

New probes, all excel-graph, 2026-11-07: probes/t4767-pivot-group-items, probes/t4767-pivot-slicers,
probes/t4767-pivot-timelines and probes/t4767-pivot-record-tuples.

Harness changes:
- tools/src/rustsheet_tools/pivot_slicers.py (new, the model is pivot_chart.py): a sheet's `slicers` (pivot or table
  slicers) and `timelines` write the slicer and timeline caches, the slicers and timelines parts, drawing frames, the
  workbook and sheet extLst entries, the defined names (unique per workbook, as Excel names them) and the pivot cache's
  x14 `pivotCacheId`. The engine adapter ignores both keys, as it does `pivot_charts`.
- the case builder (Python and the Rust mirror, new crates/rustsheet-eval/src/pivot_definition/raw_cache.rs): a grouped
  field's raw `xml.groupItems` slot and a definition's `records_xml` (one raw record value, for tuples); goldens
  t4767-pivot-group-items__GEA and t4767-pivot-record-tuples__RE. `raw_element` is now one module-level helper in
  pivot_definition.py, shared with pivot_slicers.py.
- _typos.toml ignores braced GUIDs (the extension uris).

Owners of the new off cells (decision 0245): T-4188 for probes/t4767-pivot-slicers and probes/t4767-pivot-timelines
(the engine renders those pivots unfiltered). The record-tuple case passes. Every group-item value passes, but the
shown text of the date group item fails at probes/t4767-pivot-group-items GD!A6 and GDA!A6 (Excel 1/1/2020, the
engine 43831); T-4170 and T-4171 are done, so the new T-4957 owns them. Follow-ups: the new T-4955 (whether an
Excel-saved `timelinePivotFilter` opens), T-4956 (the sidecar snapshot records pivotField items, pivot filters and
slicer and timeline caches, so the cached-download save claims can be committed) and T-4957. No M14 task (decision
0266).
