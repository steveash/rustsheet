---
id: T-1502
title: "Union walks over sheets with many rows filled in other columns: find a union area's rows per column"
milestone: M1
lane: perf
status: done
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
- [x] A unit test: the MAP above with 9,000 rows filled in column G is 80000, and a dense adversarial union still stops at `#VALUE!` in bounded time.
- [x] Storage write/load benchmarks show no material regression.
- [x] `make check` green; reviewers approve.

## Out of scope
- The `MAX_AREA_WALK` value and single-range (dense) walks.

## Outcome
- `SheetData` gains a lazily built per-column row index (`column_rows: Option<BTreeSet<(column,row)>>`) plus a layout version bumped on every add/remove. A union area over more than 1,024 of the sheet's rows with cells in at most 16 columns builds it (once per sheet) and walks its columns through it (`Model::for_each_indexed_position`, util.rs), reading 64 rows per search and merging in row-major order; other areas keep the row walk. Budget: a step per index search and per cell, a row at least one step per four live columns; references.md "Union area cap" updated.
- `MAP(A1:A10000,LAMBDA(r,SUM((B:B,C:C))))` with G1:G9000 filled is 80000 (was `#VALUE!`), 9 ms; dense adversarial unions still stop at `#VALUE!`.
- Benchmarks (release, ad-hoc timing test, main -> branch): set_cell 1M x 1 column 64 -> 68 ms, 10k x 100 24 -> 25 ms (best of 9; the index is not built by writes or loads); decode 67/11 -> 67/11 ms; index build 20-32 ms per 1M cells on first long walk. Worst budget shapes: dense 16-copy union 291 -> 339 ms, 17-column row walk 339 -> 312 ms, one row per chunk 317 ms #VALUE! -> 111 ms value, column B only 177 -> 6 ms, staggered 16 columns (one cell per row) 203 ms; all bounded. An always-on index had made fresh writes 2-4.5x slower, hence lazy.
- Eval delta: none (perf change). Follow-up: T-1521 (index upkeep under heavy rewrites, undo snapshot copies).
