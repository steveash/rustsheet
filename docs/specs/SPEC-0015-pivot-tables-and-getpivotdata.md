# SPEC-0015 — Pivot tables of a loaded workbook, and GETPIVOTDATA against them

- **Status:** draft (accepted on merge of T-1708)
- **Milestone / lane:** M6 / engine
- **Tasks:** T-1708 (this spec), T-2081, T-2082, T-2083, T-2084, T-2085

## Problem
The engine has no pivot tables, so GETPIVOTDATA is `#REF!` whatever it is given (T-1703,
`functions/lookup_and_reference/getpivotdata.rs`). That is right with no pivot in the workbook
(web.md "GETPIVOTDATA without a pivot table") but wrong against a real one:
probes/getpivotdata-real-pivot has 22 known-failure cells (eval/baseline/probes.txt). The xlsx
importer drops `pivotCacheDefinition`, `pivotCacheRecords` and `pivotTableDefinition` parts, and
the eval adapter ignores a spec's `pivot_tables` (crates/rustsheet-eval/src/case.rs
`PivotTableSpec`), including the rendered cells that formulas such as `GETPIVOTDATA("Sales",$A$3,
"Region",A5)` read.

## Goals / non-goals
Goals:
1. Keep each pivot table of a loaded workbook (its cache and its definition) in the workbook model.
2. Import them from xlsx, bounded for untrusted files.
3. GETPIVOTDATA resolves `data_field` and field/item pairs against them, matching every cell of
   probes/getpivotdata-real-pivot.
4. Spec cases with `pivot_tables` reach the engine in the eval adapter with their rendered cells.

Non-goals: refreshing a pivot (recomputing its cache from the source range, `refreshOnLoad`),
creating, editing or laying out pivots in the engine, pivot charts, OLAP/external caches,
calculated fields and items, and xlsx export of pivots (dropped on export, as today; T-2081
records it). PIVOTBY is a function (SPEC-0006), not a pivot table.

## Excel behavior to match
Rules (docs/excel-semantics/web.md "GETPIVOTDATA against a pivot table", evidence
probes/getpivotdata-real-pivot, excel-graph 2026-10-16; cell ids are sheet P):

| Rule | Cells |
|---|---|
| `data_field` is the source field name or the data field's caption, case-insensitive, untrimmed | H1, H2, H10, J2, H19 (`" Sales "` `#REF!`) |
| a caption of another aggregate, or an unknown field, is `#REF!` | J4, H9 |
| a blank cell as `data_field` is the grand total (one data field) | J7 |
| no pairs: grand total; one pair: that row or column total; two pairs: the intersection, any order | H1, H3, H4, H5, H6, J1, J5 |
| field and item names ignore case; an item may be a reference to a cell holding it | H10, J8, J6 |
| `#REF!`: an item not in the field, a field not on an axis of the pivot, the same field twice with different items, an odd number of pair arguments | H7, H18, H8, J3, H17, H15 |
| an intersection with no source rows is 0 (Sum pivot) | H13 |
| `pivot_table` is any cell of the pivot or a range covering it; a cell outside every pivot (the source too) is `#REF!` | H11, H16, H12, H20 |
| an array of items lifts and spills | L1, L5 |

Errors in `pivot_table` or `data_field` propagate before any lookup (web.md "without a pivot
table"; kept as implemented by T-1703).

Unknowns, probed by T-2085 before T-2083 relies on them (until then T-2083 gives `#REF!`): several
data fields (`"Values"` field, which caption a blank `data_field` picks), an empty intersection of
an Average, Count, Max or Min pivot, numbers, dates and booleans as items (match by value or by
text), page (filter) fields and hidden items, two row fields and their subtotals, tabular and
outline form, and whether the answer follows the cache when the file's records disagree with the
rendered cells (a file saved without refreshing).

## Design

### Data model (T-2081)
New types in `crates/rustsheet-engine/src/pivot.rs`, held by `Workbook` (bitcode `Encode`/`Decode`
like `Table`; adding fields changes the internal encoding, which no stored format depends on):

```rust
pub struct PivotCache {
    /// `worksheetSource` sheet and A1 range, kept for a later refresh; `None` for other sources.
    pub source: Option<(String, String)>,
    pub fields: Vec<CacheField>,
    /// One value index or inline value per field, in field order.
    pub records: Vec<Vec<CacheValue>>,
}
pub struct CacheField { pub name: String, pub shared_items: Vec<CacheValue> }
/// A shared item or a record value; `Shared` (an index into the field's shared items) only in records.
pub enum CacheValue { Missing, Number(f64), Text(String), Boolean(bool), Error(Error), Date(f64), Shared(u32) }

pub struct PivotTable {
    pub name: String,
    pub sheet: u32,               // worksheet index
    pub location: (i32, i32, i32, i32), // `<location ref>`, 1-based rows/columns
    pub cache: u32,               // index into Workbook::pivot_caches
    pub fields: Vec<PivotField>,  // one per cache field
    pub row_fields: Vec<AxisField>,
    pub column_fields: Vec<AxisField>,
    pub data_fields: Vec<DataField>,
}
pub struct PivotField { pub axis: Option<Axis>, pub hidden_items: Vec<u32> }
pub enum Axis { Row, Column, Page }
/// A `<field x>` of `<rowFields>`/`<colFields>`: a cache field, or `x="-2"`, the Values field.
pub enum AxisField { Field(u32), Values }
pub struct DataField { pub caption: String, pub field: u32, pub function: Aggregate }
pub enum Aggregate { Sum, Count, Average, Max, Min, Product, CountNums, StdDev, StdDevP, Var, VarP }
```

`Workbook` gains `pivot_caches: Vec<PivotCache>` and `pivot_tables: Vec<PivotTable>`. A structural
edit (insert/delete rows or columns, rename or delete a sheet) moves or drops a pivot like a table
(`Table` handling in the same edit code). An overlapping pivot is cleared, as a table is.

### xlsx import (T-2082)
`crates/rustsheet-xlsx/src/import/pivots.rs`, next to `tables.rs`:
- `xl/workbook.xml` `<pivotCaches><pivotCache cacheId r:id>` → each cache definition part; its
  relationship → `pivotCacheRecords`.
- `pivotCacheDefinition`: `<cacheSource type="worksheet"><worksheetSource ref sheet|name>`,
  `<cacheField name>` with `<sharedItems>` (`s`, `n`, `b`, `e`, `d`, `m`); other source types and
  `<cacheHierarchies>` (OLAP) give no pivot. `<fieldGroup>` and calculated fields (`formula`
  attribute) make the cache unusable: the pivot is skipped, so GETPIVOTDATA on it is `#REF!`
  (UNVERIFIED, logged by the importer as a skipped feature).
- `pivotCacheRecords`: `<r>` rows of `x v` (shared index) or inline `s n b e d m` values. With no
  records part (`saveData="0"`), the cache has no records and the pivot is kept for location
  only; T-2083 then answers `#REF!` (UNVERIFIED; Excel would refresh).
- each worksheet's relationships → `pivotTableDefinition`: `name`, `cacheId`, `<location ref>`,
  `<pivotFields>` (`axis`, `<items>` with `h="1"` hidden and `t=` subtotal rows ignored),
  `<rowFields>`, `<colFields>`, `<pageFields>`, `<dataFields>` (`name`, `fld`, `subtotal`, default
  `sum`).
- The rendered cells stay what they are: plain sheet values, read as today.

### GETPIVOTDATA (T-2083)
Arguments as today: `data_field` and `pivot_table` evaluated first and their errors returned; then:
1. The pivot: `pivot_table` must be a reference (a value is `#REF!`); the first pivot on its sheet
   whose `location` intersects the reference's range (H11, H16) is used; none is `#REF!` (H12, H20).
2. The data field: a blank `data_field` takes the only data field (several: unprobed, `#REF!`);
   otherwise the first data field whose caption, or whose source field name, equals `data_field`
   ignoring case (`str::to_lowercase` on both, as other case-insensitive lookups do) and with no
   trimming; none is `#REF!`.
3. The pairs: an odd count is `#REF!`; each field name must match (ignoring case) a cache field on
   the row or column axis of this pivot; each item must match one of that field's shared items
   that is not hidden; the item value is compared as text for a text item and by value for a number
   (numbers, dates and booleans otherwise unprobed: T-2085). The same field twice with a different
   item is `#REF!`; with the same item it is accepted (unprobed, consistent with "a field once").
4. The value: the data field's aggregate over the cache records whose fields equal every chosen
   item. Records with an item hidden in any axis or page field are left out. A Sum or Count over no
   records is 0 (H13); other aggregates over no records are `#REF!` until T-2085 says otherwise.
   An error value in a summed record propagates (unprobed).
5. An array item, field or `data_field` lifts per SPEC-0011 (L1, L5), each element a scalar call.

Aggregating the cache rather than reading the rendered cell keeps one rule for any layout and
gives the blank-intersection 0 directly. Evaluation does no I/O and reads only the model.

Dependencies: the result depends on the pivot's cache only, which no edit changes without a
refresh (out of scope), and on the `pivot_table` reference's position, which the parser already
tracks. GETPIVOTDATA is not volatile.

### Untrusted-input limits (T-2082, T-2083)
- At most 1,048,576 records per cache, 16,384 cache fields, 1,048,576 shared items per field and
  256 pivots per sheet; past a limit the cache or pivot is skipped (GETPIVOTDATA `#REF!`) and a
  `DEVIATION` entry is added to docs/excel-semantics/web.md (decision 0014). Parsing is streaming
  through the existing package reader, whose part-size limits apply.
- Shared-item indices out of range, `fld` or field indices past the cache's fields, and a location
  outside the sheet make the pivot skipped, never a panic.
- One GETPIVOTDATA call is O(records x pairs). A per-pivot index (item tuple → accumulator) built
  lazily on first use and dropped on any model edit bounds a sheet of many GETPIVOTDATA calls at
  O(records) per recalculation; T-2083 adds it only if the perf reviewer asks for it, with a bench.

### Spec cases in the eval adapter (T-2084)
A spec case has no xlsx, and its rendered pivot cells exist only in the built file
(tools/src/rustsheet_tools/pivot.py). The layout stays in one place, the Python builder:
- `PivotTableSpec` gains `rendered: BTreeMap<String, CellInput>`, the pivot's cells as the builder
  lays them out. `uv run rustsheet-spec bake-pivots <case>` fills it; spec2xlsx refuses a case
  whose `rendered` differs from the builder's layout, so it cannot drift.
- The adapter writes `rendered` as constants with the other values, then calls a new facade
  `Txn::add_pivot_table(PivotTableInput)` that builds the cache from the source range's values (the
  constants already written: the same snapshot `refreshOnLoad` gives) and the definition from the
  spec's fields. This is the only engine-side cache building, and only for this facade call.

## Alternatives considered
- Reading the rendered pivot cell instead of aggregating the cache: needs the layout of every
  form (compact, tabular, outline, subtotals) in the engine and still has to special-case blank
  cells (H13 is 0). Rejected.
- A Rust port of the builder's layout in the adapter: two layouts to keep in step. Rejected for
  baking.
- Building an xlsx in the adapter by running spec2xlsx: makes the Rust harness depend on the
  Python tools at run time. Rejected.

## Test plan
- probes/getpivotdata-real-pivot passes all its cells (22 known failures fixed) once T-2082,
  T-2083 and T-2084 land; probes/getpivotdata-evidence-floor keeps passing.
- Unit tests: import of a hand-written minimal package (shared and inline record values, a
  `saveData="0"` cache, out-of-range indices, an OLAP source), each limit, and the structural-edit
  moves.
- An xlsx with a pivot (`crates/rustsheet-xlsx/tests`) round-trips its values and answers
  GETPIVOTDATA after import.
- T-2085's probe cases decide the unknowns before T-2083 relies on them.

## Rollout
1. T-2081 (engine, M): model types, `Workbook` fields, structural-edit handling.
2. T-2082 (engine, M): xlsx import with the limits.
3. T-2085 (research, S): probe the unknowns; may run in parallel with 1-2.
4. T-2083 (functions-lookup, M): GETPIVOTDATA lookup; until it lands GETPIVOTDATA stays `#REF!`.
5. T-2084 (harness, M): baked rendered cells and the adapter facade call; it makes
   probes/getpivotdata-real-pivot score.
