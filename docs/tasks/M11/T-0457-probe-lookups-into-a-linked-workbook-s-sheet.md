---
id: T-0457
title: "Probe: lookups into a linked workbook's sheet with no cached data"
milestone: M11
lane: functions-lookup
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Oracle evidence for what Excel gives when VLOOKUP, HLOOKUP, MATCH and XLOOKUP read a
whole-column or large range of a linked workbook's sheet with no cached data, and rustsheet
matching it.

## Context
corpus-bench 59932 has about 18,000 cells such as `VLOOKUP(B2,[1]Sort!$B:$Z,2,FALSE)` and
`VLOOKUP($B12,'[1]<2010Price'!$A:$BE,13,FALSE)`. Their linked sheets are listed in
xl/externalLinks/externalLink1.xml but have no `sheetData`. rustsheet reads such an area as
`#BLOCKED!` cells (docs/excel-semantics/references.md, "External reference with no cached data
is `#BLOCKED!`"). A whole-column area is 1,048,576 x 25 cells, past the array bound, so
`ExternalArea::blocked_value` gives `#ERROR!`. The file's cached values came from the linked
workbook when it was open, so no engine can reproduce them. What matters is what Excel gives
offline.

Existing evidence: fixtures/external-link-cached-lookups (sheets with cached data) and
probes/external-ref-no-link* (no link part at all).

## Acceptance criteria
- [x] A fixture whose link part lists a sheet without `sheetData`, recalculated by the oracle:
      VLOOKUP, HLOOKUP, MATCH and XLOOKUP over one cell, a small range and a whole column of
      that sheet, found and not found.
- [x] The quirk is recorded in docs/excel-semantics/references.md; rustsheet matches it, or a
      DEVIATION entry explains why it cannot (directive 5). Except 7 cells, tracked in T-0472 (B13, B38, B44) and T-0474 (B17, B21, B49, B50).
- [x] `make check` green; reviewers approve.

## Out of scope
- Sheets whose cache holds data (already covered).

## Outcome
New fixture `fixtures/external-link-uncached-lookups.xlsx` (51 legacy formulas, excel-graph filled, 0 failures):
`external-link-cached-lookups.xlsx` with two sheets added to the link's `sheetNames`, `Sort` (no `sheetData`) and
`Bare` (empty `sheetData`), plus the existing `refreshError` sheet. Excel's answer: lookups (VLOOKUP, HLOOKUP, MATCH,
XLOOKUP) over such a sheet are `#N/A` over one cell, a small range and whole columns or rows, before the index check
(`VLOOKUP(5,[1]Sort!$B:$Z,30,FALSE)` is `#N/A`); direct reads are `#BLOCKED!` (refreshError: `#REF!`), and
`COUNTA([1]Sort!$B:$B)` is 1048576. Recorded in references.md "Lookups into a linked sheet with no cached data are
`#N/A`".

Engine: a linked sheet the cache holds no cell of is now uncached (`#BLOCKED!` on direct reads), and
`Model::external_lookup_table` gives MATCH/VLOOKUP/HLOOKUP tables `#N/A` there. A first cut also clipped whole
columns of a cached sheet to the cache's extent; the Excel-fidelity review showed that moves approximate-search
midpoints, so it was dropped and filed as **T-0474**.

Eval delta: fixtures 23 -> 24 cases, 44/51 new cells pass, 7 new-coverage known failures: B13 one-cell XLOOKUP,
B38 refreshError `#REF!`, B44 COUNTA (**T-0472**) and B17, B21, B49, B50 whole columns of a cached sheet
(**T-0474**). The cache-scan cost reviewers noted is **T-0473**. 0 regressions in every suite (poi and
corpus-bench fetched; calipers not fetched).
