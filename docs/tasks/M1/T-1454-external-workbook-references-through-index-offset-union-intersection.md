---
id: T-1454
title: "External-workbook references through INDEX, OFFSET, union, intersection, COUNTA and spill"
milestone: M1
lane: engine
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-1381]
---

## Goal
A reference into another workbook with no cached data keeps behaving as a reference through
INDEX, OFFSET, the union and the intersection operators, and COUNTA and a spilled range see it as
Excel does.

## Context
T-1381 made the written external reference a reference for ISREF, ROWS, COLUMNS, ROW, COLUMN and
AREAS through `reference_operators::external_area` (the parser still builds `WrongReferenceKind` /
`WrongRangeKind`). Known failures in probes/external-ref-no-link-shapes (excel-graph):
- A3 `=ROWS(INDEX('[other.xlsx]Sheet1'!A1:C4,2,0))` → 1; A4 `ISREF(INDEX(…,2,2))` → TRUE;
  A18 `ERROR.TYPE(INDEX(…,2,2))` → `#N/A` (the picked cell is not an error value, unlike the
  written `'[other.xlsx]Sheet1'!A1`, whose ERROR.TYPE is 11).
- A5 `=ROWS(OFFSET('[other.xlsx]Sheet1'!A1,1,1,4,2))` → `#VALUE!`.
- A15 `=AREAS((ext!A1,ext!B2))` → 2; A17 `=ROWS(ext!A1:B3 ext!A2:B5)` → 2.
- A16 `=COUNTA('[other.xlsx]Sheet1'!A1:B2)` → 0 (we count the one `#BLOCKED!`).
- A19 `='[other.xlsx]Sheet1'!A1:B2` → `#SPILL!` (the 2x2 spill hits A20; we give `#BLOCKED!`).
Rule: docs/excel-semantics/references.md "External reference with no cached data is `#BLOCKED!`".
Likely shape: give external areas a place in `Ranges`/`RefArg` (e.g. an area that is not on a sheet
of this workbook) rather than more special cases per function; probe what INDEX's picked cell
reads (ISBLANK, ISTEXT, `=INDEX(…)`, `&""`) first (excel-probe).

## Acceptance criteria
- [ ] probes/external-ref-no-link-shapes A3, A4, A5, A15:A19 leave eval/baseline/probes.txt.
- [ ] The probed value of INDEX's picked cell is recorded in docs/excel-semantics/references.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Cached values of `[n]Sheet!ref` (T-1382). Opening other workbooks (directive 5).
