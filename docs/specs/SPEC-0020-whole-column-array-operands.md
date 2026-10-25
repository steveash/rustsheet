# SPEC-0020 — Whole-column array operands: a used head and one repeated tail row

- **Status:** draft
- **Milestone / lane:** M11 / perf
- **Tasks:** T-0456 (this spec); implementation tasks in the Rollout

## Problem
In an aggregate over an element-wise expression of whole columns, every operand becomes a
1,048,576-row `Vec<Vec<ArrayNode>>`, and so does every intermediate result. For example, the CSE
formula `MIN(IF($D:$D=K2,$F:$F))` builds:
- the range (`Model::evaluate_range`, cast.rs);
- the comparison (`handle_comparison`, arithmetic.rs);
- IF's result (`if_array`, functions/logical/mod.rs);

and then walks them all (`walk_array_items`, functions/util/aggregate_args.rs). Nearly every row
lies past the sheet's used area, and is `Empty` in each operand.

Measured, release:
- **corpus-bench 59932.** Its 150 such formulas take about 55 s, about 0.37 s each (78 s through the
  harness; T-0430, T-0455).
- **corpus-bench 17049.** A sheet of 405 rows has 405 `MAX(IF(F:F=A1,G:G,""))` formulas. Load takes
  10 ms, but evaluation does not finish within 120 s. Every sample is in `if_pick` /
  `evaluate_range` under `fold_min_max`.

Both files are `!Timeout` lines in eval/baseline/corpus-bench.txt.

## Goals / non-goals
Goals:
- An aggregate whose argument is such an expression does work proportional to the used rows, plus
  one row for the rest.
- Results stay bit-identical: same value, same error and the same error precedence. The same cells
  are read and recorded, so the same restarts happen.
- Every other consumer of arrays keeps today's code.

Non-goals:
- A new general array representation for the 244 `CalcResult::Array` sites.
- A spill of a whole-column expression (`=D:D=K2` in a cell). It still materializes, and its size
  checks are unchanged.
- SUMPRODUCT, the criteria functions and lookups. SUMPRODUCT is a later consumer (Rollout); the
  others already clip to the used area or index (T-0448, T-0449).

## Excel behavior to match
No behavior changes. The invariants below must keep today's results, which are already
oracle-backed:
- The array bounds: "Array results above 1,048,576 cells are #ERROR!" and the VSTACK/EXPAND row caps
  (docs/excel-semantics/arrays.md). These depend only on the dimensions, so they apply to the logical
  size (head plus tail).
- Element-wise comparison, arithmetic and IF over arrays, including broadcasting of 1-row and
  1-column operands and `#N/A` for out-of-range positions of ragged operands (arrays.md; probes in
  test_fn_if_array.rs).
- The aggregates' item rules and the rank of an array error against a referenced error
  (`AggregateWalk`, `ErrorRule::Ranked`; probes/aggregate-array-error-vs-ref-error-*).
- Seen-empty records: a whole-column read records the cells past the used area as read empty, in
  one rectangle (`clip_to_used_area`, used_area.rs; docs/excel-semantics/references.md). A spill
  that later writes there restarts the pass.

## Design
### Representation
A crate-private type in a new top-level engine module `tailed_array.rs`, next to `arithmetic.rs`
and `cast.rs`. It holds expression evaluation (range reads, operators, IF), so it does not belong in
`functions/util`. Only the consumer lives in `functions/util/aggregate_args.rs`:

```rust
/// An array of `rows()` x `width` elements: `head`, then `tail_len` copies of `tail`.
pub(crate) struct Tailed {
    head: Vec<Vec<ArrayNode>>,   // each row `width` long
    tail: Vec<ArrayNode>,        // one row, `width` long
    tail_len: usize,
}
```

- `rows() = head.len() + tail_len`.
- `elements()` yields the row-major sequence that the materialized array would yield, without
  building it: the head rows, then `tail` repeated `tail_len` times.
- `into_array()` materializes it, for any consumer outside this spec.

### Which expressions are tailed (static check, before any evaluation)
`Model::tailed_shape(node, cell) -> Option<TailedShape>` decides from the AST alone. The decision
must come before evaluation: a fallback after a partial evaluation would read cells twice, and
would spend `copy_read_once` and budgets twice.

A node is tailed when all of the following hold:
- **Leaves.** Every leaf is one of:
  - a literal;
  - a one-cell reference, as a scalar;
  - a range reference on one sheet, with no defined name, structured or 3-D reference, INDIRECT or
    OFFSET.
- **Operators.** Every interior node is one of:
  - a comparison, arithmetic or concatenation operator;
  - a unary `-` or `+`, or the percent operator;
  - a parenthesized group;
  - IF with 2 or 3 arguments.
- **Row span.** Every range leaf with more than one row has the same top row and the same height,
  and at least one does. One-row leaves broadcast as today.
- **Array-producing.** Today's evaluation must already produce an array, so the new path changes
  only how it is held:
  - The argument's top node is an operator or an IF, never a bare range or a parenthesized range
    (`SUM(F:F)`, `SUM((F:F))`). Those are references, walked by `walk_areas` under the
    referenced-cell rules.
  - Every IF's condition contains a multi-row range leaf, so `if_pick` takes the array path
    (`if_array`). An IF whose condition is scalar (`MIN(IF($K$1,F:F,G:G))`) picks and evaluates one
    branch as a reference, and is rejected along with the whole argument.
  - A bare range may still be an operand, or an IF branch, which `evaluate_if_arg` reads as an
    array today.
- **Gain.** At least `MIN_TAILED_ROWS` rows (4,096) of the common span lie past the used area of
  every range leaf's sheet. `used_area_corner` reads this without recording anything. Otherwise
  today's path costs no more.

Functions are added to the list one by one (ISNUMBER, ISBLANK, ISERROR, NOT, LEN, IFERROR and so
on). Each is added only with an argument that the cells past the used area read the same in every
row: no ROW, COLUMN, SEQUENCE, RAND, INDEX, OFFSET or other position- or call-dependent function.

### Evaluation
`Model::evaluate_tailed(node, cell) -> Result<Tailed, CalcResult>` mirrors today's evaluation
node for node.
- **A range leaf** reads its block through the same `clip_to_used_area` call as `evaluate_range`.
  That means the same `check_range_size`, the same per-cell `evaluate_cell` reads in the same
  row-major order, and the same seen-empty rectangle for the rest. The rows up to the common head
  length `h` are the head; `tail` is `Empty` and `tail_len = height - h`. Here `h` is the largest
  used extent over all the range leaves of the argument, IF branches included, so every leaf is cut
  at the same row. Because the static check requires one common top row and height, every
  intermediate result has the same `h` too.
- **The head length comes from the clip at evaluation time.** `tailed_shape` reads the used area
  only to estimate the gain. If a spill or restart changes the used area in between, the estimate
  is stale, but the result is still right.
- **A scalar leaf** is evaluated once, as today.
- **Operators and IF** are applied to the head element by element, and once to the tail row. The
  per-element kernels are the existing ones:
  - comparison: the `apply` / `compare_values` closure of `handle_comparison`;
  - arithmetic: the existing element kernels in arithmetic.rs;
  - IF: `array_node_to_bool` and `if_arg_at`.

  They are moved into free functions that both paths call. They are not copied (directive 3).
- **Broadcast and size checks** use the logical dimensions, through `check_range_size` and
  `check_array_size`, before any tail is walked. Errors are therefore unchanged, and the elements
  walked (`width * rows()`) stay within 2^20, a wide `A:XFD` leaf included.
- **IF**, over its array condition, evaluates both branches once after the condition, and calls
  `variable_stack.copy_read_once()` first, as `if_array` does.
- **An error that today returns the whole result** (a scalar error operand of an array operation,
  `array_of_errors`) gives the same error array, as a `Tailed` whose tail is that error.

### Consumer: the aggregate walk
`walk_aggregate_arg` (aggregate_args.rs) is the single entry point. When `tailed_shape(arg)` holds,
it evaluates the argument with `evaluate_tailed` and walks `elements()` through the existing
per-element body. Otherwise it reads the argument as today.
- That per-element body comes from today's `walk_array_items`. It changes to take an iterator of
  `ArrayNode`, so both callers share it.
- The visitor therefore receives exactly the `ItemSource::ArrayElement` items it receives today, in
  the same order. Sums round alike, and the first held error is the same.
- This covers all nine users of `for_each_aggregate_number`: SUM, PRODUCT, MIN, MAX, AVERAGE,
  COUNT, the variance family and so on.

`tail_len` repeats can be skipped at once when the tail element is ignored under the walk's rules:
- `Empty`;
- or text or a boolean, when `rules.array` ignores it;

and no error is pending. The skip asks the visitor itself: the walk visits the tail element once,
and skips the remaining repeats only if that visit changed nothing. The visitor reports this from
the same `array_item` and `aggregate_number` decision it already makes, so the ignore rules are
not copied. Counting consumers (COUNTA, the A-variants) count it `tail_len` times. A
numeric tail is visited `tail_len` times, so float summation is unchanged; 2^20 adds take about a
millisecond.

### Limits (directive 5)
- `tailed_shape` and `evaluate_tailed` recurse over the same AST that today's evaluator recurses
  over, and go no deeper. They run under the evaluator's existing depth and stack guards, and a
  deep-nesting test is part of T-0469.
- No new allocation beyond the head, which is at most the used area: the same bound as today's
  clipped reads.
- The tail is one row.
- The static check is linear in the formula's AST.

## Alternatives considered
- **A general `Tailed` variant of `CalcResult::Array`.** Every one of the 244 match sites would have
  to handle or materialize it. Too wide for one change, and every missed site risks a panic or a
  wrong result.
- **Clipping operands to the used area and padding at the end.** This changes the array's
  dimensions, which ROWS, the bounds and broadcasting observe.
- **Evaluating each formula once per distinct scalar input** (cache per `K2`). This does not help
  17049, where every scalar differs.

## Test plan
- **Equivalence tests** in `test/test_tailed_array.rs`:
  - For every operator, IF form, leaf kind and aggregate, and with errors in the head, in a scalar
    and in a branch, the aggregate over the tailed expression equals the same aggregate over the
    expression materialized.
  - The materialized form is forced through a range that ends inside the used area, plus an explicit
    tail block.
  - The equivalence covers value, error kind and the restart on a spill into the tail.
- **Static-check tests** for each exclusion: names, INDIRECT, mixed heights, ROW, a bare range or
  parenthesized range argument, and an IF with a scalar condition. Equivalence tests for
  `SUM(F:F)` and `MIN(IF($K$1,F:F,G:G))` show they stay on today's path.
- **A deep-nesting test**, and a multi-column whole-column leaf over the cell limit.
- **Eval:** `rustsheet-eval check` with 0 regressions in every gated suite.
- **Measures:** corpus-bench 59932 and 17049 under 60 s through the harness (their `!Timeout` lines
  leave the baseline). A bench `aggregate -- whole_column`: 400 `MAX(IF(F:F=A1,G:G,""))` over a
  2,000-row sheet.

## Rollout
1. **T-0469 (M).** `Tailed`, `tailed_shape` and `evaluate_tailed` for leaves, operators and IF;
   the shared element kernels; the aggregate walk consumer; tests and bench. Measure: 59932 and
   17049.
2. **T-0470 (S).** The element-wise functions that corpus formulas use inside such aggregates, one
   list from a scan of the fetched suites (ISNUMBER, ISBLANK, ISERROR, NOT, LEN, IFERROR, …).
3. **T-0471 (S).** SUMPRODUCT as a second consumer (`SUMPRODUCT((A:A=x)*B:B)`), with the same
   element sequence per argument.

Until a task lands, its expressions take today's path unchanged.
