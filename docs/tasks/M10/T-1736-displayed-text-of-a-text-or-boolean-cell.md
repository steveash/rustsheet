---
id: T-1736
title: "Displayed text of a text or boolean cell goes through the format's text section"
milestone: M10
lane: functions-text
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
A text or boolean cell's displayed text goes through the format's text section (`@` with literals, an empty fourth section hides it), as Excel does.

## Context
Codes `"pre "@` (TRUE → "pre TRUE", "text" → "pre text"), `@" suffix"`, `;;;"hidden"@` ("text" →
"hiddentext", FALSE → "hiddenFALSE"), `;;;` (text and TRUE → ""), `0.0;[Red]-0.0;0.0;"<"@">"` ("" → "<>",
TRUE → "<TRUE>"), `0;-0;"zero";"text: "@` (TRUE → "text: TRUE"). Ours shows the plain text or boolean. Note that
a boolean goes through the text section here. T-1130 probes TEXT() on text; this is the cell display.
Evidence: suite gen-m10 (excel-graph, 2026-10-16, T-1611): `make eval` report, "Displayed text" section, which lists each failing text cell (Excel vs ours). Text cells are scored apart from values and are not baselined (decision 0068); the TEXT() cells on sheet T that use the same codes are value known failures in eval/baseline/gen-m10.txt. Formatter: crates/rustsheet-engine/src/formatter/ (shared by `Workbook::formatted` and TEXT). docs/excel-semantics/number-formats.md.

## Acceptance criteria
- [ ] The gen-m10 text cells under these codes pass in the report's "Displayed text" section (or the remaining ones are filed); the matching TEXT() cells on sheet T leave eval/baseline/gen-m10.txt; no regressions.
- [ ] number-formats.md updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
