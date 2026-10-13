---
id: T-1485
title: "IF and CHOOSE picking a reference into another workbook keep it a reference"
milestone: M1
lane: engine
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-1454]
---

## Goal
An IF or CHOOSE that picks a reference into another workbook (no cached data) is that reference:
reference functions read its shape and its single cell reads as empty, as Excel does.

## Context
Known failures in probes/external-ref-no-link-index (excel-graph):
- A25 `=ROWS(IF(TRUE,'[other.xlsx]Sheet1'!A1:B3))` → 3 (we give `#BLOCKED!`).
- A26 `=ERROR.TYPE(IF(TRUE,'[other.xlsx]Sheet1'!A1))` → `#N/A`: the cell IF picks reads empty,
  where the written `…!A1` is `#BLOCKED!` (ERROR.TYPE 11).
T-1454 added `Model::external_reference` (reference_operators.rs), which recognizes a written
external reference, INDEX into one, and unions/intersections of them *without evaluating anything*
when it returns `None`. IF and CHOOSE need their condition/index evaluated to know the branch, so
they cannot join that function as is: the caller must not evaluate the IF twice (nested IFs must
stay linear). Likely shape: have IF/CHOOSE's reference-mode result carry the external areas (e.g.
from `if_impl` in `IfBranchMode::Aggregate`), or let `external_reference` return the evaluated
non-external result for the caller to use. Probe CHOOSE first (excel-probe): `ROWS(CHOOSE(1,…))`,
`ERROR.TYPE(CHOOSE(1,…!A1))`, `SUM(IF(TRUE,…!A1))`, `COUNTA(IF(TRUE,…!A1:B2))`.
Rule: docs/excel-semantics/references.md "External reference with no cached data is `#BLOCKED!`".

## Acceptance criteria
- [ ] probes/external-ref-no-link-index A25 and A26 leave eval/baseline/probes.txt.
- [ ] The CHOOSE probe values are recorded in docs/excel-semantics/references.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Cached values of `[n]Sheet!ref` (T-1382). Opening other workbooks (directive 5).
