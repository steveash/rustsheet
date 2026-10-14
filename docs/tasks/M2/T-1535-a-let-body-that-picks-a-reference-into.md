---
id: T-1535
title: "A LET body that picks a reference into another workbook keeps it a reference"
milestone: M2
lane: engine
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-1487]
---

## Goal
`ROWS(LET(c,TRUE,IF(c,'[other.xlsx]Sheet1'!A1:B3)))` is 3, as Excel gives: a LET whose body is an
IF or CHOOSE picking a reference into another workbook stands for that reference.

## Context
Known failure probes/external-ref-no-link-ifchoose Calc!A26 (excel-graph; we give `#BLOCKED!`).
T-1487 added `Model::pick_branch` (external_reference.rs), which reads the condition of an IF or
CHOOSE that may pick an external reference once and hands the caller the picked argument. A LET
body cannot be handed back that way: its variables are bound only while `let_impl` runs. Likely
shape: in `let_impl`'s reference mode, read the body's external areas (`external_reference` after
`pick_branch`) while the bindings are live, and give the caller what it needs (a shape for ROWS,
COLUMNS, ROW, COLUMN, ISREF, AREAS, COUNTA). Probe the other readers first (excel-probe):
`ISREF(LET(c,TRUE,IF(c,…!A1)))`, `COUNTA(LET(c,TRUE,IF(c,…!A1:B2)))`, `ERROR.TYPE(LET(c,TRUE,IF(c,…!A1)))`.
Rule: docs/excel-semantics/references.md "External reference with no cached data is `#BLOCKED!`".

## Acceptance criteria
- [ ] probes/external-ref-no-link-ifchoose A26 leaves eval/baseline/probes.txt.
- [ ] The new probe values are recorded in docs/excel-semantics/references.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Opening other workbooks (directive 5).
