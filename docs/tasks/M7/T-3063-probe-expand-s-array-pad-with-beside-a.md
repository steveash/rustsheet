---
id: T-3063
title: "Probe EXPAND's array pad_with beside a column count past the grid or a target over the cell bound, and WRAPCOLS/WRAPROWS of an empty one-cell range or omitted vector"
milestone: M7
lane: functions-lookup
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel evidence decides two error orders that T-2479 (#2062) changed without probing them: an array pad_with `#VALUE!` against EXPAND's column `#NUM!` and against our decision-0014 cell bound, and an empty-cell `#VALUE!` from WRAPCOLS/WRAPROWS for a one-cell empty range or an omitted vector.

## Context
Steward re-review of #2062 (T-2479), reviewer-excel-fidelity MINOR findings, 2026-10-28.
- `expand.rs` `expand_kernel` now returns the deferred array-pad `#VALUE!` only after these checks: `target_rows < src_rows`, `COLUMNS_REFUSED` and `check_stack_size`. The oracle covers only the rows-past-grid case (probes/t2479-expand-padarray-past-grid G1). Two cases are unprobed:
  - `EXPAND(1,1,1048577,{0;1})`: ours gives `#NUM!`.
  - `EXPAND(1,2,600000,{0;1})`: ours gives the decision-0014 limit error, where Excel very likely gives `#VALUE!`, which needs no allocation. If Excel says `#VALUE!`, the pad error must come before `check_array_cells`, since directive 5 does not need the limit first here.
- `wrapcols_wraprows.rs` `flatten_vector` now returns `VectorError::Error` for an empty cell. That also covers `EmptyArg` (`WRAPCOLS(,NA())`) and `Z1:Z1`, but the probes cover only a bare `Z1` (G9, G10, G35, G36). The `lift_wrap` comment should then name the 2-D-array not-a-vector case only.

## Acceptance criteria
- [ ] Probe case (excel-graph) with `EXPAND(1,1,1048577,{0;1})`, `EXPAND(1,2,600000,{0;1})` (or the smallest target over our cell bound that Excel builds), `WRAPCOLS(Z1:Z1,NA())`, `WRAPROWS(Z1:Z1,1/0)`, `WRAPCOLS(,NA())`; arrays.md updated; engine matches or failures owned.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Five probe cases (probes/t3063-*, excel-graph 2026-10-29): `EXPAND(1,1,1048577,{0;1})` is `#NUM!`, `EXPAND(1,2,600000,{0;1})` is `#VALUE!`, and `WRAPCOLS(Z1:Z1,NA())`, `WRAPROWS(Z1:Z1,1/0)`, `WRAPCOLS(,NA())` are `#VALUE!`. The engine already matched all five, so no code changed; arrays.md records the rules. Eval delta: five new passing probe cells (five `C` lines in eval/baseline/probes.txt from `make eval-baseline`), no known-failure change.
