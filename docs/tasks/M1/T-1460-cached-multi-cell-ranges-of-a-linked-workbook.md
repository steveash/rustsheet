---
id: T-1460
title: "Cached multi-cell ranges of a linked workbook: [n]Sheet!A1:B2"
milestone: M1
lane: engine
status: done
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
- [x] Evidence for SUM/COUNT/COUNTA over a cached external range with text and booleans in it.
- [x] A one-cell cached reference inside an aggregate reads as a reference cell, not a value
      (`SUM([1]Uses!A1)` over cached text; T-1382 passes it as a value).
- [x] Engine matches it; unit tests cover a cached range and a cached range name.
- [x] references.md updated.
- [x] `make check` green; reviewers approve.

## Out of scope
- Refreshing links or opening the other workbook (directive 5).

## Outcome
Evidence: new fixture fixtures/external-link-cached-ranges.xlsx (82 legacy formulas over a hand-made
externalLink cache, excel-graph). A cached range or workbook range name is a reference whose cells
hold the cached values: aggregates skip text, booleans and blanks (a one-cell `SUM([1]Uses!A1)` over
text is now 0), COUNTA counts cached cells, lookups/INDEX/IS* read the values, and COUNTIF, SUMIF,
SUMIFS and COUNTBLANK are `#VALUE!` (so `COUNTBLANK` of a cached cell changed from 1 to `#VALUE!`).
Engine: `ExternalArea` carries its cache (`CachedSheet`); `external_links.rs` reads values for value
contexts, INDEX, aggregates and RANK; only a whole column or row is cut to the cached extent
(engine bound), and a union's cells are capped together. SUBTOTAL returns one `#VALUE!` for now. references.md and the fixtures README record the findings.
Eval delta: new case 75/82 passing; 7 new-coverage known failures (TYPE, SUBTOTAL, AGGREGATE,
AVERAGEA, legacy implicit intersection) filed as T-1526; 0 regressions elsewhere. Follow-up
T-0423 (per-sheet cache index, Phase 2).
