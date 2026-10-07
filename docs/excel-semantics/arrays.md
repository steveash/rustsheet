# Arrays

## Cached rich errors (`#CALC!`, `#SPILL!`, `#BLOCKED!`, ...) are stored as `#VALUE!` plus a `vm` attribute
Rule: Excel writes a dynamic-array `#CALC!` (e.g. DROP/TAKE to nothing, `SEQUENCE(0,0)`, FILTER with no
match) or `#SPILL!` into the sheet XML as `t="e"` with `<v>#VALUE!</v>` and a `vm="N"` attribute. `vm`
is 1-based into the `<bk>` blocks of `xl/metadata.xml` `<valueMetadata>`; a block's `<rc t=T v=K>` names
metadata type T (1-based into `<metadataTypes>`, `XLRICHVALUE` here; files with dynamic arrays list
`XLDAPR` first, so T is 2 there) and block K (0-based) of `<futureMetadata name="XLRICHVALUE">`, whose
`<xlrd:rvb i=I>` indexes the `<rv>` values of `xl/richData/rdrichvalue.xml`. (K and I coincide in every
evidence file; the specification says to follow the `rvb` hop, and both readers do, T-0921.) The
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
| (none: formula literal) | `#GETTING_DATA` | 8 | the literal `=#GETTING_DATA` (error-literals-new S!A2); not a rich-value error. The Graph API reports its value as `#N/A` (S!A1) though `ERROR.TYPE` is 8, `VALUETOTEXT` is "#GETTING_DATA" (S!A6), `ISERR` is TRUE, `ISNA` is FALSE (S!A3, S!A4) |

The `#GETTING_DATA` row is a formula-literal error, not a rich-value error (no errorType).

Behavior of these errors under ISERR/ISNA/TYPE/IFNA/COUNTIF, and of IMAGE, STOCKHISTORY and FIELDVALUE with error
arguments (including `FIELDVALUE({1,2},"Price")` spilling two `#FIELD!` cells), is in information.md
(probes/rich-errors-args, T-1012). An IMAGE source that is empty or a number evaluates as `#VALUE!`; its stored rich-value form is undecoded (UNVERIFIED).

A cell that only propagates the error (`ARRAYTOTEXT(A1)`) keeps the errorType in a different
`_error` structure, `errorType, propagated` (`propagated` = 1, a boolean; no subType). The
error names come from `VALUETOTEXT(cell)` (a text "#BLOCKED!" etc.), which also matches for
`#SPILL!`, `#CALC!` and `#DIV/0!`. `#BUSY!`, `#UNKNOWN!` and other codes (11, 14+) were not
produced (no formula was found that yields them in a recalculated, saved workbook; typing them as formula literals fails, syntax.md "Newer error literals"); the
harness reads unmapped codes, and a `vm` that resolves to no errorType, as `#UNDECODED!` and never
scores them.
Evidence: probes/rich-errors-calc, -field, -image, -stockhistory (excel-graph, 2026-10-05, T-0030:
ERROR.TYPE and VALUETOTEXT per cell, errorType/subType from the cached oracle workbooks);
ironcalc-calc/ERROR.TYPE.xlsx (A8 vm→13, ERROR.TYPE=14; A14 vm→8, ERROR.TYPE=9);
DROP_TAKE.xlsx C2/E2/F2/I2/L2, FILTER_ERRORS.xlsx E1, SEQUENCE.xlsx C16/E18 (all errorType 13, subType 3).
Status: confirmed from Excel-cached files. (An earlier theory that these were CSE `#VALUE!` results was
wrong: the cells are `cm="1"` dynamic arrays and the engine's `#CALC!` was already right.)
Implemented in: `value_metadata_errors` in `crates/rustsheet-xlsx/src/import/rich_errors.rs`, shared by
the importer (cached `#CALC!`/`#SPILL!`; errors the engine has no kind for stay `#VALUE!`) and the eval
snapshot reader (expected values; T-0921).

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
Element values (probes/concatenate-limit-spill, excel-graph, 2026-10-06, T-0233): arguments
broadcast like the arithmetic operators, so a row and a column give a grid
(`CONCATENATE({"a","b"},{"x";"y"})` is {"ax","bx";"ay","by"}, `CONCATENATE(A1:A2,B1:C1)` likewise),
a single-row or single-column argument stretches, and positions past a shorter argument are `#N/A`
(`CONCATENATE({"a";"b";"c"},{"x";"y"})` is {"ax";"by";#N/A}; same for ranges, `A1:A3` with `B1:B2`).
An error element stays in its position (`CONCATENATE({1;2},{#N/A;"x"})` is {#N/A;"2x"}); numbers and
booleans in arrays become text as in `&` ({1;TRUE;FALSE;1.5} gives "1-","TRUE-","FALSE-","1.5-";
1E+20 gives "1E+20", 0.000001 gives "0.000001"); blank cells are "". A 1×1 array argument gives a single-cell result.
Status: VERIFIED (CONCATENATE array shape, #SPILL! and element values)
NOT status: VERIFIED for the array shape and #SPILL! (NOT returns an array). ASSUMED (UNVERIFIED), pending probe T-0260: the per-element cast
(text, empty and error elements follow the scalar `cast_to_bool` rules); the oracle cells never showed an element value.
Tasks: T-0225 (NOT), T-0226 (CONCATENATE), T-0233 (element values), T-0260 (NOT per-element cast probe)
Implemented in: `fn_concatenate` (crates/rustsheet-engine/src/functions/text/concatenate.rs) via `text_arg` + `broadcast_text` (T-0226); `fn_not` (`functions/logical/and_or_xor_not.rs`) and its `scalar_arguments` static-analysis entry (T-0225).

## CONCATENATE with a scalar error argument returns that error, not an array
Rule: if any CONCATENATE argument is a scalar error, the result is that one error, even when another
argument is an array; Excel does not spill an array of errors.
Evidence: gen-m2/ce315bb699e1be6f Calc!A313
`=MID(CONCATENATE(INDEX(Data!B8:E11,1),"x~*",HLOOKUP(-0.5,Data!B2:E2,4)),Data!A$3,1)` gives `#REF!`, not `#SPILL!`;
gen-m2/207a5753da48c7e0 Calc!A95 (VLOOKUP over `CONCATENATE(INDEX(..,0.1),INDEX(Data!D3:D4,4,1))`) gives `#REF!`.
Status: VERIFIED
Implemented in: `fn_concatenate` (early return of the first argument error, T-0226)

## CONCATENATE result longer than 32767 characters
Superseded: Excel truncates rather than returning `#VALUE!`; see coercion.md "Text longer than 32,767 characters".


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
The comparison operators (`= <> < > <= >=`) and `&` follow the same rule: `NA()={1;2;3}` is a 3-row array of `#N/A`
(`SUM(IFERROR(NA()={1;2;3},1))` = 3, also `{1;2;3}<=NA()`, `NA()>A1:A3`, `NA()&{"a";"b"}`, `1/0>={1,2,3,4}` is 4 columns), and per element the
leftmost error wins: `INDEX({1;#DIV/0!;3}=NA(),2)` is `#DIV/0!`, `INDEX(NA()<>{1;#DIV/0!;3},2)` is `#N/A`,
`INDEX({"a";#DIV/0!}&NA(),2)` is `#DIV/0!`, `INDEX(NA()&{"a";#DIV/0!},2)` is `#N/A`.
Evidence (oracle: excel-graph, 2026-10-07): probes/error-operand-broadcast-cmp C1-C15 (T-0231).
Status: VERIFIED
Implemented in: `handle_arithmetic`, `handle_comparison`, `handle_concatenate` in `crates/rustsheet-engine/src/arithmetic.rs`
Implemented in: `rows_columns_extent` in `crates/rustsheet-engine/src/cast.rs`
Tasks: T-0209, T-0228, T-0231

## Errors broadcast through `&`, comparisons, POWER and the element-wise functions
Rule: the scalar-error broadcast above holds for every element-wise operation, not only arithmetic:
`&` (`A1:C1&1/0`), the comparisons (`A1:C1=1/0`, `1/0<A1:C1`), POWER, LEFT/RIGHT/MID, EXACT, TEXT
(also the format argument: `TEXT(A1:C1,NA())`), DATE, WEEKDAY, REPT, FIND, SUBSTITUTE and REPLACE all
give an array of errors of the array's size. Per element the leftmost error wins, including an
error element of the array itself (`A5:C5&1/0` keeps `#N/A` where A5:C5 holds it). A comparison keeps
an error element instead of turning it into TRUE/FALSE (`{1,#DIV/0!,3}>=3` is `{FALSE,#DIV/0!,TRUE}`),
so `SUM((A6:C6=1)*1)` over a range holding an error is that error. Where two arrays of different
sizes meet, the positions only one covers are `#N/A`, not `#VALUE!` (`A6:C6+{1,2}`). With no array
argument the leftmost error is the result: `WEEKDAY(1/0,NA())` is `#DIV/0!`, `TEXT("x",NA())` is
`#N/A` (the format's error wins over a text value).
Evidence: crates/rustsheet-xlsx/tests/calc_tests/INFORMATION/error_broadcast.xlsx (Excel-authored,
upstream IronCalc 4deab8f6; 230 cells, all pass).
Status: VERIFIED (probe `eval/suites/probes/cases/error-precedence-text`, T-0994). WEEKDAY with an
error return type: a text element of the date array is `#VALUE!` (`{"x",1,TRUE,""}` gives
`#VALUE!,#N/A,#N/A,#VALUE!`; `WEEKDAY("x",NA())` is `#VALUE!`), while a blank range cell counts as 0
and takes the return type's error (`#N/A`). Text functions coerce arguments left to right and the first
failing argument decides, an error argument and a failed text-to-number coercion alike, in the scalar and
the array form: `MID("abc","x",NA())` and `MID({"abc"},"x",NA())` are both `#VALUE!` (the "x" start is
reached before the error), `MID("abc",NA(),"x")` and `MID("x",1,NA())` are `#N/A`, `LEFT("abc","x")`,
`RIGHT`, `REPT("a","x")` and `FIND("a","abc","x")` are `#VALUE!`, and `REPT({"a"},NA())` is `#N/A`.
RustSheet matches (T-1022: `broadcast_text` takes an error argument as the result only when it is the first failure). TRUE in a range
behaves like TRUE in an array (`#N/A`), and `WEEKDAY(NA(),"x")` is `#N/A` (left to right).
Probe cells: probes/error-precedence-text Calc!C1-C30 (oracle: excel-graph, 2026-10-06).
An error argument also beats a later range failure (start < 1, negative length, text not found): `REPLACE("abc",0,NA(),"x")`,
`REPLACE("abc",1,-1,NA())`, `REPLACE("abc",0,1,NA())`, `MID("abc",0,NA())`, `MID("abc",NA(),-1)` and
`FIND("z","abc",NA())` are all `#N/A`, while the same calls with a number in place of the error are `#VALUE!`
(`MID("abc",1,-1)`, `FIND("z","abc")`). So every argument is coerced before any range check
(probes/error-arg-vs-range-check A1-A10, oracle: excel-graph, T-1056); `replace_element` matches since T-1056.
Implemented in: `array_of_errors` and `MISSING` (crates/rustsheet-engine/src/arithmetic.rs),
`TextArg::Error` (crates/rustsheet-engine/src/functions/text/text_arg.rs), `fn_date`, `fn_weekday`.
Tasks: T-0129, T-0994, T-1022, T-1056

## Array results above 1,048,576 cells are #ERROR!
Rule: Excel builds any array that fits the grid: `ROWS(SEQUENCE(1000,1001))` = 1000,
`ROWS(SEQUENCE(1048576))` = 1048576, and a range lifted element-wise by a scalar function becomes
an array of the range's size however large: `ROWS(DATE(11:1899,12,1))` = 1889 and
`COLUMNS(DATE(11:1899,12,1))` = 16384 (31M cells), `INDEX(DATE(11:1899,12,1),5,7)` = 336;
`=DATE(11:1899,12,1)` in C3 is `#SPILL!` (no room). A whole column lifts like any range:
with A1 = -2, A2 = 3, `ROWS(ABS(A:A))` = `ROWS(A:A+0)` = 1048576, `SUM(ABS(A:A))` = 5,
`INDEX(ABS(A:A),2)` = 3, `SUM(--ISNUMBER(A:A))` = 2, and `=ABS(A:A)` in C7 is `#SPILL!`.
Only a dimension past the grid is `#VALUE!` (`SEQUENCE(1048577)`).
RustSheet returns `#ERROR!` (not even an Excel error value) for any array above 1,048,576 cells
(one whole column): SEQUENCE, RANDARRAY, MAKEARRAY/BYROW/BYCOL results, matrix results, an *IF(S)
criteria grid (broadcast criteria included), and every range lifted into an array (`Model::evaluate_range`: scalar functions
over a range, operators, IF/IFERROR, TEXT, IS*, NOT, a bare range spilled from a cell), checked
before any cell is read, to bound allocation on untrusted input (CLAUDE.md directive 5); so
`DATE(11:1899,12,1)` and `ROWS(DATE(11:1899,12,1))` are `#ERROR!`. Past-the-grid dimensions are
`#VALUE!` as in Excel. Excel for the web refuses to open a workbook whose formula sums such a
31M-cell array (`SUM(DATE(11:1899,12,1))`: Graph `SessionMemoryQuotaExceededOnOpenWorkbook`).
Evidence: probes/resource-limits C12–C14 (excel-graph, 2026-10-05); probes/lifted-range-size
C4–C8, C10 and probes/lifted-range-rows C1–C3, E1 (excel-graph, 2026-10-06; rows C1–C3 and E1 are known
failures by design).
Status: DEVIATION (decision 0014)
Implemented in: functions/math_and_trigonometry/array_size.rs (`MAX_SIZE`, `check_array_size`,
`check_range_size`); `Model::evaluate_range` in crates/rustsheet-engine/src/cast.rs
Tasks: T-0047

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
Implemented in: `coerce_to_f64` (crates/rustsheet-engine/src/functions/math_and_trigonometry/matrix.rs, T-0943)
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

## FILTER: an array `if_empty` spills as an array
Rule: when FILTER finds no match, an array-constant `if_empty` is returned whole and spills like any
array: `FILTER(L1:L2,M1:M2,{1,2,3})` fills three cells across, `{1;2;3}` three down, `{1,2;3,4}` a 2x2
block, `SUM(..)` of it is 6. A blocked spill area (a value in the neighbouring cell) gives `#SPILL!`.
Evidence: probes/filter-if-empty-array (oracle: excel-graph, 2026-10-06); upstream ISREF.xlsx D9 (`#SPILL!`),
J17:L17 (spilled 1,2,3).
Status: VERIFIED (web)
Implemented in: `fn_filter` (crates/rustsheet-engine/src/functions/spill_functions.rs)
Tasks: T-0964
