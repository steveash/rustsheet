# SPEC-0016 — Structured references kept in the parsed formula

- **Status:** accepted (on merge of T-2194; architecture, quality, security and Excel-fidelity reviewers approved)
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
   `to_string_moved`. The xlsx export printer (`to_excel_string`) keeps printing the lowered area
   until the exporter writes table parts (step 3, T-2196). The exporter writes no
   `<tableParts>` or `xl/tables/*.xml` today, so a structured reference in an exported file would
   name a table the file does not have.
2. Its area follows the table's current metadata. Any change to the table list reparses every
   formula (`Model::reset_tables`, tables.rs), so the area is re-derived from the preserved text.
3. `#This Row` is resolved against the evaluating cell's row. Off the data rows it is `#VALUE!`
   (T-1675's rule), at evaluation time.
4. No value changes: every eval case and unit test that passes today still passes. There is one
   observable exception, from the first non-goal: after a structural edit, a reference follows
   the table metadata instead of the shifted range.

Non-goals:
- Tables that grow, shrink or move when rows or columns are inserted or deleted. Today the table
  metadata does not move with structural edits (`insert_rows` never touches
  `workbook.tables`). Excel's rules for this are unprobed, so it needs its own task. Before this
  spec, a lowered range shifted with the edit while the table stayed put. After it, the reference
  follows the table, which is the authority. Neither matches Excel when the edit is inside the
  table.
- Renaming a table or a column. There is no API for either.
- The FORMULATEXT and UI spelling (`[@Qty]` versus `[[#This Row],[Qty]]`). T-1683 probes it and
  picks the printer. This spec stores one canonical spelling and lets T-1683 decide what the
  user-facing printers show.

## Excel behavior to match
- Structured references re-resolve against the table as it is now. Evidence is the existing
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
  `to_excel_string` writes a bare name as `Table[]`.
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
unknown column, or a missing table sheet. Excel rejects those formulas, and this spec does not
change that. A `ParseErrorKind` prints the whole formula text it was parsed from
(`stringify`), so a reparse against a table list without the table (between `remove_table` and
`add_table`) keeps the formula text, and the next reparse with the table restores the reference.
The value the cell has in between is today's parse-error value; Excel's is unprobed (T-2197).

### Transparency
The wrapper is transparent the way `Group` is (SPEC-0007). `Node::ungrouped()` also steps into
`StructuredReferenceKind.lowered`, and its doc comment says so. So all 86 existing `ungrouped()`
call sites see the lowered node, with no change at the call sites. That covers evaluation,
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

### Printing
The storage and in-engine printers print a `StructuredReferenceKind` from `reference`, in the
canonical long form:
`Table[[#Spec],[Col]]`, `Table[[#Spec],[A]:[B]]`, `Table[Col]`, `Table[[A]:[B]]`, `Table[#Spec]`
or `Table[]`. Names that need escaping get Excel's `'` escapes (`'[`, `']`, `'#`, `''`, `'@`),
the inverse of `consume_escaped_until`. `ThisRowKind` is never printed on its own, because it
only appears inside a wrapper.

A bare table name (`ROWS(Sales)`, operands.rs) keeps printing as the bare name in these
printers; T-1683 may change how it is spelled. `to_excel_string` prints `lowered` until step 3,
and from then on writes a bare name as `Table[]` (see above).

`rename_sheet_in_node` and `move_formula` recurse into `lowered` as they do today. The printed
text comes from `reference`, so a sheet rename or a cut and paste leaves the text unchanged, and
the next reparse re-derives `lowered` from the table.

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
    `[#Data],[#Totals]`, none) and the column forms print back unchanged in `to_rc_format` and
    in `to_excel_string`.
  - Escaped column names round-trip.
  - ISREF, `@`, the `:` and space operators, and SUM work with `Sales[@Qty]` as with `$A$3`.
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
   `ungrouped()` transparent to it, wire the printers, and audit the 29 `ReferenceKind` sites.
   Every structured reference except `#This Row` emits the wrapper. `#This Row` keeps today's
   parse-time lowering, so its text is still lost. Export is unchanged. This step is complete on
   its own. If the diff gets too large, the printer escapes and their property test can be split
   into a second PR.
2. **T-1699 (S): `#This Row` at evaluation.** Add `ThisRowKind` and `Model::this_row_area`, and
   emit the wrapper for `#This Row` too. Re-audit the reference sites for `ThisRowKind`. Add the
   grow-the-table test and tests for `ISREF(Sales[@Qty])` and `SUM(Sales[@[A]:[B]])`.
3. **T-2196 (M): table parts in the export.** Write `<tableParts>` and `xl/tables/tableN.xml`,
   with the totals-row metadata xlsx-storage.md requires. Then switch `to_excel_string` to the
   preserved reference, with a bare name written as `Table[]`.
4. **T-1683** picks the FORMULATEXT spelling over the preserved node, and adds the lexer
   rejection of unterminated references.
5. **T-2197 (probe):** how Excel resizes, converts and deletes a table on row and column inserts
   and deletes, and what its structured references become. Implementing that in `insert_rows`
   and its siblings is filed from the probe's findings.
