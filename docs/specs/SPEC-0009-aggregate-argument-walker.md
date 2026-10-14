# SPEC-0009 — One aggregate-argument walker

- **Status:** accepted (on merge of T-1473)
- **Milestone / lane:** M2 / engine (second step: quality)
- **Tasks:** T-1473 (this spec), T-1474 (walker, SUM, AVERAGE family), T-1476 (MIN, MAX, PRODUCT, COUNT, COUNTA)

## Problem
M2's exit asks for "one implementation of aggregate-argument iteration, used by every consumer"
(docs/milestones.md; docs/architecture/overview.md design rule 2). The *range* walks are already
shared (`for_each_cell_or_error_in_areas`, `for_each_number_in_areas` and
`for_each_coerced_number_in_areas` in functions/util.rs, T-0200/T-0204). The step above them is
not. Each of these reads every argument, sorts the result into Number, Boolean, String,
EmptyArg, Array, Ranges or Error, and applies its own error precedence:

| Dispatch | File | Consumers | Own `reference_error` |
|---|---|---|---|
| `Model::for_each_value` | statistical/count_and_average.rs | AVERAGE, GEOMEAN, HARMEAN, MEDIAN, SKEW, SKEW.P, KURT, VAR.S/VAR.P, STDEV.S/STDEV.P (`variance_of_values`) | yes |
| `Model::for_each_value_skip_empty` | same | DEVSQ, AVEDEV, SUMSQ | (calls `for_each_value` one argument at a time) |
| `Model::for_each_value_a` | same | AVERAGEA, MAXA, MINA | no |
| `Model::sum_with_last_addition` | math_and_trigonometry/sum.rs | SUM | yes |
| `Model::fold_min_max` | math_and_trigonometry/mathematical.rs | MIN, MAX | yes |
| `Model::fn_product` | math_and_trigonometry/product.rs | PRODUCT | yes |
| `Model::fn_count`, `Model::fn_counta` | statistical/count.rs | COUNT, COUNTA | no (errors are items) |

That is seven argument dispatches and four copies of the `let mut reference_error = None`
precedence logic. They differ in small, partly evidenced ways (below). Each new probe finding has
to be applied in several places. T-1379, for example, had to change `for_each_value` after
T-1334 had fixed SUM, PRODUCT, MIN and MAX.

## Goals / non-goals
Goals:
- One walker owns the per-argument dispatch: reading an argument, sorting it by shape, walking
  ranges and arrays, and applying the error-precedence rule. Each consumer states its
  differences as a policy value, not as code.
- One number layer on top of it, for the consumers that fold numbers (all of the above except
  COUNT and COUNTA).
- No result changes. T-1474 and T-1476 are refactors with 0 regressions and 0 fixes in every
  suite. Behavior that differs between consumers without evidence stays different. The policy
  names the difference, and a follow-up probe can settle it.

Non-goals:
- The *IF(S) criteria walks (T-1477), the D-functions, SUBTOTAL/AGGREGATE (their hidden-row and
  nested-subtotal skips are position based), SUMPRODUCT and the two-array functions.
- MODE, LARGE/SMALL, PERCENTILE/QUARTILE, RANK and the other functions that collect a data
  array with their own rules (`collect_mode_values` etc.). They can move later if their rules
  fit the policy. This spec does not require it.
- VARA/VARPA/STDEVA/STDEVPA (`variance_of_values_a`). They are out of T-1474's and T-1476's
  scope and move onto the A-rules below in **T-1567**. Until then they are the one remaining
  dispatch, and they hold no `reference_error` of their own.
- Settling T-0236 (empty arguments of DEVSQ/AVEDEV/SUMSQ) or T-1439 (array and referenced error
  order for the `for_each_value` family). The walker has a knob for each. The probes set it.

## Excel behavior to match
Shared rules, all VERIFIED:
- **Shapes.** A range, a reference (also one that IF, CHOOSE, INDEX, OFFSET, LET or a picking
  call such as XLOOKUP, TAKE, DROP or TRIMRANGE gives back), a union and a 3-D span give their
  cells. An array gives its elements. Anything else is a direct value.
  Evidence: docs/excel-semantics/references.md; probes/subtotal-reference-operands-b, -c, -d
  (`evaluate_aggregate_arg`, `Function::picks_reference`); probes/multiarea-ifchoose,
  multiarea-let, union-defined-name.
- **Coercion by source** (coercion.md "MIN / MAX coerce value arguments like SUM" and "AVERAGE,
  MEDIAN, GEOMEAN, …, COUNT value arguments"; probes/min-max-value-args C1–C16,
  probes/average-value-args F1–F32):
  - A referenced cell gives only its number. Text, booleans and blanks there are skipped.
  - An array element gives only its number. Booleans and text there are skipped.
  - A direct value is coerced: numeric text is its number, other text is `#VALUE!`, TRUE/FALSE
    is 1/0, and an empty argument is 0.
  - For the number-folding consumers, direct text over 255 characters is `#N/A`
    (`aggregate_text_too_long`; probes/sum-long-text-args, long-text-args-min-criteria;
    numbers.md). COUNT does not count such text, and it is not an error there
    (probes/text-number-long-paths, numbers.md). COUNTA counts it like any text.
- **Error precedence** (syntax.md "Aggregate error precedence"; probes/aggregate-error-precedence
  C1–C77, probes/sum-error-precedence):
  - A *direct* error beats every referenced-cell error, whichever comes first. A direct error is
    an error literal, a computed scalar such as `1/0` or `A2*1`, or uncoercible direct text.
  - Among direct errors, the leftmost wins.
  - Among referenced-cell errors, the leftmost wins.
  - SUM only: an error inside an array ranks with the referenced errors (C50, C53–C55, C61, C77).
    For PRODUCT, MIN, MAX and AVERAGE today the first array error is returned at once. This is
    UNVERIFIED (T-1439).

Per-consumer differences that must be kept. Each has its evidence or is marked as current
behavior with no evidence:

| Consumer | Difference | Evidence |
|---|---|---|
| SUM | tracks its last addition for near-zero snapping (`SumParts`) | numbers.md "Near-zero snapping", probes/near-zero-sum-forms |
| SUM, MIN, MAX, PRODUCT | every direct value goes through `cast_aggregate_arg_to_number`, so a direct blank value is 0 and a direct LAMBDA or range value is an error | current behavior; unprobed |
| `for_each_value` family, COUNT | only a direct number, boolean, text or empty argument counts; other direct values are skipped | current behavior; unprobed |
| `for_each_value` family, COUNT | a value that a reference node gives without areas (a cached cell of a linked workbook) is read like a referenced cell, and its text and booleans are skipped | current behavior (the `ReferenceKind` guards) |
| DEVSQ, AVEDEV, SUMSQ | an empty argument is skipped, and each argument's first error is returned at once | current behavior, TODO(T-0236); T-1439 |
| AVERAGEA, MAXA, MINA | referenced cells and single-cell references coerce (text 0, TRUE/FALSE 1/0), array text is 0 for AVERAGEA and skipped by MAXA/MINA, array booleans are skipped; a cached range of a linked workbook is read through `external_reference_cells`; the first error is returned at once | coercion.md "MAXA / MINA"; statistics.md "AVERAGEA, MAXA and MINA over an array" (probes/averagea-array-constant-elements); fixtures/external-link-cached-ranges Refs!B53; error order unprobed |
| MIN, MAX | 0 when no number was seen | probes/min-max-value-args |
| PRODUCT | 0 when nothing was seen (an empty argument counts as seen); reads with `evaluate_reference_arg`, so XLOOKUP, TAKE, DROP and TRIMRANGE are values, and cached blanks of a linked workbook are kept | `seen_value`: current behavior; reading: current behavior, unprobed |
| COUNT | counts types: a number, a direct boolean, an empty argument, direct text that `cast_number` reads as a finite number (no `#N/A` for text over 255 characters, which `cast_number` does not read), a referenced number, an array number. Errors are ignored | probes/average-value-args F16, F32; probes/count-direct-text; probes/lookup-pick-aggregates D4, D9 (direct boolean); probes/text-number-long-paths; syntax.md (COUNT ignores errors) |
| COUNTA | counts every direct value but a blank one, every non-empty referenced cell (errors included), every array element (empty values included); reads with `evaluate_picked_reference_arg` (cached blanks kept) after its `external_reference` count | logic.md "IFERROR / IFNA of a blank cell is an empty value"; probes/counta-empty-value D18; probes/external-ref-no-link-shapes, -index, -ifchoose |

No new probe is needed to write this spec: every rule above is evidenced or kept as it is. The
unprobed rows stay as they are, and **T-1439** plus the new **T-1566** (below) probe them.

## Design
New module `crates/rustsheet-engine/src/functions/aggregate_args.rs` (engine lane). The
item-level API is `pub(crate)`, and nothing leaves the engine crate.

### Items
```rust
/// Where an item the walker hands on came from; the coercion rules depend on it.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ItemSource {
    /// A direct value: a literal, a computed scalar, a function's value result.
    Direct,
    /// A value a reference node gave without areas (a cached cell of a linked workbook).
    ReferenceValue,
    /// A cell of a range, reference, union or 3-D span.
    ReferencedCell,
    /// An element of an array value.
    ArrayElement,
}
```
An item's value is a `CalcResult`. An array element is converted as
`array_node_to_calc_result` (functions/util.rs) converts it, but by value, so text is moved, not
cloned. Every element is handed on, `ArrayNode::Empty` (as `EmptyCell`) too, so COUNTA can count
it. Referenced cells that are blank (`EmptyCell`, `EmptyArg`) are never handed on, because no
consumer reads them.

### Policy
```rust
pub(crate) struct AggregatePolicy {
    pub(crate) reading: ArgReading,
    pub(crate) errors: ErrorRule,
    /// DEVSQ, AVEDEV, SUMSQ: an empty argument is not an item (TODO(T-0236)).
    pub(crate) skip_empty_args: bool,
}

pub(crate) enum ArgReading {
    /// `evaluate_aggregate_arg`: picked references, cached blanks skipped (SUM, AVERAGE, MIN, …).
    Aggregate,
    /// `evaluate_picked_reference_arg`: cached blanks kept (COUNTA).
    PickedKeepingBlanks,
    /// `evaluate_reference_arg`, a single cell as a one-cell range (PRODUCT).
    Reference,
}

pub(crate) enum ErrorRule {
    /// Direct errors return at once; referenced errors are held, the leftmost wins.
    Ranked { array_errors: ArrayErrorRank },
    /// The first error in argument order returns at once (A-variants, DEVSQ/AVEDEV/SUMSQ).
    First,
    /// Errors are items for the visitor (COUNT ignores them, COUNTA counts them).
    Items,
}

pub(crate) enum ArrayErrorRank {
    /// An array error is a direct error (today's PRODUCT, MIN, MAX, AVERAGE; T-1439).
    Direct,
    /// An array error ranks with referenced errors (SUM; C50, C53–C55, C61, C77).
    Referenced,
}
```
Each consumer has one `const` policy next to its function, for example
`const SUM_ARGS: AggregatePolicy = …`. A difference between consumers is visible in that one
line.

### Walker
```rust
/// Error-precedence state across one call's arguments.
pub(crate) struct AggregateWalk {
    policy: AggregatePolicy,
    held: Option<CalcResult>,
}

impl AggregateWalk {
    pub(crate) fn new(policy: AggregatePolicy) -> Self;
    /// The held referenced error, if any, once every argument is walked.
    pub(crate) fn finish(self) -> Result<(), CalcResult>;
}

impl Model {
    /// Walks every argument with `policy`, then `finish`.
    pub(crate) fn walk_aggregate_args<F>(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
        policy: AggregatePolicy,
        visit: F,
    ) -> Result<(), CalcResult>
    where
        F: FnMut(&mut Self, ItemSource, CalcResult) -> Result<(), CalcResult>;

    /// One argument, for a consumer with a step of its own before it (COUNTA's and the
    /// A-variants' linked-workbook cells).
    pub(crate) fn walk_aggregate_arg<F>(
        &mut self,
        arg: &Node,
        cell: CellReferenceIndex,
        walk: &mut AggregateWalk,
        visit: &mut F,
    ) -> Result<(), CalcResult>
    where
        F: FnMut(&mut Self, ItemSource, CalcResult) -> Result<(), CalcResult>;
}
```
Rules the walker enforces:
1. The argument is read as `policy.reading` says. `Node::EmptyArgKind` is skipped when
   `skip_empty_args` is set.
2. `RefArg::Ranges` → `for_each_cell_or_error_in_areas`. Under `Ranked` and `First`, the first
   error cell of an argument's areas ends the walk of those areas: the per-cell closure returns
   `Err`, as `for_each_cell_in_areas` does today, so the remaining cells are not evaluated. Under
   `Ranked`, a later argument's areas are not read at all once an error is held, as every copy
   does today. An `Err` from the area walk itself (the walk budget) ranks like an error cell.
3. Errors, under `Ranked` and `First`, never reach the visitor:
   - A direct error, and under `Ranked { array_errors: Direct }` or `First` an array error, is
     returned at once.
   - Under `First`, a referenced error is also returned at once.
   - Under `Ranked`, a referenced error, and an array error under `Referenced`, is held if
     nothing is held yet. Later ones are dropped. This keeps "the leftmost wins".
4. A visitor `Err`, such as uncoercible direct text or a too-long text, is a direct error and is
   returned at once whatever the source. These are always direct today. A visitor that errs on a
   referenced item (today only the A-rules' "Unexpected Range") says why in a comment on its
   consumer's `const` policy.
5. No panics, indexing or unchecked arithmetic on cell data (directive 5). Counts stay `f64` or
   saturating, as today.
6. Order: arguments left to right, cells in `for_each_cell_in_areas` order, array elements row
   by row. It is the order every copy uses today, and the order that "leftmost wins" and
   AVEDEV's two-pass sums depend on.

The walker does not build per-cell values beyond the one `CalcResult` that
`for_each_cell_or_error_in_areas` already makes, and it does not allocate per argument. The
visitor is a generic closure, so the hot loop (SUM over a large column) stays monomorphic.

### Number layer
```rust
pub(crate) struct NumberRules {
    pub(crate) direct: DirectNumbers,
    pub(crate) cells: CellNumbers,
    pub(crate) array: ArrayNumbers,
}
pub(crate) enum DirectNumbers {
    /// Every direct value through `cast_aggregate_arg_to_number` (SUM, MIN, MAX, PRODUCT).
    Cast,
    /// Number, boolean, text (`cast_aggregate_arg_to_number`) and an empty argument (0);
    /// other direct values are skipped (the `for_each_value` family).
    CastScalars,
    /// As `CastScalars`, but text that does not cast is `#VALUE!` via `cast_number` and an
    /// empty argument is skipped (AVERAGEA, MAXA, MINA, as `for_each_value_a` does today).
    CastA,
}
pub(crate) enum CellNumbers { Numbers, CoercedA }   // `ReferenceValue` items follow it, except under `Cast`
pub(crate) enum ArrayNumbers { Numbers, TextZero }  // array booleans are always skipped

impl Model {
    pub(crate) fn for_each_aggregate_number<F: FnMut(f64)>(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
        policy: AggregatePolicy,
        rules: NumberRules,
        f: F,
    ) -> Result<(), CalcResult>;
}
```
`for_each_value`, `for_each_value_skip_empty` and `for_each_value_a` become thin wrappers or
calls with a named policy, and are removed once their callers move. `for_each_number_in_areas`
and `for_each_coerced_number_in_areas` stay. Their other callers (NPV, the D-functions, …) are
outside this spec. "Numbers of a reference" must keep one definition. T-1474 extracts the
per-cell rules of these two helpers into two free functions in util.rs,
`cell_number(&CalcResult) -> Option<f64>` (numbers only) and
`coerced_cell_number(&CalcResult) -> Option<f64>` (text 0, TRUE/FALSE 1/0). The two helpers and
the walker's `CellNumbers::Numbers` / `CoercedA` arms all call them.

The walker and `for_each_aggregate_number` are `#[inline]` and take the policy by value. With a
`const` policy the per-cell branches on `ErrorRule` and `CellNumbers` fold away in each consumer.

### Consumer mapping
| Consumer | reading | errors | skip empty | number rules | own state kept |
|---|---|---|---|---|---|
| SUM | Aggregate | Ranked(Referenced) | no | Cast / Numbers / Numbers | `SumParts` |
| AVERAGE, GEOMEAN, HARMEAN, MEDIAN, SKEW, SKEW.P, KURT, VAR.*, STDEV.* | Aggregate | Ranked(Direct) | no | CastScalars / Numbers / Numbers | — |
| DEVSQ, AVEDEV, SUMSQ | Aggregate | First | yes | CastScalars / Numbers / Numbers | — |
| AVERAGEA, MAXA, MINA | Aggregate, after the `external_reference_cells` step | First | no | CastA / CoercedA / TextZero (AVERAGEA) or Numbers (MAXA, MINA) | — |
| MIN, MAX | Aggregate | Ranked(Direct) | no | Cast / Numbers / Numbers | NaN start, 0 when none |
| PRODUCT | Reference | Ranked(Direct) | no | Cast / Numbers / Numbers | `seen_value` (set for an empty argument too) |
| COUNT | Aggregate | Items | no | — (own visitor: counts types) | — |
| COUNTA | PickedKeepingBlanks, after the `external_reference` count | Items | no | — (own visitor) | — |

`First` is today's behavior for DEVSQ/AVEDEV/SUMSQ. Today each argument is walked on its own,
and its error is returned at once (`?`). One argument has only one shape, so this is the same
as `First`.

## Alternatives considered
- **A trait per consumer** (`trait AggregateConsumer { fn number(..); fn error(..) }`). It is
  more ceremony than a closure plus a `const` policy, and makes the hot loop dynamic unless
  generic. Rejected.
- **The walker returns a `Vec` of items.** It allocates per call and loses early exit on a
  direct error. MEDIAN etc. still collect values into a `Vec` themselves, as today. Rejected.
- **Unify the unprobed differences now** (e.g. PRODUCT reading through `evaluate_aggregate_arg`,
  `Cast` vs `CastScalars`). That would change results without oracle evidence (CLAUDE.md
  directive 1). Each is a policy knob instead, and T-1566 probes them.
- **Folding COUNT/COUNTA in as a "count" number rule.** COUNTA counts errors and text, which
  no number rule yields. The visitor form already covers it with no extra concept.

## Test plan
- Pure refactor: `make eval` 0 regressions and 0 fixes in every suite after T-1474 and after
  T-1476. These must keep passing: probes/aggregate-error-precedence, sum-error-precedence,
  average-value-args, min-max-value-args, count-direct-text, counta-empty-value,
  averagea-array-constant-elements, subtotal-reference-operands-b/-c/-d, sum-long-text-args,
  near-zero-sum-forms, and fixtures/external-link-cached-ranges.
- Unit tests in `aggregate_args.rs` (or test/): one per `ErrorRule` arm, covering direct before
  and after a referenced error, two referenced errors, an array error under each rank, and a
  visitor error on direct text; `skip_empty_args`; the order of items across a range, an array
  and direct values; an empty range; an array holding `ArrayNode::Empty`; and early exit (after
  a held error, the cells after the error cell and a later range argument are not evaluated,
  checked by a visitor that counts items).
- `rg "let mut reference_error" crates/rustsheet-engine/src/functions` finds only
  aggregate_args.rs after T-1476. After T-1474 it also finds product.rs and mathematical.rs.
- `scripts/check_duplication.py` must not rise. Record a drop with `--update` (decision 0010).
- Performance (reviewer-performance): T-1474 adds a criterion bench
  `crates/rustsheet-engine/benches/aggregate.rs` beside `benches/edit.rs`. It covers
  `=SUM(A1:A1000000)` and `=AVERAGE(A1:A1000000)` over a numeric column, plus `=SUM(A1:A1000000)`
  with an error in A2 (early exit). The bench runs on `main` before the change and on the branch
  after, in release. Results must be within 5%, and the numbers go in the task's Outcome.

## Rollout
1. **T-1474** (engine, opus, M): add `aggregate_args.rs` (walker, policy, number layer, unit
   tests). Move SUM and every `for_each_value` / `for_each_value_skip_empty` /
   `for_each_value_a` consumer onto it, then delete those three helpers.
2. **T-1476** (quality, sonnet, S): move MIN, MAX (`fold_min_max`), PRODUCT, COUNT and COUNTA
   onto it with the policies above.
3. Probes (T-1439, T-1566) may then change a policy constant, each in its own task with
   evidence and a coercion.md / syntax.md update.

The old dispatches stay in place until their task moves their consumers. Each step is
independently mergeable.
