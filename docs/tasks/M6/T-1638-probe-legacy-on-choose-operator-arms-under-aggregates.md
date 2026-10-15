---
id: T-1638
title: "Probe legacy @ on CHOOSE operator arms under aggregates and on array constants in other scalar functions"
milestone: M6
lane: engine
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's legacy `@` placement is evidenced for CHOOSE operator arms nested under aggregates and for one-element array constants in scalar functions other than ABS, and our import/export matches it.

## Context
T-1633 added `Function::Choose` to `legacy_value_operand` (arms intersect in every context, not only at the root) and `add_scalar_array_at` (`@{x}` for any scalar argument), but only probes/legacy-one-cell-shape-intersection H3, P3 (root) and M3 (ABS) back them. Code: crates/rustsheet-engine/src/expressions/parser/static_analysis/legacy.rs. Reviewer-raised.

## Acceptance criteria
- [ ] A probe with legacy FORMULATEXT and off-row values for SUM, INDEX, MMULT, MODE and LOOKUP wrapping `CHOOSE(1,range op)`, and for `{5}` in other scalar functions (ROUND, TYPE), run through the oracle.
- [ ] Any mismatch is fixed or recorded in docs/excel-semantics/references.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Added probes/legacy-choose-arm-aggregate-and-array-constant-at (27 cells, excel-graph): 23 pass. SUM/MAX over a CHOOSE operator arm, `ROUND({5},0)` and the `@` placement match. `TYPE({5})` takes no `@` (ours added one; fixed in `legacy.rs`, the `@` now only goes on `Signature::Scalar` arguments). INDEX/MODE/LOOKUP over a CHOOSE arm differ (H1, H3, I10, J1): recorded in references.md and filed as T-1643.
