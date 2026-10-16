---
id: T-1616
title: "N over an array lifts element-wise instead of #N/IMPL!"
milestone: M2
lane: functions-logic
status: in-progress
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
N over an array (or an array-valued expression) gives an array of N of each element, as Excel does, instead of `#N/IMPL!`.

## Context
probes/legacy-index-array-index L1, L10 (excel-graph, T-0294): `SUMPRODUCT(INDEX(B1:B6,N(IF(1,{2;3}))))` is 5 in Excel
(the classic way to feed an array to legacy INDEX); the engine is `#N/IMPL!` because `fn_n`
(crates/rustsheet-engine/src/functions/information/mod.rs) rejects an array argument. A range keeps its top-left cell
(probes/info-logic-argument-modes A6:A7), so only arrays lift. Probe first (excel-probe skill): `N({1,"a",TRUE})`
spilled, `SUM(N({1,2}))`, `N(IF(1,{2;3}))` dynamic and legacy, `N(B1:B3*1)`; also legacy `SUMPRODUCT(INDEX(B1:B6,ROW(B1:B2)))` and `SUMPRODUCT(INDEX(ABS(B1:B6),0))` with FORMULATEXT, on and off the row (left unprobed by T-0294). Semantics: docs/excel-semantics/criteria.md
(the INDEX passage under "Beyond *IF(S)").

## Acceptance criteria
- [ ] probes/legacy-index-array-index L1, L10 pass, and a probe case covers the formats above (excel-graph) and passes.
- [ ] `make check` green; reviewers approve.

## Out of scope
- T, and other information functions.
