---
id: T-1984
title: "Probe SUBTOTAL over TAKE of SWITCH with an array case value, IFS with only a later array condition, and IF/CHOOSE/LET-wrapped array selectors"
milestone: M2
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for what Excel spills for SUBTOTAL over TAKE of a SWITCH/IFS whose array lives somewhere other than the selector, and for wrapped array selectors; engine fixes filed if any cell differs.

## Context
T-1914 made `lifts_to_values` (static_analysis/aggregate.rs) treat SWITCH/IFS with an array
value or condition as lifting, recursing through IF/CHOOSE/SWITCH/IFS arms and LET, so static
analysis agrees with the evaluator and the debug scalar-context assertion is not reached. Only
the bare forms are oracle-backed (probes/subtotal-take-array-selector A30:B31, A40:B41). The
engine today gives, unevidenced (H1:H2 = 3,4; I1:I2 = 6,8):
- `SUBTOTAL(9,TAKE(SWITCH(1,{1,2},H1:H2,I1:I2),2))` → 14 (array case value; no lift)
- `SUBTOTAL(9,TAKE(IFS(TRUE,H1:H2,{FALSE,TRUE},I1:I2),2))` → 7 (first condition TRUE short-circuits)
- `SUBTOTAL(9,TAKE(IF(TRUE,SWITCH({1,2},1,H1:H2,2,I1:I2)),2))`, `CHOOSE(1,IFS(...))`,
  `LET(x,SWITCH(...),x)` wrapped → 2x2 `#VALUE!` like the bare call.
Probe each in clear space (so the spill shape shows) with the `excel-probe` skill.

## Acceptance criteria
- [ ] A probe case with oracle values covers the five formulas above.
- [ ] Findings recorded in docs/excel-semantics/references.md; an engine task filed for any mismatch.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
