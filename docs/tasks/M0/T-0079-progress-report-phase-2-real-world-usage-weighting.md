---
id: T-0079
title: "Progress report phase 2: real-world usage weighting from calipers and poi call counts"
milestone: M0
lane: harness
status: todo
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
- [ ] report.json `functions[]` gains `real_world_calls`: how many formula cells in the calipers
  and poi suites call the function. The suite list is one named constant. Unit test.
- [ ] The `parity` block gains one line: the share of real-world calls landing within 15
  digits, weighting each function's attributed within-15-digit share by its
  `real_world_calls`. The `top-functions` block (T-0073) gains a "real-world calls" column.
- [ ] When calipers or poi is skipped, the line says so instead of showing a partial number
  (test).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the catalog `prior` values.
