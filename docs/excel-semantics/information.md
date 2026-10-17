# Information functions (IS*)

## IS* type tests lift over arrays and ranges
Rule: ISNUMBER, ISTEXT, ISNONTEXT, ISLOGICAL, ISBLANK, ISERROR, ISERR and ISNA given a
multi-cell range or an array return a boolean array of the same shape and spill. A single value
gives one boolean. Elements are tested as stored: an empty cell is ISBLANK and ISNONTEXT, an
error element is ISERROR (ISERR excludes `#N/A`), and an error produced inside the array
expression (`{1,"a",TRUE}*1`) is an error element.
Evidence: probes/is-functions-array-lift (excel-graph) `Calc!A1:H6`, `A9:C9`, `A10:C10`, 3x2
`ISBLANK(Data!A1:B3)` in `A11:B13`; probes/text-number-words F12:H12.
Status: VERIFIED (excel-graph)
Tasks: T-0247

## ISEVEN/ISODD over a range
Rule: `ISEVEN(range)` / `ISODD(range)` with a multi-cell range argument return a single `#VALUE!`
with no spill, over a numbers-only range (2, 3, 4.5, -7, 0) as well as a mixed one (1, "x",
TRUE, #N/A, empty, #DIV/0!). There is no implicit intersection either: the formula placed on a
row inside the range (`=ISEVEN(Data!C1:C5)` at row 2) is still `#VALUE!`, as is a 2-D range
(`Data!C1:D3`). They do not lift over ranges like the other IS* functions. An array constant does
lift: `=ISEVEN({2,3})` spills {TRUE, FALSE}, `=ISODD({2,3})` spills {FALSE, TRUE}. A single-cell
reference is an ordinary value.
Evidence: probes/is-functions-array-lift (Calc!A15, B15, D15, E15, J2, K2, L3, M30:N30, M32:N32;
excel-graph oracle sidecar, 2026-10-06). Desktop Excel was not checked.
Also covered by M34:P34 (`{4.5,-7,"3",TRUE}` -> TRUE, FALSE, FALSE, #VALUE!: parity of the truncated
absolute value, text cast, boolean #VALUE!). UNVERIFIED sub-rules (no probe yet): an array
expression derived from a range (`ISEVEN(Data!C1:C5+0)`), arguments of magnitude >= 2^63, and the
dynamic-array flag Excel writes for the range form.
Implemented in: `Model::parity_predicate` (functions/information/mod.rs).
Status: VERIFIED (excel-graph)
Tasks: T-0247, T-0248

## IS* and TYPE over every scalar value kind
Only a truly empty cell is blank: `ISBLANK` is FALSE for a formula returning `""`, a single space,
`0`, FALSE, text and errors, and TRUE for an unused cell (`Data!Z99`) and for a blank passed
through `IF(1,Data!A1)`. `ISTEXT` is TRUE for `""` (literal and formula result), `1&""` (text "1"), a space
and the text `"TRUE"`/`"1"`; FALSE for blanks, numbers (date-formatted too), booleans and errors.
`ISNONTEXT` is the exact complement (TRUE for blank and errors). `ISLOGICAL` is TRUE only for
real booleans (`1=1`, a FALSE cell), not `"FALSE"`/`"TRUE"` text. `ISNA` is TRUE only for `#N/A`
(`NA()`, `#N/A`, `NA()+1`), FALSE for `#DIV/0!`/`#VALUE!`. `TYPE`: blank 1, number 1,
text (including `""`) 2, boolean 4, error 16, array/range 64 (evidenced by probes/info-logic-argument-modes, not this probe), and `TYPE(LAMBDA(x,x))` is 128
(the engine returned `#N/IMPL!`; fixed in T-1278).
Evidence: probes/is-functions-scalar-kinds (Calc!A1:F12 direct references, A20:F24 direct
arguments; excel-graph oracle sidecar). Desktop Excel was not checked.
Implemented in: `fn_type`, `fn_isblank` and the IS* helpers (functions/information/mod.rs).
Status: VERIFIED (excel-graph)
Tasks: T-1278

## Resource bound on lifted IS* ranges
Rule: Excel spills `ISBLANK(A:A)` (1,048,576 rows) and even `ISBLANK(A:XFD)`. RustSheet returns
`#ERROR!` for a lifted IS* range above 1,048,576 cells (one whole column), to bound allocation on
untrusted input (CLAUDE.md directive 5); a whole-column argument lifts (`SUM(--ISNUMBER(A:A))` = 2,
probes/lifted-range-size C8), a whole-row range of more than 64 rows does not.
Evidence: probes/lifted-range-size C8; resource limit, see arrays.md "Array results above 1,048,576 cells".
Status: DEVIATION (decision 0014)
Implemented in: functions/information/mod.rs (`is_predicate`, via `Model::evaluate_range`)
Tasks: T-0247, T-0047

## IS*, TYPE, IFNA and COUNTIF on the newer rich errors (`#BLOCKED!`, `#CONNECT!`, `#FIELD!`)
Rule: an error cell from IMAGE (`#BLOCKED!`), STOCKHISTORY (`#CONNECT!`) or FIELDVALUE (`#FIELD!`)
behaves the same way for every test: ISERROR and ISERR are TRUE, ISNA is FALSE, TYPE is 16,
`IFNA(cell,"y")` returns the error unchanged (it is not `#N/A`), and `ERROR.TYPE` is 11 for
`#BLOCKED!`, 10 for `#CONNECT!` and 13 for `#FIELD!`; `ERROR.TYPE(cell+#DIV/0!)` is the same code, so the
first error operand propagates (the rich kind wins over the later `#DIV/0!`). `COUNTIF(range,"#BLOCKED!")`,
`"#CONNECT!"` and `"#FIELD!"` all count 0, even over the cell holding that error, and the negated
`COUNTIF(range,"<>#BLOCKED!")` (likewise `<>#CONNECT!`, `<>#FIELD!`) counts that cell (1), so the error is
never equal to the criterion text: the criteria
table stays at today's twelve names (SPEC-0003 criteria bullet; T-1013 changes nothing there).
Evidence: probes/rich-errors-args A40:C41 and B42:B48 (IMAGE), A50:C51, B52:B58 (STOCKHISTORY), A60:C61, B62:B68 (FIELDVALUE);
excel-graph, 2026-10-07 (T-1012).
Status: VERIFIED (excel-graph)
Tasks: T-1012

## IMAGE/STOCKHISTORY/FIELDVALUE argument errors
Rule: for IMAGE and STOCKHISTORY an error argument wins: `IMAGE(1/0)`, `IMAGE("x",1/0)`,
`STOCKHISTORY(1/0,DATE(2020,1,2))` and `STOCKHISTORY("QZXQZXQ",1/0)` are `#DIV/0!`. FIELDVALUE
does not follow that rule for its first argument: `FIELDVALUE(1/0,"Price")` is `#FIELD!`, while
an error in `field_name` wins (`FIELDVALUE(A10,1/0)` is `#DIV/0!`). `IMAGE(A1)` of an empty cell
and `IMAGE(A10)` of a number evaluate as `#VALUE!` (ERROR.TYPE 3, TYPE 16, `VALUETOTEXT` "#VALUE!"), not
`#BLOCKED!`. Excel stores those two cells as rich values (the oracle cannot decode their errorType), so
their stored form is UNVERIFIED. `FIELDVALUE({1,2},"Price")` is an array result (`#FIELD!` in both cells, the second
spilling into B28), not a scalar.
Evidence: probes/rich-errors-args A20:A28, B28 and C22:G23; excel-graph, 2026-10-07 (T-1012).
The raw `IMAGE(A1)`/`IMAGE(A10)` cells could not be decoded by the oracle (rich-value error type
unknown), so they are probed through ERROR.TYPE/ISERROR/VALUETOTEXT/TYPE wrappers.
Status: VERIFIED (excel-graph) for values; stored form of IMAGE(empty/number) UNVERIFIED
Tasks: T-1012

## IMAGE and STOCKHISTORY never fetch
Rule: `IMAGE(source, ...)` evaluates to `#BLOCKED!` and `STOCKHISTORY(stock, start_date, ...)` to
`#CONNECT!` for the probed arguments: a URL source (`https://example.invalid/none.png`) or a ticker, after a single argument error and
IMAGE's empty/number `#VALUE!` (see above; more argument kinds below).
Argument kinds (T-1141, probes/rich-errors-arg-kinds, excel-graph, 2026-10-08) VERIFIED for the probed arguments only: IMAGE `source`,
STOCKHISTORY `stock`, FIELDVALUE `value` (array and range) and FIELDVALUE `field_name` (array) lift over an array or range
argument as an ordinary scalar-argument function would. UNVERIFIED: IMAGE's `alt_text`/`sizing`/`height`/`width`, STOCKHISTORY's
other arguments, a range `field_name`, and a one-cell range for IMAGE or STOCKHISTORY. `ROWS`/`COLUMNS` of `IMAGE({"a","b"})` are 1 and 2 and of
`IMAGE(Q1:Q2)` (Q1="a", Q2="b") 2 and 1; `STOCKHISTORY({"A","B"},DATE(2020,1,2))` is 1x2 and `STOCKHISTORY(Q1:Q2,...)` 2x1, each
element `#CONNECT!`; `FIELDVALUE(5,{"a","b"})` is 1x2 and `FIELDVALUE(Q1:R2,"Price")` 2x2 of `#FIELD!` (ERROR.TYPE 13), and
`FIELDVALUE({1},"Price")` is 1x1. A one-cell range `value` of FIELDVALUE is a scalar: `ROWS(FIELDVALUE(Q1:Q1,"Price"))` is `#FIELD!`. A number
or boolean `stock` is `#CONNECT!` (ERROR.TYPE 10; an empty-cell `stock` is UNVERIFIED, the engine gives `#CONNECT!`); text, boolean and empty-cell `value` of FIELDVALUE are `#FIELD!`.
`IMAGE("")`, `IMAGE(TRUE)` and `IMAGE` of non-URL text (`"a"`, `"b"`) are `#VALUE!` (ERROR.TYPE 3), not `#BLOCKED!`; only a
URL-shaped source reaches the fetch (which strings count as a URL: the T-1189 bullets below). `IMAGE(1/0,NA())` is `#DIV/0!` (a `source`
error wins over an `alt_text` error; other orders are not probed). The raw IMAGE cells (A10, B10, A13, A14, A16, A19) and the `ROWS`/`COLUMNS` of `IMAGE("")` and `IMAGE(TRUE)` came back with an undecodable rich error type and stay pending; the wrapped `ERROR.TYPE` cells carry the evidence. The VERIFIED rules above cover argument kinds; the DEVIATION status below covers only the missing fetch.
The engine lifts over array and range `source`/`stock`/`value`/`field_name` and gives `#VALUE!` for non-URL IMAGE sources (the URL rule is in the T-1189 bullets below); `ERROR.TYPE` lifts over arrays too (probes/rich-errors-arg-kinds F10:G10, F13:G14, F40:G41; over a direct multi-cell range it follows the IS* lifting evidence). Shapes, error elements and URLs (probes/rich-errors-shapes-urls, excel-graph, 2026-10-11, T-1189):
- FIELDVALUE with two array arguments of different shapes broadcasts to the larger shape, all `#FIELD!` (no `#N/A`
  padding): `{1,2}` with `{"a","b","c"}` is 1x3, `{1;2}` with `{"a","b","c"}` 2x3, `{1,2,3}` with `{"a";"b"}` 2x3 (A2:C7). A `field_name`
  shorter than the result along an axis wider than 1 pads with `#N/A` (below, T-1345).
- An error element of an array `field_name` stays itself: `FIELDVALUE(5,{"a",#DIV/0!,#N/A})` is `#FIELD!`, `#DIV/0!`,
  `#N/A`. An error element of an array `value` is `#FIELD!` like a scalar error `value` (A9:C10).
- A later argument's error lifts over an array first argument: `IMAGE({url,url},1/0)` and `STOCKHISTORY({"A","B"},1/0)`
  are 1x2 arrays of `#DIV/0!`, and the column forms have 2 rows (A12:B15).
- An error element inside an array `source`/`stock` keeps its error; the other elements are `#BLOCKED!` / `#CONNECT!` (A18:C19).
- `ERROR.TYPE` over a direct multi-cell range lifts (`ERROR.TYPE(E1:G1)` spills 2, 7, 2), and over a one-cell range it is
  that cell's code, one row (A21:A23).
- An IMAGE source is a URL (`#BLOCKED!`) when, after leading spaces, tabs and no-break spaces, it starts with `http:` or `https:` in any case and has
  something after the scheme other than slashes: `HTTPS://EXAMPLE.INVALID/A.PNG`, `https:example.invalid/a.png`,
  ` https://example.invalid/a.png` and `https://example.invalid` are `#BLOCKED!`; `https://`, `ftp://...`, `www....`,
  `data:image/png;base64,...`, `file:///c:/a.png` and `example.invalid/a.png` are `#VALUE!` (A25:A35). Also `#VALUE!`: bare
  `http:`, `https:` and `https:/`. Also `#BLOCKED!`: `https:///x`, `http://x`, trailing whitespace, and a leading tab or no-break space
  (probes/rich-errors-image-sizing, excel-graph, 2026-10-11, T-1345).
- IMAGE option validation (probes/rich-errors-image-sizing, T-1345; every case wrapped in ERROR.TYPE, 11 = `#BLOCKED!`, 3 = `#VALUE!`):
  `sizing` 0, 1, 2, 1.5 (truncated) and TRUE reach the fetch; -1, 4, 9, text, `""` and an omitted `IMAGE(url,"alt",)` are
  `#VALUE!`. `sizing` 3 alone, or `IMAGE(url,,3)`, is `#VALUE!`; with a `height` (`3,10`), or `3,0,10`, `3,10,0` and `3,1.5,10.5`, it reaches the
  fetch; `3,0,0`, a negative `height` or `width` and a text `height` are `#VALUE!`. `sizing` 0 or 1 with a `height` (`0,10`, `1,10,10`) is
  `#VALUE!`. UNVERIFIED: a `width` without a `height` (`0,,10`, `3,,10`; the engine is `#VALUE!` for both), a negative `height` under `sizing` 2, `sizing` 2 with a `height`, `3,0` (height 0, no width), a fractional `sizing` near the limits (-0.5, 3.5), an
  empty cell or numeric text as `sizing`/`height`.
- IMAGE with an array `sizing`, `height` or `width` lifts: the source and those options broadcast against each other and each
  element is checked on its own (`IMAGE({url,url},"alt",{0,9})` is `#BLOCKED!`, `#VALUE!`; `{9;0}` with a 1x2 source is 2x2;
  a scalar source with `{0;9;1}` is 3x1; a length-1 array repeats; a shorter array is `#N/A` past its end,
  `IMAGE({url,url,url},"alt",{0,9})` is `#BLOCKED!`, `#VALUE!`, `#N/A`). A `sizing` of `{3,3}` with `height` `{5,-1}` is valid, then
  invalid, and so on per element (probes/rich-errors-image-array-options, 61 cells, excel-graph, 2026-10-12, T-1370).
  An error element inside an option array is that error itself, per element, not `#VALUE!`: `IMAGE(url,"alt",{0,#DIV/0!})` is
  `#BLOCKED!`, `#DIV/0!`; `{#DIV/0!,0}` is `#DIV/0!`, `#BLOCKED!`; `{9,#DIV/0!}` is `#VALUE!`, `#DIV/0!`. With several options the first
  error in argument order (`sizing`, `height`, `width`) wins for that element (`{#DIV/0!,0}` with `{5,#N/A}` is `#DIV/0!`, `#N/A`;
  `{0,#N/A}` with `{5,5}` is `#VALUE!` for the invalid 0-with-height, then `#N/A`), and it holds for a scalar or array source. Two errors in one element: the earlier option wins (`{#DIV/0!,0}` with `{#N/A,5}`
  is `#DIV/0!` first; `sizing` `{#N/A,..}`, `height` `{#DIV/0!,..}`, `width` `{#NUM!,..}` is `#N/A`). An error element beats an invalid option
  (`{9,0}` with height `{#DIV/0!,5}` is `#DIV/0!`, `#VALUE!`), and a `source` error element beats an option error element
  (`IMAGE({#N/A,url},"alt",{#DIV/0!,0})` is `#N/A`, `#BLOCKED!`; `IMAGE({url,#N/A},"alt",{0,#DIV/0!})` is `#BLOCKED!`, `#N/A`)
  (probes/rich-errors-image-array-error-elements, 41 cells, excel-graph, 2026-10-12, T-1417; VERIFIED).
  A scalar error option beside an array option also lifts: `IMAGE(url,"alt",#DIV/0!,{5,5})` and `IMAGE(url,"alt",{0,0},#DIV/0!)` are
  1x2 of `#DIV/0!`, not one scalar error (`ERROR.TYPE` 2, 2), and `IMAGE({url,url},"alt",3,#N/A,{5,5})` is `#N/A`, `#N/A`. An option error
  element beats a non-URL `source` element in the same column: `IMAGE({"x",url},"alt",{#DIV/0!,0})` is `#DIV/0!`, `#BLOCKED!`;
  `{1,url}` with `{#N/A,0}` is `#N/A`, `#BLOCKED!`; `{"",url}` with `{#DIV/0!,9}` is `#DIV/0!`, `#VALUE!`. Without an option error the
  non-URL element is `#VALUE!` (`{TRUE,url}` with `{0,#N/A}` is `#VALUE!`, `#N/A`) (T-1427; VERIFIED).
  An error in `alt_text` lifts the same way: `IMAGE(url,#N/A,{0,0})` is `#N/A`, `#N/A`, and a scalar error option beside an array `alt_text`
  gives the array's shape (`IMAGE(url,{"a","b"},#DIV/0!)` is `#DIV/0!`, `#DIV/0!`; `IMAGE(url,{"a","b"},3,#N/A)` is `#N/A`, `#N/A`). With
  several errors the first in argument order wins: `IMAGE(url,"alt",#N/A,{#DIV/0!,0})`, `IMAGE(url,"alt",#N/A,{5,5},#DIV/0!)` and
  `IMAGE(url,#N/A,{0,0},#DIV/0!)` are all `#N/A`, and `IMAGE(url,{#N/A,"b"},#DIV/0!)` is `#N/A`, `#DIV/0!` element by element
  (probes/rich-errors-image-array-error-elements, T-1427; VERIFIED).
  An array `alt_text` lifts and shapes the grid like any other IMAGE argument: `IMAGE(url,{"a","b"})` and `IMAGE(url,{"a","b"},0)` are
  1x2 of `#BLOCKED!`, `IMAGE(url,{"a";"b"},{0,0})` is 2x2, `{"a","b","c"}` with `{0,0}` is `#BLOCKED!`, `#BLOCKED!`, `#N/A` (and `{"a","b"}`
  with `{0,0,0}` the same), and an error element in `alt_text` propagates per element (`{#N/A,"b"}` with `{0,0}` is `#N/A`, `#BLOCKED!`;
  `{"b",#N/A}` with `{0,0,0}` is `#BLOCKED!`, `#N/A`, `#N/A`) (95 cells, T-1427; VERIFIED).
  A non-URL scalar source keeps the lift: `IMAGE("x",{"a","b"})` is `#VALUE!`, `#VALUE!` (1x2, not a scalar), and a scalar error option beside an array option
  still shapes the grid and wins over the source: `IMAGE(1,"alt",#DIV/0!,{5,5})` is `#DIV/0!`, `#DIV/0!` (probes/rich-errors-image-array-error-elements A189, A193, T-1432; VERIFIED; the engine already matches).
- IMAGE with an invalid `sizing` (`IMAGE({url,url},"alt",9)`) is `#VALUE!` per element (A16).
- A `field_name` array shorter than the `FIELDVALUE` result along an axis wider than 1 is `#N/A` past its end:
  `FIELDVALUE({1,2,3},{"a","b"})` is `#FIELD!`, `#FIELD!`, `#N/A`; `{1,2,3}` with `{"a";"b"}` is 2x3 of `#FIELD!`, and `{1;2;3}` with `{"a","b"}` is 3x2 of `#FIELD!`; the row-axis form (`{1;2;3}` with `{"a";"b"}`) is UNVERIFIED (probes/rich-errors-image-sizing). A
  shorter `value` is not padded (above).
The engine matches all of these (A16 since T-1345; the option lift since T-1370; `Model::image_grid`).
Excel fetches the image or the quotes; formulas here do no I/O (CLAUDE.md directive 5).
Excel returns the same errors only when the fetch fails: `IMAGE("https://example.invalid/none.png")` is
`#BLOCKED!` and `STOCKHISTORY("QZXQZXQ",DATE(2020,1,2))` is `#CONNECT!`. A reachable URL or a real ticker
is the deviation. Which of the two errors Excel gives for other failures is unknown beyond these two cells.
Evidence: probes/rich-errors-image A1, probes/rich-errors-stockhistory A1, probes/rich-errors-args A40, A50,
probes/rich-errors-arg-kinds (excel-graph); the engine's results are unit-tested in `test/test_fn_offline_errors.rs`.
Status: DEVIATION (decision 0014)
Implemented in: functions/information/offline_errors.rs
Unverified: the localized (de/es/fr/it) names of IMAGE, STOCKHISTORY and FIELDVALUE; the language
tables use the English names.
Tasks: T-1013, T-1141, T-1179, T-1189, T-1345, T-1370

## FIELDVALUE of a plain value is `#FIELD!`
Rule: `FIELDVALUE(value, field_name)` is `#FIELD!` for a number `value` and for an error `value` (the
error does not win), and `FIELDVALUE({1,2},"Price")` spills a 1x2 array of `#FIELD!` (argument-error
precedence is in "IMAGE/STOCKHISTORY/FIELDVALUE argument errors" above). No value here is a linked data type
(decision 0027).
Text, boolean and empty-cell values are `#FIELD!` too, and a multi-cell range or array lifts (T-1141, probes/rich-errors-arg-kinds:
VERIFIED, see "IMAGE and STOCKHISTORY never fetch").
Evidence: probes/rich-errors-field A1, probes/rich-errors-args A60:A61, A20:A28 (excel-graph).
Status: VERIFIED (excel-graph)
Implemented in: functions/information/offline_errors.rs
Tasks: T-1013

## Information function argument modes (range, array, literal)
Rule: how the other M5 information functions take a range, an array and a direct value (T-1088):
- ERROR.TYPE and ISFORMULA lift over a multi-cell range: `ERROR.TYPE(E1:F1)` over `#DIV/0!`, `#N/A` spills 2, 7, and
  `ISFORMULA(A3:B3)` over a formula and a constant spills TRUE, FALSE. `ERROR.TYPE(1)` is `#N/A`.
- TYPE takes a range or an array whole: `TYPE(A2:C2)` and `TYPE({1,2})` are 64, with no spill. `TYPE` of one number cell is 1. In a legacy
  formula TYPE's range is implicitly intersected instead: 1 on its rows, 16 off them (references.md "Legacy
  formulas intersect the ranges in value arguments", probes/legacy-type-intersection, T-1526). An array constant is not intersected: `=TYPE({5})` is 64 with no `@`
  (references.md "Legacy `@` on CHOOSE operator arms", T-1638).
- N gives the top-left cell of a range, with no lift and no implicit intersection: `N(A2:C2)` is 2 (A2) and
  `N(B2:C2)` from a row and column outside the range is 3 (B2). `N("5")` and `N` of a text cell are 0, and `N(TRUE)` is 1.
- ISREF takes any value: a range is TRUE (no spill), while `ISREF(1)` and `ISREF(A1:A2+0)` are FALSE.
- SHEET: a range gives its sheet's number (no spill), `SHEET("Data")` that sheet's number, `SHEET()` the formula's
  sheet, and `SHEET(1)` and `SHEET("Nope")` are `#N/A`.
- CELL: `CELL("row",B2)` is 2, and info_type lifts over an array (`CELL({"row","col"},B2)` spills 2, 2). An unknown
  info_type (`"bogus"`) is `#VALUE!`, and so is `CELL("contents",A2:C2)` (a multi-cell range). `CELL("address",ref)` for a
  reference on another sheet names the workbook file: `[<file>.xlsx]Data!$B$2`.
- A reference argument must be a reference. Excel for the web refuses a workbook (`FileCorruptTryRepair`) holding
  `=ISFORMULA(1)`, `=ISFORMULA("A1")`, `=CELL("row",1)` or `=CELL("row",{1,2})`, one scratch workbook per formula.
  The generator therefore passes those parameters a cell reference only (`cell: true` in eval/catalog/functions.yaml).
Evidence: probes/info-logic-argument-modes A6:A29, info-logic-argument-modes-2 A7:A8, info-modes-sheet-text,
info-modes-isref-value-op (oracle: excel-graph, 2026-10-09, T-1088). The refusals come from scratch fills that failed and were not kept.
Status: VERIFIED (excel-graph)
Ours: matches since T-1218 (SHEET/N of a range, CELL's array info_type, ISFORMULA's lift, `CELL("contents")` superseded by T-1230 below).
Settled by T-1230 (next entry): ironcalc-calc CELL.xlsx B14 (top-left of a same-sheet range) and probes/info-logic-argument-modes A18 (`#VALUE!`, which is the formula's own-sheet cell Calc!A2) both fit "the top-left cell, read on the formula's own sheet".
`CELL("address")` across sheets depends on the file name, which the engine does not know (T-1218 leaves it unimplemented).
Tasks: T-1088, T-1218

## ISFORMULA over reference kinds (excel-graph)
Rule: ISFORMULA is TRUE for a cell holding a formula, whatever it returns (`=1+1`, `=""`, `=1/0`), and FALSE for a constant number, a constant text and a blank cell. A spill anchor (`=SEQUENCE(3)`) is TRUE and a spilled child (the cell below it) is FALSE. A cell on another sheet works the same, and the reference may come from INDIRECT, OFFSET or INDEX; `ISFORMULA(Data!D1:D3)` lifts.
Evidence: probes/ifna-isformula-edges Calc!B1:B13 (oracle: excel-graph, 2026-10-12, T-1279).
Status: VERIFIED (excel-graph)
Ours: matches except `ISFORMULA(INDEX(range,1))`, which Excel gives as TRUE and we give as #VALUE! (Calc!B12; T-1453).
Tasks: T-1279, T-1453

## CELL("contents", ref) reads the formula's own sheet (excel-graph)
Rule (excel-graph): `CELL("contents", ref)` reads the top-left cell of `ref` taken **on the formula's own sheet**, whatever sheet `ref` names; a multi-cell range is never implicitly intersected, and a legacy (non-array) formula gives the same value as a dynamic one. `=CELL("contents",Data!A2)` on Calc is Calc!A2 (12.5), not Data!A2 (2.5); `Data!A7:C7` is Calc!A7 (21.5); `Data!E1:F1` is Calc!E1 (blank, 0). The earlier "other sheet = `#VALUE!`" reading (probes/info-logic-argument-modes A18) was this: Calc!A2 there is `=XOR(Data!B1)` = `#VALUE!`. Same-sheet ranges give the top-left cell (`A1:A5` in C1, C3, C8 all 11.5), legacy twins included. Legacy-twin evidence for the other info_types is in the next entry (T-1433).
Evidence: probes/info-cell-contents-range Calc!C1:F8, S2:S7, T1:T6, V1:V5 (oracle: excel-graph, 2026-10-12, T-1230).
Status: VERIFIED (excel-graph); desktop UNVERIFIED. A desktop Excel that honours the sheet is likely (this looks like a web bug); the desktop check is T-1075 item 9 (decision 0039: excel-graph is the oracle of record until then).
Ours: matches since T-1230 (`cell_info` reads the top-left cell on `cell.sheet`; CELL's `reference` argument has a vector signature, so a legacy formula does not intersect it).
Also: `CELL({1,"row"},B2)` is `#VALUE!` (a non-string element) and `CELL({"row",1},B2)` spills 2, `#VALUE!`; `N(range)` with an error top-left cell is that error (Calc!J1, K1); the info_type array lifts per element.
Tasks: T-1230

## CELL("row"/"col"/"address"/"type", range): no intersection; "type" reads the formula's own sheet
Rule (excel-graph, 2026-10-15): a legacy (non-array) `CELL(info_type, range)` gives the same value as its dynamic twin and is never implicitly intersected, whether the formula row or column is inside the range or not. `CELL("row",A1:A5)` is 1 and `CELL("address",A1:A5)` is `$A$1` in rows 3 and 8; `CELL("col",B20:D20)` is 2 in columns C, D (inside) and F, G (outside); `CELL("type",A1:A5)` is "v" (the top-left cell; the probe cannot tell it from an intersected cell, since both are numbers, so no-intersection for "type" rests on the vector signature, UNVERIFIED). `row`, `col` and `address` use the reference's own address (`CELL("row",Data!A2:A5)` is 2, `CELL("col",Data!C20:D20)` is 3). `"type"` reads the top-left cell at the same address **on the formula's own sheet**, like `"contents"`: `CELL("type",Data!A7)` is "b" when Data!A7 is text but Calc!A7 is blank, `Data!A8` is "l" (Calc!A8 is text, Data!A8 blank) and `Data!A9` is "b" (Data!A9 a number, Calc!A9 blank).
Cross-sheet `CELL("address",Data!A2)` was observed as `[<file>.xlsx]Data!$A$2` with the oracle's temporary file name in the T-1433 oracle run; it cannot be scored (not in the case, no stored evidence).
Evidence: probes/info-cell-legacy-range (40 cells; oracle: excel-graph, 2026-10-15, T-1433).
Status: VERIFIED (excel-graph); desktop UNVERIFIED (same caveat as "contents", T-1075 item 9).
Ours: matches (`cell_info` "TYPE" reads `cell.sheet` since T-1433). Cross-sheet "address" is `#N/IMPL!` (no case, file-name dependent).
Tasks: T-1230, T-1433

## Oracle refusal: XLM GET.WORKBOOK (precaution, not Excel evidence)
The oracle appends a canary sheet to every workbook it recalculates, which changes `SHEETS()`, `INFO()`
and the XLM `GET.WORKBOOK(...)` sheet list. Workbooks that use `GET.WORKBOOK(` (defined names or macro
sheets) are refused by `embed_canary` for that reason only; this says nothing about what Excel returns
(T-0014, tools/src/rustsheet_tools/oracle/canary.py).

## SHEET of a reference, a name in text and a non-reference value
Rule (probes/volatile-and-info-rand-true-false-sheet D1-D28, excel-graph, 2026-10-15, T-1627; sheets S, T, U in order):
`SHEET()` is the formula's own sheet (1) and `SHEET(ref)` the reference's sheet, for a cell or a range
(`SHEET(T!A1:B2)` 2, `SHEET(A1:B2)` 1); `SHEET("T")` and `SHEET("t")` are the sheet of that name (2, case-insensitive),
`SHEET("Nope")`, `SHEET("")`, `SHEET(" T")` and `SHEET("S!A1")` are `#N/A`. A number, a boolean, an error and `ROW(A1)` are
`#N/A`. A reference returned by OFFSET (1), INDIRECT (2), INDEX (`SHEET(INDEX(T!A1:B2,1,1))` 2), IF (`SHEET(IF(TRUE,U!A1))` 3)
and a union of two areas on sheet T (`SHEET((T!A1,T!B2))` 2) all resolve to their sheet. Ours gives `#N/A` for the INDEX and IF
forms and `#VALUE!` for the union (3 cells; fix T-1631).
Volatile functions through shape-stable formulas (probes/volatile-now-today-shapes, probes/volatile-and-info-rand-true-false-sheet):
NOW, TODAY and RAND are numbers (`TYPE` 1, never text or boolean), `NOW()-NOW()` is 0 within one recalculation,
`INT(NOW())=TODAY()`, `RAND()` is in [0,1) and two calls differ, `TRUE()` and `FALSE()` behave as the literals in
arithmetic, text, comparison and logical contexts (`TRUE()=1` is FALSE, B11, since a boolean never equals a number, and a
boolean ranks above every number: `TRUE()>5` is TRUE, `FALSE()<5` FALSE, B26/C26;
`VALUE(TRUE())` is `#VALUE!`, B28); the engine matches all of them. The eval clock is pinned, so a cell comparing
NOW or TODAY with a threshold the real clock could cross (`NOW()>45000`) cannot be scored; `NOW()>0` and `YEAR(TODAY())>=1900`
can never change and are.
A legacy (non-dynamic) formula does not implicitly intersect SHEET's single reference argument: `SHEET(Table1[])` gives the table's sheet (1..4 for Table1..Table4 on four sheets; Excel-cached values in crates/rustsheet-xlsx/tests/calc_test_no_export/tables.xlsx sheet9 A5:A8; T-1674).
Status: VERIFIED (excel-graph)
Tasks: T-1627, T-1631, T-1674

## N over an array lifts element-wise (T-1616)
Rule: N of an array or array-valued expression is an array of N of each element (number kept, TRUE 1, text,
FALSE and blanks 0, an error element kept); a range still gives its top-left cell (probes/info-logic-argument-modes
A6:A7). Dynamic: `N(IF(1,{2;3}))` spills 2, 3, `N(B1:B3*1)` 1, 2, 3, `N({1,#N/A})` 1, `#N/A`, `N({"",FALSE})` 0, 0,
`N(N({4,5}))` 4, 5, `SUM(N({1,2}))` 3, `ROWS(N({1;2;3}))` 3, `N({1,"a",TRUE})` 1, 0, 1 (`#SPILL!` with data in its
spill range), `SUM(N({1,"a",TRUE}))` 2 dynamic and legacy.
Legacy: `SUM(N({1,2}))` is 3 and `SUMPRODUCT(INDEX(B1:B6,N(IF(1,{2;3}))))` 5, but a legacy cell
`=N(IF(1,{2;3}))` is `#VALUE!` (not the top-left 2), though FORMULATEXT shows it with no `@`, and legacy
`=N(B1:B3*1)` off the rows `#VALUE!`.
Evidence: probes/n-array-lift (excel-graph, 2026-10-16, T-1616).
Not implemented: the legacy cell `=N(IF(1,{2;3}))` gives 2, 3 spilled in ours instead of `#VALUE!` (S!D4:D5,
T-1713, which needs the legacy marker of spec T-1727). T, CHOOSE, INDEX and IFS arguments and nested uses
(`=ABS(N(IF(1,{2;3})))`) behave the same, while N of an array constant or of TRANSPOSE gets an `@` (2):
references.md "Legacy formulas put `@` on array-valued calls and array constants" and "Legacy N and T over a
reference-returning call" (probes/legacy-array-result, probes/legacy-n-t-reference-args).
Status: VERIFIED (excel-graph)
Tasks: T-1616, T-1713, T-1727

## T over an array lifts element-wise (T-1713)
Rule: in a dynamic formula, T of an array value lifts element-wise like N: text elements are kept
(also ""), error elements are kept, and number and boolean elements become "":
`=T({"a",1,#N/A,TRUE,""})` spills "a", "", `#N/A`, "", "", and `=T(IF(1,{"a";"b"}))` spills "a", "b".
It nests (`T(T({"b",2}))` "b", "") and consumers see the lifted array (`ROWS(T({1;2;3}))` 3,
`CONCAT(T({"x",1,"y"}))` "xy"). An array from a range expression lifts too (`T(H1:H2*1)` with
H1 = "x", H2 = 2 is `#VALUE!`, ""). A bare multi-cell range does not lift: see references.md
"T of a multi-cell range is not intersected" (`T(H1:H2)` is one "x", the top-left cell).
In a legacy cell, T over an array-valued IF is `#VALUE!` instead (references.md "Legacy N and T over
a reference-returning call"; not implemented, T-1713, which also owns the legacy spill rows
probes/legacy-array-result S!G11 and probes/legacy-n-t-reference-args S!S11, decision 0069).
Evidence: probes/legacy-array-result S!G1:G2, probes/t-array-lift (excel-graph, 2026-10-16).
Implemented in fn_t (T-1729).
Status: VERIFIED (excel-graph)
Tasks: T-1713, T-1729
