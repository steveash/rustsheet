---
id: T-0079
title: "Progress report phase 2: real-world usage weighting from calipers and poi call counts"
milestone: M0
lane: harness
status: done
priority: P3
size: M
tier: opus
depends_on: [T-0068, T-0081, T-0073]
spec: docs/specs/SPEC-0022-daily-progress-dashboard.md
---

## Goal
The dashboard shows what share of real-world function calls land within 15 digits. Calls are
weighted by how often each function appears in the Excel-saved corpora, calipers and poi, not by
the catalog prior alone.

## Context
- SPEC-0022 lists usage weighting as a later phase.
- The assessment counted 6.6M calls in calipers and POI formulas. The top 20 functions by those
  calls were 97.7% of real-world calls.
- The count belongs in the Rust report, next to T-0068's attributed counts (`RunReport::build`,
  `formula_calls::functions_in`), with no Python formula parsing.

## Acceptance criteria
- [x] report.json `functions[]` gains `real_world_calls`: how many formula cells in the calipers
  and poi suites call the function. The suite list is one named constant. Unit test.
- [x] The `parity` block gains one line: the share of real-world calls landing within 15
  digits, weighting each function's attributed within-15-digit share by its
  `real_world_calls`. The `top-functions` block (T-0073) gains a "real-world calls" column.
- [x] When calipers or poi is skipped, the line says so instead of showing a partial number
  (test).
- [x] `make check` green; reviewers approve.

## Out of scope
- Changing the catalog `prior` values.

## Outcome
- report.json `functions[].real_world_calls` counts the formula cells of the Excel-saved corpora that call each
  function (deviations included, array members without a formula of their own left out), and the top-level
  `real_world_suites` names those suites: `REAL_WORLD_SUITES` (calipers, poi) in crates/rustsheet-eval/src/report.rs is
  the one definition; the progress page reads the list from the report.
- The `parity` block gains the real-world line: each function's attributed within-15-digit share weighted by its calls.
  When calipers or poi did not run, it says which instead of a partial number. The `top-functions` block gains a
  "Real-world calls" column. SPEC-0022 rows 6 and 7, the method note and docs/architecture/eval-harness.md say so.
- `make eval` with calipers and poi fetched (2026-10-26, this branch): **99.95%** of the 1,327,360 calls to attributed
  in-scope functions in the calipers and poi formula cells land within 15 digits. The most-called functions are IF
  (273,106 cells), OR (137,989), SUM (111,180), OFFSET (109,140) and SUMIF (95,352). The count is formula cells per
  function, not call sites, so it is below the assessment's 6.6M calls. docs/progress.md shows the line on the steward's
  next `make progress-report`.
- Eval delta: none (no scoring change).
