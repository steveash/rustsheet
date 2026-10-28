---
id: T-2807
title: "Spec: bound the total work of one recalculation pass"
milestone: M7
lane: engine
status: done
priority: P2
size: M
tier: opus
depends_on: [T-2803]
---

## Goal
A spec in docs/specs/ bounds the total work of one recalculation pass, not just of one formula. Its result past the bound
is deterministic.

## Context
The SPEC-0024 non-goals record this as a residual risk. The per-formula budget (`MAX_FORMULA_WORK`, 2^28 units, about 10 s
at its worst kind) multiplies by the number of formulas, so 1,000 cells near the limit is hours. The per-call and per-formula
caps (area budgets, `MAX_LIFTED_CELL_VISITS`) have the same scaling. Questions the spec must answer:
- the unit, which should be SPEC-0024's work units summed over the pass;
- which cells give up past the bound, deterministically and independent of evaluation order where possible;
- how it interacts with iterative calculation (`iteration.rs` `MAX_SWEEP_EVALUATIONS`) and with incremental recalculation;
- the DEVIATION entry.

## Acceptance criteria
- [x] docs/specs/SPEC-XXXX answers the questions above, with a worked example of 1,000 near-limit cells.
- [x] Implementation tasks are filed (S/M).
- [x] Reviewers approve.
- [x] `make check` green; reviewers approve.

## Out of scope
- The implementation.

## Outcome
docs/specs/SPEC-0028-recalculation-work-budget.md: the units are SPEC-0024's summed at run boundaries in
`with_area_budget` (the hot path is unchanged). A pass may commit `MAX_PASS_WORK` (2^30) and a round may throw
away as much; past either, the round re-runs in share mode, each formula limited to 2^30 / (4N), so which cells
give up depends on the model, not on the evaluation order (except through thrown-away work, which the spec
names). `MAX_RECALC_WORK` (2^32) is the time ceiling over every phase; sweeps and data tables stop at it as their
count caps do. Worked example of 1,000 near-limit cells: about 2 hours today, about 30 to 38 s with the spec.
Incremental-recalculation requirements recorded for SPEC-0002 (T-0401). Implementation filed as T-3008 (M, opus)
and T-3009 (S, opus). Eval delta: none (docs only).
