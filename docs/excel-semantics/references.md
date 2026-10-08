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
A span of one sheet is also that sheet's reference (unlike multi-sheet spans above) for the functions that need a reference: AREAS gives 1,
COUNTBLANK counts the blanks (2 for `Data1:Data1!A1:B3`) and INDEX reads the cell.
A span wrapped in IF or CHOOSE inside INDEX or MATCH gives `#VALUE!`, not
`#REF!`; `SUM(IF(TRUE,Data1:Data3!A1:A2))` is 113 (numbers of all three sheets, the text "5" skipped).
CHOOSE hands a span to an aggregate the same way: `SUM(CHOOSE(1,Data1:Data3!A1:A2))` = 113,
`COUNT(CHOOSE(1,Data1:Data3!A1))` = 3, `SUM(IF(TRUE,CHOOSE(1,Data1:Data3!A1)))` = 111; and so does LET:
`LET(x,Data1:Data3!A1,SUM(x))` and `SUM(LET(x,Data1:Data3!A1,x))` are 111. As a cell value,
`=CHOOSE(1,Data1:Data3!A1)` is `#VALUE!`, not the `#REF!` of the bare span. A span bound by LET or LAMBDA,
written out or from IF or CHOOSE, is `#VALUE!` to every value reader: `=x`, `x+1`, `ROWS(..)`, `INDEX(..,2)`
(`LET(x,CHOOSE(1,Data1:Data3!A1),x)`, `LAMBDA(x,x)(CHOOSE(1,..))`); only an aggregate reads it
(`LET(x,CHOOSE(1,Data1:Data3!A1),SUM(x))` = 111). `IF(TRUE,Data1:Data3!A1) A1` is `#VALUE!`. MAX skips text
behind IF bound by LET or LAMBDA (`LET(x,IF(TRUE,K1),MAX(x))` = 0 with K1 text) as it does for IF written
in place.
Evidence: probes/3d-ref-values, probes/3d-ref-functions incl. C15:C22 (excel-graph, 2026-10-06),
probes/3d-ref-choose-let and probes/3d-ref-let-values (excel-graph, 2026-10-08).
Span edges (probes/3d-ref-span-edges, excel-graph, 2026-10-08, T-1139; every form loaded, none was refused):
a span whose end is not a sheet is `#NAME?` (not `#REF!`): `=SUM(Data1:Nope!A1)` and `=SUM(Nope:Data3!A1)` (Calc!A1, A2),
while `ISREF(Data1:Nope!A1)` is FALSE (A3). The explicit implicit intersection of a span is `#REF!`, not
`#VALUE!`: `=@Data1:Data3!A1` (B1) and `=SUM(@Data1:Data3!A1:A2)` (B3); FORMULATEXT keeps `=@Data1:Data3!A1` (B4);
a one-sheet span is an ordinary reference: `=@Data1:Data1!A1` = 1 (B2). IF-wrapped spans: `AREAS(IF(TRUE,Data1:Data3!A1:A2))`,
`VLOOKUP(1,IF(TRUE,Data1:Data3!A1:B2),2,FALSE)`, `HLOOKUP(...)` and `COUNTBLANK(IF(TRUE,Data1:Data3!A1:B3))` are `#VALUE!`
(C1, C3, C4, C5); the one-sheet span `Data1:Data1` works: AREAS 1 (C2), COUNTBLANK 2 (C6).
Status: VERIFIED (web). Engine mismatches (baselined, T-1169): the engine gives `#REF!` for the missing span end,
`#VALUE!` for `@` of a span, `#REF!` for the HLOOKUP form and -4 for the COUNTBLANK form.
Implemented in: `get_sheet_span_value` (`#REF!`) and `get_sheet_span_reference` (`#VALUE!`) in
crates/rustsheet-engine/src/reference_operators.rs; VLOOKUP, MATCH and COUNTBLANK reach
the `#VALUE!` through `evaluate_single_sheet_arg` (functions/util.rs), which passes IF's chosen branch through
the same check (`IfBranchMode::SingleSheet`, T-1026); INDEX and AREAS read their argument with `evaluate_reference_arg` and refuse a 3-D result with `three_d_needs_one_sheet` (T-1063); IF-wrapped spans inside AREAS and VLOOKUP go through that path, HLOOKUP and COUNTBLANK do not yet (engine mismatches above, T-1169).
CHOOSE in an aggregate argument: `Model::choose_impl` (T-1064). A span bound by LET is still `#VALUE!` in the
engine, also to an aggregate (probes/3d-ref-choose-let C3:C4, 3d-ref-let-values C7, known failures, T-1158;
`Model::bind_argument` binds `#VALUE!` for a span so no value reader sees a range across sheets).
Tasks: T-0122, T-0197, T-1026, T-1064, T-1139, T-1169

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
XLOOKUP, TAKE, DROP and TRIMRANGE are references to ISREF when they pick cells of a range:
`ISREF(XLOOKUP(2,A1:A3,B1:B3))`, `ISREF(TAKE(A1:B3,1))`, `ISREF(DROP(A1:B3,1))` and `ISREF(TRIMRANGE(A1:B3))` are
TRUE, and so is `ISREF(XLOOKUP(2,{1;2;3},B1:B3))` (an array lookup vector, a range return); over an array
(`ISREF(TAKE({1,2;3,4},1))`) or when they fail (`XLOOKUP` not found, `DROP(A1:B3,5)`, `TAKE(A1:B3,0)`) they are FALSE
(probes/isref-reference-lookups, excel-graph 2026-10-07; T-0165; the array lookup vector is T-1126).
A defined name that stands for a cell or range is a reference to ISREF, a single cell included
(`ISREF(TextCell)` and `ISREF(Cells)` TRUE; a LAMBDA name, a constant name `=7` and an undefined name are FALSE), and FILTER's `if_empty` range is its
values, never a reference (`FILTER(A1:A1,B1:B1,D1:D2)` spills D1:D2, `ISREF(FILTER(..))` FALSE, `SUM(..)` = 30).
Evidence: probe isref-names-filter-if-empty (oracle: excel-graph, 2026-10-06, T-0940), matching upstream IronCalc ISREF.xlsx (ironcalc 9add6b67; Excel 16.03 cached values; to be added to calc_tests by T-0965) Sheet1!A14-A18, D11, D12, D14, D17, D20. ISREF of a LET/LAMBDA variable
bound to a reference is TRUE (A19-A26; implemented by T-0223, see "Names, LET/LAMBDA variables and @ are
references" below; T-0963 checks the ISREF.xlsx cells once T-0965 adds the workbook).
Implemented in: `evaluate_aggregate_arg` in `crates/rustsheet-engine/src/functions/util.rs` (IF, INDEX; used by SUM,
COUNT, MIN, MAX and every `for_each_value` caller: AVERAGE, MEDIAN, HARMEAN, GEOMEAN, SKEW, SKEW.P, KURT;
also AVERAGEA, MAXA, MINA, PRODUCT, CONCAT, LARGE and RANK since T-0196; VERIFIED for AVERAGEA, MAXA, MINA, PRODUCT, LARGE and RANK, see below);
ISREF of INDEX and of defined names in `fn_isref` (functions/information/mod.rs),
FILTER if_empty range in `fn_filter` (functions/spill_functions.rs)
Probe T-1003 (probes/if-index-ref-args-aggregates, excel-graph, 2026-10-07): MAXA, MINA, AVERAGEA, PRODUCT, LARGE and RANK give
the same result for `IF(TRUE,ref)` and `INDEX(ref,n)` as for the direct reference, over a text cell, a boolean cell and a mixed
range. A single text cell counts as 0 to MAXA, MINA and AVERAGEA (`MAXA(IF(TRUE,A1))` with A1 = "5" is 0) and a single TRUE as 1;
PRODUCT skips a text or boolean cell from a reference, so with no number left it gives 0 (numbers.md, "over no numbers");
LARGE of a one-cell reference with no number is `#NUM!`; RANK of a number absent
from a one-cell reference is `#N/A` (statistics.md, T-1055). Over `A1:A5` = {"5", TRUE, 7, "9", 3}: MAXA 7, MINA 0,
AVERAGEA 2.2, PRODUCT 21, LARGE(,1) 7, RANK(3,) 2.
Tasks: T-0216, T-0235, T-0940, T-1003

## Names, LET/LAMBDA variables and @ are references; FILTER's scalar if_empty is a value
Rule: a defined name for a single cell, a LET variable or LAMBDA parameter bound to a reference, and `@range`
are references, so MIN/MAX (and every `evaluate_aggregate_arg` caller) skip text in them:
`=MAX(TextCell)` and `=MIN(LocalText)` → 0 with the named cell = "text", `=MAX(TextCell,-3)` → -3,
`=LET(x,K1,MAX(x))` → 0 and `=LAMBDA(x,MIN(x,4))(K1)` → 4 with K1 = "text", `=LET(x,K1:K2,MAX(x))` → 7,
`=MAX(@A1:A3)` → the row's cell, 0 on the text row, and `#VALUE!` in a row outside A1:A3. A variable bound to
a value is still a value: `=LAMBDA(x,MAX(x))("text")` → `#VALUE!`. Where a value is wanted the variable is
its cell's value (`=LET(x,K1,x)` → "text", `=LET(x,K2,x+1)` → 8), and where a reference is wanted it is
the reference (`=LET(x,K1,ROW(x))` → 1, `ROW(TextCell)`, `OFFSET(TextCell,1,0)`, `ISREF(x)` TRUE).
FILTER with no match returns a scalar `if_empty` as itself, not as a 1x1 array: `=MAX(FILTER(A1:A3,
A1:A3=100,""))` → `#VALUE!` (MAX of the text value ""), with `"4"` → 4 and with TRUE → 1.
A one-cell reference as `if_empty` is its value too: with K1 = "text", `SUM(FILTER(..,..,K1))` and
`MAX(FILTER(..,..,K1))` are `#VALUE!`; with L1 empty, `MAX(FILTER(..,..,L1))` and `=FILTER(..,..,L1)` are 0.
Variables bound to a one-cell reference are references in other reference-taking functions as well:
`LET(x,K2,SUBTOTAL(9,x))` → 7, `LET(x,A1,SUM(x:A3))` → 12, `LET(x,K1,COUNTA(x))` → 1,
`LET(x,K1,SUMIF(x,"text",K2))` → 7, `LET(x,K1,ROWS(x))` → 1.
Not implemented: a LET whose result is a reference stays a reference to SUM (`=SUM(LET(x,K1,x))` → 0, we
give `#VALUE!`; T-1106).
Evidence: probes/min-max-reference-forms C1-G5 (excel-graph, 2026-10-07, T-0223); upstream IronCalc
MAX_MIN.xlsx (ironcalc d7b09c33) Sheet1!H39, H40, H43, H54 and MIN_MAX_extras.xlsx (ironcalc 16987342)
Sheet1!A1-A18, Sheet2!A1-A3 (Microsoft Macintosh Excel cached values) in ironcalc-calc/MATH_AND_TRIGONOMETRY.
Status: VERIFIED
Implemented in: `evaluate_node_with_reference` (model.rs), `variable_value` (functions/logical/let.rs), `evaluate_aggregate_arg`
(functions/util.rs), LET/LAMBDA bindings (functions/logical/let.rs, lambda.rs), `fn_isref`
(functions/information/mod.rs), `fn_filter` (functions/spill_functions.rs)
Tasks: T-0223

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
  binds tighter than `@`, which applies to the whole intersection after it. More forms
  (probes/intersection-at-operands, rows 1 to 5 of column H, oracle: excel-graph, 2026-10-07, T-0149):
  `=A1:A3 @A2:B2` in H1 is `#VALUE!` (`@A2:B2` has no cell in row 1), `=@A2:B2 A1:A3` in H2 is 2
  (`@(A2:B2 A1:A3)`, where `@A2:B2` alone would be `#VALUE!` in H2), `=A1 @A1 @A1` is 1,
  `=@A1:A3 A2:B2 + 1` is 3 and `=SUM(@A1:A3 A2:B2)` is 2.
- `=A1:A3 INDEX(A1:B3,2,0)`, `OFFSET(A2,0,0,1,2)` and `IF(TRUE,A2:B2)` give 2, on either side;
  `INDIRECT("A2:B2")`, `CHOOSE(1,A2:B2)`, `XLOOKUP(2,A1:A3,A1:B3)`, `SWITCH(1,1,A2:B2)`,
  `IFS(TRUE,A2:B2)`, `TAKE(A2:B3,1)`, `DROP(A1:B2,1)`, `TRIMRANGE(A2:B2)` and `LET(x,A2:B2,x)` give 2 as right operands.
- The stored form of `@` is accepted as an operand: `=A1 _xlfn.SINGLE(A1:A3)` = 1 and
  `=_xlfn.SINGLE(A1:A3) A2:B2` = 2 (row 2) load and evaluate, `=A1:A3 _xlfn.SINGLE(A2:B2)` in row 3 is `#VALUE!`
  (no cell of `A2:B2` in that row), `=A1 _xlfn.SINGLE(A1)` = 1 (probes/intersection-single-operands, oracle:
  excel-graph, 2026-10-07, T-1103). The engine matches, so exporting `A1 _xlfn.SINGLE(A1:A3)` is safe.
- A spill reference intersects: `=J1# J2:K2` = 2 with `J1 = SEQUENCE(3)`, stored as
  `_xlfn.ANCHORARRAY(J1)` (a literal `J1#` in the file is refused).
- `=A1:A3 (A2:B2,A2)` is `#VALUE!` (a two-area union is left over).
- `=(A1)(A2)`, with no space, is `#REF!`, not a syntax error: it is a call of `(A1)`, see "Calling a
  parenthesized operand". With a space, `=(A1) (A2)` is the intersection, `#NULL!`.
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
future function names, LAMBDA definitions and calls, and `@` operands are accepted); `Parser::parse_implicit`
(`@` takes the intersection after it, T-0149). The engine prints an `@` operand of an intersection as Excel's FORMULATEXT does, `=A1 @A1` and
`=@A1:A3 A2:B2` (probes/intersection-at-formulatext, T-1098), except that a left operand ending in `@` keeps its
parentheses, `(A1 @A1) B1`, since the bare text would re-parse as `A1 @(A1 B1)` (engine choice; Excel's text for it is not probed). The same text is printed when a formula is moved or pasted (`to_string_moved`). As intersection operands, XLOOKUP, TAKE,
DROP and TRIMRANGE over a single-sheet range are evaluated as the reference to the cells they pick
(`Model::evaluate_reference_form`, T-0165); everywhere else the engine still evaluates them to values,
and over an array they are not references (`=A1:A3 TAKE({1,2},1)` is `#VALUE!`, UNVERIFIED: Excel's
parser may refuse it). Counts from the end pick from the range's end (`=A1:A3 TAKE(A1:B3,-1)` is 3), a
call's own error passes through (`DROP(A1:B3,5)` and `TAKE(A1:B3,0)` are `#CALC!`, a not-found XLOOKUP `#N/A`),
and a not-found XLOOKUP with a range `if_not_found` is that reference (`=C1:C3 XLOOKUP(9,A1:A3,B1:B3,C1)` is
C1, and ISREF of it TRUE) (probes/intersection-lookup-operand-errors, excel-graph 2026-10-07).
- FORMULATEXT of an `@` operand of `:` or `@` keeps the written parentheses: `=A1:(@B1)` stays `=A1:(@B1)` (spills `1, 2` across H10:I10), `=(@(A1 B1)):C3` stays `=(@(A1 B1)):C3` and `=@(@(A1 B1))` stays `=@(@(A1 B1))` (both `#NULL!`). The engine drops the inner parentheses on the last two (probes/at-operand-formulatext, T-1104; T-1116).
- `=ANCHORARRAY(D1)` prints `=D1#` and `=ANCHORARRAY(INDEX(D1:D2,1))` prints `=INDEX(D1:D2,1)#`, `ANCHORARRAY(D1)` and `D1#` are the same stored file text (`_xlfn.ANCHORARRAY(D1)`), so this is how Excel prints that stored form; how a typed `ANCHORARRAY(D1)` prints is not probed. Engine requirement for T-1109: xlsx import must not print the automatic `@` it adds inside `#`. Stored as `_xlfn.ANCHORARRAY((@A1:B2))`, `=(@A1:B2)#` loads, prints as `=(@A1:B2)#` and is `#VALUE!` from D20 (no intersecting cell) (probes/anchorarray-at-operand, T-1111; the engine matches since T-1115). UNVERIFIED: the engine passes any `@` error through `#` (`=(@(A1 B1))#` gives `#NULL!`), and gives the left error when both operands of `:` are errors; only the `#VALUE!` and right-operand cases are probed. `=ANCHORARRAY(1)` is refused: Excel for the web will not open a workbook holding it (`FileCorruptTryRepair`, T-1111; engine in T-1117). `=@D1#` prints as `=@D1#` and evaluates to `#VALUE!` (probes/at-operand-formulatext, T-1104).
Tasks: T-0138, T-0148, T-0149, T-0042, T-0161, T-0165, T-1094, T-1098, T-1103, T-1104, T-1109, T-1111, T-1115, T-1117

## Calling a parenthesized operand
Rule: a parenthesized operand written directly before `(`, with no space, is called like a LAMBDA
(`LAMBDA(x,x+1)(2)`), and a called reference is `#REF!`, whatever its cell holds. With A1:B3 = 1, 2, 3 /
10, 20, 30: `=(A1)(A2)`, `=(A1)(A1)`, `=(A1:A3)(A2:B2)`, `=(A1:A3)(2)`, `=(A1)(A2,A3)`, `=(A1)()`,
`=(A1:A3 A2:B2)(1)` and `=(INDEX(A1:B3,1,1))(2)` are `#REF!`; so is `=(C1)(2)` where C1 holds
`=LAMBDA(x,x+1)` (C1 itself is `#CALC!`). The `#REF!` is an ordinary error: `=ISREF((A1)(A2))` is
FALSE, `=IFERROR((A1)(A2),"e")` is "e" and `=ERROR.TYPE((A1)(A1))` is 4. A parenthesized LAMBDA is
called: `=(LAMBDA(x,x+1))(2)` is 3 and `=(IF(TRUE,LAMBDA(x,x*2)))(3)` is 6. A parenthesized value is
refused: Excel for the web refuses a workbook holding `=(1)(2)` (`FileCorruptTryRepair`, one scratch
workbook, T-0149), so the engine reads it as a syntax error. A space makes it the intersection:
`=(A1) (A2)` is `#NULL!`.
A LET name called like a function: `=LET(f,A1,f(2))` is `#REF!` too, and with a value bound the results
are odd: `=LET(f,1,f(2))`, `=LET(f,A1+0,f(2))` and `=LET(f,{1,2},f(2))` are `#NAME?`,
`=LET(f,"a",f(2))` is 0 and `=LET(f,1/0,f(2))` is `#DIV/0!` (scratch probe, oracle: excel-graph,
2026-10-07, T-0149; not committed, UNVERIFIED whether the case builder's `_xlpm.` naming of a called
LET name plays a part). The engine gives `#VALUE!` for those calls (T-1092).
Evidence: probes/paren-call-operands H1:H10, probes/paren-call-empty-args, probes/paren-call-lambda-cell,
probes/paren-call-if-lambda, probes/paren-call-intersection, probes/paren-call-function-ref and
probes/intersection-operands H13 (oracle: excel-graph, 2026-10-05 and 2026-10-07, T-0138, T-0149).
Callee forms (probes/paren-call-callee-offset and -indirect H1, oracle: excel-graph, 2026-10-07, T-1094):
`=(OFFSET(A1,0,0))(2)` and `=(INDIRECT("A1"))(2)` load and are `#REF!`, calls of a reference. Excel for the
web refuses a workbook holding `=(1+1)(2)` or `=(SUM(A1))(2)` (`FileCorruptTryRepair`, one scratch
workbook each, not committed), so the engine's syntax error for a value callee is right. UNVERIFIED by
inference: the callees the engine accepts are the intersection's reference operands
(`is_reference_operand`), which include LAMBDA, calls of the reference-returning functions and names;
other value-returning callees are refused like `(1)(2)`. A second call,
`(A1)(1)(2)`, is left-over input (not probed).
Status: VERIFIED
Implemented in: expressions/parser/lambda.rs (`Parser::parse_parenthesized_call`);
functions/logical/lambda.rs (`Model::call_callee`); model.rs (the callee is evaluated with
`evaluate_aggregate_arg`, so a cell and a selecting INDEX or IF stay references)
Tasks: T-0149, T-1092

## A defined name may be an intersection
Rule: a defined name whose formula is an intersection is a reference. With `isect` =
`Sheet1!$A$1:$A$3 Sheet1!$A$2:$B$2` and `wide` = `Sheet1!$A$1:$B$3 Sheet1!$A$2:$B$3`: `=isect` = 2,
`=ISREF(isect)` = TRUE, `=isect A1:B3` = 2, `=ROWS(wide)` = 2, `=COLUMNS(wide)` = 2, `=SUM(wide)` = 55,
`=INDEX(wide,2,2)` = 30. A disjoint one, `disj` = `Sheet1!$A$1 Sheet1!$B$2`, is `#NULL!` and `ISREF(disj)` = FALSE.
Evidence: probes/defined-name-intersection Sheet1!H1:H9 (Sheet1 A1:A3 = 1, 2, 3, B1:B3 = 10, 20,
30; oracle: excel-graph, 2026-10-05, T-0043), first seen with a scratch xlsx in T-0138.
Status: VERIFIED (Excel for the web; implemented since T-0158)
Tasks: T-0138, T-0043, T-0158

## A defined name may be a constant
Rule: a defined name whose body is a constant evaluates to that value: `rate` = `0.5`, `=rate*4` = 2.
Evidence: probes/defined-name-intersection Sheet1!H10 (oracle: excel-graph, 2026-10-05, T-0043).
Status: VERIFIED (Excel for the web; implemented since T-0158)
Tasks: T-0043, T-0158

## A defined name may be a formula
Rule: any formula can be a defined name's body, and it is evaluated where the name is used. With
Sheet1 A1:A3 = 1, 2, 3 and B1:B3 = 10, 20, 30, and these names:

- `total` = `SUM(Sheet1!$A$1:$A$3)`: `=total` = 6.
- `plus1` = `Sheet1!$A$1+1`: `=plus1` = 2.
- `txt` = `"ab"`: `=txt` = "ab".
- `arr` = `{1,2,3}`: `=SUM(arr)` = 6 and `=COLUMNS(arr)` = 3.
- `chain` = `total*2`: `=chain` = 12.
- `pick` = `INDEX(Sheet1!$A$1:$B$3,2,2)`: `=pick` = 20.
- `off` = `OFFSET(Sheet1!$A$1,1,1)`: `=ROWS(off)` = 1.

A name whose body is a reference formula is a reference: `ISREF(pick)` and `ISREF(off)` are TRUE,
while `ISREF(total)` and `ISREF(chain)` are FALSE. A name that reaches itself is `#NAME?`, both
directly (`selfref` = `selfref+1`) and through another name (`loopa` = `loopb`, `loopb` = `loopa`).
`ISERROR(loopa)` is TRUE, and `ERROR.TYPE` of either is 5. Beware that names such as `cyc1` are
cell addresses (column CYC), not names.
Evidence: probes/defined-name-formula-bodies Sheet1!H1:H17 (oracle: excel-graph, 2026-10-07, T-0158).
Status: VERIFIED (Excel for the web), plus one DEVIATION (decision 0014). Name bodies may nest at most
64 deep, and one cell's formula (or, outside a cell, one outermost use) may evaluate at most
10,000 bodies in all, within the cell recursion's stack budget (evaluation.rs). Past any of these limits the use is `#NAME?`.
Excel gives `#NAME?` for a name cycle anyway, and the engine reaches it by running out of a
limit. Where Excel would evaluate, the engine still gives `#NAME?`: a chain of more than 64
distinct names, a body that uses names often enough to pass 10,000 evaluations (`d0` = `d1+d1`,
`d1` = `d2+d2`, …), and deeply nested bodies that pass the stack budget.
Tasks: T-0158

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
  `#REF!`, and `ISREF((A1,#REF!))` is FALSE (probes/union-accepted H9; implemented in `Model::union_is_reference`, T-0194). `ISREF((A1,B2))` is
  TRUE, and `=(A1,B2) A1` is 1. ISREF is also FALSE for a union with a negated operand (`(-A1,B1)`), with an
  empty-intersection operand (`(A1 B2,C1)`) and with a 3D operand (`(Sheet1:Sheet2!A1,B1)`), and for an intersection
  that is empty: `ISREF(A1 B2)` is FALSE (`#NULL!`), while `ISREF(A1 A1)`, `ISREF((A1 A1,C1))` and a multi-area cut `ISREF((A1:B2,A1:C1) A1:B1)` are TRUE, and a cross-sheet intersection
  `ISREF(Sheet1!A1 Sheet2!A1)` is FALSE (probes/isref-union-operands H1:H9, oracle: excel-graph, 2026-10-07, T-1039). The engine matches since T-1039
  (`ISREF` of an intersection evaluates its areas like a union does).
- Cross-sheet: `=(Data!A1,Other!A1)` and `=(Data!A1:A3,Other!A1:A3) Data!A2` are `#VALUE!`.
- Printing: Excel keeps the user's parentheses, so FORMULATEXT of `=((Data!A2,Data!E8),Data!B6)` is
  exactly that. The engine keeps a union operand's own parentheses (`grouped` on `OpUnionKind`), while a
  plain chain `(A1,B2,C3)` still prints as written (T-0193).
- Multi-area defined names: with `mu` = `Sheet1!$A$1,Sheet1!$B$2` (A1 = 1, B2 = 5), `=SUM(mu)` is 6,
  `=COUNT(mu)` is 2, `=ISREF(mu)` is TRUE, `=mu A1` is 1 and `=mu` is `#VALUE!`. The engine refuses
  the name body ("invalid defined name"), so the whole case is an engine error. Multi-area name
  bodies are SPEC-0004 step 5 (T-1065).
- Comma-decimal locales: xlsx stores formulas in en-US syntax with `,`, so the `;` spelling of the
  union operator exists only in the UI of such locales and cannot be probed through a file. The
  engine prints the argument separator `;` there (engine choice).
- Operand forms (T-0998, T-1031, probes/union-operand-*, oracle: excel-graph, 2026-10-06 and 2026-10-07): `=(--A1,B1)` loads, is
  `#VALUE!` and FORMULATEXT prints `=(--A1,B1)` (the engine matches since T-1024); three and four minuses
  (`=(---A1,B1)`, `=(----A1,B1)`) also load and are `#VALUE!`, so the engine accepts any number of minuses (probed up to four; five or more is an engine extension, T-1031); `=(A1%,B1)` is refused by Excel (`FileCorruptTryRepair`, as the engine does); `=(-(A1,B2),C1)` loads and is
  `#VALUE!`. A union holding a negated operand cannot be an operand of an intersection at all: Excel refuses
  (`FileCorruptTryRepair`, no oracle value) `=(-A1,B1) A1`, `=(-A1,B1) B1`, `=A1 (-A1,B1)` (the cut's right side),
  `=((-A1,B1),C1) A1` (negation in an inner union), `=(-(A1,B2),C1) A1`, `=(--A1,B1) A1` and
  `=(A1,-B1) A1` (negation on the right of the union). The engine refuses all of them (T-1024, T-1031) and extends the rule to unprobed shapes (a union nested on the right, deeper nesting, extra parentheses, a nested negated union right of the cut). Excel keeps every level of written parentheses: FORMULATEXT of
  `=((A1,B2))` is `=((A1,B2))` and of `=(((A1,B2),C1))` is `=(((A1,B2),C1))` (the engine keeps them since T-1024: `written` on `OpUnionKind`).
- Engine behavior, not probed: the union is the loosest operator inside its parenthesis and the
  intersection binds tighter, so `(A1,B1 C1)` is `(A1,(B1 C1))`. A union cut by an intersection runs the cut on each pair of areas,
  drops pairs with no shared cell, and gives `#NULL!` when nothing is left. Further cuts can bring
  several areas down to one (`(A1:A12,D1:D12) A4:D4 D4` is D4). The known gaps: a union reaching a
  function gets `#VALUE!` (`SUM((A1,B2))` before T-1062), so IFERROR silently
  takes its fallback for one, and so does a union returned by IF, CHOOSE or LET. docs/specs/SPEC-0004-multi-area-references.md
  (T-0133) designs multi-area arguments, with rollout T-1061 to T-1065. `add_implicit_intersection` (static_analysis.rs) leaves
  `OpUnionKind` unwrapped, and SPEC-0004 keeps it so: a union is already `#VALUE!` where a scalar is expected (planned, probe rows 15 and 18).
Evidence: probes/union-accepted H1:H9, probes/union-neg-operand H1:H2, probes/union-defined-name
H1:H5 (oracle: excel-graph, 2026-10-06, T-0187). The refusals come from scratch workbooks, one per
formula, which were not kept as cases.
T-0998: probes/union-operand-{double-minus,nested-parens,neg-cut-group} (same oracle and date); T-1031: probes/union-operand-{triple,quad}-minus (excel-graph, 2026-10-07); its refusals (percent,
negated operand cut by an intersection, and the T-1031 forms) came from scratch fills that failed with FileCorruptTryRepair and were not kept.
Status: VERIFIED (except the parts marked as engine choices; engine mismatches tracked in T-1024)
Implemented in: expressions/parser/reference_operators.rs (`Parser::parse_parenthesized`); reference_operators.rs
Tasks: T-0134, T-0187, T-0193, T-0194, T-0133, T-0998, T-1024, T-1031, T-1039

## Union area cap
(ISREF of a union over the cap is FALSE in the engine, where Excel presumably says TRUE; same DEVIATION.)
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
The area cap bounds the length of an area list, not the work of walking it: `MAX_AREAS` copies of
`A:A` fit in a short formula. So an aggregate argument of several areas (`SUM((A:A,C:C))`) first
adds up the cells of its areas, each clipped to its sheet's used area, in saturating arithmetic
and before any cell is read; more than 16,777,216 cells (`MAX_AREA_CELLS`, 2^24, 16 full used
columns) is `#VALUE!`. The bound is per argument, as a plain range argument's work already is.
Excel's limit, if any, is not known.
The LET variables and LAMBDA parameters in scope may hold at most 262,144 areas together (`MAX_HELD_AREAS`,
four `MAX_AREAS`): each binding may hold a fresh union (`LET(x_1,(A1,A1),x_2,(x_1,x_1),...)`), so memory would
otherwise grow with formula length. A binding that would pass the total is `#VALUE!`.
Status: DEVIATION (decision 0014)
Implemented in: reference_operators.rs (`MAX_AREAS`, `MAX_AREA_PAIRS`, `MAX_AREA_CELLS`,
`Model::union_areas`, `Model::count_union_areas`, `Model::intersect_areas`); functions/util.rs
(`Model::check_area_cells`); reference_operators.rs (`MAX_HELD_AREAS`, `VariableStack`)
Tasks: T-0134, T-1062, T-1064

## Multi-area references in functions
Rule: which functions take a union, and what they give (Sheet1 A1=1, B2=5, C3=7, D4=9; evidence
probes/multiarea-*, oracle: excel-graph, 2026-10-07, T-1061):
- Accept it, areas counted as written (overlap counted twice: `SUM((A1:B2,B2))` = 11): SUM, COUNT,
  COUNTA, AVERAGE, MIN, MAX, PRODUCT (`(A1,B2)` gives 6, 2, 2, 3, 1, 5, 5), LARGE, SMALL, RANK,
  MEDIAN, STDEV.S (`(A1,B2,C3)`: 5, 5, 2 for RANK(5,..), 5, 3.0550504633), FREQUENCY (`SUM(FREQUENCY((A1,B2),3))` = 2),
  SUBTOTAL 9 and 109 (6), INDEX with `area_num` (`INDEX((A1:B2,C3:D4),1,1,2)` = 7, area 3 `#REF!`, area 0
  `#VALUE!`, `area_num` omitted = 1; `SUM(INDEX(..,0,0,2))` = 16), AREAS (2, 3 for `((A1,B2),C3)`, 2 for
  `(A1:B2,A1:B2)`), CELL("address", `(A1,B2)`) = "$A$1, $B$2".
- `INDEX` `area_num` is read as a number and floored: below 1 is `#VALUE!`, past the last area (or NaN, or a
  huge value such as 1E+300) is `#REF!`; a reference of one area with `area_num` other than 1 is `#REF!`. `AREAS` of a
  3-D span is `#VALUE!`. Implemented by T-1063 (`index_impl`, `fn_areas`), cases in probes/multiarea-areas-* and
  probes/multiarea-index; unit tests in test/test_multi_area_arguments.rs. The 1E+300, "one area, area_num 2", `INDEX` of an
  array with `area_num` other than 1 (`#REF!`) and `AREAS` of a non-reference (1; Excel refuses the formula)
  rows are engine choices, UNVERIFIED (no probe).
- Refuse with `#VALUE!`: SUMPRODUCT, SUMIF, COUNTIF, COUNTBLANK, OFFSET, `@(A1,B2)`, the bare `=A1,B2` (stored in a
  file; it loads), `=(A1,B2)` itself. `ROWS` and `COLUMNS` give `#REF!`, and so does `ROW`.
- ISERROR and ISNUMBER of a union are TRUE and FALSE, IFERROR gives its fallback (the union is `#VALUE!` as a value).
- A union stands for its areas through `LET` (`LET(x,(A1,B2),SUM(x))` = 6, `x A1` = 1, bare `x` `#VALUE!`), `IF`/`CHOOSE`
  (`IF(TRUE,(A1,B2)) A1` = 1, `SUM(IF(TRUE,(A1,B2)))` = 6, `SUM(CHOOSE(2,A1,(A1,B2)))` = 6) and an intersection
  that leaves two areas (`SUM((A1:A3,B1:B3) A1:B2)` = 6). It stays its areas wherever it is bound or picked:
  through another variable (`LET(x,(A1,B2),y,x,SUM(y))` = 6, `y A1` = 1, bare `y` `#VALUE!`), a LET value from
  IF or CHOOSE (`LET(x,IF(TRUE,(A1,B2)),SUM(x))` = 6), a LAMBDA argument (`LAMBDA(x,SUM(x))((A1,B2))` = 6), a LET
  body (`SUM(LET(x,(A1,B2),x))` = 6, `LET(x,(A1,B2),x) A1` = 1), the else branch and a truncated index
  (`SUM(IF(FALSE,1,(A1,B2)))`, `SUM(CHOOSE(1.9,(A1,B2),C3))` = 6), nested IF/CHOOSE (`IF(TRUE,IF(TRUE,(A1,B2))) A1`
  = 1) and as a union operand (`SUM((IF(TRUE,(A1,B2)),C3))` = 13). A nested LET shadows a union binding as any
  other (`LET(x,(A1,B2),LET(x,C3,SUM(x)))` = 7; probes/multiarea-letbind, excel-graph, 2026-10-08).
- Cross-sheet: `SUM((Data!A1,Other!A1))` and `AREAS(..)` are `#VALUE!`. A name with a cross-sheet union body
  (`Data!$A$1,Other!$A$1`) loads, `SUM(xs)` and `=xs` are `#VALUE!` and `ISREF(xs)` is FALSE (unlike a same-sheet
  union name, which is TRUE).
- 3-D: `SUM(Sheet1:Sheet2!A1,(A1,B2))` = 107 (3-D argument plus the union), but a 3-D operand inside the union,
  `SUM((Sheet1:Sheet2!A1,B2))`, is `#NAME?`.
  Implemented in `evaluate_union_operand` (reference_operators.rs, T-1070): a multi-sheet span as a direct
  operand of a union gives `#NAME?`. Only the left operand under SUM is VERIFIED; the right operand, the bare
  `=(Sheet1:Sheet2!A1,B2)` value context (same check in `count_union_areas`) and the precedence against another
  operand's error (`SUM((1/0,Sheet1:Sheet2!A1))` is `#DIV/0!` in the engine) are UNVERIFIED (no probe); a 3-D
  operand of an intersection is unprobed.
- `_xlnm.Print_Area` = `Sheet1!$A$1,Sheet1!$B$2` (sheet-local): `SUM(Print_Area)` = 6, `AREAS(Print_Area)` = 2.
- FORMULATEXT of a union written as a dynamic-array formula (the only kind the probe harness writes) has no `@`
  (`=(A1,B2)`, `=SUM((A1,B2))`); a written `@(A1,B2)` is kept (`=@(A1,B2)`), and gives `#VALUE!`. The same holds for a legacy
  (non-array, no `cm`) formula: FORMULATEXT is `=(A1,B2)`, `=@(A1,B2)` and `=SUM((A1,B2))`, values `#VALUE!`,
  `#VALUE!`, 6; Excel does not add an `@` to a union in a legacy formula (probes/multiarea-at-legacy, T-1077). Controls in the same case: legacy `=A1:A2` and `=ABS(A1:A2)` (rows 7, 9) print
  `=@A1:A2` and `=ABS(@A1:A2)` and give `#VALUE!` (empty intersection), so the legacy path shows `@`s Excel adds;
  a union never gets one. The eval adapter does not yet load legacy cells as legacy (T-1096).
  So the engine must not wrap a union in `@` when printing a legacy formula.
- `AREAS(Data1:Data3!A1)` is `#VALUE!`. A workbook holding `AREAS(1)` or `AREAS("A1")` is refused by Excel for the web
  (`createSession` 501 `unsupportedWorkbook`, two scratch runs, not kept as cases, like the T-0138 refusals).
- ISREF of a cross-sheet union written in place is FALSE (`ISREF((Data!A1,A1))`), of a same-sheet one TRUE
  (`ISREF((Data!A1,Data!B2))`). An error cell inside an area is the aggregate's error as in a plain range, the
  first in formula order winning (`SUM((A1,E1))` = `#DIV/0!` with E1 = 1/0, `SUM((E2,E1))` = `#N/A` with E2 = NA()),
  COUNT skips it (1) and COUNTA counts it (2). SUMSQ, AVERAGEA, MAXA, GEOMEAN, VAR.S and DEVSQ take a union too
  (`(A1,B2)`: 26, 3, 5, 2.2360679775, 8, 8). PERCENTOF is `#NAME?` in Excel for the web, so it is not probed
  (probes/multiarea-more, oracle: excel-graph, 2026-10-08, T-1062).
Status: VERIFIED (Excel for the web)
Implemented in: AGGREGATE and the intersection; since T-1062 every aggregate on the shared argument path
(`Model::evaluate_reference_arg`, functions/util.rs: SUM, COUNT, COUNTA, AVERAGE, AVERAGEA, MIN, MAX, MINA,
MAXA, PRODUCT, the STDEV/VAR families and the other users of `for_each_value`) walks the areas one after the
other (`Model::for_each_cell_in_areas`). Since T-1068 LARGE, SMALL, RANK.EQ/RANK.AVG and the data_array of FREQUENCY
take a union the same way (`Model::values_from_areas`; FREQUENCY's bins_array is unprobed and stays a single reference).
Since T-1064 a LET variable or LAMBDA parameter bound to a union
(`Binding::Areas`, reference_operators.rs; `Model::bind_argument`, functions/logical/let.rs), the branch IF or
CHOOSE picks with a scalar condition or index (`Model::if_impl`, `Model::choose_impl`) and LET's body
(`Model::let_impl`) stay areas in reference context and as intersection operands.
UNVERIFIED (no probe): an IF or CHOOSE over an array condition keeps its value form.
SPEC-0004 and T-1065, T-1069 to T-1073 implement the rest;
until then CONCAT and TEXTJOIN read a union as `#VALUE!` (`RefArg::into_single_area`).
The other users of `for_each_value` (HARMEAN, AVEDEV,
SKEW, KURT, ...) and PERCENTOF take a union with no probe of their own.
COUNTBLANK (`#VALUE!`) and ROWS/COLUMNS (`#REF!`) of a union: `Model::rows_columns_extent` (cast.rs) and `fn_countblank`
(statistical/count.rs), T-1069. UNVERIFIED engine behavior: a cross-sheet union in ROWS/COLUMNS stays `#VALUE!`, and
a union holding an error in COUNTBLANK counts nothing (no probes).
SUBTOTAL 1-11 and 101-111 walk the areas of a union (`subtotal_walk_areas`, T-1071, T-1154; probes/multiarea-subtotal, probes/multiarea-subtotal-counts-a/b/e,
excel-graph). COUNT/COUNTA (2, 3, 102, 103) of a union count each area as written: `SUBTOTAL(3,(A1:A3,A1:A3))` = 6; an overlap counts its cells twice
(`SUBTOTAL(9,(A1:B2,B2))` = 11, COUNT 3); a SUBTOTAL cell inside an area is skipped (`(A1:B3,B3)` with B3 `=SUBTOTAL(9,A1:A2)` sums 6); an error cell
makes 1/9 an error but COUNTA counts it (`SUBTOTAL(3,(A1,C4))` = 2, `SUBTOTAL(2,(A1,C4))` = 1). A union across sheets is `#VALUE!` for 2, 3 and 9
(`(A1,Data!A1)`). The same unions through `LET(x,(A1,B2),SUBTOTAL(9,x))` (6), `IF(TRUE,(...))` and `CHOOSE(1,(...))` also work in Excel (counts 2); the engine
matches (T-1064; probes/multiarea-subtotal-counts-c/d). Hidden/filtered rows in a union remain
unprobed (the case format cannot hide rows; T-1036 owns the probe): the engine keeps or skips a row whole by the status at the area's first column.
Tasks: T-1061, T-1062, T-1064, T-1068, T-1069

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
Rule: with two cell corners that both carry the sheet prefix, the corners are ordered whichever way they are
written (`Data!B2:Data!A1`, `Data!A2:Data!B1` and `'Data'!$B$2:Data!A1` sum the same 2x2 block as
`Data!A1:Data!B2`; `ROWS(Data!B2:Data!A1)` is 2), and FORMULATEXT keeps the written order and both
prefixes (`=SUM(Data!B2:Data!A1)`, `=SUM(Data!$B$2:Data!A1)`; the quotes around `Data` are dropped).
Corners on two different sheets are not a range: `Data!A1:Other!B2` and `Data!A1:Other!A1` are `#VALUE!`,
`ISREF(Data!A1:Other!B2)` is FALSE and `IFERROR(SUM(...),"e")` yields `"e"`.
The forms with column or row corners are not ranges in Excel (T-0139, see the next entry).
None of these forms made the workbook unloadable.
Evidence: probes/sheet-qualified-corner-order Calc!A1-A4, B1, B2, B4 (order, text), A6-A9 (two sheets),
A11/B11 (control) (oracle: excel-graph, 2026-10-08, T-1138).
Status: VERIFIED (web)
Implemented in: `get_range` (crates/rustsheet-engine/src/reference_operators.rs). Known mismatch: the engine
returns TRUE for the ISREF form (Calc!A7, baselined; T-1166).
Tasks: T-1138, T-1166

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
Status: VERIFIED (Excel-cached). Open (unprobed): `[Book1]My Sheet`, brackets elsewhere in the name, an empty book (`[]Sheet1`), a book part with spaces or dots, an empty sheet part (`[Book1]`) and a cell-reference-like sheet part (`[Book1]A1`); the implementation decides on the part after `]` only.
Implemented in: `address_sheet_text` in functions/lookup_and_reference/address_areas.rs
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

## FORMULATEXT keeps the parentheses of an `@` operand; ANCHORARRAY is `#`; `:` passes an error operand on
Rule: Excel keeps the parentheses the user wrote around the operand of `@`, for every operand form
probed (intersection, range, single cell, nested `@`; `@(INDEX(…))`, `@(D1#)` and `@(name)` are not probed):
`=(@(A1 B1)):C3` prints `=(@(A1 B1)):C3`, `=@(@(A1 B1))` prints `=@(@(A1 B1))`, and `=@((A1 B1))`,
`=@(A1:B1)` and `=@(A1)` print as written (values `#NULL!`, `#VALUE!` from row 31, and 1), while a
bare `@A1:A3 A2:B2` stays bare. Typed `ANCHORARRAY(D1)` and `ANCHORARRAY(INDEX(D1:D2,1))` evaluate
as the spill operator and print `=D1#` and `=INDEX(D1:D2,1)#`. An error in either operand of `:`
is the result: `=(@(A1 B1)):C3` is `#NULL!`, `=A1:INDEX(A1:A3,5)` is `#REF!` and `=A1:(A1 B2)` is
`#NULL!`, not `#VALUE!`. The operands of `:` must be references, as for the union: Excel for the
web refuses a workbook holding `=A1:(1/0)`, `=(1/0):(A1 B1)`, `=(1/0):(A1 A2)` or `=ANCHORARRAY(1)`
(`FileCorruptTryRepair`, one scratch workbook per formula, T-1111), so no value exists for them.
`(@(A1 B1))`, `INDEX(…)` and `(A1 B2)` count as references even when they evaluate to an error.
Evidence: probes/at-operand-formulatext H12, H13, H17, J1, J30, K1, K30; probes/at-paren-operands
D30:E32; probes/range-right-operand-errors D11, E11, D14, E14; probes/anchorarray-at-operand (excel-graph).
Status: VERIFIED for these cells (refusals: excel-graph only; typed entry on desktop is an M14
question). Implemented in: `ImplicitIntersection::written` (parser/mod.rs), printed by `at_operand`
(parser/reference_operators.rs; T-1116).
Deeper levels, `@((A1,B2))` and `@(A1) B1` are probed in the entry "`@` operand parentheses: every level is kept" below (T-1132).
Engine gap: the refused forms, which the engine evaluates (T-1117).
Tasks: T-1109, T-1111, T-1115, T-1116, T-1117

## `@` operand parentheses: every level is kept; a group that is only part of the operand is kept too
Rule: FORMULATEXT prints every parenthesis level the user wrote around an `@` operand: `=@((A1))` and
`=@(((A1)))` print as written (value 1), `=@((A1:B1))`, `=@((A1,B2))` and `=@(A1:B2)` print as written (value
`#VALUE!`), `=@(((A1 B1)))` and `=@(A1:A2 B1:B2)` print as written (value `#NULL!`). When the group is
only part of the operand, `=@(A1) B1` and `=@((A1)) B1` print as written, with the parentheses, and are `#NULL!`
(`@(A1)` intersects with `B1`). The engine prints `=@A1 B1` for both (it dropped the group) until T-1134 (`ImplicitIntersection::lead`). Shapes where the group is not
exactly the leftmost operand (`@(A1 B1) C1`, `@(A1):B1 C1`) are unprobed and print without it. The stored `_xlfn.SINGLE(...)` form cannot be read through excel-graph
(FORMULATEXT shows the displayed formula only), so whether the stored form keeps the parentheses is an M14
desktop question (decision 0039).
Evidence: probes/at-paren-operands-levels D30:E39 (excel-graph, T-1132).
Status: VERIFIED (excel-graph); stored form UNVERIFIED
Tasks: T-1132, T-1134
