---
id: T-2992
title: "FORMULATEXT reads a multi-area pick's first area and a multi-cell INDIRECT element's top-left cell (probes/t2831-formulatext-indirect-multi-cell)"
milestone: M5
lane: functions-lookup
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
FORMULATEXT gives Excel's result for a multi-area reference (a bare union or a pick call returning one) and for a
lifted INDIRECT element that covers several cells. Every cell of probes/t2831-formulatext-indirect-multi-cell passes.

## Context
T-2831's probe (excel-graph, 2026-10-28; references.md "FORMULATEXT of a multi-area reference and of a multi-cell
INDIRECT element"; Data!A2, A4, B1 formulas, A1, A3 constants). 21 of its 34 cells fail today:
- A lifted INDIRECT element that names several cells is FORMULATEXT of that range's top-left cell, one result per
  element, with no spill of its own: `INDIRECT({"Data!A1:A2","Data!A3"})` spills `#N/A`, `#N/A` (A1:B1),
  `INDIRECT({"Data!A2:A3","Data!A4"})` spills `=A1+1`, `=A3*2` (A5:B5), `INDIRECT({"Data!A2:B2"})` is `=A1+1` (A21), and the
  column, 2x2 and range-of-text forms agree (A9:A14, A17:B18, A29:B29). The engine gives `#VALUE!` for each such
  element (`formula_texts_of_grid`, crates/rustsheet-engine/src/functions/lookup_and_reference/formulatext.rs).
- A multi-area reference is FORMULATEXT of its first area: `FORMULATEXT((Data!A2,Data!B1))` is `=A1+1`,
  `FORMULATEXT((Data!A1,Data!A2))` `#N/A`, and the same through `CHOOSE(1,..)`, `IF(TRUE,..)`, `CHOOSE(2,5,..)`,
  `IF(FALSE,5,..)`, `LET(x,..,x)`, `SWITCH(1,1,..)` and `IFS(TRUE,..)` (A33-A73). A first area of several cells spills:
  `CHOOSE(1,(Data!A2:A3,Data!B1))` spills `=A1+1`, `#N/A` down (A77:A78). The engine gives `#VALUE!` for the picks
  (`several_areas`, reference_operators.rs, via `fn_formulatext`) and `#ERROR!` for the bare sheet-qualified union (A33, A37).

## Acceptance criteria
- [ ] probes/t2831-formulatext-indirect-multi-cell passes in full; `make eval-baseline` drops its lines and adds none.
- [ ] references.md's entry names the implementation.
- [ ] `make check` green; reviewers approve.

## Out of scope
- FORMULATEXT of other multi-area producers (INDIRECT of a union text, OFFSET), unprobed.
