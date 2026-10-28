---
id: T-2831
title: "Probe FORMULATEXT of a multi-cell lifted INDIRECT element and a multi-area pick (T-2726)"
milestone: M5
lane: research
status: done
priority: P3
size: S
tier: opus
depends_on: []
---


## Goal
Excel's FORMULATEXT is known (oracle evidence) for two forms T-2726 implemented by inference:
a lifted INDIRECT element that covers several cells, and a pick call that yields a multi-area
reference.

## Context
T-2726 made `fn_formulatext` read pick calls and lifted INDIRECT per element
(docs/excel-semantics/references.md "FORMULATEXT of the forms T-2526 left open"). Two cases have
no probe: `FORMULATEXT(INDIRECT({"A1:A2","A3"}))` (the engine gives `#VALUE!` for the multi-cell
element, `formula_texts_of_grid`) and `FORMULATEXT(CHOOSE(1,(A1,B1)))` /
`FORMULATEXT(IF(TRUE,(A1,B1)))` (the engine gives `#VALUE!`, `several_areas`). Use the
`excel-probe` skill; put formulas in the probed cells and in plain cells so both readings show.

## Acceptance criteria
- [x] A probes case with oracle values for both forms (and the column and 2x2 INDIRECT shapes).
- [x] The finding recorded in docs/excel-semantics/references.md; an implementation task filed if the engine differs.
- [x] `make check` green; reviewers approve.

## Out of scope
- Changing the engine (a follow-up if the probe disagrees).

## Outcome
probes/t2831-formulatext-indirect-multi-cell (excel-graph, 2026-10-28; 34 cells): a multi-area reference, bare or
returned by CHOOSE, IF, LET, SWITCH or IFS, is read as its first area (a multi-cell first area spills), and a lifted
INDIRECT element that names several cells is read as its top-left cell, one result per element. Recorded in
references.md ("IF and INDEX return references to aggregates", the T-2831 paragraph). The engine fails 21 of the 34
cells (`#VALUE!`, and `#ERROR!` for the bare sheet-qualified union); implementation filed as T-2992 (sonnet). Eval delta:
new case, 21 new known failures in eval/baseline/probes.txt (new coverage, no regression).
