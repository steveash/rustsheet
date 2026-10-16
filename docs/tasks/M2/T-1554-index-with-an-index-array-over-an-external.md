---
id: T-1554
title: "INDEX with an index array over an external range is a reference to its first element (SUM, ROWS, ISREF, nested INDEX)"
milestone: M2
lane: engine
status: done
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
- [x] A probe `SUM(INDEX([1]Uses!$A$1:$B$3,{2,1},1))` (5 if the first element, 0 if the top-left cell) decides which cell the single-cell reference is.
- [x] Refs!B89:B92 of fixtures/external-link-cached-ranges pass and leave the baseline as fixed cells.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probe fixture `fixtures/external-link-cached-index-array.xlsx` (11 cells, excel-graph). In a legacy formula the
single-cell reference is the index array's first element, not the range's top-left cell:
`SUM(INDEX([1]Uses!$A$1:$B$3,{2,1},1))` is 5. Refs!B89:B92 of external-link-cached-ranges already passed on main
(the legacy formula reads the index array's first element), and the engine matches all 11 new cells too, so no
engine change was needed. references.md is corrected (its "Not implemented" note was stale) and the fixture
README updated. Eval: fixtures +1 case (11/11), 0 regressions, 0 known failures added.
