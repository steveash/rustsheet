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

## External reference with no cached data is `#BLOCKED!`
Rule: `SUM('[link-external-workbook-a.xlsx]Sheet0'!A1:B1)` whose externalBook part holds no cached
data is `#BLOCKED!`, not `#REF!`. Excel for the web writes it as a vm-coded rich-value error,
errorType 10 / subType 2, cached as `#VALUE!`; errorType 10 is `#BLOCKED!` (ERROR.TYPE 11,
docs/excel-semantics/arrays.md table, probes/rich-errors-image). Inferred from that mapping: no
ERROR.TYPE or VALUETOTEXT was taken next to the external reference itself (subType 2 there, 12 for
IMAGE).
Evidence: poi/link-external-workbook-b.xlsx Sheet0!A1 (excel-graph, refilled 2026-10-05, T-0030).
The workbook was written by Apache POI, so there is no desktop evidence (decision 0030: web-only
values are goldens).
Status: VERIFIED (Excel for the web; error name inferred from errorType 10)
Tasks: T-0124

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

## Intersection operator: separators
Rule: one or more spaces (U+0020) and line feeds (LF), in any mix, separate the two operands of an
intersection: `=A1:A3  A2:B2`, `=A1:A3<LF>A2:B2`, `=A1:A3 <LF> A2:B2` and `=A1:A3<LF><LF>A2:B2` all give 2.
Excel for the web refuses a workbook holding CR, CR+LF, TAB or NBSP (U+00A0) there
(`FileCorruptTryRepair`), even when the CR or TAB is written as a character entity. With U+3000 it loads, and
`=A1:A3<U+3000>A2:B2` is `#NAME?`.
Evidence: probes/intersection-whitespace H1:H8 (oracle: excel-graph, 2026-10-05, T-0138). The refused
workbooks come from scratch runs of `rustsheet-oracle recalc` on one formula each (T-0138); a refused workbook
has no values to commit.
Status: VERIFIED (load-time refusal observed in Excel for the web only)
Tasks: T-0138, T-0148

## Intersection operands must be references
Rule: Excel decides when it parses the formula whether each operand of an intersection is a
reference expression. A reference, a parenthesized reference, `#REF!`, `@ref`, a union and a call to a
reference-returning function (INDEX, OFFSET, IF, ANCHORARRAY) are accepted. Number, string, boolean and
error literals other than `#REF!`, a parenthesized value (`(1)`, `(1/0)`) and value functions (SUM, NA) make
Excel for the web refuse the workbook: `=1 A1`, `=(1) A1`, `=A1 (1)`, `=A1 TRUE`, `=A1 "a"`,
`=#N/A A1`, `=A1 #N/A`, `=A1 #DIV/0!`, `=(1/0) A1`, `=A1 (1/0)`, `=(1/0) (NA())`, `=(1/0) Z99`,
`=#REF! #N/A`, `=A1 SUM(A1:A2)`, `=A1 NA()`, `=SUM(A1:A2) A1`. So "which error wins" never arises
between two error operands. Accepted forms:
- `=A1 #REF!` and `=#REF! A1` are `#REF!`; `=C1:C2 C2` with C2 = `#N/A` is `#N/A` (the cell's value);
  `=C1 C2` is `#NULL!` even when both cells hold errors.
- `=A1 @A1` = 1. `=@A1:A3 A2:B2` = 2 in row 11, where `=@A1:A3` alone is `#VALUE!` (I11): the space
  binds tighter than `@`.
- `=A1:A3 INDEX(A1:B3,2,0)`, `OFFSET(A2,0,0,1,2)` and `IF(TRUE,A2:B2)` give 2, on either side.
- A spill reference intersects: `=J1# J2:K2` = 2 with `J1 = SEQUENCE(3)`, stored as
  `_xlfn.ANCHORARRAY(J1)` (a literal `J1#` in the file is refused).
- `=A1:A3 (A2:B2,A2)` is `#VALUE!` (a two-area union is left over).
- `=(A1)(A2)`, with no space, is `#REF!`, not a syntax error. The reason is not known (syntax.md, trailing input).
- `=Data!A1 Other!Z99` (different sheets, no shared coordinates) is `#VALUE!`, like overlapping cross-sheet
  operands: the sheet check comes before the disjoint check.
Evidence: probes/intersection-operands H1:H15, I11 and probes/intersection-function-operands H1:H5 (oracle:
excel-graph, 2026-10-05, T-0138). The refused workbooks come from scratch runs of one formula each (T-0138).
Status: VERIFIED (load-time refusal observed in Excel for the web only)
Tasks: T-0138, T-0148, T-0149, T-0042

## A defined name may be an intersection
Rule: a defined name whose formula is an intersection is a reference. With `isect` =
`Sheet1!$A$1:$A$3 Sheet1!$A$2:$B$2` and `wide` = `Sheet1!$A$1:$B$3 Sheet1!$A$2:$B$3`: `=isect` = 2,
`=ISREF(isect)` = TRUE, `=isect A1:B3` = 2, `=ROWS(wide)` = 2, `=COLUMNS(wide)` = 2, `=SUM(wide)` = 55,
`=INDEX(wide,2,2)` = 30. A disjoint one, `disj` = `Sheet1!$A$1 Sheet1!$B$2`, is `#NULL!` and `ISREF(disj)` = FALSE.
Evidence: scratch xlsx built with xlsxwriter: Sheet1 A1:A3 = 1, 2, 3, B1:B3 = 10, 20, 30, the three
names above via `define_name`, each formula in column H. Recalculated by excel-graph (2026-10-05, T-0138);
the case format has no defined names yet (T-0043).
Status: UNVERIFIED (scratch evidence from Excel for the web only; committed case pending T-0043)
Tasks: T-0138, T-0043

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
Implemented in: `sheet_prefix_after_colon` (crates/rustsheet-engine/src/expressions/lexer/ranges.rs)
Tasks: T-0130, T-0105

## Sheet-qualified range corners: order and unprobed forms
Rule: corners are ordered whichever way
they are written (`Data!B2:Data!A1` is `Data!A1:B2`; implemented in `get_range`); the formula text
round-trips as written; `Data!A1:Other!B2` (corners on different sheets) gives `#VALUE!` in the engine;
the forms with column or row corners are not ranges in Excel (T-0139, see the next entry).
Status: UNVERIFIED (reversed corners, round-tripping and the cross-sheet `#VALUE!` are engine behavior without
a probe; T-0139 verified only the second-prefix column/row forms)
Implemented in: `get_range` (crates/rustsheet-engine/src/reference_operators.rs)

## Sheet-qualified column or row corners on both sides are not a range
Rule: only two cell corners may both carry the sheet prefix (`Data!A1:Data!B2` = `Data!A1:B2`,
previous entry). With column or row corners the second prefix is not accepted: `Data!A:Data!B`,
`Data!A1:Data!B`, `Data!A:Data!B1`, `Data!A:Data!A`, `Data!B:Data!A` and `Data!A1:B` are `#NAME?`
(interpretation, not probed: the bare `A`/`B` after the colon is read as a name), while `Data!A:B` (219 on the probe data) and
`Data!1:2` are ordinary ranges. `Data!1:Data!2`, `Data!A1:Data!2` and `Data!$A:Data!$B` cannot be
stored: the workbook fails to load in Excel for the web (createSession 501 FileCorruptTryRepair),
so there is no value to match. Each was isolated as a one-formula case on the Data/Calc layout of
the probe case (`Calc!A1` = the formula) and failed on its own; re-run `rustsheet-oracle fill -b excel-graph`
on such a case to reproduce (2026-10-05). The engine gives `#ERROR!` (parse failure) where Excel gives `#NAME?`.
Evidence: probes/sheet-qualified-both-corners Calc!A1, A5, A6, A7, A11, A12 (oracle: excel-graph,
2026-10-05, T-0139); A2 and A4 are the single-prefix forms.
Status: VERIFIED (web)
Tasks: T-0139

## OFFSET: an empty height or width argument means the reference's size
Rule: in `OFFSET(reference, rows, cols, [height], [width])` a present-but-empty height or width
is the reference's height or width, as if the argument were left out; it is not 0.
`SUM(OFFSET(K7:L8,0,0,1,))` = K7+L7, `SUM(OFFSET(K7:L8,0,0,,1))` = K7+K8,
`SUM(OFFSET(K7:L8,0,0,,))` and `SUM(OFFSET(K7:L8,0,0,))` = the whole K7:L8.
Evidence: poi/FormulaEvalTestData_Copy.xlsx (Excel-cached, Excel 15) EverythingTests!I1008 (2.001),
J1008 (3), M1008 and N1008 (7.0009); K7:L8 = {1, 1.001; 2, 2.9999}.
Status: VERIFIED (Excel-cached)
Tasks: T-0946
