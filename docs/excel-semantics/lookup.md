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
(docs/architecture/oracle.md, "Known divergences"; which Excel wins is T-0025).

## MATCH on a 2-D lookup_array is #N/A; argument errors win
Rule: `MATCH` with a lookup_array of more than one row and column returns `#N/A` even when the value
is in it (`=MATCH(Data!C$5,Data!A5:C9,1)` with C5 = 2 inside A5:C9 → `#N/A`; no top-level 2-D MATCH
call in gen-m2 returns a number), but an error in lookup_value is returned first
(`=MATCH(MAX(MONTH(-0.5)),Data!A7:C9)` → `#NUM!`).
Evidence: gen-m2/cbdabe32a32a4012 Calc!A227, ce315bb699e1be6f Calc!A135, 629e6e8b76fab8b9 Calc!A313. Oracle: excel-graph, 2026-10-04 (gen-m2 sidecars, T-0203).
Status: VERIFIED
Tasks: T-0210

## VLOOKUP / HLOOKUP search before validating the index
Rule: the value is looked up first. Not found is `#N/A` whatever the column/row index
(`=VLOOKUP("a",Data!E2:E6,3)` → `#N/A`; index -2.5 too). Found, a non-numeric text index is `#REF!`
(not `#VALUE!`), an index of -1E+15 is `#REF!`, and "1.5" coerces to 1.
Evidence: gen-m2/1b6ceed7489e0800 A24, 2cfe279fe77a000d A85, 61cacc22c4244179 A310,
caa981b0753f455c A182, 629e6e8b76fab8b9 A53, cbdabe32a32a4012 A87 (Calc sheet). Oracle: excel-graph, 2026-10-04 (gen-m2 sidecars, T-0203).
Status: VERIFIED
Tasks: T-0211

## Approximate-match lookups: type-skipping binary search
Rule (fits all 124 simple top-level approximate lookups in gen-m2): binary search over the range;
when the middle cell is blank or of another type than the lookup value, step to the next cell of the
same type upward; if none exists up to the upper bound, set the upper bound to mid-1. A greater cell
sets the upper bound to mid-1; an equal cell is returned at once. Blanks never match and there is no
cross-type match. Full algorithm and cells in T-0212.
Evidence: gen-m2/fa437879253827be A146 (`=HLOOKUP(-2.5,Data!B3:E7,4)` → 2), cbdabe32a32a4012 A87,
a6d4377c7b1b92cd A33 (`MATCH(…,-1)` → 1), 04cfd82f54dbc2b2 A1 and d0c29dd5c530e28c A67 (`#N/A`). Oracle: excel-graph, 2026-10-04 (gen-m2 sidecars, T-0203).
Status: VERIFIED (behavior); the step rule is a fitted model, re-check it against T-0212's cells.
Tasks: T-0212

## INDEX with one index into a 2-D range is #REF!
Rule: `INDEX(range2d, n)` with only a row index (including 0 and 1E-15) is `#REF!`:
`=INDEX(Data!A1:B2,0)`, `=INDEX(Data!A1:D8,1e-15)` (85 of 90 calls; the rest are argument errors).
`INDEX(column,0)` on a single column spills (`#SPILL!` when blocked).
Evidence: gen-m2/60a543744fc4a8f7 Calc!A106, 893302d6c5b99e2e Calc!A265, 78bc88199fc96650 Calc!A61. Oracle: excel-graph, 2026-10-04 (gen-m2 sidecars, T-0203).
Status: VERIFIED
Tasks: T-0213
