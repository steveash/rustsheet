---
id: T-4956
title: "Record pivotField items, pivot filters and slicer and timeline caches in the oracle sidecar snapshot"
milestone: M16
lane: harness
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-4767]
---

## Goal
An Excel-saved file's oracle sidecar records what Excel saved for a pivot's fields and filters and for its slicers
and timelines, so claims about what Excel for the web saves can cite a committed sidecar, not the oracle's cached
downloads.

## Context
- T-4767 (pivots.md "Capability matrix 3") states several save claims as UNVERIFIED because the sidecar snapshot
  (crates/rustsheet-eval/src/xlsx_snapshot/pivots.rs) records no pivotField items, no pivot filters, and none of the
  slicer or timeline parts. Seen only in cached downloads: unselected slicer items saved as the pivotField's `h="1"`;
  a timeline selection saved as a `dateBetween` pivot filter with x15 `useWholeDay="1"`; dropped group-item
  attributes, slicer item `nd`, `slicerCacheHideItemsWithNoData@count`, `singleRangeFilterState="0"` and
  `x15:timelinePivotCacheDefinition`; sorted slicer items; added `xr10:uid` and `scrollPosition`.
- The snapshot's `group_items` maps `s` and `d` both to text (pivots.rs), so a saved date group item cannot be told
  from a string one; record the item's type.
- After the snapshot grows, refill probes/t4767-pivot-slicers, -timelines and -group-items (`rustsheet-oracle fill`
  re-reads cached downloads) and move the claims in pivots.md to VERIFIED or correct them.

## Acceptance criteria
- [ ] The sidecar `pivots` section records each pivotField's items (with `h`, `m`, `s`, `x`), each pivot filter
  (type, field, values, extensions), each group item's type, and the slicer and timeline caches and parts.
- [ ] The four t4767 probes are refilled and the pivots.md "Capability matrix 3" save claims cite them.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Writing slicers or timelines on export (T-4188).
