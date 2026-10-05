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
→ `#VALUE!` (MIN of a text value; see coercion.md). IFERROR of a blank cell is an empty value, not 0:
in `&` it is "" (`=((IFERROR(Data!D$5,Data!$A3)&" x ")&"1")` with D5 blank → " x 1";
`…&IFERROR(Data!$B8,0.2)` with B8 blank appends nothing). How an aggregate counts it is not yet shown.
Outside aggregate arguments the reference-ness is invisible: `VLOOKUP(IF(TRUE,Data!A9),…)` and
`x+IF(TRUE,Data!D11)` use the cell's value (gen-m2/207a5753da48c7e0 A270, b4392a6514a36ade A115 pass
today and regress if IF returns a reference everywhere).
Evidence: gen-m2/96a0952412cefa6c A88, 942bdade381b7537 A212, dd99be1f2454159f A264,
04cfd82f54dbc2b2 A182, dd99be1f2454159f A110, d0c29dd5c530e28c A112 (Calc sheet). Oracle: excel-graph, 2026-10-04 (gen-m2 sidecars, T-0203).
HLOOKUP's result is a reference too: `COUNT(…,HLOOKUP(…))` skips a FALSE it picks (gen-m2/60a543744fc4a8f7 A200).
Status: VERIFIED
ISREF sees the same references: `ISREF(INDEX(A1:A2,2))`, `ISREF(IF(TRUE,A1))` and `ISREF(CHOOSE(1,A1))` are TRUE,
while `ISREF(INDEX(A1:A2,3))` (an error), `ISREF(IF(TRUE,1))` and `ISREF(CHOOSE(1,5))` are FALSE (probes/isref-function-results,
excel-graph 2026-10-05; ironcalc ISREF_ISFORMULA!F9 `ISREF(INDEX(A1,1,2,2))` = FALSE). IF/CHOOSE in ISREF: T-0235.
Implemented in: `evaluate_aggregate_arg` in `crates/rustsheet-engine/src/functions/util.rs` (IF, INDEX; used by SUM,
COUNT, MIN, MAX and every `for_each_value` caller: AVERAGE, MEDIAN, HARMEAN, GEOMEAN, SKEW, SKEW.P, KURT);
ISREF of INDEX in `fn_isref` (functions/information/mod.rs)
Tasks: T-0216, T-0235

## Intersection operator (space)
Rule: `A B` (a space between two references) gives the reference to the cells both contain. In a
value context a single-cell result gives that cell's value, its error (`#DIV/0!`, `#N/A`) or, for an
empty cell, 0 (`""` under `&`). It works with three operands, whole rows and columns, `$` corners,
reversed corners (`Data!C8:B3`) and range-of-ranges (`Data!A7:A7:Data!C11`).
`Data!A1:C7 Data!C7:E12` = 0.5 (Data!C7); `Data!C1:D7 Data!C1:C12 Data!A6:E6` = TRUE (Data!C6);
`Data!Z:Z Data!290:290` = 0; `(Data!D:D Data!2:2)&""` = ""; `Data!C8:B3 Data!B8` = "a";
`(Data!C2:C11 Data!C6:E6)+0.1` = 1.1; `Data!D2:E8 Data!E8:E12` = `#DIV/0!`.
Evidence: gen-m1/4dc8ed1de05a61be Calc!A5, A2, A3, A16; gen-m1/7cb99acdd82402f1 Calc!A25, A4;
gen-m1/1ce5b2467b3b19fd Calc!A9, A25 (oracle: excel-graph, 2026-10-05; 632 cells across the three cases, T-0109).
Status: VERIFIED
Implemented in: expressions/parser/reference_operators.rs (`Parser::parse_intersection`); reference_operators.rs (`Model::get_intersection`)
Tasks: T-0105

## Intersection operator: implementation choices (UNVERIFIED)
Rule: RustSheet's choices where gen-m1 (single U+0020 spaces between references) has no evidence:
- Any Unicode whitespace between two reference operands is the operator: LF, CR, TAB, NBSP and
  U+3000 (`char::is_whitespace`, `Lexer::whitespace_follows`).
- A literal operand on the left (`=1 A1`, `=(1) A1`) or TRUE/FALSE, `@x` or an error other than
  `#REF!` on the right is a parse error; a parenthesized non-reference on the right (`=A1 (1)`)
  is `#VALUE!`; `#N/A` on the left and `#REF!` on the right are accepted and propagate.
- An operand's error wins over `#VALUE!`/`#NULL!` (`=(1/0) A1` = `#DIV/0!`).
- The cross-sheet check runs before the disjoint check (`=Data!A1 Other!Z99` = `#VALUE!`).
Evidence: none from Excel; the engine's results come from a scratch-suite run in the T-0105 review (2026-10-05).
Status: UNVERIFIED
Tasks: T-0138

## Disjoint intersection is #NULL!
Rule: an intersection of references that share no cell is `#NULL!` (all 51 `intersection/disjoint` cells):
`Data!A3:D4 Data!Z99` → `#NULL!`, `Data!B4:E11 Data!A1:A3` → `#NULL!`.
Evidence: gen-m1/4dc8ed1de05a61be Calc!A6, A10 (oracle: excel-graph, 2026-10-05, T-0109).
Status: VERIFIED
Implemented in: reference_operators.rs (`Model::get_intersection`)
Tasks: T-0105

## Cross-sheet intersection is #VALUE!
Rule: an intersection of references on two different sheets is `#VALUE!`, not `#NULL!`, even where
the coordinates overlap (all 43 `intersection/cross-sheet` cells): `Data!C5:C10 'My Data'!C9:D9` → `#VALUE!`.
Evidence: gen-m1/4dc8ed1de05a61be Calc!A4, A17, A29 (oracle: excel-graph, 2026-10-05, T-0109).
Status: VERIFIED
Implemented in: reference_operators.rs (`Model::get_intersection`)
Tasks: T-0105

## Union (comma) reference in a value context is #VALUE!
Rule: a parenthesized union `(ref1,ref2)` is `#VALUE!` as a cell result, as an operand of
arithmetic or `&`, and nested. This holds even when both areas are the same cell. An intersection that cuts
the union down to one cell gives that cell's value. `(Data!E4,Data!E4)` → `#VALUE!`;
`(Data!E12,Data!A9)/1` → `#VALUE!`; `(Data!C12,Data!E9)&""` → `#VALUE!`;
`((Data!A2,Data!E8),Data!B6)` → `#VALUE!`; `(Data!D1:D12,Data!A1:A12) Data!D11` = "a" (Data!D11).
Evidence: gen-m1/c940d3cfb0d3b083 Calc!A1, A2, A14, A25, A12; all 218 value-context cells are
`#VALUE!` and all 32 `union/cut-by-cell` cells give the cell (oracle: excel-graph, 2026-10-05, T-0109).
Status: VERIFIED
Tasks: T-0134

## Range corners may both carry the sheet prefix
Rule: `Data!B8:Data!E12` is the range `Data!B8:E12`: `Data!B8:Data!E12 Data!B12` = 1E-15
(Data!B12) and `Data!C2:Data!E4 Data!C4` = 100. A single-cell range can also start a range-of-ranges:
`Data!A7:A7:Data!C11 Data!A11` = 1.
Evidence: gen-m1/1ce5b2467b3b19fd Calc!A10, A16, A25 (41 `range/ref-colon-ref` and 53
`range/range-of-ranges` cells; oracle: excel-graph, 2026-10-05, T-0109).
Status: VERIFIED
Tasks: T-0130, T-0105
