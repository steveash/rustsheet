---
id: T-1654
title: "3-D spans of a cached external workbook ([1]Uses:Defines!) parse and read the cache"
milestone: M8
lane: engine
status: todo
priority: P2
size: S
tier: opus
campaign: visible
depends_on: []
---

## Goal
A 3-D span into a cached external workbook (`[1]Uses:Defines!$B$2`) gives Excel's values instead of `#ERROR!`.

## Context
Found by T-1609: fixtures/external-link-cached-3d-span Refs!B2:B10 (excel-graph) are `#ERROR!` in ours: the lexer/parser does not take a sheet span after a `[n]` book prefix. Excel aggregates over the cached sheets in the span (SUM 142 and 157, COUNT 4, COUNTA 9, MAX 142, the span reversed too), but ISREF is FALSE, ERROR.TYPE of the bare span is 4 (`#REF!`) and ROWS is `#VALUE!`. docs/excel-semantics/references.md "Cached values of a linked workbook". Code: crates/rustsheet-engine/src/expressions/lexer, parser, external_links.rs.

Visibly-off cells (gap triage 2026-10-21, decision 0083): 4 cells, 0 numeric past half a unit in the 15th digit and 4 with a wrong type, error or text, blamed on COUNT 1, COUNTA 1, ISREF 1, ROWS 1.

| case / cell | formula | Excel | ours |
|---|---|---|---|
| fixtures/external-link-cached-3d-span.xlsx Refs!B10 | `=ROWS([1]Uses:Defines!$A$1:$B$3)` | #VALUE! | #ERROR! |
| fixtures/external-link-cached-3d-span.xlsx Refs!B4 | `=COUNT([1]Uses:Defines!$A$1:$C$3)` | 4 | #ERROR! |
| fixtures/external-link-cached-3d-span.xlsx Refs!B5 | `=COUNTA([1]Uses:Defines!$A$1:$C$3)` | 9 | #ERROR! |
| fixtures/external-link-cached-3d-span.xlsx Refs!B7 | `=ISREF([1]Uses:Defines!$A$1)` | FALSE | #ERROR! |

## Acceptance criteria
- [ ] Visible parity: the 4 visibly-off cells listed under Context (2026-10-21) have Excel's value to 15 significant digits, or its type, error and text: the 4 lines in eval/baseline/fixtures.txt disappear (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] fixtures/external-link-cached-3d-span B2:B10 pass; baseline updated with `make eval-baseline`.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Opening or updating the linked workbook; 3-D spans of this workbook's own sheets.

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off (`#ERROR!` where Excel reads a cached external 3-D span), so this task is in the campaign. Its target is that every cell it owns is within half a unit of the 15th significant digit of Excel's value (an expected 0 exactly 0), or has Excel's type, error and text; the visible-parity criterion under Acceptance criteria states it.
