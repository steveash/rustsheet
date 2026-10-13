---
id: T-1382
title: "[n]Sheet!ref external references read the externalLink sheetDataSet cache"
milestone: M1
lane: engine
status: in-progress
priority: P3
size: M
tier: opus
depends_on: [T-1381]
---

## Goal
A formula written with Excel's external index form (`[1]Sheet!$A$1`, `[1]Sheet!Name`, `[1]!Name`)
evaluates to the value cached in the workbook's `xl/externalLinks/externalLinkN.xml`
(`externalBook/sheetDataSet`, `definedNames`), as Excel does without opening the other file.

## Context
Excel-cached evidence (desktop-written workbooks, poi suite): poi/ref-56737.xlsx and
poi/ref2-56737.xlsx Refs!E3 `=[1]Uses!$A$1` → "Hello!", G3 `=[1]Defines!NR_To_A1` → "Test A1",
I3 `=[1]!NR_Global_B2` → 142. Today all give `#ERROR!` (the `[1]` prefix does not parse).
Add-in calls through an external name (`=[1]!HsGetValue(...)` in poi/HsGetVal.xlsx,
`=[1]!LUCANET(...)` in poi/56502.xlsx) keep their cached results in the cell, not in the link
part; decide from evidence what they give (likely the uncached rule, docs/excel-semantics/
references.md) and record it. Needs the xlsx importer to keep externalLink caches in the model
(rustsheet-xlsx) and the evaluator to read them (no I/O, directive 5). Size it against the
architecture docs; split if it is bigger than M.

## Acceptance criteria
- [ ] poi/ref-56737 and poi/ref2-56737 Refs!E3, G3, I3 leave eval/baseline/poi.txt with no new F lines.
- [ ] The rule (cached `[n]` reference) is in docs/excel-semantics/references.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Refreshing links or reading the other workbook (directive 5).
