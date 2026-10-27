---
id: T-2868
title: "Port ironcalc 63ff3c35: take the Excel-authored BASE_DECIMAL_issue_1617.xlsx calc test workbook"
milestone: M5
lane: functions-math
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
The ironcalc-calc suite scores upstream's Excel-authored `BASE_DECIMAL_issue_1617.xlsx`, and every
newly scored cell matches Excel or is a recorded known failure with an owner.

## Context
Upstream IronCalc 63ff3c35 ("Adds missing test case", 2026-10-26) adds
`xlsx/tests/calc_tests/MATH_AND_TRIGONOMETRY/BASE_DECIMAL_issue_1617.xlsx` (8,185 bytes) for upstream
141a41d0, which makes `DECIMAL("",2)` 0 instead of `#VALUE!`. `rustsheet-eval snapshot` shows
application `Microsoft Excel` 16.0300 and two formula cells on sheet DECIMAL, C2 and C3
(`_xlfn.DECIMAL(A2,B2)` with A2 `=""` and `DECIMAL(A3,B3)`), both cached 0. Our rule already says
empty text is 0 (docs/excel-semantics/engineering.md, "DECIMAL of unreadable text"), so 141a41d0's
code change needs no port; this task only takes the workbook as Excel evidence. Copy it into
`crates/rustsheet-xlsx/tests/calc_tests/MATH_AND_TRIGONOMETRY/`. Precedent: T-2819. Triaged by the
2026-10-27 steward run.

## Acceptance criteria
- [x] `BASE_DECIMAL_issue_1617.xlsx` is in the ironcalc-calc suite and `make eval` scores its cells.
- [x] New failures are fixed in `functions/math_and_trigonometry/` or recorded with
  `make eval-baseline` and filed as follow-ups; no regressions.
- [x] The commit carries `Ported-From: ironcalc@63ff3c35`.
- [x] `make check` green; reviewers approve.

## Out of scope
- Upstream's unit test in `test_base_decimal.rs` is IronCalc-authored, not Excel evidence.

## Outcome
Took `BASE_DECIMAL_issue_1617.xlsx` (Excel 16.0300, `DECIMAL("",2)` cached 0) into the ironcalc-calc
suite. It scores 3/3 cells passing with no code change, confirming the existing "DECIMAL of unreadable
text" rule. ironcalc-calc: 221 cases, 72,961 cells, 72,338 pass; baseline gains one `C` line for the new case and no `F` lines (no known-failure
shrink, no regressions).
