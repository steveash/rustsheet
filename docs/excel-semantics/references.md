# References and names

## 3-D sheet-range references in aggregates
Rule: SUM, AVERAGE, MIN, MAX, COUNT and COUNTA accept `First:Last!ref`, and the sheet span may
include the formula's own sheet.
Evidence: poi/FormulaSheetRange.xlsx test!D11 = 10, D12 = 60 and poi/48703.xlsx Sheet1!B2 = 20 (oracle:
excel-graph, 2026-10-04); poi/55906-MultiSheetRefs.xlsx A3 = 66, I6 = 4 (Excel-cached).
Status: VERIFIED
Tasks: T-0122

## Undefined names may contain backslashes
Rule: an undefined name such as `HSI\10_1\1460_1\1470_1___val` parses as a name and evaluates
to `#NAME?` (not a syntax error).
Evidence: poi/46535.xlsx Others!L12, M12, M17:M20 (oracle: excel-graph, 2026-10-04).
Status: VERIFIED
Tasks: T-0123

## External reference with no cached data is `#VALUE!`
Rule: `SUM('[link-external-workbook-a.xlsx]Sheet0'!A1:B1)` whose externalBook part holds no
cached data returns `#VALUE!`, not `#REF!`. Seen on Excel for the web only (the workbook was
written by Apache POI, so it holds no desktop evidence); needs excel-desktop confirmation before
anyone relies on it.
Evidence: poi/link-external-workbook-b.xlsx Sheet0!A1 (oracle: excel-graph, 2026-10-04).
Status: UNVERIFIED (excel-desktop probe pending; web: #VALUE!)
Tasks: T-0124
