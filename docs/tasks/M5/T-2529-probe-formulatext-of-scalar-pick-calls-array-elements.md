---
id: T-2529
title: "Probe FORMULATEXT of scalar pick calls, array elements that are errors, array INDIRECT and XLOOKUP/TAKE/DROP/LET/SWITCH/IFS results"
milestone: M5
lane: research
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's FORMULATEXT result is known (oracle-evidenced) for the forms T-2526 left open.

## Context
T-2526 left these at the engine's `#ERROR!` with no evidence: `IF(TRUE,5,A1)` and `CHOOSE(1,5,A1)` (scalar pick), `IF({TRUE,FALSE},NA(),A2)` (error array elements), `INDIRECT({"A1","A2"})`, and XLOOKUP, TAKE, DROP, LET, SWITCH and IFS results. See `fn_formulatext` and docs/excel-semantics/references.md.

## Acceptance criteria
- [ ] A probe under eval/suites/probes with excel-graph oracle values covers each form; findings recorded in docs/excel-semantics/references.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
