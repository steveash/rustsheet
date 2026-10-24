# SPEC-0018 — Nested arrays: GROUPBY and PIVOTBY cells that hold a LAMBDA's array result

- **Status:** draft (accepted on merge of T-2459)
- **Milestone / lane:** M7 / engine (implementation), research (probes)
- **Tasks:** T-2459 (this spec), T-2592 (probes), T-2593 (representation, INDEX, TYPE, top-left reads), T-2594 (MAP, SCAN,
  REDUCE and aggregates over a nested cell)

## Problem
When the function GROUPBY or PIVOTBY applies returns an array (`LAMBDA(v,v)` over a group's values), Excel keeps that array
in the result's cell: the cell shows `#VALUE!`, but `TYPE(INDEX(GROUPBY(..),r,c))` is 64 and INDEX returns the group's
values. Ours makes each such cell the error `#VALUE!` (`Model::aggregate`,
functions/lookup_and_reference/group_functions.rs), because the engine has no value that is an array inside an array:
`CalcResult::Array` holds `ArrayNode` scalars (and LAMBDA ids). Known failures this leaves (eval/baseline, 2026-10-24):

- probes/t2360-array-lambda-holidays: 47 `P!` cells (the INDEX/TYPE grid P!B3..P!E37 and the lifted counts C1, C9, C17,
  C25, C33).
- probes/t2311-pivotby-value-columns-error P!C12 (`SUM(--ISERROR(PIVOTBY(..,LAMBDA(v,v))))` is 0) and P!D12 (MAP over the
  result is an error).
- gen-m7/21b4eef18424ce94 Calc!A1504 and its spill (REGEXEXTRACT whose return_mode is such a GROUPBY gets a valid mode).
- T-2398's `LAMBDA(w,w)` cells: probes/groupby-lifted-args A153 (`SCAN(GROUPBY(..,LAMBDA(w,w)),0,MIN)` is one `#N/A`) and
  gen-m7/b24ce74f08d3f91a A1850.

## Goals / non-goals
Goals: represent a nested array where Excel makes one (GROUPBY and PIVOTBY cells), with Excel's reads of it (INDEX, TYPE,
top-left when lifted, the shown `#VALUE!`), bounded in memory (CLAUDE.md directive 5).

Non-goals: nested arrays anywhere Excel gives `#CALC!` instead (BYROW, BYCOL, MAP, MAKEARRAY and SCAN whose LAMBDA returns an
array: arrays.md "BYROW and BYCOL: a LAMBDA result that is an array" and "MAKEARRAY with an array step"); a nested array that survives into a cell's stored value (Excel stores the shown
`#VALUE!`, so `=A1#` over the spill reads errors; unprobed, T-2592); arrays nested more than one level.

## Excel behavior to match
Evidence: probes/t2360-array-lambda-holidays P (excel-graph, 2026-10-23, T-2360); probes/t2311-pivotby-value-columns-error
P!C12, P!D12; probes/groupby-lifted-args A153, A161, A169 (excel-graph, 2026-10-22). docs/excel-semantics/arrays.md, the
paragraph starting "A LAMBDA array result is shown as `#VALUE!`, but the cell holds the array itself".

1. **Which cells are nested.** A value cell of GROUPBY or PIVOTBY (group, subtotal and Total cells alike) whose function
   returns an array holds that array, whatever its size: a one-record group is a 1x1 array (P!B3 `64|1x1|1`), a two-record
   group 2x1, the Total 4x1 (P!B21 `64|4x1|1`). One LAMBDA over several value columns gives each column its own array
   (P!B3 vs C3). Key, header and blank-padding cells are scalars (P!A3 `2|a`, P!D21 `16|#4`).
2. **Shown value.** The cell displays `#VALUE!`; a spilled grid shows `#VALUE!` in each nested cell.
3. **INDEX.** `INDEX(result,r,c)` that selects a nested cell returns its array: TYPE 64, ROWS/COLUMNS its shape,
   `INDEX(x,1,1)` its first value (P!B3..E37); `INDEX(GROUPBY(Data!A1:A4,Data!B1:B4,LAMBDA(w,w)),4,2)` spills the Total's
   `{1;2;3;4}` (T-2398 Context).
4. **Lifted reads take the top-left.** A function that lifts over the result reads each nested cell as its array's
   top-left value: `SUM(--ISERROR(GROUPBY(..)))` counts no error (P!C1, C9, C17, C25) unless a top-left is one
   (`{#N/A;3}`: P!C33 = 1); `SUM(--ISERROR(PIVOTBY(..)))` is 0 (t2311 P!C12); REGEXEXTRACT reads a valid return mode
   (gen-m7/21b4eef18424ce94 A1504).
5. **MAP over the result is an error** (t2311 P!D12 is IFERROR's "E"); which error is unprobed (T-2592).
6. **SCAN** over a GROUPBY result is one cell (`#N/A` for `LAMBDA(w,w)`, 0 for SUM, the error for `LAMBDA(w,1/0)`:
   groupby-lifted-args A153, A161, A169). T-2398 owns that rule; this spec only supplies the nested cell it reads.
7. **Not nested:** a LAMBDA whose body is a one-cell reference (`LAMBDA(x,Data!D2)`) and a two-parameter LAMBDA show
   `#VALUE!` in every cell too; whether those cells hold a nested array (TYPE 64 through INDEX) or the error is unprobed
   (T-2592). Until then they stay the error `#VALUE!`, as today.

Unknowns, all in T-2592's probe: the error MAP gives (and BYROW, BYCOL); SUM, COUNT, COUNTA, AVERAGE, ARRAYTOTEXT, TEXTJOIN,
CONCAT and `&` over the result; REDUCE over it; `+0` and `--` on one INDEXed nested cell (top-left or the array);
`A1#` over a spilled result; TYPE of the whole result (64 anyway); rule 7's two cases; a LAMBDA returning a GROUPBY that
itself holds nested cells (two levels); a nested array of more than 1,048,576 cells.

## Design
**Value.** A new `ArrayNode` variant, beside `ArrayNode::Lambda` (the precedent: a non-scalar element that each consumer
already has to place):

```rust
/// A cell of a GROUPBY or PIVOTBY result that holds its function's array result (SPEC-0018). Never in a parsed array
/// constant; its elements are never `Nested` (one level).
Nested(Arc<NestedArray>),

pub(crate) struct NestedArray {
    rows: Vec<Vec<ArrayNode>>, // non-empty, rectangular, no `Nested` element
}
```

`Arc` keeps a clone O(1) (results are cloned through LET bindings and the lifting driver), and the variant adds no size to
`ArrayNode` beyond one pointer. `NestedArray::new(rows) -> Option<Self>` enforces the invariants: an empty grid is no nested
array, and an element that is itself `Nested` is replaced by its top-left (rule 4; the two-level case is unprobed, so this
is the bounded choice until T-2592 says otherwise).

**One reader.** `ArrayNode::scalar(&self) -> &ArrayNode`: for `Nested`, its top-left element, otherwise itself. Every site
that today matches `ArrayNode::Lambda` to place a non-scalar (35 sites: cast.rs, arithmetic.rs, functions/util.rs,
aggregate_args.rs, lift/kinds, text_kernel.rs, spill_functions.rs, ...) reads `node.scalar()` first, so rule 4 holds at
every lifted read without a per-function rule. The compiler finds every exhaustive match; a wildcard arm is audited by
`rg 'ArrayNode::Lambda'` in the same PR.

**Producers.** `Model::aggregate` (group_functions.rs) turns an array result into `ArrayNode::Nested` instead of
`#VALUE!`; a 1x1 array too (rule 1). A reference result stays `#VALUE!` (rule 7) until T-2592 shows otherwise. No other
producer: BYROW/MAP/MAKEARRAY keep `#CALC!`.

**Consumers with their own rule.**
- INDEX selecting one element (`index_impl` and the lifted INDEX) returns `CalcResult::Array(nested.rows.clone())` for a
  `Nested` element (rule 3); an INDEX that selects a row or column of the result keeps the nested elements in it.
- TYPE of an argument that is a `Nested` element via `array_node_to_calc_result` sees an array: 64 (rule 3).
- `array_node_to_calc_result` (functions/util.rs) is the one place a single element becomes a `CalcResult`; there `Nested`
  becomes `CalcResult::Array`, so INDEX, LET and a LAMBDA parameter all hand the array on.
- Display and storage: `array_node_to_formula_value` (model.rs) and stringify show `#VALUE!` (rule 2); a spill stores the
  error, so no nested array outlives the formula's evaluation.
- MAP, SCAN, REDUCE and the aggregates follow T-2592's evidence (T-2594); until then they take `node.scalar()` like any
  lifted read.

**Bound (directive 5).** Each GROUPBY/PIVOTBY call counts the cells of the nested arrays it keeps against
`limits::MAX_SIZE` (1,048,576, the bound on any array the engine builds), together with the result's own cells. Past it the
whole result is `#ERROR!`, as an oversized array is today (DEVIATION, arrays.md "Array results above 1,048,576 cells";
decision 0014). The check runs as each function result is kept, before the next call, so the work and memory stay within
one bound. The existing per-call input cap (`Functions::refused`) is unchanged.

**Hash and equality.** `ArrayNode` derives `PartialEq`; `Nested` compares by content. The lifting driver's class hash
(functions/lift/driver.rs) hashes the top-left (`scalar()`), the value a lifted read sees.

## Alternatives considered
- **An id into a side store, as `ArrayNode::Lambda(usize)`.** Needs the store's lifetime managed per evaluation and charged
  separately; the array is plain data, so an `Arc` owns it without a store.
- **A `CalcResult::NestedArray` only at the top level.** The nested value lives inside an array (a cell of the result), so it
  has to be an `ArrayNode`.
- **Keep `#VALUE!` and special-case INDEX(GROUPBY(..)).** INDEX would have to re-run the group's function, and the lifted
  top-left reads (rule 4) would still be wrong.

## Test plan
- Unit tests: `NestedArray::new` invariants (empty, ragged, nested elements flattened); `scalar()`; the cap (a LAMBDA
  returning `SEQUENCE(1048576)` for two groups is `#ERROR!`).
- Eval: the 47 t2360 P! cells, t2311 P!C12, gen-m7/21b4eef18424ce94 A1504 (T-2593); t2311 P!D12 and T-2592's new cells
  (T-2594); groupby-lifted-args A153 and gen-m7/b24ce74f08d3f91a A1850 through T-2398.
- No regressions in probes/groupby-*, pivotby-* and gen-m7 (the existing `#VALUE!` cells of rule 7 stay).

## Rollout
1. T-2592 (research, S): probe the unknowns above; record them in arrays.md.
2. T-2593 (engine, M): `ArrayNode::Nested`, `scalar()`, the producer, INDEX/TYPE, display, the cap. Owns rules 1-4.
3. T-2594 (engine, S, after 1 and 2): MAP, SCAN, REDUCE and the aggregates over a nested cell per T-2592; rule 7 if the
   probe shows nested cells there.
4. T-2398 (SCAN over a GROUPBY result) depends on T-2593 for its `LAMBDA(w,w)` cells.
