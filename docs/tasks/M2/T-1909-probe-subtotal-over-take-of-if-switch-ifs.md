---
id: T-1909
title: "Probe SUBTOTAL over TAKE of IF/SWITCH/IFS with an array selector, and legacy CHOOSE with an array index inside a scalar function"
milestone: M2
lane: engine
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's values for two forms next to T-1882 are probed and ours match.

## Context
T-1882 made SUBTOTAL/AREAS read TAKE/DROP/TRIMRANGE's first argument as values whenever its static shape is an array
(`is_read_reference`, static_analysis/aggregate.rs), evidenced only for CHOOSE (probes/lift-arrays-choose-arms A69).
It equally covers `SUBTOTAL(9,TAKE(IF({TRUE,FALSE},A1:A2,B1:B2),1))` and SWITCH/IFS with an array selector: unprobed.
T-1882 also gave CHOOSE's index the legacy `@` of a scalar argument (`is_choose_index`, legacy.rs;
probes/legacy-choose-array-index covers top-level, SUM and ROWS only); a legacy CHOOSE with an array index inside a
scalar function (`=ABS(CHOOSE({1,2},H1:H2,I1:I2))`, `=LEN(...)`, `=N(...)`) may combine with the T-1713 call-level `@`.
Probe both groups with the excel-probe skill (FORMULATEXT for the legacy cells, on and off the arms' rows) and fix any
mismatch (from T-1882's fidelity review).

## Acceptance criteria
- [ ] A probe case with oracle values covers the cells above; references.md records the rule.
- [ ] Ours matches, or each mismatch is fixed or filed; `make check` green; reviewers approve.

## Out of scope
- CHOOSE with an array index under TAKE (T-1882).

## Outcome
Probe probes/subtotal-take-array-selector (excel-graph). ABS/LEN over a legacy array-index CHOOSE and SUBTOTAL over TAKE/DROP of IF match; mismatches filed: T-1914 (SWITCH/IFS selector is #SPILL!), T-1915 (legacy N over CHOOSE). references.md updated. Eval: 19 cells, 14 pass, 5 known failures baselined.
