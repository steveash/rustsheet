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
is `#ERROR!`). UNVERIFIED: an array col_index or range_lookup looks unlifted (a first oracle fill showed
`VLOOKUP({2;3},A1:B4,{1,2},FALSE)` spilling only 2; 3); the cells were moved to T-1269, which re-probes
them (the engine gives `#N/IMPL!`).
Unprobed: blank elements of a multi-cell lookup_value, and an error or bad-text index or range_lookup with an
array lookup_value (the engine gives one scalar error before lifting; T-1269 probes it).
Evidence: probes/vlookup-array-lookup-value (35 cells, excel-graph, 2026-10-09, T-0270).
Implemented in: `Model::lookup_each` (functions/lookup_and_reference/lookup_each.rs) through
`Model::lift_with_cost`; the spill shape follows lookup_value in static analysis.
Status: VERIFIED (web)

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

## MATCH descending search skips backward; match_type read from a cell is checked as a reference
Rule 1: in a descending search (negative match_type) the probe step that lands on a cell of another
type moves *backward* to the nearest cell of the key's type, not forward as in the ascending
pseudo-code above. `MATCH(0.5,{0.1,TRUE,3.14159},-1)` is `#N/A` (backward skip reaches 0.1, which is
below the key, so nothing qualifies; a forward skip would reach 3.14159 and give 3), while
`MATCH(0.5,{3,TRUE,0.1},-1)` = 1, `MATCH(2,{5,"x",3,1},-1)` = 3 and `MATCH(2,{5,TRUE,TRUE,1},-1)` = 1
(the range forms agree). One backward-skip reading fits all four; the evidence does not exclude
other rules for a skip that passes the low bound.
Rule 2: a literal text match_type is `#VALUE!` (`MATCH(1,A5:A7,"x")`, and also on a 2-D range:
`MATCH(1,A5:B7,"x")`), but a text match_type that comes from a *cell* is `#REF!`, for a 1-D and a 2-D
lookup_array alike (`MATCH(1,A5:A7,D1)` with D1 = "a", `MATCH(1,A5:B7,D1)`, and D2 = " x "). The 2-D
`#N/A` rule above still holds for numeric match_type (`MATCH(1,A5:B7,0)` and `,1` are `#N/A`).
`INDEX` returns that `#REF!` (`INDEX(A5:B7,MATCH(1,A5:A6,D1),3)` and `,1`), while a literal-text
match_type inside it is `#VALUE!` and `INDEX(A5:B7,NA(),3)` is `#N/A` (the row error wins over the
out-of-range column, which alone is `#REF!`).
Evidence: eval/suites/probes/cases/match-descending-argcheck.case.json J1–J20 (oracle: excel-graph,
2026-10-10, T-1305); gen-m2/f7e6d351c8538ef5 Calc!A334, 04cfd82f54dbc2b2 Calc!A81, f9d4df4efdfbcbd5 Calc!A14.
Status: VERIFIED
Tasks: T-1305, T-1312, T-1313

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
Evidence: poi/xlookup.xlsx (Excel-cached, Excel Online 16) Sheet1!C2:D2. Unprobed: the row-vector
analog and array (non-range) return arguments.
Status: VERIFIED (Excel-cached)
Open (unprobed, T-0962): reference-vs-array result (ISREF, implicit intersection by column, a blank matched cell
spilling as 0), `@`/nesting such as `UPPER(XLOOKUP(2,A1:A3,B1:C3))`, a one-cell lookup vector with a wide
return range, and a wide return range with a different row count (`#VALUE!` is a guess).
Implemented in: `Model::xlookup_pick` (functions/xlookup.rs); XLOOKUP's static-analysis result is `Unknown` since it can return an array.
Array lookup_array (T-1126, T-1127): `XLOOKUP(2,{1;2;3},B1:B3)` looks the value up in the array and picks the return range's cell, a
reference (`ISREF` is TRUE; probes/isref-reference-lookups H10, excel-graph). All of the following are from
probes/xlookup-array-return (excel-graph): an array return_array gives its element (`XLOOKUP(2,{1;2;3},{4;5;6})` is 5, E1; row arrays E4),
`if_not_found` and `#N/A` work as for ranges (E2, E3); the binary search modes 2 and -2 work over an array lookup_array with a range or
array return_array, including match modes -1/1 (E9-E15); search modes 1 and -1 explicit (E22-E24). Shape rules: a 2-D lookup array is
`#VALUE!` (E8); the return_array needs the lookup array's row count, so a column lookup array with a row return (E16), a row lookup array
with a column return (E18, E6) or a shorter return (E5, E21) is `#VALUE!`; a column lookup array accepts a wider return_array and gives the
matched row (E7, E19, E20). Still unprobed: an error or scalar return_array, wildcard match mode with a binary search mode, and a 2-D *range* lookup_array (the engine answers `#ERROR!` there, `#VALUE!` for an array).
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
XLOOKUP's if_not_found, ADDRESS's a1 and sheet_text, INDIRECT's a1 and TRIMRANGE's trim_cols.
Evidence: probes/lookup-argument-modes (117 cells, excel-graph, 2026-10-09, T-1238).
Ours: none of the lifting arguments above lifts except CHOOSE's (a range gives `#VALUE!` or an implicit intersection,
an array `#N/IMPL!`); LOOKUP/XLOOKUP/XMATCH give `#N/A` for a range lookup_value; CHOOSECOLS and CHOOSEROWS are `#VALUE!`
for a range of indexes and `#N/IMPL!` for an array; HYPERLINK and INDIRECT over an array literal are `#N/IMPL!`;
FORMULATEXT over a range is `#N/IMPL!`; INDEX of a single value is `#VALUE!`. 34 of 117 cells pass.
Status: VERIFIED (web)
Tasks: T-1238 (generator modes), T-1241 (lifts and index lists), T-1242 (INDEX of a single value)

## XLOOKUP with an array lookup_value looks up each element
`SUM(XLOOKUP({1;2},A1:A3,B1:B3))` is 30 with A1:A3 = 1,2,3 and B1:B3 = 10,20,30: an array `lookup_value` gives one result per element (10, 20).
Evidence: probes/let-name-uncalled-picked-refs H21 (excel-graph, 2026-10-09, T-1252). Other shapes (row, 2-D, with if_not_found) are not probed.
Ours: `#N/A` (a known failure, T-1264).
Status: VERIFIED (web)
Tasks: T-1252, T-1264
