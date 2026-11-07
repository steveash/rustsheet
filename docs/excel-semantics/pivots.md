# Pivot tables (M16)

Rules for rendering, refreshing and authoring pivot tables. GETPIVOTDATA rules and the import limits
stay in web.md (SPEC-0015). Program: docs/research/pivot-tables-2026-11-04/plan.md, decision 0274.

Entries marked UNVERIFIED are observations from throwaway excel-graph runs (the 2026-11-04 spike in
docs/research/pivot-tables-2026-11-04/oracle-spike.md, and the case-builder-v2 runs of T-4122). Each
names the probe task that commits its evidence. Do not implement one before that. VERIFIED entries cite
committed probes.

## Excel for the web re-renders a refreshOnLoad pivot when it opens the file
Rule: a pivot whose cache has `refreshOnLoad="1"` is rebuilt from its source and re-rendered on open,
before any calculate or refresh call. The values the file's cache held do not survive.
Evidence: probes/t4128-pivot-spike R (excel-graph, 2026-11-06, T-4128). R's area held no cells, so
whether Excel also overwrites rendered constants already in the area is a separate, open question
(T-2327). DataF!C2 is a formula
(`=1000`). The sheet caches it as 0 and the pivot cache record is missing (`<m/>`), yet R!B5 (East/A) is 1004, R!E5 is 1012 and R!E7 is 1039.
The saved file holds Excel's own cache and definition (`rowItems`, `location`), as oracle-spike.md
run 1 also showed.
Status: VERIFIED (excel-graph)
Implemented in: crates/rustsheet-xlsx/src/import/pivots.rs (`refreshOnLoad` read onto
`PivotCache::refresh_on_load`; T-4151)
Implemented in: crates/rustsheet/src/workbook/calculation.rs (`Workbook::recalculate`: the first
recalculation of a loaded workbook refreshes the marked caches, then recalculates again) and
crates/rustsheet/src/edit/pivot.rs (`refresh_pivots_on_load`; T-4151)
Implemented in: crates/rustsheet-engine/src/pivot/grid.rs (`Model::plan_pivot_refresh`,
`apply_pivot_refresh`; tested on t4128 R in `pivot/grid/tests.rs`; T-4151)

## Excel for the web renders authored pivot definitions
Rule: a definition written into the file renders in full on open, with no `rowItems`/`colItems` and a
`location` of any size. Excel computes the layout, the values, the group items and the location. It
needs:
- a cache consistent with itself (shared items, records, flags);
- `<items>` on every axis field;
- an area holding no foreign cells.

Without items or records Excel refuses the file (`OpenWorkbookBlockedWorkbook`, oracle-spike.md run
2, a scratch observation).
Evidence: every pivot of probes/t4123-pivot-capture and probes/t4128-pivot-spike except S, the
`#SPILL!` control (excel-graph, 2026-11-06): compact, tabular, a calculated field with a Top-N filter and a sort, numeric grouping.
The case builder relies on it (tools/src/rustsheet_tools/pivot_definition.py).
Status: VERIFIED (excel-graph) for rendering. The refusal of an inconsistent package is a builder
observation (T-4122 runs). It cannot become a filled probe, because Excel saves nothing for a file it
refuses.

## Excel for the web opens engine-exported pivot parts
Rule: Excel for the web opens, without repair, a package whose pivot parts the engine's export wrote
(`crates/rustsheet-xlsx/src/export/pivots.rs`): a cache that lists every value as a shared item, records of `<x v>`
indices only, `sharedItems` flags derived from the items, the case builder's version attributes and `numFmtId="0"`
on every cache field; a definition with only the non-default typed attributes, the kept rest, and for a pivot made
through the API, generated `<items>` (`h="1"` on a hidden item) and no style or extension. For a `refreshOnLoad`
pivot, Excel's re-rendered `location` and `rowItems`/`colItems` equal the engine's lines the export wrote.
Evidence: the pivot-export suite (T-4177, eval/suites/pivot-export; excel-graph, 2026-11-07): each case is the
engine's export of its workbook, and its sidecar is Excel's save of that file. Every case opened without refusal:
- t4177-example.export.json (crates/rustsheet-xlsx/tests/example.xlsx, T-4176's first scratch run, input sha256
  3eed9bc4… then; an Excel-saved pivot over the Table `Table1`, no `refreshOnLoad`): Excel saved our `location`,
  `rowItems` and `colItems`, and all 58 scored cells equal. Without `refreshOnLoad` Excel saves a definition back as
  it read it (probes/t4586), so this shows the file opens, not that the engine's lines are Excel's;
- t4177-pivot-tables.export.json (pivot_tables.xlsx, the second run; two `refreshOnLoad` pivots, PivotSum with a
  column field): re-rendered; both pivots' `location`, `rowItems` and `colItems` equal the engine's, and all 66 cells;
- t4177-api-items.export.json (the third run: a pivot added through the API over texts, a blank, TRUE, a date, 7,
  2.5 and `#N/A`, `refreshOnLoad` off): opened and kept the engine's cells; GETPIVOTDATA on `East`, `West`, the date
  and the total read the exported cache as the engine does. t4177-api-items-refresh.export.json, the same with
  `refreshOnLoad="1"` patched onto the cache, is re-rendered from the generated `<items>`: Excel's lines and every
  value equal the engine's (its date item shows as `1/1/2020`, T-4943; GETPIVOTDATA on TRUE, 7 and 2.5 is T-4154);
- t4177-stale-row-items.export.json (probes/t4586 exported): pivot A, no `refreshOnLoad`, whose saved rowItems list
  East only although its source holds West too, is saved back with those lines as written and its cells; B (no
  lines, no refresh) and C (refreshed) equal the engine's too. All 41 cells equal.
A pivot the engine collapsed to `#SPILL!` (t4177-api-spill.export.json: no `rowItems`/`colItems`, a one-cell
location, no `refreshOnLoad`) is not opened as collapsed: Excel saved location A3:B3 with only `Row Labels` and
`Sum of Sales` and no lines, and GETPIVOTDATA on it gave `#REF!`. With `refreshOnLoad` patched
on (t4177-api-spill-refresh) Excel's refresh collapsed it again (`#SPILL!` at A3) and saved rowItems 3 and colItems 1,
as for its own collapsed pivots (probes/t4128-pivot-spike S). T-4942 owns the export's collapsed definition.
Not checked, all UNVERIFIED (T-4654): the exported shared-item order is the engine's (a cache's old items first,
new ones appended), not the order Excel was seen to save (source order first, stale items after; "Item casing when
cache and source disagree, and date items", t4612); `containsNonDate` and `minDate`/`maxDate` are never written, except on a field grouped in place by dates (T-4734;
next entry).
The cache definition's unread attributes and elements are not kept: the export writes `numFmtId="0"`, the builder's
version attributes and no refresh metadata or `extLst` (T-4735).
Status: VERIFIED (excel-graph) for opening without repair, for the saved lines written back as read, and for a
`refreshOnLoad` pivot's re-render (lines and cells) from saved or generated `<items>`; the collapsed `#SPILL!` pivot
is not opened as Excel saves one (above, T-4942). The engine's own lines are compared with Excel's in each case's
"Pivot layout" verdict, reported and not yet ratcheted (T-4944).
Implemented in: crates/rustsheet-xlsx/src/export/pivots.rs, pivots/cache.rs, pivots/table.rs (T-4176). The export
writes a pivot's saved `rowItems`/`colItems` back as read until the engine lays it out again (a render or refresh
drops them, `OpaqueEntry::is_layout_lines`), then the engine's; and it drops, with a log line, a cache whose field
holds two texts that differ only in case (next entry). A render that collapses to `#SPILL!` drops the saved lines
too, and the export then writes the definition with no `rowItems`/`colItems`, which Excel opens as a header-only
pivot (t4177-api-spill above; T-4942).

## Excel for the web opens engine-exported grouped pivot caches
Rule: Excel for the web opens, without repair, and renders a refreshOnLoad pivot whose grouped cache the engine's
export wrote (T-4734) in the case builder's shapes (`tools/src/rustsheet_tools/pivot_groups.py`, read off
Excel-authored caches; its golden parts are the builder's output, not files Excel saved):
- a field grouped in place by a range lists its own values as shared items (`<x v>` records), then
  `<fieldGroup [par] base>` (`base` itself) with `<rangePr autoStart="0" autoEnd="0" ...>` (`startNum`, `endNum`,
  `groupInterval`, or `groupBy`, `startDate`, `endDate`) and its group items as `<groupItems>`;
- a date level Excel adds (`Quarters`, `Years`) is a `databaseField="0"` cacheField with no shared items and
  `<fieldGroup base>`, its `rangePr` and `groupItems`; the in-place field's `par` names the last level;
- a manual group is a `databaseField="0"` cacheField with `<fieldGroup base>`, a `<discretePr>` entry per base
  shared item and `<groupItems>`; the base field has only `<fieldGroup par/>`.
A field grouped in place by dates needs `containsNonDate="0"` (the schema's default is on) and `minDate`/`maxDate`
on its `sharedItems`: without them Excel's refresh on open dropped the date grouping and listed each date (DC
showed `44016`, `44211` where the months belong; every GETPIVOTDATA on a month or level was `#REF!`). The export
writes them for such a field whose values are all dates, `minDate` and `maxDate` its first and last date. What Excel
saved for the same field is in the sidecar eval/suites/probes/cases/t4170-pivot-date-levels.oracle.json
(`pivots[0].cache_fields[0].shared_flags`: `containsNonDate="0"`, `minDate` 2018-12-15, its first date, and
`maxDate` 2021-01-16, a day past its last date 2021-01-15). Whether the last-plus-one-day `maxDate` matters is
T-4654's.
Evidence: T-4734's scratch runs (excel-graph, 2026-11-07, `_recalc_cached` of the engine's exports, input sha256
df3fc476…, 0c8366fd… and e0579e72…, f3b660b2…, 2b8ff5a0…, 2371c560…, a2befb89…), committed by T-4177 as cases of
eval/suites/pivot-export (excel-graph, 2026-11-07), each the engine's export of the probe built from its case:
- t4177-manual-groups.export.json (t4170-pivot-manual-groups): opened; 7 pivots re-rendered, every pivot cell and
  the engine's lines equal Excel's (the GETPIVOTDATA cells that fail are the probe's own, T-4154);
- t4177-date-levels.export.json (t4170-pivot-date-levels, with `containsNonDate="0"`, `minDate`, `maxDate`):
  opened; all 410 pivot cells and the lines equal (the failing GETPIVOTDATA cells are the probe's own, T-4154);
- t4177-date-flags-needed.export.json, the same export with those three attributes removed (T-4734's first run):
  opened, and Excel's refresh dropped the months in every pivot: DC lists the nine dates in DC!A4:A12 where
  the months belong, DX keeps only its data header and grand total (511), GETPIVOTDATA on a month or level is `#REF!`. 125 cells differ from the
  engine, which keeps the grouping (T-4941);
- t4177-date-groups-no-blank.export.json (t4236): all 447 cells equal; t4177-range-groups.export.json (t4228): all
  244 equal; t4177-spike.export.json (t4128): G (a numeric range group) and R equal, K differs by its sort as the
  probe does (T-4768), and S's collapse is T-4942's;
- t4177-grouped-cache.export.json: the cache shape of T-4734's API-built run as case-builder definitions (Amount
  grouped in place by tens, Date by months with Quarters and Years, Product2 a manual group over Product, all in each
  of three caches; the facade's `Txn::add_pivot_table` takes one range grouping of the row field, so it cannot build
  the levels or the manual group): opened, and the three pivots' cells and lines equal the engine's; GETPIVOTDATA on
  Years 2019, 2020 and `<1/1/2019` (31, 480 and 512 in Excel) and on the manual group are T-4154's.
Status: VERIFIED (excel-graph), including that a field grouped in place by dates needs the flags. Which of the three
it needs is T-4941's. Not checked: a group renamed through its item's `n`, groups over blanks, a 1904 workbook (the
import refuses its date groups, so the export drops them), and whether `autoStart`/`autoEnd` on (the model keeps
no such flag; the export writes them off, which keeps the bounds the engine groups by) would change Excel's refresh.
Implemented in: crates/rustsheet-xlsx/src/export/pivots/groups.rs (`check`, `write_field_group`,
`write_date_flags`; T-4734). The model keeps a field's own values beside its range grouping
(`CacheField::ungrouped`, kept by `group_by_numbers` and `group_by_dates`), and the export writes a grouping only
when grouping those values again by the import's rule gives the field it holds; any other shape (a numeric level or
a manual group beside a range grouping, two manual groups of one field, an added field before a database field)
drops the cache and its pivots with a log line. The export's cache limits count the kept values too.

## Case-variant text items make Excel for the web call the file corrupt
Rule: a cache field whose shared items list two texts that differ only in case (`East`, `east`) makes
Excel for the web refuse the workbook (`FileCorruptTryRepair`). With them merged into one item, the
records of both casings count under it on refresh: East/A summed the `East` and `east` rows (14 = 10
+ 4) and showed `East`.
Evidence: T-4122 runs (cases a and a2: corrupt with both items, rendered with one). The merge on refresh
is VERIFIED by probes/t4134-pivot-items I1 (next entry).
Status: the refusal is a builder observation (T-4122 runs), not committable as a probe: Excel saves
nothing for a file it refuses
Implemented in: crates/rustsheet-xlsx/src/export/pivots/cache.rs (`check`: the export drops such a cache with
its pivots and a log line, `CacheField::has_case_variant_texts`; T-4176)
The check folds case as the text functions do (`ItemKey::identity`); the refusal was observed for ASCII only, so
whether Excel's corrupt-file check folds non-ASCII text the same way (`ß`/`SS`, dotted I) is UNVERIFIED (T-4640).

## Which function the countA and count custom subtotals compute
Rule: a pivot field's `countASubtotal` (subtotal line `item t="countA"`) is COUNTA: it counts every
non-blank value, text, booleans and errors included (the data-field `count`; but see Errors below). `countSubtotal` (`t="count"`)
counts numbers only (Count Numbers, the data-field `countNums`); text and booleans are not numbers. Both
lines are labelled `<item> Count`, so with both on, two lines read `East Count`; the line's place
(the `<items>` order, `countA` first in B1) is the only visible tell; the file's `rowItems` carry
`t="countA"` and `t="count"`. The subtotal function overrides the data field's
aggregate: under Sum of Val the lines are counts, not sums, and stay numbers while the Sum cells and the
Grand Total show `#N/A`.
Errors, as observed (one outer field with the subtotals, one inner row field, one `#N/A`): under a Sum
data field, the error ends the count within its inner-item cell. In record order, countA counts the error
and nothing after it, and count counts the numbers before it. Other cells count in full. Under a Count
data field the error ends nothing. The scope of this rule is open: T-4636.
Evidence: probes/t4622-pivot-count-subtotals (excel-graph, 2026-11-06, T-4622). Rows Region (custom
subtotals only) and Product, tabular. East holds five records, A: 10, `abc`; B: a blank, `#N/A`, 5. West
holds 7 (A) and `x`, TRUE, 3 (B).
- **countA alone, Sum of Val (CA):** `East Count` 3 (A6, C6), `West Count` 4 (A9, C9).
- **count alone (CN):** `East Count` 1 (C6), `West Count` 2 (C9).
- **Both (B1):** C6:C7 are 3 and 1, C10:C11 4 and 2, and A6, A7, A10 and A11 all read `East Count` or
  `West Count`.
- **Both, no error (B2, Clean, with the `#N/A` replaced by 2):** East is 4 and 3 (C6:C7), West 4 and 2.
  This is COUNTA and COUNT exactly: the blank is skipped, and `TRUE` counts for countA only.
- **Both, Count of Val (B3):** East is 4 and 2 (C6:C7). The error counts for countA and nothing is cut.
- **Error first (B4, East A: `#N/A`, 10; East B: `abc`, a blank, 5):** East is 3 and 1. A counts the
  error only, and B counts in full (2 and 1). Record order across the whole item would give 1 and 0.
- **Error last (B5, East B: a blank, 5, `#N/A`):** East is 4 and 2.
- probes/t4137-pivot-captions K5 (all numbers): `East Count` counts 4.

Status: VERIFIED (excel-graph) for the functions and the labels. The error rule is as observed
(UNVERIFIED beyond the shape probed: T-4636).
The engine's own readings, not probed (UNVERIFIED, T-4636):
- with a column field, a subtotal cell adds the cut counts of every inner-item cell (row leaf by
  column leaf) it covers;
- with two or more inner row fields, the count is cut per innermost row leaf;
- under any data field other than Sum the count is not cut (Count probed in B3; CountNums and the
  rest UNVERIFIED): the custom count keeps the Count total rule ("A Count total over an error
  field...");
- under Sum, the other custom functions (sum, avg, max, ...) over an error still return the error.
Implemented in: crates/rustsheet-xlsx/src/import/pivots/table.rs (`SUBTOTAL_FUNCTIONS`, the model);
crates/rustsheet-engine/src/pivot/accumulate.rs (`CutCount`, `Accumulator::cut_count`) and
result.rs (`PivotResult::value_as`) for the values; layout/render.rs (`function_word`) for the labels
(T-4168)

## Item identity: exact for numbers and types, case-insensitive for text
Rule: on refresh, texts that differ only in case are one item, and their records add up. In I1 the item
shows `east`; see "Item casing when cache and source disagree, and date items" (T-4582, T-4612). Numbers are
compared exactly,
so 0.3 and 0.30000000000000004 are two items. The
text `"1"` and the number 1 are two items as well.
Evidence: probes/t4134-pivot-items (excel-graph, 2026-11-06, T-4134):
- I1: east, East and EAST count 3 under `east`, and west counts 1.
- I4: 0.3 counts 2 and 0.30000000000000004 counts 1.
- I5: the number 1 counts 2 and the text `1` counts 1.

Status: VERIFIED (excel-graph)
Implemented in: crates/rustsheet-engine/src/pivot/refresh.rs (T-4141) (`ItemKey::identity`)

## Item casing when cache and source disagree, and date items
Rule:
- **Casing.** When texts that differ only in case merge into one item, the item keeps the listed
  item's slot and takes the casing of the first source record that reaches it, when that record is
  a formula the old cache held as `<m/>` (every probed case). Neither the listed casing nor the
  lowercase one decides. Whether a literal first record, whose value the cache already lists in
  another casing, relabels the slot the same way is UNVERIFIED (T-4654).
- **Date items.** A date-formatted source value does not match a cached number item of the same
  value. Its records leave the listed `<n>` items empty, so those are not rendered, and Excel appends
  new items for them, saved as date items (`<d v="2023-03-15T00:00:00"/>`). A date with a time part is
  its own item: 45000 and 45000.5 are two items under both formats probed. A date and an equal plain
  number in one field are two items as well. A time-only `h:mm` and an elapsed `[h]:mm` format count
  as dates. The appended items render as their serials, as number cells. Their shown text is the source
  cells' date format: on probes/t2085-getpivotdata-items sheet D (rows When over serials formatted m/d/yyyy) the
  sidecar's pivot `text` shows 1/1/2024, 1/2/2024 and 2/1/2024 (D!A4:A6). The rendered rowItems point at the `<n>`
  items 45292, 45293 and 45323 (x = 3, 4, 5), while the `<d>` items are listed first. The engine shows the serials
  (T-4807). The saved cache field (`containsMixedTypes="1"`, count 6) carries `minDate="2024-01-01T00:00:00"` but
  `maxDate="1900-01-01T12:51:04"`, below its own minDate; nothing reads these attributes, so the quirk is recorded only.
- **Saved order.** In the saved cache Excel lists the shared items in source order first and the
  stale ones (old items no record holds) after them, while the pivot field's items keep the old order
  with the new ones appended: `rowItems` index the pivot field's order. See T-4654.

  Plain numbers do reuse the listed slots (probes/t4134-pivot-items I4, `rowItems` x = 0, 1).

Evidence: probes/t4582-pivot-item-edges (excel-graph, 2026-11-06, T-4582):
- J1: the source's K is the formula `="east"`, then `EAST`, `East` and `west`. The builder lists
  `<m/>`, `EAST`, `west`. The saved `rowItems` are x = 1 (shown `east`, count 3; J1!A4:B4) and x = 2
  (`west`, 1). So the `EAST` slot is reused and relabeled, and the `<m/>` slot is dropped, as in J2
  and t4134 I3.
- J3 (`m/d/yyyy`) and J4 (`m/d/yyyy h:mm`): K is 45000, 45000.5, 45000, 45001. The builder lists
  `<n>` items 45000, 45000.5 and 45001. The saved `rowItems` are x = 3, 4, 5: three appended items,
  shown 45000 (count 2), 45000.5 (1) and 45001 (1) as number cells (J3!A4:A6, J4!A4:A6). The listed
  items 0–2 are not rendered.

Evidence: probes/t4612-pivot-date-items-and-casing (excel-graph, 2026-11-06, T-4612). The sidecar's
`cache_fields` carry each saved shared item's type (`shared_items`) and the `<sharedItems>`
attributes (`shared_flags`). In C1 to C3 the first record is a formula, which the case builder caches
as `<m/>`, and the "listed" item is the one the builder wrote (pivot_definition.py caches the first
literal's casing), not one the sidecar records:
- C1: K is `="EAST"`, `east`; listed `east`. C1!A4 shows `EAST` (count 2, x = 1): the first record's
  casing over the listed, lowercase and last one.
- C2: K is `="East"`, `EAST`, `east`; listed `EAST`. C2!A4 shows `East` (count 3, x = 1).
- C3: K is `="East"`, `east`, `EAST`; listed `east`. C3!A4 shows `East` (count 3, x = 1). C2 and C3
  rule out the listed, the lowercase and the last record's casing.
- T1 (`m/d/yyyy`): K is 45000, 45001, 45000; listed `<n>` 45000, 45001. Saved `rowItems` x = 2, 3,
  shown 45000 (2) and 45001 (1) (T1!A4:B5). The saved shared items are `<d>` 2023-03-15T00:00:00,
  `<d>` 2023-03-16T00:00:00, then the old `<n>` 45000 and 45001.
- T2 (`h:mm`, 0.25, 0.5, 0.25) and T3 (`[h]:mm`, 1.5, 2.25, 1.5): the same shape. x = 2, 3, shown
  0.25 and 0.5 (T2!A4:A5), 1.5 and 2.25 (T3!A4:A5), saved as `<d>` 1899-12-30T06:00:00 and
  1899-12-30T12:00:00 (T2), 1899-12-31T12:00:00 and 1900-01-01T06:00:00 (T3).
- T4: K is 45000 (`m/d/yyyy`), `x`, 45001 (`m/d/yyyy`); listed `<n>` 45000, `x`, `<n>` 45001. Saved
  x = 1, 3, 4: `x` reuses its slot (T4!A4), the dates are appended (T4!A5:A6). Shared items `<d>`,
  `x`, `<d>`, then the old `<n>` 45000 and 45001.
- T5: K is 45000 (`m/d/yyyy`), then 45000 and 45001 General; listed `<n>` 45000, 45001. Saved
  x = 0, 1, 2, shown 45000, 45001, 45000, each count 1 (T5!A4:B6): the date is a third item. Shared
  items `<d>` 2023-03-15T00:00:00, `<n>` 45000, `<n>` 45001.

Status: VERIFIED (excel-graph), the casing rule for a formula (uncached) first record only; a literal
first record is UNVERIFIED (T-4654). The saved shared-item order and the stale items' place in it are as
observed; what the export must write is T-4654's.
Implemented in: crates/rustsheet-engine/src/pivot/refresh.rs (T-4141) (`FieldReader::push` relabels the
reused slot with the first record's casing and appends `CacheItem::Date`; `ItemKey::identity` keeps
date items apart from numbers; `Model::is_date_style`)

## Shared-item flags Excel accepts for mixed fields
Rule: Excel for the web refuses a cache whose `sharedItems` flags contradict its items. The flags that
open are the ones Excel itself writes:
- booleans and errors count as strings;
- `containsSemiMixedTypes="0"` only when the field has no string and no blank;
- `containsString="0"` when it has no string;
- `containsMixedTypes="1"` when strings sit beside numbers;
- numbers with blanks carry `containsBlank="1"` and no `containsSemiMixedTypes="0"`;
- a date is a type of its own: `containsDate="1"`, and `containsMixedTypes="1"` beside a number. A
  field with a date carries no number flags (`containsNumber`, `containsInteger`, `minValue`,
  `maxValue`), even beside live numbers. Whether a date beside a string and no number is mixed is
  UNVERIFIED (T-4654): T4's field also held stale numbers.

As observed and still open (T-4654):
- stale items: in T1 (dates beside stale `<n>` items) Excel wrote `containsNonDate="0"`, so the stale
  numbers did not count there; in C1 (texts beside a stale `<m/>`) it wrote `containsBlank="1"`, so
  the stale blank did. T1's `containsMixedTypes="1"` may come from the stale numbers.
- `minDate` is the least date, but `maxDate` is below it in every date field: T1, T4 and T5
  `1900-01-10T12:50:04` beside `minDate` 2023-03-15T00:00:00, T2 `1899-12-31T03:45:03`, T3
  `1899-12-31T00:03:04`.

Evidence: T-4122 runs (bool+text, error+text and number+blank refused with other flags, rendered with
these); Excel-saved caches in the LibreOffice fixtures (oss-engines.md §5).
probes/t4612-pivot-date-items-and-casing (excel-graph, 2026-11-06, T-4612), sidecar `cache_fields`
`shared_flags`:
- T5 (`<d>`, `<n>`, `<n>`, all holding records): `containsDate="1"`, `containsMixedTypes="1"`,
  `containsSemiMixedTypes="0"`, `containsString="0"`; no `containsNumber`, `minValue` or `maxValue`;
  `minDate` 2023-03-15T00:00:00, `maxDate` 1900-01-10T12:50:04.
- T4 (`<d>`, `x`, `<d>` and two stale `<n>`): `containsDate="1"`, `containsMixedTypes="1"`; the
  string flags at their defaults; no number flags.
- T1, T2, T3 (`<d>` items and stale `<n>` ones): `containsDate="1"`, `containsMixedTypes="1"`,
  `containsNonDate="0"`, `containsSemiMixedTypes="0"`, `containsString="0"`, and the `minDate` and
  `maxDate` above.
- C1 to C3 (one text item and a stale `<m/>`): `containsBlank="1"` only.
Status: builder observation (T-4122 runs) for the refused files, not committable as a probe: Excel
saves nothing for a file it refuses. The builder writes these flags (pivot_definition.py
`shared_items`), and every filled pivot probe opens with them. The date flags are VERIFIED
(excel-graph) for the fields above; a date beside a string without numbers, stale items,
`containsNonDate` and `minDate`/`maxDate` are UNVERIFIED (T-4654).
Implemented in: crates/rustsheet-engine/src/pivot/refresh.rs (T-4141) (`SharedItemsFlags`, written by the export, T-4176)

## Items show in the order the pivot field lists them; new items are appended
Rule: on refresh Excel keeps the order of a pivot field's `<items>`. An item the refresh finds that the
field does not list yet goes after the listed ones, in source order (first appearance), not sorted.
A listed item that has no records after the
refresh is not rendered, and the items after it keep their indices. How items render:
- a number item renders as a number cell;
- a boolean as the text `TRUE` or `FALSE`;
- an error item as the text of the error (`#N/A`), not an error value;
- a blank as `(blank)`.

Evidence: probes/t4134-pivot-items (excel-graph, 2026-11-06, T-4134):
- I2: the builder lists a, FALSE, 1, b, TRUE, 2, then #N/A and blank (first seen); Excel shows that
  order in I2!A4:A11, with A6 and A9 numbers, A10 the text `#N/A` and A11 `(blank)`.
- I3: Zed and Alpha come from formulas the cache records as missing; Excel shows M, Zed, Alpha. The
  source order is M, Zed, Alpha, M, which also reads as a descending sort of the new items. The listed
  `<m/>` item (index 1) holds no record after the refresh and is left out: the saved `rowItems` are
  x = 0, 2, 3.
- probes/t4582-pivot-item-edges J2 (T-4582): new items in the non-monotone source order Alpha, Zed,
  Beta (all formulas the cache records as missing) after the listed M. Excel shows M, Alpha, Zed, Beta
  (J2!A4:A7), which is source order, not ascending (Alpha, Beta, Zed) and not descending.

The order of a newly created pivot is a UI action and not reachable from a file: Excel's UI may sort its
items ascending (by type), unprobed (T-4869).
Status: VERIFIED (excel-graph)
Implemented in: crates/rustsheet-engine/src/pivot/refresh.rs (T-4141) (`FieldReader`)
Implemented in: crates/rustsheet-engine/src/pivot/layout/render.rs (`item_cell`: how an item renders; T-4149)
Implemented in: crates/rustsheet-engine/src/pivot/result.rs (`Order`: the display order of a computed
result)

## A calculated field applies its formula to the field sums
Rule: a calculated field `Sales*2+0.1` shows, at every level including subtotals and the grand total,
the formula over the sum of Sales, not the sum of the formula over the records. East/A (Sales 10 and
4) shows 28.1, not 28.2.
Evidence: probes/t4128-pivot-spike K!D6 28.1, K!D7 38.1 (East Total, 19*2+0.1) and K!D11 92.1 (Grand
Total, 46*2+0.1) (excel-graph, 2026-11-06, T-4128).
Status: VERIFIED (excel-graph); the edge cases are the next entry's (T-4505)
Implemented in: crates/rustsheet-engine/src/pivot/calculated.rs (T-4174)

## Calculated fields as Excel for the web renders them
Rule, per shape (each its own pivot at A3, Sum data fields unless said; Data has Region, Product, Units, Price,
Zero, Unit Cost and Bob's, six records, the sums in the case description):
- **Every level computes the formula over that cell's sums**, row and column totals included. Rev = Units*Price
  shows East Total 210 = 6×35 (not the rows' 150+5, nor the records' 85), West Total 207 = 9×23 and Grand Total
  870 = 15×58 (LV). With Region on columns, the Grand Total column of A is 360 = 9×40, not 150+40 (LC).
- **An intersection no record reaches shows 0, not a blank**: in LC Product C has no East record, and C/East
  (B7) is 0 = 0×0 like C/West (C7, Units 0). Whether that is the formula over sums of 0 (`Price/Units` would be
  #DIV/0!) or always 0 is UNVERIFIED (probe pending: T-4897); the engine evaluates the formula over sums of 0.
- **Operands need not be data fields**: LC shows only Sum of Rev, NV's Ut = Units*2 shows A 18 with no Units data
  field.
- **The summary function of a data field over a calculated field is ignored**: Count, Average, Max and Min of Rev
  each show what Sum shows (A 360, B 66, C 0, Grand Total 870), under captions that keep the verb (`Count of Rev`).
  Count of Units beside them counts records (A 3) (CNT). Other functions (StdDev, Product, ...) were not tried.
- **An error shows only where the sums produce it, and does not reach the totals**: PerUnit = Price/Units is
  #DIV/0! at West/C (Units 0) only, West Total 2.555555556 (23/9); Sq = SQRT(Units-3) is #NUM! at East/B and
  West/C, East Total 1.732050808 (√3); Ratio = Price/Zero is #DIV/0! at every level (DZ). The sidecar holds the
  errors as error values.
- **A text field enters as 0**: Rt = Region*2+1 is 1 at every level and Rs = Region+Units equals Sum of Units
  (NV).
- **Constants** show at every level, totals included: 5, -2.5, and #DIV/0! for 1/0 (K). A text constant `"a"`
  shows #VALUE! at every level (ET).
- **The functions tried see each field as one number, its sum** (on item and grand-total levels; a subtotal level was not tried with functions): IF(Units>3,1,0) is 1 for A (9), 0 for C (0) and 1 for the
  Grand Total; AND(Units>0,Price>10) shows 1 and 0, saved as numbers, not TRUE and FALSE; SUM(Units,Price) is 49
  for A; CHOOSE(2,Units,Price) is Price's sum; ROUND(Price/Units,1) is 4.4 for A (40/9), #DIV/0! for C and 3.9
  for the Grand Total (FA). INDEX(Units,1) is Units' sum and ROWS(Units) is 1 (both of which [MS-OI29500] and
  CALC list as not allowed: the web opens and evaluates them); LOOKUP(Price,Units) is Units' sum; LOOKUP(Units,
  Price), MATCH(Units,Units,0), MATCH(Units,Units) and VLOOKUP(Units,Units,1,FALSE) are #N/A at every level.
  Each matches the worksheet function over plain numbers in Q!A2:A8 (MATCH(9,9,0), MATCH(9,9) and
  VLOOKUP(9,9,1,FALSE) #N/A, LOOKUP(40,9) 9, LOOKUP(9,40) #N/A, INDEX(9,1) 9, ROWS(9) 1).
- **Names**: a source field with a space is written quoted, `'Unit Cost'*2` (A 16 = 2×8), and one with an
  apostrophe doubles it, `'Bob''s'+1` (A 8 = 7+1); a calculated field named `My Calc` shows as `Sum of My Calc`
  (NM). The oracle's cached download (not a committed file) saves these formulas as written.
- **A calculated field over a calculated field** evaluates the inner one at the same level first: Rev2 = Rev*2 is
  420 for East (2×210) and Mg = Rev-Units 204 (210-6); Grand Total 1740 and 855 (CC). A reference to a calculated
  field listed after it works: Fw = Fx*2 with Fx = Units+1 is 20 for A (EF).
- **A cycle is #NAME?**: Ca = Cb+1 with Cb = Ca+1, and Cs = Cs+1, show #NAME? at every level (EC, ES). The
  oracle's cached download (not a committed file) saves each of those formulas as `#NAME?+1`.
- **Several data fields** (three in LV, DZ, NV, K, NM and CC, five in FA and CNT) each compute on their own.

Formulas that make Excel for the web refuse the whole file are in "Pivot flags Excel for the web refuses".
Evidence: probes/t4505-pivot-calc-fields LV, LC, CNT, DZ, NV, K, FA, NM, CC and
probes/t4505-pivot-calc-field-functions FX, FR, FL, FK, FM, FN, FV, ET, EC, ES, EF and Q!A2:A8 (excel-graph,
2026-11-07, T-4505); the sidecars' `pivots` sections hold every cell.
Status: VERIFIED (excel-graph) for these shapes: rows Product, or Region and Product, LC with columns Region, Sum
data fields but in CNT. The two saved-formula observations rest only on the oracle's cached downloads (input sha256
1ad2d4d1… and d8291cc3…), not committed files, so they are UNVERIFIED; the engine's export writes each formula as it
was read (a cycle stays `Ca = Cb+1`, not `#NAME?+1`), which no probe can check (a sidecar holds cells, not the saved
parts). Not probed (UNVERIFIED, probe pending: T-4897, each the engine's choice): a calculated field with a number
format, under show-values-as (the engine applies the mode to the formula's values), on a grouped field (the importer
skips a cache with both), with a filter or Top-N by it (the engine has no result), beside a custom subtotal (it shows
the formula's value), a field named in another case (matched ignoring case), an
operand holding an error (the error enters the formula), a field that only names a cycle member (`#NAME?`), a
formula whose value is an array (`#VALUE!`), empty (0) or a bare boolean (`Units>3`: 1 or 0), and the refusals the engine adds to the web's (web.md's
skip list).
Implemented in: crates/rustsheet-engine/src/pivot/calculated.rs (`check_calculated_fields`, `Plan`: the formula is
read by the engine's parser, with `Parser::set_quoted_names` for `'Unit Cost'`, checked, and evaluated per cell with
each field replaced by the cell's Sum, in an empty sandbox workbook on a fixed clock, drawing on the pivot's work
budget), crates/rustsheet-engine/src/pivot/result.rs (`calculate`), crates/rustsheet-xlsx/src/import/pivots.rs and
crates/rustsheet-xlsx/src/export/pivots/cache.rs (T-4174).

## A Top-N filter changes the subtotals and grand totals
Rule: with a `top10 val="2"` count filter on Product by Sum of Sales, each region shows only its two
largest products. The subtotals and the grand total cover only the products shown: East Total is 19,
not 22 (C dropped), and the Grand Total is 46, not 49. A `sortType="descending"` label sort on Product
lists B before A within each region.
Evidence: probes/t4128-pivot-spike K!A5:C11 (excel-graph, 2026-11-06, T-4128): East B 5, A 14, Total
19; West B 7, A 20, Total 27; Grand Total 46.
Status: VERIFIED (excel-graph); ties, percent and sum Top-N, filter order and inner-field Top-N per parent:
"Top-N filters at their edges" and the two entries after it (T-4164)
Implemented in: crates/rustsheet-engine/src/pivot/sort/labels.rs (`display_order`; T-4768) (the label sort of the inner
Product: every cell of K passes)

## Numeric and date group labels are rewritten by Excel
Rule: verified and recorded by the M6 grouping work. See web.md "Grouped pivot fields import as their
group items" (T-4225, T-4228, T-4235, T-4236, T-4249, T-4253, T-4287, T-4311, T-4316). It covers
record placement, Excel's own labels whatever the file lists, whole-number labels `lo-(lo+interval-1)`,
relabel order, and groupings dropped over blanks or dates. probes/t4128-pivot-spike G (`45000-45100`,
`45100-45200`, `45300-45400` for fractional data, empty groups hidden) agrees with it.

Excel also tolerates numeric range-group items it does not rebuild. The case builder's numeric labels (`group_labels`,
`while lo <= end`) write one bin past `endNum`: t4128 G's `<fieldGroup>` lists 7 `groupItems`
(`<45000`, five bins up to `45400-45500`, `>45400`) where Excel's own grouping of startNum 45000, endNum
45400, interval 100 has 6 (`<45000`, four bins, `>45400`), and its pivotField lists 7 `<item x>`. Excel
opens the file (refreshOnLoad), rebuilds the groups and renders its own items without error.
Status: VERIFIED (excel-graph; evidence in web.md, the tolerance by t4128 G's sidecar)
Implemented in: crates/rustsheet-xlsx/src/import/pivots/table.rs (`read_item`): on a `refreshOnLoad`
cache an `<item x>` of a numeric range group past its rebuilt groups is dropped, and the refresh lays
the field out again; without `refreshOnLoad`, on a date or manual group, or on an ungrouped field, the
pivot is still refused (T-4171; t4128 G renders Excel's 6 items). UNVERIFIED (T-4802): whether Excel
tolerates such an item without `refreshOnLoad`, or on a date group or a manual group (whose labels Excel
does not rebuild).

## Manual groups and date levels as Excel for the web renders them
Rule (manual groups, probes/t4170-pivot-manual-groups): Excel for the web opens and renders a refreshOnLoad pivot
whose cache holds a discrete group in the shape Excel saves: the source field Product with a `par`-only
`<fieldGroup par="2"/>`, and an added field Product2 (`databaseField="0"`) with `<fieldGroup base="0">`, a
`<discretePr>` entry per Product shared item and `<groupItems>` F, Group2, Group1, North (the ungrouped item first,
then the groups). Groups: Group2 = A, B; Group1 = C, D; North = E alone; F ungrouped; S values are distinct powers of
two (A 1 and 64, B 2, C 4 and 128, D 8, E 16, F 32).
- Labels: each group shows its `<groupItems>` text as written. Group2, listed first, stays `Group2` and Group1
  stays `Group1` (no renumbering by position), and the renamed group shows `North` (MC!A4, A7, A10).
- Order: the group field's rows follow its pivotField's `<item x>` order. MP lists them (`item_order`) North, F, Group1, Group2 and
  shows them in that order (MP!A4, A6, A8, A11), which rules out the `<groupItems>` order (F, Group2, Group1, North),
  the groups then the ungrouped items, the order of each group's first source item (both Group2, Group1, North, F,
  as MC shows them), and sorting by label (F, Group1, Group2, North).
- An ungrouped item is an item of the group field under its own label, with itself as its one inner item (MC!A12 F,
  A13 F, 32). A group of one is a group row with its one item under it (North 16, E 16).
- Compact (MC): a group row carries its total (Group2 67, Group1 140); the inner rows are the base items (A 65,
  B 2). Outline form (MO, `compact="0"`): Product2 and Product get a column each, the header row names them, and a
  group's total is on its own row (C4 67). Tabular (MT): the first inner item shares the group's row, and a
  `<group> Total` row follows each group, ungrouped F and the group of one too (`Group2 Total` 67, `North Total`
  16, `F Total` 32). Custom subtotals sum and count on Product2 (MS) give `Group2 Sum` 67 and `Group2 Count` 3 (the
  records A, B, A), `North Count` 1 and `F Count` 1.
- The group field alone (MG) lists Group2 67, Group1 140, North 16, F 32. The source field alone (MB, Product2 off
  the axes) lists A to F with their own totals (C 132): the group does not change it.
- Excel saved `<groupItems>` as the file listed them (the sidecars' `cache_fields`).

Rule (date levels, probes/t4170-pivot-date-levels): Excel for the web opens and renders a refreshOnLoad pivot whose
date field Date is grouped by months in place (`<fieldGroup par="3" base="0">`, the last added field) with two added
fields Quarters and Years (`<fieldGroup base="0">`), each `rangePr` from 2019-01-01 to 2020-12-31 with `autoStart`
and `autoEnd` off. Records: 2018-12-15 1 (before startDate), 2019-01-10 2, 2019-02-20 4, 2019-04-05 8, 2019-11-30
16, 2020-01-10 32, 2020-07-04 64, 2021-01-15 128 (past endDate), 2019-01-25 256.
- Nesting: rows Years, Quarters, Date show each month under its quarter under its year, so January splits by year
  (2019 Jan 258, 2020 Jan 32; DC!A9, A17). Date alone (DM) adds January of both years (290). Years and Date (DY,
  Quarters off the axes) show the months under each year. Groups without a record are not shown (2019 has Qtr1,
  Qtr2, Qtr4; `showAll="0"`).
- The record before startDate shows as `<1/1/2019` at each level, nested under itself (DC!A4:A6, 1 each), and the
  one past endDate as `>12/31/2020` at each level (A20:A22, 128).
- Compact (DC) and outline form (DO): each year and quarter row carries its total (2019 286, Qtr1 262). Tabular (DT):
  `Qtr1 Total` and `2019 Total` rows, and `<1/1/2019 Total` and `>12/31/2020 Total` at both outer levels. Custom
  subtotals sum and count on Years (DS) give `2019 Sum` 286 and `2019 Count` 5 (its five records), `<1/1/2019 Count`
  1; Quarters keeps its default `Total` rows.
- Years on the columns (DX, Quarters on the rows): columns `<1/1/2019`, 2019, 2020, `>12/31/2020`, Grand Total; rows
  include `<1/1/2019` and `>12/31/2020` for Quarters too; Qtr1 is 262 under 2019 and 32 under 2020.
- Excel saved each level's `<groupItems>` as the builder wrote them (Excel's own labels, sidecar `cache_fields`).

Evidence: probes/t4170-pivot-manual-groups (MC, MO, MT, MS, MG, MB, MP) and probes/t4170-pivot-date-levels (DC, DO, DT,
DS, DY, DM, DX) (excel-graph, 2026-11-07, T-4170); the sidecars' `pivots` sections hold every cell, the layout lines
and the saved group items. GETPIVOTDATA on these pivots: web.md "GETPIVOTDATA on manual groups and date levels". The
part shapes follow Excel-authored caches: ClosedXML's pivot-table/shared-group-field.xlsx and
shared-nested-dategroup.xlsx (Excel 15.0300) and corpus-bench 52379 and 59932 (Excel 16.0300), whose in-place field's
`par` names the last added field (48357 writes none). Excel saved the date field's `maxDate` as 2021-01-16 though
its last date is 2021-01-15, and its `minDate` as 2018-12-15, its first date (probes/t4170-pivot-date-levels sidecar,
every pivot's `cache_fields[0].shared_flags`; VERIFIED for this field, which holds whole dates only; how Excel derives
the two in general is T-4654's question). From the oracle's cached downloads only (input sha256 463d15ec… and
1de4cf34…, not committed, so UNVERIFIED; T-4802 commits evidence of them): Excel saved the `par`, `base` and
`discretePr` as written.
Status: VERIFIED (excel-graph) for these shapes: one discrete group of a text field into three groups and one
ungrouped item (in its first-item order and in MP's order), and one date field with months, quarters and years from a startDate to an endDate, on the rows (DX:
Years on the columns), compact, outline and tabular, default or sum and count subtotals. These probes give `web` yes
for catalog rows PV-0022, PV-0023, PV-0026 (`groupBy` months, quarters, years), PV-0122 to PV-0125 (`discretePr`),
PV-0232, PV-0236 and PV-0237 (`rangePr@groupBy`, `fieldGroup@base`, `@par`), which T-4767 fills. UNVERIFIED (T-4802):
a group shared by two pivots of one cache, a group renamed through its item's `n`, discrete groups over numbers,
dates, blanks or errors, a group field on the columns or pages, days,
hours, minutes or seconds beside other levels, levels that skip one, the base field off the axes with only added
levels shown, the shape without `par`, and `showAll="1"` on a level.
Implemented in: the case builder (tools/src/rustsheet_tools/pivot_groups.py, the `discrete` and `date_group` field
entries; Rust mirror crates/rustsheet-eval/src/pivot_definition/groups.rs). The engine renders every pivot of both
probes (T-4171): the importer reads the source field's bare `<fieldGroup par>` as naming its group field, not as a
group, only when `par` names an added field whose `base` is that field, the shape evidenced here; any other bare
`<fieldGroup>` still skips the cache (crates/rustsheet-xlsx/src/import/pivots/groups.rs `apply_groups`), and keeps each added field's base
(`AddedGroup`); a refresh derives every added field again from its base's new values, before the base is grouped in
place, by the M6 rules (`regroup` for a date level, `group_by_discrete` for a manual group;
crates/rustsheet-engine/src/pivot/added_groups.rs), so the pivot's fields remap by name and the layout stages lay the
group fields out in every form. The engine's own readings, UNVERIFIED (T-4802): a base item new to the source becomes a
group item of its own, labelled with the item, after the old ones; the refresh is refused (the pivot keeps its saved
cells) when that label equals a group's ignoring case, when a date level's base gains a blank (where a refresh drops an
in-place grouping, web.md), or when the source gains a column named as an added field; an added field whose base left
the source goes with it. GETPIVOTDATA on these pivots: web.md "GETPIVOTDATA on manual groups and date levels".

## A pivot that would meet a foreign cell collapses to #SPILL!
Rule: when a pivot's layout would overwrite cells it does not own, Excel for the web renders nothing
but `#SPILL!` in the anchor cell, and saves the pivot's `location` as that one cell. This matches
Microsoft's 2025 "PivotTable #SPILL!" announcement (excel-docs.md, PSPILL).
Evidence: probes/t4128-pivot-spike S (excel-graph, 2026-11-06, T-4128): the constant S!C5 lies inside
the layout, and the pivot is saved with location A3, where S!A3 is `#SPILL!`.
Status: VERIFIED (excel-graph); GETPIVOTDATA against such a pivot: "GETPIVOTDATA against a pivot collapsed to
#SPILL!" (T-4130)
Implemented in: crates/rustsheet-engine/src/pivot/grid.rs (`Model::place`, `meets_foreign`: the
anchor gets `#SPILL!`, the location becomes that cell and the geometry zero, as the sidecar saves
it; tested on t4128 S in `pivot/grid/tests.rs`; T-4151)

## The grid writer's own readings
Rule (the engine's, not probed): a pivot owns its location and its page area, and nothing else.
- A refresh empties every cell it owned that the new layout does not write, and writes the new
  cells as constants. A constant left inside the pivot's own saved area is its own: overwritten
  or cleared, never foreign. Whether Excel overwrites rendered constants it finds in a
  `refreshOnLoad` pivot's area is T-2327's.
- Foreign is a stored value or formula outside the owned area (a cell holding only a style is
  empty), and, inside or outside it, any part of another pivot's area on the sheet (even a blank
  intersection, and the new area of a pivot laid out before it in the same refresh), a merged
  range, a Table (even an empty body) and an array formula's or dynamic spill's cells. The new
  page area is checked as the location is.
  Classic Excel instead refuses a refresh that would overlap another pivot, with a dialog
  (excel-docs.md, "Overlap rules").
- A merged range inside the pivot's own area, or across its edge, refuses the refresh: the pivot
  keeps its cells. Excel makes such ranges itself with "Merge and center cells with labels"
  (`mergeItem`, excel-docs.md), so a `refreshOnLoad` pivot saved with that option keeps its saved
  cells on open; the cell writers cannot write through a merge (rendering `mergeItem` is T-4677).
- A layout that would pass the sheet's last row or column collapses to `#SPILL!` too, rather than
  failing. Microsoft's PivotTable `#SPILL!` announcement lists text, merged cells, tables and the
  sheet edge as what blocks a pivot (excel-docs.md, PSPILL); that is documented for desktop Excel,
  and Excel for the web is unprobed for merged cells, Tables and the edge.
- On a collapse the location becomes the anchor and the geometry, page counts included, becomes
  zero. That is evidenced only for a pivot with no page field (t4128 S).
- A refresh reads the cache again and lays out every pivot over that cache, each from its
  anchor (the location's top-left cell).
- A pivot's field indices follow its fields by case-folded name. A field new to the source joins
  on no axis. A field the pivot shows (row, column, page or data) that left the source refuses the
  refresh, and the pivot keeps its cells. So does a renumbering of a pivot that keeps a verbatim
  part holding a field index: one with an attribute named `field` or `fld` or ending in either
  (ECMA-376's `pivotArea@field` and `reference@field` in `formats`, `conditionalFormats`,
  `chartFormats` and `autoSortScope`, `filter@fld`, `@mpFld` and `@iMeasureFld`, `mp@field`, an
  extension's `sourceField`), or one the scan cannot read to its end (an open quote or tag, an
  unterminated comment, CDATA section or processing instruction). How Excel remaps those indices,
  and the worksheet's `pivotSelection/pivotArea@field`, is T-4712's. The other verbatim parts
  renumber. The root's and the fields' kept attributes hold no field index: of the attributes the
  schema gives those elements (eval/catalog/pivot-schema-nodes.txt), the ones so named are
  `pageField@fld` and `dataField@fld`/`@baseField`, which the importer reads itself, and
  `pivotField@dataField` and `@serverField`, two flags. `pivotTableStyleInfo` holds none, nor do
  `rowItems`/`colItems` (`x@v` index the items of the axis's n-th field, which moves with it, and
  `i@i` a data field), though those go stale on any refresh: Excel rewrites them (P_Fold saves
  `x` 0, 2, 3, 4, 5), and the engine renders from its own layout. A render or refresh drops the
  saved ones, and the export writes the engine's lines in their place. A pivot the engine has
  not laid out keeps the saved ones, written back as read (T-4176). A field's own parts (its attributes, its items') follow its new index, and a dropped
  field's go with it (T-4694). A data field's base field that left the source falls back to the
  first field.
- A refresh that cannot be planned (a source that does not resolve, such as a Table or defined
  name until T-4506; a field gone; past the shared budget) leaves the pivot and its cells as they
  were. On load each cache is planned on its own, so a cache that does not plan keeps only its own
  pivots' saved cells. The caches are refreshed one after another in workbook order, each planned
  against the grid the ones before it wrote, so a later cache reads their new cells and meets
  their new areas. From the first plan that succeeds the writes are one edit, so a write that
  fails (only possible for a plan that does not match the workbook) rolls back every
  `refreshOnLoad` pivot.
- One budget covers every `refreshOnLoad` cache of a load, and what a failed plan spent stays
  spent. A few caches over whole-column sources (each charged its source's rows times columns
  before it is read) can exhaust it, and the caches after them then keep their saved cells with
  no sign of it.
Evidence: none beyond probes/t4128-pivot-spike S (a constant in the body, no page field).
Status: UNVERIFIED (probe pending: T-4664; T-2327 for constants in a refreshOnLoad pivot's area;
T-4712 for field indices in verbatim parts under a renumbering)
Implemented in: crates/rustsheet-engine/src/pivot/grid.rs (`owned_areas`, `meets_foreign`,
`remap`, `PivotBudget`; T-4151) and model.rs (`OpaqueEntry::holds_field_index`; T-4694)

## A tabular pivot's first row header is the field name only when every pivotField has compact="0"
Rule: in a tabular pivot, the header cell of the first row field reads the field's own name (`Region`)
only when every `pivotField` carries `compact="0"`. Otherwise it reads `Row Labels`. One field at the
default gives `Row Labels` in every role and position probed:
- roles: the outer row field, the inner row field, a column field, a page field, the data field, an
  unused source field and a calculated field;
- positions in the cache: first (R1), middle (R3, R4) and last (R6, and t4128 K's calculated field).

For the roles where `outline` was varied apart from `compact` (the outer row field, the column field
and the data field), only `compact` counts and `outline` makes no difference. With every field
`compact="0"`, an outer row field (t4609 L3), a column field (L7) and the data field (t4123 T3, T4)
left at the default `outline` still give the field's name. With `compact` at the default,
`outline="0"` on the outer row field (L4) or the column field (L8) still gives `Row Labels`.
`outline` was never varied apart from `compact` on the inner row field (Product is
`compact="0" outline="0"` in every L sheet), a page field, an unused field or a calculated field.
The second row field's header is its name (`Product`) whenever it keeps its own column.

When the field name wins, `rowHeaderCaption` is ignored: L5, with every field `compact="0"` and
`rowHeaderCaption="Regions"`, shows `Region` at A3. With a column field, `Row Labels` (or the name)
sits on the row below the data caption (L1!A4, L7!A4), and the column field's header beside it reads
`Column Labels` when the column field's `compact` is at the default (L1!C3 with both attributes at
the default, L8!C3 with `outline="0"`) and its name `Channel` when every field is `compact="0"`
(L7!C3). Whether a non-compact column field reads its name while another field is at the default is
not probed.

The outer row field's two attributes split the layout of R1 (both at the default) between them:
- `compact` and `outline` both on put the inner field in the outer field's column: R1's location is
  A3:B11 and firstDataCol 1. Either off gives the inner field its own column: L3 (`compact="0"`) and
  L4 (`outline="0"`) are A3:C11 with firstDataCol 2, like R2 to R6;
- `outline` alone decides the lines and the subtotals. With it on (R1, L3) each outer item has a
  line of its own carrying its subtotal (L3!A4 `East` with C4 22 and B4 blank, A8 `West` with C8 27)
  and there are no `<item> Total` lines; with it off (L4) the item shares its first child's line and
  `East Total` (A7) and `West Total` (A10) follow, as in R2 to R6.

Evidence (excel-graph, 2026-11-06):
- probes/t4583-pivot-row-labels A3 (T-4583). The cache fields are Region, Product, Sales, Units and
  Risky; every field carries `compact="0" outline="0"` except the one named:
  - R1 Region (outer axis, first): `Row Labels`;
  - R2 Product (inner axis): `Row Labels`;
  - R3 Units (unused, middle): `Row Labels`;
  - R4 Sales (data, middle): `Row Labels`;
  - R6 Risky (unused, last): `Row Labels`;
  - R5 (none left at the default): `Region`.
- probes/t4609-pivot-row-labels-2 (T-4609). The cache fields are Region, Product, Channel and Sales;
  every pivot is tabular with rows Region, Product and Sum of Sales, and every field carries
  `compact="0" outline="0"` except as named:
  - L1 column field Channel at the default: A4 `Row Labels`, C3 `Column Labels`;
  - L2 page field Channel at the default (location A4): A4 `Row Labels`; L6, the same page field
    non-compact: A4 `Region`;
  - L3 Region `compact="0"`, `outline` at the default: A3 `Region`, outline lines, A3:C11;
  - L4 Region `outline="0"`, `compact` at the default: A3 `Row Labels`, tabular lines, A3:C11;
  - L5 none at the default, `rowHeaderCaption="Regions"`: A3 `Region`;
  - L7 column field Channel `compact="0"`, `outline` at the default: A4 `Region`, C3 `Channel`;
  - L8 column field Channel `outline="0"`, `compact` at the default: A4 `Row Labels`,
    C3 `Column Labels`.
- probes/t4123-pivot-capture A4 (T-4123): T and T4, with Sales at the default, give `Row Labels`; T2
  and T3, with every field non-compact, give `Region`.
- probes/t4128-pivot-spike K!A4 (T-4128): a calculated field at the default gives `Row Labels`.
- probes/t4137-pivot-captions K5!A3 and K6!A4 (T-4137): unused fields at the default give `Row Labels`.

oracle-spike.md run 4, whose uncommitted hand-written XML set `compact="0"` on the Sales field, showed
`Region` too.
Correction: T-4123 first read the rule as "the data field's `compact`". T-4137 narrowed it to the
non-axis fields, and T-4583's R1 and R2 show that axis fields count too. T-4583 left R1's layout to
"`compact` and `outline` both on"; T-4609's L3 and L4 split it as above.
Status: VERIFIED (excel-graph) for the roles, positions and attributes probed. UNVERIFIED (probe
pending: T-4660): the column header of a non-compact column field while another field is at the
default, and `outline` varied apart from `compact` (either way round) on the inner row field, a page
field, an unused field or a calculated field.
Implemented in: crates/rustsheet-engine/src/pivot/layout/lines.rs (the outer field's own line from
its `outline`, and in `label_columns` the shared column from `compact` and `outline` both on;
T-4147, checked against every L sheet by T-4609)
Implemented in: crates/rustsheet-engine/src/pivot/layout/render.rs (the header text, every probe
above reproduced; T-4149)

## Rendered pivot cells: blanks, repeated labels, subtotals and the Values caption
Rule:
- A row and column intersection with no records is a blank cell, not 0 (compact C!D6, West/C).
- In tabular form an outer item's label shows on its first line only; the lines below it leave the
  outer cell blank (T!A6, A7, A10), unless the field repeats its labels ("Repeat item labels"
  below).
- A subtotal line reads `<item> Total` in the outer field's column, with the inner field's cell blank
  (T!A8 `East Total`, B8 blank; A11 `West Total`), and the grand total line reads `Grand Total`
  (T!A12).
- With the Values field on columns, `Values` sits above the first data field's column (T!C3), and the
  cells above the row fields (A3, B3) are blank.
- A compact pivot with a column field puts the data field's caption (`Sum of Sales`) at the
  location's corner and `Column Labels` beside it (C!A3, B3), with `Row Labels` below (C!A4).

- The headers follow the lines: each column line labels its levels from `r` on, one header row
  per column level below the location's top row (t4138 G5: `A` in C5, `Web` and `Store` in C6
  and D6, `A Total` in E5); each row line labels its levels from `r` on in its level's label
  column, compact form putting several levels in one column (G2). The sidecars hold values
  only, so a label cell's indentation is not evidenced; the engine writes the item's text alone.
- A single data field with Values on no axis puts its caption at the corner with fields on both
  axes (C!A3), over the data column with no column field (t4138 G1!C4, G8!B8), and as the label
  of the only row with no row field (G7!B6). With Values on an axis, its items are the data
  fields' captions, and its grand-total lines read `Total <data field>` (G6!B11:B12, t4135
  A_sum!N4:S4), with no `grandTotalCaption` set; whether that caption replaces the text there is
  unprobed (the engine keeps `Total <data field>`; UNVERIFIED, T-4650).
- Items render as in "Items show in the order the pivot field lists them" (t4134 I2, I4, I5).

Evidence: probes/t4123-pivot-capture, the `pivots` cells of its sidecar (excel-graph, 2026-11-06,
T-4123); t4138-pivot-geometry G1 to G9 (T-4138); t4135-pivot-aggregates (T-4135). The engine's
render reproduces every cell of the probe pivots listed in its tests (t4123 C and T to T4 but
T's show-values-as column D, t4138 G1 to G9, t4137 K1 to K6, t4583 R1 to R6, C1, C2, t4609 L1 to
L8, t4134 I1, I2, I4, I5, t4135 all twelve, t4589, t4614, t4616, and t4622 CA, CN and B1 to B5 (T-4168).
Number formats are PV2's (T-4169).
The engine's own readings, not probed: a number or date item's subtotal label (`number_to_text`
of the value, then ` Total`); a subtotal label with Values inside its field (`<item> <data
field>`); the grand-total caption with Values on its axis (always `Total <data field>`, even with
`grandTotalCaption`); the
header over a Values level in a label column of its own (left empty). `rowHeaderCaption` beside
the all-`compact="0"` rule is settled: the field name wins (t4609 L5!A3 `Region`; Caption
attributes below).
Status: VERIFIED (excel-graph) for the probes listed. UNVERIFIED (probe pending: T-4657): number
and date item subtotal labels; (T-4650): the Values-inside and Values-axis captions and header.
Implemented in: crates/rustsheet-engine/src/pivot/layout/render.rs (`render`; T-4149)

## Repeat item labels
Rule: a pivot field whose x14 extension sets `fillDownLabels="1"` ("Repeat Item Labels", saved as
`<extLst><ext uri="{2946ED86-A175-432a-8AC1-64E0C546D7DE}"><x14:pivotField fillDownLabels="1"/>`
in the `<pivotField>`) shows its item's label on every line of the item's children, in its own
label column, where it would otherwise show on the item's first line only. The lines (`rowItems`,
`colItems`) do not change. In detail (rows Region, Product, Year and Sum of Sales over the data of
t4129, every pivot at A3):
- **Per field, not inherited.** With the flag on Region only, `East` fills A4:A11 but Product's
  cells below its first line stay blank (T3R!B5); with it on Product only, `A` fills B5 and Region
  stays blank (T3P!A5:A11); with both, both fill (T3A). On the innermost field it changes nothing
  (TIN).
- **Subtotal lines of inner fields repeat it too**: `East` beside `A Total` (T3R!A6, A9, A11). The
  field's own subtotal line reads `East Total` alone (T3R!A12), the grand total line is unchanged
  (A19), and so are `insertBlankRow` lines (TBR!A8:C8, blank) and a collapsed item's one line
  (TSD!A4:B4). With no subtotal the next item's lines follow directly (TND!A4:A8).
- **Outline form fills too**: the outer item keeps its own line and its label repeats on every
  child line below it, Product's and Year's (OST!A5:A7, O3!A5:A12), with subtotals at the top or
  at the bottom (OB!A5:A7 then `East Total` in A8; the item's own line A4 still reads `East`).
- **Compact form fills nothing** (CF!A5:A7 show the Products): a cell belongs to the deepest level
  of the line placed in it. In MX, Region (compact, flagged) shares column A with Product
  (tabular, flagged): Product's `A` repeats on its Year lines (MX!A6) and Region fills nothing.
- **On columns** the label repeats across the children's columns (CC!B4:D4 `East`, F4:G4 `West`).
- **A blank item** repeats as `(blank)` (TBL!A8).
- **The table-wide `fillDownLabelsDefault="1"` fills no existing field** (TW, TWE, OW; t4129 X14).

Evidence: probes/t4698-pivot-repeat-labels (excel-graph, 2026-11-07, T-4698): T3R, T3P, T3A,
TND, TBR, TSD, TIN, TW, TWE, OST, OB, O3, OW, CF, MX, CC, TBL; probes/t4129-pivot-layout-flags X14,
X14F (T-4129). The sidecars' `row_items`/`col_items` equal the engine's unflagged lines.
The engine's own readings, not probed (UNVERIFIED, T-4764): a flagged level whose column it
shares with a deeper level the line leaves unlabelled (the engine fills nothing); an inner column
field's subtotal column under a flagged outer column field (the engine fills, as on rows); the
Values field on a flagged field's axis (the engine fills field levels only); an explicit
`fillDownLabels="0"` beside the table default, and custom subtotal lines under a flagged field
(filled as default subtotal lines are).
Status: VERIFIED (excel-graph) for the probes listed; UNVERIFIED (T-4764) for the readings above.
Implemented in: crates/rustsheet-xlsx/src/import/pivots/table.rs (`read_extension`: the typed
`PivotField::fill_down_labels`, the extension kept verbatim for export; T-4698)
Implemented in: crates/rustsheet-engine/src/pivot/layout/render.rs (`Cells::fill_down`; T-4698)

## Caption attributes, empty and error captions, page captions, custom subtotal labels
Rule:
- `rowHeaderCaption` replaces `Row Labels` (K1!A4 `Regions`). In a tabular pivot whose every field
  is `compact="0"` it is ignored and the field name shows (probes/t4609-pivot-row-labels-2 L5!A3
  `Region`; see "A tabular pivot's first row header..." above).
- `grandTotalCaption` replaces `Grand Total` (K1!A7 `All of it`).
- `dataCaption` replaces the `Values` caption over the data fields (K1!B3 `Measures`).
- `colHeaderCaption` replaces `Column Labels` beside a real column field in compact form
  (probes/t4583-pivot-row-labels C1!B3 `Products`, with rowHeaderCaption C1!A4 `Regions`). In K1,
  whose column axis holds only Values, it shows nowhere. In tabular form with every field non-compact
  it shows nowhere either: the column header reads the field's name (C2!B3 `Product`) and the row
  header reads its name too (C2!A4 `Region`; C2 sets no rowHeaderCaption).
- With `showMissing` and `missingCaption`, an empty intersection shows the caption (K2!D6 `n/a`).
- With `showError` and `errorCaption`, under Sum, every value cell whose aggregate is an error shows
  the caption as text: the cell, its row and column totals and the grand total (K3!C5, E5, C7 and E7 `ERR`; the
  data hold one `#DIV/0!` at East/B). The empty West/C stays blank (K3!D6).
- Page fields show `(All)` with no selection, the item with one (`A`), and `(Multiple Items)` when
  several items stay visible (K4!B2:B4). What a page field shows when its hidden items leave one
  item visible, and a selected number, boolean or blank item, are unprobed: the engine shows
  `(Multiple Items)` and the item as a row label renders it (UNVERIFIED, T-4657).
- Custom subtotal functions label each subtotal line `<item> <Function>`: `East Sum`, `East Count`
  (K5!A7:A8), with Count counting records (4). `countA` is labelled `Count` too
  (probes/t4622-pivot-count-subtotals CA!A6; see "Which function the countA and count custom
  subtotals compute").
- All eleven custom subtotals on one field (probes/t4168-pivot-layout-options F1, tabular, Sum of
  Val), in the schema's attribute order, read `East Sum`, `East Count` (countA), `East Average`,
  `East Max`, `East Min`, `East Product`, `East Count` (count), `East StdDev`, `East StdDevp`,
  `East Var`, `East Varp` (F1!A6:A16), each computing its own function over the item's records
  (East 10, 4, 5, 2: C6:C16 are 21, 4, 5.25, 10, 2, 400, 4, 3.4034296427770228,
  2.947456530637899, 11.583333333333334, 8.6875). Under Count of Val the custom `sum`, `avg` and
  `max` lines still sum, average and take the maximum (F3!C6:C8 21, 5.25, 10): the subtotal's
  function wins over the data field's.
- With a single data field and no column field (tabular, K5) there is no Values row. `Row Labels` is
  at the location's corner (K5!A3), the inner row field's name beside it (B3 `Product`), and the data
  field's caption above the data column (C3 `Sum of Sales`). With a column field the corner holds the
  data caption instead (K2!A3, K3!A3; previous entry).

Evidence: probes/t4137-pivot-captions (excel-graph, 2026-11-06, T-4137). Default data-field names
(`Sum of`, `Count of`...) are written by the file, not chosen by Excel, so a file cannot probe them;
creating a pivot in the UI is where Excel picks them (T-2327 keeps that question). For other
aggregates and `#DIV/0!` under `errorCaption`, see "errorCaption replaces every error value" below.
Status: VERIFIED (excel-graph)
Implemented in: crates/rustsheet-engine/src/pivot/layout/render.rs (`render`, `page_area`; T-4149)

## Page fields render above the location
Rule: page fields render in the rows above the pivot's `location`, ending one blank row before it. Each
shows `<field name>` beside its selection: `(All)`, the item, or `(Multiple Items)`. With
`pageWrap="n"` and `pageOverThenDown`, the fields fill a row n at a time, each field-item pair two
columns wide with a blank column between pairs. The saved `rowPageCount` and `colPageCount` give the
grid (here 2 by 2).
Evidence:
- probes/t4137-pivot-captions K4 (T-4137): three page fields in A2:B4 above location A6.
- probes/t4138-pivot-geometry G8 (T-4138): three page fields wrapped two per row, `Region | (All)` in
  B5:C5, `Product | (All)` in E5:F5 and `Channel | (All)` in B6:C6, row 7 blank, location B8,
  `rowPageCount=2`, `colPageCount=2`.

Capture window (T-4123, `snapshot.py` `pivot_area`): the `rowPageCount` rows above the location, minus
one blank row, and `3 * colPageCount - 1` columns wide. Both shapes above are consistent with it. In G8
the second pair starts at E, three columns after B, and the five-column window ends at F. A capture
cannot see past its own window, though, so a page cell outside it would go unseen (row 1 of K4,
rows 1–4 of G8).
Status: VERIFIED (excel-graph) for the rendering; the window is consistent with both probes, not
independently confirmed (probe pending: T-4584). Page grids filled down with `pageWrap` are
T-4650's (G8 cannot tell the orders apart).
Implemented in: crates/rustsheet-engine/src/pivot/layout/render.rs (`page_area`: `rowPageCount`
rows ending one row above the location, three columns per page column; T-4149)

## Pivot geometry: location and header offsets per layout
Rule: Excel writes the pivot's `location` as the body's bounding box (page fields excluded), with
`firstHeaderRow`, `firstDataRow` and `firstDataCol` measured from its top left. With the location's top
left at B4 (probes/t4138-pivot-geometry, T-4138):

| sheet | layout | location | fhr, fdr, fdc | rowItems, colItems |
|---|---|---|---|---|
| G1 | compact, 1 row field | B4:C7 | 1, 1, 1 | 3, 1 |
| G2 | compact, 2 row fields, 1 column field | B4:E13 | 1, 2, 1 | 8, 3 |
| G3 | outline, 2 row fields, 2 data fields | B4:E13 | 1, 2, 2 | 8, 2 |
| G4 | tabular, 3 row fields | B4:E18 | 1, 1, 3 | 14, 1 |
| G5 | compact, 1 row field, 2 column fields | B4:J9 | 1, 3, 1 | 3, 8 |
| G6 | compact, Values on rows | B4:C12 | 1, 1, 1 | 8, 1 |
| G7 | 1 column field only | B4:F6 | 1, 2, 1 | 1, 4 |
| G9 | 3 data fields, no axis field | B4:D6 | 1, 2, 0 | 1, 3 |

In compact form all row fields share one column (fdc 1). Outline and tabular give each row field its own
column (fdc = number of row fields), but each rests on one probe (G3, G4). With no row field, fdc is 1
when there is a column field (G7) and 0 when there is no axis field at all (G9, and G8 in the page-field entry above). fdr is 1
plus the number of `colFields`, and the Values field counts when it is on columns: G3 and G9 (Values
only) have fdr 2, as does G2 (one column field and one data field), and G5 has 3. With Values on rows,
each item is followed by one line per data field, and the grand total reads `Total <data field>`
(G6!B11:B12).

A Values field on rows needs `dataOnRows="1"` on the definition: without it Excel for the web refuses
the file as corrupt (`FileCorruptTryRepair`). Case builder v2 now writes it. This comes from scratch
excel-graph runs of T-4138, not committed: G6's package was refused without the attribute and opened
with it, all else equal.
The engine's layout (T-4147) reproduces the saved `location`, `firstHeaderRow`, `firstDataRow`,
`firstDataCol`, `rowPageCount` and `colPageCount` of every probe pivot in scope (listed under "Layout
lines" below). Page fields: with no `pageWrap` and the default order, every page field is in one column
(t4137 K4: 3 by 1); with `pageWrap` and `pageOverThenDown`, `pageWrap` fields per row (t4138 G8: 2 by
2). A row field shares its column with the next level only with `compact` and `outline` both on
(t4583 R1); with both off (R2), `compact="0"` alone (t4609 L3) or `outline="0"` alone (L4) the next
level has its own column.
Status: VERIFIED (excel-graph) for the table and the T-4147 reproduction above; the refusal is a builder
observation. UNVERIFIED (probe pending: T-4650): `firstHeaderRow` other than 1, and the page grid's
other two orders (G8's 2 by 2 cannot tell them apart).
Implemented in: crates/rustsheet-engine/src/pivot/layout/lines.rs (`geometry`, `location`; T-4147)
Implemented in: crates/rustsheet-xlsx/src/import/pivots/table.rs (`geometry`: the saved offsets typed
in `PivotTable::geometry`, T-4147)

## Layout lines: rowItems and colItems per form
Rule: Excel saves each axis's lines as `<i t r i><x v/></i>`. Each axis is its fields in order, with
the Values field (`x="-2"`) as one more level whose items are the data fields; a line's tuple is one
`x` per level down to its own, `r` counts the leading `x`s it shares with the line before (never its
own last one), and `<x>` lists the rest. An `x` is the item's position in the pivot field's `<items>`,
or the data field's index at the Values level (`i` repeats it).
- Every tuple of one item per level is a `t="data"` line.
- On rows, an item of a field in outline form (`outline` on, which compact form also has) has a line
  of its own above its children, which carries its subtotal values (with `subtotalTop`, the
  default). An item of a tabular field (`outline="0"`) shares its first child's line, and its
  subtotal lines follow its children. The form is per field and follows `outline` alone: an outer
  field left at the default inside a tabular pivot is outline form (t4583 R1), and so is one with
  only `compact="0"` (t4609 L3), while one with only `outline="0"` is tabular (L4).
- On columns every item shares its first child's line, and subtotal lines follow the children.
- A field has subtotal lines only when a field other than Values is inside it. They are one
  `t="default"` line, or one line per custom subtotal in the field's order (`t="sum"`, `t="count"`).
  Custom subtotal lines follow the item's children in compact form too (in plain outline form,
  `outline` on and `compact` off, it is the engine's reading: UNVERIFIED, T-4699), and the item's own line
  holds no values (t4168 F2: `East` B4 blank, `East Sum` B7 after `A` and `B`).
- A row field with `insertBlankRow` has a `<i t="blank" r="0"><x v="item"/></i>` line after each
  item, below its subtotal lines, and its cells are all blank: tabular (t4168 BR, A7:C7 and
  A12:C12, before the grand total) and compact (BR2, after the children, whose subtotal is on top).
- A collapsed item (`item@sd="0"`) has one line, its own tuple, showing the item's total and no
  children: in compact form `East` 21 on A4:B4 (t4168 SD), in tabular form `East` with the inner
  field's cell blank and no `East Total` line (SD2 A4:C4, `rowItems` `<x v="0"/>` alone).
- The grand total is one `t="grand"` line `<x/>` per data field when Values is on the axis, one
  otherwise, and none on an axis whose only level is Values.
- An axis with no level at all saves one empty line `<i/>`.

Evidence (excel-graph, 2026-11-06), the sidecars' `row_items` and `col_items`, all equal to the engine's
lines (T-4147):
- probes/t4123-pivot-capture C, T to T4 (compact with a column field; tabular with Values on columns);
- probes/t4138-pivot-geometry G1 to G9 (G2 compact, G3 outline, G4 tabular with three row fields, G5
  two column fields with `default` lines after their children, G6 Values on rows with two `grand`
  lines, G7 and G9 `<i/>`, G8 page fields only);
- probes/t4137-pivot-captions K1 to K6 (K5 tabular `sum` and `count` lines), t4583-pivot-row-labels
  R1 to R6, C1, C2, t4609-pivot-row-labels-2 L1 to L8, t4134-pivot-items I1, I2, I4, I5,
  t4135-pivot-aggregates (all twelve), t4589-pivot-total-order, t4614-pivot-total-order-2 and
  t4616-pivot-total-order-3 (every sheet);
- probes/t4168-pivot-layout-options F1 to F3, BR, BR2, SD, SD2 (T-4168: eleven custom subtotals
  tabular and compact, blank rows, collapsed items);
- the lines of probes/t4128-pivot-spike S (its #SPILL! location is the grid's, T-4151);
- the Excel-saved pivot of crates/rustsheet-xlsx/tests/example.xlsx: three compact row fields whose
  `<items>` list the cache items out of order, so `x` is the position in `<items>`, not the cache index.

Not compared in these engine unit tests, because their harness cannot rebuild the cache or tree
yet: t4128 R, t4134 I3, t4582 J1, J2 and t4585 N1 have formulas in their sources, which the case
file does not evaluate; t4582 J3 and J4 depend on date items against cached numbers (T-4612);
t4128 G is grouped and t4128 K has a calculated field, a Top-N filter and a descending
sort (T-4174, T-4165). The eval scores them all through the engine since T-4152: t4128 R, t4134 I3
and t4582 J1 to J4 pass there, as do t4585 N1 and the t4627 P_ pivots since T-4694, whose renamed
fields renumber (the grid writer's unit tests compare them against their sidecars); t4128 G passes
there since T-4171, and K does not load (T-4174).

The engine's own readings, not probed:
- `subtotalTop="0"` on an outline row field: the item's own line holds no values and a `default` line
  follows its children; with
  `defaultSubtotal="0"` and no custom subtotal there is no subtotal line, and `defaultSubtotal="1"` with
  custom subtotals is the default line alone; `rowGrandTotals="0"` drops the grand line of `rowItems`
  and `colGrandTotals="0"` that of `colItems` (an Excel-saved corpus file shows the latter,
  oss-engines.md §0.5, not committed);
- the Values field outer or in the middle of an axis: a field with Values inside gets one subtotal line
  per data field after its children; on rows with `outlineData`, each data field has a line of its own
  above the inner items;
- an item the pivot field does not list (Excel's files list every one) takes the positions after the
  listed items, in display order.

Status: VERIFIED (excel-graph) for the probes listed. UNVERIFIED (probe pending: T-4649): subtotal
placement and grand totals off; (T-4650): the Values field away from the innermost level and
`outlineData`; the unlisted-item positions are an engine convention for files Excel does not write.
(T-4699): `insertBlankRow` on the innermost row field or a column field (the engine adds no line) and
after a collapsed item (it adds one); custom subtotals after the children in plain outline form; a
collapsed item whose field has Values inside it (several data fields: one line, the first data field's
values); a collapsed item on columns, in a middle field, or in a field with
`subtotalTop="0"` or custom subtotals (one line, the data field's own aggregate). Repeat item labels
change no line (probes/t4698-pivot-repeat-labels, every sheet; "Repeat item labels" below).
Items with no data (`showAll`): docs/excel-semantics/web.md "GETPIVOTDATA on a recordless item of an ungrouped pivot field (T-4248)" (t4311 H,
a grouped row field) and crates/rustsheet-engine/src/pivot/result.rs; an axis of several fields, a
rendered column field and an item the page filter empties are UNVERIFIED (T-4657).
Implemented in: crates/rustsheet-engine/src/pivot/layout/lines.rs (`lay_out`, `AxisPlan::lines`; T-4147;
`blank_line`, `collapsed`, T-4168)

## Aggregates over text, blanks, booleans and errors
Rule (each aggregate in a value field that also holds other kinds of value):
- **Skipped values.** Text, blanks and booleans are skipped by Sum, Average, Max, Min, Product, StdDev,
  StdDevp, Var, Varp and CountNums. A boolean is not a number: `TRUE` is not 1.
- **Count** counts every non-blank value: numbers, text, booleans and errors. Its grand total can still
  be an error (see "A Count total over an error field is the error when the pivot mixes
  aggregates").
- **CountNums** counts numbers only. An error does not poison its item cells. Its totals can be the
  error (see "A Count total over an error field is the error when the pivot mixes aggregates").
- **Errors.** An error poisons every aggregate except Count and CountNums. The aggregate shows the error
  (`#N/A`) in the cell holding it and in every subtotal and grand total that covers that cell. Cells that
  do not cover it are unaffected.
- **One value.** Over a single number, StdDev and Var are `#DIV/0!`, while StdDevp and Varp are 0. This
  holds wherever the other values in the cell are text, blank or boolean. A data error wins over it:
  East/A of Err holds one number beside `#N/A` and shows `#N/A` (A_stdDev!L6, A_var!L6).
- **Empty intersections.** An intersection with no records is blank for every aggregate, Count and
  CountNums included.
- **Blank records.** A Sum cell whose records are all blank is blank too, not 0 (an item whose one record has
  no value).

Evidence: probes/t4135-pivot-aggregates (excel-graph, 2026-11-06, T-4135). Each sheet A_<aggregate> has
rows Region, columns Kind and six value fields. The Data!A2:H5 records are East/B, East/A, East/A and
West/A, and East/A row 4 holds the odd value: text `abc`, a blank, `TRUE` or `#N/A`.
- **Skipped values:** A_sum!I6:K6 are 10 for Txt, Blk and Bool, against H6 = 14 for Num. A_average!I6:K6
  are 10 and A_product!I6:K6 are 10.
- **Count:** A_count!I6, K6 and L6 are 2 for Txt, Bool and Err; J6 is 1 for Blk.
- **CountNums:** A_countNums!I6:L6 are 1.
- **Errors:** column L (Err) of A_sum, A_average, A_max, A_min, A_product, A_stdDev, A_stdDevp, A_var and
  A_varp is `#N/A` at row 6, at the East total R6 and at the grand totals L8 and R8. West/A (L7) shows
  its own one-value result: 7 for Sum to Product, `#DIV/0!` for StdDev and Var, and 0 for StdDevp and
  Varp.
- **One value:** A_stdDev and A_var are `#DIV/0!` at East/B (B6:G6) and West/A (H7:M7), and at I6
  (East/A Txt: 10 and `abc`). A_stdDevp and A_varp are 0 there.
- **Empty intersections:** West/B (B7:G7) is blank on every sheet.
- **Blank records:** probes/t4164-pivot-value-sort (excel-graph, 2026-11-07, T-4164) SEd B7 and SEa B4: Sum of Amt
  of item D, whose one record has a blank Amt, is blank (found by T-4827).
Status: VERIFIED (excel-graph)
UNVERIFIED (probe pending: T-4633): a cell whose records hold no
number at all (only text, blanks or booleans) is unprobed beyond a Sum over blanks alone; the engine shows that one
blank and gives the rest the worksheet functions' answers (Sum over text or booleans, Max, Min and Product 0, Average,
StdDev and Var `#DIV/0!`, StdDevp and Varp `#DIV/0!` with no number), and a Sum total that covers only blank
records blank too. A show-values-as mode reads such a blank Sum cell as empty, not as 0 (also UNVERIFIED,
T-4633). A date in a value field counts as its serial number, also unprobed. When
a cell or total covers two different errors, the engine shows the first in accumulation order, also
unprobed (T-4633).
Implemented in: crates/rustsheet-engine/src/pivot/result.rs (`compute`, `PivotResult::value_as`; a blank Sum in
`PivotResult::plain_as`, T-4827), crates/rustsheet-engine/src/pivot/accumulate.rs (`Accumulator::finish`)

## Summation order of Sum, Average and Product totals
Rule:
- **Item cells.** A cell accumulates its records in record order.
- **Sum, when every data field is a Sum.** A total adds leaf cells, in display (first-seen) item
  order, and never the subtotals between them:
  - a cell of a subtotal or grand-total row adds the cells of the leaf rows below it, in its column,
    down the rows;
  - a total in a leaf row adds that row's leaf-column cells below its column, across the columns.

  So a column total adds the column's cells, a row total the row's cells, and the grand-grand total
  the row totals (the Grand Total column) down the rows. With one axis field the Grand Total adds the
  item cells. These are sums of the child cells, not of the records.
- **Sum beside any other aggregate.** Count, CountNums, Max, Min, Product, StdDev, Var, Varp, StdDevp
  and Average each switch Sum to the records. A row or column total then accumulates the records
  item by item, in the display item order of the field being totalled and in record order within
  each item. Child cells are not used, so the same data can give a different Sum total once any
  non-Sum field is added.
- **Average (alone or beside other fields), Var, StdDev, Varp, StdDevp and Product.** A row or column
  total, and a total over a pivot's only axis field, uses that same item-by-item accumulation of the
  records. Average divides its sum by the count.
- **A cell that is a total on both axes**, outside the Sum-only case. This covers the grand-grand
  total, a subtotal by a subtotal and a subtotal by a grand total. The cell does not accumulate its
  records. It adds the partial results of the leaf rows below it, in its column, down the rows in
  display order. Each leaf-row cell is accumulated item by item. For Sum and Average that means the
  leaf rows' record sums. For Var, StdDev, Varp and StdDevp the leaf rows' Σx and Σx² are added
  separately, and the formula is applied to the totals. For Product the leaf rows' products are
  multiplied. Sum, Average, Var and StdDev are evidenced. For Varp, StdDevp and Product the engine
  applies this rule by analogy: their cells fit it, but other readings fit them too (UNVERIFIED,
  T-4678).
- **Formulas.** Var is the textbook (Σx² − (Σx)²/n) / (n − 1), and StdDev its square root. Varp is
  (n·Σx² − (Σx)²) / n², and StdDevp its square root. Each total feeds these formulas in the order
  above.

What is open (T-4678):
- Var, StdDev, Varp, StdDevp and Product at a subtotal crossing with two fields per axis, where the
  engine applies the leaf-row rule that T2 and T3 show for Sum and Average;
- the order of the Varp and StdDevp grand-grand total with one field per axis, which the engine takes
  from Var by analogy;
- the order of the Product grand-grand total, where one cell fits several readings;
- Max and Min beside an error field at such cells;
- a pivot with column fields only;
- Values on rows.

Evidence (excel-graph, 2026-11-06):
- probes/t4634-pivot-total-order-4 (T-4634). Python doubles were used to choose the data so that the
  candidate readings below give different values. Every claim is among the candidates tried.
  - Which mixes switch Sum to the records. X1 to X12 have rows R and columns K: Sum of V beside a
    second field of V. Each sheet uses its own 27-record source, E1 to E12. In each source the cells
    added in display order differ from the records accumulated item by item in at least five of the
    six row and column totals. The candidates for each total were:
    - the cells added in display, reverse or ascending item order;
    - the records accumulated item by item in each of those orders;
    - record order, and reverse record order.

    | Sheet | Beside Sum | Totals fitting records, item by item, display order | Totals fitting cells, display order | Example (Excel; cells would give) |
    |---|---|---|---|---|
    | X1 | Count | 6 of 6 | 1 of 6 | H6 103461.13000000002 (103461.13) |
    | X2 | CountNums | 6 of 6 | 0 of 6 | H6 107904.41000000002 (107904.40999999999) |
    | X3 | Max | 6 of 6 | 0 of 6 | H6 717.1499999999999 (717.15) |
    | X4 | Min | 6 of 6 | 0 of 6 | H6 71912.50000000001 (71912.5) |
    | X5 | Product | 6 of 6 | 1 of 6 | H6 92141.79000000001 (92141.79) |
    | X6 | StdDev | 6 of 6 | 1 of 6 | H6 19260.650000000005 (19260.649999999998) |
    | X7 | Var | 6 of 6 | 0 of 6 | H6 9706.31 (9706.310000000001) |
    | X8 | Varp | 6 of 6 | 1 of 6 | H6 114117.63999999998 (114117.64) |
    | X9 | StdDevp | 6 of 6 | 1 of 6 | H6 105410.12000000001 (105410.12) |
    | X10 to X12 | Average | 6 of 6 each | 0, 1 and 0 of 6 | X10 H6 70896.70999999999 (70896.71) |

    X10 does not separate display item order from reverse record order: both fit its six totals.

    The six totals of each sheet are H6:H8 and B9, D9 and F9. S1, Sum alone over E1, fits the cells in
    display order in all six (E5:E7, B8:D8), with E5 103461.13 against X1 H6 103461.13000000002 over
    the same records.
  - Mixed grand-grand total, Sum and Average. These are X1 to X12 H9, X10 to X12 I9, and A1 E8
    (Average alone over E11). The candidates were:
    - the row totals or the column totals added in display, reverse or ascending order, with each
      total taken as either cells or records;
    - the cells added row by row or column by column;
    - the records row-major or column-major, item by item over R or over K, in record order, or in
      reverse record order;
    - each of those in every combination of display, reverse and ascending orders, 56 readings in
      all.

    Each sheet fits between 2 and 14 of them, for example X2 H9 230121.09000000005 fits 2 and X12 H9
    102980.28999999998 fits 2. Only one reading fits all sixteen cells: the row totals' record sums,
    each accumulated item by item, added down the rows in display order. For X2 H9, record order gives
    230121.08999999997 and the column totals' record sums 230121.09.
  - Var-family grand-grand total. The sources are G1 and G2 (36 records, R first seen Q X B F N C, K
    first seen M Z A, values 800 to 1300 so that rounding differences grow). The candidates (Σx, Σx²)
    were accumulated in many ways:
    - records row-major or column-major in every combination of display, reverse and ascending
      orders, by R item, by K item, in record order and in reverse record order;
    - the cells' sums added row by row or column by column;
    - each row's sums added down the rows, or each column's sums added across the columns, in every
      order.

    These were combined with four Var formulas (textbook, n·Σx², two-pass, Welford) and five Varp
    formulas (n·Σx², textbook /n, Σx²/n − mean², two-pass, Welford):

    | Cell | Aggregate | Excel | row partials, display | row-major records | column partials | record order |
    |---|---|---|---|---|---|---|
    | V1 E11 | Var | 22664.877200872772 | 22664.877200872772 | 22664.87720087405 | 22664.877200872772 | 22664.877200872772 |
    | V2 E11 | Varp | 22035.297278626465 | 22035.297278626465 | 22035.297278627568 | 22035.297278626465 | 22035.297278626465 |
    | V3 E11 | StdDevp | 148.4429091557642 | 148.4429091557642 | 148.44290915576792 | 148.4429091557642 | 148.4429091557642 |
    | V6 E11 | Var | 19695.45869357139 | 19695.45869357139 | 19695.45869357224 | 19695.45869357139 | 19695.458693570963 |
    | V7 E11 | Varp | 19148.362618749892 | 19148.362618749892 | 19148.362618750813 | 19148.362618749892 | 19148.362618749525 |
    | X7 I9 | Var | 6029222.866869232 | 6029222.866869232 | 6029222.866869232 | 6029222.866869231 | 6029222.866869231 |
    | X8 I9 | Varp | 112914562.50268146 | 112914562.50268146 | 112914562.50268148 | 112914562.50268146 | 112914562.50268148 |

    X6 I9 (StdDev, 21611.79840781048) and X9 I9 (StdDevp, 12429.806341554762) fit all of these.
    - **Var and StdDev** (V1, V6 E11; X6, X7 I9). Of the candidates, only textbook Var over the rows'
      sums fits all four cells. Each row's sums are accumulated item by item and then added down the
      rows in display order. X7 I9 refutes column partials, and V1 and V6 E11 refute row-major order.
    - **Varp and StdDevp** (V2, V3, V7 E11; X8, X9 I9). These cells fit the rows' sums with the n·Σx²
      form. On their own they also fit at least two other readings: column partials in display order,
      and cell sums added row by row (R ascending, K in display order). The engine takes the Varp and
      StdDevp order from Var by analogy (UNVERIFIED, T-4678).
  - Var-family row and column totals. V1, V2, V3, V6 and V7 E5:E10 and B11:D11, and X6 to X9 I6:I8
    and C9, E9, G9, all fit the item-by-item display-order accumulation. Varp and StdDevp take Var's
    order. Over one axis field, V4 B10 (Varp, rows R, 22035.2972786272) and V5 B7 (rows K,
    22035.29727862628) each fit only the item-by-item display order with the n·Σx² form.
  - Product (X5, beside Sum). The row and column totals I6:I8, C9, E9 and G9 fit the item-by-item
    product. The cells' product differs in I6, I8 and C9. The grand-grand I9 is 7.444417258525587e+39.
    That one cell fits about 8 of the 54 product readings tried. Among them are the product of the row
    totals' products in display order, the shared leaf-row rule, and row-major records with R
    reversed. Several cell-product orders also fit. It does not exclude those, so it is consistent
    with the shared rule but does not settle it (UNVERIFIED, T-4678). Among the readings it does
    refute are row-major order (7.444417258525588e+39), the column totals (7.444417258525589e+39) and
    record order (7.444417258525589e+39).
  - Sum alone over one axis field. O1 and O2 are rows R and rows K over F1, and O3 and O4 the same
    over F2. The candidates were the item cells added in display, reverse or ascending order, the
    records item by item in each order, record order, and reverse record order. The item cells in
    display order are the only one that fits all four: O1 B7 154959.82000000004, O2 B7 154959.82,
    O3 B7 22667.72 and O4 B7 22667.72. Item-by-item display order gives 154959.82000000007,
    154959.82000000004, 22667.719999999994 and 22667.719999999994.
  - Average alone over one axis field: A2 B7 5739.252592592595 (rows R) and A3 B7 839.545185185185
    (rows K). Together they fit only the item-by-item display-order sum divided by 27. A2 alone also
    fits reverse item order, and A3 alone ascending item order. The item cells would give
    5739.252592592594 and 839.5451851851852. Among the cells probed, Average never follows the cells:
    A1 E5:E7 and B8:D8 (2-D) fit item by item too.
  - Two fields per axis. H1 has 32 records: rows P (H, E), then Q (t, s); columns U (G, D), then W
    (d, c); compact form, so subtotals sit on the outer item's row. Seventeen readings were tried, and
    the data was chosen so that every pair differs somewhere:
    - a total adds its children (the subtotals) or its leaf cells, rows first or columns first;
    - the leaf cells flat, row-major or column-major;
    - the records row-major, column-major, by row leaf, by column leaf, or in record order;
    - the engine's earlier mixed reading.

    In the first table, each total adds leaf cells and each subtotal adds its children. In the
    second, the leaf-row record sums are added, and the earlier mixed engine is row-major records
    with the grand-grand cell the sum of the outer subtotals.

    | T1, Sum alone | Excel | adds leaf cells | adds subtotals |
    |---|---|---|---|
    | H12, grand by grand | 153907.15 | 153907.15 | 153907.14999999997 |
    | G12, grand by D | 29529.63 | 29529.63 | 29529.630000000005 |
    | H6, H by grand | 19409.61 | 19409.61 | 19409.609999999997 |

    | T2 (Sum beside Average) and T3 (Average alone) | Excel | leaf-row record sums | earlier mixed engine |
    |---|---|---|---|
    | T2 N13, grand by grand | 153907.15000000002 | 153907.15000000002 | 153907.14999999997 |
    | T2 F13, grand by G | 124377.52 | 124377.52 | 124377.51999999999 |
    | T2 N10, E by grand | 134497.54 | 134497.54 | 134497.53999999998 |
    | T3 H12, grand by grand | 4809.598437500001 | 4809.598437500001 | 4809.598437499999 |
    | T3 D12, grand by G | 7773.595 | 7773.595 | 7773.594999999999 |

    T1 fits only "rows first, leaf cells" in every cell (B6:H12). Adding subtotals fails seven cells:
    B12, F12, G12, H12 and H6:H8. In T2 (Sum and Average, B7:O13) and T3 (B6:H12), only the leaf-row
    rule fits every cell. Row-major records fail four cells in each. Adding the row subtotals fails two,
    adding the column subtotals three, and adding the leaf columns five.
- probes/t4616-pivot-total-order-3 (T-4616). D1 has 27 records, three per cell: R items first seen
  Q, X, B, and K items first seen M, Z, A. Python doubles were used to choose values for which the
  candidate totals differ. The candidates are:
  - the row's cells added in display, reverse or ascending item order;
  - records accumulated item by item in display, reverse or ascending item order;
  - record order, and reverse record order.

  | Total | cells, display | cells, reverse | cells, ascending | items, display | items, reverse | items, ascending | records | records, reverse |
  |---|---|---|---|---|---|---|---|---|
  | row Q | 19313.14 | 19313.139999999996 | 19313.14 | 19313.14 | 19313.140000000007 | 19313.14 | 19313.140000000003 | 19313.140000000003 |
  | row X | 5223.799999999999 | 5223.8 | 5223.8 | 5223.799999999999 | 5223.8 | 5223.8 | 5223.8 | 5223.8 |
  | row B | 871.3100000000001 | 871.3100000000001 | 871.3100000000001 | 871.3099999999998 | 871.3100000000001 | 871.31 | 871.31 | 871.3100000000002 |
  | column M | 4406.299999999999 | 4406.299999999999 | 4406.3 | 4406.299999999999 | 4406.299999999998 | 4406.3 | 4406.299999999999 | 4406.299999999999 |
  | column Z | 12569.289999999999 | 12569.289999999999 | 12569.289999999999 | 12569.289999999997 | 12569.29 | 12569.29 | 12569.289999999997 | 12569.29 |
  | column A | 8432.659999999998 | 8432.66 | 8432.66 | 8432.659999999998 | 8432.66 | 8432.66 | 8432.66 | 8432.66 |

  - S1 (Sum alone) shows the cells-in-display-order column: E5:E7 are 19313.14, 5223.799999999999 and
    871.3100000000001, and B8:D8 are 4406.299999999999, 12569.289999999999 and 8432.659999999998. No
    other candidate fits all six.
  - S2 (Sum beside Average, Values inside K) shows the items-in-display-order column: H6:H8 are
    19313.14, 5223.799999999999 and 871.3099999999998, and B9, D9 and F9 are 4406.299999999999,
    12569.289999999997 and 8432.659999999998. No other candidate fits all six.
  - S2's Average totals (I6:I8, C9, E9, G9) and S3's (Average alone, E5:E7, B8:D8) equal the
    items-in-display-order sums divided by 9; for example, S3 E7 is 96.8122222222222 = 871.3099999999998 / 9.
    No other candidate fits.
  - S4 (S1 transposed, K on rows) gives the same cells-in-display-order totals (E5:E7, B8:D8).
  - S5 (Sum alone over D2, 18 records) and S6 (Sum alone over D4, 36 records, six R items): cells in
    display order is the only candidate that fits every row total (S5 E5:E7, S6 E5:E10) and every
    column total in S6 (B11:D11). In S5's column totals, two other candidates also fit.
  - Grand-grand total, Sum alone. S1 E8 is 25408.25, the sum of E5:E7 in display order. On its own,
    S1 E8 fits many readings, among them the row totals added in any order and several cell and record
    orders. It refutes the sum of the column totals (25408.249999999993), the cells added row by row
    (25408.249999999996), record order (25408.250000000015) and row-major record accumulation
    (25408.25000000001).
  - S4 E8 is 25408.249999999993, the sum of S4's own row totals E5:E7 in display order. Of the
    readings tried that fit S1 E8, only that one gives 25408.249999999993 in S4. The others give
    25408.25, 25408.25000000001, 25408.250000000007 or 25408.249999999996. S5 E8 (14276.83) and
    S6 E11 (32198.6) also equal the sum of their row totals in display order. Of the candidates, only
    "row totals in display order" fits all four pivots.
  - Grand-grand total, mixed. S2 H9 is 25408.25. Among the readings that fit it are the sum of
    H6:H8 (in any order), reverse-row-item record accumulation, and several cell and record orders.
    It is not record order (25408.250000000015) or row-major accumulation in display
    order (25408.25000000001). Average's I9 is 941.0462962962963 = 25408.25 / 27 (S3 E8 too).
  - Cells: X/Z of D1 (three records) is 123.67999999999999 in S1 C6, S2 D7 and S4 B6, the record-order
    sum. Reverse order would give 123.68.
  - Var (D4, V1): V1's row totals E5:E10 and column totals B11:D11 fit only the textbook formula with
    item-by-item display-order sums. The two-pass, n·Σx² variant and Welford's update, each in display,
    reverse or ascending item order or in record or reverse record order, are refuted. Every V1 cell
    (B5:D10, two records each) fits the textbook formula.
  - Var's grand-grand total (V1 E11, 4337674.527019682) fits textbook Var over row-major (R, then K)
    display-order accumulation. About 38 other readings also fit: textbook, n·Σx², two-pass and
    Welford's update over many orders, and textbook over per-row or per-column partial sums.
    Column-major display order gives
    4337674.5270196805. T-4634 settles it: textbook Var over the rows' partial sums.
  - Var over one axis field: V3 (rows K) B8 is 4337674.52701968 and V4 (rows R) B11 is
    4337674.527019681. Each equals textbook Var accumulated item by item in its own field's display
    order. For V3 that is the only fitting candidate. The item cells V3 B5:B7 and V4 B5:B10 are
    textbook Var in record order. So the same 36 records give three different Var grand totals in V1,
    V3 and V4.
  - StdDev (V2, and V3/V4 column C): each cell and total is the square root of the textbook Var. In V3
    (C8, 2082.7084594392177), display-order textbook is the only fitting candidate. V2's grand-grand
    total E11 is 2082.7084594392186 = sqrt(V1 E11).
  - Product: P1 (D5) Grand Total B7 is 1801036823.2874622 and P2 (D6) B7 is 130884059.83325395. Each
    is the item-by-item display-order product. These were the candidates:
    - ascending value order, 1801036823.2874618 and 130884059.83325396;
    - record order, 1801036823.2874625 and 130884059.83325398;
    - the product of the item cells, 1801036823.2874613 and 130884059.83325401;
    - reverse and ascending item order, and descending value order.

    Each of those differs from the actual value in P1 or P2. This settles T-4614's P2 B6, where
    ascending value order also fitted.
- probes/t4614-pivot-total-order-2 (T-4614). D1 holds the records Z 6.5, A 0.1, Z 2.3, A 8.9, Z 2.4,
  A 9.4, with Z first seen.
  - S2 (rows K, Sum beside Average): Grand Total Sum 29.6 (B7), and Average 4.933333333333334 = 29.6 / 6
    (C7). The candidates were record order 29.599999999999994, reverse 29.600000000000005, ascending
    item order 29.599999999999998, and display item order 29.6.
  - S3 (Sum alone, K on columns): Grand Total 29.6 (D5). The sum of the item cells,
    11.200000000000001 + 18.4, is also 29.6, so S3 does not separate cells from records.
  - P2 (D2, Product alone): Grand Total 15743.706264 (B6), display item order. Record order gives
    15743.706264000002.
- probes/t4589-pivot-total-order S1 (T-4589): seven aggregates over records A 0.2, B 0.7, A 3.3, B 1.1.
  - The Sum Grand Total is 5.300000000000001 (B7), display item order. The item cells would give 5.3,
    and record order 5.299999999999999.
  - The Var Grand Total is 1.8691666666666658 (F7), textbook Var in display item order. Record order
    gives 1.8691666666666673, and two-pass 1.8691666666666664. StdDev (E7) is 1.3671747023210552.
- probes/t4135-pivot-aggregates (T-4135). On the A_sum sheet every data field is a Sum. East's records
  in Dec are 0.3 (B), 0.2 (A) and 0.1 (A), with Kind on columns shown B, then A.
  - A_sum!S6 is 0.6000000000000001, the sum of East's cells (0.3 + 0.30000000000000004). Display item
    accumulation would give 0.6. This fits the Sum-alone rule.
  - A_average!S6 is 0.19999999999999998 = 0.6 / 3, the item-by-item accumulation, which fits the
    Average rule.

- Varp's formula: probes/t4135-pivot-aggregates A_stdDevp!N6 and M8 (T-4142). Of the candidates
  tried, only (n·Σx² − (Σx)²)/n² fits both. On those t4135 cells it gives the same bits as dividing
  by n twice, but t4634 rules that out: dividing by n twice differs from /(n·n) at V2 E7, E8 and
  C11, at V7 E8, E9 and E10, and at X8 G9, and Excel shows /(n·n). The others were
  textbook /n, Σx²/n − mean², two-pass, Welford and (Σx² − Σx·mean)/n. t4589 S1 G5:H7 fits it too.

Correction: T-4135 first recorded Sum as the sum of the children. T-4614 then recorded item-by-item
accumulation for totals over one axis field. T-4616 showed that both happen, depending on the pivot's
data fields. T-4634 shows that any non-Sum field switches Sum to the records. It also shows that a
Sum-only total adds leaf cells, not subtotals. Finally, a total on both axes adds the leaf rows' partial
results. The engine's earlier row-major reading for the Var grand-grand total, and its sum of the
outer subtotals for a mixed grand-grand total, were wrong (t4634 V1 E11, T2 N13).

Status: VERIFIED (excel-graph) for these totals:
- every Sum total with one field per axis, Sum alone and beside each of the other ten aggregates;
- Sum alone over one axis field, and Sum alone with two fields per axis;
- Average, Var, StdDev, Varp, StdDevp and Product row and column totals, and totals over one axis
  field;
- grand-grand totals with one field per axis, for Sum, Average, Var and StdDev;
- every cell of Sum beside Average, and of Average alone, with two fields per axis.

UNVERIFIED (probe pending: T-4678). These are the same items as "What is open" above:
- Var, StdDev, Varp, StdDevp and Product at crossings with two fields per axis;
- the Varp, StdDevp and Product grand-grand order with one field per axis;
- Max and Min beside an error field at such cells;
- a pivot with column fields only;
- Values on rows.

The engine applies the leaf-row rule to the crossings, and to the Varp, StdDevp and Product
grand-grand totals.
Implemented in: crates/rustsheet-engine/src/pivot/result.rs (`compute`, `merge_crossings`,
`child_cells`, `PivotResult::value_as`), crates/rustsheet-engine/src/pivot/accumulate.rs
(`Accumulator::finish`, `Accumulator::add_folds`).

## errorCaption replaces every error value
Rule: with `showError` and `errorCaption`, every error value a pivot cell would show becomes the
caption, as text. That includes `#DIV/0!` from StdDev and Var over one value, not only errors from the
data.
Evidence: probes/t4135-pivot-aggregates EC (excel-graph, 2026-11-06, T-4135):
- East (B5:L5) shows `ERR` for every aggregate except Count (3) and CountNums (2).
- West (B6:L6) shows `ERR` for StdDev and Var, which have one value, and 0 for StdDevp and Varp.
- The Grand Total row (B7:L7) is `ERR` everywhere; see the next entry for Count and CountNums.
Status: VERIFIED (excel-graph)
Implemented in: crates/rustsheet-engine/src/pivot/layout/render.rs (`Cells::value`; T-4149)

## Value cells show their data field's number format
Rule: every value cell of a pivot, on item, subtotal and grand-total lines and in the grand-total
column, displays its value in its data field's `numFmtId`: a built-in id, or a custom format of the
workbook's `numFmts` (164 in the probe, as the builder wrote it). Excel stores the unrounded value: only the displayed text
is rounded. A data field with `numFmtId="0"` and one with no `numFmtId` both show General. Labels,
captions and the header rows keep their own text. Built-in id 14 shows as `m/d/yyyy` in en-US, not
as the ECMA table's `mm-dd-yy`.
Evidence: probes/t4169-pivot-number-formats (excel-graph, 2026-11-06, T-4169), from each pivot's
displayed text (`Range.text`, the sidecar pivots' `text`) beside its values. The source is unformatted.
- P1, compact, rows Region and Product, six data fields on columns:
  - Cur (numFmtId 7): `$1,234.50 ` (B6), `($250.25)` (B7), `$0.13 ` for 0.125 (B9), and the Grand
    Total `$4,027.13 ` (B11), whose stored value is still 4027.125.
  - Pct (10): `49212.50%` (C5, an East subtotal), `12.50%` (C9).
  - Latest (14, max of When): `2/15/2024` (D5), `4/18/2024` (D11).
  - Kg (164, the workbook's `0.0 "kg"`): `984.3 kg` (E5), `-250.3 kg` (E7), `4027.1 kg` (E11).
  - N (0): `2` (F5). Plain (none): `3042.875` (G8), `4027.125` (G11).
- P2, rows Region, columns Product, Total Val (4): `1,234.50` (B5), `-250.25` (C5), and the
  grand-total column `984.25` (D5) and `4,027.13` (D7).
Status: VERIFIED (excel-graph)
Implemented in: crates/rustsheet-engine/src/pivot/layout/render.rs (`RenderedCell::number_format`),
crates/rustsheet-engine/src/pivot/grid.rs (`Model::set_pivot_number_format`), and
crates/rustsheet-engine/src/number_format.rs (id 14, and new custom ids from 164 by convention; T-4169).
UNVERIFIED (T-4169, owner T-4717): the engine only sets a value cell's format when its data field has a
`numFmtId`, and leaves other cells' formats alone. Not probed: whether Excel resets a cell's format
when it stops being a value cell after a refresh, or when the data field has no `numFmtId` (the
"preserve cell formatting" option), and whether a showDataAs field or a page, row or column field's
`numFmtId` changes the text.
UNVERIFIED (eval-harness, T-4169, owner T-4730): the Kg cells (E5:E11) pass only because the case has
one custom format, which the builder (XlsxWriter) and the spec adapter (`set_number_formats`, by sheet
then address order) each number 164. With two or more custom formats the two numberings can differ, so
a definition's `numFmtId` could name another code in the engine than in Excel. The engine's unit test
hand-sets Q!B1 for the same reason.

## A Count total over an error field is the error when the pivot mixes aggregates
Rule: Count and CountNums never show an error in an item's cell. When the pivot's data fields mix
aggregates (Count or CountNums beside any aggregate other than Count and CountNums: Sum, Average,
Max, Min, Product, Var, Varp or StdDev among those probed), though, every total of a Count or CountNums
that covers a record holding an error is that error: `#N/A`, or the errorCaption when one is set.
This covers subtotals of an outer row field over an inner one, row and column grand totals, and the
grand total. Totals that cover no such record stay counts. With one data field, with only Count fields,
or with Count beside CountNums, every total is a count. A column field makes no difference.
Evidence (excel-graph, 2026-11-06):
- probes/t4634-pivot-total-order-4 (T-4634), over ER (t4616's D3), rows Region, then Kind. C1 to C5
  are Count of Err beside only Max, Min, Product, Var or Varp of Num, C6 is CountNums of Err beside
  Max, and C7 is Count of Err beside StdDev. In all seven, East's subtotal (B5) and the Grand Total
  (B11) are `#N/A`. The items stay counts: East A 2 (B6), East B 1 (B7; 0 under CountNums in C6), and
  West 2 (B8).
- probes/t4616-pivot-total-order-3 (T-4616), over D3: East A (Err 10), East B (Err `#N/A`), East A
  (3), West A (7), West B (5); rows Region, then Kind; no showError:
  - I1, Count of Err beside Sum of Num: East's subtotal is `#N/A` (B5), while its items A and B are 2
    and 1 (B6, B7). West's subtotal is 2 (B8), and the Grand Total is `#N/A` (B11).
  - I2, CountNums of Err beside Sum of Num: East's subtotal is `#N/A` (B5), with B at 0 (B7). West
    is 2 (B8), and the Grand Total is `#N/A` (B11).
  - I3, Count of Err alone: East 3 (B4), Grand Total 5 (B10). I4, CountNums of Err alone: East 2 (B4),
    Grand Total 4 (B10).
  - M1, Count of Err beside CountNums of Err, rows Region: East 3 and 2 (B5, C5), Grand Totals 5 and
    4 (B7, C7). M2, Count of Err beside CountNums of Num: Grand Totals 5 and 5 (B7, C7). M3, M1's
    fields with rows Region and Kind: East's subtotal 3 and 2 (B5, C5), Grand Totals 5 and 4 (B11, C11).
- probes/t4614-pivot-total-order-2 (T-4614), over D3 (East A Err 10, East B `#N/A`, West A 7), without
  showError:
  - C1, Count of Err and Count of Num, no column field: Grand Totals 3 and 3 (B7, C7).
  - C2, Count of Err beside Sum of Num, column field Kind:
    - the item cells are counts (B6 1, D6 1, B7 1);
    - East's total, B's grand total and the grand total of Count of Err are `#N/A` (F6, D8, F8);
    - West's total, which covers no error, is 1 (F7).
  - C3, CountNums of Err beside Sum of Num: Grand Total `#N/A` (B7).
  - C4, Count of Err beside Average of Num: Grand Total `#N/A` (B7).
- probes/t4589-pivot-total-order (T-4589):
  - E1, Count alone: Grand Total 3 (B6).
  - E2 and E3, mixed, with errorCaption: `ERR` (B7).
  - E4, mixed, without the caption: `#N/A` (B7).
- probes/t4135-pivot-aggregates (T-4135):
  - A_count, six Count fields: 4 (R8).
  - EC, eleven mixed aggregates: `ERR`.
Status: VERIFIED (excel-graph)
Implemented in: crates/rustsheet-engine/src/pivot/result.rs (`compute`, `PivotResult::value_as`), crates/rustsheet-engine/src/pivot/accumulate.rs (`Accumulator::finish`). The engine treats any aggregate other than Count and CountNums as
mixing. StdDevp beside Count alone is not probed, but every other aggregate is.

## Record filters: page selections and hidden items
Rule: a page field's selected item keeps only the records holding it, and a hidden item (`item h="1"`)
of a page field drops its records, before anything is accumulated; the pivot shows only what is kept.
The engine applies hidden items of row and column fields the same way (unprobed, see Status).
Evidence: probes/t4137-pivot-captions K4 (excel-graph, 2026-11-06, T-4137): Product selects `A` and the
page field Units hides 1 and 2, so of A's records (Sales 10, 20, 4 with Units 1, 2, 6) only 4 is left,
and the pivot shows 4 (K4!A7).
Two readings are the engine's own, not probed:
- a `pageField@item` that names no data item of its field (an index past the field's `<items>`, one
  naming a subtotal or blank line, or a malformed number, which T-4139's import reads as absent) is
  `(All)`;
- a hidden item on a field that is on no axis filters nothing.

Status: VERIFIED (excel-graph) for a page field's selection and hidden items (K4), and for a hidden item of an inner
row field: probes/t4164-pivot-filter-totals SH0 (Product C hidden, no `subtotalHiddenItems`) leaves C's records out of
both region subtotals (13, 10) and the Grand Total (23), and SH1H, the same with `subtotalHiddenItems="1"`, shows the
same (as probes/t4129-pivot-layout-flags SH did: East 27, Grand Total 60); a hidden item before a Top-N is
probes/t4164-pivot-filter-order HT ("Several filters on one field"). A hidden item of a column field is unprobed.
UNVERIFIED (probe pending: T-4633): the two readings above.
Implemented in: crates/rustsheet-engine/src/pivot/result.rs (`filter`)

## A pivot result is bounded in work and nodes
Rule: computing a pivot's result (its filtered records, axis trees and accumulators) spends at most
268,435,456 work units (`MAX_PIVOT_WORK`) when run on its own, or what the caller's budget has left: one
unit per record and stage, per item ordered, per sort step, per node or accumulator made and per value
fed to an accumulator. It holds at most 10,000,000 nodes (`MAX_PIVOT_NODES`): axis nodes plus one
accumulator per (row node, column node, data field) some record reaches. Past either the pivot has no
result (`PivotComputeError::Limit`), where Excel would render it.
Its cells (`render`, T-4149) hold at most 134,217,728 bytes of text together
(`MAX_PIVOT_TEXT_BYTES`, about 32 per cell of the cell cap), since each cell copies its label;
past it the render fails with `Limit`, where Excel would show the pivot.
Its layout (`lay_out`, T-4147) draws on the same work units, one per line and `x` made, and has no
result past 4,000,000 cells (`MAX_PIVOT_CELLS`: the location's rows times columns plus two per page
field, checked from the line counts before any cell is made) or when the location would pass the
sheet's last row or column. The cell cap is the resource limit; the sheet's edge is not one: past
it the layout fails with `PastSheet`, which the grid writer renders as `#SPILL!` at the anchor
("The grid writer's own readings", UNVERIFIED). The values stay without a layout, so GETPIVOTDATA
still has them. A refresh of several pivots (one cache, or every `refreshOnLoad` cache on open)
spends one budget of `MAX_PIVOT_WORK` work and `MAX_PIVOT_CELLS` cells written, cleared and
scanned together (`PivotBudget`), not one per pivot.
A calculated field's formula (T-4174) is at most 8,192 characters (`MAX_CALCULATED_FORMULA_CHARS`, Excel's cell
formula limit) and a cache holds at most 256 calculated fields (`MAX_CALCULATED_FIELDS`); past either the importer
skips the cache and `compute` has no result. Each formula is evaluated once per cell and once for the empty
intersections, its node clone and its evaluator work charged to the pivot's budget (the same units), and the
fields' dependencies are ordered without recursion (Kahn's algorithm), so a chain of 255 calculated fields costs no
stack.
Evidence: none for Excel; the work, node, text, cell and calculated-field caps are resource limits (SPEC-0032
"Untrusted-input limits", T-4142, T-4151, T-4174).
Status: DEVIATION (decision 0014)
Implemented in: crates/rustsheet-engine/src/pivot/result.rs (`MAX_PIVOT_WORK`, `MAX_PIVOT_NODES`, `charge`,
`count_nodes`)
Implemented in: crates/rustsheet-engine/src/pivot/layout/lines.rs (`MAX_PIVOT_CELLS`, `location`; T-4147)
Implemented in: crates/rustsheet-engine/src/pivot/layout/render.rs (`MAX_PIVOT_TEXT_BYTES`; T-4149)
Implemented in: crates/rustsheet-engine/src/pivot/grid.rs (`PivotBudget`; T-4151)
Implemented in: crates/rustsheet-engine/src/pivot/calculated.rs (`MAX_CALCULATED_FORMULA_CHARS`,
`MAX_CALCULATED_FIELDS`, `Sandbox::evaluate`; T-4174)
Implemented in: crates/rustsheet/src/edit/pivot.rs (`PivotError::Limit`: a pivot the API creates, sets
or refreshes past a limit is not added or changed, and nothing is written; T-4178)

## A pivot created through the API
Rule: `Txn::add_pivot_table` builds the cache from the source as Excel's refresh reads it and lays the
pivot out from the top-left cell of its location, with no `<items>` on its fields, so each axis shows its
items in source order (first appearance) and a blank as `(blank)`, as a refreshed file pivot does ("Items
show in the order the pivot field lists them"). Excel's UI may sort the items of a pivot it creates
ascending (by type); unprobed (T-4869). Three rules are the API's own choices, not Excel's evidenced
behavior: a sheet's pivot names are unique (`PivotError::DuplicateName`), `Txn::set_pivot_table` edits
a pivot where it is (`PivotError::Moved`), and a new pivot replaces whatever its anchor cell holds (what
Excel's UI does with a non-empty destination is unprobed). Past its anchor, a layout that meets a stored
value or formula collapses to `#SPILL!` at the anchor ("A pivot that would meet a foreign cell collapses
to #SPILL!"), and one that meets a Table, a merged range or another pivot, or passes the sheet's edge,
collapses the same way ("The grid writer's own readings", UNVERIFIED).
Evidence: none for a pivot Excel creates (T-4869); the rest follows the refresh entries above.
Status: UNVERIFIED (probe pending: T-4869)
Implemented in: crates/rustsheet/src/edit/pivot.rs (`Txn::add_pivot_table`, `Txn::set_pivot_table`; T-4178)

## Pivots without refreshOnLoad keep their saved cells; a source edit does not re-render
Rule: Excel for the web does not re-render a pivot whose cache lacks `refreshOnLoad` when it opens the
file. It keeps the cells the file saved in the pivot's area and saves the definition back as it found it
(`location`, `rowItems`/`colItems`, cache), even when they disagree with the cache or are absent. Nor
does it re-render a pivot after its source is edited with no refresh: the area keeps the render it had.
Evidence: probes/t4586-pivot-refresh-on-load (excel-graph, 2026-11-06, T-4586), over Data East `=10`
(cached as missing), West 20, East 5, so the saved cells (East 99, Grand Total 99, no West), a render from
the cache (East 5, West 20, total 25) and a refresh (East 15, West 20, total 35; Q!B4 sums the
source to 35) all differ:
- (a) sheet A, `refreshOnLoad` off, location A3:B5, saved cells and `rowItems` listing East and the
  grand total only: A4:B5 stay East 99, Grand Total 99 (Q!A1:D1; A6:B6 stay empty, Q!E1:F1 read 0). The saved
  `rowItems`, `colItems`, location and the cache's flags (`containsBlank`) come back unchanged.
- (b) sheet B, the same with no `rowItems` or `colItems`: the same saved cells (Q!A2:F2), and Excel saves
  the pivot with no row or column lines (the snapshot does not tell an absent `<rowItems>` from an
  empty one). Absent layout lines do not make it render.
- (c) sheet C, a `refreshOnLoad` pivot over Src, whose B2 is blank in the file: Excel renders East 5,
  West 20, Grand Total 25 on open. The oracle then writes Src!B2 = 100 through a Graph range write
  (`entered`) and runs a FullRebuild calculate, with no pivot refresh: Q!A4 reads 100, and the pivot
  still shows East 5 and total 25 (Q!A3:F3 and the sidecar's area). A refresh would show 105 and 125.

Consequences: an engine loads such a pivot's area as the plain cells the file holds and builds nothing
for it until a refresh; a calculate, including a full rebuild, is not a refresh. A source edit on a pivot without
`refreshOnLoad` does not re-render it either (probes/t4130-pivot-auto-refresh AR0, "PivotTable Auto Refresh is not
stored or honoured by Excel for the web"). Not probed:
desktop Excel (M14). In (c) the range write is the session's first call, so it lands after the load's
refresh: East 5 shows that refresh read the blank B2.
Status: VERIFIED (excel-graph)
Implemented in: crates/rustsheet/src/edit/pivot.rs (`Workbook::refresh_pivots_on_load`, `Txn::refresh_pivot_table`; T-4151, public since T-4178)

## Pivot field names for duplicate and blank source headers
Rule: on refresh Excel names each cache field after its source header, made unique ignoring case:
- Text headers are named first, left to right. A header that no earlier name equals keeps its text,
  whitespace included (`" "`). Otherwise it takes the first `<header><k>`, k = 2, 3, ..., that no
  name taken so far equals, in the header's own casing. The suffixed names count as taken, and so
  does a real header that already ends in a number: `Sales`, `Sales2`, `="Sales"` save as `Sales`,
  `Sales2`, `Sales3`, but `Sales`, `="Sales"`, `Sales2` save as `Sales`, `Sales2`, `Sales22`.
- A blank header, an empty cell or empty text (`=""`), is `Field<n>`, n its 1-based position in the
  source, not its sheet column. It is named after all text headers, later ones too, and made unique
  the same way: `X`, `=""`, `Field4` save as `X`, `Field42`, `Field4`.
- A constant header typed twice is renamed like a formula one (`Sales`, `Sales`, `SALES` save as
  `Sales`, `Sales2`, `SALES3`).
- A whole-text header `ß` blocks a later `SS`: `é`, `="É"`, `ß`, `="SS"`, `="s"&"s"` save as `é`,
  `É2`, `ß`, `SS2`, `ss3`. Items do not equate them: in the same source the Region items `ß` and
  `SS` stay apart, as do `Straße` and `STRASSE`, while `é` and `É` are one item (casing of the
  first, `é`). Whether `ß` inside a longer name blocks its `ss` spelling, and whether names compare
  by the `=` collation, are open (below).
- A record cell holding `=""` is a text item of its own, apart from an empty cell's `(blank)` item;
  its row label is an empty cell. It is listed after `(blank)`: a, (blank), then the `""` item. That
  fits new items being appended on refresh ("Items show in the order the pivot field lists them;
  new items are appended"), since the builder caches formula cells as blank, so `""` is the one new
  item.

Evidence: probes/t4585-pivot-field-names (excel-graph, 2026-11-06, T-4585): headers D!A1:F1 Region,
Sales, `="Sales"`, `=""`, `="region"`, `="Sales"&""` save as Region, Sales, Sales2, Field4, region2,
Sales3. probes/t4627-pivot-field-name-collisions (excel-graph, 2026-11-06, T-4627), sidecar `pivots`
cache_fields of the pivot on sheet P_<source>:
- SufA!A1:D1 Region, Sales, Sales2, `="Sales"` → Region, Sales, Sales2, Sales3;
- SufB!A1:D1 Region, Sales, `="Sales"`, Sales2 → Region, Sales, Sales2, Sales22;
- FldA!A1:D1 Region, Sales, Field4, `=""` → Region, Sales, Field4, Field42;
- FldB!A1:E1 Region, Sales, X, `=""`, Field4 → Region, Sales, X, Field42, Field4;
- Off!B1:G1 Region, Sales, X, `=""` (sheet column E), Y, Z → Region, Sales, X, Field4, Y, Z;
- Empty!A1:E1 Region, Sales, X, an empty cell, Y → Region, Sales, X, Field4, Y;
- Const!A1:D1 Region, Sales, Sales, SALES (constants) → Region, Sales, Sales2, SALES3;
- Space!A1:E1 Region, Sales, `=" "`, `=" "&""`, X → Region, Sales, `" "`, `" 2"`, X;
- Fold!A1:G1 Region, Sales, é, `="É"`, ß, `="SS"`, `="s"&"s"` → Region, Sales, é, É2, ß, SS2, ss3.
  Its pivot (rows Region over é, É, ß, SS, ss, Straße, STRASSE with Sales 1, 2, ..., 64) shows
  P_Fold!A4:B8 é 3, ß 4, Straße 32, SS 24, STRASSE 64. That order (Straße before SS and STRASSE)
  comes from the case builder caching formula cells as blank: `="É"`, `="SS"`, `="ss"` and
  `="STRASSE"` are no items in the saved cache, so the refresh appends SS and STRASSE after the
  saved é, ß and Straße (the sidecar's row_items `x` 0, 2, 3, 4, 5);
- Rec!A2:A6 a, `=""`, empty, `=""`, a with Sales 1, 2, 4, 8, 16: the rendered cells P_Rec!A4:B6 are
  a 17, (blank) 4 and an empty label cell 10 (the sidecar holds no shared items, so the rendered
  cells are the evidence).

Empty and Const are built with the case builder's `cache_names` (pivot_definition.py), which saves
placeholder names that Excel replaces on refresh, as it replaces formula headers' text.

The probed sources cannot tell whether the suffix search restarts at 2 or starts at the count of
earlier equal headers; both give every name above.

Open (T-4640):
- number, boolean and error headers;
- `ß` inside a longer name (`Straße` against `STRASSE`): the engine's `name_key` folds it to `ss`
  anywhere, an unprobed extrapolation of the one whole-text `ß` header;
- whether names compare by Excel's `=` collation, where `ß`/`ss` is one of several expansions
  (coercion.md "Collation equality in lookups and criteria"): `æ`/`ae`, `œ`/`oe`, `ĳ`/`ij`, `þ`/`th`,
  `á`/`a`+U+0301, U+00AD ignored;
- other case folds: `ẞ` (the engine folds it to `ß` and so treats `ẞ`, `ß`, `SS` and `ss` as one
  name, though exact MATCH keeps `ẞ = ß` apart from `ss`: lookup.md "MATCH, VLOOKUP and HLOOKUP
  exact-match keys fold case one character to one"), `ǆ`/`Ǆ`, `ı`/`I`, `ς`/`σ`, ligatures;
- a blank's `Field<n>` meeting a suffixed name from a later duplicate, and `Field4` against a typed
  `field4`;
- carried over to T-4640 from code pointers: whether a refresh matches new fields to the old
  cache's by name, case or position (`refresh_builder`, pivot/refresh.rs), and whether
  GETPIVOTDATA's item `"#N/A"` names an error item (functions/lookup_and_reference/pivot_index.rs).

Status: VERIFIED (excel-graph) for the rules above; the open points are UNVERIFIED (probe pending:
T-4640). The empty label cell is evidenced for a row item (P_Rec!A6); that a column item's is
empty too is the engine's assumption, UNVERIFIED (probe pending: T-4640).
Implemented in: crates/rustsheet-engine/src/pivot/refresh.rs (T-4141, T-4627) (`field_names`,
`name_key`); the empty label cell in crates/rustsheet-engine/src/pivot/layout/render.rs
(`Cells::lines`, T-4152)

## Capability matrix 1: what Excel for the web opens and renders
Rule: Excel for the web opens and renders a pivot carrying any one of the 143 catalog rows of the
layout, totals, aggregation and show-values-as families except three. It refuses
`pivotField@dataSourceSort="1"`, `item@d="1"` and `item@f="1"` (next entry). This covers every
ST_DataConsolidateFunction, ST_ItemType, ST_ShowDataAs and x14 ST_PivotShowAs value, every location,
axis and totals attribute, and the x14 pivotTableDefinition, pivotField and dataField extensions. The
catalog's `web` column (eval/catalog/pivot-features.tsv) records the answer per row. Its `evidence`
column names the probe and sheet.
Evidence: these cases, all excel-graph, 2026-11-06, T-4129:
- probes/t4129-pivot-show-as: the nine ECMA modes, below;
- probes/t4129-pivot-show-as-x14: the six x14 modes;
- probes/t4129-pivot-layout-flags: grand totals off, subtotalTop, subtotalCaption, subtotalHiddenItems,
  showItems, gridDropZones, dataPosition, allDrilled and defaultAttributeDrillState, pivotField
  numFmtId, a pivotArea with `axis="axisValues"`, and both x14 extensions;
- probes/t4129-pivot-item-flags: items with `c`, `e` or `m`;
- for the other rows, the earlier probes the catalog cites (t4123, t4135, t4137, t4138, t4168, t4586,
  t4622, t4311).

Also seen in Excel's saved files (T-4129 runs, from the oracle's cached downloads). No committed file
evidences these; the sidecars do not record definitions:
- Excel keeps every probed attribute and both x14 extensions as written, with three exceptions:
  - it drops an item's `m="1"`;
  - it drops `e="1"`, which is the default;
  - it drops `axis="axisValues"` on a pivotField that is on no axis (a scratch file, not a case:
    the builder sets `axis` itself), and the pivot still renders.
- `c="1"` is kept.

Status: VERIFIED (excel-graph) for rendering. "What Excel saves" is an observation of the cached
downloads; the sidecars do not record definitions.

## Pivot flags Excel for the web refuses
Rule: Excel for the web refuses to open a workbook (createSession 501 `FileCorruptTryRepair`, inner
`unsupportedWorkbook`) if a pivot has any of the following:
- a pivotField with `dataSourceSort="1"`;
- an item with `d="1"`;
- an item with `f="1"` and no `calculatedItems`;
- a `percent` dataField with `baseItem="9"` on a two-item field (Product), past its last item (T-4161
  run, excel-graph, 2026-11-07). That every `baseItem` past the field's last item other than (previous)
  and (next) is refused is UNVERIFIED (probe pending: T-4750).

- (T-4130) a pivot field item with `f="1"` even when the cache lists it in `calculatedItems`: every calculated
  item ("Excel for the web refuses calculated items");
- (T-4164) a Top-N `sum` filter whose `top10@val` is negative, top (-5) or bottom (-10);
- (T-4130) a filter of `type="unknown"`;
- (T-4130) a filter whose filterColumn criterion is a `colorFilter`, with or without `dxfId` and `cellColor`
  (`<colorFilter/>` gives `OpenWorkbookBlockedWorkbook` instead);
- (T-4130) a filter with `iMeasureHier` (an OLAP measure, on a worksheet cache);
- (T-4130) an item with `s="1"` (`s="0"` opens);
- (T-4130) a pivotField with `hideNewItems="1"` or with `hiddenLevel="1"`;
- (T-4130) a tuple list (`<tpls>`) under a cache shared item of any type (`s`, `n`, `m`, `e`). Under a record value
  (`pivotCacheRecords/r/*`) it opens.
- (T-4505) a calculated field (`cacheField@formula`) whose formula calls NOW(), TODAY(), RAND() or
  INDIRECT("A1"), holds an array constant (`SUM({1,2})+Units`) or a cell reference (`Data!C2*2`, `A1*2`), or
  names no field (`Foo*2`: the file is refused, it does not show #NAME?).

`allDrilled="1"` and `defaultAttributeDrillState="1"` on the same field open.
Evidence: T-4129 runs (excel-graph, 2026-11-06), one pivot per flag, bisected from the FA sheet of
probes/t4129-pivot-layout-flags and the sheets of probes/t4129-pivot-item-flags. T-4130 runs (excel-graph,
2026-11-07), one pivot per flag, bisected from the sheets of the t4130 probes before the refused sheets were dropped
(the probes' descriptions name them), and for the tuples scratch files built from probes/t4130-pivot-label-filters CEq
with one `<tpls c="1"><tpl fld="0" hier="0" item="0"/></tpls>` added: on the shared item Apple (refused), on a Sales
number of rows Sales (refused; control without it opens), on a blank Product item (refused; control opens), on an
error item (refused; control opens), and on the first record's Sales value as `n`, `m`, `e` or `s` (all open: input
sha256 743d27d3…, 155006e8…, 06b87095…, 4eecedac…).
T-4505 runs (excel-graph, 2026-11-07): one scratch file per formula, each a pivot at A3 on the Data sheet of
probes/t4505-pivot-calc-fields, rows Product, Sum of Units and Sum of the calculated field (input sha256 440c1e4c…
NOW, aa406971… TODAY, d53c648d… RAND, c500f97a… INDIRECT, 4f4dea18… the array constant, 4e950356… Data!C2,
88cd70f0… A1, 15b96cd7… Foo). VLOOKUP(1,{1,2},2) and GETPIVOTDATA("Units",A3) were refused too, but each also
holds an array constant or a cell reference, so neither refusal is the function's; a similar shape (rows Product, the calculated fields only) with INDEX,
ROWS, LOOKUP, MATCH, VLOOKUP over fields, a text constant and a cycle opens (probes/t4505-pivot-calc-field-functions).
T-4164 runs (excel-graph, 2026-11-07): the first
probes/t4164-pivot-top-n run, whose NBS sheet held a bottom sum filter with `val="-10"`, was refused; bisected one
sheet per file, NBS alone was refused and every other sheet opened. Rerun on files of the Data, Neg, NBS and Q sheets
only: a bottom sum `val="-10"` (input sha256 7d676540…) and a top sum `val="-5"` (0dfd4906…) are refused, and the
control with `val="10"` (57396809…, the committed NBS) opens.
Status: a builder observation, not committable as a probe: Excel saves nothing for a file it refuses.
What desktop Excel does is out of scope (decision 0266). The case builder does not refuse these
flags; a case that sets one cannot be filled.

## Show-values-as modes as Excel for the web renders them
Rule: every ECMA `showDataAs` mode and every x14 `pivotShowAs` mode renders on open. Observed in the
oracle's cached downloads, not in a committed file (the sidecars do not record definitions): Excel saves
an x14 mode as written, as the dataField extension's `pivotShowAs` with no `showDataAs` on the dataField.
What each mode showed in one pivot is listed below. The pivot has rows Region (outer) and Product,
columns Year, and Sum of Sales. The data are East A 10/4, B 5/8, C 3/– and West A 20/6, B –/7, for
2024/2025. West/B/2024 and East/C/2025 are empty intersections.
- **difference** (base Product A, DF): the base item's rows are empty. So are the Region subtotal rows
  and the Grand Total row, because the base field is the inner one. An empty intersection counts as 0:
  East/C/2025 is -4 (0 - 4) and West/B/2024 is -20.
- **difference from (previous)**, with `baseItem="1048828"` on Year (DP): the first Year column is
  empty, and so is the Grand Total column. Every other cell is this year minus the previous one,
  subtotals and the Grand Total row included (East 2025 is -6, Grand Total 2025 is -13).
- **percent** (% of Product A, PC): the base rows show 1. **An empty intersection shows `#NULL!`**
  (East/C/2025, West/B/2024), not 0 and not `#N/A`. The row totals divide the totals (East/C 3/14).
- **percentDiff** (PD): like difference, with `#NULL!` for an empty intersection.
- **runTotal** in Product (RT): the total runs down the Products within each Region. Subtotal and Grand
  Total rows are empty.
- **percentOfRow, percentOfCol, percentOfTotal** (PR, PCo, PT): an empty intersection shows 0 (a
  number), not an empty cell.
- **index** (IX): an empty intersection shows 0. The Grand Total row and column are 1.
- **percentOfParent** in Region (PP): the parent rows (East, West) are 1. The Grand Total row is
  empty.
- **percentOfParentRow** (PPR): each value is divided by its parent row item. For East and West the
  parent is the grand total row: East/2024 is 18/38. The Grand Total row is 1.
- **percentOfParentCol** (PPC): with one column field it equals percentOfRow (except a row total of 0,
  which is empty; see T-4161).
- **percentOfRunningTotal** in Product (PRT): this is the running total divided by the Region total.
  Subtotal and Grand Total rows are empty.
- **rankAscending and rankDescending** in Product (RA, RD): each column is ranked within its Region,
  and the Grand Total column too. An empty intersection is an empty cell and is not ranked. Subtotal
  and Grand Total rows are empty.

Evidence: probes/t4129-pivot-show-as and probes/t4129-pivot-show-as-x14 (excel-graph, 2026-11-06,
T-4129). The sidecars' `pivots` sections hold every cell.
Status: VERIFIED (excel-graph) for these shapes. Ties, division by zero, a missing base item, (next),
and the other base fields are in the next entry (T-4161). The engine is T-4163.

UNVERIFIED (T-4948): two dataField extension shapes the exporter writes (T-4759,
`export/pivots/show_as.rs`) are seen in no Excel-saved file. Removing a stale x14 mode, when the
model's mode is ECMA or `normal`, can leave an attribute-less `<x14:dataField/>` in its `<ext>`,
beside a `showDataAs` or beside none. A kept `extLst` in a prefixed main namespace gains the
extension as `<x:ext>`; Excel writes the main namespace unprefixed. Whether Excel for the web opens
each shape, what it renders and what it saves back are unknown until T-4948 probes them.

## Show-values-as edge cases (T-4161)
Rule: each mode's base items, ties, running totals, subtotal and grand-total cells and division by
zero render as listed below, per mode. This extends the entry above, mode by mode. Every pivot below has rows Region (outer) and Product, columns
Year, and Sum of Sales, unless it says otherwise. The cases are probes/t4161-pivot-show-as-base-items (BI),
-running-rank (RR), -zeros (Z), -edges (E), -running-zero (RZ) and -nested-parent (NP), all excel-graph,
2026-11-07, T-4161. Each case's description lists its data. "Empty" means an empty cell.

**Where the base field sits.** This applies to difference, percent, percentDiff, runTotal,
percentOfRunningTotal and the two ranks.
- Base field the inner row field (Product): items are compared within their outer item only. A
  subtotal row and the Grand Total row are empty. The Grand Total column shows values (BI DPP, PPN, QPP; RR
  RAP, RDP).
- Base field the outer row field (Region): each row is compared with the row of the same inner item
  under the other Region: West/A with East/A, and the West subtotal row with the East one. The Grand Total
  column shows values and the Grand Total row is empty. BI DRP: West/A/2023 is 20 - 10 = 10, and the West
  row in 2023 is 20 - 15 = 5.
- Base field the column field (Year): the Grand Total column is empty. Subtotal rows and the Grand Total
  row show values (BI DYP, PY1, DY1; RR RDY, RTY, PTY).

**Base items.**
- **(previous) and (next)** (`baseItem` 1048828 and 1048829) do not cross outer items. Under Product,
  the first item in each Region (for previous) or its last (for next) is its own base, as is the first or
  last Year (BI DPP, PPN, DYP, DYN). The cell of an item that is its own base is shown as that mode shows
  the base item's own cell. Difference and percentDiff leave it empty. Percent shows 1, or leaves it empty
  where the value is empty (BI PYP column 2023, PYN column 2025).
- **A base item missing from the parent.** Product C is the base item and West has no C. **Every cell of
  West's product rows is `#N/A`**, the Grand Total column too, even where the value is empty. This holds
  for difference, percent and percentDiff (BI PPC, DPC, QPC, rows 10 and 11). West's subtotal row stays
  empty, like every subtotal row under an inner base field.
- A base item that exists, over a missing combination, is only an empty base. BI QRN has base field
  Region and base item (next). The base of East/C would be West/C, which does not exist, so East/C's
  cells are empty, not `#N/A`.
- A `baseItem` past the field's last item: Excel for the web refuses the file. See "Pivot flags Excel
  for the web refuses".

**difference.** An empty value and an empty base both count as 0, even when both are empty: E D0
West/A/2025 is 0, and BI DYP East/C/2025 is 0 - 3 = -3. The base item's own cells are empty.

**percent (% of).**
- The base item's own cells are 1. They are empty where the value is empty, and `#DIV/0!` where the
  value is 0 (Z PC A/2024; E P0 the East row in 2023).
- In any other cell, the first rule that applies decides:
  1. An empty value is `#NULL!`, also when the base is empty: E P0 West/A/2025 and the West row in
     2025 have an empty value and an empty base.
  2. A present value over an empty base is empty (BI PYP East/B/2025; BI PY1 East/B/2023).
  3. A base of 0 gives `#DIV/0!` (Z PC, PY).
  4. Otherwise the cell is value/base.

**percentDiff (% difference from).** The base item's own cells are always empty, even when the base
is 0 (Z PD A). The other cells follow percent's rules (`#NULL!`, empty, `#DIV/0!`). The result is
(value - base)/base with the base's sign: E Q0 East/A/2024 is (2 - (-4))/(-4) = -1.5, not 1.5.

**runTotal (running total in).**
- An empty intersection shows the running total so far (RR RTT West/C/2024 is 27). At the run's first
  item it shows 0 (RR RTY West/C/2024).
- Under the inner field, the total restarts at each outer item (RR RTT; T-4129 RT).
- Under the outer field, it runs across Regions. A West row is East plus West for the same inner item,
  subtotal rows included: RR RTR West in 2024 is 13 + 27 = 40, and West/A in 2024 is 5 + 20 = 25.
- Under the column field, it runs across Years. The Grand Total row runs (40, 73) and the Grand Total
  column is empty (RR RTY).

**percentOfRunningTotal (% running total in).**
- The running total is divided by the run's total, which is its last value. The last item of the run
  is therefore 1, an empty intersection included (RR PTR West/C/2024).
  - Under the outer Region, the run's total is the same inner item's total over all Regions (for the
    subtotal rows, the Grand Total row): RR PTR East/A/2024 is 5/25 and East in 2024 is 13/40.
  - Under Year, it is the row's Grand Total (RR PTY).
- **Over a total of 0, a cell whose own value is 0 is empty, and a cell with a nonzero value is
  `#DIV/0!`.** What an empty value over a total of 0 shows is UNVERIFIED (probe pending: T-4750). RZ PRY: East/A (-4, then 4) is `#DIV/0!` in both years, while East/B (0, 0) is empty. In
  E PRY's Grand Total row, 2023 (value 0) is empty, while 2024 (3) and 2025 (-3, running total 0) are
  `#DIV/0!`. In Z PRT and in RZ PRP's Grand Total column, a total of 0 over 0s is empty.

**rankAscending and rankDescending.**
- **Ties share a rank, and the ranks are dense.** RR RDP East/2024 is A = B = 5, ranked 1, and C = 3,
  ranked 2 (not 3). RR RAP East/2025 is A 1, then B = C = 2. Z RA East/2024 is A = C = 0, ranked 1, and B
  2.
- Negative values rank below 0 (Z RA East/2025: B -5 is 1, C 0 is 2, A 4 is 3).
- Under the outer Region, the subtotal rows are ranked against each other. Each product is ranked
  against the same product in the other Region (RR RAR, RDR). An equal pair ties at 1: B's Grand Total is
  13 in both Regions.
- Under Year, each row is ranked across the Years. The Grand Total row is ranked and the Grand Total
  column is empty (RR RDY).
- Empty intersections are empty and are not ranked (T-4129).

**percentOfRow, percentOfCol and percentOfTotal.** An empty intersection is 0. **A denominator of 0 is
`#DIV/0!` on every cell divided by it**, the totals included:
- percentOfRow: Z PR East/B, East/C and West, and E PR's Grand Total row;
- percentOfCol: E PCo's 2023 column and its Grand Total column;
- percentOfTotal: every cell of E PT, whose grand total is 0.

**index.** (value × grand total)/(row total × column total). An empty intersection is 0.
- A row or column total of 0 is `#DIV/0!` (Z IX East/B, East/C, West; E IX 2023 and both Grand
  Totals).
- A grand total of 0 over nonzero row and column totals gives 0 (E IX 2024, 2025).
- With three row fields, the Grand Total column is 1 at every level (NP IX3).
- The order of the operations is UNVERIFIED (probe pending: T-4754). The engine computes value × grand
  total / row total / column total, left to right. Of the probed cells only T-4129 IX!B5 tells the
  orders apart (value 18, grand total 63, row total 30, column total 38: Excel 0.994736842105263, which
  this order gives; (value × grand total)/(row total × column total) and the other orders are 1 ulp
  off). That one cell is below the R4.4 web-versus-desktop floor, so it does not settle the order.

**percentOfParent (% of parent total).**
- Each row is divided by its ancestor on the base field, in the same column, however deep it is. With
  rows Region, Product and Kind and base field Region, East/A/x in 2024 is 1/7 (the East row), not 1/3
  (East/A) (NP PPR_).
- The base field's own rows are 1. Rows above the base field are empty (NP PPP, Region rows), and so is
  the Grand Total row.
- An empty intersection below the base field is 0 (NP PPR_ East/A/y/2025). With base field Kind, the
  innermost field, each leaf is its own parent: it is 1, or empty where the value is empty (NP PPK).
- On a column field (NP PPY: columns Year, then Kind; base field Year), the Kind columns are divided by
  their Year's subtotal column. Each `Year Total` column is 1, the Grand Total column is empty, and the
  Grand Total row shows values.
- **A parent of 0 leaves the cell empty, whatever the value** (Z PP West; E PP East/A/2023 is -4 over 0).

**percentOfParentRow and percentOfParentCol.**
- Each cell is divided by its immediate parent on that axis.
  - NP PPRow: Kind by Product, Product by Region, Region by the Grand Total row. The Grand Total row
    is 1.
  - NP PPCol: Kind by its `Year Total` column, and a `Year Total` column by the Grand Total column.
    The Grand Total column is 1.
- **A parent of 0 leaves the cell empty** (E PPR East/2023 and East/A/2023; Z PPR West/A, West/B).
- With one column field, percentOfParentCol divides by the row's Grand Total. It equals percentOfRow
  except that **a row total of 0 is an empty cell, not `#DIV/0!`** (Z PPC East/B; E PPC Grand Total row).

Evidence: the six cases above (excel-graph, 2026-11-07, T-4161). The sidecars' `pivots` sections hold
every cell.
Status: VERIFIED (excel-graph) for these shapes: one data field, Values not on an axis, every item
visible, and items in cache order. These are UNVERIFIED (probe pending: T-4750):
- (previous) and (next) over hidden items or a custom sort;
- percent or percentDiff where the value is empty and the base is 0;
- percentOfRunningTotal where the value is empty and the total is 0;
- whether every `baseItem` past the field's last item is refused, beyond the one probed;
- two data fields, or Values on rows.

The engine is `pivot::show_as` (T-4163), a post-pass over the computed values that computes every
(row node, column node) cell. It matches every cell of the probes above and of T-4129's. Its own
readings, where no probe shows Excel's, are UNVERIFIED (probe pending: T-4750):
- a base field on no axis makes every cell `#N/A`;
- a `baseItem` that names no item of its field makes every cell below the base field's parents
  `#N/A` (Excel for the web refused the one probed file, `baseItem="9"`: "Pivot flags Excel for the
  web refuses");
- an empty value over a percentOfRunningTotal total of 0 is empty;
- a data field with both `showDataAs` and an x14 `pivotShowAs` shows the x14 mode;
- -0 is shown as 0;
- a base item that exists over a missing combination leaves a difference cell empty, as BI QRN shows
  for percentDiff;
- an error value, base or denominator is the cell's error, the value's first; a running total stays
  an error from the first error on; a rank leaves errors out of its run;
- a cell whose denominator line has no record at all (a `showAll` item) is empty;
- a result that is not finite is `#NUM!`;
- `baseItem` indexes the base field's `<items>`, as ECMA-376 reads it (in the probes the fields list
  their items in cache order, so it is also the cache item);
- a custom subtotal line of another function than the data field's shows its plain value;
- GETPIVOTDATA on a pivot whose data field is shown as a calculation is `#REF!`, as it was while the
  importer dropped such a pivot;
- with several data fields, each is shown on its own (the probes have one).

## Layout and totals flags as Excel for the web renders them
Rule, per flag (each one on its own pivot of rows Region and Product with Sum of Sales):
- `rowGrandTotals="0"` with `colGrandTotals="0"` (NG): no Grand Total row and no Grand Total column.
- `subtotalTop="0"` on Region in outline form (ST): the East row is left empty, and an `East Total` row
  follows its products.
- `subtotalCaption="Tot"` on Region in tabular form (SC): **the subtotal label is `Tot` alone, with no
  item name**. Both East's and West's subtotal rows read `Tot`. More shapes: "subtotalCaption labels
  a field's subtotal lines" below.
- `subtotalHiddenItems="1"` with Product C hidden (SH): with the flag set, the hidden C is left out of
  East's subtotal (27, without C's 3) and of the Grand Total (60). The flag-off control is T-4164's,
  so whether the flag changes anything is not evidenced here.
- `showItems="0"` (SI), `allDrilled` and `defaultAttributeDrillState` (FA), `dataPosition="0"` with
  the Values field on columns (DPo), and `hideValuesRow="1"` with one data field (X14): no visible
  change.
- `gridDropZones="1"`, tabular (GZ): the data caption `Sum of Sales` is at A3. `Row Labels` and
  `Product` head the row fields on row 4, and the value column's header reads `Total` (C4).
  `firstHeaderRow` is 2. More shapes: "Classic layout (gridDropZones)" below.
- `numFmtId="3"` on the Year row field (NF): it renders. The saved labels are numbers; their display
  format is not captured.
- x14 pivotTableDefinition `fillDownLabelsDefault="1"` (X14): **existing fields are not filled down**,
  in tabular and outline form, with or without an empty x14 pivotField extension on the fields
  (probes/t4698-pivot-repeat-labels TW, TWE, OW). (Our reading, not evidenced: the setting is a
  default for fields added later, which a file cannot probe; UNVERIFIED, T-4764.) A field's own x14 `fillDownLabels="1"` (X14F)
  repeats its labels: `East` on every East product row.
- A `formats` entry whose pivotArea has `axis="axisValues"` (AV): it renders.

Evidence: probes/t4129-pivot-layout-flags (excel-graph, 2026-11-06, T-4129).
Status: VERIFIED (excel-graph) for these shapes.

## Capability matrix 2: filters, sorts and calculated fields and items
Rule: Excel for the web opens and applies a pivot carrying any one of the catalog rows of the filtering, sorting and
calculated families, except these, which it refuses ("Pivot flags Excel for the web refuses"): a filter of type
`unknown`, a `colorFilter` criterion, `iMeasureHier`, an item `s="1"`, `hideNewItems="1"`, `hiddenLevel="1"`, tuples
under a cache shared item, and every calculated item (its item `f="1"`). That covers every other ST_PivotFilterType
value (caption, value, Top-N count/percent/sum, fixed date, M1–M12, Q1–Q4 and every relative date), every
`autoFilter` child, `filter`, `pageField` and pivotField filter attribute, the x15 `pivotFilter` extension, every
ST_FieldSortType value, `autoSortScope` with every pivotArea and reference attribute, `nonAutoSortDefault`,
`customListSort`, and calculated fields, except the formulas listed in "Pivot flags Excel for the web refuses"
(T-4505). The catalog's `web` column (eval/catalog/pivot-features.tsv) records the
answer per row, and its `evidence` column names the probe and sheet: 146 filtering rows yes, 11 no and 4 `unknown`
(below), 42 sorting rows yes, 2 calculated rows (the calculated field) yes and 40 (calculated items) no. The grouping
and slicers-timelines rows are T-4767's.
Evidence: these cases, all excel-graph, 2026-11-07, T-4130:
- probes/t4130-pivot-label-filters (the 14 caption types), probes/t4130-pivot-value-filters (the 8 value types,
  Top-N count, percent and sum, a bottom N, a stale `filterVal`, `iMeasureFld` 1);
- probes/t4130-pivot-date-filters (the 8 fixed date types, M1–M12, Q1–Q4, `useWholeDay`, and DLit, below);
- probes/t4130-pivot-relative-date-filters (the 16 relative types and stale `val`/`maxVal`);
- probes/t4130-pivot-filter-attrs and probes/t4130-pivot-field-filters (the other criteria, filter and field
  attributes, page fields);
- probes/t4130-pivot-sort and probes/t4130-pivot-calc-items.

The four tuple rows under records (`pivotCacheRecords/r/{e,m,n,s}/tpls/tpl@hier`) opened in scratch files only
(previous entry), which are not committed: the case builder writes no tuples. Their catalog rows stay `unknown`.
Status: VERIFIED (excel-graph) for opening and rendering; what each shape shows is the next entries'. UNVERIFIED
(probe pending: T-4767, which teaches the builder to write tuples and commits the probe): that a tuple under a
record value opens.

## Pivot sorts and filters as Excel for the web renders them
Rule, per shape (each its own pivot, rows Product unless said, Sum of Sales):
- **Caption filters** match the item's text with the autoFilter's wildcards (`Ap*`, `*e`, `*an*`) or comparison;
  the `Not` forms are their complements. The match ignores case: `AP*` keeps Apple and Apricot (CBwU), `*AN*` keeps
  Banana (CCoU), `banana` equals Banana (CEqL), and Cherry and Grape are greater than `BANANA` (CGtU). captionBetween
  is inclusive at both ends: Banana to Cherry keeps both (CBtI). For a captionEqual, **the autoFilter decides, not
  `stringValue1`**: one whose `stringValue1` says Cherry and whose customFilter says Grape shows Grape (CMix); the
  other types were probed only with the two agreeing. The grand total covers only the items shown.
- **Value filters** compare each item's value of data field `iMeasureFld` (VM1 filters on the second data field, Count
  of Sales). Top-N `count` keeps the N largest (`top="0"`: smallest), `percent` keeps the largest items until they
  reach the percentage of the grand total (top 50 of 63 keeps Apple 30 and Banana 12), and `sum` keeps the largest
  until their sum reaches the value (40 keeps 30 and 12). The stored `filterVal` is ignored and recomputed (TCv).
- **Date filters** compare the item's date: fixed dates as serials, M1–M12 the month of any year (M1 keeps 1/15/2024
  and 1/10/2025), Q1–Q4 the quarter of any year. **Relative dates are evaluated against the day the web opens the
  file**: today, yesterday, tomorrow, this, last and next week, month, quarter and year, and yearToDate, each keeping
  exactly the bucket its period holds. A stale `val`/`maxVal` (and `valIso`/`maxValIso`) on thisMonth is ignored
  (RtMv).
- **A dateEqual filter on a field the file caches as numbers is dropped** and every item shows (DLit: literal
  date-formatted numbers, which the builder caches as `<n>`). The oracle's cached download of that run shows Excel
  saving the pivot with no `<filters>` and the field's items as `<d>` (with a `maxDate` of 1900-01-03T07:51:04, as
  observed). The other date types were tried this way only in a first run, not committed, where every one was
  dropped. Over DATE formulas, which the builder caches as missing, every date filter applies.
- **Other criteria under a pivot filter**: a values list (`<filters><filter val="Cherry"/>`) applies like the
  captionEqual (FVals); an `iconFilter` (FIcon) or a `dateGroupItem` on a text field (FDgi) keeps nothing (the pivot
  shows only an empty Grand Total). `hiddenButton`, `showButton`, an autoFilter `sortState`, a filter's `name`,
  `description`, `mpFld` and `evalOrder="0"` change nothing visible.
- **Several filters**: a label and a value filter on one field both apply, **whatever `multipleFieldFilters` says**
  (FMf1, FMf0: Banana, Cherry, Grape); a filter on each of two row fields both apply (FTwo).
- **Legacy AutoShow** (`autoShow`, `itemPageCount`, `rankBy`) is a Top-N on the field: top 2, or with
  `topAutoShow="0"` bottom 2 (AS, ASb). `measureFilter="1"` and `includeNewItemsInFilter="1"` change nothing visible.
- **Page fields**: with several items selected (hidden items, `multipleItemSelectionAllowed`) the page cell reads
  `(Multiple Items)` (PM); a `pageField@cap` (`Banana!`) and `@name` are not shown: the page cell reads the item,
  `Banana` (PC).
- **Label sorts**: `ascending` and `descending` sort the items' text; `manual` keeps the field's listed order (SM).
  `nonAutoSortDefault="1"` changes nothing visible (NA). With `customListSort` on (the default) Day ascending follows
  the day-name custom list (Mon, Wed, Fri, CL1); with it off, the alphabet (Fri, Mon, Wed, CL0).
- **Value sorts** (`autoSortScope` on a data field) order the items by their value, within each outer item (SVR), by
  a column item when the scope references it (SVC sorts by 2024), or by the second data field through the
  reference's `x`: SV2 sorts descending by Max of Sales, Ant 20, Bee 10, Cat 9, an order neither Sum order gives
  (Sum descending is Ant, Cat, Bee). Ties, blanks and errors: "Value sorts: ties, blanks and errors" (T-4164).

Evidence: the probes of the previous entry (excel-graph, 2026-11-07, T-4130); the sidecars' `pivots` sections hold
every cell. Data: Product sums Apple 30, Apricot 4, Banana 12, Cherry 11, Grape 6 (filters); Ant 28, Bee 14, Cat 18,
cached in the order Bee, Ant, Cat (sorts).
Status: VERIFIED (excel-graph) for these shapes. Ties, filter order and filters over totals are the next entries' (T-4164); the engine is
T-4165 (filters, Top-N, AutoShow), T-4827 (value sorts: "Value sorts: ties, blanks and errors") and T-4768 (label
sorts). The engine's own readings around the
filters are "Pivot filters: the engine's own readings".
Implemented in: crates/rustsheet-engine/src/pivot/filter.rs (`apply`; T-4165) (every caption, value, Top-N, date,
month, quarter and relative date filter, the values list, iconFilter, AutoShow and two filtered fields: the
t4130-pivot-label-filters, value-filters, date-filters, relative-date-filters, filter-attrs and field-filters cells pass
in value)
Implemented in: crates/rustsheet-xlsx/src/import/pivots/filters.rs (`read_filters`, DLit's dropped date filter; T-4165)
Implemented in: crates/rustsheet-engine/src/pivot/sort.rs (`value_sorts`; T-4827) (the value sorts SV, SVA, SVC, SVR,
SXA, SXR and SV2 pass)
Implemented in: crates/rustsheet-engine/src/pivot/sort/labels.rs (`display_order`; T-4768) and
crates/rustsheet-xlsx/src/import/pivots/table.rs (`customListSort`, read from the kept definition attributes) (the
label sorts SA, SD, NA, SM, CL1 and CL0 pass; the engine's own readings around them are "Label sorts: the engine's own
readings")

## Top-N filters at their edges: ties, N past the end, exact thresholds, signs
Rule, per shape (rows one field, Sum of one data field, one `top10` filter):
- **count** keeps the N largest (`top="0"`: smallest) **and every item tied with the Nth**: top 2 over Apple 50,
  Kiwi 30, Fig 30 keeps all three (TC2), bottom 1 over the two 10s keeps both (TB1). An N past the item count keeps
  every item (TC9, 9 of 6). A zero is an item like any other: top 3 over 2, 1, 0, -3, -8 keeps 2, 1 and 0 (NT3).
- **percent** and **sum** take the items largest first (`top="0"`: smallest first) and stop at the first item whose
  running sum reaches the threshold (the percentage of the grand total, or the `val`): reaching exactly stops (sum
  50 keeps Apple 50 alone, TS50; sum 110 keeps Apple, Kiwi and Fig, TS110; sum 3 keeps 2 and 1, not the 0 after them,
  NS3). **They keep no ties**: of two tied items only the one first in the field's `<items>` order is taken, in
  either direction (under a label sort the engine uses the sorted order instead, UNVERIFIED: "Label sorts: the
  engine's own readings", T-4799). TP40 (percent 40 of 150) and TS60 (sum 60) keep Apple and Kiwi, not Fig; TPb5 (bottom 5 percent)
  and TSb10 (bottom sum 10) keep Pear, not Lime: there the `<items>`, the cache and the source all list Kiwi and Pear
  first. With only the `<item>` elements listing Fig and Lime first (`item_order`; the cache and the source keep Kiwi
  and Pear first), TP40i keeps Fig and TSb10i Lime; with the cache, and the items with it, listing them first
  (`order`), TP40o keeps Fig and TSb10o Lime. So the `<items>` order breaks the tie, not the cache's or the source's
  (a value sort breaks ties by the cache instead: "Value sorts: ties, blanks and errors").
- The comparison is signed for **sum**: a bottom sum of 10 over -8, -3, 0, 1, 2 never reaches 10 and keeps every
  item (NBS), where an absolute comparison would stop after -8 and -3.
- **percent over a negative grand total** (-8): top 50 keeps every item (NP50) and bottom 50 keeps only -8 (NBP50).
  Two readings fit both; see Status.
- The kept items show in the field's order (its `<items>` order: TP40i shows Fig, Apple), not ranked: TC2 shows Kiwi, Apple, Fig.
- The grand total covers only the kept items (TC2 110, TS50 50, NB2 -11).

Evidence: probes/t4164-pivot-top-n (excel-graph, 2026-11-07, T-4164). Data: Product sums Kiwi 30, Apple 50, Fig 30,
Date 20, Pear 10, Lime 10 in that source order (total 150); Neg Item sums A 1, B 0, C -3, D -8, E 2 (total -8). Every
sheet's top10 has `filterVal` equal to `val`; the cached downloads show Excel saving each filter as written (an
observation of the oracle's cache only). A negative `val` on a sum filter, top (-5) or bottom (-10), makes Excel for
the web refuse the file ("Pivot flags Excel for the web refuses").
Status: VERIFIED (excel-graph) for these shapes. UNVERIFIED (probe pending: T-4799): how percent works over a negative
total, and over a positive total with items of both signs, where the two readings disagree too (bottom 50 percent of
10, -20, 30: "Pivot filters: the engine's own readings"). The running sum compared in absolute value with the percentage of the absolute total fits NP50 and NBP50, and
so does the running sum's share of the total compared with the percentage; the signed comparison that fits sum (NBS)
does not (it would keep only 2 for NP50). UNVERIFIED (unprobed, listed in T-4799): ties at the very first item of a
sum or percent filter, and ties under several data fields.
Implemented in: crates/rustsheet-engine/src/pivot/filter.rs (`Ranking::keeps`, `take_until`; T-4165) (percent over a
negative total as the absolute reading, UNVERIFIED as above)

## Several filters on one field: hidden items and label filters first, then value filters in `<filters>` order
Rule: with a Top-N and another filter on the same field, **a hidden item or a label filter is applied first and the
Top-N ranks only the items it kept, whatever the order of the filters in `<filters>`**; **a value filter and a Top-N
apply in the order `<filters>` lists them**. `multipleFieldFilters` changes neither. Over Kiwi 40, Apple 50, Fig 30,
Date 20, Lime 10 (Top-N first on all items would keep Apple and Kiwi):
- a hidden item: Apple hidden (`h="1"`) with count top 2 shows Kiwi and Fig (HT), not Kiwi alone (the top 2 of all
  items, Apple and Kiwi, less Apple); with percent top 50 it shows Kiwi and Fig (HP): 50 percent of the visible
  total 100, not of the full 150 (which would add Date);
- a label filter: captionNotEqual Apple with count top 2 shows Kiwi and Fig with either filter first and with
  `multipleFieldFilters` 1 or 0 (LT1, TL1, LT0, TL0); with percent top 50 likewise (LP1, PL1, LP0, PL0);
- a value filter: valueLessThan 45 listed before a count top 2 shows Kiwi and Fig (VT, and VT0 with
  `multipleFieldFilters="0"`), but listed after it shows **Kiwi alone** (TV1, TV0): the Top-N took Apple and Kiwi
  first, and the value filter then removed Apple.

Each pivot's grand total covers the kept items only (70, or 40 for TV1 and TV0). With a label and a value filter (no
Top-N) the order is not observable: both keep a fixed set.
Evidence: probes/t4164-pivot-filter-order (excel-graph, 2026-11-07, T-4164).
Status: VERIFIED (excel-graph) for these shapes: count Top-N with a hidden item, a label filter or a value filter;
percent Top-N with a hidden item or a label filter. UNVERIFIED (unprobed, listed in T-4799): a percent or sum Top-N
with a value filter, two value filters without a Top-N whose order matters, and a filter's `evalOrder` (written -1
here) or `id` (written in `<filters>` order here), either of which may be what Excel orders by.
Implemented in: crates/rustsheet-engine/src/pivot/filter.rs (`apply`: hidden items, then caption and date filters,
then value filters in `<filters>` order, `evalOrder` and `id` not read; T-4165)

## Filters on an inner field rank per parent; totals never count what a filter removes
Rule, rows Region then Product (East A 10, B 3, C 6; West A 1, B 9, C 4; field totals A 11, B 12, C 10):
- a Top-N or value filter on the inner field is evaluated **within each outer item**, on that item's own values:
  count top 1 keeps East A and West B (ITop), not B in both (B is the largest over the field); valueGreaterThan 5
  keeps East A and C and West B (IVal), not every item (all three exceed 5 over the field); percent top 50 keeps East
  A and West B (IPct), half of each region's own total;
- a Top-N on the outer field ranks the outer items by their totals and keeps their inner items whole (OTop: East,
  with A, B and C);
- **subtotals and the grand total cover only the items shown**, for every filter kind: ITop East 10, West 9, Grand
  Total 19; IVal 16, 9, 25; a hidden item (SH0, C hidden) 13, 10, 23; a label filter (SH1L) 9, 13, 22;
- `subtotalHiddenItems="1"` changes none of this: SH1T, SH1V and SH1L (Top-N, value and label filters) and SH1H (C
  hidden) show the same totals as without it (SH1T as ITop, SH1V as IVal, SH1H as SH0).

Evidence: probes/t4164-pivot-filter-totals (excel-graph, 2026-11-07, T-4164); probes/t4128-pivot-spike K (a Top-N
under an outer field) agrees. SH1L's flag-off control is probes/t4130-pivot-filter-attrs FTwo (a captionNotEqual
Apple filter on the inner Product, no `subtotalHiddenItems`: West 21 without Apple), on that case's data; SH1T and
SH1V have ITop and IVal as theirs, and SH1H has SH0.
Status: VERIFIED (excel-graph) for these shapes (compact form, one data field, subtotals at the top). UNVERIFIED
(unprobed, listed in T-4799): inner-field filters when the pivot has a column field or several data fields.
Implemented in: crates/rustsheet-engine/src/pivot/filter.rs (`ValueStep::run` ranks within each tuple of the outer
fields of the axis and drops the records of what it removes, so every total covers only what is shown; T-4165)

## Value sorts: ties, blanks and errors
Rule (`autoSortScope` on a data field, rows Product):
- **ties keep the order of the cache the file wrote when descending, and the reverse when ascending**: Kiwi, Fig
  and Plum all 30 (cached, listed and in the source in that order) sort Apple, Kiwi, Fig, Plum, Date descending (STd)
  and Date, Plum, Fig, Kiwi, Apple ascending (STa); not alphabetically. The `<items>` order does not break ties: with
  only the items written Fig, Date, Plum, Apple, Kiwi the descending order is still Kiwi, Fig, Plum (STo). The source
  order does not either: with the cache (and the items with it) written Plum, Fig, Kiwi, Apple, Date and the source
  still Kiwi, Fig, Plum, the ties sort Plum, Fig, Kiwi descending (STc) and Kiwi, Fig, Plum ascending (STca). Excel
  saves every one of these caches in source order (the sidecars' cache fields), so the tie order is the file's cache
  as read, not the cache Excel writes back. (A Top-N breaks ties by the `<items>` order instead: "Top-N filters at
  their edges".)
- **a blank sorts as 0** and **an error sorts last in both directions**: Sum of Amt C 9, A 5, E 2, D (blank), B #N/A
  descending (SEd) and D, E, A, C, B ascending (SEa). Sorting by the 2024 column (SCd, SCa), where Fig has no record
  and Date has -1: Apple 50, Kiwi 10, Plum 5, Fig (blank), Date -1 descending and the reverse ascending, so the blank
  sits between 5 and -1.
- the scope's column reference picks the column: Year item 1 (2025) sorts Fig 30, Plum 25, Date 21, Kiwi 20, Apple
  (blank) (SC2).

Evidence: probes/t4164-pivot-value-sort (excel-graph, 2026-11-07, T-4164).
Status: VERIFIED (excel-graph) for these shapes. UNVERIFIED (probe pending: T-4799): whether a blank ties with a real
0 (no item held 0; the engine ties them, by the cache order) and the order of several errors (the engine keeps the
cache order in both directions); ties of an inner field sorted under an outer field (unprobed; the engine breaks them
by the cache order, as for an outer field). UNVERIFIED (unprobed, T-4799), the engine's own readings: a
`reference@x` is an index into the field's `<items>` (every probe lists them in cache order, so a cache index fits
too) and only its first `x` is read; a reference to a field on no axis, on the sorted field's own axis or past
the cache is ignored; a scope whose data field (the data reference's `x`) or column item is past the definition's, or
that has no data reference, sorts nothing, as does a scope under `sortType="manual"`; an item `showAll` shows with no
record sorts as 0; a value sort on a column field reads as on a row field, the row items its scope; the value is the
one a value filter ranks by, so a Count or Average item with no number sorts as its cell shows it (0, `#DIV/0!` last).
A value sort by a calculated data field leaves the pivot without a result, as a value filter by one does
(UNVERIFIED, T-4897).
Implemented in: crates/rustsheet-engine/src/pivot/sort.rs (`value_sorts`; T-4827) (STd, STa, STo, STc, STca, SEd,
SEa, SCd, SCa and SC2, and t4130-pivot-sort SV, SVA, SVC, SVR, SXA, SXR and SV2, pass)
Implemented in: crates/rustsheet-xlsx/src/import/pivots/sort.rs (`read_auto_sort`; T-4827)

## Label sorts: the engine's own readings
Rule: none probed; these are what the engine (`pivot::sort::labels`, T-4768) does where the probes are silent (every
probe's sorted field holds three texts). Each is UNVERIFIED (probe pending: T-4799):
- a field of mixed item types sorts ascending numbers and dates (by value) first, then the names of the built-in
  custom lists, then the other texts, then booleans (FALSE first), then errors, as the worksheet's SORT puts numbers
  before text; descending reverses all of it; a blank item (`(blank)`) sorts last both ways;
- texts the collation ties (`apple` and `Apple` in one cache) and errors keep the cache order ascending and the reverse
  descending;
- under `customListSort`, a name matches a built-in list ignoring ASCII case, and the listed names sort before every
  other text, by list (abbreviated days, days, abbreviated months, months: the Custom Lists dialog's order) and then
  by position; only the abbreviated day names are evidenced (CL1), the other three lists are Excel's other built-in
  lists, unprobed;
- a field grouped by ranges of numbers or dates sorts its groups in their own order (the cache's), reversed descending,
  not by their labels' text; a manually grouped field sorts its group labels as texts;
- a field with an `autoSortScope` is never label-sorted, also when its scope sorts nothing ("Value sorts: ties, blanks
  and errors"), so it then keeps its `<items>` order;
- a Top-N filter breaks its ties by the field's display order, the label sort's under one (the `<items>` order with
  none, "Top-N filters at their edges");
- a label sort applies to a column field as to a row field, and to an inner field across every parent (K is a row
  field).
Evidence: none (engine readings).
Status: UNVERIFIED (probe pending: T-4799)

## A field shows its items in the `<items>` order, not the cache's, until a sort applies
Rule: with no sort (`sortType` absent or `manual`) a field shows its items in the order of its `<item>` elements,
whatever order the cache lists the shared items in. A label sort or a value sort replaces that order.
- cache Bee, Ant, Cat (x 0, 1, 2), items written x 2, 0, 1: Cat, Bee, Ant with no sortType (IO) and with `manual`
  (IOM);
- the cache written Cat, Ant, Bee and the items x 2, 0, 1: Bee, Cat, Ant (IOO), neither the written cache order nor
  the source order (Bee, Ant, Cat);
- the same items with `ascending`: Ant, Bee, Cat (IOA); by value descending: Ant 28, Cat 18, Bee 14 (IOV).
- the sidecars' cache fields show Excel saving the shared items in source order (Bee, Ant, Cat) for every one of
  these, whatever the file wrote.

Evidence: probes/t4164-pivot-item-order (excel-graph, 2026-11-07, T-4164); `item_order` in the case builder writes the
`<item>` elements in an order of their own (T-4164). IOD (`descending`) shows Cat, Bee, Ant, which is also its items
order, so it does not tell the two apart.
Status: VERIFIED (excel-graph) for these shapes.
Implemented in: crates/rustsheet-engine/src/pivot/refresh.rs (T-4141) (the items order: IO, IOM and IOO pass)
Implemented in: crates/rustsheet-engine/src/pivot/sort/labels.rs (`display_order`; T-4768) (IOA and IOD pass)
Implemented in: crates/rustsheet-engine/src/pivot/sort.rs (`value_sorts`; T-4827) (IOV passes)

## useWholeDay: a date filter compares the item's day, not the criterion's
Rule: a date filter compares an item's full date-time with the criterion unless its x15 `pivotFilter` carries
`useWholeDay="1"`; then **the item is cut to its day and compared with the criterion as written**. Over 4/16 12:00,
4/17 0:00, 4/17 8:00, 4/17 18:00, 4/18 0:00 and 4/18 6:00 (2024):
- without the extension or with `useWholeDay="0"`: dateEqual 45399 keeps only 4/17 0:00 (WEqN, WEq0); dateNewerThan
  45399 keeps 4/17 8:00 onward (WNtN, WNt0); dateOlderThanOrEqual 45399 keeps 4/16 12:00 and 4/17 0:00 (WOeN, WOe0);
  dateBetween 45399.25 and 45399.5 keeps 4/17 8:00 (WBtN, WBt0); dateEqual 45399.5 keeps nothing (WEqT0, with the
  extension at 0 only);
- with `useWholeDay="1"`: dateEqual 45399 keeps every 4/17 item (WEq1); dateNewerThan keeps only 4/18 (WNt1);
  dateOlderThanOrEqual keeps 4/16 and every 4/17 item (WOe1); dateBetween 45399.25 and 45399.5 keeps nothing (WBt1),
  and so does dateEqual 45399.5 (WEqT1): the criterion is not widened to its day.
- A filter that keeps nothing shows only an empty Grand Total row (A4 Grand Total, B4 blank).

Evidence: probes/t4164-pivot-date-times (excel-graph, 2026-11-07, T-4164); Sales are powers of two, so each total
names its set. The date items show as `m/d/yyyy h:mm` text (`4/17/2024 8:00`, and `4/17/2024` at midnight).
Status: VERIFIED (excel-graph) for these types over DATE+TIME formulas (cached as missing). Over dates the file caches
as numbers a date filter is dropped (DLit, "Pivot sorts and filters as Excel for the web renders them").
Implemented in: crates/rustsheet-engine/src/pivot/filter.rs (`date_holds`; T-4165). The date items' label text is
T-4717's.

## Pivot filters: the engine's own readings
Rule: none probed; these are what the engine (`pivot::filter`, T-4165) does where the probes are silent. Each is
UNVERIFIED (probe pending: T-4799):
- a filter applies only to a row or column field; one on a page field or a field on no axis applies nothing;
- the value filters of two fields apply field by field, the row fields outer first, then the column fields; a field's
  AutoShow applies after its `<filters>`;
- an inner-field filter beside a column field ranks each item by its total across the columns; with several data
  fields it reads `iMeasureFld` as with one;
- an item whose value is an error is kept by no value filter or Top-N; a count Top-N with N below 1 keeps nothing; a
  `top10@val` that is no number, a Top-N criterion under a caption or date type, a `customFilters` under a Top-N type,
  and an `iMeasureFld` or `rankBy` past the data fields apply nothing; an absent `rankBy` ranks by the first data field;
- a caption filter compares a number item's General text (and a boolean's `TRUE`/`FALSE`); a date filter's condition
  on an item with no date (text, a blank) is false, `notEqual` included; a values list matches a date item by its
  serial's text, and a `dateGroupItem` matches a date item's parts down to its `dateTimeGrouping`;
- a relative date filter compares the item's day with today as TODAY gives it (the workbook's clock in its time zone),
  also when the export lays a pivot out again, and its weeks run Sunday to Saturday; with no clock (a standalone
  `compute`) it keeps every item; a dynamic type other than the 16 relative ones, M1–M12 and Q1–Q4 keeps every
  item, a non-date one included, while M1–M12 and Q1–Q4 drop a non-date item;
- a filter whose `type` is unknown (or `unknown`, which Excel for the web refuses) applies nothing; a count Top-N's N
  is floored; an AutoShow reads the schema's defaults, `itemPageCount` 10 and `topAutoShow` on;
- a percent Top-N compares the running sum's absolute value with the absolute percentage of the parent's total, for
  every total, not only a negative one: with a positive total and items of both signs the readings of "Top-N
  filters at their edges" disagree (bottom 50 percent of 10, -20, 30, total 20, threshold 10: the absolute reading
  stops at -20, |-20| reaching 10, and keeps it alone; the share reading, -100 and -50 percent before 100, keeps -20,
  10 and 30);
- the importer skips a pivot whose `<filters>` it cannot type: a `filter@fld` past the cache's fields, an unknown
  `customFilter@operator` or an unknown `dateGroupItem@dateTimeGrouping` (before T-4165 such a pivot loaded with its
  filters kept and unapplied), and one past the caps of "Pivot filters past the import limits";
- a date filter is dropped on import (as DLit) when the cache field holds a number item and no date item, whatever
  else it holds;
- a `showAll` axis does not show the items a filter removed;
- GETPIVOTDATA on a pivot with a filter or AutoShow is `#REF!`, as it is for hidden items (SPEC-0015 unknowns).
Evidence: none (engine readings).
Status: UNVERIFIED (probe pending: T-4799)
Implemented in: crates/rustsheet-engine/src/pivot/filter.rs (T-4165)

## Pivot filters past the import limits
Rule: the importer skips a pivot (it loads without it, as for every pivot import limit) whose `<filters>` holds more
than 2,048 `<filter>`s (four per field of two full 256-field axes), a filter with more than two `customFilter`s (the
schema's `CT_CustomFilters` allows two), or a values list with more than 1,000,000 `filter` and `dateGroupItem`
values together (the cache's item limit). Excel writes at most one label, one value and one date filter per field and
a values list naming the field's items, so only an authored file meets these; what Excel does with one is unknown,
and it may open it.
Evidence: none for Excel; the caps bound the memory and work of an untrusted definition (CLAUDE.md directive 5).
Status: DEVIATION (decision 0014)
Implemented in: crates/rustsheet-xlsx/src/import/pivots/filters.rs (`MAX_FILTERS`, `MAX_CONDITIONS`,
`MAX_LIST_VALUES`; T-4165)

## Excel for the web refuses calculated items
Rule: Excel for the web refuses (createSession 501 `FileCorruptTryRepair`) every workbook whose pivot lists a
calculated item: a pivotField `<item f="1">` over a cache with `<calculatedItems>`. The refusal holds for every shape
tried: formulas `A+B`, `A*2-C/3`, `Product[A]-'B'` and `AB*2` (an item over another), with rows Region and Product,
with columns Year, with A and B hidden, beside a calculated field, with a `calculatedItem@field`, with `A +B` or
`'A'+'B'`, and with `refreshOnLoad` off. With the item written `f="0"` (or with no `f`) the file opens and Excel does
not compute the item: it renders A, B, C and Grand Total 63. It saves the `calculatedItems` normalised, not as written:
from CIA0's full pivotArea it drops `type`, `dataOnly`, `labelOnly`, `grandRow`, `grandCol` and
`collapsedLevelsAreSubtotals` (keeping `field`, `cacheIndex`, `outline`, `offset`, `axis` and `fieldPosition`), and
every reference flag but `selected`. It marks the AB shared item `u="1"` (unused) and the pivotField item `m="1"`
(missing). This is from the oracle's cached download of the CI0/CIA0/CF run (input sha256 964b6558…), not a committed
file. A calculated field (`cacheField@formula`, `databaseField="0"`) renders: Sum of Dbl, Dbl = Sales*2+1,
is twice the field sum plus one at every level (A 81, Grand Total 127).
Evidence: probes/t4130-pivot-calc-items CI0, CIA0 (the item `f="0"`) and CF (excel-graph, 2026-11-07, T-4130); the
refusals are T-4130 runs bisected from that case's first sheets (CI, CIR, CIC, CIK, CIQ, CI2, CIH, CIA, CIF, all
`f="1"`) and scratch variants of CI. The builder writes the calculated item with Excel's pivotArea (`cacheIndex="1"`,
the reference naming the item); no Excel-saved file with a calculated item has been opened on the web.
Status: VERIFIED (excel-graph) for the builder's shape. Whether an Excel-saved calculated item opens on the web is
UNVERIFIED (probe pending: T-4792): T-4505 found no Excel-saved file with one. The surveyed corpora have none
(docs/research/pivot-tables-2026-11-04/oss-engines.md §5), a GitHub code search for `calculatedItem` (2026-11-07)
found only schema documentation, no way to create one was found in the web UI (2026-11-07), and `rustsheet-oracle status` offers no other
golden backend (excel-desktop n/a, out of scope by decision 0266; LibreOffice is approximate and does not support
calculated items, libreoffice.md). So nothing about what a calculated item computes can be probed today, and
every rule of excel-docs.md quirks 5–7 is UNVERIFIED (probe pending: T-4792): the formula over items per record,
solve order, item references (`Region[North]`, positional `Region[-1]`, #REF! out of range), the restrictions
(no subtotal or summary-function change, no grouping), and how they meet several data fields. T-4175 depends on
T-4792. The catalog's calculated-item rows stay `no`.

## PivotTable Auto Refresh is not stored or honoured by Excel for the web
Rule: Excel for the web does not refresh a pivot after a Graph range write to its source: after the oracle types 100 into the
source's blank Data!B2 and runs a full-rebuild calculate, a pivot saved without `refreshOnLoad` (AR0) and a
`refreshOnLoad` pivot (AR1) both still show East 5, West 20, Grand Total 25, while Q!B1 sums the source to 125. Excel
saves no Auto Refresh setting: the saved pivotCacheDefinition carries only the attributes already known
(`refreshOnLoad`, `refreshedBy="Excel Services"`, `refreshedDate`, versions, `xr:uid`) and an empty
`x14:pivotCacheDefinition` extension, and the pivotTableDefinition only the empty `xpdl:pivotTableDefinition16`
extension. Answer to plan.md §10's question: no Auto Refresh attribute is in the file Excel for the web saves, and the
web does not auto-refresh (2026-11-07).
Evidence: probes/t4130-pivot-auto-refresh (excel-graph, 2026-11-07, T-4130): the sidecar's AR0 and AR1 areas and Q;
the saved parts are the oracle's cached download of that run (input sha256 668bde32…), not a committed file. The
source edit is a Graph range write in the session, not a typed UI edit; as t4586 (c) found for a `refreshOnLoad`
pivot.
Status: VERIFIED (excel-graph) that a Graph range write does not refresh; VERIFIED (excel-graph, cached download)
that the saved file carries no Auto Refresh setting. Whether desktop Excel, where Auto Refresh is an Insider feature on by
default for new local-data pivots (excel-docs.md AUTOREFRESH), stores a flag in the file, and whether a UI edit on
the web would trigger it, is UNVERIFIED: the oracle cannot create a pivot through the UI or reach desktop Excel
(decision 0266). T-4182 records this and is dropped unless a desktop-saved file shows a flag.

## Excel for the web rewrites a PivotChart's caches when it renders the pivot
Rule: when Excel for the web renders a pivot on open, it rewrites the series of a chart bound to it (`c:pivotSource`),
one series per plotted column line, whatever the part held:
- `c:tx` reads the series' column from the location's first row down to the row above the first data row
  (`'C'!$B$3`; with a column field `M!$B$3:$B$4`, the data field's caption row and the item row), cached as the
  series' name: `Total` with no column field, the column item (`A`, `B`, `C`) with one;
- `c:cat` reads the row-label column and `c:val` the series' column, both from the first data row down to the
  location's last row, the Grand Total row included (`'C'!$A$4:$A$6`, `'C'!$B$4:$B$6`; `M!$A$5:$A$7`, `M!$D$5:$D$7`),
  while their caches hold the plotted points only (`c:strCache` East, West; `c:numCache` 18, 27);
- a cell no record reaches has no `c:pt` (M's series C: `ptCount` 2, one point, 3 for East);
- the stale caches the file held (`Stale`, StaleX, StaleY, 999, 998) are gone, and a one-series chart over a pivot of
  three column items gets three series, `c:idx`/`c:order` 0 to 2;
- a sheet named `C` is quoted in the references (`'C'!$B$3`), as an R1C1-shaped name is in formulas; `CS`, `F` and
  `M` are not;
- the value cache's `c:formatCode` is the data field's number format (next entry but one).

Excel also rewrites what lies outside the series: `c:pivotSource`'s name becomes the uploaded file's name, it adds the
c16r2 namespace, `c:date1904`, `c:lang`, the c14 style, one `c:pivotFmt` per series (with `c:marker` none and a deleted
`c:dLbl` carrying an empty c15 extension), the `c:dLbls`, axis, `c:dispBlanksAs` and `c:printSettings` defaults, a
value axis `c:numFmt` with the cache's format code and `sourceLinked="1"`, and on each series a c16 `uniqueId`
`{0000000N-XXXX-XXXX-XXXX-XXXXXXXXXXXX}`, N the series' number from 1 and the rest one per chart.
For a pivot collapsed to `#SPILL!` (CS) all three references become the anchor `CS!$A$3`; the category cache holds
East and West, and the value cache has `ptCount` 2 and one point, 0, at index 0.
Evidence: probes/t4187-pivot-chart-caches (excel-graph, 2026-11-07, T-4187): Excel's saved file is committed byte for
byte as eval/roundtrip/excel-saved/cases/t4187-pivot-chart-caches.xlsx (sha256 cc3a1b7a…, the oracle's download for
input sha256 dfa8f50e…), charts `xl/charts/chart1.xml` (C), `chart2.xml` (CS), `chart3.xml` (F) and `chart4.xml` (M);
the sidecar holds the pivots' cells. Sheets C and CS repeat probes/t4130-pivot-chart-cache (T-4130), whose download
showed the same caches but was not committed. The input charts are written by tools/src/rustsheet_tools/pivot_chart.py.
Correction: this entry's earlier wording (T-4130) put all three references on the rendered ranges, `c:tx` included
(`'C'!$A$4:$A$6`, `'C'!$B$4:$B$6`); that was wrong for `c:tx`. The committed file shows `c:tx` reads the header cell
`'C'!$B$3`, and only `c:cat` and `c:val` read the item and Grand Total rows.
Status: VERIFIED (excel-graph, committed file) for one-series clustered column charts over one-row-field pivots, with
and without a column field. UNVERIFIED (probe pending: T-4938): several row fields (a `c:multiLvlStrRef` and its
reference's columns), a series of several labels, a pivot with no row field or no data field. UNVERIFIED (probe
pending: T-4953): Values on rows, error values, non-integer values' spelling, a collapsed pivot with several series, a
pivot with no header row above its values, a quote in a label, and chart types other than a clustered column
(pivot_chart.py writes only that).
Implemented in: crates/rustsheet-xlsx/src/export/pivot_charts.rs (`rewrite_pivot_chart`, T-4187), which writes the
series as listed above and reproduces Excel's four saved chart parts byte for byte, both from Excel's own parts and
from a part with Excel's series formatting and the input's stale references and caches. It keeps
everything outside the series as the part has it (so not the `c:pivotSource` name, the added `c:pivotFmts` or the axis
`c:numFmt`), gives a series past the part's last the last one's formatting and its `uniqueId` renumbered (no id where
the part has none), names a series of several labels by its labels joined with a space (UNVERIFIED, T-4938), leaves an error point out,
and for Values on rows takes the first data field's format code (UNVERIFIED, T-4953; the `pivotFmts`, axis `numFmt`
and `uniqueId` readings too). The export does not call it yet: a save that regenerates the package drops drawings and
charts (T-4950).

## A rewritten PivotChart part is capped
Rule: the export's PivotChart rewrite refuses (`PivotChartError::TooLarge`) a chart part whose rewritten text would
pass 64 MiB (`MAX_CHART_PART_BYTES`), checked before each piece is appended. Each series copies its template's
formatting children and the category cache, so one large series times many column lines would otherwise multiply
without bound. Excel caps a chart at 255 series; 255 series of about 4,000 categories fit. What Excel writes for a
PivotChart past its own limits is unknown; the caller keeps or drops the part as it decides (T-4950).
Evidence: none for Excel; a resource limit (CLAUDE.md directive 5, T-4187 security review).
Status: DEVIATION (decision 0014)
Implemented in: crates/rustsheet-xlsx/src/export/pivot_charts.rs (`Output`, T-4187).

## A PivotChart plots the pivot's item lines
Rule: a chart bound to a pivot (`c:pivotSource`) plots one category per row leaf and one series per column line, and
leaves out the grand totals. The two Excel-saved corpus charts show:
- one category per row leaf, labelled outer level first; Values on rows is the inner level, so the categories form
  a `c:multiLvlStrCache` whose levels are listed innermost first, each outer label only at the first point of its
  run (WithChartSheet.xlsx: Year then Values on rows, `lvl` 1 holds 2005, 2006, 2007 at points 0, 2, 4, `lvl` 0 the
  data fields at every point);
- the grand-total row and column lines left out (both files);
- with no column field and Values not on columns, one series named `Total`, whatever the cell above the values
  holds (WithChartSheet.xlsx `c:tx` `Sheet2!$C$3`, cached `Total`);
- a collapsed (`sd="0"`) column item as one series, named by its item alone, though its `c:tx` reference spans every
  header row (forum-mso-de-104083.xlsx: `Tabelle1!$B$3:$B$6` cached `2015`);
- no `c:pt` for a cell no record reaches (forum-mso-de-104083.xlsx: `ptCount` 53 with 51 points for 2015, 30 for
  2017);
- a number item's label in the field's number format: forum-mso-de-104083.xlsx's KW field (items 1 to 53, `numFmtId`
  165 `"KW "00` on both the pivot field and the cache field) is cached `KW 01` to `KW 53`, while the pivot's own cells
  hold the numbers.

Evidence: POI `test-data/spreadsheet/WithChartSheet.xlsx` (the `poi` suite at 9958a2a2, sha256 792b9f21…, Excel 12):
`xl/charts/chart1.xml` over `Sheet2!PivotTable2` (pivotTable5, `cacheId` 1: `pivotCacheDefinition2`; tabular,
`dataOnRows`, grand totals on); LibreOffice `sc/qa/unit/data/xlsx/forum-mso-de-104083.xlsx` (the `pivot-corpus-lo`
suite at b1653ced, sha256 c7182682…, Excel 16): `xl/charts/chart1.xml` over `Tabelle1!PivotTable2` (one row field,
three date-group column fields, every year collapsed). Both files are fetched, not committed, so their pivots and
chart caches are transcribed into crates/rustsheet-engine/src/pivot/layout/chart/tests.rs, which always runs (the
LibreOffice file's 2,303 records reduced to the per-week, year, quarter and month sums the chart reads);
crates/rustsheet-xlsx/tests/pivot_chart_corpus.rs also checks the files themselves when they are fetched.
Status: VERIFIED (corpus files, T-4186) for the shapes above. UNVERIFIED (T-4938), the engine's reading: leaving out
subtotal lines, an item's line with `subtotalTop` above its children, an outline item's own line and `insertBlankRow`
lines (neither corpus chart has one: WithChartSheet.xlsx has item and grand-total lines only, forum-mso-de-104083.xlsx
one row field); plotting show-as values as shown; leaving out filtered and hidden items; the name of a series with
several labels (the engine gives none); the levels when categories differ in depth (a collapsed outer row item beside
expanded ones); a date item's label; whether the pivot field's or the cache field's `numFmtId` formats a label; the
category of a pivot with no row level; and a pivot with no data field. The engine's chart tests also check every
plotted point of 30 probe pivots against the cells Excel rendered, which shows the values at the lines it plots are
Excel's pivot cells, not that Excel's chart plots those lines. Writing the caches back on export, and a pivot
collapsed to `#SPILL!`, are T-4187's (previous entry).

## A PivotChart's value cache format code differs by Excel version
Rule: a pivot chart's `c:numCache` `c:formatCode` is the data field's `numFmtId` code in an Excel 16 file and
`General` in an Excel 12 one, even where the pivot field carries a format. Current Excel for the web writes the data
field's code too: `#,##0.00` for a data field with `numFmtId="4"`, `General` for one with none.
Evidence: forum-mso-de-104083.xlsx (Excel 16): every series' `c:formatCode` is format 168
(`_-* #,##0\ _€_-;…`), the data field Summe's `numFmtId="168"`; WithChartSheet.xlsx (Excel 12): `c:formatCode`
`General`, though pivotTable5's Cost field is `numFmtId="6"` (its data fields carry none);
probes/t4187-pivot-chart-caches (excel-graph, 2026-11-07, T-4187), the committed
eval/roundtrip/excel-saved/cases/t4187-pivot-chart-caches.xlsx: F's data field `numFmtId="4"` gives
`<c:formatCode>#,##0.00</c:formatCode>` (and the value axis `c:numFmt formatCode="#,##0.00" sourceLinked="1"`), C's and
M's, with no `numFmtId`, `General`. The input caches all said `General`.
Status: VERIFIED (corpus files; excel-graph, committed file) for a data field's built-in format and none. UNVERIFIED
(probe pending: T-4953): a custom format, built-in 14 (the engine's en-US `m/d/yyyy`), a percent format, and which data
field's code a series of Values on rows gets. Implemented in: crates/rustsheet-xlsx/src/export/pivot_charts.rs (`format_code`, T-4187).

## GETPIVOTDATA against a pivot collapsed to #SPILL!
Rule: GETPIVOTDATA whose pivot reference is the anchor of a pivot collapsed to `#SPILL!` returns:
- `#REF!` for the grand total (`GETPIVOTDATA("Sales",S!$A$3)`, also by the name `Sum of Sales`);
- `#SPILL!` for a field and item pair that names an item of the pivot (`"Region","East"`);
- `#REF!` for a pair that names no item (`"Region","Nope"`).

A reference that is a cell of the layout the pivot would have had but not its anchor (S!$B$4), or a range holding the
anchor (S!$A$1:$D$10), gives `#REF!` too. The control pivot N, unblocked, gives 45, 45, 18, `#REF!` (no such item),
45, 27 for the same six. `ERROR.TYPE` of the anchor is 9 (`#SPILL!`), and `ISREF` of it is TRUE.
Evidence: probes/t4130-pivot-spill-getpivotdata Q!A1:B8 (excel-graph, 2026-11-07, T-4130): S holds the constant
`blocker` at B5 inside its pivot's layout.
Status: VERIFIED (excel-graph). When the pivot recovers (the blocker removed and a refresh) is not probed. The engine
is T-4154: it gives `#SPILL!` for the grand total (by either name) and for the unknown item, and 27 for the range
reference; the named item (`#SPILL!`) and the non-anchor cell (`#REF!`) already agree.

## Classic layout (gridDropZones)
Rule: `gridDropZones="1"` (classic layout), on a tabular pivot with row fields and one data field
(Values on no axis), changes the layout only when no field is on columns. It then keeps a
column-header row
anyway: the location grows by one row, `firstDataRow` and `firstHeaderRow` are both 2, the data
field's caption sits at the location's corner (GZ!A3, G1!A3), the row fields' header row is the
second row (`Row Labels`, then each further label column's field name: GZ!A4:B4, G1!A4), and the
value column's header there reads `Total` (GZ!C4, G1!B4). The rows below, and `rowItems` and
`colItems` (one empty `<i/>` column line), are the plain layout's. With a column field (GC: rows
Region and Product, columns Year) or with two data fields on columns (GV), cells, lines and
geometry are the plain tabular layout's (GC: `firstHeaderRow` 1, `firstDataRow` 2; `Sum of Sales`
at A3 and `Column Labels` at C3, as the unflagged tabular pivot of the same shape shows them:
probes/t4609-pivot-row-labels-2 L1, `firstHeaderRow` 1, `firstDataRow` 2, L1!A3 `Sum of Sales`,
L1!C3 `Column Labels`).
Evidence: probes/t4129-pivot-layout-flags GZ (T-4129); probes/t4745-pivot-classic-subtotal-caption
GC, GV, G1 (excel-graph, 2026-11-07, T-4745).
The engine's own readings, not probed (UNVERIFIED, T-4785). The engine applies the rule to every
pivot with no column level:
- compact or outline form (the same rule);
- no row field; `colGrandTotals="0"`; a `grandTotalCaption` (the engine always writes `Total`);
- two or more data fields with Values on rows (`dataOnRows`): the engine adds the header row and
  writes `Total` but leaves the corner empty;
- row fields with no data field: the engine adds the header row and writes `Total`;
- page fields beside classic layout: the page area is placed above the grown location as usual.
Status: VERIFIED (excel-graph) for tabular pivots with row fields, one data field and Values on
no axis (GZ, G1), and for the unchanged shapes GC and GV; UNVERIFIED (T-4785) for the readings
above.
Implemented in: crates/rustsheet-xlsx/src/import/pivots/table.rs (`read_root`: the typed
`PivotOptions::grid_drop_zones`; T-4745)
Implemented in: crates/rustsheet-engine/src/pivot/layout/lines.rs (`geometry`; T-4745)
Implemented in: crates/rustsheet-engine/src/pivot/layout/render.rs (`render_within`,
`CLASSIC_TOTAL`; T-4745)

## subtotalCaption labels a field's subtotal lines
Rule: a pivot field's `subtotalCaption` is the whole label of each of its default subtotal lines,
with no item name: `Tot` where the line would read `East Total`, for every item of the field. It
holds in tabular form (t4129 SC!A7, A10), in outline form with subtotals at the bottom (SB!A8,
A12), in compact form with subtotals at the bottom (SK!A8, A12), and on columns (SCC!E4, H4).
**A `?` in the caption stands for the subtotalled item**: `? Tot` reads `East Tot` and `West Tot`
(SQ!A7, A10), not the field's name (MS-OI29500's note on `subtotalCaption` says the field name;
Excel for the web writes the item; whether desktop Excel agrees is unconfirmed, T-4785).
With subtotals at the top (outline form, SO) the subtotal shares the item's own line, which
reads the item (`East`, SO!A4) as without the caption. Custom subtotal lines keep `<item>
<Function>` (SF!A7:A8 `East Sum`, `East Count`): the caption does not replace them. `rowItems`
and `colItems` do not change.
Evidence: probes/t4129-pivot-layout-flags SC (T-4129); probes/t4745-pivot-classic-subtotal-caption
SO, SB, SK, SF, SCC, SQ (excel-graph, 2026-11-07, T-4745).
The engine's own readings, not probed (UNVERIFIED, T-4785): a subtotalCaption on a field with
Values inside it (the engine keeps `<item> <data field>`), an empty `subtotalCaption=""` (the
engine writes it, an empty label), and a caption with several `?` (the engine replaces each).
Status: VERIFIED (excel-graph) for the shapes listed; UNVERIFIED (T-4785) for the readings above.
Implemented in: crates/rustsheet-xlsx/src/import/pivots/table.rs (`read_field`: the typed
`PivotField::subtotal_caption`; T-4745)
Implemented in: crates/rustsheet-engine/src/pivot/layout/render.rs (`Cells::total_label`;
T-4745)

## Show Details
Rule (the engine's reading; Excel's Show Details sheet is not probed): the records behind a value cell, which the
API returns as cache record indices (0 for the source row below the header) in cache record order, are: for an
item cell, the records the pivot's filters keep (a page field's selection, hidden items) whose
row and column items are the cell's; for a subtotal, those under its item; for a grand total, every kept record.
Evidence that the set is right: summing, counting or averaging the returned records' source values gives Excel's
value for every value cell of the probes in crates/rustsheet-engine/src/pivot/layout/records/tests.rs (t4137 K2,
K4, K6; t4138 G1 to G9; t4164 SH0, SH1H with a hidden inner item; t4135 A_sum, A_count, A_average; t4634 T1 to T3,
A1 to A3) and of the facade's tests (crates/rustsheet/tests/pivot_records.rs: getpivotdata-real-pivot, t4128 R).
Not probed (UNVERIFIED, T-4879): the row order of the sheet Excel's Show Details creates, the columns it shows (all
source fields, in which order, with which headers and number formats), whether it is a Table and how it is named,
and what it lists for a calculated field, a grouped field, an OLAP or Data Model pivot, or a cell an error poisons.
The API does not create that sheet. Top-N, value and label filters are not applied yet (T-4165): for a pivot
with one, the API's records include records Excel's totals leave out (pivots.md "Filters on an inner field rank per
parent; totals never count what a filter removes").
Status: UNVERIFIED (probe pending: T-4879) for the Show Details sheet; the record set is checked as above.
Implemented in: crates/rustsheet-engine/src/pivot/layout/records.rs (`records_behind`, T-4184);
crates/rustsheet/src/pivot_view.rs (`Workbook::pivot_cell_records`, T-4184).

## A non-English Excel saves its own UI language's default captions
Rule: the default captions (`Row Labels`, `Column Labels`, `(Multiple Items)`) are not stored in
the pivot definition: the saved cells hold them in the saving Excel's UI language, with no
`rowHeaderCaption` or `colHeaderCaption`. The `Values` caption is stored, in that language, as
`dataCaption`. The en-US engine renders the English captions, so the pivot-corpus classifies such a pivot stale
(T-4131). That an en-US Excel refresh of such a file writes the English captions is an inference from
the captions not being stored, not observed (UNVERIFIED).
Evidence (Excel-saved desktop files, cached cells and pivotTable parts): pivot-corpus-lo
`xlsx/pivot-table/shared-numgroup.xlsx` (Hungarian, A3 `Sorcímkék`, `dataCaption="Értékek"`),
`xlsx/pivottable_invalid_formats.xlsx` (German, B3 `Spaltenbeschriftungen`, `dataCaption="Werte"`),
`xlsx/tdf122471.xlsx` (Hungarian, B3 `Oszlopcímkék`), poi `ExcelPivotTableSample.xlsx` (German, B3
`Spaltenbeschriftungen`, and its page field's multiple-items caption in German at B1).
These are non-en-US desktop workbooks, so they are no evidence for what en-US Excel writes
(decision 0030): the English captions stay the ones the en-US probes give ("Caption attributes...").
Status: VERIFIED (Excel-saved files) as a storage fact; no rule for the engine follows from it.

## What the gen-pivot batch found (T-4132)
The first gen-pivot batch (eval/suites/gen-pivot, seed 0 of `rustsheet-gen --milestone M16`: 104 pivots, one per sheet,
each case's description naming its sheets' covering-array levels; excel-graph, 2026-11-07) shows these rules. Each is
VERIFIED (excel-graph) for the shapes cited; wider claims are marked. Each has an engine task that owns its cells.
- **Header labels in outline and tabular form.** The header row names every column level, one per cell from the first
  data column: the Values field as `Values` (or `dataCaption`), then each column field by name (gen-pivot/2251b0f5e5a256e1
  P12!M5:O5 `Values`, `Rep`, `Channel`; the second column field `When` at gen-pivot/01865529b5a09c0b P02!Q5). With the
  Values field on rows, its header cell reads `Values` (P02!L7). T-4914.
- **Field and item captions.** A pivotField's `name` replaces the field's header (`Product field`,
  gen-pivot/2251b0f5e5a256e1 P03!L7) and an item's `n` its label, in its subtotal label too (`West shown`,
  gen-pivot/01865529b5a09c0b P04!M10; `Silver shown Total`, gen-pivot/a6d74990f3ae755e P06!L11). Excel for the web opens
  both (catalog PV-1113, PV-1081). T-4915.
- **Custom subtotals and the Values field.** A custom subtotal line of a field with the Values field inside it reads
  `<item> <Function> of <source field>` per data field: `Bronze Sum of Score`, `Bronze Sum of Units` under data fields
  Sum of Score and Min of Units (gen-pivot/2251b0f5e5a256e1 P04!L13:L15). T-4916.
- **One custom subtotal in compact or outline form sits on the item's line**, as a default subtotal does: no
  `<item> <Function>` line (gen-pivot/640780d7b6f4ad53 P05!L8 `Web shown` with its values; gen-pivot/36996978427905b2
  P01, gen-pivot/f9e7edbb5b74011a P09, gen-pivot/ec473260bb2065b3 P05). Two or more functions, or tabular form, give one
  line per function after the children (gen-pivot/2251b0f5e5a256e1 P04, P10). T-4925.
- **Two custom subtotals cross blank.** Where a row field's custom subtotal line meets a column field's custom subtotal
  column the cell is blank (gen-pivot/2251b0f5e5a256e1 P10!P9:U16, gen-pivot/ec473260bb2065b3 P06). T-4926.
- **Date-time items are rounded to the second** on refresh: 366.99999999 is the item 367 (gen-pivot/2251b0f5e5a256e1
  P05!N14), 60.999988426 is 60.99998842592593 (gen-pivot/ca5c8fa99c13d747 P08!M8). Those cells alone only show a unit
  coarser than about 2 ms; the whole second is shown by the refreshed cache's `xsd:dateTime` items (for example
  `1900-02-28T23:59:59` in the gen-pivot/ca5c8fa99c13d747 P08 cache field `When`). Values without a date format are
  unprobed. T-4917.
- **A value cell whose records hold no value its aggregate reads is blank**: Count over blanks
  (gen-pivot/01865529b5a09c0b P01!O20), CountNums (gen-pivot/2251b0f5e5a256e1 P01!N6), Var and Max over records with no
  number (gen-pivot/f9e7edbb5b74011a P01!O9, gen-pivot/a6d74990f3ae755e P01!R13). This answers the T-4633 question of
  "Aggregates over text, blanks, booleans and errors". T-4918.
- **Classic layout with no row field** keeps the data caption at the corner and labels the value row `Total`
  (gen-pivot/01865529b5a09c0b P08!L5, L8; gen-pivot/2251b0f5e5a256e1 P07!L9). T-4919.
- **Record filters on a refreshed date field.** When the saved cache lists a date field's values as numbers (the case
  builder does), the refresh appends date items (above). Hidden items on such a field then hide every appended item, so
  the pivot shows no record (gen-pivot/a6d74990f3ae755e P02, P08, P12; gen-pivot/ca5c8fa99c13d747 P04: `rowItems` 0), and
  a selected page item lands on the refreshed list's item at its index (gen-pivot/462f10551d480f9d P01!M3 shows 365 for
  2958465). New items of other kinds beside hidden ones are unprobed. T-4920.
- **A page field with one visible item shows it**, not `(Multiple Items)` (gen-pivot/01865529b5a09c0b P05!M3 `Cy`,
  gen-pivot/2251b0f5e5a256e1 P08!M2 `Gold`); two or more show `(Multiple Items)` (P11 of 2251b0f5e5a256e1). This answers
  the page-selection question of T-4657. T-4921.
- **A missingCaption of `-` shows 0**: every empty intersection of every pivot with `missingCaption="-"` and
  `showMissing` is the number 0, shown in the data field's format (`0`, `0.00`; gen-pivot/ca5c8fa99c13d747 P01!O9:AJ28,
  gen-pivot/ec473260bb2065b3 P03), while `n/a` shows as the caption (gen-pivot/f9e7edbb5b74011a P08). Which other captions
  read as numbers is unprobed. T-4922.
- **No row field, non-compact, Values on columns: no row-label column.** The first column is the first data column
  (gen-pivot/640780d7b6f4ad53 P02 at L5:DC8, gen-pivot/f9e7edbb5b74011a P07, gen-pivot/462f10551d480f9d P08). T-4924.
- **A collapsed item with the Values field inside it** keeps one line per data field under the item
  (gen-pivot/36996978427905b2 P05!L6:L8). T-4927.
- Date items show in the source cells' date format, as on probes/t2085-getpivotdata-items D: T-4807 owns the gen-pivot
  cells too.

Status: VERIFIED (excel-graph) for the cells cited; the unprobed extensions above are UNVERIFIED.
