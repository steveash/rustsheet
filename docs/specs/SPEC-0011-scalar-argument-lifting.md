# SPEC-0011 — One shared lifting of scalar function arguments

- **Status:** accepted (on merge of T-1767)
- **Milestone / lane:** M9 / engine (the driver), then the functions-* lanes (the families) and quality (the old helpers)
- **Tasks:** T-1767 (this spec), T-1771 (directive-5 fix, independent), T-1770 (driver, first consumers),
  T-1236, T-1372, T-1290, T-1772, T-1383, T-1241, T-1271, T-1389 (families), T-1773, T-1774, T-1775
  (old helpers), T-1776 (catalog check), T-1777 (open probes). The order is under Rollout.

## Problem
Excel lifts a scalar argument over an array: `ROWS(F(..{v;v}..))` is 2. Worksheet functions also
lift over a multi-cell range in that argument. The engine does neither for most of the M6, M7 and
M9 functions. An array argument gets `#N/IMPL!` from `cast_to_number`. A range argument is
implicitly intersected by `Model::dereference` (`#VALUE!` off the range's rows, one value on them).
This is the largest remaining failure group in the probes suite. The counts below are failing cells
in `eval/baseline/probes.txt` at ea76d91f. That file is what `make eval` reports as known failures,
and a scratch run of `rustsheet-eval run` agrees. "Lift" cells are those of the form
`ROWS|COLUMNS|INDEX(F(..))`, split by whether the lifted argument is an array literal or a range.

| Case | Checked | Failing | Lift: array | Lift: range | Owner task |
|---|---|---|---|---|---|
| probes/stat-argument-modes | 573 | 396 | 214 | 96 | T-1290, T-1772 |
| probes/fin-argument-modes | 386 | 302 | 256 | 18 | T-1372 |
| probes/eng-argument-modes | 231 | 119 | 86 | 8 | T-1383 |
| probes/m7-argument-modes | 174 | 86 | 58 | 26 | T-1271 |
| probes/lookup-argument-modes | 117 | 66 | 30 | 36 | T-1241 |
| probes/db-argument-modes | 121 | 38 | 12 | 12 | T-1389 |
| probes/date-argument-modes | 168 | 28 | 28 | 0 | T-1770, T-1236 |
| probes/scalar-lift-shapes, -b, -c (new, this task) | 111 | 90 | | | all of the above |

That is about 880 lift cells. Another 17 generated cells (gen-m7, gen-m9) are `#N/IMPL!` today,
nearly all of them an array in a scalar argument. The owner tasks list them. The cells left over
in each case (86 stat, 28 fin, ...) are other bugs: ulp misses (T-1374, T-0901), the pairing
functions (T-1292), IMSUM (T-1384) and NPV (T-1373).

Lifting already exists, in nine shapes that do not share code:

| Helper | File | Consumers | Elements | Ranges | Bound |
|---|---|---|---|---|---|
| `NumberOrArray`, `lift_numbers`, `lift_number_operands`, `number_at` | cast.rs | ROUND family, MOD, CEILING/FLOOR, DATE, DATEDIF, HOUR/MINUTE/SECOND, MUNIT | `f64`, coerced by `array_node_to_number` | lift (`evaluate_range`) | `check_array_size` |
| `lift_with_defaults` | math_and_trigonometry/mathematical.rs | CEILING.MATH, CEILING.PRECISE, FLOOR.MATH, ISO.CEILING | via `lift_number_operands` | lift | same |
| `apply_number_unary` | cast.rs | DAY/MONTH/YEAR (`date_part_fn!`), WEEKDAY, ISOWEEKNUM; also the unary `-`/`%` chain | `f64`, rewritten in place (T-1078) | lift | per range |
| `single_number_fn!` | functions/macros.rs | 34 one-argument math functions | `f64` | lift | per range |
| MROUND, SQRTPI, `parity_predicate` | mathematical.rs, information/mod.rs | MROUND, SQRTPI, ISEVEN, ISODD | own loops | refused (`#VALUE!`) | none needed (one array) |
| `lifted_arg` / `Lifted` | information/mod.rs | IS*, ERROR.TYPE, ENCODEURL, FIELDVALUE, the offline functions | `ArrayNode` as stored | lift | per range |
| `TextArg`, `broadcast_text` | text/text_arg.rs | LEFT, MID, FIND, REPT, TEXT, CONCATENATE, … (17 call sites) | `ArrayNode` | lift | **none** (a 64 MiB text budget only) |
| `Weekend::lift` | date_and_time.rs | WORKDAY.INTL, NETWORKDAYS.INTL `weekend` | masks, memoized | lift | per range |
| `lift_with_cost`, `CriteriaArg`, `count_walks` | statistical/if_ifs.rs | *IF(S), AGGREGATE, IFS, SWITCH, VLOOKUP/HLOOKUP (`lookup_each`), XLOOKUP | `CalcResult`, scalar kernel per element | lift | `check_array_size` + cell-visit cap |

The broadcast primitives are shared: `bcast_idx`, `bcast_get` and `MISSING` in arithmetic.rs. The
operators use `broadcast_arrays`. INDEX lifts its indices on its own (T-1260, index.rs), and
GROUPBY/PIVOTBY have their own lifts (groupby.rs, relative_to.rs).

`broadcast_text` never calls `check_array_size`, which breaks directive 5. A row of indices against
a column of text is a full grid, so `ROWS(LEFT(A1:A2000,COLUMN(A1:ALL1)))` builds 2,000,000 cells
(ours: 2000), where `&` and ROUND over the same operands are `#ERROR!`. `LEFT(A:A,1:1)` would need
2^34 cells. T-1771 fixes this now, separately from the driver (P0).

Directive 3 forbids a tenth copy for the statistical, financial and engineering families.
Directive 4 asks for a spec first.

## Goals / non-goals
Goals:
1. One table says, for each parameter of each lifting function, whether it lifts over arrays and
   ranges, over arrays only, or is taken whole. Static analysis (the legacy `@` and the static
   shape of a call) reads the same table. So does a test against the eval catalog.
2. One driver reads the arguments once, decides the output shape, checks `check_array_size` and
   the cell-visit cap before computing any element (directive 5), and calls a scalar kernel once per
   element.
3. The families move onto it one task at a time (stat, financial, engineering, date, lookup, M7,
   database). Then the nine helpers above become the driver or thin wrappers of it (directive 3).
4. No result changes in a migration task. Each family task changes only the cells that its probes
   evidence.

Non-goals:
- Arithmetic, comparison and `&` operators. They keep `broadcast_arrays`, a two-operand hot path,
  which already uses `bcast_idx` and `MISSING`.
- Lifts whose elements are arrays or references. A lifted TAKE or DROP element is a whole sub-array
  (arrays.md "Dynamic-array and LAMBDA function argument modes"). A lifted OFFSET or INDIRECT gives
  references. `ArrayNode` cannot hold either. T-1271 and T-1241 settle those, using the driver
  where an element is a value.
- INDEX's index lift (T-1260), GROUPBY/PIVOTBY, BYROW/MAP and the other LAMBDA helpers.
- N and T. They read a range as a reference (its top-left cell, or "") and lift only arrays
  (information.md "N over an array lifts element-wise", T-1616, T-1713), so they keep `map_elements`.
- How a function reads its whole arguments: aggregate walks (SPEC-0009), pairing (T-1292),
  IMSUM/IMPRODUCT (T-1384), NPV/FVSCHEDULE (T-1373).
- Legacy `@` on array-valued calls and array constants (T-1713, T-1717). This spec only puts the
  existing `@` rule on the table.

## Excel behavior to match
Evidence: the argument-mode probes of each family (statistics.md, financial.md, engineering.md,
dates.md, lookup.md, arrays.md, "... function argument modes"). The three probes this task added
answer the general questions: probes/scalar-lift-shapes, -b and -c (111 cells, excel-graph,
2026-10-17). The full rule is recorded in arrays.md "Scalar arguments lift: shapes, element errors,
and the Analysis ToolPak range rule". In -shapes, every formula sits on the rows of its 60-cell
ranges, so an intersection would succeed there. -b asks each question on row 3 and on row 70.

### Which parameters lift
- **Scalar parameters lift over an array**, whatever the type: numbers, `cumulative`/`type`
  booleans, unit and suffix text, dates, basis, guess. All 210 stat, 256 financial and the
  engineering `ROWS(F(..{v;v}..))` cells are 2. IMSUM and IMPRODUCT are the exception.
- **Worksheet functions also lift over a multi-cell range.** This covers the statistical
  functions (92 range cells), the core financial ones (DB … VDB, IRR's guess; 18 cells), BIT* (both
  arguments: `ROWS(BITAND(13,D1:D60))` is 60, scalar-lift-shapes H7), the date functions DATE …
  WEEKDAY and the `weekend` argument, the lookup scalars and the M7 scalars. The lift happens on the
  range's own rows too, not an intersection: `ROWS(PMT(C1:C60,10,100))` and
  `ROWS(NORM.DIST(2.5,0,1,B1:B60))` are 60 (H13–H16).
- **Analysis ToolPak (ATP) functions lift over an array but refuse a multi-cell range in any
  scalar argument.** The range gives `#VALUE!` for its call, with no intersection, even on its own
  rows: EDATE's start and months, YEARFRAC's basis, BESSELJ's n, DEC2BIN's places, DELTA, and
  EFFECT's two arguments (H1, H2, H6, H8–H12; BESSELI eng-argument-modes E1). A reference that a
  function returns is refused too: `EDATE(IF(TRUE,A1:A60),1)` and `EDATE(INDEX(A1:A60,0,1),1)`
  (H50, -b K3), and `EOMONTH(..,TAKE(..))` (gen-m9/b58b6d7cd39d195f A9). A one-cell range or
  reference is its value (H4, -b J3). An operation on a range is an array and lifts:
  `ROWS(EDATE(A1:A60+0,1))` is 60 (H5, -b N3). The ATP functions: financial.md's list (ACCRINT …
  YIELDMAT, XIRR's guess, XNPV's rate), every engineering function but BIT*, EDATE, EOMONTH,
  WEEKNUM, WORKDAY(.INTL), NETWORKDAYS(.INTL), YEARFRAC (but not `weekend`), MROUND, SQRTPI,
  ISEVEN and ISODD.
- **An ATP call with an array and a range** lifts over the array only. The range is passed whole to
  every element, which refuses it, so the call is the array's shape with all elements `#VALUE!`.
  This holds on and off the range's rows: `EDATE({45000;45031},M1:M60)` and `EDATE(A1:A60,{1;2})`
  are 2 rows of `#VALUE!`, and `EDATE({45000,45031},M1:M60)` is 1 row (-b C–I, L).
- **Whole parameters** are vector, data and list arguments: the stat data arrays, NPV's values,
  holidays, LOOKUP/XLOOKUP vectors, CHOOSECOLS indexes, TEXTSPLIT delimiters and SORT's
  sort_index. They are read as one value and never lifted.

### Shape and broadcasting
- The result is as tall as the tallest lifted argument and as wide as the widest. A size-1
  dimension repeats, and positions past a shorter argument are `#N/A`. This holds in 1-D, for
  row × column grids and in 2-D: `PMT({0.1;0.2;0.3},{10;20},100)` is 3 rows, row 3 `#N/A`;
  `NORM.DIST({1;2;3},{0,1},1,TRUE)` is 3×2; `PMT({0.1,0.2;0.3,0.4},{10,20,30},100)` has column 3
  `#N/A`; a 60-cell range against `{10;20}` is 60 rows, `#N/A` from row 3 (H17–H25, H47–H48). ATP
  calls pad the same way (H22). These are the operator rules (arrays.md "MOD lifts over array
  arguments", CONCATENATE, *IFS), so `bcast_get` + `MISSING` serve every family.

### Errors and element values
- **Each element is its own scalar call.** A failing element is an error in that position only
  (H26–H27, H34–H35). In each element the leftmost failing argument wins:
  `PMT({1;#N/A},{#DIV/0!;#NUM!},100)` is `#DIV/0!`, `#N/A` (H31–H33).
- **A scalar error, or uncoercible scalar text, next to an array is broadcast.** It does not short
  out the whole call: `PMT(NA(),{10;20},100)`, `EDATE(NA(),{1;2})`, `PMT("x",{10;20},100)` and
  `EDATE("x",{1;2})` are 2 rows (H28–H30, H44–H45). CONCATENATE is the one known exception (a scalar
  error is the whole result, arrays.md), and it keeps its own rule.
- **An array element is a typed value, and a range cell is a referenced cell.** EDATE's
  `{TRUE;45000}` is `#VALUE!`, 45031, as a typed boolean date is `#VALUE!`. NORM.S.DIST's
  `{TRUE;"1"}` reads both as 1 (H36–H40). Over 0.1, blank, "x", `=1/0`, `PMT(F1:F4,10,100)` is a
  number, -10, `#VALUE!`, `#DIV/0!` (H41–H43): a blank cell is 0, as a blank cell reference is.
- With nothing lifted, the call is exactly today's scalar call.

### Legacy formulas and implicit intersection
- A worksheet function's range gets the legacy `@` (Signature::Scalar). Legacy
  `=PMT(C1:C60,10,100)` shows `=PMT(@C1:C60,10,100)`, gives the row's value on the range's rows and
  `#VALUE!` off them (-c G, R; -shapes J1, J4).
- An ATP argument's bare range gets **no** `@` and is `#VALUE!` on every row (-c E, H, P, S;
  -shapes J2). An operation on the range gets one: `=EDATE(A1:A60+0,1)` shows
  `=EDATE(@A1:A60+0,1)` (-c F, Q). This is the aggregate rule, `legacy_value_operand`.
- Under SUMPRODUCT (an array context) nothing is intersected. Legacy
  `SUMPRODUCT(PMT(C1:C60,10,100))` sums 60 and `SUMPRODUCT(EDATE(A1:A60,1))` is `#VALUE!` (J5–J8).
  Legacy `SUM(PMT({0.1;0.2},10,100))` sums both elements (J3).
- An explicit `@` intersects in a dynamic formula: `PMT(@C1:C60,10,100)` and `EDATE(@A1:A60,1)`
  (H46, -c D).
So the driver needs no legacy flag. Every legacy difference is an `@` that
`add_implicit_intersection` puts in from the table at import, or leaves out.

### Open questions (T-1777 probes them; the design's choice in brackets)
1. An error argument left or right of a refused range, as in `EDATE(NA(),A1:A60)` [the leftmost
   wins, as MROUND does today].
2. A 3-D range in a lifted argument [`#VALUE!`, as `lift_with_cost` gives today].
3. The other ATP-era functions not yet probed over a range: QUOTIENT, FACTDOUBLE, MULTINOMIAL,
   SERIESSUM and the rest of the Analysis ToolPak list [they keep today's behavior until probed].
4. Whether Excel saves an ATP call with a range argument as a dynamic-array formula [the static
   shape ignores the refused range].
5. Lifted elements that are arrays or references (TAKE, DROP, OFFSET, INDIRECT) [non-goal, see above].

## Design
Two new modules, both `pub(crate)`. Nothing leaves the engine crate except one `#[doc(hidden)]`
listing for the catalog check (T-1776).

### The argument-kind table: `crates/rustsheet-engine/src/functions/arg_kinds.rs`
```rust
/// How one parameter takes an array or a range (SPEC-0011).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ArgKind {
    /// Lifts over an array and over a multi-cell range or reference (PMT, NORM.DIST, BITAND, DATE).
    Lift,
    /// Lifts over an array. A multi-cell range or reference is `#VALUE!` in its position, with no
    /// intersection (the Analysis ToolPak functions: EDATE, EFFECT, BESSELJ, MROUND).
    LiftArrays,
    /// Read once as written (`evaluate_node_with_reference`) and handed to every element whole
    /// (LARGE's array, holidays, LOOKUP's vectors).
    Whole,
    /// Read by the caller before the driver runs (an aggregate walk, a prepared search vector),
    /// and captured by the kernel. The driver does not evaluate it, and its slot is `EmptyArg`.
    WholeByCaller,
}

/// The parameters of one function: `params[i]` is the kind of argument `i`. Arguments past
/// `params` take `repeat`, and the call is valid with `required..=max` arguments.
pub(crate) struct ArgKinds {
    pub(crate) required: usize,
    pub(crate) params: &'static [ArgKind],
    pub(crate) repeat: Option<ArgKind>,
    pub(crate) max: usize,
}

impl ArgKinds {
    pub(crate) const fn fixed(required: usize, params: &'static [ArgKind]) -> Self;
    pub(crate) fn kind(&self, index: usize) -> Option<ArgKind>;
    pub(crate) fn accepts(&self, count: usize) -> bool;
}

impl Function {
    /// The lifting table: `None` for a function the driver does not lift (yet).
    pub(crate) fn arg_kinds(&self) -> Option<&'static ArgKinds>;
}
```
A fifth kind is known and left to its task. A parameter that lifts over an array but reads a range
whole as data is the D-functions' `criteria` (probes/db-argument-modes F11:F23; T-1389 adds it).

`arg_kinds` is one `match` that maps functions to shared `const`s
(`const ATP_2: ArgKinds = ArgKinds::fixed(2, &[LiftArrays, LiftArrays])`,
`const PMT_ARGS: ArgKinds = ArgKinds::fixed(3, &[Lift; 5])`). A family task adds its functions,
and each entry carries a comment that cites its probe. It is the one source of truth:
- **Static analysis** reads it before its own per-function tables:
  - `get_function_args_signature` maps `Lift` to `Signature::Scalar`, the three other kinds to
    `Signature::Vector`, and a bad count to `Signature::Error`.
  - `static_analysis_on_function` broadcasts the static shapes of the `Lift` arguments (arrays and
    ranges) and of the `LiftArrays` arguments (arrays only: a bare range is refused, so it adds no
    shape). An `Unknown` argument makes the call `Unknown`. A `Whole` argument adds nothing.
  - `legacy_value_operand` (legacy.rs) takes the argument index. It returns true for a
    value operation in a `LiftArrays` position, so legacy `EDATE(A1:A60+0,1)` gets its `@` and
    `EDATE(A1:A60,1)` does not.

  The family task that adds a function to the table deletes its `args_signature_scalars` and
  `scalar_arguments` lines, so a function is never described twice.
  `remove_redundant_implicit_intersection` and the export follow, because they call the same
  functions.
- **The driver** reads the table to evaluate each argument.
- **The eval catalog** is checked against it (T-1776). `mode: value` must be `Lift` or
  `LiftArrays`. `array`, `aggregate`, `aggregate_a`, `range`, `criteria` and `lambda` must be
  `Whole` or `WholeByCaller`. A new optional catalog key `lift: arrays` marks the `LiftArrays`
  parameters, so the generator stops spending cases on ranges that are refused anyway. The test
  reads the table through `#[doc(hidden)] pub fn rustsheet_engine::lift_kinds(name) ->
  Option<Vec<&'static str>>`.

### The driver: `crates/rustsheet-engine/src/functions/util/lift.rs`
This is today's `lift_with_cost` and `CriteriaArg`, moved out of if_ifs.rs and generalized. It
sits beside SPEC-0009's `aggregate_args.rs`.
```rust
/// One argument as the driver holds it.
pub(crate) enum LiftedArg {
    /// One value for every element: a scalar, a one-cell reference's value, an error, a refused
    /// range (`#VALUE!`), an omitted argument (`EmptyArg`), or a whole argument.
    Scalar(CalcResult),
    /// An array, or a multi-cell range read once into its cells.
    Grid(Vec<Vec<ArrayNode>>),
}

/// What one element costs, for the cell-visit cap: the cells its whole arguments hold.
pub(crate) enum LiftCost<'a> { Unit, Cells(u64), Walk(&'a Node) }

pub(crate) struct LiftOptions {
    /// Reuse the previous element's result when every lifted element is equal to the previous
    /// one: the *IF(S) blank-tail optimization of `lift_with_cost`. Off for scalar kernels.
    pub(crate) reuse_equal_runs: bool,
}

impl Model {
    /// Evaluates `args` by `kind.arg_kinds()`, then calls `kernel` once with the values (nothing
    /// lifted) or once per element of the broadcast shape. Wrong argument count: the count error.
    pub(crate) fn lift_call<K>(
        &mut self,
        kind: Function,
        args: &[Node],
        cell: CellReferenceIndex,
        kernel: K,
    ) -> CalcResult
    where
        K: FnMut(&mut Self, &[CalcResult]) -> CalcResult;

    /// The broadcasting core over arguments already read (was `lift_with_cost`); the *IF(S),
    /// AGGREGATE, IFS, SWITCH and lookup lifts call it directly.
    pub(crate) fn lift_values<K>(
        &mut self,
        args: Vec<LiftedArg>,
        cost: LiftCost<'_>,
        options: LiftOptions,
        cell: CellReferenceIndex,
        kernel: K,
    ) -> CalcResult
    where
        K: FnMut(&mut Self, &[CalcResult]) -> CalcResult;
}
```
Each argument is read once, by its kind:

| Kind | Array | Multi-cell range or reference | One-cell range | Anything else |
|---|---|---|---|---|
| `Lift` | `Grid` | `Grid` via `evaluate_range` (bounded by `check_range_size`, clipped to the used area) | `Scalar(evaluate_cell)` | `Scalar(value)` |
| `LiftArrays` | `Grid` | `Scalar(#VALUE!)`, no intersection | `Scalar(evaluate_cell)` | `Scalar(value)` |
| `Whole` | `Scalar` of the value as `evaluate_node_with_reference` read it, once | | | |
| `WholeByCaller` | `Scalar(EmptyArg)`, not evaluated | | | |

A range across sheets is `Scalar(#VALUE!)` (open question 2). The kernel therefore never sees a
multi-cell `Range` in a lifted position. `Model::dereference`'s intersection, which is why BESSELI
intersects today (eng-argument-modes E1), is no longer reached from these functions.

`lift_values` then runs these steps:
1. **No `Grid`:** call `kernel` once with the values moved into a `Vec` (no clones). The result
   is returned as it is, so an unlifted call is exactly today's call.
2. **Shape:** rows and columns are the maxima of the grids' dimensions.
   `check_array_size(rows, columns)` is checked before anything else is computed or allocated. Past
   the bound the call is `#ERROR!`, the existing DEVIATION (arrays.md "Array results above 1,048,576
   cells").
3. **Cost:** elements × max(1, cells of the whole arguments) must stay within
   `MAX_LIFTED_CELL_VISITS`, through `too_many_visits`, before the first element. `LiftCost::Walk`
   is today's `walk_cost`. `lift_call` uses `Cells` (the cells of its `Whole` ranges and arrays) or
   `Unit`. So `LARGE(A:A,SEQUENCE(1048576))` is refused up front and does not run for hours.
4. **Elements**, in row-major order. One `Vec<CalcResult>` buffer is kept for the whole call. The
   scalar and whole slots are written once, and only the grid slots are overwritten per element, by
   `array_node_to_calc_result(bcast_get(grid, i, j).unwrap_or(&MISSING))`. Then `kernel(&buffer)`.
   With one grid and nothing else lifted, the grid is rewritten in place, which keeps T-1078's
   whole-column speed for `ABS(A:A)`-style calls.
5. **Element result** → `ArrayNode` via one function, `element_node`. A one-cell reference is its
   value. A multi-cell range, an array or a LAMBDA is `#VALUE!`, as `calc_result_to_array_node`
   does today. This is the single place T-1271/T-1241 would change for array-valued elements.
6. **Text budget:** element strings count against `util::within_text_budget`, the 64 MiB that
   `broadcast_text` has today, so every family has one output bound.

No argument is re-evaluated per element. A `Whole` value is the same `CalcResult` for every
element. A kernel that derives something costly from it (LARGE's sorted data, WORKDAY.INTL's
weekend mask results, today's `Weekend::lift` memo) computes it on the first element and keeps it
in the closure (`FnMut`).

### Kernels
A kernel is the scalar function over values. Turning a function into one is mechanical: each
`self.get_number(&args[i], cell)` becomes `self.cast_to_number(values[i].clone(), cell)`, and the
same for `get_number_no_bools`, `get_boolean` and `get_string` → `cast_to_*`. Checks keep their
order, so the leftmost-error rule per element holds by construction (PMT reads rate, nper, pv in
order: H31–H33).
```rust
pub(crate) fn fn_edate(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
    self.lift_call(Function::Edate, args, cell, |model, values| model.edate_of(values, cell))
}
fn edate_of(&mut self, values: &[CalcResult], cell: CellReferenceIndex) -> CalcResult { … }
```
`evaluate_function` (functions/mod.rs) is unchanged. The fn-pointer dispatch and its small
stack frames (T-1110) stay as they are, and the driver runs inside `fn_edate`.

### Error handling
There is no new public API, so no `thiserror`. Everything returns `CalcResult`. A bad argument
count gives `CalcResult::new_args_number_error` before anything is evaluated, as today. No
indexing on untrusted sizes: elements are read through `bcast_get`, and dimensions are `usize`
from `Vec::len`. The cost multiplication saturates. If a function calls `lift_call` and has no
table entry, the kernel is called once with the evaluated values, plus a `debug_assert!`. This is
never a panic in release, and the family's probe cells catch it.

### Migration of the existing helpers (directive 3)
| Helper | Becomes | Task |
|---|---|---|
| `lift_with_cost`, `CriteriaArg`, `count_walks` (if_ifs.rs) | `lift_values`, `LiftedArg` (moved). `lift_criteria` stays as the *IF(S) front end with `reuse_equal_runs` | T-1770 |
| MROUND, SQRTPI, `parity_predicate` | `LiftArrays` table entries and kernels. Their hand-written range checks are deleted | T-1773, T-1774 |
| `lift_numbers`, `lift_number_operands`, `number_at`, `lift_with_defaults`, `single_number_fn!`, `date_part_fn!`, `time_part_fn!`, the function callers of `apply_number_unary` | table entries + kernels. A missing optional argument takes the kernel's default. `NumberOrArray` and `apply_number_unary` stay only for the operators (`handle_arithmetic`, `apply_unary_chain`) | T-1773 |
| `lifted_arg` / `Lifted` (IS*, ERROR.TYPE, ENCODEURL, FIELDVALUE, the offline functions) | `Lift` entries. The IS* kernels test the element as it is (`EmptyCell` is ISBLANK) | T-1774 |
| `TextArg`, `broadcast_text` (17 call sites) | `Lift` entries + text kernels. The T-1022 rule (an error argument is the result only when it is the first failure) is what a left-to-right kernel gives. CONCATENATE keeps its scalar-error short cut (arrays.md) | T-1775 |
| `Weekend::lift` | WORKDAY.INTL/NETWORKDAYS.INTL kernels with the mask memo in the closure | T-1236 |
| `lookup_each`, XLOOKUP, AGGREGATE, IFS, SWITCH | already on `lift_with_cost`, so on `lift_values` once it moves | T-1770 |
| `broadcast_arrays`, INDEX's lift, GROUPBY's lifts | unchanged (non-goals) | — |

`rg -n "fn lift_numbers|fn lift_number_operands|fn lift_with_defaults|macro_rules! single_number_fn|fn lifted_arg|fn broadcast_text|fn lift_with_cost"`
finds nothing after T-1775. `scripts/check_duplication.py` must not rise; record each drop with
`--update` (decision 0010).

### Performance
- Unlifted calls (almost every cell) build one `Vec` of argument values, then go straight to the
  kernel. The kernel's casts are the ones it ran before, minus `evaluate_node_in_context`, which the
  driver ran instead.
- Lifted calls read each argument once. A range is read once into a grid, which is bounded and
  clipped to the used area. Each element writes only its lifted slots into one reused buffer, and a
  single-grid call rewrites the grid in place. A `Number` element allocates nothing. A `String`
  element clones its `Text`, as `array_node_to_calc_result` does today.
- T-1773 adds a criterion bench next to `benches/edit.rs`, because it moves the hot math lifts. It
  covers `=SUM(ROUND(A1:A1000000,2))`, `=SUM(ABS(A1:A1000000))` and `=SUM(PMT(A1:A100000,10,100))`
  on main and on the branch, in release. The first two must be within 5%, and the numbers go in the
  task's Outcome.

## Alternatives considered
- **A helper per family** (status quo, "lift literals only" variants for financial and
  engineering). That is six more copies of shape, padding, error and bound logic, and
  `broadcast_text` shows how a copy drops the bound. Rejected (directive 3).
- **Generalize `lift_numbers` (`f64` elements).** It cannot express the per-type rules the probes
  show: EDATE's typed-boolean `#VALUE!` per element, boolean `cumulative`, unit and suffix text,
  `EmptyArg` vs a blank cell. Rejected; elements are `CalcResult`s.
- **Lift in `evaluate_function` by re-invoking `fn_x` per element with literal argument nodes**
  (AGGREGATE's `value_node` trick). It needs no kernel rewrite. But a blank cell would become
  `EmptyArgKind` (`MROUND(,1)` is `#N/A`, while a blank cell is 0, and PMT's blank cell is -10),
  text needs re-quoting, every element allocates nodes, and reference-reading kernels would misread.
  Rejected.
- **Generate the table from eval/catalog/functions.yaml.** The engine must not depend on repo data
  at build time, and the catalog has no ATP distinction. Rejected. The catalog is checked against
  the table instead (T-1776).
- **Keep the kinds in the parser's `Signature`** (add a `LiftArrays` variant there). The evaluator
  would depend on parser internals, and the signature has no `Whole`/`WholeByCaller` distinction.
  The table lives with `Function` instead, and static analysis derives `Signature` from it.
- **A trait per kernel** (`trait ScalarKernel`). This is more ceremony than a closure. Closures
  also hold per-call caches. SPEC-0009 made the same choice. Rejected.

## Test plan
- **Probe cells:** the lift cells of the seven argument-mode cases above, and probes/
  scalar-lift-shapes, -b and -c. Each family task removes its lines from eval/baseline/probes.txt
  (`make eval-baseline`), with no new F lines anywhere. Value cells that are ulp misses (PMT, NORM.S.DIST:
  T-0901, T-1374) stay listed under those tasks.
- **Unit tests** in `functions/util/lift.rs` (T-1770), against `lift_values` with counting kernels
  and against formulas:
  - shapes: row × column grid; `#N/A` padding in 1-D and 2-D; a one-cell range is a scalar; a 1×1
    array lifts to a 1×1 result.
  - errors: an element error stays in its position; a scalar error and scalar text broadcast; the
    leftmost error wins per element.
  - `LiftArrays`: a multi-cell range alone is one `#VALUE!` and the kernel is called once; next to an
    array, every element is `#VALUE!` in the array's shape; a reference from IF or INDEX is refused;
    a one-cell reference is its value.
  - `Whole`: never lifted, and evaluated once. A kernel counting calls and the identity of the whole
    value shows this.
  - the unlifted path calls the kernel exactly once with the plain values.
- **Size bound (directive 5):** `=ROWS(PMT(A:A,1:1,100))` is `#ERROR!`, and a counting kernel is
  never called. A unit test of `lift_values` with a 1,048,576×1 and a 1×2 grid makes 0 kernel
  calls. A whole argument of 2^20 cells against 2^8 elements trips the cell-visit cap before the
  first element. T-1771 adds the same test for `broadcast_text` (`ROWS(LEFT(A1:A2000,COLUMN(A1:ALL1)))`
  is `#ERROR!`).
- **Static analysis:** legacy FORMULATEXT for `Lift` vs `LiftArrays` (`=PMT(@C1:C60,10,100)`,
  `=EDATE(A1:A60,1)`, `=EDATE(@A1:A60+0,1)`: -c E–H). Static shape `Array(2,1)` for
  `EDATE({1;2},A1:A3)` and `Scalar` for `EDATE(A1:A3,1)`. Each family task adds its functions to
  test_implicit_intersection_export.rs.
- **Metamorphic** (T-1776): for every table entry, the catalog's domains give one valid scalar
  call. The test evaluates it, then the same call with one lifted argument as `{v;v}`, and checks
  that both elements equal the scalar result. This catches a function that is in the table but not
  wired to `lift_call`, or the reverse.
- **No-change migrations** (T-1773, T-1774, T-1775): `make eval` shows 0 fixes and 0 regressions,
  and the bench above holds.

## Rollout
Each step is one PR of size S/M. The old paths stay until their step.
0. **T-1771** (functions-text, P0, S): bound `broadcast_text` by `check_array_size` now. It does
   not depend on the driver.
1. **T-1770** (engine, opus, M): `arg_kinds.rs`, `util/lift.rs` (moving `lift_with_cost`), the
   static-analysis hooks and the unit tests. First consumers are EDATE, EOMONTH, WEEKNUM and
   YEARFRAC. They are `LiftArrays` throughout and cover the refused range, the array + range rule
   and the legacy no-`@` rule: date-argument-modes F/G 20, 22, 24, 26, 54, 56, 66, 68, and the EDATE
   and YEARFRAC cells of scalar-lift-shapes, -b and -c.
2. After T-1770, in parallel (one lane each):
   - **T-1236** (functions-date, S): WORKDAY, WORKDAY.INTL, NETWORKDAYS, NETWORKDAYS.INTL
     (`LiftArrays`, `weekend` `Lift`, holidays `Whole`; `Weekend::lift` folds in).
   - **T-1372** (functions-financial, M): core `Lift`, ATP `LiftArrays`, values/schedule/dates
     `WholeByCaller`.
   - **T-1290** (functions-stat, M): the functions whose every argument is scalar (the
     distributions, CONFIDENCE, STANDARDIZE, …).
   - **T-1772** (functions-stat, M): the scalar arguments next to a data array (LARGE, SMALL,
     PERCENTILE*, QUARTILE*, PERCENTRANK*, RANK*, TRIMMEAN, FORECAST, PROB, Z.TEST, T.TEST). These
     need the `Whole` read once, the prepare-once cache and the cost cap.
   - **T-1383** (functions-engineering, M): `LiftArrays` except BIT* (`Lift`); fixes BESSELI's
     intersection.
3. **T-1241** and **T-1271** (functions-lookup), and **T-1389** (functions-criteria). Their value
   lifts use the driver. Lifts whose elements are references or arrays stay their own design
   inside those tasks.
4. **T-1773** (quality, opus, M), **T-1774** (quality, sonnet, S), **T-1775** (quality, opus, M):
   move the old helpers onto the driver with no result change, and delete them.
5. **T-1776** (harness, sonnet, S, any time after T-1770): the catalog cross-check, `lift: arrays`
   in the catalog, and the metamorphic test.
6. **T-1777** (research, sonnet, S, any time): probe the open questions. A finding changes a table
   entry or a kernel in a task of its own.
