---
id: T-1407
title: "INDEX truncates a tiny negative row or column index (-1E-15) to 0 instead of #VALUE!"
milestone: M2
lane: functions-lookup
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
INDEX truncates a fractional negative row or column index above -1 (such as -1E-15) toward zero, so it selects the whole
row or column as index 0 does, instead of returning `#VALUE!`, as Excel does.

## Context
T-1289's gen-m9 suite (excel-graph sidecars, 2026-10-12; `./target/release/rustsheet-eval show gen-m9/<case>.case.json`):
- gen-m9/1466c66c61189a83 Calc!A1 `=EXPONDIST(INDEX(Data!A1:C3,3,-1e-15),SUM(..),((1="Hello World")<Data!D9))`: Excel
  0.027981694209681673, which is `EXPONDIST(0.5,12.14649,FALSE)`: x = A3 = 0.5, the first cell of row 3 (A3:C3), so
  `INDEX(Data!A1:C3,3,-1e-15)` is the whole row 3 and EXPONDIST lifts over it (the anchor of a spill). Engine `#VALUE!`
  (`INDEX(Data!A1:C3,3,-1e-15)` and `INDEX(..,3,-0.5)` are `#VALUE!`; `INDEX(..,3,0)` works).
The EXPONDIST lift over the resulting range is T-1290: this cell passes only when both land.
Semantics: docs/excel-semantics/lookup.md "INDEX truncates a tiny negative index to 0" (UNVERIFIED beyond -1E-15).
Probe with the excel-probe skill `INDEX(A1:C3,3,-1E-15)`, `INDEX(A1:C3,3,-0.5)`, `INDEX(A1:C3,3,-0.999)`,
`INDEX(A1:C3,-0.5,1)`, `INDEX(A1:C3,3,-1)` and ROWS/COLUMNS of each before implementing.
Code: `Model::index_impl` (crates/rustsheet-engine/src/functions/lookup_and_reference/index.rs).

## Acceptance criteria
- [ ] The INDEX probe cells pass; lookup.md's entry is VERIFIED.
- [ ] `INDEX(Data!A1:C3,3,-1e-15)` in gen-m9/1466c66c61189a83's data is the row A3:C3 (a unit test); gen-m9/1466c66c61189a83 A1 passes in `./target/release/rustsheet-eval show gen-m9/<case>.case.json` once T-1290 has also landed, with no new failing gen-m9 cell and no new known failures in eval/baseline/.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
