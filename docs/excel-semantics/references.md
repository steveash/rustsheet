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
Evidence: poi/46535.xlsx Others!L12, M12, M17:M20 (oracle: excel-graph, 2026-10-04); the
StructuredReferences.xlsx table names containing `\` also parse (9 cells).
Deviation note: in comma-decimal locales `\` is the array column separator, so the lexer only reads it
as a name character when the decimal separator is `.`. That rule is an assumption, not probed.
Status: VERIFIED (en-US); comma-decimal locales UNVERIFIED
Tasks: T-0123

## External reference with no cached data is a rich-value error (undecoded)
Rule: `SUM('[link-external-workbook-a.xlsx]Sheet0'!A1:B1)` whose externalBook part holds no cached
data does not give `#REF!`. Excel for the web writes the cell as a vm-coded rich-value error,
errorType 10 / subType 2 (rv `10 2`, errorType-first structure), cached as `#VALUE!`. Which error
that is (by the 8 = `#SPILL!`, 13 = `#CALC!` pattern errorType 10 may be `#BLOCKED!`) needs an
ERROR.TYPE probe; until then the harness marks it `#UNDECODED!` and does not score it.
Evidence: poi/link-external-workbook-b.xlsx Sheet0!A1 (cached oracle workbook
eval/.cache/oracle/excel-graph/6e0e5c2a…2361.xlsx, 2026-10-04). The workbook was written by Apache
POI, so there is no desktop evidence.
Status: UNVERIFIED (errorType 10 decoding and desktop behavior pending; T-0124, T-0030)
Tasks: T-0124, T-0030

## IF and INDEX return references to aggregates; IFERROR returns values
Rule: inside SUM/AVERAGE/MIN/MAX, `IF(…,ref)` and `INDEX(…)` pass a reference, so text and blanks in
it are skipped like any range cell: `=ABS(SUM(IF(FALSE,-2.5,Data!E10),Data!D4:D7))` with E10 = "abc"
→ 123456799.623; `=SUM(Data!E7:E7,INDEX(Data!B11:C11,1))` → 0.1. IFERROR returns a value, so text
from it counts as a text argument: `=SUM(Data!B9:B9,MIN(IFERROR(Data!$B$5,Data!D9)))` with B5 = "a"
→ `#VALUE!` (MIN of a text value; see coercion.md). Whether IFERROR of a blank cell counts as 0 is
not yet shown by any cell.
Evidence: gen-m2/96a0952412cefa6c A88, 942bdade381b7537 A212, dd99be1f2454159f A264,
04cfd82f54dbc2b2 A182 (Calc sheet). Oracle: excel-graph, 2026-10-04 (gen-m2 sidecars, T-0203).
Status: VERIFIED
Tasks: T-0216
