---
id: T-1502
title: "Union walks over sheets with many rows filled in other columns: find a union area's rows per column"
milestone: M1
lane: perf
status: todo
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
A union of whole columns walked again and again (a LAMBDA over MAP/BYROW) costs what the union's own columns hold, not how many rows the sheet uses in other columns, so `MAP(A1:A10000,LAMBDA(r,SUM((B:B,C:C))))` gives Excel's 80000 when another column fills 9,000 rows.

## Context
T-1497 walks union areas over existing rows (`SheetData::next_row_between`, `Model::for_each_existing_position` in functions/util.rs) and charges a step per row found, even when the area's columns are blank in it, so the walk budget `MAX_AREA_WALK` follows the sheet's used rows (T-1497 performance review; references.md "Union area cap"). A per-column row index in `SheetData` (kept in step by every write like `column_counts`, rebuilt on decode) would let a narrow area visit only its own rows. Weigh its memory and write cost on the storage benches first; a spec if it changes `SheetData` broadly.

## Acceptance criteria
- [ ] A unit test: the MAP above with 9,000 rows filled in column G is 80000, and a dense adversarial union still stops at `#VALUE!` in bounded time.
- [ ] Storage write/load benchmarks show no material regression.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
