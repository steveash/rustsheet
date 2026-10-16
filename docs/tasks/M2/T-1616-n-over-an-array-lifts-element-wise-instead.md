---
id: T-1616
title: "N over an array lifts element-wise instead of #N/IMPL!"
milestone: M2
lane: functions-logic
status: done
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
- [x] probes/legacy-index-array-index L1, L10 pass, and a probe case covers the formats above (excel-graph) and passes.
- [x] `make check` green; reviewers approve.

## Out of scope
- T, and other information functions.

## Outcome
`fn_n` lifts over an array element-wise (`n_of`, reusing `map_elements`). Probe `probes/n-array-lift` (23 cells,
excel-graph) covers the dynamic and legacy forms, plus the T-0294 leftovers (legacy `SUMPRODUCT(INDEX(B1:B6,ROW(B1:B2)))`
is 1 and `SUMPRODUCT(INDEX(ABS(B1:B6),0))` 21, on and off the rows). probes/legacy-index-array-index L1 and L10 pass.
The probe passes 21 of 23 cells: a legacy cell `=N(IF(1,{2;3}))` is `#VALUE!` in Excel and spills in ours (S!D4:D5), a
new-coverage known failure filed as T-1713. Eval: probes known failures 2284 → 2283 (fixed legacy-index-array-index L1, L10 and index-1x1-array-index H21;
+2 new coverage D4:D5), 0 regressions. information.md and criteria.md updated.
