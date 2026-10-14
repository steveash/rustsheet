---
id: T-1537
title: "INDEX with an index array into another workbook reads the picked cells as empty"
milestone: M2
lane: engine
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: [T-1487]
---

## Goal
`=INDEX('[other.xlsx]Sheet1'!A1:C4,{1,2},1)` spills 0, 0 as Excel does: each cell a lifted INDEX
picks from a reference into another workbook reads empty, as the cell a scalar INDEX picks does.

## Context
Known failures probes/external-ref-no-link-ifchoose Calc!H1, I1 (excel-graph; we give `#BLOCKED!`).
`Model::external_index` (functions/lookup_and_reference/index.rs) returns `#BLOCKED!` when the
indices lift (`IndexArgs::Lifted`); `index_impl` reaches it first. Read each element with
`external_index_area` and `Model::external_picked_value` (empty with no cache, the cached cell with
one), one element per broadcast index, as `index_lifted` does for a sheet reference.
Rule: docs/excel-semantics/references.md "External reference with no cached data is `#BLOCKED!`".

## Acceptance criteria
- [ ] probes/external-ref-no-link-ifchoose H1 and I1 leave eval/baseline/probes.txt.
- [ ] A unit test covers a lifted INDEX into a cached external range.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Opening other workbooks (directive 5).
