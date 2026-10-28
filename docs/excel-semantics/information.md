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
expression derived from a range (`ISEVEN(Data!C1:C5+0)`), and the
dynamic-array flag Excel writes for the range form. Magnitude >= 2^63 is settled below (T-2165).
Legacy (pre-dynamic-array) formulas, UNVERIFIED for ISEVEN/ISODD (T-1774 moved them onto the
SPEC-0011 `LiftArrays` table entry, so they follow the Analysis ToolPak rule probed for EDATE and
YEARFRAC, probes/scalar-lift-shapes-c E, F, H, Q): a bare range gets no `@` and is `#VALUE!`, a
value operation over a range (`ISEVEN(C1:C5+0)`) gets `@`, and `{5}` gets no `@`. Before T-1774 the
hand-written arm gave the argument a scalar signature (`@` on the range and on `{5}`).
Implemented in: `Model::lifted_parity` and the `ARRAYS_1` entry in functions/lift/kinds/general.rs.
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

## TYPE and ERROR.TYPE of text over 255 UTF-16 units
Rule: `TYPE` of text longer than 255 UTF-16 units is 64 (not 2) and `ERROR.TYPE` of it is 3, as if it were `#VALUE!`,
though the text is no error elsewhere: `ISERROR(REPT("a",32767))` is FALSE and `ISTEXT(REPT("a",256))` TRUE.
`TYPE(REPT("a",255))` is 2 and `ERROR.TYPE` of it `#N/A`; 256, 257, 1000, 8192, 16383, 16384 and 32766 characters are
64 and 3; a 256-character literal and `REPT("a",300)&""` are 64 too. Units, not characters: 127 emoji (254 units)
are 2, 128 (256 units) are 64. The same holds for REGEXREPLACE, REGEXEXTRACT and MID results of any such length,
including ones over 32,767 units (probes/regexreplace-long-result-a, -c, -i), and for a cell holding a typed
256-character constant: `TYPE(A1)` 64, `ERROR.TYPE(A1)` 3, lifted `ERROR.TYPE(A1:A2)` 3 and `#N/A` beside a 255-character
cell, `TYPE(A1:A2)` 64 (probes/type-long-text-refs). DECIMAL rejects text over the same 255
units (engineering.md "DECIMAL of unreadable text"), so the three share one constant.
Evidence: probes/type-long-text, probes/type-long-text-refs (excel-graph, 2026-10-18).
Implemented in: `fn_type`, `fn_errortype` (functions/information/mod.rs), `MAX_SHORT_TEXT_UNITS` (cast.rs).
Status: VERIFIED (excel-graph)
Tasks: T-1817

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
- A scalar error option beside a grid `source` and a wider array option lifts over the broadcast shape, not the source's:
  `IMAGE({url,url},"alt",1/0,{5,5,5})` is 1x3, `#DIV/0!`, `#DIV/0!`, `#N/A` (the source is the first argument, so its past-end
  position is `#N/A`); `{url,url}` with a column `{5;5;5}` is 3x2, `#DIV/0!` throughout (all six elements); `{url;url}` with `{5;5;5}` is 3x1, `#DIV/0!`, `#DIV/0!`, `#N/A` (the source's past-end row is `#N/A` as in the row case, probes/rich-errors-image-grid-error-past-end, T-2229), and `#N/A` throughout with `NA()`; `IMAGE({url,url,url},"alt",1/0,{5,5})` is 1x3 of
  `#DIV/0!` (the earlier error argument beats the option's past-end `#N/A`); `NA()` as the option gives `#N/A` throughout
  (probes/rich-errors-image-grid-error-option, excel-graph, 2026-10-21, T-2205). Without an array option the error fills the source's shape.
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
  A position past the end of a shorter array is an `#N/A` element of that argument, taken in argument order like any error
  element, not an `#N/A` that overrides the rest: an earlier argument's error wins (`IMAGE({#DIV/0!,#DIV/0!,#DIV/0!},"alt",{0,0})`,
  `IMAGE(url,{#DIV/0!,#DIV/0!,#DIV/0!},{0,0})` and `IMAGE(url,"alt",{#DIV/0!,#DIV/0!,#DIV/0!},{5,5})` are `#DIV/0!` in their third
  column; a scalar `#DIV/0!` height before a two-wide width too), and a later one loses (`IMAGE(url,"alt",{3,3},{#DIV/0!,#DIV/0!,#DIV/0!})`
  and a two-wide source with a three-wide `{#DIV/0!,...}` sizing are `#N/A` there). It also beats an invalid option (`{9,9,9}` with
  width `{5,5}` is `#N/A` in the third column; probed with an invalid sizing only) (probes/rich-errors-image-past-end, 8 cells, excel-graph, 2026-10-20, T-1962; VERIFIED).
- IMAGE with an invalid `sizing` (`IMAGE({url,url},"alt",9)`) is `#VALUE!` per element (A16).
- A `field_name` array shorter than the `FIELDVALUE` result along an axis wider than 1 is `#N/A` past its end:
  `FIELDVALUE({1,2,3},{"a","b"})` is `#FIELD!`, `#FIELD!`, `#N/A`; `{1,2,3}` with `{"a";"b"}` is 2x3 of `#FIELD!`, and `{1;2;3}` with `{"a","b"}` is 3x2 of `#FIELD!`; the row-axis form (`{1;2;3}` with `{"a";"b"}`) is UNVERIFIED (probes/rich-errors-image-sizing). A
  shorter `value` is not padded (above).
The engine matches all of these (A16 since T-1345; the option lift since T-1370; through the SPEC-0011 driver, `image_element`, since T-1962, which fixed the past-the-end order: `image_grid` made every past-the-end position `#N/A`).
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
Settled by T-1230 (next entry): ironcalc-calc CELL.xlsx B14 (top-left of a same-sheet range) and probes/info-logic-argument-modes A18 (`#VALUE!`, which is the formula's own-sheet cell Calc!A2) both fit "the top-left cell, read on the formula's own sheet". Superseded by T-3030: the re-filled A18 is 2 (Data!A2), the named-sheet reading.
`CELL("address")` across sheets (unimplemented in T-1218) is implemented since T-2042: see "CELL("address") of a reference on another sheet" below.
Tasks: T-1088, T-1218, T-2042

## ISFORMULA over reference kinds (excel-graph)
Rule: ISFORMULA is TRUE for a cell holding a formula, whatever it returns (`=1+1`, `=""`, `=1/0`), and FALSE for a constant number, a constant text and a blank cell. A spill anchor (`=SEQUENCE(3)`) is TRUE and a spilled child (the cell below it) is FALSE. A cell on another sheet works the same, and the reference may come from INDIRECT, OFFSET or INDEX; `ISFORMULA(Data!D1:D3)` lifts.
Evidence: probes/ifna-isformula-edges Calc!B1:B13 (oracle: excel-graph, 2026-10-12, T-1279).
Status: VERIFIED (excel-graph)
Ours: matches except `ISFORMULA(INDEX(range,1))`, which Excel gives as TRUE and we give as #VALUE! (Calc!B12; T-1453).
Tasks: T-1279, T-1453

## CELL("contents", ref) reads the sheet the reference names (excel-graph, changed 2026-10-28)
Rule (excel-graph, since 2026-10-28, T-3030): `CELL("contents", ref)` and `CELL("type", ref)` read the top-left cell of `ref` **on the sheet `ref` names**, at the top level and nested alike. A multi-cell range is never implicitly intersected.
- On Calc, `=CELL("contents",Data!A2)` is Data!A2 (2.5), `Data!A7:C7` is Data!A7 (2) and `Data!E1:F1` is `#DIV/0!` (Data!E1). These are probes/info-cell-contents-range T4, V1 (2.5), M1, S2, V4 (2) and L1 (`#DIV/0!`); the cells that changed are E3, F3, E8, F8, L1:P1, M8, R1, S2:S4, S6, S7, T1:T5, U1:U3, U5, U6, V1, V2, V4 and V5, re-filled twice on 2026-10-28 with identical values.
- `CELL("type",Data!A7)` is "l" for text in Data!A7 (probes/info-cell-legacy-range J1:K3, M6, N6, re-filled 2026-10-28).
- Sheet order does not matter. With sheets Data, Calc, Other, each reads its named sheet from every other sheet (probes/t3030-cell-first-sheet).
- Nesting does not matter. Under `+0`, N, LET, IF, `&""`, VSTACK, SUM, INDEX, IFERROR, CHOOSE, `@`, SORT and TOCOL, Data!B1 = 7 is read where Calc!B1 = 99 (probes/t3030-cell-contents-nested-sheet A1:A171, A261:A331).
- T-3030 re-filled from excel-graph every case with a cross-sheet `CELL("contents"|"type")` whose sidecar predates 2026-10-28 (decision 0153). Only CELL cells and their spills changed in value; besides them, 38 rich-error cells (GROUPBY, PIVOTBY, REGEX*, FORECAST.ETS and the like), which pre-T-2417 fills had dropped as undecoded, are now scored as `#VALUE!` and pass. info-cell-contents-range was filled twice with identical values; the second fill is not stored, so re-run `rustsheet-oracle fill -b excel-graph --force --case probes/info-cell-contents-range.case.json` to check.
- Evidence: the sidecars above, excel-graph, 2026-10-28. The old (own-sheet) values are in the sidecars at commit 67c550570 (origin/main before T-3030): info-cell-contents-range (2026-10-12), info-cell-legacy-range (2026-10-15) and info-cell-type-cross-sheet-reprobe (2026-10-22).

**History.** Fills up to 2026-10-26 read the formula's own sheet (T-1230, T-1433). Excel for the web has changed since; the single "type" cell that T-2383 saw read the referenced sheet on 2026-10-22 was an early sign. The superseded rule follows, kept for the record.
Old rule (excel-graph fills of 2026-10-12 .. 10-26): `CELL("contents", ref)` read the top-left cell of `ref` taken **on the formula's own sheet**, whatever sheet `ref` names; a multi-cell range is never implicitly intersected, and a legacy (non-array) formula gives the same value as a dynamic one. `=CELL("contents",Data!A2)` on Calc is Calc!A2 (12.5), not Data!A2 (2.5); `Data!A7:C7` is Calc!A7 (21.5); `Data!E1:F1` is Calc!E1 (blank, 0). The earlier "other sheet = `#VALUE!`" reading (probes/info-logic-argument-modes A18) was this: Calc!A2 there is `=XOR(Data!B1)` = `#VALUE!`. Same-sheet ranges give the top-left cell (`A1:A5` in C1, C3, C8 all 11.5), legacy twins included. Legacy-twin evidence for the other info_types is in the next entry (T-1433).
Evidence: probes/info-cell-contents-range Calc!C1:F8, S2:S7, T1:T6, V1:V5 (oracle: excel-graph, 2026-10-12, T-1230).
Status: VERIFIED (excel-graph, 2026-10-28); desktop UNVERIFIED (T-1075 item 9).
Ours: matches since T-3030 (`cell_info` reads the top-left cell on the reference's own sheet; T-1230 to T-3030 read `cell.sheet`). CELL's `reference` argument has a vector signature, so a legacy formula does not intersect it.
Also: `CELL({1,"row"},B2)` is `#VALUE!` (a non-string element) and `CELL({"row",1},B2)` spills 2, `#VALUE!`; `N(range)` with an error top-left cell is that error (Calc!J1, K1); the info_type array lifts per element.
Tasks: T-1230

## CELL("row"/"col"/"address"/"type", range): no intersection; "type" reads the named sheet since 2026-10-28
Rule (excel-graph, 2026-10-15): a legacy (non-array) `CELL(info_type, range)` gives the same value as its dynamic twin and is never implicitly intersected, whether the formula row or column is inside the range or not. `CELL("row",A1:A5)` is 1 and `CELL("address",A1:A5)` is `$A$1` in rows 3 and 8; `CELL("col",B20:D20)` is 2 in columns C, D (inside) and F, G (outside); `CELL("type",A1:A5)` is "v" (the top-left cell; the probe cannot tell it from an intersected cell, since both are numbers, so no-intersection for "type" rests on the vector signature, UNVERIFIED). `row`, `col` and `address` use the reference's own address (`CELL("row",Data!A2:A5)` is 2, `CELL("col",Data!C20:D20)` is 3). Since 2026-10-28 `"type"` reads the named sheet, like `"contents"` (previous entry, T-3030). The 2026-10-15 fill read the formula's own sheet: `CELL("type",Data!A7)` is "b" when Data!A7 is text but Calc!A7 is blank, `Data!A8` is "l" (Calc!A8 is text, Data!A8 blank) and `Data!A9` is "b" (Data!A9 a number, Calc!A9 blank).
Cross-sheet `CELL("address",Data!A2)` was observed as `[<file>.xlsx]Data!$A$2` with the oracle's temporary file name in the T-1433 oracle run (scored since T-2042 in its own case, next entry but one).
Evidence: probes/info-cell-legacy-range (40 cells; oracle: excel-graph, 2026-10-15, T-1433).
Status: VERIFIED (excel-graph); desktop UNVERIFIED (same caveat as "contents", T-1075 item 9).
Ours: matches (`cell_info` "TYPE" reads the reference's own sheet since T-3030). Cross-sheet "address": next entry.
Tasks: T-1230, T-1433
Conflicting cell (T-2383), an oracle artefact: `RIGHTB(CELL("type",Data!C$9),3)` was "l" in gen-m5/74b183ca6fc1d190 Calc!A347
(excel-graph, 2026-10-22 16:40), where Data!C9 is the text "abc" and Calc!C9 is blank, as if this one cell read the referenced
sheet. Re-filling the same workbook (identical input_sha256) at 18:17 gave "b", the own-sheet answer, and no other cell of the
case changed. The re-probe probes/info-cell-type-cross-sheet-reprobe (excel-graph, 2026-10-22, T-2383) fits the own-sheet rule
in all 24 cells: `CELL("type",Data!C$9)` bare, under RIGHTB, LEFT and `&""`, as `Data!C9`, `Data!$C9` and `Data!$C$9`, eleven
repeats of the gen formula (A14:A23, A347) all "b"; rows 8 and 10 "b"; Calc!E9 text gives "l" and Calc!G9 a number "v" for
`Data!E$9` / `Data!G$9` with Data blank there. So in 2026-10-22 excel-graph could, rarely and not repeatably (one observation, T-2383), answer a cross-sheet "type" from
the referenced sheet. Since 2026-10-28 it always does (T-3030, above). gen-m5/74b183ca6fc1d190 and the re-probe were re-filled, and the re-probe's 24 "b" cells are now "l"; the old values are at commit 67c550570.
CELL("contents", ref) as a VSTACK argument is `#VALUE!` (T-2414, probes/t2414-gen-m7-unexplained, excel-graph, 2026-10-23): `VSTACK(CELL("contents",Data!C11),1)`,
`VSTACK(CELL(..),Data!B7:E12)` and `ROWS(VSTACK(CELL(..),Data!B7:E12))` are all one `#VALUE!` (nothing spilled), while `CELL(..)` alone and
`CELL(..)&"|"` give 0 and "|", and `VSTACK(0,Data!B7:E12)` is the 7x4 stack (`#N/A` padded). So the CELL result, not the blank cell, makes the
VSTACK an error (gen-m7/8c4570978f18f4c2 Calc!A782 as filled then. After the T-3030 re-fill it reads Data!C11, which is not blank there, so it stacks). HSTACK (blank and non-blank targets) and TOCOL (a non-blank target only) are probed below (T-2466); other stacking or array-shaping callers of CELL are unprobed. From T-2457 to T-3030 ours errored for every
`VSTACK` argument written as `CELL("contents", ref)` with a literal info type, whatever the target held; T-2466 showed that was wrong (below).

The trigger is a **blank target**, not the call's syntax (T-2466, probes/t2466-cell-contents-stack, excel-graph, 2026-10-28):
- Non-blank targets stack normally: `VSTACK(CELL("contents",Data!B1),1)` with B1=7 spills 7;1 (A1), a text target `abc` spills
  `abc`;1 (A11), a formula target (`=B1*2`) spills its value 14;1 (A21), a range target `Data!B1:B2` uses its top-left cell (7;1, A211),
  `ROWS(..)` is 2 (A191), and a single-argument `VSTACK(CELL("contents",Data!B1))` is 7 (A181). `HSTACK(CELL("contents",Data!B1),1)` is
  7,1 (A51) and `HSTACK(1,CELL("contents",Data!G1))` is 1,`abc` (A201). `TOCOL(CELL("contents",Data!B1))` is 7 (A231).
- A blank target makes the stack one `#VALUE!` (nothing spilled) for VSTACK and HSTACK alike, on the formula's own sheet too
  (`VSTACK(CELL("contents",C11),1)` A31; `HSTACK(..Data!C11..)` A41), with an upper-case info type too (`"CONTENTS"`, A61; a mixed-case `"Contents"` was probed only on a non-blank target,
  which stacks, A71).
- The `#VALUE!` reaches through wrappers: `VSTACK(LET(x,CELL("contents",Data!C11),x),1)` (A111), `VSTACK(IF(TRUE,CELL("contents",Data!C11)),1)`
  (A121) and an info type read from a cell, `VSTACK(CELL(Data!G2,Data!C11),1)` with G2=`contents` (A151), are all `#VALUE!`; the same
  wrappers around a non-blank target stack normally (7;1, A131, A141; `LET(..)&""` gives `"7"`;1, A241), as does `CELL(..)+0` (A221).
  So the blank-target CELL result itself seems to be something a stack argument rejects (an inference: `+0` and `&""` around a blank target inside a stack are unprobed), while `CELL("contents",blank)` alone is 0 and `&"|"` is `"|"` (above).
- Other info types on a blank target are fine: `VSTACK(CELL("type",Data!C11),1)` spills `b`;1 (A161); on a non-blank target `CELL("address",Data!B1)` stacks too (A171).
- Beside a scalar error argument the first erroring argument wins, as for any scalar-error argument of VSTACK (arrays.md, T-1695):
  `VSTACK(CELL("contents",Data!C11),1/0)` is `#VALUE!` (A81), `VSTACK(1/0,CELL("contents",Data!C11))` is `#DIV/0!` (A91);
  with a non-blank target the error argument alone decides, `VSTACK(NA(),CELL("contents",Data!B1))` is `#N/A` (A101).

What the blank-target result is (T-3030, probes/t3030-cell-contents-blank-shapers, excel-graph, 2026-10-28; Data!C11 blank):
- It is an empty **value**, not a reference. `ISBLANK(..)` is TRUE (A51), `TYPE(..)` 1 (A41), `COUNTA(..)` 1 (A231),
  `ISREF(..)` FALSE (A81), `..+0` and `-..` 0, `..&""` and `T(..)` "" (A1, A11, A111, A151), `LEN(..)` 0, `..=""` and `..=0` TRUE.
- Array-taking functions reject it with `#VALUE!`. This holds for TOCOL, TOROW, TAKE, WRAPROWS, CHOOSECOLS, FILTER, MAP,
  ROWS, VSTACK around an INDEX of it (A121; whether INDEX itself rejects it is unprobed), and VSTACK through `@`, CHOOSE, IFERROR, parentheses and MAP (A21, A31, A121:A141, A161:A171, A191, A211,
  A311, A331, A351:A361, A381). A blank range target such as `Data!C11:D12` behaves the same (A291), and so does an argument
  that is not first (`VSTACK({1;2},..)`, A301).
- SORT, UNIQUE and EXPAND accept it as 0 (A201, A371; EXPAND pads with `#N/A`, A181), as do SUM, N, `+0`, `&""` and
  LET with `x+0` (A221, A101, A1, A11, A341).
- Targets that are not blank stack: a `=""` target stacks "" (A271), a `=C11` target (0) stacks 0 (A281), and a bare
  `VSTACK(Data!C11,1)` stacks 0;1 (A321).

A blank **reference** read through a wrapper (T-3053, probes/t3053-blank-reference-wrappers, excel-graph, 2026-10-28; Data!C11
blank) stays a reference that stacks as 0 through IF, LET, CHOOSE, parentheses, `@` and INDEX (`VSTACK(IF(TRUE,Data!C11),1)`,
`VSTACK(LET(x,Data!C11,x),1)`, `VSTACK(CHOOSE(1,Data!C11),1)`, `VSTACK((Data!C11),1)`, `VSTACK(@Data!C11,1)`,
`VSTACK(INDEX(Data!C11,1),1)` and `VSTACK(INDEX(Data!C11:C12,1),1)` are all 0;1; `TOCOL(IF(TRUE,Data!C11))` and
`TOCOL(LET(x,Data!C11,x))` 0; `ROWS(IF(TRUE,Data!C11))` 1; `COUNTA(IF(TRUE,..))` and `COUNTA(LET(x,..,x))` 0). IFERROR and
IFNA do not: they return the empty **value**, so `VSTACK(IFERROR(Data!C11,5),1)` and `VSTACK(IFNA(Data!C11,5),1)` are
`#VALUE!` like the CELL result (A41, A141). The CELL result stays the empty value through LET and IF
(`VSTACK(LET(x,CELL("contents",Data!C11),x),1)`, `VSTACK(IF(TRUE,CELL(..)),1)` `#VALUE!`, A161, A171). INDEX itself rejects
it: `INDEX(CELL("contents",Data!C11),1)` is `#VALUE!` and its TYPE 16 (A1, A151). So the empty value comes from a call that
returns a value (CELL, IFERROR, IFNA), and the reference-returning wrappers pass a blank reference on unchanged.

Ours (since T-3030) matches the non-blank and literal-call cells: CELL reads the named sheet, and VSTACK and HSTACK reject an
argument written as `CELL("contents", ref)` (literal info type) whose target is blank. Ours cannot yet tell the empty value from
a blank reference read through IF, LET, CHOOSE, IFERROR or parentheses, which our evaluator also turns into a bare empty cell.
So the wrapped forms (t2466 A111, A121, A151), the other array-taking callers listed above (blank-shapers A21 .. A381, nested-sheet A231)
and `COUNTA(..)` (blank-shapers A231: Excel 1, ours 0), and IFERROR/IFNA of a blank reference inside VSTACK (t3053 A41, A141) still differ: T-3053.

## CELL("address") of a reference on another sheet names the workbook file (excel-graph)
Rule: `CELL("address", ref)` for a reference on another sheet is `[<file name>]<sheet>!<address>`, the sheet name
**never quoted**, whatever it holds: `[b.xlsx]Data!$B$2`, `[b.xlsx]My Sheet!$B$2`, `[b.xlsx]2024!$C$3`,
`[b.xlsx]Bob's!$A$1` (the apostrophe not doubled), `[b.xlsx]a-b!$D$4`, `[b.xlsx]My Sheet!$XFD$1048576`. A range gives
its top-left cell (`Data!B2:C3` is `[b.xlsx]Data!$B$2`). A reference on the formula's own sheet, written with or
without its sheet name (`B2`, `Calc!B2`), is the bare `$B$2`. The file name is the workbook's own, extension
included; in the oracle it is the upload name `<sha256>.xlsx`, which the fill now records in the sidecar's
`source.workbook_name` and the eval adapter passes to the engine (`WorkbookOptions::name`, the name without `.xlsx`).
Also: `CELL("filename")`, `CELL("filename",B2)` and `CELL("filename",Data!B2)` are all `#VALUE!` in Excel for the web
(the same case's B1:B4, scored since T-2052); desktop Excel returns the full path (UNVERIFIED here; M14).
Evidence: probes/info-cell-address-cross-sheet Calc!A1:A9, B1:B4; probes/info-logic-argument-modes Calc!A19
(oracle: excel-graph, 2026-10-19, T-2042).
Status: VERIFIED (excel-graph); unsaved workbook UNVERIFIED (Excel's `Book1` form is unprobed; desktop only).
Ours: matches since T-2042; "filename" is `#VALUE!` always since T-2052. An unnamed workbook
(`WorkbookOptions::name` `None`, the default) gives `#N/IMPL!` for cross-sheet "address" rather than guess; a union on another sheet is `#N/IMPL!` (unprobed).
Tasks: T-2042, T-2052

## TYPE of a one-cell range, ISEVEN/ISODD past 2^63, CELL over a range info_type (T-2156, T-2165, T-2161)
- `TYPE` of a one-cell range is its cell's type, not 64: `TYPE(D7:D7)` and `TYPE(B3:B3)` are 1 (ours 64). Multi-cell ranges
  stay 64 (above). Evidence: gen-m5/2abdc1524e5aa685 Calc!A371, gen-m5/c0651f0e0fb544d0 Calc!A203 (excel-graph, 2026-10-20).
  The cell's kind decides: blank 1, number 1, text 2, boolean 4, error 16, text over 255 units 64, a blocked spill anchor
  (`#SPILL!`) 16 and a working numeric one 1; one-cell `OFFSET(A2,0,0,1,1)` and `INDIRECT("A2:A2")` results classify the same way (probed: OFFSET over text, blank and error; INDIRECT over text, boolean, error and blank).
  Evidence: probes/type-one-cell-range-kinds (T-2262, excel-graph).
- ISEVEN and ISODD of a value at or above 2^63 read it as even (every double from 2^53 up is an even integer):
  `ISODD(INT(1E+21))` is FALSE, `ISEVEN(3.333333333333325E+29)` and `ISEVEN(8.999999753086524E+28)` TRUE. Implemented in `Model::parity_of`
  (functions/information/mod.rs) with an f64 remainder (an i64 cast saturated to i64::MAX, which is odd). This settles the ">= 2^63" line of "ISEVEN/ISODD over a range". Evidence: gen-m5/ee1667411bbe2ccc
  Calc!A159, gen-m5/1315170eef6c413e Calc!A396, gen-m5/50d8a085335c05e6 Calc!A92.
- CELL lifts over a multi-cell range given as info_type, as over an array: `CELL(INDEX(D2:E9,0,1),A9)` is `#SPILL!` under a
  filled column (ours was `#VALUE!` before T-2161). Evidence: gen-m5/c0651f0e0fb544d0 Calc!A293.
- An error element of a CELL info_type grid passes through as that error, per element; any other non-text element (number,
  blank) is `#VALUE!`. `CELL({#N/A,"row"},B2)` spills #N/A, 2; `CELL({"row",#N/A},B2)` spills 2, #N/A; `CELL(D1:D2,B2)` with
  D1=#DIV/0!, D2="row" spills #DIV/0!, 2; a range `{"row","col"}` at B3 spills 3, 2; `{"row",5}` spills 3, #VALUE!; `{"row",blank,"col"}`
  spills 3, #VALUE!, 2. Scalar `CELL(A1,B1)` with A1=#N/A is #N/A. Evidence: probes/info-cell-info-type-grid-errors (excel-graph,
  T-2232); implemented in `lifted_cell`.
Status: VERIFIED (excel-graph, desktop Excel not checked) for the cells cited, including TYPE of a one-cell range over a
text, boolean, error or blank cell and over an OFFSET/INDIRECT result (T-2262). ISEVEN/ISODD implemented (T-2165);
CELL's range info_type implemented (T-2161, `fn_cell`); TYPE of a one-cell range implemented (T-2156, `fn_type`).

## ISEVEN of a value just below an integer (T-2382)
Rule: ISEVEN and ISODD take the parity of `TRUNC(ABS(x) + 1E-10)`: a value no more than 1E-10 below an integer counts as that
integer, whatever its size, with the same window for negatives (on the absolute value). `ISODD(7-1E-10)`,
`ISODD(1001-1E-10)`, `ISODD(1000001-1E-10)`, `ISODD(0.9999999999)` and `ISODD(-(7-1E-10))` are TRUE; `ISODD(7-1.01E-10)`,
`ISODD(1001-1.01E-10)`, `ISODD(7-2E-10)`, `ISODD(1-2^-33)` and `ISODD(6.9999999)` are FALSE. It is an absolute window, not a
15-digit cut: `ISODD(1E+15-0.25)` is TRUE (999999999999999), and values just above an integer stay put (`ISODD(6+1E-9)` FALSE).
So `ISEVEN(POWER(49,0.5-1E-15))` is FALSE: Excel's POWER is the same 6.999999999999972 as ours (the cell itself; INT and TRUNC
of it are 6), and the 1E-10 window lifts it to 7 (gen-m5/bf8903e05ec369ca Calc!A392).
The fit: `TRUNC(ABS(x)+1E-10)` in IEEE doubles matches all 128 ISEVEN/ISODD cells of the three probes. The candidates
"round to 10 decimals", "fraction ≥ 1-1E-10" and windows of 1.009E-10 or more each miss cells.
Evidence: probes/iseven-isodd-below-integer, probes/iseven-isodd-rounding-threshold, probes/iseven-isodd-rounding-window
(excel-graph, 2026-10-22, T-2382).
Implemented in: `Model::parity_of` (crates/rustsheet-engine/src/functions/information/mod.rs, T-2382)
Status: VERIFIED (excel-graph)
Tasks: T-2382

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
and a union of two areas on sheet T (`SHEET((T!A1,T!B2))` 2) all resolve to their sheet. Ours matches (T-1631).
Volatile functions through shape-stable formulas (probes/volatile-now-today-shapes, probes/volatile-and-info-rand-true-false-sheet):
NOW, TODAY and RAND are numbers (`TYPE` 1, never text or boolean), `NOW()-NOW()` is 0 within one recalculation,
`INT(NOW())=TODAY()`, `RAND()` is in [0,1) and two calls differ, `TRUE()` and `FALSE()` behave as the literals in
arithmetic, text, comparison and logical contexts (`TRUE()=1` is FALSE, B11, since a boolean never equals a number, and a
boolean ranks above every number: `TRUE()>5` is TRUE, `FALSE()<5` FALSE, B26/C26;
`VALUE(TRUE())` is `#VALUE!`, B28); the engine matches all of them. The eval clock is pinned, so a cell comparing
NOW or TODAY with a threshold the real clock could cross (`NOW()>45000`) cannot be scored; `NOW()>0` and `YEAR(TODAY())>=1900`
can never change and are.
A legacy (non-dynamic) formula does not implicitly intersect SHEET's single reference argument: `SHEET(Table1[])` gives the table's sheet (1..4 for Table1..Table4 on four sheets; Excel-cached values in crates/rustsheet-xlsx/tests/calc_test_tables/tables.xlsx sheet9 A5:A8; T-1674).
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
T, CHOOSE, INDEX and IFS arguments and nested uses (`=ABS(N(IF(1,{2;3})))`) behave the same, while N of an
array constant or of TRANSPOSE gets an `@` (2): references.md "Legacy formulas put `@` on array-valued calls and
array constants" and "Legacy N and T over a reference-returning call" (probes/legacy-array-result,
probes/legacy-n-t-reference-args).
Implemented (T-1713): the legacy rewrite marks such an N call (`NLegacy`, SPEC-0012), whose kernel reads an
array argument as `#VALUE!`; S!D4:D5 match. A dynamic `=N(IF(1,{2;3}))` is unmarked and still spills. Which
marked shapes are verified and which are extrapolated: references.md "Legacy N and T over a reference-returning
call" (probes/legacy-n-t-marker-shapes; `=N(INDEX({2;3},1))` matches since T-1921).
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
a reference-returning call"; implemented in T-1713 by the `TLegacy` marker of SPEC-0012, with no spill:
probes/legacy-array-result S!G10:G11 and probes/legacy-n-t-reference-args S!S10:S11 match, decision 0069).
Evidence: probes/legacy-array-result S!G1:G2, probes/t-array-lift (excel-graph, 2026-10-16).
Implemented in fn_t (T-1729).
Status: VERIFIED (excel-graph)
Tasks: T-1713, T-1729

## SHEETS counts the sheets a reference spans (VERIFIED, excel-graph)
Status: VERIFIED (excel-graph). Tasks: T-2115, T-2181.
Rule: `SHEETS(reference)` is the number of sheets the reference covers: 1 for a cell or range on one sheet, written
bare, sheet-qualified or quoted (`SHEETS(A1)`, `SHEETS($A$1:$B$2)`, `SHEETS(Data!A1:B2)`, `SHEETS('Beta'!A1)`), and the
count of sheets between the corners, inclusive, for a 3-D span in either order: with sheets Alpha, Beta, Gamma, Data,
`SHEETS(Alpha:Gamma!A1)` and `SHEETS(Gamma:Alpha!A1)` are 3, `SHEETS(Beta:Data!A1:B2)` 3, `SHEETS(Alpha:Data!B2)` 4.
The result is a plain number (`TYPE` 1, `ISNUMBER` TRUE, `+0`, `*1`, `=3` TRUE). A reversed span reads the same in SUM too
(`SUM(Gamma:Alpha!A1)` = `SUM(Alpha:Gamma!A1)`).
Evidence: probes/info-sheets-info-types Alpha!D1:D24, D50:D52 (excel-graph, 2026-10-20, T-2115). The engine implements a single-area
reference and a 3-D span in either order (T-2181); only a written reference (cell, range, 3-D span) is read; a union, a lifted reference grid, a name, OFFSET,
INDIRECT, INDEX, an IF/CHOOSE pick, a cross-sheet range operator between two sheet-qualified cells
(`Alpha!A1:Beta!B1`), a non-reference and an error argument stay `#N/IMPL!`. An unqualified cell joined to a cell on
another sheet is `#VALUE!`: that is how the unquoted cell-like span `SHEETS(S1:S3!A1)` reads, `S1:(S3!A1)`, as in
Excel (T-2193), and as the range operator itself is between corners on two sheets (references.md "Sheet-qualified
range corners"). Unprobed, because the oracle canary refuses them (T-2113): a union, INDIRECT,
OFFSET, a defined name, a non-reference or error argument, bare `SHEETS()`, and a range operator across sheets
(`SHEETS(Alpha!A1:Beta!B1)`, refused 2026-10-21).
Probe trap: sheets named like cells (S1, S2, S3) break an unquoted 3-D span on Excel for the web: `SUM(S1:S3!A1)`,
`SUM(S3:S1!A1)` and `SHEETS(S1:S3!A1)` are `#VALUE!` and `SHEETS(S3:S1!A1)` is 1, while the quoted `'S1:S3'!A1` works
(`SUM` 13, `SHEETS` 3) and so does a single-sheet `S2!A1` (probes/sheet-names-like-cells-3d-spans B1:B7, excel-graph,
2026-10-20). Excel writes such names quoted; give probe sheets names that are not cell references.

## INFO is #VALUE! in Excel for the web
Status: VERIFIED (excel-graph); desktop pending T-2182. Tasks: T-2115, T-2190, T-2182.
Rule (web): every INFO call is `#VALUE!`, whatever the type_text: "recalc", "system", "release", "directory",
"osversion", "origin", the legacy "memavail", "memused" and "totmem", `""` and `"x"`, in lower, upper or mixed case
(`TYPE(INFO("release"))` 16, `ERROR.TYPE(INFO("x"))` 3, `ISTEXT(INFO("recalc"))` FALSE, `LEN(INFO("osversion"))>0`
`#VALUE!`). So no value turned out host-dependent. As with ENCODEURL and FILTERXML (web.md), this is the golden for
Phases 1-3 (decision 0039): the engine follows it in T-2190, which is expected to differ from desktop Excel; T-2182
(M14) asks desktop. Unprobed: `INFO(#N/A)`, `INFO(1)`, `INFO(TRUE)`, an array argument (the oracle canary refuses a
non-literal INFO, T-2113), and `INFO("numfile")` (canary-dependent). The engine also returns `#VALUE!` for `"numfile"` and its
engine-only `"timezone"`/`"timezones"` (T-2190), unprobed and awaiting oracle confirmation (T-2182).
Evidence: probes/info-sheets-info-types Alpha!D25:D49 (excel-graph, 2026-10-20, T-2115).
