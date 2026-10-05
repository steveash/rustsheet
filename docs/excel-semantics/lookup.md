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

## Whole-row / whole-column search vectors (T-0119)

Empty cells never match an exact lookup, so MATCH/LOOKUP/HLOOKUP/VLOOKUP/XMATCH search only up to the sheet's last used row/column of a `A:A` / `1:1` reference. For approximate (sorted) search this also means the empty tail is ignored; the sorted-tail behaviour on an unbounded range has not been oracle-probed (needs an oracle probe of `MATCH(99,A:A,1)` with trailing blanks).
