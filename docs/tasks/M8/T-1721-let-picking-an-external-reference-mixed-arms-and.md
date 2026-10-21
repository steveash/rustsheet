---
id: T-1721
title: "LET picking an external reference: mixed arms, and OFFSET over it, as Excel (T-1710 probe)"
milestone: M8
lane: engine
status: in-progress
priority: P3
size: M
tier: opus
campaign: visible
depends_on: []
---

## Goal
Ours matches Excel for a LET whose body may pick a reference into another workbook alongside a non-external argument, and for OFFSET over a LET that picks an external reference.

## Context
Found by the T-1710 probe (probes/external-ref-no-link-let-index, excel-graph 2026-10-16), five cells fail:
- A3 `ISREF(OFFSET(LET(c,TRUE,…!A1),1,1))` is FALSE in Excel, ours TRUE.
- A4 `ROWS(LET(c,TRUE,IF(c,…!A1:B3,"x")))` and A5 `ROWS(LET(c,TRUE,IF(c,…!A1:B3,D1:D2)))` are 3, ours `#BLOCKED!`.
- A6 `ISREF(LET(c,TRUE,IF(c,…!A1,D1)))` is TRUE, ours FALSE.
- A7 `COUNTA(LET(c,TRUE,IF(c,…!A1:B2,1)))` is 0, ours 1.
Code: `let_picks_external` / `picks_only_external` and `Model::pick_reference` in crates/rustsheet-engine/src/external_reference.rs leave a LET whose body may pick another arm on the old path. Widening it needs the LET read with names bound while still returning a reference into this workbook when the other arm is picked (`PickedReference::Node` borrows from the node, outside the `let_scope` closure), hence tier opus. Spec: docs/excel-semantics/references.md "External reference with no cached data is `#BLOCKED!`".

Visibly-off cells (gap triage 2026-10-21, decision 0083): 5 cells, 1 numeric past half a unit in the 15th digit and 4 with a wrong type, error or text, blamed on ISREF 2, ROWS 2, OFFSET 1, COUNTA 1. Every one is listed above.

## Acceptance criteria
- [ ] Visible parity: the 5 visibly-off cells listed under Context (2026-10-21) have Excel's value to 15 significant digits, or its type, error and text: the 5 lines in eval/baseline/probes.txt disappear (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] The five failing cells above pass and leave eval/baseline/probes.txt smaller; A1 and A2 keep passing.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off (wrong ISREF, ROWS and COUNTA of a LET that picks an external reference), so this task is in the campaign. Its target is that every cell it owns is within half a unit of the 15th significant digit of Excel's value (an expected 0 exactly 0), or has Excel's type, error and text; the visible-parity criterion under Acceptance criteria states it.
