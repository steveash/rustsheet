# SPEC-0011 — One shared lifting of scalar function arguments

- **Status:** accepted (on merge of T-1767)
- **Milestone / lane:** M9 / engine (the driver), then the functions-* lanes (the families) and quality (the old helpers)
- **Tasks:** T-1767 (this spec).
  - Independent directive-5 fixes: T-1771, T-1780, T-1781, T-1786.
  - The driver: T-1784 (pure move), T-1770 (table, hooks, first consumers).
  - The families: T-1236, T-1372, T-1290, T-1772, T-1383, T-1241, T-1271, T-1389.
  - The old helpers: T-1773, T-1785, T-1774, T-1775, T-1787.
  - The catalog check and the open probes: T-1776, T-1777.
  - Findings of T-1777: T-1800, T-1801, T-1802, T-1803.
  - Split from T-1241: T-1810 (the reference-valued lifts of OFFSET, INDIRECT and TRIMRANGE; CHOOSE onto the driver).
  - The order is under Rollout.

## Problem
Excel lifts a scalar argument over an array: `ROWS(F(..{v;v}..))` is 2. Worksheet functions also
lift over a multi-cell range in that argument. The engine does neither for most of the M6, M7 and
M9 functions. An array argument gets `#N/IMPL!` from `cast_to_number`. A range argument is
implicitly intersected by `Model::dereference` (`#VALUE!` off the range's rows, one value on them).
This is the largest remaining failure group in the probes suite.

The counts below are a **snapshot: 2026-10-17, main at ea76d91f**. They are failing cells in
`eval/baseline/probes.txt`, which is what `make eval` reports as known failures, and a scratch run of
`rustsheet-eval run` agreed. "Lift" cells are those of the form `ROWS|COLUMNS|INDEX(F(..))`, split
by whether the lifted argument is an array literal or a range. The split is a text heuristic, so
it is approximate.

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

### Today's lifting code
The engine already lifts in many places that share no code. The table is a snapshot of the
same date and commit. Each row has a migration task, or is a non-goal with its reason.

| Code | File | Consumers | Becomes | Task |
|---|---|---|---|---|
| `lift_with_cost`, `CriteriaArg`, `count_walks` | statistical/if_ifs.rs | *IF(S), AGGREGATE, IFS, SWITCH, `lookup_each` (VLOOKUP/HLOOKUP), XLOOKUP | moved as the driver core, `lift_values` | T-1784 |
| `is_lifted` | logical/mod.rs | IFS, SWITCH: when to call `lift_with_cost` | `LiftedArg::is_grid`, deleted | T-1784 |
| `NumberOrArray`, `lift_numbers`, `lift_number_operands`, `number_at` | cast.rs | ROUND family, MOD, CEILING/FLOOR, DATE, DATEDIF, HOUR/MINUTE/SECOND, MUNIT | table entries + kernels. `NumberOrArray` stays for the operators only | T-1773, T-1785 |
| `lift_with_defaults` | math_and_trigonometry/mathematical.rs | CEILING.MATH, CEILING.PRECISE, FLOOR.MATH, ISO.CEILING | table entries; the kernel takes the defaults | T-1773 |
| `single_number_fn!` | functions/macros.rs | 34 one-argument math functions | table entries + kernels | T-1773 |
| `apply_number_unary` | cast.rs | DAY/MONTH/YEAR (`date_part_fn!`), WEEKDAY, ISOWEEKNUM; the unary `-`/`%` chain | function callers move. It stays for `apply_unary_chain`, an operator | T-1785 |
| `time_part_fn!`, `date_part_fn!` | date_and_time.rs | HOUR, MINUTE, SECOND, DAY, MONTH, YEAR | table entries + kernels | T-1785 |
| MROUND's and SQRTPI's own range checks | mathematical.rs | MROUND, SQRTPI | `LiftArrays` entries | T-1785 |
| `parity_predicate` | information/mod.rs | ISEVEN, ISODD | `LiftArrays` entries | T-1774 |
| `lifted_arg` / `Lifted`, and `map_elements` in offline_errors.rs | information/ | IS*, ERROR.TYPE, ENCODEURL, FIELDVALUE, STOCKHISTORY and the other offline functions | `Lift` entries. `map_elements` stays only for N and T (below) | T-1774 |
| `TextArg`, `broadcast_text` | text/text_arg.rs | 17 call sites: LEFT, MID, FIND, REPT, TEXT, CONCATENATE, … | `Lift` entries + text kernels | T-1775 |
| `Weekend::lift` | date_and_time.rs | WORKDAY.INTL, NETWORKDAYS.INTL `weekend` | a kernel with the mask memo in its closure | T-1236 |
| CHOOSE's index lift (`choose_array`) | lookup_and_reference/mod.rs | CHOOSE | `index_num` is `Lift`, the value arms are `Whole`. `choose_array` becomes the kernel | T-1241 |
| IF's `IfArg`, `if_arg_at` | logical/mod.rs | IF (condition and branches broadcast, logic.md) | non-goal: IF evaluates its branches lazily and returns references. It keeps its own broadcast, which already uses `bcast_idx` | — |
| `lifted_relative_to`, `lifted_groupby` | relative_to.rs, groupby.rs | GROUPBY, PIVOTBY | non-goal: LAMBDA calls with their own cell-visit budget (SPEC-0006) | — |
| `broadcast_static_result` | expressions/parser/static_analysis.rs | the static shape of a call | stays as the shape primitive; the table-driven `lifted_static_result` calls it | T-1770 |
| `map_elements` (N, T) | information/mod.rs, text/t_function.rs | N, T | non-goal: they read a range as a reference (its top-left cell, or "") and lift only arrays (information.md, T-1616, T-1713) | — |
| `broadcast_arrays` | arithmetic.rs | operators | non-goal: a two-operand hot path, which already uses `bcast_idx`/`MISSING` | — |
| INDEX's index lift | lookup_and_reference/index.rs | INDEX | non-goal: its elements are references (T-1260) | — |

The last migration task (T-1775) ends with a grep showing that no lift outside the driver
remains, apart from the non-goals above (Test plan).

### Not driver paths, same hole (directive 5)
`broadcast_text` never calls `check_array_size`. A row of indices against a column of text is a
full grid: on a sheet whose A1:A2000 is filled, `ROWS(LEFT(A1:A2000,COLUMN(A1:ALL1)))` builds
2,000,000 cells (ours: 2000), where `&` and ROUND over the same operands are `#ERROR!`.
`evaluate_range` allocates a whole range even where it is blank, so `LEFT(A:A,1:1)` asks for
2^34 cells on any sheet. T-1771 fixes this (P0).

Reading the code of the functions that build arrays outside the lifting paths shows the same
hole. None of them calls `check_array_size` before it allocates:
- **EXPAND** (lookup_and_reference/expand.rs) casts `rows` and `columns` with `n as usize` and
  calls `Vec::with_capacity(target_rows)`. `EXPAND(1,1E9,1E9)` asks for 10^18 cells.
- **HSTACK, VSTACK** (hstack_vstack.rs) pad short arguments with `#N/A` to the widest or tallest
  argument. `VSTACK(A:A,1:1)` is 2^20 × 2^14 cells. **T-1780** (P0) fixes them.
- **WRAPROWS, WRAPCOLS** (wrapcols_wraprows.rs) cast `wrap_count` with `n as usize`. WRAPCOLS then
  allocates `vec![..; wrap_count]` before it pads.
- **MMULT** (math_and_trigonometry/mmult.rs) builds rows(array1) × columns(array2):
  `MMULT(A:A,1:1)` is 2^34 cells.
- **TEXTSPLIT** pads its result. **T-1781** (P0) fixes it.
- **XMATCH:** not traced (no allocation path read). T-1786 checks it.

T-1786 (P0) bounds EXPAND, WRAPROWS/WRAPCOLS and MMULT before they allocate. Its acceptance tests
include `=EXPAND(1,1E9,1E9)`, which must be `#ERROR!` at once. T-1780's include
`=ROWS(VSTACK(A:A,1:1))`. Each adds its functions to the arrays.md DEVIATION entry.

Directive 3 forbids yet another copy for the statistical, financial and engineering families.
Directive 4 asks for a spec first.

## Goals / non-goals
Goals:
1. One table says, for each parameter of each lifting function, how it takes an array or a range.
   Static analysis (the legacy `@` and the static shape of a call) is derived from the same table,
   and so is the eval catalog's `lift: arrays` key.
2. One driver reads the arguments once and decides the output shape. It bounds the output cells,
   the kernel work and the input cells before computing any element (directive 5). Then it calls a
   scalar kernel once per element.
3. The families move onto it one task at a time (stat, financial, engineering, date, lookup, M7,
   database). Then the existing lifting code becomes the driver, or is a stated non-goal
   (directive 3).
4. No result changes in a migration task. Each family task changes only the cells that its probes
   evidence.

Non-goals:
- The non-goal rows of the table above, each with its reason.
- Lifts whose elements are arrays or references. A lifted TAKE or DROP element is a whole sub-array
  (arrays.md "Dynamic-array and LAMBDA function argument modes"). A lifted OFFSET or INDIRECT gives
  references. `ArrayNode` cannot hold either. T-1271 and T-1241 settle those, using the driver
  where an element is a value.
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
A rule below names the cells it rests on, and says so when it rests on one cell.

### Which parameters lift
- **Scalar parameters lift over an array**, whatever the type: numbers, `cumulative`/`type`
  booleans, unit and suffix text, dates, basis, guess. All 210 stat, 256 financial and the
  engineering `ROWS(F(..{v;v}..))` cells are 2. IMSUM and IMPRODUCT are the exception.
- **Worksheet functions also lift over a multi-cell range.** This covers the statistical
  functions (92 range cells), the core financial ones (DB … VDB, IRR's guess; 18 cells), BIT*
  (BITAND's second argument too: `ROWS(BITAND(13,D1:D60))` is 60, scalar-lift-shapes H7), the date
  functions DATE … WEEKDAY, the lookup scalars and the M7 scalars. The lift happens on the range's own
  rows too, not an intersection: `ROWS(PMT(C1:C60,10,100))` and `ROWS(NORM.DIST(2.5,0,1,B1:B60))`
  are 60 (H13–H16). Only a written range was probed. A reference that a function returns
  (`PMT(IF(TRUE,C1:C60),10,100)`, `PMT(INDEX(C1:C60,0,1),10,100)`) is open question 6.
- **Analysis ToolPak (ATP) functions lift over an array but refuse a multi-cell range.** The range
  gives `#VALUE!` for its call, with no intersection, even on its own rows.
  - Arguments probed over a range:
    - every first scalar argument of the financial ATP functions (fin-argument-modes) and of the
      engineering functions (eng-argument-modes, BESSELI E1 on its range's rows);
    - EDATE's start and months, YEARFRAC's basis, BESSELJ's n, DEC2BIN's places, DELTA's first
      argument, and EFFECT's nominal_rate and npery (H1, H2, H6, H8–H12).
  - **Other later arguments are refused too** (T-1777, probes/scalar-lift-open-a Y1-Y30; for example PRICE's or COUP*'s frequency, ACCRINT's
    basis, CONVERT's units, COMPLEX's suffix, an IM* second argument). T-1777 sampled them: all refused (open question 3).
    T-1372 and T-1383 must probe a later argument before they enter it as `LiftArrays`. The table
    describes a function whole, so until then that function has no entry and keeps today's code.
  - A reference that a function returns is refused too: `EDATE(IF(TRUE,A1:A60),1)` and
    `EDATE(INDEX(A1:A60,0,1),1)` (H50, -b K3), and `EOMONTH(..,TAKE(..))` (gen-m9/b58b6d7cd39d195f A9).
  - A one-cell range or reference is its value (H4, -b J3).
  - An operation on a range is an array, and that lifts: `ROWS(EDATE(A1:A60+0,1))` is 60 (H5, -b N3).
  - The ATP functions: financial.md's list (ACCRINT … YIELDMAT, XIRR's guess, XNPV's rate), every
    engineering function but BIT*, EDATE, EOMONTH, WEEKNUM, WORKDAY(.INTL), NETWORKDAYS(.INTL),
    YEARFRAC, MROUND, SQRTPI, ISEVEN and ISODD.
  - **Exception:** the `weekend` argument of WORKDAY.INTL and NETWORKDAYS.INTL lifts over a range
    (`INDEX(WORKDAY.INTL(45000,5,B1:B3),2)` = 45007; dates.md "Date function argument modes"), so it
    is `Lift`.
- **An ATP call with an array and a range** lifts over the array only. The range is passed whole to
  every element, which refuses it, so the call has the array's shape. The probed elements are all
  `#VALUE!`, on and off the range's rows:
  - `EDATE({45000;45031},M1:M60)`: both elements (-b C, D), ROWS 2 (E);
  - `EFFECT({0.1;0.2},M1:M60)`: both elements (H, I);
  - `EDATE(A1:A60,{1;2})`: ROWS 2 (F) and element 1 only (G);
  - `EDATE({45000,45031},M1:M60)`: one row (L), elements all `#VALUE!` (-a W26-W29).

  T-1777 probed them: all `#VALUE!` (probes/scalar-lift-open-a W24-W30).
- **Whole parameters** are vector, data and list arguments: the stat data arrays, NPV's values,
  holidays, LOOKUP/XLOOKUP vectors, CHOOSECOLS indexes, TEXTSPLIT delimiters and SORT's
  sort_index. They are read as one value and never lifted.
- **The D-functions' `criteria`** lifts over an array literal, while a range is the criteria
  table, read whole (probes/db-argument-modes F11:F23). This is the fifth kind below.

### Shape and broadcasting
- The result is as tall as the tallest lifted argument and as wide as the widest. A size-1
  dimension repeats, and positions past a shorter argument are `#N/A`. This holds in 1-D, for
  row × column grids and in 2-D:
  - `PMT({0.1;0.2;0.3},{10;20},100)` is 3 rows, row 3 `#N/A`;
  - `NORM.DIST({1;2;3},{0,1},1,TRUE)` is 3×2;
  - `PMT({0.1,0.2;0.3,0.4},{10,20,30},100)` has column 3 `#N/A`;
  - a 60-cell range against `{10;20}` is 60 rows, `#N/A` from row 3 (H17–H25, H47–H48).

  ATP calls pad the same way (H22). These are the operator rules (arrays.md "MOD lifts over array
  arguments", CONCATENATE, *IFS), so `bcast_get` + `MISSING` serve every family.

### Errors and element values
- **Each element is its own scalar call.** A failing element is an error in that position only
  (H26–H27, H34–H35).
- **Error precedence within an element** is the kernel's own scalar check order, which is the
  leftmost for PMT and EDATE: `PMT({1;#N/A},{#DIV/0!;#NUM!},100)` element 2 is `#N/A` (H33), element 1,
  with only nper failing, is `#DIV/0!` (H31), and `INDEX(PMT({#NUM!},{#N/A},100),1)` is `#NUM!`, the
  reversed one `#N/A` (probes/scalar-lift-open-a W14-W17, T-1777). It is not the leftmost for
  YEARFRAC, which checks `basis` first (-d W1-W5; -a W6-W7), so the driver gives a refused range to
  the kernel as a `#VALUE!` in its slot and does not pick an error by position (T-1800).
- **A scalar error, or uncoercible scalar text, next to an array is broadcast.** It does not short
  out the whole call: `PMT(NA(),{10;20},100)` is 2 rows with element 2 `#N/A` (H28, H29).
  `EDATE(NA(),{1;2})`, `PMT("x",{10;20},100)` and `EDATE("x",{1;2})` are 2 rows, and
  every element is the scalar's own error: `#N/A`, `#VALUE!`, `#VALUE!` (H30, H44, H45; probes/scalar-lift-open-a W18-W23). CONCATENATE is the one
  known exception (a scalar error is the whole result, arrays.md), and it keeps its own rule.
- **An array element is a typed value, and a range cell is a referenced cell.**
  - EDATE's `{TRUE;45000}` is `#VALUE!`, 45031, because a typed boolean date is `#VALUE!`.
  - NORM.S.DIST's `{TRUE;"1"}` reads both as 1 (H36–H40).
  - Over 0.1, blank, "x", `=1/0`, `PMT(F1:F4,10,100)` is a number, -10, `#VALUE!`, `#DIV/0!`
    (H41–H43): a blank cell is 0, as a blank cell reference is.
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
  Legacy `SUM(PMT({0.1;0.2},10,100))` sums both elements (J3). Its FORMULATEXT is the written text,
  with no `@` (probes/scalar-lift-open-a AB1).
- An explicit `@` intersects in a dynamic formula: `PMT(@C1:C60,10,100)` and `EDATE(@A1:A60,1)`
  (H46, -c D).

So the driver needs no legacy flag. Every legacy difference is an `@` that
`add_implicit_intersection` puts in from the table at import, or leaves out.

### Open questions (T-1777 probed them: probes/scalar-lift-open-a to -d, excel-graph 2026-10-17; the design's choice in brackets)
1. **Answered.** An error beside a refused range [the leftmost wins]: it is the kernel's own check
   order, which is the leftmost for EDATE, EFFECT, ACCRINTM and PMT (`EDATE(NA(),A1:A60)` `#N/A`,
   `EDATE(A1:A60,NA())` `#VALUE!`; -a W1-W13, -d W8-W15) but not for YEARFRAC, which checks `basis` first
   (-d W1-W5, -a W6-W7). The driver must let the kernel decide (T-1800).
2. **Answered.** A 3-D range in a lifted argument [`#VALUE!`]: `#REF!` in a `Lift` argument (PMT,
   NORM.DIST, BITAND; -b W2, W3, W6, W10, W11, -d W17-W18, W21-W23, W25, W29), `#VALUE!` in a `LiftArrays` argument (EDATE; -b W4, W5, W7, -d
   W19-W20, W26) (T-1801).
3. **Answered.** Later ATP arguments are refused like the probed ones (PRICE, COUP*, ACCRINT, CONVERT,
   COMPLEX, IM*, DOLLARDE/FR; -a Y1-Y18), and so are QUOTIENT, FACTDOUBLE and SERIESSUM's x, n, m
   (-a Y19-Y21, Y24-Y26): they refuse a range, and they lift over an array (probes/math-atp-array-lift
   and -b, T-1802). GCD and LCM give one result (ROWS 1), as MULTINOMIAL does, whose value
   was probed (-d W30); SERIESSUM's coefficients is read whole (-a Y22-Y23, Y27-Y30).
4. **Answered, with a limit.** `=EDATE({45000;45031},A1:A60)` spills 2 rows, as the static shape says
   (-a AA3, AC3). FORMULATEXT carries no marker, so how Excel saves it needs the file (M14).
5. **Still open.** Lifted elements that are arrays or references (TAKE, DROP, OFFSET, INDIRECT) [non-goal]: not probed.
6. **Answered.** A function-returned multi-cell reference in a `Lift` argument [lifts like a written
   range]: it does (IF, INDEX, OFFSET, INDIRECT, CHOOSE; -a W31-W36).
7. **Answered.** The precedence and element-value gaps: precedence is the kernel's order (-a W14-W17
   leftmost for PMT and EDATE, H33 now confirmed); the scalar-broadcast elements are the scalar's own
   error (-a W18-W23); `EDATE(A1:A60,{1;2})` and `EDATE({45000,45031},M1:M60)` elements are all `#VALUE!`
   (-a W24-W30, W70-W73); the legacy J3's FORMULATEXT has no `@` (-a AB1).
8. **Answered.** A stacked result past the grid [`#VALUE!` through `check_array_size`]: Excel gives `#NUM!`
   for rows past the grid and allows columns past it (-c, -c2; T-1803). **Still open:** the 2^34-cell
   `ROWS(HSTACK(A:A,1:1))`, where the backend gave 0 (-c3), which is not trusted.

## Design
New module `crates/rustsheet-engine/src/functions/lift/`, all `pub(crate)`:
- `mod.rs` re-exports;
- `kinds.rs` holds the table;
- `driver.rs` holds the driver (`lift_values` and the broadcast `lift_grids`);
- `call.rs` holds `lift_call` and its read phase;
- `arg_buf.rs` holds `ArgBuf`;
- `tests/` holds the unit tests.

Static analysis reads the table in `expressions/parser/static_analysis/lifted.rs`
(`lifted_signature`, `lifted_static_result`, `lifts_arrays_at`); T-1770 moved OFFSET's and
INDIRECT's shapes to `static_analysis/reference_result.rs` and `broadcast_shapes` to `static_analysis/broadcast.rs` to make room for the dispatch.

It is a directory module of its own, not part of `functions/util/`. Static analysis and every
family import the table, while `util/` is the walk helpers.

The one item that leaves the crate is a testing hook for the catalog check (T-1776):
`#[doc(hidden)] pub fn lift_kinds(name: &str) -> Option<Vec<&'static str>>`. It is hidden, not
part of the public API (SPEC-0001), and comes with no stability promise.

### The argument-kind table (`functions/lift/kinds.rs`)
```rust
/// How one parameter takes an array or a range (SPEC-0011).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ArgKind {
    /// Lifts over an array and over a multi-cell range or reference (PMT, NORM.DIST, BITAND, DATE).
    Lift,
    /// Lifts over an array. A multi-cell range or reference is `#VALUE!` in its position, with no
    /// intersection (the Analysis ToolPak functions: EDATE, EFFECT, BESSELJ, MROUND).
    LiftArrays,
    /// Lifts over an array; a range is handed to the kernel whole, as data (the D-functions'
    /// `criteria`, probes/db-argument-modes F11:F23; entered by T-1389).
    LiftArraysRangeWhole,
    /// Read once as written (`evaluate_node_with_reference`) and handed to every element whole
    /// (LARGE's array, holidays, LOOKUP's vectors).
    Whole,
    /// Read by the caller before the driver runs (an aggregate walk, a prepared search vector),
    /// and captured by the kernel. The driver does not evaluate it, and its slot is `EmptyArg`.
    WholeByCaller,
}

/// What one kernel call may cost, beyond reading its arguments (security rule below).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum KernelWork {
    /// O(1), or a loop with a fixed iteration cap that does not depend on argument values.
    Constant,
    /// A loop whose length depends on argument values. One call is charged `weight` units
    /// against the visit cap, and the kernel also keeps its own per-call cap, as T-1197 bounds MULTINOMIAL's loop.
    Weighted(u32),
}

/// The parameters of one function: `params[i]` is the kind of argument `i`. Arguments past
/// `params` take `repeat`, and the call is valid with `required..=max` arguments.
pub(crate) struct ArgKinds {
    pub(crate) required: usize,
    pub(crate) params: &'static [ArgKind],
    pub(crate) repeat: Option<ArgKind>,
    pub(crate) max: usize,
    pub(crate) work: KernelWork,
}

impl ArgKinds {
    pub(crate) const fn fixed(required: usize, params: &'static [ArgKind], work: KernelWork) -> Self;
    pub(crate) fn kind(&self, index: usize) -> Option<ArgKind>;
    pub(crate) fn accepts(&self, count: usize) -> bool;
}

impl Function {
    /// The lifting table: `None` for a function the driver does not lift (yet).
    pub(crate) fn arg_kinds(&self) -> Option<&'static ArgKinds>;
}
```
`arg_kinds` is one `match` that returns a `&'static` to shared `const`s
(`const ATP_2: ArgKinds = ArgKinds::fixed(2, &[LiftArrays, LiftArrays], Constant)`). The driver
calls it once per call. A family task adds its functions, and each entry carries a comment that
cites its probe.

**The table wins: it is the single source of truth.**
- **Static analysis** asks the table first. Only a function without an entry falls through to the
  old hand-written arms, which T-1770 moves into `fallback_signature` and `fallback_shape`.
  - `get_function_args_signature` maps `Lift` to `Signature::Scalar`, the other kinds to
    `Signature::Vector`, and a bad count to `Signature::Error`.
  - `static_analysis_on_function` calls `lifted_static_result`. That function feeds
    `broadcast_static_result` the shapes of the `Lift` arguments (arrays and ranges) and of the
    `LiftArrays`/`LiftArraysRangeWhole` arguments (arrays only: a bare range adds no shape). An
    `Unknown` argument makes the call `Unknown`, and a `Whole` argument adds nothing.
  - `legacy_value_operand` (legacy.rs) takes the argument index. It returns true for a value
    operation in a `LiftArrays` position, so legacy `EDATE(A1:A60+0,1)` gets its `@` and
    `EDATE(A1:A60,1)` does not.

  A family task deletes the old arms of each function it enters. A unit test enforces this: for
  every `Function` with an entry, the two fallbacks return `None`, so no function is in the table
  and in a hand-written arm. `remove_redundant_implicit_intersection` and the export follow,
  because they call the same functions.
- **The driver** reads the table to evaluate each argument.
- **The eval catalog** derives `lift: arrays` from the table (T-1776). A generator command writes
  the key from the `lift_kinds` hook, and the test fails when the committed catalog differs. One
  mapping rule is stated here and in T-1389 and T-1776:

  | Table kind | Catalog |
  |---|---|
  | `Lift` | `mode: value` |
  | `LiftArrays` | `mode: value`, `lift: arrays` |
  | `LiftArraysRangeWhole` | `mode: criteria` or `range`, `lift: arrays` |
  | `Whole`, `WholeByCaller` | `mode: array`, `aggregate`, `aggregate_a`, `range`, `criteria` or `lambda`, without `lift` |

  The generator stops passing ranges to `mode: value, lift: arrays` parameters, because they are
  refused anyway.

### The driver (`functions/lift/driver.rs`)
This is today's `lift_with_cost` and `CriteriaArg`, moved out of if_ifs.rs (T-1784) and
generalized (T-1770).
```rust
/// One lifted argument as the driver holds it.
pub(crate) enum LiftedArg {
    /// One value for every element: a scalar, a one-cell reference's value, an error, a refused
    /// range (`#VALUE!`), an omitted argument (`EmptyArg`), or a whole argument.
    Scalar(CalcResult),
    /// An array, or a multi-cell range read once into its cells.
    Grid(Vec<Vec<ArrayNode>>),
}

/// What one element costs, for the cell-visit cap.
pub(crate) enum LiftCost<'a> {
    /// `KernelWork::Constant` and no whole data: elements × 1.
    Unit,
    /// Every element walks `cells` (today's *IF(S) criteria walk): elements × cells.
    Cells(u64),
    /// The kernel prepares its whole data once (LARGE sorts it): cells + elements.
    Prepared(u64),
    /// `walk_cost` of `range`, `walks` times per element (the *IF(S) front end: one walk per
    /// criterion).
    Walk { range: &'a Node, walks: u64 },
}

pub(crate) struct LiftOptions {
    /// Reuse the previous element's result when every grid slot equals the previous element's:
    /// the *IF(S) blank-tail optimization of `lift_with_cost`. Off for scalar kernels.
    pub(crate) reuse_equal_runs: bool,
    /// The table's `KernelWork` (`lift_call`); `Constant` for the direct callers.
    pub(crate) work: KernelWork,
}

impl LiftOptions {
    /// What every direct caller passes (*IF(S), AGGREGATE, IFS, SWITCH, the lookups).
    pub(crate) const REUSE_EQUAL_RUNS: Self;
}

/// A per-element kernel: the scalar function over the argument values of one element.
pub(crate) type Kernel<'k> = dyn FnMut(&mut Model, &[CalcResult]) -> CalcResult + 'k;

impl Model {
    /// Reads `args` by `kind.arg_kinds()` (non-generic, `#[inline(never)]`, so the reading code
    /// is not copied into every caller's frame, T-1110), then runs the kernel once (nothing
    /// lifted) or once per element. Wrong argument count: the count error.
    pub(crate) fn lift_call(
        &mut self,
        kind: Function,
        args: &[Node],
        cell: CellReferenceIndex,
        kernel: &mut Kernel<'_>,
    ) -> CalcResult;

    /// The broadcasting core (was `lift_with_cost`). The *IF(S), AGGREGATE, IFS, SWITCH and
    /// lookup lifts call it directly, with their arguments evaluated: a range there, a one-cell
    /// range included, lifts as a grid (their T-1784 contract). It reads them into `LiftedArg`s
    /// and runs the same broadcast (`lift_grids`) as `lift_call`.
    pub(crate) fn lift_values(
        &mut self,
        args: Vec<CalcResult>,
        cost: LiftCost<'_>,
        options: &LiftOptions,
        cell: CellReferenceIndex,
        kernel: &mut Kernel<'_>,
    ) -> CalcResult;
}
```
As built (T-1770): `lift_call` runs `lift_call_with(kinds, args, LiftCost::Unit, …)`, which a
future `Prepared` or `Cells` consumer calls with its own cost. The direct callers keep their
T-1784 totals exactly: *IF(S) passes `Walk { walks: criteria }`, `lookup_each` and AGGREGATE
multiply their `Cells` by the lifted arguments, IFS and SWITCH pass `Cells(arguments)`; XLOOKUP
passes its search cost once (it divided it by its three criteria before, so its bound only
loosens by the rounding).
`ArgBuf<T>` is a fixed inline array of 8 slots with a `Vec` fallback for longer calls. It adds no
new dependency. A call with 8 arguments or fewer does not touch the heap to hold its values. The
kernel is `&mut dyn FnMut`: there is one indirect call per element, and one copy of the loop for
every function instead of a monomorphized copy each.

Each argument is read once, by its kind:

| Kind | Array | Multi-cell range or reference | One-cell range | Anything else |
|---|---|---|---|---|
| `Lift` | `Grid` | `Grid` via `evaluate_range` | `Scalar(evaluate_cell)` | `Scalar(value)` |
| `LiftArrays` | `Grid` | `Scalar(#VALUE!)`, no intersection; INDEX, XLOOKUP, TAKE, DROP and TRIMRANGE are read in their reference form, so a multi-cell one is refused too (TAKE: gen-m9/b58b6d7cd39d195f A9; INDEX and IF: scalar-lift-shapes H50, -b K3; XLOOKUP, DROP and TRIMRANGE: scalar-lift-open-e W10-W22, T-1801) | `Scalar(evaluate_cell)` | `Scalar(value)` |
| `LiftArraysRangeWhole` | `Grid` | `Scalar(the range)`, read whole | `Scalar(the range)` | `Scalar(value)` |
| `Whole` | `Scalar` of the value as `evaluate_node_with_reference` read it, once | | | |
| `WholeByCaller` | `Scalar(EmptyArg)`, not evaluated | | | |

A range across sheets (`Sheet1:Sheet2!C1:C2`, 3-D) is `Scalar(#REF!)` in a `Lift` position and
`Scalar(#VALUE!)` in a `LiftArrays` position (open question 2; T-1801). That holds for a one-cell
3-D reference, in a later argument and beside an array (every element carries it). Evidence: `Lift` `#REF!`
probes/scalar-lift-open-b W2, W3, W6, W10, W11 and -d W17-W18, W21-W23, W25, W29; `LiftArrays` `#VALUE!`
-b W4, W5, W7, -d W19-W20, W26, and for a one-cell 3-D range scalar-lift-open-e W1-W9. The kernel therefore never sees a
multi-cell `Range` in a `Lift` or `LiftArrays` position. `Model::dereference`'s intersection, which
is why BESSELI intersects today (eng-argument-modes E1), is no longer reached from these functions.

`lift_values` (and the read phase before it) runs these steps:
1. **No `Grid`:** the read phase hands the kernel the values directly, as a slice of the inline
   buffer. The read phase keeps the values in an `ArgBuf<CalcResult>` and records the grid
   positions separately, so this slice is borrowed, not copied; `LiftedArg` is only built when a
   grid exists. No `LiftedArg`, grid or shape code runs, and nothing goes on the heap for 8 arguments or
   fewer. The result is returned as it is, so an unlifted call is exactly today's call.
2. **Input bound:** before each range is read into a grid, `check_range_size` bounds that grid,
   and the running total of grid cells must stay within `MAX_LIFTED_CELL_VISITS`. So the read phase
   allocates at most that many input cells, however many arguments the call has. No parser limit
   on the argument count exists, so this is the bound.
3. **Output bound:** rows and columns are the maxima of the grids' dimensions.
   `check_array_size(rows, columns)` is checked before any element is computed or allocated. Past
   the bound the call is `#ERROR!`, the existing DEVIATION (arrays.md "Array results above 1,048,576
   cells"). That is a separate rule from the grid bound: a stack with rows past the grid is `#NUM!` in
   Excel and columns past it are allowed (T-1803), so the stacking functions keep their own answer
   there and `check_array_size`'s cell-count `#ERROR!` stays above 2^20 cells.
4. **Work bound:** before the first element, the total cost must stay within
   `MAX_LIFTED_CELL_VISITS` (`too_many_visits`, saturating arithmetic). The total is the `LiftCost`
   of the call: `elements × 1` for `Unit`, `elements × cells` for `Cells`/`Walk`,
   `cells + elements` for `Prepared`. The table's `KernelWork::Weighted(w)` multiplies the
   per-element term by `w`.
   - `LARGE(A:A,SEQUENCE(1048576))` with a per-element walk is refused up front.
   - `LARGE(A1:A1000000,SEQUENCE(200))`, which sorts once, costs about 1,000,200 and runs.

   Steps 2–4 bound the output, the kernel calls and the input. They do not bound what one kernel
   call does inside, which is why `KernelWork` exists.
5. **Elements**, in row-major order, into one buffer kept for the whole call. Scalar and whole
   slots are written once. Only the grid slots are overwritten per element, by
   `array_node_to_calc_result(bcast_get(grid, i, j).unwrap_or(&MISSING), cell)` (functions/util.rs).
   - **In place:** when exactly one argument is a grid (any number of scalar or whole arguments
     beside it, as in `ROUND(A1:A1000000,2)`), the output has the grid's shape. Each element is
     moved out of the grid (`mem::replace(node, ArrayNode::Empty)`), converted by value, given to
     the kernel, and its result written back into the same slot. This is T-1078's contract: at most
     one allocation per output element, and none for a number. Kernels moved onto the driver build
     element errors with `String::new()` messages (T-1770, T-1773 and T-1785 acceptance), so a
     failing number element allocates nothing either; about 211 sites under functions/ allocate a
     message today, and only the migrated kernels change.
   - **`reuse_equal_runs`:** the element is compared with the previous position slot by slot,
     through `bcast_get` at both positions. No per-element key `Vec` is built.
6. **Element result** → `ArrayNode` via the existing `cast::calc_result_to_array_node`. A range,
   an array or a LAMBDA is `#VALUE!`, as today. There is no new helper, and T-1787 deletes the four
   private copies of that function in functions/logical/.
7. **Text budget:** element strings count against `util::within_text_budget`, the 64 MiB
   `broadcast_text` has today. The size check comes first: a call above the array bound is
   `#ERROR!` before any text is built, and the budget's error stays `#VALUE!`.

No argument is re-evaluated per element. A `Whole` value is the same `CalcResult` for every
element. A kernel that derives something costly from it (LARGE's sorted data, WORKDAY.INTL's
weekend mask results, today's `Weekend::lift` memo) computes it on the first element and keeps it
in its closure. It declares `LiftCost::Prepared`.

As built (T-1772): LARGE, SMALL, PERCENTILE*, QUARTILE*, PERCENTRANK* and TRIMMEAN take their data
argument as `WholeByCaller`, not `Whole`. Their readers look at the argument's node (a typed scalar
against a cell reference, a union's areas, an external range, `evaluate_node_in_context` for a
single cell), which a `Whole` value read by `evaluate_node_with_reference` cannot tell apart, so a
`Whole` read would change scalar results. `statistical/sorted_data.rs` reads the numbers once after
the count check, the kernel sorts them on its first element, and the call goes through
`lift_call_costed` with `LiftCost::Prepared(numbers)` (TRIMMEAN: `Cells(numbers)`, since every
element averages the kept values). `Whole` stays unused until T-1236's holidays.

As built (T-1383): the engineering functions are the first `KernelWork::Weighted` entries. The four
Bessel functions loop up to about 2·10^5 times per element at the order cap (`MAX_ORDER`, 100,000,
now also BESSELI's and BESSELK's per-call cap), so they are `Weighted(2^16)`: a lifted call runs at most
2,048 elements, about 2 s at the cap in release (engineering.md "BESSEL order cap", a DEVIATION). HEX2DEC
and OCT2DEC take one argument, as the parser's signature already said; their old code ignored a second one.

As built (T-1384): IMSUM and IMPRODUCT are the first `WholeByCaller` entries outside the sorted-data
functions: `COMPLEX_FOLD` repeats `WholeByCaller` up to 255 arguments. The entry is declarative, as
T-1772's are: it gives the argument count, the static shape (one value) and the legacy export rule (no
`@`); the function reads its own arguments through SPEC-0009's `walk_aggregate_read` and never enters
the lift driver, so its `KernelWork` is not charged. Its own read cap (`MAX_LIFTED_CELL_VISITS` over
the cells walked) is engineering.md "IMSUM and IMPRODUCT read cap", a DEVIATION.

As built (T-1822): ACCRINT, the COUP* family, DURATION, MDURATION and PRICE are `LiftArrays` and the first
`Weighted` entries whose loop is bounded by the calendar rather than by a cap constant (their per-call cap is
`check_settlement` plus the calendar's end): a coupon schedule walk steps
one period at a time, at most about 32,400 quarters once a settlement below serial 1 is `#NUM!` (Excel's rule,
probes/fin-coupon-walk-bounds; before, a settlement of -1E15 walked about 10^6 quarters). Each walk is charged 2^16
units (`COUPON_WALK_WEIGHT`, about 1 ms at the longest schedule): two for COUP* and ACCRINT (1,024 elements), four
for DURATION, MDURATION and PRICE (512; financial.md "Coupon schedule work cap", a DEVIATION; T-1846 narrows it
to a schedule-length budget). YIELD, the ODD*
functions, CUMIPMT/CUMPRINC, AMOR* and the scalars beside XNPV's, XIRR's and FVSCHEDULE's data are T-1841 to T-1843 and T-1845.

As built (T-1290): 44 statistical functions whose every argument is scalar are `Lift` entries. Kernels that call
statrs' incomplete beta (a 140-step continued fraction) a fixed number of times are `Constant` (0.14 to 0.3 us an
element in release, BINOM.DIST with 1E18 trials 3.2 us), and so is PERMUT, whose product stops once it is infinite (at
most 171 factors). BINOM.INV and CRITBINOM are `Weighted(4096)`: a halving search over a `u64`, at most 64 cdfs, up to
about 65 us an element, so a lift is capped at 32,768 elements, about 2.1 s (statistics.md, a DEVIATION). The quantile searches, the incomplete-gamma functions and
HYPGEOM.DIST loop by value with no per-call cap, so they stay out until T-1848 bounds them.

As built (T-1842): YIELD, ODDFPRICE, ODDFYIELD, ODDLPRICE and ODDLYIELD are `LiftArrays` and `Weighted` from their
measured worst case. Their schedule walks are hoisted out of the solvers first, a tighter per-call bound with the same
values: `CouponSchedule` and `OddFirstSchedule` (functions/financial/financial_bonds.rs) walk once per element, so a
solver step is a sum over the coupons (up to 32,400, 0.55 ms in release) rather than four walks and a sum (5 ms). YIELD
and ODDFYIELD are charged 2^24 (`YIELD_SOLVER_WEIGHT`: at most 397 prices, about 0.23 s an element at the bound, so 8
elements, about 1.8 s), ODDFPRICE eight walks (256 elements) and the closed-form ODDLPRICE and ODDLYIELD two (1,024);
financial.md "Yield and odd-coupon work cap", a DEVIATION that T-1869 narrows to a schedule-length budget.

### Kernels
A kernel is the scalar function over borrowed values: `&[CalcResult]`. It never clones a slot.
It reads a value through borrowing casts, `cast_number_ref`, `cast_bool_ref`, `cast_string_ref` and
`cast_number_no_bools_ref` (`&CalcResult` → `Result<_, CalcResult>`). As built (T-1770) the casts
keep the messages the owned casts had ("Expecting number", ...), so a cast failure still allocates
its message; only the errors a kernel builds itself are `String::new()`. The owned `cast_to_*`
become `dereference` plus the borrowing cast, so each coercion keeps one definition. An error is
cloned only on the failure path. Turning a function into a kernel is mechanical:
`self.get_number(&args[i], cell)` becomes `self.cast_number_ref(&values[i], cell)`, and the same for
the other getters. Checks keep their order, so the kernel's own check order decides per element, by construction
(PMT reads rate, nper, pv in order; YEARFRAC reads basis, start, end). The driver must not scan the
arguments for the first error before calling the kernel: a refused range reaches the kernel as
`#VALUE!` in its slot (T-1800).
```rust
pub(crate) fn fn_edate(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
    self.lift_call(Function::Edate, args, cell, &mut |model, values| model.edate_of(values, cell))
}
fn edate_of(&mut self, values: &[CalcResult], cell: CellReferenceIndex) -> CalcResult { … }
```
`evaluate_function` (functions/mod.rs) is unchanged. The fn-pointer dispatch and its small
stack frames (T-1110) stay as they are, and the driver runs inside `fn_edate`.

**Security rule for kernel work.** A table entry is `KernelWork::Constant` only if its kernel is
O(1) or loops a fixed number of times, whatever the argument values. A value-dependent kernel
must be `Weighted` and also keep a per-call cap of its own. Examples are BINOM.INV, POISSON,
COMBIN, FACT and MULTINOMIAL (T-1197), the RATE/IRR/YIELD solvers (they have iteration caps, so
they are `Constant` only if the cap is fixed), and the Bessel series. T-1776 checks a list of
value-dependent functions against the table, so a family task cannot enter one as `Constant`
silently.

### Error handling, recursion and stack
- **Errors:** everything returns `CalcResult`, and the only public item is the hidden testing hook,
  so no `thiserror` is needed. A bad argument count gives `new_args_number_error` before anything is
  evaluated, as today. No indexing on untrusted sizes: elements are read through `bcast_get`,
  dimensions are `usize` from `Vec::len`, and cost arithmetic saturates.
- **A missing table entry** is a programming error, not an input condition. `lift_call` is only
  ever called with the constant `Function` of the `fn_x` that calls it, so no workbook can reach the
  case. If it happens, the kernel is called once with the evaluated values, behind a
  `debug_assert!`. It is never a panic in release, and the family's probe cells and T-1776's
  metamorphic test catch it.
- **Recursion:** the driver adds none. It evaluates each argument once through
  `evaluate_node_in_context`/`evaluate_node_with_reference`, under their existing depth limits, and
  the kernel does not evaluate nodes.
- **Stack:** the read phase is non-generic and `#[inline(never)]`, so it is one frame of fixed size
  (T-1110). Only the kernel closure differs per function.

### Performance
- Unlifted calls (almost every cell) hold their values inline and go straight to the kernel. The
  kernel's casts are the ones it ran before, minus `evaluate_node_in_context`, which the read phase
  ran instead.
- Lifted calls read each argument once. A single-grid call is rewritten in place (step 5).
  Otherwise one reused buffer holds the slots: a `Number` element allocates nothing, and a `String`
  element clones its `Text`, as today.
- Benchmarks live in `crates/rustsheet/benches/` (`aggregate.rs`, `edit.rs` and `text.rs` exist).
  Every number goes in its task's Outcome, measured on main and on the branch in release:

  | Task | Bench | Budget |
  |---|---|---|
  | T-1784 | a lifted whole-column COUNTIFS (`=SUM(COUNTIFS(A:A,A1:A100000))`, the `reuse_equal_runs` path) and a *IF(S) bench | within 5% |
  | T-1773 | per-row unlifted cells over ≥ 100k rows, full recalc: `=ROUND(A{r},2)`, `=ABS(A{r})`, `=MOD(A{r},7)` | within 5% each |
  | T-1773 | lifted `=SUM(ROUND(A1:A1000000,2))` and `=SUM(ABS(A1:A1000000))` | within 5% |
  | T-1785 | per-row unlifted `=DAY(A{r})` over ≥ 100k rows | within 5% |
  | T-1775 | `benches/text.rs` | within 5% |
  | T-1372 | `=SUM(PMT(A1:A100000,10,100))` | none: a new path, a smoke number only |

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
  at build time, and the catalog has no ATP distinction. Rejected; the catalog is derived from the
  table instead (T-1776).
- **Keep the kinds in the parser's `Signature`** (add a `LiftArrays` variant there). The evaluator
  would depend on parser internals, and the signature has no `Whole`/`WholeByCaller` distinction.
  The table lives with `Function` instead, and static analysis derives `Signature` from it.
- **A trait per kernel** (`trait ScalarKernel`). This is more ceremony than a closure. Closures
  also hold per-call caches. SPEC-0009 made the same choice. Rejected.
- **A generic kernel parameter** (`K: FnMut`) instead of `&mut dyn FnMut`. It saves one indirect
  call per element, but copies the loop into every function and grows `fn_x` frames. Rejected
  (T-1110).

## Test plan
- **Probe cells:** the lift cells of the seven argument-mode cases above, and probes/
  scalar-lift-shapes, -b and -c. Each family task removes its lines from eval/baseline/probes.txt
  (`make eval-baseline`), with no new F lines anywhere. Value cells that are ulp misses (PMT,
  NORM.S.DIST: T-0901, T-1374) stay listed under those tasks.
- **Unit tests** in `functions/lift/` (T-1770), against `lift_values` with counting kernels and
  against formulas:
  - shapes: row × column grid; `#N/A` padding in 1-D and 2-D; a one-cell range is a scalar; a 1×1
    array lifts to a 1×1 result.
  - errors: an element error stays in its position; a scalar error and scalar text broadcast; the
    kernel's own check order decides per element (`INDEX(PMT({#NUM!},{#N/A},100),1)` is `#NUM!`, the
    reversed one `#N/A`). The driver does not pick the first error itself:
    `INDEX(YEARFRAC(1/0,{45000;45031},B1:B60),1)` is `#VALUE!` and
    `INDEX(YEARFRAC({45000;45031},B1:B60,NA()),1)` is `#N/A` (the range, an error and basis, which YEARFRAC reads first).
  - `LiftArrays`: a multi-cell range alone is one `#VALUE!` and the kernel is called once; next to an
    array, every element is `#VALUE!` in the array's shape; a reference from IF or INDEX is refused;
    a one-cell reference is its value.
  - `Whole`: never lifted, and evaluated once. A kernel counting calls and the identity of the whole
    value shows this.
  - the unlifted path calls the kernel exactly once and allocates no `LiftedArg` or grid.
  - the table-wins test: no function with an entry has a fallback arm.
- **Size bounds (directive 5).** The tests use explicit ranges on a fixture whose cells are filled,
  so a pass cannot come from blank cells:
  - `=ROWS(PMT(A1:A1048576+0,A1:B1+0,100))` is `#ERROR!` and the counting kernel is never called;
  - grids whose summed input cells exceed the cap are refused before the next grid is read;
  - a whole argument of 2^20 cells against 2^8 elements under `LiftCost::Cells` trips the work bound
    before the first element, while `Prepared` lets it run;
  - a `Weighted` entry is charged its weight.

  `clip_to_used_area` changes which cells `evaluate_range` reads, never the shape it returns or
  the bound it checks: `check_range_size` and `check_array_size` use the written shape (arrays.md
  "Array results above 1,048,576 cells", which documents the clipping). It is not a DEVIATION,
  because no result changes. T-1771 and T-1786 add the same kind of tests for their functions.
- **Nesting and stack:** `test_nested_formula_stack` gains a deeply nested lifted formula through a
  migrated function (`EDATE(EDATE(…{45000;45031}…,1),1)`, T-1770). T-1773 keeps that test's
  existing subject, ABS, passing after ABS moves onto the driver.
- **Static analysis:** legacy FORMULATEXT for `Lift` vs `LiftArrays` (`=PMT(@C1:C60,10,100)`,
  `=EDATE(A1:A60,1)`, `=EDATE(@A1:A60+0,1)`: -c E–H). Static shape `Array(2,1)` for
  `EDATE({1;2},A1:A3)` and `Scalar` for `EDATE(A1:A3,1)`. Each family task adds its functions to
  test_implicit_intersection_export.rs.
- **Metamorphic** (T-1776): for every table entry, the catalog's domains give one valid scalar
  call. The test evaluates it, then the same call with one lifted argument as `{v;v}`, and checks
  that both elements equal the scalar result. This catches a function that is in the table but not
  wired to `lift_call`, or the reverse.
- **No-change migrations** (T-1784, T-1773, T-1785, T-1774, T-1775): `make eval` shows 0 fixes and
  0 regressions, and the benches above hold.
- **No ad-hoc lift remains** (T-1775's acceptance, the last migration):
  - `rg -n "fn lift_numbers|fn lift_number_operands|fn lift_with_defaults|macro_rules! single_number_fn|fn lifted_arg|fn parity_predicate|fn broadcast_text|enum TextArg|fn lift_with_cost|fn is_lifted|enum CriteriaArg" crates/rustsheet-engine/src` finds nothing.
  - `rg -n "bcast_get|bcast_idx" crates/rustsheet-engine/src` finds only functions/lift/,
    arithmetic.rs and the non-goal rows of the inventory.
  - `scripts/check_duplication.py` does not rise, and each drop is recorded with `--update`
    (decision 0010).

## Rollout
Each step is one PR of size S/M. The old paths stay until their step.
0. **Directive-5 fixes, independent of the driver:**
   - **T-1771** (functions-text, P0, S): bound `broadcast_text`.
   - **T-1780** (functions-lookup, P0): bound HSTACK/VSTACK padding.
   - **T-1781** (functions-text, P0): bound TEXTSPLIT padding.
   - **T-1786** (functions-lookup, P0, M): bound EXPAND, WRAPROWS/WRAPCOLS and MMULT; check XMATCH.
   - **T-1803** (functions-lookup, P2, S): a stack with rows past the grid is `#NUM!`, columns past it are allowed (beside T-1780's bound).
1. **The driver:**
   - **T-1784** (engine, opus, S): move `lift_with_cost`, `CriteriaArg`, `count_walks` and
     `is_lifted` into `functions/lift/driver.rs` as `lift_values`, with 0 eval diff and the
     COUNTIFS/*IF(S) benches.
   - **T-1770** (engine, opus, M): the table, `lift_call`, the read phase, the static-analysis hooks
     and fallbacks, and the unit and size tests. EDATE, EOMONTH, WEEKNUM and YEARFRAC are the first
     consumers, `LiftArrays` throughout: date-argument-modes F/G 20, 22, 24, 26, 54, 56, 66, 68, and
     the EDATE and YEARFRAC cells of scalar-lift-shapes, -b and -c.
2. **After T-1770, in parallel (one lane each):**
   - **T-1800** (engine, S; the YEARFRAC kernel-order part can land before T-1770): kernel order
     for error precedence; the driver never scans for the first error. T-1372, T-1383 and T-1290 wait for it.
   - **T-1801** (engine, S): a 3-D range is `#REF!` in `Lift`, `#VALUE!` in `LiftArrays`. T-1372, T-1383
     and T-1290 wait for it.
   - **T-1802** (functions-math, S): probe, then enter QUOTIENT, FACTDOUBLE and SERIESSUM.
   - **T-1236** (functions-date, S): WORKDAY(.INTL), NETWORKDAYS(.INTL); `weekend` is `Lift`.
   - **T-1372** (functions-financial, M): later ATP arguments are probed before they are entered.
   - **T-1290** (functions-stat, M): the functions whose every argument is scalar.
   - **T-1772** (functions-stat, M): the first real `Whole` + `Prepared` consumer (LARGE k and the
     other scalar arguments next to a data array).
   - **T-1383** (functions-engineering, M): later arguments are probed before they are entered.
3. **T-1241** (CHOOSE's index moves in), **T-1271** (functions-lookup), **T-1389**
   (functions-criteria; adds `LiftArraysRangeWhole`).
4. **The old helpers, with no result change:**
   - **T-1773** (quality, opus, M): `single_number_fn!`, `lift_numbers`, `lift_with_defaults`,
     ROUND/MOD/CEILING.
   - **T-1785** (quality, opus, M): the date and time parts, DATE, DATEDIF, WEEKDAY, ISOWEEKNUM,
     MUNIT, MROUND, SQRTPI.
   - **T-1774** (quality, sonnet, S): IS* and ISEVEN.
   - **T-1775** (quality, opus, M; last, with the no-ad-hoc-lift grep): the text functions.
   - **T-1787** (quality, sonnet, S): the private `calc_result_to_array_node` copies.
5. **T-1776** (harness, sonnet, S, any time after T-1770): the derived catalog key, the
   cross-check, the `KernelWork` list check and the metamorphic test.
6. **T-1777** (research, sonnet, S, any time): probe the open questions. A finding changes a table
   entry or a kernel in a task of its own.
