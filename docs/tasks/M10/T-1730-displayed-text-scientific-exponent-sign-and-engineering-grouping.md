---
id: T-1730
title: "Displayed text: scientific exponent sign and engineering grouping (E+0 for a zero exponent, ##0.0E+0, 00.0E+0)"
milestone: M10
lane: functions-text
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Scientific formats show the exponent sign and group the mantissa as Excel's displayed text does.

## Context
Codes `0.00E+00`, `0.0E+00`, `0E+0`, `#E+0`, `0.000E-00`, `##0.0E+0`, `##0.00E+00`, `00.0E+0` (~240 failing
text cells). Excel: 1.5 `0.00E+00` "1.50E+00" (ours "1.50E-00": a zero exponent gets `-`); `0.000E-00` 1.5
"1.500E00" (ours "1.500E-00"); `##0.0E+0` 123456.789 "123.5E+3" and 61 "61.0E+0" (engineering: the exponent a
multiple of the integer-placeholder count; ours "1.2E+5", "6.1E+1"); `00.0E+0` 36.5 "36.5E+0", 2958465.99999
"03.0E+6" (ours "03.6E+1", "02.0E+6"). T-0937 (literals between E and the digits) is separate.
Evidence: suite gen-m10 (excel-graph, 2026-10-16, T-1611): `make eval` report, "Displayed text" section, which lists each failing text cell (Excel vs ours). Text cells are scored apart from values and are not baselined (decision 0068); the TEXT() cells on sheet T that use the same codes are value known failures in eval/baseline/gen-m10.txt. Formatter: crates/rustsheet-engine/src/formatter/ (shared by `Workbook::formatted` and TEXT). docs/excel-semantics/number-formats.md.

## Acceptance criteria
- [ ] The gen-m10 text cells under these codes pass in the report's "Displayed text" section (or the remaining ones are filed); the matching TEXT() cells on sheet T leave eval/baseline/gen-m10.txt; no regressions.
- [ ] number-formats.md updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
