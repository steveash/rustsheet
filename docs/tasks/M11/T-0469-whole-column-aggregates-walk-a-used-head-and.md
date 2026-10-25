---
id: T-0469
title: "Whole-column aggregates walk a used head and one repeated tail row (SPEC-0020)"
milestone: M11
lane: perf
status: todo
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
- [ ] `tailed_array.rs`: `Tailed`, `tailed_shape`, `evaluate_tailed` for literals, scalar
      and range references, comparison/arithmetic/concatenation/unary/percent operators and IF; the
      element kernels shared with today's paths (not copied).
- [ ] `walk_aggregate_arg` walks a tailed argument's elements through the shared per-element body.
- [ ] Equivalence, static-check (bare range, scalar-condition IF) and deep-nesting tests from the
      spec's test plan; mutation check that a wrong
      head length or tail value fails a test.
- [ ] Bench `aggregate -- whole_column`; corpus-bench 59932 and 17049 under 60 s through the
      harness, their `!Timeout` lines leave the baseline via `make eval-baseline`; release
      timings before/after in the Outcome.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Functions inside the expression (T-0470) and SUMPRODUCT (T-0471).
