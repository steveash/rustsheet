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

Writing them (T-1014): Excel reads a constant rich error written in this form with one `_error` value per
kind and `subType` 0 for every kind, in a workbook whose metadata lists `XLDAPR` first and `XLRICHVALUE`
second, with `rdRichValueTypes.xml` as Excel writes it: fixtures/rich-error-export.xlsx (our exporter's
output; excel-graph, 2026-10-08) gives ERROR.TYPE 9, 10, 11, 13, 14 and VALUETOTEXT the names.
The same workbook with A1 stored as the plain literal `<c t="e"><v>#SPILL!</v></c>` instead is rejected on load
(createSession 501 `unsupportedWorkbook`, excel-graph 2026-10-08, T-1014), like the other rich kinds (syntax.md).

Behavior of these errors under ISERR/ISNA/TYPE/IFNA/COUNTIF, and of IMAGE, STOCKHISTORY and FIELDVALUE with error
arguments (including `FIELDVALUE({1,2},"Price")` spilling two `#FIELD!` cells), is in information.md
(probes/rich-errors-args, T-1012). An IMAGE source that is empty, a number, a boolean or non-URL text evaluates as `#VALUE!` (probes/rich-errors-arg-kinds, information.md); its stored rich-value form is undecoded (UNVERIFIED).

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
the importer (cached `#CALC!`/`#SPILL!`/`#CONNECT!`/`#BLOCKED!`/`#FIELD!`; unmapped errorTypes stay `#VALUE!`) and the eval
snapshot reader (expected values; T-0921).

## MOD lifts over array arguments
Rule: `MOD(array, n)` returns an array, one `MOD` per element, so `SUM(MOD(SEQUENCE(4),2))` is 2. The
two arguments broadcast like the arithmetic operators (a size-1 dimension repeats, positions past a
shorter array are `#N/A`); the zero divisor (`#DIV/0!`) and huge quotient (`#NUM!`) rules hold per
element, and an error element stays in its position.
Evidence: probes/geomean-overflow-cut-3 E4 and probes/mod-array-lift (excel-graph, 2026-10-07):
ranges, a per-element `#DIV/0!`, an `#N/A` element staying in place, a scalar divisor repeating, `#N/A`
past the end of a shorter divisor, `MOD(NA(),{1,2})` giving `#N/A` in every element, row x column
broadcast, text elements cast like a scalar (`"x"` gives `#VALUE!`) and booleans as 1/0.
Status: VERIFIED
Tasks: T-1119
Implemented in: `Model::lift_numbers` (DATE uses it too) (crates/rustsheet-engine/src/cast.rs), used by `fn_mod`.

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
NOT status: VERIFIED for the array shape and #SPILL! (NOT returns an array), and VERIFIED for the per-element cast:
each element is cast exactly like a scalar argument. Text "TRUE" and "false" are booleans (other casings not probed), so NOT gives
FALSE/TRUE; other text ("abc", "1", " TRUE", "" and a formula's "") is `#VALUE!`; a number is FALSE when non-zero;
an error stays; an empty cell in a range is FALSE, so NOT gives TRUE. A range and an array literal give the same
element values (`NOT({"TRUE","abc","1",#N/A,0,2,"false"," TRUE",""})` is FALSE, `#VALUE!`, `#VALUE!`, `#N/A`, TRUE,
FALSE, TRUE, `#VALUE!`, `#VALUE!`), and `NOT({""})` is `#VALUE!`. Evidence: probes/not-array-cast (38 cells, the scalar
`NOT(cell)` and `NOT("text")` forms alongside; excel-graph, 2026-10-09, T-0260); the engine already matched every cell.
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
(`MID("abc",1,-1)`, `FIND("z","abc")`). So REPLACE, MID and FIND coerce every argument before any range check (not probed for other text functions)
(probes/error-arg-vs-range-check A1-A10, oracle: excel-graph, T-1056); `replace_element` matches since T-1056.
Implemented in: `array_of_errors` and `MISSING` (crates/rustsheet-engine/src/arithmetic.rs),
`TextArg::Error` (crates/rustsheet-engine/src/functions/text/text_arg.rs), `fn_date`, `fn_weekday`.
Tasks: T-0129, T-0994, T-1022, T-1056

## CEILING/FLOOR family and MROUND lift over arrays
Rule: ISO.CEILING, CEILING, CEILING.MATH, CEILING.PRECISE, FLOOR, FLOOR.MATH, FLOOR.PRECISE and MROUND lift element-wise over an array or multi-cell range in every argument, except MROUND (below). An error in `number` still broadcasts over an array in another argument (`CEILING.MATH(<error>,sig,{9x1 mode})` is a 9x1 array, `#SPILL!` where it cannot spill).
Evidence: gen-m7/e5a361112d83d0c3 Calc!A9 (ISO.CEILING, excel-graph 2026-10-11); gen-m9/89693ab0cafc29df Calc!A14 (CEILING.MATH, excel-graph 2026-10-12); probes/ceiling-floor-mround-array-lift A1:A18 (CEILING, FLOOR, CEILING.PRECISE, FLOOR.PRECISE, FLOOR.MATH, CEILING.MATH and MROUND with an array in each argument; excel-graph 2026-10-15, T-1589). The family reads a boolean element as 1/0 (`CEILING({TRUE,2},1)` spills `1, 2`, `FLOOR(5,{TRUE,2})` spills `5, 4`, A15:A18).
MROUND keeps its scalar rules per element: a boolean element or cell is `#VALUE!`, not 1/0 (`MROUND({TRUE,2.6},1)` spills `#VALUE!, 3`, `MROUND(1,{TRUE,2})` spills `#VALUE!, 2`, `MROUND(INDIRECT("K1"),1)` and `MROUND(TRUE,1)` are `#VALUE!`; A19:A25). MROUND judges its two arguments left to right: an error, a missing argument (`#N/A`) or a boolean (`#VALUE!`) in `number` wins over whatever `multiple` holds, so `MROUND(TRUE,)` is `#VALUE!` but `MROUND(,TRUE)` is `#N/A` (A27:A28), `MROUND(1/0,TRUE)` and `MROUND(1/0,)` are `#DIV/0!`, `MROUND(,1/0)` is `#N/A`, `MROUND(TRUE,1/0)` is `#VALUE!`, `MROUND("x",)` is `#VALUE!` but `MROUND(,"x")` is `#N/A` (A23, A27:A42); an error in `number` still broadcasts over an array `multiple` (`MROUND(1/0,{TRUE,2})` spills `#DIV/0!, #DIV/0!`, A22). MROUND is an Analysis ToolPak function and does not lift over a multi-cell range in either argument: `MROUND(M1:N1,1)`, `MROUND(1,M1:N1)`, `MROUND(M1:M3,1)` and `MROUND(M1:N1,M1:N1)` over plain numbers are one `#VALUE!` with no spill (`COLUMNS(MROUND(M1:N1,1))` is `#VALUE!`, A43:A49, A55), while an array constant lifts (`COLUMNS(MROUND({1.2,2.6},1))` is 2, A50) and so do CEILING and FLOOR.MATH over the same range (A51:A52, A30:A31); same rule as ISEVEN (information.md). Unprobed: an error argument next to a multi-cell range. `MROUND("3",)` is `#N/A` and `MROUND("3",2)` is 4 (A53:A54). The scalar rule "every missing argument before any boolean" the engine had was wrong for `MROUND(TRUE,)`.
Status: VERIFIED
Implemented in: functions/math_and_trigonometry/mathematical.rs (`lift_with_defaults`, `lift_numbers`)
Tasks: T-1357, T-1589

## ROUND, ROUNDUP, ROUNDDOWN and TRUNC lift over arrays
Rule: `ROUNDDOWN(array, num_digits)` returns an array, element-wise, with either argument an array; an error in either argument broadcasts (`ROUND(A1:B1,1/0)` and `ROUND(1/0,A1:B1)` are `#DIV/0!`).
Evidence: gen-m2/893302d6c5b99e2e Calc!A313; probes/round-concat-array-lift (excel-graph, 2026-10-05).
Status: VERIFIED
Implemented in: functions/math_and_trigonometry/mathematical.rs (`round_family`), cast.rs (`lift_numbers`)
Tasks: T-0222, T-1400

Non-number elements (probes/round-concat-array-edges A1-A7, excel-graph 2026-10-13, T-1400): the elements are coerced as scalar arguments are. `ROUND({TRUE,"2.5",""},0)` and `ROUNDUP(...)` spill `1, 3, #VALUE!` (also the column form `{TRUE;"2.5";""}`), `ROUNDDOWN(...)` spills `1, 2, #VALUE!` (numeric text is read as a number, a logical as 1, the empty string is `#VALUE!`). `ROUND({"abc"},0)` is `#VALUE!`, and `ROUND({"abc",1.5},0)` spills `#VALUE!, 2` (a bad element is an error in its own cell only). The same coercion applies to num_digits: `ROUND(2.567,{TRUE,"1",""})` spills `2.6, 2.6, #VALUE!`. The engine matches (`lift_numbers`).

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
criteria grid (broadcast criteria included), a GROUPBY result (groups x columns, checked while grouping, before any
LAMBDA call; T-1322), the broadcast result of `lift_numbers` (ROUND family, MOD, DATE), a lifted INDEX's broadcast indices (T-1260), the broadcast of a binary operator over two arrays
(arithmetic, `&` and comparisons: a whole column against a whole row, `=D:D+4:4`, would be 2^34
cells, and even a whole column against two columns, `SUM(A1:A1048576*B1:C1)`, is 2^21; T-1420), the broadcast of IF, IFERROR and IFNA (`=IF(D:D,4:4)`, `=IFERROR(D:D,4:4)`; T-1448), and every range lifted into an array (`Model::evaluate_range`: scalar functions
over a range, operators, IF/IFERROR, TEXT, IS*, NOT, a bare range spilled from a cell), checked
before any cell is read, to bound allocation on untrusted input (CLAUDE.md directive 5); so
`DATE(11:1899,12,1)` and `ROWS(DATE(11:1899,12,1))` are `#ERROR!`. Past-the-grid dimensions are
`#VALUE!` as in Excel. A cached range of a linked workbook (`[1]Sheet!A1:B2`, T-1460) is read as
an array of its cached cells under the same bound. An aggregate that skips blanks (SUM, COUNT,
AVERAGE, AVERAGEA, MIN, MAX, PRODUCT, the other `for_each_value` statistics, SUBTOTAL, AGGREGATE's reference
form, RANK) first cuts any area to the rows and columns the cache spans, which changes none of its
results, so `SUM([1]Uses!$A$2:$B$1048576)` adds the cached cells (T-1550). Every other reader keeps
the written shape with its blanks (only a whole column or row is cut to the cache): TEXTJOIN, BYROW,
BYCOL, a LET variable, lookups, LARGE/SMALL and value contexts, so there an area above
1,048,576 cells that is not a whole column or row (`TEXTJOIN(",",TRUE,[1]Uses!$A$2:$B$1048576)`,
`VLOOKUP(x,[1]Uses!$A$1:$C$1048576,2,0)`) is `#ERROR!`. The cells an aggregate reads from a union
of cached areas are capped together, so `SUM(([1]Uses!$A:$A,…))` over eight whole columns of a
cache spanning A1:A1048576 is `#VALUE!` (test_external_links.rs
`union_of_sparse_whole_columns_is_bounded`); Excel gives the value.
Excel for the web refuses to open a workbook whose formula sums such a
31M-cell array (`SUM(DATE(11:1899,12,1))`: Graph `SessionMemoryQuotaExceededOnOpenWorkbook`).
Evidence: probes/resource-limits C12–C14 (excel-graph, 2026-10-05); probes/lifted-range-size
C4–C8, C10 and probes/lifted-range-rows C1–C3, E1 (excel-graph, 2026-10-06; rows C1–C3 and E1 are listed in
eval/suites/probes/deviations, decision 0036).
Status: DEVIATION (decision 0014)
Implemented in: limits.rs (`MAX_SIZE`, `check_array_size`,
`check_range_size`); `Model::evaluate_range` in crates/rustsheet-engine/src/cast.rs;
`broadcast_arrays` in crates/rustsheet-engine/src/arithmetic.rs; `ExternalArea::blocked_value` in
crates/rustsheet-engine/src/external_reference.rs (a spilled range into another workbook, T-1454);
`Model::external_area_value` and `Model::external_aggregate_value` in
crates/rustsheet-engine/src/external_links.rs (a cached range of a linked workbook, T-1460)
Tasks: T-0047, T-1420, T-1454, T-1460

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
does the same); a blank does not count as 0. `MDETERM` of a single numeric cell is that number. A boolean is not a
number either: MDETERM, MINVERSE and MMULT give `#VALUE!` for a boolean cell (`MINVERSE(E1)`, E1 = FALSE;
gen-m7/7c48e2456500644f Calc!A19, T-1275), a boolean value (`MDETERM(TRUE)`, `MINVERSE(FALSE)`), and a boolean
element of a larger matrix (`MDETERM({TRUE,2;3,4})`, `MMULT(D1:E2,...)` with D1 = TRUE)
(probes/matrix-boolean-elements G1-G13, excel-graph 2026-10-15, T-1358).
Evidence: poi/MatrixFormulaEvalTestData.xlsx (Excel-cached, Excel 16) Sheet1!H10 `MDETERM(B10:D12)`
and H30 `MDETERM(B30:D32)` (all blank / only B30 = 1) are `#VALUE!`; H14:J16 `MINVERSE(B14:D16)` and
H34:J36 `MINVERSE(B34:D36)` are `#VALUE!` in every cell; H50 `MDETERM(B50)` = 1.
Implemented in: `coerce_to_f64` (crates/rustsheet-engine/src/functions/math_and_trigonometry/matrix.rs, T-0943)
Status: VERIFIED (Excel-cached; the boolean rule excel-graph)
Tasks: T-0943, T-1358

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

## Dynamic-array and LAMBDA function argument modes
Rule: the scalar arguments of the M7 functions lift over a multi-cell range and over an array, one call per element:
SEQUENCE's rows (`SEQUENCE(A1:A3)` and `SEQUENCE({1;2;3})` are 3x1, first element 1) and start
(`SEQUENCE(2,2,{1;10})` is 2x1), the rows of TAKE (`TAKE(A1:B3,A1:A2)` and `TAKE(A1:B3,{1;2})` are 2x1), DROP (2x1),
EXPAND (`EXPAND(A1:A2,{3;4})` 2x1, first 1) and MAKEARRAY
(`MAKEARRAY(A1:A2,2,LAMBDA(r,c,r+c))` 2x1, first 2), WRAPROWS's wrap_count, WRAPCOLS's pad_with, TOCOL's ignore,
UNIQUE's by_col, SORTBY's sort_order (`SORTBY(A1:A3,B1:B3,{1;-1})` is 2x1), TEXTBEFORE's instance_num, the text of
TEXTSPLIT, TEXTBEFORE, TEXTAFTER, REGEXTEST, REGEXEXTRACT and REGEXREPLACE over a range (`TEXTSPLIT(C1:C3,",")` with
C1:C3 = "a,b", "c-d", "e,f" is 3x1, first "a") and REGEX patterns over an array (`REGEXTEST("abc",{"a";"z"})` is 2x1).
A lifted TAKE or DROP keeps each call's whole result as the element: `INDEX(TAKE(A1:B3,{1;2}),1,1)` is the 1x2 row
{1,10} (it spills; ROWS 1, COLUMNS 2) and `INDEX(DROP(A1:B3,{1;2}),1,1)` the 2x2 block {2,20;3,30}. The other lifts
keep one value per element: INDEX of a lifted EXPAND, WRAPROWS, TOCOL, SEQUENCE or MAKEARRAY element has 1 row and 1
column, although `EXPAND(A1:A2,3)` and `WRAPROWS(A1:A3,1)` alone are 3x1.
These take an array whole: the delimiters of TEXTSPLIT, TEXTBEFORE and TEXTAFTER are a list of delimiters
(`TEXTSPLIT("a,b-c",{",";"-"})` is 1x3; `TEXTAFTER(C1:C3,{",";"-"})` is 3x1 "b", "d", "f" and
`TEXTBEFORE(C1:C3,{",";"-"})` "a", "c", "e", not broadcast against the text), SORT's sort_index is a
list of sort keys (`SORT(A1:B3,{1;2})` is 3x2) and its sort_order must have as many entries
(`SORT(A1:B3,1,{1;-1})` is `#VALUE!`), and FILTER's if_empty (2x1, first 9; see "FILTER: an array `if_empty`").
A function passed by name (eta-reduced, stored `_xleta.SUM`) works wherever a LAMBDA does and gets as many
arguments as the LAMBDA would: `BYROW(A1:B3,SUM)` = `BYROW(A1:B3,LAMBDA(r,SUM(r)))` (3x1, first 11),
`GROUPBY(D1:D3,A1:A3,SUM)` (3x2, first "x": two groups and a total row), `BYCOL(A1:B3,MAX)` 1x2, first 3,
`SCAN(0,A1:A3,MAX)` 3x1, `MAKEARRAY(2,2,SUM)` 2x2, first 2 (SUM(row,col)). REDUCE's initial_value may be left
empty (`REDUCE(,A1:A3,LAMBDA(a,v,a+v))` = 6). An uncalled `LAMBDA(x,x)` is one value (`ROWS` 1, `INDEX(..,1,1)`
`#CALC!`); a BYROW LAMBDA that returns a row is `#CALC!`; `ISOMITTED(A1)` outside a LAMBDA is FALSE;
`REGEXTEST(A1,1)` with A1 = 1 is TRUE (both numbers read as text); `TOCOL(A1:E1,"1")` reads "1" as ignore = 1 (4 rows past the blank E1, ignore 0 gives 5);
`SEQUENCE(0)` and the mismatched `SORTBY(A1:A3,B1:B2)` are `#CALC!` and `#VALUE!`.
Evidence: probes/m7-argument-modes (68 questions, excel-graph, 2026-10-09, T-1083). Three questions are left out because
excel-graph returns an error it cannot decode: `GROUPBY(D1:D3,A1:A2,SUM)` (heights differ), `MAP(A1:A3,LAMBDA(x,y,x))`
(too many parameters) and `GROUPBY(D1:D3,A1:A3,{1;2;3})` (an array as the function). A LAMBDA with an optional
parameter: `LAMBDA(x,[y],ISOMITTED(y))(1)` is TRUE, `(1,2)` FALSE, and an empty argument `(1,)` is omitted too (TRUE);
an omitted parameter's value is 0 (`LAMBDA(x,[y],y)(1)` = 0) and "" as text (`y&"|"` = "|");
`LAMBDA(x,[y],IF(ISOMITTED(y),x,x+y))` gives 1 for `(1)` and 3 for `(1,2)`; `LAMBDA([x],[y],ISOMITTED(x)+ISOMITTED(y))()`
is 2 (probes/lambda-optional-params A1-A8, excel-graph, 2026-10-11, T-1274; the engine matches all of them). The
case builder stores `[y]` as `_xlop.y` (xlsx-storage.md).
Ours: none of the lifts above (`#VALUE!` for a range, `#N/IMPL!` for an array; `SEQUENCE(A1:A3)` is one row);
PIVOTBY is `#NAME?` and GROUPBY exists since T-1322 ("GROUPBY and PIVOTBY" below; `_xleta.` names work since T-1466, see "Functions passed by name" below); INDEX of a scalar result is `#VALUE!` (T-1242); `ROWS(LAMBDA(x,x))`
is `#VALUE!`; a BYROW row result is not `#CALC!`; `REGEXTEST(A1,1)` is `#VALUE!`. 60 of the 174 scored cells pass.
Status: VERIFIED (web)
Tasks: T-1083 (generator modes)

## GROUPBY and PIVOTBY
Rule: groups are keyed like UNIQUE (case-insensitive text, first spelling shown) and always sorted ascending:
numbers < text < FALSE < TRUE < blank, with the blank-key group shown as the number 0 and kept last even
when sorting descending. Function inputs keep source order, and the total row applies the function to all
(filtered) values, not to the group results (AVERAGE total 13.05 = 130.5/10). Filler cells of total rows, the
corner and PIVOTBY combinations without records are empty text `""`, not blanks (a combination summing to 0
shows 0). Labels: "Total"; with subtotals (depth 2), the grand total is "Grand Total".
field_headers: omitted auto-detects headers without showing them; 0 none, 1 yes/hidden, 2 generated
("Row Field 1", "Value 1"), 3 yes/shown (over data without a header row it consumes the first record);
4 is `#VALUE!`. A LAMBDA returning an array gives `#VALUE!` per cell. `HSTACK(SUM,AVERAGE)` adds columns
and a header row of function names; `VSTACK(SUM,MAX)` repeats each group row per function with a
name column. PIVOTBY relative_to: 0 column total, 1 row total, 2 grand total; with one field per axis 3
equals 1 and 4 equals 0. Out-of-range sort_order, a filter of another height, an all-FALSE filter and
field_relationship 2 are `#VALUE!`.
T-1328 follow-up (probes/groupby-pivotby-followup, -followup2, -followup3; spills paired with ROWS/ISERROR/ERROR.TYPE so
excel-graph's undecodable errors are still answered): values of another height than the keys, total_depth 3 or -3 (and 2 with a
single key) and field_relationship 1 with total_depth ±2 are all `#VALUE!`; the all-FALSE filter is `#VALUE!` (never `#CALC!`),
also scalar `FALSE`/`0`/`{FALSE}`, and a scalar `TRUE` filters nothing. PIVOTBY with two value columns: there are two Total
columns (one per value), both `""` in every row (P!Y97 confirmed). Multi-key sort arrays are signed key-column indices whose priority is the column order
(`{2,-1}` = `{-1,2}`), duplicates and 0 are `#VALUE!`. Header auto-detect looks at the first row of the values only: text over
a non-text second row is a header; a first value cell of text over rows that mix text and non-text is `#VALUE!` (otherwise mixed values aggregate, text ignored). Number keys compare exactly (0.1+0.2 and
0.3 differ; 0, -0, computed 0 equal; blank is its own group shown as 0); equal errors share a group, sorted after numeric keys (against text/boolean/blank unprobed).
An array field_relationship (e.g. a filter-shaped column in argument 8) is lifted per element (10x1 of the group result; `{FALSE}` 1x1; a
numeric column 10x1 `#VALUE!`; scalar FALSE/0 act as 0) (followup G!A861-A981, followup2 G!A1-A161). With two keys, sort_order 3 (the value
column) orders parents by their aggregate (sum, max and mean not separated) and children by value within each parent, and the blank parent is not pinned last (followup2 G!A261).
Evidence: probes/groupby-pivotby (61 formulas over sheets G and P, excel-graph, 2026-10-11, T-1276);
every anchor is listed in docs/specs/SPEC-0006-groupby-pivotby.md.
T-1322 follow-up (probes/groupby-lambda-forms and probes/groupby-blank-values, excel-graph, 2026-10-16): the LAMBDA forms
of the sheet G anchors give the eta forms' results. The function gets each group's values as an array, not a reference
(`LAMBDA(x,ISREF(x))` is FALSE), with blank cells as true blanks: `ISBLANK(INDEX(x,1,1))` is TRUE, TYPE 1, LEN 0, and
COUNTA counts them (the blank-value group's COUNTA is 1), ARRAYTOTEXT shows them as nothing (`"2, b, TRUE, 1, a, FALSE, 2, , 1.5, B"`).
A LAMBDA whose body is a one-cell reference (`LAMBDA(x,Data!D2)`) is `#VALUE!` in every cell, like an array result; so is a
two-parameter LAMBDA (each cell, not the whole result). `field_headers` 2 over two keys and two values generates
"Row Field 1", "Row Field 2", "Value 1", "Value 2"; `field_headers` 3 with `total_depth` -1 puts the header row first, then
Total. A header row over no records (`GROUPBY(A1,D1,f,1)`), a non-function (`5`) and fractional options (`total_depth`
0.5, `field_headers` 2.5) are `#VALUE!` (ERROR.TYPE 3). An empty-text key (`=""`) is its own group, separate from the blank
key: it shows `""`, sorts as text (before "a"; after it descending), and the blank key stays last.
Unprobed: whether one row of text values with no rows below is a header (the engine takes it as data), and where
error keys sort against text, booleans and the blank key (the engine uses `sort_key_cmp`'s order: after booleans,
before blank).
Ours: GROUPBY (T-1322, functions/lookup_and_reference/groupby.rs): the shared `Grouping` core (hash grouping on
`cell_key` with -0 as 0, sorting with `sort_key_cmp` and numbers compared exactly), a LAMBDA or eta-name function called
once per group and value column through one `LambdaCaller`, field_headers and auto-detect, total_depth 0/±1, `""`
fillers, several value columns, the output size checked with `check_array_size` before any call. Not yet: sort_order,
filter_array (T-1323), total_depth ±2 and field_relationship (T-1324), function arrays (T-1325; all
`#N/IMPL!` until then), PERCENTOF's relative set (T-1325; called with one argument today), a one-cell reference result (`#VALUE!` in Excel, its value here: T-1325), PIVOTBY
(T-1326, T-1327).
Status: VERIFIED (web)
Tasks: T-1276, T-1328, T-1322

## WRAPCOLS of a single value is not padded; wrap_count below 1 is #NUM!
Rule: `WRAPCOLS` of a single value returns that value as a 1x1 result, not a column padded with `#N/A` to wrap_count:
`WRAPCOLS(FALSE,4)` is FALSE in one cell and `WRAPCOLS("1.5",4)&DROP(C10,0,3)` is one `#CALC!` (neither spills over the
occupied cells below). A wrap_count of 0.5 is `#NUM!` (`HYPERLINK(WRAPCOLS(C12,0.5,..),..)`).
Evidence: gen-m7/bb4617644fee40c9 Calc!A30, gen-m7/d38b52bfa1a9f9e6 Calc!A26, gen-m7/f68fafa0ca2eba26 Calc!A39 (excel-graph, 2026-10-11, T-1275).
Ours: pads to wrap_count (`ROWS(WRAPCOLS(1,4))` = 4, so both cells are `#SPILL!`), and wrap_count 0.5 is `#VALUE!`.
Status: VERIFIED for single values (web); a multi-element vector shorter than wrap_count (`WRAPCOLS({1,2},4)`), WRAPROWS,
and wrap_count 0 or negative are UNVERIFIED
Tasks: T-1348

## MAP over arrays of different sizes, or a single value and an array
Rule: MAP does not reject arrays of different sizes: `MAP(B4:E8,A4:D11,B5:E7,LAMBDA(z,x,acc,A3))` (5x4, 8x4, 3x4) and
`MAP(WEEKDAY(7538),B4:E11,LAMBDA(acc,y,acc))` (a number and 8x4) both return arrays (`#SPILL!` in a cell with occupied
neighbours). The element values (broadcast and `#N/A` padding as in array arithmetic?) are not evidenced.
Evidence: gen-m7/a0cbc75358b429b6 Calc!A31, gen-m7/b5186d194fec0fdc Calc!A13 (excel-graph, 2026-10-11, T-1275).
Ours: `#VALUE!` ("all arrays must have the same dimensions").
Status: VERIFIED that the result is an array (web); its size and values are UNVERIFIED
Tasks: T-1349

## BYROW, BYCOL, MAP, SCAN and REDUCE hand a range's slices to the LAMBDA as references
Rule: over a range (written, or picked by TAKE and the like), BYROW and BYCOL give the LAMBDA each row or column as a
reference and MAP, SCAN and REDUCE each cell as a reference. With A1 x, A3 z, B3 y (A2:B2, B1 blank):
`BYCOL(A1:B3,LAMBDA(c,COUNTA(c)))` is 2, 1 (blank cells are skipped, as for any reference), `BYROW(..,COUNTA(r))` 1, 0, 2,
`BYROW(..,COUNTBLANK(r))` 1, 2, 0, `BYROW(A1:B3,LAMBDA(r,ROW(r)))` 1, 2, 3, `ISREF` of the slice is TRUE in all five
helpers, `BYCOL(TAKE(A1:B3,2),LAMBDA(c,COUNTA(c)))` 1, 0 and `LET(x,A1:B3,BYCOL(x,..))` 2, 1. Over an array the slices are
arrays and every element counts: `BYCOL(IFERROR(A1:B3,1),LAMBDA(c,COUNTA(c)))` is 3, 3. An eta-reduced COUNTA,
`BYCOL(A1:B3,COUNTA)`, is `#VALUE!`. A whole column or row keeps its full size, not the used area:
`ROWS(BYROW(H:H,LAMBDA(r,1)))` and `BYCOL(H:H,LAMBDA(c,ROWS(c)))` are 1048576, `COLUMNS(BYCOL(60:60,LAMBDA(c,1)))` and
`BYROW(60:60,LAMBDA(r,COLUMNS(r)))` 16384 (G1:G4).
Evidence: probes/lambda-helper-range-slices (excel-graph, 2026-10-12, T-1395).
Ours: BYROW and BYCOL pass references since T-1395 (`by_slices_source`, bycol_byrow.rs); a range keeps its full size
(a whole column gives 1,048,576 calls and full-column slices) under the 2^20-cell array bound of "Array results above
1,048,576 cells". MAP, SCAN and REDUCE pass each cell of a one-sheet range as a one-cell reference too (`lambda_cells`, T-1442); a reference the LAMBDA returns is read when it becomes a MAP or SCAN output cell, while a REDUCE or SCAN accumulator stays as returned (what Excel does with `ISREF(a)` on an accumulator is unprobed); the bare COUNTA of E32 is `#VALUE!` since T-1466 (a bare name read from a file is unknown, and BYROW and BYCOL refuse anything but a LAMBDA; see "Functions passed by name").
Status: VERIFIED (excel-graph)
Tasks: T-1395, T-1442

## BYROW and BYCOL: a LAMBDA result that is an array
Rule: if the LAMBDA returns a 1x1 array (`{7}`, `SEQUENCE(1)`, a one-element array slice) BYROW and BYCOL use its one
value; if it returns anything bigger than one cell for any slice (a row or column slice, `{7,8}`, `{7;8}`,
`SEQUENCE(2,1)`) the whole call is one `#CALC!` cell. This holds for a range source, an array-constant source and a mix of
scalar and array results (`BYROW(A1:B3,LAMBDA(r,IF(SUM(r)>5,r,0)))` is `#CALC!`; with `SEQUENCE(1)` in place of `r` it is
0, 1, 1). Over `{1;2;3}` or `{1,2,3}` a slice is one cell, so `BYROW({1;2;3},LAMBDA(r,r))` is 1, 2, 3 and
`BYCOL({1,2,3},LAMBDA(c,c))` 1, 2, 3, while `LAMBDA(r,{7,8})` is `#CALC!`. Over A1:B3 (1, 2; 3, 4; 5, 6) `BYCOL(..,LAMBDA(c,c))`
is `#CALC!`, `LAMBDA(c,{7})` 7, 7 and `LAMBDA(c,SEQUENCE(1))` 1, 1.
Evidence: probes/byrow-bycol-array-results (excel-graph, 2026-10-16, T-1670).
Ours: bycol_byrow.rs `calc_result_to_array_node` unwraps a 1x1 array; `makes_calc_error` leaves it alone (T-1670).
Status: VERIFIED (excel-graph)
Tasks: T-1664, T-1670

## Functions passed by name (`_xleta.`)
Rule: a function passed by name, stored `_xleta.NAME` (any case, `_xleta._xlfn.STDEV.S` and `_xleta.STDEV.S` alike), is a
LAMBDA that takes as many arguments as it is given and passes them on: with A1 x, A3 z, B3 y, `BYCOL(A1:B3,_xleta.COUNTA)`
is 2, 1 (the slices are references), `BYROW(..,_xleta.COUNTA)` 1, 0, 2, `MAKEARRAY(2,2,_xleta.COUNTA)` 2 everywhere (two
arguments), `MAKEARRAY(2,2,_xleta.POWER)` 1, 1; 2, 4, `MAKEARRAY(2,2,_xleta.IF)` 1, 2; 1, 2, `MAP(A1:A3,_xleta.ISREF)` TRUE x3,
`SCAN(0,A1:A3,_xleta.COUNTA)` 2, 1, 2, `REDUCE(0,A1:A3,_xleta.SUM)` 0, `BYROW({1,2;3,4},_xleta.SUM)` 3, 7 and
`LET(f,_xleta.SUM,f(1,2))` 3. On its own it is a value like an uncalled LAMBDA: `=_xleta.SUM` is `#CALC!` and
`ISERROR(_xleta.SUM)` FALSE. A function whose result is an array for a row is `#CALC!` in BYROW (`_xleta.ABS`,
`_xleta.IF` with one argument); that `#CALC!` is one cell, not a spill of errors (probes/eta-reduced-names E49 and E101, nothing
below them in the sidecar; T-1660 spill-extent check; ours is one `#CALC!` since T-1664: `CalcResult::is_multi_value`, bycol_byrow.rs; mixed calls, BYCOL, array-constant sources and 1x1 array results are in "BYROW and BYCOL: a LAMBDA result that is an array" above, T-1670). BYROW and BYCOL refuse anything but a LAMBDA with `#VALUE!`, whatever it is: an error
(`BYCOL(A1:B3,NA())`), a number (`BYCOL(A1:B3,1)`), an unknown `_xleta.` name, and a bare function name read from a file
(`BYCOL(A1:B3,COUNTA)`, `BYROW(A1:B3,SUM)`: unknown names, xlsx-storage.md). `MAKEARRAY(2,2,_xleta.ABS)` (too many
arguments) and `BYROW(A1:B3,_xleta.NOW)` give an error excel-graph cannot decode (left pending).
Unprobed: other errors (`#DIV/0!`, `#REF!`) and an array as BYROW's or BYCOL's function (the engine says `#VALUE!` for
both), a special form passed by name (`_xleta.LET`, `_xleta.LAMBDA`, `_xleta.SUBTOTAL`, `_xleta.OFFSET`) and a call with too
few arguments (`LET(f,_xleta.SUM,f())`): the engine gives whatever the function gives for those arguments.
Evidence: probes/eta-reduced-names (30 questions, excel-graph, 2026-10-14, T-1466); probes/m7-argument-modes F28:H51.
Ours: an unbound `_xleta.` name of a known function evaluates to a LAMBDA whose definition is made per call
(`Model::unbound_name`, `eta_definition`, lambda.rs); BYROW and BYCOL take only a LAMBDA (`by_slices_lambda`). Not yet:
`ISERROR(_xleta.SUM)` is TRUE, and `_xleta.IF` with one argument is `#ERROR!` where Excel gives `#CALC!` (T-1511). GROUPBY calls an eta name through the same path (T-1322); PIVOTBY does not exist yet.
Status: VERIFIED (excel-graph)
Tasks: T-1466

## BYROW and BYCOL of a single value
Rule: BYROW and BYCOL take a single value as a 1x1 array and call the LAMBDA once: `"ü"&BYROW(COUNT(..),LAMBDA(z,z))` with
the COUNT 7 is "ü7", and `BYROW("",_xleta.PRODUCT)` is 0 (PRODUCT of the row {""}).
Evidence: gen-m7/93976dd9a3578ed9 Calc!A31, gen-m7/59cc50701f4937c9 Calc!A4 (excel-graph, 2026-10-11, T-1275).
Ours: `#VALUE!` for BYROW and BYCOL of a number or text (`BYROW(7,LAMBDA(z,z))`; MAP, SCAN and REDUCE of 7 work).
Status: VERIFIED (web)
Tasks: T-1350

## SORT argument errors come before validation
Rule: an error-valued argument of SORT wins over an invalid other argument: `SORT(B11:E12,ROUND(0,7),A3:C5,<#N/A>)`
(sort_index 0, a 9-cell sort_order range and a by_col that is `#N/A`) is `#N/A`.
Evidence: gen-m7/c5d278a8662ef4b2 Calc!A39 (by_col `HLOOKUP(ADDRESS(19,19),Data!C8:C12,3)`) (excel-graph, 2026-10-11, T-1275).
Ours: `#VALUE!` (sort_index 0 is rejected before by_col is looked at).
Status: VERIFIED (web) for this cell; SORTBY and the order among several error arguments are UNVERIFIED
Tasks: T-1351

## SORT: omitted sort_index / sort_order take their defaults

An omitted (empty) `sort_index` or `sort_order` is the same as leaving it off: `SORT({3;1;2},,)`
is ascending by column 1 (3 values; `INDEX(...,1)` = 1) and `SORT({3;1;2},,-1)` is descending
(`INDEX(...,1)` = 3). Evidence: probes/counta-empty-value D22-D24 (excel-graph, T-1294); the engine
treats the empty argument as the default (T-1394).

T-1446 probed every combination (probes/sort-omitted-args, excel-graph, 2026-10-13): an omitted `sort_index`,
`sort_order` or `by_col` is the default whatever its position or how many empties trail
(`SORT(a,,,)`, `SORT(a,,-1,)`, `SORT(a,1,,)`, `SORT(a,,,TRUE)` by columns). In SORTBY an omitted
`sort_order` (`SORTBY(a,k,)`, and the dangling order after a second key `SORTBY(a,k1,,k2,)`) is
ascending. The Graph backend refuses to load a workbook holding `SORTBY(a,k,,)`, `SORTBY(a,k,-1,)` or
`SORTBY(a,k,,k2)` (two empties after a key, or an empty order followed by a second key and no
further argument; `SORTBY(a,k1,,k2,)` loads): UNVERIFIED, desktop
question (decision 0039).

Status: VERIFIED (excel-graph) for all probed combinations; the three Graph-refused SORTBY forms are UNVERIFIED (desktop question).
Tasks: T-1394, T-1446.

## WRAPROWS and WRAPCOLS: wrap_count 0 or below is #NUM!, above the vector it clamps, a scalar is 1x1
Rule (probes/dynamic-array-reshape-and-isomitted C20-C52, D20-D52, excel-graph, 2026-10-15, T-1627; B5:H5 is 1..7):
`WRAPROWS(B5:H5,0,"p")` and `WRAPROWS(B5:H5,-1,"p")` are `#NUM!`, and so are the WRAPCOLS forms. A wrap_count above
the vector's length does not pad: `WRAPROWS(B5:H5,8,"p")` and `WRAPROWS(B5:H5,100,"p")` are the 1x7 row `1,2,3,4,5,6,7`
(`WRAPCOLS` the 7x1 column), the same as wrap_count 7. A scalar is a one-element vector: `WRAPROWS(5,2)` is 5
(`WRAPCOLS(5,2)` too). Ours gives `#VALUE!` for 0 and -1, pads to wrap_count for 8 and 100 (1x8, 1x100) and `#N/A`
for a scalar (18 cells; fix T-1630). Everything else probed matches: wrap_count 1-7 with a pad, an omitted pad (`#N/A`
fill), pad_with 0, "" and `#N/A`, a column vector, an inline array, a 2-D range (`#VALUE!`), a text
wrap_count (`#VALUE!`; C43, D43); a boolean wrap_count counts as a number (TRUE is 1: `WRAPCOLS(B5:H5,TRUE)` is the 1x7 row 1..7, D44; `WRAPROWS(B5:H5,TRUE)` is `#SPILL!` in C44 because the cell below blocks it, so its 7x1 shape is inferred, not shown;
re-probe T-1632); a fractional wrap_count
truncates (2.5 is 2: `WRAPROWS(B5:H5,2.5,"p")` is 4x2, `WRAPCOLS` 2x4, C38-D39) and an error vector.
TOROW over A1:C3 (1, "x", blank / #N/A, 4, TRUE / 7, 8, 9), ignore 0-3 with scan_by_column FALSE and TRUE, a scalar, inline
arrays, a bad ignore (4, -1, "a") and an error argument all match ours (probes/dynamic-array-reshape-and-isomitted B20-B49). Ignore 1 keeps
errors (B22/B23 are `#N/A`) and keeps an empty string `""` (`COLUMNS(TOROW({1,"",3},1))` is 3, B48); a boolean ignore is
accepted (`TOROW({1,2,3},TRUE)` is 1,2,3, B46; which mode TRUE selects is not shown, T-1632).
ISOMITTED inside `LAMBDA(x,[y],...)` matches ours in every probed form (A20-A49): an omitted argument is TRUE, an empty slot
(`(1,)`, A24-A25) counts as omitted and so does a required parameter left empty (`(,2)`, A34), while `""` (A26-A27) and a reference to a
filled cell (A28-A31) count as supplied (FALSE; a reference to a blank cell is not probed, T-1632); outside a LAMBDA `ISOMITTED(5)`, `ISOMITTED(A1)` and `ISOMITTED(#N/A)` are FALSE (A43-A45).
Status: VERIFIED (excel-graph)
Tasks: T-1627, T-1630
