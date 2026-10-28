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
A scalar function over a one-sheet span's range lifts and spills like the plain range
(probes/3d-ref-one-sheet-span-lift, excel-graph, 2026-10-22, T-2351): `=ABS(Data1:Data1!A1:A3)` spills 1, 5, `#VALUE!`
exactly as `=ABS(Data1!A1:A3)`, `ROWS(ABS(..))` is 3 and `SUM(ABS(Data1:Data1!A1:B1))` 4, while the two-sheet
`=ABS(Data1:Data2!A1:A3)` is `#REF!`. Excel stores the one-sheet span as the plain reference: `FORMULATEXT` of
`=ABS(Data1:Data1!A1:A3)` is `=ABS(Data1!A1:A3)`.
In a legacy-import workbook (formula stored without the dynamic-array flag) the span takes the automatic `@` exactly as the plain
range does (probes/3d-ref-one-sheet-span-legacy-at, excel-graph, 2026-10-24, T-2564): `SIN(Data1:Data1!A1:A10)` is 0.8414709848078965
with `FORMULATEXT` `=SIN(@Data1!A1:A10)`, identical to `SIN(Data1!A1:A10)`; and (dynamic-array formulas, explicit `@`) `FORMULATEXT` of
`=@Data1:Data1!A1` is `=@Data1!A1`, the same as `=@Data1!A1` (value 1).
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

## 3-D references when sheets move or are renamed
Rule (excel-graph, replayed sheet edits, T-1608):
- Renaming an end sheet renames it in the formula: `=SUM(Jan:Mar!A1)` becomes `=SUM(Jan:Last!A1)`.
  When the new name needs quotes, the whole span prefix is quoted: `=SUM('First One:Mar'!A1)`.
  Evidence: probes/sheet-ops/sheet-ops-rename-span-end, sheet-ops-rename-span-start (B1, B2).
- Moving a middle sheet out of the span drops it, and moving a sheet between the ends adds it.
  This includes a sheet just added there, which is empty. Evidence: sheet-ops-move-out-of-span,
  sheet-ops-move-into-span and sheet-ops-add-into-span.
- **Moving an end sheet past the other end shrinks the span to the sheets still between them.**
  Excel rewrites the formula: with Calc, Jan, Feb, Mar, moving Mar to position 1 gives
  `=SUM(Jan:Feb!A1)` (30), and moving Jan to position 3 gives `=SUM(Feb:Mar!A1)` (50). The span is
  not re-resolved by name. Evidence: sheet-ops-move-span-end-first, sheet-ops-move-span-start-last.
  A span left with one sheet becomes a plain reference: moving Feb before Jan turns
  `=SUM(Jan:Feb!A1)` into `=SUM(Jan!A1)` (sheet-ops-move-two-sheet-span-end-first). Spans in
  defined names shrink the same way (sheet-ops-move-span-end-in-name).
  The engine rewrites them before the move (`move_sheet_in_node`, T-1691), and its undo puts the
  written text back.
- Moving the sheet that holds the formula changes nothing (sheet-ops-move-formula-sheet).
Unprobed: moving the first sheet of a two-sheet span past the last (the engine writes the last
sheet alone), a span written backwards (`Mar:Jan`, which the engine rewrites in tab order when it
shrinks), deleting an end sheet and inserting or deleting rows or columns. The excel-graph backend
cannot delete a sheet (T-1181), and the M14 desktop question is T-1310. The engine makes a span
that lost an end sheet `#REF!` and does not shift a span for row or column edits.
Status: VERIFIED (Excel for the web) for renames and moves; UNVERIFIED for deletes
Tasks: T-1691, T-0198

## A sheet rename rewrites references, names and FORMULATEXT, but not INDIRECT text
Rule (excel-graph, T-1608):
- Renaming a sheet rewrites every reference to it, and FORMULATEXT shows the new name. This
  covers cell, range, whole-column and whole-row references, and the formula's own sheet.
- The new name is quoted when it needs quotes: `'My Data'!A1`, `'2024'!A1`, and `'AB12'!A1` for a
  name that looks like a cell. Renaming a quoted name to a plain one drops the quotes.
- A case-only rename (`Data` to `DATA`) shows the new case.
- Workbook-scoped names follow the sheet.
- INDIRECT text keeps the name it was written with. `INDIRECT("Inputs!A1")` reaches the renamed
  sheet, and `INDIRECT("data!A2")` still resolves after a case-only rename. INDIRECT text naming
  the old name names a missing sheet. That is `#REF!` in desktop Excel, and Excel for the web
  differs (lookup.md, "INDIRECT to a sheet that does not exist is #REF!"), so the sheet-op probes
  do not score those cells.
- Sheets renamed in a chain, or swapped through a scratch name, keep their references: each
  formula follows its sheet, not the name.
Evidence: probes/sheet-ops/sheet-ops-rename-cell-refs, -needs-quotes, -digits, -cell-like,
-from-quoted, -case-only, -defined-names, -twice, -swap, -formula-sheet-self, -indirect-text.
A sheet-qualified sheet-scoped name (`=Data!Local`) also keeps working after the rename (3, and
`#SPILL!` for the bare two-cell name), which the engine does not: T-1693
(sheet-ops-rename-sheet-scoped-name).
Status: VERIFIED (Excel for the web)
Tasks: T-1693

## INDIRECT of sheet-qualified name text
`INDIRECT("<Sheet>!<Name>")` reaches a name scoped to that sheet from any sheet; its prefix follows
formula quoting. Rule, evidence and the calipers pv26_020 anonymisation note: lookup.md "INDIRECT of
`Sheet!Name` text" (probes/indirect-sheet-scoped-name, T-2660).

## INDIRECT does not take 3-D text
Rule (excel-graph): INDIRECT of span text is `#REF!`. `SUM(INDIRECT("Jan:Mar!A1"))` is `#REF!`,
while `SUM(Jan:Mar!A1)` written in the formula sums the span.
Evidence: probes/sheet-ops/sheet-ops-move-out-of-span A3 (the engine matches).
Status: VERIFIED (Excel for the web)

## Undefined names may contain backslashes
Rule: an undefined name such as `HSI\10_1\1460_1\1470_1___val` parses as a name and evaluates
to `#NAME?` (not a syntax error).
Evidence: poi/46535.xlsx Others!L12, M12, M17:M20 (oracle: excel-graph, 2026-10-04); the
StructuredReferences.xlsx table names containing `\` also parse (9 cells).
Deviation note: in comma-decimal locales `\` is the array column separator, so the lexer only reads it
as a name character when the decimal separator is `.`. That rule is an assumption, not probed.
Status: VERIFIED (en-US); comma-decimal locales UNVERIFIED
Tasks: T-0123

## Undefined names may contain typographic quotes
Rule: a token written with the typographic quotes `“` (U+201C) and `”` (U+201D) where a string literal was
meant is a *name* to Excel, so an undefined one evaluates to `#NAME?`, not to a syntax error:
`IF(B2="domestic",“D”,IF(B2="international",“I”,“N/A”))` is `#NAME?`.
Evidence: corpus-bench/48354/1_48354_input.xlsx Sheet1!D2:D6 (5 cells; the stored `<f t="array">` keeps the
U+201C/U+201D characters and the workbook's own cached value is `#NAME?`, as is the excel-graph sidecar).
The quoted token is an ordinary undefined name, not an error shape: it is `#NAME?` only when evaluated
(`IF(FALSE,“D”,1)` is 1), `ISREF(“D”)` is FALSE, `ERROR.TYPE(“D”)` is 5, `“D”&"x"` and `SUM(“A1”)` are
`#NAME?` (`“A1”` is a name, not a cell), `=“N/A”` is `#NAME?`, and FORMULATEXT keeps the curly quotes.
Evidence: probes/t0481-curly-quote-names A1-A10 and -b A1-A2 (excel-graph, 2026-10-26, stored formulas).
Unprobed: whether a *typed* formula behaves the same (the oracle cannot type one), whether a name can be
*defined* with these characters, and other punctuation as a name character (`€`, a quote mid-name
`a“b`, `"x"“y"`; probes/t0481-curly-quote-names-c, pending), and a quoted token as a sheet prefix (`“D”!A1`,
which ours reads as a sheet name).
Ours (T-0481): the lexer reads U+201C and U+201D as name characters, anywhere in a name, and a token
holding one is a name even where it would spell a cell (`is_typographic_quote`, `is_quoted_name` in
crates/rustsheet-engine/src/expressions/utils/mod.rs, used by the lexer); defined names still refuse them.
Status: VERIFIED (stored formulas, excel-graph + Excel-cached); typed formulas and the wider character set UNVERIFIED
Tasks: T-0481

## A structured reference into a linked workbook is `#REF!`
Rule: a table reference qualified by an external-link index, `[1]!Table3[#Data]`, is `#REF!` — not the
`#BLOCKED!` an uncached cell or sheet reference of the same book gives ("External reference with no cached
data is `#BLOCKED!`" below), and not a value, with a link part that caches no table data:
`VLOOKUP(E8,[1]!Table3[#Data],2,FALSE)` is `#REF!` with a link part that caches only the other book's
`sheetNames` (empty `sheetData`, no `definedNames`).
Evidence: corpus-bench/58994/1_58994_input.xlsx Sheet1!F8 (excel-graph sidecar, 2026-10-26, gap triage).
Whether a link part that *does* cache the table's cells changes it is unprobed.
Ours: the lexer reads `[1]!Table3[#Data]` as a structured reference whose table name keeps the prefix; no such table exists, so a stored formula is `#REF!`. A typed one stays a parse error
(`#ERROR!`), which is not evidenced against Excel. The sheet-qualified form `[1]Sheet!Table3[Col]` and
the empty-bracket `[1]!Table3[]` are unprobed (the latter lexes as an external name; T-0482).
Status: VERIFIED (excel-graph)
Tasks: T-0482

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
external reference once, and `Model::pick_reference` hands the picked argument to ROWS,
COLUMNS, ROW, COLUMN, ISREF, AREAS, COUNTA, COUNTBLANK, INDEX and the aggregates; IF's and CHOOSE's value read it as INDEX's (T-1487).
INDEX with an index array into an external reference lifts per element (`index_lifted` via
`external_index_args`): each element is the top-left cell of the scalar pick, read with
`external_picked_value` (`=INDEX(…!A1:C4,{1,2},1)` spills 0, 0; probes/external-ref-no-link-ifchoose
Calc!H1, I1; T-1537). Over a cached range the same lift gives the cached cell per element (`INDEX([1]Uses!$A$1:$B$3,{1,2},1)` is
"Hello!", 5; `{2,3},2` is "7", 0 for the blank), a whole-row or whole-column element (index 0) keeps only its
top-left cached cell (`{0,2},1` is "Hello!", 5; `{1,2},0` is "Hello!", 5), an out-of-range element is a per-element `#REF!`
(`{1,4},1` is "Hello!", #REF!) and a cached `#N/A` stays that element (`INDEX(…A1:C3,1,{1,2,3})` is "Hello!", 10, #N/A)
(fixtures/external-link-cached-ranges Refs!B83:D88, excel-graph, 2026-10-14, T-1546; the engine matches).
In a legacy (non-array) formula, an INDEX with an index array over an external range is a single-cell
reference to its *first element*, not to the range's top-left cell. `SUM(INDEX(…,{1,2},1))` is 0 ("Hello!" is text), `ROWS(…)` is 1, `ISREF(…)` is TRUE and a further
`INDEX(INDEX(…,{1,2},1),2)` is `#REF!` (Refs!B89:B92). With `{2,1}` the two candidates differ, and the first element
(A2, 5) wins: `SUM(INDEX([1]Uses!$A$1:$B$3,{2,1},1))` is 5, `INDEX(INDEX(…,{2,1},1),1)` 5, `…&""` "5",
`COUNTA(…)` 1, `ROWS(…)` 1, `ISREF(…)` TRUE, `INDEX(INDEX(…,{2,1},1),2)` `#REF!`; across a row
`SUM(INDEX(…,1,{2,1}))` is 10 (B1) and `COLUMN(INDEX(…,1,{2,1}))` 2. Entered as a two-cell array formula (B10:C10), the
same INDEX gives one result per element: 5, "Hello!" (fixtures/external-link-cached-index-array Refs!B1:B10, C10, excel-graph,
2026-10-16, T-1554). The engine matches all of these.
A LET is transparent here too: a LET whose body is (or IF, CHOOSE or a nested LET picks) a
reference into another workbook is that reference. `ROWS(LET(c,TRUE,IF(c,…!A1:B3)))` is 3,
`ROWS(LET(c,TRUE,…!A1:B3))` 3, `ROWS(LET(c,TRUE,LET(d,1,CHOOSE(d,…!A1:B4))))` 4,
`ROWS(IF(TRUE,LET(c,1,…!A1:B3)))` 3, `ROWS(LET(c,TRUE,d,2,IF(c,INDEX(…!A1:C4,d,0))))` 1,
`COLUMNS(LET(c,1,CHOOSE(c,…!A1:C1)))` 3, `ROW(LET(c,TRUE,IF(c,…!B3)))` 3, `COLUMN(…C2…)` 3,
`ISREF(LET(c,TRUE,IF(c,…!A1)))` and `ISREF(LET(c,TRUE,…!A1))` TRUE,
`AREAS(LET(c,TRUE,IF(c,(…!A1,…!B2))))` 2, `COUNTA(LET(c,TRUE,IF(c,…!A1:B2)))` 0,
`COUNTBLANK(…)` `#VALUE!`, `SUM(LET(c,TRUE,IF(c,…!A1)))` `#BLOCKED!`; its one cell reads empty
(`=LET(c,TRUE,IF(c,…!A1))` 0, `&"x"` "x", ISBLANK TRUE, ERROR.TYPE `#N/A`). When the IF picks no
reference, LET gives IF's value: `ROWS(LET(c,FALSE,IF(c,…!A1:B3)))` is 1, `ISREF(…)` FALSE, and a
condition error is that error (`ROWS(LET(c,1/0,IF(c,…!A1:B3)))` `#DIV/0!`).
Evidence: probes/external-ref-no-link-let Calc!A1:A20, external-ref-no-link-ifchoose A26
(excel-graph, 2026-10-16, T-1535). Probed since (T-1710, probes/external-ref-no-link-let-index,
excel-graph 2026-10-16): INDEX over such a LET is INDEX over the reference it picks
(`ERROR.TYPE(INDEX(LET(c,TRUE,IF(c,…!A1:C4)),2,2))` `#N/A`, `ROWS(INDEX(…,2,0))` 1), and a LET body
that may also pick another argument is still read with its names bound and the IF's pick made:
`ROWS(LET(c,TRUE,IF(c,…!A1:B3,"x")))` and `ROWS(LET(c,TRUE,IF(c,…!A1:B3,D1:D2)))` are 3,
`ISREF(LET(c,TRUE,IF(c,…!A1,D1)))` TRUE and `COUNTA(LET(c,TRUE,IF(c,…!A1:B2,1)))` 0 (A4:A7). OFFSET
over such a LET is not a reference: `ISREF(OFFSET(LET(c,TRUE,…!A1),1,1))` is FALSE (A3), as OFFSET
over the bare external reference is (external-ref-no-link-index A16, A17).
Engine (T-1721): `Model::pick_reference` reads every LET whose body may pick a reference into
another workbook (`may_be_external`) with its names bound (`let_scope`); when the body picks
another argument (a reference into this workbook, a value), that argument is read there, in
reference context, as a LET's body is (`PickedReference::Read`). OFFSET takes its reference
through `pick_reference`, so OFFSET over a LET picking an external reference is `#VALUE!`.
Unprobed: what Excel gives when such a LET picks the other argument (ours reads it as any LET body:
`ISREF(LET(c,FALSE,IF(c,…!A1,D1)))` TRUE, `ROWS(LET(r,D1:D2,c,FALSE,IF(c,…!A1:B3,r)))` 2).
`ROWS(CHOOSE({1,2},…!A1:B3,A1))` is 3, and the engine matches since T-1508, which broadcasts CHOOSE's
arms over an index array (external-ref-no-link-ifchoose A32).
Unprobed: ROW and COLUMN of an IF lifted over an array condition that may pick an external
reference (`ROW(IF({TRUE,FALSE},…!A1,…!A1))`; the engine gives `#VALUE!`, UNVERIFIED), OFFSET of one
with an array or two error arguments, a `[n]` index with no
externalLink part, and ROW/COLUMN of a multi-cell external area (the engine spills the row or column
numbers as for a sheet reference, UNVERIFIED).
Status: VERIFIED (Excel for the web; error name inferred from errorType 10 and ERROR.TYPE 11)
Tasks: T-0124, T-1381, T-1382, T-1454, T-1487, T-1535, T-1537, T-1546, T-1554

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
(fixtures/external-link-cached-ranges Refs!B24, B55, B77). A name or sheet the cache does not hold is `#BLOCKED!` (probed: fixtures/external-link-cached-lookups Refs!B18-B20, T-1609; see "a name the cache does not hold" below). Unprobed, so UNVERIFIED: a book index with no link part and a book written by file name (`[other.xlsx]`) are `#BLOCKED!` as an
uncached reference is; a sheet with `refreshError="1"` that caches cells is read like any other (one
that caches none reads `#REF!`: "Lookups into a linked sheet with no cached data are `#N/A`", T-0472); `[0]!Missing` is
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
- the reference readers see a reference: ISREF TRUE, ROWS 3, COLUMNS 2, AREAS 1, RANK 2,
  SUBTOTAL(9,…) 15, `AGGREGATE(9,6,…!$A$1:$C$2)` 16 (the error skipped); legacy `TYPE(…)` in row 62
  is 16, an implicit intersection that finds no cell (below);
- lookups, INDEX and element-wise functions read the values: VLOOKUP, HLOOKUP, MATCH, XMATCH,
  XLOOKUP, `INDEX(…,2,2)` "7", `INDEX(…,3,2)` (blank) 0, `SUMPRODUCT(--ISTEXT(…))` 2, ISLOGICAL 1,
  ISBLANK 1, `CONCAT` "Hello!1057TRUE", `TEXTJOIN("|",FALSE,…)` "Hello!|10|5|7|TRUE|";
- the functions that need the other workbook open are `#VALUE!`: COUNTIF, SUMIF, SUMIFS (also over
  `[1]!NR_Range`) and COUNTBLANK, a one-cell `COUNTBLANK([1]Uses!$B$3)` included; OFFSET of one
  (`SUM(OFFSET([1]Uses!$A$1,0,1,2,1))`) is `#VALUE!`;
- in a legacy (non-dynamic) formula, the range is implicitly intersected as a range of this
  workbook is ("Legacy formulas intersect the ranges in value arguments"): in rows 38, 44 and 56,
  outside the range's rows, `SUM(IFERROR(…*1,100))` and `SUMPRODUCT(IFERROR(…*1,100))` are 100 and
  `SUM(…!$A$2:$A$3+0)` `#VALUE!`;
- a sheet-scoped name of a range, `[1]Defines!NR_Local_Range`, is `#BLOCKED!` (ROWS too; ERROR.TYPE
  11), although a sheet-scoped name of one cell reads its cached value; a sheet the cache does not
  hold, `[1]Nope!$A$1:$B$2`, is `#BLOCKED!`.
A cached empty string and cached text (T-1550; fixtures/external-link-cached-text, excel-graph,
cache `Uses` A1 the empty string `t="str"` `<v></v>`, B1 18, A2 12, B2 TRUE, A3 the text "8", A4
"abc", A5 30):
- COUNTA counts the cached empty string as a value, as it counts a `=""` cell of this workbook:
  `COUNTA([1]Uses!$A$1)` 1, over `$A$1:$A$5` 5; `LEN([1]Uses!$A$1)` is 0;
- GCD and LCM read a cached range as a range of this workbook (coercion.md "GCD / LCM /
  MULTINOMIAL range cells"): numeric text counts as its number (`GCD(…!$A$2:$A$3)` 4, LCM 24),
  text, a boolean and the empty string are `#VALUE!` (`GCD`/`LCM` of `$A$2:$A$5`, `$A$2:$B$2`,
  `$A$1:$A$2`), a one-cell `GCD([1]Uses!$A$3)` over "8" is 8, and blanks past the cache read 0
  (`GCD(…!$A$5:$A$9)` 30, LCM 0);
- an area taller than the array bound reads its cached cells: `SUM([1]Uses!$A$2:$B$1048576)` 42,
  COUNT 2, MAX 30, AVERAGEA 8.6.

Lookups, names and spans over the cache (T-1609; fixtures/external-link-cached-lookups and
fixtures/external-link-cached-3d-span, excel-graph, 2026-10-15, cache as in T-1460 above; Excel for
the web keeps the cache on upload):
- lookups read a partly cached range with its uncached cells blank: `VLOOKUP("hello!",…$A$1:$C$3,2,FALSE)`
  10 (case-insensitive), `VLOOKUP(5,…$A$1:$D$9,4,FALSE)` 0 (uncached D2), not found `#N/A`,
  `HLOOKUP(10,…$A$1:$F$4,2,FALSE)` "7", `MATCH(1,…$C$1:$C$5,0)` 2 (past the cached `#N/A`),
  `MATCH(100,…$B$1:$B$9)` 1, `XLOOKUP` "7" and its default "none", `XMATCH(TRUE,…)` 3,
  `LOOKUP(10,…$B$1:$B$5,…$A$1:$A$5)` "Hello!", `INDEX(…$A$1:$D$5,5,4)` 0, INDEX/MATCH over
  `[1]!NR_Range` "7", `MATCH("*o!",…,0)` 1 (wildcards), and `SUMPRODUCT((…$A$1:$A$9="")*1)` 6 (the
  uncached A4:A9 equal ""); a relative `[1]Uses!A1` reads "Hello!";
- sheet and name lookups ignore case: `[1]!nr_global_b2` 142, `[1]uses!$B$1` 10, `[1]defines!nr_to_a1`
  "Test A1";
- a name the cache does not hold (`[1]!NoSuchName`), a sheet-scoped name read without its sheet
  (`[1]!NR_To_A1`), a workbook-scoped name read through a sheet (`[1]Uses!NR_Global_B2`) and one
  cell of a sheet the cache does not hold (`[1]Nope!$A$1`) are `#BLOCKED!` (`ERROR.TYPE([1]!NoSuchName)` 11), yet
  `ISREF([1]Nope!$A$1)` is TRUE;
- a 3-D span of the cached sheets aggregates over them: `SUM([1]Uses:Defines!$B$2)` 142,
  `SUM(…!$A$1:$B$2)` 157, COUNT 4, COUNTA 9, MAX 142, and `SUM([1]Defines:Uses!$B$2)` for the span reversed 142
  (fixtures/external-link-cached-3d-span Refs!B9; the other aggregates reversed are unprobed); but `ISREF([1]Uses:Defines!$A$1)` is FALSE, ERROR.TYPE of the bare span 4 (`#REF!`) and
  ROWS `#VALUE!`, as for a span of this workbook ("3-D references outside the aggregates"), while
  without a link `SUM('[other.xlsx]S1:S2'!A1)` is `#BLOCKED!` (T-1381). Engine (T-1654): the lexer
  reads the unquoted `[n]First:Last!` prefix as the quoted `'[n]First:Last'!` (the formula text
  keeps the quoted form, as for `'[1]Uses'!`), and `Model::cached_span_cells` gives the cells the
  cache holds in the written area on the sheets between the two ends in `sheetNames` order, in
  sheet, row and column order, in one walk of the cache, which the aggregates and COUNTA read
  (blanks skipped); it is `#REF!` as a value and `#VALUE!` where one sheet is needed. Unprobed, so
  UNVERIFIED (T-2331 probes them): a span with an end the cache does not hold, kept `#BLOCKED!`
  here (a span of this workbook with a missing end is `#NAME?`); blanks for the readers that keep
  them (`TEXTJOIN(",",FALSE,[1]Uses:Defines!$A$1:$B$2)`), skipped here; a one-sheet span
  (`[1]Uses:Uses!A1`), read here as a span (`#REF!` as a value, not that sheet's cell as a
  one-sheet span of this workbook is); `@` of a span; and the reversed span in the other
  aggregates.
A workbook whose formulas reference a sheet with `refreshError="1"` (`[1]'56737'!$A$1:$B$2`) does
not open in Excel for the web ("unsupportedWorkbook"), so what that reads is a desktop question.
Engine: a cached area carries its cache (`external_links.rs`); aggregates and RANK read its cells
as reference cells, several areas as one row of cells. Engine-only bound (not Excel behavior, which
is unobservable here): a whole column or row is cut to the extent the cache holds, as a whole
column of this workbook is cut to its used area; an aggregate that skips blanks cuts any other
area to that extent too, and every other reader keeps its shape and blanks (T-1550). The
array bound on the rest (an area or a union's cells above 1,048,576) is a DEVIATION: see
arrays.md "Array results above 1,048,576 cells". SUBTOTAL, AGGREGATE's reference form and RANK
read the cached cells through `Model::external_reference_cells`, AVERAGEA coerces them as MAXA
does, and an automatic `@` on a cached range reads the cached cell it intersects
(`Model::cached_external_at`; T-1526). Unprobed: `@` of an uncached external range, and of a
cached one inside an aggregate's argument (the engine reads the picked cell as a value).
`Sheet!Name` with no book prefix (poi/56737.xlsx Uses!D2 `=Defines!NR_To_A1`, Excel-cached
"Test A1") is the name scoped to that sheet of this workbook; `[0]!Name` and `[0]Sheet!Name` resolve
the same way at parse time, so ISREF and a rename see them. A sheet-qualified workbook-scoped name
(`Sheet!GlobalName`), an unknown `Sheet!Nope` (engine: `#NAME?`) and a missing sheet are UNVERIFIED (T-1459).
Status: VERIFIED (Excel-cached cells above); the cases listed as unprobed are UNVERIFIED
Tasks: T-1382, T-1459, T-1460, T-1526, T-1550, T-1609, T-1654

## Criteria and sum ranges of a linked workbook are `#VALUE!`
Rule: a criteria or sum range of COUNTIF, COUNTIFS, SUMIF, SUMIFS, AVERAGEIF, AVERAGEIFS, MAXIFS or
MINIFS that is a range of a linked workbook is `#VALUE!` without the other file open, even though
other functions read its cached cells (SUM, INDEX, lookups: "Cached values of a linked workbook"
above). A whole column counts too: `COUNTIFS('[1]XA Data Pull'!$J:$J,">="&W$1,...)` is `#VALUE!`.
Evidence: corpus-bench 52450 sheet1, all 1,830 formulas with a COUNTIFS over `'[1]XA Data Pull'!`
ranges (W5:Y5 and the rest of their block), Excel-cached `#VALUE!` (desktop workbook; no oracle run).
Unprobed: a range of a linked sheet the file caches nothing for, where ours is `#VALUE!` too (an
array of `#BLOCKED!`, which is not a range).
Ours: IMPLEMENTED: `ifs_range` (functions/statistical/if_ifs.rs) makes any argument that is not a
range of this workbook `#VALUE!`; `ifs_range_arg` gives a linked range that error without building its
array (`Model::is_external_array`, external_links.rs; T-0477). A single cached cell is `#VALUE!` the
same way; a range past the array bound keeps that bound's error (arrays.md).
Status: VERIFIED (desktop, cached values)
Tasks: T-0477

## Lookups into a linked sheet with no cached data are `#N/A`
Rule: a sheet the externalLink part lists in `sheetNames` but caches no cell of (no
`sheetData` for its `sheetId`, or an empty `<sheetData sheetId="4"/>`) reads `#BLOCKED!` as a
reference with no link does: `=[1]Sort!$B$2` is `#BLOCKED!`, ERROR.TYPE 11, ISBLANK FALSE,
`SUM([1]Sort!$B$2:$C$3)` and `SUMPRODUCT(…)` `#BLOCKED!`, `INDEX([1]Sort!$B$2:$C$3,1,1)+1` 1,
`COUNTIF(…,5)` `#VALUE!`; a cell a cached sheet does not hold stays empty (`ERROR.TYPE([1]Uses!$Z$99)`
`#N/A`). But VLOOKUP, HLOOKUP, MATCH and XLOOKUP over it are `#N/A`, found or not
(nothing is), over one cell, a small range, whole columns (`VLOOKUP(5,[1]Sort!$B:$Z,2,FALSE)`, 1048576
x 25 cells) and whole rows alike, exact or approximate; ERROR.TYPE of each is 7, IFERROR takes the
fallback and XLOOKUP's if_not_found is returned (`"none"`). The `#N/A` comes before the index check:
`VLOOKUP(5,[1]Sort!$B:$Z,30,FALSE)` is `#N/A`, not `#REF!`. The same holds for a sheet whose
`sheetData` has `refreshError="1"` and no cells, but a direct read of that one is `#REF!`
(`ERROR.TYPE('[1]56737'!$B$2)` is 4). Unlike a reference with no link, `COUNTA([1]Sort!$B:$B)` is
1048576: it counts every blocked cell. Over whole columns of a cached sheet Excel gives: `VLOOKUP(5,[1]Uses!$A:$C,2,FALSE)` is the cached "7", `VLOOKUP(6,[1]Uses!$A:$C,3,TRUE)`
1, `VLOOKUP(5,[1]Uses!$A:$Z,26,FALSE)` 0 and `VLOOKUP(99,…)` `#N/A`.
Evidence: fixtures/external-link-uncached-lookups Refs!B1:B51 (excel-graph, 2026-10-25, T-0457).
Unprobed, do not extrapolate: LOOKUP, XMATCH, INDEX/MATCH and defined names (`[1]!Name`) into such a
sheet, a sheet `sheetNames` does not list, and `[other.xlsx]` with no link part. Also unprobed: a range of a
refreshError sheet (ours keeps `#BLOCKED!`), XLOOKUP and COUNTA over a refreshError sheet (ours: as over any
uncached sheet), and XLOOKUP that matches in a local range but returns one uncached cell (ours: `#BLOCKED!`).
Engine: a sheet the cache holds no cell of is uncached (`cached_sheet`, external_links.rs), so it
reads `#BLOCKED!`; `Model::external_lookup_table` gives VLOOKUP's, HLOOKUP's and MATCH's table
`#N/A` on such a sheet. XLOOKUP searches one cell of it as a one-cell vector of its `#BLOCKED!`, as it
searches a range, so it finds nothing (`uncached_cell_vector`). COUNTA counts every cell of it
(`external_blocked_count`). The importer keeps the sheets marked `refreshError`
(`ExternalBook::refresh_error_sheets`), and a cell of one reads `#REF!` (T-0472). VLOOKUP, HLOOKUP and
MATCH read a range of several cells of a cached sheet in place (`LookupTable::External`,
`Model::cached_external_lookup_area`) instead of building it as an array, so whole columns and rows
match B17, B21, B49 and B50 (T-0474). The search vector keeps the area's full length (the cached
cells, then an empty tail), as a whole column of this workbook does; a vector cut to the cache's
extent would move the approximate search's midpoints (lookup.md "Approximate match"). UNVERIFIED, a hand
expectation awaiting an oracle cell (T-0479): an approximate whole-column lookup over an unsorted cached
column (cache 1,5,2,9,3) is the local result. `VLOOKUP(4,[1]Uses!$A:$C,1,TRUE)` is 1 in the engine, as
over a local whole column, and a unit test pins that in test_external_links.rs.
Status: VERIFIED for the oracle cells above (excel-graph only; no desktop evidence, decision 0030); the
unsorted approximate case is UNVERIFIED
Tasks: T-0457, T-0472, T-0474, T-0479

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
ISFORMULA reads the cell INDEX picks, as it reads the cell OFFSET or INDIRECT gives: `ISFORMULA(INDEX(Data!D1:D3,1))` is TRUE
with Data!D1 = `=1+1` (probes/ifna-isformula-edges Calc!B12, excel-graph; B10, B11 for INDIRECT and OFFSET). The engine
matches since T-1453 (`evaluate_call_with_reference` evaluates INDEX in its reference form).
INDEX as the reference argument of FORMULATEXT, CELL and OFFSET (probes/index-reference-args, excel-graph 2026-10-20, T-2059;
Data!A2 `=A1+1`, B2 `=B1*2`, A1/A3 constants): the picked cell is a reference everywhere. `FORMULATEXT(INDEX(Data!A1:A3,2))` is
`=A1+1` and `INDEX(Data!A1:B3,2,2)` gives `=B1*2` (A2, A3 of Calc); a constant pick is `#N/A` (`INDEX(Data!A1:A3,1)`); an
out-of-range index is `#REF!` (`INDEX(Data!A1:A3,9)`) and an array-constant source is `#VALUE!` (`INDEX({1,2,3},2)`), not `#N/A`.
`INDEX(Data!A1:A3,0)` is the whole column, so FORMULATEXT spills its three results (`#N/A`, `=A1+1`, `#N/A`).
`CELL("address",INDEX(Data!A1:B3,2,2))` is the cross-sheet `[file]Data!$B$2`, `"row"` and `"col"` are 2, and `"contents"` is 20
(Data!B2 `=B1*2`; the named-sheet reading of information.md "CELL("contents", ref) reads the sheet the reference names",
re-filled 2026-10-28 by T-3030. The 2026-10-1x fill gave 2, Calc!B2's value); `INDEX(..,9)` is `#REF!` and an array
constant `#VALUE!`; `INDEX(Data!A1:B3,0,2)` gives the address of its top-left cell `[file]Data!$B$1`.
`OFFSET(INDEX(Data!A1:A3,2),1,0)` is Data!A3 (3), `OFFSET(INDEX(Data!A1:B3,2,2),-1,0)` is Data!B1 (10), a 2-row OFFSET off the
picked cell has 2 rows, `OFFSET(INDEX(Data!A1:B3,0,2),1,0)` spills B2:B4 (20, 30, 0), `INDEX(..,9)` is `#REF!` and an array
constant `#VALUE!`. The engine matches, FORMULATEXT's `#REF!` and `#VALUE!` forms
(`FORMULATEXT(INDEX(Data!A1:A3,9))`, `FORMULATEXT(INDEX({1,2,3},2))`) since T-2137. `fn_formulatext` returns INDEX's own error, and
`#VALUE!` when INDEX picks a value rather than a cell. FORMULATEXT of an erroring or array-shaped reference-returning call (probes/formulatext-erroring-references, excel-graph
2026-10-23, T-2301): FORMULATEXT returns the call's own error, unchanged. `INDEX(A1:A3,NA())` is `#N/A`, `INDEX({1,2,3},9)`
`#REF!`; `OFFSET(A1,NA(),0)` is `#N/A`, `OFFSET(A1,"x",0)` `#VALUE!`, `OFFSET(A1,-9,0)` `#REF!`; `INDIRECT(NA())` is `#N/A` and
`INDIRECT("zz zz")` `#REF!` (a missing sheet, `INDIRECT("Nope!A1")`, is left out: the web oracle resolves it to an empty cell, lookup.md, decision 0030);
`CHOOSE(9,..)` and `CHOOSE(0,..)` are `#VALUE!` and `CHOOSE(NA(),..)` `#N/A`; `IF(NA(),..)` is `#N/A`, `IF("x",..)` `#VALUE!`,
`IF(1/0,..)` `#DIV/0!`, and an error in the arm taken, `IF(TRUE,NA(),..)`, is `#N/A`. An array-shaped result is not a reference:
`INDEX({1,2;3,4},0,1)`, `INDEX({1,2;3,4},1,0)` and `INDEX({1,2;3,4},0,0)` spill `#VALUE!` over the shape of the result (2x1, 1x2, 2x2),
`INDEX({1,2;3,4},2,2)` is one `#VALUE!`, and `IF({TRUE,FALSE},..)` and `CHOOSE({1,2},..)` spill a 1x2 of `#VALUE!`. The engine matches all 28 cells (`fn_formulatext`, T-2526). FORMULATEXT of the forms T-2526 left open (probes/formulatext-pick-scalars-and-arrays, excel-graph 2026-10-25, T-2529; Data!A1 `1`, A2 `=A1+1`, A3 `3`): a pick call is a reference only when the arm it takes is a cell, and then FORMULATEXT reads that cell. `IF(FALSE,5,Data!A2)`, `CHOOSE(2,5,Data!A2)`, `IF(TRUE,Data!A2,5)`, `SWITCH(1,1,Data!A2,Data!A1)`, `SWITCH(9,1,5,Data!A2)`, `IFS(TRUE,Data!A2,..)` and `LET(x,Data!A2,x)` give `=A1+1`; a taken arm that is a plain value gives `#VALUE!` (`IF(TRUE,5,Data!A1)`, `CHOOSE(1,5,Data!A1)`, `SWITCH(1,1,5,Data!A1)`, `IFS(TRUE,5,..)`, `LET(x,5,x)`, `LET(x,Data!A2,x+1)`, `XLOOKUP(9,..,..,"nf")`, and TAKE or DROP of `SEQUENCE(3)`). No match is the call's own error: `SWITCH(9,1,5)` and `IFS(FALSE,5)` are `#N/A`, and `XLOOKUP(9,Data!A1:A3,Data!A1:A3)` is `#N/A`. XLOOKUP, TAKE and DROP that pick a cell read it: `XLOOKUP(2,Data!B1:B3,Data!A1:A3)` and `DROP(Data!A1:A3,1,0)` (spills `=A1+1`, `#N/A`), `TAKE(Data!A1:A3,2)` (spills `#N/A`, `=A1+1`), `TAKE(Data!A1:A3,1)`, `TAKE(Data!A1:A3,-1)`, `TAKE(Data!A1:A3,1,-1)`, `DROP(Data!A1:A3,2)` and `XLOOKUP(3,..)` pick the constant cells A1 or A3 and give `#N/A`. An array selector or lookup value is not a reference: `XLOOKUP({1,3},..)` spills 1x2 `#VALUE!`. Array elements keep their own error: `IF({TRUE,FALSE},NA(),Data!A2)` spills `#N/A`, `#VALUE!`; `IF({TRUE,FALSE},Data!A2,NA())` spills `#VALUE!`, `#N/A`; `CHOOSE({1,2},NA(),Data!A2)` spills `#N/A`, `#VALUE!`; `IF({TRUE,TRUE},NA(),Data!A2)` spills `#N/A`, `#N/A`. `INDIRECT` of an array is evaluated per element, each element read as FORMULATEXT of that reference: `INDIRECT({"Data!A2"})` is `=A1+1`; `INDIRECT({"Data!A1","Data!A2"})` spills `#N/A`, `=A1+1`; the column form spills down; `INDIRECT({"Data!A2","zz zz"})` spills `=A1+1`, `#REF!`; `INDIRECT(Data!C1:D1)` (cells holding the text) behaves the same. The no-match errors are read as the call's own `#N/A`, which the probe cannot tell apart from FORMULATEXT's not-a-formula `#N/A`. The engine matches all 49 cells: `fn_formulatext` reads a pick call's argument as SUBTOTAL does (`evaluate_picked_reference_arg`), a lifted INDIRECT per element (`formula_texts_of_grid`), and a value as `#VALUE!` per element with an error element kept (T-2726).
FORMULATEXT of a multi-area reference and of a multi-cell INDIRECT element (probes/t2831-formulatext-indirect-multi-cell, excel-graph, 2026-10-28, T-2831; Data!A1 `1`, A2 `=A1+1`, A3 `3`, A4 `=A3*2`, B1 `=A1*10`): a multi-area reference is read as its first area, and a lifted INDIRECT element that names several cells as its top-left cell. `FORMULATEXT((Data!A2,Data!B1))` is `=A1+1` and `FORMULATEXT((Data!A1,Data!A2))` `#N/A` (the first area's cell is a constant), and pick calls that return the union read the same way: `CHOOSE(1,(Data!A2,Data!B1))`, `IF(TRUE,..)`, `CHOOSE(2,5,..)`, `IF(FALSE,5,..)`, `LET(x,..,x)`, `SWITCH(1,1,..)` and `IFS(TRUE,..)` give `=A1+1`, and `CHOOSE(1,(Data!A1,Data!A2))` and `IF(TRUE,(Data!A1,Data!A2))` `#N/A`. A first area of several cells spills as FORMULATEXT of a range does: `CHOOSE(1,(Data!A2:A3,Data!B1))` spills `=A1+1`, `#N/A` down. AREAS of the two picks is 2. A lifted INDIRECT element gives one result per element, never a spill of its own: `INDIRECT({"Data!A1:A2","Data!A3"})` spills `#N/A`, `#N/A`; `INDIRECT({"Data!A2:A3","Data!A4"})` spills `=A1+1`, `=A3*2`; `INDIRECT({"Data!A2:B2"})` is the single `=A1+1`; `INDIRECT({"Data!A2:A2"})` `=A1+1`; the column forms spill down the same way; `INDIRECT({"Data!A2","Data!A1:B2";"Data!A3:A4","Data!B1"})` spills `=A1+1`, `#N/A`; `#N/A`, `=A1*10`; and `INDIRECT(Data!C1:D1)` over cells holding `Data!A1:A2` and `Data!A3` spills `#N/A`, `#N/A`. The engine gives `#VALUE!` for each multi-cell element (`formula_texts_of_grid`) and for each multi-area pick (`several_areas`), and `#ERROR!` for the bare sheet-qualified union: 21 of the probe's 34 cells fail (T-2992).
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
Probe T-1378 (probes/subtotal-reference-operands-f, excel-graph, 2026-10-11): TAKE and DROP pick cells (SUBTOTAL reads them, AREAS 1, ISREF TRUE) or values (SUBTOTAL and AREAS spill `#VALUE!` per element, ISREF FALSE) according to the arm the IF, CHOOSE, SWITCH or IFS actually selects: `TAKE(IF(TRUE,A1:A4,SEQUENCE(4)),2)`, `TAKE(CHOOSE(1,A1:A4,SEQUENCE(4)),2)` and `TAKE(IFS(TRUE,A1:A4,FALSE,SEQUENCE(4)),2)` are cells, `CHOOSE(2,...)` and `IF(FALSE,A1:A4,SEQUENCE(4))` are values. An array selector lifts the call to values even when every arm is a range: `SUBTOTAL(9,TAKE(SWITCH({1,2},1,H1:H2,2,I1:I2),2))` and `SUBTOTAL(9,TAKE(IFS({TRUE,FALSE},H1:H2,{FALSE,TRUE},I1:I2),2))` spill a 2x2 of `#VALUE!` as `TAKE(CHOOSE({1,2},H1:H2,I1:I2),2)` does, while the IF twin `SUBTOTAL(9,TAKE(IF({TRUE,FALSE},H1:H2,I1:I2),1))` spills 1x2 (probes/subtotal-take-array-selector A20:B20, A30:B31, A40:B41, excel-graph, T-1909; implemented in `lifts_to_values`, T-1914). The array need not be the selector (probes/subtotal-take-array-selector-b, excel-graph, 2026-10-19, T-1984): an array SWITCH case value, `SUBTOTAL(9,TAKE(SWITCH(1,{1,2},H1:H2,I1:I2),2))`, spills a 2x2 of `#VALUE!` (implemented in `switch_pick`, which lifts on an array case value, T-2012). IFS with a first condition TRUE short-circuits and stays cells, `IFS(TRUE,H1:H2,{FALSE,TRUE},I1:I2)` is 7; `IFS({TRUE,FALSE},H1:H2,TRUE,I1:I2)` is a 2x2 `#VALUE!`; `IFS(FALSE,H1:H2,{TRUE,FALSE},I1:I2)` (an earlier FALSE, then an array condition) spills `#VALUE!` in column A and `#N/A` in column B, in both rows. Wrapping does not matter: the SWITCH/IFS array form under `IF(TRUE,...)`, `CHOOSE(1,...)` or `LET(x,...,x)`, and an array LET variable as the selector (SWITCH, IFS, CHOOSE), all spill a 2x2 of `#VALUE!`. A multi-cell range selector, `IF(H1:H2>3,H1:H2,I1:I2)` or `CHOOSE(J1:J2,H1:H2,I1:I2)`, spills a 2x1 (one column) of `#VALUE!`. The one-cell `INDIRECT("K1")` (TRUE) is read as a scalar condition: `SUBTOTAL(9,TAKE(IF(INDIRECT("K1"),H1:H2,I1:I2),2))` is 7 (implemented in `if_pick`, which reads a one-cell range as a scalar condition, T-2012). Every one-cell range condition is read the same way (probes/if-switch-one-cell-selector, excel-graph, 2026-10-20, T-2079): `IF(K1:K1,...)`, `IF(OFFSET(K1,0,0),...)`, `IF(INDEX(K1:K3,1),...)`, `IF(INDIRECT("$K$1"),...)` and `IF(INDIRECT("K1",TRUE),...)` under `SUBTOTAL(9,TAKE(...,2))` are all 7, and `ISREF(IF(K1:K1,H1,I1))` and `ISREF(IF(OFFSET(K1,0,0),H1,I1))` are TRUE; a one-cell condition holding text is `#VALUE!` and one holding `#N/A` is `#N/A`. SWITCH lifts only when it reaches the array case: an earlier match short-circuits and stays cells (`SWITCH(1,1,H1:H2,{1,2},I1:I2)` 7), and an earlier error case value is that error, not a lifted array (`SWITCH(1,NA(),H1:H2,{1,2},I1:I2)` and `SWITCH(1,#N/A,...)` are `#N/A`). LET variable names and LAMBDA parameters are case-insensitive (`LET(x,A1:A4,X)` reads `x`; probes/let-lambda-name-case: `LAMBDA(x,X+1)(2)` 3, `LAMBDA(Ab,aB*2)(4)` 8, `LET(f,LAMBDA(x,x+1),F(2))` 3, and a parameter of another case shadows the LET name, `LET(x,1,LAMBDA(X,x*10)(2))` 20, `LET(x,1,LET(X,5,x))` 5); an inner LET rebinding a name shadows the outer one. FORMULATEXT shows each use of a LET or LAMBDA name, called or not, in the spelling of the innermost declaration that binds it, not as typed: `LET(x,5,X*2)` reads `LET(x,5,x*2)`, `LAMBDA(Ab,aB*2)(4)` reads `LAMBDA(Ab,Ab*2)(4)`, `LET(Ab,LAMBDA(Q,q+1),aB(2))` reads `LET(Ab,LAMBDA(Q,Q+1),Ab(2))`, `LET(x,1,y,X+1,Y)` reads `LET(x,1,y,x+1,y)`, and a shadowing `LET(x,1,LET(X,5,x))` reads `LET(x,1,LET(X,5,X))` (probes/let-name-use-spelling, excel-graph, 2026-10-17, T-1720; implemented in `CallNameSpellings::apply`). A LET or LAMBDA that binds one name twice is refused in any case: Excel for the web refuses (createSession 501 `FileCorruptTryRepair`) a single-cell workbook holding `LET(x,1,x,2,x)`, `LET(x,1,X,2,x)`, `LET(é,1,É,2,é)`, `LAMBDA(x,x,x)(1,2)`, `LAMBDA(x,X,x+X)(1,2)`, `LAMBDA(ä,Ä,ä+Ä)(1,2)` or `LAMBDA(x,[X],x)(1)` (an optional parameter too) (`rustsheet-oracle recalc -b excel-graph`, 2026-10-12, T-1387; refused inputs cannot be committed), while `LET(x,1,LET(X,2,x))` loads and gives 2. The engine gives `#ERROR!` for each (a parse error for LAMBDA, the LET duplicate check). Names may start with a non-ASCII letter, and one-to-one letter pairs fold case (probes/let-name-non-ascii: `LET(é,1,É+1)` 2, `LET(Ä,2,ä*10)` 20, `LET(Ärger,5,äRGER+1)` 6, `LAMBDA(ß,ß*2)(3)` 6, `LET(é,1,LAMBDA(É,é*10)(2))` 20); a fold that changes length does not: `LET(ß,1,ẞ+1)` is 2 but `LET(ß,1,SS+1)` is `#NAME?` (probes/let-name-sharp-s, which stores the use as `_xlpm.SS`: spec2xlsx decides which names are bound with Python `str.lower()`, so a probe of a fold where Python and Excel may differ must write the `_xlpm.` prefix itself). Other scripts follow the same rules (probes/let-name-non-ascii-scripts, excel-graph, 2026-10-18, T-1431): `LET(名前,1,名前+1)` is 2; the Turkish dotless `ı` is its own letter that folds to neither `I` nor `i`, `LET(ı,1,ı+1)` is 2 and `LET(ı,1,I+1)` and `LET(ı,1,i+1)` are `#NAME?`; the letter number `Ⅻ` folds to `ⅻ` (`LET(Ⅻ,1,Ⅻ+1)` and `LET(Ⅻ,1,ⅻ+1)` are 2). Excel refuses (createSession 501 `FileCorruptTryRepair`, `rustsheet-oracle recalc -b excel-graph`) a single-cell workbook whose LET names the Turkish dotted `İ` or the circled letter `Ⓐ`, in any spelling of the use (`LET(İ,1,İ+1)`, `LET(İ,1,i+1)`, `LET(Ⓐ,1,Ⓐ+1)`, `LET(Ⓐ,1,ⓐ+1)`); refused inputs cannot be committed, and the engine accepts them (an open gap for formulas entered through the API: T-1859). The earlier refusal of `LET(é,1,É+1)` was spec2xlsx leaving a non-ASCII name without its `_xlpm.` prefix; before T-1387 the engine also rejected a name whose first letter is non-ASCII, unless it uppercased to ASCII (`ß` → `SS`). The same identifier check (`is_valid_a1_identifier`) validates defined names, so a defined name may now start with a non-ASCII letter too; probes/defined-name-non-ascii (excel-graph, T-1431) confirms it: defined names `é1` and `名前` work (`=é1+1` 6, `=É1+1` 6, `=名前+1` 8).
Tasks: T-0216, T-0235, T-0940, T-1003, T-1378, T-1387, T-1431, T-1453, T-1859, T-1984, T-2012, T-2079, T-2831, T-2992

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
A LET or a LAMBDA call whose result is its variable bound to a reference stays a reference where one is
wanted (probes/let-lambda-reference-results, excel-graph, 2026-10-19, T-1106): with K1 = "text", K2 = 7,
`SUM(LET(x,K1,x))`, `SUM(LAMBDA(x,x)(K1))` and `MAX(LAMBDA(x,x)(K1))` are 0, `SUM(LAMBDA(x,x)(K1:K2))` 7,
`COUNTA(LAMBDA(x,x)(K1))` 1, `COUNT(LET(x,K1:K2,x))` 1; `ISREF` is TRUE for `LAMBDA(x,x)(A1)`,
`LAMBDA(x,x)(A1:A3)`, `LET(x,A1,x)`, `LET(x,K1,x)`, `LET(x,A1:A3,y,x,y)` and `LET(x,A1:A3,IF(TRUE,x))`,
FALSE for `LET(x,5,x)`, `LAMBDA(x,5)(A1)` and `LET(x,A1:A3,x+0)`; `ROWS(LAMBDA(x,x)(A1:A3))` is 3. In a
cell the result is the value: `=LET(x,K1,x)` and `=LAMBDA(x,x)(K1)` are "text".
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

## Calling a call's result
Rule: a call followed directly by `(...)`, with no space, calls what the first call returned: `=LAMBDA(a,LAMBDA(b,a*b))(3)(4)`
is 12, `=LAMBDA(a,LAMBDA(b,LAMBDA(c,a*100+b*10+c)))(1)(2)(3)` is 123, and so are a parenthesized LAMBDA's call
(`=(LAMBDA(a,LAMBDA(b,a*b)))(3)(4)` is 12), a LET name's (`=LET(f,LAMBDA(a,LAMBDA(b,a*b)),f(3)(4))` is 12,
`f(1)(2)(3)` too) and a LAMBDA passed as an argument (`=LAMBDA(x,x)(LAMBDA(y,y*2))(5)` is 10). A returned value is called
as a LET-bound value is ("Calling a parenthesized operand" above): `=LAMBDA(x,x)(1)(2)` is `#NAME?`, `=LAMBDA(x,x)("a")(2)` is
0, `=LAMBDA(x,x)(1/0)(2)` is `#DIV/0!` and `=LAMBDA(x,x)(A1)(2)` is `#REF!` (an ordinary error: IFERROR catches it). A called
reference called again stays `#REF!`: `=$A$1(1)(2)`, `=$A$1:A2(1)(2)`, `=(A1)(1)(2)`. A built-in's result may be called too,
but only a built-in that can return a reference (or a LAMBDA): `=INDEX(A1:A3,1)(2)`, `=OFFSET(A1,0,0)(2)`,
`=INDIRECT("A1")(2)`, `=CHOOSE(1,A1:A3)(2)`, `=TAKE(A1:A3,1)(2)`, `=DROP(A1:A3,1)(2)`, `=TRIMRANGE(A1:A3)(2)` and
`=XLOOKUP(1,A1:A3,A1:A3)(2)` are `#REF!`;
`=IF(TRUE,LAMBDA(x,x*2))(3)`, `=CHOOSE(1,LAMBDA(x,x*2))(3)`, `=IFS(TRUE,LAMBDA(x,x*2))(3)`,
`=SWITCH(1,1,LAMBDA(x,x*2))(3)` and `=LET(f,LAMBDA(x,x*2),f)(3)` are 6; a value such a built-in returns is called as a
LET-bound value is, `=INDEX({1,2},1)(2)` `#NAME?`. FORMULATEXT prints the call bare, `=IF(TRUE,LAMBDA(x,x*2))(3)`.
Excel for the web refuses a workbook holding `=SUM(1)(2)`, `=ABS(1)(2)`, `=IFERROR(LAMBDA(x,x*2),0)(3)` or
`=IFNA(LAMBDA(x,x*2),0)(3)` (FileCorruptTryRepair, one single-formula workbook each, T-2038, T-2405), so IFERROR, though it
passes a LAMBDA through, is not callable; the engine reads these as syntax errors. With a space,
`=LAMBDA(a,LAMBDA(b,a*b))(3) (4)` is refused the same way (no intersection), and the engine reads it as a syntax error too.
`LAMBDA(x,LAMBDA(y,x+y))(1)()` gives an error the oracle could not decode (unprobed). An unknown name's call chained,
`=FOO(1)(2)`, is `#NAME?`, and a parenthesized value a function returned, `=(IF(TRUE,1))(2)`, is called like a LET-bound
value, `#NAME?`. These are exactly the built-ins an intersection accepts (`is_reference_operand`). A defined-name LAMBDA's call chains
like a LET name's: with `MYFN` = `=LAMBDA(x,LAMBDA(y,x*y))`, `=MYFN(3)(4)` is 12, and with `MYFN` = `=LAMBDA(x,x)`,
`=MYFN(1)(2)` calls the returned number, which is `#NAME?` as for a LET-bound value (T-2501).
Evidence: probes/lambda-escaping-closures H3, H8, H10; probes/chained-calls-lambda B1:B10; probes/chained-calls-ref-1..4, -6
(oracle: excel-graph, 2026-10-22, T-2038); probes/chained-calls-builtin-* (excel-graph, 2026-10-23, T-2405); probes/chained-calls-name-curried, -value (excel-graph, 2026-10-25, T-2501); the refused
forms were single-formula workbooks, not kept as cases (a refused workbook gives no values).
Status: VERIFIED (Excel for the web)
Implemented in: expressions/parser/lambda.rs (`Parser::chain_calls`, after a call and a name call), parser/operands.rs
(`parse_named_call`, after a built-in `is_reference_operand` accepts), node_frames.rs (a called value,
`Model::call_variable`), functions/logical/lambda.rs (`Model::evaluate_callee`: a called TAKE, DROP, TRIMRANGE or
XLOOKUP over a range is the range it picks, `#REF!`; probes/chained-calls-builtin-take, -drop, -trimrange, -xlookup,
T-2499).
Tasks: T-2038, T-2405, T-2499, T-2501

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

## The names in a sheet-local name's body are the workbook's, wherever it is used
Rule: a name used inside a defined name's body resolves as if from no sheet: a sheet-local body
does not see its own sheet's local names. With workbook `Rate` = 1, `Data!Rate` = 10,
`Data!DoubleRate` = `Rate*2` and workbook `TripleRate` = `Rate*3`: `=Data!DoubleRate` from Sheet1
and `=DoubleRate` from Data are both 2 (not 20), `=TripleRate` is 3 from Sheet1 and from Data,
`=Rate` from Data is 10, and `=DoubleRate` from Sheet1 is an error (`ISERROR` TRUE).
Evidence: probes/defined-name-local-formula-scope (excel-graph, 2026-10-27, T-2635; the body is
stored in the file as `Rate*2` with `localSheetId` of Data). Upstream IronCalc f8f68e27 resolves the
body in the name's sheet (20); Excel does not, so it is not ported. Ours matches. A body typed into
Name Manager on desktop (which may bind `Rate` as it is typed) is unprobed.
Status: VERIFIED (Excel for the web).
Tasks: T-2635

## A sheet-qualified name may start like a cell reference
Rule: after a sheet prefix, a cell address that more name characters follow (letters, digits,
`_` or `.`) is a name, not the cell. With workbook `Q1_Total` = 100, `Q1.Rate` = 5 and Data!Q1 =
77: `=Data!Q1_Total` is 100, `=Data!Q1` is 77, `=Sheet1!Q1_Total*2` is 200,
`=SUM(Data!Q1_Total,Data!Q1)` and `=Data!Q1_Total+Data!Q1` are 177 (Q1 a number), and `=Data!Q1.Rate` is 5. An
undefined one (`=Data!A1B`) is `#NAME?` (`ERROR.TYPE` 5). Being a workbook name reached through a
sheet prefix, Excel's FORMULATEXT shows it with the file name as the prefix (see "A
sheet-qualified workbook name is rewritten to the workbook's file name"; T-2834).
Evidence: probes/defined-name-qualified-like-reference and
probes/defined-name-qualified-like-reference-more (excel-graph, 2026-10-27, T-2635; upstream
IronCalc 2aa0e653 lexes `_` the same way). Unprobed, and read as a name by ours: a `\` or a
typographic quote after the cell, and a rejected dotted corner (`Data!A1.B2x`); a range that name
characters follow (`Data!A1:B2x`) is unprobed and stays a range then an error. Ours matches except that FORMULATEXT cell (T-2834).
Status: VERIFIED (Excel for the web).
Tasks: T-2635, T-2834

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
operand and LAMBDA call of the formula draws on the same budget, like the pair budget. A single
range argument is not counted. It is walked over every position of its used area while that is
the cheaper walk, and over its existing cells like a union's area once a far cell stretches the
used area: a single range whose clipped area holds at least 65,536 positions and more than four
per cell of its columns (`SUM(A:XFD)` with data in A1 and XFD1048576, 1.7e10 positions;
`COUNTA(A:CV)` beside one cell in CV1048576) costs the data, never its positions (T-1690,
T-2710). Below 65,536 positions the dense walk is used, so one walk visits at most 65,536 blank
positions, plus at most four per cell of its columns above that. Every single-range walk of an
aggregate argument takes this path: the dense `ClipBlocks` walk (SUM, COUNT, COUNTA, COUNTBLANK,
which counts the cells that are not blank and takes them from the area's size, LARGE, GCD and
LCM, which read any number of blanks as one 0, SUBTOTAL, AGGREGATE, ...) and the `ClipWholeLines`
walk of TEXTJOIN's 3-D path, which reads blanks as empty values: it hands each run of blank
positions between two existing cells on as a count, in row-major order, so the delimiters they
add are counted, not read. It does not draw on the budget: that walk costs less than the dense
one it replaces, so a LAMBDA repeating it (`SUM(BYROW(SEQUENCE(200),LAMBDA(r,SUM(A:Z))))` over
100,000 cells) gives the same value as the dense walk, and the change adds no `#VALUE!` of the
walk budget. A budget would not do: over filled cells a step is a cell, so it would make a
filled `SUM(A:Z)` `#VALUE!` where Excel computes it. Over
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
`MAX_AREA_WALK`, `SLOTS_PER_WALK_STEP`); functions/util.rs (`Model::for_each_position_in_area`, `Model::walk_area`, `Model::for_each_cell_or_error_in_whole_range`); functions/util/existing_walk.rs (`Model::for_each_existing_position`, `Model::for_each_indexed_position`, `Model::walks_sparsely`); sheet_data.rs (`SheetData::rows_in_column_between`, `SheetData::cells_in_columns`); reference_operators.rs (`MAX_HELD_AREAS`, `VariableStack`)
Tasks: T-0134, T-1062, T-1064, T-1065, T-1162, T-1462, T-1497, T-1502, T-1690, T-2710

## Seen-range cap
Rule: one formula evaluation may put at most 65,536 distinct rectangles on record as read empty in a
pass (`MAX_SEEN_RANGES`); a formula that reads more is `#VALUE!` ("Too many distinct ranges read").
A function walking a whole column or row (and a lookup down a column, or a SUMIFS-shaped criteria
range) visits only the used area and records the rest as one rectangle read empty, as does an
aggregate walking an explicit block that reaches at least 1,024 cells past the used area, while the
budget has room (a shorter block, or one met once the budget is short, is walked and recorded cell by
cell, which is uncapped; T-0405), and a single range of at least 65,536 positions that a far cell
leaves mostly blank (more than four per cell of its columns) is walked over its existing cells and
recorded as one rectangle, whether or not the budget has room ("Union area cap", T-1690, T-2710):
its dense walk would read every position and record each blank one, unbounded in time and memory
(`SUM(A:XFD)` beside one far cell is 1.7e10 positions), so past the cap such a formula is
`#VALUE!` (DEVIATION; `=SUM(MAP(SEQUENCE(65536),LAMBDA(r,SUM(OFFSET(Sheet2!1:1,r,0)))))+SUM(A:J)`
beside 1 in J1048576 is `#VALUE!`, one row fewer is 1). Smaller blocks alone never make a formula
`#VALUE!`. A bounded single-column operand of a tailed array argument (`SUM(A2:A70000*1)`, SPEC-0020
Amendment 1, T-0491) records its rows past the used area as one such rectangle while the budget has
room, where the materialized reading records them cell by cell, so a formula at the cap reaches it one
rectangle sooner (DEVIATION, directive 5; never with the budget short, which reads it the ordinary
way). Every area of
a union is recorded whole, whatever its size, as is a multi-cell range a MAP or SCAN LAMBDA returns
for an element (read over its existing cells only, since the whole call is `#CALC!` whatever they hold,
arrays.md "MAP and SCAN with a LAMBDA that returns a multi-cell range"; T-1769); which every spill commit of the pass is checked
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
(`Model::record_seen_empty_range`); functions/logical/bycol_byrow.rs (`Model::lambda_result_value`);
functions/util/existing_walk.rs (`Model::walks_sparsely`)
Tasks: T-1463, T-1769, T-2710, T-0491

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


## TAKE, DROP and TRIMRANGE over XLOOKUP pick its cells (T-1998)
Rule: an XLOOKUP that returns cells of a range (a scalar lookup value, a range `return_array` or `if_not_found`)
stays a reference as the first argument of TAKE, DROP and TRIMRANGE, also when it is the arm a scalar IF or CHOOSE
picks, a LET body or under a nested TAKE or TRIMRANGE; SUBTOTAL, AREAS and ISREF then read the cells. With H1:H2 =
3,4, I1:I2 = 6,8, J1:J2 = 1,2: `SUBTOTAL(9,TAKE(XLOOKUP(1,J1:J2,H1:I2),1,1))` is 3, `SUBTOTAL(9,DROP(..,0,1))` 6,
`SUBTOTAL(9,TRIMRANGE(XLOOKUP(1,J1:J2,H1:I2)))` 9, `AREAS(TAKE(..,1))`, `AREAS(DROP(..))` and `AREAS(TRIMRANGE(..))`
1 (no spill), `ISREF(TAKE(..,1))` TRUE; the same through `IF(TRUE,..)`, `IF(FALSE,1,..)`, `CHOOSE(1,..)`,
`CHOOSE(2,H1:H2,..)`, `IF(TRUE,CHOOSE(1,..))`, `LET(v,1,XLOOKUP(..))`, `LET(v,XLOOKUP(..),v)`,
`TAKE(TAKE(..,1,2),1,1)` and `DROP(TRIMRANGE(..),0,1)`; a nested XLOOKUP lookup value
(`XLOOKUP(XLOOKUP(1,J1:J2,J1:J2),J1:J2,H1:H2)`), a literal lookup array and a range `if_not_found` keep the
reference; `SUBTOTAL(9,TAKE(TAKE(H1:I2,1),1,1))` is 3. Values instead, so SUBTOTAL is `#VALUE!` per element and
ISREF FALSE: a lifting lookup value (`TAKE(XLOOKUP({1;2},J1:J2,H1:H2),1)` is one `#VALUE!`,
`TRIMRANGE(XLOOKUP({1;2},..))` spills two), a literal `return_array`, and `IFERROR(XLOOKUP(..),0)`. A lifting
argument next to a reference one gives one `#VALUE!`, not a spill
(`SUBTOTAL(9,TAKE(XLOOKUP({1;2},J1:J2,H1:H2),1),TAKE(XLOOKUP(2,J1:J2,H1:H2),1))`, A94), and
`SUM(TAKE(XLOOKUP(1,J1:J2,H1:I2),1))` is 9 (A90). The same holds directly below SUBTOTAL and AREAS (T-2009, probes/subtotal-if-choose-over-xlookup, excel-graph,
2026-10-19): `SUBTOTAL(9,IF(TRUE,XLOOKUP(1,J1:J2,H1:I2)))`, `SUBTOTAL(9,CHOOSE(1,..))`, `SUBTOTAL(9,LET(v,1,..))`,
`SUBTOTAL(9,IF(FALSE,1,..))`, `SUBTOTAL(9,IF(TRUE,CHOOSE(1,..)))` and `SUBTOTAL(9,IF(TRUE,TAKE(H1:I2,1)))` are 9 with
no spill, as is the LET binding `SUBTOTAL(9,LET(v,XLOOKUP(..),v))` (A22); `AREAS(IF(TRUE,..))`,
`AREAS(CHOOSE(2,H1,..))` and `AREAS(LET(v,1,..))` are 1 (A10-A42, A86); a lifting lookup value or a literal return
array under the IF still spills `#VALUE!` (A46:A47, A50:B50). Consumers that tell cells from values agree
(probes/subtotal-if-choose-over-xlookup-b, excel-graph, 2026-10-19): with K1 = "x", L1 = TRUE,
`AVERAGEA(IF(TRUE,XLOOKUP(1,J1:J2,K1:L2)))`, `AVERAGEA(LET(v,1,..))` and the direct form are 0.5,
`AVERAGEA(IF(TRUE,..,0),5)` 2, `MAXA(CHOOSE(1,..))` 1, `MINA(IF(FALSE,1,..))` 0,
`AVERAGEA(IF(TRUE,TAKE(K1:L2,1)))` 0.5; `IMSUM(IF(TRUE,XLOOKUP(1,J1:J2,M1:N2)))` with M1 = "1+i", N1 blank is "1+i";
`AND(BYROW(IF(TRUE,XLOOKUP(1,J1:J2,H1:I2)),LAMBDA(r,ISREF(r))))` and `LET(v,IF(TRUE,XLOOKUP(..)),ISREF(v))` are TRUE
and `LET(v,CHOOSE(1,XLOOKUP(..)),SUBTOTAL(9,v))` 9 (A10-A62). The MINA (A22) and IMSUM (A34) cells give the same
result over values, so they do not tell a reference from values; the others do. Implemented in
`Model::picked_reference_arg_reading` (functions/util.rs). IFS and SWITCH pick the same way (T-2014):
`SUBTOTAL(9,IFS(TRUE,XLOOKUP(..)))` and `SUBTOTAL(9,SWITCH(1,1,XLOOKUP(..)))` are 9 (A78, A82) and
`ISREF(IF(TRUE,XLOOKUP(..)))` and `ISREF(CHOOSE(1,XLOOKUP(..)))` TRUE (A90, A94). The value IFS or SWITCH chooses is
read in reference context as IF's branch is, a union included (probes/subtotal-ifs-switch-reference-picks,
excel-graph, 2026-10-19): `AREAS(IFS(TRUE,XLOOKUP(..)))` and `AREAS(SWITCH(1,1,..))` are 1, `ISREF` of either, of
`IFS(TRUE,H1)`, `SWITCH(1,1,H1)`, `IFS(TRUE,(H1,I2))`, `LET(v,1,XLOOKUP(..))` and `IF(TRUE,TAKE(H1:I2,1))` TRUE,
`ISREF(IFS(TRUE,1))` FALSE; `SUBTOTAL(9,IFS(TRUE,(H1,I2)))`, `SUBTOTAL(9,SWITCH(1,1,(H1,I2)))` and
`SUM(IFS(TRUE,(H1,I2)))` 11, `AREAS(IFS(TRUE,(H1,I2)))` and `AREAS(SWITCH(2,1,H1,(H1,I2)))` (the default) 2; a later
TRUE condition (`IFS(FALSE,1,TRUE,TAKE(H1:I2,1))`), SWITCH's default and an IF over IFS or IFS over IF are 9; no match
is `#N/A`, and a lifting lookup value or an array condition spills `#VALUE!` (subtotal-ifs-switch-reference-picks
A10-A110; the spills A66:A67, A70:B70). With K1 = "x", L1 = TRUE, `SUM(IFS(TRUE,XLOOKUP(1,J1:J2,K1:L2)),5)` and the
SWITCH form are 5 and `COUNTA(IFS(TRUE,..))` 2 (A82, A86, A106), as over values, so they do not tell the two apart.
TAKE, DROP and TRIMRANGE read IFS and SWITCH the same way (probes/take-drop-over-ifs-switch-picks, excel-graph,
2026-10-19; T-2071): `SUBTOTAL(9,TAKE(IFS(TRUE,XLOOKUP(1,J1:J2,H1:I2)),1,1))` and the SWITCH form are 3,
`SUBTOTAL(9,DROP(IFS(TRUE,..),0,1))` and `DROP(SWITCH(2,1,H1,..),0,1)` (the default) 6, `TRIMRANGE` of either 9,
`AREAS(TAKE(IFS(TRUE,..),1))` 1, `ISREF(TAKE(SWITCH(1,1,..),1))` TRUE, `TAKE(IF(TRUE,IFS(TRUE,..)),1,1)` and
`TAKE(LET(v,1,SWITCH(v,1,..)),1,1)` 3; `TAKE(IFS(FALSE,1),1,1)` is `#N/A` and a lifting lookup value `#VALUE!`
(A10-A50, A82, A86). ISREF agrees: `ISREF(IF(TRUE,DROP(H1:I2,1)))`, `ISREF(IF(TRUE,TRIMRANGE(H1:I2)))`,
`ISREF(IFS(TRUE,DROP(H1:I2,1)))`, `ISREF((H1,XLOOKUP(..)))`, `ISREF(LAMBDA(x,x)(XLOOKUP(..)))` and
`ISREF(LAMBDA(x,IF(TRUE,XLOOKUP(..)))(1))` are TRUE, `ISREF(IFS({TRUE,FALSE},H1,TRUE,I1))` (an IFS that lifts) FALSE
(A54-A78). A LAMBDA body is read the same way wherever its call stands for a reference (probes/lambda-body-picked-references,
excel-graph, 2026-10-19; T-2075): a body that is XLOOKUP, TAKE or DROP over a range, or one the arm an IF, CHOOSE, IFS,
SWITCH or LET picks, makes `ISREF(LAMBDA(x,..)(1))` TRUE (A10-A34); with K1:K2 = text "5","7",
`SUM(LAMBDA(x,IF(TRUE,XLOOKUP(1,J1:J2,K1:K2)))(1))` is 0, the cell's text skipped (A38, A42, A62);
`SUBTOTAL(9,LAMBDA(x,IF(TRUE,XLOOKUP(..)))(1))` 9, AREAS 1 and COUNTA over a blank cell 0 (A46-A58); and as the
function of GROUPBY or PIVOTBY each such body is a reference result, so every aggregate is `#VALUE!` (A66-A78, A86, A90),
while `IF(TRUE,XLOOKUP(..))+0` is a value, 3 (A82). Implemented in: `LambdaCaller::keeping_references` reads the
body with `Model::evaluate_picked_reference_arg` (functions/logical/lambda.rs).
SUM, COUNT and COUNTA over `IF(TRUE,XLOOKUP(1,J1:J2,K1:L2))` with K1 = "x", L1 = TRUE give 5 (with a 5), 1 and 2 (A54-A74), which an array of those values gives too, so they do not
tell a reference from values.
Evidence: probes/take-drop-trimrange-over-xlookup A10-A94, probes/take-drop-trimrange-over-xlookup-b A10-A62
(excel-graph, 2026-10-19).
Implemented in: `Model::evaluate_picked_source` (functions/lookup_and_reference/drop_take.rs), used by
`take_drop_impl` and `trimrange_impl` in their reference form. Static analysis still counts a nested XLOOKUP as
lifting (`lifts_to_values`), which leaves the shape to run time.
Status: VERIFIED (excel-graph)
Tasks: T-1998, T-2009, T-2014, T-2071, T-2075

## SUBTOTAL lifts over an array function_num
Rule: an array or multi-cell range function_num lifts: one SUBTOTAL per element, spilling in its shape.
`SUBTOTAL({9,1},A1:A3)` over 1, 2, 3 spills 6, 2 (`INDEX(..,1,2)` is 2, `ISERROR` is FALSE), `ROWS(SUBTOTAL({9;1},A1:A3))`
is 2, and `SUBTOTAL(TEXTSPLIT("9,1",","),A1:A3)` spills 6, 2 (text numbers are read as numbers). A 1x1 array is the plain
value (`SUBTOTAL({9},..)`, `SUBTOTAL(SEQUENCE(1,1,9),..)`, `SUBTOTAL(9+{0},..)` are 6). An invalid element is `#VALUE!`
(`{"x"}`, `{12}`). A vertical array or a range function_num (`{9;109}`, `B1:B2`) written in a cell with no room below
is `#SPILL!`, so it spills too. A one-cell reference is its value (`SUBTOTAL(B1,..)` with B1 = 9 is 6).
Evidence: probes/subtotal-array-function-num A1-A14 (excel-graph, 2026-10-18, T-1361); gen-m9/d5bd4272655c5211 Calc!A14
`SUBTOTAL(TEXTSPLIT(..,C11:E11,"-"),C1:E1)` is `#VALUE!` (inferred: its TEXTSPLIT elements are not valid function numbers).
Implemented in: `Model::fn_subtotal` (crates/rustsheet-engine/src/functions/subtotal.rs), `subtotal_static_result`.
Status: VERIFIED (web)
Tasks: T-1868

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
Rule: T, like N, reads a multi-cell range as its top-left cell, in legacy and dynamic formulas
alike, with no implicit intersection and no lift: with A1:A3 = "x", 5, "y", `T(A1:A3)` is "x" from
a row outside the range (F10 legacy, F12 dynamic) and from A3's row (G3); with B1:B3 = 5, "ab", "c",
`T(B1:B3)` is "" from row 2 too (F2 legacy, J2 dynamic), not the intersected "ab". An error top-left
is that error (`T(E1:E2)` with E1 `=1/0` is `#DIV/0!`, J10/J12), a row range reads its left cell
(`T(G1:H1)` "p", L10). A legacy formula shows no `@` (FORMULATEXT `=T(A1:A3)`), nor over an OFFSET
or a literal-condition IF of the range (`T(OFFSET(A1:A3,0,0))`, `T(IF(1,A1:A3))` "x", P10, Q10),
while an operator is a value and gets one (`=T(@A1:A3&"")`, `#VALUE!` off the rows, R10). N gives
the same picture (H2/H10/H12 7, I10 0 for text top-left, K10 `#DIV/0!`). Over an array constant a
legacy formula adds the `@` (`=T(@{"a","b"})` "a", `=N(@{5,6})` 5, M10/O10) and a dynamic one
spills (M14:N14, M16:N16).
Evidence: probes/t-n-range-top-left (excel-graph, 2026-10-17, T-0985); probes/t-array-lift S!E7,
probes/text-argument-modes F9; poi/FormulaEvalTestData_Copy.xlsx (Excel-cached, Excel 15)
EverythingTests!M1352, N1352, O1352 (`T(H12:J12)` from column M, `T(L7:N7)`, `T(J1351:J1353)`: all "").
Status: VERIFIED. Not implemented: the legacy `@` over an array constant under N and T (M20, O20
lack it; M10 spills into N10, and O10 is `#SPILL!`), T-1717.
Implemented in: functions/text/t_function.rs (`Model::fn_t`);
expressions/parser/static_analysis/legacy.rs (`reads_reference_whole`)
Tasks: T-0985, T-1728

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
index, so it counts the picked cell (1) (T-1507). The CHOOSE grids (H20, H34) spill 2x2 since T-1508 (lookup.md, "CHOOSE
over an index array broadcasts its value arguments").
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
and, plain (excel-graph, T-1333, probes/implicit-intersection-one-row-plain): `=@Data!B2:C2` is Data!B2 in
another sheet's B2 and B5 (column wins, the row is irrelevant), Data!C2 in C2 and C5, and `#VALUE!` in E2 (outside
the columns). The legacy form `=Data!B2:C2` (no `@`, legacy formula) gives the same values. On the same sheet
`=@B10:C10` is B10's value in B12 and C10's in C12, `#VALUE!` in E12 and in E10 (row 10, outside B:C); the legacy
form agrees.
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
Ours: FORMULATEXT drops the groups in the reference-only cells too (T-1340); `ISREF(LET(x,A1:A3,x))` (grouped or not) is TRUE as in Excel (T-1106).
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

`=ISREF(@INDEX(B1:B3,2))` is TRUE (probes/let-bind-implicit-index H10, excel-graph 2026-10-13). Fixed by T-1506: `is_reference` has an `ImplicitIntersection(INDEX)` arm. `@XLOOKUP` and `@TAKE` keep the reference the same way (T-1509): `ISREF(@XLOOKUP(2,A1:A3,B1:B3))` and `ISREF(@TAKE(B1:B3,1))` are TRUE, also LET-bound (probes/isref-implicit-xlookup-take H1, H3, H5, H6). An `@` whose intersection fails (formula row outside the picked rows) or a failing INDEX is not a reference: `ISREF(@TAKE(B1:B3,2))`, `ISREF(@INDEX(B1:C3,0,1))` and `ISREF(@INDEX(B1:B3,9))` are FALSE (H7:H11), plain or LET-bound. `@XLOOKUP`/`@TAKE` also keep the reference as the operand of `#`/ANCHORARRAY (`ANCHORARRAY(@TAKE(D1:D3,1))` spills D1#) and as LOOKUP's `result_vector` (a one-cell reference is read on past to the right, `LOOKUP(3,A1:A3,@TAKE(F1:F3,1))` reads H1; an `@` that fails to intersect is `#VALUE!`); as a lookup index argument the value and the reference read the same (probes/anchorarray-lookup-implicit-xlookup-take, excel-graph 2026-10-14, T-1527).
Evidence strength: excel-graph oracle, 11 + 17 cells (T-1527); the postfix `#` spelling of the ANCHORARRAY case is inferred, not probed. Status: VERIFIED (web).
Tasks: T-1506, T-1509, T-1527

## Legacy formulas intersect the ranges in value arguments
Rule: in a legacy (non-dynamic, no `cm`) formula a range is implicitly intersected wherever Excel
evaluates it as a value, not only at the formula's root (FORMULATEXT of these forms is unprobed;
for `=ABS(A1:A2)` it shows the `@`, probes/multiarea-at-legacy).
With B1:B6 = 1,2,3,1,2,3, each formula on row 2 (on the range's rows, B2 = 2) and row 10 (off):
- a scalar argument intersects arithmetic and comparisons over a range, not only a bare range:
  `SUM(IFERROR(B1:B6*1,100))` 2 / 100, `SUM(IF(A1:A6="a",1,0))` 1-or-0 / `#VALUE!`
  (array-context-legacy-sumproduct R1, R10), `SUM(IF(TRUE,B1:B6+0))` 2 / `#VALUE!`;
- an aggregate's arithmetic argument is a value too, while a bare or parenthesized range stays a
  reference: `SUM(B1:B6+0)` 2 / `#VALUE!`, `SUM(-B1:B6)` -2 / `#VALUE!`, `AVERAGE`, `MAX`, `MEDIAN`,
  `AVERAGEA(B1:B6*1)` 2 / `#VALUE!`, `LARGE(B1:B6*1,1)` 2 / `#VALUE!`, `COUNT(B1:B6*1)` 1 / 0,
  `COUNTA(B1:B6*1)` 1 / 1, `SUM(B1:B6,B1:B6*1)` 14 / `#VALUE!`, but `SUM(B1:B6)` and `SUM((B1:B6))` 12 on
  both rows; MMULT, INDEX and LOOKUP evaluate such an argument as an array (criteria.md, T-0286);
- TYPE's argument is intersected whole: `TYPE(B1:B6)`, `TYPE((B1:B6))` and `TYPE(B1:B6*1)` are 1 on
  row 2 and 16 on row 10, `TYPE(B1:D1)` (a row, read from column G) 16 — where the dynamic
  `TYPE(A2:C2)` is 64 (information.md);
- under SUMPRODUCT (an array context), the value of IFERROR and IFNA and the condition of IF still
  intersect, at any depth: `SUMPRODUCT(IFERROR(B1:B6*1,100))` and `SUMPRODUCT(IFERROR(B1:B6,100))`
  2 / 100, `SUMPRODUCT(IFNA(B1:B6*1,100))` 2 / `#VALUE!`, `SUMPRODUCT(IFERROR(1/(B1:B6-2),100))`
  100 / 100, `SUMPRODUCT(ABS(IF(B1:B6>1,B1:B6,0)))` 12 / `#VALUE!` (the condition intersects, the
  value branch stays the whole range), while `SUMPRODUCT(B1:B6+0)` is 12 on both rows;
- an Analysis ToolPak function's scalar argument is read like an aggregate's argument (probed for EDATE's and YEARFRAC's
  first argument). A bare range gets no `@` and is
  `#VALUE!` on any row, and an operation on it is intersected: legacy `=EDATE(A1:A60,1)` keeps its text and is `#VALUE!`,
  and `=EDATE(A1:A60+0,1)` shows `=EDATE(@A1:A60+0,1)` (probes/scalar-lift-shapes-c, T-1767; arrays.md "Scalar
  arguments lift: shapes, element errors, and the Analysis ToolPak range rule");
- the reference functions are unchanged: `ROWS(B1:B6)` 6, `SUBTOTAL(9,B1:B6)` and
  `AGGREGATE(9,6,B1:B6)` 12, `AVERAGEA(B1:D1)` (C1 "x", D1 TRUE) 0.667 on both rows.
The same holds for the other aggregates (probes/legacy-value-argument-intersection-b, excel-graph,
2026-10-14, T-1551): over `B1:B6*1`, whose only on-row value is 2, every one gives its row-2 value and `#VALUE!` on row 10:
MIN, MINA, MAXA, PRODUCT, GEOMEAN, HARMEAN, SMALL, PERCENTILE.INC, QUARTILE.INC, GCD and LCM 2; SUMSQ 4;
STDEV.P, VAR.P, DEVSQ and AVEDEV 0; STDEV.S, VAR.S, KURT and SKEW `#DIV/0!`; CONCAT and TEXTJOIN (third
argument) "2"; a parenthesized operation, `SUM((B1:B6*1))`, a comparison,
`SUM(B1:B6>1)` 1 / `#VALUE!`, and a concatenation, `SUM(B1:B6&"")` 2 / `#VALUE!`, intersect too, and
`SUMPRODUCT(ABS(IFERROR(B1:B6,100)))` is 2 / 100. The exception is `MODE.SNGL(B1:B6*1)`: 1 on both rows (it
evaluates the argument as an array). `MAXA`/`MINA(XLOOKUP("a",A1:A6,A1:A6))` over a text cell is 0.
The compatibility and remaining statistics aggregates intersect too (probes/legacy-value-argument-intersection-c, excel-graph,
2026-10-14, T-1556): over `B1:B6*1`, STDEV, VAR, STDEVA and VARA `#DIV/0!` on row 2; STDEVP, VARP, STDEVPA and VARPA 0;
PERCENTILE, QUARTILE, PERCENTILE.EXC 2; QUARTILE.EXC `#NUM!`; PERCENTRANK and PERCENTRANK.INC 1 (`PERCENTRANK(B1:B6*1,2)`);
every one is `#VALUE!` on row 10. MODE and MODE.MULT are 1 on both rows (an array argument, like MODE.SNGL). Excel for the web
refuses to open a workbook holding `RANK(2,B1:B6*1)` (RANK, RANK.EQ, RANK.AVG take a reference) or `COUNTBLANK(B1:B6*1)`
(`FileCorruptTryRepair`), so those are unprobed.
`FORMULATEXT` of a legacy `SUM(B1:B6+0)` is `=SUM(@B1:B6+0)`. Dynamic `AVERAGEA({1,"a",TRUE})` is 0.5 (an array's booleans are skipped, statistics.md
"AVERAGEA, MAXA and MINA over an array", T-1555) and `AVERAGEA("1/2/2020",1)` is 21916.5.
A range of a linked workbook is intersected the same way, reading the cached cell
(fixtures/external-link-cached-ranges Refs!B38, B44, B56, B62; "Cached values of a linked workbook").
Evidence: probes/legacy-value-argument-intersection, probes/legacy-type-intersection (excel-graph,
2026-10-14, T-1526), probes/array-context-legacy-sumproduct (T-0286).
Engine: `add_implicit_intersection` (parser/static_analysis.rs) adds the automatic `@` at import
and in `legacy_formula_text`: a scalar argument whose static shape is not a scalar, the arithmetic
argument, parenthesized or not, of the probed aggregates (`legacy_value_operand`: SUM, AVERAGE, AVERAGEA,
MAX, COUNT, COUNTA, LARGE, MEDIAN and the T-1551 list above), TYPE's argument (`is_scalar_context`), a range of another workbook,
and under SUMPRODUCT IF's condition and IFERROR's and IFNA's value
(`for_each_array_context_condition`); `remove_redundant_implicit_intersection` mirrors it so the
export drops exactly those `@`s. Unprobed: the arguments after the first of CONCAT, GCD and LCM and TEXTJOIN's delimiter and ignore-empty (the engine
intersects every argument of these functions), the arithmetic argument of the compatibility
aggregates RANK, RANK.EQ, RANK.AVG, COUNTBLANK and PERCENTRANK.EXC (T-1556; the others are in `legacy_value_operand`), IFERROR/IFNA nested below another
function under SUMPRODUCT (intersected), a nested aggregate under SUMPRODUCT, and the other control functions
(CHOOSE's index under SUMPRODUCT lifts, T-0294).
Status: VERIFIED
Tasks: T-0293, T-1526, T-1551, T-1555, T-1556

## Legacy MATCH, VLOOKUP and HLOOKUP intersect an operation in their lookup table (T-1862)
Rule: in a legacy (no `cm`) formula, arithmetic, a comparison or a concatenation over a range in
MATCH's `lookup_array` or the table of VLOOKUP and HLOOKUP is a value, intersected like an
aggregate's arithmetic argument, so the lookup then searches one value ("MATCH, VLOOKUP and
HLOOKUP over a single value", lookup.md). With A1:A6 = a,b,a,c,b,a and B1:B6 = 1,2,3,1,2,3, on row
2 (on the ranges' rows) / row 10 (off them):
- `=MATCH(2,B1:B6*1,0)` shows as `=MATCH(2,@B1:B6*1,0)`: `#N/A` / `#VALUE!`; so do
  `=MATCH(2,B1:B6*1)`, `=MATCH(2,-B1:B6*-1,-1)` and `=MATCH(TRUE,B1:B6>1,0)`;
- the corpus INDEX/MATCH idiom, typed without Ctrl+Shift+Enter: `=MATCH(1,(A1:A6="b")*(B1:B6=2),0)`
  shows as `=MATCH(1,(@A1:A6="b")*(@B1:B6=2),0)`, `#N/A` / `#VALUE!`, and so does its INDEX;
  `=MATCH(1,(A2=A3:A4)*1,0)` is `#VALUE!` on row 2 too (A3:A4 misses it; corpus-bench/49300);
- `=MATCH("b",A1:A6&"",0)` `=MATCH("b",@A1:A6&"",0)`: `#VALUE!` on both rows (text is no table);
- `=VLOOKUP(2,B1:B6*1,1,FALSE)` `=VLOOKUP(2,@B1:B6*1,1,FALSE)` and HLOOKUP: `#N/A` / `#VALUE!`;
- unchanged: a bare or parenthesized range (`=MATCH(2,B1:B6,0)`, `=MATCH(2,(B1:B6),0)` 2 / 2), the
  `INDEX(…,0)` workaround (`=MATCH(1,INDEX((A1:A6="b")*(B1:B6=2),0),0)` 2 / 2), an array constant
  operand (`=MATCH(2,{1,2,3}*1,0)` 2 / 2) and XMATCH, a dynamic-array function
  (`=XMATCH(2,B1:B6*1)` 2 / 2). In `=MATCH(2,IF(B1:B6>0,B1:B6),0)` IF's condition is intersected and
  its arm is not (`=MATCH(2,IF(@B1:B6>0,B1:B6),0)`, 2 / `#VALUE!`; next entry).
Evidence: probes/legacy-match-lookup-array-intersection (excel-graph, 2026-10-18, T-1862),
corpus-bench/49300 Sheet1!C2.
Engine: `legacy_value_operand` (`is_lookup_table`, static_analysis/legacy.rs); IF's arm in the last
form: `is_reference_arm` (T-1935, next entry).
Status: VERIFIED
Tasks: T-1862, T-1935

## Legacy IF passes a range arm whole (T-1935)
Rule: in a legacy (no `cm`) formula an IF arm that is a bare or parenthesized range is not
intersected: IF returns the range, and the `@` goes on the IF call where its result is a value,
none where it is a range argument. With A1:A6 = a,b,a,c,b,a, B1:B6 = 1,2,3,1,2,3, D1:F1 = 1,2,3, on
row 2 (on the ranges' rows) / row 10 (off them):
- in a range argument, no `@` and the whole range: `=SUM(IF(TRUE,B1:B6))` 12 / 12, `COUNT`,
  `COUNTA` 6 / 6, `MAX` 3 / 3, `=SUM(IF(TRUE,(B1:B6)))` 12 / 12, `=SUM(IF(B2>0,B1:B6))` 12 / 12,
  `=MATCH(3,IF(TRUE,B1:B6),0)`, `=MATCH(3,IF(FALSE,0,B1:B6),0)`,
  `=MATCH(3,IF(TRUE,IF(TRUE,B1:B6)),0)`, `=VLOOKUP(3,IF(TRUE,B1:B6),1,FALSE)`,
  `=HLOOKUP(3,IF(TRUE,D1:F1),1,FALSE)`, `=LOOKUP(3,IF(TRUE,B1:B6))` and `=INDEX(IF(TRUE,B1:B6),3)`
  3 / 3, `=ROWS(IF(TRUE,B1:B6))` 6 / 6, `=SUMIF(IF(TRUE,A1:A6),"a",B1:B6)` 7 / 7 (a reference:
  SUMIF takes no array there), `=MATCH(3,IF(TRUE,{1,2,3}),0)` 3 / 3;
- a range condition is still intersected: `=MATCH(3,IF(@B1:B6>0,B1:B6),0)` 3 / `#VALUE!`,
  `=SUM(IF(@B1:B6>0,B1:B6))` 12 / `#VALUE!`;
- in a value position the `@` is on the call: `=IF(TRUE,B1:B6)` shows as `=@IF(TRUE,B1:B6)` and
  `=ABS(IF(TRUE,B1:B6))` as `=ABS(@IF(TRUE,B1:B6))`, 2 / `#VALUE!`;
- an operation in the arm, and IFERROR's value, are values as before:
  `=MATCH(3,IF(TRUE,@B1:B6+0),0)` `#N/A` / `#VALUE!`, `=MATCH(3,IFERROR(@B1:B6,0),0)` `#N/A` / `#N/A`;
  CHOOSE's arm stays whole (`=MATCH(3,CHOOSE(1,B1:B6),0)` 3 / 3).
- read as a value anywhere else, the IF is intersected whole, `@` on the call, also under a range
  argument: `=SUM(IF(TRUE,B1:B6)+0)` shows as `=SUM(@IF(TRUE,B1:B6)+0)` 2 / `#VALUE!`, and so do
  `=SUM(-@IF(TRUE,B1:B6))`, `=SUM(@IF(TRUE,B1:B6)>1)`, `=COUNT(@IF(TRUE,B1:B6)*1)` (1 / 0),
  `=MAX(@IF(TRUE,B1:B6)&"")`, `=MATCH(3,@IF(TRUE,B1:B6)*1,0)` (`#N/A` / `#VALUE!`),
  `=SUM(IF(TRUE,@IF(TRUE,B1:B6)+0))`, VLOOKUP's `col_index_num` (`=VLOOKUP(2,B1:C6,@IF(TRUE,B1:B6),FALSE)`)
  and `=@IF(TRUE,B1:B6)+1`; a bare-range condition at the root gives `=@IF(@B1:B6,B1:B6,C1:C6)`;
- not intersected: `=SUMPRODUCT(IF(TRUE,B1:B6)+0)` 12 / 12 (an array context), `=SUM((IF(TRUE,B1:B6)))`
  12 / 12, `=AND(IF(TRUE,B1:B6))` TRUE / TRUE, and INDEX of an IF with mixed arms,
  `=INDEX(IF(TRUE,B1:B6,5),3)`, 3 / 3 with no `@`.
Evidence: probes/legacy-if-arm-range-argument and -b (excel-graph, 2026-10-19, T-1935),
probes/legacy-match-lookup-array-intersection J2, Y2.
Engine: `is_reference_arm`, `is_range_arm`, `returns_range_arm` (static_analysis/if_arm.rs; moved out of legacy.rs, T-1969).
Other reference arms pass whole the same way (T-1969; B1:B6 = 1,2,3,1,2,3, row 2 / row 10, Rng = B1:B6):
`=SUM(IF(TRUE,Rng))`, `=SUM(IF(TRUE,OFFSET(B1,0,0,6)))`, `=SUM(IF(TRUE,INDEX(B1:C6,0,1)))` and
`=SUM(IF(TRUE,B1:INDEX(B1:B6,6)))` are 12 / 12 with no `@`. IFS's value arms too:
`=SUM(IFS(TRUE,B1:B6))` 12 / 12, `=MATCH(3,IFS(TRUE,B1:B6),0)` 3 / 3, and at the root
`=IFS(TRUE,B1:B6)` shows as `=@IFS(TRUE,B1:B6)` (2 / `#VALUE!`).
Value contexts hold the same way (probes/legacy-if-arm-other-references K-T, excel-graph): `=IF(TRUE,OFFSET(B1,0,0,6))`
shows `=@IF(TRUE,OFFSET(B1,0,0,6))`, `=ABS(IFS(TRUE,B1:B6))` `=ABS(@IFS(TRUE,B1:B6))`,
`=INDEX(IF(TRUE,OFFSET(B1,0,0,6)),3)` is 3. A single-cell INDEX arm, `=IF(TRUE,INDEX(B1:B6,3))`, shows no
`@` at all in Excel; ours matches (`returns_single_cell_index`, T-2326) for exactly that shape: an IF whose value arms are all `INDEX(one-column range, literal non-zero row)`. UNVERIFIED, so still `@`: IFS, a 2-D range (two-argument INDEX gives a whole row), a computed row (`INDEX(B1:B6,1+2)`), mixed arms.
Also in Excel (same probe, our engine passes): `=IF(TRUE,OFFSET(B1,0,0))` (one cell, no resize) shows no `@` and
`=SUM(IF(TRUE,OFFSET(B1,0,0)))` is 1 (S, T); `=IF(TRUE,Rng)` shows `=@IF(TRUE,Rng)` (R);
`=IF(TRUE,B1:INDEX(B1:B6,6))` shows `=@IF(...)` (N); `=SUM(IF(TRUE,INDEX(B1:B6,3)))` is 3 with no `@` (P).
Excel drops the `@` for both one-cell arms (M12, S12).
Evidence: probes/legacy-if-arm-other-references (excel-graph, 2026-10-21, T-1969).
Status: VERIFIED
Tasks: T-1935, T-1969, T-2326

## Legacy formulas put `@` on array-valued calls and array constants (T-1713)
Rule: a legacy (non-array) formula reads an array-valued call or array constant in a value
position, and at the top level, through an implicit `@` that FORMULATEXT shows: `=IF(1,{2;3})` is
`=@IF(1,{2;3})` (2), `=ABS(IF(1,{-2;3}))` `=ABS(@IF(1,{-2;3}))` (2), `=LEN(IF(1,{"ab";"c"}))` 2,
`=N({2;3})` `=N(@{2;3})` (2), `=ABS({-2;3})` 2, `=ISNUMBER(IF(1,{2;3}))` TRUE, `=-IF(1,{2;3})`
`=-@IF(1,{2;3})` (-2), `=N(TRANSPOSE({2,3}))` `=N(@TRANSPOSE({2,3}))` (2). Each dynamic twin spills
(`=N(IF(1,{2,3}))` is `#SPILL!` there, its spill range blocked).
Exceptions: a top-level SWITCH gets no `@` and the legacy cell spills (T-1847, below). N and T over `IF(...)` get no `@`, and the legacy cell is `#VALUE!` (`=N(IF(1,{2;3}))`,
`=N(IF(1,{2,3}))`, `=T(IF(1,{"a";"b"}))`), while `=N(IF(1,2))` is 2. N and T take a reference or a
value, and IF can return a reference: CHOOSE, INDEX and IFS behave the same, see "Legacy N and T over
a reference-returning call" below.
Under a vector argument no `@` is added: `=SUM(N({1,2}))` is 3 (probes/n-array-lift A3). In IF's
arms the `@` goes on the IF, not inside it: `=@IF(1,N(IF(1,{2;3})))` (probes/legacy-n-t-reference-args
P20).
Evidence: probes/legacy-array-result (excel-graph, 2026-10-16), probes/n-array-lift D4,
probes/t-n-range-top-left M, O (`=T(@{"a","b"})`, `=N(@{5,6})`).
Implemented (T-1717): `add_legacy_intersection` (expressions/parser/static_analysis.rs) carries an
`arrays` flag, true at the top of the cell and down through the scalar arguments of a function that
computes from them (`legacy_array_context`, static_analysis/legacy.rs; not IF, IFERROR, IFNA, CHOOSE,
IFS, SWITCH, LET or LAMBDA, nor N/T over IF, SWITCH or a CHOOSE, INDEX or IFS that returns only array constants, T-1713, or INDEX of a reference, T-1908). Where it holds, an array constant argument
gets the `@`, and so does an array-valued call of more than one cell whose array is one of its
arguments or comes from a non-scalar one (`legacy_array_call_at`). Probed since in T-1847 (below): IFNA, IFS, the IFERROR argument, T over SWITCH and the array-constant `@` under LEN and ISNUMBER/SEARCH match. An array constant inside an operator takes its own `@`, not one on the call: `=N({2;3}+0)` shows as `=N(@{2;3}+0)` and `=ABS({2;3}+0)` as `=ABS(@{2;3}+0)`, 2 with no spill (probes/legacy-array-at-shapes I20, J20, I11, J11; ours matches since T-1911). The same holds under a unary sign (`=ABS(-@{2;3})`, `=ABS(-@{5})`), parentheses (`=N((@{2;3}))`, `=ABS((@{2;3}))`), `&` (`=LEN(@{"ab";"c"}&"x")`), comparisons (`=N(@{2;3}>1)`) and range-mixed operands, where the range takes an `@` too (`=N(@{2;3}+@Y1:Y2)`, `#VALUE!` from a row outside Y1:Y2). A one-element constant gets `@` in a scalar-argument position (`=ABS(@{5}+0)`) but the probed `=SUM(ABS({5}+0))` shows no `@` (other aggregates and multi-element constants under SUM are not probed). Probed in T-1981 (probes/legacy-array-at-operators, excel-graph, 2026-10-22); ours matched without changes.
In the probed shapes a legacy cell puts no `@` on a SWITCH, its results or a call over it: `=SWITCH(1,1,{2;3})`, `=SWITCH(1,1,Z30:Z32)` and `=SWITCH(1,1,Z30:AA31)` show no `@` and spill (1, 2, 3 for the range), as do `=ABS(SWITCH(1,1,{-2;3}))`, `=-SWITCH(1,1,{2;3})` and `=LEN(SWITCH(1,1,{"ab";"c"}))`; the one-element `=SWITCH(1,1,{5})` shows `=SWITCH(1,1,{5})` with no `@` on the constant, and several pairs, a default and a typed `=@SWITCH(1,1,{2;3})` (kept, value 2) follow (probes/legacy-switch-at-shapes, 57 cells, excel-graph, 2026-10-20, T-1964; C20, C11 earlier, T-1910), unlike IF/IFS/IFNA.
Generalised (probes/legacy-switch-callers, 177 cells, excel-graph, 2026-10-22, T-2121): a SWITCH at any depth of a legacy formula switches off every automatic `@` in that formula, not only on the SWITCH and its results. FORMULATEXT shows none for `=SIN(SWITCH(1,1,{2;3}))`, `=ROUND(SWITCH(..),0)`, `=N(SWITCH(..))`, `=SUM(SWITCH(1,1,Z30:Z32))` (6), `=INDEX(SWITCH(1,1,Z30:Z32),1)`, `=INDIRECT(SWITCH(1,1,"Z30:Z32"))`, `=OFFSET(Z30,0,0,SWITCH(1,1,3))`, `=INDEX(Z30:Z32,SWITCH(1,1,{1;2}))`, `=SWITCH(1,1,{2;3})+0`, `=ABS((SWITCH(..)))`, `=ABS(SWITCH(1,1,Z30:Z32))`, results that are operations or calls, a nested SWITCH, `=IF(1,SWITCH(..))`, a range or array lookup value or case (`=SWITCH(Z30:Z32,1,5)`, `=SWITCH({1;2},1,5,2,6)`), a SWITCH past the first argument, and the scalar `=SWITCH(1,1,2)`; each cell spills as its dynamic twin does. A range outside the SWITCH loses its `@` too: `=SWITCH(1,1,2)+Z30:Z32`, `=Z30:Z32+SWITCH(1,1,{2;3})`, `=ABS(Z30:Z32)+SWITCH(1,1,2)`, `=SWITCH(1,1,2)&Z30:Z32`, `=IF(SWITCH(1,1,1),Z30:Z32)` and `=SUM(Z30:Z32*SWITCH(1,1,2))` spill like the dynamic cell, while the control `=ABS(Z30:Z32)` keeps `=ABS(@Z30:Z32)` (`#VALUE!`). A typed `=ABS(@SWITCH(Z30:Z32,1,{-2;3}))` keeps its `@`, value 2.
Dynamic-array-only functions and LET, LAMBDA and names (probes/legacy-at-let-lambda-dynamic-funcs, 163 cells, and probes/legacy-at-defined-name-dynamic-funcs, 37 cells, excel-graph, 2026-10-22, T-2387): a SWITCH inside a LET body (`=LET(x,SWITCH(1,1,{2;3}),x)`, `=ABS(LET(x,Z30:Z32,SWITCH(1,1,x)))`) or inside a LAMBDA body, called directly or through a LET-bound lambda, removes every `@` like a SWITCH at top level, and ours matches. The same holds for a call of XLOOKUP, FILTER, SORT, SEQUENCE, UNIQUE or TAKE anywhere in a legacy formula (`=ABS(Z30:Z32)+XLOOKUP(2,Z30:Z32,Z30:Z32)`, `=ABS(FILTER(Z30:Z32,Z30:Z32>1))`, `=ABS(Z30:Z32)+SORT(Z30:Z32)`: FORMULATEXT shows no `@`, spills), and ours matches since T-2387 (`NO_AUTOMATIC_AT`), also as a SUMIF criterion (`=SUMIF(Z30:Z32,UNIQUE(Z30:Z32))`, `=LARGE(SUMIF(Z30:Z32,UNIQUE(FILTER(...))),1)`, XLOOKUP and SORT twins: no `@`). IFS, MAXIFS and LET do not: `=ABS(Z30:Z32)+IFS(1,1)` shows `=ABS(@Z30:Z32)+@IFS(1,1)` (#VALUE!), `=ABS(Z30:Z32)+LET(x,1,x)` shows `=ABS(@Z30:Z32)+LET(x,1,x)` and `=ABS(Z30:Z32)+MAXIFS(...)` keeps its `@` on the range. A LAMBDA call (`=ABS(Z30:Z32)+LAMBDA(x,x)(1)`) removes them too (spills 2, 3, 4), and ours matches since T-2419 (`removes_automatic_at`). A LET returning a range takes the `@` itself, not the caller: `=ABS(LET(x,Z30:Z32,x))` and `=ABS(LET(x,1,Z30:Z32))` show `=ABS(@LET(...))` (#VALUE!); ours matches since T-2419 (`let_returns_range`: the result a range or a variable bound to one). A defined name takes the `@` in a value position whatever its body is: `=ABS(swArr)` shows `=ABS(@swArr)` (spills only 2), `=ABS(rngZ)` `=ABS(@rngZ)`, and `=ABS(Z30:Z32)+swOne` shows `=ABS(@Z30:Z32)+@swOne` (#VALUE!), for names whose body is SWITCH, a range, LET, XLOOKUP or `ABS(range)`; a SWITCH reached only through a name does not remove the host formula's `@`s; `=SUM(swArr)` has none. At the root `=swOne` and `=rngZ` show `=@swOne` and `=@rngZ`, and in an IF arm the name stays whole like a range while the IF takes the `@` (`=@IF(TRUE,swOne)`, `=SUM(@IF(TRUE,swOne)+0)`, `=ABS(@IF(TRUE,rngZ))`). A defined-name lambda's call takes the `@` itself (`=ABS(@Z30:Z32)+@myId(1)`), while a LET-bound lambda's call removes every `@` like a LAMBDA call (`=ABS(Z30:Z32)+LET(f,LAMBDA(x,x),f(1))` spills 2, 3, 4). The other dynamic-array-era functions split (`=ABS(Z30:Z32)+F(..)` with F over scalars or Z30:Z32, probes/legacy-at-dynamic-funcs-2, excel-graph, 2026-10-23, T-2440): SORTBY, VSTACK, HSTACK, DROP, TEXTSPLIT, TOCOL, TOROW, WRAPROWS, WRAPCOLS, CHOOSEROWS, CHOOSECOLS, EXPAND, BYROW, BYCOL, MAP, REDUCE, SCAN and MAKEARRAY remove every `@` (the cell spills; `=ABS(Z30:Z32)+LEN(TEXTSPLIT("ab","b"))` is `#SPILL!` all the same), while XMATCH, TEXTBEFORE, TEXTAFTER, ARRAYTOTEXT, VALUETOTEXT, GROUPBY (under ROWS), REGEXEXTRACT, REGEXTEST, TRIMRANGE, PERCENTOF, TEXTJOIN and CONCAT keep `@Z30:Z32`. Of those, REGEXEXTRACT and TRIMRANGE take an `@` themselves (`LEN(@REGEXEXTRACT("a1","[0-9]"))`, `+@TRIMRANGE(Z30:Z32)`), and TEXTBEFORE, TEXTAFTER and REGEXTEST over scalars take none. The LET shapes T-2419 implemented from one probe hold: `=LET(x,Z30:Z32,x)` shows `=@LET(x,Z30:Z32,x)`, and `=ABS(LET(x,LET(y,Z30:Z32,y),x))` and `=ABS(LET(x,Z30:Z32,(x)))` show `=ABS(@LET(...))`, all `#VALUE!` off the rows. Ours matches all of this since T-2440 (`add_legacy_intersection`'s defined-name and named-call arms, `is_range_arm`, `NO_AUTOMATIC_AT`, `calls_bound_variable`; the bound-lambda test does not track scopes). Unprobed: the other dynamic-array-era functions (RANDARRAY, PIVOTBY, ISOMITTED, IMAGE); a bare LAMBDA definition outside a call; a name whose body is a single cell, a constant, an array constant or another sheet's reference (ours: the `@` too); and calls of functions ours does not know, which parse as named calls and take the `@` (ours).
Ours matches (`add_root_legacy_intersection`, static_analysis/legacy.rs, replacing T-1964's `spilling_switch_position`): a formula holding a SWITCH gets no automatic `@`, and the export (`remove_redundant_implicit_intersection`) keeps every `@` in it, since the import adds none. Probed in T-2387: a SWITCH inside a LET or LAMBDA body does the same, and so do the other functions of `NO_AUTOMATIC_AT` (previous paragraph); a SWITCH behind a defined name does not (T-2440).
One-cell array-valued calls: `=ABS(TRANSPOSE({5}))` takes the `@` in Excel and ours (below), while CHOOSE's one-cell result takes none (probes/legacy-one-cell-shape-intersection J3). The export drops exactly these `@`s (`remove_redundant_legacy_intersection`). The N/T
exception's `#VALUE!` is implemented by the legacy N/T marker (T-1713, next section).
Probed in T-1847 (probes/legacy-array-at-shapes, excel-graph, 2026-10-18; FORMULATEXT shown; each multi-cell dynamic twin spills, except F1 which is `#SPILL!`, its row spill blocked by G1):
`=IFNA({2;3},0)` `=@IFNA({2;3},0)` (2), `=IFS(1,{2;3})` `=@IFS(1,{2;3})` (2), `=IFERROR(ABS({-2;3}),0)`
`=@IFERROR(ABS({-2;3}),0)` (2: the `@` on the IFERROR, not inside), `=LEN({"ab";"c"})` `=LEN(@{"ab";"c"})` (2),
`=ABS(TRANSPOSE({5}))` `=ABS(@TRANSPOSE({5}))` (5: a one-cell array still gets the `@`),
`=ISNUMBER(SEARCH({"a","b"},Z1))` `=ISNUMBER(SEARCH(@{"a","b"},Z1))` (TRUE),
`=N({2;3}+0)` `=N(@{2;3}+0)` (2) and `=ABS({2;3}+0)` `=ABS(@{2;3}+0)` (2): the `@` goes on the array constant
inside an operator. SWITCH is the exception again: `=SWITCH(1,1,{2;3})` shows no `@` and the legacy cell spills 2, 3,
and `=T(SWITCH(1,1,{"a";"b"}))` shows none and spills "a", "b". Ours matches, SWITCH over a range and under ABS, LEN or a minus included (T-1964; any nesting, T-2121 above). IFNA, IFS, the IFERROR argument and
T over SWITCH are confirmed.
Status: VERIFIED (excel-graph)
Tasks: T-1713, T-1717, T-1847, T-1910, T-1911, T-1964, T-2121, T-2419, T-2440

## Legacy N and T over a reference-returning call (T-1713)
Rule: in a legacy (non-array) formula, N and T read their argument as a reference: no `@` goes on it,
on a range in it, or on an IF, CHOOSE, INDEX, IFS or OFFSET call in it (FORMULATEXT shows none). A
reference result is read as N/T of a reference, its top-left cell: with Z1:Z2 = 7, 8, `=N(Z1:Z2)`,
`=N(OFFSET(Z1:Z2,0,0))` and `=N(IF(1,Z1:Z2))` are 7 off the range's rows. An array result is
`#VALUE!`, whatever its size and also nested in a scalar position: `=N(IF(1,{2;3}))`,
`=N(CHOOSE(1,{2;3}))`, `=N(INDEX({2;3},0,1))`, `=N(IFS(1,{2;3}))`, `=N(IF(1,{2;3},{4;5}))`,
`=N(IF({1;0},2,3))`, `=N(IF(1,{2}))`, `=N(IF(1,{2,3;4,5}))`, `=N(IF(1,{"x";3}))`,
`=T(CHOOSE(1,{"a";"b"}))`, `=T(IF(1,{2;3}))`, `=ABS(N(IF(1,{2;3})))`, `=N(IF(1,{2;3}))+0` and
`=N(IF(1,{2;3}))&""` are all `#VALUE!` with no spill. In an array context the array lifts instead:
`=SUM(N(IF(1,{2;3})))` is 5, and under an `@`-ed IF it is intersected: `=IF(1,N(IF(1,{2;3})))` is
shown `=@IF(1,N(IF(1,{2;3})))` and is 2.
Exceptions: IFERROR gets the `@` (`=N(IFERROR({2;3},0))` is `=N(@IFERROR({2;3},0))`, 2), and SWITCH
gets none and the legacy cell spills: `=N(SWITCH(1,1,{2;3}))` shows no `@` and gives 2, 3 in two
cells, as a dynamic formula would.
Edits (SPEC-0012 rule 6, T-1853): inserting a row above a legacy `=N(IF(1,{2;3}))+A5` adjusts it to `+A6`. The cell stays a plain `<f>`
and stays `#VALUE!`; C2 `=N(IF(1,A5:A6))+A5` likewise stays legacy with the value 2, and FORMULATEXT shows `=N(IF(1,{2;3}))+A6`,
while the dynamic twin spills 3, 4 (probes/sheet-ops/sheet-ops-insert-row-legacy-cell, excel-graph, 2026-10-19, T-1933; first seen
in a one-off session, docs/research/t1853-legacy-cell-edits-2026-10-18.md). Ours matches since T-1852. Copy, move and fill are unprobed: Graph v1.0 refuses `copyFrom`, `moveTo` and
`autoFill` (400). They stay rule 6's assumption, and their desktop check is T-1075 item 12.
Evidence: probes/legacy-n-t-reference-args (excel-graph, 2026-10-16), probes/legacy-n-t-marker-shapes (excel-graph, 2026-10-18); row insert: probes/sheet-ops/sheet-ops-insert-row-legacy-cell (excel-graph, 2026-10-19, T-1933).
The neighbouring shapes (probes/legacy-n-t-reference-args-2, excel-graph, 2026-10-18, T-1768; Z1:Z2 = 7, 8,
Y1:Y2 = "a", "b", W1:W2 = 1, 2): N and T over CHOOSE or INDEX of a reference take no `@` on the call and read
the reference it returns (CHOOSE: its top-left cell; INDEX with a row and column number: that one cell; `=N(CHOOSE(1,Z1:Z2))` and `=N(INDEX(Z1:Z2,1))` are 7, `=T(CHOOSE(1,Y1:Y2))` and
`=T(INDEX(Y1:Y2,1))` are "a"), and so do `=T(Y1:Y2)`, `=T(OFFSET(Y1:Y2,0,0))` and `=T(IF(1,Y1:Y2))`. A range in a
scalar slot gets the `@` on the range only, never on the call, and the cell is `#VALUE!` in row 10, which the range does not
cover (rows 1-2 unprobed): `=N(IF(@W1:W2,Z1,Z2))`, `=N(OFFSET(Z1,@W1:W2,0))`, `=N(INDEX(Z1:Z2,@W1:W2))`, `=N(CHOOSE(@W1:W2,Z1,Z2))` and the
T versions (all `#VALUE!` in row 10). The engine matches all of them since T-1908 (N and T over CHOOSE or INDEX of a reference no longer get an `@` on the call).
More CHOOSE and INDEX shapes (probes/legacy-n-t-choose-index-shapes, excel-graph, 2026-10-19, T-1918; Z1:Z2 = 7, 8,
Y1:Y2 = "a", "b"; FORMULATEXT shows no `@` on the call in any, and the `@` stays on an array-constant index): `=N(INDEX(Z1:Z2,@{1;2}))`,
`=N(INDEX(Z1:Z2,0))`, `=N(INDEX(Z1:Z2,0,1))`, `=N(CHOOSE(2,Z1,Z1:Z2))`, `=N(CHOOSE(2,Z2,Z1:Z2))` (7: the picked range's top-left cell), `=T(INDEX(Y1:Y2,@{1;2}))`, `=N(CHOOSE(3,Z1,Z2,Z1:Z2))`, `=N(CHOOSE(TRUE,Z1:Z2,Z2))`,
`=T(INDEX(Y1:Y2,0))` and `=T(CHOOSE(2,Y1,Y1:Y2))` read the top-left cell of the reference picked (7, "a"); `=N(INDEX(Z1:Z2,2))` and
`=N(CHOOSE(2,Z1:Z2,Z2))` are 8. A CHOOSE with a literal index mixing references and array constants is read by the arm the
index picks: `=N(CHOOSE(1,Z1,{2;3}))` and `=N(CHOOSE(1,Z1:Z2,{2;3}))` are 7, `=N(CHOOSE(2,Z1,{2;3}))`, `=N(CHOOSE(1,{2;3},Z1))` and
`=T(CHOOSE(2,Y1,{"a";"b"}))` are `#VALUE!`, and none gets an `@` on the call. Implemented (T-1918,
`chooses_between_references_and_array_constants`); the other cells already matched. One-element array arms in such a
CHOOSE (`{2}`) are unprobed and keep the old `@`.
N over a CHOOSE with an array-constant index over reference arms (probes/subtotal-take-array-selector D1, D3, D5, D7, excel-graph, T-1909; H1:H2 = 3, 4, I1:I2 = 6, 8): `=N(CHOOSE({1,2},H1:H2,I1:I2))` shows no `@` (FORMULATEXT is the formula as typed) and is `#VALUE!` both on the arms' rows and off them, where `=ABS(...)` and `=LEN(...)` over it keep `=ABS(@CHOOSE(@{1,2},...))` and `=LEN(@CHOOSE(@{1,2},...))`, which read 3 and 1 on the rows and are `#VALUE!` off them. Implemented (T-1915, `chooses_references_by_an_array_index`). Every other shape probed reads the same, no `@` and `#VALUE!` on the arms' rows and off them (probes/legacy-n-t-choose-array-index-shapes, excel-graph, 2026-10-20, T-2100; V1:V2 = a, b, W1:W2 = c, d, X1:X2 = 3, 4, Y1:Y2 = 6, 8): T over text or number range arms (`=T(CHOOSE({1,2},V1:V2,W1:W2))`, `=T(CHOOSE({1,2},X1:X2,Y1:Y2))`), single-cell arms (`=N(CHOOSE({1,2},X1,Y1))`, `=T(CHOOSE({1,2},V1,W1))`, `=N(CHOOSE({1;2},X1,Y1))`), a cell beside a range (`=N(CHOOSE({1,2},X1:X2,Y1))`), a column or 2-D index (`{1;2}`, `{1,2;2,1}`), a reversed index (`{2,1}`), even a one-element index (`=N(CHOOSE({1},X1:X2,Y1:Y2))`), text, boolean and out-of-range elements (`{"1","2"}`, `{TRUE,FALSE}`, `{1,3}`), one arm (`=N(CHOOSE({1,1},X1:X2))`) and three (`{1,2,3}`). Arms other than plain cell or range references under an array index are unprobed and keep the old `@`; arms on another sheet and an explicit `@` on the index are unprobed too.
More shapes (probes/legacy-n-t-marker-shapes, excel-graph, 2026-10-18, T-1713 review; Z1 = 7; FORMULATEXT shows no `@`
in any of them, and none spills): `=T(IF(1,{"a"}))`, `=N(CHOOSE(1,{2}))`, `=T(CHOOSE(1,{"a"}))`, `=N(IFS(1,{2}))`
(one-element arms), `=N(INDEX({2,3;4,5},1))`, `=N(INDEX({2;3},1))`, `=N(IF(1,{2;3}+0))` (an operator in IF's arm),
`=LEN(N(IF(1,{2;3})))`, `=N(IF(1,{2;3},Z1))` (a reference in the other arm), `=N(CHOOSE({1;2},{2;3},{4;5}))` and
`=T(CHOOSE({1;2},{"a";"b"},{"c";"d"}))` are all `#VALUE!`. So CHOOSE's array index takes no `@` here, unlike
`=@CHOOSE(@{1,2},H1:H2,I1:I2)` over references (probes/legacy-choose-array-index A3). `=N(INDEX({2},1,1))` is 2.
Implemented (T-1713): the text is identical to the dynamic formula's, which spills, so the legacy rewrite
records the `#VALUE!` reading in the stored formula, as an internal N/T kind (SPEC-0012, legacy-n-t-marker.md;
`mark_legacy_reading`, static_analysis/legacy.rs, placed under T-1717's `arrays` flag; the kinds are T-1851's).
It marks an N or T in a value position whose argument is a bare IF, or a CHOOSE, INDEX or IFS returning only array
constants, that is not statically a scalar; the marked kernel reads an array as `#VALUE!` and a reference as its
top-left cell. These shapes take no `@` on the call (`leaves_call_whole`), nor on CHOOSE's index, and a one-element
array constant in the marked argument's returned arms (IF's arms, CHOOSE's and IFS's values) takes none either.
`=N(INDEX({2;3},1))` is `#VALUE!` because that INDEX returns a 1x1 array, the row it picks (lookup.md "INDEX of an
array: a whole row stays an array", T-1921). The marker is never printed: the export writes a plain `<f>` for a
legacy cell and `cm="1"` for a dynamic twin.
Verified (all cells above match; legacy-n-t-marker-shapes F10 since T-1921): IF with array-constant, one-element, operator and mixed reference arms and an
array condition; CHOOSE with array-constant and one-element values and an array index; IFS with one value pair;
INDEX of an array constant with one index on a 2x2 or with row and column; and these positions: the top of the cell, ABS's
and LEN's argument, and an operand of `+0` or `&""`. Extrapolated, unprobed: other scalar functions and operators
around the call (T-1717's `arrays` flag, T-1847); IFS with more than one pair; INDEX with an area number; T over IFS or INDEX; and an
IF nested in the marked argument's arms (the one-element `@` stays there).
A range, an OFFSET of one with number-literal offsets, or an IF with a literal condition and such arms, gets no `@`
under N and T since T-1728 (`reads_reference_whole`, static_analysis/legacy.rs; a range in a scalar slot, an operator,
an array constant or another call keeps the old `@`; CHOOSE with a literal index over such references and INDEX of
one get none either, T-1908, and are read as references, not marked; the other literal indices, INDEX indices and mixed arms are probed in legacy-n-t-choose-index-shapes, T-1918). An IF whose condition the rewrite intersects (`=N(IF(@W1:W2,Z1,Z2))`) is a scalar and is not marked.
Re-entering a marked cell through a row/column insert or delete, a row/column move, fill or paste keeps the marker since T-1852:
the display text goes back through the legacy entry (`Model::reenter_user_input`). Only the row insert is probed; the rest is rule 6's assumption.
Redo of a paste, fill or cut-rewrite re-enters through the legacy entry too, so the marker survives (T-2018; rule 6's assumption, not oracle-expressible).
Implemented since T-1717: the IFERROR `@`, the SWITCH spill and the `@IF`.
Status: VERIFIED (excel-graph)
Tasks: T-1713, T-1717, T-1727, T-1728, T-1768, T-1851, T-1852, T-1853, T-1908, T-1921, T-1933, T-2018

## Edge shapes of a cell-shaped call name
Rule: a call whose callee is spelled as a cell on the grid is a call of that cell, `#REF!`. That includes:
- the last cell, `=xfd1048576(1)`;
- anchored cells, `=$D$1(1)` and `=d$1(1)`;
- a sheet-qualified cell, `=Sheet1!D1(1)`;
- an empty argument list, `=d1()`;
- such a call intersected with a range, `=d1(1) D1`.

`ISERROR(d1(1))` is TRUE and `ERROR.TYPE(d1(1))` is 4. FORMULATEXT prints the callee bare and upper-cased:
`=XFD1048576(1)`, `=$D$1(1)`, `=D$1(1)`, `=Sheet1!D1(1)`, `=D1()`.

A cell-like name off the grid is an unknown name, `#NAME?`: a column past XFD (`=xfe1(1)`), row 0 (`=A0(1)`), row
1048577 (`=A1048577(1)`), and a non-ASCII letter (`=ß1(1)`). FORMULATEXT upper-cases the column past XFD
(`=xfe1(1)` prints `=XFE1(1)`; the row shapes were probed in upper case only), and keeps `=ß1(1)` as written.

An R1C1-shaped callee in an A1 workbook (`=R1C1(1)`, `=R[1]C[1](1)`) makes Excel refuse the workbook
(createSession 501 `unsupportedWorkbook`), so its value is not observable through the web oracle.

A range or a spill written as a callee without parentheses is a call too, `#REF!`, and FORMULATEXT keeps it bare:
`=Sheet1!$A$1:$A$2(2)` and `=$A$1:A2(2)` show as written, `=A1#(2)` as written, also over a real spill anchor `A1: =SEQUENCE(3)` (probes/callee-unparenthesized-range-sheet,
-range-plain, -spill-plain, -spill-anchor, excel-graph, 2026-10-22, T-2008; ours parsed them as `#ERROR!` before). `=$A$1(2)#` makes Excel
refuse the workbook (createSession 501), so it has no oracle value. The lower-case row shapes `=a0(1)` and `=a1048577(1)` are
`#NAME?` and print as written, lower case (probes/callee-unparenthesized-lower-row0, -lower-row-over; ours matched).

Not probed: a called cell pushed off the grid by a structural edit. That needs a column insert, which no case
file can express; ours turns `=XFD1(1)` into `=#REF!(1)` (`#ERROR!`).
Evidence: probes/cell-call-edges-a, -b, -c, -f, -g (excel-graph, 2026-10-19, T-1806).
Ours: matches since T-1806. In A1 text, a reference token directly followed by `(` is a call of that reference
(`reference_or_call_operand`, parser/operands.rs), and `callee_operand` prints any cell callee bare.
Status: VERIFIED (excel-graph)
Tasks: T-1758, T-1806, T-2008

## Legacy N and T marker spelling: a cell-like name Excel never calls
Rule: Excel calls a workbook LAMBDA name by its name, `_RS_LEGACY_N` included; but a name spelled like
an A1 cell (`NLG1`, `TLG1`) is never a name in a formula: Excel opens a workbook defining it, yet `NLG1`
reads the cell and `NLG1(1)` is a call of that cell, `#REF!`. So RustSheet stores a legacy N/T as
`NLG1(`/`TLG1(` (SPEC-0012, `functions/kinds.rs`), a spelling no callable name can have.
Evidence (excel-graph): probes/legacy-marker-spelling-name-plain, -xlfn (2026-10-18, T-1912): Excel accepts
`_RS_LEGACY_N`, `_RS_LEGACY_T` and `_xlfn._RS_LEGACY_N` as defined LAMBDA names (`=LAMBDA(x,x*10+7)`) and
calls them: each call gives 17 where `N(1)` is 1. probes/legacy-marker-spelling-name-nlg1, -tlg1
(2026-10-19, T-1936): with a LAMBDA name `NLG1` (`TLG1`) defined the workbook opens, `ISREF(NLG1)` is TRUE,
`SUM(NLG1)` is 0 (the empty cell), `NLG1(1)` is `#REF!`. Spellings Excel refuses as a defined name outright
(the workbook does not open: createSession 501 `unsupportedWorkbook`, so no case): `R`, `RC`, `R1C1` and a
256-character name (255 is accepted); those read as R1C1 references in the engine's stored-text lexer, so
they were not chosen.
With no name defined, `=_RS_LEGACY_N(1)` and `=_RS_LEGACY_T("a")` are `#NAME?` and `=NLG1(1)`, `=TLG1("a")`
`#REF!` (probes/legacy-marker-spelling-undefined, 2026-10-19).
Ours: matches since T-1936: `=_RS_LEGACY_N(1)` calls the name, or is `#NAME?` without one, also after a
reload; `=NLG1(1)` is `#REF!` (test_legacy_n_t_marker.rs). The new spelling is exact and case-sensitive.
Stored text cannot tell T-1851's spelling (`_RS_LEGACY_N(`) from a call of a name of it, so it is no
longer read as the marker: a legacy N/T marked in a model file saved between T-1851 and T-1936
(2026-10-18/19) reloads as that name call, `#NAME?` without the name; re-import the xlsx to restore it.
Since T-1997 a loaded workbook may define such a name (`Model::define_loaded_name`, `Txn::load_defined_name`) and
its formulas read the cell; `define_name` and the Name Manager path still refuse it. Only `NLG1` and `TLG1` were
probed: accepting every A1-spelled name on load is inferred from them, and that Excel's Name Manager refuses to
create one is unverified.
Status: VERIFIED (excel-graph)
Implemented in: functions/kinds.rs (`LEGACY_SPELLINGS`, `Function::from_legacy_spelling`); expressions/parser/operands.rs (`parse_named_call`); defined_names.rs (`define_loaded_name`); expressions/utils/mod.rs (`is_cell_spelled_name`)
Tasks: T-1851, T-1912, T-1936, T-1997

## `@` of a non-reference array as a function argument is its top-left value
Rule (probes/rank-at-if-ref, excel-graph, 2026-10-15, T-1596): where an argument takes a reference (aggregates, RANK's `ref`,
COUNTBLANK's `range`), `@` of an array that is not a reference is the array's top-left value, a plain value, never the whole array.
With A1:B2 = 7,1/5,3: `SUM(@(A1:B2*1))`, `SUM(@IF(A1:B2>0,A1:B2))`, `MAX(@(A1:B2*1))` and `LARGE(@(A1:B2*1),1)` are 7,
`COUNT(@IF(A1:B2>0,A1:B2))` is 1, `AVERAGE(@IF(A1:B2>0,A1:B2*2))` is 14 and `SUM(@{3,4})` is 3. `@` of an IF that picks a
reference still intersects it: `SUM(@IF(TRUE,A1:B2))` off the range rows and columns is `#VALUE!`. `COUNTBLANK(@IF(A1:B2>0,A1:B2))`
is `#VALUE!` (a value is not a reference). For RANK see statistics.md, "RANK of a non-reference array".
Ours: matches since T-1596 (`evaluate_node_with_reference` in model.rs; before, the whole array reached the function, so
`SUM(@(A1:B2*1))` was 16).
Status: VERIFIED (excel-graph)
Tasks: T-1596

## One-cell ranges and 1x1 arrays are not scalars
Rule (probes/legacy-one-cell-shape-intersection, excel-graph, 2026-10-15, T-1265; FORMULATEXT of legacy formulas):
Excel puts the legacy implicit-intersection `@` on a one-cell range written as a range and on a 1x1 array in a scalar
argument, as on any multi-cell shape: `=ABS(@E1:E1)`, `=ABS(@E1:E1+1)`, `=ABS(@{5})`, `=ABS(@CHOOSE(1,{5}))`,
`=AGGREGATE(@CHOOSE(1,{9}),0,E1:E6)`, and at the root `=@E1:E1+1` and `=@CHOOSE(1,{5})`. Inside CHOOSE the `@` goes on
the range operand of an arm's operator, not on the call: `=CHOOSE(1,@E1:E1+1)`, `=CHOOSE(1,@E1:E2+1)`. SUBTOTAL's
reference argument takes none: `=SUBTOTAL(9,CHOOSE(1,{5}))` is `#VALUE!` with no `@`. So the static analysis counts a
one-cell range (`StaticResult::Range(0, 0)`) as a 1x1 array when broadcasting, and these formulas are dynamic. Ours
matches every cell (the CHOOSE-arm `@` and `ABS(@{5})` are in `legacy.rs`, T-1633; larger array constants in a scalar
argument are unprobed).
Status: VERIFIED (excel-graph)
Tasks: T-1265, T-1633

## Defined names: relative references, 3-D ranges and the sheet-qualified fallback (gen-m8a)
Evidence: eval/suites/gen-m8a (T-1606, excel-graph, 2026-10-15), generated by
`rustsheet-gen --policy names-3d`; tab order Jan, Feb, Gap (blank), Mar, Calc, Apr, My Data, Other.
Status: VERIFIED (excel-graph). Tasks: T-1639, T-1640, T-1641. Ours (2026-10-15): relative names
were read as absolute (from Calc!K1 `=rel_same+1` was Jan!A1+1, not Jan!K1+1; fixed by T-1639); every use of `span3` and
`span_calc` is `#REF!` (COUNT 0); `Feb!loc` and `Other!loc` are errors, not 10.
- **Relative references in defined names.** A name stored as `=Jan!A1` (xlsx keeps relative name
  references relative to A1) is read at the same offset from the cell that uses it: from Calc!K1,
  `rel_same` (`=Jan!A1`) is Jan!K1, `rel_right` (`=Jan!B1`) Jan!L1, `rel_down` (`=Jan!A2`)
  Jan!K2, `rel_col_abs` (`=Jan!$A1`) Jan!$A1 and `rel_row_abs` (`=Jan!A$1`) Jan!K$1 (from K6:
  Jan!$A6, Jan!K$1). A formula-bodied name over relative names (`=rel_same+rel_right`) follows
  the using cell too. Case 051f538767ab0cfb; fixed by T-1639 (`NameReference::at`). Open, not
  probed: a relative name pushed past the last row or column (`=Jan!A2` used in row 1048576),
  where Excel is believed to wrap round to the other side (the engine does not wrap, as it does
  not for a relative reference inside a formula-bodied name); `INDIRECT("rel_same")`, which
  the engine reads from the INDIRECT's cell; and a mixed-anchor name used above its anchor
  (`Jan!$B$5:$B1` from row 1), which the engine reads as B1:B5, as an inline reference. Probe:
  T-2188.
- **3-D references in defined names.** A name bound to a 3-D range (`span3` = `=Jan:Mar!$A$1:$B$2`,
  `span_calc` = `=Mar:Apr!$A$1:$C$3`) is read by the aggregates like the inline span (`COUNT(span3)`
  10, `STDEV.S`, `SUMSQ`, a LAMBDA `sumsq(span3)`). Where the bare inline span and its arithmetic
  are `#REF!` (`=Calc:Feb!D8:F12+0.25`), the name is `#VALUE!`: `=span3`, `span3+100`. INDEX and ROWS
  give `#VALUE!` and ISREF FALSE, as they do inline. `IFERROR(span3,x)` and `IFERROR(span_calc,x)`
  are `#VALUE!`, while the inline `IFERROR(Feb:Apr!A7:B12,0.5)` is 0.5, yet `ISERROR(span3)` and
  `ISERROR(span_calc)` are TRUE. A span that crosses the
  formula's own sheet (Calc) and the blank sheet Gap aggregates normally. Cases 8982982d63e058ed,
  b221d72cc501675a; implemented by T-1640 (IFNA is unprobed; the engine gives it the same
  `#VALUE!`, which IFNA does not replace either way).
- **A name whose span is quoted on one side only is `#NAME?`.** `span_all` = `=Jan:'My Data'!$A$1`
  (the valid spelling is `='Jan:My Data'!$A$1`) is `#NAME?` in every value use (`=span_all`,
  `span_all+2`, `sumsq(span_all)`); `COUNT(span_all)` is 0, `COUNTA(span_all,7)` 2 (the error
  counts as a value), `ISERROR(span_all)` TRUE and `IFERROR(span_all,2)` 2. The engine already
  matches.
- **A sheet-qualified name falls back to the workbook name.** With workbook `loc` = 10,
  `Calc!loc` = 20 and `Jan!loc` = 30, `Feb!loc` and `Other!loc` (sheets with no local `loc`) are
  10, and `ISERROR(Feb!loc)` is FALSE; `only1` and `Other!only1` (defined only as `Jan!only1`) are
  `#NAME?` from Calc (ERROR.TYPE 5, IFERROR its fallback).
  Cases c238d93c128fd5ee, 6322c0f2638aa11b; fixed by T-1641.
- **A sheet-qualified sheet-scoped name follows a rename of its sheet.** With `Data!Local` =
  `=Data!$A$1:$A$2` and Calc `=Data!Local` (`#SPILL!`) and `=SUM(Data!Local)` (3), renaming Data to
  Inputs keeps both values (probe `sheet-ops-rename-sheet-scoped-name`, excel-graph; T-1693).
- **A sheet-qualified name that resolves to nothing follows a rename too.** `=Data!Missing` and
  `=SUM(Data!Missing)` (no `Missing` anywhere) are `#NAME?`, and after renaming Data to Inputs their
  FORMULATEXT is `=Inputs!Missing` and `=SUM(Inputs!Missing)` (probe
  `sheet-ops-rename-undefined-qualified-name`, excel-graph 2026-10-26, T-1701). Ours matches.
- **A sheet-qualified workbook name is rewritten to the workbook's file name.** With workbook `loc` =
  `=Feb!$A$1` (5) and no `Feb!loc`, Excel shows `=Feb!loc` as `=<file>.xlsx!loc` from load on, the file name
  quoted as a sheet name would be (inferred from two file names, one digit-led and quoted, one letter-led and not): FORMULATEXT gives `='53aa…f48.xlsx'!loc`, `=SUM('53aa…f48.xlsx'!loc)` and
  `='53aa…f48.xlsx'!loc+1` (values 5, 5, 6; `ISREF` TRUE) with no rename (probe
  `sheet-ops-workbook-name-via-prefix-no-rename`), and `=ebfe…145.xlsx!loc` (unquoted: the name starts with a
  letter) after renaming Feb to Mar, values still 5 (probe `sheet-ops-rename-workbook-name-via-prefix`;
  excel-graph 2026-10-26, T-1701). So a rename never touches it. The formula's own sheet is rewritten too
  (`=Calc!loc` on Calc shows `=<file>.xlsx!loc`), and a file that stores `[0]!loc` (the own-workbook prefix)
  loads as the workbook name with the same file-name text, even on a sheet with its own `loc`
  (`Local!loc` = 7 beside workbook `loc` = 5: `=[0]!loc` on Local is 5, `=loc` 7; probe
  `workbook-name-own-book-prefix`, excel-graph 2026-10-27, T-2834). Ours matches (T-2834): the parser
  writes the file-name prefix, reads `<file>.xlsx!loc` (any case) back as the workbook name, and stores and
  saves it as `[0]!loc`, so a sheet rename, a workbook rename and an xlsx round trip keep it. Unprobed: a
  file name with a space or other characters a sheet name would quote (ours quotes it as a sheet name),
  an unnamed workbook (ours keeps the typed `Feb!loc` until a re-parse shows `[0]!loc`), and `[0]!loc` inside a
  defined name's body in the file (ours writes it there too; T-2937).
- A sheet named like a cell address (`S1`) makes Excel Online refuse the whole workbook when a
  formula or name refers to it unquoted (`createSession` 501 `unsupportedWorkbook`), so the
  generator avoids such names.

## Legacy `@` on CHOOSE operator arms under aggregates, and on array constants in other scalar functions
In a legacy formula the `@` goes on the range operand of a CHOOSE arm's operator wherever the CHOOSE sits:
`=SUM(CHOOSE(1,@E1:E2+1))`, `=MAX(CHOOSE(1,@B1:B6+0))`, `=INDEX(CHOOSE(1,@B1:B6*1),3)` (no `@` on the INDEX call),
and the aggregate then reads the intersected value: off the range's rows (row 10) SUM and MAX are `#VALUE!`, and
`=MODE(CHOOSE(1,B1:B6*1))` is `#VALUE!`. On the range's rows the arm is a scalar: `=INDEX(CHOOSE(1,B1:B6*1),3)` is `#REF!`
and `=LOOKUP(2,CHOOSE(1,B1:B6*1))` is 1. A one-element array constant takes an `@` in ROUND (`=ROUND(@{5},0)`) but not
in TYPE: `=TYPE({5})` shows no `@` and is 64. Ours matches all of these (T-1643). The rules ours applies stop at the
probed shapes: a legacy INDEX over a CHOOSE whose arms are all arithmetic takes no `@` on the call
(`legacy_call_reads_a_value`; a dynamic `=INDEX(CHOOSE(1,B1:B6*1),0)` still spills); LOOKUP (array and vector forms) reads
any scalar lookup_vector as a one-element array, picked by IF or CHOOSE or written directly (`LOOKUP(2,1)` is 1, `LOOKUP("b","a")` is "a", `LOOKUP(2,1,5)` is 5, `LOOKUP(2,{1})` is 1, `LOOKUP(2,IF(TRUE,1))` is 1: probes/mode-direct-error-lookup-scalar-vector B1-B5, T-1651); INDEX reads a scalar IF or CHOOSE picked as a one-element array, `INDEX(IF(TRUE,5),1)` is 5 (B6); MODE propagates the error of an argument IF or CHOOSE picked, and every other argument error too (direct, referenced or named: probes/mode-typed-arguments A30-A33, probes/mode-error-and-computed-arguments A1-A6, A11-A13, T-1291).
`MODE(IF(TRUE,1/0),1,1)` and direct errors in MODE, MODE.SNGL and MODE.MULT (`1/0`, `NA()`, `#DIV/0!`, a cell, a name) all propagate the error (probes/mode-direct-error-lookup-scalar-vector A1-A16, T-1651).
INDEX reads any scalar first argument as a one-element array, not only an IF or CHOOSE pick (T-1645): `INDEX(XMATCH(2,{1,2,3}),1,1)` is 2, `INDEX(ROWS({1;2;3}),1,1)` 3, `INDEX(AREAS(A1:B3),1,1)` 1, `INDEX(ISOMITTED(A1),1,1)` FALSE, `INDEX(REGEXTEST(A1,1),1,1)` TRUE, `INDEX(REDUCE(0,A1:B3,LAMBDA(a,v,a+v)),1,1)` 66 (probes/lookup-argument-modes H18, H19, H24, H28, H30, H31, H39; probes/m7-argument-modes H31, H39, H46, H55; excel-graph). A literal or computed scalar too: `INDEX(5,1)` 5, `INDEX("a",1,1)` "a", `INDEX(1+1,1)` 2, `INDEX(TRUE,1,1)` TRUE, `INDEX(LET(x,7,x),1)` 7, `INDEX(5,0)` 5, and an index past it is `#REF!` (`INDEX(5,2)`, `INDEX(5,1,2)`); lifted indices read it the same way, `INDEX(XMATCH(2,{1,2,3}),{1,2})` 2, `#REF!` and `INDEX(5,{1;1})` 5, 5 (probes/index-literal-scalar, excel-graph, 2026-10-17). lookup-argument-modes H1 (`INDEX(ADDRESS(A1:A3,1),1,1)` "$A$1") is not evidence for the scalar rule: Excel's ADDRESS lifts there (`ROWS(ADDRESS(A1:A3,1))` is 3, F1), ours gives one scalar, so H1 passes for the wrong reason until ADDRESS lifts (T-1241). Ours gave `#VALUE!` ("Expecting a Range") before. probes/catalog/function-name-forms A4 (`IFERROR(INDEX(ERROR.TYPE(GROUPBY(...)),1,1),0)`, 0) is unaffected: ERROR.TYPE lifts over GROUPBY's array, so INDEX picks `#N/A`.
LOOKUP with a boolean or empty scalar lookup_vector (T-1709; its own evidence is probes/lookup-bool-empty-scalar-vector, 10 cells, excel-graph 2026-10-16): a boolean scalar is a one-element array like the number and text ones: `LOOKUP(TRUE,TRUE)` is TRUE, `LOOKUP(TRUE,TRUE,5)` 5, `LOOKUP(TRUE,TRUE,"r")` "r", `LOOKUP(FALSE,FALSE)` FALSE, and `LOOKUP(FALSE,TRUE)` and `LOOKUP(1,TRUE)` are #N/A (reasons unprobed: probably FALSE sorting before TRUE in the approximate search, and a boolean not matching a number). An empty lookup_vector (`IF(TRUE,Z99)`, which may be the reference Z99 rather than a scalar, or `Z99` itself, Z99 empty) gives #N/A for lookup values 0, "" and 1 (`LOOKUP(0,IF(TRUE,Z99))`, `LOOKUP("",IF(TRUE,Z99),5)`, `LOOKUP(1,IF(TRUE,Z99),5)`, `LOOKUP(0,Z99)`). Ours already matched all 10 cells (`scalar_as_array` in cast.rs); no code change.
Evidence: probes/legacy-choose-arm-aggregate-and-array-constant-at, probes/lookup-bool-empty-scalar-vector (excel-graph, 2026-10-15), probes/lookup-argument-modes, probes/m7-argument-modes, probes/index-literal-scalar (excel-graph).
Status: VERIFIED (excel-graph)
Tasks: T-1638, T-1643, T-1645, T-1651, T-1709, T-2100

## Legacy CHOOSE with an array index intersects the index and the call (T-1882)
Rule: a legacy (non-dynamic) `=CHOOSE({1,2},H1:H2,I1:I2)` is shown as `=@CHOOSE(@{1,2},H1:H2,I1:I2)`: the array index
takes an `@` (its first element, as in any scalar argument) and the call takes the whole-call `@` of a reference result,
so it is 3 on the arms' first row and `#VALUE!` off their rows (row 5). A scalar index shows only the call's `@`
(`=@CHOOSE(1,H1:H2,I1:I2)`, 3 and `#VALUE!` likewise). Under an aggregate there is no `@` at all and the index lifts:
legacy `=SUM(CHOOSE({1,2},H1:H2,I1:I2))` is 21 and `=ROWS(...)` 2. H1:H2 = 3, 4; I1:I2 = 6, 8.
Inside a scalar function (probes/subtotal-take-array-selector, excel-graph, 2026-10-18, T-1909) the same text is shown for ABS and LEN:
`=ABS(CHOOSE({1,2},H1:H2,I1:I2))` is `=ABS(@CHOOSE(@{1,2},H1:H2,I1:I2))` (3 on row 1, `#VALUE!` on row 5) and so is LEN (1, `#VALUE!`).
N is the exception: `=N(CHOOSE({1,2},H1:H2,I1:I2))` shows no `@` and is `#VALUE!` on both rows (it reads a reference; ours adds the `@`s and gives 3: T-1915).
Also from that probe (each case in clear space so a spill shows its shape): `=SUBTOTAL(9,TAKE(IF({TRUE,FALSE},H1:H2,I1:I2),1))`
and the DROP form spill a 1x2 row of `#VALUE!` (SUBTOTAL lifts over the non-reference array), the scalar-condition IF form is 3,
and the SWITCH / IFS forms with an array selector (`TAKE(...,2)`) spill a 2x2 block of `#VALUE!`; ours gives a single `#VALUE!` for
those two (T-1914). An earlier layout with the cases on consecutive rows showed `#SPILL!` for them: that was the spills colliding.
Evidence: probes/legacy-choose-array-index (excel-graph, 2026-10-18).
Implemented in: `is_choose_index` / `legacy_array_context` (expressions/parser/static_analysis/legacy.rs).
Status: VERIFIED (excel-graph)
Tasks: T-1882, T-1909, T-1914, T-1915, T-2100

## R1C1-shaped sheet and defined names
Rule (case-insensitive; odd, Excel is right): Excel quotes a sheet name in formula text, and refuses it
as a defined name, when the name is an R1C1 reference or starts with one, by this test (and also
when it is an A1 reference, which is a separate test: `R1`, `C3`, `RC99999`, `C16385` are A1 cells):
- `R`, `C` and `RC` on their own are quoted (`'RC'!A1`, `'r'!A1`); a letter after them ends the
  test: `RCR`, `RCD`, `RCC`, `CR`, `RD` are not quoted (`=RCR!A1`).
- `R<row>` with row 1..1048576 followed by `C` is quoted whatever follows, even an out-of-range or
  missing column: `R1C`, `R12C`, `R1C1`, `R1C0D`, `R1C16385`, `R1C99999999999`, `R1CD`, `R1C_`, `R1C1.`.
- Otherwise `R<row>`, `C<column>` (1..16384) or `RC<column>` is quoted when nothing or anything but
  a dot follows: `R1_`, `R1R`, `R1D`, `R1048576D`, `R0001D`, `C1D`, `C1R1`, `C16384D`, `RC1D`.
  Out of range it is not: `R0`, `C0`, `R0C1`, `R0C0`, `R1048577`, `R1048577C1`, `R0D`, `C16385D`,
  `RC16385D`, `RC2147483648`. A dot after the number is not quoted by this test (`R1.`, `R1.5`,
  `R1.A`, `C1.5`, `RC1.`, `R2.3C`), nor a dot before it (`R.1`, `RC.1`).
- A1 has no prefix rule (`A1B`, `A1_`, `A1.5` are not quoted), but a Lotus-style range is quoted:
  an A1 cell, one or more dots, an A1 cell, then nothing or a dot and anything (`A1.B2`, `A1..B2`,
  `A1...B2`, `B2.A1`, `XFD1.A1`, `Z9.Z9`, `A1.B2.C3`, and so `R1.C1`, `R1.R1`, `C1.C1`, `RC1.C1`,
  `R1.C16385`, `R1.C1.5`); not `A1.B2D`, `A1.B`, `A.B2`, `A0.B1`, `A1.XFE1`, `R1.R1048577`, `R1.C0`,
  `R1.C1D`, `R1.RC`, `RC.C1`. In a formula the dots are the range operator: `SUM(A1.B2)`,
  `ROWS(A1..B2)` and `SUM(B2.A1)` are `SUM(A1:B2)` (7), `ROWS(A1:B2)` (2) and `SUM(A1:B2)` in
  FORMULATEXT (probes/lotus-dot-range). The probed forms (probes/lotus-dot-range-*, T-1659): `$`
  cells (`$A$1.B2`, `A$1..$B$2`, `A1.$B$2`), a sheet prefix on the first cell (`Data!A1.B2`,
  `'Data'!B2..A1` is `Data!A1:B2`), lowercase (`a1.b2` prints `A1:B2`), whitespace around the
  dots (`A1 . B2` prints `A1:B2`; whitespace on one side only, `A1 .B2`, `A1. B2`, and `A1 .5` are unprobed, and ours reads the first two as the range too), ISREF (TRUE), an intersection (`A1.B2 B1:B3`) and a colon after
  it (`ROWS(A1.B2:C3)` is `A1:B2:C3`, 3) all read the range. Names, kept as typed and `#NAME?`:
  whole columns (`A.B`, `A..B`), `A1.5`, an off-grid second cell (`A1.XFE1`), `A1.B2D`, a name
  as the second part (`A1.foo` with `foo` defined) and a call (`A1.INDEX(B2:B2,1)`, `#NAME?`).
  Excel rejects the file (Graph `FileCorruptTryRepair`) for `ROWS(1..2)`, `A1.B2.C3`, `A1.B2.x`,
  `Data!A1.Data!B2` and `ROWS(A1:B2.C3)`; ours reads the first two as names. A LET or LAMBDA
  name spelled so is the variable in its call and the range outside it (probes/lotus-dot-name-*,
  r1c1-let-name-r1-c1): `LET(R1.C1,5,R1.C1*2)` 10, `LAMBDA(A1.B2,A1.B2*2)(4)` 8,
  `LAMBDA([A1.B2],ISOMITTED(A1.B2))()` TRUE, `LET(A1.B2,5,A1.B2)+SUM(A1.B2)` 20 (printed
  `...+SUM(A1:B2)`), `LAMBDA(x,LAMBDA(A1.B2,A1.B2+x)(1)+SUM(A1.B2))(2)` 18, and
  `LET(a1.b2,5,A1.B2*2)` 10, which FORMULATEXT prints `=LET(a1.b2,5,a1.b2*2)`: the uses take the
  declaration's spelling (ours keeps them as typed, T-1720). These files hold every use inside
  the call as `_xlpm.A1.B2` (spec2xlsx `prefix_bound_names` prefixes the whole call), so a use
  before its binding (`LET(x,SUM(A1.B2),A1.B2,x,...)`, `#NAME?` with `SUM(_xlpm.A1.B2)` in
  FORMULATEXT) shows only how Excel reads that prefixed file; it has no case, and ours reads the
  unprefixed use as the range.
Defined names: a workbook defining any name the R1C1 test quotes is rejected by Graph (`createSession`
501 `unsupportedWorkbook`): probed `RC`, `rc`, `R1C`, `R12C`, `R1C1`, `C1D`, `R1C1D`, `R1C16385`,
`R1_`, `R1R`, `R1D`, `C1R1`, `RC1D`, `R1048576D`, `R1CD`, `R1C1.`. `RCR`, `RCD`, `CR`, `R1.5` and
`C1.5` work (7). A Lotus-shaped name loads, but a use reads the range: with `R1.C1`, `R1..C1` and
`C1.C1` defined, `=R1.C1*1` is `#SPILL!` and `=C1.C1*1` 0, ISREF TRUE (ours matches: T-1659).
A name spelled like an out-of-range R1C1 reference (`R0`, `C0`, `R0C1`, `r0c1`, `R1048577`,
`R1048577C1`) loads but every use is `#NAME?` (`=R0*1`, `SUM(R0)`, and `ISREF(INDIRECT("R0"))` is
FALSE; FORMULATEXT keeps `=R0*1`). Ours matches (T-1657): the parser does not resolve it, and
the R1C1 lexer reads an out-of-range uppercase `R0C1` as a name, so it survives the R1C1 storage.
Table names are not probed;
ours applies the defined-name rule to them.
LET and LAMBDA names are not tested at all: `LET(n,5,n*2)` is 10 for `RC`, `R1C`, `R1C1`, `r1c1`,
`R5C7`, `R12C`, `R0C1`, `R0`, `R1048577`, `R1_`, `R1.C1`, `R1C1D`, `C1D`, `RCR` and `C`, and
FORMULATEXT keeps the name as typed. Ours matches, and takes LAMBDA parameters spelled so too
(`LAMBDA(R1C1,R1C1*2)(5)` is 10 and `LAMBDA(R5C7,[R1C1],R5C7*2+ISOMITTED(R1C1))(5)` is 11, FORMULATEXT keeps the text; probes/r1c1-bare-lambda-one and
-lambda-opt, excel-graph 2026-10-16, T-1715; ours matches). Formulas are stored in R1C1 form, where an in-range uppercase `R<n>C<m>`
reads back as a cell, so such a name is stored as `_xlpm.R1C1`, the prefix Excel's own xlsx form
puts on every LET and LAMBDA name and the parser drops. An out-of-range one (`R0C1`) is a name in R1C1 form already (T-1657). A bare `=R1C1`,
`=R5C7*1` or `=SUM(R5C7)` outside a LET in A1 mode is rejected by Graph too (`createSession` 501 `unsupportedWorkbook`, T-1715), so it is unprobed; ours reads it as a name (`#NAME?`), as it does `=r1c1`.
A bare `RC`, `R1C1D`, `R0C1` or `RCR` token in a cell formula of an uploaded file makes Graph reject
the workbook (so it is unprobed).
Unprobed: brackets cannot occur in sheet or defined names, so `R[1]C` forms are not covered;
lowercase Lotus-shaped defined names (`a1.b2`; in a formula `a1.b2` is the range); non-ASCII digits.
Evidence: probes/r1c1-shaped-sheet-names, -lower, -2, -3, -4, -5, -lower-2 (FORMULATEXT of
`='<name>'!A1`), r1c1-name-* and r1c1-name-use-* (defined names; the rejected names have no case),
r1c1-let-name-*, lotus-dot-range (excel-graph, 2026-10-15), lotus-dot-range-*, lotus-dot-name-* (2026-10-16).
Implemented in: `is_r1c1_shaped` and `is_dotted_a1_range` (expressions/utils/mod.rs), used by
`name_needs_quoting` (both) and `is_valid_identifier` (the R1C1 test); `is_valid_a1_identifier`
(LET, LAMBDA and the lexer) takes them all; `reads_as_r1c1_cell` (`variable_text`,
parser/stringify/bound_variables.rs) picks the names the R1C1 storage escapes;
`is_out_of_range_r1c1_name` (names never resolved: `name_operand`, INDIRECT, the R1C1 lexer).
Dotted ranges in formulas: `consume_dotted_corner` (lexer/ranges.rs; the forms with `$` or
whitespace) and `parse_dotted_a1_range` through `dotted_range_operand` (parser/operands.rs; an
identifier such as `A1.B2`, unless an enclosing LET or LAMBDA binds it: `bind_name`).
The unknown call `A1.INDEX(...)` keeps its spelling (spelling rule in "Unknown function call names", T-1719).
Status: VERIFIED (excel-graph)
Tasks: T-1652, T-1659

## LET and LAMBDA names spelled like a cell
Rule: a LET name or LAMBDA parameter spelled like a relative A1 cell (`f1`, `add5`, `c1`, `A1`),
stored in Excel's `_xlpm.` form, binds like any other name within its call, and is the cell again
outside it. Stored so (the case builder writes every bound name with `_xlpm.`):
`LET(f1,1,f1+1)` 2, `LAMBDA(f1,f1*2)(3)` 6, `LET(c1,10,r1,2,c1*r1)` 20, `LET(A1,5,a1*2)` 10,
`LET(x,LET(xy1,3,xy1+1),x)` 4, `LAMBDA(x,[f1],ISOMITTED(f1))(1)` TRUE, and with F1 holding 7,
`LET(f1,1,LAMBDA(f1,f1*10)(2))+f1` 27 (the last `f1`, outside the LET, is stored bare and is the
cell); a bound name may be called, `LET(mk,LAMBDA(a,LAMBDA(b,a+b)),add5,mk(5),add5(1))` 6 and the
`f1`/`f2` twin 100001. FORMULATEXT prints the names without the prefix, each use in its
declaration's spelling (`=LET(A1,5,A1*2)`) and the cell outside in uppercase (`...+F1`).
Typed, Excel refuses the same formulas: entering `=LET(f1,1,f1+1)`, the `add5` formula or
`=LAMBDA(f1,f1*2)(3)` through Excel for the web's input parser (Graph range `values` PATCH) is
rejected with 400 `InvalidArgument`, while `=LET(x,1,x+1)` entered so gives 2. The refused
entries cannot be committed as cases.
Ours: the A1 parser reads a cell-shaped name in a LET or LAMBDA name position as the variable,
and every later use within the call too (`bind_name`, parser/operands.rs, the
mechanism of the Lotus-style names above), whether the text is typed or stored; a use before the binding (`LET(x,F1,F1,2,...)`) stays the cell (its stored
form would prefix it; unprobed). The R1C1 storage writes such a name as `_xlpm.f1` (the R1C1
lexer refuses a cell-shaped identifier), and the xlsx export as `_xlpm.add5` like any bound
name. Typed entry refuses it, as Excel does: `Input::Formula` and `Input::Parse` (engine
`set_typed_user_input`) answer `EditError::InvalidInput` for a formula that binds a cell-shaped
name, while stored text (xlsx import, re-parses, `Input::StoredFormula`, which the harness uses)
binds it (decision 0103, T-2497). The refusal extends past the three probed formulas (UNVERIFIED):
an uppercase name (`LET(A1,...)`), an optional `[f1]` parameter, nested bindings, and desktop
Excel (the evidence is Excel for the web's input parser). Lotus-dot names typed are not refused
(unprobed). `$F$1` or `Sheet1!F1` is
never a name (`LET($F$1,1,F1+1)` is `#VALUE!`; unprobed). Unprobed too: a bound name as a
range end, `LET(f1,1,f1:G2)` (ours reads the variable, T-2497).
Evidence: probes/let-cell-shaped-name-stored, let-cell-shaped-name-typed-control, and
lambda-escaping-closures H4, H5 (excel-graph, 2026-10-23); the refused typed entries were
single-cell cases like -typed-control (`rustsheet-oracle fill`, 2026-10-23), not committed.
Status: VERIFIED (excel-graph)
Implemented in: `bind_name`, `binds_reference_shaped_name` (parser/operands.rs),
`Parser::parse_operand`, `reads_back_as_cell` (parser/stringify/bound_variables.rs); the typed
refusal: `Parser::set_typed`, `Parser::typed_refusal` (parser/mod.rs), `Model::set_typed_user_input`
(typed_input.rs), `Input::StoredFormula` (crates/rustsheet edit.rs).
Tasks: T-0439, T-2040, T-2497

## Structured references: this row, totals and ranges
Rule (gen-m8b, excel-graph, 2026-10-16, T-1607):
- `Sales[[#This Row],[Qty]]` (typed `Sales[@Qty]`) in a cell outside the table reads the table's cell on the
  formula's own row, like an implicit intersection, from any column (`@Sales[Qty]` too). On a row that is not one of the table's data rows (the header row, the totals row,
  above or below the table) it is `#VALUE!`: this-row e1bf5aa0ffb4ed87, `Sales` at A2:C16, Data!AA11 gives
  the row-11 value (`#N/A`), Data!AE19, AF17 and AH22 give `#VALUE!`; a cell in the header row (Data!AA1
  for `Tiny` at J1:K2) is `#VALUE!` too.
- The formula's row is used from another sheet too, and the result is a reference: with `Sales` at S!B3:C7
  (data rows 4-7), `=Sales[@Qty]` on sheet F row 6 is S!C6's value and `ISREF(Sales[@Qty])` on F row 5 is
  TRUE; off the data rows (S rows 1, 2, 3 = the header, 9 and 10; F rows 1 and 2) the value is `#VALUE!`
  and `ISREF` FALSE.
  `ISREF` of `Sales[Qty]`, `Sales` and `Sales[#Headers]` is TRUE (excel-graph, 2026-10-21, T-2197,
  probes/table-edits/table-this-row-off-data-isref).
- Inside the table (a formula column) `[@Col]` reads the same row; `Sales[Col]` there is the whole column
  (in-table 19fdc25163755713).
  Ours (T-2979): an operand that starts with `[` in a cell of a table (its header and totals rows included) is a
  structured reference to that table: `[@Col]`, `[Col]`, `[[#This Row],[Col]]`. It prints without the name, a
  `#This Row` one in its `@` form (`=[[#This Row],[Col]]` prints `=[@Col]`, as calculated-column rule 6 shows for
  `=[@Col]*2`; how Excel prints the typed long form is unprobed). Counting the header and totals rows as inside the
  table is an engine choice too, unprobed (T-3000). Storage (R1C1) keeps the table as a `[T]` prefix
  (`[T][[#This Row],[Col]]*2`), so the same text in two tables of one sheet reads each its own table and a re-parse
  (a sheet rename) prints it unqualified again; the export writes the qualified form Excel saves
  (`T[[#This Row],[Col]]*2`, xlsx-storage.md). Outside every table the operand is still a parse error. Clears the
  `[@Col]` cells of probes/t2786-whitespace-structured-refs-runs and the t2957 calculated-column cases.
- `Table[#Totals]` and `Table[[#Totals],[Col]]` of a table without a totals row are `#REF!`
  (names-case 5b5c0e20e82153bb K100 `ROWS(Stock[#totals])`; also the throwaway T-1607 probe
  `SUM(Sales[[#Totals],[A]])`). With one, they are the totals row; `#All` counts it, `#Data` does not
  (totals 9cb2e511e5e01db0).
- A column range is the same area whichever column is written first: `Sales[[2024]:[a.b]]`, with the
  columns in the order Price, a.b, 2024, has `COLUMNS` 2 (column-range 44cecbb1ecd5b2da K143).
- A table with no data is written (as Excel keeps it) with one blank data row; it reads as one row:
  `Stock` at E1:E2 with nothing below the header has `ROWS(Stock[[#Data],[Cost]:[Cost]])` = 1 and `#All` 2, and
  its `#Totals` is `#REF!` (column-range 44cecbb1ecd5b2da K180, K23, K185).
Status: VERIFIED (excel-graph)
Implemented in: partly; the `@` shorthand is implemented (T-1672), rows outside the table and reversed ranges (T-1675, `structured_reference_operand` in operands.rs),
totals rows from the eval adapter T-1671. Since T-1699 (SPEC-0016 step 2) `#This Row` is decided at the evaluating cell's row
(`Node::ThisRowKind`, resolved by `Node::ungrouped_at`), not when the formula is parsed, so it follows the table when the table list changes.
Tasks: T-1607, T-1671, T-1672, T-1675, T-1699

## Structured references: names and item specifiers
Rule (gen-m8b, excel-graph, 2026-10-16, T-1607): table names, column names and the item specifiers
`#All`, `#Data`, `#Headers`, `#Totals` match ignoring case: `SUM(SALES[key])`, `ROWS(stock[#all])` and
`Tiny[[#HEADERS],[Cost]]` compute (names-case 5b5c0e20e82153bb). Several item specifiers combine:
`ROWS(Sales[[#Headers],[#Data],[Note]])` = 11 and `ROWS(Sales[[#Data],[#Totals]])` = 10 for a table of 10
data rows without totals (specials e804cf2f8af00680 K76, K19). A bare table name and `Sales[]` are the data
rows: `ROWS(Sales)` = `ROWS(Sales[])` = 10 (K116, K29), and VLOOKUP, MATCH and INDEX take the bare name
(lookups 27252667210b1d03). Inside the brackets `'` escapes the next character, and `[`, `]`, `#`, `'` and `@`
in a column name must be escaped: `Stock[C'#]`, `Sales[it''s]`, `Wide[Rate '[%']]`; a column name that is not
one word needs its own brackets after `@` (`Sales[@[Unit Price]]`) (escaped d86ef297e840325f, 386/400 pass
in the engine already).
Status: VERIFIED (excel-graph)
Implemented in: escapes, case-insensitive names and the two contiguous specifier pairs (Headers+Data, Data+Totals) in expressions/lexer/structured_references.rs and parser/operands.rs (T-1673); other orders are unprobed, the
bare name and `Table[]` are the data area (T-1674). Case-insensitive `#This Row` is unprobed (assumed like the other specifiers); no model-level test yet for the HeadersData/DataTotals areas with a totals row (needs T-1671).
Tasks: T-1607, T-1673, T-1674

## Structured references: the `@` shorthand
Rule: `T[@]`, `T[@Col]`, `T[@[Col]]` and `T[@[A]:[B]]` are the UI spelling of `#This Row` and evaluate identically to the long form; `'@` inside a column name is a literal `@`.
Evidence: gen-m8b in-table 19fdc25163755713 and this-row e1bf5aa0ffb4ed87 (excel-graph).
Status: VERIFIED for evaluation; UNVERIFIED for FORMULATEXT (RustSheet shows the lowered `=$A$2`; Excel's output unprobed) and for rejecting unbracketed multi-word `T[@Unit Price]` / unterminated `T[@Qty`.
Implemented in: `Lexer::consume_this_row_shorthand`.
Tasks: T-1672, T-1683

## Structured references when the table changes
Rule (excel-graph, 2026-10-21, T-2197; probes/table-edits, `Sales` at S!B3:C7 with header row 3, data rows
4-7 and in the totals cases a totals row 8, the formulas on sheet F; each `insert_row` is one whole sheet row
inserted above row r, Graph `range('r:r')/insert` shift Down): a formula keeps its structured reference
through the edit (FORMULATEXT still prints `=SUM(Sales[Qty])`, `=COUNTA(Sales[Region])`,
`=Sales[[#Totals],[Qty]]`) and evaluates against the table as it now is:
- Insert above the header row or any row before it (r = 3, r = 1): the table moves down by one row
  (`ROW(Sales[#Headers])` 4, `ROWS(Sales)` 4); a plain `S!C4:C7` shifts to `S!C5:C8`
  (table-insert-row-above-header, table-insert-row-before).
- Insert between the header and the first data row (r = 4), strictly inside the data (r = 6) or above the
  last data row (r = 7): the table grows by one blank data row (header stays at 3, `ROWS(Sales)` 5,
  `#All` 6, `SUM(Sales[Qty])` 15) (table-insert-row-above-first-data, -inside-data, -above-last-data).
- Insert between the last data row and the totals row (r = 8 with a totals row): the table grows too (data
  5 rows, `#All` 7, the totals row moves to 9 and `Sales[[#Totals],[Qty]]` is still 15)
  (table-insert-row-above-totals).
- Insert just below the table (r = 8 without a totals row, r = 9 with one): the table is unchanged
  (table-insert-row-below, table-insert-row-below-totals).
- Quirk: a plain range onto the table's data column grows with the two edge inserts. `=SUM(S!C4:C7)` (the
  data rows) becomes `=SUM(S!C4:C8)` after the insert above row 4, where an ordinary insert at a range's
  first row shifts it (`S!C5:C8`, as above the header), and after the insert above the totals row 8, where
  an ordinary insert just below a range leaves it (as `S!C4:C7` stays for r = 8 without totals).
- So SPEC-0016's edge-case table was wrong on two rows: an insert at r == d1 and at r == t *grows* the
  table (it said fallback); the move, grow-inside and none rows hold (fixed by T-2321).
- `#This Row` through the move and the grow (excel-graph, 2026-10-28, T-2845; table-insert-row-this-row-above-header,
  -this-row-inside-data, -calc-column-above-header, -calc-column-inside-data): every `Sales[@Qty]` keeps its text, on
  every row. That includes a cell on sheet F that stays put, a cell beside the table that moves with the insert, and a
  formula inside the table. Each one evaluates at its own row against the table as it is after the edit:
  - insert above the header (r = 3, table now B4:C8, data rows 5-8): F!A4:A8 `=Sales[@Qty]` give #VALUE! (row 4 is the
    header now), 1, 2, 4, 8, where before the edit they gave 1, 2, 4, 8, #VALUE!. F!A8 went from off the data rows to
    on them (`ROW(Sales[@Qty])` 8, `ISREF` TRUE).
  - insert inside the data (r = 6, table now B3:C8): F!A4:A8 give 1, 2, 0 (the new blank data row), 4, 8.
  - S!E4:E7 `=Sales[@Qty]` beside the table move one row down and keep their text and values (1, 2, 4, 8); one below
    or above the data rows stays #VALUE!.
- FORMULATEXT prints a `#This Row` reference in its `@` form: `=Sales[@Qty]` for one typed (and stored)
  `Sales[[#This Row],[Qty]]`, and, inside its own table, without the table name: `=Sales[@Qty]*2` in S!D4 prints
  `=[@Qty]*2`, as the calculated-column rule says (same cases).
- A whole-row insert inside the data does not fill a column of identical `=Sales[@Qty]*2` formulas into the new row
  when the file has no `calculatedColumnFormula` for it (the spec builder writes none): the new S!D6 is empty
  (FORMULATEXT #N/A, value 0). `SUM(Sales[Dbl])` stays 30 (table-insert-row-calc-column-inside-data). Whether a real
  calculated column is filled is not probed.
- Excel refuses a whole-row or whole-column insert that would push any cell of a table off the sheet, even when the
  cells pushed off are empty. Two probes, `Sales` on S!B1048573:C1048576 with an insert above row 10 and `Sales` on
  S!XFC3:XFD6 with an insert left of column C, both got Graph 409 `InsertDeleteConflict`, "This won't work because it
  would move cells in a table on your worksheet." (T-2845, 2026-10-28; no eval case, because the oracle fill fails at
  the edit).
- A loaded formula naming a table the file does not have (`=SUM(Nope[Qty])`) or a column its table does not
  have (`=SUM(Sales[Nope])`) is loaded by Excel for the web with the reference rewritten: `#REF!`, and FORMULATEXT prints
  `=SUM(#REF!)` (table-unknown-name-load, table-unknown-column-load).
Status: VERIFIED (excel-graph)
Implemented in: partly (T-2195, SPEC-0016 step 1; T-1699, step 2). Every structured reference, `#This Row` included,
is kept as written (`Node::StructuredReferenceKind`, parser/structured_reference.rs) and its area is
re-derived whenever the table list changes; a sheet rename moves the tables with the sheet; a sheet delete
first rewrites the other sheets' references to its tables as plain ranges (so they become `#REF!`, as
before), and `restore_sheet` puts the tables back. A whole row or column insert or delete moves a table
when it is wholly before it, grows it when rows are inserted below its header row and at or above its totals
row (`h < r <= t`, so at its first data row and its totals row too: T-2321) and
shrinks it when the deleted rows are inside them, data edges included, with a data row left (T-2338), drops
its totals row on a delete of exactly that row (T-2338), and then every structured reference to it keeps its
text, in cell formulas on any sheet, conditional formats and defined names (T-2297, `table_edits.rs`). A delete
of its only data row keeps the row and clears the table's cells of it (`Model::last_data_row`, T-2338), and a
plain range shrinks on a delete of its edge rows (`Displacement::range_corners`, T-2338). Every other edit
that crosses a table (a delete touching the header row, of every data row at once, a column edit inside
it, row and column moves, cell shifts) leaves the table as it is, and an edit that changes the area a
structured reference resolves to rewrites it to the displaced plain range (the fallback of SPEC-0016); one
that leaves the area unchanged keeps the text. A `#This Row` reference keeps the fallback rule even when
its table follows the edit: on another sheet, a row off the data rows keeps its text and so reads the
moved or grown table's row there, but a row on them is rewritten to the plain area it read, and a moved cell
beside or inside the table is rewritten too. Excel keeps the text in all of them (the T-2845 bullet above; T-3037).
FORMULATEXT prints the long `Sales[[#This Row],[Qty]]` where Excel prints `Sales[@Qty]` (T-1683). An insert at
the first row of a table without a header row is a fallback too (unprobed, T-3039). An insert that would
carry a table past the last row or column runs (the cells pushed off are empty) and leaves the table, where Excel
refuses it (T-3038). The plain-range quirk is implemented for every range inside the table's
data rows and columns that touches the edited edge (`Displacement::range_corners`, stringify.rs, T-2321);
only the whole data column is probed, the other shapes are a hand generalisation (T-2846). Column inserts
inside grow the table (T-2339). A wholly deleted plain range prints `#REF!:#REF!` where Excel prints `S!#REF!`
(plain-range-delete-row-first A6, T-2855). With no table of that name (engine API `remove_table`), the formula is a parse
error that keeps its text and is exported as `#REF!`; a loaded formula naming an unknown table or column
(xlsx import, `Input::StoredFormula`) is `#REF!` and FORMULATEXT prints `=SUM(#REF!)`, as Excel does
(`Parser::set_loaded`, T-2322; the rewrite also covers shapes beyond the two probed ones, unprobed: T-2604); a typed one stays a parse error (`#ERROR!`, the text kept). FORMULATEXT shows the stored
canonical form (`=SUM(Sales[Qty])`), whose spelling is unprobed (T-1683). Since T-1671 the eval adapter
registers a spec table with its totals row; an insert above the totals row grows the table and keeps
`Sales[[#Totals],[Qty]]` (T-2321), and since T-2338 the totals row's deletes (delete it or the last data row) keep it too (decision 0128).
A `#This Row` reference keeps its text through an edit only where its cell keeps its row: a structural edit or a cut and paste
that moves the formula to another row rewrites it to the area it read on its old row (`$A$3`, or `#VALUE!` off the data rows),
its value before step 2. Excel keeps `Sales[@Qty]` and reads the new row through a row insert (T-2845, above; T-3037); cut and paste is desktop-only and unprobed (T-2325).
A copy and paste (not a cut) keeps the text, so the copy reads the row it lands on (hand expectation, unprobed; before step 2 it
kept reading the old row). A duplicated sheet, which gets no table, lowers each `#This Row` cell on its own row, so it reads
what it read (`#VALUE!` off the data rows). A formula on another sheet that refers to a deleted sheet's table is `#REF!` on
every row, off the data rows too (before step 2 those were `#VALUE!`), which matches "Delete the table's sheet" below
(table-delete-sheet: every structured reference becomes `#REF!`). A defined name has no cell, so its `#This Row` is resolved
at row 1, where its body is parsed, as before step 2 (unprobed, hand: Excel may resolve it at the calling cell's row; T-2595).
Tasks: T-2194, T-2195, T-1699, T-2197, T-2297, T-2321, T-2322, T-2595, T-2845, T-2846, T-3037, T-3038, T-3039

## Structured references under row deletes, column edits, convert, table and sheet delete
Rule (excel-graph, 2026-10-22, T-2324; probes/table-edits, the same `Sales` at S!B3:C7 as the insert cases
above, Qty 1,2,4,8, formulas on F, plus `COLUMNS(Sales)` and `COLUMN(Sales[Qty])`; `delete_row` is Graph
`range('r:r')/delete` shift Up, `insert_column` / `delete_column` `range('C:C')/insert|delete` shift
Right/Left):
- Row deletes. Above the table (r = 1) the table moves up (`ROW(Sales[#Headers])` 2, plain `S!C4:C7` becomes
  `S!C3:C6`); just below it (r = 8) nothing changes (table-delete-row-before, -below). Excel refuses a delete
  of the header row: 409 InsertDeleteConflict, "This won't work because it would move cells in a table on your
  worksheet" (no case: there is no value to score; excel-graph only, desktop unprobed, T-2325). A delete of the first data row (r = 4), one inside the
  data (r = 5) or the last data row (r = 7) shrinks the table (`ROWS(Sales)` 3; `SUM(Sales[Qty])` 14, 13, 7)
  and the plain `S!C4:C7` becomes `S!C4:C6` in all three (table-delete-row-first-data, -inside-data,
  -last-data). With a totals row, deleting the last data row moves the totals row up (`#All` 5,
  `ROW(Sales[#Totals])` 7, `Sales[[#Totals],[Qty]]` 7) (table-delete-row-last-data-totals); deleting the
  totals row removes it: `Sales[[#Totals],[Qty]]` and `ROW(Sales[#Totals])` are `#REF!` though FORMULATEXT still
  prints `=Sales[[#Totals],[Qty]]`, and `ROWS(Sales[#All])` is 5 (table-delete-row-totals). Deleting row 4 four
  times, every data row, leaves the header and one blank data row: `ROWS(Sales)` 1, `#All` 2, `SUM` 0,
  `COUNTA(Sales[Region])` 0, the plain range `S!C4:C4` (table-delete-row-all-data). The last of the four deletes
  is not an ordinary row delete: a plain range onto only that row survives as `S!C4:C4` instead of `#REF!`, so
  Excel keeps the table's one data row, cleared: nothing below the table moves on that last delete. `=S!C10`
  below the table reads `=S!C7` after the four deletes (three rows up, not four), `=SUM(S!C9:C10)` reads
  `=SUM(S!C6:C7)`, and `ROWS(Sales)` is 1 (table-delete-row-all-data-below, excel-graph 2026-10-27, T-2338).
  What happens to that row's cells outside the table, and to one delete of every data row at once, is
  unprobed (T-2848).
- A plain range, with no table involved, shrinks when a row delete takes its first or last rows but not all
  of them: `=SUM(S!C4:C7)` and `=SUM(S!$C$4:$C$7)` become `S!C4:C6` (`$C$4:$C$6`) for a delete of row 4 and
  of row 7, `=SUM(S!C7:C9)` becomes `=SUM(S!C6:C8)` / `=SUM(S!C7:C8)`, and a range wholly deleted
  (`=SUM(S!C4:C4)`, row 4) becomes `#REF!`, printed `=SUM(S!#REF!)` (plain-range-delete-row-first, -last,
  excel-graph 2026-10-27, T-2338). Only one-row deletes are probed: the engine's shrink on a multi-row delete
  across an edge (rows 3-5 over `C4:C7` give `C3:C4`) is a hand generalisation, and whole-row ranges and
  column deletes still give `#REF!` corners (T-2851).
- Column inserts. At A or B (left of or at the table's first column) the table moves right
  (`COLUMN(Sales[Qty])` 4, plain `S!C4:C7` becomes `S!D4:D7`); at C, between its columns, it grows
  (`COLUMNS(Sales)` 3, `COLUMN(Sales[Qty])` 4); at D, just right of it, nothing changes
  (table-insert-column-before, -left-edge, -inside, -right-edge). The new table column is named `Column1`
  and its header cell holds that text (`INDEX(Sales[#Headers],1,2)` and `INDIRECT("S!C3")` "Column1", the data
  cells blank, `INDIRECT("Sales[Column1]")` a column COUNTA counts 0 in); a second insert at C names the new
  C `Column2` and the first moves to D (table-insert-column-inside-header, table-insert-two-columns-inside-header,
  excel-graph 2026-10-27, T-2339).
- Column deletes. Deleting A moves the table left (table-delete-column-before). Deleting a table column drops
  it: deleting C (Qty) makes `SUM(Sales[Qty])` `#REF!`, printed `=SUM(#REF!)`, and the plain `S!C4:C7`
  `=SUM(S!#REF!)`, while `COLUMNS(Sales)` is 1 and `ROWS(Sales)` 4 (table-delete-column-qty); deleting B
  (Region) makes `COUNTA(Sales[Region])` 1, a COUNTA of the `#REF!` error, printed `=COUNTA(#REF!)`
  (table-delete-column-region).
- Convert to Range (`tables/convertToRange`): every value is kept and each structured reference from another
  sheet is rewritten as an absolute, sheet-qualified range: `=SUM(S!$C$4:$C$7)`, `=COUNTA(S!$B$4:$B$7)`,
  `=COLUMN(S!$C$4:$C$7)`; `ROWS(Sales)` 4, `ROWS(Sales[#All])` 5 and `ROW(Sales[#Headers])` 3 keep their values
  (table-convert-to-range). On the table's own sheet the range is not sheet-qualified: `=SUM($C$4:$C$7)`,
  `=COUNTA($B$4:$B$7)`, `ROWS(Sales[#All])` 5, and a `#This Row` reference beside the table prints
  `=$C4*2` (absolute column, relative row) (table-convert-same-sheet, T-2342).
  A defined name `=Sales[Qty]` (or `=Sales[#All]`) keeps working: `SUM(QtyName)` is 15, `ROWS(QtyName)` 4,
  `SUM(AllName)` 15, and FORMULATEXT of a cell using the name still prints `=SUM(QtyName)` (the name's own body
  is not readable through the oracle). A `#This Row` reference off the data rows is not lowered: above the
  header (S!F1), below the table (S!F9) and on another sheet (T!A1) it is `#VALUE!` after the convert and
  FORMULATEXT still prints `=Sales[@Qty]`, as the structured text of a table that no longer exists
  (table-convert-names-this-row, excel-graph 2026-10-25, T-2643).
- Delete table (`DELETE tables('Sales')`): every structured reference is `#REF!` (`SUM`, `ROWS`, `ROW`,
  `COLUMNS`, `COLUMN`) and prints as `#REF!` (`=SUM(#REF!)`); `COUNTA(Sales[Region])` is 1; and the table's
  cells are gone too: `SUM(S!C4:C7)` is 0 (table-delete-table).
- Delete the table's sheet (`DELETE worksheets('S')`): every reference is `#REF!` (`COUNTA` 1), and FORMULATEXT
  prints `=SUM(#REF!)` for the structured reference and `=SUM(#REF!C4:C7)` for the plain `S!C4:C7`
  (table-delete-sheet). SPEC-0016's assumption (`#REF!`) holds for the values.
- Renaming the table's sheet (S to Data) keeps every structured reference and value; the plain range prints
  `=SUM(Data!C4:C7)` (table-rename-sheet).
Cut and paste of table cells, row and column moves, undo after a sheet delete and a duplicated sheet have no
Graph REST endpoint, so only desktop Excel can answer them (M14, T-2325); a duplicated sheet gets no table in
the engine and its references to the source sheet's tables are written as the retargeted plain ranges (Excel
is believed to add a renamed table to the copy, hand).
Status: VERIFIED (excel-graph)
Implemented in: partly. The engine treats every table as SPEC-0016's fallback (above): the row delete above,
inside and below the table, the column inserts at D, the sheet rename and the totals-row delete already
match in value, and FORMULATEXT prints the displaced plain range where Excel keeps the text (T-2297 for the
move and inside classes); the edge row deletes (T-2338) and the column grow and table-column delete match
too (T-2339: `column_span` and `edit_columns` in table_edits.rs grow or shrink the table and its column list,
naming an inserted column `ColumnN` in its header cell, and `deleted_column` in stringify/structured.rs
prints a reference to a deleted column as `#REF!`, a column range that loses both ends too; one that loses
one end prints its displaced plain area, and a delete of every column or one straddling an edge stays a fallback, all unprobed, as are
`ColumnN` naming with gaps or case clashes and a multi-column insert: T-2899). A wholly deleted plain range still prints `#REF!:#REF!` (T-2855), and INDIRECT of a
structured reference's text is `#REF!` (T-2911). Convert to range matches (T-2342): `Model::convert_table_to_range` prints each structured
reference as the absolute area it lowered to (defined names too, T-2643). A cell with a `#This Row` reference
off the data rows is lowered at its own row (`Model::lower_this_row_references`, T-2643), so it is `#VALUE!` as
in Excel, but FORMULATEXT prints `=#VALUE!` where Excel keeps `=Sales[@Qty]` (T-2763). A deleted table or sheet matches
(T-2340): `delete_table` and `take_sheet` rewrite the stored text of every structured reference to the
removed tables (bare `Sales` included) to `#REF!` (`Model::rewrite_table_references`, tables.rs), and an
undo of the sheet delete (`restore_sheet`, `UserModel` undo) puts the old text back; a reference to a
missing sheet is displayed with `#REF!` for its sheet (`is_missing_sheet`, stringify/wrong_reference.rs)
while its stored text keeps the name, and the lexer reads `#REF!C4:C7` back as `#REF!` (engine choice: whether
Excel accepts that text typed in is unprobed). Known gaps (T-2612): because the stored text keeps the name, a
sheet added later under the old name revives a plain reference (Excel's formula has lost the sheet), and the
xlsx export writes `S!C4:C7` where Excel stores `#REF!C4:C7`. The `#REF!` display also covers a sheet that never
existed (a loaded `Nope!A1`), an engine choice with no evidence. The adapter's delete_table clears the whole table range (header, text and Qty columns: COUNTA 0 each)
and keeps the rows in place: `S!C9` still 99 and `SUM(S!C9:C10)` still 104 with unshifted text
(T-2341, table-delete-table-cells, excel-graph 2026-10-22).
Tasks: T-2197, T-2323, T-2324, T-2325, T-2297, T-2338, T-2339, T-2340, T-2341, T-2342, T-2643, T-2855, T-2911, T-2899

## Calculated columns: a formula typed into a table column
Rule (excel-graph `entered` cells, typed one at a time in the case's order, 2026-10-27, T-2957; table `T` A1:C4,
columns `Col` 1,2,4, `Other` 10,20,40 and an empty `Calc` C; FORMULATEXT of each `Calc` cell in F):
1. **A formula typed into one data cell of a column whose other data cells are all blank fills every data row.**
   `=[@Col]*2` typed into the middle row C3 fills C2:C4 (2, 4, 8, each printing `=[@Col]*2`); a totals row is not
   filled (C5 of a table A1:C5 with totals row stays blank, `ISFORMULA` FALSE) (t2957-fill-one-totals).
2. **A different formula typed into a cell of a filled column re-fills the whole column**, the earlier typed row
   included: C2 `=T[[#This Row],[Col]]*2`, then C3 `=[@Col]*3`, leaves `=[@Col]*3` in C2:C4 (3, 6, 12)
   (t2957-refill-second). probes/t2786-whitespace-structured-refs-runs C2:C4 (three typed this-row formulas, each
   re-filling) agrees. Whether typing into the row typed first re-fills too is unprobed: the case format cannot type
   one cell twice.
3. **A constant typed into a filled column is an exception**: C2 `=[@Col]*2` then C3 `5` leaves C3 the number 5
   (FORMULATEXT `#N/A`) and C2, C4 the formula (2, 8) (t2957-constant-exception). **A column with an exception is
   not re-filled**: a later `=[@Col]*3` typed into C4 stays in C4 alone (12; C2 keeps `*2`, C3 keeps 5)
   (t2957-constant-then-refill). One of four runs of this sequence came back with every `Calc` cell blank; three
   reruns agree on the rule above, so that run is taken as a Graph anomaly.
4. **No fill when another data cell already holds a value**: with the constant 5 in C2 (stored in the file),
   `=[@Col]*2` typed into C3 stays in C3 alone (4), and C4 stays blank (t2957-existing-constant).
5. **An A1 formula fills shifted, like a fill-down**: `=A2*2` typed into C2 gives `=A3*2`, `=A4*2` in C3:C4 (2, 4,
   8) (t2957-a1-reference).
6. **Every filled row prints the typed cell's text** for a structured formula (`=[@Col]*2`, `=[@Col]*3` in every
   row); an A1 formula prints its shifted text. How a typed long form (`=T[[#This Row],[Col]]*2`) prints is not
   answered: in refill-second it was re-filled over.
7. **Typing just right of the table expands it** (t2957-expand-right-data, -header, each run twice): `=A2*2` typed
   into D2 grows `T` to A1:D4 (`COLUMNS(T[#All])` 4), names the new column `Column1` (D1 holds that text, as for a
   column insert above) and fills D2:D4 with `=A2*2`, `=A3*2`, `=A4*2`. `=1+1` typed into the header-row cell D1
   grows `T` to A1:D4 too, but D1 becomes the **text** `0` (not 2, not a formula; the column is named `0`), and
   D2:D4 stay blank. Typing a constant beside the table, below its last row, or beside a totals row is unprobed.
What Excel saves for a filled column is in xlsx-storage.md ("A calculated column is saved as
`calculatedColumnFormula` plus one formula per row"). Rules 1-4 depend only on the column's other data cells (all
blank, or all the one calculated formula). Rule 8 (T-2983, excel-graph 2026-10-28, same table and `FORMULATEXT` in F;
stored cells come from spec2xlsx, which writes no `calculatedColumnFormula`):
8. **A stored formula is not a calculated column.**
   (a) Table A1:C5, stored `=A2*2`, `=A3*2`, `=A4*2` in C2:C4, C5 blank, then `=A5*3` typed into C5: only C5 changes
   (C2:C4 stay 2, 4, 8; F2:F5 `=A2*2`, `=A3*2`, `=A4*2`, `=A5*3`), so a column of one stored formula with no
   `calculatedColumnFormula` is not a calculated column and is not filled (t2983-identical-stored-refill).
   (b) Stored `=A2*2` in C2 and `=A4*2` in C4, C3 blank, then `=A4*3` typed into C4: C2 stays 2, C3 stays blank (F3
   `#N/A`), C4 is 12 (t2983-formulas-and-blank-enter-last). (c) The same with `=A3*3` typed into the blank C3: C2 2,
   C3 6, C4 8, nothing filled (t2983-formulas-and-blank-enter-blank). (c) is a second witness for (a): C2 and C4 are
   one relative formula. (b) follows from (a) too, so **whether a blank beside a real calculated-column formula blocks
   the fill is unprobed** (T-3025). Only A1 formulas are probed.
(T-2969 probes a stored column formula.)
Evidence: probes/t2957-* (excel-graph, 2026-10-27): fill-one-totals C2:C4, F2:F5, G5; refill-second C2:C4, F2:F4;
constant-exception C2, C4, F2:F4; constant-then-refill C2, C4, F2:F4; existing-constant C3, F2:F4; a1-reference
C2:C4, F2:F4; expand-right-data D2:D4, F1:F4, H1:H4; expand-right-header F1:F4, H1:H4.
The constant 5 in C3 (constant-exception, constant-then-refill) and the blank C4 (existing-constant) are not
formula cells, so the sidecars do not hold them: they are read from each case's saved `xl/worksheets/sheet1.xml`
(eval/.cache/oracle/excel-graph/<input_sha256>.xlsx).
Ours (T-2958, T-3024): a formula typed into a table's data rows (`Model::set_typed_user_input`, the facade's
`Input::Parse` and `Input::Formula`) fills the column's other data rows when they are all blank, or when they all hold
one shared formula and the column is a calculated column (`Model::calculated_column`, calculated_columns.rs), by
writing that shared formula, which is stored relative to its cell, so an A1 formula fills shifted. A column is a
calculated column when its `TableColumn::calculated_column_formula` is set, which a typed formula that fills or
re-fills it does (only whether it is set is read; its text, the first data row's formula in English, is an engine
choice); a typed constant
keeps it. So a column of identical stored formulas is not re-filled (rule 8a). It stays set until the column is
removed: clearing or overwriting the column's cells does not clear it, an engine choice (what Excel does is unprobed). A blank cell beside formulas blocks the
fill in the engine, an unprobed choice (T-3025). The column formula is neither read from nor written to xlsx yet
(T-2969 probes whether Excel honors it on load; T-3033 then imports and exports it), so a column Excel saved as
calculated is, after import, no calculated column for a typed re-fill. A stored formula (`Input::StoredFormula`, xlsx import) never fills;
the facade logs the filled cells and the table as it was, so they are one edit with the typed cell. The eval adapter
types `entered` cells after the tables and every stored cell, as the oracle types them after load. The unqualified `[@Col]` the probes type resolves to the
table holding the cell since T-2979 (below).
Ours (T-2968): rule 7. A formula typed (not stored) into the cell just right of a table's header row or a data row
grows the table by one column (`Model::table_expansion`, calculated_columns.rs), before the fill of rules 1-5 runs
over the new column. The new column is named as a column insert names one (`Column1`, `table_edits::append_column`)
and the name is written into its header cell. A formula typed into the header cell itself is not stored: the header
is the text `0`. This is only probed with `=1+1`, and the oracle's H3 (`D1&""`) does not tell the text `0` from the number 0; headers are text, so the engine writes text. Whether Excel derives the `0` from the formula is unknown. If the
table already has a column named `0`, the engine falls back to `ColumnN` (an engine choice). The table does not grow,
and the cell is typed as before, in four unprobed cases: a constant typed there, a table with a totals row, a new
column whose other cells are not all blank, and a new column that another table overlaps. Nor does it grow for a formula that does not parse there as typed (a parse error, or typed text decision 0103 refuses), since Excel refuses that entry and changes nothing. That includes an unqualified `=[@Col]*2` typed beside the table, which is a parse error outside every table (T-2979): what Excel does with it there is unprobed (T-3003). The facade's edit holds
the growth, the header cell and the filled rows, and its rollback puts the table back.
Status: VERIFIED (excel-graph; desktop unprobed; rule 3's no-re-fill half on 3 of 4 runs)
Implemented in: rules 1-5 and 8, crates/rustsheet-engine/src/calculated_columns.rs (T-2958, T-3024); rule 7, the same file (T-2968)
Tasks: T-2957, T-2958, T-2968, T-2969, T-2983, T-3024, T-3025, T-3033

## Unknown function call names: spelling in FORMULATEXT
Rule (excel-graph, 2026-10-16, T-1719): a call to a name that is no function (`=foo(1)`) is `#NAME?`
and FORMULATEXT prints the name with the spelling of the *first* occurrence of that name (compared
case-insensitively) in the workbook, not the spelling typed in that cell, and not lowercased: with
`=foo(1)` in A1, `=Foo(1)` and `=FOO(1)` in A2/A3 all print `=foo(1)`; `=my_Fn(1,2)+sum(3)` in A4
prints `=my_Fn(1,2)+SUM(3)` and a later `=My_Fn(1)` / `=MY_FN(1)` print `=my_Fn(1)`; `=Bar(1)`, then
`=BAR(1)` / `=bAr(1)` print `=Bar(1)`; `=my.fn(1)` then `=My.Fn(1)` prints `=my.fn(1)`;
`=A1.INDEX(1)` then `=a1.Index(1)` prints `=A1.INDEX(1)`. A name never seen before keeps its typed
spelling (`x1y`, `Xyz2`, `_Foo`, `Foo_`), except `=Ab1(1)` (cell-reference-shaped name), which prints
`=AB1(1)` and is `#REF!` (a call of the cell AB1; ours matches: `parse_named_call` makes a cell-shaped
name that is no function a `LambdaCallKind` of the reference, printed bare, T-1758; ours also reads the last column XFD and the last row 1048576 as cells, `XFD1048576(1)` is a call (unprobed, T-1806); A1 grammar only: in R1C1 mode `AB1(1)` stays a name). Unprobed: a column past XFD, a row past 1048576 or 0 (ours: unknown name, `#NAME?`), `$`-anchored and
sheet-qualified callees (ours: parse error) and non-ASCII letters such as `ß1(1)` (ours: unknown name; only ASCII
names are read as cells; edge shapes: T-1806).
"First" is in cell order of the probe (row-major, A before B), the order Excel loads the xlsx in;
whether a later edit can re-spell a name is unprobed.
Ours (T-1719): a workbook-level registry (`CallNameSpellings`, lowercase name → first spelling) gives
every stored formula's free callee the registered spelling, on load in sheet and shared-formula
order and on each edit; a callee bound by an enclosing LET or LAMBDA takes none and prints the spelling of its
declaration (T-1720, T-1761). The harness sets a spec case's cells row by row so the first-met order matches the xlsx.
Reach (excel-graph, 2026-10-17, T-1756, probes/unknown-function-name-reach):
- A call of a defined LAMBDA name prints the defined name's spelling, not the first typed one:
  `OtherFn` defined, `=otherfn(1)` then `=OTHERFN(1)` both print `=OtherFn(1)`. Ours matches: a free callee that is a defined name takes the defined name's spelling (T-1763; scope and non-LAMBDA defined names unprobed, ours ignores both).
- A callee bound by an enclosing LET or LAMBDA prints the binding's spelling
  (LET, whatever case the call used: `=LET(Gg,LAMBDA(x,x*2),gG(3))` prints `Gg(3)`; LAMBDA, same-case call only probed:
  `=LAMBDA(Hh,Hh(1))(LAMBDA(x,x))` keeps `Hh`). Ours matches (T-1720 respells every bound use to its declaration).
- An unknown name used as a value shares the table with unknown callees, in either order: `=Qux+1` then
  `=qux(1)` prints `=Qux(1)`; `=wib(1)` then `=Wib+1` prints `=wib+1`. Ours matches (T-1760). Unprobed: unknown-name shapes built by the parser (U+3000 intersection text, out-of-range R1C1-looking names) also go through the table.
- Cross-sheet: the first spelling in sheet order wins (`=Zed(1)` on Sheet1, `=ZED(1)`/`=zed(2)` on Sheet2 print
  `Zed`; `=MMM(1)` on Sheet1 beats `=Mmm(1)` on Sheet2). Ours matches.
- Non-ASCII: Ä/ä and É/é fold (`=ÄBC(1)` then `=äbc(1)` prints `=ÄBC(1)`), and ß folds with SS: `=straße(1)` then
  `=STRASSE(1)` prints `=straße(1)`. Ours matches: the registry keys on the upper-cased name (T-1762). Folds (excel-graph, 2026-10-17, T-1818, probes/unknown-function-name-folds): ς, σ and Σ are one name (`=ςw(1)` then `=σw(1)` prints `=ςw(1)`), so are the ligatures ﬁ/FI and ﬂ/FL and ǆ/Ǆ; ı/I, ſ/S, µ (U+00B5)/μ and µ/Μ are NOT folded (each prints its own spelling). Ours matches: the key is the upper-cased name except that ı, ſ and µ stay as they are (`spelling_key`).
Unprobed: re-spelling after edits (edit order, deleting the first user; needs the sheet_ops replay, T-1608).
More folds (excel-graph, 2026-10-17, T-1825, probes/unknown-function-name-folds-2): ϵ/ε, ﬀ/FF, ﬃ/FFI, ﬄ/FFL, ﬆ/ST and the capital sharp s ẞ (with ß and with SS) are one name; ϐ/β, ϑ/θ, ϕ/φ, ϖ/π, ϰ/κ, ϱ/ρ, ﬅ/ST (unlike ﬆ) and ŉ/ʼN are NOT folded; İ/i is not either (İ against ASCII I is unprobed). Ours: `spelling_key` (UNFOLDED list; ẞ is SS).
Unprobed folds (still merged by upper-casing): U+0345, ǰ and ΐ (a workbook with the combining-mark spelling is rejected by excel-graph, FileCorruptTryRepair).
Evidence: probes/unknown-function-name-spelling, probes/unknown-function-name-reach, probes/unknown-function-name-folds, probes/unknown-function-name-folds-2.
Status: VERIFIED (excel-graph)
Tasks: T-1719, T-1756, T-1760, T-1761, T-1762, T-1763

## A LAMBDA call returning a union or a one-cell reference (T-1951)

A written LAMBDA call keeps what its body returns in reference context, as LET does: `SUM(LAMBDA(x,x)((A1,B1)))` is 11,
`ISREF(LAMBDA(x,x)((A1,B1)))` TRUE and `AREAS(LAMBDA(x,x)((A1,B1)))` 2, the same as `LET(x,(A1,B1),x)`, and
`INDEX(LAMBDA(x,x)((A1:A3,B1:B2)),1,1,2)` reads the second area. A body that gives one cell is that cell for OFFSET and
COUNTIF too: `OFFSET(LAMBDA(x,x)(A1),1,0)` and `OFFSET(LET(x,A1,x),1,0)` read A2, `COUNTIF(LAMBDA(x,x)(A1),">0")` counts A1
(probes/lambda-union-reference-callers, excel-graph, 2026-10-19). Implemented by `call_callee_as_reference` returning the
`RefArg` uncollapsed and by `evaluate_node_with_reference` reading a LET or LAMBDA call as a reference.

A union is not accepted by OFFSET or COUNTIF, however it arrives: `OFFSET((A1,B1),1,0)`, `COUNTIF((A1,B1),">0")` and the same
union returned by `LAMBDA(x,x)((A1,B1))` or `LET(x,(A1,B1),x)` (single-area and multi-area unions alike) all give `#VALUE!`
(probes/lambda-union-offset-countif C1-C10, excel-graph, 2026-10-19, T-1993). Ours already matches.

Evidence: probes/lambda-union-reference-callers C1-C7, D1-D11; probes/lambda-union-offset-countif C1-C10 (excel-graph, 2026-10-19)
Status: VERIFIED (excel-graph)
Tasks: T-1951, T-1993

## A LET name or LAMBDA parameter shadows a defined name of the same name, in any case

`LET(mk,LAMBDA(a,a+1),mk(1))` is 2 with a workbook name `Mk` = `LAMBDA(a,a+100)` (the unshadowed `Mk(1)`
and `mk(1)` are 101). A LET name or LAMBDA parameter spelled like a defined name, in the same or another
case, is the variable up to the end of its scope: `LET(pv,2,pv+1)`, `LET(Pv,2,Pv+1)` and `LET(PV,2,pv+1)` are 3
with `Pv` = 7 (`=Pv+1` is 8), `LAMBDA(pv,pv+1)(2)` is 3, `LAMBDA(PV,Pv*10)(2)` is 20. A LAMBDA closure keeps the
defined name it was written against: `LET(f,LAMBDA(a,a+Pv),LET(pv,50,f(1)))` is 8, not 51.
The name is bound inside its own value expression too, so it hides the defined name there: `LET(pv,pv+1,pv)` is
`#NAME?` (not 8), and `LET(pv,2,x,Pv,x)` is 2. A LAMBDA body that names its own LET name is `#NAME?` as well:
`LET(Pv,LAMBDA(a,Pv),Pv(1))` (B2). It does without a defined name too: `LET(y,LAMBDA(a,y),y(1))` and
`LET(Y,LAMBDA(a,y),y(1))` are `#NAME?` (probes/let-lambda-body-own-name A1:A2, excel-graph, 2026-10-24), as are
`LET(y,y+1,y)` and a forward use, `LET(f,LAMBDA(a,g(a)),g,LAMBDA(a,a+1),f(1))`; `LET(y,LAMBDA(a,a+1),y(1))` is 2.
A LET name or LAMBDA parameter spelled like a built-in function, called, is the variable only in the stored form, where the call is written `_xlpm.Pv(1)`; typed, the function always wins. Stored (spec2xlsx writes every same-spelled word inside the LET or LAMBDA call as `_xlpm.`), in any case and wherever the name is in scope: `LET(Pv,LAMBDA(a,a),Pv(1))` is 1 and `LET(Pv,LAMBDA(a,Pv),Pv(1))` `#NAME?` (probes/let-lambda-body-own-name A6:A7); `LET(abs,LAMBDA(a,a*2),ABS(-3))` is -6, `LAMBDA(Pv,Pv(2))(LAMBDA(a,a+1))` 3, `LET(If,LAMBDA(a,a+1),If(1))` 2 and `LET(Today,LAMBDA(a,a),Today(4))` 4; the call takes the LAMBDA's arity, so `LET(Sum,LAMBDA(a,a*2),SUM(1,2))`, `PV(1,2,3)` and `Pv()` are `#VALUE!`; a bound non-LAMBDA called, `LET(Pv,2,Pv(1))`, is `#NAME?`; out of the LET's scope the function is back (`LET(x,LET(Sum,..,Sum(1)),SUM(x,1))` is 3) (probes/let-name-function-spelled-call, excel-graph, 2026-10-24). Typed through Excel's input parser, the call is the function, and FORMULATEXT shows it upper-cased: `=LET(Sum,LAMBDA(a,a*2),Sum(3))` is 3 (`SUM(3)`), `=LET(sum,SUM(1,2),sum*2)` 6, `=LET(abs,LAMBDA(a,a*2),ABS(-3))` 3, `=LAMBDA(Abs,ABS(-2))(5)` 2, `=LET(x,LET(Sum,..,Sum(1)),SUM(x,1))` 2; and `=LET(Pv,LAMBDA(a,a),Pv(1))`, `=LAMBDA(Pv,Pv(2))(LAMBDA(a,a+1))` and `=LET(Pv,2,Pv(1))` are refused (Graph PATCH 400), as `=PV(1)` is (probes/let-function-name-typed-*, excel-graph, 2026-10-24). So a real file never holds a bare call of a bound name: Excel writes `_xlpm.` on every use. Probing the stored form with a call of the function inside the name's own value or before its binding measures the case builder, not Excel: it prefixes those too (`LET(Sum,SUM(1,2),Sum)` is stored `_xlpm.SUM(1,2)` and is `#NAME?`). In the engine the parser records every LET name and LAMBDA parameter declared bare (`bound_names`) and, in A1 text that is not typed, skips the function lookup for a callee so bound (`calls_bound_name`, T-2567): that is unprefixed stored text (`Input::StoredFormula`, the harness's case formulas), which stands for the builder's `_xlpm.` file. Excel's files (xlsx import, also A1 text) declare `_xlpm.Sum`, so a bare `SUM(` there stays the function (`LET(_xlpm.sum,SUM(1,2),_xlpm.sum*2)` is 6, as typed-05), and `_xlpm.Sum(` is the variable; how Excel opens a third-party file that declares the name bare is unprobed (the engine reads it like the harness text); typed entry refuses the three refused formulas, and any typed call of a function with fewer or more arguments than it takes (`=PV(1)`), with the lifting table's counts (`Parser::typed_refusal`, T-2568); stored text keeps the call (`#ERROR!` for a function). In the engine the parser binds a LET name in its own value (so the defined name stays hidden there), but the evaluator gives it no value there: `let_scope` patches ids only into the later pairs and the body (T-2558).
Evidence: probes/let-name-shadows-defined-name A1-A13, B1-B3 (excel-graph, 2026-10-23 and 2026-10-24). Implemented in the parser
(`bind_name`, which also skips the defined-name lookup for bound names).
Status: VERIFIED (excel-graph)
Tasks: T-2057, T-2558, T-2567, T-2568

## INDIRECT: a 1x1 array a1 argument is #VALUE! (T-2436)
Rule: INDIRECT's a1 argument given as a one-element array is `#VALUE!`, while a scalar boolean is read as usual and a longer
array lifts: `INDIRECT("B1",{1})` and `INDIRECT("B1",{TRUE})` are `#VALUE!`, `INDIRECT("B1",TRUE)` is B1's 5 and
`ROWS(INDIRECT("B1",{1;1}))` 2 (probes/t2436/computed-scalar A39:A42); in the sweep `INDIRECT("Data!B3",TOCOL(2,0,Data!A5))`
is `#VALUE!` (06b4e8f5de409d5e A261) and `INDIRECT("Data!A1",FILTER(..)<6 numbers>)` six `#VALUE!` elements
(f2bd5f380ec002bf A383). This answers T-2347's question for INDIRECT (a1 by T-2436, ref_text by T-2577).
T-2577 probe: a 1x1 array lifts INDIRECT into a one-element array of references, ref_text too:
`INDIRECT({"B1"})` and `INDIRECT({"B1"},TRUE)` are `#VALUE!`, `ROWS(INDIRECT("B1",{0}))` 1, `ISREF(INDIRECT("B1",{1}))`
FALSE, `ISERROR(..)` TRUE, `ERROR.TYPE(..)` 3, `SUM(..)` and `MAX(..)` 5, `COUNT(..)` 0 (probes/t2577-indirect-ref-array
A1:A28). One index into it picks the element: `F.TEST(INDEX(INDIRECT("B1",{1}),1),{-1,0,1})` is B1's one-value `#DIV/0!`
(A100), where `INDEX(OFFSET(A1,{0;1},0),2)` is `#VALUE!` (lookup.md); INDEX of it read directly and of a 1xN grid with one
index are unprobed. The paired functions over it are statistics.md "An array of references is #NUM! in the paired functions".
Evidence: probes/t2436/computed-scalar, probes/t2436/06b4e8f5de409d5e, probes/t2436/f2bd5f380ec002bf (excel-graph,
2026-10-24), probes/t2577-indirect-ref-array (excel-graph, 2026-10-26).
Ours: since T-2577 (`lifts_one_by_one` for INDIRECT, as for OFFSET, TAKE and DROP).
Status: VERIFIED (excel-graph)
Tasks: T-2577, T-2347
