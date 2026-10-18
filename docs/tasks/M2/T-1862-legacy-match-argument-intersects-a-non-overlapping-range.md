---
id: T-1862
title: "Legacy MATCH argument intersects a non-overlapping range to VALUE (corpus-bench 49300)"
milestone: M2
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
corpus-bench/49300 `Sheet1!C2` matches Excel's cached `#VALUE!`.

## Context
Split from T-1791. A legacy (no `cm`) cell `=INDEX(Data!C3:AY4,MATCH(1,(Sheet1!A2=Data!B3:B4)*(Sheet1!B2=Data!C2:AY2),0))` is `#VALUE!` in Excel and `#N/A` in the engine. Suspect implicit intersection of `Data!B3:B4` from row 2 of another sheet (no overlap: `#VALUE!`) inside a legacy cell's MATCH lookup_array argument; check how the xlsx import marks the formula legacy (T-1717 added `@` placement for legacy formulas) and which `@` the engine inserts here. Inspect with `./target/release/rustsheet-eval show corpus-bench/49300/1_49300_input.xlsx` (fetch the suite first). Out of scope: other legacy-parse work (T-1096).

## Acceptance criteria
- [ ] corpus-bench/49300 C2 passes, with a unit or spec test for the cause; any quirk goes into docs/excel-semantics/.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
