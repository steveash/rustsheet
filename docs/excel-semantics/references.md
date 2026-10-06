# References and names

## 3-D sheet-range references in aggregates
Rule: SUM, AVERAGE, MIN, MAX, COUNT and COUNTA accept `First:Last!ref`, and the sheet span may
include the formula's own sheet. The span is the sheets from First to Last in tab order, whichever
way it is written (`Data3:Data1!A1` = `Data1:Data3!A1`); text, booleans and blanks in it are skipped
like in any range, and the first error cell wins. A name that needs quotes quotes the whole span
(`'Data2:Q 4'!A1`). Also accepted (same probes): AVERAGEA, MAXA, MINA, PRODUCT, STDEV, VAR, MEDIAN,
LARGE, RANK, SUMSQ and CONCAT (`=CONCAT(Data1:Data3!A1)` → "110100").
Evidence: poi/FormulaSheetRange.xlsx test!D11 = 10, D12 = 60 and poi/48703.xlsx Sheet1!B2 = 20 (oracle:
excel-graph, 2026-10-04); poi/55906-MultiSheetRefs.xlsx A3 = 66, I6 = 4 (Excel-cached);
probes/3d-ref-aggregates (excel-graph, 2026-10-06).
Status: VERIFIED
Implemented in: `get_sheet_span_range` (crates/rustsheet-engine/src/reference_operators.rs) and
`for_each_cell_or_error_in_range` (crates/rustsheet-engine/src/functions/util.rs); AVERAGEA, MAXA,
MINA, PRODUCT, CONCAT, LARGE and RANK take their arguments through `evaluate_aggregate_arg` too (T-0196).
CONCAT no longer rejects a cross-sheet range with #VALUE!. LARGE/RANK read 3-D spans through
`values_from_range` (functions/statistical/chisq.rs), which stays unclipped and positional on a single sheet
because two-input functions pair their lists by position.
Tasks: T-0122, T-0196

## 3-D references outside the aggregates
Rule: where a value is needed a 3-D reference is `#REF!`: the bare formula `=Data1:Data3!A1`, an
operator (`+1`), and value functions such as ABS; so ERROR.TYPE gives 4, ISERROR TRUE, ISREF and
ISBLANK FALSE and IFERROR its fallback. Where one sheet's reference is needed it is `#VALUE!`: INDEX,
ROWS, COLUMNS, ROW, AREAS, VLOOKUP, MATCH, SUMIF, COUNTIF and COUNTBLANK. SUMPRODUCT gives `#REF!`.
A span of one sheet (`=Data1:Data1!A1`) is that sheet's reference: 1.
Evidence: probes/3d-ref-values, probes/3d-ref-functions (excel-graph, 2026-10-06).
Status: VERIFIED, except two engine choices without a probe (UNVERIFIED): a span whose end is not a
sheet (`=SUM(Data1:Nope!A1)`) is `#REF!` like any reference to a missing sheet, and the explicit
implicit intersection `=@Data1:Data3!A1` is `#VALUE!`.
Implemented in: `get_sheet_span_value` (`#REF!`) and `get_sheet_span_reference` (`#VALUE!`) in
crates/rustsheet-engine/src/reference_operators.rs. INDEX, AREAS, VLOOKUP, MATCH and COUNTBLANK do
not give `#VALUE!` yet (T-0197).
Tasks: T-0122, T-0197

## 3-D references when sheets or cells change (unprobed)
Rule (engine, not Excel evidence): renaming an end sheet renames it in the formula, and moving a
sheet re-resolves the span by name. Deleting an end sheet makes the reference `#REF!`, and inserting
or deleting rows or columns does not shift the span. Excel's own rules for these edits are not probed.
Status: UNVERIFIED
Tasks: T-0198

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
A defined name that stands for a cell or range is a reference to ISREF, a single cell included
(`ISREF(TextCell)` and `ISREF(Cells)` TRUE; a LAMBDA name, a constant name `=7` and an undefined name are FALSE), and FILTER's `if_empty` range is its
values, never a reference (`FILTER(A1:A1,B1:B1,D1:D2)` spills D1:D2, `ISREF(FILTER(..))` FALSE, `SUM(..)` = 30).
Evidence: probe isref-names-filter-if-empty (oracle: excel-graph, 2026-10-06, T-0940), matching upstream IronCalc ISREF.xlsx (ironcalc 9add6b67; Excel 16.03 cached values; to be added to calc_tests by T-0965) Sheet1!A14-A18, D11, D12, D14, D17, D20. ISREF of a LET/LAMBDA variable
bound to a reference is TRUE (A19-A26, not implemented: T-0963).
Implemented in: `evaluate_aggregate_arg` in `crates/rustsheet-engine/src/functions/util.rs` (IF, INDEX; used by SUM,
COUNT, MIN, MAX and every `for_each_value` caller: AVERAGE, MEDIAN, HARMEAN, GEOMEAN, SKEW, SKEW.P, KURT;
also AVERAGEA, MAXA, MINA, PRODUCT, CONCAT, LARGE and RANK since T-0196, UNVERIFIED for those: probe in T-1003);
ISREF of INDEX and of defined names in `fn_isref` (functions/information/mod.rs),
FILTER if_empty range in `fn_filter` (functions/spill_functions.rs)
Tasks: T-0216, T-0235, T-0940, T-1003

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
Implemented in: expressions/parser/reference_operators.rs (`Parser::parse_intersection`); expressions/lexer/mod.rs (`Lexer::separator`)
The engine turns a U+3000 intersection into one undefined name holding its text; where Excel's name
begins and ends, and so how references inside it update on row/column inserts, is UNVERIFIED.
Tasks: T-0138, T-0148

## Intersection operands must be references
Rule: Excel decides when it parses the formula whether each operand of an intersection is a reference
expression. A reference, a parenthesized reference, `#REF!`, `@ref`, a union and a call to a
reference-returning function (INDEX, OFFSET, IF, ANCHORARRAY, INDIRECT, CHOOSE, XLOOKUP, LET, SWITCH,
IFS, TAKE, DROP, TRIMRANGE) are accepted. Number, string, boolean and error literals other than `#REF!`,
a parenthesized value (`(1)`, `(1/0)`), a parenthesized value operator and value functions make Excel
for the web refuse the workbook: `=1 A1`, `=(1) A1`, `=A1 (1)`, `=A1 TRUE`, `=A1 "a"`, `=#N/A A1`, `=A1
#N/A`, `=A1 #DIV/0!`, `=(1/0) A1`, `=A1 (1/0)`, `=(1/0) (NA())`, `=(1/0) Z99`, `=#REF! #N/A`, `=A1
SUM(A1:A2)`, `=A1 NA()`, `=SUM(A1:A2) A1`, `=A1 (B1+C1)`, `=A1 (-B1)`, `=A1 (B1&"")`, `=A1 (B1=1)`, `=A1
(B1%)`, `=A1 (B1*C1)`, `=A1 (B1/C1)`, `=A1 (B1^2)`, and with A1:B3 = 1, 2, 3 / 10, 20, 30: `=A1:A3
ABS(A2)`, `ROW(A2)`, `IFERROR(A2:B2,0)`, `IFNA(A2:B2,0)`, `CHOOSEROWS(A1:B3,2)`,
`FILTER(A1:B3,A1:A3=2)`, `VSTACK(A2:B2)`, `SORT(A2:B2)`, `UNIQUE(A2:B2)`, `VLOOKUP(2,A1:B3,1)`,
`HLOOKUP(2,A2:B3,1)`, `LOOKUP(2,A1:A3,A1:A3)`, `MATCH(2,A1:A3,0)`, `ROWS(A2:B2)` and `=A2:B2
CHOOSECOLS(A1:B3,1)`. IFERROR and IFNA pass a range through, and CHOOSEROWS/CHOOSECOLS select rows and
columns of one, like TAKE and DROP, yet are refused: what counts is Excel's fixed list of
reference-returning functions, not the argument. So "which error wins" never arises between two error
operands. Accepted forms:
- `=A1 #REF!` and `=#REF! A1` are `#REF!`; `=C1:C2 C2` with C2 = `#N/A` is `#N/A` (the cell's value);
  `=C1 C2` is `#NULL!` even when both cells hold errors.
- `=A1 @A1` = 1. `=@A1:A3 A2:B2` = 2 in row 11, where `=@A1:A3` alone is `#VALUE!` (I11): the space
  binds tighter than `@`.
- `=A1:A3 INDEX(A1:B3,2,0)`, `OFFSET(A2,0,0,1,2)` and `IF(TRUE,A2:B2)` give 2, on either side;
  `INDIRECT("A2:B2")`, `CHOOSE(1,A2:B2)`, `XLOOKUP(2,A1:A3,A1:B3)`, `SWITCH(1,1,A2:B2)`,
  `IFS(TRUE,A2:B2)`, `TAKE(A2:B3,1)`, `DROP(A1:B2,1)`, `TRIMRANGE(A2:B2)` and `LET(x,A2:B2,x)` give 2 as right operands.
- A spill reference intersects: `=J1# J2:K2` = 2 with `J1 = SEQUENCE(3)`, stored as
  `_xlfn.ANCHORARRAY(J1)` (a literal `J1#` in the file is refused).
- `=A1:A3 (A2:B2,A2)` is `#VALUE!` (a two-area union is left over).
- `=(A1)(A2)`, with no space, is `#REF!`, not a syntax error. The reason is not known (syntax.md, trailing input).
- `=Data!A1 Other!Z99` (different sheets, no shared coordinates) is `#VALUE!`, like overlapping cross-sheet
  operands: the sheet check comes before the disjoint check.
Evidence: probes/intersection-operands H1:H15, I11 and probes/intersection-function-operands H1:H5
(oracle: excel-graph, 2026-10-05, T-0138), H6:H12 (2026-10-06, T-0161). H13 `=A1:A3 LET(x,A2:B2,x)` and
H14 `=A1:A3 TRIMRANGE(A2:B2)` are both 2 (2026-10-06, T-0166; the case builder now writes `_xlpm.` bound
names and `_xlfn.` prefixes; without them Excel refuses the file or gives `#NAME?`). The refused
workbooks come from scratch runs of one formula each (T-0138, T-0161); each refusal was
`FileCorruptTryRepair`, and the same CHOOSEROWS and IFERROR calls load on their own (=2).
UNVERIFIED builder edge: a LET name that equals a function name or is used before its declaration is
still written with `_xlpm.`; how Excel stores that is not probed.
Status: VERIFIED (load-time refusal observed in Excel for the web only)
Implemented in: expressions/parser/reference_operators.rs (`is_reference_operand`: only the accepted
functions above. UNVERIFIED by inference: built-in functions not probed are refused, while unknown or
future function names, LAMBDA definitions and calls, and `@` operands are accepted; `@` and union
operands are T-0149). The engine still evaluates XLOOKUP, TAKE, DROP and TRIMRANGE to arrays, so those four
intersections are `#VALUE!` (T-0165).
Tasks: T-0138, T-0148, T-0149, T-0042, T-0161, T-0165

## A defined name may be an intersection
Rule: a defined name whose formula is an intersection is a reference. With `isect` =
`Sheet1!$A$1:$A$3 Sheet1!$A$2:$B$2` and `wide` = `Sheet1!$A$1:$B$3 Sheet1!$A$2:$B$3`: `=isect` = 2,
`=ISREF(isect)` = TRUE, `=isect A1:B3` = 2, `=ROWS(wide)` = 2, `=COLUMNS(wide)` = 2, `=SUM(wide)` = 55,
`=INDEX(wide,2,2)` = 30. A disjoint one, `disj` = `Sheet1!$A$1 Sheet1!$B$2`, is `#NULL!` and `ISREF(disj)` = FALSE.
Evidence: probes/defined-name-intersection Sheet1!H1:H9 (Sheet1 A1:A3 = 1, 2, 3, B1:B3 = 10, 20,
30; oracle: excel-graph, 2026-10-05, T-0043), first seen with a scratch xlsx in T-0138.
Status: VERIFIED (Excel for the web; the engine answers `#NAME?` for these bodies, T-0158)
Tasks: T-0138, T-0043, T-0158

## A defined name may be a constant
Rule: a defined name whose body is a constant evaluates to that value: `rate` = `0.5`, `=rate*4` = 2.
Evidence: probes/defined-name-intersection Sheet1!H10 (oracle: excel-graph, 2026-10-05, T-0043).
Status: VERIFIED (Excel for the web; the engine answers `#NAME?`, T-0158)
Tasks: T-0043, T-0158

## A sheet-qualified #REF! is #REF!
Rule: a reference that Excel has already replaced by `#REF!` but that keeps its sheet prefix
evaluates to `#REF!`. That holds as a cell formula (`='1'!#REF!`) and as the body of a defined name:
with `sale_1` and `sale_2` both defined as `Sheet1!#REF!`, `=sale_1*sale_2` is `#REF!`. With an
undefined name on the left, `=aaa*sale_1` is `#NAME?`, so the left operand's error wins.
Evidence: poi/customIndexedColors.xlsx `_TGK_HIDDEN!B1` (Excel 16.0) and poi/49783.xlsx Sheet1!A2, B2
(Excel 14.0); desktop Excel cached values. Probe sheet-qualified-ref-error-forms (oracle: excel-graph, 2026-10-06, T-0169)
confirms: `=Sheet1!#REF!`, `=SUM(Sheet1!#REF!)` and `=NoSuchSheet!#REF!` are all `#REF!`; names with a bare error-literal
body (`=#N/A`, `=#DIV/0!`, `=#REF!`) evaluate to that error; and `SHEET(name)` of an error-literal name is that same error
(`#REF!` for `Sheet1!#REF!`, `#N/A` for `=#N/A`), not a distinct error.
Status: VERIFIED
Implemented in: `consume_range` in crates/rustsheet-engine/src/expressions/lexer/mod.rs (formula), `ParsedDefinedName::ErrorLiteral` in model.rs / new_empty.rs / defined_names.rs (name body), `fn_sheet` (SHEET of such a name).
Tasks: T-0152, T-0169

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
An intersection that leaves two areas is `#VALUE!` even when both are the same cell:
`=A1:A3 (A2:B2,A2)` (probes/intersection-function-operands H5, see "Intersection operands must be references").
Status: VERIFIED
Implemented in: expressions/parser/reference_operators.rs (`Parser::parse_parenthesized`, `union_text`);
reference_operators.rs (`Model::get_union`, `Model::get_intersection` over the areas of a union)
Tasks: T-0134

## Union: parsing and evaluation
Rule: how Excel parses and evaluates unions beyond the forms gen-m1 measured.
- The comma is the union operator only directly inside parentheses: `(A1,B2)` is one union,
  `SUM(A1,B2)` is two arguments, `SUM((A1,B2))` is one union argument, and `=A1,B2` is a syntax
  error. (Engine choice, not probed.)
- Operands must be references. Excel for the web refuses a workbook (`FileCorruptTryRepair`) that
  holds `=(A1+B1,C1)`, `=(1,A1)`, `=(A1,"a")`, `=(A1,#N/A)`, `=(A1,SUM(B1))` or `=(A1,(B1+1))`
  (one scratch workbook per formula, T-0187). The engine reads the five non-reference operands as
  syntax errors, and so is `(A1+B1,C1)` since T-0193 (it used to read as `A1+(B1,C1)`).
- A negated reference is accepted: `=(-A1,B1)` loads, gives `#VALUE!`, and FORMULATEXT prints it as
  `=(-A1,B1)`. The engine reads it as a union of `-A1` and `B1` and prints that (T-0193).
- An error operand's own error comes before the `#VALUE!`: `=(A1,#REF!)` and `=(#REF!,A1) A1` are
  `#REF!`, and `ISREF((A1,#REF!))` is FALSE (the engine gives TRUE, T-0194). `ISREF((A1,B2))` is
  TRUE, and `=(A1,B2) A1` is 1.
- Cross-sheet: `=(Data!A1,Other!A1)` and `=(Data!A1:A3,Other!A1:A3) Data!A2` are `#VALUE!`.
- Printing: Excel keeps the user's parentheses, so FORMULATEXT of `=((Data!A2,Data!E8),Data!B6)` is
  exactly that. The engine keeps a union operand's own parentheses (`grouped` on `OpUnionKind`), while a
  plain chain `(A1,B2,C3)` still prints as written (T-0193).
- Multi-area defined names: with `mu` = `Sheet1!$A$1,Sheet1!$B$2` (A1 = 1, B2 = 5), `=SUM(mu)` is 6,
  `=COUNT(mu)` is 2, `=ISREF(mu)` is TRUE, `=mu A1` is 1 and `=mu` is `#VALUE!`. The engine refuses
  the name body ("invalid defined name"), so the whole case is an engine error. Multi-area values are
  spec T-0133.
- Comma-decimal locales: xlsx stores formulas in en-US syntax with `,`, so the `;` spelling of the
  union operator exists only in the UI of such locales and cannot be probed through a file. The
  engine prints the argument separator `;` there (engine choice).
- Engine choices still unprobed: only a single unary minus is accepted on an operand (`(--A1,B1)`
  and `(A1%,B1)` are refused, `(-(A1,B2),C1)` is accepted); only one level of written parentheses
  is kept in printing (`((A1,B2))` prints as `(A1,B2)`); a negated operand cut by an intersection is
  unknown. The union is the loosest operator inside its parenthesis and the
  intersection binds tighter, so `(A1,B1 C1)` is `(A1,(B1 C1))`. A union cut by an intersection runs the cut on each pair of areas,
  drops pairs with no shared cell, and gives `#NULL!` when nothing is left. Further cuts can bring
  several areas down to one (`(A1:A12,D1:D12) A4:D4 D4` is D4). The known gaps: a union reaching a
  function gets `#VALUE!` (`SUM((A1,B2))`, `AREAS`, `INDEX(...,area_num)`), so IFERROR silently
  takes its fallback for one, and so does a union returned by IF, CHOOSE or LET (T-0133). `add_implicit_intersection` (static_analysis.rs) leaves
  `OpUnionKind` unwrapped; T-0133 should revisit it.
Evidence: probes/union-accepted H1:H9, probes/union-neg-operand H1:H2, probes/union-defined-name
H1:H5 (oracle: excel-graph, 2026-10-06, T-0187). The refusals come from scratch workbooks, one per
formula, which were not kept as cases.
Status: VERIFIED (except the parts marked as engine choices)
Implemented in: expressions/parser/reference_operators.rs (`Parser::parse_parenthesized`); reference_operators.rs
Tasks: T-0134, T-0187, T-0193, T-0194, T-0133

## Union area cap
Rule: intersecting unions multiplies their areas (`(A1,A2,A3) (A1:A3,B1:B3)` has six pairs), and
balanced nesting (`((A1,A2),(A3,A4))`) keeps a union's depth logarithmic, so the parser's depth
cap does not bound its leaves. The engine bounds every area list it builds, not just products, at
65,536 areas (`MAX_AREAS`), with `#VALUE!` past it: an intersection of more than `MAX_AREAS` pairs,
and a union whose accumulated areas would exceed `MAX_AREAS`, checked after each operand before
the next is evaluated (so `(I,A1,#REF!)`, with `I` an intersection of 65,536 areas, is `#VALUE!`,
not `#REF!`). No operator ever returns more than `MAX_AREAS` areas, so the pairwise cost of one
intersection is at most `MAX_AREAS` comparisons. A chain of cuts (`(U1 U2) A1 A1 ...`) would
repeat that per three characters of formula, so one top-level reference expression may compare at most
1,048,576 pairs in all (`MAX_AREA_PAIRS`), with `#VALUE!` past it; a reference-returning function
used as an operand (`INDEX`, `OFFSET`, `IF`) starts its own budget, so total work stays linear in
formula length. In a value context a union
keeps only its area count, not the areas. Excel's limits, if any, are not known, and neither is
its result past them; it would differ where later cuts bring the areas back down to one, or where
a later operand's error would come first.
Status: DEVIATION (decision 0014)
Implemented in: reference_operators.rs (`MAX_AREAS`, `MAX_AREA_PAIRS`, `Model::union_areas`,
`Model::count_union_areas`, `Model::intersect_areas`)
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
on such a case to reproduce (2026-10-05).
Implemented: only these probed shapes lex as an identifier (sheet prefix kept, so the formula text round-trips) that evaluates to `#NAME?`; other parse failures keep `#ERROR!`.
Evidence: probes/sheet-qualified-both-corners Calc!A1, A5, A6, A7, A11, A12 (oracle: excel-graph,
2026-10-05, T-0139); A2 and A4 are the single-prefix forms.
Status: VERIFIED (web)
Tasks: T-0139, T-0145

## OFFSET: an empty height or width argument means the reference's size
Rule: in `OFFSET(reference, rows, cols, [height], [width])` a present-but-empty height or width
is the reference's height or width, as if the argument were left out; it is not 0.
`SUM(OFFSET(K7:L8,0,0,1,))` = K7+L7, `SUM(OFFSET(K7:L8,0,0,,1))` = K7+K8,
`SUM(OFFSET(K7:L8,0,0,,))` and `SUM(OFFSET(K7:L8,0,0,))` = the whole K7:L8.
Evidence: poi/FormulaEvalTestData_Copy.xlsx (Excel-cached, Excel 15) EverythingTests!I1008 (2.001),
J1008 (3), M1008 and N1008 (7.0009); K7:L8 = {1, 1.001; 2, 2.9999}.
Status: VERIFIED (Excel-cached)
Implemented in: `offset_size` / `fn_offset` in crates/rustsheet-engine/src/functions/lookup_and_reference/mod.rs
Tasks: T-0946

## OFFSET negative and fractional height/width
Rule: the height/width `h` (after the rows/cols offset has been applied, the "anchor" is the reference's
top-left cell) names an area from the anchor to `floor(h - 1)` cells on when `h > 0`, or `ceil(h + 1)`
cells on when `h < 0`; the area is the span between the two, so a negative size extends up/left.
`OFFSET(A10,0,0,-3,1)` is A8:A10 (and width `-3` on C10 is A10:C10). The sizes in (0, 1) reach one cell
back (`OFFSET(A10,0,0,0.5,1)` is A9:A10), those in (-1, 0) one cell on (`-0.5` is A10:A11), and
fractions otherwise truncate toward zero (1.5 and -1.5 are one cell, -2.9 is two cells up, 2.5 two down).
Only exactly 0 is `#REF!` (as is an area reaching past row 1/column A or the sheet's end: `OFFSET(A5:B6,0,0,,-2)` is `#REF!`). A multi-cell reference
anchors at its top-left: `OFFSET(A5:B6,0,0,-2)` is A4:B5.
Evidence: probe offset-negative-height-width (oracle: excel-graph, 2026-10-06, T-0956; 80 cells).
Status: VERIFIED
Implemented in: `offset_size` / `fn_offset` in crates/rustsheet-engine/src/functions/lookup_and_reference/mod.rs
Tasks: T-0956

## ADDRESS leaves a [Book]Sheet name unquoted
Rule: ADDRESS does not quote a `sheet_text` made of a `[workbook]` part and a plain sheet name:
`ADDRESS(2,3,3,TRUE,"[Book1]Sheet1")` is `[Book1]Sheet1!$C2`, while names with spaces or `$` are
quoted (`'Me lo invente'!$C$2`, `'A Wird One$'!$A$1`) and `"Sheet2"` is not.
Evidence: poi/FormulaEvalTestData_Copy.xlsx (Excel-cached, Excel 15) EverythingTests!F128;
ironcalc-calc LOOKUP_AND_REFERENCE/ADDRESS.xlsx (Excel-cached) Sheet1!E2, B33, B38.
Status: VERIFIED (Excel-cached). Open (unprobed): `[Book1]My Sheet` and brackets elsewhere in the name.
Tasks: T-0984

## T of a multi-cell range is not intersected
Rule: in a legacy formula, `T(range)` over several cells is not the implicitly intersected cell:
`T(L7:N7)` in N1352 is "" although N7 is text, `T(J1351:J1353)` in O1352 is "" although
J1352 is text, and `T(H12:J12)` from column M (no intersection) is "", not `#VALUE!`.
Evidence: poi/FormulaEvalTestData_Copy.xlsx (Excel-cached, Excel 15) EverythingTests!M1352, N1352,
O1352 (all top-left cells are non-text: 545, 1.001, blank).
Status: VERIFIED for these cells; UNVERIFIED (probe pending, T-0985): whether T reads the top-left
cell or is always "" for a multi-cell range, N's behaviour, and dynamic-array formulas.
Tasks: T-0985
