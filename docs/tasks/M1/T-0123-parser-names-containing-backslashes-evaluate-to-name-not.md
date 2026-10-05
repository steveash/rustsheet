---
id: T-0123
title: "Parser: names containing backslashes evaluate to #NAME?, not #ERROR! (6 known failures)"
milestone: M1
lane: engine
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Identifiers that contain `\` (valid characters in Excel defined names) parse as names. An
undefined one evaluates to `#NAME?` like Excel, instead of failing to parse (`#ERROR!`).

## Context
Evidence: `poi/46535.xlsx` (excel-graph, T-0019), sheet Others, all ErrorMismatch, expected
`#NAME?`, actual `#ERROR!`: L12 `=IF(HSI\10_1\1460_1\1470_1___val<>"",HSI\10_1\1460_1\1470_1___val,"")`,
M12 (`HSI\10_1\1460_2\1470_1___val`), M17, M18, M19, M20 (`HSI\10_1\1340_{10,11,7,8}\1420_1\1430_1___val`).
The names are not defined in the workbook. The lexer in
`crates/rustsheet-engine/src/expressions/lexer/` rejects `\` inside an identifier.

## Acceptance criteria
- [x] The 6 lines above disappear from eval/baseline/poi.txt with no new F lines.
- [x] Lexer unit tests: `A\B`, `\name`, `HSI\10_1___val` lex as one name token, and the formula
      text round-trips through parse and print.
- [x] `make check` green; reviewers approve.

## Out of scope
Defining such names through the API. Other name-character rules not covered by this evidence.

## Outcome
The lexer accepts `\` inside and at the start of names when the decimal separator is `.` (in comma-decimal locales it stays the array column separator). Also allowed in `is_valid_a1_identifier`. poi known failures 950 -> 935 (15 fixed, 0 regressions); the 6 listed lines plus 9 more names with backslashes.
