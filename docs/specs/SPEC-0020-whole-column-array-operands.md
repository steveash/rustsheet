# SPEC-0020 — Whole-column array operands: a used head and one repeated tail row

- **Status:** accepted
- **Milestone / lane:** M11 / perf
- **Tasks:** T-0456 (this spec); T-0469, T-0470, T-0471 (Rollout)

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
A new top-level engine module, `tailed_array.rs`, next to `arithmetic.rs` and `cast.rs`. Only the
consumer lives in `functions/util/aggregate_args.rs`.

```rust
/// An aggregate argument read as a head and a tail: the head's elements, then `tail_len`
/// copies of the tail's one element (no tail when the pass is being abandoned).
pub(crate) struct Tailed<T> {
    pub(crate) head: T,          // the walk's read of the argument (a one-column array)
    pub(crate) tail: Option<T>,  // a one-element array
    pub(crate) tail_len: usize,
}
```

Revised by T-0469: the head and the tail are two ordinary evaluations of the argument (see
Evaluation). So the elements are never materialized past the head, and no new array type reaches
any other consumer.

### Which expressions are tailed (static check, before any evaluation)
`Model::tailed_leaves(node, cell)` decides from the AST alone, before anything is evaluated. It
walks the AST with an explicit stack, so operator chains of any length cost no recursion.

A node is tailed when all of the following hold:
- **Leaves.** Every leaf is one of:
  - a literal (`NumberKind`, `BooleanKind`, `StringKind`);
  - a one-cell reference (`ReferenceKind`), as a scalar;
  - a single whole column (`RangeKind` with rows 1 to 1,048,576 and one column) on one sheet.

  These are excluded: every other node kind (`WrongRangeKind` and `WrongReferenceKind` for
  external workbooks, `SheetSpanKind`, `ThisRowKind`, `SpillRangeOperator`,
  `ImplicitIntersection`, `DefinedNameKind`, structured references), and every function call
  except IF, which rules out INDIRECT and OFFSET.

  T-0469 takes single columns only. A 1,048,576-row array of one column is exactly the array
  bound, so no size check differs between the head and the full array. Multi-column and partial
  ranges keep today's path.
- **Operators.** Every interior node is one of:
  - a comparison, arithmetic or concatenation operator;
  - a unary `-` or the percent operator (the AST has no unary plus: `+F:F` parses as the bare
    reference, which the top-node rule rejects);
  - a parenthesized group;
  - IF with 2 or 3 arguments.
- **Array-producing.** Today's evaluation must already produce an array, so the new path changes
  only how it is held:
  - The argument's top node is an operator or an IF, never a bare range or a parenthesized range
    (`SUM(F:F)`, `SUM((F:F))`). Those are references, walked by `walk_areas` under the
    referenced-cell rules.
  - Every IF's condition contains a whole-column leaf, so `if_pick` takes the array path
    (`if_array`). An IF whose condition is scalar (`MIN(IF($K$1,F:F,G:G))`) picks and evaluates one
    branch as a reference, and is rejected along with the whole argument.
  - A bare range may still be an operand, or an IF branch, which `evaluate_if_arg` reads as an
    array today.
- **Gain.** At least `MIN_TAILED_ROWS` rows (4,096) lie past the last used row over the leaves'
  sheets (`used_area_end_of`, which reads and records nothing).
- **Not under iterative calculation, Excel-order passes or a blocked read.** In those modes a
  second read of a cell is not the same as the first.

### Evaluation
Revised by T-0469: `Model::read_tailed(arg, cell, read)` runs the walk's own reading of the
argument (`evaluate_aggregate_arg` or `evaluate_picked_reference_arg`) twice, through the ordinary
evaluator. So every operator, IF, broadcast, size check, error path (`array_of_errors`, scalar
errors) and `copy_read_once` is today's code, and nothing is copied.

While a read runs, a `TailView` names the argument's whole-column leaves by node address, and the
formula's cell, since formulas evaluated meanwhile may share parsed nodes. `evaluate_node_in_context`
asks `tailed_operand` about each such leaf.
- **Head.** The head length `h` is the last used row over the leaves' sheets, at least 1. Each
  leaf first makes the `clip_to_used_area` call that `evaluate_range` makes for a whole column,
  which records the rows past its used area as read empty. It then evaluates as the array of its
  rows 1 to `h`, which `evaluate_range` reads cell by cell.
- **Tail.** Each leaf evaluates as the one-element array `{Empty}`: the value of every row past
  `h`, where every leaf is empty and every scalar is what it is in every row. The tail reads the
  one-cell references again. An evaluated cell is read as before, and a read that closes a cycle
  repeats marks that are already set, so neither changes the outcome.
- **The used area grew while the head was evaluated** (a spill evaluated on the way, past `h`).
  The head is then too short. The argument is read once more the ordinary way, its cells already
  evaluated. The same happens if the head is not one column or the tail not one element. That
  second read records each column's seen-empty rectangle again, against the formula's seen-range
  budget (docs/excel-semantics/references.md "Seen-range cap"). A formula already near that cap
  could reach it one rectangle sooner. This is a directive 5 limit, not a value Excel shows, and
  needs a spill to grow the used area in the middle of the formula.
- **The pass is being abandoned** (a restart or an unwind after the head). The tail is skipped,
  since the value is discarded.

### Consumer: the aggregate walk
`walk_aggregate_arg` (aggregate_args.rs) is the single entry point. When `read_tailed` gives a
head and a tail, it walks the head's elements, then the tail element `tail_len` times, through
the existing per-element body. Otherwise it reads the argument as today.
- That per-element body comes from today's `walk_array_items`, now `walk_array_nodes` over an
  iterator of `ArrayNode`, so both callers share it.
- The visitor therefore receives exactly the `ItemSource::ArrayElement` items it receives today, in
  the same order. Sums round alike, and the first held error is the same.
- This covers every consumer of the walk: SUM, PRODUCT, MIN, MAX, AVERAGE, COUNT, COUNTA, AND,
  OR, the variance family and so on.

By default the walk visits the tail element `tail_len` times, exactly as it visits the
materialized array. COUNTA counts every `ArrayElement`, `Empty` included (probes/counta-empty-value
D18), and AND, OR, XOR and COUNT also use this walk, so no skip is assumed.

Only a walk whose visitor folds numbers skips the repeats.
- `for_each_aggregate_number` gives its walk its `rules.array` (`AggregateWalk::numbers`).
- `AggregateWalk::ignores(element)` is true for `Empty`, a boolean, and text under
  `ArrayNumbers::Numbers`, and only while no error is held. Those are exactly the elements for
  which `aggregate_number(rules, ItemSource::ArrayElement, _)` is `Ok(None)`, and visiting such
  an element calls nothing.
- Every other walk sets no rules. A numeric tail is always visited `tail_len` times, so float
  summation is unchanged; 2^20 adds take about a millisecond.

### Limits (directive 5)
- **Recursion.** The parser's depth cap does not count the links of a value-operator chain. Such
  chains are left-nested (expressions/parser/chain.rs, T-0128), so `SUM(A:A+A:A+…)` with 4,000
  links is a tree 4,000 deep.
  - `tailed_leaves` walks the AST with an explicit stack, so it does not recurse at all. The two
    evaluations are the ordinary evaluator's, with its chain loop (`evaluate_operator_chain`) and
    its depth and stack guards.
  - A 4,000-link chain test is part of T-0469.
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
- **Equivalence tests** in `test/test_tailed_array.rs` (as built: each formula evaluated with the
  tailed reading on and with it off, through a test-only switch, plus a count of tailed walks):
  - For every operator, IF form, leaf kind and aggregate, and with errors in the head, in a scalar
    and in a branch, the aggregate over the tailed expression equals the same aggregate over the
    expression materialized.
  - The materialized form is forced through a range that ends inside the used area, plus an explicit
    tail block.
  - The equivalence covers value, error kind and the restart on a spill into the tail.
- **Static-check tests** for each exclusion: names, INDIRECT, mixed heights, ROW, a bare range or
  parenthesized range argument, and an IF with a scalar condition. Equivalence tests for
  `SUM(F:F)` and `MIN(IF($K$1,F:F,G:G))` show they stay on today's path.
- **A deep-nesting test, a long-chain test**, and a multi-column whole-column leaf over the cell
  limit.
- **Equivalence for each consumer of the walk:** SUM, AVERAGE, MIN, MAX, COUNT, COUNTA, the
  A-variants, AND and OR, the variance family. It covers:
  - an `Empty` tail produced through an IF branch (`COUNTA(IF(D:D="",F:F))`);
  - the ranked-error probes (probes/aggregate-array-error-vs-ref-error-*) with a whole-column
    operand;
  - a leaf on another sheet whose used extent is shorter than the common head length `h`, padded
    with `Empty`;
  - error tails built by `array_of_errors` from scalar error operands.
- **Eval:** `rustsheet-eval check` with 0 regressions in every gated suite.
- **Measures:** corpus-bench 59932 and 17049 under 60 s through the harness (their `!Timeout` lines
  leave the baseline). A bench `aggregate -- whole_column`: 400 `MAX(IF(F:F=A1,G:G,""))` over a
  2,000-row sheet.

## Rollout
1. **T-0469 (M).** `Tailed`, `tailed_leaves` and `read_tailed` for single whole columns, operators
   and IF; the operand hook in `evaluate_node_in_context`; the aggregate walk consumer; tests and
   bench. Measure: 59932 and 17049.
2. **T-0470 (S).** The element-wise functions that corpus formulas use inside such aggregates, one
   list from a scan of the fetched suites (ISNUMBER, ISBLANK, ISERROR, NOT, LEN, IFERROR, …).
   Dropped: the scan found no element-wise function inside a whole-column aggregate (T-0470
   Outcome), so these functions keep today's path.
3. **T-0471 (S).** SUMPRODUCT as a second consumer (`SUMPRODUCT((A:A=x)*B:B)`), with the same
   element sequence per argument.

Until a task lands, its expressions take today's path unchanged.
