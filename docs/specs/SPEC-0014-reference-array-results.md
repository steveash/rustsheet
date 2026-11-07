# SPEC-0014 — Arrays of references: lifted OFFSET, INDIRECT, TRIMRANGE, TAKE and DROP

- **Status:** draft (accepted on merge of T-1955)
- **Milestone / lane:** M6 / engine
- **Tasks:** T-1955 (this spec), T-1956, T-1957, T-1838, T-2063, T-2064, T-2065; amendment "Mixed grids": T-4569 (spec),
  T-4570 (implementation)

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
   `#VALUE!` (b1d E109): it is a 1x1 array of references holding element 2, shown by its value view, and where a
   reference is taken it stays one, so `INDEX(INDEX(OFFSET(A1,{0;1},0,2),2),1,1)` spills A2:A3, ROWS of the one-index
   INDEX is 1, ISREF of the nested INDEX TRUE, and `SUM` of the one-index INDEX reads only the element's top-left
   cell, as rule 4 does (2, not 6); INDIRECT and the mixed XLOOKUP grid alike (probes/t4701-index-one-index-ref-grid,
   T-4701; `index_reference_arg`, `column_element`). `ISREF` of the whole result is FALSE, through LET too (E209, b2
   E209).
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

T-2065 answered the open list, one formula per workbook (probes/t2065-lift-*, excel-graph 2026-10-26, data as in
b2; E1 holds the formula):
- Aggregates over all elements: `STDEV.S(OFFSET(A1,{0;1},0))` 70.0035713374682 and `VAR.S(..)` 4900.5 (the values 1 and
  100), `SUMSQ(OFFSET(A1,{0;1},0))` 10001. Every element is one cell there, so top-left cell versus whole element
  was probed by T-2816: only the top-left cell of a multi-cell element counts (`STDEV.S(OFFSET(A1,0,0,{1;2}))` 0,
  `SUMSQ` 2 (top-left, not row intersection: `SUMSQ(OFFSET(A2,0,0,{1;2}))` in E1 is 20000); AVERAGEA/MAXA: text 0, blank and TRUE skipped, unlike a reference argument where TRUE is 1; docs/excel-semantics/lookup.md).
- Per-element callers (rule 6), each spilling one value per element: `COUNTIFS(OFFSET(A1,{0;1},0),">0")` 1, 1;
  `AVERAGEIF(OFFSET(A1,0,0,{1;2}),">0")` 1, 50.5; `MAXIFS(OFFSET(A1,{0;1},0),OFFSET(A1,{0;1},0),">0")` 1, 100.
- `CONCAT(OFFSET(A1,{0;1},0))` is `#VALUE!` (a value view, as TEXTJOIN in rule 2).
- `AVERAGEA(OFFSET(A4,{0;1;2},0))` and `MAXA(..)` are 0 with A4 the text "5", A5 blank and A6 TRUE: the
  result is 0: not 5 and not 1, but which elements count is open, as the data holds no number (follow-up task).
- Failed to open on Excel for the web on 2026-10-26 (createSession "retries exhausted", two runs, alone in a
  workbook): MEDIAN, LARGE and SMALL over a lifted OFFSET. Desktop question T-1075 item 18 (decision 0039); until
  answered they keep today's value view. Answered since, on the web: `#VALUE!` (lookup.md "MEDIAN, LARGE and SMALL
  over a lifted OFFSET", probes/t2065-lift-median, -large, -small, 2026-10-29). T-4569 found that holds only when
  the references are on the formula's own sheet. Over references on another sheet the three are `#N/A`
  (probes/t4569-xlookup-ref-element-max-min: t2065's formulas on its own data sheet Local!E1:E3 `#VALUE!`; the same
  read from another sheet Local!E4:E5, Calc A31:A32, A40:A44 `#N/A`; one fill, 2026-11-06). The amendment's
  rule 5 carries this.

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
    CELL, ISFORMULA, N, T; COUNTIFS, AVERAGEIF and MAXIFS since T-2823 (T-2065's answers). One
    helper, `Model::lift_over_ref_grid(grid, cell, |model, elem| ..)`,
    runs the scalar call per element and builds a `CalcResult::Array` (a SUMIF whose range and
    sum_range are both grids broadcasts them as SPEC-0011 does). Its cost is the consumer's:
    `LiftCost::Unit` for ROW, COLUMN, AREAS, CELL, ISFORMULA, N and T (O(1) per element);
    `LiftCost::Cells` of each element's clipped cells for SUBTOTAL, COUNTIF, SUMIF and COUNTBLANK, and
    cells times criteria pairs for SUMIFS, COUNTIFS and MAXIFS (as lookup_each.rs charges). Past the budget
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

## Amendment: mixed grids, value elements beside references (T-4569)

### Problem
XLOOKUP with an array lookup_value and a range if_not_found returns a grid whose hits are values and whose misses
are the range itself, a real reference (lookup.md "XLOOKUP's if_not_found and ADDRESS's sheet_text over an array";
probes/t4563-xlookup-not-found-ref-element, probes/t4569-xlookup-ref-element-max-min, excel-graph, 2026-11-06).
`RefElement` holds only references and errors, and SPEC-0018's nested cell is not a reference (ISREF, TYPE, AREAS,
CELL fail), so neither carrier can hold this result. The driver turns a kernel's non-reference value into
`#VALUE!` (`functions/lift/driver.rs`, `CallResult::Reference`).

### Excel behavior to match
`X(r)` is `XLOOKUP({2;9},A1:A3,B1:B3,r)`: element 1 is the hit 20, and element 2 is `r`. Cells are t4563's unless
marked t4569. Every rule of this spec holds for the reference element unchanged. The value element is read as
an array value. In detail:
1. **Value view** (rule 2). The grid shows 20, `#VALUE!`. A lifted operator gives 20, `#VALUE!`, even for a one-cell
   `r` (A46, A57:A70). ISERROR is FALSE, TRUE; ISNUMBER TRUE, FALSE (t4569 C30:D30); ISTEXT FALSE, FALSE; TYPE of
   the result 64; ISREF FALSE (A85-A105, t4569 A29).
2. **INDEX** (rule 3). `INDEX(X(r),2,1)` is the reference: ISREF TRUE, TYPE 64 or 1, AREAS 1, CELL names its cell,
   SUM 7, it spills (A1:A53), and `MAX(INDEX(..,2,1))` is 60 (t4569 A17). `INDEX(X(r),1,1)` is 20. One-argument
   `INDEX(X(r),2)` is a 1x1 grid holding the element: it shows `#VALUE!`, and `INDEX(that,1,1)` hands the range back
   (A25, V25:W25). Rule 3's "`INDEX(r,2)` over a column of elements is `#VALUE!`" is this case seen through the value
   view.
3. **Top-left aggregates** (rule 4, extended). SUM, AVERAGE, AVERAGEA, MIN, MAX, MINA, MAXA, PRODUCT, SUMSQ and STDEV.S read
   the reference element's top-left cell as the reader's `GridReading` reads a top-left cell, and the value element
   as an array value: SUM 23 = 20 + F1,
   25 = 20 + E1 (A73, A109); over G1:G2 = 50, 60 MAX 50, AVERAGE 35, SUM 70, STDEV.S 21.2132…; over H1:H2 = 1, 2
   PRODUCT 20, SUMSQ 401 (t4569 A1-A15, A20, A23). MIN and MINA there are 1 under either reading, so they confirm
   nothing. A text or TRUE value element is read as `TopLeftElements` reads a top-left cell, that is as an array
   element: SUM, AVERAGE and COUNT skip both (50, 50, 0), AVERAGEA counts text as 0 and skips TRUE (25, 50), and
   MAXA and MINA skip both (50, 50) (t4569 A50:A63, the hit "t" or TRUE beside G1:G2's element).
4. **Not numbers** (rule 5). COUNT 1 (the value 20 counts, the reference element does not), COUNTA 2, SUMPRODUCT 20:
   the reference element is 0 (A77, A81, t4569 A9, A10, A21).
5. **Order statistics: the sheet decides.** LARGE, SMALL and MEDIAN over a grid holding a reference element give
   `#N/A` when its references are on another sheet than the formula's. This covers a mixed grid (t4569 A12, A13,
   A22) and a grid of references alone, from OFFSET or INDIRECT (A31, A32, A40:A44, Local!E4:E5, Local!F6). They give
   `#VALUE!` when the references are on the formula's own sheet (t2065-lift-*, Local!E1:E3; data and formulas as
   t2065, same fill), however the reference is written: sheet-qualified absolute `Local!$A$1`, qualified relative
   `Local!A1`, unqualified absolute `$A$1`, or INDIRECT of unqualified texts (t4569 Local!F1:F5), and for a mixed grid
   read on its own sheet, unqualified or qualified (Local!F7:F8). So it is the referenced sheet, not the written
   qualifier, that decides (t4569 Local!F1:F9, predictions in docs/research/t4569/predictions.md). A mixed grid on
   its own sheet keeps rule 3 for the top-left aggregates: `MAX` is 100, the reference's top-left cell (Local!F9).
   A grid with references on both sheets: the sheet of its first element decides, `#VALUE!` when that is the
   formula's own sheet, `#N/A` when it is another (t4697 Local!H1:H9, H16, H17; `{"A1";"Data!G1"}` is `#VALUE!`,
   `{"Data!G1";"A1"}` is `#N/A`). A mixed XLOOKUP grid has one reference element, so its sheet decides (H14, H15).
6. **Per-element callers** (rule 6). They lift over every element. A value element is passed as the scalar
   non-reference argument it is: ROW, AREAS, COUNTIF, SUBTOTAL, COUNTBLANK and CELL give `#VALUE!` for it, and N
   gives 20. The reference element is the call on the range: ROW 1, AREAS 1, COUNTIF 2, N 50, SUBTOTAL 110,
   COUNTBLANK 0, CELL 1 (t4569 C24:D34).

### Design
- **Representation.** One more variant, `RefElement::Value(ArrayNode)`. The node is a scalar: `Boolean`, `Number`,
  `String` or `Empty`. An error stays `RefElement::Error`. A `Lambda` or `Nested` value is stored as `Error(VALUE)`,
  as the driver stores every non-reference value today. `RefElement` must stay within its 32-byte size assertion
  (T-1956). If `ArrayNode` makes it larger, box the value (`Value(Box<ArrayNode>)`), since value elements are rare.
  T-4570 measures this and keeps the assertion. `RefGrid` and its `Arc<[RefElement]>` are otherwise unchanged. It is still not a `CalcResult`, and it
  still never reaches storage. Rejected alternatives: `ArrayNode::Reference`, for the reasons above; and a separate
  `MixedGrid` type, which would duplicate every consumer (directive 3).
- **Producer.** The driver's `CallResult::Reference` mode keeps a scalar value as `Value`, replacing today's
  `#VALUE!` (driver.rs). OFFSET, INDIRECT, TRIMRANGE, TAKE and DROP never return a non-reference value, so their
  grids are unchanged. T-4570 confirms this by running their probes unchanged. XLOOKUP gets a reference form
  (`evaluate_reference_form` dispatch), used only when lookup_value lifts and if_not_found is a `CalcResult::Range`.
  There `xlookup_if_not_found` keeps the range, and each miss returns it, so the grid gets one `Reference` element
  per miss. Otherwise XLOOKUP stays on its value path. When every element is a hit, the reference form returns
  today's `CalcResult::Array` and builds no grid: a grid always holds at least one `Reference` element.
- **Readers.** Each consumer adds one arm, and none needs a new walker.
  - `to_values`: `Value(v)` gives `v`. This is the only value view (rule 1).
  - INDEX (`index_grid`): `Value(v)` gives `v` as a value, not a reference. The one-index form is still a 1x1
    sub-grid, so `INDEX(INDEX(X(r),2),1,1)` gives the range (rule 2).
  - The aggregate walker (`GridReading`): `TopLeft` and `TopLeftElements` both read `Value(v)` as an array
    element, the reading `TopLeftElements` already gives a top-left cell. So SUM, AVERAGE and COUNT skip text and
    booleans, AVERAGEA counts text as 0 and skips TRUE, and MAXA and MINA skip both (rule 3, t4569 A50:A63).
    `NotANumber` counts a number value for COUNT and reads it for SUMPRODUCT.
  - Order statistics (rule 5): LARGE, SMALL and MEDIAN given a `RefArg::Grid` with a reference element return
    `#N/A` if the first reference element is on a sheet other than the formula's, and `#VALUE!` if it is on the
    formula's own sheet (probed, rule 5; t4697 H1:H6, H9, H16, H17 separate the first element from the last, any and
    the majority). T-4570 implemented "any reference element on another sheet"; T-4714 changes it to the first.
    This is one scan of the elements in their argument reader. Probed (probes/t4704 A46:C61, T-4704): the
    first element in order decides, an error or a reference, so `{2;9}` (error first) gives the error and `{9;2}`
    (reference first) gives `#N/A` (A49:C49, A58:C58, SMALL A88:C88); MAX takes the error in either order
    (A91:C91). Implemented by T-4741.
  - Per-element callers (`ref_lift.rs`): `Value(v)` calls the scalar function with `v` as a value argument. That is
    `#VALUE!` where a reference is required, and N reads the value. `put_back` keeps the result.
    Probed (probes/t4704, T-4704): OFFSET over a mixed grid spills a 2x1 of `#VALUE!` (A4:C8; matched by T-4739, which reads XLOOKUP in its reference form in `offset_source`);
    FORMULATEXT (`#VALUE!`, `#N/A`) and INDIRECT (`#REF!`, `#VALUE!`) match; COUNTIF and SUMIF over an XLOOKUP with a
    text if_not_found spill a 2x1 of `#VALUE!`, with a miss or all hits (A64:C74). So does any lifted XLOOKUP,
    with or without an if_not_found (probes/t4742-countif-array-range-arg): a value element is `#VALUE!`, an error element
    its error. Implemented by T-4742 (`Model::xlookup_reference_elements`).
  - ROWS/COLUMNS, ISREF and the pass-throughs (IF, CHOOSE, LET, VSTACK, TAKE, DROP) need no change: they handle
    elements without reading them.
  - **Readers already on the walker, unprobed for mixed grids.** VAR.S, VAR.P, STDEV.P (`GridReading::TopLeft`,
    variance.rs) and VARA, VARPA, STDEVA, STDEVPA (`TopLeftElements`) take the walker's `Value(v)` arm above with no
    change of their own, as STDEV.S (probed, rule 3) does. Their mixed-grid cells are unprobed.
  - **Unprobed readers** off the walker fall back to the value view, as rule 2 makes every reader that does not opt
    in. This covers MODE, PERCENTILE, QUARTILE, the IFS family's criteria ranges, and TEXTJOIN. Each is a
    probe task before it opts in. RANK, RANK.EQ and RANK.AVG are probed (probes/t4704 A25:C38, T-4704): they lift per
    element, `#VALUE!` for the hit and the rank in the referenced range for the miss, so they do not fall back
    (matched by T-4740).
- **Bounds** (directive 5). No new budget. An XLOOKUP grid has the lifted lookup_value's shape, which the lift
  already passes through `check_array_size`. Each element is one `xlookup_one` call charged to
  `MAX_LIFTED_CELL_VISITS`, as today. A miss stores one `Range` (the shared if_not_found) and copies no cells. A
  top-left aggregate reads one cell per reference element, and a value element costs O(1). The order-statistic
  check scans the elements once, bounded by the grid's checked size. A per-element caller charges
  `LiftCost::Cells` for a reference element, as rule 6 does, and `LiftCost::Unit` for a value element. A
  `Value(String)` holds a text the kernel already built, so there is no extra allocation beyond today's
  `ArrayNode`.
- **What changes.** XLOOKUP under a lifted lookup_value with a range if_not_found, and LARGE, SMALL and MEDIAN over
  a grid whose first reference is on another sheet (`#N/A` replaces today's `#VALUE!`). Their same-sheet `#VALUE!` (the
  t2065 cells, which pass today) stays. Nothing else.

### As implemented (T-4570)
Where the implementation departs from the text above, or settles what it left open:
- **Representation.** `RefElement::Value(ArrayNode)` unboxed: `RefElement` stays within 32 bytes
  (`test_xlookup_ref_element.rs`). `RefElement::as_node` is the one element-to-array conversion
  (`to_array`, `to_values` and INDEX's one-index form read it).
- **Producer.** `lift_references_costed` charges `cells` visits per element, so XLOOKUP's grid costs what its value
  lift does (`LiftCost::Cells` of the lookup vector's scanned cells). The reference form is
  `Model::xlookup_reference(args, cell, as_reference)`; it is read by `reference_arg_read` (INDEX, LARGE, SMALL),
  `picked_reference_arg_read` (the aggregates), `value_or_grid` (SUMPRODUCT) and the per-element read-ahead.
- **Keyed lookups.** An XLOOKUP whose `if_not_found` is a reference is not a keyed lookup (SPEC-0020 Amendment 2,
  `tailed_array.rs keyed_lookup`): its misses are references, which the head-and-tail reading cannot hold, so a
  whole-column key builds its grid in full.
- **INDEX, one index.** One index into a one-column grid that picks a reference element is a 1x1 sub-grid where a
  reference is taken (`Model::index_reference_arg`, `column_element`; T-4701), so `INDEX(INDEX(X(r),2),1,1)` hands
  the range back (t4563 V25:V26), and its value view everywhere else (A25, T25:U25, W25:Z25). A picked value or
  error element stays a 1x1 value array (`INDEX(INDEX(X(r),1),1,1)` is 20, t4701 C4). One index into a grid of
  several columns stays `#VALUE!`.
- **Per-element callers.** `lifts_into_reference_elements` adds an XLOOKUP with an `if_not_found` to OFFSET and
  INDIRECT as the argument read ahead; one that gives no grid is put back as its reference form's result. The
  static analysis gives such a call the lookup_value's shape, so `COUNTIF(X(r),">0")` spills (t4569 A26). VSTACK's
  reference form keeps OFFSET and INDIRECT only.
- **Order statistics.** `RefGrid::order_statistic_error` is the one scan: `None` (the value view, `#VALUE!`) when
  the first element is a reference on the formula's own sheet, else the first error element, else `#N/A`, when any
  reference element is on another sheet (T-4714). LARGE and SMALL read it in their data reader (`numbers_from_ranked_arg`;
  the RANK family does not), MEDIAN through `GridReading::OrderStatistic`.
- **RANK family (T-4740).** `numbers_from_rank_ref_arg` hands any array of references back whole
  (`RankData::PerElement`), and `rank_per_element` lifts over its elements as OFFSET over a grid does
  (`RefGrid::index_array`): a reference element ranks against its own range, a value element is `#VALUE!`, an
  error element is itself. This covers a lifted OFFSET or INDIRECT as `ref` too, verified by
  probes/t4801-rank-lifted-offset-indirect (before T-4740 it was the value view, `#VALUE!` per element).

### Test plan and rollout
T-4570 implements it: the t4543 W45:W46, t4563 and t4569 cells it owns leave eval/baseline. Unit tests: a mixed
grid never reaches storage; the size assertion holds; an all-hit lift builds no grid. Metamorphic tests:
`SUM(X(r))` equals `SUM(hits) + top-left(r)` times the misses; `COUNTA(X(r))` equals the element count.

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
   As built, T-1956 lifts through a separate driver entry, `Model::lift_references`, sharing the shape check
   with `lift_grids` (its output is `RefElement`s, not `ArrayNode`s), and moved the LET/IF/CHOOSE pass-through,
   the held-grid budget, a producer over a grid reference argument and the deep-nest test to T-2078.
6. **T-2065** (research, S, done): probed the open aggregates one formula per workbook (answers above;
   MEDIAN, LARGE and SMALL never open on Excel for the web).
