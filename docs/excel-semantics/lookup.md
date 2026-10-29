# Lookup & reference

## INDIRECT to a sheet that does not exist is #REF! (desktop Excel)
Rule: `INDIRECT("NoSuchSheet!A1")` (A1 or R1C1 style) is #REF!, so `ROW`/`COLUMN` of it are
#REF! too, and dependants propagate it.
Evidence: ironcalc-calc LOOKUP_AND_REFERENCE/ROW_COLUM.xlsx ROW_COLUMN!K7, L7
(`ROW/COLUMN(INDIRECT("NonExist!W3"))`) and INDIRECT.xlsx r1c1_in_other_sheets!A5
(`INDIRECT("Sheet7!R2C2",FALSE)`), cached by Microsoft Macintosh Excel 16.
Status: VERIFIED (Excel-cached values)
Implemented in: the engine already returns #REF! (those cells pass).
Oracle caveat: **Excel for the web disagrees.** Through the Graph oracle the same formulas give
a reference to an empty cell (`INDIRECT(...)` = 0, `ISREF(...)` = TRUE, `ROW(...)` = 3), probed
2026-10-04. The web gives the same after a sheet edit: a sheet renamed away, or a name added and
then renamed (T-1608, the unscored cells of probes/sheet-ops/sheet-ops-rename-indirect-text,
-add-then-rename and -add-resolves-indirect). Never take oracle values as evidence for references to missing sheets
(docs/architecture/oracle.md, "Known divergences"; desktop wins per decision 0030).

## INDIRECT resolves a defined name that holds a reference
Rule: `INDIRECT("<name>")` of a defined name whose body is a cell or range reference is that
reference: `ISREF(INDIRECT("RCR"))` is TRUE with `RCR` = `Sheet1!$B$1`. Text that is neither a
reference nor such a name is `#REF!`, so `ISREF(INDIRECT("R0"))` is FALSE, also when `R0` is a
defined name, since a name spelled like an out-of-range R1C1 reference is never resolved
(references.md "R1C1-shaped sheet and defined names").
The value is read through the name (`SUM(INDIRECT("nm"))` is 7, a range name sums and `ROWS` counts it, `ISREF` is TRUE).
`a1` FALSE and TRUE both resolve a name (`INDIRECT("nm",FALSE)` is 7). Name text is case-insensitive
(`"NM"`, `"mixedcase"`) and trailing spaces are ignored (`"nm "` resolves) but a leading space is `#REF!`.
A `Sheet!name` prefix picks the name scoped to that sheet (`INDIRECT("Sheet2!nm")` reads Sheet2's own `nm`
though a workbook `nm` exists), and a prefix on a workbook-scoped name falls back to it (`"Sheet1!nm"`, `"Sheet2!gl"`);
an unprefixed name is read sheet scope first. A name whose body is a formula is `#REF!` and `ISREF` FALSE:
`OFFSET(...)`, `Sheet1!$B$1+1`, a constant `=5` and a text constant `="B1"`. A multi-cell name spills as a range
(`INDIRECT("rng")` in a cell with occupied cells below is `#SPILL!`).
Evidence: probes/r1c1-name-use-rcr A2, r1c1-name-use-r0, -c0, -r0c1, -r1048577 A2, probes/indirect-defined-name-text
A1:A30, Sheet2!C1:C2 (an unprefixed name text on Sheet2 reads Sheet2's own `nm`, not the workbook one; excel-graph, 2026-10-15 and 2026-10-16, T-1682).
Implemented in: `indirect_defined_name` (functions/lookup_and_reference/mod.rs); ISREF evaluates
an INDIRECT argument (functions/information/mod.rs).
Status: VERIFIED (excel-graph)
Tasks: T-1657, T-1682

## INDIRECT of `Sheet!Name` text: the prefix must be spelled as a formula would spell it
Rule: `INDIRECT("<Sheet>!<Name>")` reads a name that exists only at that sheet's scope from any
sheet (`INDIRECT(A1&"!tn")` on Summary, A1 `CoB`, is CoB's `tn`; a range name sums, counts and
spills; `ISREF` TRUE), sheet and name text case-insensitive. Unprefixed text reads the INDIRECT's
own sheet scope only: `INDIRECT("tn")` is `#REF!` on a sheet with no `tn` though other sheets
define one, and `"Summary!tn"` is `#REF!` too. The prefix follows formula quoting: unquoted, the
sheet name may hold only letters, digits, `_` and `.`, so `"Co A!tn"`, `"P&L!tn"` and `"O'B!tn"`
are `#REF!` (also with `a1` FALSE) while `"2024!tn"`, `"Q1!tn"` and `"x.y!tn"` resolve; quoted,
`''` is one quote (`"'O''B'!tn"` resolves) and any sheet name may be quoted (`"'CoB'!tn"`).
Verified for ASCII sheet names only; unquoted non-ASCII names (`"Données!tn"`) and letter-after-digit
names (`"1Q!tn"`) are unprobed (the engine accepts them).
Cell text follows the same quoting for `"Co A!B1"`, `"P&L!B1"` and `"O'B!B1"` (`#REF!`), and
`"Q1!B1"`, `"x.y!B1"` and `"2024!B1"` (an unquoted all-digit sheet, `"y2024"`) resolve; the engine
quotes a digit-leading prefix for its lexer (T-2678). A typed `=2024!B1`, the R1C1 form `INDIRECT("2024!R1C1",FALSE)` and `"2024!B1:2024!B3"` are unprobed.
Evidence: probes/indirect-sheet-scoped-name (excel-graph, 2026-10-25, T-2660).
Calipers note: `calipers/pv_goldens/pv26_020` and `pv26_2_020` `Summary!C17:...17` are not evidence
for this rule: the corpus anonymisation renamed the text in `Summary!B17` to `Company0691` but left
the sheet it names as `ConAgra` (whose `TargetName`, D24, is the cached `Company2277`), so the
workbook as shipped has no `Company0691` sheet and `#REF!` is right for it (lookup.md "INDIRECT to
a sheet that does not exist is #REF!"). probes/t2867-calipers-anonymised-inputs (excel-graph,
2026-10-27) confirms it on the shipped inputs: `#REF!` for C17:O17 and for MAX, AVERAGE, MEDIAN and
MIN over M7:M21. Those cells are scored against the probe through cached overrides (decision 0133, T-2867).
Implemented in: `indirect_defined_name` (functions/lookup_and_reference/mod.rs).
Status: VERIFIED (excel-graph)
Tasks: T-2660, T-2867

## INDIRECT's a1 argument is coerced like a boolean; text must be TRUE/FALSE
Rule: `a1` is coerced like a logical argument: a nonzero number is TRUE (`INDIRECT("H2",2)` and
`INDIRECT("H2",-1)` = 5, `"TRUE"` text = TRUE), `0` is FALSE, so `INDIRECT("H2",0)` reads "H2" as R1C1
and is `#REF!`, while `INDIRECT("R2C8",0)` = 5. Empty-string text is `#VALUE!` in both styles (B4, B10) and `"x"` is `#VALUE!` in A1 style (B5).
Not probed: `"FALSE"` text and booleans read from cells.
Evidence: eval/suites/probes/cases/indirect-text-broadcast.case.json B3:B10 (oracle: excel-graph, 2026-10-05, T-0957).
Status: VERIFIED (Excel for the web only; the engine matches all of these)

## INDIRECT R1C1 relative offsets
Rule: `INDIRECT("R[-1]C[6]",FALSE)` is relative to the cell holding the formula (`R[-11]C[6]` in B12 reads H1, "data";
B11's `R[-1]C[6]` = 0 reads the empty H10). An offset that leaves the sheet is not `#REF!`: it **wraps modulo the sheet size** (1048576 rows, 16384
columns), in both directions, for rows and columns (`R[-20]C` in D15 reads D1048571, `R[1048560]C` in D30 reads D14,
`RC[-5]` in C5 reads XFB5, `RC[16383]` in C6 reads B6; a clamp would have read D1/D1048576/A5/XFD6). An offset equal to
minus the sheet size (`R[-1048576]C`, `RC[-16384]`) is `#REF!` (positive offsets of the sheet size or more are not probed; the engine treats them symmetrically); `R[-1048575]C[-1]` wraps to row 16
(0, an empty cell). A result row exactly at the last row (`R[1048560]C` in B16) is also fine.
RustSheet matches (`parse_r1c1_number`, engine utils.rs).
Evidence: indirect-text-broadcast.case.json B11:B16 (T-0957); indirect-r1c1-offset-overflow.case.json (oracle:
excel-graph, 2026-10-14, T-0960).
Status: VERIFIED (Excel for the web only)
Tasks: T-0960

## MATCH on a 2-D lookup_array is #N/A; argument errors win
Rule: `MATCH` with a lookup_array of more than one row and column returns `#N/A` even when the value
is in it (`=MATCH(Data!C$5,Data!A5:C9,1)` with C5 = 2 inside A5:C9 → `#N/A`; no top-level 2-D MATCH
call in gen-m2 returns a number), but an error in lookup_value is returned first
(`=MATCH(MAX(MONTH(-0.5)),Data!A7:C9)` → `#NUM!`).
Evidence: gen-m2/cbdabe32a32a4012 Calc!A227, ce315bb699e1be6f Calc!A135, 629e6e8b76fab8b9 Calc!A313. Oracle: excel-graph, 2026-10-04 (gen-m2 sidecars, T-0203).
A 2-D *array* is `#N/A` too: `MATCH(2,{1,2;3,4},0)`, `MATCH(2,{1,2;3,4})`, `MATCH(2,{1,2;3,4},-1)` and
`MATCH(2,A1:B2*1,0)` (probes/lookup-2d-vector H14:H17, excel-graph, 2026-10-07, T-0224).
Status: VERIFIED
Tasks: T-0210, T-0224

## MATCH, VLOOKUP and HLOOKUP over a single value
Rule: a `lookup_array` (MATCH) or table (VLOOKUP, HLOOKUP) that is a single value rather than a
reference or an array is no table: text that is not a number is `#VALUE!`, any other value `#N/A`,
an error is itself; a one-cell reference, also picked by IF or INDEX, is searched as a one-cell
range. A2 = "b", B2 = 2: `MATCH("b","b",0)`, `MATCH(2,"b",0)`, `MATCH("b","b")`,
`MATCH("b",A2&"",0)`, `MATCH("b",T("b"),0)`, `VLOOKUP("b","b",1,FALSE)` and HLOOKUP's are
`#VALUE!`; `MATCH(2,2,0)`, `MATCH(2,3,0)`, `MATCH(2,2)`, `MATCH(TRUE,TRUE,0)`, `MATCH("2","2",0)`,
`MATCH(2,B2*1,0)`, `VLOOKUP(2,2,1,FALSE)`, `VLOOKUP("2","2",1,FALSE)` and
`VLOOKUP(TRUE,TRUE,1,FALSE)` `#N/A`; `MATCH(2,1/0,0)` `#DIV/0!`; `MATCH(2,{2},0)`,
`MATCH("b",{"b"},0)`, `MATCH("b",A2,0)`, `MATCH(2,B2,0)`, `MATCH(2,IF(TRUE,B2),0)` and
`MATCH(2,INDEX(B2:B3,1),0)` 1, `VLOOKUP(2,B2,1,FALSE)` 2, `HLOOKUP("b",A2,1,FALSE)` "b". A legacy
formula reaches this through its intersected lookup table (references.md "Legacy MATCH, VLOOKUP
and HLOOKUP intersect an operation in their lookup table"). LOOKUP differs: it reads a single
value as a one-element array ("LOOKUP vector form with 2-D or short vectors"). One-cell
references picked by CHOOSE, OFFSET or INDIRECT, and a defined name for one cell, are one-cell
ranges too (`MATCH("b",CHOOSE(1,A2),0)`, `MATCH(2,nmB,0)` are 1; `VLOOKUP(2,nmB,1,FALSE)` is 2).
Single texts as the MATCH/VLOOKUP table: `1/2/2020`, `$1`, `50%`, ` 2 `, `1E2`, `2:30` and
`1,000` read as numeric (`#N/A`); `TRUE`, the empty text and `0x1` do not (`#VALUE!`), matching
the engine's `cast_number`.
Evidence: probes/match-single-value-lookup-array (excel-graph, 2026-10-18, T-1862);
probes/match-single-cell-forms-and-texts (36 cells, excel-graph, 2026-10-20, T-1939).
Engine: `Model::lookup_table` (lookup.rs) and `Model::evaluate_single_sheet_arg` (util.rs).
Status: VERIFIED
Tasks: T-1862, T-1939

## LOOKUP vector form with 2-D or short vectors
Rule: in `LOOKUP(value, lookup_vector, result_vector)`:
- A 2-D lookup_vector is searched like the array form searches its array: along its first column, or
  along its first row when it has more columns than rows. With A1:B2 = 1,2;3,4 and C1:C2 = 10,20,
  `LOOKUP(2,A1:B2,C1:C2)` = 10 (A1:A2 searched), `LOOKUP(99,A1:B2,C1:C2)` = 20, and
  `LOOKUP(0,A1:B2,C1:C2)` = `#N/A`. With A1:D2 = 1,2,5,7;3,4,6,8, `LOOKUP(5,A1:D2,F1:F4)` = F3
  (A1:D1 searched). Arrays and computed arrays behave the same (`LOOKUP(2,{1,2;3,4},{5,6})` = 5).
- A 2-D result_vector is `#N/A`: `LOOKUP(2,A1:A2,C1:D2)`, `LOOKUP(2,{1,2},{5,6;7,8})`.
- A result *range* shorter than the match position is read on past its end: down for a column,
  and to the right for a row or a single cell. `LOOKUP(5,A1:C2,F1:F2)` = F3, `LOOKUP(7,A1:D1,F1:F2)` =
  F4, `LOOKUP(99,A1:B2,F1)` = G1 and `LOOKUP(3,A1:A2,F1)` = G1. A result *array* is not:
  `LOOKUP(99,A1:B2,{10})` = `#N/A` while `LOOKUP(2,A1:B2,{10})` = 10.
- A value not found is `#N/A` before the result vector is looked at.
Edges (probes/lookup-vector-edges, excel-graph, 2026-10-07, T-0269):
- A single-cell lookup_vector is a one-element vector: `LOOKUP(2,C1,D1)` and `LOOKUP(3,C1,D1)` with C1 = 2 are D1, `LOOKUP(0,C1,D1)` is `#N/A`.
- A scalar result_vector is a one-element array: `LOOKUP(2,A1:A2,5)` is 5 (match at position 1) and `LOOKUP(99,A1:A2,5)` (position 2) is `#N/A`,
  as is `"x"`; unlike a one-cell range it is not read past.
- In both the array and vector forms, any scalar lookup_vector (direct, array constant `{1}`, or picked by IF or CHOOSE) is a one-element array (`LOOKUP(2,1)` is 1, `LOOKUP("b","a")` is "a": probes/mode-direct-error-lookup-scalar-vector B1-B5, T-1651); booleans too (`LOOKUP(TRUE,TRUE)` is TRUE), and an empty lookup_vector (a blank cell or `IF(TRUE,Z99)`) gives #N/A (probes/lookup-bool-empty-scalar-vector, T-1709; the probe cannot tell a one-element blank array from an empty value that never matches, and `IF(TRUE,Z99)` may be a reference); e.g. legacy `=LOOKUP(2,CHOOSE(1,B1:B6*1))` is 1 on the range's rows (references.md "Legacy `@` on CHOOSE operator arms under aggregates", T-1643).
- Reading past the sheet's last row or column gives a blank, 0, not `#N/A`: `LOOKUP(99,A1:A3,F1048575:F1048576)` and
  `LOOKUP(99,A5:B5,XFD5)` are 0, while a position still on the sheet reads its cell (`LOOKUP(3,A1:A3,F1048575:F1048576)` is F1048576's 12,
  `LOOKUP(99,A5:B5,XFC5:XFD5)` is XFD5).
- `OFFSET(F1,0,0)` as result_vector reads past its end like a plain `F1`, and so does a reference-returning `INDEX(F1:F1,1)`
  or `INDEX(F1:G1,1,1)` (G1 / 8); `lookup_result` keeps INDEX as a reference (T-0274).
Evidence: probes/lookup-2d-vector H1:H13, H19, probes/lookup-2d-vector-wide L1:L13 (excel-graph, 2026-10-07, T-0224), probes/lookup-vector-edges A1:A21 (T-0269).
Status: VERIFIED
Implemented in: `fn_lookup`, `lookup_result` in `crates/rustsheet-engine/src/functions/lookup_and_reference/lookup.rs`
Tasks: T-0224, T-0269

## VLOOKUP / HLOOKUP lift over an array lookup_value
Rule: an array or multi-cell range lookup_value is looked up element by element into a same-shaped
spilled array (`VLOOKUP({2;3},A1:B4,2,FALSE)` → "two"; "three", a row lifts across, `INDEX(A1:B2,1,0)`
too). Each element follows the scalar rules, searching before validating the index: with index 0,
`{2;99}` gives `#VALUE!`, `#N/A`; with an index past the table `#REF!`, `#N/A`. An error element is
itself (`{2;#DIV/0!;"x"}` → "two", `#DIV/0!`, `#N/A`). Approximate match lifts the same way. A whole
column lifts too (`ROWS(VLOOKUP(D:D,A1:B4,2,FALSE))` is 1048576) and the engine bounds it by the
shared cell-visit cap (criteria.md): elements times the evaluated cells of the searched column or row,
whatever the table is (DEVIATION, decision 0014: `VLOOKUP(SEQUENCE(1048576),SEQUENCE(1048576,2),2,FALSE)`
is `#ERROR!`).
A blank element of a multi-cell lookup_value is looked up exactly as the scalar blank-cell key: as the number 0, in every
form. That holds for the blank middle cell of a range (`VLOOKUP(Keys!A1:A3,..)`, `HLOOKUP(Keys!C1:E1,..)`,
`MATCH(Keys!A1:A3,..)`), for the blank element of an array (`IF({1;0;1},Keys!A1:A3,Keys!G1)`), and for a whole-column
lookup_value read inside the used area and far below it (`INDEX(VLOOKUP(Keys!A:A,..),1000)`). Against tables holding a 0
row, a `=""` row, a FALSE row or only a blank row among numbers, the blank element finds the 0 row and nothing else in
exact mode (`#N/A` against the `""`, FALSE and blank tables), lands on the 0 row where there is one and otherwise on
the last number below 0 (the -1 row) in the approximate VLOOKUP/HLOOKUP TRUE and `MATCH(..,1)`, and takes the smallest number at or above 0 in `MATCH(..,-1)`. It is
never `""` or FALSE, and it never matches a blank row. The non-blank elements of the same lookup_value are unaffected.
Evidence for the blank elements: probes/t3096-lookup-value-blank-elements (200 Calc cells, plus the 3 `=""` table
cells on Data that make the baseline's 203; excel-graph, 2026-10-29, T-3096; Calc rows 16-22 the range middle, 37-43
the array element, 44-51 the whole column, each identical to the scalar control rows 2-8 across the four tables in
columns B-E). The engine matches every cell: `lift_values` hands a blank element to `lookup_index` as the blank key.
Evidence: probes/vlookup-array-lookup-value (35 cells, excel-graph, 2026-10-09, T-0270).
Implemented in: `Model::lookup_each` (functions/lookup_and_reference/lookup_each.rs) through
`Model::lift_values` (functions/lift/driver.rs); the spill shape follows lookup_value in static analysis.
Status: VERIFIED (web)

## VLOOKUP / HLOOKUP with an array index or range_lookup
Rule: lookup_value and range_lookup lift together, broadcast like any two arrays (a 3x1 lookup_value
against a 2x1 range_lookup gives `#N/A` in the third row; a column against a row gives 2-D). Each pair
is a scalar call, so an error or text range_lookup element is that element's result
(`VLOOKUP({2;3},A1:B4,2,{#DIV/0!;TRUE})` is `#DIV/0!`, "three"; `{"a";"b"}` is `#VALUE!` twice).
A scalar text range_lookup beside a lifted lookup_value lifts the same way, one array of that error rather than one `#VALUE!`:
`WRAPROWS(Data!E6:E11,1,VLOOKUP(OFFSET(Data!D6:E8,2,0),Data!B5:C5,3,"1"))` spills a 3x2 grid of E6's 3.14159, so the VLOOKUP there is a 3x2
array that WRAPROWS's pad_with lifts over (gen-m7/f7e94a817e5385ac Calc!A2430, excel-graph, 2026-10-22). Every element is
the scalar call's result, read through INDEX and ERROR.TYPE (probes/vlookup-scalar-range-lookup-lifted, excel-graph,
2026-10-23, T-2411):
- `VLOOKUP({1;2},A1:B2,2,"1")` is `#VALUE!` twice (G1-G4). A bad range_lookup fails before any search.
- An error range_lookup is that error per element: `NA()` gives `#N/A` and `1/0` gives `#DIV/0!` (G7, G29).
- An error lookup_value element keeps its own error (`{1;#DIV/0!}`).
- A range lookup_value lifts the same way, and so does HLOOKUP (`HLOOKUP({1,2},D1:E2,2,"x")`, 2 columns of `#VALUE!`).
- A value index that fails (text `"a"` or `1/0`) beside a lifted lookup_value lifts too. Its error is every element's
  (except an error lookup_value element, which keeps its own, T-2456 H2), ahead of a bad range_lookup and of the search:
  - `VLOOKUP({1;2},A1:B2,"a",NA())` is `#VALUE!` per element (G20).
  - `VLOOKUP({1;99},A1:B2,1/0,"1")` is `#DIV/0!` per element (G24).
  - Under exact match 99 is not found, and the index error still wins: `VLOOKUP({1;99},A1:B2,"a",FALSE)` is `#VALUE!` and
    `VLOOKUP({1;99},A1:B2,1/0,FALSE)` is `#DIV/0!` in the 99 element (G33, G39). The control
    `VLOOKUP({1;99},A1:B2,2,{FALSE;FALSE})` is `#N/A` there (G36).
  - The same holds beside an array range_lookup: `VLOOKUP({1;2},A1:B2,"a",{#N/A;TRUE})` is `#VALUE!` in every element
    (G26), and so is `VLOOKUP({1;99},A1:B2,1/0,{FALSE;FALSE})` with `#DIV/0!` (G35).
  - (With approximate match, 99 is found in the last row of A1:A2, so `{1;99}` cells G21, G25, G30 do not test not-found.)
- An index of 0 is still checked after the search: `VLOOKUP({1;99},A1:B2,0,FALSE)` is `#N/A` in the 99 element (G37).
  Beside a bad range_lookup, every element is that range_lookup's `#VALUE!` (`VLOOKUP({1;99},A1:B2,0,"1")`, G28).
- Text range_lookup that reads as a boolean works: `VLOOKUP({1;2},A1:B2,2,"TRUE")` is "one", "two" (G18, G19).
- The scalar `VLOOKUP(1,A1:B2,2,"1")` stays one `#VALUE!`.
Each element of those claims is probed (probes/vlookup-t2411-elements, excel-graph, 2026-10-25, T-2456):
`VLOOKUP({1;#N/A},A1:B2,"a")` is `#VALUE!` in the first element and `#N/A` in the second (H1, H2: the failed index
does not hide an error lookup_value element); `HLOOKUP({1,2},D1:E2,2,"x")` is `#VALUE!` in both columns (H3, H4); and
`VLOOKUP({1;2},A1:B2,2,"TRUE")` is "one", "two" (H5, H6).
Ours: matches since T-2411.
The index array lifts only when lookup_value and range_lookup are both single values: then a scalar
search runs, not found is `#N/A` (before any index is looked at), and otherwise every index element is
validated in reading order and the first bad one is the error of the whole call, with no spill
(`{0,9}` is `#VALUE!`, `{9,0}` is `#REF!`, `{2,"a"}` and `{2,#DIV/0!}` are `#REF!`, an element is read like
a referenced cell, so text and errors are `#REF!`); valid indexes give a same-shaped array
(`VLOOKUP(2,A1:B4,{2,1},FALSE)` is "two", 2). A fractional element is floored with the near-integer snap, as a scalar
index is (`{2.9999999,1.999999}` reads columns 3 and 1, `{3.9999999,1.9999999}` is `#REF!` on a 3-column table;
probes/lookup-argument-edges H366-H370, T-2234). When lookup_value or
range_lookup lifts, the index uses only its first element (`VLOOKUP({2;3},A1:B4,{1;2},FALSE)` is 2, 3,
`{2;"a"}` is "two", "three", `{#DIV/0!;2}` is `#REF!` twice, a 2-D index likewise).
A multi-cell *range* range_lookup lifts like an array and spills (gen-m2/942bdade381b7537 Calc!A381,
d3e2df0bef2f173a Calc!A20, f7e6d351c8538ef5 Calc!A296: `#SPILL!` on a collision;
04cfd82f54dbc2b2 Calc!A19: a 1x4 range whose first element is `#N/A`; excel-graph, T-0203, T-1468).
An error lookup_value or index does not stop that lift either: a `#VALUE!` lookup_value (the inner
HLOOKUP's `E12-A6` in f7e6d351c8538ef5 Calc!A296) or index (`E$2/0.1` in 942bdade381b7537 Calc!A381)
with a lifting range_lookup spills, giving `#SPILL!` on a collision (T-1468). The element values of
such a spill are the error itself, per row, with room to spill: an error lookup_value, an error
index (`1/0`, `NA()`) and a text index (`"a"` is `#VALUE!`, not the `#REF!` of an array index element)
each fill every row with that scalar error (VLOOKUP with a range or an array range_lookup; HLOOKUP and
`NA()` probed with an array range_lookup only, the engine shares one path) (probes/vlookup-error-arg-lifting-range-lookup, 22 cells, excel-graph, 2026-10-13, T-1491).
The index error is every element's even where the value is not found: `VLOOKUP(99,A1:B2,"a",{FALSE;FALSE})` is `#VALUE!`
per element (probes/vlookup-scalar-range-lookup-lifted G34), and it beats an error range_lookup element
(`VLOOKUP(99,A1:B2,"a",{TRUE;#N/A})` is `#VALUE!` in the second, G32). Ours: matches since T-2411 (`argument_error_each`);
before it, a not-found element was `#N/A`.
A multi-cell range *index* lifts once the value is found, and `@` intersects it (T-1426; see "VLOOKUP / HLOOKUP search
before validating the index"). With a table error, an array index gives one scalar error, not a spill
(`VLOOKUP(2,1/0,{2,3})` is `#DIV/0!`, probes/lookup-argument-edges H372, H375).
When a lookup_value element and a range_lookup element are both errors, the lookup_value's error wins:
`VLOOKUP({#N/A;2},A1:C3,2,{#DIV/0!;TRUE})` spills `#N/A`, "y", and `VLOOKUP({2;#N/A},A1:C3,2,{#DIV/0!;#NUM!})`
spills `#DIV/0!`, `#N/A`. The same holds unlifted: `VLOOKUP(NA(),A1:C3,2,1/0)` is `#N/A`, and `VLOOKUP(1/0,A1:C3,2,NA())`
is `#DIV/0!`. A range_lookup error still beats a value that is not found: `VLOOKUP(9,A1:C3,2,1/0)` is `#DIV/0!`.
HLOOKUP behaves alike (probes/lookup-argument-edges H30-H63, excel-graph, 2026-10-21, T-2234).
A blank element of a computed index array is `#VALUE!` for the whole call, not a spill:
`VLOOKUP(2,A1:C3,IF({1,0},2,Z1))` and `CHOOSE({1,2},2,Z1)` with Z1 blank, and `INDEX(E10:E11,{1;2})` with E11 blank. The
same holds for `{2,1}*Z1`, though that is just an index below 1. A text element (`IF({1,0},2,"")`) is `#REF!`. Not found is
`#N/A` in each case. A blank reference index (`Z1`, `+Z1`) is `#VALUE!`. A lifted lookup_value with a range index reads
only the range's first cell: `VLOOKUP({2;2},A1:C3,E10:E11)` (E11 blank) and `VLOOKUP({2;2},A1:C3,E14:E15)` (2, 3) both
spill "y" twice (same probe, H126-H189, H360-H364).
Evidence: probes/vlookup-array-index-range-lookup (155 cells, excel-graph, 2026-10-12, T-1269); the
first-fill reading that an array range_lookup "looks unlifted" (T-0270) does not hold: each probe here has its own rows, so no spill collides.
Implemented in: `Model::lookup_each` and `Model::pick_each` (lookup_each.rs), `LookupIndexArg::Array`
(index_arg.rs); static analysis broadcasts lookup_value, index and range_lookup.
Status: VERIFIED (web)
Tasks: T-1269

## VLOOKUP / HLOOKUP search before validating the index
Rule: the value is looked up first. Not found is `#N/A` whatever the column/row index
(`=VLOOKUP("a",Data!E2:E6,3)` → `#N/A`; indexes -1, 0, -2.5 and past the table too). Only a found
value goes on to index validation:
- An index below 1 after truncation (-1, 0, 0.5, a blank cell) is `#VALUE!`; one past the table, or
  below the 32-bit integer range (-1E+15, -2147483649), is `#REF!`.
- A *reference* index is read only then: a referenced cell holding non-numeric text or an error is
  `#REF!` (not `#VALUE!`, not the error itself); numeric text ("1.5") and booleans coerce.
- A *value* index (a literal or a formula result) is coerced before the search, so literal text
  (`"a"`, `""`) is `#VALUE!` and an error (`1/0`, `NA()`) is that error even when nothing is found.
  Arguments fail left to right: lookup_value, then the index, then range_lookup (`VLOOKUP(2,A1:C3,"a",1/0)`
  is `#VALUE!`).
  A value index's error also beats a not-found lookup and a range_lookup error: `VLOOKUP(FALSE,C7:D9,WEEKDAY(31330,-0.0001),INDEX(B1:E7,4))`
  (FALSE not in the table, range_lookup `#REF!`) is `#NUM!` (gen-m6/5cc04d6f42b23b51 Calc!A27, excel-graph, 2026-10-09, T-1239).
Evidence: probes/lookup-index-order H1–H50 (excel-graph, 2026-10-06, T-0211); gen-m2/1b6ceed7489e0800 A24,
2cfe279fe77a000d A85, 61cacc22c4244179 A310, caa981b0753f455c A182, 629e6e8b76fab8b9 A53,
cbdabe32a32a4012 A87 (Calc sheet; excel-graph, 2026-10-04, gen-m2 sidecars, T-0203).
Cut-off (probes/lookup-index-shapes H19-H36, excel-graph, 2026-10-06, T-0250): an index of
-2147483648 is `#REF!`, -2147483647, -65536, -32769, -32768 and -32767 are `#VALUE!`, in VLOOKUP and
HLOOKUP alike; not found is `#N/A` for all of them. A fractional index is floored before that test, so
-2147483647.5, -2147483647.9999999 and -2147483648.25 are `#REF!`, while -2147483646.5 and -0.5 are `#VALUE!`. An
array element floors the same way: `{-2147483647.5;-2147483648.5}` is `#REF!`. 2147483647.5 and 2147483648.5 are
`#REF!` (past the table), and 3.5 is column 3 (probes/lookup-argument-edges H192-H333, excel-graph, 2026-10-21,
T-2234).
Near an integer the index snaps up, with the absolute bound that INDEX, LEFT and RIGHT use (2^-22 + 2^-33,
"INDEX rounds an index up near an integer"). B1:CW1 holds 1 to 100, so `VLOOKUP(1,B1:CW1,x)` is the index used:
- `2-2^-22*(1+2*2^-12)` is 2, but `2-2^-22*(1+3*2^-12)` is 1;
- the same at 3, 50 and 100, and with HLOOKUP;
- 2.9999999 is 3, and 3.9999999 is past a 3-column table (`#REF!`).

Evidence: probes/lookup-index-near-integer (excel-graph, 2026-10-21, T-2234), for a literal and a reference index
alike.
Index shapes (same probe): `OFFSET(E1,..)` and `INDIRECT("E1")` results are references (a cell holding
text or an error is `#REF!` when found, `#N/A` when not found). So is `INDEX(E1:E4,2)`: text or an
error in the INDEXed cell is `#REF!` found / `#N/A` not found (probes H2, H3, H11, H12; T-0251).
A multi-cell range index is implicitly intersected: `VLOOKUP(2,A1:C3,@E1:E2)` from row 1 reads E1,
from row 2 reads E2 (text, so `#REF!`), from row 5 is `#VALUE!` (found), and also `#VALUE!` when the
value is not found (value index fails before the search); `VLOOKUP(2,A1:C3,E1:E2)` unspilled in J10 is
`#REF!` and `#N/A` when not found (K7). Entered as a dynamic-array formula, it spills instead.
`VLOOKUP(2,A1:C3,E1:E2)` with E1:E2 = 2, 3 spills "y" and 20 from J10, from N1 and from P2 (rows inside
E1:E2 too), and is `#SPILL!` when the cell below is filled. `ROWS(..)` is 2, and `@E1:E2` from row 14 is `#VALUE!`.
HLOOKUP behaves alike (probes/lookup-argument-edges J10-X2, excel-graph, 2026-10-21, T-2234).
HLOOKUP twins (probes/lookup-index-hlookup H1-H6, excel-graph, 2026-10-07, T-0264): `INDEX(E1:E4,1)` (number)
gives the row, `INDEX(E1:E4,2)` (text) and `INDEX(E1:E4,3)` (error) are `#REF!` found and `#N/A` not found,
as in VLOOKUP. An INDEX index that itself fails (`INDEX(E1:E4,9)`) is `#REF!` in HLOOKUP and VLOOKUP, found
or not found (H7-H10): a failing index is not turned into `#N/A` by a failed search. Whether the INDEX error propagates as is or is always
mapped to `#REF!` is unprobed (an INDEX failing with another error, e.g. `INDEX(E1:E4,"x")`, would tell).
A table_array error comes after lookup_value and before the index and range_lookup. `VLOOKUP(NA(),1/0,2)` and
`VLOOKUP(NA(),#REF!,2)` are `#N/A`. `VLOOKUP(2,1/0,NA())`, `VLOOKUP(2,1/0,2,NA())` and `VLOOKUP(9,1/0,2)` are `#DIV/0!`.
`VLOOKUP(2,#REF!,"a")` is `#REF!`. A lifted lookup_value spills the table error once per element: `VLOOKUP({2;9},1/0,2)`
gives two `#DIV/0!` (probes/lookup-argument-edges H66-H123, excel-graph, 2026-10-21, T-2234).
A lifted range_lookup spreads the table error too, as `VLOOKUP(2,1/0,2,{TRUE;FALSE})` spilling `#DIV/0!` twice shows.
A lookup_value error still comes first, element by element: `VLOOKUP(NA(),1/0,2,{TRUE;FALSE})` spills `#N/A` twice, and
`VLOOKUP({#N/A;2},1/0,2)` spills `#N/A`, `#DIV/0!` (same probe, H336-H358, HLOOKUP alike).
A table that is the engine's own `#ERROR!` (arrays.md "Array results above 1,048,576 cells are #ERROR!", a DEVIATION)
is returned once, not spread over a lifted lookup_value, so the limit still refuses the call.
Status: VERIFIED
Implemented in: `Model::lookup_index_arg` / `Model::resolve_lookup_index`
(crates/rustsheet-engine/src/functions/lookup_and_reference/index_arg.rs), shared by VLOOKUP and HLOOKUP.
Tasks: T-0211, T-0264, T-2234

## Approximate-match lookups: type-skipping binary search
Rule: VLOOKUP/HLOOKUP with range_lookup TRUE (or omitted), LOOKUP, and MATCH with a positive
match_type run one binary search over the vector that only compares cells of the lookup value's
own type (number, text or boolean; a blank lookup value is the number 0):
```
lo = 0; hi = n-1; best = none
while lo <= hi:
    mid = floor((lo+hi)/2)
    m = first index >= mid with the lookup value's type   # skips blanks, errors, other types
    if m > hi: hi = mid-1; continue
    c = compare(v[m], key)                                  # text case-insensitive
    if c == 0: return the last index of the run of adjacent equal cells starting at m
    if c < 0: best = m; lo = m+1
    else: hi = m-1
return best (none -> #N/A)
```
Blanks never match and there is no cross-type match (`VLOOKUP(TRUE,{blank;-1},1,TRUE)` is `#N/A`).
The run of equal cells stops at the first cell that is not equal, whatever its type
(`MATCH(2,{0,0,0,2,"x",2,9})` = 4). A negative match_type is no binary search at all: see "MATCH descending
search scans from the top" below. Any negative match_type searches descending and any positive one
ascending (`MATCH(5,{9,5,5,5,1},-2)` = 2, `MATCH(2,…,2)` = `MATCH(2,…,1)`). Only the sign of a fractional match_type counts: `-0.5` and `-0.0001` search descending (gen-m2/f7e6d351c8538ef5 Calc!A160, gen-m2/96a0952412cefa6c Calc!A241, T-1304) and `0.5`, `0.0001` and `0.999` search ascending, not exact (`MATCH(2.5,{1,2,3,4,5},0.5)` = 2; probes/match-fractional-match-type C1-C3, T-1391). A blank MATCH key is 0
in an approximate search (`MATCH(blank,{1,blank,0,blank,2})` = 3).
Exact match (range_lookup FALSE, MATCH 0) only matches cells of the key's type: a blank cell never
matches 0 or "" (`MATCH(0,{blank;blank;blank},0)` and `VLOOKUP("",{blank;blank;blank},1,FALSE)` are
`#N/A`), while a blank lookup value is 0 here too (`MATCH(blank,{1,blank,0,blank,2},0)` = 3,
`MATCH(blank,{"";0},0)` = 2). Numbers must be exactly equal, without the 15-digit rounding of the
comparison operators: `MATCH(0.1+0.2,{0.3},0)` and `VLOOKUP(0.1+0.2,{0.3},1,FALSE)` are `#N/A`.
A match_type that is not a number is `#VALUE!` even with a blank key (`MATCH(blank,…,"x")`).
Text comparison uses Excel's collation, not code-point order (T-0220).
The approximate searches compare numbers as exact doubles, not at 15 digits like the comparison operators (T-3073,
probes/t3073-lookup-near-equal-approximate, excel-graph 2026-10-29). A table number a few ULP from the key is larger
or smaller than it, never equal, so the search steps past it. In VLOOKUP and HLOOKUP (TRUE) and in `MATCH(key,…,1)`
over the unsorted {2; X; 0.5}, key 1 gives `#N/A` for X = 1+k·2^-52 and 3 for X = 1-k·2^-53 (k = 1, 2, 3, 8;
Sheet1 A1-C8). A 15-digit compare would give 2 for every X that rounds to 1 at 15 digits: all of A1-C7. 1-8·2^-53
reads 0.999999999999999, so row 8 fits both. `MATCH(0.3,{2;0.1+0.2;0.5},1)` is `#N/A` (A17-C17). The keys 1+k·2^-52
and 1-k·2^-53 against a 1 in the table give the mirror answers (A9-C16; row 16 fits both). Rows 18-20 fit both
models: 0.1+0.2 against 0.3 is decided by the 0.5, and the pair either side of 1+5E-15 reads differently at 15
digits too. The run of equal cells is a run of exactly equal
doubles. Over 1, 1+2^-52, 1+2·2^-52, 2 the keys 1, 1+2^-52 and 1+2·2^-52 give 1, 2 and 3, where a 15-digit run would
give 3 for all three (A30-C31; A32-C32 fit both). VLOOKUP, HLOOKUP and MATCH agree on every cell. The engine still compares at 15 digits
there (`compare_values`): T-3099.
Evidence: eval/suites/probes/cases/lookup-approximate-search.case.json N1–N44 (oracle: excel-graph,
2026-10-06, T-0212); gen-m2/fa437879253827be A146 (`=HLOOKUP(-2.5,Data!B3:E7,4)` → 2), cbdabe32a32a4012 A87,
a6d4377c7b1b92cd A33 (`MATCH(…,-1)` → 1), 04cfd82f54dbc2b2 A1 and d0c29dd5c530e28c A67 (`#N/A`)
(excel-graph, 2026-10-04, T-0203); 419 gen-m2 cells fixed by T-0212;
crates/rustsheet-xlsx/tests/calc_tests/LOOKUP_AND_REFERENCE/MATCH.xlsx C7 (duplicate run, Excel-cached).
Status: VERIFIED
Implemented in: `SearchVector::approximate_search` and `SearchVector::exact_position` in
`crates/rustsheet-engine/src/functions/lookup_and_reference/lookup_table.rs`
Tasks: T-0212, T-0220, T-3073, T-3099

## MATCH descending search scans from the top; a text match_type read from a cell is #REF!
Rule 1: a descending search (negative match_type) is a linear scan from the first cell, not a binary
search. It passes over blanks, errors and cells of another type, ends at the first cell equal to the key
(the first of a run of equal cells), and otherwise stops at the first cell of the key's type that is
below the key, giving the last cell before it that is above the key (`#N/A` if the first one is already
below, or there is none). Over the unsorted 0.5, 0.3, 0, -2.5, 2.5, -1, 1, 0.1, 100, keys 0, a blank
(0) and -0.0001 give 3 and 0.0001 gives 2, over the whole vector and its 5- to 8-cell prefixes alike, and
`""` is `#N/A` (probes/lookup-match-descending-blank A1-A18, excel-graph, 2026-10-21, T-2278; a binary search
gives 9). The scan also gives every earlier cell this section was first written for, which a backward
skip from a binary-search midpoint was fitted to: `MATCH(0.5,{0.1,TRUE,3.14159},-1)` is `#N/A` (0.1 is
below the key), `MATCH(0.5,{3,TRUE,0.1},-1)` = 1, `MATCH(2,{5,"x",3,1},-1)` = 3, `MATCH(2,{5,TRUE,TRUE,1},-1)` = 1
(the range forms agree) and `MATCH(-14,{FALSE,0.5},-1)` = 2 (gen-m2 b4392a6514a36ade Calc!A277). So do the 20
cells of probes/match-descending-forward-continuation J1-J20 (excel-graph, T-1396), a prefix of text, boolean or
error cells before a run of numbers (or text): `MATCH(3,{"a","b","c",5,3,1},-1)` = 5,
`MATCH(1,{TRUE,FALSE,"c",9,7,5,3,1},-1)` = 8, `MATCH("b",{1,TRUE,2,"c","b","a"},-1)` = 5, `{#N/A,#N/A,5,4,3,2}` and
prefixes of 2-4 text, boolean or mixed cells (J11-J14) give the same positions as the typed tail alone; an inexact key picks the last cell >= key
(`MATCH(4.5,{"a","b","c",9,8,7,6,5,4,3},-1)` = 8) and a key above everything is #N/A (`MATCH(1,{"a","b",0.5,0.2},-1)`);
range forms (a column and a row) agree, as does `MATCH(5,{9,5,5,5,1},-2)` = 2 above. Replacing the backward-skip
binary search with the scan changed no other cell of any suite (T-2278). A key-equal cell after a smaller one is never
reached (`MATCH(5,{1,9,5},-1)` `#N/A`, a binary search gives 3; with 5, 9, 5 in B1:B3 `MATCH(5,B1:B3,-1)` = 1), and a 100-cell
unsorted vector gives the scan's answers too (no switch of algorithm by size): probes/lookup-t2278-followups A8-A14
(excel-graph, 2026-10-22).
The scan compares numbers as exact doubles, as the ascending search does (T-3073, probes/t3073-lookup-near-equal-approximate,
excel-graph 2026-10-29):
- `MATCH(key,{X;key;0.5},-1)` is 2 where X is a few ULP above the key, because X is above it and the key cell is the
  first equal one. It is `#N/A` where X is a few ULP below, because the first cell is already below (Sheet1 D1-D20). A
  15-digit compare would give 1 on D1-D7, D9-D15, D17 and D18. D8, D16, D19 and D20 fit both models.
- Over 2, 1+2·2^-52, 1+2^-52, 1, 0.5 the keys 1, 1+2^-52 and 1+2·2^-52 give 4, 3 and 2 (D30-D32; D32 fits both).
The engine compares at 15 digits there (T-3099).
Rule 2: a literal text match_type is `#VALUE!` (`MATCH(1,A5:A7,"x")`, and also on a 2-D range:
`MATCH(1,A5:B7,"x")`), but a text match_type that comes from a *cell* is `#REF!`, for a 1-D and a 2-D
lookup_array alike (`MATCH(1,A5:A7,D1)` with D1 = "a", `MATCH(1,A5:B7,D1)`, and D2 = " x "). The 2-D
`#N/A` rule above still holds for numeric match_type (`MATCH(1,A5:B7,0)` and `,1` are `#N/A`).
`INDEX` returns that `#REF!` (`INDEX(A5:B7,MATCH(1,A5:A6,D1),3)` and `,1`), while a literal-text
match_type inside it is `#VALUE!` and `INDEX(A5:B7,NA(),3)` is `#N/A` (the row error wins over the
out-of-range column, which alone is `#REF!`).
An error column beats a row index below 1 as well: `INDEX(Data!B2:E6,TAN(-0.5),MATCH("1",Data!B5:C8,1))` (row -0.546, column
`#N/A` from MATCH over a 2-D range) is `#N/A` (VERIFIED, gen-m5/372a676d47aa1427 Calc!A371 inside `MMULT(Data!A1,..)`, excel-graph, 2026-10-22, T-2252 triage).
Implemented (T-2375) in `index_numbers` (functions/lookup_and_reference/index.rs): the cast errors of row and column come
before a negative row or column; `INDEX(range,-0.5,1)` stays `#VALUE!`.
Rule 3: the cell stays a reference through IF, IFS, CHOOSE, SWITCH and LET, so `MATCH(2,A5:A7,IFS(TRUE,D1))`
with D1 = "x" is `#REF!` too (as are the same with IF, CHOOSE, SWITCH, `LET(a,D1,a)`); D2 = 1
gives 2. IFERROR returns the value, so `IFERROR(D1,0)` is `#VALUE!`, like literal text
(probes/match-type-text-passthrough K1:K9, excel-graph, T-1399).
Nesting of IF, IFS and SWITCH inside each other changes nothing (IFS or SWITCH inside CHOOSE or a LET body is unprobed): `IFS(TRUE,IF(TRUE,D1))`, `IF(TRUE,IFS(TRUE,D1))`, `IFS(TRUE,SWITCH(1,1,D1))` and the SWITCH default branch (`SWITCH(9,1,D2,D1)`) are `#REF!` for D1 = "x" (D2 = 1 gives 2).
A multi-cell range is its top-left cell, plain or picked by IF, IFS, SWITCH, LET or CHOOSE (no implicit intersection): with D1 = "x", D2 = 1, D3 = "y", `MATCH(2,A5:A7,D1:D2)` is `#REF!` and `D2:D3` gives 2 (the IF-picked forms K2 and K3, in rows 2 and 3, rule out implicit intersection). A one-sheet 3-D span (`Sheet1:Sheet1!D1`) behaves as the cell
(probes/match-type-nested-passthrough K1:K17, excel-graph, T-1452).
Evidence: eval/suites/probes/cases/match-descending-argcheck.case.json J1–J20 (oracle: excel-graph,
2026-10-10, T-1305); gen-m2/f7e6d351c8538ef5 Calc!A334, b4392a6514a36ade Calc!A277, 04cfd82f54dbc2b2 Calc!A81, f9d4df4efdfbcbd5 Calc!A14.
Implemented in: Rule 1 `SearchVector::descending_search` (lookup_table.rs); `fn_match` / `match_type_arg` in `crates/rustsheet-engine/src/functions/lookup_and_reference/mod.rs` (Rule 2: non-numeric text from a cell only; numeric text and booleans in the cell are unprobed). Rule 3: `match_type_arg` evaluates through `evaluate_chosen_reference` (util.rs), which IF, IFS, SWITCH, LET and CHOOSE share; aggregates such as SUM are deliberately unchanged (no oracle evidence).
Status: VERIFIED
Tasks: T-1305, T-1312, T-1313, T-1399, T-1452, T-2278

## INDEX with one index into a 2-D range is #REF!
Rule: `INDEX(range2d, n)` with only a row index (including 0 and 1E-15) is `#REF!`:
`=INDEX(Data!A1:B2,0)`, `=INDEX(Data!A1:D8,1e-15)` (85 of 90 calls; the rest are argument errors).
`INDEX(column,0)` on a single column spills (`#SPILL!` when blocked).
An array (constant or array-valued expression) is different: one index selects the whole row,
`INDEX({1,2;3,4},1)` spills `{1,2}`, `SUM(INDEX(A1:B2*1,2))` = 7, `SUM(INDEX({1,2;3,4},0))` = 10, and
an index past the last row is `#REF!`. So `INDEX(A1:B2,2)` is `#REF!` but `INDEX(A1:B2*1,2)` is `{3,4}`.
Evidence: gen-m2/60a543744fc4a8f7 Calc!A106, 893302d6c5b99e2e Calc!A265, 78bc88199fc96650 Calc!A61. Oracle: excel-graph, 2026-10-04 (gen-m2 sidecars, T-0203).
Arrays: probes/index-single-index D1–D6 (excel-graph, 2026-10-05).
Status: VERIFIED
Implemented in: `fn_index` in `crates/rustsheet-engine/src/functions/lookup_and_reference/mod.rs`
Tasks: T-0213

## VLOOKUP and HLOOKUP hand a blank result cell to their caller as 0 (T-2168)
Rule: when VLOOKUP, HLOOKUP or LOOKUP lands on a blank cell, it gives the blank's *value*, the same empty value IFERROR
gives for a blank (logic.md "IFERROR / IFNA of a blank cell is an empty value"), not a reference to it. The cell shows 0;
ISBLANK of it is TRUE, ISNUMBER and ISTEXT FALSE, ISREF FALSE, TYPE 1, N 0, T "", LEN 0, `&""` "", ASC "", SUM, MAX and MIN
read 0, and `LET(x,..,ISBLANK(x))` is TRUE. Unlike a blank reference, COUNT, COUNTA and AVERAGE count it as the number 0:
`COUNT(VLOOKUP(..))` and `COUNTA(VLOOKUP(..))` are 1, `AVERAGE(VLOOKUP(..),4)` is 2, through IF and LET too
(`COUNT(IF(TRUE,VLOOKUP(..)))` 1, `LET(x,VLOOKUP(..),COUNT(x))` 1). Exact and approximate match, both LOOKUP forms
(`LOOKUP(2,A1:A3,B1:B3)`, `LOOKUP(2,A1:B3)`) behave the same. IFS and SWITCH turn the empty value into 0:
`IFS(TRUE,VLOOKUP(..))&""`, `IFS(FALSE,1,TRUE,VLOOKUP(..))&""`, `SWITCH(1,1,VLOOKUP(..))&""` and
`IFS(TRUE,IFERROR(B2,1))&""` are "0" and `ISBLANK(IFS(TRUE,VLOOKUP(..)))` FALSE, while `IFS(TRUE,B2)&""` of a blank
reference, `IF(TRUE,VLOOKUP(..))&""`, `IF(TRUE,IFERROR(B2,1))&""` and `CHOOSE(1,VLOOKUP(..))&""` stay "". INDEX and
INDEX/MATCH return the reference, so a blank there is skipped (`COUNT(INDEX(B1:B3,2))` 0, `AVERAGE(..,4)` 4, `MAX(..,-1)`
-1, ISREF TRUE). An array result keeps the blank element empty: `COUNT(VLOOKUP({2,3},..))` is 1, `AVERAGE` of it 5,
`AVERAGE(VLOOKUP(2,..,{2},FALSE),4)` 4, `COUNT(HLOOKUP(2,D1:F2,{2}))` 0.
So `ASC(IFS(..,VLOOKUP(123456789.123,D3:E7,2),..))` landing on blank E6 is "0" (through IFS), and
`SEC(AVERAGE(HLOOKUP(2.5,B8:D11,4),-2.5,B7:B9))` averages the blank result as 0, 1.2724471433871758 = SEC(-2/3).
XLOOKUP already returns a blank return cell as 0 (below). An omitted value argument is the same empty value
(probes/t2189-*, excel-graph, T-2189, each formula its own case): `IFS(TRUE,)&""` and `SWITCH(1,1,)&""` are "0",
`ISBLANK(IFS(TRUE,))` is FALSE and `COUNT(SWITCH(1,1,))` is 1; the engine agrees on all four. (An earlier attempt to put
`IFS(TRUE,)` in a larger workbook got a createSession 501; it is not refused alone.) Excel for the web refuses a workbook holding
one of `AVERAGEA`, `PRODUCT`, `COUNTBLANK`, `SUMSQ`, `STDEV.P` over
`VLOOKUP(..)` (most likely COUNTBLANK, which takes only a reference); those are unprobed.
Evidence: gen-m5/49bfbf67a1409563 Calc!A103, gen-m5/aac6230c2fb40e1e Calc!A334 (excel-graph, 2026-10-20);
probes/lookup-blank-result-to-caller H1:O20 and probes/lookup-vector-blank-result-to-caller H1:I19 (excel-graph,
2026-10-20, T-2168).
Ours: matches since T-2168 (`CalcResult::blank_as_empty_value` on the scalar result of VLOOKUP, HLOOKUP and LOOKUP;
`empty_value_as_zero` on what IFS and SWITCH pick).
Status: VERIFIED (excel-graph)

## Whole-column / whole-row search vectors: blank tail
Rule: as above, blanks never match and the type-skipping search steps over them, so a whole
column with data in A1:A3 behaves like A1:A3 for an approximate lookup (`MATCH(99,A:A,1)` is
3, not 1048576).
Evidence: probes/lookup-whole-column-blank-tail (Excel oracle, Col/ColFar/Row/RowFar; identical with
and without a far used cell Z100). `MATCH(99,A:A,1)` = 3, `MATCH(9.99E+307,A:A)` = 3,
`VLOOKUP(99,A:B,2)` = "c", `HLOOKUP(99,1:2,2)` = "c", `LOOKUP(99,A:A,B:B)` = "c",
`LOOKUP(20,A:A,B:B)` = "b", `LOOKUP(30,A1:A3,E:E)` = 0 (blank result), same as the bounded
`A1:A10` forms. `MATCH(99,A:A,-1)` and `MATCH(5,A:A,1)` are #N/A. Exact lookups never match a
blank: `VLOOKUP(0,A:B,2,FALSE)`, `VLOOKUP("",A:B,2,FALSE)`, `HLOOKUP(FALSE,1:2,2,FALSE)`,
`MATCH(0,A:A,0)`, `XMATCH(0,A:A)` (and `XMATCH(0,1:1)`) are #N/A, and so are a blank-cell lookup
value in `MATCH(G1,A:A,0)`, `MATCH(G1,A:A,1)`, `VLOOKUP(G1,A:B,2[,FALSE])`.
Status: VERIFIED
Implemented in: `SearchVector::approximate_search`/`exact_position`; XMATCH exact mode skips
blanks (T-0126); linear XMATCH/XLOOKUP in `linear_scan` (T-1018).
Tasks: T-0126, T-1018

## XMATCH / XLOOKUP linear search over blanks
Rule: in an exact or wildcard search (search_mode 1/-1) a blank candidate never matches a non-blank
lookup value; a blank lookup value (`G1`, an empty cell) matches the first blank candidate. For
match_mode -1/1 and a numeric lookup value a blank candidate is larger than every number and never
smaller than one. So with 5,_,10,_,20,_ in A1:A6: `XMATCH(0,A1:A6,-1)` is #N/A, `XMATCH(0,A1:A6,1)` is 1,
`XMATCH(7,A1:A6,1)` is 3 (10 is the smaller of the larger candidates) but `XMATCH(25,A1:A6,1)` and
`XMATCH(99,A:A,1)` are 2 (the first blank, the only candidate "larger" than 25). `XMATCH(0,A1:A6,2)`
is #N/A. XLOOKUP matches XMATCH in every one of these (`XLOOKUP(0,A1:A6,B1:B6,"nf")` is "nf",
`XLOOKUP(0,A1:A6,B1:B6,"nf",1)` is "a", `XLOOKUP(G1,A1:A6,B1:B6,"nf")` is "b"). Error cells are skipped.
Evidence: probes/xmatch-xlookup-blanks (Excel oracle; Col, Err, Row sheets, bounded and whole column/row).
Status: VERIFIED
Implemented in: `linear_scan` (functions/lookup_and_reference/xmatch.rs), shared by XLOOKUP.
Text and boolean lookup values (probes/xmatch-text-bool-equal-runs, Excel oracle): with b,_,d,_,f,_ in
A1:A6, a blank is *larger* than every text, as it is larger than every number: `XMATCH("z",A1:A6,1)` is 2 (the first
blank), `XMATCH("z",A1:A6,-1)` is 5, `XMATCH("a",A1:A6,-1)` is #N/A (a blank is never the smaller candidate),
`XMATCH("a",A1:A6,1)` is 1. Binary search also treats a blank as larger (`XMATCH("z",A1:A6,1,2)` is 6, `XMATCH("z",A1:A6,-1,2)` is 5). With b,#N/A,d,#DIV/0!,f in C1:C5 (Txt!M31-M48) `XMATCH("z",C1:C5,1)` is 4, the #DIV/0! cell (the #N/A at C2 is not chosen over it),
`XMATCH("z",C1:C5,-1)` is 5, `XMATCH("z",C1:C5,1,2)` is 4, the error cell again, and `XMATCH("z",C1:C5,-1,2)` is 3; whether the linear modes skip #N/A and
how errors order among themselves is not settled by this probe. Booleans over B1:B6 = FALSE,_,TRUE,_,TRUE,_ (Txt!M19-M30):
`XMATCH(FALSE,B1:B6,1,-2)` is 6 (a blank), `XMATCH(FALSE,B1:B6,-1,-2)` is #N/A, `XMATCH(TRUE,B1:B6,1,-2)` is 5.
`XLOOKUP("z",A1:A6,A1:A6,"nf",1)` returns the blank return cell as 0.
A blank lookup value (`G1`) never matches a 0, `""` or FALSE candidate in an exact search (`XMATCH(G1,D1:D1)`,
D1=0, is #N/A, likewise D2 = `=""`, D3 = FALSE, and with search_mode 2/-2). With match_mode -1 the `""` and FALSE
candidates are the next smaller item (`XMATCH(G1,D3:D3,-1)` is 1), with match_mode 1 they are #N/A. `XMATCH(G1,D:D)` is 4, the first blank. `XLOOKUP(G1,D1:D1,E1:E1,"nf")` is "nf" for all three candidates.
Status: VERIFIED
Implemented in: `linear_scan`/`compare_blank_last` (functions/lookup_and_reference/xmatch.rs): a blank candidate is larger than every number, text and boolean; a blank lookup value matches only a blank (T-1153). The same ordering holds for a blank lookup value against numbers, `""` and FALSE in match modes -1/1, linear and binary (`XMATCH(G1,P1:P2,-1)` is 2 with -5,5; `XMATCH(G1,P2:P2,1)` is #N/A), and in wildcard mode (`XMATCH(G1,D:D,2)` is the first blank, `XMATCH(G1,D1:D3,2)` is #N/A): probes/xmatch-blank-extensions Ext!M1-M37.

## XMATCH / XLOOKUP binary search (search_mode 2/-2) probes blanks and errors
Rule: search_mode 2/-2 is the classic binary search on inclusive bounds, probing the lower middle
`(lo+hi)/2`. Unlike MATCH's approximate search ("skip upwards past other types", T-0212), it never
skips a blank or an error cell. It compares the cell like any other, and a blank or an error is
larger than every number. An equal value does not stop it: see "binary search over runs of equal values" below. Otherwise match_mode -1/1 return the
last probed smaller/larger cell, which may be a blank. With 5,_,10,_,20,_ in A1:A6:
`XMATCH(10,A1:A6,0,2)` is 3 and `XMATCH(20,A1:A6,0,2)` is 5 (mid 2 is 10). `XMATCH(7,A1:A6,1,2)`
is 2, the blank A2. Over the whole column, `XMATCH(99,A:A,-1,2)` is 1 and `XMATCH(99,A:A,1,2)` is
2: the probes 524287, 262143, ..., 3, 1, 0 are all blank until A1. `XMATCH(99,A:A,-1,-2)` is #N/A.
Along a row, `XMATCH(99,1:1,-1,2)` is 1. With 5,#N/A,10,#DIV/0!,20, `XMATCH(15,A1:A5,-1,2)` is 3
(the #DIV/0! probe counts as larger). XLOOKUP agrees (`XLOOKUP(10,A1:A6,B1:B6,"nf",0,2)` is "c").
Evidence: probes/xmatch-xlookup-blanks Col!M16-M26, M44-M46, Err!M6-M7, Row!A18-A19 (Excel oracle).
Status: VERIFIED
Implemented in: `binary_search` (functions/binary_search.rs), shared by XMATCH and XLOOKUP.
XLOOKUP over a range searches the full vector, so cells past the used area are probed as blanks.
Runs of equal values: see "binary search over runs of equal values" below (T-1135).

Performance invariant (T-0119, T-0127, not an Excel rule): MATCH, LOOKUP, HLOOKUP, VLOOKUP and XMATCH evaluate a
search vector only up to the sheet's last used row (column, for ranges running to XFD); the rest
is a virtual run of blanks of the full length (`SearchVector`,
`functions/lookup_and_reference/lookup_table.rs`), so results are those of a full walk whatever
the used area (unit tests compare against a sheet whose used area reaches the last row/column).
T-0212's "skip upwards past other types" step must jump over that tail in O(1) (every index
from the evaluated length on is blank), not walk it.

## XLOOKUP: return_array may be wider than the lookup vector
Rule: with a column lookup vector, a return range with the same number of rows and several columns
returns the whole matched row: `XLOOKUP(8389,B5:B14,C5:D14)` is the 1x2 array
{"Dianne Pugh","Finance"}.
The row-vector analog holds: a row lookup vector over a taller return range returns the matched
column (`XLOOKUP("b",F1:H1,F2:H3)` spills 20; 200, probes/xlookup-array-lookup-value-args C33).
Evidence: poi/xlookup.xlsx (Excel-cached, Excel Online 16) Sheet1!C2:D2; probes/xlookup-row-column-picks (T-0962).
Status: VERIFIED (Excel-cached)
Probed (probes/xlookup-row-column-picks, excel-graph, 2026-10-11, T-0962): the picked row or column is a
reference (`ISREF` TRUE, E3, F15); `@` intersects it with the formula cell: `@XLOOKUP(2,A1:A3,B1:C3)` is "two" in column B of row 2 and 0 (the blank C2)
in column C (B2, C2; G3 likewise down a picked column), and `#VALUE!` outside the picked row (E4, G15), never its first value; a blank cell in it spills as 0 (F1, E16); `UPPER(...)` of it spills
element-wise with a blank as "" (E7, E19). A one-cell lookup vector takes either orientation from the return:
`XLOOKUP(1,A1:A1,B1:C1)` is the row (E9), `XLOOKUP("a",F1:F1,F2:F4)` the column (E23); 1x1 arrays likewise (`XLOOKUP(1,{1},{5,6})` and `{5;6}`, E44, E46), and a 2-D return fits neither (`#VALUE!`, E43). A return range of
another row count (column lookup) or column count (row lookup) is `#VALUE!` (E11, E27), as is a scalar
return_array (E35, E36). Array return arguments follow the same rules over a range lookup vector
(`XLOOKUP(2,A1:A3,{"p","q";"r","s";"t","u"})` is "r", "s", E13; a row lookup over a 2x3 array gives the
matched column, E31), and an array row lookup accepts a taller array return (`XLOOKUP("b",{"a","b"},{1,2;3,4})`
is 2; 4, E28). Reverse search, `if_not_found` and `SUM` over the picked column work as for one cell (E34, E37, E41).
Implemented in: `Model::xlookup_table`, `xlookup_orientation`, `Model::xlookup_pick` and `xlookup_pick_array` (functions/xlookup.rs); `@` through `evaluate_at_operand_reference`. XLOOKUP's static-analysis result is `Unknown` since it can return an array.
Array lookup_array (T-1126, T-1127): `XLOOKUP(2,{1;2;3},B1:B3)` looks the value up in the array and picks the return range's cell, a
reference (`ISREF` is TRUE; probes/isref-reference-lookups H10, excel-graph). All of the following are from
probes/xlookup-array-return (excel-graph): an array return_array gives its element (`XLOOKUP(2,{1;2;3},{4;5;6})` is 5, E1; row arrays E4),
`if_not_found` and `#N/A` work as for ranges (E2, E3); the binary search modes 2 and -2 work over an array lookup_array with a range or
array return_array, including match modes -1/1 (E9-E15); search modes 1 and -1 explicit (E22-E24). Shape rules: a 2-D lookup array is
`#VALUE!` (E8); a column lookup array needs the return's row count and a row lookup array its column count, so a column lookup array with a row return
(E16), a row lookup array with a column return (E18, E6) or a shorter return (E5, E21) is `#VALUE!`; a column lookup array accepts a wider
return_array and gives the matched row (E7, E19, E20), a row lookup array a taller one and gives the matched column
(probes/xlookup-row-column-picks E28). A scalar return_array is `#VALUE!` (same probe, E35, E36). Still unprobed: an error return_array, and
wildcard match mode with a binary search mode. A 2-D *range* lookup_array is `#VALUE!` as well (probes/xlookup-array-lookup-value-args A15, per element).
Tasks: T-0948

## XMATCH / XLOOKUP binary search over runs of equal values
Rule: search_mode 2 returns the *first* cell of a run of equal values and search_mode -2 the *last*, for match_mode 0,
-1 and 1 alike, including runs that start at the first cell or fill the range: `XMATCH(40,A1:A3,m,2)` is 1 and
`XMATCH(40,A1:A3,m,-2)` is 3 with {40,40,40}; {40,40,40,40,40} gives 1 and 5; {30,40,40,40,50} gives 2 and 4;
descending {60,50,40,40,40} gives 3 and 5; seven 40s give 1 and 7. Linear search_mode 1 gives the first cell
(1) for every match_mode. XLOOKUP agrees (`XLOOKUP(40,B1:B5,D1:D5,"nf",0,2)` is 30, with `-2` it is 50).
Evidence: probes/xmatch-text-bool-equal-runs Run!M1-M52 (Excel oracle). The engine matches all of them (T-1153).
A binary search remembers an equal element and keeps bisecting past it, to the left when ascending (2) and to the right when
descending (-2), so a gap inside a run ends it, for every match mode. With FALSE,_,TRUE,_,TRUE,_ in B1:B6,
`XMATCH(TRUE,B1:B6,-1,-2)` is 5 (Txt!M21, M24); with 60,40,70,40, `XMATCH(40,Q1:Q4,m,-2)` is 4 and with 40,30,40,40,50,
`XMATCH(40,R1:R5,m,2)` is 1, not 3 (probes/xmatch-blank-extensions Ext!M38-M43). In a contiguous run that is the run's first
cell ascending, its last descending, match_mode 1 included (Run!M5, M11, M17, M35, M51).
Status: VERIFIED
Implemented in: `binary_search` (functions/binary_search.rs).
Tasks: T-1135, T-1153

## XLOOKUP / XMATCH with an omitted lookup_value is a blank cell
Rule: an omitted lookup_value (`XMATCH(,H1:H4)`, `XLOOKUP(,...)`) is indistinguishable from a reference to a blank cell, in every
match mode and search mode: exact, -1, 1, wildcard (2), reverse linear (-1) and binary (2, -2). With H1:H4 = 1, blank, "a", empty
text, `XMATCH(,H1:H4)`, with match_mode -1, 1 and 2, and with search_mode -1 are all 2, the blank cell's position (and the
`XLOOKUP` forms return the 2nd return value). Empty text is not equal to the omitted value: `XMATCH("",H1:H4)` is 4. In binary
search over J1:J5 = 1, "a", TRUE, blank, blank, `XMATCH(,J1:J5,0,2)` is 4 and the -1 and 1 match modes give 4 too; the descending search
gives #N/A (exact and 1) and 1 (match_mode -1); `XMATCH("",J1:J5,0,2)` is #N/A. Inline arrays holding no blank, such as
`XLOOKUP(,{"";1},{1;2})`, `XMATCH(,{"";1})` and `XMATCH(,{1;"a";TRUE},0,2)`, are #N/A, so an omitted value is not empty text; it is not 0 either, since `XMATCH(0,J1:J5,0,2)` is #N/A where the omitted form is 4.
Evidence: probes/xlookup-omitted-lookup-value (excel-graph, 2026-10-08, 40 cells, omitted and G9-blank twins identical), plus
XLOOKUP.xlsx Errors!N1 (Excel-authored) for binary.
Status: VERIFIED (web)
Implemented in: `Ord for CalcResult` (calc_result.rs), used by `binary_search`; `is_blank` / `linear_scan` (xmatch.rs) read `EmptyArg` as a blank in the linear modes (T-1188).
Tasks: T-1153, T-1157, T-1188

## Lookup and reference function argument modes
Rule: these probed scalar arguments lift over a multi-cell range and over an array: ADDRESS's row_num, column_num and
abs_num (`ROWS(ADDRESS(A1:A3,1))` = 3), CHOOSE's index_num, HYPERLINK's link_location and friendly_name, INDIRECT's
ref_text (`INDIRECT(C1:C3)` with C1:C3 = "A1", "B2", "A3" is 3x1, first value 1), the lookup_value of LOOKUP, XLOOKUP and
XMATCH (`XLOOKUP(A1:A3,A1:A3,B1:B3)` is 3x1, first 10), XLOOKUP/XMATCH match_mode (`XMATCH(2,A1:A3,{0;1})` is 2x1),
OFFSET's rows (`OFFSET(A1,A1:A2,0)` is 2x1, first 2) and TRIMRANGE's trim_rows (`TRIMRANGE(A1:B5,{0;1})` is 2x1, first
element `#SPILL!`). FORMULATEXT and ROW over a range of cells give one result per cell (`FORMULATEXT(A1:A3)` is 3x1, first
element `#N/A` for a constant cell; `COLUMN(A1:B3)` is 1x2). CHOOSECOLS and CHOOSEROWS do not lift their indexes: a range
or array of indexes is read whole as a list, whatever its orientation (`CHOOSECOLS(A1:B3,A1:A2)` and
`CHOOSECOLS(A1:B3,{1,2})` are both 3x2; `CHOOSEROWS(A1:B3,A1:A2)` is 2x2). The vector and array arguments take an
array literal whole: `LOOKUP(2,{1,2,3},{4,5,6})` = 5, `LOOKUP(2,{1,2,3})` = 2, `XLOOKUP(2,{1,2,3},{4,5,6})` = 5,
`XMATCH(2,{1,2,3})` = 2, `ROWS({1;2;3})` = 3, `COLUMNS({1,2})` = 2, `CHOOSEROWS({1;2;3},{1;3})` is 2x1,
`TRANSPOSE({1,2,3})` is 3x1. INDEX of a single value is that value (`INDEX(LOOKUP(2,{1,2,3},{4,5,6}),1,1)` = 5,
`INDEX(AREAS(A1:B3),1,1)` = 1). UNVERIFIED (not probed): OFFSET's cols, height and width, XLOOKUP/XMATCH search_mode,
XLOOKUP's if_not_found, ADDRESS's sheet_text, INDIRECT's a1 and TRIMRANGE's trim_cols; which problem wins inside a
CHOOSECOLS/CHOOSEROWS index list between uncoercible text and a later error, or between two errors (an error beats an
out-of-range index: VERIFIED, see the T-2273 entry; ours returns uncoercible text's `#VALUE!` or the first error found at once).
XLOOKUP's match_mode and search_mode both lift (probes/lookup-argument-modes; probes/xlookup-array-lookup-value-args;
probes/xlookup-array-search-mode-indirect). An error lookup_value does not stop the lift when match_mode is an array: the
result is still an array, so a blocked spill is `#SPILL!`. Real oracle cell (gen-m6/fe0972380d030e4a Calc!A224,
excel-graph, T-2290): `=XLOOKUP(VALUE((Data!$E$1&Data!D6)),Data!A5:A10,Data!C1:D9,ISERR(FIND(" x ","ü",1)),DECIMAL(OFFSET(Data!B10:E11,5,3,3,2),21),0)`
is `#SPILL!`; its lookup_value is `#VALUE!` and its scalar search_mode `0` is invalid. VERIFIED (T-2746, excel-graph, 2026-10-26,
probes/t2746-xlookup-error-lookup-array-modes, 32 cells): with an unblocked spill every element holds the lookup_value's
own error (`#DIV/0!` for `1/0`, `#N/A` for `NA()`, `#VALUE!` for `"x"+1`), and an error lookup_value beats an invalid
match_mode or search_mode element (`{0;9}`, `{9;0}`, `{9;8}`, `{1;9}`), also when both modes are arrays; a non-error
lookup_value with `{0;9}` gives the match then `#VALUE!`. if_not_found does not replace an error lookup_value
(`XLOOKUP(NA(),..,"nf",{0;1})` is `#N/A`). With only scalar criteria the error is returned as is. Ours matches (no code change).
Evidence: probes/lookup-argument-modes (117 cells, excel-graph, 2026-10-09, T-1238); ADDRESS's a1 and HYPERLINK's
friendly_name over an array spill: gen-m7/f68fafa0ca2eba26 A7, gen-m7/2bb922eef72e785e A22 (excel-graph, 2026-10-11).
Ours: ADDRESS, HYPERLINK, LOOKUP's and XMATCH's lookup_value, XMATCH's match_mode and XLOOKUP lift, FORMULATEXT gives
one result per cell, and CHOOSECOLS/CHOOSEROWS read their index lists whole (T-1241). OFFSET, INDIRECT and TRIMRANGE
lift too, into arrays of references (T-1810 probed them; T-1955 to T-1957 implement them),
and INDEX of a single value is `#VALUE!` (T-1242).
Status: VERIFIED (web)
Tasks: T-1238 (generator modes), T-1241 (lifts and index lists), T-1810 (reference-valued lifts; split into
T-1955 to T-1959), T-1242 (INDEX of a single value)

## A reference-valued lookup_value is looked up by its cell's value
Rule: when lookup_value is a one-cell reference returned by a function or written as a range (`CHOOSE(1,J1)`,
`OFFSET(J1,0,0)`, `INDIRECT("J1")`, `J1:J1`), VLOOKUP, HLOOKUP, MATCH, XMATCH, XLOOKUP and LOOKUP use the
referenced cell's value, exactly as for `IF(TRUE,J1)`: found, between-values, above-all and not-found text all give
the same results across the five shapes, in exact and approximate modes. `HLOOKUP(CHOOSE(1,Data!D7),Data!A8:A10,
AVERAGE(..),..)` finds it and fails on the row index with `#VALUE!`. (The XMATCH text-vs-number order seen in
the same probe has its own entry below.)
Evidence: probes/lookup-reference-lookup-value (220 cells: 5 shapes x 4 values x 11 lookups; excel-graph,
2026-10-11, T-1256); gen-m6/963a217976538d5d Calc!A32 (excel-graph, 2026-10-09, T-1239).
Status: VERIFIED
Implemented in: VLOOKUP, HLOOKUP and XLOOKUP lift the lookup_value, so a 1x1 range lifts to its value
(`lookup_each`, xlookup.rs); MATCH, XMATCH and LOOKUP read it through `Model::lookup_value_arg`
(functions/lookup_and_reference/mod.rs); a multi-cell one lifts in XMATCH and LOOKUP (T-1241) and in MATCH (next entry).
Tasks: T-1256

## MATCH lifts over lookup_value
Rule: an array (a 1x1 array included) or a multi-cell range as MATCH's lookup_value gives one position per element, row-major, and
spills: `ROWS(MATCH(B1:B2,A1:A3,0))` and `ROWS(MATCH({30;10},A1:A3,0))` are 2, `COLUMNS(MATCH({30,10,99},..))` is 3, and a
missing element is `#N/A` in that slot only. A text element is searched as text (`MATCH({"x"},C1:C3)` is 2, E10), and so is one
that reads like an error: in gen-m7/4629e931955aa4b8 A19, `MATCH(TEXTSPLIT("#N/A",..),E8:E10)-LOG10(0)` is `#NUM!`, so the MATCH of
the text `"#N/A"` returned a position, not `#N/A` (which would have won the subtraction). An error element is that slot's
result before an invalid match_type is checked: `INDEX(MATCH({#N/A;1},A1:A3,"x"),1)` is `#N/A` (an invalid lookup_array next
to an error element is not probed).
Empty-text elements lift like any other text: in the CSE dedup idiom `MATCH(r&"",r&"",)` (match_type omitted, so 0) a blank
cell of `r` becomes `""` and matches the first `""` of the lookup array, so `SUM(MATCH(A1:A3&"",A1:A3&"",))` is 3 over three blanks
and `INDEX(MATCH(B1:B4&"",B1:B4&"",),4)` is 2 when B2 and B4 are the blanks (probes/match-empty-text-lift D4-D8; corpus-bench 530-28
P8:Q8). This differs from the blank cells themselves: `MATCH("",A1:A3,0)` over blanks is `#N/A` (D9), as above.
An array-valued match_type also lifts, broadcast with lookup_value (`ROWS(MATCH(2,A1:A3,{0;1}))` is 2, `INDEX(MATCH(B1:B2,A1:A3,{0;1}),1)` is 3); a multi-cell range match_type still reads its top-left cell (probes/match-type-nested-passthrough), and static analysis spills only for an array one.
A scalar error lookup_value lifts over an array match_type too: `ROWS(MATCH(NA(),A1:A3,{0;1}))` is 2, `INDEX(MATCH(NA(),A1:A3,{0;1}),2)` is
`#N/A`, and `#DIV/0!` fills both slots of `MATCH(#DIV/0!,..,{0;1})`; `MATCH(25,A1:A3,{0;"x"})` is `#N/A` then `#VALUE!` (the second
slot's text match_type, not the error, decides; E3 itself is `#SPILL!` in the probe because E4:E5 block the spill).
Evidence: probes/match-lookup-value-lift (19 cells; excel-graph, 2026-10-18, T-1866); probes/match-error-lookup-value-array-match-type (8 cells; excel-graph, 2026-10-18, T-1926); probes/match-empty-text-lift (13 cells; excel-graph, 2026-10-18, T-1861); gen-m7/4629e931955aa4b8 Calc!A19 (excel-graph, T-1361).
Status: VERIFIED
Implemented in: `Model::fn_match` (functions/lookup_and_reference/mod.rs) through `lift_values`; static analysis (`match_static_result`) spills by lookup_value and an array match_type.
Tasks: T-1866, T-1861, T-1917, T-1926

## A reference-valued lookup_value pointing at a blank or error cell
Rule: `CHOOSE(1,J5)`, `OFFSET(J5,0,0)`, `INDIRECT("J5")`, `J5:J5` and the direct `J5` behave identically in VLOOKUP, HLOOKUP,
MATCH, XMATCH, XLOOKUP and LOOKUP.
- An error cell (`#DIV/0!`, `#N/A`) is returned as that error by all eleven forms (exact and approximate, every shape).
- A blank cell as the key, against an all-number table (10..50, no 0, no blanks), gives `#N/A` in every exact and approximate form
  (VLOOKUP, HLOOKUP, MATCH 0/1, XMATCH, XLOOKUP, LOOKUP), except the next-smaller modes of XMATCH and XLOOKUP, which take the last
  row: `XMATCH(J5,A1:A5,-1)` is 5 and `XLOOKUP(J5,A1:A5,B1:B5,"nf",-1)` is "e". That is the existing blank-key behavior, not a new
  rule: the blank is the number 0 in the approximate lookups (below 10, so `#N/A`; see "Approximate-match lookups") and larger than
  every number in XMATCH/XLOOKUP (so -1 takes the last row; see "XMATCH / XLOOKUP linear search over blanks"). The probe has no 0,
  `""` or blank candidates, so it does not test those against a reference-shaped key. The plain blank-cell key against
  tables that do hold a 0, a `=""`, a FALSE or a blank row is probes/t3096-lookup-value-blank-elements Calc rows 2-8: it
  finds only the 0 row (exact), and lands as the number 0 would in the approximate searches (above).
Evidence: probes/lookup-reference-lookup-value-blank-error (165 cells: 5 shapes x {blank, #DIV/0!, #N/A} x 11 lookups;
excel-graph, 2026-10-12, T-1371). The engine already matches every cell.
Status: VERIFIED
Tasks: T-1371, T-3096

## XMATCH and XLOOKUP next-smaller mode: text sorts above every number
Rule: with match_mode -1 (exact or next smaller), a text lookup_value that is not found among numbers takes the last number:
`XMATCH("x",{10;20;30;40;50},-1)` is 5 and `XLOOKUP("x",A1:A5,B1:B5,"nf",-1)` returns the last row's value, because text
sorts above every number. Exact mode gives `#N/A`, and the approximate `MATCH("x",A1:A5,1)` and `LOOKUP("x",..)` also give `#N/A`
(they do not extend the number-before-text order across types).
Evidence: probes/lookup-reference-lookup-value rows 13/17/21/25/29, the J4 "x" rows (S and U columns: 5, "e"; excel-graph, 2026-10-11, T-1256).
Status: VERIFIED
Tasks: T-1256

## INDEX lifts over array and range indices
Rule: in a dynamic-array formula, a multi-cell range or array as row_num, column_num or area_num lifts INDEX
element-wise. The indices broadcast like arithmetic operands, and each element is the scalar INDEX of its indices:
- `INDEX(A1:A5,{1,3})` spills A1 and A3, and so do `INDEX(A1:A5,SEQUENCE(2))` and `INDEX(A1:A5,J1:J2)`.
- `INDEX(A1:B5,{1;2;3},{1,2})` is 3x2, every row-column pair; `{1,2}` against `{1;2}` is 2x2.
- `INDEX((A1:A5,B1:B5),{1,2},1,{1,2})` lifts area_num.
- Each element follows the scalar rules: an error element stays itself (`{1,#N/A}`); out of range is `#REF!`
  (`{1,9}`); "2" is 2 and "x" is `#VALUE!`; 1.9 floors to 1; one index into a 2-D range is `#REF!` per element
  (`INDEX(A1:B5,{1,2})`, `INDEX(A1:B5,{0,FALSE})`, gen-m6 A25's `INDEX(Data!C2:D9,CHOOSEROWS(..))`, also under `&`).
- An element that would select a whole row, column or array keeps only its top-left value: `INDEX(A1:A5,{0,2})` is
  A1, A2; `SUM(INDEX(A1:B5,0,{1,2}))` = A1 + B1; `INDEX({1,2;3,4},{1,2})` is 1, 3 (the first value of each row).
- The elements are values, not references: `ROWS(INDEX(A1:A5,{1,3}))` is 1, `COLUMNS` 2, and `ISREF` of it is a single
  FALSE. `SUM` of it is the sum of the values.
Edge cases (probes/index-lifted-edge-cases, T-1376):
- Mismatched index lengths broadcast like arithmetic: `INDEX(A1:B5,{1,2,3},{1,2})` is 10, 2, `#N/A` (the third pair has no
  column element); `{1,2}` against `{1,2,3}` is the same; `{1;2;3}` against `{1,2}` is a 3x2 grid (first row 10, 1);
  `{1;2}` against `{1;2;3}` is a 3x1 column 10, 2, `#N/A`.
- A negative element is `#VALUE!` in its own slot (`{1,-1}` is 10, `#VALUE!`; `{-1,-2}` is two `#VALUE!`). A boolean element
  is its number: `{TRUE,2}` is A1, A2, `{FALSE,2}` is A1 (as `{0,FALSE}` in probes/index-array-indices), A2, and `{TRUE,FALSE}` is A1, A1.
- A scalar error index next to an array index spreads over the whole result: `INDEX(A1:A5,{1,3},#N/A)` is `#N/A`, `#N/A`;
  `1/0` gives `#DIV/0!`, `#DIV/0!`; the error in row_num (`INDEX(A1:A5,#N/A,{1,3})`), in either position of a 2-D range, or
  in area_num spreads the same.
- An empty selected cell spills as 0 (`INDEX(D1:D5,{1,2})` with D2 blank is 10, 0; `INDEX(D1:D5,{2,4,5})` is 0, "x", TRUE and the
  scalar `INDEX(D1:D5,2)` is 0), `&"x"` makes it "x", `+1` makes 1,
  `=0` and `=""` are TRUE, `SUM(INDEX(D1:D5,{1,2,3}))` is 40, `ISBLANK` of the lifted result is FALSE, TRUE (the elements keep blankness).
- `COUNTBLANK(INDEX(D1:D5,{1,2,3}))` is a 1x3 spill of `#VALUE!` (one per element) and
  `COUNTA(INDEX(D1:D5,{1,2,3}))` is 3, counting the blank element (references.md, "COUNTBLANK of a value that is not a
  reference is #VALUE!"; T-1437, T-1395).
- A 1x1 array index lifts too: `INDEX(D1:D5,{2})` is a 1x1 array, not the reference the scalar index 2 picks, and so are
  `{2}` as column_num, as both indices, as row_num with column_num 0, `{2}+0`, `SEQUENCE(1)+1` and `{1}` as area_num
  (probes/index-1x1-array-index, excel-graph, 2026-10-14, T-1507). ISREF of it is FALSE, TYPE 64 (1 for the scalar
  index), ROWS 1; what needs a reference is `#VALUE!`: AREAS, the range operator (`ROWS(INDEX(D1:D5,{2}):D5)`,
  `SUM(INDEX(D1:D5,{1}):D3)`), OFFSET, `CELL("address",…)`, SUMIF, COUNTIF and COUNTBLANK (countblank-array-values H1).
  COUNTA counts its blank element (1; the scalar index gives 0) while ISBLANK of it is TRUE and `&"|"` gives "|";
  `N(INDEX(D1:D5,{1}))` is 10 (T-1557: already fixed by the T-1616 element-wise lift).
The legacy, implicitly intersected form is T-0294.
DEVIATION (decision 0014): a lifted INDEX whose broadcast indices exceed 1,048,576 elements is `#ERROR!`, the shared
array bound (arrays.md, "Array results above 1,048,576 cells are #ERROR!").
Evidence: probes/index-array-indices (60 cells, excel-graph, 2026-10-11, T-1260); probes/index-lifted-edge-cases (excel-graph, 2026-10-12, T-1376); probes/index-1x1-array-index (excel-graph, 2026-10-14, T-1507); gen-m6/2c1a6e93a1974f48 Calc!A25
(excel-graph, 2026-10-09, T-1239).
Status: VERIFIED (excel-graph)
Implemented in: `Model::index_impl` / `index_lifted` (functions/lookup_and_reference/index.rs).
Tasks: T-1260, T-1376, T-1437, T-1507

## INDEX of a scalar or a 1x1 array
Rule: a scalar (number, text, boolean) or a 1x1 array is a 1x1 range for INDEX. Row and column 0 or 1 return the value (`INDEX(5,0,0)`,
`(5,0,1)`, `(5,1,0)`, `(5,1)`, `(5,0)`, `INDEX("a",1)`, `INDEX(TRUE,1,1)`, `INDEX({7},1,1)`, `INDEX({7},0,0)` return it; `INDEX(5,1.9,1)` is 5); a
row or column of 2 is `#REF!` (`INDEX(5,2,1)`, `(5,1,2)`, `(5,2)`, `INDEX("a",2)`, `INDEX({7},2,1)`, `INDEX({7},1,2)`), a negative index or a text
index is `#VALUE!` (`INDEX(5,-1,1)`, `(5,1,-1)`, `(5,"a",1)`), an error value is that error (`INDEX(#N/A,1,1)`), and area_num 1 is
accepted while 2 is `#REF!` (`INDEX(5,1,1,1)` is 5, `INDEX(5,1,1,2)`).
So `INDEX(LOOKUP(2,{1,2,3},{4,5,6}),1,1)` is 5 and `INDEX(ROWS({1;2;3}),1,1)` is 3.
Evidence: probes/index-scalar-indexes A1:A23 (excel-graph, 2026-10-17, T-1242); probes/lookup-argument-modes H18, H19, H24, H28, H30, H31, H39.
Status: VERIFIED (excel-graph)
Ours: implemented (the engine matched every cell before T-1242).

## INDEX of an array: a whole row stays an array
Rule: INDEX of an array (not a reference) that selects a whole row or column returns an array, even when that row or column
is one cell; only a selection by a row and a column number is a scalar. With one index into a multi-row array the index is
a row number and the result is that row: `INDEX({2;3},1)` and `INDEX({2;3},2)` are 1x1 arrays (`TYPE` 64, `ROWS` 1, value
2 and 3), as `INDEX({2,3;4,5},1)` is a 1x2 one; so is `INDEX({2;3},1,0)`. With one index into a one-row array the index is
a column number and picks one cell, a scalar: `INDEX({2,3},1)` and `INDEX({2},1)` have `TYPE` 1, as do
`INDEX({2;3},1,1)` and `INDEX({2,3;4,5},1,1)`. `TYPE({2})` is 64. A reference result is unaffected:
`TYPE(INDEX(Z1:Z2,1))` is 1 (one cell). So a legacy N or T over it is `#VALUE!` (references.md "Legacy N and T over a
reference-returning call"): `=N(INDEX({2;3},2))`, `=T(INDEX({"a";"b"},1))` and `=N(INDEX({2;3},1,0))` are `#VALUE!`, while
`=N(INDEX({2,3},2))` is 3 and `=N(INDEX({2},1))` 2; a legacy `=TYPE(INDEX({2;3},1))` is shown `=TYPE(@INDEX({2;3},1))`, 1.
Every whole-row and whole-column pick (an index of 0) of a scalar or a 1x1 array is an array too: `TYPE` is 64 and `ROWS` and
`COLUMNS` are 1 for `INDEX({7},0,0)`, `INDEX({7},1,0)`, `INDEX({7},0,1)`, `INDEX({7},0)`, `INDEX(5,0,0)` and `INDEX(5,0)`;
`INDEX({2,3},0,1)` (one cell of a one-row array's column) is a 1x1 array (`TYPE` 64, value 2) and `INDEX({2;3},0,1)` a 2x1 one.
`INDEX({2,3},0)` is the whole 1x2 row (`TYPE` 64, `COLUMNS` 2; a spill that collides is `#SPILL!`). A legacy `=N(...)` over
any of them is `#VALUE!` and `FORMULATEXT` shows the formula as typed, with no `@`, including for the scalar `INDEX(5,0,0)`
and `INDEX(5,0)`.
Evidence: probes/index-one-index-array-shape (excel-graph, 2026-10-19, T-1921); probes/legacy-n-t-marker-shapes F10;
probes/index-zero-index-scalar-shape (excel-graph, 2026-10-20, T-1943).
Status: VERIFIED (excel-graph)
Ours: implemented (T-1921, `index_from_array` in functions/lookup_and_reference/index.rs); a legacy N over `INDEX(5,0,0)` or
`INDEX(5,0)` (a scalar first argument) is `#VALUE!` with no `@` (T-2191, `indexes_a_scalar_literal` in static_analysis/legacy.rs).
The same holds for T and for every scalar literal: a legacy `=T(INDEX(5,0,0))`, `=T(INDEX("a",0,0))`, `=N(INDEX("a",0,0))`,
`=N(INDEX(TRUE,0))`, `=N(INDEX(-5,0,0))` (a negated number), `=N(INDEX(5,,0))` (an empty row reads as 0) and the one-index
forms are all `#VALUE!`, and `FORMULATEXT` shows them with no `@`.
Evidence: probes/index-zero-index-scalar-t-n (excel-graph, 2026-10-24, T-2368). Ours: implemented (T-2368).
The same holds with an empty column or row argument (`=N(INDEX(5,0,))`, `=N(INDEX(5,))`, `=N(INDEX(5,,))`), and for a
parenthesised scalar, a unary plus, or a negated text or boolean: `=N(INDEX((5),0,0))`, `=T(INDEX(+"a",0,0))`,
`=N(INDEX(-TRUE,0,0))`, `=T(INDEX(-"a",0,0))`, `=T(INDEX(("a"),0))` and `=N(INDEX((-5),0))` are all `#VALUE!`, and so is
any chain of groups and minus around the literal (`--5`, `-(5)`, `-("a")`, `((5))`, `(TRUE)`, `(-"a")`, `-(-5)`) and a
grouped 0 index (`INDEX(5,(0))`); `FORMULATEXT` shows them all with no `@`. FORMULATEXT drops a unary plus on a number (`+5` prints `5`) but keeps one on text
or a boolean (`+"a"`, `+TRUE`). Ours matches since T-2723.
Excel shows no `@` on INDEX of a scalar literal under N or T, whatever the indices. When every index is nonzero it takes
the element: `=N(INDEX(5,1))` and `=N(INDEX(5,1,1))` are 5, `=T(INDEX("a",1))` is "a", `=N(INDEX(TRUE,1))` is 1,
`=T(INDEX(5,1))` is "". A computed index (`INDEX(5,C1)` with C1 1, `INDEX(5,0+1)`, `INDEX(5,IF(TRUE,1))`) and a negated or
grouped scalar take the element too (`=N(INDEX(-5,1))` is -5). An array or range index keeps its own `@`
(`=N(INDEX(5,@{1,1}))` is 5).
`=N(INDEX(5,2))` is `#REF!` and `=N(INDEX(5,-1))` is `#VALUE!`. One 0 or empty index beside a 1 takes the whole scalar:
`=N(INDEX(5,1,0))`, `=N(INDEX(5,0,1))`, `=N(INDEX(5,1,))` and `=N(INDEX(5,,1))` are `#VALUE!`, and so is an index cell
holding 0 or blank (`INDEX(5,C2)` with C2 0)
(probes/t2725-index-scalar-nonzero-t-n A1:B24, excel-graph 2026-10-25, T-2725). Ours: implemented (T-2725,
`indexes_a_scalar_literal`).
Evidence: probes/t2601-index-scalar-forms-t-n A1:B28 (excel-graph, 2026-10-25, T-2601). Ours: implemented (T-2601,
`indexes_a_scalar_literal`); the kept unary plus (B14, B15) since T-2723.

## INDEX rounds an index up near an integer; a tiny negative index is 0
Rule: INDEX reads a row or column index as LEFT/RIGHT/REPLACE read a count (text.md "LEFT, RIGHT and REPLACE round
counts up near an integer"): truncated, except that a value within 2^-22 + 2^-33 (about 2.38535E-7) below an integer
counts as that integer, an absolute bound. A negative index no further below 0 than half that bound,
2^-23 + 2^-34 (about 1.192675E-7), counts as 0 (the whole row or column); anything further below is `#VALUE!`.
`INDEX(A1:C3,3,-1E-15)` is the whole row 3, as `INDEX(A1:C3,3,0)`, and
`EXPONDIST(INDEX(A1:C3,3,-1e-15),12.14649,FALSE)` is 0.027981694209681673, the density at A3 = 0.5, the first cell of
A3:C3 (EXPONDIST lifts over the row; the cell holds the spill's anchor).
Probed (excel-graph, 2026-10-12, T-1407), row and column index alike:
- Negatives: -1E-15, -1E-7, -1.19E-7, -2^-23, -1.19267E-7 and -1.1926749E-7 are index 0; -1.1926751E-7, -1.19268E-7,
  -1.193E-7, -1.2E-7,
  -1.5E-7, -1E-6, -0.5, -0.999, -0.9999999, -1 and -2147483648.5 are `#VALUE!`.
- Positives: `COLUMNS(INDEX(A1:C3,3,1-2.3853E-7))` is 1 (column 1) and `1-2.3854E-7` is 3 (index 0, the whole row);
  `ROW(INDEX(A1:A100000,500-2.3853E-7,1))` is 500 and `500-2.3855E-7` is 499; at 100000, 2.38E-7 below snaps and
  2.39E-7 does not. Both cuts bracket the text functions' bound, which is known to 2^-36.
- area_num -0.5 is `#VALUE!`; `INDEX((A1:C3,E1:G3),3,-1E-15,2)` is the third row of the second area (16, 17, 18).
- area_num rounds up the same way: with `(A1:C3,E1:G3)`, `2-1E-7` and `2-2E-8` are the second area (111) and `1-1E-7` the first,
  while `2-3E-7` is outside the bound and floors to the first; `-1E-8` is `#VALUE!` (area_num below 1). A near-integer element of an
  array row or column index rounds too: `SUM(INDEX(A1:C3,{1;3}-{0;1E-7},1))` is 42 (rows 1 and 3: 11+31) and `{1;3}-{0;3E-7}` is 32 (rows 1 and 2). A tiny negative element counts as 0, which inside a
  lifted index gives the top-left value, not the whole column (`{0;1}-{1E-8;0}` is 22, `{0,2}-{1E-8,0}` is 23)
  (probes/index-area-num-snap J1:J11, excel-graph, T-1424).
Evidence: probes/index-tiny-negative, probes/index-snap-tolerance and probes/index-area-num-snap.
Implemented in: `index_number` and `index_numbers` (area_num) (functions/lookup_and_reference/index.rs) with `near_integer_count` and
`NEAR_INTEGER_TOLERANCE` (functions/util/near_integer.rs). The negative cut is half the positive bound to within 1E-14
(-1.1926749E-7 vs -1.1926751E-7); the engine takes it as exactly half, inclusive.
Status: VERIFIED (web)
Tasks: T-1407, T-1424

## XLOOKUP with an array lookup_value looks up each element
`SUM(XLOOKUP({1;2},A1:A3,B1:B3))` is 30 with A1:A3 = 1,2,3 and B1:B3 = 10,20,30: an array `lookup_value` gives one result per element (10, 20).
The result has the shape of `lookup_value` (column, row or 2-D array, or a multi-cell range such as `Data!D1:D3`; a whole column gives a 1048576-row result), each element looked up by the scalar rules: an element not found is `#N/A` (or `if_not_found`), an error element is itself (`{2;#DIV/0!;"x"}` is two, `#DIV/0!`, `#N/A`), and `match_mode` applies per element. An element is a value, never a reference (`ISREF` is FALSE). With a return range wider than the lookup column, each element keeps only the first cell of its matched row: `XLOOKUP({2;3},A1:A4,C1:E4)` is 12, 13 in one column (`COLUMNS` is 1, against 3 for the scalar `XLOOKUP(2,...)`); likewise a row lookup over a taller return range keeps the first row (`XLOOKUP({"b";"c"},F1:H1,F2:H3)` is 20, 30).
`match_mode` and `search_mode` lift too, pairwise with `lookup_value`: `XLOOKUP({2.5;2.5},A1:A4,B1:B4,,{0;-1})` is `#N/A`, "two" and `XLOOKUP({1;1},{1;1},{"f";"l"},,0,{1;-1})` is "f", "l". Search modes 2 and -2, reverse search, wildcards and an array lookup_array all work per element. A mode array lifts on its own over a scalar lookup_value (`COLUMNS(XLOOKUP(2,A1:A4,C1:E4,,{0;0}))` is 1, and a reference `if_not_found` is then `#VALUE!` per element), and so does a 1x1 array (`COLUMNS(XLOOKUP({2},A1:A4,C1:E4))` is 1). A blank cell of a multi-cell lookup_value looks up as a blank, by the scalar rule: it matches a blank cell of the lookup vector and nothing else (probes/xlookup-array-lookup-value-args A69-A78).
Argument errors do not collapse the result: every element gets the error, in the scalar order. A lookup_value error element comes first (`XLOOKUP({2;#N/A},..,,5)` is `#VALUE!`, `#N/A`), then a bad match_mode or search_mode (`5`, `1/0`, search_mode `3` are `#VALUE!`, `#DIV/0!`, `#VALUE!` per element, as in the scalar form, A67), then a lookup_array or return_array error or shape mismatch (`1/0` either way is `#DIV/0!` per element; a return range of another length or a 2-D *range* lookup_array is `#VALUE!` per element, and scalar, A68). An error `if_not_found` shows only where nothing matches (`XLOOKUP({2;9},..,1/0)` is "two", `#DIV/0!`; with a bad match_mode as well the match_mode error wins).
Quirk: under a lifted lookup_value an array `if_not_found` is not lifted but gives its first element (`XLOOKUP({2;9},A1:A4,B1:B4,{"x";"y"})` is "two", "x"), and a reference `if_not_found`, even one cell, is `#VALUE!` where it applies (`XLOOKUP({9;2},..,Data!C1)` and `..,Data!C1:C2)` are `#VALUE!`, "two"), while the scalar `XLOOKUP(9,..,Data!C1:C2)` spills 11, 12.
DEVIATION (decision 0014, CLAUDE.md directive 5): each element is charged one search of the lookup vector (its evaluated cells, so a whole column costs its used area) against the shared cell-visit cap of criteria.md; `ROWS(XLOOKUP(SEQUENCE(1048576),SEQUENCE(1048576),SEQUENCE(1048576)))` is `#ERROR!` where Excel computes it.
Evidence: probes/let-name-uncalled-picked-refs H21 (T-1252), probes/xlookup-array-lookup-value and probes/xlookup-array-lookup-value-args (excel-graph, 2026-10-10 and 2026-10-11, T-1264).
Implemented in: `Model::xlookup_impl` (functions/xlookup.rs) through `Model::lift_values` (functions/lift/driver.rs), evaluating the table once (`xlookup_table`).
Status: VERIFIED (web)
Tasks: T-1252, T-1264

## TRIMRANGE with a fractional trim argument
Rule: trim_rows and trim_cols must be exactly 0, 1, 2 or 3; any fractional value is `#VALUE!`, including `0.9`,
`1.5`, `2.5` and `3.9` (no truncation), in either argument (`(A1:C3,3,2.5)`, `(A1:C3,2.5,2.5)`), as are `4` and `-1`.
Text `"2"` and `TRUE` coerce (3x3 kept for full data). So `TRIMRANGE` is the `#VALUE!` source in
`REPLACE("abc",TRIMRANGE(Data!C10:E11,2.5,3),1,"")`; REPLACE itself lifts over a multi-cell start_num or num_chars
and spills (`REPLACE("abc",E1:F2,1,"")` is a 2x2 spill).
Evidence: probes/trimrange-fractional-trim H1-H10, J1, L1 (excel-graph, 2026-10-17, T-1352); gen-m7/bcad709c30cfdf64 Calc!A20.
Status: VERIFIED
Tasks: T-1352

## XMATCH with an omitted argument; SORT with omitted sort arguments
`XMATCH(1,{1,2,3},)` (omitted match_mode) is 1 and `XMATCH(,{0,1,2})` (omitted lookup_value) is #N/A in this array (VERIFIED, probes/counta-empty-value D20-D21, excel-graph, T-1294). `SORT({3;1;2},,)` sorts ascending (`INDEX(...,1)` = 1, three elements) and `INDEX(SORT({3;1;2},,-1),1)` = 3 (D22-D24); the engine differs on the omitted-argument forms, see T-1394. (A bare `=SORT({3;1;2},,-1)` in D24 first read #SPILL! only because D25 and D26 were occupied; the probe was changed to INDEX.)
Status: VERIFIED (excel-graph; excel-desktop unprobed)
Tasks: T-1394

## CHOOSE over an index array broadcasts its value arguments
Rule: with an array (or range) index, CHOOSE lifts like IF over an array condition. Every value argument is
read as a scalar or an array (a range as its values). The result spans the largest extent of the index and of
all value arguments, including an argument that no index element picks. Each position takes the picked argument's
element at that position, with size-1 broadcasting. A ragged position (no element there in the index, or in the
picked argument) is `#N/A`. An index element out of range is `#VALUE!`. An arm is not reduced to one cell.
With D1=10, D2 empty, D3=30, D4="x", E1:E3 = 11, 12, 31:
- `CHOOSE({1,2},D1:D2,D3:D4)` is the 2x2 `{10,30;0,"x"}`, and `CHOOSE({1;2},D1:E1,D3:E3)` is `{10,11;30,31}`.
- `CHOOSE({1;2},D1:D2,D3:D4)` is `{10;"x"}`.
- `CHOOSE({1,2},D1:D3,D3:D4)` is 3x2 with `#N/A` at the bottom right.
- `CHOOSE({1,2},5,D3:D4)` is `{5,30;5,"x"}`, and `CHOOSE({1,2},D1:D2,1/0)` has a column of `#DIV/0!`.
- `CHOOSE({1,2,3},D1:D2,D3:D4)` has a third column of `#VALUE!`.
- `CHOOSE({1,2},{1,2,3},{4;5})` is `{1,4,#N/A;1,5,#N/A}`.
- A range index (`F1:G1` = 1,2) is the same as `{1,2}`.
- ROWS and COLUMNS of the 2x2 are 2, and `SUM(CHOOSE({1,2},D1:D2,E1:E2))` is 33.
- `CHOOSE({1,1},D1:D2,D1:E3)` is 3x2: the unpicked arm's 3 rows count.
- `CHOOSE({1,2},D1:D2,D3:D4,D1:F3)` is 3x3, with `#N/A` beyond the picked 2x2.
So `COUNTBLANK(CHOOSE({1,2},D1:D2,D3:D4))` and `RANK(10,CHOOSE(…))` are 2x2 grids of `#VALUE!` (references.md).
`SUMPRODUCT(CHOOSE({1,2},B1:B6,B1:B6))` is 24 on and off the row (criteria.md).
Evidence: probes/choose-array-index-range-arms (excel-graph, 2026-10-14, T-1508); probes/countblank-array-values H20,
H34; probes/array-context-legacy-sumproduct H1, H10; probes/external-ref-no-link-ifchoose A32.
Status: VERIFIED (excel-graph)
Ours: every range arm's cells are read in order, so a cycle through any arm is one (`=ROWS(CHOOSE({1,2},A:A,1:1))` in A1
is circular, 0), but no arm is copied until the result's size, taken from the arms' bounds, is checked; then only the
picked range arms are copied, so `CHOOSE({1},C:C,D:D,…)` with 254 whole-column arms holds one column (T-1564; IF over an
array condition alike).
Implemented in: `choose_array` (functions/lookup_and_reference/mod.rs), using `BroadcastArg` and
`evaluate_broadcast_arm` (functions/lift/broadcast_arg.rs).
Tasks: T-1508, T-1564

## XMATCH/XLOOKUP reference-form lookup and return arrays; capital Thorn
A single-cell reference is a one-cell vector for XMATCH `lookup_array` and XLOOKUP `lookup_array`/`return_array`: `XMATCH(5,A1)` is 1, `XMATCH(5,one)` (single-cell defined name) is 1, `LET(r,A1,XMATCH(5,r))` is 1 and `XLOOKUP(5,A1,B3)` returns B3's value. A blank single-cell `return_array` yields 0 (`XLOOKUP(5,one,B2)` is 0). `XMATCH(10,@A1:A3)` is `#N/A` (on row 1 the `@` yields the scalar 5, not a vector containing 10) and `XLOOKUP(5,A1,@B1:B3)` on row 7 is `#VALUE!` (the `@` finds no cell on row 7 to intersect, so the error arises before XLOOKUP runs; the probe cannot separate that from a scalar `return_array`). A 3-D span `Sheet1:Sheet3!A1` as lookup_array or return_array is `#VALUE!` (not `#REF!`). `"Þ"="TH"` is TRUE and `"Þ"<"TI"` is TRUE (Thorn collates as TH), so `XMATCH("Þ",D1:D3)` is 1, `XMATCH("TI",D1:D3,1)` is 3.
Evidence: probes/xmatch-xlookup-reference-forms G1, H1-H16, probes/xmatch-xlookup-3d-lookup-array H1-H3 (excel-graph, 2026-10-16, T-1698).
Status: VERIFIED (excel-graph); the engine already matches every cell.
Tasks: T-1698

## OFFSET height/width/rows from a one-cell TAKE or DROP of a range
Rule: TAKE and DROP of a range return a reference, so a one-cell result is a scalar for OFFSET's rows, cols, height and width, not a lifting
1x1 array: `ROWS(OFFSET(B9:C12,5,1,DROP(H2:H4,-2,0),2))` is 3 for H2 = 3 (1 for H3 = 1), `COLUMNS(..)` 2, `INDEX(..,1,1)` the cell. The same holds through
`IF(TRUE,DROP(..))`, `MAX(..)` and `N(..)`, and with `+0` (`TAKE(H2:H4,1)+0` is 3 rows). An array constant `{3}` or `{1E-15}`, `SEQUENCE(1,1,3)`,
`INDEX(H2:H4,{1})` and `CHOOSE({1},3)` lift: 1 row, `INDEX(..,1,1)` is `#SPILL!`. A two-cell `DROP(H1:H3,1,0)` lifts to a 2x1 array of references.
A scalar height or width that is not 0 but truncates to 0 gives 2 rows (columns) reaching one row *up* (left) from the start: the zero check
sees the number, the end is start + trunc(height) - 1, so the area is the row above the start and the start itself (the mechanism is inferred
from the values below, which it explains in full). `ROWS(OFFSET(B9:C12,5,1,h,2))`
is 2 for h = 1E-15, 1E-300, 0.1, 0.5, 0.9999, -1E-15 and -0.5, 1 for 1, 1.5, 1.9999, -1 and -1.5, 2 for 2.5; `COLUMNS(..,2,1E-15)` and
`COLUMNS(..,2,0.5)` are 2; `ROWS(OFFSET(A1,0,0,1E-15))` is `#REF!` (no row above row 1); `ROWS(OFFSET(B9,0,0,-0.5))` is 2 and
`ROW(OFFSET(B9,0,0,-0.5))` spills (`#SPILL!` in one cell), `ROW(OFFSET(B9,0,0,-1.5))` 9 (probes/t3072-take-drop-one-cell-reference P23-P41; T-3072).
A CHOOSE or LET picking a one-cell DROP is a scalar height too: `ROWS(OFFSET(B9:C12,5,1,CHOOSE(1,DROP(H2:H4,-2,0))))` and
`ROWS(OFFSET(B9:C12,5,1,LET(x,DROP(H2:H4,-2,0),x)))` are 3, also with `+0` (P19-P22).
Evidence: probes/t2454-offset-array-height, probes/t2454-offset-one-cell-array-args, probes/t2414-gen-m7-unexplained A141, A151, A161, A171
(excel-graph, 2026-10-23, T-2454).
Ours: `offset_values` reads each number through `evaluate_picked_source`, so a TAKE/DROP (or an IF, CHOOSE or LET picking one) is its reference (T-2519). A one-cell
TAKE/DROP of a single-sheet range stays its reference in value context (`take_drop_reference`), so the `+0` and `N(..)` forms are scalar too (T-2519).
Status: VERIFIED (excel-graph); implemented: the engine matches every cell of the three probes, the near-zero heights included (T-2454, T-2519, T-3072)
Tasks: T-2454, T-2519, T-3072

## TAKE/DROP of a one-cell reference is a reference
Rule: TAKE and DROP of a one-cell reference (`E7`, not only `E7:E7`) return that reference, so a one-cell result is a scalar under an operator and
reads as a reference cell everywhere. With E7 text and E9 blank, `ROWS(E9-TAKE(E7,2,42))`, `ROWS(E9-TAKE(E7:E7,1))`, `ROWS(E9-DROP(E7,0))` and
`ROWS(TAKE(E7,1)+0)` are `#VALUE!`; `TYPE(1+TAKE(C8,1))`, `TYPE(1+DROP(C8,0))` and `TYPE(1+TAKE(C8:C9,1))` are 1 while `TYPE(1+TAKE({5},1))`
and `TYPE(1+TAKE(C8,{1}))` (a lifted count) are 64. A blank one-cell TAKE is SUMXMY2's blank one-cell reference: `SUMXMY2(TAKE(C11,-1,1),B6:C10)`
and `SUMXMY2(TAKE(C11:C12,1),B6:C10)` are `#VALUE!` as `SUMXMY2(C11,B6:C10)`, while a number (`TAKE(C8,1)`, `C8`, `5`, `TAKE({5},1)`) against
the 10-cell range is the count's `#N/A` (coercion.md "SUMX2MY2 / SUMX2PY2 / SUMXMY2 pairing"). The information functions see the reference:
`ISREF(TAKE(E7,1))`, `ISREF(TAKE(E7:E8,1))`, `ISREF(DROP(E7:E8,1))` and `ISREF(TAKE(E7:E8,2))` are TRUE; `ISBLANK(TAKE(C11,1))`,
`ISBLANK(TAKE(C11:C12,1))` and `ISBLANK(DROP(C11:C12,1))` TRUE; `TYPE(TAKE(E7,1))` 2 but `TYPE(TAKE(E7:E8,2))` 64;
`CELL("address",TAKE(E7:E8,-1))` `$E$8`, `CELL("address",TAKE(E7,1))` `$E$7`, `CELL("row",DROP(E7:E8,1))` 8, `CELL("type",TAKE(C11:C12,1))` `b`;
`ISTEXT(TAKE(E7:E8,1))` TRUE and `ISFORMULA(TAKE(E7:E8,1))` FALSE.
Evidence: probes/t3072-take-drop-one-cell-reference P1-P18, P42-P57 (excel-graph, 2026-10-28, T-3072); gen-m9/7674c89c824aca43 Calc!A305
(`CHITEST((E9-TAKE(E7,2,42)),C8:C10)` `#VALUE!`), gen-m9/7c20351562a2d71d Calc!A277.
Ours: `take_drop_reference` reads a one-cell reference argument, on this sheet or another, through `evaluate_node_with_reference`, as a
one-cell range, before choosing its value or reference read (T-3072).
Status: VERIFIED (excel-graph); implemented
Tasks: T-2519, T-3072

## OFFSET, INDIRECT and TRIMRANGE lift into arrays of references
Rule: a multi-cell range or an array as OFFSET's rows, cols, height or width, INDIRECT's ref_text or a1, or
TRIMRANGE's trim_rows or trim_cols lifts the call element-wise, and each element is a *reference* (the scalar
call's result), not a value. What the caller sees depends on who reads the elements:
- Spilled as the cell's result, every element is `#VALUE!`, even a one-cell reference: `=OFFSET(A1,{0;1},0)`,
  `=OFFSET(A1,0,{0,1})`, `=INDIRECT(C1:C3)`, `=TRIMRANGE(A1:B5,{0;1})` and `=OFFSET(A1,0,0,{1;2})` spill only
  `#VALUE!`. An element that is an error stays itself: `=INDIRECT("A2",D1:D2)` (D1 TRUE, D2 FALSE) is
  `#VALUE!`, `#REF!` ("A2" read as R1C1).
- INDEX picks an element as the reference itself: a one-cell element gives its value (`INDEX(OFFSET(A1,0,{0,1}),1,2)`
  is B1, `INDEX(INDIRECT(C1:C3),1,1)` is A1, `INDEX(TRIMRANGE(A1,{0;1}),2,1)` is A1) and `ISREF` of the pick is TRUE;
  a multi-cell element spills its cells (`INDEX(OFFSET(A1,0,0,1,A1:A2),2,1)` spills A1:B1 as 1, 10) or is `#SPILL!`
  where its area is blocked (`INDEX(OFFSET(A1,0,0,{1;2}),2,1)`, `INDEX(INDIRECT({"A1";"A1:A2"}),2,1)`,
  `INDEX(TRIMRANGE(A1:B5,0,{0;1}),1,1)`). This is the nested-array `#SPILL!` of lookup-argument-modes H34:H35.
- The shape is the lifted argument's: `ROWS`/`COLUMNS` count elements (`ROWS(OFFSET(A1,0,0,{1;2}))` is 2,
  `COLUMNS(TRIMRANGE(A1:B5,0,{0;1}))` is 1), and `ISREF` of the whole result is a single FALSE.
- SUM, AVERAGE, MIN, MAX and PRODUCT read each element's top-left cell as a range cell (text,
  booleans and blanks skipped; an error element is its error): with A1 = 1, A2 = 100,
  `SUM(OFFSET(A1,0,0,{1;2}))` and `SUM(OFFSET(A1,0,0,{2;3}))` are 2, `SUM(OFFSET(A2,0,0,{1;2}))` 200,
  `SUM(OFFSET(A1,{0;1},{0,1}))` 303, `SUM(INDIRECT({"A1";"Nope"}))` and `AVERAGE(OFFSET(A1,{0;-1},0))` `#REF!`. COUNT counts no element (0), COUNTA every
  element, a blank one too, and SUMPRODUCT reads each as 0. SUBTOTAL, COUNTIF, SUMIF(S), COUNTBLANK, AREAS,
  ROW, COLUMN, CELL, ISFORMULA, N and T lift, once per whole element (`SUBTOTAL(9,OFFSET(A1,0,0,{1;2;3}))`
  spills 1, 101, 10101). Operators, IS* functions, IFERROR, TEXTJOIN and AGGREGATE see `#VALUE!` per element;
  IF, CHOOSE, LET, VSTACK, TAKE and DROP inside SUM hand the elements on; OFFSET over such a result lifts
  again. A lifted TAKE's grid is handed on the same way by IF and CHOOSE in every consumer and
  lifts again as OFFSET's reference (`SUMPRODUCT(IF(TRUE,TAKE(A1:B3,{1;2})))` is 0, `ROWS(OFFSET(TAKE(A1:B3,{1;2}),1,0))`
  is 2), and TAKE over an error source with a lifted count is one error per element
  (`ROWS(TAKE(NA(),{1;2}))` is 2; probes/m7-lift-take-drop-sortby, T-2299). DROP behaves the same
  (`ROWS(DROP(NA(),{1;2}))` is 2, `SUMPRODUCT(IF(TRUE,DROP(A1:B3,{1;2})))` and the CHOOSE form 0, `ROWS(OFFSET(DROP(A1:B3,{1;2}),1,0))` 2;
  probes/drop-lifted-grid-handon F1:F3, F9, F10, excel-graph, T-2919), and an error source beats an invalid lifted count for both:
  `ROWS(TAKE(NA(),{1;"x"}))` and the DROP form are 2, every element `#N/A` (same probe F4:F8, F11). TAKE and DROP keep the elements they count: `SUM(TAKE(OFFSET(A1,{0;1;2},0),1))` is 1 and
  `SUM(DROP(OFFSET(A1,{0;1;2},0),1))` 10100 (E101, -b2 E153). Ours: since T-2174 for an unlifted count. Inside SUM
  only a direct OFFSET or INDIRECT source keeps it (TRIMRANGE, IF, LET and nested TAKE/DROP sources there read
  values), and a lifted count over such a grid is unprobed (T-2212). Full table with cells: SPEC-0014 "Excel behavior to match" (probes/reference-lift-consumers,
  -b1c, -b1d, -b2, excel-graph, 2026-10-19, T-1955). COUNTIFS, AVERAGEIF and MAXIFS lift per element too
  (T-2065 probes; ours since T-2823); see "Statistical aggregates over a lifted OFFSET" below for STDEV.S,
  VAR.S, SUMSQ, AVERAGEA, MAXA, CONCAT, MEDIAN, LARGE and SMALL. AVERAGEIFS and MINIFS are unprobed.
- Per element, the scalar rules hold: an element off the sheet is `#REF!` (`OFFSET(A1,{0;-1},0)`, second element);
  `INDIRECT("R2C1",{TRUE;FALSE})` is `#REF!`, A2; a scalar error argument next to the lifted one is that error in every
  element (`INDEX(OFFSET(A1,{0;1},NA()),1,1)` is `#N/A`).
- A TAKE or DROP whose rows or cols lifts over a range is the same kind of result: `INDEX(TAKE(A1:B3,{1;2}),1,1)`
  spills A1:B1 as 1, 10 (ROWS 1, COLUMNS 2), `INDEX(DROP(A1:B3,{1;2}),1,1)` is the 2x2 A2:B3 (`#SPILL!` where blocked),
  and the outer result is 2x1 (probes/m7-argument-modes F4:I6, K1-K4; arrays.md "Dynamic-array and LAMBDA function
  argument modes"). Spilled directly its elements are `#VALUE!`, SUM reads their top-left cells (2), SUMPRODUCT 0,
  IF and CHOOSE hand it on and OFFSET over it lifts again, as for OFFSET; over an array value TAKE keeps top-left
  values (probes/m7-lift-take-drop-sortby, T-1960). The engine gives it to INDEX, ROWS, COLUMNS, SUMPRODUCT, the
  aggregates and LET as a lifted OFFSET's, while IF, CHOOSE and OFFSET's reference argument read it as a value (T-1838;
  T-2299 owns the difference).
- XMATCH's search_mode lifts too, as a value (for the array modes only the second element is probed, `INDEX(…,2,1)`): it is 2 in
  `XMATCH(2,A1:A3,0,{1;-1})` and 3 in `XMATCH(2,{1;2;2},0,{2;-2})`; an invalid mode element is `#VALUE!` in its slot
  (`{1;3}`); a boolean range element counts as its number (TRUE is 1). The first array-mode elements are unprobed, as is an error lookup_value next to a lifted search_mode (`XMATCH(NA(),A1:A3,0,{1;-1})`, implemented as a 2x1 of `#N/A` by analogy with MATCH's match_type, UNVERIFIED). Implemented (T-1958): `fn_xmatch` lifts search_mode with lookup_value and match_mode.
- ERROR.TYPE and ISERROR of an INDEX pick whose multi-cell element cannot spill are the cell's `#SPILL!`, not a code or
  TRUE (`ERROR.TYPE(INDEX(OFFSET(A1,0,0,{1;2}),2,1))`, `ISERROR(INDEX(TRIMRANGE(A1:B5,{0;1}),2,1))`; F11, G11, H11).
Evidence: probes/reference-lift-elements (excel-graph, 2026-10-19, T-1810); probes/lookup-argument-modes F14:H15,
F20:H21, F34:H35 (excel-graph, 2026-10-09, T-1238).
Status: VERIFIED (excel-graph)
Implemented in: XMATCH's search_mode: xmatch.rs (T-1958). The reference carrier (SPEC-0014, T-1956): `RefGrid` and
`RefArg::Grid` (reference_operators.rs) with its value view `RefGrid::to_values`; OFFSET as producer
(`Model::offset_reference`, lookup_and_reference/offset.rs, lifted by `Model::lift_references` in lift/driver.rs; its
spill shape from `static_analysis_offset`); INDEX's pick (`Model::index_grid`, index.rs), ROWS/COLUMNS (the value
view's shape) and ISREF (FALSE). The aggregates (T-2063): SUM, AVERAGE, MIN, MAX and PRODUCT through the
walker's `GridReading::TopLeft` (`Model::walk_top_left_cells`, util/aggregate_args.rs), COUNT and COUNTA through
the value view (`GridReading::Values`), SUMPRODUCT through `Model::sumproduct_array` (sumproduct.rs); the aggregates already see a grid through IF and
CHOOSE (`SUM(IF(TRUE,r))` 101, b1c E69, E73), which their reference reading picks. INDIRECT
(`Model::indirect_reference`) and TRIMRANGE (`Model::trimrange_reference`) are producers on the same carrier (T-1957;
shared helpers in lookup_and_reference/ref_lift.rs); TRIMRANGE's array, computed-reference and error sources:
"TRIMRANGE lifted over an array, a computed reference or an error" below (T-2089). The
per-element callers (T-2064): `Function::per_reference_element` lists them and the arguments that take a reference,
and `Model::call_per_reference_element` (lookup_and_reference/ref_lift.rs, called from `Model::evaluate_function`)
runs each once per element with the element as an absolute reference in its place (a SUMIF or SUMIFS with several
grids broadcasts them); COUNTIF, SUMIF and SUMIFS spill by `criteria_and_grid_static_result`. SHEET of a grid is
one `#N/A` (b1d E101), and VSTACK of grids in a reference position stacks them (`Model::vstack_reference`, b2 E225).
The other arguments are read once before the elements run and put back as literals or absolute references, so a
nest of such calls is evaluated once per level. Unprobed (T-2292): an error element in a per-element call (ours: the
call with that error in the reference's place, `AREAS(OFFSET(A1,{0;-1},0))` 1, `#REF!`); a criteria or function_num
array beside a grid (ours: each element's call cut to its top-left value, so `COUNTIF(OFFSET(A1,{0;1},0),{">0",">5"})`
is 2x1, where a broadcast would give 2x2); VSTACK of several grids (ours: narrower ones padded with `#N/A` elements);
an argument beside a grid that reads as an empty value, a 3-D range or a LAMBDA (ours: the call is `#VALUE!`,
`COUNTIF(OFFSET(A1,{0;1},0),IF(TRUE,C5))` with C5 blank); a grid reached through IF, CHOOSE, LET, TAKE, DROP or TRIMRANGE by a per-element caller (ours: the value view, only
a direct OFFSET or INDIRECT is read per element).
DEVIATION (decision 0014, CLAUDE.md directive 5): before any element runs, each element of a per-element call is
charged one visit (ROW, COLUMN, AREAS, CELL, ISFORMULA, N, T) or the cells of its reference inside the used area
(SUBTOTAL, COUNTIF, SUMIF, AVERAGEIF, COUNTBLANK; SUMIFS, COUNTIFS and MAXIFS times their criteria pairs) against the shared cell-visit cap of
criteria.md; past it the call is `#ERROR!` where Excel computes it
(`SUBTOTAL(9,OFFSET(A:A,0,0,,SEQUENCE(1,1000)))` with A1048576 filled). The pass-through of IF, CHOOSE, LET and a LAMBDA parameter, and OFFSET over a grid:
"Arrays of references through LET, IF, CHOOSE and OFFSET" below (T-2078).
Tasks: T-1810, T-1955, T-1956, T-1957, T-1958, T-2063, T-2064, T-2065, T-2078, T-2089, T-2823

## TRIMRANGE lifted over an array, a computed reference or an error
Rule: what a TRIMRANGE lifted over trim_rows or trim_cols holds per element depends on its source (A1 = 5, A2 = 7,
the rest blank):
- A reference, written or not, gives references, as for a written range: a defined name (`srcOne` = `S!$A$1`),
  `CHOOSE(1,A1)`, `INDIRECT("A1")`, `OFFSET(A1,0,0)`, `IF(TRUE,A1)`, `A1:A1` and `A1` all give
  `INDEX(TRIMRANGE(src,{0;1}),2,1)` = 5, `ISREF` of the pick is TRUE (probed for the name, CHOOSE and INDIRECT), and
  `=TRIMRANGE(CHOOSE(1,A1),{0;1})` spills `#VALUE!`, `#VALUE!`.
  SUM reads each element's top-left cell: `SUM(TRIMRANGE(src,{0;1}))` is 10 for those one-cell sources and for the
  multi-cell `A1:B5`, `CHOOSE(1,A1:B5)`, a name for `S!$A$1:$B$5` and `INDIRECT("A1:B5")`.
- An array is trimmed to an array, and each element is that array's top-left *value*, not `#VALUE!`:
  `=TRIMRANGE({1,2},{0;1})` spills 1, 1; `INDEX(…,1,1)` and `INDEX(…,2,1)` are 1 (a 1x1, `ROWS*100+COLUMNS` of the pick
  is 101), `ISREF` of the pick FALSE, `SUM(…)` 2, `COUNTA(…)` 2, `ROWS` 2 and `COLUMNS` 1;
  `INDEX(TRIMRANGE({1;2},0,{0,1}),1,2)` is 1 and its SUM 2.
- An error source is that error in every element, in the trim argument's shape: `=TRIMRANGE(NA(),{0;1})` spills
  `#N/A`, `#N/A`, `ROWS(…)` is 2, `COUNTA(…)` 2, `ERROR.TYPE(INDEX(…,2,1))` 7, and `ERROR.TYPE(…)` itself spills (here
  `#SPILL!`, the cell below being taken); `TRIMRANGE(1/0,0,{0,1})` is a 1x2 of `#DIV/0!`; a written reference to an
  `#N/A` cell (`TRIMRANGE(A9,{0;1})`) is a 2x1 too, `ERROR.TYPE` of its pick 7. Unlifted, `TRIMRANGE(NA())` stays one
  `#N/A`.
Evidence: probes/trimrange-lift-sources (44 cells, excel-graph, 2026-10-20, T-2089).
Status: VERIFIED (excel-graph)
Implemented in: `Model::lifted_trimrange` (lookup_and_reference/trimrange.rs): the source read by
`evaluate_picked_reference_arg`, a reference lifted by `Model::lift_references`, anything else by `Model::lift_values`
with its top-left value per element; SUM and the walker's other aggregates take TRIMRANGE's grid from
`picked_reference_arg_reading` (functions/util.rs). Probed in T-2120 (probes/trimrange-lift-edge-sources, excel-graph, 2026-10-20): an error source with an invalid
trim element is the source's error in every element (`TRIMRANGE(NA(),{0;5})` and `TRIMRANGE(1/0,{0;5})` are 2 elements,
`ERROR.TYPE` 7 and 2); a multi-area source `(A1,B2)` is `#VALUE!` in every element (`ROWS` 2, `ERROR.TYPE` 3); an
all-blank single column (C20:C21) keeps its full extent while trim_rows is 0, whatever trim_cols is: `TRIMRANGE(C20:C21,0)`
and `(…,0,3)` are the 2x1 range, so `=TRIMRANGE(C20:C21,{0;1})` spills `#VALUE!` (the untrimmed multi-cell element) then
`#REF!`; a nonzero trim_rows gives `#REF!` (`(…,3,0)`, `(…,1,0)`); the unlifted call behaves the same.
Other all-blank shapes (probes/t2127-trimrange-blank-shapes, excel-graph, 2026-10-20, T-2127): a blank source keeps its
full extent only when nothing is trimmed (0, 0), or when only the single-cell axis of a line is trimmed: the 2x1 with
trim_cols 1-3 (and trim_rows 0), and the 1x3 with trim_rows 1-3 (and trim_cols 0, so `TRIMRANGE(C25:E25,3,0)` is the
1x3). Everything else is `#REF!` (ERROR.TYPE 4): the 2x3 for any trim but (0, 0), the 1x3 with a nonzero trim_cols,
and a 1x1 (`TRIMRANGE(C40,3,0)`) for any trim but (0, 0). So trim_rows 0 does not keep a multi-column source.
Tasks: T-2089, T-2120, T-2127

## Arrays of references through LET, IF, CHOOSE and OFFSET
Rule: a LET variable or a LAMBDA parameter bound to a lifted OFFSET (or INDIRECT, TRIMRANGE) holds the array of
references itself, and IF and CHOOSE hand the one they pick on, so every reader that sees the bare call's elements
sees them through these too; OFFSET whose reference argument is such an array lifts again over its elements.
With A1 = 1, A2 = 100, A3 = 10000:
- INDEX picks the element: `LET(x,OFFSET(A1,{0;1},0),INDEX(x,2,1))` is 100, also through a second variable
  (`LET(x,..,y,x,INDEX(y,2,1))`), a LAMBDA parameter (`LET(f,LAMBDA(r,INDEX(r,2,1)),f(OFFSET(A1,{0;1},0)))`),
  `INDEX(IF(TRUE,..),2,1)`, `INDEX(CHOOSE(1,..),2,1)` and `INDEX(LET(x,..,x),2,1)`; `ISREF(INDEX(x,1,1))` is TRUE
  and `SUM(INDEX(x,2,1))` over `OFFSET(A1,0,0,{1;2})` reads the whole element (101).
- ROWS/COLUMNS count elements (`LET(x,..,ROWS(x))` 2) and `ISREF(x)`, `ISREF(IF(TRUE,..))` are FALSE.
- The top-left aggregates read the elements through the variable (`LET(x,..,SUM(x))` 101, `SUM(IF(TRUE,x))` 101),
  and SUMPRODUCT reads each element as 0 through IF, CHOOSE and LET (`SUMPRODUCT(IF(TRUE,OFFSET(A1,{0;1},0)))` 0).
- OFFSET over the array lifts again, broadcast with its own lifted arguments: `ROWS(OFFSET(OFFSET(A1,{0;1},0),1,0))`
  is 2, its SUM 10100, `SUM(OFFSET(OFFSET(A1,{0;1},0),{0,1},0))` 10201 (a 2x2), `SUM(OFFSET(OFFSET(A1,0,0,{1;2}),1,0))`
  200 (SUM reads each moved element's top-left cell, A2 twice), also over a variable, IF, CHOOSE or LET; an error element stays itself
  (`ERROR.TYPE(INDEX(OFFSET(OFFSET(A1,{0;-1},0),1,0),2,1))` 4) and spilled the result is `#VALUE!` per element.
- INDIRECT and TRIMRANGE grids do the same: `LET(x,INDIRECT({"A1";"A2"}),INDEX(x,2,1))` is 100, its ROWS 2, its SUM
  101, `SUM(OFFSET(INDIRECT({"A1";"A2"}),1,0))` 10100, `SUMPRODUCT(LET(x,INDIRECT(..),x))` 0,
  `LET(x,TRIMRANGE(A1:B3,0,{0;1}),SUM(INDEX(x,2,1)))` 30303 and `SUM(OFFSET(TRIMRANGE(A1:B3,0,{0;1}),1,0))` 200.
Evidence: probes/reference-lift-consumers E89, E145, E149, E209, reference-lift-consumers-b1c E77, E81 (excel-graph,
2026-10-19, T-1955); probes/reference-lift-pass-through E1-E34, G1, H1, reference-lift-pass-through-b E1-E10
(excel-graph, 2026-10-20, T-2078).
Status: VERIFIED (excel-graph)
Implemented in: `Binding::Grid` (variable_stack.rs), bound by `Model::bind_argument` and read by
`Model::variable_reference` (functions/logical/let.rs); `Model::evaluate_value_or_grid` (functions/util.rs) for
SUMPRODUCT; `Model::offset_source` and `Model::offset_over_grid` (lookup_and_reference/offset.rs). IF and CHOOSE
already handed grids on to INDEX and the aggregates through their reference reading.
Tasks: T-2078

## Arrays of references held by LET variables are bounded
Rule: Excel's limit on the arrays of references the LET variables and LAMBDA parameters in scope may hold is not
known. Ours: their distinct grids may hold 4 x 2^20 elements together (a grid shared by several bindings counts once);
a binding past that is `#VALUE!`, as a union binding past its own budget is (references.md "Union area cap").
Evidence: none of Excel's (directive 5 bound); unit test `variable_stack::tests::held_grids_are_charged_once_and_bounded`.
Status: DEVIATION (decision 0014)
Implemented in: `VariableStack::insert` (variable_stack.rs), `MAX_HELD_GRID_ELEMENTS`.
Tasks: T-2078

## XLOOKUP: an error lookup_value beats a size mismatch
Rule (VERIFIED, probes/workday-serial-zero F40, excel-graph, 2026-10-19, T-1254): `XLOOKUP(NA(),A1:A4,B1:B5)` is
`#N/A`, not the `#VALUE!` of lookup and return arrays of different sizes: the lookup value's error is returned first.
The modes come next, before the size mismatch: both are read as numbers in argument order, and only then is either checked,
so an error search_mode beats an unknown match_mode, while a text match_mode's `#VALUE!` and an error match_mode beat an error
search_mode (`XLOOKUP(1,B1:B3,B1:B3,0,"x",1/0)` `#VALUE!`, `XLOOKUP(..,0,NA(),1/0)` `#N/A`; probes/lookup-t2278-followups A6, A7):
`XLOOKUP(<blank>,Data!E5:E9,Data!C3:D8,0.3,101,INDEX(Data!D4:E11,0))` is `#REF!` (INDEX of a 2-D range with one index,
"INDEX with one index into a 2-D range is #REF!"), and so is INDIRECT, ROWS and COLUMNS of it, while without that
search_mode it is `#VALUE!` (the 6-row return array against the 5-cell lookup array, and match_mode 101 alike)
(probes/xlookup-array-search-mode-indirect A1-A10, A21, excel-graph, 2026-10-22, T-2278; gen-m6/e3a562557696e2fc A33).
INDIRECT of an error is that error (`INDIRECT(1/0)` `#DIV/0!`, `INDIRECT(NA())` `#N/A`), and an array search_mode of valid
modes lifts (`ROWS(XLOOKUP(2,..,{1;2}))` 2).
Implemented in: `xlookup_modes` (crates/rustsheet-engine/src/functions/xlookup.rs).
Tasks: T-1254, T-2278

## CHOOSEROWS and CHOOSECOLS: an error argument beats an out-of-range index (T-2273)
Rule (excel-graph, gen-m6, 2026-10-21): an error value in any argument, the array or a later index, is the result even when an
earlier index is 0 or out of range: `CHOOSECOLS(Data!A1,1E+15,ACOSH(-0.3))` is `#NUM!`, `CHOOSEROWS(Data!A7:B8,4,FLOOR(0.3,-0.0001),1)`
`#NUM!`, `CHOOSECOLS(SQRT(2958465),2,-2,INDEX(Data!A8:D12,6,2))` `#REF!`, `CHOOSEROWS(CHOOSEROWS(Data!D11:E12,1),1E-15,6,2-FLOOR(7,0))`
`#DIV/0!`, `CHOOSEROWS(Data!B11:E11,2,-1,..SUMX2MY2(<mismatched>))` `#N/A` (gen-m6/121a7c17092038d9 Calc!A298, 07171fa3f7493f51 A333,
ee56a4ed5a612a0d A19, 535caabe1e635e2d A136, 7ac4ea222977f3b4 A391; 26 cells in T-2273). This answers the UNVERIFIED "which problem
wins inside a CHOOSECOLS/CHOOSEROWS index list" above for errors against bad indexes; which of two errors wins is still unprobed.
Ours: implemented (T-2273): an out-of-range index is remembered and the scan goes on, so an error in any later argument wins and
the `#VALUE!` is returned only if none is found. Between two errors, ours keeps the first in argument order (UNVERIFIED).
Status: VERIFIED
Tasks: T-2273

## ADDRESS: argument errors first, sheet_text "TRUE"/"FALSE" quoted, bounds (T-2274, T-2275)
Rule (excel-graph, gen-m6, 2026-10-21):
- An error in any argument beats ADDRESS's own `#VALUE!` for a bad row, column or abs_num: `ADDRESS(-1,3.14159,2,NA())` is `#N/A`,
  `ADDRESS(10,0.1/DATE(2055,0,5),3,INDEX(Data!A6:A7,3))` `#REF!`, `ADDRESS(18,0.1,-2.5,MINUTE(-0.5)<..,..)` `#NUM!`
  (gen-m6/0fd242eb9b2456a8 Calc!A280, 38d4bcdafaed1275 A35, 07171fa3f7493f51 A245; 15 cells in T-2274).
- A sheet_text of "TRUE" or "FALSE" is quoted like a name that needs quotes: `ADDRESS(9,26,3,TRUE,"TRUE")` = `'TRUE'!$Z9` and
  `ADDRESS(6,20,4,TRUE,<a 1x1 array holding the boolean FALSE>)` = `'FALSE'!T6` (a boolean sheet_text becomes "FALSE" and is quoted) (c96ac1ada46179c4 A197, ee56a4ed5a612a0d A306).
- A row past 1048576 or a column past 16384 is `#VALUE!` whatever abs_num and a1 are, R1C1 relative included:
  `ADDRESS(1E+15,7)`, `ADDRESS(21887494,5)`, `ADDRESS(123456831,3,1,..,"Data")`, `ADDRESS(17,123456789.123,2,FALSE)`,
  `ADDRESS(2,45306,2,FALSE,"Data")` (36d5ce763ea63631 A154, b54448af3fde07ca A220, d0c30225243fde60 A63, 04fdc9a67474aab9 A98,
  acdfeaf6e6eadd4e A379).
- Bounds per part (probes/t2275-address-bounds, 282 cells over abs_num 1-4 and a1 TRUE/FALSE, excel-graph, 2026-10-21, T-2275):
  row_num and column_num are floored, not truncated, in every mode (2.9 is 2, 1048576.9 is 1048576; -0.5, -1.5, -2.4, -2.5
  and -2.6 are -1, -2, -3, -3 and -3; 0.5 and 0.9 are 0). A part written as an index (every part in A1 style, whatever
  abs_num; the R part for abs_num 1-2 and the C part for 1 and 3 in R1C1) must then be 1..1048576 for a row and 1..16384 for a
  column. A relative R1C1 part (R for abs_num 3-4, C for 2 and 4) is an offset in -1048575..1048575 or -16383..16383, so
  `ADDRESS(1048576,3,3,FALSE)`, `ADDRESS(-1048576,3,4,FALSE)` and `ADDRESS(5,16384,2,FALSE)` are `#VALUE!` while
  `ADDRESS(-1048575,3,3,FALSE)` = `R[-1048575]C3`, `ADDRESS(1048575,3,3,FALSE)` = `R[1048575]C3`, `ADDRESS(5,-16383,4,FALSE)` =
  `R[5]C[-16383]` and `ADDRESS(5,16383,2,FALSE)` = `R5C[16383]` (K1:K4 probe both positive edges). An offset of 0 is written
  bare: `ADDRESS(0,3,3,FALSE)` = `RC3`, `ADDRESS(9,0,2,FALSE,"Data")` = `Data!R9C`, `ADDRESS(0,0,4,FALSE)` = `RC`. So
  `ADDRESS(10,-2.5,4,FALSE)` = `R[10]C[-3]` (3a26794015dfbe9d A275), and `DATEVALUE(ADDRESS(9,0,2,FALSE,"Data")&OFFSET(Data!A4:D10,0,-1))`
  is the OFFSET's `#REF!` (44194b2fefa30de2 A307).
- gen-m6/be0e3a1b0fc3f7a0 A167 `ADDRESS(9,3,4,(Data!A5=MULTINOMIAL(<blank>)),T("1E3"))` is `#VALUE!` because
  `MULTINOMIAL(<blank cell>)` is (coercion.md "GCD / LCM / MULTINOMIAL range cells", T-2294); ADDRESS itself gives
  `ADDRESS(9,3,4,FALSE,"1E3")` = `'1E3'!R[9]C[3]`, and quotes "1A", "A1" and "R1C1" sheet_text alike.
Status: VERIFIED (web).
Implemented in: `address_index` and `r1c1_part` (crates/rustsheet-engine/src/functions/lookup_and_reference/address_areas.rs,
T-2275, T-2274); every argument is cast before abs_num and the bounds are checked, and `address_sheet_text`
quotes a `sheet_text` of "TRUE" or "FALSE" (T-2274; other spellings are unprobed).
Ours: implemented (T-2274, T-2275).
Tasks: T-2274, T-2275

## HYPERLINK: an error link_location is the result (T-2277)
Rule (excel-graph, gen-m6, 2026-10-21): when link_location is an error, HYPERLINK returns that error as one value, whatever
friendly_name is: with a number friendly_name (`HYPERLINK(<328-character text>,GCD(..))` is `#VALUE!`: next entry), with an array one
(`HYPERLINK(XLOOKUP(<size mismatch>),MUNIT(2))` is `#VALUE!`, not a spill) and with a range one (`HYPERLINK(HLOOKUP(100,Data!A6:C11,7)&..,
OFFSET(Data!A6:C12,6,42))` is `#REF!`); `HYPERLINK("abc"&NA(),OFFSET(..)^..)` is a single `#N/A` that spills nothing (gen-m6/0bcac2d96e96b3df
Calc!A17, 44194b2fefa30de2 A169, d0c30225243fde60 A377, c6621490a4d67f4e A214/B214).
Ours: implemented (T-2277): an error link_location, or one past 255 characters, is the whole result over a scalar, array or range
friendly_name (`fn_hyperlink` reads link_location before the lift).
Status: VERIFIED
Tasks: T-2277

## HYPERLINK: a link_location past 255 characters is #VALUE! (T-2277)
Rule (excel-graph, 2026-10-22, probes/t2277-hyperlink-location-length A1-A15): a link_location of more than 255 characters is
`#VALUE!`, with or without friendly_name and over an array friendly_name (one `#VALUE!`, no spill): `HYPERLINK(REPT("a",255),1)` = 1,
`REPT("a",256)` is `#VALUE!` (also `HYPERLINK(REPT("a",256))`), `"#"&REPT("a",254)` = 1 and `"#"&REPT("a",255)` is `#VALUE!`: the
`#` of an internal link counts. This was the cause of gen-m6/0bcac2d96e96b3df Calc!A17 (a 328-character TEXTJOIN location), not a
TEXTJOIN error. Characters outside the BMP count as two (UTF-16 units, like LEN; excel-graph, 2026-10-27,
probes/t2358-hyperlink-location-non-bmp A1-A9): `REPT("😀",127)&"a"` (255 units, 128 code points) = 1, `REPT("😀",128)`
(256 units) is `#VALUE!`, as are 200 and 255 emoji; `LEN(REPT("😀",128))` = 256.
Ours: implemented (T-2277, confirmed T-2358: counts UTF-16 units).
Status: VERIFIED
Tasks: T-2277, T-2358

## LOOKUP past a short result vector gives a blank's empty value; MMULT of it is #VALUE! (T-2278)
Rule: when LOOKUP's match position lies past a one-cell result vector, it reads on to the blank cell there and gives its
empty value ("VLOOKUP and HLOOKUP hand a blank result cell to their caller as 0"): it shows 0, but it is no reference
(`ISREF` FALSE, with a two-cell result vector too), `ROWS` of it is `#VALUE!`, and a matrix function treats it as a blank
matrix cell (arrays.md "MDETERM / MINVERSE: a blank cell in the matrix is #VALUE!"): `MMULT(LOOKUP(TRUE,Data!B2:B10,Data!E11:E11),2.5)`
and `WEEKDAY(MMULT(LOOKUP(..),Data!C$2))` are `#VALUE!` (TRUE matches B10, the 9th cell).
Evidence: probes/lookup-short-result-vector A19-A26 (excel-graph, 2026-10-21, T-2278); gen-m6/50f0dd9a8ee2cb57 Calc!A315.
The same gen-m6 workbook's A204, `DATE(2088,MATCH(<blank>,Data!E4:E12,-2.2),30)` = 68757, is the descending scan above
("MATCH descending search scans from the top"), and e3a562557696e2fc A33 is "XLOOKUP: an error lookup_value beats a size mismatch".
So are the other empty values: an omitted argument (`MMULT(,2)`), IFERROR of a blank (`MDETERM`, `MMULT` and `MINVERSE` of
`IFERROR(<blank>,1)`) and `MMULT(IF(TRUE,<blank>),2)` are `#VALUE!` (probes/lookup-t2278-followups A1-A5, excel-graph, 2026-10-22).
Implemented in: `Model::eval_matrix_arg` (matrix.rs) reads the empty value as blank, as a blank cell.
Status: VERIFIED (excel-graph)
Tasks: T-2278

## XMATCH and XLOOKUP floor a fractional mode, snapping near an integer (T-2656)
Rule (excel-graph, 2026-10-25): XMATCH and XLOOKUP read `match_mode` and `search_mode` the way DATE and TIME read their
arguments (dates.md "TIME: an argument past 32767 is `#NUM!`"): the value is floored toward minus infinity. A value just
below an integer counts as that integer: within `NEAR_INTEGER_TOLERANCE` (2^-22 + 2^-33) for a positive value, and half of
that (2^-23 + 2^-34) for a negative one. Neither the 15-digit cut of INT nor truncation applies. So `0.5^1E-15`
(0.9999999999999993), `1-1E-7`, 0.9999999999999 and 1.99999999999999 read 1, 1, 1 and 2, while `1-2.5E-7` reads 0.
`XMATCH(FALSE,{1,2,3},0,0.5^1E-15)` is therefore `#N/A`, not `#VALUE!`. Ties and other fractions floor: 0.5 reads 0, 1.5 reads 1,
2.5 and 2.6 read 2, while -0.4 and -0.5 read -1, -1.5 reads -2 and -2.5 reads -3, all the same for both modes and both functions.
The edges are bracketed to a relative 2^-14 (an absolute ~2^-36), and the engine constants sit on the snapping side:
- `1-2^-22*(1+2^-11)` reads 1 and `1-2^-22*(1+9*2^-14)` reads 0. This holds at that resolution below 1, 2 and 3 for
  XMATCH (snap-edge-fine; xlookup-mode-snap-edge column E) and for XLOOKUP's search_mode. The XMATCH 2^-25 grid
  agrees below 3 (2^-22 reads 3, 2^-22 + 2^-25 reads 2). The bound is absolute.
- `-(2^-23*(1+2^-11))` reads 0 and `-(2^-23*(1+9*2^-14))` reads -1, at that resolution below 0 and -1 for XMATCH and
  below 0, -1 and -2 for XLOOKUP.
- XLOOKUP's match_mode and search_mode snap at the same points as XMATCH's: `1-1E-7` and `-1-1E-8` snap, `1-2.5E-7` and
  `-1E-7` do not (probes/t2656-xlookup-mode-snap-edge, every non-regex cell).
A tiny negative mode reads 0 (`-1E-7`, `-1E-10`, `-1E-300`). TRUE reads 1, and the text "1" and "1.5" read 1. ±1E300 is
`#VALUE!`, as is any value that floors outside the valid modes.
A match_mode of 3 (or one that floors to 3) is the regex mode in both functions: `XMATCH(2.5,{1,2,3},3)` is `#N/A`, the
XLOOKUP twin returns its if_not_found, and the pattern "?" is `#VALUE!` in both ("XMATCH and XLOOKUP regex mode" below; T-2669).
Evidence: probes/t2656-xmatch-mode-near-integer, -rounding, -snap, -snap-boundary, -snap-bisect, -snap-grid, -snap-edge,
-snap-edge-fine, and probes/t2656-xlookup-mode-snap-edge (1138 cells, excel-graph, 2026-10-25);
probes/t2496-fin-subexpressions P!A1, P!A2; gen-m9/01523d247594952e Calc!A51.
Status: VERIFIED (excel-graph), within the brackets above. Ours matches every cell (XLOOKUP's regex mode since T-2669).
Implemented in: functions/util/near_integer.rs `near_integer_whole` (shared with DATE and TIME), read by `Model::xmatch_of`
(lookup_and_reference/xmatch.rs) and `xlookup_modes` (xlookup.rs).
Tasks: T-2656, T-2669

## XMATCH and XLOOKUP regex mode (T-2669)
Rule (excel-graph, 2026-10-25): `match_mode` 3 is a regular-expression match in XLOOKUP as in XMATCH.
- **Matching:** the pattern is found anywhere in a candidate's text (`"o"` finds "Dog"; anchors work: `"^o"` finds nothing,
  `"^c.t$"` finds "cat"), case-sensitively (`"dog"` skips "Dog"; `"(?i)dog"` finds it).
- **Text of the values:** both sides read as text the way a scalar reads as text (`calc_scalar_to_string`). A number is
  its text, so `"12"` and `"12\.5"` find 12.5, `"E"` finds 1E+20 and the lookup value `0.1+0.2` is the pattern "0.3". A
  boolean is TRUE or FALSE, so `"TRUE"` and the lookup value TRUE find TRUE but `"true"` does not. A blank cell is "", so
  `"^$"` finds a blank as well as a `""` cell, and the empty pattern finds the first candidate. A blank cell as the
  lookup value is the empty pattern too (`XMATCH(Z99,A1:A3,3)` is 1, with search_mode -1 it is 3).
- **No match:** an error candidate never matches (`"#N/A"` skips #N/A).
- **Errors:** an invalid pattern (`"("`) is `#VALUE!`, never XLOOKUP's if_not_found. A binary search_mode (2 or -2) is
  `#VALUE!` in regex mode, as in wildcard mode.
- **Search direction:** search_mode -1 finds the last match.
- **Lifting:** a lifted lookup_value lifts one search per element (`XLOOKUP({"o";"zzz";"^c"},...,"nf",3)` spills 20, "nf",
  10).
- **Syntax:** the pattern syntax is PCRE2's, as in the REGEX* functions. In Excel, lookbehind `(?<=D)og` and `(?<!D)og`,
  lookahead `o(?=g)` and `d(?!o)`, backreferences `(a)\1\1`, `foo\Kbar`, possessive `a++a` and atomic `(?>a+)a`
  (both no match on "aaa"), `(?i)`, `\d{2}$`, `[[:digit:]]` and `\p{Lu}` all behave as in REGEXTEST
  (probes/t2680-lookup-regex-pcre2). Ours runs the lookups on the REGEX* engine since T-2680; before that, the `regex`
  crate refused the PCRE2-only forms with `#VALUE!`.
- **DEVIATION (decision 0014):** Excel answers every regex lookup. Ours bounds one XMATCH or XLOOKUP call, all its lifted
  elements together, by the REGEX* call limits (text.md "Lifted TEXTSPLIT, TEXTBEFORE, TEXTAFTER and REGEX* calls are
  capped"): 128 MiB of compile limits, 64 MiB held, and 2^30 units of search work, about 1 s of the slowest searches.
  Past any of them the whole call is `#ERROR!`, as a REGEX* call is: a lookup whose backtracking pattern scans long texts,
  `XMATCH("\G(a|a)*b",REPT("a",30000)&SEQUENCE(1000),3)`, is `#ERROR!` (about 0.5 s) where Excel would give `#N/A`, as no
  text holds a "b" (unprobed). So are the REGEX* parse
  limits (the 250-level nesting `#VALUE!`, the case-fold budget). Before T-2680 a lookup's regex was bounded only by the
  lift's cell-visit cap and the `regex` crate's size limit and linear-time search, not by a REGEX*-style budget.
Evidence: probes/t2669-xlookup-regex-mode (60 cells: XLOOKUP in D beside XMATCH in E), -blanks (14 cells) and -lift (12
cells: lifting, a blank-cell pattern, PCRE2 syntax) and probes/t2680-lookup-regex-pcre2 (45 cells); the regex
cells of probes/t2656-xmatch-mode-near-integer G7, G21, -rounding G15, t2656-xlookup-mode-snap-edge C8.
Status: VERIFIED (excel-graph). Ours matches every cell. Before T-2669, XLOOKUP rejected mode 3, XMATCH read a boolean as
"true", and a non-text lookup value gave no pattern; before T-2680, PCRE2-only syntax was `#VALUE!`.
Implemented in: functions/lookup_and_reference/xmatch.rs `regex_pattern` and `regex_scan`, used by XMATCH's
`linear_search` and XLOOKUP's (xlookup.rs). They match on functions/text/regex_functions.rs `LookupRegexes`: one REGEX*
`PatternCache` per XMATCH or XLOOKUP call, shared by its lifted elements (repeated patterns compile once), so a call has
one REGEX* call's compile and work budget (SPEC-0013).
Tasks: T-2669, T-2680

### Statistical aggregates over a lifted OFFSET (an array of references)
Evidence: probes/t2065-lift-* (one formula per workbook, excel-graph 2026-10-26; data as probes/reference-lift-consumers-b2).
- `STDEV.S(OFFSET(A1,{0;1},0))` is 70.0035713374682, `VAR.S` 4900.5, `SUMSQ` 10001: single-cell elements read as their values (multi-cell elements unprobed).
- COUNTIFS, AVERAGEIF and MAXIFS lift per element, spilling one value each (1, 1; 1, 50.5; 1, 100). CONCAT is `#VALUE!`.
  The engine implements these three since T-2823 (`Function::per_reference_element`, each element charged as
  COUNTIF/SUMIFS are, with the same bound DEVIATION); AVERAGEIFS and MINIFS are unprobed and read the value view.
  Which arguments lift is inferred from SUMIF/SUMIFS (COUNTIFS every criteria range, AVERAGEIF range and average_range,
  MAXIFS max_range and every criteria range); forms with only some ranges lifted are unprobed (T-2833).
- `AVERAGEA` and `MAXA(OFFSET(A4,{0;1;2},0))` over "5", blank, TRUE are 0: the text is not read as 5 and TRUE not as 1; which elements count is open (the data holds no number).
Status: VERIFIED (excel-graph) for the values above. COUNTIFS, AVERAGEIF and MAXIFS implemented (T-2823); the rest
not yet (T-2824).

### MEDIAN, LARGE and SMALL over a lifted OFFSET
probes/t2065-lift-median, -large, -small failed to open on Excel for the web on 2026-10-26 (createSession "retries exhausted", two runs).
Status: UNVERIFIED (desktop question, M14 T-1075 item 18, decision 0039).
Tasks: T-2065, T-2063, T-2064, T-2823, T-2824

## MATCH match_type that is a reference to an error cell is #REF! (excel-graph)
Rule: `MATCH(30,A1:A5,M2)` with M2 `=NA()` gives `#REF!` (ERROR.TYPE 4) in Excel for the web, not the #N/A the
error carries; the literal `NA()` as match_type and the same reference as VLOOKUP/HLOOKUP range_lookup propagate #N/A.
Evidence: probes/m2-argument-kind-gaps Calc!A25 (T-2861, excel-graph, 2026-10-27); the other 78 cells of that case (empty text,
error, fraction, boolean and numeric text as literal and as a reference in DAY, YEAR, IF, INDEX, MATCH, VLOOKUP, HLOOKUP, LEFT, RIGHT,
MID, SUMIF, SUMIFS) match ours.
This extends Rule 2 of "MATCH descending search scans from the top; a text match_type read from a cell is #REF!" and the VLOOKUP/HLOOKUP
reference-index rule: a non-numeric value (text, `=""`, an error) read from a cell in these arguments is #REF! (A22, A25; HLOOKUP row_index_num
M1/M2 = 4 at A32, A35, while the literals give 3 and 7 at A31, A33). T-2965 extends `match_type_arg`, not a parallel path.
All seven error kinds (#N/A, #DIV/0!, #NUM!, #REF!, #VALUE!, #NAME?, #NULL!) as the referenced cell give #REF!, also when
IF picks the reference and for a multi-cell range of error cells (E8, no implicit intersection: its row is outside the range): probes/match-type-error-reference
E1:E8, F1:F7 (excel-graph, 2026-10-28).
A literal error as match_type still propagates it (`NA()` gives #N/A, `#DIV/0!` gives #DIV/0!; probes/m2-argument-kind-gaps A23, A24).
Status: VERIFIED (web).
Tasks: T-2861, T-2965. Implemented in: `match_type_arg` (lookup_and_reference/mod.rs).
