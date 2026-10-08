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
`#CONNECT!` whatever the arguments (after argument errors and IMAGE's empty/number `#VALUE!`, see above).
Excel fetches the image or the quotes; formulas here do no I/O (CLAUDE.md directive 5).
Excel returns the same errors only when the fetch fails: `IMAGE("https://example.invalid/none.png")` is
`#BLOCKED!` and `STOCKHISTORY("QZXQZXQ",DATE(2020,1,2))` is `#CONNECT!`. A reachable URL or a real ticker
is the deviation. Which of the two errors Excel gives for other failures is unknown beyond these two cells.
Evidence: probes/rich-errors-image A1, probes/rich-errors-stockhistory A1, probes/rich-errors-args A40, A50
(excel-graph); the engine's results are unit-tested in `test/test_fn_offline_errors.rs`.
Status: DEVIATION (decision 0014)
Implemented in: functions/information/offline_errors.rs
Tasks: T-1013

## FIELDVALUE of a plain value is `#FIELD!`
Rule: `FIELDVALUE(value, field_name)` is `#FIELD!` for every plain (non linked data type) value, which is
every value here (decision 0027); an array `value` gives an array of `#FIELD!` (argument-error precedence
is in "IMAGE/STOCKHISTORY/FIELDVALUE argument errors" above).
Evidence: probes/rich-errors-field A1, probes/rich-errors-args A60:A61, A20:A28 (excel-graph).
Status: VERIFIED (excel-graph)
Implemented in: functions/information/offline_errors.rs
Tasks: T-1013
