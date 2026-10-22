---
id: T-2347
title: "Probe whether INDIRECT and TRIMRANGE lift a 1x1 array argument"
milestone: M7
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: [T-2298]
---

## Goal
Excel evidence for whether a 1x1 array argument lifts INDIRECT and TRIMRANGE into one reference element, as it does OFFSET, TAKE and DROP, and the engine matching it.

## Context
From T-2298. `OFFSET(A1:B2,{1},0)` and `DROP(A1:B3,{1})` have ROWS 1 in Excel (probes/m7-one-cell-operand-lift F6, F16, F17), so OFFSET, TAKE and DROP read their arguments with `lifts_one_by_one` (functions/lookup_and_reference/ref_lift.rs). INDIRECT and TRIMRANGE still use `lifts`, where a 1x1 array is its value, unprobed. The spilled `#VALUE!` of a 1x1 lifted element is probed only for TAKE (probes/m7-lift-take-drop-sortby W65), not for DROP or OFFSET.

Probe (excel-probe skill, one new case; A1:B3 = 1,10;2,20;3,30): `ROWS(INDIRECT({"A1:B2"}))`, `COLUMNS(INDIRECT({"A1:B2"}))`, `ROWS(INDEX(INDIRECT({"A1:B2"}),1,1))`, `ROWS(TRIMRANGE(A1:B3,{1}))`, `ROWS(INDEX(TRIMRANGE(A1:B3,{1}),1,1))`, `ISREF(INDIRECT({"A1"}))`, and spilled directly `=DROP(A1:B3,{1})`, `=OFFSET(A1:B2,{1},0)`, `=INDIRECT({"A1:B2"})`.

## Acceptance criteria
- [ ] The probe case is oracle-filled (excel-graph) and its finding is in docs/excel-semantics/arrays.md ("A 1x1 array in a lifted parameter").
- [ ] If Excel lifts a 1x1 array there, INDIRECT and TRIMRANGE use `lifts_one_by_one` and the case passes; otherwise the `lifts` doc cites the probe. Either way, no regression.
- [ ] `make check` green; reviewers approve.

## Out of scope
- EXPAND and WRAPROWS (T-1835).
