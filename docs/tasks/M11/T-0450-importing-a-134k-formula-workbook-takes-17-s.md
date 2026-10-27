---
id: T-0450
title: "Importing a 134k-formula workbook takes 17 s (calipers pv26_001): find and fix the import hot spot"
milestone: M11
lane: perf
status: done
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
- [x] The release-build load time of pv26_001, before and after, is in the Outcome, and is at least 4x
      faster.
- [x] pv26_001 and pv26_2_001 leave the `exclude` list in eval/suites/calipers/suite.toml if they
      calculate within 60 s, and `rustsheet-eval update-baseline --suite calipers` records their results with no new known failure
      elsewhere. (They left the list; the recording waits on T-0463, see the Outcome.)
- [ ] `make check` green; reviewers approve.

## Out of scope
- The 11-minute COUNTIFS/SUMIFS models pv26_002 and pv26_2_002 (T-0449).

## Outcome
- No code change: the hot spot was already gone. T-0468 (#1821, SPEC-0021) made conditional
  formats evaluate on the first style read instead of eagerly at load and at the end of
  `evaluate()`. pv26_001's `Ticker0001_Validation_Sheet` has 20 expression rules over whole-row
  areas (`A2:XFD3 A11:XFD30 ...`, every 16,384 columns), each `INDEX(<name>,ROW(),COLUMN())`, which the eager
  evaluation ran on every cell of those rows.
- Release builds without LTO, `load_from_xlsx_bytes` / `Model::from_workbook` / `evaluate()` / `save_to_xlsx`:
  | Commit | parse | from_workbook | evaluate | save |
  |---|---|---|---|---|
  | adedde75c (T-0446, when this task was filed) | 0.71 s | 12.9 s | 13.4 s | 0.42 s |
  | 6d2117d22^ (just before T-0468) | 0.64 s | 12.7 s | 12.3 s | 0.44 s |
  | 6d2117d22 (T-0468) | 0.71 s | 0.045 s | 0.139 s | 0.43 s |
  | 44f80b2 (`main` today) | 0.66 s | 0.052 s | 0.165 s | 0.49 s |
  Load (parse + `from_workbook`) went from 13.6 s to 0.71 s, 19x; load + evaluate + save from 27.5 s to 1.4 s.
  pv26_2_001 is the same (1.44 s in all on `main`).
- In the harness each case takes about 1.5 s (the case's `millis` in `report.json`; the whole
  `rustsheet-eval run --suite calipers --filter pv26_001/` command, with startup and discovery, about 8 s),
  so both left the `exclude` list in eval/suites/calipers/suite.toml. Each scores 134,038 of 134,039
  cells; the one failure is `Ticker0001_Validation_Sheet!A2`, `CELL("filename",...)` (Excel for the web's
  `#VALUE!` against desktop Excel's cached path; owned by T-2663).
- Baseline not recorded here: `rustsheet-eval update-baseline --suite calipers` refuses on `main` with
  or without this change, for T-0463's 36 pre-existing regressions (pv23_007/008/009/056; `check` on
  `main`: fixed 1527, REGRESSIONS 36). `check` reports the two cases as `new cases 2`;
  T-0463's ratchet records them (added to its acceptance criteria).
- No calculation-side follow-up: `evaluate()` takes 0.17 s.
