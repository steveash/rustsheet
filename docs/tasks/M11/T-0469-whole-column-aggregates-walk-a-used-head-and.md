---
id: T-0469
title: "Whole-column aggregates walk a used head and one repeated tail row (SPEC-0020)"
milestone: M11
lane: perf
status: done
priority: P2
size: M
tier: opus
depends_on: [T-0456]
---

## Goal
An aggregate (SUM, MIN, MAX, AVERAGE, COUNT, ...) over an element-wise expression of whole columns
does work proportional to the used rows, with bit-identical results (SPEC-0020 Rollout 1).

## Context
Spec: docs/specs/SPEC-0020-whole-column-array-operands.md (Design: representation, the static
check, evaluation, the aggregate-walk consumer). corpus-bench 59932 (150 `MIN(IF($D:$D=K2,$F:$F))`,
about 55 s) and 17049 (405 `MAX(IF(F:F=A1,G:G,""))`, over 120 s) are `!Timeout` in
eval/baseline/corpus-bench.txt (T-0430). Code: `evaluate_range` (cast.rs), `handle_comparison`
(arithmetic.rs), `if_pick`/`if_array` (functions/logical/mod.rs), `walk_aggregate_arg` and
`walk_array_items` (functions/util/aggregate_args.rs).

## Acceptance criteria
- [x] `tailed_array.rs`: `Tailed`, `tailed_shape`, `evaluate_tailed` for literals, scalar
      and range references, comparison/arithmetic/concatenation/unary/percent operators and IF; the
      element kernels shared with today's paths (not copied).
- [x] `walk_aggregate_arg` walks a tailed argument's elements through the shared per-element body.
- [x] Equivalence, static-check (bare range, scalar-condition IF) and deep-nesting tests from the
      spec's test plan; mutation check that a wrong
      head length or tail value fails a test.
- [x] Bench `aggregate -- whole_column`; corpus-bench 59932 and 17049 under 60 s through the
      harness, their `!Timeout` lines leave the baseline via `make eval-baseline`; release
      timings before/after in the Outcome.
- [x] `make check` green; reviewers approve.

## Out of scope
- Functions inside the expression (T-0470) and SUMPRODUCT (T-0471).

## Outcome
- As built (SPEC-0020 revised in the same PR):
  - `tailed_array.rs` decides from the AST alone (`tailed_leaves`, an explicit stack, so chains
    cost no recursion) whether an aggregate argument is an element-wise expression of single whole
    columns, literals and one-cell references under value operators and IF. Every IF condition
    must reach a whole column, and the top node must not be a bare range.
  - `read_tailed` then runs the walk's own read of the argument twice through the ordinary
    evaluator: once with each whole column read down to the last used row (after its
    `clip_to_used_area` record), and once with each one the element `{Empty}`.
  - The leaves are matched by node address and formula cell in `evaluate_node_in_context`'s
    `RangeKind` arm (`tailed_operand`; `range_corners` is extracted from that arm).
  - No operator, IF or error kernel is copied.
  - The aggregate walk (`walk_array_nodes`, now shared with `walk_array_items`) visits the head,
    then the tail element once per remaining row. `for_each_aggregate_number` alone skips a tail
    its rules ignore (`AggregateWalk::ignores`). If the used area grows mid-head, or the head or
    tail has an unexpected shape, the argument is read the ordinary way.
- Bench `cargo bench -p rustsheet --bench aggregate -- whole_column` (400
  `MAX(IF(F:F=A1,G:G,""))` over 2,000 rows, release): 109.3 s on `main` -> 0.217 s.
- corpus-bench, release, load + evaluate:
  - 17049 (405 `MAX(IF(F:F=A1,G:G,""))`): over 120 s -> 0.27 s; all 405 cells match Excel.
  - 59932 (150 `MIN/MAX(IF($D:$D=K2,$F:$F))`): about 55 s -> 2.6 s; 14,734 of 32,948 cells match.
    The rest come from VLOOKUPs into the linked workbook `[1]Sort!$B:$Z` (`#ERROR!` upstream,
    T-0457), which the whole-column aggregates then pass on.
  - Both `!Timeout` lines left eval/baseline/corpus-bench.txt (`make eval-baseline`: fixed 2, 0
    regressions in every suite; 59932 is now a value-failure line).
- Tests: `test_tailed_array.rs` evaluates every case with the tailed reading on and off (a
  test-only switch) and counts the tailed walks. It covers:
  - IF over whole-column conditions, every operator, and numeric, text and boolean tails under
    the number-folding aggregates;
  - scalar error operands, operands on another sheet, a one-row head and a cycle through a
    scalar operand;
  - the excluded forms (none tailed), a 4,000-link chain and a spill into the tail (restart).

  A unit test in aggregate_args.rs covers a text tail under `TextZero`. Mutations of the tail
  length, the tail value and each branch of the skip rule each fail a test.
