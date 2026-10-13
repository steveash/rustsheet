---
id: T-1473
title: "Spec: one aggregate-argument walker for SUM, AVERAGE, MIN, MAX, PRODUCT, COUNT and COUNTA"
milestone: M2
lane: engine
status: todo
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
A short spec in docs/specs/ (the next free SPEC number) defines the single walker over an aggregate
function's arguments. It covers which argument shapes yield which values, how direct values are coerced, and
the error-precedence rule. It also names every M2 consumer that moves onto it. With it, M2's exit criterion
"one implementation of aggregate-argument iteration, used by every consumer" can be implemented and checked.

## Context
T-0200/T-0204 unified the *range* walks (`for_each_number_in_areas`, `for_each_coerced_number_in_areas` in
functions/util.rs). The per-argument dispatch above them is still written four times, each with its own
`let mut reference_error = None` precedence logic. Each copy evaluates `evaluate_picked_reference_arg` and
classifies Number/Boolean/String/EmptyArg/Array/Ranges/Error:
- `Model::for_each_value` / `for_each_value_skip_empty` / `for_each_value_a`
  (functions/statistical/count_and_average.rs): the AVERAGE family, GEOMEAN, VAR/STDEV, DEVSQ, MEDIAN and more;
- `Model::sum_with_last_addition` (functions/math_and_trigonometry/sum.rs): SUM;
- `Model::fold_min_max` (functions/math_and_trigonometry/mathematical.rs): MIN, MAX. Its doc says "coerced as SUM does";
- `Model::fn_product` (functions/math_and_trigonometry/product.rs): PRODUCT.
COUNT and COUNTA (functions/statistical/count.rs) have their own dispatch too.
The evidence for the shared rules is already recorded: docs/excel-semantics/syntax.md "Aggregate error
precedence" (probes/aggregate-error-precedence, probes/sum-error-precedence) and coercion.md (probes/min-max-value-args,
probes/average-value-args). The differences between consumers must be kept: AVERAGE counts an empty argument as
0 while DEVSQ/SUMSQ skip it (TODO(T-0236)), SUM tracks its last addition for near-zero snapping, the A-variants
coerce in ranges, and COUNT counts types instead of values.
Use docs/specs/TEMPLATE.md.

## Acceptance criteria
- [ ] docs/specs/SPEC-NNNN-aggregate-argument-walker.md: the walker's signature, the per-consumer policy (the
      differences above, each with its evidence link) and the error-precedence rule. It includes the migration
      order: T-1474 (walker, SUM, AVERAGE family), then T-1476 (MIN, MAX, PRODUCT, COUNT, COUNTA).
- [ ] docs/architecture/overview.md's "one definition per concept" line points to the spec.
- [ ] No engine code changes; `make check` green; reviewers approve.

## Out of scope
- The implementation (T-1474, T-1476).
- *IF(S) criteria walks (T-1477) and the D-functions.
