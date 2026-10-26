---
id: T-2807
title: "Spec: bound the total work of one recalculation pass"
milestone: M7
lane: engine
status: todo
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
- [ ] docs/specs/SPEC-XXXX answers the questions above, with a worked example of 1,000 near-limit cells.
- [ ] Implementation tasks are filed (S/M).
- [ ] Reviewers approve.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The implementation.
