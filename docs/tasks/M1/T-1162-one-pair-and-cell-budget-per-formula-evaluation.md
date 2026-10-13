---
id: T-1162
title: "One pair and cell budget per formula evaluation for multi-area references"
milestone: M1
lane: perf
status: done
priority: P2
size: M
tier: opus
depends_on: [T-1064]
---

## Goal
One formula evaluation shares one `MAX_AREA_PAIRS` budget and one `MAX_AREA_CELLS` budget across all its arguments and operands, so the worst-case work of a formula holding LET-bound unions is no larger than that of its literal-union equivalent.

## Context
T-1064 performance review: the pair budget restarts per argument and per IF/CHOOSE/LET operand (`evaluate_reference_arg` → `reference_ranges` creates a new budget), and LET makes 256-area unions cheap to build, so a ~7K-character formula (`LET(u_1,(A1,A1),...,u_8,..., SUM(IF(1,u_8 u_8 A1 ...),...))`, 120 args) measured 1.06 s per cell against 0.25 s for the literal-union equivalent. Likewise `SUM(x,x,...)` with a LET-bound union walks `MAX_AREA_CELLS` per argument (round-1 finding 6). Shape: a budget counter on `Model` reset per cell evaluation, replacing the per-argument/per-operand budgets (reference_operators.rs `MAX_AREA_PAIRS`, functions/util.rs `check_area_cells`). Also: let `intersect_areas` borrow `&[Range]` from `Areas` instead of `areas.0.to_vec()` in `evaluate_areas`, and keep the allocation-free pair path when both evaluated operands are single ranges (`LET(r,A1:C3,r B2)` measured +7–10%). Update references.md "Union area cap" (DEVIATION, decision 0014).

## Acceptance criteria
- [x] Unit test: a formula repeating a LET-bound union across arguments hits `#VALUE!` (pairs or cells) once the shared budget is spent; the literal-union tests are unchanged.
- [x] Measured: the 120-argument formula above is no slower than its literal-union equivalent (numbers in the Outcome).
- [x] No eval regressions; references.md "Union area cap" describes the per-formula budget.
- [x] `make check` green; reviewers approve.

## Out of scope
- Changing `MAX_AREAS` or `MAX_HELD_AREAS`.

## Outcome
- `AreaBudget` (new `area_budget.rs`, with `MAX_AREA_PAIRS`, `MAX_AREA_CELLS`, their messages and `check_area_cells`, moved from reference_operators.rs and functions/util.rs) on `Model` holds what the running formula has left of `MAX_AREA_PAIRS` and `MAX_AREA_CELLS`; `Model::with_area_budget` gives a fresh one to each cell formula run (`run_cell_formula`, so a cell read mid-formula has its own and restores the outer one), each conditional-format evaluation and each `evaluate_formula` call. The per-argument / per-operand `let mut budget = MAX_AREA_PAIRS` and the `budget: &mut usize` threading are gone; `check_area_cells` now spends from the shared cell budget.
- `evaluate_areas` returns an `AreaList` (`One` / `Owned` / `Shared`): a LET-bound union is borrowed from its `Arc` instead of `to_vec`, two single areas intersect with no list, and `reference_ranges` hands a shared list on without a copy.
- AGGREGATE's reference form now checks its multi-area arguments against the cell budget too (it walked area by area unbounded).
- Tests: `a_let_bound_union_shares_the_pair_budget_across_arguments`, `a_let_bound_union_shares_the_cell_budget_across_arguments`, `aggregate_of_a_union_obeys_the_area_cell_bound`; the literal-union tests are unchanged.
- Measured (release, best of 5, one cell): the 120-argument LET formula (6,840 chars) 0.619 s → 0.006 s (`#VALUE!`, budget spent after the first arguments); its literal-union equivalent 0.647 s → 0.005 s. Within budget, one argument: LET 0.005 s, literal 0.005 s. `LET(r,A1:C3,SUM(r B2,...))` (2,000 cells × 200 cuts): 19.78 G → 18.65 G instructions under callgrind (−5.7%); wall time within noise (0.091 s vs 0.095 s).
- File-size ratchet: `Binding`/`VariableStack` moved unchanged to `variable_stack.rs` and `Model::evaluate_formula` to `outside_cell.rs` (with `evaluate_outside_cell`, the conditional formats' entry point), so reference_operators.rs stays under 1,000 lines and model.rs shrinks (3,499 → 3,476; the protected lock is left for a steward ratchet).
- references.md "Union area cap" describes the per-formula budgets (DEVIATION, decision 0014).
- Eval: no change (no case reaches the budgets).
- Follow-ups: T-1461 (AGGREGATE borrows its areas), T-1462 (probe large multi-area walks in MAP/BYROW), T-1463 (bound seen-range records of long walks).
