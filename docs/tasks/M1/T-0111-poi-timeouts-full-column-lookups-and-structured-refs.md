---
id: T-0111
title: "Fix 60s timeouts on POI full-column VLOOKUP and structured-reference workbooks"
milestone: M1
lane: engine
status: done
priority: P0
size: M
depends_on: []
---

## Goal
`poi/VLookupFullColumn.xlsx` and `poi/StructuredRefs-lots-with-lookups.xlsx` evaluate within the harness timeout.

## Context
Found by T-0003. Both run at or beyond the 60s case timeout (so they are currently excluded in `eval/suites/poi/suite.toml`). Likely cause: lookup functions evaluate full-column ranges cell by cell (`evaluate_cell` per position) instead of bounding to the used range. Untrusted workbooks must not cause unbounded work (CLAUDE.md directive 5).

## Acceptance criteria
- [x] Both cases finish in under 10s in release mode.
- [x] Both are removed from the `exclude` list in `eval/suites/poi/suite.toml` (they were excluded because near-timeout runs made the gate nondeterministic) and baselined via `make eval-baseline`; any cell mismatches are recorded as ordinary failures.
- [x] `make check` green; reviewers approve.

## Out of scope
General dependency-graph rewrite (M3).

## Outcome
VLOOKUP's search column is no longer materialized past the sheet's last used row (new cheap
`SheetData::last_row`), so `A:B`-style lookups cost O(used rows), not 1M cell evaluations. Both POI
cases now run (VLookupFullColumn 0.02s, StructuredRefs-lots-with-lookups ~3.6s, release) and are
un-excluded; `LookupTable` moved to its own module (file-size ratchet).

Second change: `main` carried a poi regression from T-0914 (TANH(-0.99505…) is 1 ULP off); TANH uses
an `e^2x` form for 0.9 <= |x| < 1, which fits that cell and SINH_COSH_TANH!D15/D17.

Baseline: besides the two re-included cases (24 newly scored cells failing), `eval/baseline/poi.txt`
gained ~26 crash-only `C`/`F` lines. These cases already existed in the fetched suite but the T-0003
baseline predates T-0007's crash-only reporting, so they were never recorded; none is caused by this
change. Filed T-0118 (panics, xmlbomb limits) and T-0119 (HLOOKUP/MATCH/XMATCH/LOOKUP bounds).
