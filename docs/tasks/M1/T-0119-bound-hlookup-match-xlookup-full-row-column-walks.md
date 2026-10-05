---
id: T-0119
title: "Bound HLOOKUP/MATCH/XMATCH/LOOKUP whole-row and whole-column walks by the used range"
milestone: M1
lane: engine
status: todo
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
Lookup functions other than VLOOKUP no longer evaluate up to 1M (column) or 16k (row) empty cells per call for `A:A` / `1:1` references.

## Context
T-0111 bounded VLOOKUP's search column via `SheetData::last_row` (`LookupTable::first_column`). HLOOKUP's `first_row`, MATCH, XMATCH and LOOKUP still walk the full range: a hostile workbook can cost ~1M evaluations per call (directive 5). Empty cells never match, so truncation at the last used row/column is safe; add a last-used-column helper alongside `last_row`.

## Acceptance criteria
- [ ] Unit tests with full-row/column references for each function, including a range starting below the data.
- [ ] A timing note in the PR for `MATCH(x, A:A, 0)` in 10k cells before/after.
- [ ] `make check` green; reviewers approve.

## Out of scope
Per-recalc search-column caching (M3).

## Handoff (resumed)
Tried: truncate search vectors at the sheet-wide last used row/column (`LookupTable::first_row/first_column`, `as_vector(.., search=true)`, `clip_search_rows/columns` in lookup_table.rs, recording the skipped tail with `record_seen_empty_range`); LOOKUP's 3-arg result vector is cut after element `l`; `SheetData::last_column` added. `make check` green, eval delta none, timing 159.5 s -> 0.33 s (200 x MATCH(i,A:A,0), 10k cells, debug). Spill-restart tests for MATCH/LOOKUP/HLOOKUP pass. Security, architecture, quality, performance reviewers APPROVE (round 2).
Failed: excel-fidelity reviewer requested changes twice (blocking, then request-changes). The clip changes values, and the premise "empty cells never match" is false in this engine: `compare_values` treats Empty as 0/""/FALSE, so
- approximate search depends on vector length: `MATCH(99,A:A,1)` went 1048576 -> last used row (and would depend on unrelated filled cells such as B100);
- exact HLOOKUP/VLOOKUP/LOOKUP of 0/FALSE/blank used to match the first empty cell and now give #N/A unless another row extends further;
- rows are clipped for bounded ranges but columns only for open-ended ones (inconsistent).
No oracle evidence supports either old or new values.
Next (reviewer's preferred design): keep the vector length n and model the clipped tail as a virtual run of EmptyCell (no evaluation for indices past the last used row/column; binary search probes identical positions; for exact search a target equal to Empty matches the first tail index). Value-neutral, still bounded (directive 5). Keep the seen-empty recording. Then drop the "Empty cells never match" claim from docs/excel-semantics/lookup.md (the engine matches Empty to 0/FALSE/blank); the unit tests that pin `MATCH(99,A:A,1)`=3, `HLOOKUP(99,1:2,2)`="c", `LOOKUP(30,A:A,E:E)`=0 must change to the old values (or be marked unverified, pending T-0126). Extend T-0126 with exact-match probes (`VLOOKUP(0,A:B,2,FALSE)`, `HLOOKUP(FALSE,1:2,2,FALSE)`, blank lookup value with a blank tail) and bounded-range sorted cases; add `expected_source: hand` probe cases in eval/suites/probes. Open perf note: `last_column` is O(rows) per open-ended row lookup; consider caching per pass (M3).
