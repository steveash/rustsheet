---
id: T-1019
title: "XMATCH/XLOOKUP binary search_mode over blanks and error cells"
milestone: M1
lane: engine
status: in-progress
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
XMATCH and XLOOKUP with search_mode 2/-2 give Excel's results over ranges with blank and error cells, matching the oracle.

## Context
Remaining failures of probes/xmatch-xlookup-blanks after T-1018: Col!M16, M17, M20, M23, M44 and Err!M6, Row!A18 (see the open note in docs/excel-semantics/lookup.md, XMATCH / XLOOKUP linear search over blanks). Excel's binary search skips blanks and errors like MATCH's approximate search (`SearchVector::approximate_search`, T-0212); the engine's `binary_search_*` helpers (functions/binary_search.rs) do not. Reuse the T-0212 search rather than adding another. Also probe text/boolean lookup values with match_mode -1/1 over blanks and a blank lookup value against 0/""/FALSE cells (XMATCH/XLOOKUP linear modes, `linear_scan`), and move `linear_scan`/`MatchMode` to a neutral lookup module. Probe unsorted-data results with the oracle first (M20/M23 are unsorted).

## Acceptance criteria
- [ ] Cells above pass; baseline updated; unit tests pin them.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
