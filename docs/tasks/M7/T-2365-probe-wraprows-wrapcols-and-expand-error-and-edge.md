---
id: T-2365
title: "Probe WRAPROWS, WRAPCOLS and EXPAND error and edge arguments beside a lifted argument"
milestone: M7
lane: functions-lookup
status: todo
priority: P2
size: S
tier: sonnet
depends_on: [T-1835]
---

## Goal
Excel evidence decides how WRAPROWS, WRAPCOLS and EXPAND treat an error or edge argument beside a lifted one, and ours
matches it (or the open cases are filed as tasks).

## Context
T-1835 made a WRAPROWS/WRAPCOLS vector error (or a not-a-vector `#VALUE!`) every lifted element's result, before
wrap_count's `#NUM!`. Its only evidence, gen-m7/90de8755a2e565e6 A889:D894, cannot tell that from "a lifted error element
in pad_with is that element's result": there pad_with is a 6x4 array of `#VALUE!`s too (T-1835 Excel-fidelity review).
arrays.md "EXPAND, WRAPROWS and WRAPCOLS: scalar arguments lift, EXPAND's pad_with does not" marks it UNVERIFIED.
Probe (excel-probe skill), with A1:A3 = 1,2,3:
- `ROWS(WRAPROWS(1/0,2,{0;1}))`, `INDEX(WRAPROWS(1/0,-1,{0;1}),2,1)` (vector error beside a clean lifted pad_with)
- `INDEX(WRAPROWS(A1:A3,-1,{NA();0}),1,1)` and `,2,1` (error pad_with element beside a bad wrap_count)
- `ROWS(WRAPCOLS(A1:C3,2,{0;1}))` (2-D not-a-vector beside a lifted argument)
- `ROWS(EXPAND(1/0,{3;4}))`, `ROWS(EXPAND(NA(),3,{1;2}))` (EXPAND error array beside a lifted argument; ours one error)
- `EXPAND(A1:A2,3,1,{0})` (1x1 array pad_with; ours whole-call `#VALUE!`), `EXPAND(A1:A2,3,1,B1:B2)` with a multi-cell
  range pad_with (ours intersects), `EXPAND(1,1048577,1,{0;1})` (array pad_with vs rows past the grid; ours `#VALUE!`)
Code: `lift_wrap` in functions/lookup_and_reference/wrapcols_wraprows.rs, `fn_expand` in expand.rs.

## Acceptance criteria
- [ ] The probe case lands with oracle values; arrays.md's entry is VERIFIED or corrected with the evidence.
- [ ] Every probed cell passes, or its known failure is owned by a follow-up task.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Lifting changes to other functions (TOCOL, UNIQUE, TAKE/DROP).
