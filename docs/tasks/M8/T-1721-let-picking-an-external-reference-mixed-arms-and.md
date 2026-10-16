---
id: T-1721
title: "LET picking an external reference: mixed arms, and OFFSET over it, as Excel (T-1710 probe)"
milestone: M8
lane: engine
status: todo
priority: P3
size: M
tier: opus
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

## Acceptance criteria
- [ ] The five failing cells above pass and leave eval/baseline/probes.txt smaller; A1 and A2 keep passing.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
