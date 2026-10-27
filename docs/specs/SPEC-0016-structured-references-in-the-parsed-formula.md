# SPEC-0016 — Structured references kept in the parsed formula

- **Status:** accepted (2026-10-21, T-2194)
- **Milestone / lane:** M8 / engine
- **Tasks:** T-2194 (this spec); T-2195 (step 1), T-1699 (step 2), T-2196 (step 3); T-1683 builds on steps 1-2; T-2197 (probe)

## Problem
The parser turns every structured reference into a plain cell or range node at parse time
(`Parser::structured_reference_operand`, parser/operands.rs). The stored formula is printed from
that node (`to_rc_format`), so the structured reference is lost as soon as the formula is entered
or imported. Measured on `main` (2026-10-20) with a table `Sales` over A1:B3 (data rows 2-3):

| Cell | Typed | Stored and exported | Value |
|---|---|---|---|
| E1 | `=SUM(Sales[Qty])` | `=SUM($A$2:$A$3)` | right |
| D3 | `=Sales[@Qty]` | `=$A$3` | right |
| D4 | `=Sales[@Qty]` | `=#VALUE!` | right now, but frozen |

After the table is re-added over A1:B4 (`remove_table` + `add_table`, which reparses every
formula), all three keep their old text. E1 still sums A2:A3, and D4 is still `#VALUE!` though
row 4 is now a data row. Excel keeps the structured reference in the formula and evaluates it
against the table as it is (gen-m8b in-table 19fdc25163755713). What Excel does when a table is
resized, converted to a range or deleted is unprobed (T-2197). Re-adding a table through
`remove_table` + `add_table` is behavior of the engine API only.

There are three consequences:

1. Our own storage loses the reference, so a later change to the table list cannot reach it.
2. `#This Row` is decided once, from the row of the cell where it was typed. The `#VALUE!` that
   T-1675 returns off the data rows is baked into the stored text (T-1699).
3. FORMULATEXT cannot show a structured reference (T-1683).

A fourth problem stops the obvious fix. Shared formulas are parsed once per sheet with the
context `(row 1, column 1)` (`Model::parse_formulas`, new_empty.rs). A preserved `[#This Row]`
re-parsed there would resolve against row 1 for every cell. So `#This Row` can only be resolved
when the cell's row is known, which is at evaluation time.

## Goals / non-goals
Goals:
1. A structured reference survives parse and print in the storage and in-engine printers:
   `to_rc_format` (storage), `to_english_string`, `to_localized_string`, `to_string_displaced` and
   `to_string_moved`. The xlsx export printers (`to_excel_string` and `to_excel_array_formula_string`, which share its export flag) keep printing the lowered area
   until the exporter writes table parts (step 3, T-2196). The exporter writes no
   `<tableParts>` or `xl/tables/*.xml` today, so a structured reference in an exported file would
   name a table the file does not have.
2. Its area follows the table's current metadata. Any change to the table list reparses every
   formula (`Model::reset_tables`, tables.rs), so the area is re-derived from the preserved text.
3. `#This Row` is resolved against the evaluating cell's row. Off the data rows it is `#VALUE!`
   (T-1675's rule), at evaluation time.
4. No value changes: every eval case and unit test that passes today still passes, and every
   formula that exists when a structural edit runs has the value it has today after the edit (see
   "Structural edits" below). Moving table metadata is the one new behavior.

Non-goals:
- Excel's own rules for tables under structural edits (resize, convert to range, delete, inserts
  and deletes that cross a table's columns). They are unprobed (T-2197). This spec only keeps
  today's values through those edits (see "Structural edits").
- Renaming a table or a column. There is no API for either.
- The FORMULATEXT and UI spelling (`[@Qty]` versus `[[#This Row],[Qty]]`). T-1683 probes it and
  picks the printer. This spec stores one canonical spelling and lets T-1683 decide what the
  user-facing printers show.

## Excel behavior to match
- Structured references evaluate against the table's current metadata. The evidence is for a
  fixed table only; behavior after the table changes is unprobed (T-2197). The cases are the
  gen-m8b cases (in-table 19fdc25163755713, this-row e1bf5aa0ffb4ed87, column-range
  44cecbb1ecd5b2da). Excel computed them from formulas written in the long form
  (`Sales[[#This Row],[Qty]]`), so the values need no further probe.
- `#This Row` off the data rows (header row, totals row, above or below the table) is `#VALUE!`
  (docs/excel-semantics/references.md "Structured references: this row, totals and ranges").
- `#Totals` on a table without a totals row is `#REF!`, as today.
- In the file, `@` is stored in its long form, `[#This Row]`. A stored `Sales[@A]` loads but is
  `#REF!` (docs/excel-semantics/xlsx-storage.md "Structured references in the file", VERIFIED).
  So the storage form uses `[#This Row]`, and so will the export form once T-2196 lands.
- A bare table name is stored in the file as `Sales[]`. A stored bare `Sales` makes Excel refuse
  the file as corrupt (same entry). So when step 3 prints structured references into the file,
  `to_excel_string` and `to_excel_array_formula_string` write a bare name as `Table[]`.
- The escape `'` goes before the special character (`Sales[C'#]`); a quote first is refused (same
  entry; gen-m8b d86ef297e840325f).
- Unprobed: what Excel does to its structured references when a table is resized, converted to a
  range or deleted (T-2197; recorded UNVERIFIED in references.md). Whether the stored text keeps
  the typed case of table and column names or uses the table's spelling is also unprobed. The
  canonical printer writes the specifiers in Excel's case (`#All`) and the names as typed, and
  T-1683 settles the names along with the `@` spelling.

## Design
### Node
Add one wrapper variant to `Node` (parser/mod.rs):

```rust
/// A structured reference as written (`Sales[[#This Row],[Qty]]`), with the area it resolves
/// to for the table list the formula was parsed against.
StructuredReferenceKind {
    reference: Box<StructuredReference>,
    lowered: Box<Node>,
},
```

`StructuredReference` holds the parsed source: `table_name`, `specifier: Option<TableSpecifier>`,
`table_reference: Option<TableReference>` (as `TokenType::StructuredReference` carries them
today) and `bare: bool`, which marks a bare table name (`ROWS(Sales)`). It is boxed so that `Node`
does not grow; the `NODE_SIZE` assert stays green.

`lowered` is what `structured_reference_operand` returns today: a `ReferenceKind`, a `RangeKind`
or an `ErrorKind`. The wrapper is emitted for an `ErrorKind` lowering too (`#Totals` without a
totals row, an unparsable table range), so that text is not lost either. There is one change, for `#This Row` (step 2): it lowers to a new leaf,

```rust
ThisRowKind {
    sheet_name: Option<String>,
    sheet_index: u32,
    column1: i32,
    column2: i32,
    data_start: i32,
    data_end: i32,
},
```

`ThisRowKind` does not depend on the row of the formula's cell. It is evaluated against
`cell.row`: when `data_start <= cell.row <= data_end`, it is the absolute area
`(cell.row, column1)..(cell.row, column2)`; otherwise it is `CalcResult::Error(VALUE)`. The
bounds come from the table at parse time and are refreshed by the reparse in goal 2, so the
evaluator needs no table lookup. They are computed with the checked arithmetic that
`structured_reference_operand` uses today: table metadata comes from an untrusted file, and an
overflow lowers to `#REF!`. Evaluation is a plain comparison and builds the area with no
unchecked arithmetic.

A parse error inside a structured reference stays a `ParseErrorKind`: an unknown table, an
unknown column, or a missing table sheet. This is today's behavior, and this spec does not change
it; Excel's result for such a formula in a loaded file is unprobed. A `ParseErrorKind` prints the whole formula text it was parsed from
(`stringify`), so a reparse against a table list without the table (between `remove_table` and
`add_table`) keeps the formula text, and the next reparse with the table restores the reference.
The value the cell has in between is today's parse-error value; Excel's is unprobed (T-2197).

### Transparency
The wrapper is transparent the way `Group` is (SPEC-0007). `Node::ungrouped()` also steps into
`StructuredReferenceKind.lowered`, and its doc comment says so. So every existing `ungrouped()`
call site (about 86; `rg "\.ungrouped\(\)" crates/rustsheet-engine/src`) sees the lowered node, with no change at the call sites. That covers evaluation,
ISREF/ISFORMULA-style shape checks, static analysis and reference operators. Every site that
matches on `Node::ReferenceKind` or `Node::RangeKind` without calling `ungrouped()`
(`rg "Node::(ReferenceKind|RangeKind)" crates/rustsheet-engine/src`) is checked one by one in
step 1. Each one either calls `ungrouped()` first, gets an explicit `StructuredReferenceKind` arm
that delegates to `lowered` (as `evaluate_node_with_reference` in model.rs needs), or is shown not
to receive parser output. A missed site must fall through to an error value, never to a panic.

`ThisRowKind` is a reference for every caller that asks "is this a reference" (ISREF, `@`,
reference operators, SUM's reference arm). One helper does the job:
`Model::this_row_area(&self, node, cell) -> Option<Result<Area, Error>>` (model.rs).
`evaluate_node_with_reference` uses it, so a `ThisRowKind` becomes the same `CalcResult::Range`
a `ReferenceKind` gives. It is not open-coded at each site. Step 2 re-audits every `ReferenceKind`/`RangeKind` site,
static analysis included (shape and `StaticResult`), for `ThisRowKind`.

Step 2 as built (T-1699): `ThisRowKind` also carries `cell: bool`, for one column named alone
(`Sales[@Qty]`), so it resolves to the `ReferenceKind` its parse-time lowering was and the other forms to
a `RangeKind`. The helper is `Node::this_row_at(row)`, the node the parse-time lowering gave on that row
(the cell or range, or `ErrorKind(VALUE)`), not a `Model` method returning an `Area`: the sites that match
on the kind of an argument call `Node::ungrouped_at(row)` (or `Node::resolved_at(row)` where they do not
ungroup), so each sees exactly the node it saw before step 2, and evaluation (`evaluate_node_in_context`,
`evaluate_node_with_reference`) resolves it the same way. Static analysis, which has no row, reads it as it
is on a data row (`Node::ungrouped_static`). A structural edit that moves a formula to another row
re-enters it with its `#This Row` areas lowered at the old row (`Model::moved_text`), and a cut and
paste to another row prints the lowered area, so the moved formula reads what it read; a formula that keeps
its row keeps its text. A duplicated sheet lowers each `#This Row` cell on its own row
(`lower_this_row_cells`, tables.rs), and a defined name, which has no cell, resolves its `#This Row` at row 1,
where its body is parsed (`Node::lower_this_rows_at`), both as before step 2.

### Printing
The storage and in-engine printers print a `StructuredReferenceKind` from `reference`, in the
canonical long form:
`Table[[#Spec],[Col]]`, `Table[[#Spec],[A]:[B]]`, `Table[Col]`, `Table[[A]:[B]]`, `Table[#Spec]`
or `Table[]`. Names that need escaping get Excel's `'` escapes (`'[`, `']`, `'#`, `''`, `'@`),
the inverse of `consume_escaped_until`. `ThisRowKind` is never printed on its own, because it
only appears inside a wrapper. Where a wrapper prints its lowered area instead, the rule at the
end of "Structural edits" applies.

A bare table name (`ROWS(Sales)`, operands.rs) keeps printing as the bare name in these
printers; T-1683 may change how it is spelled. `to_excel_string` prints `lowered` until step 3,
and from then on writes a bare name as `Table[]` (see above).

`rename_sheet_in_node` and `move_formula` recurse into `lowered` as they do today. The printed
text comes from `reference`, so a cut and paste leaves the text unchanged, and the next reparse
re-derives `lowered` from the table. A sheet rename leaves the text unchanged too, but the reparse
finds the table only if its `sheet_name` follows the rename: see "Sheet rename, delete and
restore".

### Structural edits
Today `insert_rows`, `delete_rows`, `insert_columns` and `delete_columns` (actions.rs) displace
every formula and never touch `workbook.tables`. `displace_cells` walks the cells, and
`shift_cell_formula` rewrites a cell only when `to_string_displaced` gives a different text. So
`=SUM(Sales[Qty])` follows its data when a row is inserted above the table. Once the text is printed from `reference`, the
displaced text would equal the old text, the cell would not be rewritten, and the next reparse
would read the table's unmoved range. Step 1 keeps the values of every formula that exists at
the time of the edit, using the mechanism below. Moving the table metadata is new behavior,
though: formulas typed after the edit resolve against the moved table, where today they use the
unmoved one. The rule that a table moves with an edit wholly before it, and grows with rows
inserted inside its data, is a hand assumption. T-2197 is to confirm it.

The mechanism covers every edit that rewrites formulas through `to_string_displaced` or
`to_string_moved`: row and column inserts and deletes, row and column moves (`move_rows_action`,
`move_columns_action`: `DisplaceData::RowMove`/`ColumnMove`), cell shifts, and cut and paste
(`move_formula` → `to_string_moved`, cut_paste.rs). It also covers every formula such an edit
rewrites: cell formulas, conditional-format formulas (`displace_cfvo`), and defined names.

Step 1 (T-2195) shipped every table as *fallback*, refined: a reference whose displaced or moved area is
the same text as its area keeps its written text, and only one whose area changes prints the area. T-2297
adds the *move* and *grow/shrink* classes on top of that rule.

1. **Classify, before the walk.** For a `DisplaceData::Row` or `DisplaceData::Column` edit, each
   table whose `sheet_name` is the edited sheet is classified as *move*, *grow/shrink* or
   *fallback* by the table below, and its new `reference` is computed by the same rule
   `to_string_displaced` applies to a range. A table on another sheet is untouched. For every
   other `DisplaceData` variant (cell shifts, row and column moves), every table on the edited
   sheet is *fallback*. For a cut and paste, every table whose range intersects the cut area
   or the paste area is *fallback*, and no table moves.
2. **Walk.** `to_string_displaced` and `to_string_moved` take the set of *fallback* table names (a
   new parameter; empty for callers that are not structural edits). A `StructuredReferenceKind` whose table is
   in the set prints its displaced `lowered` node, which is today's text. Any other prints
   `reference`, unchanged. Formulas on every sheet are walked, as today, so a formula on another
   sheet that refers to a table on the edited sheet is handled the same way.
3. **Update, then reparse.** After the walk, the *move* and *grow/shrink* tables get their new
   `reference` in `workbook.tables`, then `Model::reset_tables` hands the list to the parser and
   reparses. A cell rewritten during the walk may have been parsed against the old list; the
   reparse in this step corrects it. `shift_cell_formula` does not reparse a cell whose text is
   unchanged, and the parser holds its own copy of the tables, so this `reset_tables` is required:
   without it, cells keep stale areas. The full reparse happens only when a table on the edited
   sheet changed.

Edge cases, for a table with header row `h`, data rows `d1..d2` and an optional totals row `t`
(columns `c1..c2`). A row edit at row `r`, inserting `n` rows or deleting `r..r+n-1`:

| Edit | Class | Why |
|---|---|---|
| Insert, `r <= h` | move | every area shifts by `n` |
| Insert, `r` after the last table row | none | no area changes |
| Insert, `d1 < r <= d2` | grow | data, `#All` and totals areas grow or shift together, as the lowered ranges do |
| Insert, `r == d1` (between header and data) | fallback | the lowered data area shifts and the header does not: not a table shape |
| Insert, `r == t` (between data and totals) | fallback | the data area stays and the totals row shifts: not a table shape |
| Delete wholly above `h` or wholly below the last row | move / none | |
| Delete wholly inside `d1..d2`, at least one data row left | shrink | |
| Delete touching `h` or `t`, all data rows, or straddling an edge of the table | fallback | the lowered areas become trimmed or `#REF!`, which a table cannot represent |

T-2197 probed the row inserts (references.md "Structured references when the table changes"): Excel
*grows* the table on an insert at `r == d1` and at `r == t`, so those two fallback rows are superseded
(T-2321); the other insert rows hold.

A column edit at column `c`: an insert with `c <= c1`, or a delete wholly left of `c1`, is a
move; an edit wholly right of `c2` changes nothing; every other column edit that crosses
`c1..c2` is a fallback, since it would change the column list.

As built (T-2297, `table_edits.rs`): the walk takes the set of tables that *follow* the edit
(move, grow, shrink, or left alone by an edit wholly after them) instead of the fallback set
(`to_string_displaced_keeping`); a reference to any other table keeps step 1's refined rule. A
delete at the first or last data row is a fallback too, not a shrink, because its undo (an insert at
that row) would be a fallback and not grow the table back; T-2338 adds those edges with T-2321's
insert edges. A table that would grow or move past the last row or column is a fallback. A `#This
Row` reference keeps step 1's rule even when its table follows the edit (T-2843 probes it), and an
insert at the first row of a table without a header row is a fallback (T-2843). The insert of
rows or columns that would push a table off the sheet is allowed (the cells pushed off are empty)
and the table does not follow it: an engine decision (directive 5).

A fallback leaves the table metadata as it is today and loses the structured text of the
formulas that refer to that table, as today. T-2197's findings replace the fallback. The interim
rule is recorded in references.md "Structured references when the table changes". If the
classification costs more than it is worth in step 1, the split hint in T-2195 lets every table
be *fallback* at first, which is exactly today's behavior.

From step 2, a `ThisRowKind` printed as a lowered area (a fallback, or `to_excel_string` until
step 3) is first resolved at the printing cell's row. It becomes the absolute cell or row range on
that row, or `#VALUE!` off the data rows, which is exactly today's text. Only then is it
displaced.

### Sheet rename, delete and restore
These are not displacing edits, but once the text is printed from `reference` they would change
existing values, so step 1 covers them. Today a table's area is resolved at parse time, and the
lowered range carries the sheet index, so `rename_sheet_by_index` (new_empty.rs) and `take_sheet`
(sheet_undo.rs) keep today's values without touching `workbook.tables`. Under this spec, the
reparse after these edits reaches `structured_reference_operand` (operands.rs), which looks the
sheet up by the table's `sheet_name`. A stale name is "table sheet not found", which becomes a
parse-error value.

1. **Rename.** `rename_sheet_by_index` sets `sheet_name` on every table of the renamed sheet to the
   new name before `reset_parsed_structures`, so every formula that refers to the table, on any
   sheet, keeps its text and its value. Excel keeps structured references working after a sheet
   rename (certain; hand). References.md records it as hand evidence.
2. **Delete.** Before the sheet is taken, every table on it is treated as *fallback* for a walk
   over the formulas on the remaining sheets (and defined names): their structured references print
   their lowered areas, which the delete then turns into `#REF!`, exactly today's text and value.
   The tables are then removed from `workbook.tables` and kept with the taken sheet, so a sheet
   added later under the old name does not bring a deleted table's references back. Excel's
   result for a cross-sheet structured reference to a deleted sheet's table is assumed to be
   `#REF!` (unprobed; a question for T-2197; superseded: T-2324 probed it, see below).
   As built (T-2340, after T-2324 probed it): the structured references print `#REF!` instead
   of their lowered areas (FORMULATEXT `=SUM(#REF!)`, as Excel), and the delete keeps what it
   rewrote with the taken sheet.
3. **Restore.** `restore_sheet` (undo of a delete) puts the kept tables back in
   `workbook.tables`. The other sheets' formulas stay as the delete left them (today's undo
   behavior for plain references).
   As built (T-2340): the restore (and `UserModel` undo) puts back the structured text the delete
   rewrote to `#REF!`.
4. **Export.** Until T-2196, `to_excel_string` prints `lowered`, never `reference`. After a rename
   or a delete no formula is left as a parse error holding a structured reference, so an export
   then never writes a structured reference into a file without table parts (xlsx-storage.md
   "Structured references in the file"). The engine-API missing-table path is different: after
   `remove_table` the formula is a `ParseErrorKind` that prints its structured text, so T-2195
   decides what `to_excel_string` writes for it.

### Shared formulas and storage
The storage text is now `Sales[[#This Row],[Qty]]` in every cell of a filled column. It does not
depend on the cell, so one shared formula serves the whole column, as Excel stores it. Parsing it
at `(1, 1)` is now correct, because nothing in `lowered` depends on the context row.

### Resource limits
`StructuredReferenceKind` adds one level of depth per reference and parses nothing new, so the
parse depth caps of T-1169 hold unchanged. `ungrouped()` and the printers recurse one level more
per reference, so a depth test with many nested structured references is part of step 1. The
printers take table and column names from untrusted files: they never panic on any name (empty,
non-ASCII, special characters), and a property test checks that printing then parsing gives the
same reference. `ThisRowKind` evaluation is O(1).

## Alternatives considered
- **Resolve every structured reference at evaluation against `workbook.tables`.** This needs no
  bounds in the node, but every evaluation then does a case-insensitive table lookup and a column
  scan. Every `ungrouped()` site would also need the model and the cell to see a reference.
  Rejected because the reparse already gives fresh metadata.
- **Lower `#This Row` to a row-relative `ReferenceKind`** (`R[0]C1`) and drop the bounds check.
  The off-table `#VALUE!` would then be lost: the reference would read the off-table row's
  cell, where Excel gives `#VALUE!` (this-row e1bf5aa0ffb4ed87). Rejected.
- **Keep only the source text next to the cell and re-lower in `parse_formulas` with each cell's
  context.** This breaks shared formulas, which are parsed once per sheet. Rejected.

## Test plan
- Unit tests (model level, `crates/rustsheet-engine/src/test/test_structured_references.rs`):
  - E1, D3 and D4 from the table above. The stored text is kept (`get_cell_formula`). After
    re-adding the table over A1:B4, E1 sums A2:A4 and D4 reads A4 (T-1699's acceptance test).
  - Every specifier (`#All`, `#Data`, `#Headers`, `#Totals`, `[#Headers],[#Data]`,
    `[#Data],[#Totals]`, none) and the column forms print back unchanged in `to_rc_format` (and,
    from step 3, in `to_excel_string`; until then it prints the lowered area).
  - Escaped column names round-trip.
  - ISREF, `@`, the `:` and space operators, and SUM work with `Sales[@Qty]` as with `$A$3`.
    Excel evidence for SUM: gen-m8b in-table 19fdc25163755713 D11 (`SUM(Sales[@[2024]:[Val]])`).
    `ISREF` of `#This Row` is unprobed (specials e804cf2f8af00680 has `ISREF` of `#Totals` only):
    TRUE on a data row is a hand expectation, and off the data rows is a question for T-2197.
- Structural edits: one test per row of the edge-case table and per column case, each pinning
  today's values before the change. The move and grow/shrink cases keep the structured text; the
  fallback cases print today's lowered text. One test has the formula on a different sheet from
  the table.
- Sheet rename, delete and restore: after renaming the table's sheet, a same-sheet and a
  cross-sheet `=SUM(Sales[Qty])` keep their text and value; a cross-sheet formula is `#REF!` after
  the table's sheet is deleted, stays `#REF!` when a sheet with the old name is added, and the
  tables come back with `restore_sheet`; `to_excel_string` after a rename prints the lowered range
  (no structured text in the export).
- From step 2: `to_excel_string` of `Sales[@Qty]` on and off the data rows prints today's `$A$3` and
  `#VALUE!`.
- Missing table: after `remove_table` the cell's formula text is unchanged, and after
  `add_table` the reference works again.
- Printer round trip: a property test over adversarial table and column names
  (`[`, `]`, `#`, `'`, `@`, spaces, empty, non-ASCII) checks `parse(print(x)) == x`.
- xlsx: until step 3, an exported workbook keeps today's lowered ranges (a regression test). In
  step 3, a workbook with a table and structured references exports its table parts and the same
  formula text it imported. If an Excel-authored workbook in rustsheet-xlsx `tests/calc_tests`
  has structured references, the test uses it.
- Eval: `make eval` shows no regression (gen-m8b is the main suite). No gen-m8b case uses
  FORMULATEXT. FORMULATEXT of a structured reference is T-1683's to probe and fix.
- Parser unit tests (`parser/tests/test_tables.rs`) are updated: today they assert the lowered
  text (`$A$3`). They will assert the preserved text, and assert the lowered area through
  `ungrouped()`.

## Rollout
1. **T-2195 (M): static areas.** Add `StructuredReference` and `StructuredReferenceKind`, make
   `ungrouped()` transparent to it, wire the printers, audit the reference match sites, and move
   table metadata on structural edits with the fallback above.
   Every structured reference except `#This Row` emits the wrapper. `#This Row` keeps today's
   parse-time lowering, so its text is still lost. Export is unchanged. This step is complete on
   its own. If the diff gets too large, the printer escapes and their property test can be split
   into a second PR.
2. **T-1699 (S): `#This Row` at evaluation.** Add `ThisRowKind` and its resolver (built as `Node::this_row_at`, see "Step 2 as built"), and
   emit the wrapper for `#This Row` too. Re-audit the reference sites for `ThisRowKind`. Add the
   grow-the-table test and tests for `ISREF(Sales[@Qty])` and `SUM(Sales[@[A]:[B]])`.
3. **T-2196 (M): table parts in the export.** Write `<tableParts>` and `xl/tables/tableN.xml`,
   with the totals-row metadata xlsx-storage.md requires. Then switch `to_excel_string` to the
   preserved reference, with a bare name written as `Table[]`.
   As built (T-2196): `export/tables.rs` writes one part per table (numbered in sheet order, by name
   within a sheet), after the sheet's external hyperlinks in its rels part; a totals-row formula cell
   as `totalsRowFunction="custom"` with its `<totalsRowFormula>`, a text cell as `totalsRowLabel`;
   an `<autoFilter>` over the header and data rows. The export prints every resolved structured
   reference from `reference` (`StructuredReference::file_text`), so a `#This Row` is no longer
   lowered at the cell's row there; an unresolved one is still `#REF!`. The criteria of an applied
   filter are not kept (T-2838).
4. **T-1683** picks the FORMULATEXT spelling over the preserved node, and adds the lexer
   rejection of unterminated references.
5. **T-2197 (probe):** how Excel resizes, converts and deletes a table on row and column inserts
   and deletes, and what its structured references become. Implementing that in `insert_rows`
   and its siblings is filed from the probe's findings.
