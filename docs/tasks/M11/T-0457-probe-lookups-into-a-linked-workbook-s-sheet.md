---
id: T-0457
title: "Probe: lookups into a linked workbook's sheet with no cached data"
milestone: M11
lane: functions-lookup
status: in-progress
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
- [ ] A fixture whose link part lists a sheet without `sheetData`, recalculated by the oracle:
      VLOOKUP, HLOOKUP, MATCH and XLOOKUP over one cell, a small range and a whole column of
      that sheet, found and not found.
- [ ] The quirk is recorded in docs/excel-semantics/references.md; rustsheet matches it, or a
      DEVIATION entry explains why it cannot (directive 5).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Sheets whose cache holds data (already covered).
