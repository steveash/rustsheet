---
id: T-2350
title: "Port ironcalc 1fd0ff4f: take the extended LOG_LOG10_LN.xlsx calc test workbook"
milestone: M5
lane: functions-math
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
The ironcalc-calc suite scores upstream's extended `LOG_LOG10_LN.xlsx`, and every newly scored
LOG/LOG10/LN cell either matches Excel or is a recorded known failure with an owner.

## Context
Upstream IronCalc 1fd0ff4f ("extend xlsx testing for LOG, LOG10, LN", 2026-10-13) grows
`base/src/test/calc_tests/MATH_AND_TRIGONOMETRY/LOG_LOG10_LN.xlsx` from 13,094 to 16,512 bytes.
Our copy is `crates/rustsheet-xlsx/tests/calc_tests/MATH_AND_TRIGONOMETRY/LOG_LOG10_LN.xlsx`. The
workbook is Excel-authored, so its cached values are oracle evidence (check that the new cells are
Excel-computed, not IronCalc-written, before trusting them). Triaged by the 2026-10-22 steward run.

## Acceptance criteria
- [ ] The workbook is replaced by upstream's 1fd0ff4f version; `make eval` reports the new cells.
- [ ] New failures are fixed in `functions/math_and_trigonometry/` or recorded with
  `make eval-baseline` and filed as follow-up tasks; no regressions.
- [ ] Commit carries `Ported-From: ironcalc@1fd0ff4f`.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Upstream 7e271963 / 34b8bb6e (dependency bumps). Upstream's statrs 0.18 -> 0.19.1 bump changed
  CHISQ results (its `test_fn_chisq.rs` expectation changed from 7 to 6.999999999), so do not take statrs 0.19
  as part of this task.
- eb9ca75b (version bump only).

## Outcome
Took upstream `LOG_LOG10_LN.xlsx` (Excel-authored: docProps Application is Microsoft Excel) from ironcalc@1fd0ff4f. ironcalc-calc coverage for the workbook grows 93 -> 227 cells, all 227 pass (new-coverage failures: none, regressions 0); the baseline only records the new checked-cell count. The new file replaces the 3-sheet Macintosh workbook, so the numbers.md "LOG with one argument is LOG10" citation was re-pointed at the new cells and the base-10, domain and coercion findings were recorded. Provenance: created by a script (openpyxl), saved in Microsoft Excel 16.0300 (app.xml, calcChain, 17-digit caches); it has inert webextension parts. No follow-ups.
