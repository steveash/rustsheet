---
id: T-2333
title: "Probe a LET that may pick an external reference when it picks the other argument (ISREF, ROWS, ROW, COUNTA, OFFSET)"
milestone: M8
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel evidence for what a LET whose body may pick a reference into another workbook gives when it
picks the other argument, so the engine's reading of it rests on oracle cells.

## Context
T-1721 made `Model::pick_reference` (crates/rustsheet-engine/src/external_reference.rs) read such a
LET with its names bound and, when the body picks the other argument, read that argument there in
reference context (`PickedReference::Read`). docs/excel-semantics/references.md "External reference
with no cached data is `#BLOCKED!`" marks that case unprobed. The unit test
`let_body_picking_an_external_reference_or_another_argument`
(crates/rustsheet-engine/src/test/test_external_links.rs) states ours, with D1 1, D2 "x":
`ISREF(LET(c,FALSE,IF(c,…!A1,D1)))` TRUE, `ISREF(LET(c,FALSE,IF(c,…!A1,1)))` FALSE,
`ROWS(LET(r,D1:D2,c,FALSE,IF(c,…!A1:B3,r)))` 2, `COUNTA(LET(r,D1:D3,c,FALSE,IF(c,…!A1:B2,r)))` 2,
`ROW(LET(r,D2,c,FALSE,IF(c,…!B3,r)))` 2, `ROW(LET(c,FALSE,IF(c,…!B3,1)))` `#VALUE!`,
`AREAS(LET(c,FALSE,IF(c,…!A1,(D1,E1))))` 2, `ROWS(…(D1,E1)…)` `#REF!`,
`INDEX(LET(r,D1:D2,c,FALSE,IF(c,…!A1,r)),2)` "x", `COUNTBLANK(LET(r,D1:D3,…))` 1,
`ISREF(OFFSET(LET(c,FALSE,IF(c,…!A1,D1)),1,1))` TRUE, `OFFSET(LET(c,FALSE,IF(c,…!A1,D1)),1,0)` "x",
and `OFFSET(LET(c,TRUE,…!A1),1/0,1)` `#DIV/0!`. `…` is `'[other.xlsx]Sheet1'!` with no externalLink
part, as probes/external-ref-no-link-let-index.

## Acceptance criteria
- [ ] A probes case (excel-probe skill, excel-graph) holds the cells above; any cell ours misses is
  a known failure with a follow-up task, and references.md states the rule with cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
