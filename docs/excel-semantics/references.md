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
(`LET(x,CHOOSE(1,Data1:Data3!A1),SUM(x))` = 111). A span bound directly behaves the same way, like the bare
span (probes/3d-ref-let-bound-span, excel-graph, 2026-10-09, T-1158). Aggregates read it, including through a second
variable or a LAMBDA: `LET(x,Data1:Data3!A1,y,x,SUM(y))`, `LAMBDA(x,SUM(x))(Data1:Data3!A1)` and
`LET(x,..,SUM(IF(TRUE,x)))` are 111, `COUNT(x)` is 3, `SUM(x,x)` is 222 and `MAX(x)+0` is 100. `INDEX(x,1)`, `ROWS(x)`,
`x+1` and `LAMBDA(x,x)(..)` are `#VALUE!`, as `INDEX(Data1:Data3!A1,1)` and `ROWS(Data1:Data3!A1)` are, and `ISREF(x)` is
FALSE, as `ISREF(Data1:Data3!A1)` is. More readers (probes/3d-ref-let-bound-span-readers, same date) also treat it
like the bare span. CONCAT and TEXTJOIN walk it (`"110100"`, `"1,10,100"`). AGGREGATE(9,0,..), AREAS and
COUNTBLANK are `#VALUE!`, and `SUM((x,Data1!B1))` is `#VALUE!`. The bare union `SUM((Data1:Data3!A1,Data1!B1))` is
`#NAME?`, not `#VALUE!` (the engine matches). `IF(TRUE,Data1:Data3!A1) A1` is `#VALUE!`. MAX skips text
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
Both ends missing (probes/3d-ref-span-missing-ends, excel-graph, 2026-10-10, T-1181): `Nope1:Nope2!A1` is `#NAME?` like a span
with one missing end (`=SUM(Nope1:Nope2!A1)`, `=Nope1:Nope2!A1`; `ISREF` FALSE, `ISERROR` TRUE), and `=COUNT(Nope1:Nope2!A1:B3)` is 0.
CHOOSE-wrapped spans in COUNTBLANK act like IF-wrapped ones: `COUNTBLANK(CHOOSE(1,Data1:Data3!A1:B3))` and
`COUNTBLANK(CHOOSE(2,1,Data1:Data3!A1:B3))` are `#VALUE!`, `COUNTBLANK(CHOOSE(1,Data1:Data1!A1:B3))` is 2, and a span with a missing
end in CHOOSE is `#NAME?`. A reversed span (`SUM(Data3:Data1!A1)`) is the same sheets: 111.
COUNTBLANK of an error (probes/countblank-choose-errors, excel-graph, 2026-10-10, T-1181): an argument that evaluates to an error,
not a reference, gives that error: `COUNTBLANK(Data1:Nope!A1:B3)` and its IF and CHOOSE forms `#NAME?`, `COUNTBLANK(CHOOSE(1,#N/A))`,
`CHOOSE(1,NA())` and `IF(TRUE,NA())` `#N/A`, an empty intersection in CHOOSE `#NULL!`; also (probes/countblank-error-forms-b,
same date) a direct empty intersection `COUNTBLANK(A5:A6 B5:B6)` `#NULL!`, `COUNTBLANK(INDIRECT("Q0"))` and
`COUNTBLANK(OFFSET(A1,-5,0))` `#REF!`. A reference to one cell holding an error still counts 0, however it is handed on
(ironcalc-calc COUNT.xlsx E18; probes/countblank-error-cell-refs and countblank-error-forms-b E1, E7): `A1`, `A1:A1`, a cell name,
`CHOOSE(1,A1)`, `INDEX(A1:A2,1)`, `OFFSET(A1,0,0)`, `IF(TRUE,A1)`, `XLOOKUP(1,{1,2,3},A1:C1)` and `TAKE(A1:C1,1,1)` with A1 an error. A reference to one missing sheet (`Nope!A1:B3`) is left out: the web oracle resolves it to
an empty cell (lookup.md); the engine gives `#REF!` there, following the desktop evidence in lookup.md. Not probed: an error
kind other than those above reaching COUNTBLANK as a value (`#DIV/0!`, `#NUM!`, `#SPILL!`, `#CALC!`, e.g. `COUNTBLANK(CHOOSE(1,1/0))`);
the engine returns it like the others.
Status: VERIFIED (web).
Implemented in: `get_sheet_span_value` (`#REF!`) and `get_sheet_span_reference` (`#VALUE!`) in
crates/rustsheet-engine/src/reference_operators.rs; VLOOKUP, MATCH and COUNTBLANK reach
the `#VALUE!` through `evaluate_single_sheet_arg` (functions/util.rs), which passes IF's chosen branch through
the same check (`IfBranchMode::SingleSheet`, T-1026); INDEX and AREAS read their argument with `evaluate_reference_arg` and refuse a 3-D result with `three_d_needs_one_sheet` (T-1063); IF-wrapped spans inside AREAS and VLOOKUP go through that path, HLOOKUP and COUNTBLANK read it through `evaluate_single_sheet_arg` (T-1170). A span whose end is not a sheet is `#NAME?` (`Model::wrong_sheet_error`, reference_operators.rs); `@` of a multi-sheet span is `#REF!` (`evaluate_at_operand_reference`).
CHOOSE in an aggregate argument: `Model::choose_impl` (T-1064). A span bound by LET or LAMBDA is a `Binding::Span`
(`Model::bind_argument`, functions/logical/let.rs). Only `Model::variable_reference` hands it out as a range, so it reaches the aggregates;
every value read is `three_d_needs_one_sheet`, and ROWS/COLUMNS and COUNTBLANK refuse it through `RefArg::into_single_sheet_area` (T-1158).
COUNTBLANK of an error argument: `fn_countblank` (functions/statistical/count.rs, T-1181) reads its argument once
through `evaluate_picked_reference_arg`, so a reference stays a range and only an error value is returned.
Tasks: T-0122, T-0197, T-1026, T-1064, T-1139, T-1158, T-1170, T-1181

## 3-D references when sheets or cells change (unprobed)
Rule (engine, not Excel evidence): renaming an end sheet renames it in the formula, and moving a
sheet re-resolves the span by name. Deleting an end sheet makes the reference `#REF!`, and inserting
or deleting rows or columns does not shift the span. Excel's own rules for these edits are not probed.
The excel-graph oracle only recalculates an uploaded workbook and cannot delete a sheet, so a span that lost an end sheet
cannot be probed there (T-1181); the M14 desktop oracle question is T-1310.
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
The same holds with no externalLink part in the workbook at all, and the result is still a
reference of the written shape whose cells read `#BLOCKED!`: `='[other.xlsx]Sheet1'!A1` and
`=SUM(…!A1:B1)` are `#BLOCKED!`, ERROR.TYPE of either is 11, `ISREF(…!A1)` is TRUE, ISERROR TRUE,
IFERROR takes the fallback, `COUNT(…!A1:B1)` is 0 and `ROWS(…!A1:B3)` is 3. The unquoted prefix
`=SUM([other.xlsx]Sheet1!A1:B1)` parses and is `#BLOCKED!` too.
Evidence: probes/external-ref-no-link Calc!A1:A10 (excel-graph, 2026-10-11, T-0124).
With a cache (`[1]Sheet!ref` and an externalBook `sheetDataSet`), Excel reads the cached value:
`=[1]Uses!$A$1` → "Hello!", `=[1]!NR_Global_B2` → 142 (poi/ref-56737.xlsx Refs!E3, I3, Excel-cached).
The shape functions read it as a reference too: `COLUMNS(…!A1:C2)` is 3, `AREAS(…!A1:C2)` 1,
`ROW(…!B5)` 5, `COLUMN(…!B5)` 2, `ROWS(…!A:A)` 1048576, `ISREF(…!A1:B3)` TRUE, and the unquoted
`ROWS([other.xlsx]Sheet1!A1:B3)` 3. INDEX keeps it a reference (`ROWS(INDEX(…!A1:C4,2,0))` 1,
`ISREF(INDEX(…!A1:C4,2,2))` TRUE), but the cell INDEX picks is not an error value:
`ERROR.TYPE(INDEX(…!A1:C4,2,2))` is `#N/A`, where `ERROR.TYPE(…!A1)` is 11. OFFSET on it is
`#VALUE!`; a union of two is 2 areas, the intersection `…!A1:B3 …!A2:B5` has 2 rows, `ISBLANK(…!A1)`
is FALSE and `COUNTA(…!A1:B2)` 0 (it counts no `#BLOCKED!`). `='[other.xlsx]Sheet1'!A1:B2` spills
(`#SPILL!` with the spill area occupied). A 3-D prefix naming another workbook is no reference:
`SUM('[other.xlsx]S1:S2'!A1)` is `#BLOCKED!` and `ISREF('[other.xlsx]S1:S2'!A1)` FALSE; a range
whose corners are in two workbooks (`ISREF('[other.xlsx]Sheet1'!A1:Calc!B2)`) is FALSE.
Evidence: probes/external-ref-no-link-shapes Calc!A1:A20 (excel-graph, 2026-10-12, T-1381).
The cell INDEX picks reads as **empty**, not `#BLOCKED!`: `INDEX(…!A1:C4,2,2)` is 0 in a cell,
`+1` gives 1, `&"x"` gives "x", ISBLANK is TRUE, ISERROR, ISTEXT and ISNUMBER FALSE, and ERROR.TYPE
`#N/A`; `INDEX(…!A1,1,1)`, the area INDEX picks from a union, and the one cell an intersection
leaves (`ERROR.TYPE(…!A1 …!A1:B2)`) read empty too. Several cells are `#BLOCKED!` each, spilled:
`=INDEX(…!A1:C4,2,0)` spills three, `='[other.xlsx]Sheet1'!A1:B2` and `=…!A1:B2*1` four, and
`ERROR.TYPE(…!A1:A3)` lifts (`#SPILL!` where blocked). Aggregates over any of these are `#BLOCKED!`
(`SUM(INDEX(…,2,2))`, `SUM(INDEX(…,2,0))`, `SUM((…!A1,…!B2))`, `SUM(…!A1:B3 …!A2:B5)`,
`SUMPRODUCT(…!A1:B2)`), but COUNTA counts nothing in them, even the written `…!A1` that reads
`#BLOCKED!` (`COUNTA(…!A1)` 0, `COUNTA(…!A1:B2,1)` 1), COUNT gives 0 and COUNTBLANK `#VALUE!`.
`OFFSET(…!A1,1,1)` is `#VALUE!` (ISREF FALSE, ERROR.TYPE 3), `OFFSET(…!A1:B2,0,0)` too. A union or
intersection of an external reference with one on this workbook is `#VALUE!` (`AREAS((…!A1,E1))`,
`ROWS(…!A1:B3 E1:F3)`); disjoint external areas intersect to `#NULL!`; `ISREF(…!A1:B3 …!A2:B5)` and
`ISREF(INDEX((…!A1,…!B2),1,1,2))` are TRUE, `ROWS(INDEX((…!A1:B2,…!C3:D5),1,1,2))` 1. IF and LET keep
the reference too: `ROWS(IF(TRUE,…!A1:B3))` and `LET(r,…!A1:B3,ROWS(r))` are 3, and the cell IF picks
reads empty (`ERROR.TYPE(IF(TRUE,…!A1))` is `#N/A`), so only a written external reference read
directly as a value is `#BLOCKED!`.
Evidence: probes/external-ref-no-link-index Calc!A1:A36, C1:F7 (excel-graph, 2026-10-13, T-1454).
Engine: a sheet prefix holding `[` (a sheet name cannot) gives `#BLOCKED!` instead of `#REF!`; the
lexer reads the unquoted `[book]Sheet!` prefix (T-1381). `Model::external_reference`
(external_reference.rs) gives the areas of a written external reference, an INDEX into one and a
union or intersection of them; ISREF, ROWS, COLUMNS, ROW, COLUMN, AREAS, COUNTA, COUNTBLANK, OFFSET,
INDEX's value, the intersection's value and the aggregates read it (T-1454). A `[n]` reference the
externalLink cache holds, one cell or several, reads the cache on all these paths
(`external_links.rs`; "Cached values of a linked workbook" below, T-1382, T-1460); there COUNTBLANK
is `#VALUE!` too, a one-cell `COUNTBLANK([1]Uses!$B$3)` included
(fixtures/external-link-cached-ranges Refs!B13, B69).
IF and CHOOSE that pick an external reference **are** that reference, through nested IFs too:
`ROWS(CHOOSE(1,…!A1:B3))` 3, `ROWS(IF(TRUE,…!A1))` 1, `COLUMNS(CHOOSE(2,A1,…!A1:C1))` 3,
`ROW(IF(TRUE,…!B3))` 3, `ROWS(IF(TRUE,IF(FALSE,A1,…!A1:B3)))` 3, `ISREF(IF(TRUE,…!A1))`,
`ISREF(CHOOSE(1,…!A1:B3))` and `ISREF(IF(TRUE,…!A1:B3,A1))` TRUE, `AREAS(IF(TRUE,(…!A1,…!B2)))` 2,
`COUNTA(IF(TRUE,…!A1:B2))` and `COUNTA(CHOOSE(1,…!A1))` 0, `COUNTA(IF(TRUE,…!A1),1)` 1,
`COUNTBLANK(IF(TRUE,…!A1:B2))` `#VALUE!`, and INDEX into one picks from it
(`ERROR.TYPE(INDEX(IF(TRUE,…!A1:C4),2,2))` `#N/A`, `ROWS(INDEX(IF(TRUE,…!A1:C4),2,0))` 1). The one
cell they pick reads empty, as INDEX's does: `=IF(TRUE,…!A1)` and `=CHOOSE(1,…!A1)` are 0, `&"x"`
gives "x", `+1` 1, ISBLANK TRUE, `ERROR.TYPE(CHOOSE(1,…!A1))` `#N/A`; several cells spill
`#BLOCKED!` (`=IF(TRUE,…!A1:B2)`, `=CHOOSE(1,…!A1:B2)`), and aggregates stay `#BLOCKED!`
(`SUM(IF(TRUE,…!A1))`, `SUM(CHOOSE(1,…!A1:B2))`, `SUM(IF(TRUE,…!A1:B3,A1))`). An array condition
lifts IF as usual: `ROWS(IF({TRUE,FALSE},…!A1:B3,…!A1:B3))` is 3, and `ROW(IF(FALSE,…!A1))`
(FALSE, no reference) `#VALUE!`. COUNTA of the two-area external union `(…!A1,…!E1)` is 0; it
counts 1 for an argument that gives an error instead of a reference: the union or intersection
with a local cell (`COUNTA((…!A1,E1))`, `COUNTA(…!A1 E1)`, both `#VALUE!`) and the disjoint
`…!A1:B3 …!D4:E5` (`#NULL!`). OFFSET reads its numbers before giving its `#VALUE!`:
`OFFSET(…!A1,1/0,1)`, `OFFSET(…!A1,1,1/0)` and `OFFSET(…!A1,1,1,1,1/0)` are `#DIV/0!`, while
`OFFSET(…!A1,1,1,0)` (height 0) is still `#VALUE!`.
Evidence: probes/external-ref-no-link-ifchoose Calc!A1:A34, H1:N2, external-ref-no-link-ifchoose-b
Calc!A1:A6 (excel-graph, 2026-10-14, T-1487).
Engine: `Model::pick_branch` reads the condition or index of an IF or CHOOSE that may pick an
external reference once and hands the picked argument to ROWS, COLUMNS, ROW, COLUMN, ISREF, AREAS,
COUNTA, COUNTBLANK, INDEX and the aggregates; IF's and CHOOSE's value read it as INDEX's (T-1487).
Not yet implemented: `ROWS(LET(c,TRUE,IF(c,…!A1:B3)))` is 3 (a LET body picking one; we give
`#BLOCKED!`), `ROWS(CHOOSE({1,2},…!A1:B3,A1))` is 3 (we give 1), and INDEX with an index array into
an external reference reads empty cells (`=INDEX(…!A1:C4,{1,2},1)` spills 0, 0; we give `#BLOCKED!`).
Unprobed: ROW and COLUMN of an IF lifted over an array condition that may pick an external
reference (`ROW(IF({TRUE,FALSE},…!A1,…!A1))`; the engine gives `#VALUE!`, UNVERIFIED), OFFSET of one
with an array or two error arguments, a `[n]` index with no
externalLink part, and ROW/COLUMN of a multi-cell external area (the engine spills the row or column
numbers as for a sheet reference, UNVERIFIED).
Status: VERIFIED (Excel for the web; error name inferred from errorType 10 and ERROR.TYPE 11)
Tasks: T-0124, T-1381, T-1382, T-1454, T-1487

## Cached values of a linked workbook (`[n]Sheet!A1`, `[n]!Name`, `[0]!Name`)
Rule: in xlsx, Excel writes a reference into another workbook with the 1-based index of its
`externalReference` in `workbook.xml`: `[1]Sheet!$A$1`, a workbook-scoped defined name of that book
`[1]!Name`, and a sheet-scoped one `[1]Sheet!Name`. Without opening the other file, Excel gives the
values it cached in `xl/externalLinks/externalLinkN.xml` (`externalBook`: `sheetNames`,
`definedNames` with `refersTo` and, for a sheet-scoped name, `sheetId` = index in `sheetNames`,
and `sheetDataSet/sheetData[@sheetId]/row/cell`): `=[1]Uses!$A$1` → "Hello!" (cached `t="str"`),
`=[1]Defines!NR_To_A1` → "Test A1" (name `refersTo="='Defines'!$A$1" sheetId="1"`) and
`=[1]!NR_Global_B2` → 142 (`refersTo="='Defines'!$B$2"`); a second link reads its own part
(`=[2]Uses!$C$1` → "Hello!"). `[0]` is the workbook itself: `=[0]!NR_Global_B2` is the defined name
`NR_Global_B2` → 142, and `=IF(Defines!B2 = 42, [0]!NR_Global_B2, -1)` → -1.
Evidence: poi/ref-56737.xlsx Refs!E3, G3, I3; poi/ref2-56737.xlsx Refs!E3, G3, I3, E5, G5, I5;
poi/56737.xlsx Uses!C5, D3, D5 (Excel 2010-written, Excel-cached values).
An add-in function called through an external name, `=[1]!HsGetValue(…)` (poi/HsGetVal.xlsx
Sheet1!B5, C5 cached "#Missing") and `=[1]!LUCANET(…)` (poi/56502.xlsx Tabelle1!A4 cached
"pls. update"), keeps its result only in the cell: the externalLink part caches nothing for it, so
those values came from the add-in. What Excel gives on recalculation without the add-in is
unprobed; the engine gives `#NAME?` (an unknown function).
Engine: `rustsheet-xlsx` keeps each link's cache in `Workbook::external_books` (a link that is not
to a workbook, or whose part is missing or unreadable, is an empty book that keeps its index); the
lexer reads `[n]!Name` and `[n]Sheet!Name` as one identifier, which the parser keeps as a defined
name with its prefix, and `external_links.rs` reads the cache. A cell the cache does not hold reads
as empty: `=[1]Uses!$B$3` is 0, `SUM` and `COUNTA` of `[1]Uses!$E$1:$F$2` are 0
(fixtures/external-link-cached-ranges Refs!B24, B55, B77). Unprobed, so UNVERIFIED: a book, sheet or name the cache does not hold and a book written by file name (`[other.xlsx]`) are `#BLOCKED!` as an
uncached reference is; a sheet with `refreshError="1"` is read like any other; `[0]!Missing` is
`#NAME?` and `[0]Nope!Name` (no such sheet) `#REF!`, as the plain name and a missing sheet are; only
evaluation strips the `[0]` prefix, so ISREF and a rename of the name do not see through it
(T-1459).
Cached ranges (T-1460; fixtures/external-link-cached-ranges, excel-graph, cache `Uses` A1 "Hello!",
B1 10, C1 `#N/A`, A2 5, B2 the text "7", C2 1, A3 TRUE, and a workbook name `NR_Range` =
`'Uses'!$A$1:$B$3`): a range of a cached sheet, `[1]Uses!$A$1:$B$3`, and a workbook-scoped name of a
range, `[1]!NR_Range`, are references whose cells hold the cached values, read as a range of this
workbook is read:
- aggregates skip text, numeric text, booleans and blanks: SUM 15, COUNT 2, AVERAGE 7.5, MAX 10,
  MIN 5, PRODUCT 50, MEDIAN 7.5, LARGE 10, SUMPRODUCT 15, `SUM(…,1)` 16, a union 15, `SUM(INDEX(…,0,2))`
  10; a one-cell reference too, `SUM([1]Uses!$A$1)` over text, `SUM([1]Uses!$A$3)` over TRUE and
  `SUM`/`COUNT([1]Uses!$B$2)` over "7" are all 0; an error cell is the aggregate's error
  (`SUM(…!$C$1:$C$2)` `#N/A`); the *A functions coerce as in a range (AVERAGEA 3.2, MAXA 10);
  COUNTA counts the cells the cache holds (5; 7 over `$A$1:$D$10`; 3 over `$A:$A`); cells and
  whole columns past what the cache holds read empty (`SUM(…!$E$1:$F$2)` 0, `SUM(…!$B:$B)` 10);
- the reference readers see a reference: ISREF TRUE, ROWS 3, COLUMNS 2, AREAS 1, TYPE 16, RANK 2,
  SUBTOTAL(9,…) 15, `AGGREGATE(9,6,…!$A$1:$C$2)` 16 (the error skipped);
- lookups, INDEX and element-wise functions read the values: VLOOKUP, HLOOKUP, MATCH, XMATCH,
  XLOOKUP, `INDEX(…,2,2)` "7", `INDEX(…,3,2)` (blank) 0, `SUMPRODUCT(--ISTEXT(…))` 2, ISLOGICAL 1,
  ISBLANK 1, `CONCAT` "Hello!1057TRUE", `TEXTJOIN("|",FALSE,…)` "Hello!|10|5|7|TRUE|";
- the functions that need the other workbook open are `#VALUE!`: COUNTIF, SUMIF, SUMIFS (also over
  `[1]!NR_Range`) and COUNTBLANK, a one-cell `COUNTBLANK([1]Uses!$B$3)` included; OFFSET of one
  (`SUM(OFFSET([1]Uses!$A$1,0,1,2,1))`) is `#VALUE!`;
- in a legacy (non-dynamic) formula, arithmetic on the range is implicitly intersected as a range
  of this workbook is: `SUM(IFERROR(…*1,100))` and even `SUMPRODUCT(IFERROR(…*1,100))` are 100,
  `SUM(…!$A$2:$A$3+0)` `#VALUE!`;
- a sheet-scoped name of a range, `[1]Defines!NR_Local_Range`, is `#BLOCKED!` (ROWS too; ERROR.TYPE
  11), although a sheet-scoped name of one cell reads its cached value; a sheet the cache does not
  hold, `[1]Nope!$A$1:$B$2`, is `#BLOCKED!`.
A workbook whose formulas reference a sheet with `refreshError="1"` (`[1]'56737'!$A$1:$B$2`) does
not open in Excel for the web ("unsupportedWorkbook"), so what that reads is a desktop question.
Engine: a cached area carries its cache (`external_links.rs`); aggregates and RANK read its cells
as reference cells, several areas as one row of cells. Engine-only bound (not Excel behavior, which
is unobservable here): a whole column or row is cut to the extent the cache holds, as a whole
column of this workbook is cut to its used area; any other area keeps its shape and blanks. The
array bound on cached ranges (an area or a union's cells above 1,048,576) is a DEVIATION: see
arrays.md "Array results above 1,048,576 cells". Not
yet matched: TYPE (64), SUBTOTAL and AGGREGATE (`#VALUE!`, they take a reference), AVERAGEA (`#N/IMPL!`, no array support) and the legacy implicit
intersection of an external range (T-1526).
`Sheet!Name` with no book prefix (poi/56737.xlsx Uses!D2 `=Defines!NR_To_A1`, Excel-cached
"Test A1") is the name scoped to that sheet of this workbook; `[0]!Name` and `[0]Sheet!Name` resolve
the same way at parse time, so ISREF and a rename see them. A sheet-qualified workbook-scoped name
(`Sheet!GlobalName`), an unknown `Sheet!Nope` (engine: `#NAME?`) and a missing sheet are UNVERIFIED (T-1459).
Status: VERIFIED (Excel-cached cells above); the cases listed as unprobed are UNVERIFIED
Tasks: T-1382, T-1459, T-1460

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
VLOOKUP's and HLOOKUP's picked cell is a reference to an aggregate only when the row/column index argument is itself a
cell reference: `COUNT(HLOOKUP(42,F5:G6,G5))` (G5 holds 2, picking FALSE) is 0, `SUM(VLOOKUP(2,I5:J7,A2))` picking text "5" is 0,
`MAX(VLOOKUP(3,I5:J7,A2))` picking text "x" is 0 (probes/lookup-pick-aggregates P1-P3, P5, P6, Q1, Q2, Q4, Q6; a parenthesized
reference counts as one). With a literal or computed index (`INT(Data!A10)`) the pick is a plain value: `COUNT(VLOOKUP(1,A1:B3,2,FALSE))`
picking FALSE is 1, `SUM` of a picked "5" is 5, `MAX` of a picked "x" is `#VALUE!` (D1-D3, D13, E1-E14, P4, Q3). That holds for exact and
approximate match, a fractional literal index, a table on another sheet, and array tables (D4, D5, D8, D9); a reference
lookup value changes nothing (P8). `ISREF` of the lookup result is FALSE either way (D12, E6, E12, P7), so the reference-ness is
visible only to aggregates. This explains gen-m2/60a543744fc4a8f7 A200 (index `Data!C9` is a cell, the picked FALSE is skipped: -2;
replicated in L1-L7). A bare `=HLOOKUP(..)` is the picked value (N1-N7).
`INDEX(A1:A2,3) A1:A2` is `#REF!`, and so is `SUM` of it (D10, D11); `INDEX(A1:A3,2) A1:A3` is the picked cell (E16, E17).
Status: VERIFIED (excel-graph 2026-10-09, T-0292). The engine matches: `hlookup_impl`/`vlookup_impl` give the pick as a reference only when `lookup_index_arg` returned a reference (T-0295). Index shapes (probes/lookup-pick-index-shapes, excel-graph 2026-10-11, T-0299): every reference-returning index keeps the pick a reference: `COUNT(HLOOKUP(42,F5:G6,<idx>))` picking FALSE is 0 for `INDEX(G5:G5,1)`, `INDEX(G5:G6,1)`, `IF(TRUE,G5)`, `IF(FALSE,1,G5)`, `CHOOSE(1,G5)`, `OFFSET(G5,0,0)`, `OFFSET(G5,0,0,1,1)`, and a defined name `=Sheet1!$G$5`, and the same for VLOOKUP (A2-A6, A8-A10, B1-B5), while a literal index 2 gives 1 (A1, B7); `ISREF` of each such index is TRUE (C1-C5). The engine matches, the `IF` shapes included (`lookup_index_arg` reads an `IF` index as the reference it picks, T-1377). A multi-cell range index (`H1:H2`, A7/B6) is 0 whether it is implicitly intersected, lifted or a reference pick, so those cells do not decide it. Multi-cell index (probes/lookup-index-multicell-if, excel-graph 2026-10-13, T-1426, H1=1, H2=2): Excel does not implicitly intersect it as a reference (probed in rows 1 and 2 only, where H1:H2 sits): `COUNT(HLOOKUP(42,F5:G6,H1:H2))` is 1 in both row 1 and row 2 (an intersection that keeps the pick a reference would give 1 then 0; an intersection to a value is not told apart by these cells, the spills and ROWS are what show lifting), as are the `VLOOKUP` form and `IF(TRUE,H1:H2)` as the index; the plain formula lifts and spills one pick per index cell (`HLOOKUP(42,F5:G6,H1:H2)` J1 2, J2 FALSE; `IF(TRUE,H1:H2)` the same; `VLOOKUP(9,F5:G7,IF(TRUE,H1:H2))` L1 5, L2 FALSE; `ROWS(HLOOKUP(42,F5:G6,IF(TRUE,H1:H2)))` 2, O1), and `ISREF` of the lifted pick is FALSE. `IF(TRUE,(G5,G6))` as the index is `#REF!` (not `#VALUE!`; M1), so `COUNT` of it is 0. The engine matches (`lookup_index_arg` lifts a multi-cell range index as an array of its values and gives `#REF!` for an `IF` union, T-1485). Lifted lookups (probes/lookup-index-multicell-lifted, excel-graph 2026-10-13, T-1485): the same index with an array `lookup_value` (`VLOOKUP({9,2},F5:G7,H1:H2)` A1 5, B1 `#N/A`), an array `range_lookup` (`HLOOKUP(42,F5:G6,H1:H2,{TRUE,FALSE})` D1 2, E1 `#N/A`) or a range `lookup_value` (`VLOOKUP(F5:F6,F5:G7,H1:H2)` J1 3.14159, J2 5, J10 and J11 the same) gives identical results in row 1 and in row 10, so it is never intersected there either; the engine matches (a lifted lookup reads the top-left index cell only; whether Excel reads another cell for other index values is unprobed). The `IF` union index is validated only after a match: `HLOOKUP(99,F5:G6,IF(TRUE,(G5,G6)),FALSE)` is `#N/A` (M1), as is `HLOOKUP(99,F5:G6,H1:H2,FALSE)` (N1). Unprobed: a one-row or 2-D range index, and a multi-cell index from `INDEX(..,0)`, `OFFSET`, `CHOOSE` or a defined name (the engine lifts them like `H1:H2`); a range index holding text or an error gives one `#REF!` for the whole call (probes/lookup-index-shapes J10).
ISREF sees the same references: `ISREF(INDEX(A1:A2,2))`, `ISREF(IF(TRUE,A1))` and `ISREF(CHOOSE(1,A1))` are TRUE,
while `ISREF(INDEX(A1:A2,3))` (an error), `ISREF(IF(TRUE,1))` and `ISREF(CHOOSE(1,5))` are FALSE (probes/isref-function-results,
excel-graph 2026-10-05; ironcalc ISREF_ISFORMULA!F9 `ISREF(INDEX(A1,1,2,2))` = FALSE). The engine matches all six (T-0235).
XLOOKUP, TAKE, DROP and TRIMRANGE are references to ISREF when they pick cells of a range:
`ISREF(XLOOKUP(2,A1:A3,B1:B3))`, `ISREF(TAKE(A1:B3,1))`, `ISREF(DROP(A1:B3,1))` and `ISREF(TRIMRANGE(A1:B3))` are
TRUE, and so is `ISREF(XLOOKUP(2,{1;2;3},B1:B3))` (an array lookup vector, a range return); over an array
(`ISREF(TAKE({1,2;3,4},1))`) or when they fail (`XLOOKUP` not found, `DROP(A1:B3,5)`, `TAKE(A1:B3,0)`) they are FALSE
(probes/isref-reference-lookups, excel-graph 2026-10-07; T-0165; the array lookup vector is T-1126).
A defined name that stands for a cell or range is a reference to ISREF, a single cell included
(`ISREF(TextCell)` and `ISREF(Cells)` TRUE; a LAMBDA name, a constant name `=7` and an undefined name are FALSE), and FILTER's `if_empty` range is its
values, never a reference (`FILTER(A1:A1,B1:B1,D1:D2)` spills D1:D2, `ISREF(FILTER(..))` FALSE, `SUM(..)` = 30).
Evidence: probe isref-names-filter-if-empty (oracle: excel-graph, 2026-10-06, T-0940; re-confirmed 2026-10-09 with the LAMBDA name body stored as `_xlpm.x`, T-1209), matching upstream IronCalc ISREF.xlsx (ironcalc 9add6b67; Excel 16.03 cached values; to be added to calc_tests by T-0965) Sheet1!A14-A18, D11, D12, D14, D17, D20. ISREF of a LET/LAMBDA variable
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
Probe T-1378 (probes/subtotal-reference-operands-f, excel-graph, 2026-10-11): TAKE and DROP pick cells (SUBTOTAL reads them, AREAS 1, ISREF TRUE) or values (SUBTOTAL and AREAS spill `#VALUE!` per element, ISREF FALSE) according to the arm the IF, CHOOSE, SWITCH or IFS actually selects: `TAKE(IF(TRUE,A1:A4,SEQUENCE(4)),2)`, `TAKE(CHOOSE(1,A1:A4,SEQUENCE(4)),2)` and `TAKE(IFS(TRUE,A1:A4,FALSE,SEQUENCE(4)),2)` are cells, `CHOOSE(2,...)` and `IF(FALSE,A1:A4,SEQUENCE(4))` are values. LET variable names and LAMBDA parameters are case-insensitive (`LET(x,A1:A4,X)` reads `x`; probes/let-lambda-name-case: `LAMBDA(x,X+1)(2)` 3, `LAMBDA(Ab,aB*2)(4)` 8, `LET(f,LAMBDA(x,x+1),F(2))` 3, and a parameter of another case shadows the LET name, `LET(x,1,LAMBDA(X,x*10)(2))` 20, `LET(x,1,LET(X,5,x))` 5); an inner LET rebinding a name shadows the outer one. A LET or LAMBDA that binds one name twice is refused in any case: Excel for the web refuses (createSession 501 `FileCorruptTryRepair`) a single-cell workbook holding `LET(x,1,x,2,x)`, `LET(x,1,X,2,x)`, `LET(é,1,É,2,é)`, `LAMBDA(x,x,x)(1,2)`, `LAMBDA(x,X,x+X)(1,2)`, `LAMBDA(ä,Ä,ä+Ä)(1,2)` or `LAMBDA(x,[X],x)(1)` (an optional parameter too) (`rustsheet-oracle recalc -b excel-graph`, 2026-10-12, T-1387; refused inputs cannot be committed), while `LET(x,1,LET(X,2,x))` loads and gives 2. The engine gives `#ERROR!` for each (a parse error for LAMBDA, the LET duplicate check). Names may start with a non-ASCII letter, and one-to-one letter pairs fold case (probes/let-name-non-ascii: `LET(é,1,É+1)` 2, `LET(Ä,2,ä*10)` 20, `LET(Ärger,5,äRGER+1)` 6, `LAMBDA(ß,ß*2)(3)` 6, `LET(é,1,LAMBDA(É,é*10)(2))` 20); a fold that changes length does not: `LET(ß,1,ẞ+1)` is 2 but `LET(ß,1,SS+1)` is `#NAME?` (probes/let-name-sharp-s, which stores the use as `_xlpm.SS`: spec2xlsx decides which names are bound with Python `str.lower()`, so a probe of a fold where Python and Excel may differ must write the `_xlpm.` prefix itself). Other scripts (CJK) and letters such as Turkish `İ` are unprobed (T-1431). The earlier refusal of `LET(é,1,É+1)` was spec2xlsx leaving a non-ASCII name without its `_xlpm.` prefix; before T-1387 the engine also rejected a name whose first letter is non-ASCII, unless it uppercased to ASCII (`ß` → `SS`). The same identifier check (`is_valid_a1_identifier`) validates defined names, so a defined name may now start with a non-ASCII letter too; that is unprobed for defined names.
Tasks: T-0216, T-0235, T-0940, T-1003, T-1378, T-1387

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
parentheses, `(A1 @A1) B1`, since the bare text would re-parse as `A1 @(A1 B1)` (engine choice; Excel's text for it is not probed). The same text is printed when a formula is moved or pasted (`to_string_moved`); since T-1342 that includes the parentheses around a cut of a union, `=SUM(((A1,B1) B1))` moves as `=SUM(((J10,K10) K10))`, which the moved text used to drop (UNVERIFIED for Excel's cut/paste text, which is not probed; it matches FORMULATEXT, F6 below). As intersection operands, XLOOKUP, TAKE,
DROP and TRIMRANGE over a single-sheet range are evaluated as the reference to the cells they pick
(`Model::evaluate_reference_form`, T-0165); everywhere else the engine still evaluates them to values,
and over an array they are not references (`=A1:A3 TAKE({1,2},1)` is `#VALUE!`, UNVERIFIED: Excel's
parser may refuse it). Counts from the end pick from the range's end (`=A1:A3 TAKE(A1:B3,-1)` is 3), a
call's own error passes through (`DROP(A1:B3,5)` and `TAKE(A1:B3,0)` are `#CALC!`, a not-found XLOOKUP `#N/A`),
and a not-found XLOOKUP with a range `if_not_found` is that reference (`=C1:C3 XLOOKUP(9,A1:A3,B1:B3,C1)` is
C1, and ISREF of it TRUE) (probes/intersection-lookup-operand-errors, excel-graph 2026-10-07).
- FORMULATEXT of an `@` operand of `:` or `@` keeps the written parentheses: `=A1:(@B1)` stays `=A1:(@B1)` (spills `1, 2` across H10:I10), `=(@(A1 B1)):C3` stays `=(@(A1 B1)):C3` and `=@(@(A1 B1))` stays `=@(@(A1 B1))` (both `#NULL!`). The engine drops the inner parentheses on the last two (probes/at-operand-formulatext, T-1104; T-1116).
- `=ANCHORARRAY(D1)` prints `=D1#` and `=ANCHORARRAY(INDEX(D1:D2,1))` prints `=INDEX(D1:D2,1)#`, `ANCHORARRAY(D1)` and `D1#` are the same stored file text (`_xlfn.ANCHORARRAY(D1)`), so this is how Excel prints that stored form; how a typed `ANCHORARRAY(D1)` prints is not probed. Engine requirement for T-1109: xlsx import must not print the automatic `@` it adds inside `#`. Stored as `_xlfn.ANCHORARRAY((@A1:B2))`, `=(@A1:B2)#` loads, prints as `=(@A1:B2)#` and is `#VALUE!` from D20 (no intersecting cell) (probes/anchorarray-at-operand, T-1111; the engine matches since T-1115). UNVERIFIED: the engine passes any `@` error through `#` (`=(@(A1 B1))#` gives `#NULL!`), and gives the left error when both operands of `:` are errors; only the `#VALUE!` and right-operand cases are probed. `=ANCHORARRAY(1)` is refused: Excel for the web will not open a workbook holding it (`FileCorruptTryRepair`, T-1111); the engine reads it as a syntax error since T-1117. `=@D1#` prints as `=@D1#` and evaluates to `#VALUE!` (probes/at-operand-formulatext, T-1104). Value operands of `:` and `#` that T-1111 did not probe (T-1216): see the entry "FORMULATEXT keeps the parentheses of an `@` operand" below.
Tasks: T-0138, T-0148, T-0149, T-0042, T-0161, T-0165, T-1094, T-1098, T-1103, T-1104, T-1109, T-1111, T-1115, T-1117, T-1216

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
A LET name called like a function is `#REF!` when bound to a reference, whatever the reference:
`=LET(f,A1,f(2))`, `=LET(f,A1:A3,f(2))`, `=LET(f,C9,f(2))` (C9 empty), `=LET(f,A1,f())`,
`=LET(f,INDEX(A1:A3,2),f(2))` and `=LET(f,IF(TRUE,A1),f(2))`, an ordinary error again
(`IFERROR` catches it, `ERROR.TYPE` is 4). So LET binds what INDEX, CHOOSE and IF select as a
reference (`=LET(x,INDEX(A1:A3,2),ISREF(x))` and `=LET(x,CHOOSE(2,A1,A2),ISREF(x))` are TRUE,
`=LET(x,INDEX(A1:A3,2),ROW(x))` is 2, and COUNT through an INDEX-bound text cell is 0), but VLOOKUP
and HLOOKUP bind their value: `=LET(x,VLOOKUP(2,A1:B3,2,FALSE),ISREF(x))` is FALSE, COUNT through it
counts the text "5" (1), calling it gives 0 as for text, and a LAMBDA argument behaves the same. Bound to a
value, the call gives odd results, without the arguments mattering: a number or boolean is `#NAME?`
(`=LET(f,1,f(2))`, `=LET(f,A1+0,f(2))`, `=LET(f,TRUE,f(2))`), text is the number 0 (`=LET(f,"a",f(2))`,
`"abc"`, `"1"`, `f()` too; `TYPE` of it is 1 and `+5` gives 5), an error is itself
(`=LET(f,1/0,f(2))` is `#DIV/0!`), and an array is called element by element, keeping its shape:
`=LET(f,{1,2},f(2))` spills `#NAME?`,`#NAME?` (`#SPILL!` when blocked), `=LET(f,{1;"a"},f(2))` spills
`#NAME?`;0 and `ROWS(LET(f,{1,2;3,4},f(2)))` is 2. Bound to a LAMBDA it is called
(`=LET(f,LAMBDA(x,x+1),f(2))` is 3). The case builder writes the called name as `_xlpm.f(2)`, and
Excel reads that back as `f(2)` (FORMULATEXT gives `=LET(f,A1,f(2))`), so it is Excel's own form.
Probed in T-1224 (probes/let-name-call-forms, excel-graph, 2026-10-09): a name bound to a union is `#REF!` too (`=LET(f,(A1,A2),f(2))`,
`=LET(f,(A1,A2),f())`, `=LAMBDA(f,f(2))((A1,A2))`), as is one bound to `XLOOKUP(2,A1:A3,B1:B3)`, `TAKE(A1:A3,1)`, `DROP(A1:A3,1)`,
`OFFSET(A1,0,0)` or `TRIMRANGE(A1:A3)`; `bind_argument` (functions/logical/let.rs) binds XLOOKUP, TAKE, DROP and TRIMRANGE as the reference they pick (T-1232); erroring arguments do not matter (`=LET(f,A1,f(1/0))` is `#REF!`, `=LET(f,"a",f(1/0))` is 0, `=LET(f,1,f(NA()))` is `#NAME?`, `=LET(f,{1,2},f(1/0))` spills `#NAME?`,`#NAME?`
(H5:I5), `=LET(f,LAMBDA(x,x+1),f(1/0))` is `#DIV/0!`). `=LAMBDA(f,f(2))()` is a call with too few arguments, not an empty element: it is
an error whose type is `#VALUE!` (`ERROR.TYPE` 3, `IFERROR` catches it, H15:H17); the bare cell returns an error type the oracle client cannot
decode (probes/let-name-call-forms description). UNVERIFIED: the engine's mapping of an empty element to `#NAME?` (`called_value` in
lambda.rs). A workbook holding `=LAMBDA([f],f(2))()` (an omitted optional parameter) is refused by Excel for the web (`FileCorruptTryRepair`,
`unsupportedWorkbook`, one scratch workbook, T-1224), so that form cannot be a case. Ours: the union forms (H1, H2, H13) are `#REF!` as in Excel since T-1231; the engine gives `#NAME?`/`#SPILL!` for the
XLOOKUP/TAKE/DROP/TRIMRANGE forms (T-1232); the rest match.
Uncalled (probes/let-name-uncalled-picked-refs, excel-graph, 2026-10-09, T-1252): a LET name or LAMBDA parameter bound to
`XLOOKUP(2,A1:A3,B1:B3)`, `TAKE(A1:A3,2)`, `DROP(A1:A3,1)` or `TRIMRANGE(A1:A3)` is the reference it picks: `ISREF`, `CELL("address",f)`, `ROWS`, `SUM` and
`COUNTIF` through the name all read the cells, as they do for the written call; the engine matches on all 40 cells. Not a quirk of the binding:
`SUM(XLOOKUP({1;2},A1:A3,B1:B3))` is 30 in Excel (an array `lookup_value` looks up each element), and the engine gives `#N/A` (H21, a known failure).
A LAMBDA parameter called like a function follows the same rule: `=LAMBDA(f,f(2))(1)` and `(TRUE)` are
`#NAME?`, `("a")` is 0, `(1/0)` is `#DIV/0!`, `(A1)` is `#REF!` and `(LAMBDA(x,x*3))` is 6.
Excel for the web refuses a workbook holding `=LET(f,"",f(2))` (`createSession` fails, three scratch
workbooks, T-1092), so empty text cannot be a case.
Evidence: probes/paren-call-operands H1:H10, probes/paren-call-empty-args, probes/paren-call-lambda-cell,
probes/paren-call-if-lambda, probes/paren-call-intersection, probes/paren-call-function-ref and
probes/intersection-operands H13 (oracle: excel-graph, 2026-10-05 and 2026-10-07, T-0138, T-0149);
called LET names: probes/let-name-call, let-name-call-refs, let-name-call-arrays, let-name-call-bool and
let-name-call-text-*, let-name-call-forms (T-1224), let-bind-reference-forms, called LAMBDA parameters: probes/lambda-param-call (oracle: excel-graph,
2026-10-09, T-1092).
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
functions/logical/lambda.rs (`Model::call_callee`, `Model::call_variable` for a called LET name
or LAMBDA parameter);
functions/logical/let.rs (`Model::bind_argument`, which binds in reference context); model.rs (the
callee is evaluated with `evaluate_aggregate_arg`, so a cell and a selecting INDEX or IF stay references)
Tasks: T-0149, T-1092, T-1224, T-1231, T-1232, T-1252

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
  `SUM(A1,B2)` is two arguments, `SUM((A1,B2))` is one union argument; outside every parenthesis
  and argument list the comma is the bare union, see "Bare union" below.
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
  `=((A1,B2))` is `=((A1,B2))` and of `=(((A1,B2),C1))` is `=(((A1,B2),C1))` (the engine keeps them since T-1024; since T-1342 they are the union's `Group` node).
- Grouping and cuts (T-1140, probes/union-cut-precedence, oracle: excel-graph, 2026-10-08; VERIFIED (web)): the union is the
  loosest operator inside its parenthesis and the intersection binds tighter, so `(A1,B1 C1)` is `(A1,(B1 C1))`: `=SUM((A1,B1 C1))`
  is `#NULL!` (F1; either grouping gives it, so F1 does not settle the grouping), while `=SUM((A1:B1,B1:C1 B1))` is 35 (F2, which does settle it: `A1:B1` plus `B1`, 11+12+12; the other grouping gives 24) and
  `=SUM(((A1,B1) B1))` is 12 (F3). FORMULATEXT of the three prints the typed text unchanged (F4:F6; the engine keeps the parentheses around a cut of a parenthesized union since T-1176; since T-1342 they are a `Group` node around the cut; the printed text keeps exactly the written levels, never more). A union cut by an intersection runs
  the cut on each pair of areas and drops pairs with no shared cell: `=SUM((A1:A12,D1:D12) A4:D4)` is 85 (F8, A4 plus D4). Further cuts
  bring several areas down to one: `=(A1:A12,D1:D12) A4:D4 D4` is D4 = 44 (F7) and `ROWS` of it is 1 (F9). A cut leaving one of two
  areas gives that area: `=SUM((A1:A3,C1:C3) A2:B2)` is 21 (F14) and `=(A1:A3,C1:C3) A2:B2` is 21 as a cell value (F15). Empty cuts:
  `=SUM((A1,B2) C3)` is `#NULL!` (F10), `ISREF` of it is FALSE (F11), `IFERROR(SUM(...),"e")` gives `"e"` (F12) and `ERROR.TYPE` is 1 (F13).
  Not probed (engine behavior): a union on the right of a cut, cuts by three or more areas of one union, and unions of more than two areas.
- The known gaps: a union reaching a
  function gets `#VALUE!` (`SUM((A1,B2))` before T-1062), so IFERROR silently
  takes its fallback for one, and so does a union returned by IF, CHOOSE or LET. docs/specs/SPEC-0004-multi-area-references.md
  (T-0133) designs multi-area arguments, with rollout T-1061 to T-1065. `add_implicit_intersection` (static_analysis.rs) leaves
  `OpUnionKind` unwrapped, and SPEC-0004 keeps it so: a union is already `#VALUE!` where a scalar is expected (planned, probe rows 15 and 18).
- Multi-area defined names (T-1065): a name whose body is a union stands for its areas wherever a reference
  is accepted (`SUM(mu)` 6, `COUNT(mu)` 2, `mu A1` 1, AREAS, INDEX's area_num, AGGREGATE's array form), through
  another name or a LET variable too; `=mu` stays `#VALUE!` (probes/union-defined-name H1:H5). The engine
  evaluates the body at each use (`Model::defined_name_reference`), under the name depth and budget limits, so a
  name that reaches itself through a union (`loop` = `Sheet1!$A$1,loop`) is `#NAME?` like any self-reference.
  A cross-sheet body is `#VALUE!` in SUM (probes/multiarea-xname). Probed (probes/union-defined-name-uses, excel-graph
  2026-10-09): `AREAS(mu)` 2, `INDEX(mu,1,1,2)` 5, `AGGREGATE(14,0,mu,1)` 5 and `(15,0,mu,1)` 1, a name of a name (`alias` =
  `mu`: `SUM` 6, `alias A1` 1, `ISREF` TRUE, `AREAS` 2), a CHOOSE body returning a union (`SUM(ch)` 6, `COUNT(ch)` 2),
  `LET(x,mu,SUM(x))` 6, `SUM(FREQUENCY(mu,2))` 2 and `COUNTBLANK(mu)` `#VALUE!`. A name whose body is not a union keeps
  its path: `FREQUENCY(arr,2)` with `arr` = `{1,2,3}` is {2;1}. So is an array behind another name (`aa` = `=arr`), a LET variable or IF (H19:H22). `COUNTBLANK(bad)` with `bad` = `=1+"a"` is `#VALUE!` in
  Excel (engine 0, T-1199). The self-referential case is an engine extension of probes/defined-name-formula-bodies H11
  bounded per decision 0014, not probed. A sheet-local `Print_Area` works the same (probes/multiarea-printarea, T-1193; storage in xlsx-storage.md "Built-in defined names").
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
repeat that per three characters of formula, and a LET-bound union per argument
(`LET(u,...,SUM(IF(1,u u A1),IF(1,u u A1),...))`), so one formula evaluation may compare at most
1,048,576 pairs in all (`MAX_AREA_PAIRS`), with `#VALUE!` past it, wherever in the formula the
pairs are compared: every argument, IF/CHOOSE/LET operand and reference-returning function draws on
the same budget. Each cell's formula run (a cell it reads has its own), each conditional-format
evaluation and each `evaluate_formula` call gets a fresh one (T-1162). In a value context a union
keeps only its area count, not the areas. Excel's limits, if any, are not known, and neither is
its result past them; it would differ where later cuts bring the areas back down to one, or where
a later operand's error would come first.
The area cap bounds the length of an area list, not the work of walking it: `MAX_AREAS` copies of
`A:A` fit in a short formula, a LET-bound union can be passed again and again
(`LET(x,(A:A,A:A),SUM(x,x,...))`), and a LAMBDA can walk one per row of a MAP. So the areas of a
union (an aggregate argument of several areas: SUM, COUNT, COUNTA, SUBTOTAL, AGGREGATE, LARGE, ...)
are walked over the cells that exist only, not every position of the used area: every function
that takes a union skips blank cells, and the cells are met in the same row-major order, so the
first error cell is the same. The walk pays as it goes from a budget of 16,777,216 steps
(`MAX_AREA_WALK`, 2^24) per formula evaluation: one step per area walked (so a repeated walk of
an empty union is paid for too), one per cell handed to the function, and, for an area over more
than 1,024 of the sheet's rows, one per search of the sheet's column index (built by the first
such walk, then kept in step by every write): one per column of the area that holds cells (at
least one) to find them, then, when there are at most 16 such columns, one per read of up to 64
of a column's rows, merged in row-major order with a row costing at least a step per four columns
still being merged (T-1502), and one per column again after a cell the walk hands on writes to
the sheet (a spill), since the reads are then out of date. Such a walk keeps the columns it found
at its start; a spill into another column of the area is caught by the area's read-empty record. An area with cells in more columns is walked by rows instead: one
step per row found (at least one, even with no cell in the area's columns), one per unallocated
chunk of 1,024 rows passed over, and one per 16 row slots tested in allocated chunks
(`SLOTS_PER_WALK_STEP`); so is a shorter area, whose row walk is a chunk of slots at most. Past it the argument is `#VALUE!`. Every multi-area argument, IF/CHOOSE/LET
operand and LAMBDA call of the formula draws on the same budget, like the pair budget; a single
range argument is not counted and is walked as before (every position of its used area). Over
sparse data a union then costs what the data costs: `=SUM(BYROW(A1:A1000,LAMBDA(r,SUM((B:B,C:C)))))`
with three cells in 10,000 used rows is about 50 steps per walk, and `MAX_AREAS` copies of `A:A`
over a cell in row 1,000 fit. Over filled cells a step is a cell: sixteen arguments of a 16-column
union over 65,536 filled rows, or a MAP of 300 walks of `(A:A,A:A)` over them, are `#VALUE!`.
The slot rate is measured (T-1497, release build): a step handed to a function costs about 15 ns
and a slot test about 1.4 ns. With it the budget's worst shapes, a 16-column union of `A:A` passed
again and again over 65,536 filled rows, over one row in every chunk of the sheet, over 65,536 rows
with a cell in column B only, or over a cell in the last row only, each stop at `#VALUE!` within
0.31 s; the old dense budget's worst was 1.09 s (one row per chunk).
Excel computes formulas well past this budget's old dense form (probes/union-walk-repeated-in-lambda,
excel-graph, 2026-10-13, T-1462): `=SUM(BYROW(A1:A1000,LAMBDA(r,SUM((B:B,C:C)))))` on a sheet with
10,000 used rows is 8000 (2*10^7 positions), and so are the MAP, COUNTA and SUBTOTAL(9,..) forms;
`MAP(A1:A10000,...)` (2*10^8 positions) is 80000; the bounded union `(B1:B10000,C1:C10000)` too.
With data in row 1,048,576 `=SUM((B:B,C:C),...)` with nine union arguments (9*2^21 positions) is
27. The engine now agrees on all of them (T-1497). MAP over a union
(`=SUM(MAP((B1:B3,C1:C3),LAMBDA(x,x)))`, `ROWS(...)`) is `#VALUE!` in both. The oracle gives no
timings, so whether Excel walks only data is not known; a union over filled cells past the budget
remains a DEVIATION. Rows filled in other columns cost a narrow area nothing, since its rows come
from the column index (T-1502): `MAP(A1:A10000,LAMBDA(r,SUM((B:B,C:C))))` is 80000 as in Excel
when another column fills 9,000 rows. An area with cells in more than 16 columns is still walked
by rows, paying for rows where its columns are blank, so a wide area over a sheet whose rows other
columns fill can reach the budget before Excel would.
A union area is recorded as read-empty as one rectangle per formula root, once however often it is
walked, so repeated walks do not grow the record that each spill is checked against.
Excel's actual limit, if any, is not known; the probe only shows it is above 2*10^8 walked cells.
The LET variables and LAMBDA parameters in scope may hold at most 262,144 areas together (`MAX_HELD_AREAS`,
four `MAX_AREAS`): each binding may hold a fresh union (`LET(x_1,(A1,A1),x_2,(x_1,x_1),...)`), so memory would
otherwise grow with formula length. A binding that would pass the total is `#VALUE!`.
A defined name's union body is capped like a written union, at each use: more than `MAX_AREAS` areas is
`#VALUE!` where the name is used, never a load failure, and the name still loads. No per-workbook cap is
needed (SPEC-0004 planned one for stored area lists): names keep only their parsed body, never an area list,
so memory is bounded by the workbook's formula text and each use by the caps above.
Status: DEVIATION (decision 0014)
Implemented in: reference_operators.rs (`MAX_AREAS`, `Model::union_areas`,
`Model::count_union_areas`, `Model::intersect_areas`); area_budget.rs (`MAX_AREA_PAIRS`,
`MAX_AREA_WALK`, `SLOTS_PER_WALK_STEP`); functions/util.rs (`Model::for_each_position_in_area`, `Model::for_each_indexed_position`); sheet_data.rs (`SheetData::rows_in_column_between`); reference_operators.rs (`MAX_HELD_AREAS`, `VariableStack`)
Tasks: T-0134, T-1062, T-1064, T-1065, T-1162, T-1462, T-1497, T-1502

## Seen-range cap
Rule: one formula evaluation may put at most 65,536 distinct rectangles on record as read empty in a
pass (`MAX_SEEN_RANGES`); a formula that reads more is `#VALUE!` ("Too many distinct ranges read").
A function walking a whole column or row (and a lookup down a column, or a SUMIFS-shaped criteria
range) visits only the used area and records the rest as one rectangle read empty, and every area of
a union is recorded whole, whatever its size; which every spill commit of the pass is checked
against (evaluation.rs module doc). A rectangle already on record for the formula's root is free, so
repeating a walk costs nothing (T-1497: `MAX_AREAS` copies of `A:A` walked by a MAP record one), but
each new reference makes a new one, and nothing else bounds them: `=SUM(MAKEARRAY(1024,1024,
LAMBDA(r,c,SUM(OFFSET(1:1,r-1,0,c)))))` recorded 1,048,576 (32 bytes each in the list and again in
the set, about 70 MB; every spill commit meets each once, at a cost of its cells plus the
rectangles, T-1512); it is now `#VALUE!` once 65,536 are on
record (T-1463, release build: 1.5 s to reach the cap). The error is the formula's, set when its run
ends, not the argument's: a rectangle that goes unrecorded leaves a read the pass does not know of,
so no value computed from it, `IFERROR`'s fallback included, may stand. The budget is per formula
run like the area budgets ("Union area cap"): a cell the formula reads has its own, and its
rectangles are paid by the run that records them first, so at the cap the result can depend on which
formula of a root read a rectangle first. Excel's limit, if any, is not known; a formula reading
more than 65,536 distinct such rectangles, whole rows
(`MAP(SEQUENCE(65537),LAMBDA(r,SUM(OFFSET(1:1,r,0))))`) or bounded union areas
(`MAP(SEQUENCE(70000),LAMBDA(r,SUM((OFFSET(A1,r,0),B1))))`), is presumably a value there. A
rectangle past the cap is dropped, not widened to a covering one: a spill into it does not restart
the pass, so where that spill would let a re-run stay under the cap, the result depends on the
order; a covering rectangle would instead risk a false circular reference for the root's other
anchors.
Status: DEVIATION (decision 0014)
Implemented in: area_budget.rs (`MAX_SEEN_RANGES`, `AreaBudget::spend_seen_range`,
`Model::with_area_budget`); seen_ranges.rs (`SeenRanges::push`); evaluation.rs
(`Model::record_seen_empty_range`)
Tasks: T-1463

## Multi-area references in functions
Rule: which functions take a union, and what they give (Sheet1 A1=1, B2=5, C3=7, D4=9; evidence
probes/multiarea-*, oracle: excel-graph, 2026-10-07, T-1061):
- Accept it, areas counted as written (overlap counted twice: `SUM((A1:B2,B2))` = 11): SUM, COUNT,
  COUNTA, AVERAGE, MIN, MAX, PRODUCT (`(A1,B2)` gives 6, 2, 2, 3, 1, 5, 5), LARGE, SMALL, RANK,
  MEDIAN, STDEV.S (`(A1,B2,C3)`: 5, 5, 2 for RANK(5,..), 5, 3.0550504633), FREQUENCY (`SUM(FREQUENCY((A1,B2),3))` = 2),
  SUBTOTAL 9 and 109 (6), INDEX with `area_num` (`INDEX((A1:B2,C3:D4),1,1,2)` = 7, area 3 `#REF!`, area 0
  `#VALUE!`, `area_num` omitted = 1; `SUM(INDEX(..,0,0,2))` = 16), AREAS (2, 3 for `((A1,B2),C3)`, 2 for
  `(A1:B2,A1:B2)`), CELL("address", `(A1,B2)`) = "$A$1, $B$2".
- CELL of a union (probes/multiarea-cellfn, multiarea-cellfn-types, excel-graph 2026-10-08, T-1072):
  "address" lists the top-left cell of every area in formula order, duplicates kept, joined by `", "`
  (`(A1:A2,B2)` = "$A$1, $B$2", `(B2,A1)` = "$B$2, $A$1", `(A1,A1)` = "$A$1, $A$1", `(A1:B2,C3)` =
  "$A$1, $C$3"). "row", "col", "contents" and "type" read the first area's top-left cell (1, 1, 1, "v"). A
  union across sheets is `#VALUE!`. `COLUMN` of a union is `#REF!` like `ROW`. UNVERIFIED: `CELL("filename",
  union)` (excel-graph answers `#VALUE!` even where the plain-reference form is the comparison, so the web
  value says nothing about desktop); the engine reads the first area.
- `INDEX` `area_num` is read as a number and floored: below 1 is `#VALUE!`, past the last area (or NaN, or a
  huge value such as 1E+300) is `#REF!`; a reference of one area with `area_num` other than 1 is `#REF!`. `AREAS` of a
  3-D span is `#VALUE!`. Implemented by T-1063 (`index_impl`, `fn_areas`), cases in probes/multiarea-areas-* and
  probes/multiarea-index; unit tests in test/test_multi_area_arguments.rs. The 1E+300, "one area, area_num 2", `INDEX` of an
  array with `area_num` other than 1 (`#REF!`)
  rows are engine choices, UNVERIFIED (no probe). `AREAS` of a non-reference: see "SUBTOTAL and AREAS take references only".
- SUM, COUNT and COUNTA of a union equal the same call with the areas as separate arguments, whatever the cells hold
  (probes/multiarea-sum-types, excel-graph, 2026-10-08, T-1151). With A1:A3 = 1, "x", TRUE and B1:B3 = 2, FALSE, "5":
  `SUM((A1:A3,B1:B3))` = 3 (text and booleans in references are skipped), `COUNT` 2, `COUNTA` 6; whole columns inside a union
  behave the same (`SUM((A1:A3,B:B))`, `SUM((A:A,B:B))` = 3, `COUNT((A:A,B:B))` = 2, `COUNTA` 6). Errors: the first error in
  area order wins, `SUM((C1,D1))` = `#DIV/0!` and `SUM((D1,C1))` = `#N/A`, as for separate arguments; an error in a later
  area beats numbers in an earlier one (`SUM((A1,C1))`, `SUM((A1:A3,C1:D1))` = `#DIV/0!`). Single text or boolean cells
  in a union give 0 (`SUM((A2,B3))`, `SUM((A3,B2))`). The engine already agrees on all 24 cells.
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
  Implemented in `evaluate_union_operand` (reference_operators.rs, T-1070). VERIFIED by T-1159
  (probes/multiarea-3d-union-h1..h8, excel-graph, 2026-10-08): a multi-sheet span as a direct operand of a union
  is `#NAME?` whichever side it is on and whichever context reads it: `SUM((A1,Sheet1:Sheet2!B2))`, the bare
  `=(Sheet1:Sheet2!A1,B2)`, `SUM((Sheet1:Sheet2!A1,Sheet1:Sheet2!B2))` and `AREAS((A1,Sheet1:Sheet2!B2))` are all
  `#NAME?`. The `#NAME?` also beats another operand's error: `SUM((E1,Sheet1:Sheet2!A1))` and
  `SUM((Sheet1:Sheet2!A1,E1))` with E1 = `1/0` are `#NAME?`, not `#DIV/0!`. A 3-D operand of an intersection is
  `#VALUE!` on either side: `SUM(Sheet1:Sheet2!A1 A1:B2)` and `SUM(A1:B2 Sheet1:Sheet2!A1)`. The engine matches all
  eight. Probe-harness limit: a computed or literal-error operand inside a union (`(1/0,x)`, `(#DIV/0!,x)`) makes
  Excel for the web refuse the workbook (`FileCorruptTryRepair`), so the error operand is a cell reference.
- `_xlnm.Print_Area` = `Sheet1!$A$1,Sheet1!$B$2` (sheet-local): `SUM(Print_Area)` = 6, `AREAS(Print_Area)` = 2.
- FORMULATEXT of a union written as a dynamic-array formula (the only kind the probe harness writes) has no `@`
  (`=(A1,B2)`, `=SUM((A1,B2))`); a written `@(A1,B2)` is kept (`=@(A1,B2)`), and gives `#VALUE!`. The same holds for a legacy
  (non-array, no `cm`) formula: FORMULATEXT is `=(A1,B2)`, `=@(A1,B2)` and `=SUM((A1,B2))`, values `#VALUE!`,
  `#VALUE!`, 6; Excel does not add an `@` to a union in a legacy formula (probes/multiarea-at-legacy, T-1077). Controls in the same case: legacy `=A1:A2` and `=ABS(A1:A2)` (rows 7, 9) print
  `=@A1:A2` and `=ABS(@A1:A2)` and give `#VALUE!` (empty intersection), so the legacy path shows `@`s Excel adds;
  a union never gets one. The eval adapter loads `legacy_formulas` cells through `Input::LegacyFormula` (T-1096), and the whole case
  passes on that path.
  So the engine must not wrap a union in `@` when printing a legacy formula.
- `AREAS(Data1:Data3!A1)` is `#VALUE!`. A workbook holding `AREAS(1)` or `AREAS("A1")` is refused by Excel for the web
  (`createSession` 501 `unsupportedWorkbook`, two scratch runs, not kept as cases, like the T-0138 refusals).
- ISREF of a cross-sheet union written in place is FALSE (`ISREF((Data!A1,A1))`), of a same-sheet one TRUE
  (`ISREF((Data!A1,Data!B2))`). An error cell inside an area is the aggregate's error as in a plain range, the
  first in formula order winning (`SUM((A1,E1))` = `#DIV/0!` with E1 = 1/0, `SUM((E2,E1))` = `#N/A` with E2 = NA()),
  COUNT skips it (1) and COUNTA counts it (2). SUMSQ, AVERAGEA, MAXA, GEOMEAN, VAR.S and DEVSQ take a union too
  (`(A1,B2)`: 26, 3, 5, 2.2360679775, 8, 8) (probes/multiarea-more, oracle: excel-graph, 2026-10-08, T-1062).
  PERCENTOF takes a union in either argument (`PERCENTOF((A1,B2),(A1,B2))` = 1,
  `PERCENTOF(A1,(A1:A3,B2))` = 0.025; probes/percentof-snap-overflow-union, excel-graph, 2026-10-13, T-0416).
- The remaining `for_each_value` aggregates over `(A1,B2,C3,D4)` = 1, 5, 7, 9 (probes/multiarea-foreach, excel-graph,
  2026-10-08, T-1150): HARMEAN 2.7510917031, AVEDEV 2.5, SKEW -0.7528371991, SKEW.P -0.4346507596, KURT
  0.3428571429, MINA 1, STDEV.P, STDEVP and STDEVPA 2.9580398915, STDEV and STDEVA 3.4156502553, VAR.P, VARP and
  VARPA 8.75, VAR and VARA 11.6666666667. All take the union. `MODE` and `MODE.SNGL` are the exception: a union
  (`MODE((A1,B2,C3,D4,E5))`) is `#VALUE!`, not a mode (the engine matches since T-1184).
Status: VERIFIED (Excel for the web)
Implemented in: AGGREGATE and the intersection; since T-1062 every aggregate on the shared argument path
(`Model::evaluate_reference_arg`, functions/util.rs: SUM, COUNT, COUNTA, AVERAGE, AVERAGEA, MIN, MAX, MINA,
MAXA, PRODUCT, the STDEV/VAR families and the other users of `for_each_value`, and VARA, VARPA, STDEVA and
STDEVPA since T-1184) walks the areas one after the
other (`Model::for_each_cell_in_areas`). Since T-1068 LARGE, SMALL, RANK.EQ/RANK.AVG and the data_array of FREQUENCY
take a union the same way (`Model::values_from_areas`; FREQUENCY's bins_array is unprobed and stays a single reference).
Since T-1064 a LET variable or LAMBDA parameter bound to a union
(`Binding::Areas`, reference_operators.rs; `Model::bind_argument`, functions/logical/let.rs), the branch IF or
CHOOSE picks with a scalar condition or index (`Model::if_impl`, `Model::choose_impl`) and LET's body
(`Model::let_impl`) stay areas in reference context and as intersection operands.
UNVERIFIED (no probe): an IF or CHOOSE over an array condition keeps its value form.
SPEC-0004 and T-1065, T-1069 to T-1073 implement the rest;
until then CONCAT and TEXTJOIN read a union as `#VALUE!` (`RefArg::into_single_area`).
PERCENTOF takes a union
too (T-0416, probes/percentof-snap-overflow-union D8, D9, D13).
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

## Bare union
Rule: outside every parenthesis and argument list, a comma is the union operator, and it binds
tighter than arithmetic and looser than the space: `=A1,B2` loads, is `#VALUE!` (a union as a
value) and prints `=A1,B2`; `=1+A1,B2` is `1+(A1,B2)` = `#VALUE!`, `=NA()+A1,B2` is `#N/A` (the
left error first), `=A1,B2+NA()` is `(A1,B2)+NA()` = `#VALUE!`, `=A1,B2 C3` and `=A1 B1,B2` are
`#NULL!` (the empty intersection is the union's error); `=A1:A2,B2,C3`, `=-A1,B2`, `=A1,B2+1` and
`=(A1,B2),C3` load and are `#VALUE!`. FORMULATEXT prints every one as written. The operands must be
references: Excel for the web refuses a workbook holding `=1,A1`, `=A1,1`, `=SUM(A1),B2` or
`=A1,B2,` (one scratch workbook per formula, excel-graph 2026-10-08, T-1073). Inside parentheses
the union is the loosest operator instead (`(A1+B1,C1)` is refused, "Union: parsing and
evaluation"). Desktop typed entry is unprobed (M14).
Evidence: probes/multiarea-bare H1, probes/multiarea-bare-precedence H1:J10 (excel-graph).
Status: VERIFIED. Implemented in: `Parser::parse_bare_union` (parser/reference_operators.rs), printed
by `union_text` without parentheses (`OpUnionKind::bare`; a union in parentheses gets them from its
`Group`, T-1342). A defined-name body parses the same way, so
`Sheet1!$A$1,Sheet1!$B$2` is now a union name body; evaluating such names is T-1065.
Tasks: T-1061, T-1073, T-1065

## SUBTOTAL and AREAS take references only
Rule: every SUBTOTAL argument after the function number, and the AREAS argument, must be written as
a reference operand, the same set the intersection accepts ("Intersection operands must be
references"): references, unions, intersections, `@`, `(A1)`, `#REF!`, names, LAMBDA, and the
reference-returning calls INDEX, OFFSET, IF, CHOOSE, LET, XLOOKUP, IFS, SWITCH, TAKE, DROP,
TRIMRANGE, INDIRECT. Anything else makes Excel for the web refuse the workbook (createSession 501
`unsupportedWorkbook`, `FileCorruptTryRepair`): `SUBTOTAL(9,{1,2})`, `SUBTOTAL(2,{1,2})`,
`SUBTOTAL(109,{1,2})`, `SUBTOTAL(9,A1:A2*1)`, `SUBTOTAL(9,SUM(A1:A2))`, `SUBTOTAL(9,ABS(A1:A2))`,
`SUBTOTAL(9,1)`, `SUBTOTAL(9,"A1")`, `SUBTOTAL(9,TRUE)`, `SUBTOTAL(9,A1:A2,{1,2})`,
`SUBTOTAL(9,FILTER(..))`, `SUBTOTAL(9,VSTACK(A1:A2))`, `SUBTOTAL(9,-A1)`, `SUBTOTAL(9,#N/A)`, an
empty argument (`SUBTOTAL(9,A1,)`, `SUBTOTAL(2,A1,)`) and a nested SUBTOTAL (`SUBTOTAL(9,SUBTOTAL(9,A1:A2))`,
also after a range), plus `AREAS(1)` and `AREAS("A1")` (T-1063). One scratch workbook per formula,
excel-graph 2026-10-08 (T-1163); a refused workbook cannot be a case. An argument written as a
reference operand that evaluates to something else loads and gives:
- a value is `#VALUE!`: `SUBTOTAL(9,IF(TRUE,5))`, `SUBTOTAL(2,IF(TRUE,"a"))`, `SUBTOTAL(3,IF(TRUE,"a"))`,
  `SUBTOTAL(9,LAMBDA(x,x))` (also 2, 3, 109 and through LET), `AREAS(IF(TRUE,5))`, `AREAS(LAMBDA(x,x))`;
- an error is itself: `SUBTOTAL(2,IF(TRUE,1/0))` = `#DIV/0!`;
- the non-reference result wins over an error cell in a range, whatever the order:
  `SUBTOTAL(9,C1,IF(TRUE,5))` and `SUBTOTAL(9,IF(TRUE,5),C1)` are `#VALUE!` with C1 = `#DIV/0!`;
- an array spills, `#VALUE!` per element and an error element kept: `SUBTOTAL(9,IF(TRUE,{1,#N/A}))`
  spills `#VALUE!`, `#N/A`; `SUBTOTAL(9,A1:A2,IF(TRUE,{1,2}))` spills two `#VALUE!`.
- two non-reference arguments: the first one wins, whatever it holds: `SUBTOTAL(9,IF(TRUE,"a"),IF(TRUE,1/0))` is
  `#VALUE!`, `SUBTOTAL(9,IF(TRUE,1/0),IF(TRUE,"a"))` is `#DIV/0!`;
- AREAS of an array spills `#VALUE!` per element: `AREAS(IF(TRUE,{1,2}))` fills two cells;
- an invalid function number is `#VALUE!` next to arguments that resolve to references or values
  (`SUBTOTAL(12,A1:A2)`, `SUBTOTAL(12,IF(TRUE,5))`, `SUBTOTAL(0,IF(TRUE,5))`), but an error in the arguments
  beats it: `SUBTOTAL(99,#REF!)` is `#REF!`, `SUBTOTAL(12,IF(TRUE,1/0))` is `#DIV/0!`, and an array argument still
  spills (`SUBTOTAL(12,IF(TRUE,{1,2}))` is two `#VALUE!`);
- operands taken over from the intersection rule load and are the cells they pick: OFFSET, XLOOKUP, IFS, SWITCH,
  TAKE, DROP, TRIMRANGE, INDIRECT and a name holding a range (`SUBTOTAL(9,DROP(A1:A2,1))` = 2); a name holding a
  constant (`=5`) is `#VALUE!`.
Evidence: probes/subtotal-reference-operands (excel-graph, T-1163), probes/subtotal-reference-operands-b
(excel-graph, 2026-10-09, T-1175) and probes/subtotal-reference-operands-c (excel-graph, 2026-10-09, T-1206).
Further rules (-c): TAKE over a literal array is an array, not a reference, so `SUBTOTAL(9,TAKE({1,2;3,4},1))` and
`AREAS(TAKE({1,2},1))` spill two `#VALUE!` (H1:I2); `AREAS(IF(TRUE,{1,#N/A}))` spills `#VALUE!`, `#N/A`: the error
element is kept (H3:I3). SUM, COUNT and AVERAGE treat a single-cell XLOOKUP/TAKE/DROP/TRIMRANGE result as a
reference. Through XLOOKUP: SUM ignores a text (H4) and a boolean (H5, H6) picked cell and gives 0, COUNT ignores a
boolean (H7) and a blank (H8), AVERAGE of a boolean picked cell is `#DIV/0!` (H9). The TAKE cells (H10 text, H11 and
H12 boolean, H13 COUNTA of a blank gives 0) and the SUM blank case H14 give the same values but cannot tell a
reference from a 1x1 array or a blank value, so they are not evidence for the rule (T-1211). AVERAGE is probed with
a boolean only.
Status: VERIFIED for these cells. Implemented in: `Parser::function_call` (parser/reference_operators.rs),
`Model::subtotal_reference_args` (functions/subtotal.rs), `fn_areas`, `not_a_reference_array`, spill shape in
`subtotal_static_result` and `areas_static_result`, XLOOKUP/TAKE/DROP/TRIMRANGE as references in
`Model::evaluate_picked_reference_arg` (SUBTOTAL, AREAS, SUM, COUNT and `for_each_value`). `for_each_value` is shared,
so MEDIAN, GEOMEAN, HARMEAN, SKEW, KURT, VAR*/STDEV*, AVEDEV, DEVSQ and SUMSQ also read these results as references.
VERIFIED by T-1211 (probes/subtotal-reference-operands-d, excel-graph, 2026-10-09): with B1 = "x", B2 = TRUE, B3 blank
and the numbers 5, 7 next to the picked cell, `XLOOKUP` of the text, boolean and blank cell all leave MEDIAN (6),
STDEV.S, SUMSQ (74) and GEOMEAN (with 2, 8: 4) as if the cell were absent, and MIN (5) and MAX (7) ignore it too;
COUNT is 2 and COUNTA 3, 3, 2 (the blank is not counted). AVERAGEA counts the cell as a reference does: text is 0
(`AVERAGEA(XLOOKUP(1,..),5,7)` = 4), TRUE is 1 (13/3), a blank is skipped (6). MIN, MAX and AVERAGEA now read the call
through `evaluate_picked_reference_arg`. A computed array source is an array, not a reference:
`SUBTOTAL(9,TAKE(SEQUENCE(4),1))` and `AREAS(TAKE(SEQUENCE(4),1))`, and the DROP forms, are `#VALUE!`, and
`SUBTOTAL(9,TAKE(SEQUENCE(4),2))` spills two `#VALUE!` (J4:J5); `SUBTOTAL(9,TAKE(A1:A4,1))` is 1. Sized by
`is_read_reference` (static_analysis/aggregate.rs). Unrelated finding: `GEOMEAN(5,7)` is 5.916079783099616 in Excel (H28)
and the engine gives ...615 (1 ulp, `NumericUlp`, a known failure for T-1284; the other GEOMEAN cells use 2, 8 to stay out
of it). MIN, MAX and AVERAGEA over a TAKE/DROP result of one or two cells behave like the XLOOKUP cells (L1:L7).
Typed entry is a desktop
question (M14).
VERIFIED by T-1332 (probes/subtotal-reference-operands-e, excel-graph, 2026-10-11): reference-ness follows the value
through IF and LET. `XLOOKUP(1,A1:A4,SEQUENCE(4))` is an array: `SUBTOTAL(9,..)` and `AREAS(..)` are `#VALUE!`, `ISREF` FALSE;
`XLOOKUP(1,A1:A4,LET(x,A1:A4,x))` and `XLOOKUP(1,A1:A4,IF(TRUE,A1:A4))` are references (SUBTOTAL 1, AREAS 1, ISREF TRUE;
the ISREF of the LET-SEQUENCE form is FALSE). `TAKE(IF(TRUE,A1:A4),2)` and `TAKE(LET(x,A1:A4,x),2)` are references
(SUBTOTAL 3, AREAS 1), while `TAKE`/`DROP` of `IF(TRUE,SEQUENCE(4))` or `LET(x,SEQUENCE(4),x)` are arrays: SUBTOTAL and
AREAS spill two `#VALUE!`, ISREF FALSE. `is_read_reference` follows an IF branch or LET body (and a LET variable) to tell the two (T-1365).
Tasks: T-1063, T-1163, T-1174, T-1175, T-1206, T-1211, T-1332, T-1365 (decision 0045)

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
Implemented in: `get_range` (crates/rustsheet-engine/src/reference_operators.rs) and `is_reference`
(crates/rustsheet-engine/src/functions/information/mod.rs), which is TRUE for `a:b` only when `get_range` gives
a range (T-1166).
The same holds when an operand errors or is not a reference (probes/isref-range-operands, excel-graph
2026-10-09, T-1177): `ISREF(A1:INDEX(A1:A3,5))` (`#REF!`, E1), `ISREF(INDEX(A1:A3,5):A1)`, `ISREF(A1:(A1 B2))`
(`#NULL!`), `ISREF(A1:IF(TRUE,1))` and `ISREF(A1:CHOOSE(1,5))` (`#VALUE!`) are FALSE, ISERROR TRUE; with a
reference, `ISREF(A1:INDEX(A1:A3,2))` and `ISREF(A1:(A1:B2 B1:B3))` are TRUE (`SUM` 3 and 33).
An IF corner is the reference it picks, on either side: `ISREF(A1:IF(TRUE,B2))`, `ISREF(IF(TRUE,B2):A1)` and
`ISREF(A1:IF(FALSE,B2,B3))` are TRUE, `SUM(A1:IF(TRUE,B2))` is 33, `ROWS(A1:IF(TRUE,B3))` 3 and `=A1:IF(TRUE,A1)`
1. CHOOSE likewise (`SUM(A1:CHOOSE(1,B2))` = 33), which the engine already matched; the IF arm was added to
`range_operand` (T-1177). T-1227 (probes/isref-if-range-corners, excel-graph 2026-10-10): an IF corner that picks a
union is the union's bounding box (`SUM(A1:IF(TRUE,(B1,B2)))` 33, `ROWS` 2, `COLUMNS` 2, `ISREF` TRUE; `SUM(A1:IF(TRUE,(B3,B1)))`
and `SUM(A1:IF(TRUE,(B1,B3)))` both 66, `ROWS` 3, so neither first nor last area decides); an error is passed on
(`ERROR.TYPE(A1:IF(TRUE,#N/A))` 7, `...INDEX(B1:B3,9)` 4, `ISREF` FALSE); an IF with an array condition is `#VALUE!`
(`SUM(A1:IF({TRUE,FALSE},B2,B3))`, `SUM(A1:IF({TRUE},B2))`; `ISREF` FALSE, `ERROR.TYPE` 3); a 3-D span pick is `#VALUE!`
(`SUM(A1:IF(TRUE,Data1:Data3!B2))`, ISREF FALSE). Only the union differed from the engine (`#VALUE!`); fixed in `range_operand`. A union on the left corner (`IF(TRUE,(B1,B2)):A1`) is inferred symmetric, not probed.
Note for probes: `ERROR.TYPE(` of a multi-cell range spills, so a #SPILL! over an occupied cell is a probe artifact; read such
corners with ROWS/COLUMNS. `=A1:(1/0)` is refused
by the Graph workbook builder (`unsupportedWorkbook`, as T-1111 found below), so it is not in the probe.
Tasks: T-1138, T-1166, T-1177, T-1227

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

## A sheet-qualified cell off the grid is a name
Rule: column letters with a row of 0 or past row 1048576 are not a cell reference after a sheet
prefix either, but a sheet-scoped name, as they are without one: `=Data!H0`, `=Data!H00` and
`=Data!A1048577` are `#NAME?`, `ISREF(Data!H0)` is FALSE, and `Data!H0:I1` is that name joined
to I1 by `:`, so `ROWS(Data!H0:I1)`, `SUM(Data!G0:H2)` and `AGGREGATE(Data!H0:I1,0,Data!E1:E6)`
are one `#NAME?` (no spill). A leading zero is still a row: `Data!H01` is H1. An off-grid second
corner is a name too, joined to the first cell by `:`: `ISREF(Data!A1:B0)` and `ISREF(A1:B0)` are
FALSE, `=Data!A1:B0`, `SUM(A1:B0)`, `SUM(H0:I1)` and `SUM(Data!A1:B1048577)` are `#NAME?`.
An off-grid row next to a `$` (`$H$0`, `H$0`, `$H0`, `Data!$H$0`, `Data!H$0`, `SUM(Data!$H0:I1)`)
cannot be stored: Excel for the web refuses the workbook (createSession 501 FileCorruptTryRepair,
each formula tried alone, 2026-10-09), so there is no value to match; the engine keeps a parse
error (`#ERROR!`) for them, where before T-0296 it read row 0 (`=$H$0` was 0).
Implemented: the lexer reads such a corner as an identifier (sheet prefix kept), and ends a range
token at the first cell when the second corner is off the grid; before T-0296 a row-0 corner lexed
as a reference, found by the formula_eval fuzz target.
Evidence: probes/aggregate-range-function-num-grid Calc!A16, A21:A29 and probes/off-grid-corners
(oracle: excel-graph, 2026-10-09, T-0296).
Status: VERIFIED (web)
Tasks: T-0296

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
cell or is always "" for a multi-cell range, and N's behaviour. In a dynamic-array formula T does not lift
over the range either: `T(A1:A3)` with A1 = 5, A2 = "ab", A3 = TRUE is a single "", no spill (probes/text-argument-modes
F9, excel-graph, 2026-10-09, T-1087).
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
question). Implemented in: the written group is a `Node::Group` child of the `@` (parser/mod.rs), printed by `at_text`
(parser/reference_operators.rs; T-1116, T-1343); the refusals in `Parser::parse_range`, `Parser::parse_implicit`
and the ANCHORARRAY call in parser/operands.rs (T-1117).
Deeper levels, `@((A1,B2))` and `@(A1) B1` are probed in the entry "`@` operand parentheses: every level is kept" below (T-1132).
Engine rule (T-1117): the parser takes for each operand of `:`, for the argument of ANCHORARRAY and for
the operand of `#` (stored as `_xlfn.ANCHORARRAY`) exactly the reference operands of the intersection
(`is_reference_operand`, entry "Intersection operands must be references") and reads every other operand
as a syntax error, so the four refused forms are no longer evaluated (they used to be, and `=A1:(1/0)`
printed as `=A1:1/0`, which does not re-parse to the same formula). T-1216 (excel-graph, 2026-10-09; one scratch workbook per formula, a control `=SUM(A1:B1)` in the same shape loaded) verified the inferred refusals: Excel refuses (`FileCorruptTryRepair`)
`=A1:"a"`, `=A1:#N/A`, `=#N/A:A1`, `=A1:SUM(B1)`, `=A1:IFERROR(B1:B2,0)`, `=ANCHORARRAY(SEQUENCE(3))`, `=ANCHORARRAY(A1+0)` and the negated
references `=(-A1):B1`, `=A1:(-B1)` and `=(-A1)#` (unlike the union, which accepts `(-A1,B1)`). It loads, as `#VALUE!`, `=A1:LAMBDA(x,x)`, `=A1:(@1)` and
`=ANCHORARRAY(@1)` (printed `=@1#`) (probes/colon-hash-value-operands). The engine matches all of these (`=ANCHORARRAY(@1)` evaluates to `#VALUE!` and prints bare as `=@1#`; T-1226). Resolved by T-1240 (below): the print follows the written parentheses. Since T-1246 the engine prints the parentheses that were written around the `@` and no others (since T-1343 they are a `Node::Group` around it). The `#VALUE!` for the `@` of a non-number value is probed for `@"a"` and `@SUM(A1)` (T-1240); `@TRUE` is still inferred. The printed `=@1#` re-parses as `@(1#)`, which the engine refuses; move and paste print `(@1)#`, which round-trips. So the intersection's reference operands are not the whole rule: `@1` (the `@` of a value) is also accepted, by `:` and by `#` (`@TRUE` is inferred). Still UNVERIFIED: a value operand of a typed `#` (`(1/0)#`).
Stored `ANCHORARRAY` of an `@` operand (probes/anchorarray-at-operand-kinds, excel-graph, T-1240): FORMULATEXT follows the **written parentheses**, not the operand kind. Written without parentheses, every operand prints bare: `=@A1:B2#`, `=@"a"#`, `=@SUM(A1)#`, `=@INDEX(D1:D2,1)#`, `=@LAMBDA(x,x)#`, `=@-1#`; `ANCHORARRAY((@1))` prints `=(@1)#`. Values: `@A1:B2`, `@1`-in-parentheses, `@"a"`, `@SUM(A1)`, `@LAMBDA(x,x)` and `@-1` are `#VALUE!`; `ANCHORARRAY(@INDEX(D1:D2,1))` spills the spill range of D1 (the `@` of a reference keeps the reference). `ANCHORARRAY(@1/0)` is refused by Excel for the web (`FileCorruptTryRepair`, one scratch workbook). The engine matches the prints since T-1246 (`spill_operand`, parser/reference_operators.rs); move and paste still print `(@1)#`, which round-trips (the bare `@1#` re-parses as `@(1#)`). The engine matches the values of `@-1` (parsed as the `@` of a negation, a value) and `@INDEX(..)` (kept as the reference) since T-1247. UNVERIFIED (T-1285): the signed operand now also parses at top level (`=@-1`, previously a parse error); the engine takes the sign, a bare union/intersection and `%` inside the `@` but not `^` (`@-2^2` is `(@-2)^2`), and `@-B1:B3` off-row gives the top-left element of the negated array. UNVERIFIED (extends the written-parentheses rule): an `@` of a space intersection (`ANCHORARRAY(@(A1:B2 B1))` prints `=@A1:B2 B1#`) and more than one written level (`ANCHORARRAY(((@1)))` prints `=((@1))#`).
Tasks: T-1109, T-1111, T-1115, T-1116, T-1117, T-1216, T-1226, T-1240, T-1246, T-1247

## `:` joins multi-cell operands and names into their bounding box
Rule: the operands of `:` may be multi-cell references or defined names (a parenthesized body
included), and the result is the bounding box of both: with `rg` = `=(Sheet1!$B$1:$B$2)` and
`ig` = `=(INDEX(Sheet1!$B$1:$B$2,2))` (B1:B3 = 1, 2, 3), `ROWS(rg:B3)` is 3 and `SUM(ig:B3)` is 5.
A name spelled like a column (`ig`, `rg`) before `:` is still a name when the right side is not a
column range (`ig:B3`). Other reference-position uses of a grouped name agree with the bare
reference: `OFFSET(rg,1,0,1,1)` is 2, `AREAS(rg)` 1, `INDEX(rg,2)` 2, `CELL("address",rg)` `$B$1`.
Evidence: probes/defined-name-grouped-reference A8:A13 (excel-graph, 2026-10-09).
Unnamed multi-cell operands join the same way (T-1205, probes/range-operator-unnamed-operands, excel-graph 2026-10-09; B1:B3 = 1, 2, 3, C1:C3 = 10, 20, 30): `ROWS(OFFSET(B1,0,0,2,1):B3)` is 3, `SUM(INDEX(B1:B3,1):INDEX(B1:B3,3))` 6, `SUM(B1:B2:C3)` and `SUM(B1:B2:C3:C2)` 66, `ROWS((B1:B2):B3)` 3, and the OFFSET form is a reference (`ISREF` TRUE). An undefined name on either side of `:` is `#NAME?` (`SUM(xx:B3)`, `SUM(B3:xx)`), `ISREF(xx:B3)` FALSE and the bare `=xx:B3` is `#NAME?` too, and so is a bare column letter: `=SUM(A:B3)` is accepted and gives `#NAME?` (probes/range-operator-column-letter-before-colon), which is why the lexer reads a column-spelled identifier before a non-column-range `:` as a name.
Status: VERIFIED. Corner order: see "Sheet-qualified range corners" above.
Implemented in: `Model::get_range`, `range_operand` and `defined_name_range_operand`
(reference_operators.rs, defined_names.rs); the lexer keeps a column-spelled name before `:` an identifier (lexer/mod.rs).
Tasks: T-1173, T-1205

## COUNTBLANK of a name that is not a reference is an error
Rule: `COUNTBLANK(name)` where `name` is not a reference is an error, never a count. A name whose body (or error literal) evaluates to an error gives that error: `=1+"a"` `#VALUE!`, `=1/0` and `=0/0` `#DIV/0!`, `=NA()` `#N/A`, `=SQRT(-1)` `#NUM!`, `=Sheet1!#REF!` `#REF!`, `=Sheet1!B1 Sheet1!B3` (empty intersection) `#NULL!`, an undefined name in the body `#NAME?`. Any other value is `#VALUE!`: `=5`, `="x"`, `=""` (not a blank), `=TRUE`. Names that are references count blanks as for a range, whatever the body spelling: `=Sheet1!$B$1:$B$3` and `=(Sheet1!$B$1:$B$3)` give 1 with B2 empty, a single-cell name or body (`=Sheet1!$C$1`, `=(Sheet1!$C$2)`) counts that one cell (an error cell is 0, an empty cell 1), and `=INDEX(Sheet1!$C$1:$C$3,2)` is a reference too (A16:A21). The error rule covers only names that are not references. A name spelled like a cell address (`bad2` is column BAD, row 2) is that cell, not a name: pick other names in probes.
A literal expression argument (`COUNTBLANK(1/0)`, `COUNTBLANK(1+"a")`) is refused: Excel for the web will not open a workbook holding it (`FileCorruptTryRepair`, T-1199), so no value exists. A first draft of the single-cell probe, using the names `c1n`, `cpb`, `cpn`, `cpe`, `ixb` and `ixn`, was refused the same way; longer names worked (cause unknown, UNVERIFIED).
A body that is an IF or CHOOSE follows the same rule: one that picks a scalar is `#VALUE!` (`=IF(TRUE,5,6)`, `=CHOOSE(1,5,6)`,
`=IF(TRUE,"x",1)`, `=IF(TRUE,"",1)`), one that picks an error is that error (`=IF(TRUE,1/0,1)` `#DIV/0!`,
`=CHOOSE(1,NA(),1)` `#N/A`), and one that picks a range or a single cell counts it as a reference (`=IF(TRUE,Sheet1!$B$1:$B$3)`,
`=IF(FALSE,5,Sheet1!$B$1:$B$3)` and `=CHOOSE(1,Sheet1!$B$1:$B$3,5)` give 1; `=IF(TRUE,Sheet1!$C$2)` and
`=CHOOSE(1,Sheet1!$C$2)` give 1 for an empty cell; `=IF(TRUE,Sheet1!$C$3)` and `=IF(TRUE,Sheet1!$C$1)` give 0 for a
number and an error cell; CHOOSE was probed with the empty cell only) (probes/countblank-formula-names-b, excel-graph,
2026-10-09, T-1207). LET bodies behave like the other bodies once stored with `_xlpm.` variables (T-1209): `=LET(x,5,x)` is `#VALUE!`,
`=LET(x,1/0,x)` is `#DIV/0!`, `=LET(x,Sheet1!$B$1:$B$3,x)` and `=LET(x,Sheet1!$C$2,x)` (empty cell) give 1
(probes/countblank-formula-names-b A14:A17, excel-graph, 2026-10-09). A name whose body is another formula name is unprobed.
Evidence: probes/countblank-formula-names A1:A21 and probes/countblank-formula-names-b A1:A17 (excel-graph, 2026-10-09); probes/union-defined-name-uses H12.
Status: VERIFIED. Implemented in: `fn_countblank` (functions/statistical/count.rs).
Tasks: T-1199, T-1207, T-1209

## COUNTBLANK of a value that is not a reference is #VALUE!, one per element of an array
Rule: an argument that evaluates to a value rather than a reference makes COUNTBLANK `#VALUE!`, as for a name
(above), and an array value lifts it: the result is an array of that shape holding `#VALUE!` in every slot. D1=10,
D2 empty, D3=30, D4="x", D5=TRUE: `COUNTBLANK(INDEX(D1:D5,{1,2,3}))` is a 1x3 spill of `#VALUE!` (H131 of
probes/index-lifted-edge-cases), `INDEX(D1:D5,{2,4})` 1x2, `INDEX(D1:D5,{1;2})` 2x1, `INDEX(D1:D5,{1,2},{1;1})` 2x2,
and even the 1x1 `INDEX(D1:D5,{2})` (a blank cell) is `#VALUE!`; `CHOOSE({1,2},D1:D2,D3:D4)`, `IF({1,1},D1:D2)` and
`IF({1,0},D1:D2,D3:D4)` are 2x2 grids of `#VALUE!`; `SUM` of the INDEX result is `#VALUE!` and `ISERROR` of it TRUE per
element. A scalar value picked by CHOOSE is `#VALUE!` too: `COUNTBLANK(CHOOSE(1,5))` and `COUNTBLANK(CHOOSE(1,""))` (an
empty string is not a blank here). A workbook holding `COUNTBLANK(SEQUENCE(1,2))` and `COUNTBLANK(TOROW(D1:D3))`, with
or without `COUNTBLANK(D1:D2+0)`, would not open in Excel for the web (`FileCorruptTryRepair`, as for the literal
expressions above), so those have no value. An error element stays that error, by the error-argument rule: with
D6 = `=NA()`, `COUNTBLANK(INDEX(D1:D6,{5,6}))` is `#VALUE!`, `#N/A` and `COUNTBLANK(IF({1,0},NA(),D1))` is `#N/A`,
`#VALUE!` (H30, H32). RANK's `ref` behaves the same over that CHOOSE: `RANK(10,CHOOSE({1,2},D1:D2,D3:D4))` is a 2x2 of
`#VALUE!` (H34; statistics.md, "RANK of a non-reference array").
Evidence: probes/countblank-array-values (excel-graph, 2026-10-13, T-1437); probes/index-lifted-edge-cases H131:J131
(excel-graph, 2026-10-12, T-1376).
Status: VERIFIED (excel-graph), except the 1x1 `INDEX(D1:D5,{2})`: the engine treats a 1x1 array index as a scalar
index, so it counts the picked cell (1) (T-1507). The CHOOSE grids (H20, H34) spill 1x2, not 2x2, because the engine's `CHOOSE({1,2},D1:D2,D3:D4)`
is itself 1x2 (T-1508).
Implemented in: `fn_countblank` (functions/statistical/count.rs) and `reference_parameter_static_result`
(expressions/parser/static_analysis/reference_parameter.rs), shared with RANK.
Tasks: T-1437

## `@` operand parentheses: every level is kept; a group that is only part of the operand is kept too
Rule: FORMULATEXT prints every parenthesis level the user wrote around an `@` operand: `=@((A1))` and
`=@(((A1)))` print as written (value 1), `=@((A1:B1))`, `=@((A1,B2))` and `=@(A1:B2)` print as written (value
`#VALUE!`), `=@(((A1 B1)))` and `=@(A1:A2 B1:B2)` print as written (value `#NULL!`). When the group is
only part of the operand, `=@(A1) B1` and `=@((A1)) B1` print as written, with the parentheses, and are `#NULL!`
(`@(A1)` intersects with `B1`). The engine printed `=@A1 B1` for both (it dropped the group) until T-1134. Excel keeps the group for every shape where it is
not the whole operand, too: `=@(A1 B1) C1`, `=@((A1 B1)) C1`, `=@(A1):B1 C1`, `=@(A1) B1 C1` and `=@(A1) B1:C1`
print as written and are `#NULL!` (probes/at-group-not-whole-operand, excel-graph, 2026-10-09, T-1147). The engine matches all five
(T-1200; since T-1343 the group is a `Node::Group` where it was written).
More shapes (probes/at-group-leftmost-shapes, excel-graph, 2026-10-09, T-1245): Excel prints `=@((A1)):B1 C1`, `=@(A1):B1:C1 D1`,
`=@(A1:B1):C1 D1`, `=@(A1 B1) C1 D1`, `=@(A1 B1 C1) D1`, `=@(A1):OFFSET(B1,0,0) C1`, `=@((A1 B1) C1) D1`, `=@(A1):(B1) C1`, `=@(A1):B1`
(`#VALUE!`; the other nine are `#NULL!`) and `=@(A1 B1):C1` all as written. The engine matches all ten: every
group is a `Node::Group` where it was written (T-1340, T-1343; `=@((A1 B1) C1) D1` and `=@(A1):(B1) C1` since T-1343). With the range as the whole operand (probes/at-group-range-corner-whole, excel-graph, 2026-10-09,
T-1251) Excel prints `=@(A1:B1):C1`, `=@((A1)):B1:C1`, `=@((A1)):B1`, `=@(A1):B1:C1` and `=@(A1):(B1)` as written (all `#VALUE!`);
the engine matches all five (the last since T-1340).
The stored `_xlfn.SINGLE(...)` form cannot be read through excel-graph
(FORMULATEXT shows the displayed formula only), so whether the stored form keeps the parentheses is an M14
desktop question (decision 0039).
Groups nested inside the operand and not wrapping it add no level to the `@` operand. Excel keeps those inner parentheses as
written, as it does in every probed shape (syntax.md "FORMULATEXT keeps every written parenthesis"; probes/at-operand-nested-groups,
excel-graph 2026-10-09, T-1178). The engine prints them all as written (`=@(A1+(B1))`,
`=@((A1) (B1))`, `=@(((A1)) B1)`, `=@((A1)+(B1))`): each group is a `Node::Group` (T-1340, T-1343).
Evidence: probes/at-paren-operands-levels D30:E39 (excel-graph, T-1132); probes/at-group-not-whole-operand (T-1147); probes/at-group-leftmost-shapes (excel-graph, T-1245); probes/at-group-range-corner-whole (excel-graph, T-1251).
Status: VERIFIED (excel-graph); stored form UNVERIFIED
Tasks: T-1132, T-1134, T-1178, T-1200, T-1244, T-1245, T-1251, T-1340, T-1343

## Order statistics over unions
Rule: LARGE, SMALL, RANK, RANK.AVG and FREQUENCY over a union `(a,b,...)` read every area's cells;
overlapping areas count twice (`LARGE((A1:B2,B2),2)` = 9 with A1:B2 = 7,1,3,9 and B2 = 9; `SMALL((A1:B2,B2),2)` = 3;
`RANK(7,(A1:B2,B2))` = 3; `RANK.AVG(9,(A1:B2,B2))` = 1.5). The order of the areas does not change the result
(`LARGE((C3,B2,A1),2)` = `LARGE((A1,B2,C3),2)` = 7, `RANK(7,(C3,B2,A1))` = `RANK(7,(A1,B2,C3),1)` = 2). A real error cell in an area propagates:
`LARGE((A1,E1),1)`, `SMALL((A1,E1),1)`, `RANK(7,(A1,E1))`, `RANK.AVG(7,(A1,E1),1)` and `FREQUENCY((A1,E1),{4})` are all `#DIV/0!` with E1 = `1/0`, while a text cell
in an area ("#DIV/0!" in F1) is ignored (`LARGE((A1,F1),1)` = 7, `RANK(7,(A1,F1))` = 1). SMALL reads `IF(..)`, `INDEX(..)` and a 3-D reference like LARGE
(`SMALL(IF(A1:C1>0,A1:C1),2)` = 5, `SMALL(INDEX(A1:C2,0,2),1)` = 1, `SMALL(Sheet1:Data!A1,1)` = 7). RANK with an array (not a
reference) as the ref argument, `RANK(5,IF(A1:C1>0,A1:C1))`, is `#VALUE!` spilled over the array's shape (1x3).
Evidence: probes/multiarea-order-stats (excel-graph, 2026-10-09, T-1152). Ours: matches (the RANK-of-array row since T-1203; see statistics.md "RANK of a non-reference array").
Status: VERIFIED (excel-graph)
Tasks: T-1068, T-1152, T-1203

## A structured reference to a table whose stored range does not parse is `#REF!`
Rule: when a table's stored `ref` is not a valid range (an untrusted or corrupt xlsx), every structured
reference to that table evaluates to `#REF!`. This is the choice made when the engine's `expect` on the parse
was removed (a panic on untrusted input is forbidden); the neighbouring cases of a missing table or a missing
table sheet are parse errors, so the two differ. Excel never writes such a `ref` and its handling of a file that
has one is unknown (likely repair or refusal). The same holds for a stored header or totals row count that
does not fit the row arithmetic (a count above `i32::MAX`, or one that overflows `i32` with the range's rows): `#REF!`, not a
wrap or a panic (T-1234).
Evidence: none for Excel; input hardening (CLAUDE.md directive 5).
Status: DEVIATION (decision 0014)
Implemented in: expressions/parser/operands.rs (`structured_reference_operand`)
Tasks: T-1210, T-1234

## Signed and function operands of @
Rule (probes/at-signed-operand, excel-graph, 2026-10-11, T-1285): `@` takes a signed operand and the sign binds inside it. `=@-1` = -1; `=@+1` = 1 and prints `=@1` (unary plus dropped); `=@-B1:B3` = top-left of the negated range, -5 (formula on row 31, off the range rows, and not #VALUE!); `=@-B1 B1:C1` = -5; `=@-B1` = -5; `=@-B1:B3*2` = -10; `=@-1%` = -0.01; `=@-2^2` = 4, `(-2)^2`, and FORMULATEXT prints it as written `=@-2^2` (fixed in T-1320). `=ANCHORARRAY(@IF(TRUE,D1))` spills D1's array (`@` of an IF that picks one cell keeps the reference, like `@INDEX`; the engine gave #VALUE! until T-1321). `=@OFFSET(B1,0,0,3,1)` and `=@INDIRECT("B1:B3")` placed off-row are #VALUE! and print as written.
Rule (probes/at-if-spill-anchor-operand, excel-graph, 2026-10-11, T-1337): `ANCHORARRAY(@IF(TRUE,D1:D3))` implicitly intersects the picked range at the formula cell, then takes that cell as the anchor: in H1 it spills D1# (1,2,3); in J2 the pick is D2, which is not a spill anchor, so #REF!; in L5 (off the range rows) #VALUE!. `ANCHORARRAY(@IF(TRUE,(D1,E1)))` (a picked union) and `ANCHORARRAY(@IF(FALSE,D1,5))` (a non-reference) are #VALUE!. Outside spill anchors `@IF(TRUE,B1)` stays a reference: `VLOOKUP(1,A1:C3,@IF(TRUE,B1))` with B1 text "2" gives "2" (column index "2" coerced), and `LOOKUP(99,A1:A3,@IF(TRUE,F1))` = 1, equal to the plain `LOOKUP(99,A1:A3,F1)` (1, although F1 is 77; unexplained), while a scalar result_vector `LOOKUP(99,A1:A3,77)` is #N/A. The engine matches since T-1368.
Status: VERIFIED (excel-graph)
Implemented in: expressions/parser/operands.rs (signed operand, T-1247); FORMULATEXT of `=@-2^2` fixed (T-1320); `ANCHORARRAY(@IF(...))` in `spill_anchor_operand` (reference_operators.rs, T-1321; spill anchors; the other lookup callers of `eval_ref_or_index` are probed for `@IF` by T-1337: VLOOKUP matches); LOOKUP result_vector `@IF` in `lookup_result` (functions/lookup_and_reference/lookup.rs, T-1368)
Tasks: T-1285, T-1320, T-1321, T-1337, T-1368

## FORMULATEXT of @ next to ^
Rule (probes/at-power-text, excel-graph, 2026-10-11, T-1320): `@` is never parenthesised as an operand of `^`; written parentheses are kept. `=@A1^2`, `=2^@A1`, `=@-A1^2`, `=@2^2`, `=@(A1)^2`, `=@A1:A2^2` (#VALUE!), `=@-2^2+1`, `=2^@-2` (0.25), `=@SUM(A1)^2` and `=@-2%^2` (0.0004) print as written; `=@A1^2^3` (729, left-associative) prints bare; `=(@-2)^2` keeps its parentheses.
Implemented in: expressions/parser/stringify.rs (`operator_text`, power arm; `=@A1^2^3` by T-1340; the written group of `=(@-2)^2` is a `Node::Group` since T-1343).
Rule (probes/power-chain-text, excel-graph, 2026-10-11, T-1331): FORMULATEXT of a `^` chain follows the written parentheses, not the operator's associativity. `=A1^2^3` (729) and `=2^3^2` (64) print bare; `=(A1^2)^3` (729), `=(2^3)^2`, `=A1^(2^3)` (6561), `=2^(3^2)` and `=(A1^2)^3^2` print as written; `=A1^2^3^2` prints bare. The two writings parse to different trees: a written group is a `Node::Group` (SPEC-0007), and the power arm prints a left `^` operand bare.
Implemented in: expressions/parser/stringify.rs (`operator_text`, power arm) and the `Group` node (T-1340).
Status: VERIFIED (excel-graph)
Tasks: T-1320, T-1331, T-1340

## Implicit intersection of a one-row range by a cell in that row
Rule: `@` (or a scalar context) over a one-row range intersects by column whenever the formula cell's column lies
inside the range, including when the cell is also in the range's row number and on another sheet:
`@XLOOKUP(2,Data!A1:A3,Data!B1:C3)` (the reference Data!B2:C2) is Data!B2 entered in another sheet's B2 and
Data!C2 in C2. Column ranges intersect by row; a 2-D range never intersects (`#VALUE!`).
Evidence: probes/xlookup-row-column-picks B2, C2, G3, E4, G15 (excel-graph, 2026-10-11, T-0962), through `@XLOOKUP`
only; a plain `@Data!B2:C2` is unprobed (T-1333).
Implemented in: `implicit_intersection` (crates/rustsheet-engine/src/implicit_intersection.rs), shared by every
implicit intersection.
Status: VERIFIED (web)
Tasks: T-0962, T-1333

## Written parentheses are transparent in reference context
Excel for the web (excel-graph, 2026-10-11, probes/written-parens-reference-context, T-1339; plain formula
against the same formula with written groups, each with FORMULATEXT): `ISREF(A1)` and `ISREF((A1))` are TRUE;
`ROWS(A1:A3)` and `ROWS(((A1:A3)))` are 3; `SUM(A1,B1)` and `SUM(((A1,B1)))` are 11 (the union survives the
group); `LET(x,A1:A3,ROWS(x))` and `LET(x,(A1:A3),ROWS(x))` are 3; `ANCHORARRAY(IF(TRUE,D1))` and
`ANCHORARRAY(@(IF(TRUE,D1)))` spill the same values (D1 = SEQUENCE(3)). FORMULATEXT prints every written group
back (`=ISREF((A1))`, `=ROWS(((A1:A3)))`, `=SUM(((A1,B1)))`, `=LET(x,(A1:A3),ROWS(x))`), and prints ANCHORARRAY(x) as
`x#`: `=IF(TRUE,D1)#` and `=@(IF(TRUE,D1))#`. The one value a group changes is the root snap:
`=0.1+0.2-0.3` is 0 but `=(0.1+0.2-0.3)` and `=((0.1+0.2-0.3))` are 5.551115123125783e-17 (already implemented,
root `Group`, T-1341). Not probeable through excel-graph: `=(INDEX(D1:D3,1))#` (the Graph workbook is rejected as
unsupported, `FileCorruptTryRepair`), so a group directly before `#` stays unevidenced.
Ours: the values and the root snap match; FORMULATEXT drops the redundant groups (H10, H11, H14) and plain
`ANCHORARRAY(IF(TRUE,D1))` now spills like the grouped form (T-1363: an IF that picks one cell is the reference).
Reference-only checks (excel-graph, 2026-10-11, probes/written-parens-reference-only, T-1364): `ISREF((A1:A3))`,
`ISREF(LET(x,(A1:A3),x))` are TRUE like the plain forms; `OFFSET((A1:A3),1,0)` spills 2, 3, 0 like `OFFSET(A1:A3,1,0)`;
`CELL("address",(A1))` is `$A$1` like `CELL("address",A1)`; `ANCHORARRAY(@IF(TRUE,D1))` and
`ANCHORARRAY((INDEX(D1:D3,1)))` both spill 1, 2, 3 like `ANCHORARRAY(INDEX(D1:D3,1))`. Built as
`ANCHORARRAY((INDEX(D1:D3,1)))`, Graph accepts the `)#` shape and FORMULATEXT prints `=(INDEX(D1:D3,1))#`
(`ANCHORARRAY(@IF(TRUE,D1))` prints `=@IF(TRUE,D1)#`). So a written group is transparent for references in every probed cell; no
non-transparent cell exists for SPEC-0007 (T-1340).
Ours: `ISREF(LET(x,A1:A3,x))` (grouped or not) is FALSE (T-1106); FORMULATEXT drops the groups in the reference-only cells too (T-1340).
Evidence strength: `ISREF((A1))`, `ISREF((A1:A3))` and the union in `SUM(((A1,B1)))` show a reference survives a group.
Status: VERIFIED (web).
Tasks: T-1339, T-1340, T-1363, T-1364

## ANCHORARRAY / `#` over IF and non-anchor operands

VERIFIED by T-1369 (probes/anchorarray-if-operands, excel-graph, 2026-10-11): `ANCHORARRAY(IF(TRUE,D1))`,
`ANCHORARRAY(@IF(TRUE,D1))` and `ANCHORARRAY(IF(FALSE,A1,D1))` over a spill anchor D1 (`=SEQUENCE(3)`) spill D1#.
`ANCHORARRAY(IF(TRUE,A1))` and `ANCHORARRAY(A1)` with a plain-value A1 are `#REF!`. Any multi-cell range anchor is
`#REF!`, not `#ERROR!`: `ANCHORARRAY(D1:D3)`, `ANCHORARRAY(A1:B2)`, `ANCHORARRAY(IF(TRUE,D1:D3))` and
`ANCHORARRAY(IF(TRUE,A1:B2))`; a one-cell range `ANCHORARRAY(D1:D1)` spills like `D1#`. Excel for the web refuses
the workbook (`FileCorruptTryRepair`) for the written `IF(TRUE,D1)#`, `@IF(TRUE,D1)#`, `ROWS(IF(TRUE,D1)#)` and
`ISREF(IF(TRUE,D1)#)`, so those stay a desktop question (M14). The ANCHORARRAY function form is what was probed;
the engine gives the written `(range)#` form the same `#REF!` by inference (both are one `SpillRangeOperator`).
Evidence strength: excel-graph oracle, 21 cells. Status: VERIFIED (web).
Tasks: T-1369

### LET / LAMBDA binding of `@INDEX(..)` (T-1503)

`@INDEX(B1:B3,2)` bound by LET or passed to a LAMBDA keeps the reference, like the unprefixed INDEX: ISREF TRUE, ROW 2, value 20, `COUNT(x)` 1, and calling the name is `#REF!` (probes/let-bind-implicit-index H1:H9, excel-graph 2026-10-13).
Evidence strength: excel-graph oracle, 9 cells. Status: VERIFIED (web).
Tasks: T-1503

### ISREF of a written `@INDEX(..)` (T-1506)

`=ISREF(@INDEX(B1:B3,2))` is TRUE (probes/let-bind-implicit-index H10, excel-graph 2026-10-13). Fixed by T-1506: `is_reference` has an `ImplicitIntersection(INDEX)` arm. `@XLOOKUP` and `@TAKE` keep the reference the same way (T-1509): `ISREF(@XLOOKUP(2,A1:A3,B1:B3))` and `ISREF(@TAKE(B1:B3,1))` are TRUE, also LET-bound (probes/isref-implicit-xlookup-take H1, H3, H5, H6). An `@` whose intersection fails (formula row outside the picked rows) or a failing INDEX is not a reference: `ISREF(@TAKE(B1:B3,2))`, `ISREF(@INDEX(B1:C3,0,1))` and `ISREF(@INDEX(B1:B3,9))` are FALSE (H7:H11), plain or LET-bound. UNVERIFIED: `@XLOOKUP`/`@TAKE` as the operand of `#`/ANCHORARRAY, LOOKUP's `result_vector` and a lookup index argument; the engine reads values there (no probe yet).
Evidence strength: excel-graph oracle, 11 cells. Status: VERIFIED (web).
Tasks: T-1506, T-1509
