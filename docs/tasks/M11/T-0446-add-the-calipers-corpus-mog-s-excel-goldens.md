---
id: T-0446
title: "Add the Calipers corpus (mog's Excel goldens) as a fetched eval suite"
milestone: M11
lane: harness
status: todo
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
- [ ] pv26_002 and pv26_2_002 are excluded with a comment naming T-0449, which owns their calculation
      time, because a timed-out case's worker keeps running until the run ends.
- [ ] `eval/suites/calipers/suite.toml` fetches Calipers at a pinned full commit id (`[fetch]` with
      `rev` and `subdir = "verification/cases"`), includes `**/golden.xlsx`, excludes `_disabled/**`,
      scores cached values, and is gated with a `cells` baseline.
- [ ] `rustsheet-eval fetch --suite calipers` then `make eval-baseline` records the suite's current known
      failures in eval/baseline/calipers.txt; no other baseline changes.
- [ ] docs/research/mog-head-to-head-2026-10-22.md records the comparison, its method and its numbers;
      docs/excel-semantics/recalc.md notes that Excel-authored data-table workbooks now exist in a suite.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fixing any failure the suite finds (T-0441, T-0447, T-0448, T-0449 own the known ones).
- Calipers' Office.js scripts and its formula-text and style comparisons.
