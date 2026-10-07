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
- Changing how a union parses or prints. The lexing rule (below) and FORMULATEXT behaviour are
  settled and VERIFIED by T-0134, T-0187, T-0193 and T-1024; this spec only restates them.
- Spilling or displaying a multi-area value. A union as a cell result stays `#VALUE!` (VERIFIED).
- New area-count limits. `MAX_AREAS` and `MAX_AREA_PAIRS` (references.md "Union area cap",
  DEVIATION) bound every list this spec creates too.
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
| 3 | `=LARGE((A1,B2,C3),2)`, `SMALL`, `RANK(5,(A1,B2,C3))`, `MEDIAN`, `STDEV.S` | per function |
| 4 | `=FREQUENCY((A1,B2),3)`, `=SUMPRODUCT((A1,B2))`, `=SUMIF((A1,B2),">0")`, `=COUNTIF((A1,B2),">0")` | `#VALUE!` where Excel refuses |
| 5 | `=COUNTBLANK((A1,B2))`, `=ROWS((A1,B2))`, `=COLUMNS((A1,B2))` | per function |
| 6 | `=AREAS((A1,B2))`, `=AREAS(((A1,B2),C3))`, `=AREAS((A1:B2,A1:B2))` | 2, 3, 2 |
| 7 | `=INDEX((A1:B2,C3:D4),1,1,2)`, `INDEX(…,1,1,3)`, `INDEX(…,1,1,0)`, `INDEX(…,1,1)` (area_num omitted) | C3's 7, `#REF!`, `#VALUE!`, A1's 1 |
| 8 | `=SUM(INDEX((A1:B2,C3:D4),0,0,2))` | 16 |
| 9 | `=LET(x,(A1,B2),SUM(x))`, `=LET(x,(A1,B2),x A1)`, `=LET(x,(A1,B2),x)` | 6, 1, `#VALUE!` |
| 10 | `=IF(TRUE,(A1,B2)) A1`, `=SUM(IF(TRUE,(A1,B2)))`, `=SUM(CHOOSE(2,A1,(A1,B2)))` | 1, 6, 6 |
| 11 | `=SUM((A1:A3,B1:B3) A1:B2)` (an intersection leaving two areas, as an argument) | sum of A1:B2 |
| 12 | `=SUM((Data!A1,Other!A1))`, `=AREAS((Data!A1,Other!A1))` | `#VALUE!` (cross-sheet) |
| 13 | `=SUM(Sheet1:Sheet2!A1,(A1,B2))` and `=SUM((Sheet1:Sheet2!A1,B2))` (3-D operand in a union) | 3-D arg works, union with 3-D `#VALUE!` |
| 14 | `_xlnm.Print_Area` = `Sheet1!$A$1,Sheet1!$B$2` (sheet-local): `=SUM(Print_Area)`, `=AREAS(Print_Area)` | 6, 2 |
| 15 | `=ISERROR((A1,B2))`, `=IFERROR((A1,B2),"x")`, `=ISNUMBER((A1,B2))` | TRUE, "x", FALSE |
| 16 | `=@(A1,B2)` and the FORMULATEXT of a union in a pre-dynamic-array workbook | `#VALUE!`, no `@` |
| 17 | `=SUBTOTAL(9,(A1,B2))`, `=SUBTOTAL(109,(A1,B2))` | 6, 6 |
| 18 | `=CELL("address",(A1,B2))`, `=OFFSET((A1,B2),1,1)`, `=ROW((A1,B2))` | `#VALUE!` where Excel refuses |

## Design

### Lexing and printing (unchanged)
- The comma is the union operator only *directly* inside parentheses: `Parser::parse_parenthesized`
  sets `union_allowed`, and `parse_primary` clears it for every other operand, so `SUM(A1,B2)` is two
  arguments and `SUM((A1,B2))` one. `=A1,B2` stays a syntax error in a cell (T-0134).
- New: a **defined-name body** is parsed as if it sat directly inside parentheses, so
  `Sheet1!$A$1,Sheet1!$B$2` is one union (probes/union-defined-name). This is a parser entry
  point (`parse_name_body`, next to `parse_internal_formula`), not a lexer change. A name body
  `SUM(A1,B2)` is still a call with two arguments.
- Comma-decimal locales: the union operator is the locale's argument separator (`;` in de-DE),
  as the engine prints it today (`argument_separator` in stringify.rs). xlsx always stores `,`, so
  this cannot be probed through a file. It stays the engine choice recorded in references.md.
- Printing keeps every written parenthesis (`written`/`grouped` on `OpUnionKind`, T-0193,
  T-1024). A name body round-trips without added parentheses: `Sheet1!$A$1,Sheet1!$B$2` is
  exported as stored.

### Representation: `Areas`, outside `CalcResult`
```rust
// reference_operators.rs
/// The areas of a reference that has more than one, in formula order, duplicates kept.
/// 2..=MAX_AREAS entries, each a single-sheet range with its corners in order.
pub(crate) struct Areas(Box<[Range]>);

/// What a reference-accepting argument evaluates to.
pub(crate) enum RefArg {
    /// A union (or a name, LET variable, IF/CHOOSE branch or intersection that stands for one).
    Areas(Areas),
    /// Anything else, exactly what `evaluate_aggregate_arg` returns today (a `Range` for a
    /// single area, a value, an array or an error).
    Value(CalcResult),
}
```
`Areas` is **not** a `CalcResult` variant. That is the main decision of this spec:
- `CalcResult` is matched in hundreds of places in the evaluator and functions (39 of them name
  `CalcResult::Lambda` explicitly, a proxy for the exhaustive ones). A new variant would reach every
  one, and every `_ =>` arm would treat a union as some non-error value. `ISERROR((A1,B2))` would turn FALSE and `IFERROR` would stop catching it,
  the opposite of the VERIFIED value-context rule. Keeping `Areas` in its own type means a union
  can reach only the code that asks for one, and the compiler enforces that.
- `evaluate_node_in_context` (value context) keeps returning `#VALUE!` for a union, through
  `get_union`, unchanged. It never returns `Areas`, so no existing caller changes.
- A single area stays `RefArg::Value(CalcResult::Range { .. })`, so existing single-area code
  (and 3-D references, which are still a `Range` whose corners sit on two sheets) is untouched.
  `RefArg::areas()` gives every caller a uniform `&[Range]` view (one area for a `Range`).
- Overlapping areas are kept twice, as written. Whether `SUM((A1:B2,B2))` counts B2 twice is
  probe row 1.

### The reference-context evaluator
`Model::evaluate_reference_arg(node, cell) -> RefArg` (util.rs, next to and replacing the body of
`evaluate_aggregate_arg`, which becomes `self.evaluate_reference_arg(node, cell)` collapsed to a
`CalcResult`, with `Areas` giving `#VALUE!`, for callers not yet migrated):
- `OpUnionKind`: `reference_areas` (exists), giving `Areas` (a union has at least two operands, so
  at least two areas; an operand error is returned as `Value(error)`, first error wins as today).
- `OpIntersectKind` with `is_multi_area`: `intersect_areas`, giving `Areas` when more than one area
  is left and `Value(Range)` for one (probe row 11; today `#VALUE!`).
- `DefinedNameKind` whose body is multi-area: the name's areas (step 5).
- `NamedVariableKind` bound to `Areas`: those areas (step 4).
- `IF`, `CHOOSE` and `INDEX` in reference form: the chosen branch is evaluated with
  `evaluate_reference_arg` again (the existing `as_reference` flag on `if_impl`/`index_impl`
  is the hook; CHOOSE gets the same flag). Recursion depth is bounded by the parser's depth cap,
  as for `is_multi_area`.
- Everything else: `Value(evaluate_aggregate_arg's current result)`.

`reference_areas`, `count_union_areas` and the `MAX_AREA_PAIRS` budget stay the only code that
builds area lists. The new evaluator calls them; it does not walk unions itself (directive 3).

### Function contract: opt-in
A function accepts multi-area references in an argument only by calling `evaluate_reference_arg`
for it and handling `RefArg::Areas`. Every other argument position evaluates through the existing
value or single-sheet paths (`evaluate_node_in_context`, `evaluate_single_sheet_arg`) and keeps
getting `#VALUE!` for a union, with no code change. Concretely:
- Aggregates that already go through `evaluate_aggregate_arg` (SUM, COUNT, COUNTA, AVERAGE, MIN,
  MAX, PRODUCT, the STDEV/VAR families, …) switch to `evaluate_reference_arg` and
  walk `Areas` area by area with the existing `for_each_*_in_range` walkers. The walkers already
  clip whole rows and columns to the used area, so a union of `A:A,B:B` costs what the two
  columns cost.
- AGGREGATE's syntax-tree special case (`Node::OpUnionKind => self.reference_areas(..)`) is
  replaced by the evaluator, so it also accepts names and `LET` variables.
- AREAS: `RefArg::areas().len()`; a non-reference is `#VALUE!` as today.
- INDEX: `area_num` selects `areas()[area_num - 1]`; out of range `#REF!`, below 1 `#VALUE!`
  (probe row 7).
- ISREF: `Areas` is TRUE; the existing `union_is_reference` becomes a call to the evaluator.
- Functions whose Excel behaviour is unknown (probe rows 3-5, 12, 13, 17, 18) are migrated only
  after T-1061 records their evidence. T-1061 files one task per group that Excel accepts.

### LET, IF, CHOOSE
`LET` evaluates each binding with `evaluate_reference_arg`, and the variable stack stores a
`RefArg`-like binding (`Value(CalcResult)` or `Areas`). A value read of an `Areas` binding
(`NamedVariableKind` in `evaluate_node_in_context`) is `#VALUE!`, and a reference read gives the
areas. So `LET(x,(A1,B2),SUM(x))` is 6 and `LET(x,(A1,B2),x)` is `#VALUE!` (probe row 9). This is
the only place outside the evaluator that sees `Areas`. The `HashMap` stays keyed by the same ids.

### Defined names
`ParsedDefinedName` gets `Areas(Box<[Range]>)` for a body that is a union of absolute references
only (the Print_Area shape), resolved once at load time like `RangeReference`. A body with
relative references or other operators inside the union goes through T-0158's general-formula
body (it adds one, evaluated per use) and reaches `Areas` through `evaluate_reference_arg`. Names
are not evaluated recursively by this design: a body naming another name is T-0158's (bounded)
path. Value read of a multi-area name: `#VALUE!` (VERIFIED `=mu`). `ISREF(name)` is TRUE.
Name bodies over `MAX_AREAS` areas are refused at load ("invalid defined name") rather than
truncated (DEVIATION, the same "Union area cap" entry).

### `add_implicit_intersection` and `OpUnionKind`
It stays unwrapped. The pass inserts `@` only where a scalar is expected, and there a union is
already `#VALUE!` through the value path. Wrapping would add an `@` to the printed formula of
legacy workbooks that Excel may not show (probe row 16), for no change in value. If T-1061 shows
Excel prints `@(A1,B2)` for a legacy union, a follow-up adds the wrap.

### What changes for existing formulas
Formulas that pass a union to an opted-in function stop being `#VALUE!`: `SUM((A1,B2))` becomes 6,
and `IFERROR(SUM((A1,B2)),0)` stops taking its fallback. A union in a value context, and in any
function that has not opted in, is still `#VALUE!`, so `IFERROR((A1,B2),"x")` is still "x".

### Errors and limits
- An operand error inside the union wins, in operand order, as today (`union_areas`).
- Every `Areas` is built by `union_areas`/`intersect_areas`, so the `MAX_AREAS` and
  `MAX_AREA_PAIRS` caps (DEVIATION, decision 0014) apply unchanged. `Areas` is a boxed slice
  (16 bytes on the stack), so `RefArg` adds no size to `CalcResult`.
- No panics: `area_num` is range-checked before indexing, `Areas` is never empty.

### Dependency graph (SPEC-0002 requirement)
When the graph lands, a union (written, named or LET-bound) registers one range vertex per area,
deduplicated. A union reached through IF/CHOOSE is a dynamic dependency like any other
reference-returning function.

## Alternatives considered
1. **A `CalcResult::Areas` variant.** It is the most direct, and LET and names would need no
   special storage. Rejected: about 40 exhaustive matches change, and every `_ =>` arm silently
   misreads a union as a non-error value (`ISERROR`, `IFERROR`, `TYPE`, `N`, …). Keeping the value
   rule would then depend on auditing every function, not on the type.
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
  entry point (name body, LET, INDEX area_num), and whole-column unions (`SUM((A:A,C:C))` costs
  what two column scans cost, not more).
- Metamorphic: `SUM((r1,r2,…))` = `SUM(r1,r2,…)` for any references (gen-m1 generator family,
  added with T-1062), and `AREAS(u)` equals the number of top-level operands of a union of plain
  references.
- Existing VERIFIED cells (gen-m1/c940d3cfb0d3b083, probes/union-*) must stay green at every step.

## Rollout
1. **T-1061** (research, S, sonnet): probe the table above, record findings in references.md,
   file a task for each further function group Excel accepts.
2. **T-1062** (engine, M, opus; after T-1061): `Areas`, `RefArg`, `evaluate_reference_arg`;
   migrate the aggregates on `evaluate_aggregate_arg`, AGGREGATE and ISREF.
3. **T-1063** (engine, S, sonnet; after T-1062): AREAS and INDEX `area_num`.
4. **T-1064** (engine, M, opus; after T-1062): LET bindings, IF and CHOOSE in reference context.
5. **T-1065** (engine, S, opus; after T-1062 and T-0158): multi-area defined-name bodies,
   `_xlnm.Print_Area`, the name-body parser entry point.
Until each step lands, the old path stays: a union reaching an unmigrated function is `#VALUE!`.
