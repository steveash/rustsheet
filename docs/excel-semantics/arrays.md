# Arrays

## Cached rich errors (`#CALC!`, `#SPILL!`, `#BLOCKED!`, ...) are stored as `#VALUE!` plus a `vm` attribute
Rule: Excel writes a dynamic-array `#CALC!` (e.g. DROP/TAKE to nothing, `SEQUENCE(0,0)`, FILTER with no
match) or `#SPILL!` into the sheet XML as `t="e"` with `<v>#VALUE!</v>` and a `vm="N"` attribute. `vm`
indexes `xl/metadata.xml` `<valueMetadata>`; its `<rc v=K>` indexes `xl/richData/rdrichvalue.xml`. The
`errorType` field (13 = `#CALC!`, 8 = `#SPILL!`) must be found **by key name** in that value's
structure (`xl/richData/rdrichvaluestructure.xml`): desktop files put it first, but Excel for the web
writes the `_error` keys alphabetically (`colOffset, errorType, rwOffset, subType`), so there the
first field is `colOffset`. (The earlier "first field is the errorType" was wrong for those files;
T-0208, gen-m2/78bc88199fc96650 Calc!A61 rv `0 8 5 1` is `#SPILL!`.) A reader that ignores `vm`
sees `#VALUE!`. Every rich error is stored this way, not only the dynamic-array ones; errorType is
always ERROR.TYPE − 1 and `subType` (the reason) does not change the error:

| errorType | error | ERROR.TYPE | produced by (probe cell) |
|---|---|---|---|
| 8 | `#SPILL!` | 9 | `SEQUENCE(2)` blocked by a value (rich-errors-calc A1) |
| 9 | `#CONNECT!` | 10 | `STOCKHISTORY` of an unknown ticker (rich-errors-stockhistory A1, subType 4) |
| 10 | `#BLOCKED!` | 11 | `IMAGE` of an unreachable URL (rich-errors-image A1, subType 12); an uncached external reference (poi/link-external-workbook-b A1, subType 2) |
| 12 | `#FIELD!` | 13 | `FIELDVALUE` of a plain number (rich-errors-field A1; extra `Price` field) |
| 13 | `#CALC!` | 14 | FILTER with no match (rich-errors-calc A4, subType 3) |

A cell that only propagates the error (`ARRAYTOTEXT(A1)`) keeps the errorType in a different
`_error` structure, `errorType, propagated` (`propagated` = 1, a boolean; no subType). The
error names come from `VALUETOTEXT(cell)` (a text "#BLOCKED!" etc.), which also matches for
`#SPILL!`, `#CALC!` and `#DIV/0!`. `#BUSY!`, `#UNKNOWN!` and other codes (11, 14+) were not
produced (no formula was found that yields them in a recalculated, saved workbook; none was tried); the
harness reads unmapped codes, and a `vm` that resolves to no errorType, as `#UNDECODED!` and never
scores them.
Evidence: probes/rich-errors-calc, -field, -image, -stockhistory (excel-graph, 2026-10-05, T-0030:
ERROR.TYPE and VALUETOTEXT per cell, errorType/subType from the cached oracle workbooks);
ironcalc-calc/ERROR.TYPE.xlsx (A8 vm→13, ERROR.TYPE=14; A14 vm→8, ERROR.TYPE=9);
DROP_TAKE.xlsx C2/E2/F2/I2/L2, FILTER_ERRORS.xlsx E1, SEQUENCE.xlsx C16/E18 (all errorType 13, subType 3).
Status: confirmed from Excel-cached files. (An earlier theory that these were CSE `#VALUE!` results was
wrong: the cells are `cm="1"` dynamic arrays and the engine's `#CALC!` was already right.)
Implemented in: `read_rich_errors` in `crates/rustsheet-eval/src/xlsx_snapshot.rs` (eval expected values).

## NOT and CONCATENATE lift over array arguments
Rule: `NOT(array)` and `CONCATENATE(array, …)` return an array, one result per element, like the
`&` operator and TRIM. They do not return a scalar `#VALUE!`. When the result cannot spill (the
next cell is occupied) the cell shows `#SPILL!`. `CONCAT`, by contrast, joins a whole range into
one string.
Evidence (oracle: excel-graph, 2026-10-04, gen-m2 sidecars as decoded by T-0208; each argument is a
multi-cell column from `INDEX(range,0,…)` and the row below is occupied, so the cell is `#SPILL!`):
gen-m2/75840b9c71a50596 Calc!A29 `=NOT(INDEX(Data!B2:E8,Data!C$10,3))` (C10 = FALSE);
gen-m2/f7e6d351c8538ef5 Calc!A106 `=NOT(INDEX(Data!B3:E9,(Data!B$10-0.1),4))` (B10 = 0.2);
gen-m2/207a5753da48c7e0 Calc!A297 `=(AVERAGE(…)=CONCATENATE(INDEX(Data!C11:C12,0),("#N/A"&Data!D4)))`;
gen-m2/b4392a6514a36ade Calc!A348 `=TRIM(CONCATENATE(INDEX(Data!A7:D11,Data!A8,3)))` (A8 = 0.3).
Status: VERIFIED
Tasks: T-0225 (NOT), T-0226 (CONCATENATE)
Implemented in: `fn_concatenate` (crates/rustsheet-engine/src/functions/text/concatenate.rs) via `text_arg` + `broadcast_text` (T-0226).

## CONCATENATE with a scalar error argument returns that error, not an array
Rule: if any CONCATENATE argument is a scalar error, the result is that one error, even when another
argument is an array; Excel does not spill an array of errors.
Evidence: gen-m2/ce315bb699e1be6f Calc!A313
`=MID(CONCATENATE(INDEX(Data!B8:E11,1),"x~*",HLOOKUP(-0.5,Data!B2:E2,4)),Data!A$3,1)` gives `#REF!`, not `#SPILL!`;
gen-m2/207a5753da48c7e0 Calc!A95 (VLOOKUP over `CONCATENATE(INDEX(..,0.1),INDEX(Data!D3:D4,4,1))`) gives `#REF!`.
Status: VERIFIED
Implemented in: `fn_concatenate` (early return of the first argument error, T-0226)

## CONCATENATE result longer than 32767 characters
Rule: a CONCATENATE result (per element for arrays) longer than 32767 characters is `#VALUE!`; this also
bounds the allocation of array broadcasts (CLAUDE.md directive 5). Applies to scalar results too.
Evidence: none from the oracle (Microsoft documents the limit). Open: exactly 32767 vs 32768, per-element in
arrays, and whether Excel counts UTF-16 code units (the code counts `chars()`).
Status: UNVERIFIED (probe pending, T-0233)
Implemented in: `fn_concatenate` (T-0226)


## A scalar error operand broadcasts against an array operand
Rule: an arithmetic operator (`+ - * / ^`) whose one operand is a scalar error (or text that is
not a number) and whose other operand is an array or multi-cell range returns an array of that
operand's size, every element the scalar's error, not a scalar error: `NA()-{1;2;3}` is
`{#N/A;#N/A;#N/A}`, `"abc"*{1,2}` is `{#VALUE!,#VALUE!}`. Per element the leftmost error wins:
`INDEX({1;#DIV/0!;3}+NA(),2)` is `#DIV/0!`, `INDEX(NA()+{1;#DIV/0!;3},2)` is `#N/A`. So a cell such as
`=(…+COUNTIFS(<mismatched shapes>))-INDEX(Data!A5:C12,0,3)` is an 8-row array and shows `#SPILL!` when
the cells below are occupied.
Evidence (oracle: excel-graph, 2026-10-05): probes/error-operand-broadcast C2, C4, C5, C9, C11–C15
(`SUM(IFERROR(NA()-{1;2;3},1))` = 3); gen-m2/dd99be1f2454159f Calc!A56 (expected `#SPILL!`).
The same probe shows ROWS/COLUMNS accept an array expression (`ROWS(NA()-{1;2;3})` = 3, C1/C3/C6–C8/C10);
ROWS/COLUMNS count an array without reading its elements, so an all-error array still has its size.
A scalar non-reference argument counts as 1x1 (`ROWS(5)`, `COLUMNS(TRUE)`, `ROWS("a")`, `ROWS(1+1)` = 1) while an error argument passes through (`ROWS(NA())` = `#N/A`); array constants and `SEQUENCE` results count their dimensions (`ROWS({1;2;3})` = 3, `COLUMNS({1,2;3,4})` = 2, `ROWS(SEQUENCE(5))` = 5). Evidence: probes/rows-columns-args C1-C14 (T-0237).
Status: VERIFIED
Implemented in: `handle_arithmetic` in `crates/rustsheet-engine/src/arithmetic.rs`
Implemented in: `rows_columns_extent` in `crates/rustsheet-engine/src/cast.rs`
Tasks: T-0209, T-0228

## Array results above 1,000,000 cells are #ERROR!
Rule: Excel builds any array that fits the grid: `ROWS(SEQUENCE(1000,1001))` = 1000 and
`ROWS(SEQUENCE(1048576))` = 1048576; only a dimension past the grid is `#VALUE!`
(`SEQUENCE(1048577)`). RustSheet returns `#ERROR!` (not even an Excel error value) for SEQUENCE,
RANDARRAY, MAKEARRAY/BYROW/BYCOL results and matrix results above 1,000,000 cells, to bound
allocation on untrusted input (CLAUDE.md directive 5); past-the-grid dimensions are `#VALUE!` as in Excel.
Evidence: probes/resource-limits C12–C14 (excel-graph, 2026-10-05; C12 and C14 are known failures by design).
Status: DEVIATION (decision 0014)
Implemented in: functions/math_and_trigonometry/array_size.rs (`MAX_SIZE`, `check_array_size`)

## The edit API refuses a cell inside a spill extent above 4,194,304 cells
Rule: Excel lets you edit any cell. `Workbook` edits (crates/rustsheet) snapshot the spill footprint
of an array formula they touch; when that imported footprint is larger than 2^22 cells, the edit is
refused with `EditError::InvalidInput` ("spill extent too large to edit"), so an untrusted file cannot
make one edit walk the whole grid (CLAUDE.md directive 5).
Evidence: none for Excel; this is a resource limit, not an Excel observation.
Status: DEVIATION (decision 0014)
Implemented in: crates/rustsheet/src/edit.rs (`MAX_FOOTPRINT_CELLS`)

## MDETERM / MINVERSE: a blank cell in the matrix is #VALUE!
Rule: MDETERM and MINVERSE return `#VALUE!` when any cell of the matrix range is blank (MMULT
does the same); a blank does not count as 0. `MDETERM` of a single numeric cell is that number.
Evidence: poi/MatrixFormulaEvalTestData.xlsx (Excel-cached, Excel 16) Sheet1!H10 `MDETERM(B10:D12)`
and H30 `MDETERM(B30:D32)` (all blank / only B30 = 1) are `#VALUE!`; H14:J16 `MINVERSE(B14:D16)` and
H34:J36 `MINVERSE(B34:D36)` are `#VALUE!` in every cell; H50 `MDETERM(B50)` = 1.
Status: VERIFIED (Excel-cached)
Tasks: T-0943

## Legacy (CSE) array formula over a larger range
Rule: a legacy array formula entered over a range larger than its result fills the range: a 1x1
result is repeated into every cell, a single-row result is repeated down and a single-column result
across, and every other cell is `#N/A`.
Evidence: poi/MatrixFormulaEvalTestData.xlsx (Excel-cached, Excel 16) Sheet1: `TRANSPOSE(B46)` over
H46:J48 is 1 in all 9 cells (also `MINVERSE(B54)` H54:J56, `MMULT(B58,E58)` H58:J60);
`B62:C62+E62:F62` over H62:J64 repeats the row down with J62:J64 `#N/A`; `TRANSPOSE(B66:C66)` over
H66:J68 has row 68 `#N/A` (47 `#N/A` fill cells in H46:J136).
Status: VERIFIED (Excel-cached)
Tasks: T-0950
