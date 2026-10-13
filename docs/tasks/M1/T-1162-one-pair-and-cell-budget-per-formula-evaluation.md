---
id: T-1162
title: "One pair and cell budget per formula evaluation for multi-area references"
milestone: M1
lane: perf
status: in-progress
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
- [ ] Unit test: a formula repeating a LET-bound union across arguments hits `#VALUE!` (pairs or cells) once the shared budget is spent; the literal-union tests are unchanged.
- [ ] Measured: the 120-argument formula above is no slower than its literal-union equivalent (numbers in the Outcome).
- [ ] No eval regressions; references.md "Union area cap" describes the per-formula budget.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing `MAX_AREAS` or `MAX_HELD_AREAS`.
