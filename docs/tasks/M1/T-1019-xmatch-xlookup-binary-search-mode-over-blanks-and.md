---
id: T-1019
title: "XMATCH/XLOOKUP binary search_mode over blanks and error cells"
milestone: M1
lane: engine
status: done
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
- [x] Cells above pass; baseline updated; unit tests pin them.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
- The oracle contradicted the Context's guess. Excel's binary search_mode does **not** skip blanks
  the way MATCH's approximate search does: M20/M23 (`XMATCH(99,A:A,±1,2)` = 1/2) need the blanks
  probed. Excel runs a classic inclusive-bounds binary search on the lower middle, where blanks
  and errors are larger than numbers and -1/1 return the last probed smaller/larger cell.
- `binary_search` (functions/binary_search.rs) replaces the four `binary_search_*` helpers and is
  shared by XMATCH and XLOOKUP (range and array paths). Unit tests pin every binary-mode oracle
  cell (test_binary_search.rs). XLOOKUP over a range now searches the full `SearchVector`
  (virtual empty tail) rather than the vector clipped to the used area, so its answer no longer
  depends on the used area. Runs of equal values return what main returned (unprobed, T-1134);
  their start is found by galloping. docs/excel-semantics/lookup.md has the new VERIFIED entry.
- Eval delta: probes fixed 7 (Col!M16, M17, M20, M23, M44, Err!M6, Row!A18), regressions 0;
  xmatch-xlookup-blanks now passes 69/69.
- Not done: the text/boolean and blank-lookup probes and the module move, filed as T-1134. The
  move is out of scope there too, since nothing needs it yet.
