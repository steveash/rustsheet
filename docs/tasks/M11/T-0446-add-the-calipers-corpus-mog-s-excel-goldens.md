---
id: T-0446
title: "Add the Calipers corpus (mog's Excel goldens) as a fetched eval suite"
milestone: M11
lane: harness
status: done
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
The Excel goldens of Calipers, the verification corpus of the mog engine
(https://github.com/fundamental-research-labs/calipers, pinned), are a fetched, gated eval suite. Every
`golden.xlsx` is recalculated and scored against Excel's own cached values, so work on data tables
(T-0441), iterative calculation (T-0447) and range-heavy recalculation (T-0448, T-0449) is measured on
the corpus a competing engine is developed against.

## Context
Owner request (2026-10-22), after a head-to-head of rustsheet and mog (docs/research/mog-head-to-head-2026-10-22.md).
- The corpus has 458 `verification/cases/<suite>/<case>/golden.xlsx` files, all saved by desktop Excel:
  107 real financial models (`pv_goldens`, with formula caches), 83 load-and-save round trips, 5
  recalculation cases and 263 Office.js script outputs. The engine cannot run the Office.js scripts,
  but their goldens are ordinary Excel-saved workbooks whose cached values are an oracle like POI's.
- Measured before this task (values only, 132 cases both engines loaded): 99.55% of 1.56M formula
  cells bit-exact. Rustsheet refuses 19 `pv_goldens` workbooks and 2 round trips (`Not Implemented Error:
  data table formulas`, T-0441); two models with `<calcPr iterate="1">` (pv23_021, pv26_004) give `#CIRC!`
  (T-0447); pv26_002 and pv26_2_002 take about 11 minutes each to calculate.
- Licence: Calipers is MIT. The `pv_goldens` workbooks were imported into it from a repository that is
  not public (lyfegame/shortcut, anonymised by its authors), so the suite is fetched into
  `eval/.cache`, never vendored, as corpus-bench is.
- Nothing about Calipers' own `verify` gate (formula text, styles) is in scope: the suite scores values.

## Acceptance criteria
- [x] pv26_002 and pv26_2_002 are excluded with a comment naming T-0449, which owns their calculation
      time, because a timed-out case's worker keeps running until the run ends.
- [x] `eval/suites/calipers/suite.toml` fetches Calipers at a pinned full commit id (`[fetch]` with
      `rev` and `subdir = "verification/cases"`), includes `**/golden.xlsx`, excludes `_disabled/**`,
      scores cached values, and keeps a `cells` baseline that `rustsheet-eval check --suite calipers` gates.
- [x] `rustsheet-eval fetch --suite calipers` then `rustsheet-eval update-baseline --suite calipers` records the suite's current known
      failures in eval/baseline/calipers.txt; no other baseline changes.
- [x] docs/research/mog-head-to-head-2026-10-22.md records the comparison, its method and its numbers;
      docs/excel-semantics/recalc.md notes that Excel-authored data-table workbooks now exist in a suite.
- [x] `make check` green; reviewers approve.

## Out of scope
- Fixing any failure the suite finds (T-0441, T-0447, T-0448, T-0449 own the known ones).
- Calipers' Office.js scripts and its formula-text and style comparisons.

## Outcome
- eval/suites/calipers fetches Calipers 9e47ea4f (`rustsheet-eval fetch --suite calipers`, 458 goldens, about
  170 MB) and scores 454 of them. pv26_002 and pv26_2_002 (11 minutes, T-0449) and pv26_001 and pv26_2_001
  (17 s import, T-0450) are excluded. With the last two included, `make check` timed out
  corpus-bench 68-47 under the extra load.
- eval/baseline/calipers.txt: 173 cases with formula cells, 6,679 known failures. These include 21
  `!EngineError` workbooks with data tables (T-0441) and the `#CIRC!` cells of pv23_021 and pv26_004
  (T-0447). No other baseline changed.
- `gate = false`, so `make check` does not run it. In the same parallel run as the other suites,
  corpus-bench 59969 and 68-47 passed the 60 s case timeout in 2 of 3 `make check` runs; without
  calipers, `make eval-check` was green. `rustsheet-eval check --suite calipers` gates it, and
  `rustsheet-eval update-baseline --suite calipers` ratchets it.
- Excel stores the members of a volatile spill as an empty `<f ca="1"/>`; they were scored against
  RANDARRAY's random values and flapped. The snapshot reader now treats them as array members, and
  `scorable_cells` skips members of a volatile anchor (xlsx-storage.md).
- Final baseline: 6,634 known failures. `make eval` takes about 4 minutes longer with the suite fetched.
- Comparison numbers: docs/research/mog-head-to-head-2026-10-22.md.

