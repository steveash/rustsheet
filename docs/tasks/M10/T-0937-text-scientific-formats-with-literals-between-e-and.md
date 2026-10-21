---
id: T-0937
title: "TEXT scientific formats with literals between E and the exponent digits (10 known failures)"
milestone: M10
lane: functions-text
status: done
priority: P3
size: S
tier: opus
campaign: visible
depends_on: []
---

## Goal
Scientific formats that put literal text between `E+`/`E-` and the exponent digits format like
Excel instead of returning `#VALUE!`.

## Context
Evidence: `poi/NumberFormatApproxTests.xlsx` (excel-graph, T-0019), formula `=TEXT(Cn, Bn)`, all
TypeMismatch with actual `#VALUE!`:
- `|#.#|E-|#|`: A40 (123456.789) "|1.2|E|5|", A96 (1.23456789E-05) "|1.2|E|-5|", A136 (0) "|0.|E|0|",
  A176 (-123456.789) "-|1.2|E|5|", A216 (-1.23456789E-05) "-|1.2|E|-5|".
- `|#.#|E+|#|`: A41 "|1.2|E|+5|", A97 "|1.2|E|-5|", A137 "|0.|E|+0|", A177 "-|1.2|E|+5|",
  A217 "-|1.2|E|-5|".
What the evidence shows: the exponent sign goes next to the exponent digits, after the literal
`|`, not right after `E`. The number's minus sign goes at the very front. A zero mantissa with
`#.#` prints "0." (an integer digit appears even with a `#` placeholder). Formatter:
`crates/rustsheet-engine/src/formatter/` (lexer and the scientific branch of `format.rs`).

## Acceptance criteria
- [x] The 10 lines above (A40, A41, A96, A97, A136, A137, A176, A177, A216, A217) disappear from
      eval/baseline/poi.txt with no new F lines.
- [x] Rules recorded in `docs/excel-semantics/number-formats.md` citing these cells.
- [x] `make check` green; reviewers approve.

## Out of scope
The `?`/fraction failures in `poi/NumberFormatTests.xlsx` (excel-cached, pre-existing).

## Visible-parity stage
Decision 0083: a wrong TEXT or displayed-text result is visible to every user of the format, so this task is in the campaign. Its acceptance criteria are already the visible-parity stage.

## Outcome
- Since T-1730 the cells no longer gave `#VALUE!`, but they put `E` and its sign after the literal ("|1.2||E5|"). Now
  `formatter/parser.rs` pushes the `E` as a literal where the format's `E+`/`E-` sits, and `format.rs` prints only the
  exponent's sign before its first digit. A zero mantissa prints "0" for the last `#` integer placeholder.
- Eval delta: poi known failures 142 -> 132 (the 10 listed lines); gen-m10 text failures 1376 -> 1374 (`#E+0` on 0 now
  displays "0E+0", 5eb95c63c7d6180c F!G13 and 71ad5f93362f4ff4 F!V19); 0 regressions in every gated suite.
- Rule extended in `docs/excel-semantics/number-formats.md` "Scientific formats with literals after `E`"; unit test
  `test_scientific.rs` `scientific_literals_before_the_exponent_digits`.
