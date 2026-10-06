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
