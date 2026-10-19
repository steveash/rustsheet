---
id: T-0405
title: "Bound range walks: cached sheet dimensions and a cell-visit budget for 3-D spans"
milestone: M3
lane: perf
status: in-progress
priority: P3
size: M
tier: opus
depends_on: [T-0049]
---

## Goal
A range walk costs no more than the data it reads, also for 3-D spans over many sheets.

## Context
T-0122 review (performance, security; minor): `for_each_cell_or_error_in_range`
(crates/rustsheet-engine/src/functions/util.rs) now walks a 3-D span sheet by sheet. Per sheet,
`clip_to_used_area` (evaluation.rs) calls `Worksheet::dimension()`, which scans every cell of the
sheet (T-0049 tracks a cached used area), so `SUM(S1:S50!A:A)` scans all 50 sheets on every
evaluation. And an explicit block that is neither whole-row nor whole-column
(`Data1:Data999!A2:Z1048575`) is walked cell by cell on every sheet: the pre-existing single-sheet
cost times the span's sheet count.

## Acceptance criteria
- [ ] A walk visits only stored cells (or the used area) of each sheet, for explicit blocks too.
- [ ] A bench case for a 3-D whole-column SUM over 50 sheets (or a deterministic visit-count test).
- [ ] `make check` green; reviewers approve.

## Out of scope
- 3-D reference semantics (T-0196, T-0197, T-0198).
