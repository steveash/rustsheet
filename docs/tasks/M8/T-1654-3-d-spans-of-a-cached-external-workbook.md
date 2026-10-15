---
id: T-1654
title: "3-D spans of a cached external workbook ([1]Uses:Defines!) parse and read the cache"
milestone: M8
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
A 3-D span into a cached external workbook (`[1]Uses:Defines!$B$2`) gives Excel's values instead of `#ERROR!`.

## Context
Found by T-1609: fixtures/external-link-cached-3d-span Refs!B2:B10 (excel-graph) are `#ERROR!` in ours: the lexer/parser does not take a sheet span after a `[n]` book prefix. Excel aggregates over the cached sheets in the span (SUM 142 and 157, COUNT 4, COUNTA 9, MAX 142, the span reversed too), but ISREF is FALSE, ERROR.TYPE of the bare span is 4 (`#REF!`) and ROWS is `#VALUE!`. docs/excel-semantics/references.md "Cached values of a linked workbook". Code: crates/rustsheet-engine/src/expressions/lexer, parser, external_links.rs.

## Acceptance criteria
- [ ] fixtures/external-link-cached-3d-span B2:B10 pass; baseline updated with `make eval-baseline`.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Opening or updating the linked workbook; 3-D spans of this workbook's own sheets.
