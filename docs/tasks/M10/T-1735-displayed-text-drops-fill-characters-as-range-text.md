---
id: T-1735
title: "Displayed text drops * fill characters, as Range.text does (accounting formats, *.0.00, 0.00*-)"
milestone: M10
lane: functions-text
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
A `*x` fill (repeat) character adds nothing to the displayed text, as Excel's `Range.text` shows it; `_x` still adds a space.

## Context
The four accounting built-ins (`_(* #,##0_);...`, `_("$"* #,##0.00_);...`; 320 text cells, all fail),
`*.0.00` and `0.00*-` (146 fail). Excel: 1000 under `_(* #,##0_)...` " 1,000 ", 1234567890 under
`_("$"* #,##0.00_)...` " $1,234,567,890.00 ", PI() under `*.0.00` "3.14", under `0.00*-` "3.14"; ours adds a
space for the fill (" $ 36.50 ", " 3.14", "3.14 "). Check what TEXT() gives for the same codes on sheet T (Excel's
TEXT may differ from the displayed text) before changing the shared formatter.
TEXT shows the same bug: `TEXT(7,"0*""a")` is "7a" in Excel and "7 a" with us
(probes/text-format-length-limit-o A37, a known failure in eval/baseline/probes.txt; T-1486). The `*`
takes the `"` as its fill character, and a fill adds nothing in a number section, but format.rs prints
a space for every `TextToken::Spacer` (in `format_number`, both the date and the number branches).
Evidence: suite gen-m10 (excel-graph, 2026-10-16, T-1611): `make eval` report, "Displayed text" section, which lists each failing text cell (Excel vs ours). Text cells are scored apart from values and are not baselined (decision 0068); the TEXT() cells on sheet T that use the same codes are value known failures in eval/baseline/gen-m10.txt. Formatter: crates/rustsheet-engine/src/formatter/ (shared by `Workbook::formatted` and TEXT). docs/excel-semantics/number-formats.md.

## Acceptance criteria
- [ ] The gen-m10 text cells under these codes pass in the report's "Displayed text" section (or the remaining ones are filed); the matching TEXT() cells on sheet T leave eval/baseline/gen-m10.txt; no regressions.
- [ ] number-formats.md updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
