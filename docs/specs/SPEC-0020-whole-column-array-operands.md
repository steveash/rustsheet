# SPEC-0020 — Whole-column array operands: a used head and one repeated tail row

- **Status:** accepted
- **Milestone / lane:** M11 / perf
- **Tasks:** T-0456 (this spec); T-0469, T-0470, T-0471 (Rollout); T-0488 (Amendment 1); T-0491,
  T-0492, T-0493, T-0494 (its Rollout)

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

## Amendment 1 (T-0488): UNIQUE, LOOKUP, SMALL and LARGE; bounded ranges; ROW()
Status: accepted with the spec. It widens what is tailed and adds consumers, and supersedes
three things above for what it lists. The non-goals "every other consumer keeps today's code"
and "lookups" no longer cover LOOKUP's vector form, UNIQUE, SMALL and LARGE. "Which expressions
are tailed" now also takes bounded single-column spans and `ROW(<leaf range>)`. `Tailed` covers
such a span. For the expressions T-0469 already takes, nothing changes.

### Problem
Gap triage 2026-10-27 found the three slowest corpus-bench cases, all passing, outside the
aggregate walk. Each formula still builds 1,048,576-row arrays, or 69,999-row ones in 59969:
- **45896** (17.6 s): 27 `TEXTJOIN(",",TRUE,UNIQUE(IF(A2=ZORD!A:A,ZORD!C:C,"")))` over a 75-row
  sheet. UNIQUE consumes the whole-column IF. TEXTJOIN gets UNIQUE's short result, so TEXTJOIN
  need not be tailed for this file.
- **33935** (8.6 s): 29 `IF(L3>1,LOOKUP(2,1/(J:J<>""),J:J))`, the last-non-empty idiom. LOOKUP's
  approximate search consumes the whole-column quotient.
- **59969** (5.7 s): `IFERROR(INDEX(Report!G:G,SMALL(IF((Report!$A$2:$A$70000=Form!$B$3)*
  (Report!$B$2:$B$70000=Form!$B$5),ROW(Report!$A$2:$A$70000)),ROWS($A$1:A1))),"")`. Its ranges
  are bounded, they run far past the used area, and ROW() is a position-dependent operand. SMALL is
  the consumer.

The bench `aggregate -- tailed_consumers` (T-0488) reproduces the three shapes over 2,000 rows.

### 1. Bounded ranges past the used area are tailed leaves
A range leaf may be any single-column `RangeKind` on one sheet, written with its rows `top` to
`bottom`, not only rows 1 to 1,048,576.
- **Common span.** Every range leaf of the argument has the same `top` and `bottom`. If they
  differ, the argument is refused. Today's evaluation broadcasts and pads ragged operands with
  `#N/A` (arrays.md), and one head length cannot express that.
- **Head and tail.** The head ends at `H = clamp(used_end, top, bottom)`, where `used_end` is the
  last used row over the leaves' sheets (as before). Each leaf evaluates as rows `top..=H`, so a
  used area that ends above `top` still gives a one-row head. The tail stands for
  `tail_len = bottom - H` rows, and the gain rule becomes `tail_len >= MIN_TAILED_ROWS`.
- **Records.** `clip_to_used_area` cuts only whole columns and rows. For a bounded range,
  `evaluate_range` reads every cell past the used area one by one, each on record as read empty.
  So `tailed_operand` calls `clip_block_to_used_area` instead (T-0491). It records the same cells
  as one rectangle, the record T-0405's block walks make, and makes the same cut as
  `clip_to_used_area` for a whole column. When the formula's seen-range budget cannot pay for the
  rectangle, the block is not cut: the head is then stale, and the argument is read the ordinary
  way.
- **Size checks.** They apply to the logical size `bottom - top + 1`. For one column that is never
  above the array bound, so as before no size check differs.
- **Whole columns.** A whole column is the case `top = 1`, `bottom = 1,048,576`. A formula that
  mixes `F:F` with `F2:F70000` has two spans and is refused.

### 2. ROW() over a leaf range: a position leaf with a sentinel tail
`ROW(r)` is position-dependent, so its tail row is not one value. Within the argument's
expression it is an arithmetic sequence. Only one thing matters: whether a tail row's
*result* depends on that sequence. In 59969 it does not, because the IF's condition is 0 in
every tail row and the IF picks FALSE.

The sentinel rule:
- **Static check.** A `FunctionKind { kind: Row, args: [r] }` node is a **position leaf** when `r`
  is a range leaf node with the argument's common span. The leaf is the ROW node. `r`'s sheet joins
  the leaves' sheets. ROW with no argument, or over anything else, keeps the refusal. COLUMN is not
  taken: no corpus formula needs it.
- **Head.** The position leaf evaluates as the column `{top; top+1; …; H}`, which is ROW of the
  head range. ROW reads no cell, so there is nothing to record.
- **Tail.** The position leaf evaluates as the one-element array `{#N/IMPL}` (`Error::NIMPL`), the
  sentinel.
- **Fallback.** If the tail element *is* `Error::NIMPL`, the argument has no tail row:
  `Tailed::elements` returns `None`, and the consumer reads the argument the ordinary way. Every
  consumer already does this for a shape it cannot take. A `Tailed` built with position leaves
  carries a flag saying the sentinel check applies.

Why a tail element other than the sentinel is the value of every tail row:
- The tailed expressions are value operators and IF (with ROW only as a leaf). Each output element
  is a function of the same-position operand elements.
- Each operator returns an error operand's error regardless of the other operand's value, the
  left one first (arithmetic.rs). IF returns its condition's error, and otherwise one branch.
- By induction, each subexpression is either independent of the ROW values or the sentinel.
  - An operator with a sentinel operand returns the sentinel, unless the other operand is an error
    that takes precedence. In that case the result is that error for every ROW value too.
  - An IF with a sentinel condition returns the sentinel. Otherwise the IF returns the branch it
    picks.
- A real `#N/IMPL` from a cell or scalar in the tail is mistaken for the sentinel. That only causes
  the fallback, which is today's path, so the collision is safe.
- The argument holds only for the value operators and IF. A function that turns an error into a
  value (IFERROR, ISERROR, ISNA, …) would hide the sentinel, so any task that adds a tailed
  function must prove the rule again for it, or refuse ROW leaves beneath it.

### 3. Consumers
Each consumer reads its argument with `read_tailed` and the reading it uses today, so the head and
the tail are that reading's two evaluations. When `Tailed::elements` returns `None`, the consumer
falls back as the aggregate walk does. The shape checks and the lift costs use the logical size,
`head + tail_len` rows.
- **UNIQUE** (`fn_unique`, `eval_to_array_keep_blank`). `unique_of` keys rows, keeps the first
  occurrence of each key, and with `exactly_once` keeps the keys counted once. For any `n >= 2`,
  the head followed by `n` copies of the tail row gives the same result as the head followed by
  exactly two copies, and `tail_len >= 4,096`. So the consumer runs `unique_of` unchanged on
  `head ++ [t, t]`. The value, the `#CALC!` of an empty result, and the order are identical by
  construction.
  - Tailed only when `by_col` and `exactly_once` are absent or literals; this is a static check.
    With `by_col` TRUE, one column is one key over 1,048,576 rows, and the result is the full
    column. The lifted flags (an array `by_col`) keep today's path.
  - The lift cost is `LiftCost::Cells` of the logical size, so the budget is charged as today.
- **LOOKUP, vector form** (`lookup_search`, `evaluate_node_with_reference`). This applies to
  `lookup_vector` with a `result_vector` present. In the array form the result is taken from the
  table itself, so that form keeps today's path.
  - `SearchData` is already a head followed by a virtual tail of empty cells (`len` past
    `values`). `approximate_search` never enters that tail, because it only stops on elements of
    the target's kind.
  - A tail element of no `value_kind` (an error, as `1/FALSE` gives in 33935, or an empty
    element) is skipped the same way for every target. The search vector is then
    `values = head`, `len = head + tail_len`, and every position it finds is in the head.
  - A tail of a searched kind (number, text, boolean) keeps today's path. A run of equal keys at
    the end would need the binary search's midpoints over the run, and no corpus formula needs it.
  - The vector checks (`rows > 1`, the by-row rule) use the logical size. `result_vector` is read
    as today, after the search.
- **SMALL and LARGE** (`ranked_numbers`, through `numbers_from_reference_arg`). Today the
  argument's array is first checked by `nested_array_error`, which returns `#N/A` if any element is
  a nested array, whatever comes before it. Then `values_from_array` gives the first plain error in
  row-major order (a Lambda or nested element is `#VALUE!` there), or else the numbers. Any other
  element is no number. A tailed argument has literals, one-cell references, ranges, the value
  operators and IF, none of which can give a nested or Lambda element, so only the plain-error rule
  applies.
  - Tailed: the head's first error. Failing that, the tail's error if the tail is one. Failing
    that, the head's numbers, followed by `tail_len` copies of the tail if it is a number.
  - A FALSE, text or empty tail adds nothing (59969). A numeric tail allocates `tail_len` numbers,
    at most 1,048,576 `f64`s, which is today's bound.
  - The RANK family (`numbers_from_rank_ref_arg`) keeps today's path. Its arrays become arrays of
    `#VALUE!`.
- **TEXTJOIN** (contract only; not filed). No corpus file needs it: 45896's TEXTJOIN consumes a
  short UNIQUE result. If a file does, the contract below holds. In each case the first visit of
  the tail element decides what every later copy does:
  - An empty-text or `Empty` tail with ignore_empty TRUE adds nothing.
  - With ignore_empty FALSE, an `Empty` tail adds `tail_len` delimiters through `push_empties`, the
    count used for blank runs. An empty-text tail also adds `tail_len` delimiters, by the same count.
  - Any other text, number or boolean tail is pushed copy by copy until the join is over the limit.
    That takes at most 32,768 pushes, since each copy adds at least one unit. Past the limit,
    nothing more changes.
  - An error tail is returned at its first copy, unless the join is over the limit once the
    delimiter before that copy is counted (`exceeds_with_separator`). In that case the error is
    dropped, as `push_cell` drops it (probes/text-result-limit-errors,
    probes/textjoin-array-limit-empty-strings).

### 4. Invariants per consumer
- **Values and errors.** Each consumer sees its argument's element sequence through the same
  reading and kernel as today: `unique_of`, `approximate_search`, `nested_array_error` with
  `values_from_array`, or the TEXTJOIN walk. The value, the error kind and the error precedence are
  the materialized path's, so later arguments are evaluated or skipped exactly as today.
- **Reads and records.** These are the T-0469 rules: the same clip records, the one-cell
  references read again in the tail, the stale-head fallback, and no tail when the pass is
  abandoned. A bounded leaf records the rectangle its own `evaluate_range` records. A position leaf
  reads nothing.
- **Size checks.** The `#ERROR!` bound above 1,048,576 cells and the VSTACK/EXPAND caps
  (arrays.md) apply to the logical size, as do the lift costs. One column never exceeds the bound,
  so no check changes outcome. The cell budget (SPEC-0024) is charged for the cells
  `evaluate_range` builds, which is the head, as in T-0469.

### Test plan (per Rollout task)
- **Equivalence tests** in `test/test_tailed_array.rs`, each formula with `TAILED_OFF` and without,
  as T-0469 does, and with a count of tailed reads:
  - bounded spans that start above, inside and below the used area;
  - mixed spans (refused);
  - a ROW tail picked (fallback) and not picked (tailed);
  - a real `#N/IMPL` in the tail (fallback);
  - UNIQUE with `exactly_once` TRUE and FALSE, a tail equal to a head value, an all-tail result,
    and by_col (refused);
  - LOOKUP with an error tail, an empty tail and a numeric tail (refused), and the array form
    (refused), with a target whose kind differs from the head's last element, so the search's
    midpoints are exercised;
  - SMALL and LARGE with an error in the head and in the tail, and with numeric, FALSE and empty
    tails.
- **Eval.** `rustsheet-eval check`, 0 regressions in every gated suite.
- **Measures.** The corpus-bench file each task names, under 1 s through the harness, and the
  matching `aggregate -- tailed_consumers` line.

### Rollout
1. **T-0491 (M).** Bounded range leaves and ROW position leaves in `tailed_leaves`, `read_tailed` and
   `tailed_operand`, with the sentinel check in `Tailed::elements`, for the existing consumers
   (aggregate walk, SUMPRODUCT). Measure: the equivalence tests; the speed-up is measured by T-0492.
2. **T-0492 (S, after T-0491).** SMALL and LARGE as consumers. Measure: corpus-bench 59969 under 1 s,
   and the SMALL bench line.
3. **T-0493 (S).** UNIQUE as a consumer. Measure: corpus-bench 45896 under 1 s, and the
   TEXTJOIN/UNIQUE bench line.
4. **T-0494 (S).** LOOKUP's vector-form search over a tailed `lookup_vector` with a skipped-kind tail.
   Measure: corpus-bench 33935 under 1 s, and the LOOKUP bench line.

TEXTJOIN, COLUMN, LOOKUP's searched-kind tail and UNIQUE's lifted flags are not filed. Each is
filed when a corpus file needs it. Until a task lands, its expressions take today's path unchanged.
