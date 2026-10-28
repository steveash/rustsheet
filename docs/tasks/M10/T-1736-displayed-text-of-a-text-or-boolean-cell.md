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

More codes (gap triage 2026-10-25, same root cause): the four accounting formats' text section `_(@_)` pads a text, a
boolean or an empty string with the width of `(` before and of `)` after, so Excel shows " text ", " TRUE ", " FALSE " and,
for `=""`, two spaces; ours shows the plain value. 27 gen-m10 displayed-text cells (`make eval` report, "Displayed text"):
- `_(* #,##0_);_(* \(#,##0\);_(* "-"_);_(@_)` (8): 05c1c646c95a2d60 F!M4 ("text" → " text "), F!M17; 5255389143dbcef6 F!H4,
  F!H11 (`""` → "  "); 659cd5776d423ce3 F!C11; eec351e03ccedfe0 F!R2 (TRUE → " TRUE "), F!R12, F!R20
- `_("$"* #,##0.00_);_("$"* \(#,##0.00\);_("$"* "-"??_);_(@_)` (7): 0e3d8eba9432a18d F!A13, F!A20; 20512106c9574cdd F!K4;
  4c6a49fe4a6fa404 F!F2, F!F12, F!F18; 659cd5776d423ce3 F!V11
- `_("$"* #,##0_);_("$"* \(#,##0\);_("$"* "-"_);_(@_)` (6): 0e3d8eba9432a18d F!I13, F!I20; 20512106c9574cdd F!S4;
  4c6a49fe4a6fa404 F!N2, F!N12, F!N18
- `_(* #,##0.00_);_(* \(#,##0.00\);_(* "-"??_);_(@_)` (6): 0e3d8eba9432a18d F!J13, F!J20; 20512106c9574cdd F!T4;
  4c6a49fe4a6fa404 F!O2, F!O12, F!O18
number-formats.md "Displayed text" records the rule (gap triage 2026-10-25).

## Acceptance criteria
- [ ] The gen-m10 text cells under these codes pass in the report's "Displayed text" section (or the remaining ones are filed); the matching TEXT() cells on sheet T leave eval/baseline/gen-m10.txt; no regressions.
- [ ] The 27 accounting-format cells listed under Context (2026-10-25) pass in the "Displayed text" section too.
- [ ] number-formats.md updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
