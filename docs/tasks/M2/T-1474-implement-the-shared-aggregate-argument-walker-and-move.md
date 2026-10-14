---
id: T-1474
title: "Implement the shared aggregate-argument walker and move SUM and the AVERAGE family onto it"
milestone: M2
lane: engine
status: done
priority: P1
size: M
tier: opus
spec: docs/specs/SPEC-0009-aggregate-argument-walker.md
depends_on: [T-1473]
---

## Goal
The walker that T-1473's spec defines exists once. SUM and every current `for_each_value*` consumer (AVERAGE
family, GEOMEAN, VAR/STDEV, DEVSQ, MEDIAN and the rest) use it, with no change in any result.

## Context
See T-1473 for the four duplicated argument dispatches and the rules they share. This task replaces
`Model::for_each_value`, `for_each_value_skip_empty` and `for_each_value_a` (functions/statistical/count_and_average.rs)
and the dispatch loop in `Model::sum_with_last_addition` (functions/math_and_trigonometry/sum.rs) with the
spec's walker. SUM's last-addition tracking for near-zero snapping stays SUM's own (docs/excel-semantics/numbers.md
"Near-zero snapping").

## Acceptance criteria
- [x] One walker, as the spec specifies. `rg "let mut reference_error"` finds it only in the walker, product.rs and mathematical.rs (those two are T-1476's).
- [x] `make eval` shows 0 regressions and 0 fixes in every suite (a pure refactor). probes/aggregate-error-precedence, probes/sum-error-precedence and probes/average-value-args still pass.
- [x] scripts/check_duplication.py does not rise; record any drop with `--update` per decision 0010.
- [x] `make check` green; reviewers approve.

## Out of scope
- MIN, MAX, PRODUCT, COUNT, COUNTA (T-1476).
- Settling T-0236 (empty arguments of DEVSQ/SUMSQ); keep today's behavior.

## Outcome
`functions/aggregate_args.rs` holds SPEC-0009's walker (`walk_aggregate_args` /
`walk_aggregate_arg`, `AggregatePolicy`, `AggregateWalk`) and its number layer
(`for_each_aggregate_number`, `NumberRules`), with unit tests for each error rule, the early exit,
item order and empty array elements.
- SUM (`SUM_ARGS`: Ranked, array errors with referenced) and the `for_each_value` family
  (`AVERAGE_ARGS`, `SKIP_EMPTY_ARGS`, `A_ARGS`) now call it.
- `for_each_value`, `for_each_value_skip_empty` and `for_each_value_a` are thin wrappers with a
  named policy. The last one keeps its linked-workbook step.
- `cell_number` and `coerced_cell_number` (util.rs) are now the one per-cell rule for both the
  walker and `for_each_(coerced_)number_in_areas`.
- Only the variants these consumers use exist so far. `ErrorRule::Items` and the PRODUCT/COUNTA
  readings come with T-1476, so that no variant is dead.
- `rg "let mut reference_error"` now finds only product.rs and mathematical.rs.
- Duplication went from 6,522 to 6,395 lines (`--update`).

Bench `cargo bench -p rustsheet --bench aggregate` (new; release, median of 5, 1,000,000-row
column), main vs branch, interleaved runs:

| case | main | branch |
|---|---|---|
| SUM | 110 / 109 ms | 110 / 109 ms |
| AVERAGE | 115 / 99 ms | 103 / 105 ms |
| SUM, error in A2 | 57 / 78 ms | 64 / 58 ms |

The first version was 1.8× slower. A fast path for a referenced number fixed that.
