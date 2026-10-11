---
id: T-1211
title: "Probe MEDIAN/STDEV/GEOMEAN/SUMSQ/MIN/MAX/COUNTA over single-cell XLOOKUP/TAKE results and SUBTOTAL over computed-array TAKE (follow-up of T-1206)"
milestone: M1
lane: functions-math
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Aggregates sharing `for_each_value` (MEDIAN, GEOMEAN, STDEV.S, SUMSQ, ...), MIN, MAX, COUNTA, AVERAGEA over a single-cell XLOOKUP/TAKE result holding text, a boolean or a blank, and SUBTOTAL/AREAS over `TAKE(SEQUENCE(4),1)`, are probed on excel-graph and the engine matches.

## Context
T-1206 made SUM, COUNT and `for_each_value` read these results as references (`evaluate_picked_reference_arg`); the other functions are unprobed. See references.md "SUBTOTAL and AREAS take references only". Extend probes/subtotal-reference-operands-c or add -d.

## Acceptance criteria
- [ ] A probe covers the functions above; references.md states each rule with the case as evidence.
- [ ] The engine matches; eval baseline shrinks accordingly.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probed (probes/subtotal-reference-operands-d, excel-graph): MEDIAN, STDEV.S, GEOMEAN, SUMSQ and COUNT/COUNTA already matched; MIN, MAX and AVERAGEA now read a single-cell XLOOKUP/TAKE result as a reference, and SUBTOTAL/AREAS over `TAKE(SEQUENCE(..))` size as arrays (spill). Rules in references.md. 42 probe cells, 41 pass; 1 known failure (H28, T-1284). Follow-up: T-1284 (GEOMEAN 1 ulp).
