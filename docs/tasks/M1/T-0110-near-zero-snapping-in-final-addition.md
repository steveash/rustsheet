---
id: T-0110
title: "Near-zero snapping in final addition/subtraction"
milestone: M1
lane: engine
status: done
priority: P2
size: S
tier: opus
depends_on: [T-0009, T-1017]
---

## Goal
Implement Excel's rule once VERIFIED in docs/excel-semantics/numbers.md.

## Context
Probe cells A2–A4, A15. Also probes/literal-digits C4 (`=0.1+0.2-0.3` = 0, engine 5.55E-17), handed over by T-0144.

## Acceptance criteria
- [x] Probe cells pass (including probes/literal-digits C4); rule documented with evidence; implemented in one place.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Excel's near-zero snapping is implemented in one place, `snap_near_zero` (arithmetic.rs), which uses the fitted rule from numbers.md. It runs only at the cell formula's root, through `Model::evaluate_formula_root` (called by `run_cell_formula`), for an outermost `+`/`-` or SUM (`fn_sum_outermost`; SUM moved to sum.rs). A `GroupedRootKind` root (T-1017) is not snapped.
The work was redone on current main, which has operator chains (T-0128), instead of merging the 2026-10-06 WIP.
Eval:
- probes known failures 269 -> 264 (5 fixed).
- probes-numeric/30bc5f8fab02d0a2 passes 1296/1296, including the 114 `parenthesized-sub` cells.
- 0 regressions in every suite.
- gen-m1 is unchanged at 8 (its last-digit cells are not snapping).
Follow-up: T-1131 (probe the SUM forms the evidence does not cover).

