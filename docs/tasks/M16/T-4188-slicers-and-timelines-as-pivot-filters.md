---
id: T-4188
title: "Slicers and timelines as pivot filters"
milestone: M16
lane: engine
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-4165, T-4176]
---

## Goal
Slicer and timeline caches are imported as filter state on the pivots they drive, applied by PivotResult, and preserved on export.

## Context
excel-docs.md slicer/timeline rows; x14/x15 parts. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.
- Owner of the visibly-off cells of probes/t4767-pivot-slicers (decision 0245): the engine adapter ignores a spec's
  `slicers`, so it renders the pivots unfiltered where Excel for the web applies the slicer's selection (SL, SM, SH,
  SD) and lets it win over the pivot's hidden items (SX). Also owner of probes/t4767-pivot-timelines, whose timeline
  selections Excel applies the same way (TL, TA, TD0, TD1, TX). Findings: pivots.md "Capability matrix 3" (T-4767).

## Acceptance criteria
- [ ] Probes pass.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Drawing slicers.
