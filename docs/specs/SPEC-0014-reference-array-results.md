# SPEC-0014 — Arrays of references: lifted OFFSET, INDIRECT, TRIMRANGE, TAKE and DROP

- **Status:** draft (accepted on merge of T-1955)
- **Milestone / lane:** M6 / engine
- **Tasks:** T-1955 (this spec), T-1956, T-1957, T-1838, T-2063, T-2064, T-2065

## Problem
A multi-cell range or an array in OFFSET's rows/cols/height/width, INDIRECT's ref_text or a1, or
TRIMRANGE's trim_rows/trim_cols lifts the call, and each element of the result is a **reference**
(lookup.md "OFFSET, INDIRECT and TRIMRANGE lift into arrays of references"). A lifted TAKE or DROP
over a reference does the same (SPEC-0011 "Elements that are references"). The engine has no
carrier for such a result: `ArrayNode` holds values only (SPEC-0011 non-goal 2) and OFFSET,
INDIRECT and TRIMRANGE do not lift. The engine fails 123 of the cells of T-1955's probes
(probes/reference-lift-consumers, -b1c, -b1d, -b2) and the cells T-1956, T-1957 and T-1838 own
(probes/reference-lift-elements, lookup-argument-modes F14:H15, F20:H21, F34:H35, m7-argument-modes
F4:I6, K1-K4), plus gen-m7/cb748b6a9b81fbb7 A28,
gen-m9/675763979c1defb6 A25 and gen-m9/adb3fd63c7368166 A34.

## Goals / non-goals
Goals: one carrier for all five producers (directive 3); every consumer the probes cover reads it as
Excel does; bounded work and memory (directive 5); nothing reaches cell storage.
Non-goals: nested *value* arrays (an array element that is itself an array of values; SPEC-0011
rule 2 keeps the top-left value); the statistical aggregates whose Excel results are not yet known
(open questions below); multi-area elements (a lifted call whose element is a union does not exist
among the five producers).

## Excel behavior to match
Evidence: probes/reference-lift-elements (T-1810), probes/reference-lift-consumers and
reference-lift-consumers-b1c, -b1d, -b2 (T-1955); all excel-graph, 2026-10-19. Data: A1 = 1, A2 = 100,
A3 = 10000, B1 = 2, B2 = 200, A4 = "5", A5 blank, A6 TRUE, A7 = 7, so a multi-cell element's top-left
cell is told from its sum. Cell ids are the consumers case's unless named.

1. **Shape.** The result has the lifted arguments' broadcast shape; ROWS/COLUMNS count elements
   (`ROWS(OFFSET(A1,{0;1},{0,1}))` 2, b2 E149; `ROWS(LET(x,OFFSET(..),x))` 2, b1c E81).
   Per element the scalar rules hold, and an element that fails is its error (`OFFSET(A1,{0;-1},0)`
   second element `#REF!`).
2. **Value view.** Wherever a value is read, every reference element is `#VALUE!`, an error element
   stays itself: spilled (`=OFFSET(A1,{0;1},0)`), through operators (`+1`, `-`, `&`, `=`: E33-E46,
   E137; `ROWS(..+1)` 2), IF, CHOOSE, FILTER, SORT, VSTACK and `INDEX(..,0,1)` when spilled
   (E85, E105, E121, E153-E161), ISERROR TRUE, ISNUMBER FALSE, ERROR.TYPE 3 (E93, E141, b2 E205),
   IFERROR's fallback (`IFERROR(..,"x")` spills x, b1d E113; `SUM(IFERROR(..,0))` 0, b2 E229), TEXTJOIN
   `#VALUE!` (b1c E65), `SUM(..*1)` `#VALUE!` (E49, E53), AGGREGATE `#VALUE!` (E213, b2 E173), XLOOKUP
   and MATCH find nothing (`#N/A`, b2 E217, E221).
   SHEET is neither view: `SHEET(OFFSET(A1,{0;1},0))` is one `#N/A`, not lifted and not a spill
   (b1d E101); T-2064 owns it.
3. **Reference view: INDEX.** `INDEX(r,i,j)` is element (i,j) itself: one cell is its value with ISREF
   TRUE (E81), a multi-cell one spills or is `#SPILL!` (reference-lift-elements); `SUM(INDEX(..,2,1))`
   reads the whole element (101, E177). With one index over a column of elements, `INDEX(r,2)` is
   `#VALUE!` (b1d E109). `ISREF` of the whole result is FALSE, through LET too (E209, b2 E209).
4. **Reference view: top-left aggregates.** SUM, AVERAGE, MIN, MAX and PRODUCT read each element's
   **top-left cell** as a range cell: text, booleans and blanks are skipped
   (`SUM(OFFSET(A1,0,0,{1;2}))` 2, `SUM(OFFSET(A1,0,0,{2;3}))` 2, `SUM(OFFSET(A2,0,0,{1;2}))` 200,
   `SUM(INDIRECT({"A1:A2";"B1:B2"}))` 3, `SUM(INDIRECT(C1:C3))` 103, MAX/MIN 1, AVERAGE 1 or 50.5,
   PRODUCT 1 and 100: E1-E25, E117, E125, E129, E133, E181, E189, E197, E201, E205, b2 E157;
   `SUM(OFFSET(A6,0,0,{1;2}))` 0, `MAX(OFFSET(A5,0,0,{1;2}))` 0,
   `MIN(OFFSET(A4,0,0,{1;3}))` 0, `SUM(OFFSET(A4,{0;3},0))` 7, `AVERAGE(OFFSET(A4,{0;1;2;3},0))` 7:
   b2 E117-E121, E161, E165, E201). An error element is that error (`AVERAGE(OFFSET(A1,{0;-1},0))`,
   `MAX(..)` and `SUMPRODUCT(..)` `#REF!`, `SUM(INDIRECT({"A1";"Nope"}))` `#REF!`: b2 E181-E189, E213). The grid may be 2-D (`SUM(OFFSET(A1,{0;1},{0,1}))` 303,
   b2 E145) and sits beside other arguments (`SUM(..,1)` 102, b1d E105).
5. **Reference view: not numbers.** COUNT counts no element (0, E13, E193); COUNTA counts every
   element, a blank one too (2, E17, b2 E169); SUMPRODUCT reads an element as 0 (E29), an error
   element as its error (b2 E189).
6. **Reference view: per-element calls.** A function that takes a reference argument lifts over
   the elements, calling itself once per element with the whole element: SUBTOTAL (9 and 109; with
   further arguments too, `SUBTOTAL(9,r,A1)` 2, 101), COUNTIF, SUMIF (sum_range a
   grid too: 2, 202), SUMIFS, COUNTBLANK, AREAS, ROW, COLUMN, `CELL("row",..)`, ISFORMULA, N and T
   (E57-E78, E97, E109, E165, E169, E185; b1c E85-E93; b1d E97; b2 E125-E133, E177, E193, E197).
   `SUMPRODUCT(SUBTOTAL(9,OFFSET(A1,{0;1;2},0)))` is 10101 (E113).
7. **Pass-through.** IF, CHOOSE, LET, VSTACK, TAKE and DROP in a reference position hand the
   elements on: `SUM(IF(TRUE,r))`, `SUM(CHOOSE(1,r))`, `SUM(LET(x,r,x))`, `SUM(VSTACK(r))` are 101
   (b1c E69-E77, b2 E225), `LET(x,r,INDEX(x,2,1))` 100 (E89), `SUM(TAKE(r,1))` 1, `SUM(DROP(r3,1))`
   10100 (E101, b2 E153). A producer whose reference argument is a grid lifts again:
   `ROWS(OFFSET(OFFSET(A1,{0;1},0),1,0))` 2, `SUM(..)` 10100 (E145, E149), `SUM(OFFSET(A1:A2,{0;1},0))` 101.

Open (the Excel-for-web session failed, "retries exhausted", on every workbook holding these,
twice): MEDIAN, STDEV.S, VAR.S, LARGE, SMALL, COUNTIFS, AVERAGEIF, MAXIFS, SUMSQ, AVERAGEA, MAXA and
CONCAT over a reference grid (T-2065). Until answered, these keep today's behaviour of a value view.

## Design

### Representation: `RefGrid`, outside `CalcResult`
```rust
// reference_operators.rs (beside Areas, SPEC-0004)
/// One element of an array of references: the reference a scalar call returned, or its error.
pub(crate) enum RefElement { Reference(Range), Error(Error) }

/// A lifted call whose elements are references. 1..=check_array_size rows x columns, row-major,
/// every row as wide; a Reference is on one sheet with its corners in order. Shared (Arc), so a
/// LET binding or a pass-through is never copied.
#[derive(Clone)]
pub(crate) struct RefGrid { rows: usize, columns: usize, elements: Arc<[RefElement]> }

pub(crate) enum RefArg { Ranges(Ranges), Grid(RefGrid), Value(CalcResult) }
```
`RefGrid` is **not** a `CalcResult` variant, for the reason SPEC-0004 gives for `Areas`: hundreds
of `_ =>` arms would treat it as some value, and rule 2 (every element `#VALUE!` to a value
reader) would break silently. It reaches only code that asks for `RefArg`, and the compiler
enforces that. It is not an `ArrayNode` variant either: `ArrayNode` is the parser's array-constant
type and the spill payload, so a reference there could reach cell storage, where structural edits
would not move it.

- **Value view, one definition.** `RefGrid::to_values(&self) -> CalcResult` gives a
  `CalcResult::Array` with `#VALUE!` (empty message) per reference element and the error per error
  element. `RefArg::into_single_area` and every value reader go through it. `evaluate_node_in_context`
  of a producer returns that array, so a spill, operator, IS* function or IFERROR needs no change
  (rule 2).
- **Producers.** OFFSET, INDIRECT, TRIMRANGE, TAKE and DROP each get a reference form
  `fn x_reference(&mut self, args, cell) -> RefArg`; their value form is `x_reference(..)` then the
  value view. A producer whose arguments do not lift stays on today's path, allocation-free:
  `RefArg::Ranges(Ranges::One)` goes straight to `CalcResult::Range`, and T-1956 times scalar OFFSET
  and INDIRECT recalc before and after (no bench suite exists yet). The lift is SPEC-0011's driver: the kernel returns the scalar call's `CalcResult::Range`
  or error, and a new `CallResult::Reference` mode collects those into a `RefGrid` instead of
  `ArrayNode`s (one broadcast, directive 3). `evaluate_reference_form`
  (lookup_and_reference/mod.rs) and `evaluate_reference_arg` (util.rs) dispatch the five producers
  to these forms. A producer's reference argument that is itself a `RefGrid` lifts again over its
  elements (rule 7).
- **Consumers** opt in, as SPEC-0004's do; anything that does not ask sees the value view:
  - INDEX (`index_impl`): picks the element; a whole row/column (`0`) is a sub-grid; one index over
    a grid is `#VALUE!` (rule 3).
  - ROWS/COLUMNS (`rows_columns_extent`): the grid's shape. ISREF (`is_reference`): FALSE.
  - The aggregate walker (SPEC-0009, `for_each_value` and the count walkers): a `RefGrid` argument
    is walked element by element; the descriptor of each aggregate says `TopLeft` (SUM, AVERAGE,
    MIN, MAX, PRODUCT: the element's top-left cell read as a range cell) or `NotANumber` (COUNT
    skips, COUNTA counts, SUMPRODUCT reads 0). An error element is the result. Others keep the
    value view until T-2065 answers them.
  - Per-element callers (rule 6): SUBTOTAL, COUNTIF, SUMIF, SUMIFS, COUNTBLANK, AREAS, ROW, COLUMN,
    CELL, ISFORMULA, N, T. One helper, `Model::lift_over_ref_grid(grid, cell, |model, elem| ..)`,
    runs the scalar call per element and builds a `CalcResult::Array` (a SUMIF whose range and
    sum_range are both grids broadcasts them as SPEC-0011 does). Its cost is the consumer's:
    `LiftCost::Unit` for ROW, COLUMN, AREAS, CELL, ISFORMULA, N and T (O(1) per element);
    `LiftCost::Cells` of each element's clipped cells for SUBTOTAL, COUNTIF, SUMIF and COUNTBLANK, and
    cells times criteria pairs for SUMIFS and COUNTIFS (as lookup_each.rs charges). Past the budget
    the call is the bound's error where Excel computes a value
    (`SUBTOTAL(9,OFFSET(A:A,0,0,SEQUENCE(1000)))`): T-2064 records that DEVIATION (decision 0014)
    when it lands the charge.
  - Pass-through (rule 7): IF and CHOOSE in reference form, LET (`Binding::Grid(RefGrid)`, an `Arc`
    clone), VSTACK in a reference position (T-2064), TAKE and DROP (T-1838). `VariableStack` gets
    its own held-elements budget for grids, `MAX_HELD_GRID_ELEMENTS = 4 * MAX_SIZE` (2^22), apart
    from the union cap (`MAX_HELD_AREAS`, 2^18, which would refuse a legal 2^20-element grid). Only a
    newly built grid is charged; an `Arc` clone (`LET(x,r,y,x,..)`) costs no memory and is not. The
    budget bounds VSTACK doubling a bound grid at each binding; past it the binding is `#VALUE!`, a
    DEVIATION that T-1956 records under lookup.md (decision 0014), as Excel's limit is not known.

### Bounds (directive 5)
- Element count: every producer, one over a grid included, and every per-element consumer that
  broadcasts two grids (SUMIF's range and sum_range) checks the broadcast shape against
  `check_array_size` before any element is computed, as every lift does. A grid argument (n x 1)
  beside a lifted argument (1 x m) gives n x m, so the check is on the output shape, not the input.
  A `RefElement` is at most 32 bytes (T-1956 asserts its size), no larger than the `ArrayNode` it
  replaces. A `Reference` is built only from a scalar call's in-sheet result (OFFSET's and
  INDIRECT's own bounds checks run per element), so no consumer sees an off-sheet corner.
- Work: each element is one call charged to the lift's `MAX_LIFTED_CELL_VISITS` budget. A producer
  over a grid is one lift whose elements are the grid's, so nesting (`OFFSET(OFFSET(..),..)`) costs
  one budget per level, at most the parser's depth cap times the budget in all; T-1956 tests a deep
  nest. Whether the budget becomes one per formula evaluation (as `MAX_AREA_WALK` is) is left to
  SPEC-0011's owner; this spec adds no second budget. A
  per-element consumer (rule 6) charges the clipped cells of each element it walks to the same
  budget before walking (`LiftCost::Cells`), so `SUBTOTAL(9,OFFSET(A:A,0,0,SEQUENCE(1000)))` is
  `#ERROR!`-bounded rather than 10^9 reads. A top-left aggregate reads one cell per element.
- Nesting: recursion follows the formula tree, so the parser's depth cap bounds it. A grid may grow
  at a level, by broadcast with another lifted argument (`OFFSET(OFFSET(A1,{0;1},0),{0,1,2},0)` is
  2x3) or by VSTACK concatenating grids; every grid built passes `check_array_size` (VSTACK:
  `check_stack_size`) on its full broadcast or stacked shape, the grid argument's own shape
  included, before any element runs.
- No panics: element access is by checked index; an empty grid is never built (shape ≥ 1x1).

### What changes for existing formulas
Nothing that does not lift: a producer with scalar arguments still returns `CalcResult::Range`
(`RefArg::Ranges`). The cell-storage boundary is unchanged, because a `RefGrid` never becomes a
`CalcResult`.

### Dependency graph
A `RefGrid` is the result of OFFSET/INDIRECT (volatile) or TAKE/DROP/TRIMRANGE (whose dependencies
are their arguments'), so it adds no static dependency: the per-element consumers of a TAKE, DROP or
TRIMRANGE grid read cells inside the argument's static range, and an OFFSET or INDIRECT grid depends
through volatility. T-0401 carries a pointer to this paragraph.

## Alternatives considered
- **A `CalcResult::References` variant.** Every match on `CalcResult` would need an arm and every
  `_ =>` arm would silently misread it (SPEC-0004's argument). Rejected.
- **`ArrayNode::Reference`.** Reuses broadcast and spill code, but puts references into the parser's
  constant type and the spill payload (storage), and into 456 `ArrayNode::` sites. Rejected.
- **A lazily evaluated lifted node** that INDEX, ROWS and SUM evaluate per element when asked. No new
  type, but each consumer re-runs the producer per element (OFFSET twice under `SUM(INDEX(..))`), and
  LET could not bind it without keeping the node and its scope alive. Rejected.

## Test plan
- The probe cells above, plus reference-lift-elements and the m7 / lookup-argument-modes cells
  each rollout task owns: they leave eval/baseline as each task lands.
- Unit tests per task: a grid never reaches storage (spill of a grid is `#VALUE!` cells); a lift past
  `check_array_size` or the visit budget is the bound's error before any element runs;
  `RefGrid::to_values` keeps error elements.
- Metamorphic: `SUM(OFFSET(A1,{0;1},0))` equals `SUM(A1,A2)`; `ROWS` of any producer grid equals the
  broadcast shape of its lifted arguments.

## Rollout
1. **T-1956** (opus, M): `RefGrid`, `RefArg::Grid`, the value view, `CallResult::Reference` in the
   driver, OFFSET as producer; INDEX, ROWS/COLUMNS, ISREF and LET/IF/CHOOSE pass-through.
2. **T-1957** (M): INDIRECT and TRIMRANGE as producers, on T-1956's carrier.
3. **T-1838** (M): TAKE and DROP as producers (SPEC-0011 "Elements that are references").
4. **T-2063** (M): the aggregate walker's `TopLeft`/`NotANumber` rules (rules 4, 5).
5. **T-2064** (M): per-element callers (rule 6) through `lift_over_ref_grid`, and VSTACK's pass-through.
6. **T-2065** (research, S, any time): probe the open aggregates one formula per workbook (the
   combined workbooks failed to open).
