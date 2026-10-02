---
id: T-0111
title: "Fix 60s timeouts on POI full-column VLOOKUP and structured-reference workbooks"
milestone: M1
lane: engine
status: todo
priority: P0
size: M
depends_on: []
---

## Goal
`poi/VLookupFullColumn.xlsx` and `poi/StructuredRefs-lots-with-lookups.xlsx` evaluate within the harness timeout.

## Context
Found by T-0003. Both run at or beyond the 60s case timeout (so they are currently excluded in `eval/suites/poi/suite.toml`). Likely cause: lookup functions evaluate full-column ranges cell by cell (`evaluate_cell` per position) instead of bounding to the used range. Untrusted workbooks must not cause unbounded work (CLAUDE.md directive 5).

## Acceptance criteria
- [ ] Both cases finish in under 10s in release mode.
- [ ] Both are removed from the `exclude` list in `eval/suites/poi/suite.toml` (they were excluded because near-timeout runs made the gate nondeterministic) and baselined via `make eval-baseline`; any cell mismatches are recorded as ordinary failures.
- [ ] `make check` green; reviewers approve.

## Out of scope
General dependency-graph rewrite (M3).
