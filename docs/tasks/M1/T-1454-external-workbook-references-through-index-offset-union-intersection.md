---
id: T-1454
title: "External-workbook references through INDEX, OFFSET, union, intersection, COUNTA and spill"
milestone: M1
lane: engine
status: done
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
- [x] probes/external-ref-no-link-shapes A3, A4, A5, A15:A19 leave eval/baseline/probes.txt.
- [x] The probed value of INDEX's picked cell is recorded in docs/excel-semantics/references.md.
- [x] `make check` green; reviewers approve.

## Out of scope
- Cached values of `[n]Sheet!ref` (T-1382). Opening other workbooks (directive 5).

## Handoff (resumed)
- Claimed 2026-10-13 by an opus run near the end of its session budget. Released untouched: no code
  and no probe were written, and the branch holds only the claim and this note.
- Reading so far: `reference_operators::external_area` gives the shape of a written external
  reference. `Ranges`/`RefArg` hold only on-sheet `Range`s, so INDEX, OFFSET, the union and
  intersection operators, COUNTA and the spill path never see an external area.
- The cells imply this: an external area's cells, read through INDEX or a range, are not
  `#BLOCKED!` (ERROR.TYPE `#N/A`, COUNTA 0). Only the written single cell is `#BLOCKED!`. Probe what
  INDEX's picked cell reads before designing (ISBLANK, ISTEXT, `=INDEX(..)`, `&""`).
- Do not represent "another workbook" with a sentinel sheet index. Code that indexes worksheets by
  `sheet` would then panic or misread it (directive 5). Add an explicit variant to `Ranges`/`RefArg`
  instead, as the task suggests.

## Outcome
- Probed what the reached cells read (probes/external-ref-no-link-index, 47 cells, excel-graph):
  one cell INDEX, an intersection or IF picks reads **empty**; several spill `#BLOCKED!`; aggregates
  are `#BLOCKED!` but COUNTA counts nothing, COUNTBLANK and OFFSET are `#VALUE!`; a union or
  intersection across two workbooks is `#VALUE!`. Recorded in docs/excel-semantics/references.md.
- `Model::external_reference` (external_reference.rs) gives the areas of a written external
  reference, INDEX into one, and unions/intersections of them, evaluating nothing when the
  expression is none of these; ISREF, ROWS/COLUMNS, ROW/COLUMN, AREAS, COUNTA, COUNTBLANK, OFFSET,
  INDEX, the intersection and `evaluate_reference_arg` read it instead of per-node special cases.
  INDEX's window geometry is shared by both paths (`index_window`). A written external range spills.
- Eval delta: probes known failures 2826 -> 2820 (8 fixed: external-ref-no-link-shapes A3, A4, A5,
  A15:A19; the new case adds 2 known failures, A25 and A26, IF picking an external reference).
- Follow-up: T-1485 (IF and CHOOSE picking an external reference).

