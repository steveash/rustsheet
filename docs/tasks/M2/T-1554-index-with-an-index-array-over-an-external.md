---
id: T-1554
title: "INDEX with an index array over an external range is a reference to its first element (SUM, ROWS, ISREF, nested INDEX)"
milestone: M2
lane: engine
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
`SUM`, `ROWS`, `ISREF` and a nested `INDEX` over `INDEX([1]Uses!$A$1:$B$3,{1,2},1)` give what Excel gives (0, 1, TRUE, `#REF!`) instead of `#BLOCKED!`/FALSE.

## Context
Found by T-1546: fixtures/external-link-cached-ranges Refs!B89:B92 (excel-graph) show that Excel treats an INDEX with an
index array over an external range as a reference to its first element only (docs/excel-semantics/references.md,
"INDEX with an index array into an external reference"). The engine lifts it per element in `index_lifted` via
`external_index_args` and gives `#BLOCKED!` to the consumers of it as a reference.

## Acceptance criteria
- [ ] A probe `SUM(INDEX([1]Uses!$A$1:$B$3,{2,1},1))` (5 if the first element, 0 if the top-left cell) decides which cell the single-cell reference is.
- [ ] Refs!B89:B92 of fixtures/external-link-cached-ranges pass and leave the baseline as fixed cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
