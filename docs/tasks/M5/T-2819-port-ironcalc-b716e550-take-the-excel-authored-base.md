---
id: T-2819
title: "Port ironcalc b716e550: take the Excel-authored BASE_DECIMAL.xlsx calc test workbook"
milestone: M5
lane: functions-math
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
The ironcalc-calc suite scores upstream's Excel-authored `BASE_DECIMAL.xlsx`, and every newly
scored BASE/DECIMAL cell matches Excel or is a recorded known failure with an owner.

## Context
Upstream IronCalc b716e550 ("extend testing for BASE, DECIMAL", 2026-10-25) replaces
`xlsx/tests/calc_tests/MATH_AND_TRIGONOMETRY/BASE.xlsx` (10,051 bytes) with `BASE_DECIMAL.xlsx`
(20,925 bytes). Its docProps/app.xml reads `Microsoft Excel` 16.0300 and it has a calcChain, so
its cached values are oracle evidence once you confirm the new cells are Excel-computed (17-digit
caches), not written by IronCalc. Our copy is
`crates/rustsheet-xlsx/tests/calc_tests/MATH_AND_TRIGONOMETRY/BASE.xlsx`. The code is
`fn_base` (math_and_trigonometry/mathematical.rs) and `fn_decimal` (decimal_text.rs).
f2f0b089 only reformats upstream's unit test. Precedent: T-2634. Triaged by the 2026-10-26
(18:47 UTC) steward run.

## Acceptance criteria
- [x] Our `BASE.xlsx` is replaced by upstream's `BASE_DECIMAL.xlsx` (b716e550); `make eval`
  reports the new cells; any excel-semantics citation of the old file is re-pointed.
- [x] New failures are fixed in `functions/math_and_trigonometry/` or recorded with
  `make eval-baseline` and filed as follow-up tasks; no regressions.
- [x] Commit carries `Ported-From: ironcalc@b716e550`.
- [x] `make check` green; reviewers approve.

## Out of scope
- Upstream's `test_base_decimal.rs` unit tests are IronCalc-authored, not Excel evidence; take
  only those whose expectations an Excel cell confirms.

## Outcome
Replaced `BASE.xlsx` with upstream's Excel-authored `BASE_DECIMAL.xlsx` (Microsoft Excel 16.0300,
calcChain present). All 358 checked cells pass; no code change needed, no regressions, no
follow-ups. Baseline case line renamed (61 -> 358 checked cells). No excel-semantics file cited the
old workbook.
