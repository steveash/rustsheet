---
id: T-2365
title: "Probe WRAPROWS, WRAPCOLS, EXPAND and TOCOL error and edge arguments beside a lifted argument"
milestone: M7
lane: functions-lookup
status: todo
priority: P2
size: S
tier: sonnet
depends_on: [T-1835]
campaign: visible
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
- `ROWS(TOCOL(1/0,0,{0;1}))`, `ROWS(TOCOL(1/0,{0;1}))` (TOCOL error array beside a lifted argument; T-2313 made it fill
  the shape by analogy with TOROW's gen-m7/611ce3bbfab27cef A919, UNVERIFIED for TOCOL)
Code: `lift_flatten` in tocol_torow.rs, `lift_wrap` in functions/lookup_and_reference/wrapcols_wraprows.rs, `fn_expand` in expand.rs.

gen-m7 cells (T-2254 triage, excel-graph, 2026-10-22):
- `WRAPCOLS(EXPAND({1},2,3),FILTER(Data!D1:E12,(Data!B1:B12=0.1)))` is `#CALC!` (f7e94a817e5385ac A3517): the empty FILTER's `#CALC!` as
  wrap_count comes before the 2x3 not-a-vector `#VALUE!`. Ours `#VALUE!`. Add `WRAPCOLS({1,2;3,4},NA())` and `WRAPROWS({1,2;3,4},1/0,0)`
  to the probe and fix the order.
- The multi-cell range pad_with question above is answered by gen-m7 (17 formulas, one `#VALUE!` each) and is now T-2415's; drop it from
  the probe.
Owned: 1 known-failure lines in eval/baseline/gen-m7.txt, 1 formulas.
- gen-m7/f7e94a817e5385ac.case.json A3517 `=WRAPCOLS(EXPAND({1},2,3),FILTER(Data!D1:E12,(Data!B1:B12=0.1)))`: A3517 Excel #CALC!, ours #VALUE!; cells (1): A3517

## Acceptance criteria
- [ ] The probe case lands with oracle values; arrays.md's entry is VERIFIED or corrected with the evidence.
- [ ] Every probed cell passes, or its known failure is owned by a follow-up task.
- [ ] Visible parity: the gen-m7 line listed under Context (T-2254 triage) disappears from eval/baseline/gen-m7.txt (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Lifting changes to other functions (UNIQUE, TAKE/DROP).

## Visible-parity stage
Decision 0083 (gap triage 2026-10-22, T-2254): every cell this task owns has the wrong type, error, text or value (none is a
last-digit difference), so the task is in the campaign. Its target is that every cell it owns has Excel's type, error and text,
or is within half a unit of the 15th significant digit of Excel's value; the visible-parity criterion under Acceptance criteria
states it.
