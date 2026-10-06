---
id: T-0181
title: "Importer: cells and rows without an r attribute take the next position (3 poi !EngineError cases)"
milestone: M1
lane: engine
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
The xlsx importer accepts `<c>` cells and `<row>` rows that omit the optional `r` attribute, placing
each one in the next position (the column after the previous cell in the row, the row after the
previous row), as the eval snapshot reader already does. Three POI workbooks that fail to import
today then load.

## Context
Found by the gap triager on 2026-10-06 (poi suite, "Cases that did not run" in report.md). All three
fail with `EngineError: import failed: invalid workbook: XML Error: Missing "r" XML attribute`:
- `poi/54288.xlsx`: rows carry `r`, most cells do not (sheet1: 1,954 of 2,048 `<c>` have no `r`;
  row 3 is `<c r="C3" s="311"><v>41229</v></c><c s="96" />...`, so the second cell is D3).
- `poi/59746_NoRowNums.xlsx`: neither `<x:row>` nor `<x:c>` has `r`
  (`<x:row hidden="0" ...><x:c s="1" t="inlineStr"><x:is><x:t>Checked</x:t>...` is A1, then B1, ...).
- `poi/InlineString.xlsx`: one `<x:row>` with one `<x:c t="inlineStr">`, no `r` on either (A1).

Semantics: docs/excel-semantics/xlsx-storage.md "Cells and rows without r are positional". The eval
snapshot reader (`read_sheet_cells` / `start_cell` in `crates/rustsheet-eval/src/xlsx_snapshot.rs`)
already places r-less cells and rows this way: `rustsheet-eval snapshot` of 59746_NoRowNums.xlsx
lists A1 = "Checked", of InlineString.xlsx A1 = "😜more text", of 54288.xlsx C3 = 41229. None of the
three workbooks has formulas, so they add no scored cells; the fix is checked by import and unit tests.

Code: `start_cell` in `crates/rustsheet-xlsx/src/import/sheet_data.rs` returns the error when `r` is
absent and `end_cell` parses `cell.reference`; the row path errors with "Row without a row index (r
attribute)" when neither the row nor its first cell has `r`. Track the previous row index and the
next column in the sheet-data parser state, as the snapshot reader does (`row + 1`, `col + 1`), and
keep the existing bounds checks: a position past the grid (after XFD or row 1,048,576) is a typed
error, never a panic (CLAUDE.md directive 5). Run `target/release/rustsheet-eval fetch --suite poi`
first to get the workbooks.

## Acceptance criteria
- [ ] `F	poi/54288.xlsx	!EngineError`, `F	poi/59746_NoRowNums.xlsx	!EngineError` and
      `F	poi/InlineString.xlsx	!EngineError` disappear from eval/baseline/poi.txt
      (`make eval-baseline`), with no new F lines in any baseline.
- [ ] Importer unit tests: a cell without `r` after `C3` lands in D3; a first cell without `r` lands
      in column A; a row without `r` after row 5 is row 6; a first row without `r` is row 1; an
      r-less cell after XFD is an error, not a panic.
- [ ] The xlsx-storage.md entry above gets an `Implemented in:` line.
- [ ] `make check` green; reviewers approve.

## Out of scope
- `poi/47889.xlsx` (`Missing <cellXfs> section in styles.xml`) and the other import failures listed
  in report.md (missing rels and Strict OOXML: T-0112).

## Outcome
The sheet-data reader places r-less rows (previous row + 1) and cells (previous column + 1) positionally; out-of-grid positions are import errors. Unit tests in `import/sheet_data.rs`. poi baseline: 54288, 59746_NoRowNums and InlineString `!EngineError` lines gone, 690 -> 679 known failures (the baseline tool also dropped 5 other lines already fixed on main), 0 regressions.
