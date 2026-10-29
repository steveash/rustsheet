---
id: T-3162
title: "Port ironcalc 8711004d/85ddea84: take upstream's consolidated Excel-authored BASE_DECIMAL.xlsx (issues 1611, 1615, 1616, 1618)"
milestone: M5
lane: functions-math
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
The ironcalc-calc suite scores upstream's current Excel-authored `BASE_DECIMAL.xlsx`, and every
newly scored cell matches Excel or is a recorded known failure with an owner.

## Context
Upstream IronCalc fixed four DECIMAL/BASE issues on 2026-10-28/29 (05f9b8be leading spaces in
DECIMAL text, #1618; 66c295c5 BASE of a negative fraction is `#NUM!`, #1611; 93e11073 DECIMAL of
signed text is `#NUM!`, #1615; a90ceccf DECIMAL text over 255 characters is `#VALUE!`, #1616),
then 8711004d and 85ddea84 folded every `BASE_DECIMAL_issue_*.xlsx` into new sheets of
`xlsx/tests/calc_tests/MATH_AND_TRIGONOMETRY/BASE_DECIMAL.xlsx` (26,735 bytes at upstream
85ddea84). `rustsheet-eval snapshot` shows application `Microsoft Macintosh Excel` 16.0300 and
sheets BASE and DECIMAL (241 and 117 formula cells, as in our copy) plus issue_1618 (13),
issue_1611 (4), issue_1613 (5), issue_1617 (5), issue_1615 (3) and issue_1616 (2).
Our code already follows all four rules (`decimal_text.rs` `decimal_from_text` and its 255-unit
check; `mathematical.rs` `base_text` checks the sign before truncating), so no code port is
expected; this task takes the workbook as Excel evidence. Replace our
`crates/rustsheet-xlsx/tests/calc_tests/MATH_AND_TRIGONOMETRY/BASE_DECIMAL.xlsx` with upstream's;
our separate `BASE_DECIMAL_issue_1613.xlsx` and `BASE_DECIMAL_issue_1617.xlsx` then duplicate two
sheets, so drop them only if their cells are scored identically from the new sheets. The file is
Mac-desktop authored: a one-ulp difference from the web oracle follows decision 0074, not a fix.
Precedents: T-2868, T-2819. Triaged by the 2026-10-29 18:47 UTC steward run.

## Acceptance criteria
- [ ] The new `BASE_DECIMAL.xlsx` is in the ironcalc-calc suite and `make eval` scores its cells.
- [ ] New failures are fixed in `functions/math_and_trigonometry/` or recorded with
      `make eval-baseline` and filed as follow-ups; no regressions.
- [ ] The commit carries `Ported-From: ironcalc@85ddea84`.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Upstream's unit tests in `test_base_decimal.rs` are IronCalc-authored, not Excel evidence.
