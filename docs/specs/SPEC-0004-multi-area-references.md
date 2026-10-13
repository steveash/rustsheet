# SPEC-0004 — Multi-area references (unions) in the engine

- **Status:** accepted (on merge of T-0133)
- **Milestone / lane:** M1 / engine (probes in research)
- **Tasks:** T-0133 (this spec); rollout T-1061, T-1062, T-1063, T-1064, T-1065

## Problem
T-0134 made the union operator `(A1,B2)` parse (`Node::OpUnionKind`) and taught the intersection
to work on lists of areas (`reference_operators.rs`: `union_areas`, `intersect_areas`,
`MAX_AREAS`). It deliberately gave the engine no multi-area *value*: wherever a union reaches
anything but an intersection, `get_union` answers `#VALUE!`. Today:
- `=SUM((A1,B2))`, `=COUNT((A1,B2))` and every other aggregate of a union are `#VALUE!`, so
  `IFERROR(SUM((A1,B2)),0)` silently gives 0.
- `AREAS` always answers 1 (`address_areas.rs`, an IronCalc stub) and `INDEX` treats any
  `area_num` other than 1 as `#REF!` (`lookup_and_reference/mod.rs`).
- A union bound by `LET` or returned by `IF`/`CHOOSE` is `#VALUE!` before the caller sees it.
- A defined name whose body is a union does not load: `parse_defined_names` (`new_empty.rs`) keeps
  only cell, range, LAMBDA and error-literal bodies. `_xlnm.Print_Area` with two areas
  (`Sheet1!$A$1,Sheet1!$B$2`) is the common real-world case. probes/union-defined-name H1:H5
  (T-0187) fails all 5 cells.
- One function, AGGREGATE, already takes unions, by looking at the syntax tree
  (`aggregate.rs`, `Node::OpUnionKind => self.reference_areas(..)`). That works only for a union
  written in place, not one behind a name or a `LET` variable.

This spec settles the representation before code (CLAUDE.md directive 4), so that AREAS, INDEX,
the aggregates, LET/IF/CHOOSE and names all share one path instead of growing more
syntax-tree special cases.

## Goals / non-goals
Goals:
- One representation for "a reference of several areas", reachable from a union written in place,
  a defined name, a `LET` variable and the reference-returning functions (`IF`, `CHOOSE`, `INDEX`).
- A function contract: a function *opts in* to multi-area arguments. Every other position keeps
  today's `#VALUE!`, so the rule "a union where a value is needed is `#VALUE!`" (VERIFIED,
  references.md) holds by construction, not by review.
- Multi-area defined-name bodies, including `_xlnm.Print_Area`.
- A probe list for every Excel behaviour this needs that has no evidence yet.

Non-goals:
- Changing how a union parses or prints in a cell. FORMULATEXT behaviour is VERIFIED (T-0187,
  T-0193, T-1024). The comma rule is an engine choice from T-0134 (probe row 19). This spec only
  restates both, and adds the name-body entry point.
- Spilling or displaying a multi-area value. A union as a cell result stays `#VALUE!` (VERIFIED).
- Raising the area-count limits. `MAX_AREAS` and `MAX_AREA_PAIRS` (references.md "Union area
  cap", DEVIATION) bound every list this spec creates too. The spec adds a work bound for walking
  areas and a per-workbook bound for name bodies (see "Errors and limits").
- The dependency graph (SPEC-0002, T-0401). It must give each area of a union its own range
  vertex, and that requirement is recorded here, but nothing in this rollout needs the graph.

## Excel behavior to match
VERIFIED (docs/excel-semantics/references.md, "Union (comma) reference in a value context is
#VALUE!" and "Union: parsing and evaluation"):
- A union where a value is needed is `#VALUE!`: as a cell result, as an operand of arithmetic or
  `&`, nested, even `(E4,E4)` (gen-m1/c940d3cfb0d3b083).
- An intersection that cuts a union down to one area gives that area. Several areas left are
  `#VALUE!` in a value context (probes/intersection-function-operands H5).
- An operand's own error comes before the union: `=(A1,#REF!)` is `#REF!` and
  `ISREF((A1,#REF!))` is FALSE. `ISREF((A1,B2))` is TRUE (probes/union-accepted).
- Cross-sheet unions are `#VALUE!` as values (probes/union-accepted). Whether a function such as
  `SUM((Data!A1,Other!A1))` accepts one is unknown, see the probe list.
- Multi-area defined names: with `mu` = `Sheet1!$A$1,Sheet1!$B$2` (no parentheses in the stored
  body), `=SUM(mu)` is 6, `=COUNT(mu)` is 2, `=ISREF(mu)` is TRUE, `=mu A1` is 1 and `=mu` is
  `#VALUE!` (probes/union-defined-name H1:H5). So a name body is parsed with the comma as a union
  at its top level, and a name stands for its areas wherever a reference is accepted.

Unknown, to probe first (T-1061; each row lists the result the rollout will implement *if* Excel
agrees; a different oracle answer changes the implementation task, not this design):

| # | Formula (A1 = 1, B2 = 5, C3 = 7, D4 = 9, rest empty unless stated) | Planned result |
|---|---|---|
| 1 | `=SUM((A1,B2))`, `=SUM((A1,B2),C3)`, `=SUM((A1:B2,B2))` (overlap counted twice?) | 6, 13, 11 |
| 2 | `=COUNT((A1,B2))`, `COUNTA`, `AVERAGE`, `MIN`, `MAX`, `PRODUCT` of `(A1,B2)` | 2, 2, 3, 1, 5, 5 |
| 3 | `=LARGE((A1,B2,C3),2)`, `SMALL`, `RANK(5,(A1,B2,C3))`, `MEDIAN`, `STDEV.S` | 5, 5, 2, 5, 3.055 (all accepted; T-1061) |
| 4 | `=FREQUENCY((A1,B2),3)`, `=SUMPRODUCT((A1,B2))`, `=SUMIF((A1,B2),">0")`, `=COUNTIF((A1,B2),">0")` | FREQUENCY sum 2; the other three `#VALUE!` (T-1061) |
| 5 | `=COUNTBLANK((A1,B2))`, `=ROWS((A1,B2))`, `=COLUMNS((A1,B2))` | `#VALUE!`, `#REF!`, `#REF!` (T-1061) |
| 6 | `=AREAS((A1,B2))`, `=AREAS(((A1,B2),C3))`, `=AREAS((A1:B2,A1:B2))` | 2, 3, 2 |
| 7 | `=INDEX((A1:B2,C3:D4),1,1,2)`, `INDEX(…,1,1,3)`, `INDEX(…,1,1,0)`, `INDEX(…,1,1)` (area_num omitted) | C3's 7, `#REF!`, `#VALUE!`, A1's 1 |
| 8 | `=SUM(INDEX((A1:B2,C3:D4),0,0,2))` | 16 |
| 9 | `=LET(x,(A1,B2),SUM(x))`, `=LET(x,(A1,B2),x A1)`, `=LET(x,(A1,B2),x)` | 6, 1, `#VALUE!` |
| 10 | `=IF(TRUE,(A1,B2)) A1`, `=SUM(IF(TRUE,(A1,B2)))`, `=SUM(CHOOSE(2,A1,(A1,B2)))` | 1, 6, 6 |
| 11 | `=SUM((A1:A3,B1:B3) A1:B2)` (an intersection leaving two areas, as an argument) | sum of A1:B2 |
| 12 | `=SUM((Data!A1,Other!A1))`, `=AREAS((Data!A1,Other!A1))` | `#VALUE!` (cross-sheet) |
| 13 | `=SUM(Sheet1:Sheet2!A1,(A1,B2))` and `=SUM((Sheet1:Sheet2!A1,B2))` (3-D operand in a union) | 107; `#NAME?` (not `#VALUE!`; T-1061) |
| 14 | `_xlnm.Print_Area` = `Sheet1!$A$1,Sheet1!$B$2` (sheet-local): `=SUM(Print_Area)`, `=AREAS(Print_Area)` | 6, 2 |
| 14b | a name `xs` = `Data!$A$1,Other!$A$1` (cross-sheet body): `=SUM(xs)`, `=xs`, `=ISREF(xs)` | the error row 12 records |
| 15 | `=ISERROR((A1,B2))`, `=IFERROR((A1,B2),"x")`, `=ISNUMBER((A1,B2))` | TRUE, "x", FALSE |
| 16 | `=@(A1,B2)` and the FORMULATEXT of a union in a pre-dynamic-array workbook | `#VALUE!`; no `@` in a dynamic-array formula; the legacy half is VERIFIED the same: no `@` added (probes/multiarea-at-legacy, T-1077) |
| 17 | `=SUBTOTAL(9,(A1,B2))`, `=SUBTOTAL(109,(A1,B2))` | 6, 6 |
| 18 | `=CELL("address",(A1,B2))`, `=OFFSET((A1,B2),1,1)`, `=ROW((A1,B2))` | "$A$1, $B$2", `#VALUE!`, `#REF!` (T-1061) |
| 19 | `=A1,B2` as a cell formula | the file loads and gives `#VALUE!`; typed entry unprobed (T-1061) |
| 20 | `=AREAS(1)`, `=AREAS("A1")`, `=AREAS(Data1:Data3!A1)` | `#VALUE!` for the 3-D span; `AREAS(1)`, `AREAS("A1")` make Excel for the web refuse the workbook (T-1061) |

## Design

### Lexing and printing (unchanged)
- The union is built only in `Parser::parse_parenthesized` (expressions/parser/reference_operators.rs):
  after the first expression inside a parenthesis it loops on the argument separator, checks each
  operand with `is_union_operand`, and builds `OpUnionKind`. A comma anywhere else, such as in a
  function's argument list, is an argument separator. So `SUM(A1,B2)` is two arguments and
  `SUM((A1,B2))` one. `=A1,B2` is a syntax error in a cell. That last part is an engine choice,
  not probed (references.md "Union: parsing and evaluation"); probe row 19 checks it.
- New: a **defined-name body** is read with the comma as a union at its top level, so
  `Sheet1!$A$1,Sheet1!$B$2` is one union (VERIFIED, probes/union-defined-name). This is a parser
  entry point (`parse_name_body`, next to `parse_internal_formula`). It extracts the separator loop
  of `parse_parenthesized` into one function that both callers use, with no second copy and no flag.
  It keeps the parser's depth cap. A name body `SUM(A1,B2)` is still a call with two arguments.
- Comma-decimal locales: the union operator is the locale's argument separator (`;` in de-DE),
  as the engine prints it today (`argument_separator` in stringify.rs). xlsx always stores `,`, so
  this cannot be probed through a file. It stays the engine choice recorded in references.md.
- Printing keeps every written parenthesis (`written`/`grouped` on `OpUnionKind`; VERIFIED for
  FORMULATEXT by probes/union-accepted and probes/union-operand-nested-parens, T-0193, T-1024). A
  name body round-trips without added parentheses: `Sheet1!$A$1,Sheet1!$B$2` is exported as stored.

### Representation: `Areas`, outside `CalcResult`
```rust
// reference_operators.rs
/// The areas of a reference that has more than one, in formula order, duplicates kept.
/// 2..=MAX_AREAS entries, each a range on ONE sheet with its corners in order.
/// Shared, so a LET binding or a defined name is never copied on read.
#[derive(Clone)]
pub(crate) struct Areas(Arc<[Range]>);

impl Areas {
    /// The only constructor: refuses fewer than 2 or more than MAX_AREAS areas, and any area
    /// whose corners sit on two sheets (a 3-D reference), with the `#VALUE!` to give instead.
    pub(crate) fn new(areas: Vec<Range>, cell: CellReferenceIndex) -> Result<Areas, CalcResult>;
}

/// The ranges of a reference argument: one range (a 3-D one included) or several areas.
/// Derefs to `&[Range]`; a single range is borrowed, never copied.
pub(crate) enum Ranges { One(Range), Areas(Areas) }

/// What a reference-accepting argument evaluates to.
pub(crate) enum RefArg {
    /// A reference: a plain range, or a union (or a name, LET variable, IF/CHOOSE branch or
    /// intersection that stands for one).
    Ranges(Ranges),
    /// Anything else (a value, an array or an error), never a `CalcResult::Range`.
    Value(CalcResult),
}
```
`Areas` is **not** a `CalcResult` variant. That is the main decision of this spec:
- `CalcResult` is matched in hundreds of places in the evaluator and functions, about 40 of them
  naming `CalcResult::Lambda` explicitly (a proxy for the exhaustive ones). A new variant would
  reach every one, and every `_ =>` arm would treat a union as some non-error value. The planned
  `ISERROR((A1,B2))` TRUE and `IFERROR((A1,B2),"x")` "x" (probe row 15, following the VERIFIED
  value-context rule) would silently break. Keeping `Areas` in its own type means a union reaches
  only code that asks for one, and the compiler enforces that. If row 15 comes back otherwise,
  this argument is revisited before T-1062.
- `evaluate_node_in_context` (value context) keeps returning `#VALUE!` for a union, through
  `get_union`, unchanged. It never returns `Areas`, so no existing caller changes.
- A single area is `RefArg::Ranges(Ranges::One(..))`, built from the `CalcResult::Range` with no
  allocation, so every caller walks one range and several areas through the same arm
  (`for_each_cell_in_areas` takes `&[Range]`). As implemented by T-1062; the first draft of this
  spec kept a single range in `RefArg::Value`, which would have given every caller two walk arms.
- **3-D references are not areas.** A `Range` whose corners sit on two sheets stays
  `Ranges::One` and is never put into `Areas`. The area-list conversion (`Ranges::from_list`)
  refuses it and any list on two sheets, and so does `union_areas` today, through
  `evaluate_intersection_operand`. For one, the aggregates keep their
  3-D walk, while AREAS, INDEX and ISREF keep their VERIFIED `#VALUE!`/FALSE through
  `evaluate_single_sheet_arg` (references.md "3-D references outside the aggregates",
  probes/3d-ref-functions). `RefArg::single_sheet_areas()` gives `&[Range]`, a borrowed
  one-element view for a single-sheet `Range` with no allocation, and `None` for a 3-D one.
- Overlapping areas are kept twice, as written. Whether `SUM((A1:B2,B2))` counts B2 twice is
  probe row 1.

### The reference-context evaluator
`Model::evaluate_reference_arg(node, cell) -> RefArg` lives in util.rs. In the end state,
`evaluate_aggregate_arg` is deleted: T-1062 migrates every caller, and a temporary wrapper
(`Areas` collapsed to `#VALUE!`) may exist only within that PR. `union_is_reference` is likewise
replaced by a call to the evaluator (T-1062). The cases:
- `OpUnionKind`: `reference_areas` (exists), giving `Areas`. A union has at least two operands, so
  at least two areas. An operand error is returned as `Value(error)`; the first error wins, as today.
- `OpIntersectKind`: see "Intersection operands" below. More than one area left gives `Areas`, one
  gives `Value(Range)` (probe row 11; today `#VALUE!`).
- `DefinedNameKind` whose body is multi-area: the name's `Areas`, a cheap `Rc` clone (step 5).
- `NamedVariableKind` bound to `Areas`: those areas, an `Rc` clone (step 4).
- `IF`, `CHOOSE` and `INDEX` in reference form: the chosen branch is evaluated with
  `evaluate_reference_arg` again. The existing `as_reference` flag on `if_impl`/`index_impl`
  is the hook, and CHOOSE gets the same flag. Recursion follows the formula tree, so the parser's
  depth cap (`MAX_FORMULA_DEPTH`) bounds it (`IF(c,IF(c,…))`).
- Everything else: `Value` of what `evaluate_aggregate_arg` returns today.

`reference_areas`/`union_areas`/`intersect_areas` and the `MAX_AREA_PAIRS` budget stay the only
code that builds area lists. The new evaluator calls them and does not walk unions itself
(directive 3).

### Intersection operands
`mu A1` (VERIFIED 1), `LET(x,(A1,B2),x A1)` (row 9) and `IF(TRUE,(A1,B2)) A1` (row 10) need an
intersection whose operand is a name, a variable or a function result standing for several areas.
Today `is_multi_area` decides by syntax alone, and `evaluate_intersection_operand` yields one
range. The change is in T-1064, with LET and IF/CHOOSE; T-1065 adds names as operands:
- As built (T-1064, amending the first plan): `is_multi_area` stays as a syntactic gate and is
  widened to every node that *may* stand for several areas: a union, a LET variable, `IF`,
  `CHOOSE` and `LET` (T-1065 adds defined names whose body is `Areas`). An intersection with no such
  operand keeps the allocation-free pair path, so `A1:B2 B1:B3` and `name A1` allocate nothing;
  `evaluate_areas` evaluates a widened operand through `evaluate_reference_arg` and builds a list
  from the result (an `Areas`, or one single-sheet range). The first plan, branching on evaluated
  results and deleting `is_multi_area`, would evaluate every operand into a `RefArg` before
  knowing whether a list is needed; the gate keeps that cost off the common plain-reference chain
  for the same results.
- `get_intersection`/`intersect_areas` keep taking `&Node` and evaluate their operands
  themselves. Re-intersecting a LET variable (`x x x …`) spends the caller's `MAX_AREA_PAIRS`
  budget and does not restart it; an IF, CHOOSE or LET operand starts its own, as INDEX and
  OFFSET already do, so total work stays linear in formula length.

### Work bound for walking areas (DEVIATION)
The area cap bounds list length, not work. Nested cuts of `U = (A:A,A:A,A:A,A:A)` reach
`MAX_AREAS` copies of `A:A` in about 180 characters, and an aggregate over them would walk the
column 65,536 times (directive 5). So:
- One walker, `Model::for_each_cell_in_areas` (util.rs, with its position, number and error
  variants), walks one range or an `Areas`. It is the only way an `Areas` is walked. The used
  area is looked up per area, which is O(1) since `Worksheet::used_area_end` (T-0049).
- Before walking, it sums the clipped sizes of all areas, which is arithmetic only. The sum is
  computed in `u64` with saturating arithmetic and stops as soon as it passes `MAX_AREA_CELLS`,
  because one whole-sheet area alone is about 1.7e10 cells. The bound is per argument on purpose:
  `SUM(u1,u2,…)` scales with its argument count exactly as plain range arguments already do. More than
  `MAX_AREA_CELLS` = 2^24 (16,777,216) cells in one `Areas` argument gives `#VALUE!`. That is 16
  full used columns of 1,048,576 rows, far above any real Print_Area. Excel's own limit, if any,
  is not known. This is recorded as a DEVIATION under references.md "Union area cap"
  (decision 0014) by T-1062. Superseded by T-1162 (one budget per formula evaluation) and T-1497:
  union areas are now walked over their existing cells only, paying `MAX_AREA_WALK` steps as they
  go (references.md "Union area cap" has the current rule).
- `seen_ranges` (evaluation.rs) gets at most two entries per walked area (`clip_to_used_area`),
  so it is bounded by 2 × `MAX_AREAS` per argument.

### Function contract: opt-in
A function accepts multi-area references in an argument only by calling `evaluate_reference_arg`
for it and handling `RefArg::Areas`. Every other argument position evaluates through the existing
value or single-sheet paths (`evaluate_node_in_context`, `evaluate_single_sheet_arg`) and keeps
getting `#VALUE!` for a union, with no code change. Concretely:
- The aggregates on `evaluate_aggregate_arg` (SUM, COUNT, COUNTA, AVERAGE, MIN, MAX, PRODUCT, the
  STDEV/VAR families, …) switch to `evaluate_reference_arg` and walk `Areas` with
  `for_each_cell_in_areas`.
- AGGREGATE's syntax-tree special case (`Node::OpUnionKind => self.reference_areas(..)`) is
  replaced by the evaluator, so it also accepts names and `LET` variables. Its array form now
  evaluates through `evaluate_node_with_reference`, and its `Array`, `Range`, number and error
  arms keep their current behaviour.
- AREAS: the number of areas of `single_sheet_areas()`. A 3-D reference stays `#VALUE!`
  (VERIFIED). `AREAS` of a non-reference (`AREAS(1)`) is 1 in the engine today, with no evidence:
  probe row 20.
- INDEX: `area_num` is read as a number, then truncated. It is checked before any integer
  conversion: below 1 is `#VALUE!` (as today), NaN or past the number of areas is `#REF!`, and only
  then is it converted to an index (probe row 7).
- ISREF: `Areas` is TRUE, and a 3-D reference keeps FALSE (VERIFIED).
- Functions whose Excel behaviour is unknown (probe rows 3-5, 12, 13, 16-18, 20) are migrated only
  after T-1061 records their evidence. T-1061 files one task per group that needs an engine change.

### LET, IF, CHOOSE
`variable_stack` (shared by LET and LAMBDA) changes from `CalcResult` to a `Binding` enum:
`Value(CalcResult)` or `Areas(Areas)`. LET values and LAMBDA arguments are bound by one helper,
`Model::bind_argument`: a node `is_multi_area` admits is evaluated with `evaluate_reference_arg`,
anything else as before. Every reader of `variable_stack` except `evaluate_reference_arg` sees
`#VALUE!` for an `Areas` binding. So `LET(x,(A1,B2),SUM(x))` is 6, `LET(x,(A1,B2),x)` is
`#VALUE!` (probe row 9), and `LET(x,(A1,B2),y,x,SUM(y))`, `LET(x,IF(TRUE,(A1,B2)),SUM(x))` and
`LAMBDA(x,SUM(x))((A1,B2))` are 6 (probes/multiarea-letbind, which overturned the first plan's
value-evaluated LAMBDA arguments). LET in reference context evaluates its body there too
(`SUM(LET(x,(A1,B2),x))` = 6). `variable_stack` is a `VariableStack` (next to `Binding`) that caps the areas all
bindings in scope hold at `4 * MAX_AREAS` (DEVIATION, references.md "Union area cap"). The ids are
unchanged.

### Defined names
- As built (T-1065, amending the plan below): no `ParsedDefinedName::Areas` and no per-workbook cap.
  A union body stays T-0158's `Formula` body and is evaluated at each use in reference context
  (`Model::defined_name_reference`) under the name depth and budget limits, so the one builder
  (`union_areas`) and its caps apply unchanged and names store no area list. `is_multi_area` became
  a `Model` method that is true for a name with a `Formula` body (a cell or range name keeps the
  pair path). A sheet-local Print_Area needs sheet-scoped names in the case loader and `_xlnm.`
  name mapping on import, which are not multi-area work: T-1193.
- `parse_name_body` returns a `Node`. A body that is a union of absolute single-sheet references
  only (the Print_Area shape) becomes `ParsedDefinedName::Areas(Areas)`, resolved once at load by
  evaluating the node through `reference_areas`, the same builder, not a second walker. Reads are
  `Rc` clones.
- Any other multi-area body (relative references, intersections, functions) goes through T-0158's
  general-formula body, evaluated per use, and reaches `Areas` through `evaluate_reference_arg`.
  Names are not resolved recursively here: a body naming another name is T-0158's bounded path.
- A body that fails to parse, holds a cross-sheet or 3-D operand (`Ranges::from_list` refuses it; probe
  rows 12, 13, 14b; T-1065 matches the error Excel gives there), or has more than `MAX_AREAS` areas becomes `InvalidDefinedNameFormula`. That is the
  name's error value, never a load failure, as for any bad body today. The total areas over all
  names of a workbook are capped at `MAX_AREAS`. Past it, further multi-area names are refused the
  same way, so memory stays bounded at 65,536 × 24 bytes per workbook. Both limits are added to the
  "Union area cap" DEVIATION by T-1065.
- Scope: Print_Area is sheet-local. Lookup follows the existing local-then-global name scoping,
  unchanged (probe row 14).
- A value read of a multi-area name is `#VALUE!` (VERIFIED `=mu`), and `ISREF(name)` is TRUE.

### `add_implicit_intersection` and `OpUnionKind`
It stays unwrapped. The pass inserts `@` only where a scalar is expected, and there a union is
already `#VALUE!` through the value path (for arithmetic VERIFIED; for scalar function arguments
planned, probe rows 15, 18). Wrapping would add an `@` to the printed formula of legacy workbooks
that Excel does not show (probe row 16, VERIFIED for legacy formulas by T-1077), for no change in
value. So the union stays unwrapped and no follow-up is needed.

### What changes for existing formulas
Formulas that pass a union to an opted-in function stop being `#VALUE!`: `SUM((A1,B2))` becomes 6,
and `IFERROR(SUM((A1,B2)),0)` stops taking its fallback. A union in a value context, and in any
function that has not opted in, is still `#VALUE!`, so `IFERROR((A1,B2),"x")` is still "x" (planned,
probe row 15).

### Errors and limits
- An operand error inside the union wins, in operand order, as today (`union_areas`).
- Every `Areas` is built by `union_areas`/`intersect_areas`, so the `MAX_AREAS` and
  `MAX_AREA_PAIRS` caps apply unchanged. Walking is bounded by `MAX_AREA_CELLS`, and name bodies by
  the per-workbook cap. All three are DEVIATIONs (decision 0014). Building `Areas` from the `Vec`
  that `union_areas` returns copies it once into the `Rc`. After that, reads never copy.
- No panics: `area_num` is range-checked as a float before conversion, and `Areas` is never empty.

### Dependency graph (SPEC-0002 requirement)
When the graph lands, a union (written, named or LET-bound) registers its areas deduplicated,
bounded by `MAX_AREAS`. It may fall back to one bounding range per sheet, since coarser
dependencies only cause extra recalculations, never missed ones. A union reached through IF/CHOOSE
is a dynamic dependency like any other reference-returning function. T-0401 carries a pointer to
this paragraph.

## Alternatives considered
1. **A `CalcResult::Areas` variant.** It is the most direct, and LET and names would need no
   special storage. Rejected: every exhaustive match changes, and every `_ =>` arm silently
   misreads a union as a non-error value (`ISERROR`, `IFERROR`, `TYPE`, `N`, …; planned values,
   probe row 15). Keeping the value rule would then depend on auditing every function, not on the
   type.
2. **Syntax-tree only (AGGREGATE's current approach).** Each function matches
   `Node::OpUnionKind` and calls `reference_areas`. Rejected: it cannot see a union behind a name,
   a LET variable or an IF/CHOOSE branch, and it duplicates the match in every function.
3. **3-D-style encoding in `Range`** (the way a 3-D reference is a `Range` with corners on two
   sheets). Rejected: a list of areas cannot be packed into two corners, and the 3-D trick is
   already a FIXME.
4. **Expanding a union to an array of values before the function sees it.** Rejected: it loses
   the reference (AREAS, INDEX, ISREF, intersection need areas), changes blank-cell and text
   semantics (ranges skip text, arrays do not always), and allocates for whole columns.

## Test plan
- T-1061 adds the probe cases above under `eval/suites/probes/` (one workbook per group, since
  Excel for the web refuses a whole workbook it cannot parse), recorded in references.md.
- Each implementation task turns its probe rows green and adds unit tests in
  `test/` for: errors inside unions, `MAX_AREAS` at the limit and one past it through each new
  entry point (name body, LET, INDEX area_num), whole-column unions (`SUM((A:A,C:C))`), and the
  worst case: `MAX_AREAS` whole-column areas over a populated sheet gives `#VALUE!` from the
  `MAX_AREA_CELLS` check without walking (T-1062).
- 3-D cells of probes/3d-ref-functions stay green at every step (T-1062, T-1063).
- Metamorphic, only if probe row 1 confirms it: `SUM((r1,r2,…))` = `SUM(r1,r2,…)` for same-sheet,
  non-3-D references (gen-m1 generator family, T-1062), and `AREAS(u)` equals the number of
  top-level operands of a union of plain same-sheet references (T-1063).
- Existing VERIFIED cells (gen-m1/c940d3cfb0d3b083, probes/union-*) must stay green at every step.

## Rollout
1. **T-1061** (research, S, sonnet): probe the table above, record findings in references.md,
   file a task for each further function group Excel accepts.
2. **T-1062** (engine, M, opus; after T-1061): `Areas`, `RefArg`, `evaluate_reference_arg`,
   `for_each_cell_in_areas` with `MAX_AREA_CELLS`; migrate the aggregates on
   `evaluate_aggregate_arg`, AGGREGATE and ISREF. If the aggregate migration grows past M, split
   it into a second task.
3. **T-1063** (engine, S, sonnet; after T-1062): AREAS and INDEX `area_num`.
4. **T-1064** (engine, M, opus; after T-1062): intersection operands through the evaluator, the
   `Binding` for `variable_stack`, LET, IF and CHOOSE in reference context.
5. **T-1065** (engine, S, opus; after T-1064 and T-0158): multi-area defined-name bodies,
   `_xlnm.Print_Area`, the name-body parser entry point, the per-workbook name-area cap.
Until each step lands, the old path stays: a union reaching an unmigrated function is `#VALUE!`.
