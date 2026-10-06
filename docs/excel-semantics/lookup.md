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
2026-10-04. Never take oracle values as evidence for references to missing sheets
(docs/architecture/oracle.md, "Known divergences"; desktop wins per decision 0030).

## INDIRECT's a1 argument is coerced like a boolean; text must be TRUE/FALSE
Rule: `a1` is coerced like a logical argument: a nonzero number is TRUE (`INDIRECT("H2",2)` and
`INDIRECT("H2",-1)` = 5, `"TRUE"` text = TRUE), `0` is FALSE, so `INDIRECT("H2",0)` reads "H2" as R1C1
and is `#REF!`, while `INDIRECT("R2C8",0)` = 5. Empty-string text is `#VALUE!` in both styles (B4, B10) and `"x"` is `#VALUE!` in A1 style (B5).
Not probed: `"FALSE"` text and booleans read from cells.
Evidence: eval/suites/probes/cases/indirect-text-broadcast.case.json B3:B10 (oracle: excel-graph, 2026-10-05, T-0957).
Status: VERIFIED (Excel for the web only; the engine matches all of these)

## INDIRECT R1C1 relative offsets
Rule: `INDIRECT("R[-1]C[6]",FALSE)` is relative to the cell holding the formula (`R[-11]C[6]` in B12 reads H1, "data";
B11's `R[-1]C[6]` = 0 reads the empty H10). An offset that leaves the sheet is not `#REF!` (`R[-20]C` in B15 is 0, an
empty cell); whether it wraps or clamps is unknown, because the probe's target cell is empty. A result row exactly at the last row (`R[1048560]C` in B16)
is also fine.
RustSheet today: `parse_r1c1_number` (engine utils.rs) wraps and returns `#REF!` when the offset magnitude is
at least the sheet size; T-0960 should check both that and the wrap itself against the oracle.
Evidence: indirect-text-broadcast.case.json B11:B16 (oracle: excel-graph, 2026-10-05, T-0957).
Status: UNVERIFIED for the wrap-around itself (T-0960)
Tasks: T-0960

## MATCH on a 2-D lookup_array is #N/A; argument errors win
Rule: `MATCH` with a lookup_array of more than one row and column returns `#N/A` even when the value
is in it (`=MATCH(Data!C$5,Data!A5:C9,1)` with C5 = 2 inside A5:C9 → `#N/A`; no top-level 2-D MATCH
call in gen-m2 returns a number), but an error in lookup_value is returned first
(`=MATCH(MAX(MONTH(-0.5)),Data!A7:C9)` → `#NUM!`).
Evidence: gen-m2/cbdabe32a32a4012 Calc!A227, ce315bb699e1be6f Calc!A135, 629e6e8b76fab8b9 Calc!A313. Oracle: excel-graph, 2026-10-04 (gen-m2 sidecars, T-0203).
Status: VERIFIED for range references. UNVERIFIED sub-rule: a 2-D *array* (literal `{1,2;3,4}` or computed)
is assumed `#N/A` by analogy; no oracle case covers it (probe pending, T-0224).
Tasks: T-0210, T-0224

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
Evidence: probes/lookup-index-order H1–H50 (excel-graph, 2026-10-06, T-0211); gen-m2/1b6ceed7489e0800 A24,
2cfe279fe77a000d A85, 61cacc22c4244179 A310, caa981b0753f455c A182, 629e6e8b76fab8b9 A53,
cbdabe32a32a4012 A87 (Calc sheet; excel-graph, 2026-10-04, gen-m2 sidecars, T-0203).
Cut-off (probes/lookup-index-shapes H19-H36, excel-graph, 2026-10-06, T-0250): an index of
-2147483648 is `#REF!`, -2147483647, -65536, -32769, -32768 and -32767 are `#VALUE!`, in VLOOKUP and
HLOOKUP alike; not found is `#N/A` for all of them. A fractional index next to the cut-off is unprobed.
Index shapes (same probe): `OFFSET(E1,..)` and `INDIRECT("E1")` results are references (a cell holding
text or an error is `#REF!` when found, `#N/A` when not found). So is `INDEX(E1:E4,2)`: text or an
error in the INDEXed cell is `#REF!` found / `#N/A` not found, **not** implemented yet (the engine
coerces the INDEX value: `#VALUE!` / the error), baselined in probes H2, H3, H11, H12 (T-0250 follow-up).
A multi-cell range index is implicitly intersected: `VLOOKUP(2,A1:C3,@E1:E2)` from row 1 reads E1,
from row 2 reads E2 (text, so `#REF!`), from row 5 is `#VALUE!` (found), and also `#VALUE!` when the
value is not found (value index fails before the search); `VLOOKUP(2,A1:C3,E1:E2)` unspilled in J10 is
`#REF!` and `#N/A` when not found (K7); whether it spills into J11 is unprobed.
Status: VERIFIED
Implemented in: `Model::lookup_index_arg` / `Model::resolve_lookup_index`
(crates/rustsheet-engine/src/functions/lookup_and_reference/index_arg.rs), shared by VLOOKUP and HLOOKUP.
Tasks: T-0211

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
ascending (`MATCH(5,{9,5,5,5,1},-2)` = 2, `MATCH(2,…,2)` = `MATCH(2,…,1)`). A blank MATCH key is 0
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
Implemented in: `SearchVector::approximate_position`/`exact_position`; XMATCH exact mode skips
blanks (T-0126). XMATCH with match_mode other than 0 or binary search_mode over blanks is unprobed.

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
Evidence: poi/xlookup.xlsx (Excel-cached, Excel Online 16) Sheet1!C2:D2. Unprobed: the row-vector
analog and array (non-range) return arguments.
Status: VERIFIED (Excel-cached)
Open (unprobed, T-0962): reference-vs-array result (ISREF, implicit intersection by column, a blank matched cell
spilling as 0), `@`/nesting such as `UPPER(XLOOKUP(2,A1:A3,B1:C3))`, a one-cell lookup vector with a wide
return range, and a wide return range with a different row count (`#VALUE!` is a guess).
Implemented in: `Model::xlookup_pick` (functions/xlookup.rs); XLOOKUP's static-analysis result is `Unknown` since it can return an array.
Tasks: T-0948
