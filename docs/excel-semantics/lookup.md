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

## Sorted lookups over a blank tail (whole column / row)
Rule (hypothesis): approximate-match search (MATCH type 1/-1, LOOKUP, VLOOKUP/HLOOKUP sorted,
XMATCH binary modes) ignores the empty cells after the data, so `MATCH(99,A:A,1)` with numbers
in A1:A3 is 3 (the widely used "last number in a column" idiom `MATCH(9.99E+307,A:A)`).
Evidence: none yet; probe cases are T-0126.
Status: UNVERIFIED (probe pending, T-0126)
Engine today: compares each probed empty cell as 0 / "" / FALSE, so these lookups over `A:A`
land on row 1048576 (`MATCH(99,A:A,1)` = 1048576, `VLOOKUP(99,A:B,2)` = 0). Exact matches treat
an empty cell the same way: `VLOOKUP(0,A:B,2,FALSE)` matches the first empty row. Do not
change either until T-0126 has oracle values.

## Whole-column / whole-row search vectors are walked only up to the used area (T-0119)
Not an Excel rule but a performance invariant: MATCH, LOOKUP, HLOOKUP and VLOOKUP evaluate a
search vector only up to the sheet's last used row (column, for ranges running to XFD); the
rest is a virtual run of empty cells of the full length (`SearchVector` in
`functions/lookup_and_reference/lookup_table.rs`), so results are those of the full walk
whatever the used area. Unit tests compare each case against a sheet whose used area reaches
the last row/column. (XMATCH still truncates via `clip_to_used_area`, T-0127.)
