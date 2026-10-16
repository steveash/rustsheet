# Criteria (COUNTIF/SUMIF family, D-functions, wildcard lookups)

## "<>text" counts non-text cells
Rule: `COUNTIF(range,"<>x")` counts every cell not equal to `x` (case-insensitive): numbers,
booleans, errors and truly blank cells included.
Evidence: probes/criteria-gaps C1 (`"<>abc"` → 10 of 12 cells: 10, TRUE, #N/A, a blank and the
other texts; "abc" and "ABC" excluded), C12 (`"<>ABC"` → 10); probes/criteria-operands C9.
Oracle: excel-graph, 2026-10-05 (T-0011).
Status: VERIFIED
Implemented in: functions::criteria::result_is_not_equal_to_string

## Wildcard text matching
Rule: `*` and `?` match whole text values; non-text never matches; the pattern `"abc~"` does not
match "abc".
Evidence: probes/criteria-grammar C8 (`"abc~"` → 0), C9 (`"*"` → 4: the four text cells including
"" and the text "#N/A"; the number and TRUE excluded), C10 (`"a?c"` → 1). Oracle: excel-graph, 2026-10-04.
Status: VERIFIED

## Wildcard escapes, case and trailing tilde
Rule: `~` escapes only `*` and `?` (`"~*"` matches the text "*", `"a~*c"` "a*c", `"a~?c"` "a?c").
Before any other character, `~` is a literal tilde: `"a~c"` matches "a~c", and a trailing lone `~`
is literal (`"abc~"` matches "abc~", not "abc"). `~~` is **not** an escape for one tilde: `"~~"`
does not match the text "~" and `"a~~c"` does not match "a~c". Matching, with or without
wildcards, is case-insensitive (`"ABC"` → abc and ABC; `"A?C"`, `"*C"` → 5).
Verified with COUNTIF only, for patterns without `*`/`?`; inside wildcard patterns and in MATCH/XMATCH see "Tildes inside wildcard patterns and wildcard-mode lookups" below. D-functions are unprobed for `~~`.
Evidence: probes/criteria-gaps C4–C14 over 10, abc, TRUE, #N/A, blank, ABC, abc~, a*c, a?c, a~c,
*, ~: 1, 1, 1, 0, 0, 1, 2, 5, 10, 5, 1; probes/criteria-grammar C8. Oracle: excel-graph, 2026-10-05 (T-0011).
Status: VERIFIED
Implemented in: functions::criteria::wildcard_text_matcher

## Tildes inside wildcard patterns and wildcard-mode lookups
Rule: once a criterion holds `*` or `?`, or always for a MATCH(..,0)/XMATCH(..,2) text lookup value, `~` makes the next
character literal whatever it is: `~x` reads as `x`, `~~` as one `~`, `~*`/`~?` as `*`/`?`; a trailing `~` is dropped.
This differs from the plain-text criterion rule above, where `~` before a non-wildcard and a trailing `~` stay literal.
Over Data!A1:A8 = a~c, ac, abc~, abc, ~, a~~c, a~b, a~bx (SUMIF weights 1..128): `"a~c*"` 2 (ac), `"~~*"` 16 (~),
`"abc~*"` 0, `"*~"` 255, `"a~~c*"` 1, `"*~c"` 43, `"a~?*"` 0, `"a~b?"` 8 (abc); `MATCH("a~c",..,0)` 2,
`MATCH("abc~",..,0)` 4, `MATCH("~~",..,0)` 5, `MATCH("a~~c",..,0)` 1, `MATCH("~",..,0)` #N/A; `XMATCH("a~c",..,2)` 2.
Evidence: probes/criteria-wildcard-tilde (excel-graph, 2026-10-16, T-1642).
Status: VERIFIED
Implemented in: `WildcardPattern::new` (functions/criteria.rs)

## Wildcards match line breaks
Rule: `?` matches one CHAR(10) or CHAR(13) and `*` spans them, in criteria and in wildcard MATCH/XMATCH alike. Over
Data!A1:A5 = "a"&CHAR(10)&"b", CHAR(10), "ab", "a"&CHAR(13)&"b", "a"&CHAR(10) (SUMIF weights 1, 2, 4, 8, 16):
`"a*"` 29, `"?"` 2, `"a?b"` 9, `"*b"` 13, `"*"` 31, `"a*b"` 13, `"???"` 9, `"*"&CHAR(10)&"*"` 19,
`"a"&CHAR(10)&"*"` 17; `MATCH("a?b",..,0)` 1, `MATCH("?",..,0)` 2, `XMATCH("a*b",..,2)` 1, `COUNTIFS(..,"a*")` 4.
Evidence: probes/criteria-wildcard-newline (excel-graph, 2026-10-16, T-1642).
Status: VERIFIED
SEARCH agrees (probes/search-wildcard-newline, excel-graph 2026-10-16, T-1666): `SEARCH("a?b","x"&"a"&CHAR(10)&"b")` 2,
`SEARCH("a*b","a"&CHAR(10)&"b")` 1, `SEARCH("?",CHAR(10))` and `SEARCH("?",CHAR(13))` 1, `SEARCH("a?b","xa"&CHAR(13)&"b")` and `SEARCH("a*b","xa"&CHAR(13)&"yb")` 2, `SEARCH("?b","xa"&CHAR(13)&"b")` 3, `SEARCH("*b","xxb",2)` 2 and `,3)` 3, `SEARCH("*"&CHAR(10)&"*","ab"&CHAR(10)&"c")` 1;
over "😀b": `"?b"` 2, `"??b"` 1, `"???b"` #VALUE!, `"*b"` 1; `SEARCH("?","😀")` and `SEARCH("??","😀")` 1, `"???"` #VALUE!;
`SEARCH("a?b","😀a😀b",2)` #VALUE! (the pair is two units); `SEARCH("*","")` and `SEARCH("?","")` #VALUE!.
Status: VERIFIED
Implemented in: `WildcardPattern` (functions/criteria.rs), which matches any unit for `?`/`*`; SEARCH uses its
unanchored `find_from`. The regex translation both used before (`.` skips `\n`) is gone.

## D-function plain-text equality matches by prefix
Rule: in DSUM/DCOUNT/DGET/... a plain text criterion (no operator, no wildcard) matches cells that *start with* it ("ab" matches "abc"); every other part of the grammar is shared with COUNTIF.
An explicit leading `=` (`"=ab"`) turns the prefix match into an exact match (probes/criteria-operands D18).
Evidence: probes/criteria-grammar C11 (`DCOUNTA` with criterion "ab" over ab, abc, xab → 2).
The prefix is read unit by unit, so a lone surrogate is a unit of its own (probes/d-function-prefix-lone-surrogates, excel-graph 2026-10-16, T-1667): over H, L, "😀", U+FFFD, "x", H&"a", "a"&L, "a" (v = 1, 2, 4, ..., 128), `DSUM` with criterion H gives 37 (H, "😀", H&"a"), L gives 2 (L only, not "a"&L), "😀" gives 4, U+FFFD gives 8 (never a lone half), H&"a" gives 32, "a"&L gives 64, "a" gives 192 ("a" and "a"&L), `"="&H` gives 1 and `"="&L` gives 2, "x" gives 16; `DCOUNTA` counts agree (H 3, "a" 2).
Oracle: excel-graph, 2026-10-04.
Status: VERIFIED
Implemented in: functions::criteria::build_criteria_with (CriteriaFlavor::Database)

## D-function criteria are not trimmed into an operator criterion
Rule: leading spaces stop a criterion from being an operator criterion: `"  > 1 "` and `" >1"` are
plain-text criteria (DSUM over Qty 1, 2, 4 → 0). After the operator, though, the operand is read as
a number with surrounding spaces ignored: `">1 "`, `"> 1"` and `">"&" 1 "` (`"> 1 "`) all act as
`>1` (→ 6) in D-functions; COUNTIF behaves the same: over 1, 2, 4 `"> 1"`, `">1 "`, `"> 1 "` → 2, `">= 1"` → 3, `"< 2"` → 1,
`"= 1"`, `"=1 "`, `"1 "`, `" 1"`, `"=  1  "` → 1, `"<> 1"` → 2; only ASCII spaces are ignored (`">"&CHAR(9)&"1"` → 0). Plain-text
D-function criteria keep their spaces in the prefix match: `" ab"` matches only " ab", `"ab "`
only "ab " (over ab, "ab ", " ab"), and the prefix match is case-insensitive (`"AB"` → 2).
Evidence: smoke/d-function-criteria Sheet1!J2 (`"  > 1 "` → 0); probes/criteria-gaps C15 (`" >1"`
→ 0), C16–C18 (→ 6), C22 (→ 2), C19–C21 (1, 1, 2); probes/criteria-padded-operand C1–C18. Oracle: excel-graph, 2026-10-04 and 2026-10-05 (T-0011).
Status: VERIFIED
Implemented in: functions::database::criteria_cell_matches (no trimming before the operator), functions::criteria::parse_criterion_number (spaces around the numeric operand)

## Numeric criteria against text cells and non-ASCII spaces
Rule: a numeric criterion (`"1"`, `"1 "`, `" 1"`, `"= 1"`, `"=1 "`) matches number cells and text cells
whose ASCII-space-trimmed text parses to that number (`"1"`, `"1 "`, `" 1"`). CHAR(160) is not
ignored: `"1"&CHAR(160)` and `CHAR(160)&"1"` (and `"="&CHAR(160)&"1"`) are text criteria that match
only the cell with exactly that text, and CHAR(160)-padded text cells never match a numeric
criterion. `"<> 1"` / `"<>1 "` count every cell except number 1 (text "1" still counts, 5 of 6).
`">= 1"`, `"> 0 "` match number cells only (→ 1). `"<>"&CHAR(160)&"1"` → 5 (all but the exact text).
`">"&CHAR(160)&"0"` (a text comparison) matches 4 of the 5 text cells, all but " 1": text order puts
CHAR(160) right after the space and before digits (coercion.md "Text comparison uses Excel's
collation", T-0244).
Evidence: probes/criteria-padded-text C1–C14 over A1:A6 = "1", "1 ", " 1", CHAR(160)&"1",
"1"&CHAR(160), 1 → 4,4,4,4,4,5,5,1,1,1,1,1,4,5. Oracle: excel-graph, 2026-10-05 (T-0242).
Status: VERIFIED
Implemented in: functions::criteria::result_is_equal_to_number; C13 ordering in functions::util::collation::cmp_text_excel (T-0244, T-0220)
Tasks: T-0242

## "=" matches blank cells, not an empty-string cell
Rule: `COUNTIF(range,"=")` counts truly blank cells but not a cell holding the empty string, while
`COUNTIF(range,"")` counts both.
Evidence: probes/criteria-grammar C3 (`""` → 1), C4 (`"="` → 0) over a range with one empty-string
cell and no blank cells; probes/criteria-gaps C2 (`"="` → 1), C3 (`""` → 1) over a range with one
blank cell and no empty string. Oracle: excel-graph, 2026-10-04 and 2026-10-05 (T-0011).
Status: VERIFIED
Implemented in: `build_criteria_with` (`crates/rustsheet-engine/src/functions/criteria.rs`), Standard flavor.
Also verified for SUMIF, AVERAGEIF, COUNTIFS, SUMIFS, AVERAGEIFS, MAXIFS (`"="`) and MINIFS (`""`): over A1:A5 = 1,
"" (empty-string cell), blank, 4, "x" with B = 10,20,40,80,160, `"="` selects only the blank cell
(SUMIF 40, AVERAGEIF 40, COUNTIFS 1, SUMIFS 40, AVERAGEIFS 40, MAXIFS 40) and `""` selects both
(SUMIF 60, AVERAGEIF 30, COUNTIFS 2, SUMIFS 60, AVERAGEIFS 30, MINIFS 20). Evidence:
probes/criteria-ifs-bare-equals C1-C12. Oracle: excel-graph, 2026-10-05 (T-0246).
Tasks: T-0227, T-0246

## `<`, `>`, `<=`, `>=` with a boolean or error operand
Rule: `COUNTIF(r,">TRUE")`, `"<TRUE"`, `">#N/A"` and `"<#N/A"` match nothing, even over TRUE or
#N/A cells; `"<=TRUE"`/`">=TRUE"` (and `<=`/`>=` with `#N/A`) behave as equality and count exactly
the matching boolean / error cells. The operand is never compared as text ("abc" is not <= TRUE).
Evidence: probes/criteria-operands C1-C8 over 10, "abc", TRUE, #N/A, "abd", 5: 0,0,1,1,0,0,1,1.
Oracle: excel-graph, 2026-10-04.
Status: VERIFIED
Implemented in: functions::criteria::bool_or_error_operand_matcher

## Text `<=` / `>=` criteria, and dotted dates are text in en-US
Rule: `"<=text"` and `">=text"` compare the cell text with the operand, case-insensitively, like
`"<"` and `">"`: over "10.1.2026", "10.3.2026", "10.6.2026", `">=10.3.2026"` counts 2,
`"<=10.3.2026"` 2, `">=1.3.2026"` 3 and `"<=1.3.2026"` 0. Dotted text is not a date in en-US
(dates.md, "The en-US date-text grammar"), so these criteria are text comparisons, and DMIN, DMAX,
DSUM, DCOUNT, SUMIF and DGET over that table follow them (900, 2100, 4200, 3, 4200, 1200).
Month-name date text is a date in criteria: over 46204, 46174, 46235 (1 Jul, 1 Jun, 1 Aug 2026),
`">=July 1, 2026"` counts 2, `"<Jul 2026"` 1 and `"July 1, 2026"` 1.
Evidence: probes/date-text-dotted-criteria E1–E16. Oracle: excel-graph, 2026-10-05.
Status: VERIFIED
Implemented in: `functions::criteria::result_is_less_or_equal_than_string`,
`result_is_greater_or_equal_than_string` (T-0024; they compared the wrong way round before)

## Criteria kinds in COUNTIF and D-functions (more evidence)
Rule: `"<>abc"`, `"<>TRUE"`, `"<>#N/A"` count every non-matching cell (6 of 7, blank included);
`"=TRUE"` / `"=#N/A"` count the boolean / error cells; `">abc"`/`"<abd"` compare text. In
D-functions (DCOUNTA) `"<>ab"`, `"a*"`, `"ab?"`, a numeric criterion cell, a boolean criterion
cell and `"<>3"` follow the shared grammar; an explicit `"=ab"` is exact, not a prefix (1 over ab, abc, xab, 3, TRUE, where plain `ab` gives
2), while an error criterion cell (`=NA()`) matches nothing.
Evidence: probes/criteria-operands C9-C16, D17-D25. Oracle: excel-graph, 2026-10-04.
Status: VERIFIED

## *IF(S) lift array criteria
Rule: an array or multi-cell range as a criterion makes COUNTIF(S), SUMIF(S), AVERAGEIF(S),
MAXIFS and MINIFS return an array of results, one per criteria element:
`SUM(COUNTIFS(A1:F1,{"bar","baz"}))` = 3. Several array criteria broadcast against each other like
the element-wise operators (arrays.md): a row and a column give a grid
(`COUNTIFS(A1:A6,{"a","b"},B1:B6,{1;2;3})` is 3x2), a length-1 dimension stretches, and a position
past a shorter array uses an `#N/A` criterion, which counts the `#N/A` cells (usually 0, not an
`#N/A` result: `COUNTIFS(E1:E3,{"q","q"},F1:F3,{1,1,1})` is 1 at position 3 because E1 is `#N/A`
and F1 is 1). Each element is computed as the scalar function would be (an AVERAGEIFS element with no
match is `#DIV/0!`). A 1x1 array criterion gives a single value. In a legacy (non-dynamic) formula
Excel intersects a COUNTIF range criterion instead: poi/FormulaEvalTestData_Copy.xlsx
EverythingTests!F296 `COUNTIF(G7:K7,E8:G8)` is 2 (Excel-cached).
Evidence: probes/ifs-array-criteria H1-H39 (oracle: excel-graph, 2026-10-06, T-0207);
poi/70005-countifs.xlsx Sheet1!A3, D3 (3), A4, D4 (4) (oracle: excel-graph, 2026-10-04).
Status: VERIFIED
Legacy formulas (fixtures/ifs-legacy-range-criteria.xlsx, oracle excel-graph 2026-10-07, T-0263): a legacy
formula intersects a multi-cell range criterion for every *IF(S) function, not only COUNTIF: with D1 = "a",
D2 = "c" over A1:A6 = a,b,a,c,b,a and B1:B6 = 1,2,3,1,2,3, a formula in row 1 uses D1 and in row 2 uses D2:
COUNTIF/COUNTIFS 3 and 1, SUMIF/SUMIFS 7 and 1 (SUMIF with a one-cell sum range B1 too), AVERAGEIF(S) 2.333.. and 1,
MAXIFS 3 and 1, MINIFS 1 and 1. The fixture writes MAXIFS/MINIFS with the `_xlfn.` prefix Excel itself
stores. All of them intersect through the criterion arguments' scalar signatures (T-0278).
A failed intersection is not an error result: the criterion becomes the error value `#VALUE!`, which then
matches `#VALUE!` cells (poi/FormulaEvalTestData_Copy.xlsx EverythingTests!E1324, `SUMIF(AA7:AA13,C10:D10,C7)` = 1.1
in row 1324: AA9 is `#VALUE!` and the resized sum range gives C9); with no such cell the count is 0 and AVERAGEIF(S)
is `#DIV/0!`. Inside SUMPRODUCT there is no intersection at all: SUMPRODUCT evaluates its arguments as arrays, so
`SUMPRODUCT(COUNTIF(A1:A6,D1:D2))` is 4, SUMIF/SUMIFS 8, AVERAGEIF(S) 3.33.., MAXIFS 4, MINIFS 2 in any row,
while `SUM(COUNTIF(A1:A6,D1:D2))` intersects (0 off-row) (probes/ifs-legacy-sumproduct, oracle excel-graph
2026-10-08, T-0278).
Beyond *IF(S): which legacy arguments get an array context under SUMPRODUCT (probes/array-context-legacy-sumproduct,
excel-graph 2026-10-09, T-0286; A1:A6 = a,b,a,c,b,a, B1:B6 = 1,2,3,1,2,3, each formula on row 1 and off-row on row 10,
`=B1:B6*1` off-row is `#VALUE!` as the control). Array context, same value on and off the row: arithmetic and wrapped
functions nested in SUMPRODUCT (`ABS(B1:B6)` 12, `ABS(B1:B6-2)` 4, `LEN(A1:A6)` 6, `--(A1:A6="a")` 3,
`--(COUNTIF(A1:A6,D1:D2)>0)` 2), `CHOOSE({1,2},B1:B6,B1:B6)` (24), and the value branches of IF
with a scalar condition (`SUMPRODUCT(IF(TRUE,B1:B6))` 12; a condition that intersects to TRUE with an array branch is not probed). Not array context: the condition of IF still intersects, even directly under
SUMPRODUCT: `SUMPRODUCT(IF(B1:B6>1,B1:B6,0))` is 0 on row 1 and `#VALUE!` on row 10, `SUMPRODUCT(IF(A1:A6="a",1,0))` 1 and
`#VALUE!`, exactly like `SUM(IF(A1:A6="a",1,0))` (presumably it needs CSE; an inference, not probed). LOOKUP's vector and MMULT/INDEX arguments evaluate
their arithmetic as arrays without SUMPRODUCT: `LOOKUP(2,1/(B1:B6>1),B1:B6)` 3, `SUM(MMULT(B1:B6*1,{1}))` 12,
`INDEX(B1:B6*1,2)` 2, on and off the row. An array `row_num` in legacy INDEX gives its first element only:
`SUMPRODUCT(INDEX(B1:B6,{1;2;3}))` and `SUMPRODUCT(INDEX(B1:B6*1,{1;2;3}))` are 1. The engine matches the IF condition since T-1526 (references.md "Legacy formulas intersect the ranges in value
arguments", which also covers IFERROR and IFNA under SUMPRODUCT) and INDEX's indices since T-0294 (below).
CHOOSE with an array index matches since T-1508 (lookup.md, "CHOOSE over an index array
broadcasts its value arguments").

Legacy INDEX puts an
automatic `@` on every array constant in its `row_num`, `column_num` and `area_num`, also under SUMPRODUCT and inside
arithmetic, and FORMULATEXT shows it: `=SUM(INDEX(B1:B6,{2;3}))` is 2 and shows as `=SUM(INDEX(B1:B6,@{2;3}))`,
`SUMPRODUCT(INDEX(B1:B6,{2;3},1))` 2, `INDEX(B1:B6,2,{1,1})` 2, `INDEX((B1:B6,B1:B6),1,1,{2;1})` 1, `INDEX(B1:B6,{2;3}+0)`
shows `@{2;3}+0`, and a legacy cell `=INDEX(B1:B6,{3;2})` shows `=@INDEX(B1:B6,@{3;2})`; a range index intersects
(`SUMPRODUCT(INDEX(B1:B6,C1:C3))` is B2 on row 1 and `#VALUE!` off-row, shown `@C1:C3`), while the one function call
probed gets none: `SUMPRODUCT(INDEX(B1:B6,N(IF(1,{2;3}))))` is 5 with no `@` (the classic trick; the engine matches
since T-1616, N lifting over an array). An array-returning call is intersected like a scalar argument:
legacy `SUMPRODUCT(INDEX(B1:B6,ROW(B1:B2)))` is 1 (B1) on and off the range's rows, and a function in INDEX's first
argument keeps SUMPRODUCT's array context: legacy `SUMPRODUCT(INDEX(ABS(B1:B6),0))` is 21 on and off them
(probes/n-array-lift A12, D2, A13, D3). Evidence: probes/legacy-index-array-index (excel-graph, 2026-10-15, T-0294);
fixtures/external-link-cached-ranges Refs!B89:B92 (`SUM`, `ROWS`, `ISREF` and `INDEX` of a legacy
`INDEX([1]Uses!$A$1:$B$3,{1,2},1)`, Excel-cached). Implemented in `add_index_array_at` (expressions/parser/static_analysis/legacy.rs).

Dynamic edge cases (probes/ifs-array-criteria-edges, oracle excel-graph 2026-10-07): a SUMIFS or COUNTIFS whose ranges
differ in shape with an array criterion is an array of `#VALUE!` (1x2 for `{"a","b"}`), not one scalar `#VALUE!`;
a blank cell in a range criterion (D3, criteria range D1:D3) matched none of the non-empty text cells A1:A6: COUNTIF/COUNTIFS/SUMIFS/SUMIF/MAXIFS
give 0 and AVERAGEIFS `#DIV/0!` at that position, and the COUNTIF result keeps its 3x1 shape (ROWS = 3; the others were probed only at INDEX(...,3,1); unprobed: blank, `""` or 0 cells in the searched range);
for SUMIFS and SUMIF an error in the sum range is `#N/A` only at the criteria positions whose rows select it
(`SUMIFS(E1:E3,A1:A3,{"a","b"})` with E1 = #N/A is #N/A, 5; AVERAGEIFS and MAXIFS probed only at position 1: `#N/A`);
`COUNTIFS(E1:E3,{"a","b"})` ignores it (0 at position 1). A `NA()`/non-range first argument with an array criterion
(`COUNTIF(NA(),{"a","b"})`, `COUNTIFS(NA(),{"a"})`, `SUMIFS(NA(),...)`) made excel-graph reject the whole workbook as unsupported,
so those cases are unprobed.
DEVIATION (directive 5, decision 0014): a lifted call whose walks would visit more than 2^27 cells
(used cells of the walked range x criteria count x distinct consecutive criteria elements) is
`#ERROR!` before any walk; Excel computes it. Example: `SUM(COUNTIF(A1:A3000,SEQUENCE(50000)))`
over 3,000 used cells (unit test `too_many_cell_visits_is_an_error_without_walking`). VLOOKUP/HLOOKUP over an array lookup_value
use the same cap, at elements x evaluated cells of the searched column or row (lookup.md, T-0270; unit test
`lifted_lookup_over_an_array_table_hits_the_visit_cap`).
AGGREGATE's array function_num, options and k lift through the same code and cap (numbers.md "AGGREGATE", T-1038).
GROUPBY and PIVOTBY charge their two-parameter LAMBDA calls against the same cap (arrays.md "GROUPBY and PIVOTBY:
two-parameter LAMBDAs and the cell-visit cap", T-1325).
Implemented in: `Model::lift_criteria` / `lift_ifs` (crates/rustsheet-engine/src/functions/statistical/if_ifs.rs),
`criteria_static_result` (expressions/parser/static_analysis.rs)
Tasks: T-0207, T-0263, T-0278

## SUMIFS / COUNTIFS / AVERAGEIFS / MINIFS / MAXIFS: every range must have the first range's shape
Rule: if any criteria (or sum) range is shaped differently from the first range, SUMIFS/COUNTIFS
return `#VALUE!`, even when the cell counts are equal (2x4 vs 4x2). `=SUMIFS(Data!B7:C9,Data!A10:D12,"<5")`
→ `#VALUE!`; all 145 mismatched top-level calls in gen-m2 are `#VALUE!`, all 26 same-shape calls are numbers.
Evidence: gen-m2/04cfd82f54dbc2b2 Calc!A150 (3x2 vs 3x4), 1b6ceed7489e0800 Calc!A372 (8x1 sum_range vs
1x2 criteria range), 207a5753da48c7e0 Calc!A327 (COUNTIFS 5x3 vs 2x4), 7469c3e920e1ade4 Calc!A51 (COUNTIFS
2x4 vs 4x2, same size). Oracle: excel-graph, 2026-10-04 (gen-m2 sidecars, T-0203).
AVERAGEIFS, MINIFS and MAXIFS follow the same rule (`=MINIFS(A1:B2,A1:D1,"<9")` and
`=AVERAGEIFS(A1:D2,A1:B4,"<99")` are `#VALUE!`; probes/ifs-shape-mismatch F1–F8, excel-graph 2026-10-05).
SUMIF and AVERAGEIF never error: they resize the sum/average range (probes/ifs-shape-mismatch F9–F11,
`=AVERAGEIF(A1:D1,"<99",A1)` = 2.5; T-0214).
Status: VERIFIED
Implemented in: `fn_countifs` and `apply_ifs` (`strict_shape`) in `crates/rustsheet-engine/src/functions/statistical/if_ifs.rs`
Tasks: T-0209

## SUMIF resizes sum_range from its top-left cell and never errors
Rule: SUMIF sums the range of the criteria range's shape that starts at sum_range's top-left cell,
whatever sum_range's own shape: `=SUMIF(Data!C2:E2,"<>1",Data!B4:B10)` sums B4:D4 (123456831.123).
71 of 71 mismatched calls in gen-m2 return numbers. AVERAGEIF resizes its average_range the same way.
Evidence: gen-m2/629e6e8b76fab8b9 Calc!A217, 04cfd82f54dbc2b2 Calc!A344 (`=SUMIF(Data!C3:C5,"?b?",Data!C10:C10)`
= 0.5, C10:C12 is used), f0d1bacec5de8146 Calc!A127; probes/ifs-shape-mismatch F9-F11. Oracle: excel-graph, 2026-10-04 (gen-m2 sidecars, T-0203).
Status: VERIFIED
Implemented in: `resize_to` in `crates/rustsheet-engine/src/functions/statistical/if_ifs.rs`
Tasks: T-0214

### SUMIF/AVERAGEIF resize clamps at the sheet edge; single-cell ranges are 1x1
Rule: when the resized sum_range would run past XFD1048576 Excel clamps it to the sheet edge (no `#REF!`):
`SUMIF(A1:B2,">0",XFD1)` sums XFD1:XFD2. After clamping, AVERAGEIF averages only the cells inside the clamped range (`AVERAGEIF(A1:B2,">0",XFD1048576)` = 30/1, not 30/4).
A single-cell range is a 1x1 range (probed for SUMIF, SUMIFS, MINIFS, MAXIFS, COUNTIF; AVERAGEIF/COUNTIFS/MAXIFS share the code paths but are not probed): as the first range of
SUMIFS/MINIFS/MAXIFS (`SUMIFS(D1,A1:A1,">0")`) and as a criteria range (`=SUMIF(A1,">0",D1)`,
`SUMIFS(D1:D1,A1,">0")`, `COUNTIF(A1,">0")`, `SUMIF(A1,{">0"},D1)`), none of them `#VALUE!`.
Evidence: probes/sumif-edge-single-cell F1-F15 (oracle: excel-graph)
Status: VERIFIED
Implemented in: `resize_to`, `apply_ifs`, `fn_countifs` in `crates/rustsheet-engine/src/functions/statistical/if_ifs.rs`
Tasks: T-0234

### Conditional-format Equal/NotEqual compare on 15 digits (no absolute epsilon)
Rule: the engine's conditional-format "cell value equal to / not equal to" rules compare with the same 15-significant-digit rule as `=` and criteria equality (`cmp_excel_precision`), not an absolute `f64::EPSILON` tolerance, so tiny values never equal 0. NaN orders above every number and equals itself in `CalcResult`'s `Ord`; -0.0 equals 0.0.
Tests: `test_equal_and_not_equal_use_15_digit_comparison`, `cmp_excel_precision_total_order_on_nan_and_signed_zero`; callers of `cmp_excel_precision` (e.g. `functions/criteria.rs`) now see NaN == NaN. Evidence: none direct. The Graph oracle cannot evaluate conditional formats, so the Excel behavior for tiny values is inferred from the 15-digit comparison rule.
Status: UNVERIFIED (probe pending: T-1051)
Implemented in: `ValueOperator::Equal`/`NotEqual` in `crates/rustsheet-engine/src/conditional_formatting.rs`, `cmp_excel_precision` in `number_format.rs`
Tasks: T-1033

Exception: an error-valued criterion that is a stored `#BUSY!` (e.g. `COUNTIF(A1:A2,A1)`) matches nothing, not even `#BUSY!` cells, unlike a `#N/A` criterion (probes/error-stored-busy-behaviors C15, T-1314). COUNTIFS, SUMIF, SUMIFS, AVERAGEIF and DCOUNTA agree (probes/aggregate-error-precedence C79-C92, T-1334): a stored `#BUSY!` criterion matches nothing (`COUNTIFS(A1:A2,A1)` 0, `COUNTIFS(A1:A2,"<>"&A1)` 0 (the concatenation passes the error on, so it is the same criterion as C82), `DCOUNTA(...,J1:J2)` 0) while a stored `#N/A` criterion matches the `#N/A` cell (`COUNTIFS(A1:A2,A2)` 1, `DCOUNTA(...,K1:K2)` 1); `AVERAGEIF` with nothing matched is `#DIV/0!` (C86). Limitation: the SUMIF/SUMIFS cells (C79-C81, C87, C88) have a text or empty sum cell aligned with the error rows, so they give 0 either way and only show that no error propagates; the COUNTIFS and DCOUNTA cells are the real evidence.

### COUNTIF text criteria use the collation for equality (T-1590)
Rule: a text criterion `"ae"`, `"=ae"` or `"<>ae"` compares with Excel's text collation, the same one as `<`/`>`: over {"æ","ae","a-e"}
`COUNTIF(r,"ae")` and `COUNTIF(r,"=ae")` are 2 (æ equals ae), `COUNTIF(r,"<>ae")` is 1; a criterion `"æ"` also counts 2; `"a-e"` counts 1
(a hyphen is not ignored in equality). The ordered operators agree (`"<=ae"` 2, `">=ae"` 3, `">ae"` 1, `"<ae"` 0).
Evidence: probes/text-collation-lookups Calc!F1:H2, K1:K12 (oracle: excel-graph, 2026-10-15). Status: VERIFIED
Implemented in: `result_is_equal_to_string`, `result_is_not_equal_to_string` (functions/criteria.rs), through `cmp_text_excel` (T-1594). The same code serves SUMIF/AVERAGEIF/*IFS and the D-functions' explicit `=`/`<>` criteria; SUMIF, SUMIFS and COUNTIFS are probed too (T-1597); AVERAGEIF(S), MAXIFS, MINIFS and the D-functions are not.
Tasks: T-1590, T-1594, T-1597

SUMIF, SUMIFS and COUNTIFS agree with COUNTIF cell for cell (T-1597): over {æ, ae, a-e, ß, ss, straße, aex, æx}, `"ae"`, `"=ae"` and `"æ"` match æ and ae (2 rows), `"<>ae"` the other 6; `"ss"` matches ß and ss (2 rows, not straße) and `"<>ß"` excludes both (6 rows). Evidence: probes/text-collation-ifs Calc!A1:D6 (excel-graph, 2026-10-15). Status: VERIFIED

### Wildcard criteria do not use the collation: no expansions (T-1597)
Rule: `?` and `*` in a text criterion match by character (case-insensitive per "Wildcard text matching" above; this case is lowercase only), with no ae/æ or ss/ß expansion, unlike the equality operators above. Over {æ, ae, a-e, ß, ss, straße, aex, æx}: `"a?"` matches only ae (not æ), `"a*"` matches ae, a-e and aex (not æ), `"?"` matches æ and ß (one character each, not ss), `"??"` matches ae, ss and æx, `"*ss*"` matches only ss (not ß or straße), `"s?"` matches only ss, `"æ*"` matches æ and æx (not aex), `"*e"` matches ae, a-e and straße. COUNTIF, SUMIF, SUMIFS and COUNTIFS agree.
Evidence: probes/text-collation-ifs Calc!A7:D14 (excel-graph, 2026-10-15). Status: VERIFIED
Implemented in: `wildcard_text_matcher` (functions/criteria.rs); the engine already passes all 56 cells of the case.
Tasks: T-1597
