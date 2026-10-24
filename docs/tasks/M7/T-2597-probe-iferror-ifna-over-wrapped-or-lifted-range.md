---
id: T-2597
title: "Probe IFERROR/IFNA over wrapped or lifted range forms: LET, IF, TAKE, DROP and lifted TRIMRANGE with blank cells"
milestone: M7
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel-graph evidence for whether IFERROR and IFNA read blank cells as 0 when the range comes through LET, IF, TAKE, DROP, a lifted TRIMRANGE, or IFNA over TRIMRANGE.

## Context
T-2579 (probes/t2579-array-value-edges B70-B88, excel-graph) showed IFERROR/IFNA over a multi-cell range, and over a
direct TRIMRANGE of one, read each blank cell as 0 (a one-cell range keeps the blank's empty value). Ours applies that only
to a direct range or TRIMRANGE argument (`if_error_like`, functions/logical/mod.rs); wrapped forms keep blanks empty.
Probe over the same D sheet (1, blank, TRUE, "x", 5, 1/0): COUNT and `INDEX(..,2)&"|"` of `IFERROR(LET(x,D!A1:A6,x),-1)`,
`IFERROR(IF(TRUE,D!A1:A6),-1)`, `IFERROR(TAKE(D!A1:A6,5),-1)`, `IFERROR(DROP(D!A1:A6,1),-1)`,
`IFERROR(TRIMRANGE(D!A1:A6,{0;1}),-1)` (lifted) and `IFNA(TRIMRANGE(D!A1:B6),-1)`. docs/excel-semantics/arrays.md
"Array-value edges in the gen-m9 sweep (T-2436)".

## Acceptance criteria
- [ ] A probe case with oracle values for each form above; arrays.md records the answers.
- [ ] Any form ours gets wrong is filed as an engine task with the probe cells (or fixed here if size S).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other functions over these forms.
