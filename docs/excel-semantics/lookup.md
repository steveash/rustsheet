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
Unprobed: blank elements of a multi-cell lookup_value.
Evidence: probes/vlookup-array-lookup-value (35 cells, excel-graph, 2026-10-09, T-0270).
Implemented in: `Model::lookup_each` (functions/lookup_and_reference/lookup_each.rs) through
`Model::lift_values` (functions/lift/driver.rs); the spill shape follows lookup_value in static analysis.
Status: VERIFIED (web)

## VLOOKUP / HLOOKUP with an array index or range_lookup
Rule: lookup_value and range_lookup lift together, broadcast like any two arrays (a 3x1 lookup_value
against a 2x1 range_lookup gives `#N/A` in the third row; a column against a row gives 2-D). Each pair
is a scalar call, so an error or text range_lookup element is that element's result
(`VLOOKUP({2;3},A1:B4,2,{#DIV/0!;TRUE})` is `#DIV/0!`, "three"; `{"a";"b"}` is `#VALUE!` twice).
The index array lifts only when lookup_value and range_lookup are both single values: then a scalar
search runs, not found is `#N/A` (before any index is looked at), and otherwise every index element is
validated in reading order and the first bad one is the error of the whole call, with no spill
(`{0,9}` is `#VALUE!`, `{9,0}` is `#REF!`, `{2,"a"}` and `{2,#DIV/0!}` are `#REF!`, an element is read like
a referenced cell, so text and errors are `#REF!`); valid indexes give a same-shaped array
(`VLOOKUP(2,A1:B4,{2,1},FALSE)` is "two", 2; a fractional element truncates). When lookup_value or
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
Matches the engine (`LookupIndexArg::Failed`).
A multi-cell range *index* stays implicitly intersected (T-1426). Unprobed: which error wins when a
lookup_value element and a range_lookup element are both errors (the engine gives the lookup error),
and blank elements of a computed index array (read as 0, `#VALUE!`).
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
HLOOKUP alike; not found is `#N/A` for all of them. A fractional index next to the cut-off is unprobed.
Index shapes (same probe): `OFFSET(E1,..)` and `INDIRECT("E1")` results are references (a cell holding
text or an error is `#REF!` when found, `#N/A` when not found). So is `INDEX(E1:E4,2)`: text or an
error in the INDEXed cell is `#REF!` found / `#N/A` not found (probes H2, H3, H11, H12; T-0251).
A multi-cell range index is implicitly intersected: `VLOOKUP(2,A1:C3,@E1:E2)` from row 1 reads E1,
from row 2 reads E2 (text, so `#REF!`), from row 5 is `#VALUE!` (found), and also `#VALUE!` when the
value is not found (value index fails before the search); `VLOOKUP(2,A1:C3,E1:E2)` unspilled in J10 is
`#REF!` and `#N/A` when not found (K7); whether it spills into J11 is unprobed.
HLOOKUP twins (probes/lookup-index-hlookup H1-H6, excel-graph, 2026-10-07, T-0264): `INDEX(E1:E4,1)` (number)
gives the row, `INDEX(E1:E4,2)` (text) and `INDEX(E1:E4,3)` (error) are `#REF!` found and `#N/A` not found,
as in VLOOKUP. An INDEX index that itself fails (`INDEX(E1:E4,9)`) is `#REF!` in HLOOKUP and VLOOKUP, found
or not found (H7-H10): a failing index is not turned into `#N/A` by a failed search. Whether the INDEX error propagates as is or is always
mapped to `#REF!` is unprobed (an INDEX failing with another error, e.g. `INDEX(E1:E4,"x")`, would tell).
Status: VERIFIED
Implemented in: `Model::lookup_index_arg` / `Model::resolve_lookup_index`
(crates/rustsheet-engine/src/functions/lookup_and_reference/index_arg.rs), shared by VLOOKUP and HLOOKUP.
Tasks: T-0211, T-0264

## Approximate-match lookups: type-skipping binary search
Rule: VLOOKUP/HLOOKUP with range_lookup TRUE (or omitted), LOOKUP, and MATCH with a non-zero
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
               (the first index of the run, walking down, for a descending search)
    if (c < 0 ascending) or (c > 0 descending): best = m; lo = m+1
    else: hi = m-1
return best (none -> #N/A)
```
Blanks never match and there is no cross-type match (`VLOOKUP(TRUE,{blank;-1},1,TRUE)` is `#N/A`).
The run of equal cells stops at the first cell that is not equal, whatever its type
(`MATCH(2,{0,0,0,2,"x",2,9})` = 4). Any negative match_type searches descending and any positive one
ascending (`MATCH(5,{9,5,5,5,1},-2)` = 2, `MATCH(2,…,2)` = `MATCH(2,…,1)`). Only the sign of a fractional match_type counts: `-0.5` and `-0.0001` search descending (gen-m2/f7e6d351c8538ef5 Calc!A160, gen-m2/96a0952412cefa6c Calc!A241, T-1304) and `0.5`, `0.0001` and `0.999` search ascending, not exact (`MATCH(2.5,{1,2,3,4,5},0.5)` = 2; probes/match-fractional-match-type C1-C3, T-1391). A blank MATCH key is 0
in an approximate search (`MATCH(blank,{1,blank,0,blank,2})` = 3).
Exact match (range_lookup FALSE, MATCH 0) only matches cells of the key's type: a blank cell never
matches 0 or "" (`MATCH(0,{blank;blank;blank},0)` and `VLOOKUP("",{blank;blank;blank},1,FALSE)` are
`#N/A`), while a blank lookup value is 0 here too (`MATCH(blank,{1,blank,0,blank,2},0)` = 3,
`MATCH(blank,{"";0},0)` = 2). Numbers must be exactly equal, without the 15-digit rounding of the
comparison operators: `MATCH(0.1+0.2,{0.3},0)` and `VLOOKUP(0.1+0.2,{0.3},1,FALSE)` are `#N/A`.
A match_type that is not a number is `#VALUE!` even with a blank key (`MATCH(blank,…,"x")`).
Text comparison uses Excel's collation, not code-point order (T-0220).
Evidence: eval/suites/probes/cases/lookup-approximate-search.case.json N1–N44 (oracle: excel-graph,
2026-10-06, T-0212); gen-m2/fa437879253827be A146 (`=HLOOKUP(-2.5,Data!B3:E7,4)` → 2), cbdabe32a32a4012 A87,
a6d4377c7b1b92cd A33 (`MATCH(…,-1)` → 1), 04cfd82f54dbc2b2 A1 and d0c29dd5c530e28c A67 (`#N/A`)
(excel-graph, 2026-10-04, T-0203); 419 gen-m2 cells fixed by T-0212;
crates/rustsheet-xlsx/tests/calc_tests/LOOKUP_AND_REFERENCE/MATCH.xlsx C7 (duplicate run, Excel-cached).
Status: VERIFIED
Implemented in: `SearchVector::approximate_search` and `SearchVector::exact_position` in
`crates/rustsheet-engine/src/functions/lookup_and_reference/lookup_table.rs`
Tasks: T-0212, T-0220

## MATCH descending search skips backward; a text match_type read from a cell is #REF!
Rule 1: in a descending search (negative match_type) the probe step that lands on a cell of another
type moves *backward* to the nearest cell of the key's type, not forward as in the ascending
pseudo-code above. `MATCH(0.5,{0.1,TRUE,3.14159},-1)` is `#N/A` (backward skip reaches 0.1, which is
below the key, so nothing qualifies; a forward skip would reach 3.14159 and give 3), while
`MATCH(0.5,{3,TRUE,0.1},-1)` = 1, `MATCH(2,{5,"x",3,1},-1)` = 3 and `MATCH(2,{5,TRUE,TRUE,1},-1)` = 1
(the range forms agree). One backward-skip reading fits all four; the evidence does not exclude
other rules for a skip that passes the low bound. When the backward skip finds no cell of the key's
type between the live range's low bound and the midpoint, the search continues forward from the
midpoint: `MATCH(-14,{FALSE,0.5},-1)` = 2 (gen-m2 b4392a6514a36ade Calc!A277; a backward-only skip
gives #N/A). The forward continuation is verified by 20 further cells (probes/match-descending-forward-continuation J1-J20, excel-graph, T-1396):
with a prefix of text, boolean or error cells before a run of numbers (or text), the descending
search behaves as a plain binary search over the cells of the key's type (J1, J3 and J11-J16 have nothing of the key's type at or below the midpoint; the longer vectors J2 and J7-J10 only exercise the ordinary search after a mixed prefix). `MATCH(3,{"a","b","c",5,3,1},-1)` = 5,
`MATCH(1,{TRUE,FALSE,"c",9,7,5,3,1},-1)` = 8, `MATCH("b",{1,TRUE,2,"c","b","a"},-1)` = 5, `{#N/A,#N/A,5,4,3,2}` and
prefixes of 2-4 text, boolean or mixed cells (J11-J14) give the same positions as the typed tail alone; an inexact key picks the last cell >= key
(`MATCH(4.5,{"a","b","c",9,8,7,6,5,4,3},-1)` = 8) and a key above everything is #N/A (`MATCH(1,{"a","b",0.5,0.2},-1)`);
range forms (a column and a row) agree. The engine already matches all 20 cells.
Rule 2: a literal text match_type is `#VALUE!` (`MATCH(1,A5:A7,"x")`, and also on a 2-D range:
`MATCH(1,A5:B7,"x")`), but a text match_type that comes from a *cell* is `#REF!`, for a 1-D and a 2-D
lookup_array alike (`MATCH(1,A5:A7,D1)` with D1 = "a", `MATCH(1,A5:B7,D1)`, and D2 = " x "). The 2-D
`#N/A` rule above still holds for numeric match_type (`MATCH(1,A5:B7,0)` and `,1` are `#N/A`).
`INDEX` returns that `#REF!` (`INDEX(A5:B7,MATCH(1,A5:A6,D1),3)` and `,1`), while a literal-text
match_type inside it is `#VALUE!` and `INDEX(A5:B7,NA(),3)` is `#N/A` (the row error wins over the
out-of-range column, which alone is `#REF!`).
Rule 3: the cell stays a reference through IF, IFS, CHOOSE, SWITCH and LET, so `MATCH(2,A5:A7,IFS(TRUE,D1))`
with D1 = "x" is `#REF!` too (as are the same with IF, CHOOSE, SWITCH, `LET(a,D1,a)`); D2 = 1
gives 2. IFERROR returns the value, so `IFERROR(D1,0)` is `#VALUE!`, like literal text
(probes/match-type-text-passthrough K1:K9, excel-graph, T-1399).
Nesting of IF, IFS and SWITCH inside each other changes nothing (IFS or SWITCH inside CHOOSE or a LET body is unprobed): `IFS(TRUE,IF(TRUE,D1))`, `IF(TRUE,IFS(TRUE,D1))`, `IFS(TRUE,SWITCH(1,1,D1))` and the SWITCH default branch (`SWITCH(9,1,D2,D1)`) are `#REF!` for D1 = "x" (D2 = 1 gives 2).
A multi-cell range is its top-left cell, plain or picked by IF, IFS, SWITCH, LET or CHOOSE (no implicit intersection): with D1 = "x", D2 = 1, D3 = "y", `MATCH(2,A5:A7,D1:D2)` is `#REF!` and `D2:D3` gives 2 (the IF-picked forms K2 and K3, in rows 2 and 3, rule out implicit intersection). A one-sheet 3-D span (`Sheet1:Sheet1!D1`) behaves as the cell
(probes/match-type-nested-passthrough K1:K17, excel-graph, T-1452).
Evidence: eval/suites/probes/cases/match-descending-argcheck.case.json J1–J20 (oracle: excel-graph,
2026-10-10, T-1305); gen-m2/f7e6d351c8538ef5 Calc!A334, b4392a6514a36ade Calc!A277, 04cfd82f54dbc2b2 Calc!A81, f9d4df4efdfbcbd5 Calc!A14.
Implemented in: `fn_match` / `match_type_arg` in `crates/rustsheet-engine/src/functions/lookup_and_reference/mod.rs` (Rule 2: non-numeric text from a cell only; numeric text and booleans in the cell are unprobed). Rule 3: `match_type_arg` evaluates through `evaluate_chosen_reference` (util.rs), which IF, IFS, SWITCH, LET and CHOOSE share; aggregates such as SUM are deliberately unchanged (no oracle evidence).
Status: VERIFIED
Tasks: T-1305, T-1312, T-1313, T-1399, T-1452

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
CHOOSECOLS/CHOOSEROWS index list (ours: the first error, out-of-range index or uncoercible text in row-major order).
Evidence: probes/lookup-argument-modes (117 cells, excel-graph, 2026-10-09, T-1238); ADDRESS's a1 and HYPERLINK's
friendly_name over an array spill: gen-m7/f68fafa0ca2eba26 A7, gen-m7/2bb922eef72e785e A22 (excel-graph, 2026-10-11).
Ours: ADDRESS, HYPERLINK, LOOKUP's and XMATCH's lookup_value, XMATCH's match_mode and XLOOKUP lift, FORMULATEXT gives
one result per cell, and CHOOSECOLS/CHOOSEROWS read their index lists whole (T-1241). OFFSET, INDIRECT and TRIMRANGE
still do not lift (their elements are references; T-1810), and INDEX of a single value is `#VALUE!` (T-1242).
Status: VERIFIED (web)
Tasks: T-1238 (generator modes), T-1241 (lifts and index lists), T-1810 (reference-valued lifts), T-1242 (INDEX of a single value)

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
An array-valued match_type also lifts, broadcast with lookup_value (`ROWS(MATCH(2,A1:A3,{0;1}))` is 2, `INDEX(MATCH(B1:B2,A1:A3,{0;1}),1)` is 3); a multi-cell range match_type still reads its top-left cell (probes/match-type-nested-passthrough), and static analysis spills only for an array one.
A scalar error lookup_value lifts over an array match_type too: `ROWS(MATCH(NA(),A1:A3,{0;1}))` is 2, `INDEX(MATCH(NA(),A1:A3,{0;1}),2)` is
`#N/A`, and `#DIV/0!` fills both slots of `MATCH(#DIV/0!,..,{0;1})`; `MATCH(25,A1:A3,{0;"x"})` is `#N/A` then `#VALUE!` (the second
slot's text match_type, not the error, decides).
Evidence: probes/match-lookup-value-lift (19 cells; excel-graph, 2026-10-18, T-1866); probes/match-error-lookup-value-array-match-type (8 cells; excel-graph, 2026-10-18, T-1926); gen-m7/4629e931955aa4b8 Calc!A19 (excel-graph, T-1361).
Status: VERIFIED
Implemented in: `Model::fn_match` (functions/lookup_and_reference/mod.rs) through `lift_values`; static analysis (`match_static_result`) spills by lookup_value and an array match_type.
Tasks: T-1866, T-1917, T-1926

## A reference-valued lookup_value pointing at a blank or error cell
Rule: `CHOOSE(1,J5)`, `OFFSET(J5,0,0)`, `INDIRECT("J5")`, `J5:J5` and the direct `J5` behave identically in VLOOKUP, HLOOKUP,
MATCH, XMATCH, XLOOKUP and LOOKUP.
- An error cell (`#DIV/0!`, `#N/A`) is returned as that error by all eleven forms (exact and approximate, every shape).
- A blank cell as the key, against an all-number table (10..50, no 0, no blanks), gives `#N/A` in every exact and approximate form
  (VLOOKUP, HLOOKUP, MATCH 0/1, XMATCH, XLOOKUP, LOOKUP), except the next-smaller modes of XMATCH and XLOOKUP, which take the last
  row: `XMATCH(J5,A1:A5,-1)` is 5 and `XLOOKUP(J5,A1:A5,B1:B5,"nf",-1)` is "e". That is the existing blank-key behavior, not a new
  rule: the blank is the number 0 in the approximate lookups (below 10, so `#N/A`; see "Approximate-match lookups") and larger than
  every number in XMATCH/XLOOKUP (so -1 takes the last row; see "XMATCH / XLOOKUP linear search over blanks"). The probe has no 0,
  `""` or blank candidates, so it does not test those against a reference-shaped key.
Evidence: probes/lookup-reference-lookup-value-blank-error (165 cells: 5 shapes x {blank, #DIV/0!, #N/A} x 11 lookups;
excel-graph, 2026-10-12, T-1371). The engine already matches every cell.
Status: VERIFIED
Tasks: T-1371

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
`NEAR_INTEGER_TOLERANCE` (functions/util.rs). The negative cut is half the positive bound to within 1E-14
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
Implemented in: `choose_array` (functions/lookup_and_reference/mod.rs), using IF's `evaluate_if_arg`/`if_arg_at`
(functions/logical/mod.rs).
Tasks: T-1508

## XMATCH/XLOOKUP reference-form lookup and return arrays; capital Thorn
A single-cell reference is a one-cell vector for XMATCH `lookup_array` and XLOOKUP `lookup_array`/`return_array`: `XMATCH(5,A1)` is 1, `XMATCH(5,one)` (single-cell defined name) is 1, `LET(r,A1,XMATCH(5,r))` is 1 and `XLOOKUP(5,A1,B3)` returns B3's value. A blank single-cell `return_array` yields 0 (`XLOOKUP(5,one,B2)` is 0). `XMATCH(10,@A1:A3)` is `#N/A` (on row 1 the `@` yields the scalar 5, not a vector containing 10) and `XLOOKUP(5,A1,@B1:B3)` on row 7 is `#VALUE!` (the `@` finds no cell on row 7 to intersect, so the error arises before XLOOKUP runs; the probe cannot separate that from a scalar `return_array`). A 3-D span `Sheet1:Sheet3!A1` as lookup_array or return_array is `#VALUE!` (not `#REF!`). `"Þ"="TH"` is TRUE and `"Þ"<"TI"` is TRUE (Thorn collates as TH), so `XMATCH("Þ",D1:D3)` is 1, `XMATCH("TI",D1:D3,1)` is 3.
Evidence: probes/xmatch-xlookup-reference-forms G1, H1-H16, probes/xmatch-xlookup-3d-lookup-array H1-H3 (excel-graph, 2026-10-16, T-1698).
Status: VERIFIED (excel-graph); the engine already matches every cell.
Tasks: T-1698
