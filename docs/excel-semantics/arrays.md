# Arrays

## Cached rich errors (`#CALC!`, `#SPILL!`, `#BLOCKED!`, ...) are stored as `#VALUE!` (or `#N/A`) plus a `vm` attribute
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
sees `#VALUE!`, except for errorType 6, whose cells cache `<v>#N/A</v>` (see below). Every rich error is stored this way, not only the dynamic-array ones; errorType is
always ERROR.TYPE − 1 and `subType` (the reason) does not change the error:

| errorType | error | ERROR.TYPE | produced by (probe cell) |
|---|---|---|---|
| 2 | `#VALUE!` | 3 | REGEXEXTRACT's invalid return_mode spilled by MAP, GROUPBY or EXPAND (rich-errors-spilled-lambda A16, B51, B66; T-2417) |
| 6 | `#N/A` | 7 | REGEXEXTRACT with no match (subType 7) or a capture group that took no part (subType 9), with an `<fb t="e">#N/A</fb>` fallback (rich-errors-regex-group A4:A6; T-2442) |
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

A `#VALUE!` from a GROUPBY function argument (`LAMBDA(w,REGEXEXTRACT(...,-1))`) is stored in every
spilled value cell as a rich error of a third `_error` structure, `errorType, ptg, subType`, with
values (2, 6, 0). Its real value is `#VALUE!` (ERROR.TYPE 3 through INDEX, probes/groupby-lifted-args
A33 and B33). errorType 2 is `#VALUE!`, as errorType = ERROR.TYPE − 1 says: probes/rich-errors-spilled-lambda
(excel-graph, 2026-10-22, T-2417) spills one error kind per block from MAP, GROUPBY, EXPAND and plain array arithmetic.
Only REGEXEXTRACT's `#VALUE!` (an invalid return_mode) is stored as a rich error, `_error(errorType, subType)` = (2, 0),
in every spilled cell of MAP (A16:A17), GROUPBY (B51:B53) and EXPAND (B66, A67:B67), and its ERROR.TYPE and
VALUETOTEXT through INDEX are 3 and `#VALUE!` (E16, F16, E51, E66). `#DIV/0!`, `#N/A`, `#NUM!`, `#REF!`, `#NULL!` and the
`#VALUE!` of `"a"+x` in the same positions are plain `t="e"` literals. The cached excel-graph recalcs also store
errorType 2 in `errorType, iftab[, ptg], subType` structures (subType 0, 33, 39, 40, 59, 133); the reader finds
errorType by key name and subType does not change the error, so all of them decode to `#VALUE!`. subTypes 0 and 133 have direct ERROR.TYPE
evidence; the others rest on the T-0030 rule (their cached `<v>#VALUE!</v>` is no evidence: every rich error but errorType 6
caches `#VALUE!`). subType 133 is the `#VALUE!` of REGEXEXTRACT's return_mode 2 with a pattern without
capture groups (`REGEXEXTRACT("abc","b",2)`, matching or not): it is stored as (2, 133) when scalar, spilled by MAP or
padded by EXPAND, and as (errorType 2, ptg 3, subType 133) when padded as `REGEXEXTRACT(..)+1` (A14:B15); ERROR.TYPE is 3 directly and through INDEX (probes/rich-errors-regex-group
A1:A3, A8:A9, A11:B12, A14:B15, H/K columns; excel-graph, 2026-10-23, T-2442). In a lifted return_mode, an invalid
number element (`{0,5}`'s 5, `{-1;0}`'s -1, TOROW's 7) is stored as (2, 0), anchor (A30) or spilled member (B19, B21,
B23:B24), a valid mode 2 beside the pattern without groups as (2, 133) (B25, group "b"'s 2, and B26, the total row), and a text element (`"x"`,
a GROUPBY header) as a plain `#VALUE!` literal (A17:B17, A19, A23:A26).

errorType 6 is `#N/A` (ERROR.TYPE 7): REGEXEXTRACT's no-match `#N/A` (return_mode 0 and 2 probed; mode 1 not) is stored as the
rich error (6, 7) and a capture group that took no part in the match (`REGEXEXTRACT("b","(a)?(b)",2)`'s first element) as (6, 9).
Both carry Excel's own fallback `<fb t="e">#N/A</fb>` ahead of their fields, and their ERROR.TYPE is 7 and VALUETOTEXT
`#N/A`, directly and through INDEX, with ISNA TRUE (probes/rich-errors-regex-group A4:A6; excel-graph, 2026-10-23, T-2442).
Unlike every other rich error, their sheet cells cache `<v>#N/A</v>`, not `#VALUE!` (A4:A6 in the cached
recalc), so for errorType 6 the cached value agrees with ERROR.TYPE and a reader that ignores `vm` already sees `#N/A`.
The `<fb>` element is not a field, so the reader's errorType position is unchanged.
Evidence: gen-m7/18b80d3fd292d63a C951:E962 (all `vm="10"`) and a scratch recalc of
`GROUPBY(Data!A1:A4,Data!B1:B4,LAMBDA(w,REGEXEXTRACT("a","[0-9]",-1)))` (excel-graph,
2026-10-22, T-2397; recalc input_sha256 e07a9239… and 7cf52886…). That scratch recalc reported all four spilled value cells, so no case of a
spilled rich error missing from the workbook was reproduced. The T-2254 triage's 28 "unreported" spill cells
(gen-m7/0624b684889e9228 A612, 21b4eef18424ce94 A1504, 9bb61f6ae34c7e20 A136) were also in the workbook: the fill
had dropped them as undecoded (re-filled 2026-10-22, T-2397). Those cases store errorType 2 (subType 0, 39, 59 or 133;
some with `ptg` and `iftab` fields) and errorType 6 (subType 7) rich errors; errorType 2 is decoded since T-2417 and errorType 6 since T-2442. gen-m7/0624b684889e9228's sidecar still holds the 22 `#UNDECODED!` (errorType 2) cells of its pre-T-2417 fill; a re-fill decodes them all to `#VALUE!` (checked 2026-10-23, T-2442) and also scores the spill members of A2004, so it lands with T-2415. `rustsheet-oracle fill` keeps such cells in
the sidecar as `#UNDECODED!` instead of dropping them, because a spill member missing from the sidecar
is scored as an expected blank (T-2397).
Status: confirmed (storage; errorType 2 = `#VALUE!`, errorType 6 = `#N/A`, excel-graph).

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
Implemented in: `Model::numbers_fn` (functions/math_and_trigonometry/mathematical.rs) over the SPEC-0011 driver (functions/lift/), used by `fn_mod` (T-1773).

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
More evidence: ironcalc-calc/LOGICAL/NOT.xlsx (Excel-authored, 115 checked cells, T-2820): numeric-looking text ("1E3", "50%", "1/1/2024") and `NOT("")` give `#VALUE!`, computed text "TRUE" gives FALSE, an empty cell gives TRUE, and the lift holds over INDEX/ROWS/SUM and an anchor-array spill reference.
Tasks: T-0225 (NOT), T-0226 (CONCATENATE), T-0233 (element values), T-0260 (NOT per-element cast probe)
Implemented in: `fn_concatenate` (crates/rustsheet-engine/src/functions/text/concatenate.rs) through the lift driver (`Model::lift_text`, T-0226, T-1775); `fn_not` (`functions/logical/and_or_xor_not.rs`) and its `scalar_arguments` static-analysis entry (T-0225).

## CONCATENATE with a scalar error argument returns that error, not an array
Rule: if any CONCATENATE argument is a scalar error, the result is that one error, even when another
argument is an array; Excel does not spill an array of errors.
Evidence: gen-m2/ce315bb699e1be6f Calc!A313
`=MID(CONCATENATE(INDEX(Data!B8:E11,1),"x~*",HLOOKUP(-0.5,Data!B2:E2,4)),Data!A$3,1)` gives `#REF!`, not `#SPILL!`;
gen-m2/207a5753da48c7e0 Calc!A95 (VLOOKUP over `CONCATENATE(INDEX(..,0.1),INDEX(Data!D3:D4,4,1))`) gives `#REF!`.
Status: VERIFIED
Implemented in: no early return exists (T-1775 found none on main either): `fn_concatenate` gives an array of that error, one per element, whose top-left cell is the error the evidence cells show. Whether Excel's result is one error or an array of errors (`ROWS(CONCATENATE(NA(),{"a","b"}))`) is unprobed.

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
RustSheet matches (T-1022: an error argument is the result only when it is the first failure; `text_result` in functions/text/text_kernel.rs since T-1775). TRUE in a range
behaves like TRUE in an array (`#N/A`), and `WEEKDAY(NA(),"x")` is `#N/A` (left to right).
Probe cells: probes/error-precedence-text Calc!C1-C30 (oracle: excel-graph, 2026-10-06).
An error argument also beats a later range failure (start < 1, negative length, text not found): `REPLACE("abc",0,NA(),"x")`,
`REPLACE("abc",1,-1,NA())`, `REPLACE("abc",0,1,NA())`, `MID("abc",0,NA())`, `MID("abc",NA(),-1)` and
`FIND("z","abc",NA())` are all `#N/A`, while the same calls with a number in place of the error are `#VALUE!`
(`MID("abc",1,-1)`, `FIND("z","abc")`). So REPLACE, MID and FIND coerce every argument before any range check (not probed for other text functions)
(probes/error-arg-vs-range-check A1-A10, oracle: excel-graph, T-1056); `replace_element` matches since T-1056.
Implemented in: `array_of_errors` and `MISSING` (crates/rustsheet-engine/src/arithmetic.rs),
`text_result` (crates/rustsheet-engine/src/functions/text/text_kernel.rs), `fn_date`, `fn_weekday`.
Tasks: T-0129, T-0994, T-1022, T-1056

## CEILING/FLOOR family and MROUND lift over arrays
Rule: ISO.CEILING, CEILING, CEILING.MATH, CEILING.PRECISE, FLOOR, FLOOR.MATH, FLOOR.PRECISE and MROUND lift element-wise over an array or multi-cell range in every argument, except MROUND (below). An error in `number` still broadcasts over an array in another argument (`CEILING.MATH(<error>,sig,{9x1 mode})` is a 9x1 array, `#SPILL!` where it cannot spill).
Evidence: gen-m7/e5a361112d83d0c3 Calc!A9 (ISO.CEILING, excel-graph 2026-10-11); gen-m9/89693ab0cafc29df Calc!A14 (CEILING.MATH, excel-graph 2026-10-12); probes/ceiling-floor-mround-array-lift A1:A18 (CEILING, FLOOR, CEILING.PRECISE, FLOOR.PRECISE, FLOOR.MATH, CEILING.MATH and MROUND with an array in each argument; excel-graph 2026-10-15, T-1589). The family reads a boolean element as 1/0 (`CEILING({TRUE,2},1)` spills `1, 2`, `FLOOR(5,{TRUE,2})` spills `5, 4`, A15:A18).
MROUND keeps its scalar rules per element: a boolean element or cell is `#VALUE!`, not 1/0 (`MROUND({TRUE,2.6},1)` spills `#VALUE!, 3`, `MROUND(1,{TRUE,2})` spills `#VALUE!, 2`, `MROUND(INDIRECT("K1"),1)` and `MROUND(TRUE,1)` are `#VALUE!`; A19:A25). MROUND judges its two arguments left to right: an error, a missing argument (`#N/A`) or a boolean (`#VALUE!`) in `number` wins over whatever `multiple` holds, so `MROUND(TRUE,)` is `#VALUE!` but `MROUND(,TRUE)` is `#N/A` (A27:A28), `MROUND(1/0,TRUE)` and `MROUND(1/0,)` are `#DIV/0!`, `MROUND(,1/0)` is `#N/A`, `MROUND(TRUE,1/0)` is `#VALUE!`, `MROUND("x",)` is `#VALUE!` but `MROUND(,"x")` is `#N/A` (A23, A27:A42); an error in `number` still broadcasts over an array `multiple` (`MROUND(1/0,{TRUE,2})` spills `#DIV/0!, #DIV/0!`, A22). MROUND is an Analysis ToolPak function and does not lift over a multi-cell range in either argument: `MROUND(M1:N1,1)`, `MROUND(1,M1:N1)`, `MROUND(M1:M3,1)` and `MROUND(M1:N1,M1:N1)` over plain numbers are one `#VALUE!` with no spill (`COLUMNS(MROUND(M1:N1,1))` is `#VALUE!`, A43:A49, A55), while an array constant lifts (`COLUMNS(MROUND({1.2,2.6},1))` is 2, A50) and so do CEILING and FLOOR.MATH over the same range (A51:A52, A30:A31); same rule as ISEVEN (information.md). An error argument next to a multi-cell range wins as a scalar: `MROUND(NA(),E1:E3)` is one `#N/A` (`ROWS` of it `#N/A`), and so is a reference a function returns, `MROUND(TAKE(E1:E3,2),1)` being one `#VALUE!` (probes/date-math-lift-driver-edges G14-G15, G19-G20). A scalar boolean, missing argument or non-numeric text next to an array `multiple` or `number` is broadcast, one element per array element (`ROWS(MROUND({5;7},TRUE))`, `ROWS(MROUND(TRUE,{2;3}))`, `ROWS(MROUND({5;7},))`, `ROWS(MROUND(,{2;3}))` and `ROWS(MROUND({5;7},"x"))` are 2, G11-G13, G16-G18), and a referenced cell is judged like a literal (`MROUND(I1,)` with I1 `=1/0` is `#DIV/0!`, with I1 TRUE `#VALUE!`, a blank I3 then TRUE `#VALUE!`, a blank then a missing argument `#N/A`; G44-G47, G51-G52). A 3-D reference is `#VALUE!` (G27), and a legacy `MROUND(E1:E3,2)` takes no `@`: it is `#VALUE!` on the range's rows (K3). A multi-cell range next to an array follows the Analysis ToolPak rule: the array lifts and every element is `#VALUE!` (`ROWS(MROUND(E1:E3,{1;2}))` and `ROWS(MROUND({45000;45001},E1:E2))` are 2, `INDEX(MROUND(E1:E3,{1;2}),1)` is `#VALUE!`), while non-numeric text before a range is one `#VALUE!` (`MROUND("x",E1:E2)`, `ROWS` of it `#VALUE!`), and a one-cell range is its value (`ROWS(MROUND(E1:E1,NA()))` is `#N/A`; N5, N7-N11). `MROUND("3",)` is `#N/A` and `MROUND("3",2)` is 4 (A53:A54). The scalar rule "every missing argument before any boolean" the engine had was wrong for `MROUND(TRUE,)`.
Status: VERIFIED
Implemented in: functions/math_and_trigonometry/ceiling_floor.rs (`numbers_fn`, the SPEC-0011 driver, T-1773), mathematical.rs (`fn_mround`, a `LiftArrays` table entry whose kernel judges the arguments left to right; T-1785)
Tasks: T-1357, T-1589, T-1785

## ROUND, ROUNDUP, ROUNDDOWN and TRUNC lift over arrays
Rule: `ROUNDDOWN(array, num_digits)` returns an array, element-wise, with either argument an array; an error in either argument broadcasts (`ROUND(A1:B1,1/0)` and `ROUND(1/0,A1:B1)` are `#DIV/0!`).
Evidence: gen-m2/893302d6c5b99e2e Calc!A313; probes/round-concat-array-lift (excel-graph, 2026-10-05).
Status: VERIFIED
Implemented in: functions/math_and_trigonometry/mathematical.rs (`round_family` over `numbers_fn`), functions/lift/ (the SPEC-0011 driver; T-1773)
Tasks: T-0222, T-1400

Non-number elements (probes/round-concat-array-edges A1-A7, excel-graph 2026-10-13, T-1400): the elements are coerced as scalar arguments are. `ROUND({TRUE,"2.5",""},0)` and `ROUNDUP(...)` spill `1, 3, #VALUE!` (also the column form `{TRUE;"2.5";""}`), `ROUNDDOWN(...)` spills `1, 2, #VALUE!` (numeric text is read as a number, a logical as 1, the empty string is `#VALUE!`). `ROUND({"abc"},0)` is `#VALUE!`, and `ROUND({"abc",1.5},0)` spills `#VALUE!, 2` (a bad element is an error in its own cell only). The same coercion applies to num_digits: `ROUND(2.567,{TRUE,"1",""})` spills `2.6, 2.6, #VALUE!`. The engine matches (`kernel_number` in functions/lift/mod.rs).

## Stacked results past the grid
Rule: a VSTACK with more rows than the grid is `#NUM!`, while an HSTACK with more columns than the grid is allowed (EXPAND, WRAPROWS and WRAPCOLS share it, and so does a VSTACK of an input wider than the grid): `ROWS(VSTACK(A:A,B1))`, `ROWS(VSTACK(A1:A1048576,B1))`, `ROWS(VSTACK(A:A,B1:B2))` and `COLUMNS(VSTACK(A:A,B1:B2))`
are `#NUM!`, and so is `ROWS(VSTACK(A:A,1:1))`; `COLUMNS(HSTACK(A1:XFD1,A1))`, `COLUMNS(HSTACK(A1:XFD1,A1:A2))` and
`COLUMNS(HSTACK(A1:A2,1:1))` are 16385 (`ROWS` of the second is 2), and `COLUMNS(HSTACK(A1:XFC1,A1:A2))` 16384. A row
wider than the grid is built as TEXTSPLIT's is ("textsplit-wide"). The 2^34-cell `ROWS(HSTACK(A:A,1:1))` is 0 in the Graph
backend, an answer not trusted; ours is `#ERROR!` outside the stacked row and column, the cell-count bound ("Array results above 1,048,576 cells").
Evidence: probes/scalar-lift-open-c W1-W10, -c2 W1, -c3 W1 (excel-graph, 2026-10-17, T-1777).
EXPAND: `ROWS(EXPAND(1,1048577,1))`, `ROWS(EXPAND(A1:A2,1048577,1))` and `EXPAND(1,1E9,1E9)` are `#NUM!`, while
`COLUMNS(EXPAND(1,1,16385))` and `COLUMNS(EXPAND(A1:A2,2,16385))` are 16385 (16384 for `EXPAND(1,1,16384)`). WRAPCOLS and
WRAPROWS allow columns past the grid: `ROWS(WRAPCOLS(SEQUENCE(16385),1))` is 1 and `COLUMNS(WRAPROWS(SEQUENCE(16385),16385))`
16385; their rows never pass the grid (wrap_count is clamped to the vector, and `SEQUENCE(1048577)` is itself `#VALUE!`).
`COLUMNS(VSTACK(TEXTSPLIT(REPT(",",16384),","),1))` is 16385.
Evidence: probes/expand-wrap-past-grid W1-W10 (excel-graph, 2026-10-19, T-1967).
Far larger sizes give other errors (gen-m7, excel-graph, 2026-10-22, T-2254 triage): EXPAND with rows 1E+21
(`EXPAND(1e-15,ROUND(1E+21,45),1)`, gen-m7/2d9b48c6363e7a16 Calc!A2454) or 1.83E+42 (8c4570978f18f4c2 A4006) is `#VALUE!`, not `#NUM!`
(so the error changes somewhere between 1E+9 and 1E+21 rows); `EXPAND("a",4,123456789.123)` is `#NUM!` (4dac39f89709e2fc A2150), a size
Excel refuses itself; and MAKEARRAY allows columns past the grid as HSTACK does:
`MAKEARRAY(4,25000,_xleta.AVERAGE)` is `#SPILL!` (21b4eef18424ce94 A3348), the 4 x 25,000 result built and not spillable (ours was `#VALUE!` before T-2408).
The thresholds (probes/expand-makearray-past-grid A1-A15 and -big, excel-graph, 2026-10-23, T-2408): EXPAND rows or columns of
2^31 or more are `#VALUE!` (`ROWS(EXPAND(1,2^31,1))`, 1E10 through 1E21, `COLUMNS(EXPAND(1,1,2^31))`), and rows below 2^31 past
the grid `#NUM!` (`2^31-1`); a negative rows is `#VALUE!` (smaller than the array). Columns past the grid have a cap of Excel's
own: 16,385 are built with 1 and 4 rows, while 123,456,789 in one row and 2^27 in four are `#NUM!`. The cap is 1,048,576 columns,
2^20, the grid's row count: in one row, `COLUMNS(EXPAND(1,1,n))` is n for 16386, 32769, 65537, 131073, 262145, 524289 and 1048576,
and `#NUM!` for 1048577, 2^21, 2^22, 2^24 and 2^26 (probes/t2452-expand-cols-<n>, one cell per workbook, excel-graph, 2026-10-25,
T-2452). It counts columns, not cells: `COLUMNS(EXPAND(1,2,524288))` and `(1,2,524289)` are 524288 and 524289, and
`ROWS(EXPAND(1,1048576,2))`, 2^21 cells, is 1048576 (probes/t2452-expand-2x524288, -2x524289, -1048576x2). Ours refuses from
1,048,577 columns (`COLUMNS_REFUSED`), after the smaller-than-the-array `#VALUE!` and before the rows check; below it, a result
above 2^20 cells is `#ERROR!` (the DEVIATION below; -2x524289 and -1048576x2 are listed in probes/deviations, decision 0113).
VSTACK checks its rows before it reads a referenced cell, after it evaluates every other argument: a stack past the grid
is `#NUM!` even when a reference argument holds the formula's own cell, so `ROWS(VSTACK(A:A,1:1))` in W1 (-c2),
`ROWS(VSTACK(2:2,A:A))` in W2, `ROWS(VSTACK(A:A,W8))` in W8 and `COLUMNS(VSTACK(A:A,12:12))` in W12 are `#NUM!`, not a
circular reference; an error argument wins over the size wherever it stands, `ROWS(VSTACK(A:A,3:3,1/0))` in W3,
`ROWS(VSTACK(1/0,A:A,4:4))` in W4, `ROWS(VSTACK(A:A,1/0))` and `ROWS(VSTACK(1/0,A:A))` are `#DIV/0!`; and an argument
that is an expression reading the cell is evaluated, so it is a cycle: `ROWS(VSTACK(A:A,B9+W9))` in W9 is 0
(recalc.md "A circular reference without iteration keeps the cells' values", here a cached 0). ROWS, COLUMNS and ISREF of the formula's own row read no
cell: `ROWS(5:5)` in W5 is 1, `COLUMNS(6:6)` in W6 16384, `ISREF(7:7)` in W7 TRUE.
Evidence: probes/vstack-self-past-grid W2-W12 (excel-graph, 2026-10-21, T-1965).
Ours: `limits::check_stack_size` in `fn_hstack`, `fn_vstack` (T-1803), EXPAND, WRAPROWS and WRAPCOLS (T-1967). `fn_vstack`
evaluates every argument but a one-cell reference, a range only to its corners, and checks the rows of every argument
before reading a reference (T-1965); that an evaluated array's rows count too, `ROWS(VSTACK(SEQUENCE(1048576),1:1))` in
row 1 `#NUM!`, follows from that order but is unprobed. W9 is 0 since T-0445 (recalc.md), and so is -c3 W1, which sits in row 1, which it stacks.
Status: VERIFIED (excel-graph)
Tasks: T-1803, T-1967, T-1965, T-2408, T-2452

## Arrays past the grid: MAKEARRAY, SEQUENCE and the lifts over them
Rule: an array value may be wider or taller than the grid; only a spill of it is `#SPILL!`. `COLUMNS(MAKEARRAY(1,20000,LAMBDA(r,c,c)))`
is 20000 (B20), its `INDEX(..,1,19999)` 19999 (B21), its `SUM` of ones 20000 (B22) and `COLUMNS(MAKEARRAY(..)+1)` 20000 (B30);
`ROWS(MAKEARRAY(1048577,1,LAMBDA(r,c,1)))` is 1048577 (B23); `COLUMNS(SEQUENCE(1,16385))` is 16385 (B24), while
`ROWS(SEQUENCE(1048577))` is `#VALUE!` (probes/resource-limits C13), so SEQUENCE refuses rows past the grid but not columns;
`COLUMNS(HSTACK(SEQUENCE(1,10000),SEQUENCE(1,10000)))` and `COLUMNS(TRANSPOSE(SEQUENCE(20000)))` are 20000 (B28, B29);
a function lifted over such an array lifts it whole, so `ERROR.TYPE(MAKEARRAY(1,16385,LAMBDA(r,c,c)))` in a cell is
`#SPILL!` (B26), and `BETAINV(..,MAKEARRAY(4,51638,..),..)` is too (gen-m9/4250f880ab640c49 Calc!A237).
`MAKEARRAY(3,1E15,..)` is `#VALUE!` (gen-m7/b2078b5d702c691a A2553).
Evidence: probes/t2579-array-value-edges B20-B32 (excel-graph, 2026-10-24, T-2579); probes/resource-limits C13.
Ours: `limits::check_built_array_size` in MAKEARRAY and SEQUENCE (after SEQUENCE's own rows check against the grid): any
shape within the cell bound of the next section is built; above it, SEQUENCE's columns past the grid are `#VALUE!`, as `SEQUENCE(3,123456789)`
is in Excel (gen-m9/f2bd5f380ec002bf Calc!A177) and `MAKEARRAY(3,1E15,..)` (A2553), and so are rows of 2^31 or more, EXPAND's
threshold (`SIZE_LIMIT_EXCLUSIVE`), as `MAKEARRAY(1E15,4,..)` is (gen-m7/8c4570978f18f4c2 A2736, gen-m9/895a33aa8352864a A238,
gen-m9/f280df8ba95f96b9 A89); any other shape is the bound's `#ERROR!`, so `ROWS(MAKEARRAY(1048577,1,..))` (B23) is `#ERROR!`
(DEVIATION, decision 0014, listed in probes/deviations). MAKEARRAY has no column cap at EXPAND's: `COLUMNS(MAKEARRAY(1,1048576,..))`
is 1048576, `(1,1048577,..)` 1048577 and `COLUMNS(MAKEARRAY(2,524289,..))` 524289 (probes/t2452-makearray-cols-1048576, -1048577,
-2x524289, excel-graph, 2026-10-25, T-2452; `MAKEARRAY(1,123456789,..)` never opened in excel-graph, so it is unprobed), so ours
refuses MAKEARRAY's columns with `#VALUE!` only from 2^31, as its rows, and the two built cells above the bound are its `#ERROR!`
(DEVIATION, decision 0113). Unprobed: where between 16,385 and 123,456,789 columns SEQUENCE's own `#VALUE!` starts, and where
between 1,048,577 and 1E15 (rows) or 2^31 (columns) MAKEARRAY's does. The lifting
driver's broadcast (`lift::driver::broadcast_shape`) checks the cell count only (`check_array_cells`), so B26 is the
spill's `#SPILL!`. A237 is `#ERROR!`: its 206,552 BETAINV elements pass the quantile's lift cost cap (statistics.md), T-2607.
Status: VERIFIED (excel-graph)
Tasks: T-2579, T-2600, T-2452

## Array results above 1,048,576 cells are #ERROR!
Rule: Excel builds any array that fits the grid: `ROWS(SEQUENCE(1000,1001))` = 1000,
`ROWS(SEQUENCE(1048576))` = 1048576, and a range lifted element-wise by a scalar function becomes
an array of the range's size however large: `ROWS(DATE(11:1899,12,1))` = 1889 and
`COLUMNS(DATE(11:1899,12,1))` = 16384 (31M cells), `INDEX(DATE(11:1899,12,1),5,7)` = 336;
`=DATE(11:1899,12,1)` in C3 is `#SPILL!` (no room). A whole column lifts like any range:
with A1 = -2, A2 = 3, `ROWS(ABS(A:A))` = `ROWS(A:A+0)` = 1048576, `SUM(ABS(A:A))` = 5,
`INDEX(ABS(A:A),2)` = 3, `SUM(--ISNUMBER(A:A))` = 2, and `=ABS(A:A)` in C7 is `#SPILL!`.
Some functions refuse a dimension past the grid with `#VALUE!` (`SEQUENCE(1048577)`), others build it ("Arrays past the grid" above).
RustSheet returns `#ERROR!` (not even an Excel error value) for any array above 1,048,576 cells
(one whole column): SEQUENCE, RANDARRAY, MAKEARRAY/BYROW/BYCOL results, matrix results, an *IF(S)
criteria grid (broadcast criteria included), a GROUPBY result (groups x columns, checked while grouping, before any
LAMBDA call; T-1322), the broadcast result of `lift_numbers` (DATE) and of the SPEC-0011 driver (ROUND family, MOD; functions/lift/driver.rs), a lifted INDEX's broadcast indices (T-1260), the broadcast of a binary operator over two arrays
(arithmetic, `&` and comparisons: a whole column against a whole row, `=D:D+4:4`, would be 2^34
cells, and even a whole column against two columns, `SUM(A1:A1048576*B1:C1)`, is 2^21; T-1420), the broadcast of IF, IFERROR and IFNA (`=IF(D:D,4:4)`, `=IFERROR(D:D,4:4)`; T-1448), the broadcast of the element-wise text functions (`broadcast_text`: LEFT, RIGHT, MID, FIND, SEARCH, LEN, REPT, EXACT, CONCATENATE, TEXT, CHAR, CODE, BAHTTEXT and the other callers; `=LEFT(A:A,1:1)` would be 2^34 cells and FIND's numbers do not count against the 64 MiB text budget, so `ROWS(FIND("a",A1:A2000,COLUMN(A1:ALL1)))` is `#ERROR!`; T-1771), the padded result of HSTACK and VSTACK (rows x summed columns, or summed rows x columns, checked before any argument is padded or copied: `ROWS(HSTACK(A1:A2000,A1:ALL1))` is `#ERROR!`, and a stack past the grid follows "Stacked results past the grid" below (`check_stack_size`: rows past it are `#NUM!` as in Excel, columns past it are allowed, so `ROWS(HSTACK(A:A,1:1))`, 2^34 cells, is `#ERROR!`; T-1803); T-1780), the padded results of EXPAND, WRAPCOLS and WRAPROWS and the product of MMULT (rows x columns, checked before allocating and, for EXPAND and MMULT, after their own argument checks: `ROWS(EXPAND(1,2000,1000))` and `ROWS(MMULT(A1:A2000,B1#))` with a 1,000-column B1# are `#ERROR!`, and `EXPAND(1,1E9,1E9)` is `#NUM!` past the grid as in Excel (`check_stack_size`; "Stacked results past the grid"); WRAPCOLS and WRAPROWS first clamp wrap_count to the vector's length, as Excel does (a count of 2^31 or more is `#NUM!` first, T-2312), so they pad at most wrap_count - 1 cells; XMATCH builds no result array, only a one-row or one-column search vector; T-1786), the padded result of MAP over arrays of different sizes (largest rows x largest columns, checked before allocating; T-1349), the padded result of TEXTSPLIT (rows x widest row, checked before any row is padded: a 32,767-char text with the column delimiters first and the row delimiters after pads ~2^28 cells, and `ROWS(TEXTSPLIT(REPT(",",1500)&REPT(";",1500),",",";"))`, 1501 x 1501, is `#ERROR!`, where Excel builds the array and gives 1501 for ROWS and for COLUMNS, and 1025 for the 1025 x 1025 text just above 2^20, and 1024 for the 1024 x 1024 text of exactly 2^20 cells (A5, which fits) and 1001 for 1001 x 1001 (A6): probe `textsplit-padded-size` A1:A6; the bound is the cell count only, `check_array_cells`, because Excel builds a row wider than the grid: `COLUMNS(TEXTSPLIT(REPT(",",16384),","))` is 16385 and the 16383-comma text 16384, probe `textsplit-wide` A1:A2, and a 32,001-row split gives 32001, A4; the bare 16,385-column spill is `#SPILL!` in Excel, A6; T-1781), the broadcast of SPEC-0011's lifting driver (`lift_call`: EDATE, EOMONTH, WEEKNUM and YEARFRAC, and LARGE, SMALL, PERCENTILE*, QUARTILE*, PERCENTRANK* and TRIMMEAN, charged numbers + elements, TRIMMEAN numbers × elements, statistics.md, T-1772; checked before any element runs, and the call is also `#ERROR!` when the cells of its lifted arrays and ranges add up past 2^27 (`limits::MAX_LIFTED_CELL_VISITS`, checked before the next range is read) or when its kernel calls, weighted by the table's `KernelWork`, would cost more than that cap (RATE is charged the time of its 200-step cap per element, 2^13 units, so a lifted RATE of more than 16,384 elements is `#ERROR!`, T-2005, T-2814); Excel has neither limit; T-1770), the lifted SEQUENCE and MAKEARRAY (SPEC-0011 "Array-valued functions", T-1834: each lifted element is charged the cells it builds, `KernelWork::Output`, MAKEARRAY 128 per cell for its LAMBDA calls, against the same 2^27 cap, so `=ROWS(SEQUENCE(SEQUENCE(1024),1024))`, about 2^29 cells, and `=ROWS(MAKEARRAY(SEQUENCE(2048),1024,LAMBDA(r,c,r)))` are `#ERROR!`; Excel is expected to answer 1024 and 2048, worked out from probes/m7-argument-modes F1, F2 and F23, not measured), the lifted TAKE and DROP over an array value (T-1838: each element may copy the whole array, so the call is charged the array's cells per element, `LiftCost::Cells`, against the same 2^27 cap before any element runs: `=ROWS(TAKE(SEQUENCE(1000,1000),SEQUENCE(1000)))`, about 2^30, is `#ERROR!` where Excel is expected to answer 1000; over a range each element is a reference and is charged 1), the lifted TOCOL and UNIQUE (T-1836: the array is read once and every element walks it, so the call is charged the array's cells per element, `LiftCost::Cells`, plus the cells it builds, against the same 2^27 cap before any element runs: `=ROWS(TOCOL(SEQUENCE(131072),SEQUENCE(1100,1,0,0)))`, about 1.4 x 2^27, is `#ERROR!` where Excel is expected to answer 1100, worked out from probes/m7-argument-modes F9 and F12, not measured), and every range lifted into an array (`Model::evaluate_range`: scalar functions
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
C4–C8, C10 and probes/lifted-range-rows C1–C3, E1 (excel-graph, 2026-10-06; rows C1, C2 and E1 are listed in
eval/suites/probes/deviations, decision 0036; C3 passes since T-0483); probes/textsplit-padded-size A1–A4 (TEXTSPLIT's padded result, excel-graph, 2026-10-17;
listed in eval/suites/probes/deviations, decision 0091, T-2223); probes/t2579-array-value-edges B23 (`ROWS(MAKEARRAY(1048577,1,..))`,
1048577 in Excel, excel-graph, 2026-10-24; listed in eval/suites/probes/deviations, decision 0108, T-2600).
Status: DEVIATION (decision 0014)
Implemented in: limits.rs (`MAX_SIZE`, `check_array_size`,
`check_range_size`); `Model::chosen_indices` (CHOOSECOLS/CHOOSEROWS) and FORMULATEXT over a range in
crates/rustsheet-engine/src/functions/lookup_and_reference/; `Model::read_lift_args`, `Model::lift_grids` and `Model::lift_references` (OFFSET's array of references, T-1956) in
crates/rustsheet-engine/src/functions/lift/; `Model::evaluate_range` in crates/rustsheet-engine/src/cast.rs;
`broadcast_arrays` in crates/rustsheet-engine/src/arithmetic.rs; `ExternalArea::blocked_value` in
crates/rustsheet-engine/src/external_reference.rs (a spilled range into another workbook, T-1454);
`Model::external_area_value` and `Model::external_aggregate_value` in
crates/rustsheet-engine/src/external_links.rs (a cached range of a linked workbook, T-1460)
In a **spill** context the deviation is not needed: Excel's answer follows from the shape alone, so it costs
no allocation. `=SUMPRODUCT(book2!A:A=book1!A3)*book2!H:J` in book1!B3 (a scalar against a whole-column
3-column range, 3,145,728 cells) is `#SPILL!` in Excel; `=ABS(A:A)` and `=DATE(11:1899,12,1)` in a cell are the
same shape-only `#SPILL!` (above). Since T-0480 the formula of a dynamic-array anchor whose root is an operator
chain (`+ - * / ^ & =` and the other comparisons, parenthesized or not, and operator operands down to 8 levels,
`=1+H:J*2`) is `#SPILL!` when a link would broadcast to more than 1,048,576 cells and to a shape that runs past
the grid from the anchor, decided from the operand shapes before the broadcast (`Model::evaluate_spilling_chain`,
`unspillable_broadcast` in arithmetic.rs). Since T-0483 the same holds for a root (or operator operand) that is
a call in the lifting table that gives one value per element (not SORTBY or UNIQUE), a unary operator or `%`:
`=DATE(11:1899,12,1)` in C1 or C3 and `=-H:J` or `=H:J%` in C3
are `#SPILL!`, decided from the shapes of the ranges in its `Lift` arguments, nested lifts and unary operators
included, before anything is lifted (`Model::unspillable_lift` in spill_shape.rs; probes/lifted-range-rows C3).
The deviation still covers value contexts (`ROWS`, `SUM`, `INDEX` over the same array:
`=ROWS(SUMPRODUCT(A:A=A1)*H:J)` and `=ROWS(-H:J)` stay `#ERROR!`), an over-bound result whose shape fits the grid
from its anchor (`=1*H:J` or `=-H:J` in K1: Excel spills it or answers `#SPILL!` for an occupied cell, ours is
`#ERROR!`), and spills of other over-bound arrays (a function result that is not a lift, a lift whose range comes
from a name or a function). Only the `*` broadcast of corpus-bench/55049 and the DATE lift of
probes/lifted-range-rows C3 are oracle-backed; the unary operators, the 2^34-cell lifts (`=LEFT(A:A,1:1)`), the
other operators, an error operand
(`=1/0*H:J`, expected from probes/error-operand-broadcast) and 2^34-cell broadcasts (`=D:D+4:4`) follow the same shape rule unprobed (T-0484).
Evidence: corpus-bench/55049/1_55049_input.xlsx book1!B3:B23 (21 cells, excel-graph sidecar, 2026-10-26, gap
triage); test_lifted_range_bound.rs `broadcast_above_the_bound_that_cannot_spill_is_spill` and
`lift_above_the_bound_that_cannot_spill_is_spill`.
Tasks: T-0047, T-1420, T-1454, T-1460, T-0480, T-0483, T-0484

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
A single blank cell is a blank matrix too: `MINVERSE(B4)`, `MDETERM(D$1)`, `MMULT(100,C$9)` and `MMULT(1E-15,B11)`
with the cell blank are `#VALUE!`. Implemented in `Model::eval_matrix_arg` (matrix.rs, T-2164); any blank result, not only a direct reference, stays blank (a blank through INDEX or CHOOSE is unprobed). So does the empty value a lookup gives for a blank result cell: `MMULT(LOOKUP(TRUE,Data!B2:B10,Data!E11:E11),2.5)` is `#VALUE!` (probes/lookup-short-result-vector A20, T-2278; lookup.md "LOOKUP past a short result vector"), as do `MMULT(,2)`, IFERROR of a blank and `IF(TRUE,<blank>)` (probes/lookup-t2278-followups A1-A5). Evidence (excel-graph, 2026-10-20): gen-m5/cf893ec19f53a7fc
Calc!A204, gen-m5/fc81c2f704bef138 Calc!A381, gen-m5/3e134e52393061b1 Calc!A10, gen-m5/d9fb1e918971111b Calc!A213 (T-2164).

## MDETERM: partial-pivot LU, no singularity cutoff
Rule: MDETERM is Gaussian elimination with partial pivoting (the largest absolute entry of the column, first such row on
ties), rows updated as `a[r][k] -= (a[r][c] / pivot) * a[c][k]`, the determinant being the sign of the row permutation
times the product of the pivots. A tiny pivot is kept, not treated as 0, so a singular matrix can return its rounding
residual: `MDETERM({1,2,3;4,5,6;7,8,9})` (as a range) = 6.661338147750939E-16, not 0. An exactly-zero column gives 0
(`MDETERM({1,2;2,4})` = 0). Tiny nonzero pivots are kept (see the overflow/underflow line below). `MDETERM({1,2,3;4,5,6;7,8,10})` =
-2.9999999999999982, the same elimination's rounding (not -3.0000000000000004 as LAPACK-style variants give). Which
elimination fits was checked in IEEE doubles: dividing each product by the pivot instead of forming the multiplier first
gives -4.66E-15 and -3.0000000000000036, and no pivoting gives 0 and -3.
Evidence: probes/stats-evidence-floor-t1623 S!Q107, S!Q120, S!Q121 (excel-graph).
MDETERM of an overflowing or underflowing pivot product: `MDETERM({1E+200,0;0,1E+200})` is `#NUM!`; `{1E-20}` is 1E-20, `{1E-300}` 1E-300,
`{1E-17,0;0,1}` 1E-17, `{1E-200,0;0,1E-200}` 0 (underflow), `{1E+200}` 1E+200 (probes/minverse-singular-near-singular H1:H6, excel-graph, T-2067).
Implemented in: `fn_mdeterm`/`lu_decompose` (crates/rustsheet-engine/src/functions/math_and_trigonometry/matrix.rs, T-1751)
Status: VERIFIED (excel-graph)
Tasks: T-1751

## MINVERSE: singularity rule
Rule: Excel inverts every matrix whose partial-pivot LU has no exactly-zero pivot, however tiny,
and returns `#NUM!` only for an exactly-zero one. `MINVERSE({1,2,3;4,5,6;7,8,9})` (last partial-pivot LU pivot 1.11E-16) is a residual inverse
{-4503599627370498, 9007199254740992, -4503599627370496; 9007199254740996, -1.8014398509481984E+16, 9007199254740991;
-4503599627370498, 9007199254740992, -4503599627370495.5}; `{1E-20}` is 1E+20, `{1E-17,0;0,1}` is {1E+17,0;0,1}, and `{1E-300}` is
9.999999999999999E+299, the plain IEEE reciprocal `1/1E-300` (no scaling), not a quirk. `#NUM!` only for an exactly-zero pivot:
`{1,2;2,4}`, and also `{1,1;1,1.0000000000000002}`, because the literal 1.0000000000000002 is read at 15 significant digits as
1 (numbers.md "Formula number literals are read at 15 significant digits"), so that matrix is `{1,1;1,1}` with a last pivot of
exactly 0, not 2.22E-16. The inverse is the MDETERM elimination above (multiplier first) followed by forward substitution
with L and back substitution with U per identity column (subtract then divide by the pivot): in IEEE doubles that reproduces
all nine 3x3 residual entries bit for bit (T-2362). Dividing each U entry by the pivot before subtracting, or multiplying by
the pivot's reciprocal, misses some of them.
An inverse element past the double range makes the whole result `#NUM!`, not just that element: `MINVERSE({1E-200,1;0,1E-200})`
(back substitution overflows to -1E+400 in the corner) is `#NUM!` in every cell: `INDEX(...,1,1)` and `INDEX(...,2,2)`, which are finite on their own, are `#NUM!` too; the
non-overflowing `MINVERSE({1E-200,0;0,1E-200})` is {1E+200,0;0,1E+200}, although both matrices have the same underflowing determinant
(1E-400), so it is the overflowing inverse element, not determinant underflow, that gives `#NUM!`. Evidence: probes/minverse-overflow-computed-pivot Calc!I2:I8
(excel-graph, 2026-10-24, T-2386).
A 1x1 inverse past the double range is `#NUM!` too: A3 = `=3E-308-2.5E-308` is a kept subnormal (about 5E-309; `(3E-308-2.5E-308)=0` is FALSE,
numbers.md "Subnormal results become 0"), `MDETERM(A3)` keeps it (`(MDETERM(A3))=0` is FALSE) and `MINVERSE(A3)` / `MINVERSE(3E-308-2.5E-308)`
are `#NUM!`, consistent with its inverse (about 2E+308) overflowing; a kept subnormal with a finite inverse (just below 2.2E-308) was not probed (Calc!P1:P4, A3:A4; graph reports the subnormal cells as 0, decision 0042).
`MINVERSE(A1)` with A1 = `=1E-300*1E-10` is `#NUM!` for a different reason: `*` flushes 1E-310 to a true 0 (Calc!A5 `A1=0` is TRUE;
numbers.md "Subnormal results become 0"), so the 1x1 matrix is exactly singular (Calc!I1).
A near-singular pivot computed from cells is inverted: B1:C2 = 1, 1, 1, `=1+2^-52` (the cell holds 1+2^-52 exactly, unlike the
literal) gives `MDETERM` 2.220446049250313E-16 and `MINVERSE` {4503599627370497,-4503599627370496;-4503599627370496,4503599627370496}
(Calc!I4, J1:J4). That is consistent with the 15-digit literal explanation above for `{1,1;1,1.0000000000000002}` (the cell-held value is invertible, the literal is not; the literal's stored value was not probed separately). The 3x3
`{1,2,3;4,5,6;7,8,9}` read from cells (E1:G3) gives the same residual inverse as the array literal (Calc!K1:K9, spill L1:N3).
MINVERSE inverts a 1x1 pivot far below 2.2E-16: `MINVERSE(1.0000000000000085E-108)` is 9.999999999999914E+107 (before T-2362 ours was `#NUM!`),
gen-m5/9284730052aff46c Calc!A39 (excel-graph, 2026-10-20).
`MINVERSE((Data!E1+Data!E1)/TRUNC(2^31))` = `MINVERSE(9.313225746154786E-25)` is 1.0737418239999999E+24, the IEEE reciprocal
(before T-2362 ours was `#NUM!`): gen-m5/12a3d13f2a5aa246 Calc!A258 (excel-graph, 2026-10-22, T-2252 triage).
Evidence: probes/minverse-singular-near-singular (46 cells, excel-graph, T-2067); gen-m5/9284730052aff46c Calc!A39.
Implemented in: `fn_minverse` (crates/rustsheet-engine/src/functions/math_and_trigonometry/matrix.rs, T-2362)
Status: VERIFIED (excel-graph)
Tasks: T-2067, T-2362

## LOG, DECIMAL, BASE, ATAN2, ARABIC, DOLLAR, FIXED and NUMBERVALUE return arrays over array arguments (T-2159, T-2160)
Rule: these functions lift: given an array (or a multi-cell range from INDEX with a 0 row or column) they return an array,
which is `#SPILL!` when its spill range is blocked and shows its first element when it spills. Vertical results under a
filled column: `DECIMAL(..,INDEX(A1:D5,1E-15,4))`, `DECIMAL(INDEX(A8:D10,0,3)&..,25)`, `BASE(338,25,INDEX(A5:D11,0,0))`,
`ATAN2(..,INDEX(A2:A5,0,..))`, `ARABIC(INDEX(C5:E12,<blank>,3))` (so `ERROR.TYPE` of it is `#SPILL!`), `DOLLAR(INDEX(C4:C9,0),-2)`,
`DOLLAR(ABS(MUNIT(4)))`, `FIXED(MUNIT(3),3,<#REF!>)`, `NUMBERVALUE("(100)",INDEX(A4:B9,0,2),"#N/A")` and
`NUMBERVALUE(..,REPT(INDEX(B5:E10,0,0),..))` are `#SPILL!`. Horizontal results spill: `BASE(INDEX(C10:E10,0)*(7*B11),34)` shows
"0" and `LOG(INDEX(B8:E12,2,0.1)^-1.5,1^"?b")` shows `#VALUE!` (its first element: the base is `#VALUE!`).
Evidence (excel-graph, 2026-10-20, T-2114): gen-m5/043ee562f32eb901 Calc!A204, gen-m5/1d22a002d4f380b3 Calc!A115,
gen-m5/8a387c26d182e73c Calc!A42, gen-m5/aac6230c2fb40e1e Calc!A258, gen-m5/c0651f0e0fb544d0 Calc!A84, gen-m5/f0f205865a3a4328 Calc!A208,
gen-m5/6361fdf5993da954 Calc!A378, gen-m5/7d1a5d223624fc70 Calc!A380, gen-m5/d586a1e0c9896942 Calc!A364,
gen-m5/aac6230c2fb40e1e Calc!A213, gen-m5/ae9ed30812c4d62c Calc!A154; consistent with it but not evidence (a scalar `#VALUE!` base): gen-m5/5704666097cbc5b2 Calc!A153.
Argument kinds (excel-graph, 2026-10-20, T-2198, probes/t2198-m5-lift-argument-kinds, rows 10-27): every argument of all eight
functions lifts both over an array constant and over a 3x1 range given directly, on rows off the range (`ROWS` is 3 in each
case; element 2 equals the scalar control, a value in every row; ARABIC has one argument, so it has no mixed-shape cell, only row 36). So every argument is `Lift` (arrays and ranges), none is `LiftArrays`; unlike the
Analysis ToolPak functions (numbers.md "QUOTIENT, FACTDOUBLE and SERIESSUM argument modes"), no argument refuses a range.
| Function | Arguments that lift over a range and an array |
|---|---|
| LOG | number, base |
| DECIMAL | text, radix |
| BASE | number, radix, min_length |
| ATAN2 | x_num, y_num |
| ARABIC | text |
| DOLLAR | number, decimals |
| FIXED | number, decimals, no_commas |
| NUMBERVALUE | text, decimal_separator, group_separator |
Two array arguments broadcast: `{1;2}` with `{1,2,3}` is 2x3 (rows 29-35 A-C), and `{v;v}` with a 3x1 array gives `#N/A` in
the missing third row (D29-D35).
Implemented in: LOG, DECIMAL, BASE, ATAN2 and ARABIC on the SPEC-0011 driver with every argument `Lift`
(`functions/lift/kinds/general.rs`; `fn_log`, `fn_base`, `fn_atan2` in math_and_trigonometry/mathematical.rs, `fn_decimal`
in decimal_text.rs, `fn_arabic` in arabic_roman.rs; T-2159), and DOLLAR, FIXED and NUMBERVALUE
likewise (`functions/lift/kinds/text.rs`; `fn_dollar`, `fn_fixed`, `fn_numbervalue` in
text/string_format.rs; T-2160). The engine reads an element's arguments left to right, the
first error winning, then runs each function's own checks in their old order; two errors in one element are unprobed.
Status: VERIFIED (excel-graph) for the cells above; implemented for all eight (T-2159, T-2160).

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
The rest of the scalar parameters lift too (probes/m7-lift-take-drop-sortby, T-1960): SEQUENCE's columns and step
(`SEQUENCE(2,{1;3})`, `SEQUENCE(3,1,1,{1;2})` 2x1, each element 1x1), MAKEARRAY's cols (`MAKEARRAY(2,{1;2},LAMBDA(r,c,r+c))`
2x1, elements 2, 2), EXPAND's columns, WRAPROWS's pad_with (`WRAPROWS(A1:A3,2,{0;1})` 2x1), WRAPCOLS's wrap_count over
an array literal, TOCOL's scan_by_column, UNIQUE's exactly_once, SORTBY's later sort_orders and the columns of TAKE
and DROP (`TAKE(A1:B3,2,{1;2})` elements A1:A2 and A1:B2; `DROP(A1:B3,1,{1;2})` B2:B3, then `#CALC!`). EXPAND's
pad_with does not: `EXPAND(A1:A2,3,1,{0;1})` is one `#VALUE!`. Over an array value a lifted TAKE or DROP keeps the
top-left value, as EXPAND does: `TAKE({1,10;2,20;3,30},{1;2})`, `TAKE(SEQUENCE(3,2),{1;2})` and `TAKE(A1:B3*1,{1;2})`
are 1, 1, `DROP({1,10;2,20;3,30},{1;2})` 2, 3 (each element 1x1, ISREF FALSE; spilled directly 1, 1). Over a range its
elements are references and behave as a lifted OFFSET's (lookup.md "OFFSET, INDIRECT and TRIMRANGE lift into arrays
of references"): `=TAKE(A1:B3,{1;2})` spills `#VALUE!`, `#VALUE!`, `+0` gives `#VALUE!` per element, `SUM` is 2 (the
top-left cells), SUMPRODUCT 0, ISREF of the whole FALSE and of `INDEX(..,1,1)` TRUE; IF and CHOOSE hand the
references on (ISREF of an element TRUE, `SUM(IF(TRUE,..))` 2, `SUMPRODUCT(IF(TRUE,..))` 0), OFFSET over them lifts
again (`OFFSET(TAKE(A1:B3,{1;2}),1,0)` elements A2:B2, A2:B3), and `TAKE(NA(),{1;2})` is two `#N/A` elements.
A 1x1 array in a lifted parameter lifts to one element, but a single cell does not: `SEQUENCE({3})`,
`SEQUENCE({3},2)`, `SEQUENCE(A3+{0})` and `EXPAND(A1:A2,{3})` are the 1x1 1 (ROWS 1; `=SEQUENCE({3})` spills one
cell), `TOCOL(A1:B3,{0})` and `UNIQUE(D1:D3,{FALSE})` are 1x1, `TAKE(A1:B3,{2})` is one reference element A1:B2
(spilled `#VALUE!`), while `SEQUENCE(A3*1)`, `SEQUENCE(INDIRECT("A3")*1)`, `SEQUENCE(A3:A3)`, `SEQUENCE(IF(TRUE,A3))`
and `SEQUENCE(INDEX({3},1,1))` with A3 = 3 spill 3 rows: arithmetic over a one-cell reference is that cell's value
here, not a 1x1 array (row 39), and so through any operator: `SEQUENCE(-INDIRECT("A3")*-1)`,
`SEQUENCE(--INDIRECT("A3"))`, `SEQUENCE(INDIRECT("A3")%*100)`, `SEQUENCE(INDIRECT("A3")&"")`,
`SEQUENCE(IF(INDIRECT("A3")=3,3))`, `SEQUENCE(OFFSET(A3,0,0)+0)`, `SEQUENCE(--A3:A3)` and
`TAKE(A1:B3,--INDIRECT("A3"))` all have 3 rows, where the same operators over `{3}` give one element
(probes/m7-one-cell-operand-lift F1-F15). OFFSET lifts a 1x1 array too: `OFFSET(A1:B2,{1},0)` has ROWS and
COLUMNS 1 (F16, F17), as `DROP(A1:B3,{1})` (F6; its element has 2 rows, F7). A lifted element error stays in place:
`INDEX(SEQUENCE({1;0}),2,1)` is `#CALC!`. UNIQUE's and SORTBY's later elements follow their own rule ("UNIQUE's and
SORTBY's lifted elements" below).
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
Evidence: probes/m7-argument-modes (68 questions, excel-graph, 2026-10-09, T-1083); probes/m7-lift-take-drop-sortby
(Sheet1, excel-graph, 2026-10-21, T-1960: column E holds each row's formula, F:S its ROWS, COLUMNS and per element
`INDEX(INDEX(..,i,1),1,1)`, ROWS, COLUMNS and SUM of `INDEX(..,i,1)`; U1-U21 the scalar questions, W45-W75 direct spills). Three questions are left out because
excel-graph returns an error it cannot decode: `GROUPBY(D1:D3,A1:A2,SUM)` (heights differ), `MAP(A1:A3,LAMBDA(x,y,x))`
(too many parameters) and `GROUPBY(D1:D3,A1:A3,{1;2;3})` (an array as the function). A LAMBDA with an optional
parameter: `LAMBDA(x,[y],ISOMITTED(y))(1)` is TRUE, `(1,2)` FALSE, and an empty argument `(1,)` is omitted too (TRUE);
an omitted parameter's value is 0 (`LAMBDA(x,[y],y)(1)` = 0) and "" as text (`y&"|"` = "|");
`LAMBDA(x,[y],IF(ISOMITTED(y),x,x+y))` gives 1 for `(1)` and 3 for `(1,2)`; `LAMBDA([x],[y],ISOMITTED(x)+ISOMITTED(y))()`
is 2 (probes/lambda-optional-params A1-A8, excel-graph, 2026-10-11, T-1274; the engine matches all of them). The
case builder stores `[y]` as `_xlop.y` (xlsx-storage.md).
Ours: TEXTSPLIT, TEXTBEFORE, TEXTAFTER and REGEX* lift as above since T-1272 (rows 13-22, 55, K11:K13; text.md "TEXTSPLIT,
TEXTBEFORE, TEXTAFTER and REGEX* lift their text and scalar arguments"); SEQUENCE and MAKEARRAY lift since T-1834 (F1:H3,
F23:H23, K9, K10; SPEC-0011 "Array-valued functions"; SEQUENCE's columns and step and MAKEARRAY's cols lift by the spec's
per-parameter rule, which T-1960 confirmed; a 1x1 array argument lifts to one element since T-2298, a one-cell range read through any operator being the cell's value: `number_or_array`, `value_or_array` and `string_or_array` in cast.rs, OFFSET's, TAKE's and DROP's `lifts_one_by_one`; EXPAND's and WRAPROWS's 1x1 cases lift since T-1835); TOCOL's ignore and UNIQUE's by_col lift since T-1836 (F9:H9, K8, F12:H12; the array is read once and handed to every element whole, so UNIQUE of an empty array is empty, UNVERIFIED against Excel, while an error array fills the lifted shape since T-2313 ("TOROW, TOCOL, SORT and TEXTJOIN lifted arguments"); TOCOL's scan_by_column and UNIQUE's exactly_once lift by the spec's per-parameter rule, which T-1960 confirmed, but UNIQUE's later elements are not yet empty where Excel's are, T-2300);
EXPAND's rows and columns and WRAPROWS's and WRAPCOLS's wrap_count and pad_with lift since T-1835, and EXPAND's array pad_with is a whole-call `#VALUE!`;
TAKE and DROP lift since T-1838 (reference elements; IF, CHOOSE and OFFSET do not hand the grid on, and `TAKE(NA(),{1;2})` is one `#N/A`: T-2299);
none of the other lifts above (`#VALUE!` for a range, `#N/IMPL!` for an array); SORT reads a sort_index list and a
sort_order list of the same length since T-1837 (F10:H11; `fn_sort` with `Model::sort_number_list`, spill_functions.rs;
row and column lists alike, and with by_col the keys are rows: `SORT(A1:C3,{1,2},{1,-1},TRUE)` sorts columns by row 1,
then row 2 descending, and `SORT(A1:C3,1,{1,-1},TRUE)` is `#VALUE!`, probes/sort-error-args H57:J71). With rows
(1,9), (1,2), (0,5), a tie on the first key is broken by the next (`SORT(..,{1;2})` 0,5,1,2,1,9; `{1;2},{1;-1}`
0,5,1,9,1,2), a single sort_order applies to every key (`{1;2},-1` 1,9,1,2,0,5), a key may repeat (`{1;1}`) but more
keys than the array has columns (rows, by_col) is `#VALUE!` (`{1;1;1}`, `{1;2;1}`; `{1;1;2}` over three columns
sorts), a 2-D list (`{1,2;2,1}`) and an error element are `#VALUE!` as validation errors, so a later argument's error
wins (`SORT(..,IF({TRUE;FALSE},1,NA()),1/0)` is `#DIV/0!`), a range sort_index with a blank cell (D1 = 1, D2 blank) is
`#VALUE!` (A9; read as 0 here), and text "1" in an array constant is key 1 (A13); an error cell inside a range list is
read as an error element by analogy, UNVERIFIED (probes/sort-index-key-ties, excel-graph, 2026-10-20, T-1837);
GROUPBY and PIVOTBY exist since T-1322 and T-1326 ("GROUPBY and PIVOTBY" below; `_xleta.` names work since T-1466, see "Functions passed by name" below); INDEX of a scalar result is `#VALUE!` (T-1242); `ROWS(LAMBDA(x,x))`
is 1 since T-1511 ("An uncalled LAMBDA is one value" below); a BYROW row result is not `#CALC!`. 91 of the 174 scored cells pass (F35:H35, PIVOTBY, since T-1326).
Status: VERIFIED (web)
Tasks: T-1083 (generator modes)

## UNIQUE's and SORTBY's lifted elements
Rule: when UNIQUE's by_col or exactly_once, or SORTBY's sort_order, lifts over entries that are not all equal, only
the first element is its call's top-left (for UNIQUE, the first whose call is not an error; an error element stays);
every later element is empty, even where its entry equals the first. An empty element spills as 0, and ISBLANK of it is
TRUE, `INDEX(..,i,1)&"|"` is "|", TYPE 1, SUM of it 0, and ROWS, COLUMNS and `INDEX(INDEX(..,i,1),1,1)` of it `#VALUE!`
(SORTBY's cells and, for UNIQUE, probes/m7-unique-lift-elements). When every entry is equal, each element is the
top-left of its call.
- UNIQUE in detail (probes/m7-unique-lift-elements, excel-graph, 2026-10-22, T-2300; D1:D3 = x,y,x, E1:E2 = FALSE,TRUE):
  - The empty element spills as 0, ISBLANK of it is TRUE, `&"|"` is "|", TYPE 1, SUM 0 and ROWS `#VALUE!`.
  - The same holds over each array argument tried: a range, an array constant, `D1:D3&""`, a one-cell range D1, a scalar 5, a
    two-column range and a range E1:E2 of entries.
  - The first element is the call of the first entries: `UNIQUE(D1:D3,FALSE,{1;2})` is "y", empty.
  - Equal means the same type and value. `{0;FALSE}` differs (x, empty) although both calls give the same array, and so
    does `{0.2,1E+15}`.
  - With two lifted arguments, both must be uniform. `UNIQUE(D1:D3,{FALSE;FALSE},{FALSE;TRUE})` is x, empty;
    `{TRUE;TRUE},{FALSE;FALSE}` is x, x.
  - A 2x2 broadcast (`{FALSE;TRUE}` against `{FALSE,TRUE}`) or a 2-D by_col keeps only the top-left element.
  - An error stays: `{FALSE;#N/A}` is x, `#N/A`. And the first element whose call is not an error is the one kept:
    `{#N/A;FALSE}` is `#N/A`, x.
  - An error array is every element's result: `UNIQUE(SUMPRODUCT(<mismatched sizes>),HSTACK(0.2,1E+15))` is `#VALUE!`,
    `#VALUE!` (gen-m7/8044dce50616cfe9 Calc!A1434:B1434).
  - Ours (T-2300): the driver option `CallResult::ArrayFirst` (SPEC-0011 rule 2, "As built (T-2300)"). INDEX of any empty
    value is `#VALUE!` there (`scalar_as_one_element_array`). Only the empty element is evidenced (m7-lift-take-drop-sortby
    L25); other empty values in INDEX's first argument (`INDEX(IFERROR(B1,1),1,1)` with B1 blank) are `#VALUE!` too
    (probes/m7-unique-lift-corners, T-2348).
- With A1:A3 = 3,1,2 and C1:C3 = 2,1,3: `SORTBY(A1:A3,C1:C3,{1;-1;1})` is 1, empty, empty (it spills {1;0;0}:
  probes/sort-error-args H97-H99); `{1;-1}` 1, empty; `{-1;1}` 2, empty; `{1;1;-1}` 1, empty, empty; E1:E3 = 1,-1,1
  the same as the literal; `{1;1}`, `{1;1}+0` 1, 1; `{1;1;1}` 1,1,1; `{-1;-1;-1}` 2,2,2. The same over B1:B3 =
  30,10,20 (10 or 20 first) and text D1:D3 = c,a,b ("a" or "b" first), over a 2-column array and with other keys
  (F1:F3 = 1,2,3: `{1;-1;1}` 3, empty, empty).
- SORTBY's later elements are `#VALUE!` instead, SUM `#VALUE!` too, where an entry is not 1 or -1 (`{1;2}`,
  `{1;"x"}`: element 2) or the call has a second by_array/sort_order pair (`SORTBY(A1:A3,C1:C3,{1;-1},A1:A3,1)`,
  `SORTBY(A1:A3,C1:C3,1,A1:A3,{1;-1})` and `{-1;1}`: element 2). A uniform later sort_order lifts
  (`SORTBY(A1:A3,C1:C3,1,A1:A3,{1;1})` and `{-1;-1}`: 1, 1). `{1,-1}` is 1x2 and `{1}`, `{-1}` 1x1.
- With D1:D3 = x,y,x: `UNIQUE(D1:D3,FALSE,{FALSE;TRUE})` is "x", empty; `{TRUE;FALSE}` "y", empty;
  `{FALSE;FALSE;TRUE}` "x", empty, empty; `UNIQUE(D1:D3,{FALSE;TRUE})` and `{TRUE;FALSE}` "x", empty;
  `UNIQUE(D1:D3,{FALSE;FALSE})` "x", "x"; `UNIQUE(D1:D3,FALSE,{TRUE;TRUE})` "y", "y".
- TOCOL's scan_by_column (`TOCOL(A1:B3,0,{FALSE;TRUE})`: element 2 is 1, ROWS 1) and the other M7 lifts do not show it.
- Uniform entries spanning several columns keep the call's top-left only in the first column (probes/m7-unique-lift-corners,
  excel-graph, 2026-10-23, T-2348; D1:D3 = x,y,x, A1:B3 = 1,2,3 / 10,20,30): the other columns' elements are empty (spill 0).
  `UNIQUE(D1:D3,FALSE,{FALSE,FALSE;FALSE,FALSE})` is x, x in column one and 0, 0 in column two; a 2x3 and a 3x2 likewise
  (`UNIQUE(A1:B3,FALSE,{FALSE,FALSE,FALSE;FALSE,FALSE,FALSE})`: 1, 1 then 0s; `{FALSE,FALSE;FALSE,FALSE;FALSE,FALSE}`: 1, 1, 1 then 0s),
  and a 1x3 `UNIQUE(A1:B3,{FALSE,FALSE,FALSE})` is 1, 0, 0 while a 3x1 `{FALSE;FALSE;FALSE}` is 1, 1, 1. It does not depend
  on by_col (TRUE, FALSE and the by_col-only form give the same cells). Entries that are not all equal keep only the top-left
  (`{FALSE,TRUE;FALSE,FALSE;FALSE,FALSE}`: 1 then 0s). The same as gen-m7/2c156ba98ef0220a.case.json Calc!A1284 (3x3: 100
  down column A, 0 in B:C; excel-graph, 2026-10-22, T-2254 triage). Ours: UNIQUE only (`CallResult::ArrayFirstColumn`,
  `empty_past_first_column` in lift/driver.rs) since T-2348. UNVERIFIED: whether an error element past the first column stays
  (ours: it does, `UNIQUE(D1:D3,FALSE,{"a","a"})` is unprobed), and SORTBY's uniform 1xN sort_order, which keeps every
  element's top-left in ours (`CallResult::ArrayFirst`; with a second by_array/sort_order pair its emptied elements are `#VALUE!`);
  both belong with T-2349's SORTBY probes.
- Lifted-element corners (same probe): text entries are `#VALUE!` elements whatever their case (`UNIQUE(D1:D3,FALSE,{"a";"A"})`
  and `{"a";"a"}` are `#VALUE!`, `#VALUE!`: exactly_once is not read as text), so the case question never arises; `{0;-0}` is
  x, x (equal). A call error from a non-error entry stays and the first good element is kept: `{"abc";FALSE;TRUE}` is
  `#VALUE!`, x, empty; `{FALSE;"abc";TRUE}` is x, `#VALUE!`, empty. INDEX of an empty value from IFERROR
  (`INDEX(IFERROR(AZ1,1),1,1)`, `LET(x,IFERROR(AZ1,1),INDEX(x,1,1))`, AZ1 blank) is `#VALUE!`, as for UNIQUE's empty element.
Evidence: probes/m7-lift-take-drop-sortby (sheet Sort, Sheet1 rows 25, 26, 29, 30, 35-37; excel-graph, 2026-10-21, T-1960).
UNVERIFIED for SORTBY (taken from UNIQUE's rule, unprobed; T-2349):
- a leading invalid entry (`{2;1}`): ours `#VALUE!`, then the next element's top-left;
- a mixed `{1;-1;2}`: ours 1, empty, `#VALUE!`, with the error per slot;
- an error array: ours one scalar error;
- an error by_array: ours the error in every element.
Ours: implemented. The driver option `CallResult::ArrayFirst` serves UNIQUE (T-2300) and SORTBY (T-1839). SORTBY's sort_orders lift (`repeat: [WholeByCaller, Lift]`), and its later elements are `#VALUE!` once the call has a second by_array/sort_order pair (`LiftOptions::later_error`).
Status: VERIFIED (web)
Tasks: T-2300 (UNIQUE), T-1839 (SORTBY)

## Scalar arguments lift: shapes, element errors, and the Analysis ToolPak range rule
Rule: a function whose scalar argument gets an array lifts, one call per element, and the arguments
broadcast like the arithmetic operators. A worksheet function also lifts over a multi-cell range in
that argument. An Analysis ToolPak function does not take a multi-cell range in the scalar
arguments probed (listed below): the range makes its call `#VALUE!`, with no implicit intersection,
in a dynamic formula and in a legacy one. The `weekend` argument of WORKDAY.INTL and
NETWORKDAYS.INTL is the exception: it lifts over a range (`INDEX(WORKDAY.INTL(45000,5,B1:B3),2)` =
45007; dates.md "Date function argument modes").
- **Shape.** The result has the largest height and the largest width among the lifted arguments. A
  size-1 dimension repeats, and a position past a shorter argument is `#N/A`:
  `ROWS(PMT({0.1;0.2;0.3},{10;20},100))` is 3, with element 3 `#N/A` and element 2
  `PMT(0.2,20,100)`. `NORM.DIST({1;2;3},{0,1},1,TRUE)` is a 3x2 grid. In
  `PMT({0.1,0.2;0.3,0.4},{10,20,30},100)` column 3 is `#N/A` in both rows, and
  `PMT({0.1;0.2},{10,20,30},100)` is a full 2x3 grid. A 60-cell range against `{10;20}` is 60 rows,
  `#N/A` from row 3 on. EDATE pads the same way (`EDATE({45000;45031;45061},{1;2})` element 3).
- **Errors per element.** Each element follows the scalar rules on its own. An element that fails
  is an error in its own position only (`PMT({0.1;"x"},10,100)` is a number, then `#VALUE!`;
  `NORM.DIST(2.5,0,{1;-1},TRUE)` is a number, then `#NUM!`). In each element the leftmost failing
  argument decides. This rests on one cell: element 2 of `PMT({1;#N/A},{#DIV/0!;#NUM!},100)` is
  `#N/A` (H33). Element 1, where only nper fails, is `#DIV/0!` (H31, a sibling formula with the same element-1 inputs). A scalar error, or scalar text
  that is not a number, next to an array is broadcast: `PMT(NA(),{10;20},100)` is 2 rows with
  element 2 `#N/A` (H28-H29). `EDATE(NA(),{1;2})`, `PMT("x",{10;20},100)` and `EDATE("x",{1;2})` are
  2 rows and every element is the scalar's own error: `INDEX(EDATE(NA(),{1;2}),1)` and `2` are `#N/A`,
  `INDEX(PMT("x",{10;20},100),1)` and `2` and `INDEX(EDATE("x",{1;2}),1)` and `2` are `#VALUE!`
  (probes/scalar-lift-open-a W18-W23, T-1777; H30, H44, H45 are the ROWS).
- **Elements are typed values.** An array element is read like the same value typed as the
  argument. EDATE's `{TRUE;45000}` gives `#VALUE!`, then 45031, because a typed boolean date is
  `#VALUE!` for EDATE. NORM.S.DIST's `{TRUE;"1"}` gives NORM.S.DIST(1) twice, which is also
  `NORM.S.DIST(TRUE,TRUE)`. A range cell is read like a referenced cell: over F1:F4 = 0.1, blank,
  "x", `=1/0`, `PMT(F1:F4,10,100)` gives a number, -10 (the blank cell is 0), `#VALUE!`, `#DIV/0!`.
- **Ranges in worksheet functions lift, also on their own rows.** On the rows of the range, where
  an intersection would succeed, `ROWS(PMT(C1:C60,10,100))`, `ROWS(PMT(0.1,D1:D60,100))`,
  `ROWS(NORM.DIST(E1:E60,0,1,TRUE))` and `ROWS(NORM.DIST(2.5,0,1,B1:B60))` are 60, and so is
  `ROWS(BITAND(13,D1:D60))` in a later argument. An explicit `@` intersects:
  `PMT(@C1:C60,10,100)` is one number.
- **Ranges in Analysis ToolPak functions are refused, in the arguments probed.** On the range's own rows,
  `ROWS(EDATE(A1:A60,1))`, `ROWS(EDATE(45000,B1:B60))`, `ROWS(YEARFRAC(45000,46000,B1:B60))`,
  `ROWS(BESSELJ(2.5,B1:B60))`, `ROWS(DEC2BIN(9,D1:D60))`, `ROWS(DELTA(B1:B60,1))`,
  `ROWS(EFFECT(C1:C60,4))` and `ROWS(EFFECT(0.1,D1:D60))` are `#VALUE!`. A reference returned by a
  function is refused too: `EDATE(IF(TRUE,A1:A60),1)` and `EDATE(INDEX(A1:A60,0,1),1)`. A one-cell
  range or reference is its value (`EDATE(A1:A1,1)` = 45031, `EDATE(INDEX(A1:A60,2),1)` = 45032).
  An operation on a range is an array, and that lifts: `ROWS(EDATE(A1:A60+0,1))` and
  `ROWS(EDATE(45000,M1:M60*1))` are 60, `ROWS(EDATE(TAKE(A1:A60,2)+0,1))` is 2. An explicit `@`
  intersects (`EDATE(@A1:A60,1)` on row 3 is 45033, and `#VALUE!` off the range's rows). The
  arguments probed over a range are every first scalar argument of the financial and engineering
  ATP functions (fin-argument-modes, eng-argument-modes), and these later arguments: EDATE's months,
  YEARFRAC's basis, BESSELJ's n, DEC2BIN's places and EFFECT's npery. Other later arguments are
  refused too (probes/scalar-lift-open-a, T-1777, each with a scalar control in column Z that
  works): `ROWS(F(..))` is `#VALUE!` for PRICE's `frequency` and `basis` (Y1, Y2), COUPDAYS's and COUPNUM's
  `frequency` (Y3, Y4) and COUPDAYS's `basis` (Y5), ACCRINT's `basis` and `frequency` (Y6, Y7), CONVERT's
  `from_unit`, `to_unit` and number (Y8-Y10), COMPLEX's `suffix` (Y11), IMSUB's and IMDIV's second argument
  and IMSUB's first (Y12, Y13, Y15), IMPOWER's `n` (Y14) and DOLLARDE's and DOLLARFR's `fraction` and
  DOLLARDE's first argument (Y16-Y18). So the rule holds in every later argument probed.
  probes/eng-later-argument-lift (T-1383) probes the rest of the engineering later arguments on the
  range's own rows: BESSELI's, BESSELK's and BESSELY's `n`, the `places` of every base conversion,
  COMPLEX's `i_num`, DELTA's `number2`, GESTEP's `step` and ERF's `upper_limit` are `#VALUE!` (H1-H11,
  H16-H19), while BITOR's and BITXOR's `number2` and BITLSHIFT's and BITRSHIFT's `shift_amount` lift
  to 60 rows, on and off the range's rows (H12-H15, L71), as BITAND's do (H7, H20).
- **The other Analysis ToolPak-era functions** (probes/scalar-lift-open-a Y19-Y30, T-1777). QUOTIENT
  (either argument), FACTDOUBLE and SERIESSUM's `x`, `n` and `m` refuse a range like the rest (`#VALUE!`,
  Y19-Y21, Y24-Y26). SERIESSUM's `coefficients` over a range is read whole (Y27, `SERIESSUM(2,0,1,B1:B3)` is
  7, -d W31). MULTINOMIAL, GCD and LCM give one result over a range (only MULTINOMIAL's value is probed, -d W30) (ROWS 1, Y22, Y23,
  Y28-Y30; `MULTINOMIAL(B1:B3,2)` = 59.999999999999986, -d W30).
  probes/math-atp-array-lift (T-1802) shows that QUOTIENT, FACTDOUBLE and SERIESSUM's `x`, `n` and `m`
  lift over an array, as the other ATP arguments do, while SERIESSUM's coefficients and MULTINOMIAL, GCD
  and LCM stay whole over an array too (numbers.md "QUOTIENT, FACTDOUBLE and SERIESSUM argument modes").
- **A range next to an array in an Analysis ToolPak call.** The call lifts over the array only.
  The range is passed whole to each element's call, which refuses it. So the shape comes from the
  array, and the probed elements are `#VALUE!`, on the range's rows and off them.
  `EDATE({45000;45031},M1:M60)` and `EFFECT({0.1;0.2},M1:M60)` are 2 rows with both elements
  `#VALUE!` (-b C, D, H, I). `EDATE(A1:A60,{1;2})` is 2 rows with both elements `#VALUE!` (-b F, G;
  -open-a W24, W25, W30). `EDATE({45000,45031},M1:M60)` is one row of two columns (-b L; -open-a W28,
  W29) with both elements `#VALUE!` (W26, W27), also off the range's rows (W70-W73).
- **Error precedence is the kernel's own order.** Within an element the first failing argument in the
  function's own scalar check order decides, and a refused range counts as a `#VALUE!` argument in its
  position (probes/scalar-lift-open-a, -d, T-1777). For most functions that is the leftmost:
  `INDEX(PMT({#NUM!},{#N/A},100),1)` is `#NUM!` and the reversed `INDEX(PMT({#N/A},{#NUM!},100),1)`
  is `#N/A`, and so are EDATE's with `{#NUM!},{#N/A}` and `{#N/A},{#NUM!}` (-open-a W14-W17), with H33's
  leftmost-error element confirmed by W14-W17. Beside a refused range: `EDATE(NA(),A1:A60)` is `#N/A` and
  `EDATE(A1:A60,NA())` is `#VALUE!` (W1, W2; ROWS the same, W11, W12), `EFFECT(1/0,C1:C60)` is `#DIV/0!` and
  `EFFECT(C1:C60,1/0)` is `#VALUE!` (W3, W4), `EDATE("x",A1:A60)` and `EDATE(A1:A60,"x")` are `#VALUE!`
  (W9, W10), and ACCRINTM does the same in its first four arguments, also next to an array (-d W8-W15). A
  scalar error beside a range and an array follows the same order (`YEARFRAC(NA(),A1:A60,{1;2})` is `#N/A`,
  `YEARFRAC(B1:B60,NA(),{1;2})` is `#VALUE!`; W5, W8). YEARFRAC is the exception that shows it is the kernel's order, not
  the argument position: it checks `basis` first, then the start, then the end, in the scalar call too
  (`YEARFRAC(1/0,45000,NA())` and `YEARFRAC(NA(),1/0,1)` are `#N/A`, `YEARFRAC(45000,NA(),1/0)` is
  `#DIV/0!`, -d W1-W5), so `INDEX(YEARFRAC(1/0,{45000;45031},B1:B60),1)` is `#VALUE!` (the range is the basis, -open-a W6) and
  `INDEX(YEARFRAC({45000;45031},B1:B60,NA()),1)` is `#N/A` (W7). SPEC-0011's "leftmost" is therefore wrong
  for YEARFRAC (T-1800). The basis is also range-checked before the dates are read:
  `INDEX(YEARFRAC(NA(),{45000;45031},{5;1}),1)` is `#NUM!` and element 2 `#N/A` (probes/yearfrac-basis-order
  A13, A14; dates.md "Date function argument modes").
- **Legacy (non-dynamic) formulas.** A worksheet function's range gets the usual implicit `@`:
  legacy `=PMT(C1:C60,10,100)` shows `=PMT(@C1:C60,10,100)` and gives the intersected value on
  the range's rows and `#VALUE!` off them. Legacy `=SUM(PMT(C1:C60,10,100))` is that one value,
  where the dynamic formula sums 60. An Analysis ToolPak argument's bare range gets no `@`:
  legacy `=EDATE(A1:A60,1)` and `=YEARFRAC(A1:A60,46000)` keep their text and are `#VALUE!`, on
  the range's rows too. An operation on the range gets the `@`, as in an aggregate's argument
  (references.md "Legacy formulas intersect the ranges in value arguments"): legacy
  `=EDATE(A1:A60+0,1)` shows `=EDATE(@A1:A60+0,1)` and is 45033 on row 3. Under SUMPRODUCT, an
  array context, nothing is intersected. Legacy `=SUMPRODUCT(PMT(C1:C60,10,100))` sums 60,
  `=SUMPRODUCT(EDATE(A1:A60,1))` is `#VALUE!`, and `=SUMPRODUCT(EDATE(A1:A60+0,1))` and
  `=SUMPRODUCT(NORM.DIST(E1:E60,0,1,TRUE))` lift. Legacy `=SUM(PMT({0.1;0.2},10,100))` sums both
  elements (-40.12681517653708), like the dynamic formula. Its FORMULATEXT is the written text, with no `@` (`=SUM(PMT({0.1;0.2},10,100))`; probes/scalar-lift-open-a AA1, AB1, T-1777).
Evidence: probes/scalar-lift-shapes H1-H50, J1-J8, K1-K8; probes/scalar-lift-shapes-b (each
question on row 3 and row 70); probes/scalar-lift-shapes-c (legacy formulas and their FORMULATEXT).
All three: excel-graph, 2026-10-17, T-1767. The per-family argument lists are in statistics.md,
financial.md, engineering.md, dates.md, lookup.md and the "argument modes" entries above. The ATP
range rule is the one MROUND and ISEVEN follow ("CEILING/FLOOR family and MROUND lift over
arrays", information.md "ISEVEN/ISODD over a range").
- **A dynamic ATP call with a range** (probes/scalar-lift-open-a AA3, AB3, AC3, T-1777).
  `=EDATE({45000;45031},A1:A60)` entered as a dynamic formula spills the array's shape: `ROWS(AA3#)` is 2 and
  both cells are `#VALUE!` (AA3, AA4). Its FORMULATEXT is the written text with no `@` and no
  braces (AB3), the same for `EDATE({45000;45031},B1:B60)` (AA6, AB6), for the plain `=PMT({0.1;0.2},10,100)`
  (AA9, AA10, AB9: -16.27453948825116 in AA9 and the spill cell AA10 -23.852275688285914) and for the legacy `=EDATE(A1:A60,1)` (AA12, AB12). So
  FORMULATEXT does not show whether Excel saves the cell as a dynamic-array formula (the `cm` flag of the
  file); that needs the file itself (desktop Excel, M14), and the static shape above is what the cell does.
- **A multi-cell reference that a function returns lifts in a worksheet function.** Where the same
  reference in an ATP argument is refused, `ROWS(PMT(IF(TRUE,C1:C60),10,100))`,
  `ROWS(PMT(INDEX(C1:C60,0,1),10,100))`, `ROWS(NORM.DIST(OFFSET(E1,0,0,60),0,1,TRUE))`,
  `ROWS(PMT(INDIRECT("C1:C60"),10,100))` and `ROWS(PMT(CHOOSE(1,C1:C60),10,100))` are 60, as is the operation
  `PMT(C1:C60+0,10,100)` (probes/scalar-lift-open-a W31-W36, T-1777).
- **A 3-D range is `#REF!` in a lifted worksheet function and `#VALUE!` in an ATP one.**
  `Sheet1:Sheet2!A1:A2` is accepted by SUM (182002, probes/scalar-lift-open-b W1) but not by ROWS (`#VALUE!`,
  W12). `ROWS(PMT(Sheet1:Sheet2!C1:C2,10,100))` is `#REF!` (W2, W3), also in a later argument (W6), for
  NORM.DIST (W10, W11), BITAND (-d W29), for a one-cell 3-D range (`PMT(Sheet1:Sheet2!C1,10,100)`, -d W21,
  W22) and beside an array (`INDEX(PMT({0.1;0.2},10,3D),1)` and 2, -d W17, W18; `INDEX(PMT(3D,{10;20},100),1)`,
  -d W25). EDATE gives `#VALUE!`: `ROWS(EDATE(3D,1))` (W4), `ROWS(EDATE(45000,3D))` (W7) and, beside an array,
  both elements (-d W19, W20, W26). The shape beside an array is the array's (-b W8, W9: ROWS 2). `ISREF(3D)` is FALSE
  and `3D+0` is `#REF!` (-d W24, W27, W28). SPEC-0011 assumed `#VALUE!` in both kinds (T-1801).
- **A stacked result past the grid** (probes/scalar-lift-open-c, -c2, -c3, T-1777): rows past the grid are `#NUM!`,
  columns past it are allowed; the rule and its cells are "Stacked results past the grid" above (T-1803).
- Not probed: lifted elements that are arrays or references (TAKE, DROP, OFFSET, INDIRECT).
Ours: EDATE, EOMONTH, WEEKNUM and YEARFRAC follow the rule (T-1770): every argument is
`LiftArrays` in the lifting table, so they lift over an array, refuse a multi-cell range or a
returned reference (TAKE and INDEX included) with `#VALUE!`, and a 3-D range with `#VALUE!` (multi-cell: scalar-lift-open-b W4, W5, W7, -d W19-W20, W26; one-cell, `EDATE(Sheet1:Sheet2!A1,1)`, and in WEEKNUM, EOMONTH and YEARFRAC: scalar-lift-open-e W1-W9, also beside an array, W2-W3, whose shape stays the array's, W8), and get no legacy `@` on a bare
range. DROP, TRIMRANGE and XLOOKUP results that span several cells are refused too, and one-cell results are their value (scalar-lift-open-e W10-W22, T-1801); all their cells in the three scalar-lift-shapes probes pass, and since T-1800 so do scalar-lift-open-a W6, W7 and -d W1-W7 (YEARFRAC checks basis, start, end; the driver hands a refused range to the kernel as `#VALUE!` in its slot and picks no error by position). Since T-1383
the engineering functions lift too, and since T-1372 28 financial functions (PMT, PV, FV, NPER,
RATE, IPMT, PPMT, ISPMT, RRI, PDURATION, SLN, SYD, DDB as `Lift`; EFFECT, NOMINAL, DOLLARDE,
DOLLARFR, ACCRINTM and the discount securities and T-bills as `LiftArrays`), so the PMT, EFFECT and
ACCRINTM cells here pass (scalar-lift-open-a W4, W14, W15; -b W9; -d W8-W18). Since T-1822 ACCRINT,
the COUP* family, DURATION, MDURATION and PRICE are `LiftArrays` as well (scalar-lift-open-a Y1-Y7). Since T-1290 the
44 statistical functions whose every argument is scalar and whose kernels are bounded lift as `Lift`
(statistics.md "Statistical function argument modes"), so NORM.DIST's cells here pass (H15, H16, H20, H21, H34,
H35, J8, K8), and NORM.S.DIST's H38 and H40 now lift and miss only by its known last-digit residue, as the scalar H39
does (statistics.md "The normal CDF is not always correctly rounded", T-0968). Since T-1848 the quantile functions
BETA.INV, BETAINV, F.INV, F.INV.RT, FINV, T.INV, T.INV.2T, TINV and CONFIDENCE.T lift as `Lift` too, their searches
capped (statistics.md "Quantile searches: per-call cap and lifting"), and since T-1872 so do CHISQ.DIST,
CHISQ.DIST.RT, CHIDIST, CHISQ.INV, CHISQ.INV.RT, CHIINV, GAMMA.DIST, GAMMADIST, GAMMA.INV, GAMMAINV, POISSON and
POISSON.DIST, their incomplete gamma capped (statistics.md "Incomplete gamma: per-call cap and lifting"), and since
T-1873 HYPGEOM.DIST and HYPGEOMDIST, HYPGEOM.DIST's cdf sum capped (statistics.md "HYPGEOM.DIST: per-call cap and
lifting"). The other families
listed in SPEC-0011 do not lift yet (`#N/IMPL!` for an array; a range is intersected in a dynamic
formula). SPEC-0011's rollout tasks fix them.
Status: VERIFIED (excel-graph)
Implemented in: `Function::arg_kinds` (crates/rustsheet-engine/src/functions/lift/kinds/),
`Model::lift_call` and its read phase (functions/lift/call.rs), the broadcast `Model::lift_grids`
(functions/lift/driver.rs), and the static-analysis hooks `lifted_signature`,
`lifted_static_result` and `lifts_arrays_at` (expressions/parser/static_analysis/lifted.rs);
kernels `edate_of`, `eomonth_of` (functions/date_and_time/month_shift.rs), `weeknum_of`
(weeknum.rs) and `yearfrac_of` (yearfrac.rs); the financial kernels in functions/financial/
(annuity.rs, discount.rs, rate_conversions.rs, accrint.rs, financial_bonds.rs) and financial_depreciation.rs; the
statistical kernels in functions/statistical/ (normal.rs, binom.rs, beta.rs, fisher.rs, t_dist.rs and the rest; the
capped quantile search in inverse_beta.rs, the capped incomplete gamma in incomplete_gamma.rs, the capped
hypergeometric cdf in hypegeom.rs).
Tasks: T-1767 (SPEC-0011), T-1770, T-1290, T-1372, T-1383, T-1236, T-1822, T-1872, T-1873

## GROUPBY and PIVOTBY
Rule: groups are keyed like UNIQUE (case-insensitive text, first spelling shown) and always sorted ascending:
numbers < text < FALSE < TRUE < blank, with the blank-key group displayed as 0 when spilled (a true blank inside the array) and kept last even
when sorting descending. Function inputs keep source order, and the total row applies the function to all
(filtered) values, not to the group results (AVERAGE total 13.05 = 130.5/10). Filler cells of total rows, the
corner and PIVOTBY combinations without records are empty text `""`, not blanks (a combination summing to 0
shows 0). Labels: "Total"; with subtotals (depth 2), the grand total is "Grand Total".
field_headers: omitted auto-detects headers without showing them; 0 none, 1 yes/hidden, 2 generated
("Row Field 1", "Value 1"), 3 yes/shown (over data without a header row it consumes the first record);
4 is `#VALUE!`. A LAMBDA returning an array gives `#VALUE!` per cell. `HSTACK(SUM,AVERAGE)` adds columns
and a header row of function names; `VSTACK(SUM,MAX)` repeats each group row per function with a
name column. PIVOTBY relative_to: 0 column total, 1 row total, 2 grand total; with one field per axis 3
equals 1 and 4 equals 0. Out-of-range, fractional or non-numeric sort_order, a filter of another height or shape, an all-FALSE filter, a filter
element that is an error or text other than "TRUE"/"FALSE" and field_relationship 2 are `#VALUE!`; an error value in any
argument wins over them (first in argument order).
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
A LAMBDA array result is shown as `#VALUE!`, but the cell holds the array itself, a nested array (probes/t2360-array-lambda-holidays
P, excel-graph, 2026-10-23, T-2360): with `LAMBDA(v,v)`, `TYPE(INDEX(GROUPBY(..),r,c))` is 64 and INDEX returns the group's values
(`ROWS`x`COLUMNS` 2x1 for a two-record group, 4x1 for the total), even a 1x1 array for a one-record PIVOTBY cell (P!B3). One LAMBDA
over several value columns gives each column its own array. A consumer that lifts over the result reads each cell's top-left, so
`SUM(--ISERROR(GROUPBY(..)))` counts no error (P!C1, C9, C17, C25) unless a nested array's top-left is one (`{#N/A;3}`, P!C33 = 1),
and the gen-m7 REGEXEXTRACT whose return_mode is such a GROUPBY gets a valid mode, not `#VALUE!` (gen-m7/21b4eef18424ce94 A1504),
while MAP over the result is not an error (T-2592 below). Blank keys show as true blanks in INDEX (`TYPE` 1,
`&""` is "", G!A12, B3), not 0. Ours since T-2593: each such cell is a nested array (`ArrayNode::Nested`), shown as `#VALUE!`, which
INDEX returns (TYPE 64) and an element-wise lift reads as its top-left (P!B3:E37, C1:C33 and gen-m7/21b4eef18424ce94 A1504 pass);
0 for a blank key until T-2854, which keeps the blank key a true blank (`ArrayNode::Empty`) inside the array: INDEX gives
TYPE 1 and `&""` "", COUNT skips it (probes/pivotby-ttest-array2: `COUNT(PIVOTBY(..))` 108, and the TTEST that reads it,
gen-m9/c2684066d8df263e Calc!A214), while a spilled blank element still displays 0. IFERROR and IFNA read a blank array
element as 0, as they read a blank cell of a multi-cell range: `TEXTJOIN("|",FALSE,IFERROR(GROUPBY(..),"#"))` shows the blank
key as "0" (probes/groupby-sort-filter-args G!B15, B55, groupby-sort-filter-args2 G!B10; `if_error_like`, logical/mod.rs).
IF passes a blank element through as a blank: `TTEST(..,IF(TRUE,PIVOTBY(..)),1,2)` equals the TTEST without the IF
(probes/pivotby-ttest-array2 P!A41, P!A31). Whether IFERROR reads blanks of other arrays (FILTER, INDEX lifted over blank
cells, VSTACK) as 0, and what N and arithmetic do with a blank element, is unprobed (T-2999); ours reads every array's
blank as 0 in IFERROR and IFNA.
T-2592 (probes/t2592-nested-groupby-cells, excel-graph, 2026-10-24; `GROUPBY(D!A1:A5,D!C1:D5,LAMBDA(v,v))` is 3x3: keys a, b, Total by
key, C, D; group a `{1;2}`/`{10;20}`, b `{3;4}`/`{30;40}`, Total 4x1; the shown headers are off). Nested cells in practice:
- A LAMBDA whose body is a one-cell reference (`LAMBDA(x,D!D2)`) still displays `#VALUE!` in every spilled cell (probes/groupby-lambda-forms
  G!K193:K198, excel-graph, 2026-10-16), but INDEX reads such a cell as its value, not an error (`TYPE` 1, 10; A21): the cell is
  reference-like, shown as `#VALUE!` and read as the referenced value, never a nested array. A two-parameter `LAMBDA(x,y,x)` also displays
  `#VALUE!` (G!K217:K222) but holds a nested array like `LAMBDA(v,v)` (`64|2x1|3`, A22). A computed 1x1 result is nested too
  (`LAMBDA(v,{7})` and `LAMBDA(v,SEQUENCE(1))` `64|1x1`, A33, A34), and subtotal cells (two row fields, `total_depth` 2) are: a
  one-record group `64|1x1`, a two-record subtotal `64|2x1`, the Total `64|4x1` (A35:A41).
- MAP, BYROW and BYCOL over the result are not errors: a scalar step returns an array of the result's shape (3x3, 3x1, 1x3; A1:A5) and the
  LAMBDA is handed each nested cell whole: `MAP(..,LAMBDA(x,ROWS(x)))` is 1 for a key, 2 for group b's C cell, 4 for the Total's
  (A100:A102), `TYPE(x)` is 64 for a nested cell (A103) and `SUM(x)` its sum (7 for `{3;4}`, A105); BYROW's row and BYCOL's column
  hold the nested cells too (`TYPE(INDEX(r,1,2))` 64, A106, A107). A LAMBDA that returns `x` itself gives `#CALC!` (ERROR.TYPE 14, A104).
  MAP over a PIVOTBY result with a scalar step is fine as well (A108, A109), so t2311 P!D12's "E" is not MAP itself: its step is `IFERROR(x&"",..)`, and `&""` on a nested cell is still an array (A18), which a MAP step cannot return (`#CALC!`, A104).
- REDUCE visits every one of the 9 cells with the nested array as `x` (a count 9; `a+ROWS(x)` 19, A14, A15).
- Aggregates read through the nested cells, flattening them: SUM 220, COUNT 16, COUNTA 19 (keys count; A6:A8); AVERAGE is `#N/A`
  (ERROR.TYPE 7, A9), ARRAYTOTEXT `#CALC!` (14, A10), TEXTJOIN and CONCAT `#VALUE!` (3, A11, A12). `&""` over the whole result keeps
  the 3x3 shape (A13).
- `+0`, `--` and `&""` on one INDEXed nested cell leave it an array (`64|2x1|3`, A16:A18).
- `ZZ1#` over a spilled result reads every nested cell as `#VALUE!` (`SUM(ZZ1#)` and `INDEX(ZZ1#,2,2)` ERROR.TYPE 3, A19, A20).
- Two levels: a LAMBDA returning a GROUPBY gives a nested cell that is the inner 3x3 result, whose own cells are nested again
  (`64|3x3|a`, inner INDEX(..,2,2) `64|2x1`, A23, A24); so one level is not the limit in Excel.
- TAKE, CHOOSECOLS, VSTACK, SORT, FILTER and an INDEX of a whole row or column all keep the nested cells (A26:A32).
Ours since T-2593 (SPEC-0018): the nested cells of rules 1, 3 and 8 (A16:A18, A22:A41), the one-cell reference body read by INDEX
as its value (A21), and BYROW's row and BYCOL's column holding nested cells (A106, A107). Since T-2594: MAP and REDUCE hand
the LAMBDA each nested cell as its array (A100:A105, A109, A15), and t2311 P!D12 is `#CALC!`; ARRAYTOTEXT over a result
holding one is `#CALC!` (A10); and the aggregates follow "Aggregates over nested GROUPBY and PIVOTBY cells" below. An element-wise lift
of a two-level cell reads the inner top-left (assumed, T-2609), of a reference cell `#VALUE!` (unprobed, T-2609); a lifted INDEX
element and an operator over the whole result read a nested cell as `#VALUE!` (unprobed, T-2609). Excel's operator does not read it as `#VALUE!`:
`MEDIAN({1;2;"";4},GROUPBY(..,LAMBDA(z,z))-x,..)` is `#N/A`, as MEDIAN over nested cells, where ours is `#VALUE!`, and TEXTJOIN over
`IFERROR((GROUPBY(..)-x)&"",..)` fails as over nested cells (probes/t2475-stat-subexpressions P!A62, A63; T-2623, fix T-2686).
Aggregates over nested GROUPBY and PIVOTBY cells (probes/t2594-nested-aggregates, excel-graph, 2026-10-24, T-2594, over the T-2592
data): SUM, COUNT and COUNTA flatten a nested cell (SUM 220, COUNT 16, COUNTA 19 over GROUPBY; over PIVOTBY SUM 220, COUNT 16,
COUNTA 32; `SUM(1,GROUPBY(..),2)` 223; A26:A29). Every other aggregate tried is `#N/A`: MIN, MAX, MINA, MAXA, AVERAGEA, PRODUCT,
SUMSQ, MEDIAN, STDEV.S, VAR.P, HARMEAN, GEOMEAN, DEVSQ, AVEDEV, MODE.SNGL, KURT, SKEW, LARGE, SMALL, PERCENTILE.INC (A1:A21),
`AVERAGE(1,GROUPBY(..))` (A25), AVERAGE and MIN/MAX over PIVOTBY (A30:A32); gen-m9 MAX, HARMEAN and AVERAGE over a
`LAMBDA(z,z)` result agree (06b4e8f5de409d5e A366, 68d8ffb69ff739d8 A285, 7c20351562a2d71d A381). CONCAT is `#VALUE!` (A18),
SUMPRODUCT 0 (A22), ROWS and COLUMNS the outer shape (A23, A24). A one-cell reference body's cell is `#VALUE!` to PRODUCT and MIN
(gen-m7/bcad709c30cfdf64 A13, gen-m9/2a18b2f5a4dc43ac A371). Range-only functions (MAXIFS, RANK.EQ, COUNTBLANK, SUBTOTAL,
AGGREGATE 1-13) cannot take the result. Ours since T-2594: the walker's `NestedReading` (functions/util/aggregate_args.rs):
SUM, COUNT and COUNTA flatten, charging the walk budget; the AVERAGE family, MIN, MAX, PRODUCT, SUMSQ and the A-variants give
`#N/A`. T-2625 (probes/t2625-nested-consumers, excel-graph, 2026-10-24): VARA, STDEVA, VARPA, STDEVPA, AND, OR and XOR are `#N/A`
too (A1:A7), IMSUM and IMPRODUCT `#VALUE!` (A8, A9), MODE.MULT, QUARTILE.INC and PERCENTILE.EXC `#N/A` (A10:A12). MODE.SNGL,
LARGE, SMALL and the PERCENTILE/QUARTILE family read their data outside the walker and give `#N/A` through `nested_array_error`
(crates/rustsheet-engine/src/nested_array.rs); SUMPRODUCT reads a nested cell as 0. Inferred, unprobed: the `#N/A` beats an
error element anywhere in the array (ours checks for a nested cell first), and the 0 holds for multi-argument SUMPRODUCT. COUNT over a PIVOTBY with several key and
value columns (`COUNT(PIVOTBY(Data!C1:C12,Data!B1:D12,Data!B1:E12,LAMBDA(y,y)))` 140, A13; gen-m9/e85daedce5c2675e A35's 142)
differs from ours only in the blank keys, which Excel shows as true blanks in the row keys and in every column-key row
(A28:A31; ours was 0, so COUNT 165): fixed by T-2854 (see above).
Implemented in: crates/rustsheet-engine/src/nested_array.rs (`NestedArray`, `ArrayNode::scalar`),
crates/rustsheet-engine/src/functions/lookup_and_reference/group_functions.rs (`Model::nest`),
crates/rustsheet-engine/src/functions/lookup_and_reference/index.rs (`index_from_array`),
crates/rustsheet-engine/src/functions/lift/driver.rs (`LiftedArg::element`)
`field_headers` 2 over two keys and two values generates
"Row Field 1", "Row Field 2", "Value 1", "Value 2"; `field_headers` 3 with `total_depth` -1 puts the header row first, then
Total. A header row over no records (`GROUPBY(A1,D1,f,1)`), a non-function (`5`) and fractional options (`total_depth`
0.5, `field_headers` 2.5) are `#VALUE!` (ERROR.TYPE 3). An empty-text key (`=""`) is its own group, separate from the blank
key: it shows `""`, sorts as text (before "a"; after it descending), and the blank key stays last.
T-1326 follow-up (probes/pivotby-followup, 63 PIVOTBY questions over the same Data sheet, excel-graph, 2026-10-16):
- Headers: omitted detects them from the values only, as in GROUPBY (P!A1, P!A1021). Shown headers (3) add a first row with the
  column field names joined with ", " as text ("Product, Year"; a consumed record's 2023 becomes the text "2023", P!A61, P!A421)
  above the first column key, also when the Total column comes first (P!A1181), and a last row with the row field names and the
  value name of every column, Total columns included (P!A441, P!A521). Generated (2): "Column Field 1, Column Field 2",
  "Row Field n", "Value n" (P!A21, P!A81). Several value columns name each one only with shown headers.
- Totals: -1 puts the Total column right after the row keys and the Total row right after the headers (P!A101, P!A121, P!A1241).
  ±2 with one field on that axis is `#VALUE!` (P!A401, P!A701); 3 is `#VALUE!` (P!A381). Two values with col_total_depth 0 have
  no Total columns; with row_total_depth 0, no Total row (P!A561, P!A581).
- Sort orders follow GROUPBY: signed indices over the axis' key fields, then the values (col_sort_order only the first
  value: see below); 0, 1.5 and past the last column are
  `#VALUE!` (P!A141, P!A721, P!A161, P!A1001); scalar 2 over two row fields is the second key (P!A1041, P!A1061); `{-1}` is
  accepted (P!A641); an error argument is passed on (P!A1261). With two value columns, `{3}` sorts by the second and `{-2}`
  by the first descending (P!A1321, P!A1361); a key and a value column in one array are `#VALUE!` on both axes (`{1,3}`,
  `{1,2}`: P!A1341, P!A1381). Error precedence as in GROUPBY: an error value in any option beats an invalid option before or
  after it and a non-function (`field_headers` 4 with col_sort_order `NA()`, `NA()` with row_total_depth 3, function 5 with
  `NA()`: all `#N/A`, P!A1401-A1461). A value sort orders by the aggregate (the Total column or row,
  computed even when it shows `""`: P!A601), ties in key order (P!A741), numbers before text (P!A1121), the blank key not pinned.
  Quirks: rows are sorted by a value only when there is a Total column: with col_total_depth 0 they keep the key order (P!A781,
  P!A261, P!A1221), while columns sort by a value with or without the Total row (P!A821, P!A841). With two row fields and an
  explicit row_total_depth 0 or 1, every level sorts by its own aggregate (parents by theirs, P!A861, P!A881); with two column
  fields and an explicit col_total_depth only the children sort by value, the parents keeping their key order (P!A1141, P!A1161).
  With the depth omitted, a value sort over two fields shows subtotals and "Grand Total" as depth 2 does (rows P!A201, P!A901;
  columns P!A941, whose parents then are sorted by their aggregate), and so does GROUPBY (`GROUPBY(A2:B11,D2:D11,SUM,,,3)`,
  P!A921), unlike its explicit depth 0 (followup2 G!A261).
- Filter: a numeric column filters on non-zero (P!A1301); scalar TRUE filters nothing (P!A621); another height (P!A281) or
  `{TRUE;FALSE}` (P!A1581) is `#VALUE!`, and so is an error or other text in it, not the error (P!A1481, P!A1601). With a
  header row, the filter may be as tall as the fields or as the records, aligned with them (P!A1501, P!A1521, P!A1541,
  P!A1561), as in GROUPBY. Short col_fields are `#VALUE!` (P!A301). An error function is passed on (`NA()`, P!A1281); `5` is `#VALUE!` (P!A761).
- Cells: errors in values make their cells and totals that error (P!A321); a LAMBDA array result is `#VALUE!` per cell (shown; the cell holds a nested array, see the T-2360 paragraph above), `""`
  where there are no records (P!A341); text results show as text (P!A541). An array relative_to (here a filter column put one
  argument too far) is lifted per element: a 10x1 array (P!A681).
Unprobed: whether one row of text values with no rows below is a header (the engine takes it as data), and where
error keys sort against text, booleans and the blank key (the engine uses `sort_key_cmp`'s order: after booleans,
before blank).
T-1323 follow-up (probes/groupby-sort-filter-args, 83 questions summarised in one cell each, excel-graph, 2026-10-16;
B<n> is question n). sort_order: a scalar must be a whole number from 1 to the last column, negative for descending;
0, -3 past the columns, 1.5, 1.9, 2.5, -1.5, 0.5, -0.5, the text "2", TRUE, a blank cell and `""` are all `#VALUE!` (no
truncation and no coercion, B1-B14), `NA()` is `#N/A` (B12). An array may be a column (`{2;-1}`, B15); it names key
columns only, or a value column as its one element (`{-3}` = -3, B21-B22); a key column next to a value column (`{1,3}`,
`{3,1}`, `{2,3}`, `{-1,3}`, `{-2,-3}`), two value columns (`{2,3}`, `{3,-2}` over two values), a fraction, an error or
text element (`{1.5,2}`, `{1,#N/A}`, `{1,"2"}`) are `#VALUE!` (B16-B30). Sorting by a value: equal values keep the
ascending key order in both directions (COUNT ties North, South, blank ascending and descending, B32-B33; East before
South at 9 Units, B27-B28); text sorts as text (`"t-3"` < `"t45"` < `"t7"`, B34), numbers before text (B35), an error
value last ascending and first descending (B36-B37); with two keys -3 orders parents by their sum descending and the
children descending inside each (B22). The header row and a top Total stay in place (B38-B40).
filter_array: each element counts like a condition, a number is TRUE unless 0, a blank is FALSE, the text "TRUE"/"FALSE"
counts (B42, B44, B52); other text, `""`, any error element (even on a row the filter drops) are `#VALUE!` and not the
error (B43, B45-B46, B53); a scalar keeps or drops everything (2 keeps, `"x"` is `#VALUE!`, `NA()` is `#N/A`: B47, B54-B55);
a row of 10, a 10x2 array, a longer one and an all-blank range are `#VALUE!` (B48-B51). With a header row the filter
may be as tall as the fields (the header's element is ignored, whatever it holds: args2 B1-B5, B29-B30 below) or as tall as the
records, aligned with them (B56-B59); field_headers 3 still shows the headers.
Error precedence: every error value among the arguments wins over any invalid argument, before or after it (sort_order
`NA()` beats total_depth 3, a short filter, an all-FALSE filter, a short values range and the function 5), and among
error values the first argument wins (field_headers or total_depth `1/0` before sort_order `NA()`, sort_order before
filter_array before field_relationship): B61-B78. field_relationship: 3, -1, 0.5 are `#VALUE!`, `NA()` `#N/A`, and the
text "1" counts as 1 (with one total row, the default layout): B79-B83.
Review follow-up (probes/groupby-sort-filter-args2, 37 questions in column B, excel-graph, 2026-10-16; "args2 Bn"
below): the filter element beside a header row is never read, so text, `""` or an error there is ignored (args2 B1, B3,
B5, B29-B30), while an error beside a record is `#VALUE!` (B31-B35). A filter `VSTACK(NA(),Data!C2:C11=2023)` is `#N/A`
(B2, B4; `1/0` gives `#DIV/0!`), and B36 (`ROWS(VSTACK(NA(),Data!C2:C11))&...`, one `#N/A`) points to VSTACK returning the
scalar error rather than an array (confirmed directly, T-1695: probes/vstack-scalar-error, excel-graph 2026-10-16: a scalar error argument makes VSTACK or HSTACK that one error, whatever its position (first, last or middle) and with a row/column array beside it: `ROWS`, `COLUMNS` and `INDEX(..,2)` of `VSTACK(NA(),{1;2})`, `VSTACK({1;2},NA())`, `VSTACK({1;2},NA(),{3;4})` and the HSTACK forms are all `#N/A`, `ISERROR` TRUE, and `VSTACK(1/0,{1;2})` is `#DIV/0!` throughout). "true"/"True" count like "TRUE" (B6-B7, B10); numeric text
"1" does not (B8-B9: `#VALUE!`). `{#N/A}` and `IF({TRUE},NA())` are `#VALUE!` as sort_order and as filter_array
(B11-B14), while a one-cell reference to `=1/0` is `#DIV/0!` in either place, also before total_depth 3 (B16-B18). A
value sort over three keys is hierarchical at every level (B19-B21), and a parent's value is the function of all its
rows, not of its children's values: AVERAGE ranks East (15) before West (18.125) and `LAMBDA(x,100-SUM(x))` West (27.5)
before East (55), where the sum, the mean or the largest child would disagree (B22-B26). Boolean aggregates sort
numbers < text < FALSE < TRUE < errors, reversed descending (B27-B28, B37).
T-1325 follow-up (probes/groupby-pivotby-functions, -functions2, -functions3: 97 GROUPBY and PIVOTBY questions over the
same Data sheet, excel-graph, 2026-10-16; "functions", "functions2", "functions3" below):
- Function arrays: a row of functions (`HSTACK(SUM,MAX)`, also `VSTACK(HSTACK(..))`) adds a value column per function under a
  row of labels (`""` over the keys), with or without shown headers (functions G!A1, G!A21; under PIVOTBY's column keys, above
  the shown field names: P!A121, functions P!A621); a column of them (`VSTACK(SUM,MAX)`, also `TRANSPOSE(HSTACK(..))`) repeats
  every group row and total row once per function after a label column, which shown headers head with `""` (functions G!A61,
  G!A101, functions2 G!A681, P!A21-A61; PIVOTBY P!A601). One function in HSTACK is that function, no label row (G!A601); nested
  HSTACKs flatten (functions2 G!A581). Labels are the function names as written ("SUM", "STDEV.S", "PERCENTOF") and "CUSTOM"
  for a LAMBDA (functions G!A181, G!A521, G!A721); two or more LAMBDAs are "CUSTOM1", "CUSTOM2", ... numbered per LAMBDA in order (named functions between them
  do not count: `HSTACK(L,SUM,L)` is CUSTOM1, SUM, CUSTOM2), as columns and as rows; a single LAMBDA among named functions stays
  "CUSTOM", a lone LAMBDA in HSTACK shows no label row (G!A201), and a LAMBDA with a label value takes the label (probes/groupby-custom-labels G!A1, G!A41, G!A81, G!A121,
  G!A161, G!A201, G!A241; probes/pivotby-col-subtotals-values2 S!A181, S!A193; -values3 S!A1; T-1854). Unprobed: whether a
  labelled LAMBDA counts in the numbering (the engine counts only unlabelled ones; `HSTACK(L,L,"lab")` was undecodable). Custom labels: values beside a column of functions label its rows
  (`HSTACK(SUM,"a")` or `HSTACK(SUM,5)` is a label column of "a" or 5 before the values, a third element `HSTACK(SUM,5,MAX)`
  ignored: functions G!A221, functions2 G!A121-A181, also in PIVOTBY: P!A101), values below a row of functions label its
  columns (`VSTACK(SUM,5)`: a label row of 5; functions2 G!A161). `#VALUE!` (ERROR.TYPE 3): a 2-D array of functions, an array
  constant `{1,2}`, a value before the function (`HSTACK(5,SUM)`) (functions G!A201, G!A261, functions2 G!A101); an error element
  is that error (`HSTACK(SUM,NA())`, `VSTACK(SUM,NA())`: `#N/A`, G!A781, functions2 G!A701).
- Functions and value columns pair up: n functions over one value column, one function over n, or n over n with function i
  on column i (`HSTACK(SUM,MAX)` over Sales and Units is SUM of Sales next to MAX of Units, and `VSTACK` gives the same pairs as
  rows; PIVOTBY too, with its Total columns filled: functions G!A41, G!A81, functions2 P!A1, P!A21); 3 over 2 and 2 over 3 are
  `#VALUE!` (functions2 G!A1-A61). PIVOTBY's Total columns are `""` only for one function over several value columns (P!Y97,
  functions P!A701), and then only with two or more column groups ("GROUPBY and PIVOTBY: column totals over several
  value columns").
- Detected headers (`field_headers` omitted) are shown, as 3 shows them, with a row of two or more functions (GROUPBY and
  PIVOTBY: functions2 G!A641, functions3 P!A1), not with a column of them, a labelled one, or `field_headers` 1 (functions3
  G!A1-A81, P!A31).
- sort_order past the keys counts the output value columns: 3 is the second function of a row (functions G!A141, functions2
  G!A221, PIVOTBY P!A181); a column of functions sorts by its first function (functions2 G!A201, G!A261, P!A201, P!A221), each
  group's rows staying in function order.
- PIVOTBY's col_sort_order counts one value column past the column fields, whatever the value columns or functions:
  with one column field 2 sorts the column groups by the first value column and ±3, ±4 and 5 are `#VALUE!` over one, two
  or three value columns and over `HSTACK(SUM,MAX)` (also with row_total_depth 0 or an explicit col_total_depth); with two
  column fields ±3 sorts by the first value column and ±4 is `#VALUE!` (probes/pivotby-sort-order-past-keys S!A1-A72,
  A145-A167; pivotby-total-columns-one-group S!A73; pivotby-col-subtotals-values S!A111). Arrays follow (`{3}` `#VALUE!`
  with one field, sorting with two: S!A162-A165). row_sort_order instead counts every output value column, as GROUPBY's
  sort_order does: 3 and 4 are the second and third of three values, 5 is `#VALUE!` (S!A73-A144, two functions: S!A155-A161).
- Arguments: a function passed by name gets its minimum argument count: one (SUM, MEDIAN, SUMPRODUCT, ABS, STDEV.S), two
  (PERCENTOF, CORREL, PEARSON, RSQ, COVARIANCE.P, and IF, ROUND, SUMIF, whose array results are `#VALUE!` per cell); none or
  three or more (NOW, TEXTJOIN, XLOOKUP) make the whole result `#VALUE!` (functions G!A421, functions2 G!A281-A501). A LAMBDA gets
  one argument when it needs at most one (`LAMBDA([x],1)`; `LAMBDA(x,[y],ISOMITTED(y))` is TRUE: an optional parameter is not
  passed) and two when it needs two; three is `#VALUE!` for the whole result (functions G!A381, G!A401, functions2 G!A601).
- The second argument is the relative set's values, as an array (ISREF FALSE) in source order without the header row: in
  GROUPBY every kept row, for every group and the total (`LAMBDA(x,y,ROWS(y))` is 10, 4 with a filter keeping 4; functions
  G!A301-A361, G!A441, G!A681). So `LAMBDA(x,y,SUM(x)/SUM(y))` equals PERCENTOF (G!A281), also sorted by it (G!A661).
- A LAMBDA whose result is a reference, written (`Data!D2`, `Data!D2:D3`), picked by IF or returned by INDEX, is `#VALUE!` in
  every cell (functions G!A541-A581; PIVOTBY P!A841).
Unprobed (T-1325): mismatched heights inside a function array (the engine pads with `#N/A`, so the result is `#N/A`); which of
several error elements wins (the engine: the first evaluated); a labelled row of several functions with detected headers (the
engine does not show them); a LAMBDA without parameters (the engine: `#VALUE!`). CORREL, PEARSON and RSQ over arrays of different sizes
are `#N/A` per group in Excel, `#VALUE!` here, and their total rows differ in the last digit (functions G!B421, functions2
G!B281, G!B301): a CORREL/PEARSON/RSQ gap, T-1707.
T-1706: every function passed by name gets its catalog minimum argument count (1 or 2; 0 or 3+ is `#VALUE!` overall):
COVARIANCE.S, SLOPE, SUMX2MY2, INTERCEPT get two (grouped as usual), RANK, PERCENTILE, MOD two (`#VALUE!` per cell),
SUBSTITUTE none (`#VALUE!`; probes/groupby-eta-arity G!A1-A141). SLOPE and INTERCEPT over arrays of different sizes are
`#N/A` per group in Excel, `#VALUE!` here (G!B21:B25, G!B61:B65): T-1867.
Unprobed (T-1706): minimum-0 functions that take optional arguments (ROW, COLUMN, SHEET, SHEETS, RANDARRAY) are
extrapolated from NOW (`#VALUE!` overall); only NOW is probed.
T-1327 follow-up (PIVOTBY relative_to; probes/groupby-pivotby-functions P!A1-A841 show each cell's set with
`LAMBDA(x,y,SUM(y))`, and -functions3 P!A61-A181):
- A cell's relative set is a row selection crossed with a column selection, each all records, the cell's own row (column)
  item, or that item's parent, the item with its last key field dropped (a single field's parent, and a Total's, is all):
  0 is all rows x the cell's column, 1 the cell's row x all columns, 2 all x all, 3 the cell's row x its column's parent,
  4 its row's parent x its column. So with one field per axis 3 equals 1 and 4 equals 0 (P!A1-A81). Two row fields with 4:
  East/Apple in 2023 gets East in 2023 (15), its Total cell East (45), the Total row the column (P!A181); two column fields
  with 3: the row under the column's parent (Apple: 15, 64 in the Total row, P!A261); both with 4: West/Apple in
  Apple/2023 is 0 (P!A321). With subtotals (depth 2) a subtotal row or column is the item of its key prefix, whose
  parent is one field shorter: with 4 a row subtotal East gets the column (35, 86) and 130.5 in the Total column, with 3 a
  column subtotal Apple gets the row (P!A341-A441). A cell without records stays `""` without a call (P!A1). Three row
  fields with 4 (functions3 P!A61) agree with "one field up" but do not separate it from the top field (unprobed).
- PERCENTOF and the two-parameter LAMBDAs get it (P!Y25-A73 in the anchor case; HSTACK(SUM,PERCENTOF) and VSTACK: functions
  P!A641, functions2 P!A81); with several value columns each column has its own set and the Total columns stay `""` (P!A701,
  functions3 P!A181); a filter shrinks every set (P!A661). One-argument functions ignore it (P!A561).
- relative_to is read as a number (`"1"` and TRUE are 1); 5, -1 and 0.5 are `#VALUE!`, also with SUM (P!A461-A581). An array is
  lifted: each element is the PIVOTBY's top-left cell, the `""` corner, or `#VALUE!` for an invalid element (`{0;1}` is
  `""`,`""`; `{5;1}` `#VALUE!`,`""`; ROWS x COLUMNS of an element 1x1: P!A721, P!A741, AB1), `{1}` too (a 1x1 `""`); a
  one-cell reference is its value, a blank cell 0 (the column total) and 2023 `#VALUE!` (probes/groupby-pivotby-functions4
  P!A221-A261).
- Value sorts by a two-argument function: rows sort by their Total column cell's value, so with 4 over two row fields the
  parents tie at the grand total and keep their key order while the children tie at their parent's total (functions3
  P!A91); with 1 by their own rows (P!A121). Columns sort by their Total row cell's value the same way: over two column
  fields sorted by value descending, 3 ties the years at their product's total (Apple 2023 before Apple 2024, both 64), 0
  and 4 sort them by their own records (2024, 49, before 2023, 15), 1 and 2 tie at the grand total, also with the column
  subtotals the omitted depth adds, and over one column field 3 ties at the grand total (probes/groupby-pivotby-functions4
  P!A1-A201, PERCENTOF too).
Unprobed (T-1327): parents over three or more fields (the engine: one field up) and a parent group's set in a three-level
sort (the engine: the group one level up); an error
element in an array relative_to (the engine: that error) and an array relative_to with another invalid argument (the
engine: the other argument's error, whole).
Ours: GROUPBY (T-1322, functions/lookup_and_reference/groupby.rs): the shared `Grouping` core (hash grouping on
`cell_key` with -0 as 0, sorting with `sort_key_cmp` and numbers compared exactly), a LAMBDA or eta-name function called
once per group and value column through one `LambdaCaller`, field_headers and auto-detect, total_depth 0/±1, `""`
fillers, several value columns, the output size checked with `check_array_size` before any call. Since T-1323:
sort_order (`Model::sort_order` into a `SortOrder`; `Grouping::sort_by_keys`, and `Grouping::sort_by_aggregate` for a value
column, hierarchical over the key prefixes with one call per group and parent), filter_array (`Model::filter_rows`), the
error values first in argument order (`Model::evaluate_options`), and scalar field_relationship 0/1 (1 is `#VALUE!` with
total_depth ±2, and the default layout otherwise). Since T-1324: subtotals (grouping.rs), the table sorted by a value and
an array field_relationship (below). Since T-1325 (functions/lookup_and_reference/group_functions.rs): the function
argument is read as a grid of functions and labels (`Model::function_elements`, `Model::function_argument`), paired with
the value columns (`Functions::pair_with`) and laid out as columns or rows (`Functions::shape`); each function is called
through `Model::lambda_caller` with one or two arguments, its body read in reference context so a reference result, also
one an XLOOKUP, TAKE or DROP picks (probes/lambda-body-picked-references A66-A90, T-2075), is `#VALUE!`
(`LambdaCaller::keeping_references`); PERCENTOF passed by name sums each relative set once and divides
(`percent_of`, shared with the PERCENTOF function), and a two-parameter LAMBDA is charged against the cell-visit cap (see
"GROUPBY and PIVOTBY: two-parameter LAMBDAs and the cell-visit cap").
PIVOTBY (T-1326, functions/lookup_and_reference/pivotby.rs): two `Grouping`s of the same records (rows, columns), both size
checked while grouping and before any call; the grid with `""` cells, totals 0/±1 on both axes, the column totals of one function
over several values placed as below (T-1700), shown and generated headers, key and value sorts with the quirks above, and GROUPBY's argument readers
(`evaluate_options`, `sort_order`, `filter_rows`: their rules hold for PIVOTBY too, P!A1321-A1601 below);
`Grouping::sort_by_aggregate` takes whether parents sort by their aggregate (rows, GROUPBY) or keep their key order
(PIVOTBY columns). Function arrays since T-1325, as in
GROUPBY. Since T-1327 (functions/lookup_and_reference/relative_to.rs): `Model::relative_to` reads the argument into a row
and a column selection, `RelativeSets` builds every set the grid uses once, after the sorts, in one pass over the records
per pair of key-prefix levels (groups, subtotals and totals alike; `Grouping::prefix_ids`), the grid cells, subtotal
lines included, look theirs up, value sorts take `RelativeTo::of_sort` (the parent group from
`Grouping::sort_by_aggregate`), and an array is lifted by `Model::lifted_relative_to` without any call.
Unprobed (PIVOTBY): an error header cell in a joined column name (the engine: that error); a joined column name longer
than a cell holds (the engine: `#VALUE!`, after the 32,767-character cell limit in coercion.md "Text longer than 32,767
characters"); value sorts over three or more fields (rows: every level by its aggregate; columns: the last); and
col_total_depth -1 with a row value sort (the engine sorts, as with 1: the Total column exists).
T-1324 follow-up (probes/groupby-subtotals, 49 questions over the same Data sheet at G!A1, A31, ..., A1441, each with its
size in column Z and the ERROR.TYPE of its first cell in AA, excel-graph, 2026-10-16; "G!An" below is that case's sheet G):
- Depth: n shows the subtotals of the first n - 1 key fields, so 2 over three keys subtotals only the first (G!A1, G!A31)
  and 3 both (G!A61, G!A91), and |n| may not exceed the key fields (3 over two keys, ±2 over one: `#VALUE!` as above;
  four keys take 4, G!Z1231, but not 5, G!AA1261). Positive: each subtotal right after its groups, the deepest first;
  negative: right before them, the shallowest first, with "Grand Total" on top (G!A91). A subtotal row shows its keys,
  then `""`; its function gets all its groups' rows in source order (`LAMBDA(x,ARRAYTOTEXT(x))`: East "10, 30, 5",
  G!A331). Shown headers stay first (G!A301).
- Sorts: a key sort orders the groups as without subtotals, the blank key last (G!A241, G!A271); a value sort is
  hierarchical, parents by their aggregate (G!A151, G!A181, G!A361, G!A1411). With the depth omitted, a value sort over
  n fields shows the subtotals of every level, as depth n (three keys: G!A121, PIVOTBY rows G!A1321), and an explicit
  depth is kept (G!A151: 2; followup2 G!A261: 0).
- Table (field_relationship 1) sorted by a value: every group by its own value, ignoring the parents ("South, Banana
  -3; North, Pear 7; 0, Apple 9; West, Banana 12.5; East, Apple 15 ..."), and no subtotals, the depth omitted included
  (G!A391-A481, three keys G!A1381).
- Array field_relationship: lifted per element, the top-left cell of each element's GROUPBY or its error: an invalid
  sort_order is `#VALUE!` per element (G!A511), an error element its error (G!A541), a row array gives a row (G!A571),
  field_headers 3 the header cell (G!A601), 1 with depth 2 `#VALUE!` beside 0's "East" (G!A631), and an error value in
  an earlier argument fills every element (`NA()` as sort_order: two `#N/A`, G!A1351).
- PIVOTBY: each axis has its own depth and label: row subtotals end in a "Grand Total" row (P!A97, G!A661 for -2) while
  the Total column stays "Total", and column subtotals put the parent's key over `""` and a "Grand Total" column
  (P!M97, G!A691 for -2, which puts it first) while the Total row stays "Total"; both at once fill the subtotal
  crossings (G!A721). Shown headers put the column field names over the first group column, never over a subtotal or
  total column, and the value name under every column (G!A841, G!A871). Column parents sort by their aggregate when
  there are column subtotals (G!A961, G!A991; P!A941), keeping the key order otherwise (P!A1141). A row value sort with
  col_total_depth 0 keeps the key order (above) but its omitted row depth still shows the subtotals (G!A1021). Three
  row fields with depth 2 or 3 as GROUPBY (G!A1051, G!A1291). Row subtotals with two values leave both Total columns
  `""` (G!A751).
- Column subtotals of one function over several value columns are not the subtotals: in
  `PIVOTBY(A2:A11,B2:C11,D2:E11,SUM,,,,2)` (G!A781) the Apple and Pear subtotal columns and both Grand Total columns are
  `""`, while Banana (one child) shows Banana's Units in its Sales-subtotal column and every row's Units total in its
  Units-subtotal column; -2 is the same (G!A1141); with Year over Product (G!A1171) only the first of those two columns
  is filled. The rule is in "GROUPBY and PIVOTBY: column totals over several value columns" below (T-1700).
Ours (T-1324): `Grouping::lines` lays the groups out with their subtotals in one pass over the sorted groups (a prefix
change between neighbours opens or closes a subtotal), `effective_depth` adds the omitted depth of a value sort,
`ParentOrder` picks hierarchical, key-order or flat parents, the value sort's `Aggregates` (each group's and
parent's) are shown for the groups, subtotals and PIVOTBY Total cells instead of calling the function again, `Model::lifted_groupby` runs GROUPBY at most three times (0,
1, invalid) for any array; the subtotal rows and columns are counted in key order and size checked with
`check_array_size` before any call (a value sort keeps each parent's groups together, so it cannot add subtotals).
Unprobed (T-1324): a depth of 3 or more over three PIVOTBY column fields and value sorts over three column fields (the
engine: every level by its aggregate with subtotals), an error value before an array field_relationship that comes from
the keys or values themselves (the engine: one error), a lifted field_relationship over a 2-D array (the engine:
per element, as the row and column probed), and a 1x1 array field_relationship such as `{0}` or `{1}` (the engine lifts it
to one cell, as `{FALSE}` is in followup G!A921).
Status: VERIFIED (web)
Tasks: T-1276, T-1328, T-1322, T-1323, T-1326, T-1324, T-1700, T-1325, T-1327

## GROUPBY and PIVOTBY: column totals over several value columns
Rule: with one function over two or more value columns (`width` w), PIVOTBY puts its column subtotal and total cells as
if a subtotal spanned one column per line it covers instead of w. Lay the column lines out with every subtotal after
its groups (as for a positive col_total_depth) in the order of the key sort (a descending one too) before any value
sort, w columns per line, the total's w columns last. The total writes its value column v (0-based) into column
`k + v`, k being the number of lines; then each subtotal, ordered by its first group and a parent before its own
deeper subtotals, writes its value column v into
column `f * w + k + v`, f being the index of its first group's line and k the number of lines it covers (its groups and
deeper subtotals). A write into a group's column is dropped and a later write replaces an earlier one. Each subtotal
and total column then moves with its line into the real layout (a negative depth puts the subtotals and totals first, a
value sort reorders the lines) and shows, in every row, the cells of the subtotal or total written into it, or `""`
where nothing was written. Group columns are always right, and so are the row axis (row subtotals, G!A751) and
functions paired with the value columns or laid out as columns over one (probes/pivotby-col-subtotals-values2 S!A181,
-values3 S!A1, S!A13, S!A25).
Consequences: with one value column every write lands on its own column (S!A51). A parent with one child writes its
last value column into its first subtotal column; with two or more children everything lands in their group columns.
The total's writes land near the middle of the lines, so one parent there may show Grand Total cells in its later
subtotal columns. This had looked like "the parent at floor(parents / values)", which matches every older probe but
fails on -values2: eight single-child parents with two values put the total in P5's leaf column, so nothing shows
(S!A1), and with four values over A (three children), B, C, D and E the total writes into 14-17, so A's third and
fourth subtotal columns show the Grand Total's V1 and V2 (-values2 S!A85; floor(5 / 4) would be B). Without subtotals (depth ±1) the total's writes land in the group columns, which is why the
Total columns are `""` (P!Y97), unless there is only one column group: then the first Total column shows the last value
column's total and the others are `""`, also with -1 (probes/pivotby-total-columns-one-group S!A1-A61).
Example: `PIVOTBY(A2:A10,B2:C10,D2:E10,LAMBDA(x,ARRAYTOTEXT(x)),,,,2)` with Outer keys A (child x), B (x, y), C (z),
D (x, y, z), E (y) has 14 lines and 28 columns: A writes into columns 1-2, so A's first subtotal column shows A's V2;
the total writes into 13-14, so C's second subtotal column (13) shows the Grand Total's V1; C writes into 11-12 and E
into 23-24, the rest land on groups (probes/pivotby-col-subtotals-values S!A1).
Evidence: probes/pivotby-col-subtotals-values S!A1-A111 (two and three values, depth -2, shown headers, swapped and
duplicated column fields, a descending value sort), probes/pivotby-col-subtotals-values2 S!A1-A205 (eight and seven
single-child parents with two to four values, mixed children with two to four values, a descending key sort, value
sorts with the depth omitted, three column fields at depth 3, 2 and -3, row and column subtotals together),
probes/pivotby-col-subtotals-values3 S!A49 (three values over three column fields at depth 3),
probes/pivotby-total-columns-one-group, probes/groupby-subtotals G!A781, G!A1141, G!A1171 (excel-graph,
2026-10-17/18). ARRAYTOTEXT over distinct values (101-112, 201-212, ...) shows which records and value column each
cell takes: every cell is `""`, one group's cell or one subtotal's or total's, never a mix.
A descending key sort places in the descending order: over Outer descending (E, D, C, B, A) the total's writes land in
C's subtotal columns, which the ascending order would put on a group (-values2 S!A97).
Unprobed: writes of two subtotals into the same column (the engine: the later in the order above wins, so a parent's
deeper subtotal over a parent, and a later parent over an earlier one's deeper subtotal); several functions over several value columns other than paired (always `#VALUE!` so far: -values3
S!A37, S!A61).
The relative set of a moved cell (probes/pivotby-moved-cells-relative-sets, excel-graph, 2026-10-19, T-1857; two value
columns, col_total_depth 2, relative_to 0-4, `LAMBDA(x,y,ARRAYTOTEXT(y))` and PERCENTOF, with single-child parent C and the
Grand Total shown in parents' columns): it is the set of the subtotal or total whose cell it shows, not of the column it
lands in, in every cell of all ten grids (5x29 each). The engine already did this (`RelativeSets::of_cell` with the source
slot); the case is the evidence.
Status: VERIFIED (web)
Implemented in: crates/rustsheet-engine/src/functions/lookup_and_reference/pivotby.rs (`Placement`, `Shown`)
Tasks: T-1700, T-1857

## GROUPBY and PIVOTBY: two-parameter LAMBDAs and the cell-visit cap
Rule: Excel calls a two-parameter LAMBDA in GROUPBY or PIVOTBY once per aggregate cell with the cell's values and its whole
relative set, so N records in G groups cost about G x N cells, without limit. RustSheet charges every call the cells it
is given (the cell's rows, plus the relative set a two-parameter LAMBDA gets or PERCENTOF sums) before building its
arguments, against the 2^27-cell cap of the lifted *IF(S) calls (`too_many_visits`, criteria.md); once the charges would
pass it, no further call is made and the whole result is `#ERROR!`. One-parameter calls rarely get there (each record is
given once per output column and subtotal level): `GROUPBY` over 1048575 records with 100 functions is `#ERROR!`, with 10
it computes (unit test `one_parameter_calls_are_charged_against_the_visit_cap_too`).
Example: `GROUPBY(SEQUENCE(20000),SEQUENCE(20000),LAMBDA(x,y,ROWS(y)))` (20000 groups x 20000 rows) is `#ERROR!`, while
PERCENTOF passed by name, summed once per relative set (O(N)), and one-parameter LAMBDAs compute it (unit test
`two_parameter_lambdas_are_charged_against_the_visit_cap`, in test_fn_groupby.rs and, for 20000 PIVOTBY rows in one
column, test_fn_pivotby.rs).
PIVOTBY's relative sets hold every record once per pair of key-prefix levels the cells use (the rows' and the columns'
items, subtotal levels included); past 2^24 entries (`MAX_SET_ENTRIES`, 128 MiB of indices) the whole result is
`#ERROR!`, refused from the level counts before any set, prefix table or call is made. Only many key fields with
subtotals at many levels get there: 65535 records over 16 row and 16 column fields at depth 16 with relative_to 3 (272
level pairs) is `#ERROR!`, at depth 2 it computes (unit test `relative_sets_are_capped_before_they_are_built`).
Evidence: none for Excel; this is a resource limit (CLAUDE.md directive 5), SPEC-0006 Design step 3.
Status: DEVIATION (decision 0014)
Implemented in: crates/rustsheet-engine/src/functions/lookup_and_reference/group_functions.rs (`Model::aggregate`),
crates/rustsheet-engine/src/limits.rs (`too_many_visits`, `MAX_LIFTED_CELL_VISITS`),
crates/rustsheet-engine/src/functions/lookup_and_reference/relative_to.rs (`RelativeSets::new`, `MAX_SET_ENTRIES`)
Tasks: T-1325, T-1327

## Nested GROUPBY and PIVOTBY cells: bounds
Rule: Excel keeps a GROUPBY or PIVOTBY function's array result in its cell, at any size and depth (SPEC-0018 rules 1, 8;
"A LAMBDA array result is shown as `#VALUE!`" above). RustSheet bounds them (CLAUDE.md directive 5): one call keeps at most
1,048,576 nested cells, inner levels included (a LAMBDA returning `SEQUENCE(1048576)` for two groups is `#ERROR!`, and so is a
GROUPBY whose LAMBDA returns a GROUPBY of three 300,000-cell cells, twice); one evaluation keeps at most 2^22 cells in nested
grids (`MAX_NESTED_KEPT_CELLS`, as a LAMBDA's captures; a loop of GROUPBY calls no array size check sees); a nested cell has at
most 64 levels (`MAX_NESTED_DEPTH`); past any of them the whole result is `#ERROR!`. Copies of a nested cell share it, so
`CHOOSEROWS(r,SEQUENCE(200)^0)` copies one for free; reading one's contents (INDEX of the cell) charges its cells to a
per-evaluation walk budget of 2^27 cells (`MAX_NESTED_WALK_CELLS`), past which the read is `#ERROR!` (200 INDEX reads of a
1,048,512-cell nested cell fail at the 129th). Since T-2594 SUM, COUNT and COUNTA flattening a nested cell, and MAP, REDUCE and
SCAN handing one to the LAMBDA, charge the same budget, and past it the whole call is `#ERROR!`. Equality of two nested cells is by content but O(1) for copies of one and for
cells of different sizes, so it is not charged.
Evidence: none for Excel; this is a resource limit (CLAUDE.md directive 5), SPEC-0018 "Bound". Unit tests in
crates/rustsheet-engine/src/nested_array.rs and test/test_nested_groupby_cells.rs.
Status: DEVIATION (decision 0014)
Implemented in: crates/rustsheet-engine/src/nested_array.rs (`NestedArray::new`, `NestedBudget`),
crates/rustsheet-engine/src/functions/lookup_and_reference/group_functions.rs (`Model::nest`),
crates/rustsheet-engine/src/limits.rs (`MAX_NESTED_DEPTH`, `MAX_NESTED_WALK_CELLS`, `MAX_NESTED_KEPT_CELLS`),
crates/rustsheet-engine/src/functions/util/aggregate_args.rs (`walk_array_items`),
crates/rustsheet-engine/src/functions/logical/bycol_byrow.rs (`lambda_cells`)
Tasks: T-2593, T-2594

## WRAPCOLS of a single value is not padded; wrap_count below 1 is #NUM!; an empty single cell is #VALUE!
Rule: `WRAPCOLS` of a single value returns that value as a 1x1 result, not a column padded with `#N/A` to wrap_count:
`WRAPCOLS(FALSE,4)` is FALSE in one cell and `WRAPCOLS("1.5",4)&DROP(C10,0,3)` is one `#CALC!` (neither spills over the
occupied cells below). A wrap_count of 0.5 is `#NUM!` (`HYPERLINK(WRAPCOLS(C12,0.5,..),..)`).
Evidence: gen-m7/bb4617644fee40c9 Calc!A30, gen-m7/d38b52bfa1a9f9e6 Calc!A26, gen-m7/f68fafa0ca2eba26 Calc!A39 (excel-graph, 2026-10-11, T-1275).
Also: an error pad_with is an ordinary pad value: it fills the padded cells (WRAPCOLS_ROWS.xlsx E10, `NA()`;
`WRAPCOLS({1,2,3},2,VALUE("x"))` pads with `#VALUE!`) and when nothing is padded the array is returned
(`WRAPROWS({1,2,3,4},4,VALUE("x"))` is 1 2 3 4; `WRAPCOLS(5,3,VALUE("x"))` is 5). The `#VALUE!` of
`WRAPCOLS(D6:D6,2,VALUE("x"))` (gen-m9/675763979c1defb6 A6) is the empty single-cell input: `WRAPCOLS(Z1:Z1,2)`,
`WRAPCOLS(Z1,2)`, `WRAPROWS(Z1:Z1,1)` and a wrap_count of 0 are `#VALUE!` with any pad_with, while two empty cells
(`Z1:Z2`) are two 0s. wrap_count above the length is the length, so `WRAPCOLS({1,2},4)` is 2x1 and
`WRAPCOLS(B5:H5,100,NA())` is 7x1, unpadded. wrap_count is `#NUM!` below 1 for WRAPROWS too (gen-m9/50f10c4062fb2e98 A21;
the full set is under "WRAPROWS and WRAPCOLS" below).
Evidence: probes/wrapcols-wraprows-pad-with (excel-graph, 2026-10-18, T-1840).
Unprobed: an omitted vector (`WRAPCOLS(,2)`) and an empty cell reached indirectly (`WRAPCOLS(IF(TRUE,Z1),2)`,
`WRAPCOLS(INDEX(Z1:Z2,1),2)`); the engine treats the omitted vector like an empty cell (`#VALUE!`).
Ours: all of the above since T-1786, T-1348 and T-1840 (wrap_count is read before truncation, so 0.5 is `#NUM!`).
Status: VERIFIED (excel-graph).
Tasks: T-1348, T-1840

## MAP over arrays of different sizes, or a single value and an array
Rule: MAP does not reject arrays of different sizes, and does not broadcast: the result has the largest row count and the
largest column count of its arrays, an element that every array has is the LAMBDA's result, and any other element is
`#N/A` without the LAMBDA being called (`MAP({1;2;3},{10;20},LAMBDA(a,b,a+b))` is 11, 22, `#N/A`; with `ISNA(b)` as the
body the padded element is still `#N/A`, not TRUE). A scalar is a 1x1 array: `MAP(5,{1,2,3},LAMBDA(a,b,a+b))` is 6,
`#N/A`, `#N/A`. A row next to a column gives their bounding box with only the corner computed:
`MAP({1,2,3},{10;20},..)` is 2x3, `ROWS` 2, `COLUMNS` 3, 11 then `#N/A` everywhere else; `MAP({1;2;3},{10},..)` is 11,
`#N/A`, `#N/A`. In a cell with occupied neighbours the array is `#SPILL!` (gen-m7/a0cbc75358b429b6 A31,
gen-m7/b5186d194fec0fdc A13, gen-m9/55fc8c4ffbd7c7cc A33, gen-m9/95efb37a9874f82e A17).
Evidence: probes/map-unequal-sizes (excel-graph, 2026-10-17, T-1349); the four gen cells above (T-1275, T-1289).
Ours: padded as above since T-1349 (`fn_map`, map_reduce.rs); the padded shape is checked against the 2^20-cell bound
before allocating ("Array results above 1,048,576 cells").
Status: VERIFIED (excel-graph)
Tasks: T-1349

## MAP over single values
Rule: MAP's first array may be a single value, but each later array must be an array or a reference: a single value there
makes the whole MAP `#VALUE!`. `MAP(1,LAMBDA(x,x))`, `MAP("a",LAMBDA(x,x))` and `MAP(1,{2},LAMBDA(x,y,x+y))` work (1, "a",
3), and so do one-cell references (`MAP(Z1,Z2,LAMBDA(x,y,x+y))` is 11). `MAP({1},2,..)`, `MAP(1,2,..)`, `MAP(TRUE,FALSE,..)`,
`MAP(1,1,1,..)` and `MAP(Z1:Z2,3,..)` are `#VALUE!`, and so is `SUM(MAP(scalar,scalar,scalar,_xleta.MAX))`.
Evidence: probes/lambda-value-arguments A27-A32, B1-B10 (excel-graph, 2026-10-19, T-1511); gen-m7/44ef4f5638e25bcd Calc!A7.
Ours: `fn_map` reads its second and later arrays with `refuse_scalar` (`by_slices_source`, bycol_byrow.rs) since T-1511, an error value
included since T-1946. An error value as a later array is `#VALUE!` whatever the error (`MAP({1},NA(),..)`, `MAP({1},1/0,..)`,
`MAP({1,2},{3,4},NA(),..)`, `LEFT(NA(),2)`, `SEQUENCE(1,0)` as the later array; gen-m7 shows the same for a BASE `#NUM!` (4dac39f89709e2fc
A137), a MOD `#DIV/0!` (21b4eef18424ce94 A1393), `DROP(Data!B5:B12,0,3)` `#CALC!` (a470dc9e165bdb20 A3360) and `SEQUENCE(1,SIN(Data!E9))`
(f19581ac44da1653 A2477)). An error inside an array literal is not refused (`MAP({1},{#N/A},LAMBDA(x,y,x))` is 1), a blank or text cell
reference is an array of one (`MAP({1},B1,..)` is 1; `ISBLANK(y)` of an empty B2 is TRUE), a missing argument (`MAP({1},,..)`) is `#VALUE!`,
and an error as the first array is that error (`MAP(NA(),LAMBDA(x,x))`, `MAP(NA(),{1},..)` are `#N/A`). `MAP(1,LAMBDA(x,y,x))` is
`#VALUE!` (`ERROR.TYPE` 3, as `MAP(1,2,..)`).
INDEX of a LAMBDA: `INDEX(LAMBDA(x,x),1)(5)` is 5, an index of 2 is `#REF!` (`ERROR.TYPE` 4, also `INDEX(..,1,2)`), -1 is `#VALUE!`, and 0 is
`#CALC!` (its `ERROR.TYPE` is `#N/A`). Evidence: probes/map-later-array-error-blank (excel-graph, 2026-10-24, T-1946).
Status: VERIFIED (excel-graph)
Tasks: T-1511, T-1946

## An uncalled LAMBDA is one value
Rule: an uncalled LAMBDA, or a function passed by name, is one value that is neither an error nor text. `ROWS` and `COLUMNS`
of it are 1 (`ROWS(LAMBDA(x,x))`, `COLUMNS(_xleta.SUM)`, `ROWS(IF(1,LAMBDA(x,x)))`). INDEX with a row and a column of 0 or 1
gives it back, which a cell shows as `#CALC!` (`INDEX(LAMBDA(x,x),1,1)`, `INDEX(LAMBDA(x,x),1)`, `INDEX(_xleta.SUM,1,1)`;
`ROWS(INDEX(LAMBDA(x,x),0,0))` is 1). ISNONTEXT of it is TRUE, while ISERROR, ISERR, ISNA, ISTEXT, ISNUMBER, ISLOGICAL,
ISBLANK and ISREF are FALSE (also through `IF(1,..)`). `ISEVEN(LAMBDA(x,x))` is `#VALUE!`, `ERROR.TYPE(LAMBDA(x,x))` is
`#N/A`, `TYPE(_xleta.SUM)` is 128 and `IFERROR(LAMBDA(x,x),1)` is `#CALC!`, the uncalled LAMBDA in the cell. Joined as text
it is `#VALUE!`: `LET(v,LAMBDA(z,z),t,TYPE(v),..,t&"|"&v)` is `#VALUE!` (probes/t2475-stat-subexpressions, the bare `LAMBDA(..)`
rows; ours `#N/IMPL!`, T-2622).
Evidence: probes/lambda-value-arguments A1-A26 (excel-graph, 2026-10-19, T-1511); probes/m7-argument-modes F44:H44;
probes/eta-reduced-names E93; probes/t2475-stat-subexpressions (excel-graph, 2026-10-24, T-2475).
Ours: `value_extent` (cast.rs), `is_predicate` and `parity_of` (information/mod.rs), `index_select` (index.rs) since T-1511.
INDEX of a LAMBDA with an index of 2 is `#REF!` there, as for a scalar, which is unprobed (T-1946).
Status: VERIFIED (excel-graph)
Tasks: T-1511

## LAMBDAs that outlive their call
Rule: a LAMBDA made inside a LET or a LAMBDA call is a lexical closure: called after that LET or call is over, it reads the
values its outer names had when it was made, not their names' later bindings. `LET(g,REDUCE(LAMBDA(z,z),{1,2,3},LAMBDA(f,x,
LAMBDA(z,f(z)*10+x))),g(0))` is 123 (each step's LAMBDA captures the previous accumulator and that step's x);
`LET(mk,LAMBDA(a,LAMBDA(b,a+b)),plus,mk(5),plus(1))` 6; two closures from one maker keep their own values,
`LET(mk,..,fa,mk(1),fb,mk(100),fa(0)+fb(0)*1000)` 100001; a later binding of the same name changes nothing,
`LET(mk,LAMBDA(a,LAMBDA(b,a+b)),f,mk(1),a,50,f(0))` 1 and `LET(a,1,f,LAMBDA(b,a+b),LET(a,2,f(0)))` 1; a closure reads names of
several scopes, `LET(x,10,mk,LAMBDA(a,LAMBDA(b,a+b+x)),f,mk(1),f(100))` 111; a closure over a REDUCE chain is a function MAP can
take, `MAP({10,20},g)` spills 16, 26. An inner parameter shadows the captured one, `LET(mk,LAMBDA(x,LAMBDA(x,x*2)),f,mk(5),
f(3))` 6 (H13), and a LAMBDA made and called inside a REDUCE step reads that step's x (H17, 60). A captured reference stays a reference: `INDEX(r,2)` through it is 2, `ISREF(r)` TRUE, and a
captured union keeps its areas (`AREAS` 2, `SUM` of `(A1,A3)` 4); a captured error is read as the error (`mk(1/0)` then `f(1)` is
`#DIV/0!`), and an omitted optional parameter stays omitted (`ISOMITTED(c)` TRUE inside the closure: H20 is `"om"+0`, `#VALUE!`). Calling a call,
`LAMBDA(a,LAMBDA(b,a*b))(3)(4)`, is 12 and three levels give 123; `MAP(SCAN(LAMBDA(z,z),{1,2,3},..),LAMBDA(g,g(0)))` spills the
array of LAMBDAs SCAN makes as 1, 12, 123, and `INDEX` of that array called, `INDEX(s,1,3)(0)`, is 123. A cell holding the
REDUCE result itself (a LAMBDA) is `#CALC!`, as for any uncalled LAMBDA. LET names spelled like cells (`add5`, `f1`), stored with
their `_xlpm.` prefix, bind like any name (H4 6, H5 100001; references.md "LET and LAMBDA names spelled like a cell", T-2040).
Evidence: probes/lambda-escaping-closures H1-H25, I2, J2, I14 (excel-graph, 2026-10-19, T-0439).
Ours: when a LET or a LAMBDA call ends, `Model::close_over` gives the LAMBDAs reachable from its result (the only way one
leaves it) the variables of that scope their bodies read (one shared copy per variable, `VariableStack::shared`), and
`call_prepared` binds those not in scope for each call (T-0439); a LAMBDA that never leaves its scope captures nothing. A
captured union re-bound for a call counts toward the held-areas cap where it is called (references.md "Union area cap").
A call binds what its closure captured shared, not copied (`Binding::Shared`), so MAP over an escaped closure that captured
a large array costs no copy per call. How many cells closures may capture is capped ("LAMBDAs capture at most 4,194,304
cells", DEVIATION). An array of LAMBDAs leaving the scope ("Arrays of LAMBDAs") gives each of its elements made there the
same captures (T-2039: `ScopeResult::lambdas`). Open gaps: chained calls
`f(..)(..)` are a parse error (H3, H8, H10: T-2038), and cell-shaped LET names are read as cells (H4, H5: T-2040).
Status: VERIFIED (excel-graph)
Tasks: T-0439, T-2038, T-2039, T-2040

## LAMBDAs capture at most 4,194,304 cells
Rule: Excel's limit on what the LAMBDAs that outlive their call ("LAMBDAs that outlive their call") may capture is unprobed;
it is expected to keep every value a closure captured, however large.
Evidence: none for Excel; this is a resource limit, not an Excel observation.
Ours: the closures made while one root cell of an evaluation pass is evaluated, the cells evaluated as its dependencies
included, may hold copies of at most 4,194,304 captured cells together (`MAX_CAPTURED_CELLS`, an array counting its cells,
anything else 1; the budget is reset after each root cell, `Model::evaluate_root_cell`). A capture past it is `#NUM!`, so a
REDUCE whose steps each return a closure over a fresh large array cannot hold memory without bound (CLAUDE.md directive 5).
Status: DEVIATION (decision 0014)
Implemented in: crates/rustsheet-engine/src/functions/logical/lambda.rs (`MAX_CAPTURED_CELLS`, `Model::close_over`)
Tasks: T-0439

## How deep LAMBDA calls nest
Rule: Excel evaluates LAMBDA calls nested 300 deep, both a LAMBDA that calls itself through a parameter,
`LET(f,LAMBDA(g,n,IF(n=0,0,g(g,n-1)+1)),f(f,300))` 300, and a chain of 300 closures made by REDUCE,
`LET(g,REDUCE(LAMBDA(z,z),SEQUENCE(300),LAMBDA(f,x,LAMBDA(z,f(z)+x))),g(0))` 45150. What it gives deeper is unknown: workbooks
with 1,000 to 20,000 levels made Excel for the web's createSession fail (`rustsheet-oracle fill`, 2026-10-19).
Evidence: probes/lambda-call-depth H1-H6, I1-I6 (excel-graph, 2026-10-19, T-0439).
Ours: nested LAMBDA calls may use four times the evaluation's stack budget (decision 0080; `set_max_evaluation_stack`,
256 KB by default), counted from the cell's own base so cells and calls share it (`Model::lambda_stack_base`); past it a call
is `#NUM!` instead of overflowing the stack. With the defaults that is 408 levels of the self-call and 491 of the REDUCE chain
in a release build (116 and 147 in a debug build), so the 300-level probe cells H3 and H6 match Excel; a call 20,000 deep is
`#NUM!`. Before T-0439 such recursion overflowed the stack and aborted the process; before T-0444 (decision 0100) a level took
twice the stack and only about 200 fit, the 300-level cells being listed deviations.
Status: DEVIATION (decision 0014)
Implemented in: crates/rustsheet-engine/src/functions/logical/lambda.rs (`LAMBDA_STACK_SHARE`, `Model::lambda_stack_base`),
crates/rustsheet-engine/src/model.rs (`evaluate_node_in_context`), crates/rustsheet-engine/src/node_frames.rs.
Tasks: T-0439, T-0444

## One formula evaluation does at most 268,435,456 work units
Rule: Excel has no known per-formula work limit; a formula whose LAMBDA helpers multiply work
(`REDUCE(0,SEQUENCE(1E5),LAMBDA(a,b,a+SUM(SEQUENCE(1E5))))`) runs until done, and Excel's result for it cannot be probed
(the session times out, as the deep LAMBDA workbooks of "How deep LAMBDA calls nest" did).
Evidence: none for Excel; a resource limit (CLAUDE.md directive 5, SPEC-0024).
Ours: each formula evaluation (a cell formula, a defined name, a conditional format, `evaluate_formula`) may spend 2^28 work
units: 6 per node evaluated (each link of an operator chain is one), 3 per cell of each array value a node gives or a range
is turned into, one per 4 bytes of each text value a node gives (and of a lifted text call's results and of each text the
`&` operator makes over an array, T-2806), 512 per LAMBDA value
made plus 6 per node of each LAMBDA body cloned (the LAMBDA stored, and a call's fresh copy of its body; T-2923), plus the
per-call caps' own charges: a lifted call's cell visits and kernel units (`MAX_LIFTED_CELL_VISITS`, 1 unit each), GROUPBY/PIVOTBY's
visits, and a REGEX* or regex XLOOKUP/XMATCH call's compile budget (4 bytes a unit) and search work (32 units a unit), all of
both budgets for a call that ran out of them. Past it the formula is
`#VALUE!` ("Formula needs too much work to calculate"), which `IFERROR` cannot catch (the formula's value is the error,
whatever the formula made of it); a dependency cell has its own budget, so its `#VALUE!` is an ordinary value to its readers.
Once a charge fails no further LAMBDA call runs, so the run ends at once. Calibration (release build, idle 4-core container,
2026-10-27, T-2803 with SPEC-0024 as amended by T-2813 and T-2923): `rustsheet-eval check` has 0 regressions and no formula
past the limit; the heaviest eval formula spends 118,756,588 units (gen-m9/3e98b26cf6713e77 Calc!A204, a lifted CHISQ.INV.RT
over a PIVOTBY: 118,751,450 of them visit units, below one cap, 2^27); the heaviest outside the visit caps spends 12,582,948
(ironcalc-calc/LOOKUP_AND_REFERENCE/ROW_COLUM.xlsx, a whole-column ROW array; below 2^26). Time to the limit: the three
REDUCE/MAP/MAKEARRAY formulas of SPEC-0024 6.2, 6.7 and 6.6 s; `SUM(MAP(SEQUENCE(1000),LAMBDA(x,SUM(MAKEARRAY(1024,1024,
LAMBDA(r,c,r))))))` 7.7 s (11.4 s at 3 units a node, hence 6); LAMBDAs made on each call:
`SUM(MAP(SEQUENCE(1000),LAMBDA(x,SUM(MAKEARRAY(1024,1024,LAMBDA(r,c,MAP(SEQUENCE(1),LAMBDA(z,r+c+z))))))))` 2.9 s (124 s
before the LAMBDA charge), with the body `r+c+z+r+c+z+r+c+z` 3.3 s, with `LAMBDA(z,z)(1)` 0.8 s; MAPs of
`LEN(REPT("a",32767))` 0.9 s, of `FIND("b",REPT("a",32767))` 5.1 s, of `LEN(SUBSTITUTE(REPT("a",32767),"a","b"))` 3.0 s, of
`SEARCH("b",s)` over a LET-bound 32,767-byte `s` 5.8 s; a MAKEARRAY of REGEXTEST 0.8 s, a MAP of REGEXTEST calls that each run out of their own budget (`\G(a|a)*b` and `(.*)\1x` over
32,767 a's; such a call is charged both whole regex budgets) 2.3 and 0.7 s, of lifted RATE at its cap 2.9 s (the
second call's charge fails) and of smaller RATE calls 5.2 s, of lifted YIELD 7.6-7.9 s (each call refused by its own cap
since T-2879, so these are node units). One call at its visit cap (2026-10-27, before T-2923's re-fit, which does not change
visit units): RATE 2.4 s, ODDLPRICE 1.1 s, CHISQ.DIST and POISSON.DIST 0.4 s, GAMMA.INV and GAMMA.DIST under 0.1 s, COUNTIF
and SUMIFS over 1,024 cells 0.1 and 0.4 s. The LAMBDAs a formula makes stay stored until the evaluation ends, so a
recalculation of many such formulas still slows each later one (T-2925). Array cells read by aggregates and other
whole-array readers are not charged yet (T-2804, T-2805).
Status: DEVIATION (decision 0014)
Implemented in: crates/rustsheet-engine/src/area_budget.rs (`MAX_FORMULA_WORK`, `Model::spend_work`, `Model::with_area_budget`),
crates/rustsheet-engine/src/model.rs (`evaluate_node_in_context`), crates/rustsheet-engine/src/functions/logical/lambda.rs
(`call_prepared`, `prepare_lambda`, `fn_lambda`), crates/rustsheet-engine/src/node_frames.rs (`LambdaDefKind`),
crates/rustsheet-engine/src/arithmetic.rs (`handle_concatenate`).
Tasks: T-2712 (SPEC-0024), T-2803, T-2813, T-2923, T-2806

## Copies of a text share its bytes
Rule: Excel has no known limit on the memory one formula's texts take; `IF(SEQUENCE(1024,1024),REPT("a",32767))` copies a
32,767-character text into 2^20 cells, and only memory limits it (not probed: the oracle cannot read back a 64 GB result).
Evidence: none for Excel; a resource limit (CLAUDE.md directive 5, SPEC-0024 "Known gaps: retained memory").
Ours: a text value's bytes are shared (`Text` holds an `Arc`), so a copy into another cell, array element, LET name or
LAMBDA argument is a pointer, and an edit (`&`, truncation) copies only bytes another value still holds. New texts are paid
for by the work budget of "One formula evaluation does at most 268,435,456 work units" (one unit per 4 bytes), so one
formula makes at most about 2^30 bytes of text before it is `#VALUE!`. Measured on a release build (2026-10-27, T-2806;
peak resident memory of a process holding A1 `=REPT("a",32767)` and the formula in B1): before T-2806,
`IF(SEQUENCE(1024,1024),REPT("a",32767))`, the same over A1, `IFERROR(SEQUENCE(1024,1024)/0,..)`, `IFNA`, `CHOOSE`,
`INDEX(..,SEQUENCE(1024,1024)*0+1)`, `EXPAND(..,1024,1024,..)`, `TOCOL(IF(..))` and `SEQUENCE(1024,1024)&REPT("a",32767)` each
aborted the process past a 6 GB address-space limit (32 GB needed); now the copies peak at 115 MB (TOCOL of one: 470 MB)
and the `&` formulas stop at the budget, `#VALUE!` at 1.1 GB. `MAKEARRAY(1024,1024,LAMBDA(r,c,REPT("a",32767)))`, MAP and
SCAN shapes were already bounded by the text charge on each call's value (`#VALUE!` at 1.1 GB); a LAMBDA that returns a
LET-bound or cell text now shares it (`#VALUE!` at 25 MB, was 1.1 GB). Excel would compute the copies' values; ours
does too, so only the `&` formulas past the budget differ.
Status: DEVIATION (decision 0014)
Implemented in: crates/rustsheet-engine/src/text.rs (`Text`), crates/rustsheet-engine/src/arithmetic.rs
(`handle_concatenate`).
Tasks: T-2806

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

## MAKEARRAY with an array step; BYROW, BYCOL and MAP with a LAMBDA step
Rule: MAKEARRAY whose LAMBDA returns an array for a cell is one `#CALC!`, nothing spilled, as BYROW and BYCOL are: `MAKEARRAY(2,2,
LAMBDA(acc,z,{1,2}))` (gen-m7/0624b684889e9228 Calc!A39), `MAKEARRAY(2,3,LAMBDA(y,v,VSTACK(y,v)))` (23b5367c18059af2 A245),
`MAKEARRAY(4,2,LAMBDA(w,x,HSTACK({1,2,3})+x*Data!B6))` (f7e94a817e5385ac A208), a step `HLOOKUP(..)*SORT(Data!D6:D12)`
(b2078b5d702c691a A2785), and even `MAKEARRAY(1,1,LAMBDA(x,acc,{1,2}))` (23b5367c18059af2 A3493). The `#CALC!` then behaves as any
scalar error: as REGEXEXTRACT's pattern it fills the lifted text's 6x3 shape (f19581ac44da1653 A465), as TEXTSPLIT's ignore_empty it is
the result (21b4eef18424ce94 A1661). 20 formulas in all.
A LAMBDA whose result is itself a LAMBDA (an uncalled function value) is `#CALC!` in that element of BYROW, BYCOL and MAP, which keep their
shape: `BYCOL(Data!A11:D11,LAMBDA(x,LAMBDA(v,v)))` is four `#CALC!` (bc4a69b0b512bc04 A294), `MAP(Data!D1:E4,LAMBDA(acc,LAMBDA(w,w)))`
eight (A2102), `BYROW({"",0,FALSE},LAMBDA(acc,LAMBDA(x,x)))` one (21b4eef18424ce94 A3078), and over a one-cell range or an empty cell
alike (18 formulas). This fits "An uncalled LAMBDA is one value" (a cell shows it as `#CALC!`).
Evidence: gen-m7 (excel-graph, 2026-10-22, T-2254 triage; the cells are listed in T-2406).
Ours (T-2406): MAKEARRAY with a multi-cell step is one `#CALC!` (a 1x1 array step is unwrapped as BYROW does; unprobed, T-2463).
Since T-2039 a LAMBDA step of MAKEARRAY, BYROW, BYCOL and MAP is a LAMBDA element ("Arrays of LAMBDAs"): a cell shows it as `#CALC!`,
MAP can call it (`MAP(MAKEARRAY(1,2,LAMBDA(r,c,LAMBDA(x,x+c))),LAMBDA(g,g(10)))` is 11, 12, probes/lambda-arrays B22), and a
function reading it as a value takes it as `#VALUE!`. Before T-2039 it was `#CALC!` when the call was the cell formula's outermost
node and `#VALUE!` anywhere else. Both match the consumers probed:
`&` (gen-m7/23b5367c18059af2 A2578 `TEXTSPLIT(.&MAP(Data!Z1,LAMBDA(acc,LAMBDA(v,v))),..)` is `#VALUE!`), CELL (gen-m7/f7e94a817e5385ac
A942, the CELL row of a VSTACK) and CONFIDENCE.NORM (gen-m9/165db58d1540f014 A273). The error-testing functions (ISERROR, IFERROR,
ERROR.TYPE, TYPE) see a LAMBDA, not an error, as Excel does over SCAN's LAMBDAs (`ISERROR` FALSE, probes/lambda-arrays B8, B23);
over BYROW, BYCOL and MAP elements that is unprobed (T-2463).
Status: VERIFIED (excel-graph)
Tasks: T-2406, T-2463

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
`_xleta.IF` called with one argument is accepted, not an argument-count error: it gives `#VALUE!` for a condition that is
not an error, whatever it is (TRUE, FALSE, 1, 0, text, a blank cell; so `ISLOGICAL(f(TRUE))` is FALSE and `f(TRUE)+1`
`#VALUE!`), passes an error on (`f(NA())` `#N/A`), and lifts over an array or range condition element by element
(`f({1,0})` and `f(A1:B1)` are 1x2 `#VALUE!`, `f({1,#N/A,#DIV/0!})` `#VALUE!`, `#N/A`, `#DIV/0!`), with `f` bound by
`LET(f,_xleta.IF,..)`. So `MAP({1,0},_xleta.IF)` and `BYROW({1;0},_xleta.IF)` are `#VALUE!` per element, and BYROW and BYCOL
over two-cell slices are one `#CALC!` (E101 above). With two arguments it is IF (`f(TRUE,5)` 5, `f(FALSE,5)` FALSE);
with none (`f()`) the error is one excel-graph cannot decode (left pending). Evidence: probes/eta-if-one-argument
(excel-graph, 2026-10-19, T-1947).
Unprobed: other errors (`#DIV/0!`, `#REF!`) and an array as BYROW's or BYCOL's function (the engine says `#VALUE!` for
both), a special form passed by name (`_xleta.LET`, `_xleta.LAMBDA`, `_xleta.SUBTOTAL`, `_xleta.OFFSET`) and a call with too
few arguments (`LET(f,_xleta.SUM,f())`): the engine gives whatever the function gives for those arguments.
Evidence: probes/eta-reduced-names (30 questions, excel-graph, 2026-10-14, T-1466); probes/m7-argument-modes F28:H51.
Ours: an unbound `_xleta.` name of a known function evaluates to a LAMBDA whose definition is made per call
(`Model::unbound_name`, `eta_definition`, lambda.rs); BYROW and BYCOL take only a LAMBDA (`by_slices_lambda`). IF with
only a condition is the one-argument eta IF above (`Model::if_pick`, T-1947; stored only: a typed `=IF(x)` is refused,
by Excel (Graph PATCH 400) and by ours (T-2568), and a file-stored one is refused by excel-graph, below, T-1990, T-2939); `ISERROR(_xleta.SUM)` is FALSE since T-1511. GROUPBY and PIVOTBY call an eta name through the same path (T-1322, T-1326), with its minimum argument count (two for PERCENTOF and CORREL: "GROUPBY and PIVOTBY", T-1325), and share the "is it a LAMBDA" check with BYROW and BYCOL (`as_lambda`, bycol_byrow.rs).
A one-argument `IF(x)` stored in a file (`=IF(TRUE)`, `=IF(1)`, `=IF({1,0})`, `=IF(A1:B1)`, each alone in a workbook) is refused by
excel-graph: `createSession` answers 501 `FileCorruptTryRepair` / `unsupportedWorkbook`, so the formula cannot be loaded from a
file and has no stored value to match (probes/typed-if-one-argument, left pending, T-1990). **Typed, Excel refuses it**:
entering `=IF(TRUE)`, `=IF(FALSE)`, `=IF(1)`, `=IF(0)`, `=IF({1,0})`, `=IF(A1:B1)`, `=IF("x")`, `=IF(NA())`,
`=ISLOGICAL(IF(TRUE))` or `=IF(TRUE)+1` through Excel for the web's input parser (Graph range `values` PATCH, each alone)
is rejected with 400 `InvalidArgument`, as `=PV(1)` and other typed calls with too few arguments are (references.md, "A LET name or LAMBDA parameter
shadows a defined name of the same name, in any case", the paragraph on names spelled like a built-in function, T-2568).
An empty or given second argument is accepted: `=IF(TRUE,)` is 0, `=IF(TRUE,1)` 1, `=IF(FALSE,1)` FALSE, each printing as
typed (probes/t2939-typed-if-two-argument-controls). So a one-argument IF has no value in Excel either way, typed or
stored; the refused entries cannot be committed as cases.
Ours: typed entry (`Input::Parse`, `Input::Formula`, engine `set_typed_user_input`) refuses each of them through the
typed argument-count check (`note_argument_count`, parser/operands.rs, T-2568), and stored text keeps the `#VALUE!` per
element of the eta path (T-1947), a case where ours gives a value Excel never shows (test
`typed_if_with_one_argument_is_refused`, test_fn_if.rs). Whether desktop Excel writes a one-argument IF to a file is
unprobed.
Evidence: Graph PATCH 400 for each refused entry (excel-graph, 2026-10-28, T-2939; reproduce by typing one of them as the
only `entered` cell of a case); probes/t2939-typed-if-two-argument-controls E1:E3, F1:F3, G1:G3.
Status: VERIFIED (excel-graph)
Tasks: T-1466, T-1947, T-1990, T-2939

## BYROW and BYCOL of a single value
Rule: BYROW and BYCOL take a single value as a 1x1 array and call the LAMBDA once: `"ü"&BYROW(COUNT(..),LAMBDA(z,z))` with
the COUNT 7 is "ü7", and `BYROW("",_xleta.PRODUCT)` is 0 (PRODUCT of the row {""}).
Evidence: gen-m7/93976dd9a3578ed9 Calc!A31, gen-m7/59cc50701f4937c9 Calc!A4 (excel-graph, 2026-10-11, T-1275)); BYCOL of a scalar: gen-m9/65faaeb48419087d Calc!A23, gen-m9/1466c66c61189a83 Calc!A34.
Ours: matches (a single value is a 1x1 array and the LAMBDA is called once; unit test `byrow_bycol_of_a_single_value_call_the_lambda_once`, T-1350).
Status: VERIFIED (web)
Tasks: T-1350

## SORT argument errors come before validation
Rule: an error-valued argument of SORT wins over an invalid other argument: `SORT(B11:E12,ROUND(0,7),A3:C5,<#N/A>)`
(sort_index 0, a 9-cell sort_order range and a by_col that is `#N/A`) is `#N/A`.
Evidence: gen-m7/c5d278a8662ef4b2 Calc!A39 (by_col `HLOOKUP(ADDRESS(19,19),Data!C8:C12,3)`) (excel-graph, 2026-10-11, T-1275).
Evidence (T-1850, probes/sort-error-args, excel-graph): errors are reported left to right, before any value is validated:
- SORT: sort_index error beats sort_order error (H29 `NA(),1/0` is #N/A, H33 `1/0,NA()` is #DIV/0!); sort_index (H37) and sort_order (H53) errors beat a by_col error; an error sort_order with sort_index 0 returns the error (H13); an error by_col with sort_index 0 returns the error (H25); an error sort_order with a valid sort_index returns the error (H17).
- SORTBY: by_array1, order1, by_array2... left to right (H77, H89 `C,NA(),B,1/0` is #N/A, H93 is #DIV/0!); an error in a later pair beats an invalid earlier order (H85 `C,0,B,NA()` is #N/A); an error order beats a by_array row-count mismatch (H105).
- Invalid values: `SORT(arr,0,1)`, `SORT(arr,1,0)`, `SORTBY(arr,key,0)`, a row-count mismatch (a single value is not one on a one-column array: "SORTBY with a single-value by_array or a fractional sort_order") and a two-column by_array (`SORTBY(A1:A3,C1:B3,1)`, H101) are #VALUE!. `SORT(NA(),0,1)` and `SORTBY(NA(),...)` are #N/A.
- Array sort arguments: `SORT(arr,{1,2},1)` is a multi-key sort (H57), `SORT(arr,{1,2},{1,-1},TRUE)` too (H69), `SORT(arr,1,{1,-1})` is #VALUE! (H61, H65), and `SORTBY(A1:A3,C1:C3,{1;-1;1})` returns {1;0;0} (H97-H99; the rule is "UNIQUE's and SORTBY's lifted elements").
Ours: errors left to right and the two-column by_array rejected (T-1850). A #VALUE! sort_order is still returned after by_col's error (multi-cell order of c5d278a8662ef4b2 A39 needs it; a real `"a"+1` order against a later error argument is UNVERIFIED for both SORT and SORTBY, which returns it at once). SORT reads array sort_index/sort_order lists since T-1837 (H57:J71 pass; see "Dynamic-array and LAMBDA function argument modes" for the list rules); SORTBY's array sort_order is T-1839.
Status: VERIFIED (excel-graph)
Tasks: T-1351, T-1837, T-1850, T-1902

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
(`WRAPCOLS(5,2)` too). Ours: `#NUM!` for 0 and -1 since T-1348 (checked before truncation, so 0.5 too); the clamp above the vector's length, which
also makes a scalar 1x1, is implemented (T-1786). Everything else probed matches: wrap_count 1-7 with a pad, an omitted pad (`#N/A`
fill), pad_with 0, "" and `#N/A`, a column vector, an inline array, a 2-D range (`#VALUE!`), a text
wrap_count (`#VALUE!`; C43, D43); a boolean wrap_count counts as a number (TRUE is 1: `WRAPCOLS(B5:H5,TRUE)` is the 1x7 row 1..7, D44; `WRAPROWS(B5:H5,TRUE)` is the 7x1 column, shown by probes/dynamic-array-reshape-reprobe B20-B22; it was `#SPILL!` in C44 only because the cell below blocked it); a fractional wrap_count
truncates (2.5 is 2: `WRAPROWS(B5:H5,2.5,"p")` is 4x2, `WRAPCOLS` 2x4, C38-D39) and an error vector.
TOROW over A1:C3 (1, "x", blank / #N/A, 4, TRUE / 7, 8, 9), ignore 0-3 with scan_by_column FALSE and TRUE, a scalar, inline
arrays, a bad ignore (4, -1, "a") and an error argument all match ours (probes/dynamic-array-reshape-and-isomitted B20-B49). Ignore 1 keeps
errors (B22/B23 are `#N/A`) and keeps an empty string `""` (`COLUMNS(TOROW({1,"",3},1))` is 3, B48); a boolean ignore is
accepted (`TOROW({1,2,3},TRUE)` is 1,2,3, B46). TRUE selects ignore mode 1, dropping blanks: `COLUMNS(TOROW(A1:C3,TRUE))` is 8 (9 cells less blank C1), the same as `TOROW(A1:C3,1)`, and FALSE keeps all 9 (probes/dynamic-array-reshape-reprobe C20-C24).
`TOROW(A1:C1,1)` drops the blank C1 (2 columns, `1|x`, D20-D21; ignore 0 keeps 3, D22). A scan_by_column of 2 is not an error (`TOROW(A1:C3,0,2)` is 1x9, A21-A22) and is truthy: with ignore 3 (blanks and errors dropped) scan 0 gives `1|x|4|TRUE|7|8|9` but scan 1, 2 and TRUE all give the column order `1|7|x|4|8|TRUE|9` (F20-F23).
TEXTJOIN over A1:C3 with ignore 0, 1 is `#N/A` because A2 is `#N/A`, so those cells show no value; the shapes decide. `COLUMNS(...)=8` alone cannot tell ignore mode 1 from 2 (one blank, one error); TRUE is mode 1 because the error is kept (C22 `#N/A`). Ours matches in all 20 cells.
ISOMITTED inside `LAMBDA(x,[y],...)` matches ours in every probed form (A20-A49): an omitted argument is TRUE, an empty slot
(`(1,)`, A24-A25) counts as omitted and so does a required parameter left empty (`(,2)`, A34), while `""` (A26-A27) and a reference to a
filled cell (A28-A31) count as supplied (FALSE), and so is a reference to a blank cell: `LAMBDA(x,[y],ISOMITTED(y))(1,C1)` with C1 blank is FALSE (probes/dynamic-array-reshape-reprobe E20-E21); outside a LAMBDA `ISOMITTED(5)`, `ISOMITTED(A1)` and `ISOMITTED(#N/A)` are FALSE (A43-A45).
A function result is not omitted either: `ISOMITTED(LOOKUP(ROWS(Data!D$3),Data!C9:C12,Data!B10:B12))` (an unsorted lookup vector, a
shorter result vector over blank cells) is FALSE (gen-m7/2c5ea078f25443a2 Calc!A1087, excel-graph, 2026-10-21, T-2225). Ours: FALSE for a LOOKUP call since T-2317 (`fn_isomitted`); other built-ins that can pass an omitted parameter through (`IF(TRUE,y)`, `CHOOSE`, `LET`, `IFERROR`) and a LAMBDA call around the LOOKUP are unprobed and unchanged.
Status: VERIFIED (excel-graph)
Tasks: T-1627, T-1630, T-1632

## MAP and SCAN with a LAMBDA that returns a multi-cell range
Rule: when the LAMBDA of MAP or SCAN returns a multi-cell reference (`MAP(A1:A2,LAMBDA(r,B1:B3))`,
a row range, a range holding an error cell), Excel gives the whole call one `#CALC!` (a scalar, not a
`#CALC!` per element: `ROWS` of it is `#CALC!` too), as BYROW and BYCOL do. It does not depend on
what the range holds (numbers, an error cell among numbers, an error cell among blanks, text and booleans all alike; an all-blank range and a LAMBDA that returns a range for only some elements are unprobed). A one-cell
range is the control: its value is the element (`MAP(A1:A2,LAMBDA(r,J1:J1))` is 4, 4).
Evidence: probes/map-scan-range-result (excel-graph 2026-10-18).
Implemented in: `Model::lambda_result_value`, which still reads the range's existing cells (a cycle
through it is 0, recalc.md). Tasks: T-1792, T-1769.
The same holds for a multi-cell array value, not only a range: `MAP(Data!C4:D12,Data!A8:B12,LAMBDA(x,acc,HSTACK(acc,Data!A7:D11,acc)))`
(gen-m7/611ce3bbfab27cef Calc!A944), `SCAN(Data!B8,Data!B6:E11,LAMBDA(acc,v,HSTACK(..)))` (b24ce74f08d3f91a A430), a SCAN step that is a FILTER
result (7307132bb8820f17 A619) or a TRIMRANGE (2c5ea078f25443a2 A1717), a MAP step raising to a BYROW column (7307132bb8820f17 A1324) and a SCAN
whose array initial value is returned as the accumulator (9359f03972e94026 A1705) are each one `#CALC!` with nothing spilled (excel-graph,
2026-10-21, T-2225). Ours: implemented (T-2306, T-2309): a multi-cell array step of MAP or SCAN is the one `#CALC!` of a multi-cell range (`lambda_result_value`); REDUCE does not go through it.
A 1x1 array step is its one value, as in BYROW: `MAP(Data!B5:E6,..,LAMBDA(w,x,TRANSPOSE(w)))` is the cells' values (gen-m7/b2078b5d702c691a
Calc!A2162; f7e94a817e5385ac A842), `SCAN(..,LAMBDA(w,acc,HSTACK(PROPER(acc))))` "True", "0.3", "0" (7ea6ef6ceabacafd A87) and
`SCAN(..,LAMBDA(w,x,BYROW(w,_xleta.COUNT)))` 0, 1, 1, 1 (2d9b48c6363e7a16 A2954); so is a 1x1 array initial_value of SCAN:
`SCAN(MINVERSE(Data!C7),..)` (23b5367c18059af2 A74) and `SCAN(LEN(WRAPROWS("Hello World",4,0.3)),Data!A7:D7,LAMBDA(z,y,z*y))`, 22, 2.2E+16,
6.6E+15, 0 (7ea6ef6ceabacafd A3615) (excel-graph, 2026-10-22, T-2254 triage). Ours: implemented (T-2407): `lambda_result_value` reads a 1x1 array as its one value.
Tasks: T-2309, T-2306, T-2407

## SCAN and REDUCE keep errors in the LAMBDA
Rule: SCAN and REDUCE do not stop at an error. An error initial_value, an error accumulator and an error element are handed
to the LAMBDA like any other value, and SCAN keeps one result per element, so an error step fills its own slot and the scan
goes on. `REDUCE(<#CALC!>,Data!B8,LAMBDA(z,y,(y/Data!B12)<>CHOOSE(y,z,z)))` with B8 and B12 = 0 is `#DIV/0!` (the left operand
first; gen-m7/07d27aa0add0247f Calc!A1433); `REDUCE(<#N/A>,Data!C4:E11,LAMBDA(acc,x,MONTH(1861)))` is 2 (36809e9faa59553f A556);
`REDUCE(<#NUM!>,Data!B9:E10,LAMBDA(w,v,v))` is the last element " x " (2c5ea078f25443a2 A347); `SCAN(<#VALUE!>,Data!A8:B12,COUNT)`
is 1, 1, 2, ... (COUNT skips the error accumulator; 90de8755a2e565e6 A1223); `SCAN(JIS(..),Data!C6,COUNT)` is 1 although JIS is
`#NAME?` on the web (9359f03972e94026 A569); `SCAN(-0.5,Data!B2:E8,LAMBDA(x,acc,(3.14159^acc)/(x/x)))` spills 28 cells, the first
7.590654881982496E+20 and the later ones `#NUM!` where 3.14159^acc overflows (36809e9faa59553f A679). 39 gen-m7 formulas in all
(T-2306's list). MAP, BYROW, BYCOL and MAKEARRAY already keep per-element errors.
The initial value and the elements also look like references when they come from cells: `REDUCE(Data!A12,Data!C11,PRODUCT)` with
both cells TRUE is 0 (PRODUCT skips booleans in references; 18b80d3fd292d63a A467), `REDUCE(Data!C7,Data!E12,COUNT)` with C7 the text
"1" and E12 -2.5 is 1 (2c5ea078f25443a2 A1767), and `SCAN(Data!A5,Data!A7:C11,PRODUCT)` with A5 "ü" starts at 2.5 (18b80d3fd292d63a
A285). Both are references:
- A285 and A1767 need a reference initial value: a text value would make PRODUCT `#VALUE!` and COUNT 2.
- A467 needs a reference element too. PRODUCT of a reference TRUE and a value TRUE would be 1.
- Elements from a range, a one-cell range included, were already references (T-1442).
- Unprobed initial values:
  - a function result such as `IF(TRUE,A1)`;
  - an INDEX or XLOOKUP that picks a cell (ours: a reference, as `eval_ref_or_index` gives it);
  - a union or a 3-D span (ours: read as values, as before);
  - ISREF of the accumulator (ours: TRUE for a cell).
Evidence: gen-m7 cases above (excel-graph, 2026-10-21, T-2225).
Ours: implemented (T-2306): `fn_scan` and `fn_reduce` hand error values to the LAMBDA and SCAN keeps an error step in its slot.
Since T-2307, `lambda_initial_value` keeps a reference initial_value a reference, as `eval_ref_or_index` binds a LAMBDA argument.
A blank cell taken into an array by HSTACK counts as a 0 argument when an element reaches AVERAGE: `REDUCE(<3x1 text array>,
HSTACK(Data!Z1:Z2,{1;2}),_xleta.AVERAGE)` is 1.125 = AVERAGE(AVERAGE(AVERAGE(AVERAGE(texts,0),1),0),2) (gen-m7/f19581ac44da1653 Calc!A2088,
excel-graph, 2026-10-22). Probed (probes/t2413-blank-in-stacked-arrays, excel-graph, 2026-10-23, T-2413): the element is the
empty value IFERROR gives a blank cell (logic.md "IFERROR / IFNA of a blank cell is an empty value"), not 0 and not a blank cell.
`ISBLANK(INDEX(HSTACK(Z1:Z2,{1;2}),1,1))` is TRUE, TYPE 1 and `&"|"` "|", yet COUNT and COUNTA of it are 1 and
`AVERAGE(<it>,1)` 0.5 (A1:A6). Handed to a LAMBDA it stays that (`MAP(..,_xleta.COUNT)` 1,1,1,1, `MAP(..,LAMBDA(v,ISBLANK(v)))`
TRUE,FALSE,TRUE,FALSE, `REDUCE(0,..,LAMBDA(a,v,a+COUNT(v)))` 4; A7:A11). VSTACK of one-cell blank references (A13:A23) and
EXPAND's pad from a blank cell (`ISBLANK` TRUE for the pad elements, A32) make the same empty value. A blank cell of a *range*
stays a blank cell: over `IF(TRUE,Z1:Z2)` COUNT is 0 and MAP's COUNT 0,0 (A37:A47).
Ours: a LAMBDA's element binding gives the empty value since T-2413 (`lambda_cells`), which fixes A2088; INDEX of the
element, VSTACK and HSTACK of a blank one-cell reference and EXPAND's blank pad give it too since T-2461 (A2, A3, A6, A13,
A16, A20, A32; `SEARCH(HSTACK(Z1),"Hello World",8)` is 8, gen-m9/2a18b2f5a4dc43ac Calc!A79).
More sources (probes/t2461-blank-array-sources, excel-graph, 2026-10-25, T-2461): SORT and FILTER of a blank range make the
empty value too (`MAP(SORT(Z1:Z2),_xleta.COUNT)` 1,1, `COUNT(INDEX(SORT(Z1:Z2),1,1))` 1, ISBLANK TRUE; A1:A4, A7:A9, A11),
but TAKE and DROP of a range return a reference, so their blank cells stay blank cells: `MAP(TAKE(Z1:Z2,2),_xleta.COUNT)` is
0,0, `COUNT(INDEX(TAKE(Z1:Z2,2),1,1))` and `COUNT(INDEX(DROP(Z1:Z3,1),1,1))` are 0 and `ISREF` of either INDEX is TRUE
(`ISREF(INDEX(TAKE({1;2},1),1,1))` FALSE; A5, A6, A10, A27, A28, A36:A39); TAKE of an HSTACK keeps the empty value
(A29). The pads: a blank cell pads EXPAND, WRAPCOLS and WRAPROWS with the empty value (ISBLANK TRUE, COUNT 1; A19:A22), and an
empty pad_with differs by function: `EXPAND(1,2,2,)` pads with the empty value (A18) but `WRAPCOLS({1,2,3},2,)` and
`WRAPROWS(..,2,)` with `#N/A`, as if it were missing (ISNA TRUE; A30, A31, A34, A35), while the empty value an argument
evaluates to pads all three with the empty value (`WRAPCOLS({1,2,3},2,IFERROR(Z1,1))`, the same with the INDEX element, and
EXPAND's: ISBLANK TRUE, COUNT 1, ISNA FALSE; A40:A45). HSTACK and VSTACK of a lone blank cell
are the empty value (`COUNT(INDEX(HSTACK(Z1),1,1))` 1, `VSTACK(Z1)&"|"` "|"; A15:A17, A25), VSTACK of a blank range keeps
the range's cells as the empty value (`COUNT(INDEX(VSTACK(Z1:Z2),1,1))` 1, A23:A24), and a spill shows the empty value as 0
(`MAP(HSTACK(Z1:Z2,{1;2}),LAMBDA(v,v))`, `HSTACK(Z1,1)`, `EXPAND(1,2,2,Z1)`; C1:D7). The empty value is not an omitted
argument: `LAMBDA(x,ISOMITTED(x))(IFERROR(Z1,1))`, the same of the INDEX element, and ISOMITTED in MAP over the HSTACK are
FALSE (A12:A14).
Ours: all of the above since T-2461 (INDEX picks from TAKE and DROP as a reference, `index_source`); ISOMITTED of a
parameter bound to the empty value is FALSE since T-2649 (A12:A14): omission is how a parameter was bound (`Binding::Omitted`
for a missing argument or an empty slot), not its value. A LET name bound to an omitted parameter (`LAMBDA(x,[y],LET(z,y,
ISOMITTED(z)))(1)`), an empty LET value (`LET(z,,ISOMITTED(z))`) and an omitted parameter handed to another LAMBDA
(`LAMBDA(x,[y],LAMBDA(z,ISOMITTED(z))(y))(1)`) stay TRUE, while a LET name bound to the empty value
(`LET(z,IFERROR(Z1,1),ISOMITTED(z))`) is now FALSE like the LAMBDA form, and so is an empty REDUCE or SCAN initial value
(`REDUCE(,{1,2},LAMBDA(a,v,ISOMITTED(a)))`; all unprobed, T-2670).
Status: VERIFIED (excel-graph).
Tasks: T-2306, T-2307, T-2413, T-2461, T-2649

## MAKEARRAY rows and columns must be whole numbers
Rule: MAKEARRAY is `#VALUE!` when rows or columns is not a whole number; it does not truncate as SEQUENCE does. Evidence (gen-m7,
excel-graph, 2026-10-21, T-2225): rows 2.005 (18b80d3fd292d63a Calc!A963), 11111.111 from `SQRT(BYCOL(..,MAX))` (A395), columns 2.5
(7307132bb8820f17 A1, a cell), 2.5 again (A1137), 3.14159 (b24ce74f08d3f91a A382), 1.005 (c36ff5f42a1a4d47 A285), the fractions of
36809e9faa59553f A1285, 90de8755a2e565e6 A409, ac67253061bba16a A1222 and c36ff5f42a1a4d47 A853, and a WRAPCOLS rows array whose first
element is 3.14159 (c36ff5f42a1a4d47 A1705) are all `#VALUE!`.
Ours: `#VALUE!` for a fractional rows or columns (T-2308).
Unverified: a size within one ulp of a whole number (`(0.1+0.2)*10`) is `#VALUE!` in ours (exact test); Excel may round to 15 digits first.
An error in columns comes before a fractional rows: `MAKEARRAY(123456794.36459264,<#NUM!>,_xleta.AVERAGE)` (the columns an
`ACOS(1+2^-52)` product) is `#NUM!` (gen-m7/bc4a69b0b512bc04 Calc!A1028, excel-graph, 2026-10-22). Both sizes are read, errors first
in argument order, before either is validated: `MAKEARRAY(1.5,NA(),..)`, `(-1,NA(),..)` and `(0,NA(),..)` are `#N/A`,
`(1.5,1/0,..)` `#DIV/0!`, `(1.5,ACOS(2),..)` `#NUM!`, `(NA(),1.5,..)` `#N/A` and `(1.5,"x",..)` `#VALUE!` (probes/expand-makearray-past-grid
D9-D17, excel-graph, 2026-10-23). Columns past the grid are built, as HSTACK's are: `COLUMNS(MAKEARRAY(1,16385,..))` is 16385 (D1,
D2) and the bare `MAKEARRAY(4,25000,..)` is `#SPILL!` (gen-m7/21b4eef18424ce94 A3348); rows past the grid are unprobed (the
probe's 1,048,577-row, 1E9 and 1E6-element cells hang excel-graph's workbook open) and stay `#VALUE!` in ours.
Columns past the 2^20-cell bound are built too: `COLUMNS(MAKEARRAY(1,1048577,..))` is 1048577 (probes/t2452-makearray-cols-1048577,
T-2452; ours `#ERROR!`, the cell bound's DEVIATION), while `MAKEARRAY(3,1E15,..)` is `#VALUE!` (gen-m7/b2078b5d702c691a A2553); ours
is `#VALUE!` from 2^31 columns, EXPAND's threshold ("Arrays past the grid").
Ours: both sizes read first, then validated; columns past the grid under the cell bound (T-2408).
Status: VERIFIED (excel-graph)
Tasks: T-2308, T-2408, T-2452

## SORTBY with a single-value by_array or a fractional sort_order
Rule: a by_array sorts the array's rows when it is one column as tall as the array, and its columns when it is one row as wide
as the array; every by_array of the call must sort the same way, or the call is `#VALUE!`. So a single value (a number, text, a
boolean, a one-cell range, `{7}`) is both a one-wide row and a one-tall column: it leaves a one-column or one-row array in its
own order and is `#VALUE!` on any other array. With A1:A3 = 3,1,2 and B1:B3 = 30,10,20 (probes/t2310-sortby-single-by-array, excel-graph, 2026-10-22, T-2310):
- `SORTBY(A1:A3,5)`, `0.5`, `-1`, `"x"`, `TRUE`, `{7}`, `C1` (a cell) are 3,1,2 (H1-H49); `SORTBY(A1:B3,5)`, `0.5`,
  `1234567.89123`, `C1` are `#VALUE!` (H13-H33); `SORTBY(A1:A3,NA())` is `#N/A` (H45); `SORTBY(A1,5)` is 3 (H153).
- `SORTBY(A20:C20,5)` (a one-row array) keeps its order, and `SORTBY(A20:C20,A20:C20,-1)` sorts its columns (H53-H65).
- `SORTBY(A1:B3,{2,1})` and `SORTBY(A1:B3,{1,2},-1)` swap the columns (H69, H73); `SORTBY(A1:A3,{3,1,2})` and
  `SORTBY(A1:A3,A20:C20)` are `#VALUE!` (a row as long as the array's height does not sort its rows: H77, H81).
- Mixed directions are `#VALUE!`: `SORTBY(A1:A3,5,1,A1:A3,-1)`, `SORTBY(A1:A3,A1:A3,1,5,1)`, `SORTBY(A1:B3,A1:A3,1,{2,1},1)`
  (H85-H93). A one-value by_array on a one-row array next to a row by_array is unprobed (ours: `#VALUE!`).
A sort_order reads as its floor, snapped up near an integer (the count rule of LEFT and INDEX, `near_integer_count`): -0.0001,
-0.5, -0.9999 sort descending, 1.5 and 1.0000001 ascending; 0.0001, 0.5, 0.9999, -1.5, 2, -2 and -1E-300 (floor -1, snapped
to 0) are `#VALUE!`; `"-1"` is descending and TRUE ascending (H97-H165). That the snap tolerance is LEFT's 2^-22 + 2^-33 is
UNVERIFIED (only -1E-300 and 0.9999 bound it). SORT's own sort_order still truncates (unprobed for fractions).
This is why `SORTBY(Data!E1:E12,MAX(..)%,-1)` (gen-m7/18b80d3fd292d63a Calc!A123:A134), `SORTBY(Data!C1:C12,IFNA(TIMEVALUE(..),..),1)`
(7307132bb8820f17 A966:A977) keep their one-column arrays, `SORTBY(Data!B1:C12,(-1),1)` (0e40208f51c2454c Calc!A2) and the
by_array 0.5 on Data!B1:C12 (7d67c559ce7536fa A1358) are `#VALUE!`, and `SORTBY(Data!B1:E12,Data!E1:E12,-0.0001,..)`
(07d27aa0add0247f A658) sorts descending. The "Invalid values" list of "SORT and SORTBY argument errors" (`SORTBY(arr,key,0)`
`#VALUE!`) still holds.
Evidence: probes/t2310-sortby-single-by-array (42 formulas, excel-graph, 2026-10-22); gen-m7 (excel-graph, 2026-10-21, T-2225).
Ours: implemented (T-2310): `by_array_axes` and `sortby_of` in functions/spill_functions.rs.
Status: VERIFIED (excel-graph)
Tasks: T-2310

## PIVOTBY: an error column total over several value columns is #VALUE! (T-2311)
Rule (excel-graph, 2026-10-22, probes/t2311-pivotby-value-columns-error P!A1-A42, probes/t2311-pivotby-value-cause): with one
function over several value columns, PIVOTBY is a single `#VALUE!` when any cell of its Total columns is an error, the grand total
alone included, although those columns show `""` (with two or more column groups) or show the last value's total (one column
group, P!A35): `PIVOTBY(k,c,u:w,LAMBDA(v,IF(ROWS(v)=4,NA(),SUM(v))))` (grand total only, P!A7), an error under the hidden Total
column of one row (P!A23), an error value in the data that reaches a column total (P!A2), `AVERAGE` of a text-only group (P!A14), `1/0`
everywhere (P!A11). Not `#VALUE!`: errors in group cells or in Total row cells under the column groups (P!A1, P!A22: per cell),
`col_total_depth` 0 (no Total column, P!A9, P!A26, P!A27, though `row_total_depth` 0 still is: P!A25), one value column (P!A3, P!A4),
several functions (`HSTACK(SUM,LAMBDA(v,NA()))`, Total columns shown: P!A5, P!A24), GROUPBY (P!A18, P!A30), text in the hidden
total (P!A33), and an array result (`LAMBDA(v,v)`, P!A12: Excel's cells there are not errors to ISERROR, unexplained; GROUPBY alike: with
`GROUPBY(Data!B1:B12,Data!B1:E12,LAMBDA(acc,acc),3)` as REGEXEXTRACT's return_mode, 26 elements are "", a valid mode where ours has
`#VALUE!`, gen-m7/21b4eef18424ce94 Calc!A1504, T-2360). With the
same functions over one value column, each gen-m7 call below is a pivot with error cells (value-cause `val1` rows). This is
why five generated PIVOTBY calls are `#VALUE!` (gen-m7/18b80d3fd292d63a Calc!A913, 07d27aa0add0247f A708, A992, 90de8755a2e565e6
A1, A285). Unprobed: an error only in a column subtotal.
Ours: implemented (T-2311): `check_totals` (pivotby.rs) computes the Total column cells of each line and the Total row.
Which hidden Total cells count (T-2578, excel-graph, 2026-10-24: derived from a decode of all 256 layouts of four records over up to
four column groups, run during T-2578 and not committed; checked by probes/t2578-pivotby-hidden-total-slot, 7 layouts with 2 and 3
value columns and two column fields, 129 cells, and t2578-pivotby-total-error-scope): Excel computes the Total cell of value column `v`
(0-based) in a row only when that row has a record under the column line at value-column position `G + v` of the grid, the
line `(G + v) div V` with `G` column lines and `V` value columns (as if each line were one value column wide), and never when
that is past the last line. So with `G` 2 and 2 values both look at the second line, so only the rows with a record there count (T-2606, P!C1-C4); with 3 column groups x, y, z and 2
values, value 0 counts in the rows with a y record and value 1 in those with a z record; with 4 groups both look at the
third; with one group only value 0 (the Total cells of the last value column are shown and never count). The Total row has
records under every line, so its cells always count (P!A7, P!A35); the error decides with the true total (power-of-two
sums decode to the row's own records). The gen-m9 calls with 7 column groups have no hidden Total error under their gates and
so are arrays (`PIVOTBY(Data!B1:B12,Data!C1:C12,Data!D1:E12,_xleta.AVERAGE)` 11x17 although row 7's Total cells are `#DIV/0!`).
Consistent with two column fields (four leaf lines).
T-2606 (excel-graph, 2026-10-27, probes/t2606-pivotby-total-gate-layouts): `col_total_depth` -1 (Total column first) gates exactly as
1 (P!D1-D4 = C1-C4): with one column field p, q and 2 values the hidden Total cells count in the row with a q record only, so the
Total column is not a line of the gate in either order. A LAMBDA returning `""` in the gating cell still counts: a record there is
enough: only value 0's row total is an error (`LAMBDA(v,IF(SUM(v)=3,NA(),IF(SUM(v)=2,"",SUM(v))))`), its gating cell q/u is `""`, and
the call is `#VALUE!` (P!F1), as is its control without the `""` (P!G1). Column subtotal lines (`col_total_depth` 2 / -2, same result for both signs;
leaves p/x, p/y, q/x, 2 values, seven rows, one per non-empty leaf set, P!A1-A14) do NOT follow the single-gating-line rule: the
hidden Total cell of value 1 counts in rows r1..r7 whichever lines they hold, value 0 in rows r2, r3, r4 and not in r1, r5, r6,
r7 (r1 = p/x only, r2 = p/y, r3 = p/x+p/y, r4 = q/x, r5 = p/x+q/x, r6 = p/y+q/x, r7 = all). Not monotone in the records (r6 holds r2's
record but differs, apparently), and rows r1-r4 are confounded: the row total equals a subtotal cell there. Not decoded; ours differs (14 of the 28 cells: A1, A2, A4, A6, A7, A9,
A11, A13, B2, B4, B6, B7, B11, B13), baselined, T-2967.
Status: VERIFIED (without subtotals); subtotal layouts UNVERIFIED (probe pending, T-2967)
Tasks: T-2311, T-2578, T-2606

## EXPAND, WRAPROWS and WRAPCOLS: scalar arguments lift, EXPAND's pad_with does not
Rule (probes/m7-argument-modes F7:H8, G42, K5-K7, excel-graph 2026-10-09; probes/m7-lift-take-drop-sortby rows 20-23,
2026-10-21; T-1835): EXPAND's rows and columns, and WRAPROWS's and WRAPCOLS's wrap_count and pad_with, lift over an array
and a range, one call per element whose value is its top-left (`ROWS(EXPAND(A1:A2,{3;4}))` is 2,
`COLUMNS(WRAPCOLS(A1:A3,2,{0;1}))` is 1). EXPAND's pad_with does not lift: an array there makes the whole call `#VALUE!`
(`ROWS(EXPAND(A1:A2,3,1,{0;1}))` is `#VALUE!`, not 3 rows padded with errors). Implemented through the lifting table.
VERIFIED (excel-graph, 2026-10-23, probes/t2365-wrap-expand-tocol-error-lift; T-2365): a WRAPROWS/WRAPCOLS vector that
is an error comes before wrap_count (`INDEX(WRAPROWS(1/0,-1,{0;1}),2,1)` is `#DIV/0!`, `ROWS(WRAPROWS(1/0,2,{0;1}))` is 2)
and before a lifted error in pad_with; a bad wrap_count comes before an error pad_with element (`WRAPROWS(A1:A3,-1,{#N/A;0})`
is `#NUM!`). A not-a-vector `#VALUE!` (2-D array) comes AFTER an error wrap_count: `WRAPCOLS({1,2;3,4},NA())` is `#N/A`,
`WRAPROWS({1,2;3,4},1/0,0)` is `#DIV/0!`, and `WRAPCOLS(EXPAND({1},2,3),FILTER(<empty>))` is `#CALC!` (gen-m7/f7e94a817e5385ac
A3517); a 2-D vector beside a clean lifted argument fills the shape (`ROWS(WRAPCOLS(A1:C3,2,{0;1}))` is 2). EXPAND with an
error array beside a lifted argument is one error per element (`ROWS(EXPAND(1/0,{3;4}))` and `ROWS(EXPAND(NA(),3,{1;2}))`
are 2), and a 1x1 array pad_with is `#VALUE!` (`INDEX(EXPAND(A1:A2,3,1,{0}),3,1)`). TOCOL of an error array beside a lifted
argument also fills the shape (`ROWS(TOCOL(1/0,0,{0;1}))`, `ROWS(TOCOL(1/0,{0;1}))` are 2). The multi-cell range pad_with
is T-2415's; an array pad_with with rows past the grid hung the Graph workbook (unprobed).
A multi-cell *range* pad_with is one `#VALUE!` too, with nothing spilled, whatever the range's shape and whether any cell is padded:
`EXPAND({1;2;3},5,100,Data!A7:D7)` (gen-m7/0624b684889e9228 Calc!A2004), `EXPAND(Data!C11:D12,2,2,Data!E1:E3)` (gen-m7/2c156ba98ef0220a.case.json A185),
`EXPAND("a",4,4,Data!C7:C11)` (2d9b48c6363e7a16 A366), `EXPAND({1,2,3},5,4,Data!B5:C11)` (A2686), a 1x1 result that pads nothing,
`EXPAND(TAN(-Data!A9),1.305,1,Data!B2:B6)` (A830), and one whose spill would collide, `EXPAND(Data!$E8,YEAR(..),2,Data!C1:E3)`
(gen-m7/2c156ba98ef0220a.case.json A1577: `#VALUE!`, not `#SPILL!`); 17 formulas in all (excel-graph, 2026-10-22, T-2254 triage). Ours: implemented, a multi-cell
range is `#VALUE!` as an array is (`fn_expand`, T-2415).
An error wrap_count beats WRAPCOLS's not-a-vector `#VALUE!`: `WRAPCOLS(EXPAND({1},2,3),FILTER(..))` with an empty FILTER is `#CALC!`
(gen-m7/f7e94a817e5385ac A3517, excel-graph, 2026-10-22); ours `#VALUE!` (T-2365).
Status: VERIFIED (excel-graph) for the lifts; UNVERIFIED for the vector-error part
Tasks: T-1835, T-2365, T-2415

## WRAPROWS and WRAPCOLS with a huge wrap_count
Rule: a wrap_count of 2^31 or more is `#NUM!` in both functions, padded or not, before any clamp to the vector's length.
2^31 - 0.5, 2^31 - 1, 2^20 and 1048576.5 still wrap (truncated, then clamped: `COLUMNS(WRAPROWS(SEQUENCE(3),2^31-1))`
is 3). 2^31, 2^32, 2^53, 1E+14, 1E+15 - 1, 1E+15 and 1E+300 are `#NUM!`. So the count is checked as a 32-bit signed integer.
The gen-m7 cells agree: `WRAPROWS(Data!E6:E10,1000000000000001.5,..)` (gen-m7/7d67c559ce7536fa Calc!A384),
`WRAPCOLS(Data!D5:D7,1000000000000044.1)` (8044dce50616cfe9 A1187), `WRAPCOLS(Data!E9:E12,1000000000000000.1)`
(c36ff5f42a1a4d47 A877) and `WRAPROWS(..,ODD(2^53)*42,1E-15)` (7307132bb8820f17 A1372; ODD(2^53) is 2^53 itself,
not an error) are `#NUM!`. A wrap_count below 1 is `#NUM!` as before ("Stacked results past the grid": a count past the
length that is below 2^31 is clamped, `COLUMNS(WRAPROWS(SEQUENCE(16385),16385))` is 16385).
Evidence: probes/t2312-wrap-count-threshold B1:D20, E1:E6 (excel-graph, 2026-10-22); gen-m7 (excel-graph, 2026-10-21, T-2225).
Ours: matches (T-2312, `get_wrap_count` in functions/lookup_and_reference/wrapcols_wraprows.rs).
Status: VERIFIED (excel-graph)
Tasks: T-2312

## Arrays of LAMBDAs
Rule: an array element can be a LAMBDA. SCAN with a LAMBDA accumulator gives one per step, and so do MAKEARRAY, VSTACK of LAMBDAs
and the array functions that pick elements (TAKE, CHOOSECOLS, TOCOL, HSTACK, XLOOKUP, INDEX): MAP and REDUCE call them,
`MAP(SCAN(LAMBDA(z,z),{1,2,3},LAMBDA(f,x,LAMBDA(z,f(z)*10+x))),LAMBDA(g,g(0)))` is 1, 12, 123 (with a second array `{1,2,3}` passed
on, 11, 212, 3123; REDUCE adding `g(1)`, 1246), `MAP(VSTACK(LAMBDA(x,x),LAMBDA(x,x*2)),LAMBDA(g,g(5)))` 5, 10, and a LET name bound
to `INDEX(s,1,3)` or an XLOOKUP of the array calls (123, 12). MAP over a lone LAMBDA calls it (`MAP(LAMBDA(x,x*3),LAMBDA(g,g(5)))`
15). A cell shows each element as `#CALC!` (the bare SCAN spills three; INDEX of one, TAKE, CHOOSECOLS, HSTACK, TOCOL and VSTACK
of them likewise, and IFERROR keeps them). The array is an array: ROWS 1, COLUMNS 3, `TYPE` 64; an element is a LAMBDA, `TYPE` 128
and `ISERROR` FALSE (also element-wise). Read as a value an element is `#VALUE!`: SUM is `#VALUE!`, COUNTA counts the three and
COUNT none, `SCAN(..)=1` is three `#VALUE!`, and calling the array (`LET(s,SCAN(..),s(0))`) is three `#VALUE!`. ARRAYTOTEXT shows
each as the letter lambda (`λ, λ, λ`). CHOOSE and IF over an array condition do not pick LAMBDAs: `CHOOSE({1,2},LAMBDA(x,x),
LAMBDA(x,x*2))` and `IF({TRUE,FALSE},..)` are two `#VALUE!`, and MAP over them is two `#VALUE!` too.
Evidence: probes/lambda-arrays B1:B43, F1:F9 (excel-graph, 2026-10-23, T-2039); probes/lambda-escaping-closures H2:J2.
Ours: `ArrayNode::Lambda` holds the lambda id; SCAN, MAKEARRAY, BYROW, BYCOL and MAP keep a LAMBDA result as one
(`calc_result_to_element`) and `result_to_array` makes a lone LAMBDA a one-element array of it; an element read as a value is
`#VALUE!` and a cell shows `#CALC!`. A LET or LAMBDA call whose result is an array of LAMBDAs gives each of them its captures
("LAMBDAs that outlive their call"; walked only when the scope made a LAMBDA). CHOOSE and IF keep the `#VALUE!` of
`calc_result_to_array_node`. Not probed: UNIQUE, SORT or a comparison between two LAMBDA elements (ours: UNIQUE keys each LAMBDA
by its id; T-2477).
Status: VERIFIED (excel-graph)
Tasks: T-2039, T-2477

## TAKE and DROP with an omitted count
Rule: an omitted rows or columns argument of TAKE and DROP keeps every row or column: `TAKE({1,2,3;4,5,6},,-1)` is the last column
(3;6), `TAKE(A1:B3,,1)` the first, `DROP({1,2,3;4,5,6},,1)` the last two columns, and `TAKE(x,)`, `TAKE(x,,)` and `DROP(x,,)` the
whole array. A blank cell is 0, not omitted: `TAKE(x,D1,1)` over a blank D1 is `#CALC!`.
Evidence: probes/take-drop-omitted-rows F1:M6 (excel-graph, 2026-10-23, T-2039); probes/lambda-arrays B13.
Ours: `edge_spans` reads an omitted count as "all" (before T-2039 an omitted rows was 0, so `TAKE(x,,-1)` was `#CALC!`).
Status: VERIFIED (excel-graph)
Tasks: T-2039

## SEQUENCE and TAKE: an argument error before zero rows
Rule: SEQUENCE with 0 rows or columns, and TAKE with 0 rows, return an error in another argument rather than `#CALC!`:
`SEQUENCE(0,4,<array start>,<step>)` is `#VALUE!` (gen-m7/07d27aa0add0247f Calc!A817), `SEQUENCE(0,1,LOOKUP(NETWORKDAYS(..),..),
TOCOL(Data!E7:E11))` spills five `#NUM!` (36809e9faa59553f A334),
`SEQUENCE(0,1,ROUNDDOWN(SORT(..),-6),COLUMNS(COTH(..)))` has `#VALUE!` elements (611ce3bbfab27cef A1633) and `TAKE(Data!B9:C10,0,
LN(ROUND(..)))` is `#VALUE!` (7d67c559ce7536fa A1296). `SEQUENCE(0)` alone stays `#CALC!`. A817's error argument (its start, a UNIQUE over an XLOOKUP) is not isolated.
`SEQUENCE(WRAPROWS(Data!C2,3,Data!$A$3)+..,0,..)` (611ce3bbfab27cef A1065) is one `#VALUE!` with no error argument: its rows is
an array (WRAPROWS pads with a blank) of about 1E+15, past the grid, which is `#VALUE!` as `SEQUENCE(1048577)` is, and that also
beats the zero columns; the reading fits the cell but is not isolated by a probe.
Evidence: gen-m7 (excel-graph, 2026-10-21, T-2225).
A zero rows beats an invalid columns: `SEQUENCE(0,<negative>)` is `#CALC!` (c36ff5f42a1a4d47 A359). So SEQUENCE reads all four
arguments, an error value in any of them winning, and only then checks rows (negative or past the grid `#VALUE!`, 0 `#CALC!`), then
columns.
Ours: `sequence_kernel` in that order; TAKE reads both counts before a zero one is `#CALC!` (`edge_spans`; T-2316).
Unprobed: whether an error start beats a negative extent (ours: it does, by the reading above).
Status: VERIFIED (excel-graph)
Tasks: T-2316

## TOROW, TOCOL, SORT and TEXTJOIN lifted arguments (gen-m7)
Rule: TOROW's and TOCOL's scan_by_column lift over an array, an error initial argument filling every element
(`TOROW(<#NUM!>,3,SCAN(1.005,Data!A5:C9,MIN))` spills 15 `#NUM!`: gen-m7/611ce3bbfab27cef Calc!A919; `TOROW(Data!B3:E8,3,WRAPCOLS(..,1))`
lifts over the 1x9 array: 7d67c559ce7536fa A619), and an error scan_by_column beats an invalid ignore (`TOCOL(Data!B1:B5,Data!B8,NA())`
is `#N/A`: 8044dce50616cfe9 A1508). SORT lifts an array by_col (`SORT(Data!D6:E7,3,1e-15,<2x4 booleans>)` is a 2x4 `#VALUE!`:
7307132bb8820f17 A1076), and TEXTJOIN an array ignore_empty (`LAMBDA(y,RIGHT(TEXTJOIN(y,y,y),2))(UNIQUE(Data!C3:D4))` is `#SPILL!`:
ac67253061bba16a A1137). TEXTJOIN's ignore_empty lifts over an array and a range, one join per element read as a scalar
ignore_empty is (probes/textjoin-ignore-empty-lift, excel-graph, 2026-10-23, T-2318): `TEXTJOIN(",",{TRUE;FALSE},"a","","b")`
spills "a,b" and "a,,b", `{TRUE,FALSE;FALSE,TRUE}` is 2x2, a range (`C1:C2`, `C1:D1`, `IF(TRUE,C1:C2)`) lifts like an array, a
number element is TRUE unless 0, a "TRUE" text element is TRUE, an "x" text element `#VALUE!` and an error element that error, each
in its own place; the delimiter and the texts are read whole by every element, so a delimiter error (`TEXTJOIN(#N/A,{TRUE;FALSE},..)`),
a text error and a join over 32,767 characters (`#CALC!`) fill every element, and an array delimiter still cycles
(`TEXTJOIN({"-","+"},{TRUE;FALSE},"a","","b")` is "a-b", "a-+b"). The gen-m7 cell: `LAMBDA(y,TEXTJOIN(y,y,y))({TRUE;FALSE})` is
"TRUETRUEFALSE" twice, so RIGHT of it spills and A1137's spill is blocked.
SORT's lifted by_col elements (probes/t2366-sort-lifted-by-col, excel-graph, 2026-10-27, T-2366), `A1:A3` = 3,1,2: later elements
do not follow SORTBY's `ArrayFirst` (`ISBLANK(INDEX(SORT(A1:A3,1,1,{FALSE,TRUE}),1,2))` is FALSE; with sort_index 2 some are empty, T-2929 below). Values:
`{FALSE,TRUE}` spills 1 then `#VALUE!`, `{TRUE,TRUE}` 3 then `#VALUE!`, sort_index 3 with `{FALSE,TRUE}` `#VALUE!` then 3, and a range
by_col (`E1:E2` = FALSE,TRUE) lifts like an array (`ROWS` 2, `COLUMNS` 1: spills 1 then `#VALUE!`). Of the later elements only the
sort_index 3 one (I11) matches the call's top-left; the second (TRUE) element is `#VALUE!` with sort_index 1 (I1, I7, H18) although a
TRUE first element with sort_index 1 is 3 (H7). The rule (probes/t2929-sort-lifted-by-col-index, -shapes, -errors, -range and
-column, excel-graph, 2026-10-27, T-2929; `A1:D4` = 13,7,22,5; 9,31,2,17; 26,4,15,11; 3,19,8,28): an element whose call fails (an
invalid by_col such as "x", a sort_index past the array) keeps its error and is skipped by the rest of the rule. The first call that
sorts in each row leads it; the leads keep their call's top-left when every lead's by_col is equal (`{TRUE;TRUE;TRUE}`, a range
F1:F3 of TRUE, `{"x";TRUE;TRUE}` and `{TRUE,FALSE;TRUE,FALSE}` keep every row), and otherwise only the first lead does. Every other
element that sorts is broken: empty (spills 0, ISBLANK TRUE, `&"|"` "|", TYPE 1, ROWS of it `#VALUE!`) when the sort_index is the
single key 2 (`2`, `{2}`, a cell holding 2, `2.7`, `"2"`; not `{2,1}`, `{1,2}` or `{2,2}`) and it is either the first broken element
in row-major order or below a broken element that may be empty with the same by_col; `#VALUE!` otherwise. So, with sort_index 2,
`{TRUE,TRUE,TRUE,TRUE}` is 22, empty, `#VALUE!`, `#VALUE!` (a 1x6 likewise), `{"x",TRUE,TRUE,TRUE}` `#VALUE!`, 22, empty, `#VALUE!`,
`{TRUE,"x",TRUE,TRUE}` 22, `#VALUE!`, empty, `#VALUE!`, `{TRUE;FALSE;FALSE}` 22, empty, empty, `{TRUE;TRUE;FALSE}` and
`{TRUE;FALSE;TRUE}` 22, empty, `#VALUE!`, `{TRUE,TRUE;FALSE,FALSE}` 22, empty / `#VALUE!`, `#VALUE!`, and a 3x2 range of TRUE 22,
empty in every row; with sort_index 1, 3 or 4 every broken element is `#VALUE!`. Sort order, by_col FALSE or TRUE, the array's
shape (4x4, 3x1, 1x3, 3x2, an array constant) and a range by_col change nothing. That a failed call between two that sort is skipped
is evidenced for an invalid by_col ("x"); for a sort_index past the array it is by analogy, UNVERIFIED, as is comparing a number
by_col with a boolean one.
Evidence: gen-m7 (excel-graph, 2026-10-21, T-2225); probes/textjoin-ignore-empty-lift (excel-graph, 2026-10-23, T-2318); probes/t2366-sort-lifted-by-col (excel-graph, 2026-10-27, T-2366); probes/t2929-sort-lifted-by-col-index, -shapes, -errors, -range and -column (excel-graph, 2026-10-27, T-2929).
Ours: TOROW is in the lifting table with TOCOL's entry, an error array is every lifted element's result (TOCOL's too, by
analogy with TOROW's evidence: UNVERIFIED, T-2365 probes `ROWS(TOCOL(1/0,0,{0;1}))`), and an error ignore or scan_by_column comes before an invalid one, in argument order
(T-2313; an error ignore beside an error scan_by_column is unprobed); SORT lifts by_col since T-2319 (lifting table: array, sort_index and sort_order read whole, `sort_by_keys` per element, then `sort_lifted_elements` in spill_functions.rs keeps, empties or errs each element by the rule above (T-2929); a range by_col lifts too; the two UNVERIFIED corners are T-2947's); TEXTJOIN is in the lifting table since T-2318 (`ignore_empty` `Lift`, the delimiter and the texts read once by `fn_textjoin`, which builds at most one join per value of ignore_empty; the 255-argument cap is unprobed, Excel's documentation says 252 texts; whether a legacy formula keeps a range delimiter whole, now `Signature::Vector`, is unprobed; T-2467 probes both, an element error beside a text error and a blank cell in a range ignore_empty).
Status: VERIFIED (excel-graph)
Tasks: T-2313, T-2318, T-2319, T-2366, T-2467, T-2929, T-2947

## TOROW and UNIQUE of a lone blank cell
Rule: a one-cell reference to a blank cell stays blank in TOROW and UNIQUE: `TOROW(Data!Z1,3)` drops it and is `#CALC!` (nothing
left; gen-m7/9bb61f6ae34c7e20 Calc!A3029), and `UNIQUE(Data!Z1,TRUE)&..` joins as "" (`TEXTAFTER(UNIQUE(Data!Z1,Data!A$2)&IF(TRUE,
Data!E5,"a"),"")` is "1000000000000000": gen-m7/f7e94a817e5385ac A2942), as a blank range cell would.
Evidence: gen-m7 (excel-graph, 2026-10-22, T-2254 triage).
Ours (T-2409): TOROW, TOCOL and UNIQUE keep a lone blank reference blank (`Model::eval_to_array_keep_blank`); before, it was read as 0;
TOCOL keeps a lone blank too (gen-m9/8dd2312c96effb1b Calc!A399 `TOCOL(Data!Z1,3,..)` is `#CALC!`; gen-m9/928d08e3e1eecb5f A333 `TOROW(Data!E1,1)`; gen-m9/7980ffb6095a340d A294 `UNIQUE(Data!D$4)`); SORT and TAKE still read it as 0 and are unprobed (T-2409).
Status: VERIFIED (excel-graph)
Tasks: T-2409

## GROUPBY and PIVOTBY lift over an array field_headers or total depth
Rule: an array (or a multi-cell range) given as GROUPBY's `field_headers` or `total_depth`, or PIVOTBY's `field_headers`,
`row_total_depth` or `col_total_depth`, lifts the call. The result has the array's shape, and each element is the top-left cell
of the call with that element in the argument's place, or that call's error. A one-element array lifts too.
- `GROUPBY(k,v,SUM,{0,1})` is `a`, `a` (the first key of each result); `{0;3}` is a column of two; `{3}` is the one cell `a`;
  `{"x",1}` and `{5,1}` are `#VALUE!`, `a` (A57-A113).
- PIVOTBY's top-left cell is its empty corner: `PIVOTBY(r,c,v,SUM,{0,1})` and `PIVOTBY(r,c,v,SUM,0,{0,1})` are `""`, `""`
  (A121, A129). With an invalid later argument (a text `row_sort_order`) every element is `#VALUE!` (A137).
- Each element's call checks its arguments in its own order. Text that is no number is `#VALUE!` in the element's place, ahead
  of a later argument's error value, and an error element is that error: `GROUPBY(k,v,SUM,0,{1,"x",#N/A},,,1/0)` is `#DIV/0!`,
  `#VALUE!`, `#N/A` (A97).
- A multi-cell range lifts as an array does (`Data!D1:E1` holding 0 and 1, as field_headers or total_depth: A177, A225), and a
  blank element is valid (A185; whether as 0 or as omitted is unprobed: the data has no header to tell them apart).
- An invalid number comes after a later argument's error value (`GROUPBY(k,v,SUM,0,{5},,,1/0)` and the scalar
  `...,0,5,,,1/0)` are `#DIV/0!`: A193, A217), but text that is no number, typed or computed, comes before it,
  scalar or lifted (`GROUPBY(k,v,SUM,0,"x",,,1/0)` is `#VALUE!`: A201). Text read from a one-cell reference is unprobed (ours
  keeps the reference, so the later error wins).
- Two array options broadcast against each other as the element-wise operators do: `GROUPBY(k,v,SUM,{0,1},{0;1})` is 2x2
  (A209:B210), a row against a column is a grid (`{0,2,3}` with `{0;-1}` is 2x3), and a position past a shorter array is `#N/A`
  (`{0,2;3,0}` with `{0;-1;1}` is 3x2 with a row of `#N/A`; `{0,2,3}` with `{-1,1}` is `Total`, `Row Field 1`, `#N/A`;
  args2 A1-A23, A56). A range broadcasts as an array does (A36), and so do PIVOTBY's options, `field_headers` with
  `row_total_depth` and `row_total_depth` with `col_total_depth` (A41, A46). Each element is its own call, so of two invalid or
  error elements the earlier argument's wins: `{#N/A,0}` with `{0;#DIV/0!}` is `#N/A`, `a`; `#N/A`, `#DIV/0!` (A31), and
  `{0,"x"}` with `{0;5}` has `#VALUE!` wherever the text is (A26). A lifted `field_relationship` beside them is read per
  element too (A51).
- `sort_order` does not lift (it takes arrays of its own): `GROUPBY(k,v,SUM,0,1,{1,-1})` with one key is a single `#VALUE!` (A81).
- REGEXEXTRACT with an invalid return mode (-1) is `#VALUE!`, inside a GROUPBY function too (K1-K10). excel-graph returns those
  spilled cells as rich errors it cannot decode, so the probe reads them through INDEX and ERROR.TYPE (A33, A41; T-2397).
- Not yet matched: `SCAN(GROUPBY(..,LAMBDA(w,w)),0,MIN)` is a single `#N/A` and `SCAN(GROUPBY(..,SUM),0,MIN)` a single 0
  (A153, A161; T-2398).
Evidence: probes/groupby-lifted-args (excel-graph, 2026-10-22, T-2359), probes/groupby-lifted-args2 (excel-graph, 2026-10-23,
T-2399); gen-m7/18b80d3fd292d63a A1481, 611ce3bbfab27cef A1584,
b24ce74f08d3f91a A1778.
Ours: `Model::lift_options` (functions/lookup_and_reference/lifted_options.rs) evaluates each option once, broadcasts the array
options with the lifting driver's `broadcast_shape`, and re-runs the call once per class of the elements at a position (a
whole number within the valid range, another number, a boolean, text that is no number, an error), with every other argument
frozen to its value, so neither an array's size nor nesting multiplies the work.
`Model::lift_by_class` is shared with GROUPBY's array `field_relationship` (`Model::lifted_groupby`).
Status: VERIFIED (excel-graph)
Tasks: T-2359, T-2397, T-2398, T-2399

## An empty array as a function argument (T-2436)
Rule: an empty array (`#CALC!`, from TAKE or DROP of nothing) given as a value argument is `#VALUE!` in SWITCH's
expression and case values, IMSUM, IMPRODUCT, and the holidays of WORKDAY and NETWORKDAYS: `SWITCH(1,DROP({1},1),5,6)`,
`SWITCH(DROP({1},1),1,2,3)`, `IMSUM(DROP({1},1))`, `IMSUM("1+i",DROP({1},1))`, `IMPRODUCT(DROP({1},1))`,
`WORKDAY(1,1,DROP({1},1))` and `NETWORKDAYS(1,10,DROP({1},1))`. It stays `#CALC!` in SWITCH's result and default
(`SWITCH(1,1,DROP({1},1),3)`), CHOOSE (index or value), IFS, IF, XLOOKUP's if_not_found, SUM, MAX, ABS, LEN and N; ERROR.TYPE
gives 14 and IFERROR catches it (probes/empty-array-args-sequence A1:A21).
It is any `#CALC!` value, not only an inline TAKE or DROP: a cell holding `=DROP({1},1)` (`SWITCH(Z1,1,2,3)`, `SWITCH(1,Z1,2,3)`,
`WORKDAY(1,1,Z1)`, `IMSUM(Z1)`), FILTER with no match and `MAP({1},LAMBDA(x,{1,2}))` are `#VALUE!` too (A140:A148); `SWITCH(1,LAMBDA(x,x),2,3)`
is 3. In IMSUM an error argument before the empty array wins (`IMSUM(1/0,DROP({1},1))` `#DIV/0!`, `IMSUM("x",DROP({1},1))`
`#NUM!`, `IMSUM(<#DIV/0! cell>,DROP({1},1))` `#DIV/0!`), one after it does not (`IMSUM(DROP({1},1),1/0)` `#VALUE!`; A150:A154, A170:A172).
SEQUENCE: rows and columns both reach 1048576, and an extent that is negative or past it is `#VALUE!` even next to a zero one:
`SEQUENCE(0,1048576)` and `SEQUENCE(0.0123,16385)` are `#CALC!`, `SEQUENCE(0,1048577)`, `SEQUENCE(0,-1)` and
`SEQUENCE(0.0123,123456789.123)` `#VALUE!` (so the oversized columns are the cause, gen-m9 f280df8ba95f96b9 A222); with a
non-zero rows a columns past the grid's 16384 gives an array (A155:A157: `COLUMNS(SEQUENCE(1,20000))` is 20000, `COLUMNS(SEQUENCE(1,1048576))` 1048576, `ERROR.TYPE(SEQUENCE(1,1048576))`
`#SPILL!`) up to 1048576, `#VALUE!` above (A158). Ours builds them since T-2600 within the cell bound. An extent is floored after adding 2^-23 (the single-precision epsilon, 1.1920929E-7): `SEQUENCE(4,-1.19E-7)` and
`SEQUENCE(0,-1E-8)` are `#CALC!` (a zero extent; gen-m7 c36ff5f42a1a4d47 A359), `SEQUENCE(4,-1.2E-7)` and `SEQUENCE(0,-1)`
`#VALUE!`, and `SEQUENCE(0.99999999,1)` is 1, `SEQUENCE(0.999999,1)` `#CALC!`. The epsilon is probed near 0, 1 and 1048577 only; for large extents a relative or float32 rounding is UNVERIFIED.
Evidence: probes/empty-array-args-sequence (excel-graph, 2026-10-24), probes/t2436/computed-scalar, probes/t2436/f280df8ba95f96b9.
Ours: `CalcResult::is_empty_array` in those readers.
Status: VERIFIED (excel-graph)
Tasks: T-2580

## GROUPBY and PIVOTBY in the gen-m9 sweep (T-2436)
Rule: Excel computes these calls that ours rejected or computes differently: `PIVOTBY(Data!B1:E12,Data!B1:D12,Data!D1:E12,
_xleta.AVERAGE)` is a 16x30 array and `PIVOTBY(Data!B1:B12,Data!B1:C12,Data!B1:C12,_xleta.AVERAGE,1)` 13x25 (ROWS and COLUMNS
of them; ours was `#VALUE!`: their hidden Total cells that are errors are not computed, see "PIVOTBY: an error column total over
several value columns"), and `AVERAGEA(GROUPBY(Data!B1:E12,Data!B1:D12,_xleta.PRODUCT,0,2),{0.02,0.027},Data!B7:B7)` is
-4.454221888962187E+22 (GROUPBY 20x7; ours -4.161608772169051E+22). The GROUPBY differs only in its blank keys: Excel's are
true blanks that COUNT and AVERAGEA skip (COUNT 91, ours 100 with 0 for each; probes/t2578-groupby-product-subtotals), the
"Blank keys show as true blanks" rule above (T-2460). Over a LAMBDA(x,x) function the results' elements make MAX, HARMEAN and
AVERAGE `#N/A` and COUNT 142 (gen-m9 06b4e8f5de409d5e A366, 68d8ffb69ff739d8 A285, 7c20351562a2d71d A381, e85daedce5c2675e
A35), which the aggregates' reads of nested cells decide (SPEC-0018 rule 4; T-2594, since T-2593 made the cells nested).
Evidence: probes/t2436 (excel-graph, 2026-10-24), the cases named in T-2578; probes/t2578-* (excel-graph, 2026-10-24).
Ours: the PIVOTBY shapes since T-2578; the GROUPBY blank keys are T-2460's, the LAMBDA(x,x) cells T-2594's.
Status: VERIFIED (excel-graph)
Tasks: T-2578, T-2460, T-2593, T-2594

## Array-value edges in the gen-m9 sweep (T-2436)
Rule (one gen-m9 cell each, probes/t2436, excel-graph, 2026-10-24; mechanisms probed in probes/t2579-array-value-edges,
excel-graph, 2026-10-24, T-2579; there D!A1:A6 = 1, blank, TRUE, "x", 5, =1/0 and D!B1:B6 = 2, 4, .., 12):
- An array value may be wider or taller than the grid: `COLUMNS(MAKEARRAY(1,20000,LAMBDA(r,c,c)))` is 20000 (B20; INDEX,
  SUM and `+1` of it work, B21, B22, B30), `ROWS(MAKEARRAY(1048577,1,..))` 1048577 (B23), `COLUMNS(SEQUENCE(1,16385))`
  16385 (B24), HSTACK and TRANSPOSE build 20000 columns (B28, B29), and only a spill of one is `#SPILL!` (B26:
  `ERROR.TYPE(MAKEARRAY(1,16385,..))` in a cell). `MAKEARRAY(4,DATE(1952,-2.5,123456789.123),..)` has COLUMNS 51638
  (B25; gen-m9 4250f880ab640c49 A237 is `#SPILL!`). Ours: `#VALUE!` for B23, B24, B26 (`limits::check_array_size`; T-2600).
- IMSQRT, IMABS and the other IM functions do not lift over a multi-cell reference (`ROWS(IMSQRT(D!B1:B2))` off its rows is
  `#VALUE!`, B44; through LET too, B48, B50), but lift over an array (`IMSQRT(REDUCE(0,1,LAMBDA(a,b,{2;8})))` has 2 rows,
  B46), while SQRT and ABS lift over both (B42, B51). A REDUCE whose LAMBDA returns a range gives that reference
  (`ROWS(REDUCE(0,1,LAMBDA(a,b,D!B1:B2)))` 2, B43; IMSQRT of it `#VALUE!`, B40), and so does one returning TRIMRANGE of a
  range: the step is the trimmed reference (B49 `#VALUE!`; gen-m9 df083d85b436b49a A20). Ours since T-2596: REDUCE's
  LAMBDA caller keeps a TRIMRANGE body's reference (`LambdaCaller::keeping_trimrange_references`); a TRIMRANGE wrapped in
  IF or LET, and the other LAMBDA helpers returning TRIMRANGE, are unprobed.
- COMBIN lifts over arrays and ranges as COMBINA does: `INDEX(COMBIN(18,{1,2;3,4}),2,2)` is 3060, `SUM(COMBIN(18,FILTER(..)))`
  21777.000000000004, `SUM(COMBIN(D!B1:B3,1))` 12, a text element `#VALUE!` (B60-B68; gen-m9 c18dbf21f11093f3 A174's inner
  `COMBIN(18,FILTER(..))` is 1).
- IFERROR and IFNA over a multi-cell range give its values with each blank cell as 0: `COUNT(IFERROR(D!A1:A6,-1))` is 4
  (1, 0, 5, -1), AVERAGE 1.25, AVEDEV 1.875, DEVSQ 20.75, SUM 5, `INDEX(IFERROR(D!A1:A6,-1),2)&"|"` "0|" (B70-B84; gen-m9
  fec39efaba9284c3 A253; through TRIMRANGE of a range too, B74, B84). TRUE and "x" stay a boolean and text, which AVEDEV skips in an array as in a range (B79, B80). IFNA alike
  (`COUNT(IFNA(D!A1:A6,-1))` is 3, the `#DIV/0!` kept; `INDEX(IFNA(..),2)&"|"` "0|": B85, B86). A one-cell range is the
  blank's empty value, not 0 (`ISBLANK(IFERROR(D!A2:A2,1))` TRUE, `&"|"` "|": B87, B88; logic.md "IFERROR / IFNA of a blank
  cell").
Evidence: probes/t2436 (excel-graph, 2026-10-24); probes/t2579-array-value-edges (excel-graph, 2026-10-24).
Ours (since T-2579): COMBIN lifts (`fn_combin`, lift/kinds/general.rs) and `if_error_like` reads a multi-cell range's
blanks, and those of the range a direct TRIMRANGE keeps, as 0 (`blanks_as_zero`); wrapped forms (LET, IF, TAKE, DROP, a
lifted TRIMRANGE), IFNA through TRIMRANGE and a TRIMRANGE trimmed to one blank cell are unprobed (T-2597); wide arrays
T-2600; the REDUCE/TRIMRANGE reference since T-2596.
Status: VERIFIED (excel-graph)
Tasks: T-2579, T-2600, T-2596
