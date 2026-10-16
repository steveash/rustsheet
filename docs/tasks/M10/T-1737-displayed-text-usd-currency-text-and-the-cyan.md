---
id: T-1737
title: "Displayed text: [$USD] currency text and the [Cyan] color are accepted, not #VALUE!"
milestone: M10
lane: functions-text
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Codes Excel accepts, `[$USD]` currency text and the `[Cyan]` color, format instead of giving `#VALUE!`.

## Context
`#,##0.00 [$USD]`: -1.5 → "-1.50 USD", 1234567890 → "1,234,567,890.00 USD" (ours `#VALUE!`, 75/80 fail);
`[Green]#,##0;[Magenta](#,##0);[Cyan]0`: 0 → "0" (ours `#VALUE!`; Green and Magenta work). Excel's eight color
names are Black, Blue, Cyan, Green, Magenta, Red, White, Yellow, plus `[ColorN]`. T-1716 is about locale tags
that localize month names; a bracket with only currency text is a literal.
Evidence: suite gen-m10 (excel-graph, 2026-10-16, T-1611): `make eval` report, "Displayed text" section, which lists each failing text cell (Excel vs ours). Text cells are scored apart from values and are not baselined (decision 0068); the TEXT() cells on sheet T that use the same codes are value known failures in eval/baseline/gen-m10.txt. Formatter: crates/rustsheet-engine/src/formatter/ (shared by `Workbook::formatted` and TEXT). docs/excel-semantics/number-formats.md.

## Acceptance criteria
- [ ] The gen-m10 text cells under these codes pass in the report's "Displayed text" section (or the remaining ones are filed); the matching TEXT() cells on sheet T leave eval/baseline/gen-m10.txt; no regressions.
- [ ] number-formats.md updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
