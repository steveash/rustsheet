---
id: T-1711
title: "Displayed text of fractions, round-half #,##0 and negative dates matches Excel"
milestone: M10
lane: engine
status: todo
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
`Workbook::formatted` (and TEXT where it shares the formatter) gives Excel's displayed text for the three cells probes/displayed-text-formats fails.

## Context
T-1610's first displayed-text probe (probes/displayed-text-formats, excel-graph `Range.text`) fails three cells: `# ?/?` on 12.5 is "12 1/2" in Excel, " 1/2" in ours (S!A17); `TEXT(1234.5,"#,##0")` is "1,235", ours "1,234" (S!A21, a value failure too: half rounds away from zero); a negative serial under `yyyy-mm-dd` displays as 255 `#` characters, ours `#VALUE!` (S!B4). excel-graph's text does not depend on the column width (B1:B3), so width is not needed. Code: crates/rustsheet-engine/src/formatter/format.rs. Rule: docs/excel-semantics/number-formats.md "Displayed text (`Range.text`) of a formatted cell".

## Acceptance criteria
- [ ] probes/displayed-text-formats S!A21 leaves eval/baseline/probes.txt, and its text section shows A17, A21 and B4 passing (`make eval`).
- [ ] number-formats.md updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
