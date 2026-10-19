---
id: T-1976
title: "2-argument SUMIF/AVERAGEIF evaluate their shared range argument once"
milestone: M2
lane: functions-criteria
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
`SUMIF(range,criteria)` and `AVERAGEIF(range,criteria)` (no sum/average range) evaluate `range` once per formula,
so a volatile range (`AVERAGEIF(OFFSET(A1,RANDBETWEEN(0,5),0,1),">0")`) uses one draw for both its criteria and
its summed cells, as Excel evaluates each argument once.

## Context
T-0408 review (Excel fidelity): `fn_averageif` (functions/statistical/if_ifs.rs) and the 2-argument SUMIF in
functions/math_and_trigonometry/mathematical.rs clone the range node into both the sum-range and the
criteria-range roles, so `resolve_sum_ranges` evaluates it twice. T-0408 made every other *IF(S) range argument
evaluate once (criteria.md "A lifted *IF(S) evaluates its range arguments once per call",
probes/ifs-volatile-range-once). Only observable with a volatile range whose origin varies.

## Acceptance criteria
- [ ] A probe case (excel-probe skill; `invariant` cells, e.g. AVERAGEIF over a varying-origin OFFSET whose
      result is draw-independent only when evaluated once) has an excel-graph oracle and passes.
- [ ] The shared range is evaluated once (pass its value to both roles instead of cloning the node).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The 3-argument forms (already evaluated once since T-0408).
