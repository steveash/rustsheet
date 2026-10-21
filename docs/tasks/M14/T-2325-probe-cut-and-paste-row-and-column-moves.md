---
id: T-2325
title: "Probe cut and paste, row and column moves, sheet-delete undo and sheet duplication on tables with desktop Excel"
milestone: M14
lane: research
status: blocked
priority: P3
size: S
tier: opus
depends_on: [T-1074]
---

## Goal
docs/excel-semantics/references.md "Structured references when the table changes" records what desktop
Excel does to a table and the structured references to it on a cut and paste of table cells, a row or
column move through the table, undo of a sheet delete, and a duplicated sheet.

## Context
T-2197 found no Graph REST endpoint for these edits (no range move or cut/paste, no undo, no worksheet
copy), so excel-graph cannot probe them; the rest of T-2197's questions are probed (probes/table-edits) or
go to T-2324. SPEC-0016 treats all four as fallback (plain ranges); the engine gives a duplicated sheet no
table, where Excel is believed (hand) to add a renamed table to the copy (from T-2195).

## Acceptance criteria
- [ ] Desktop evidence for each of the four edits in references.md, or a recorded reason one cannot be had.
- [ ] Desktop cross-check of the T-2197 plain-range quirk (`=SUM(S!C4:C7)` becomes `S!C4:C8` on a row
  inserted above the first data row or above the totals row; seen through Graph's `range/insert` only).
- [ ] An engine task filed for each finding the engine misses.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Implementation.

## Blocked because
Phase 4 (decision 0039): only desktop Excel over COM (`Range.Cut`, `Range.Insert` with a cut, `Worksheet.Copy`,
`Application.Undo`) can perform these edits. Unblock when `cd tools && uv run rustsheet-oracle status` shows
`excel-desktop OK` and T-1074 is done.
