# SPEC-0006 — GROUPBY and PIVOTBY

- **Status:** accepted (on merge of T-1276)
- **Milestone / lane:** M7 / functions-lookup (grouping core, GROUPBY, PIVOTBY)
- **Tasks:** T-1276 (this spec); implementation tasks are listed under Rollout

## Problem
Neither function exists: `GROUPBY(...)` and `PIVOTBY(...)` evaluate to `#NAME?` (docs/excel-semantics/arrays.md
"Dynamic-array and LAMBDA function argument modes"). The synthetic generator calls both, and
probes/m7-argument-modes F33-H35 score them. Implemented in one go they would be size L. Both
also depend on behavior Excel does not document: header detection, how keys are ordered, the
filler values in total rows, and what `relative_to` means. This spec pins that behavior down
from oracle evidence and cuts the work into S/M tasks.

## Goals / non-goals
Goals:
1. One grouping core shared by both functions. It groups rows by a tuple of keys, aggregates
   one or more value columns through a function value, and lays out the totals.
2. GROUPBY first, then PIVOTBY (a GROUPBY whose groups are split again across column keys).
3. Every documented argument. Undocumented behavior follows the probe, never the documentation.

Non-goals:
- Eta-reduced names (`_xleta.SUM`) as values. T-1466 added them (it superseded T-1273) for BYROW and the other LAMBDA
  helpers. This spec only needs a function value it can call with one argument (two for
  PERCENTOF-style relative totals). GROUPBY and PIVOTBY get them through the same LAMBDA call path.
- Rendering pivot-like formatting. Excel's bold total rows are cell styles, which the engine
  does not produce for spills.

## Excel behavior to match
All evidence: probes/groupby-pivotby (excel-graph, 2026-10-11, T-1276), sheets G (GROUPBY) and
P (PIVOTBY), over Data!A1:I11. Row 1 holds headers (Region, Product, Year, Sales, Units, ...).
Rows 2-11 hold 10 records: the regions East, West, North, South, a lowercase `east`, and one
blank region. Cell addresses below are the anchors in that case. The rules are recorded in
docs/excel-semantics/arrays.md "GROUPBY and PIVOTBY".

### Grouping and ordering
- Keys compare like UNIQUE: text is case-insensitive, and the group shows the first spelling met
  (`East`, `east` → `East`, G!A1). Numbers compare exactly: 0.1+0.2 and 0.3 are two groups (0.3 first), 1E-17 is not 0, 1 and 1+1E-15 are two groups, but 0 and a computed 0 are one group (a cell `=-0` is stored as +0, so a true negative zero is unprobed). The blank key is a separate group from numeric 0 (both show 0, the blank one last, also when descending). Equal errors form one group, sorted after the numbers and before Total (their place against text, booleans and blank is unprobed) (`1/{1;0;2;0}` keys: 0.5, 1, #DIV/0!). Evidence: probes/groupby-pivotby-followup G!A761-A821, G!A741, G!A841 (T-1328).
- Groups are sorted ascending by key, with no option to keep source order. Mixed key types sort
  numbers < text < FALSE < TRUE < blank (G!A193). That is `sort_key_cmp` in
  `functions/spill_functions.rs`. Errors as keys: equal errors share one group, sorted after the numbers (see above).
- A blank key is its own group, shown as the number 0, and it sorts last even when sorting
  descending, for key sorts (G!A1, G!A145); sorting by a value column does not pin it (see sort_order).
- Within a group, function inputs keep source order: ARRAYTOTEXT gives "Apple, Pear, Apple"
  (G!AB97). The total row applies the function to all values in source order, not to the group
  results: AVERAGE's total is 13.05 = 130.5/10, MAX's is 40 (G!S73, G!J97).

### GROUPBY(row_fields, values, function, [field_headers], [total_depth], [sort_order], [filter_array], [field_relationship])
- **Shape**: one column per key field and one per value column (times the number of
  functions). There is one row per group, plus the total rows and an optional header row.
- **field_headers**: omitted means auto-detect. With A1:A11/D1:D11 it treats row 1 as headers but
  does not show them (G!J1 = G!A1). `0` means no headers: "Region" becomes a group, with
  SUM("Sales") = 0 (G!S1). `1` means headers, not shown (G!AB1). `2` means no headers, with
  generated ones shown: "Row Field 1", "Value 1" (G!A25). `3` means headers, shown (G!J25).
  `3` over A2:A11 consumes the first data row as the header row, so East|10 becomes the header
  and the total is 120.5 (G!S25). `4` is `#VALUE!` (G!A217).
- **total_depth**: omitted means 1, a grand total at the bottom labeled "Total". `0` means no
  totals (G!A49). `2` adds a subtotal after each first-level group, with the key and a blank
  second key, and the grand total is then labeled "Grand Total" (G!J49). Negative values put
  totals above their group (G!S49 for `-2`, G!AB49 for `-1`, the last labeled "Total").
  T-1324 (probes/groupby-subtotals): `n` subtotals the first n - 1 key fields and may not exceed
  the number of keys; with the depth omitted, a value sort over n keys shows every level's
  subtotals; a table relationship sorted by a value sorts every group by its own value. Details
  in docs/excel-semantics/arrays.md "GROUPBY and PIVOTBY".
- **Filler cells** (the blank key columns of total rows, the corner above the key columns) are
  empty text `""`, not empty cells (oracle `t:"s"`, G!AC33). Only the blank-key group shows 0.
- **sort_order**: an integer column index, 1-based over keys then values. Negative means
  descending. `-1` sorts keys descending, but blank stays last and the total stays at the bottom
  (G!A145). `2` and `-2` sort by the value column (G!J145, G!S145): rows are ordered by value, blank key included (blank 9 sits in the middle); with two keys and sort_order 3, parents are ordered by their aggregate (consistent with sum, max or mean; not separated by this probe) and children by value inside each parent, again without pinning blank (followup2 G!A261). An index past the last column
  is `#VALUE!` (G!S217). The array `{2,-1}` with two key fields gives region descending, then
  product ascending (G!AB145). Multi-key arrays name key columns with a sign: the priority is the column
  order, not the array order. `{2,-1}` equals `{-1,2}` (Region desc, Product asc), `{1,-2}` equals `{-2,1}` (Region asc,
  Product desc), `{2,1}` equals `{1,2}` (both asc); a one-element array is accepted (`{1}`, `{2}`: both asc);
  `{1,-1}` (a column twice), `{0,1}` are `#VALUE!` (PIVOTBY's row_sort_order 0 was `#VALUE!` too, but in a formula with other changed arguments: confounded); `{1,2,3}` over two keys is
  `#VALUE!` (followup G!A341-A541, followup2 G!A181-A241; T-1328; probed under the default hierarchy relationship only).
  T-1323 (probes/groupby-sort-filter-args B1-B40): fractions, 0, text, TRUE and a blank cell are `#VALUE!`; an array may
  be a column; it names a value column only as its one element (`{-3}`), mixing key and value columns is `#VALUE!`; equal
  values keep the ascending key order in both directions; values sort like keys (numbers < text, errors last ascending).
- **filter_array**: a Boolean column as tall as the data. FALSE rows are dropped before grouping,
  so groups with no rows vanish and totals cover only kept rows (G!A169). A height mismatch is
  `#VALUE!` (G!AB217). All FALSE is `#VALUE!`, not `#CALC!` (ERROR.TYPE 3, unlike FILTER's 14), also as a scalar `FALSE`, `0` or `{FALSE}`, with two key
  fields, and for PIVOTBY; a scalar `TRUE` filters nothing (followup3 G!A1-A133; G!A241).
  T-1323 (probes/groupby-sort-filter-args B42-B60): numbers count (0 drops), blanks drop, the text "TRUE"/"FALSE" counts;
  other text, an error element, a row or a 2-column filter are `#VALUE!`; with a header row the filter may be as tall as
  the fields or as the records.
- **Argument errors** (T-1323, B61-B83): an error value in any argument wins over every invalid argument, and the first
  in argument order wins among error values.
- **field_relationship**: `0` (hierarchy, the default) gives the same result as omitted
  (G!S169). `2` is `#VALUE!` (G!J241). `1` (table) with total_depth 2 or -2 is `#VALUE!` (ERROR.TYPE 3; followup G!A181, G!A261); with total_depth 1 or omitted it
  equals 0 (T-1328; probed with default sorting only). An array in argument 8 is lifted per element (a 10x1 column of group results, `{FALSE}` 1x1, a numeric column 10x1 `#VALUE!`, scalar FALSE/0 act as 0; followup G!A861-A981, followup2 G!A1-A161).
- **function** forms: an eta name (`SUM`, `AVERAGE`, `COUNT`, `COUNTA`, `MAX`, `PERCENTOF`,
  `ARRAYTOTEXT`) or a one-parameter LAMBDA (`LAMBDA(x,ROWS(x))`, text results too, G!A121,
  G!AB241). A LAMBDA that returns an array gives `#VALUE!` in each cell, totals included (G!J121).
  `HSTACK(SUM,AVERAGE)` adds a value column per function. With headers shown, it adds a header
  row of function names ("SUM", "AVERAGE") above the field header row (G!S121).
  `VSTACK(SUM,MAX)` repeats each group row once per function, with a label column of function
  names between keys and values (G!AB121).
- **PERCENTOF** inside GROUPBY gets the group's values and all values: each group is its share of
  the grand total, and the total is 1 (G!S97).
- **Values**: several value columns aggregate side by side (G!J73). SUM ignores text and
  Booleans in values (G!S193). An error value makes its group's result and the total that error
  (G!J193). Numeric keys sort numerically (G!AB169, G!S241).
- **Errors** (decoded with ERROR.TYPE, T-1328): values one row shorter than the keys (G!AB193), longer or offset values,
  a shorter key range, and PIVOTBY with short values are all `#VALUE!` (ERROR.TYPE 3). So are total_depth 3 (one or two keys, G!J217) and -3 (one key only probed)
  and total_depth 2 with a single key field.

### PIVOTBY(row_fields, col_fields, values, function, [field_headers], [row_total_depth], [row_sort_order], [col_total_depth], [col_sort_order], [filter_array], [relative_to])
- **Shape**: a header row (column keys plus "Total"), then one row per row group and a "Total"
  row. The first column holds the row keys. The corner cell is `""` (P!A1). A row/column
  combination with no records is `""`, not 0 (P!A1, East/2025). A combination whose values sum
  to 0 shows 0 (West/Apple/2023 in P!A97).
- Column keys sort and group like row keys. The total column applies the function to the whole
  row group, and the bottom-right cell to everything.
- **field_headers 3** adds, above the column keys, a row with the column field name, and below
  them a row with the row field name followed by the value field name under every column
  (P!M1). The column field name ("Year") sits only above the first column key; the rest of
  that row is `""`.
- **row_total_depth / col_total_depth**: `0` drops the total row / column (P!Y1, P!A25).
  `2` adds subtotals, labeled as in GROUPBY with "Grand Total" (P!A97 rows; P!M97 columns, where
  a subtotal column has the first-level key in the first header row and `""` in the second).
- **row_sort_order / col_sort_order**: `-1` reverses the keys (P!M25). `2` sorts rows by the
  total column ascending, and `-2` sorts columns by the total row descending (P!M121).
- **filter_array**: as in GROUPBY. Filtering out 2025 drops that column (P!Y73).
- **Several value columns** interleave per column key (2023/Sales, 2023/Units, ...). There
  are two Total columns (one per value column, 9 columns in all), both headed `Total` and `""` in every row, including the Total row
  (P!Y97, re-checked in followup G!A281: header row `""`,2023,2023,2024,2024,2025,2025,Total,Total; Total row 35,15,86,21,9.5,19,`""`,`""`). The Total
  row itself is filled per value column. Unlike several functions, several value columns get
  no header row naming them (no "Sales"/"Units" row).
- **Several functions** (`HSTACK(SUM,MAX)`) interleave the same way. A second header row names
  the functions, and the Total columns are filled (P!A121).
- **relative_to** (PERCENTOF and other two-argument functions get the cell's values and a
  reference set). Omitted or `0`: the column total (each column sums to 1, P!Y25). `1`: the row
  total (P!A49). `2`: the grand total (P!M49). With one row field and one column field, `3` gives
  `1`'s numbers and `4` gives `0`'s (P!Y49, P!A73). They are the "parent" totals, which with a
  single level are the row and column totals.
- **COUNT** and the other aggregates behave as in GROUPBY (P!M73).

## Design
Module `crates/rustsheet-engine/src/functions/lookup_and_reference/groupby.rs` (new; the group
and pivot helpers sit next to the other table functions). No public API change.

```rust
/// Rows grouped by a key tuple: the unit GROUPBY lays out as rows and PIVOTBY as a grid.
struct Grouping {
    /// One entry per group, sorted; `rows` are indices into the (filtered) source rows.
    groups: Vec<Group>,
}
struct Group { keys: Vec<ArrayNode>, rows: Vec<usize> }

/// What one aggregate cell is computed from: the rows of its group and the rows of the
/// `relative_to` set (PERCENTOF's second argument; all rows for GROUPBY).
struct Aggregate<'a> { rows: &'a [usize], relative: &'a [usize] }
```

1. **Arguments**: `row_fields` / `col_fields` / `values` go through `Model::eval_to_array`
   (spill_functions.rs). `filter_array` must be a column of the same height. Headers are split off
   by `field_headers`, and auto-detect uses the rule probed above. The detection rule (T-1328, followup3
   matrix over array constants): only the first row of the *values* decides; the key's first cell is irrelevant. The row is a
   header when its first value cell is text and the next row's is not text (`{"v";1;2;3}`: header; `{5;1;2;3}`, `{TRUE;..}` and
   `{#N/A;..}`: data; all-text `{"v";"1";"2";"3"}`: data; `{5,"w";..}` over two columns: data). A header is hidden
   (the key range then drops its first row too).
   (Probed with SUM and the first value column only; do not generalise to other functions or later columns.) The result is `#VALUE!` when the first value cell is text and the rows below mix text with non-text
   (`{"v";1;2;"3"}`, followup3 G!A205/A277/A349; `"h","1",2,3`, followup2 G!A521; `"v","1","2",<blank>`, followup2 G!A341/A381). When the
   first cell is not text, mixed values aggregate normally and text is ignored (G!S193); all-text values SUM to 0.
2. **Grouping**: a hash map from `Vec<CellKey>` (`cell_key` in spill_functions.rs, case-folded
   text) to the group index. The key is built once per source row, never once per function or
   value column, and the map records each group's first spelling. Then sort the groups with
   `sort_key_cmp` field by field, with blank last in both directions. `cell_key` and
   `sort_key_cmp` are private today; widen them to `pub(crate)` and never copy them. Each call of
   `sort_key_cmp` converts both nodes and clones their text. If the group sort shows up in a
   benchmark, change it to compare borrowed values, or convert each group's keys once before
   sorting; that is a refactor of the shared helper, not a second comparator. Hierarchical
   totals (depth 2) need groups ordered by their key prefix. The lexicographic sort already gives
   that order, so the subtotal rows are inserted in one linear pass, not by any quadratic search.
3. **Aggregation**: for each (group, value column, function) build a column array of the
   group's values in source order and call the function value: a `LambdaCaller` (`Model::lambda_caller`, T-1560) for a
   LAMBDA, or the T-1466 eta-call path (`eta_definition`, lambda.rs) for a name. Any array result becomes `#VALUE!`, a 1x1 array included: never unwrap it
   (G!J121 is `#VALUE!` for the one-record groups too). A function array (`HSTACK`/`VSTACK` of function values) is a list of functions plus
   an orientation. Two-parameter functions (PERCENTOF) also get the relative set's values:
   - Each distinct relative set is built once and shared, never rebuilt per call. GROUPBY has one
     (all rows). PIVOTBY has one per row total, one per column total and one grand total
     (`Rc<[ArrayNode]>` or one array node).
   - The built-in PERCENTOF (eta name) gets a fast path: compute each relative set's denominator
     once and divide, so its total work is O(N).
   - A user LAMBDA with two parameters must receive the whole relative array on every call, so
     its work is O(G x N) (G output cells, N relative rows). It is charged against the shared
     cell-visit cap (`MAX_LIFTED_CELL_VISITS`, if_ifs.rs) before any call. The constant is private today: widen it, or a helper that charges visits against it, to `pub(crate)`; never copy it. A refusal is a
     directive-5 DEVIATION, which the implementing task (T-1325 / T-1327) records in arrays.md
     with a test.
4. **Sorting by values** (sort_order 2.. / row_sort_order) sorts groups by their aggregate in
   that column, with the totals pinned. The blank-key group is not pinned there: it sorts by its
   value like any group (South, North, 0, East, West in G!J145 and P!M121). Blank-last applies
   only to key sorts.
5. **Layout**: assemble `Vec<Vec<ArrayNode>>` with `ArrayNode::String("")` fillers and the
   labels "Total" / "Grand Total" (en-US; the engine is en-US only, CLAUDE.md).
6. **Limits** (directive 5):
   - Grouping is O(N x K) for N rows and K key fields, plus the O(G log G x K) group sort.
   - One-argument aggregation is O(N x value columns x functions) per axis of totals.
   - Two-argument functions are bounded as in step 3: O(N) for built-in PERCENTOF; a
     two-parameter LAMBDA is charged against the cell-visit cap.
   - The output size (groups x columns, and PIVOTBY's row groups x column groups) is checked with
     `check_array_size` before the output, or any per-cell function call, is allocated.
   - The number of LAMBDA calls is bounded by that output size and nothing else; BYROW has no
     separate call budget either.
   - Each implementation task adds an adversarial test: about 1e6 distinct keys in GROUPBY, and a
     PIVOTBY cross-product of two high-cardinality fields that must be refused or computed within
     the size check, not allocated first.
7. **Static analysis**: the result is `StaticResult::Unknown` (it spills), and the argument
   signatures are vectors except the integer options.

## Alternatives considered
- Implement PIVOTBY as a GROUPBY over (row keys, col keys) followed by a reshape. It is rejected
  as the public path: the reshape cannot produce the column totals and `relative_to` sets without
  recomputing them. The shared `Grouping` core gives the same reuse without a second pass.
- Reuse SORT on the output array. It is rejected because totals and the blank-last rule would have
  to be undone afterwards.

## Test plan
- probes/groupby-pivotby is the acceptance case: every anchor above, including the spilled
  cells (the oracle sidecar records each spill cell). Each implementation task names the anchors
  it turns green and removes them from `eval/baseline/probes.txt`.
- probes/m7-argument-modes F33-H35.
- Unit tests for grouping order (mixed types, case folding, blank last when descending) and for
  the `""` fillers vs the 0 blank key.
- The generator (`rustsheet-gen --milestone M7`) already emits both; its pass rate should only
  rise.

## Rollout
Ordered; each is S/M. The grouping core lands with GROUPBY; PIVOTBY reuses it.
1. GROUPBY core (opus, M): grouping, ordering, LAMBDA functions, field_headers 0-3 and
   auto-detect, total_depth 0/±1, `""` fillers, several value columns, errors.
2. GROUPBY options (sonnet, S): sort_order (scalar and array), filter_array, the `#VALUE!`
   argument errors.
3. GROUPBY subtotals (opus, M): total_depth ±2 with "Grand Total", field_relationship.
4. Function arrays (sonnet, S; after T-1466): eta names, HSTACK/VSTACK of functions, and the
   PERCENTOF relative set in GROUPBY. DONE (T-1325): labels, pairing with value columns, arities and the
   reference result are in docs/excel-semantics/arrays.md (probes/groupby-pivotby-functions*).
5. PIVOTBY (opus, M): the grid on the shared core, totals and subtotals on both axes, sort
   orders, filter, headers, several values and functions.
6. PIVOTBY relative_to (opus, S): the reference sets 0-4 for two-argument functions. DONE (T-1327): a row selection
   crossed with a column selection, parents one key field up, probed over two fields per axis and with subtotals
   (docs/excel-semantics/arrays.md, probes/groupby-pivotby-functions*).
7. DONE (T-1328) Probe follow-up (sonnet, S; research): the undecoded cells (G!AB193, G!J217),
   field_relationship 1 with subtotals (G!J169), the two-values PIVOTBY Totals (P!Y97), the
   multi-key sort array, the header auto-detect rule, errors as keys, and whether all-FALSE
   filters are `#VALUE!` or `#CALC!`. A cell excel-graph cannot answer goes to M14 Phase 4
   (decision 0039). Done in T-1328: all questions answered by excel-graph, none left for Phase 4.
