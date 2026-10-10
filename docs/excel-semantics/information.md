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
URL-shaped source reaches the fetch (which exact strings count as a URL is not probed). `IMAGE(1/0,NA())` is `#DIV/0!` (a `source`
error wins over an `alt_text` error; other orders are not probed). The raw IMAGE cells (A10, B10, A13, A14, A16, A19) and the `ROWS`/`COLUMNS` of `IMAGE("")` and `IMAGE(TRUE)` came back with an undecodable rich error type and stay pending; the wrapped `ERROR.TYPE` cells carry the evidence. The VERIFIED rules above cover argument kinds; the DEVIATION status below covers only the missing fetch.
The engine lifts over array and range `source`/`stock`/`value`/`field_name` and gives `#VALUE!` for non-URL IMAGE sources (URL-shaped = starts with http:// or https://, not probed further); `ERROR.TYPE` lifts over arrays too (probes/rich-errors-arg-kinds F10:G10, F13:G14, F40:G41; over a direct multi-cell range it follows the IS* lifting evidence). UNVERIFIED (engine guesses, probe task filed): FIELDVALUE with two array arguments of different shapes (the engine fills the larger shape with `#FIELD!`; the usual broadcast would pad `#N/A`), an error element inside an array `field_name` (engine: `#FIELD!`), an error element inside an array `source`/`stock` (engine: the element keeps its error), `ERROR.TYPE` over a direct multi-cell or one-cell range (engine: lifts like the IS* tests), an array first argument of IMAGE/STOCKHISTORY with an error in a later argument (engine: that scalar error), and which strings count as an IMAGE URL (engine: `http://`/`https://` prefix).
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
Tasks: T-1013, T-1141, T-1179

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
- TYPE takes a range or an array whole: `TYPE(A2:C2)` and `TYPE({1,2})` are 64, with no spill. `TYPE` of one number cell is 1.
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
Ours: matches since T-1218 (SHEET/N of a range, CELL's array info_type, ISFORMULA's lift, `CELL("contents")` of a multi-cell range on another sheet = `#VALUE!`).
UNVERIFIED: ironcalc-calc CELL.xlsx B14 (excel-cached, a legacy CSE array formula) shows `CELL("contents",A10:A14)` on the formula's own sheet giving the top-left cell (3.4, not the intersection with row 14), while probes/info-logic-argument-modes A18 (a dynamic-array formula) is `#VALUE!` for an other-sheet row range. We key on the sheet, but the evidence does not separate that from "the formula cell is not inside the range" or "CSE reads the top-left, dynamic arrays give `#VALUE!`". Also unprobed: a non-string element in an array `info_type` (we give `#VALUE!`) and `N(range)` whose top-left cell is an error (we pass the error through).
`CELL("address")` across sheets depends on the file name, which the engine does not know (T-1218 leaves it unimplemented).
Tasks: T-1088, T-1218
