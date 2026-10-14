---
id: T-1460
title: "Cached multi-cell ranges of a linked workbook: [n]Sheet!A1:B2"
milestone: M1
lane: engine
status: in-progress
priority: P3
size: M
tier: opus
depends_on: [T-1454]
---

## Goal
A multi-cell reference into a linked workbook whose values Excel cached, `[1]Sheet!A1:B2` (and a
cached name for a range), gives those cached values where Excel uses the cells: aggregates
(`SUM`, `COUNT`, `COUNTA`), a spill, lookups.

## Context
Found in T-1382, which reads one-cell `[n]Sheet!A1` and one-cell `[n]!Name` from
`Workbook::external_books` (`crates/rustsheet-engine/src/external_links.rs`). A range of the same
book stays `#BLOCKED!` because a `CalcResult::Range` can only point at this workbook's sheets.
Decide the representation (an array of the cached values, or a reference kind that reads the
cache) against how aggregates treat text and booleans in a reference compared with an array, and
probe what Excel gives: the oracle needs an Excel-saved workbook with an externalLink cache (the
spec2xlsx builder writes none), so this may need a desktop-written fixture. T-1454 handles the
uncached shapes (INDEX, OFFSET, unions, intersections, COUNTA, spill) and should land first.
docs/excel-semantics/references.md "Cached values of a linked workbook" lists the gap.

## Acceptance criteria
- [ ] Evidence for SUM/COUNT/COUNTA over a cached external range with text and booleans in it.
- [ ] A one-cell cached reference inside an aggregate reads as a reference cell, not a value
      (`SUM([1]Uses!A1)` over cached text; T-1382 passes it as a value).
- [ ] Engine matches it; unit tests cover a cached range and a cached range name.
- [ ] references.md updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Refreshing links or opening the other workbook (directive 5).
