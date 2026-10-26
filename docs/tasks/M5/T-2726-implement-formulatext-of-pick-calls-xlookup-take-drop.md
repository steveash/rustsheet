---
id: T-2726
title: "Implement FORMULATEXT of pick calls, XLOOKUP/TAKE/DROP/LET/SWITCH/IFS results and array INDIRECT (probes/formulatext-pick-scalars-and-arrays)"
milestone: M5
lane: functions-lookup
status: todo
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
The engine's FORMULATEXT matches Excel on every cell of probes/formulatext-pick-scalars-and-arrays.

## Context
T-2529 probed these forms; the engine misses 43 of 49 cells (`#ERROR!` for 42; Calc!O1 `INDIRECT({"Data!A2"})` is `#VALUE!` since T-2577 made a 1x1-array INDIRECT a one-element array of references, which FORMULATEXT must read per element). Findings are in docs/excel-semantics/references.md ("FORMULATEXT of the forms T-2526 left open"). Code path: `fn_formulatext`.

## Acceptance criteria
- [ ] All cells of the probe pass and its lines leave eval/baseline/probes.txt.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
