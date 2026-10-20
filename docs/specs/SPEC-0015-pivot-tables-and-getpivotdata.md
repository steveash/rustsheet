# SPEC-0015 — Pivot tables of a loaded workbook, and GETPIVOTDATA against them

- **Status:** draft (accepted on merge of T-1708)
- **Milestone / lane:** M6 / engine
- **Tasks:** T-1708 (this spec), T-2081, T-2082, T-2083, T-2084, T-2085, T-2086

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

Unknowns. Each is probed by T-2085, and until then T-2083 answers `#REF!` for it (or keeps today's
T-1703 behavior where one exists), never a guessed value:
1. several data fields: the `"Values"` field, what a blank `data_field` picks, and a source name
   shared by two data fields (Sum and Count of Sales);
2. an empty intersection of any aggregate but Sum (H13 covers Sum only);
3. numbers, dates and booleans as items (match by value or by text);
4. page (filter) fields with a selected item, and hidden items on an axis;
5. the same field twice with the same item;
6. an error, blank or text value in an aggregated record's data field (the probe source has none);
7. an array `data_field`, `pivot_table` or field name (only array items are evidenced: L1, L5;
   web.md records the others as unprobed; T-1703 gives one `#REF!`, kept);
8. `""` as `data_field` against a pivot (J7 is an empty cell; `""` is only probed with no pivot);
9. a reference overlapping two pivots, or pointing into a page-field area outside `<location>`;
10. two row fields and their subtotals, tabular and outline form;
11. whether the answer follows the cache when the records disagree with the rendered cells (a file
    saved without refreshing), and the summation order (cache order or display order) on
    non-representable decimals;
12. grouped fields (`<fieldGroup>`, date grouping), calculated fields, and a cache saved without
    records (`saveData="0"`): skipped at import (`#REF!`), a non-parity gap T-2085 records.

## Design

### Data model (T-2081)
New types in `crates/rustsheet-engine/src/pivot.rs`, held by `Workbook` (bitcode `Encode`/`Decode`
like `Table`; adding fields changes the internal encoding, which no stored format depends on).
Records are stored by column as indices into each field's interned items, so a record costs 4
bytes per field, no per-record allocation, and matching is an integer compare:

```rust
pub struct PivotCache {
    /// The `worksheetSource`, kept for a later refresh; `None` for other sources.
    pub source: Option<CacheSource>,
    pub fields: Vec<CacheField>,
    /// How many records; every `CacheField::records` has this length.
    pub record_count: u32,
}
pub struct CacheSource { pub sheet_name: String, pub range: String }
pub struct CacheField {
    pub name: String,
    /// The field's distinct values: its `<sharedItems>`, then inline record values interned at
    /// import (deduplicated), so records only ever hold indices.
    pub items: Vec<CacheItem>,
    /// Per record, an index into `items`.
    pub records: Vec<u32>,
}
pub enum CacheItem { Missing, Number(f64), Text(String), Boolean(bool), Error(Error), Date(f64) }

pub struct PivotTable {
    pub name: String,
    pub sheet_id: u32,            // the worksheet's stable `sheet_id`, not its index
    pub location: CellArea,       // `<location ref>`, 0-based, converted at the xlsx boundary
    pub cache: u32,               // index into Workbook::pivot_caches
    pub fields: Vec<PivotField>,  // one per cache field
    pub row_fields: Vec<AxisField>,
    pub column_fields: Vec<AxisField>,
    pub data_fields: Vec<DataField>,
}
/// `hidden` is a bitset over the cache field's `items` (bounded by the import limits).
pub struct PivotField { pub axis: Option<Axis>, pub hidden: Vec<u64> }
pub struct CellArea { pub top: u32, pub left: u32, pub bottom: u32, pub right: u32 }
pub enum Axis { Row, Column, Page }
/// A `<field x>` of `<rowFields>`/`<colFields>`: a cache field, or `x="-2"`, the Values field.
pub enum AxisField { Field(u32), Values }
pub struct DataField { pub caption: String, pub field: u32, pub function: Aggregate }
pub enum Aggregate { Sum, Count, Average, Max, Min, Product, CountNums, StdDev, StdDevP, Var, VarP }
```

`Workbook` gains `pivot_caches: Vec<PivotCache>` and `pivot_tables: Vec<PivotTable>`. Every index
in them (cache, field, shared item, data field) is validated at import (T-2082) and is read with
`get()` all the same: an out-of-range index is `#REF!`, never a panic. Location and sheet follow the
api-design convention (0-based `u32`, stable `sheet_id`, docs/architecture/api-design.md principle 3).

Structural edits: tables have no such handling today (`workbook.tables` is only touched by
tables.rs, the parser, SUBTOTAL and export), so this is new code, T-2086: the row/column
insert/delete actions (`actions.rs` `insert_rows`, `delete_rows`, `insert_columns`,
`delete_columns`) shift or shrink `location` like the cells they move, a deletion that removes a
pivot's top-left cell drops the pivot (Excel refuses that edit; the engine cannot), sheet deletion
drops the sheet's pivots, and a sheet rename rewrites `CacheSource::sheet_name`. Until T-2086
lands, any of these edits on a sheet holding a pivot drops that sheet's pivots (GETPIVOTDATA
`#REF!`), a known gap listed in the web.md section, not a silent wrong answer.

### xlsx import (T-2082)
`crates/rustsheet-xlsx/src/import/pivots.rs`, next to `tables.rs`:
- `xl/workbook.xml` `<pivotCaches><pivotCache cacheId r:id>` → each cache definition part; its
  relationship → `pivotCacheRecords`. Every relationship target goes through the existing
  package.rs resolver (`resolve_target`): `TargetMode="External"`, absolute or escaping (`..`)
  targets, a missing part, a duplicate or unknown `cacheId`, and a definition reached twice skip
  that cache or pivot, never a panic or a read outside the package.
- `pivotCacheDefinition`: `<cacheSource type="worksheet"><worksheetSource ref sheet|name>`,
  `<cacheField name>` with `<sharedItems>` (`s`, `n`, `b`, `e`, `d`, `m`); other source types and
  `<cacheHierarchies>` (OLAP) give no pivot. `<fieldGroup>` and calculated fields (`formula`
  attribute) make the cache unusable: the pivot is skipped, so GETPIVOTDATA on it is `#REF!`
  (UNVERIFIED, logged by the importer as a skipped feature).
- `pivotCacheRecords`: `<r>` rows of `x v` (shared index) or inline `s n b e d m` values, the inline
  ones interned into the field's items. With no records part (`saveData="0"`) the cache and its
  pivots are skipped, so GETPIVOTDATA is `#REF!` (unknown 12; Excel would refresh).
- each worksheet's relationships → `pivotTableDefinition`: `name`, `cacheId`, `<location ref>`,
  `<pivotFields>` (`axis`, `<items>` with `h="1"` hidden and `t=` subtotal rows ignored),
  `<rowFields>`, `<colFields>`, `<pageFields>`, `<dataFields>` (`name`, `fld`, `subtotal`, default
  `sum`).
- The rendered cells stay what they are: plain sheet values, read as today.

### GETPIVOTDATA (T-2083)
Arguments as today: `data_field` and `pivot_table` evaluated first and their errors returned; then:
Names are resolved once per call against the pivot's small sets (data fields, axis fields, each
field's items), case-folded with `str::to_lowercase` there and never per record. Unknown n below
refers to the list in "Excel behavior to match".
1. The pivot: `pivot_table` must be a reference (a value is `#REF!`); the pivot on its sheet whose
   `location` intersects the reference's range (H11, H16); none is `#REF!` (H12, H20); more than
   one is `#REF!` (unknown 9). A pivot with a page field, or with any hidden item, answers `#REF!`
   (unknown 4).
2. The data field: an empty-cell `data_field` takes the data field when there is exactly one (J7);
   `""` keeps T-1703's `#REF!` (unknown 8). Otherwise the data field whose caption, or whose source
   field name, equals `data_field` ignoring case and untrimmed; none is `#REF!`; a pivot with more
   than one data field is `#REF!` (unknown 1).
3. The pairs: an odd count is `#REF!` (H15); each field name must match (ignoring case) a cache field
   on the row or column axis (H8, J3); each item must match a text item of that field ignoring case
   (H7, H18, J6, J8); a number, date or boolean item or field is `#REF!` (unknown 3); the same field
   twice is `#REF!`, with different items (H17) or the same (unknown 5).
4. The value: the data field's aggregate over the records whose fields equal every chosen item. A
   Sum over no records is 0 (H13); any other aggregate over no records is `#REF!` (unknown 2). A
   record holding an error, a blank or text in the data field makes the answer `#REF!` (unknown 6).
5. An array of items lifts per SPEC-0011 (L1, L5), each element a scalar call; an array
   `data_field`, `pivot_table` or field name keeps T-1703's single `#REF!` (unknown 7).

Aggregating the cache rather than reading the rendered cell keeps one rule for any layout and
gives the blank-intersection 0 directly. Evaluation does no I/O and reads only the model.

Dependencies: `pivot_table` is evaluated as a reference argument, so the dependency graph records
it as it does for any lookup function's reference argument. The answer also depends on the pivot's
existence, location and cache: any change to `pivot_tables` or `pivot_caches` (add through
`Txn::add_pivot_table`, move or drop by a structural edit, T-2086) must dirty every GETPIVOTDATA
cell of the workbook. The engine recalculates everything on each evaluation today
(`Model::evaluate`), so this holds now; an incremental engine must keep it. GETPIVOTDATA is not
volatile.

### Untrusted-input limits (T-2082, T-2083)
- Totals, counted while parsing and checked before each push, so nothing past them is allocated:
  at most 10,000,000 record values per cache (records x fields) and 1,048,576 records; 1,000,000
  shared items over all fields of a cache; 16,384 cache fields; 256 pivot fields on an axis, 256
  data fields and 64 pivots per sheet; 10,000,000 record values over the whole workbook. Past a
  limit the whole cache (and every pivot on it) or the pivot is dropped, never truncated, so the
  answer is `#REF!` rather than a wrong total, deterministically. T-2082 adds the `DEVIATION` entry
  to docs/excel-semantics/web.md (decision 0014) in the same PR. Parsing streams through the
  existing package reader, whose part-size limits also apply.
- Shared-item indices out of range, `fld` or field indices past the cache's fields, and a location
  outside the sheet make the pivot skipped, never a panic.
- GETPIVOTDATA work (required in T-2083, with a bench in crates/rustsheet/benches: 100k records,
  1k GETPIVOTDATA cells and a lifted-array case, numbers before and after the index): each pivot
  builds, lazily on first use, an index from its records with one accumulator per distinct tuple
  of axis-item indices: O(records) time, at most O(records) entries. The index lives as long as
  the pivot and its cache (it is derived state, skipped by the bitcode encoding) and is dropped only
  when that pivot or cache changes, not on other edits, since the cache never changes without a
  refresh. A call sums the accumulators of the tuples matching its pairs, and its answer is
  memoized in the index per (data field, pairs). So N calls cost O(records + N x tuples) at worst
  and O(records + N) when calls repeat. A lifted call is bounded by SPEC-0011's output-cell limit
  like every lifted function. The import never pre-sizes from a file's `count` attributes.
- The same cache builder (columnar, interning, counting the limits) serves the xlsx importer and
  `Txn::add_pivot_table`: one definition (directive 3).
- Results are never NaN or infinite: a non-finite result is `#NUM!`; an aggregate with no defined
  value (Average, StdDev, Var over too few records) is `#REF!` until T-2085 has Excel's answer,
  never a division by zero.

### Spec cases in the eval adapter (T-2084)
A spec case has no xlsx, and its rendered pivot cells exist only in the built file
(tools/src/rustsheet_tools/pivot.py). The layout stays in one place, the Python builder:
- `PivotTableSpec` gains `rendered: BTreeMap<String, CellInput>`, the pivot's cells as the builder
  lays them out. `uv run rustsheet-spec bake-pivots <case>` fills it; spec2xlsx refuses a case
  whose `rendered` differs from the builder's layout, so it cannot drift. The layout's match with
  Excel was a one-off check (web.md); T-2084 commits evidence for it: a probe reading the
  rendered label, caption, blank and grand-total cells the GETPIVOTDATA probes use (A5, C3, ...)
  through plain references, filled by the oracle.
- The adapter writes `rendered` as constants with the other values, then calls a new facade
  `Txn::add_pivot_table(PivotTableInput) -> Result<(), EditError>` (a typed `EditError::Pivot`
  variant via `thiserror`) that builds the cache from the source range's values (the constants
  already written: the same snapshot `refreshOnLoad` gives) and the definition from the spec's
  fields. It is test support, not public API: `#[doc(hidden)]`, not mirrored in the Python binding
  (explicitly deferred per api-design.md), and the only engine-side cache building.

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
1. T-2081 (engine, M): model types and `Workbook` fields; structural edits drop a sheet's pivots.
2. T-2082 (engine, M): xlsx import with the limits.
3. T-2085 (research, S): probe the unknowns; may run in parallel with 1-2.
4. T-2083 (functions-lookup, M): GETPIVOTDATA lookup; until it lands GETPIVOTDATA stays `#REF!`.
5. T-2084 (harness, M): baked rendered cells and the adapter facade call; it makes
   probes/getpivotdata-real-pivot score (needs T-2083 merged for the cells to pass).
6. T-2086 (engine, M): pivots follow structural edits; any time after T-2081.
