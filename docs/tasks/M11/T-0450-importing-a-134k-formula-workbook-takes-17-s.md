---
id: T-0450
title: "Importing a 134k-formula workbook takes 17 s (calipers pv26_001): find and fix the import hot spot"
milestone: M11
lane: perf
status: in-progress
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
Loading calipers/pv_goldens/pv26_001 (and its twin pv26_2_001) takes a small fraction of today's 17 s,
and the whole workbook calculates well within the 60 s eval case timeout. They then rejoin the
gated calipers suite.

## Context
Found while adding the calipers suite (T-0446). pv26_001 is an Excel-saved financial model with
134,045 formulas, mostly plain references, SUM (14k), IF, ISERROR, AVERAGE, MATCH and VLOOKUP. With
release builds on an idle 4-core box:
- rustsheet: load + save without recalculation 16.8 s; load + recalc + save 32.5 s, peak 88 MB.
- mog: load + save 6.1 s; load + recalc + save 8.3 s.
pv23_042 (122k formulas) loads in 1.1 s, so formula count alone does not explain it.

Under a parallel `make eval` the two workbooks pass 60 s, and a timed-out worker keeps running. They
also pushed corpus-bench 68-47 past its timeout during `make check`, so eval/suites/calipers/suite.toml
excludes them, citing this task.

Profile the import (`perf record` on a release build of
`Workbook::open` + `recalculate`); the hot spot could be in the importer, formula parsing,
shared-formula expansion, or name and style resolution. File any calculation-side cause beyond the
import as a follow-up.

## Acceptance criteria
- [ ] The release-build load time of pv26_001, before and after, is in the Outcome, and is at least 4x
      faster.
- [ ] pv26_001 and pv26_2_001 leave the `exclude` list in eval/suites/calipers/suite.toml if they
      calculate within 60 s, and `rustsheet-eval update-baseline --suite calipers` records their results with no new known failure
      elsewhere.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The 11-minute COUNTIFS/SUMIFS models pv26_002 and pv26_2_002 (T-0449).
