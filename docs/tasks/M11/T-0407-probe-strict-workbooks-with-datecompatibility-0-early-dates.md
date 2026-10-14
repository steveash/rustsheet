---
id: T-0407
title: "Probe Strict workbooks with dateCompatibility=0: early dates and t=d serials"
milestone: M11
lane: research
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Know what Excel computes for dates in a workbook whose `<workbookPr dateCompatibility="0">` (every
Excel-saved Strict OOXML file has it), and record it in docs/excel-semantics/xlsx-storage.md.

## Context
Found by T-0112: poi/SimpleStrict.xlsx, SampleSS.strict.xlsx, sample.strict.xlsx and 57914.xlsx
all declare `dateCompatibility="0"`; nothing in crates/ reads the attribute, so they import as
1900-compatible workbooks. ECMA-376 ties the attribute to the 1900 date base compatibility (the
serial-60 leap-year bug and dates before 1900-03-01). T-0406 (`t="d"` cells) depends on the answer.

## Acceptance criteria
- [ ] A fixture workbook (gated `fixtures` suite) with `dateCompatibility="0"` holds `=DATE(1900,2,28)`,
  `=DATE(1900,3,1)`, `=DATE(1900,2,29)`, `=TEXT(60,"yyyy-mm-dd")`, `=DAY(59)`, `=DATEVALUE("1900-01-01")`
  and a `t="d"` cell `1900-01-01` read by `=A1`; oracle-filled with excel-graph.
- [ ] The Strict entry in docs/excel-semantics/xlsx-storage.md records the finding (VERIFIED or
  CONFLICTING), and a follow-up implementation task is filed if Excel differs from today's engine.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Implementing the behavior (follow-up task).
