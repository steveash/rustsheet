# xlsx storage quirks

## Built-in defined names carry `_xlnm.` in workbook.xml only
Rule: Excel's built-in names (`Print_Area`, `Print_Titles`, `_FilterDatabase` and others ECMA-376 lists) are stored as `<definedName name="_xlnm.Print_Area"
localSheetId="0">` but formulas use the bare name: `=SUM(Print_Area)`. The importer drops the prefix
for the three named here and the exporter puts it back; any other `_xlnm.` name is kept as written.
A sheet-local `Print_Area` written without the prefix (as xlsxwriter's `define_name("Sheet1!Print_Area")`
does) also computes: Excel for the web gave `SUM(Print_Area)` 6 and `AREAS(Print_Area)` 2 for it.
Evidence: probes/multiarea-printarea H1:H2 (excel-graph, 2026-10-07; its spec2xlsx workbook stores
the unprefixed form). The prefixed spelling is ECMA-376 Part 1 (`definedName`); no oracle case
stores it yet.
Status: VERIFIED (unprefixed); prefixed form per ECMA-376
Implemented in: crates/rustsheet-xlsx/src/util.rs (`name_from_xlsx`, `name_to_xlsx`), T-1193; only Print_Area, Print_Titles and _FilterDatabase are mapped, the other ECMA built-ins load as written.
Open (T-1250 found no evidence): no Excel-saved workbook in the repo or the oracle holds an `_xlnm.` name (checked all 264 tracked/untracked `.xlsx`), and excel-graph cannot save a workbook, so whether a user-defined `Criteria`/`Database`/`Extract` saves with the prefix, whether `ROWS(_FilterDatabase)` resolves under an AutoFilter, and what Excel writes for both `_xlnm.Print_Area` and plain `Print_Area` in one scope are unanswered. The mapping stays narrow on purpose; a desktop-Excel save is needed (M14, Phase 4, decision 0039).

## Control characters in text are stored as `_xHHHH_`
Characters invalid in XML 1.0 (U+0000–U+0008, U+000B, U+000C, U+000E–U+001F) are written as
`_xHHHH_` (hex, upper-case) in shared strings, inline strings and cached `<v>` of `t="str"`
cells; a literal `_xHHHH_` look-alike is written `_x005F_xHHHH_`.
Evidence: `ironcalc-calc/TEXT/CHAR.xlsx` (Excel-cached): `=CHAR(1)` is stored as `_x0001_`.
Readers must decode (`rustsheet_xlsx::import::decode_xlsx_escapes`); this is not CHAR behavior.
A lone (unpaired) surrogate in a cached `<v>` is written the same way: the Excel-recalculated
file of probes/text-surrogate-positions A6 (`LEFT("a😀b",2)&MID("a😀b",3,2)`, excel-graph
2026-10-09) holds `a😀_xDE00_b`. `decode_xlsx_escapes` cannot decode a surrogate into a `String`
and keeps the escape literally, so the oracle value is the text `a😀_xDE00_b` (SPEC-0010: the
harness renders our lone units the same way, T-1578; the writer writes them so, T-1577).
A lone high surrogate is written the same way, as `_xD83D_`: probes/text-lone-surrogates B1
(`MID("😀",1,1)`), B3 (`a_xD83D_`) and B6 (`😀a_xD83D_`) (excel-graph, 2026-10-15, T-1582).
Status: VERIFIED (control characters; lone low and lone high surrogates)

## Shared-formula members store no text; the master's text applies shifted
Only the master `<f t="shared" ref=.. si=..>` carries text; members are `<f t="shared" si=../>` and
compute the master's formula with every relative reference moved by (member − master) rows/columns
(absolute `$` parts stay). Evidence: poi/NumberFormatApproxTests.xlsx Tests!A96 is a member of si 5
whose master text is `TEXT(C41, B41)` and computes `TEXT(C96, B96)`. `rustsheet-eval show`/`snapshot`
print the shifted text (spacing kept) via `shift_formula_text`; `formula_kind` still carries `si`.
Spec: ECMA-376 Part 1 §18.3.1.40 (`f`). Excel never writes a member whose shifted references fall off
the sheet; `#REF!` for that case is only the harness's display choice, not observed Excel behavior.

## A hyperlink range above 10,000 cells links only its top-left cell
Rule: Excel applies a `<hyperlink ref="A1:XFD1048576">` to every cell of its range. The importer
expands a range of at most 10,000 cells to one link per cell; a larger range (a corrupt or
malicious file can name the whole sheet) gives only the top-left cell the link, to bound memory
(CLAUDE.md directive 5). Hyperlinks do not affect calculated values.
Evidence: none for Excel; this is a resource limit, not an Excel observation.
Status: DEVIATION (decision 0014)
Implemented in: crates/rustsheet-xlsx/src/import/worksheets.rs (`MAX_HYPERLINK_RANGE_CELLS`)

## A non-finite cell value `<v>` imports as 0
Rule: a numeric `<v>` that does not read as a finite double (`NaN`, `inf`, `1e400`) is imported
as 0, as an unreadable one already was; no cell holds a NaN or an infinity. Excel never writes
such a value and its handling of a file that has one is unknown (likely repair or rejection).
Stored values up to f64::MAX are kept: the 1E+308 limit of numbers.md is about typed text.
Evidence: none for Excel; input hardening (CLAUDE.md directive 5), ported from ironcalc@58279cda.
Status: DEVIATION (decision 0014)
Implemented in: crates/rustsheet-xlsx/src/import/util.rs (`parse_cell_number`)

## Import resource limits reject a workbook outright
Rule: the importer rejects a workbook with `XlsxError::Limit` when its zip lists more than 5,000
entries, when the parts it reads inflate past 256 MiB in total (bytes actually inflated, not the
sizes the zip declares), when one shared string or cell text (`<v>`, inline `<is>`) is longer than
32,767 characters (Unicode scalar values, counted after `_xHHHH_` escapes are decoded), or when a
part uses an entity reference other than the five predefined ones and character references (declared
entities are never expanded: no "billion laughs"). The limits are
`rustsheet_xlsx::import::ImportLimits`; the public API passes its `LoadOptions` sizes through.
Excel's own cell limit is 32,767 characters (believed to be UTF-16 code units, which never count
fewer than scalar values, so this bound is never stricter than Excel's), but what Excel does with a
longer string in a file (truncate, repair or refuse) is not probed (T-0004); xlsx parts never
declare entities, and OPC forbids DTDs in package parts. Entity references in attribute values were
already rejected.
Evidence: none for Excel; these are resource limits (CLAUDE.md directive 5), not Excel observations.
Status: DEVIATION (decision 0014)
Implemented in: crates/rustsheet-xlsx/src/import/package.rs (`ImportLimits`, `Package`),
crates/rustsheet-xlsx/src/import/xml.rs (`reference_of`)

## Phonetic (furigana) runs are not cell text
Rule: a shared string's text is its plain `<t>` plus the `<r><t>` rich-text runs. `<rPh><t>` phonetic
runs (ECMA-376 §18.4.6) are annotations and are not part of the cell value, so they do not count
towards `LEN` (and so, by inference only, not towards the 32,767-character limit: not probed). Evidence: fixtures/shared-string-phonetic-runs.xlsx
(oracle: excel-graph, 2026-10-06, T-0170): A1 is "Tokyo" with an `<rPh>` run of 5 katakana;
`=LEN(A1)` is 5 (not 10), `=A1&"!"` is "Tokyo!" and `=EXACT(A1,"Tokyo")` is TRUE. The unit test
`test_phonetic_runs_are_not_cell_text` pins the behavior. PHONETIC() is not implemented.
Status: VERIFIED
Implemented in: `read_shared_strings_from_xml` in crates/rustsheet-xlsx/src/import/shared_strings.rs
Tasks: T-0164, T-0170

## Classic CEILING is stored without `_xlfn.`
Rule: the classic `CEILING` is a plain function name in the file (`<f>CEILING(2.5,2)</f>`); only the
2010/2013 forms carry the prefix (`_xlfn.CEILING.MATH`, `_xlfn.CEILING.PRECISE`). Excel also accepts
a hand-written `_xlfn.CEILING(2.5,2)` on load (value 4) and re-stores it as `CEILING(2.5,2)`.
Evidence: probes/ceiling-xlfn-prefix A1:A4, all 4 (oracle: excel-graph, 2026-10-06, T-0172); the
recalculated workbook's `<f>` text was `CEILING(2.5,2)` for both A1 and A2 and `_xlfn.CEILING.MATH(...)`
for A3 and A4.
Status: VERIFIED
Implemented in: `Function::to_xlsx_string` (crates/rustsheet-engine/src/functions/xlsx_names.rs); test
`test_export_ceiling_has_no_xlfn_prefix`.
Tasks: T-0172

## Newer functions need `_xlfn.` in the file, eta arguments `_xleta.`
Rule: a function added after Excel 2007 is recognised only under its stored prefix: `PERCENTOF(1,2)`
stored bare is `#NAME?`, `_xlfn.PERCENTOF(1,2)` calculates. A function passed by name as a LAMBDA
argument (eta-reduced, as in `GROUPBY(..., SUM)`) is stored `_xleta.SUM`; a bare `SUM` there is `#NAME?`.
A prefix on a classic function is accepted (`_xlfn.SUM(1,2)` = 3, `_xlfn.NORMSDIST(0)` = 0.5, see
also CEILING above); an unknown name with the prefix is still `#NAME?`, and so is `_xlfn.JIS`
(text.md).
Evidence: probes/catalog/function-name-forms A1 (5 = #NAME?), A2 (no error), A3 (5), A4 (no error),
A5 (no error), A6 (5); function-name-forms-2 A3 `_xlfn.SUM(1,2)` = 3; probes/catalog/normsdist-stored-form
A4 `_xlfn.NORMSDIST(0)` = 0.5 (oracle: excel-graph, 2026-10-07, T-0040; A1 until T-0052). Consequence for case specs: the
case builder (tools/src/rustsheet_tools/spec2xlsx.py) prefixes every call of a name on its own table
(the engine exporter's list plus the names only XlsxWriter knew; T-0052), whatever else shares the
formula; GROUPBY and PIVOTBY are not on it, so specs spell them `_xlfn.`
(probes/catalog/availability-*). The classic NORMSDIST/NORMSINV give the same values bare as
prefixed: normsdist-stored-form A1-A2 and availability-27 A8-A9 (refilled bare) and A4
`_xlfn.NORMSDIST(0)` = 0.5 (excel-graph, 2026-10-07, T-0052).
Status: VERIFIED
Implemented in: `Function::to_xlsx_string` (functions/xlsx_names.rs; test `newer_functions_export_with_xlfn_prefix`, T-0411). The engine parser resolves a bare newer name read from a file, and the
exporter never writes `_xleta.` (T-1089).
Comparison (T-0412): the exporter's arms were checked against XlsxWriter 3.2.9's future-function list; every
name there that has an engine function already has an arm. XlsxWriter also prefixes ANCHORARRAY (the `#`
operator), SINGLE (the `@` operator, handled by the parser), FILTERXML, QUERYSTRING and WEBSERVICE; the engine
has no Function variant for the last three, so nothing is
exported for them. GROUPBY and PIVOTBY (not on XlsxWriter's list) have `_xlfn.` arms since T-1322 and T-1326. PERCENTOF has an arm since T-0416. The case builder (which reads these arms) now prefixes it, and a
sheet's `bare_function_names` list keeps a cell's names as typed, so function-name-forms A1 still
stores the bare form (refilled 2026-10-13: still `#NAME?`). The gen-m5/m6/m7 cases that call PERCENTOF
were refilled with the prefixed form. Two existing arms
add a prefix Excel-authored workbooks omit (`GAMMALN`, `ISO.CEILING`): T-0417.
Tasks: T-0040, T-0412, T-0416, T-0417, T-1089

## Written parentheses are stored as written; `@(x)` is `_xlfn.SINGLE((x))`
Rule: the stored `<f>` text keeps every redundant parenthesis as written. A group directly after `@`
stays inside SINGLE's own parentheses:

| Entered | Stored `<f>` |
|---|---|
| `=Z1+(Z2)` | `Z1+(Z2)` |
| `=((Z1))` | `((Z1))` |
| `=-(Z1)` | `-(Z1)` |
| `=IF((Z1>0),(Z2),Z3)` | `IF((Z1&gt;0),(Z2),Z3)` |
| `=(Z1^2)^3` | `(Z1^2)^3` |
| `=SUM((Z1:Z2))` | `SUM((Z1:Z2))` |
| `=(1)+((2))` | `(1)+((2))` |
| `=@(Z1+(Z2))` | `_xlfn.SINGLE((Z1+(Z2)))` |
| `=@(A1:A3)` | `_xlfn.SINGLE((A1:A3))` |
| `=@((A1:A3))` | `_xlfn.SINGLE(((A1:A3)))` |
| `=SIN(@(A1:A3))` | `SIN(_xlfn.SINGLE((A1:A3)))` |
| `=SIN(@((A1:A3)))` | `SIN(_xlfn.SINGLE(((A1:A3))))` |
| `=@A1:A3` (no group) | `A1:A3` |

The Graph range API reads the formulas back exactly as entered.
Evidence: formulas entered through the Graph range API and the file Excel for the web saved
(excel-graph, 2026-10-11, T-1244).
Ours: export prints from the parse tree, which keeps a written group as a `Node::Group` (T-1340), so every shape
above is stored as written; the `@` operand's group is the `@` node's `Group` child, printed inside SINGLE's own
parentheses (T-1343). Import of `_xlfn.SINGLE((x))` prints `@(x)` back. A `@` with a written group is never dropped as redundant, even in a scalar context (`=SIN(@(A1:A3))`);
only the group-less `@A1:A3` loses its `@`. The engine does the same (test_implicit_intersection_export.rs).
Evidence for the `A1:A3` rows: same method, excel-graph, 2026-10-12, T-1398, sheet1.xml C1 `=@(A1:A3)`, C2 `=@((A1:A3))`,
C3 `=SIN(@(A1:A3))`, C4 `=@A1:A3`, C5 `=SIN(@((A1:A3)))`.
Status: VERIFIED (web)
Implemented in: expressions/parser/stringify.rs (`Group`, T-1340; `_xlfn.SINGLE((x))`, T-1343).
Tasks: T-1244, T-1340, T-1343, T-1398

## Typed `SINGLE(x)` is the `@` operator
Rule: a formula typed with `SINGLE(x)`, in any case, is the implicit-intersection operator `@x`, the same as the stored
`_xlfn.SINGLE(x)`, and FORMULATEXT shows it as `@`, without SINGLE's own parentheses: `=SINGLE(C1:C3)` beside row 2 is
the row's cell (20) and shows `=@C1:C3`, off rows 1-3 it is `#VALUE!`; `=single(C1:C3)` is the same; `=SINGLE(5)` is 5
(`=@5`), `=SINGLE({1,2})` 1 (`=@{1,2}`), `=SINGLE(ERROR.TYPE(#REF!))` 4 (`=@ERROR.TYPE(#REF!)`), and
`=FTEST({1;2;3},SINGLE(ERROR.TYPE(#REF!)))` `#DIV/0!` (`=FTEST({1;2;3},@ERROR.TYPE(#REF!))`). Typed `=SINGLE()` and
`=SINGLE(1,2)` are refused (Graph PATCH 400 `InvalidArgument`, as for other wrong argument counts, decision 0103).
Evidence: probes/single-typed-01 to -06 and -09, each cell typed through Graph `values` (excel-graph, 2026-10-27,
T-2938); -07 (`=SINGLE()`) and -08 (`=SINGLE(1,2)`) were refused, so they have no case. probes/t2576-error-type-cell-paired
Calc!D27, written bare and built as `_xlfn.SINGLE(` by the case builder, agrees.
Ours: the parser reads a bare `SINGLE(` with one argument as `@`, typed, through the API and in stored text alike; a
typed call with other than one argument is refused (`set_typed_user_input`), and stored text keeps it as a parse error.
Known gaps: a bare `SINGLE(` read from a file is likely `#NAME?` in Excel, as other bare newer names are ("Newer functions
need `_xlfn.`"; unprobed for SINGLE); ours reads it as `@`, the gap T-1089 owns. A LET or LAMBDA name spelled `single`
and called (`LET(single,LAMBDA(x,x*2),single(3))`) is `@3` in ours, as `anchorarray` is; Excel's reading is unprobed.
Status: VERIFIED (web)
Implemented in: crates/rustsheet-engine/src/expressions/parser/operands.rs (`parse_named_call`).
Tasks: T-2938

## A one-cell dynamic formula is stored as a one-cell array with `cm="1"`
Rule: a dynamic-array formula whose result is one cell is stored like a spilling one, with the
anchor alone as the range: `<c r="F3" cm="1"><f t="array" ref="F3">SUM(ROW(C2:C4))</f>`. A plain
`<f>` is a legacy formula, which Excel and our import intersect ("Legacy formulas intersect the
ranges in value arguments", references.md), so `SUM(B1:B6+0)` stored plain would read back as
`SUM(@B1:B6+0)`: `#VALUE!` off the range's rows instead of 12.
Evidence: Excel-authored workbooks in crates/rustsheet-xlsx/tests/calc_tests:
LOOKUP_AND_REFERENCE/ROW_COLUM.xlsx sheet6 F3 `SUM(ROW(C2:C4))` and sheet7 A1
`MAX((C1:C20<>"")*ROW(C1:C20))`, MATCH_LOOKUP_arrays.xlsx D9, simple_arrays.xlsx A1
`SUM({1,2,3}+{4,5,6})`. Excel also writes `cm` where no `@` is at stake (the array constants of
simple_arrays A1).
Ours: a formula typed with a one-cell result is a plain formula cell. The export writes it as a
one-cell dynamic array exactly when reading its stored text as legacy would add an `@`
(`legacy_reading_intersects`, T-1552), with every `@` it has kept as `_xlfn.SINGLE`; otherwise it
writes a plain `<f>`. Re-exporting the 274 Excel-authored and fixture workbooks under
crates/rustsheet-xlsx/tests and eval/suites adds no array formula (2026-10-15).
Status: VERIFIED (Excel-authored files)
Implemented in: crates/rustsheet-xlsx/src/export/worksheets.rs (`plain_formula`, T-1552).
Tasks: T-1552

## Members of a volatile spill are stored as empty `<f ca="1"/>`
Rule: desktop Excel saves each spilled member of a volatile dynamic array, apart from the anchor, as an
empty formula element with the calculate-always flag and its value:
`<c r="F261" cm="1"><f t="array" aca="1" ref="F261:F265" ca="1">_xlfn.RANDARRAY(5)</f><v>0.57…</v></c>`
then `<c r="F262"><f ca="1"/><v>0.61…</v></c>` through F265. Members of a non-volatile spill are plain
values with no `<f>`. An empty `<f ca="1"/>` inside an array's `ref` is therefore a spill member, not a
formula of its own.
Evidence: calipers roundtrip/formulas_dynamic_arrays/golden.xlsx (desktop Excel, Calipers 9e47ea4f):
the RANDARRAY spills at F261, F267, F269, F273 and F279 (T-0446).
Ours: the import loads the members as the anchor's spill. The eval snapshot reader marks them array
members, and the harness skips them as volatile, as it does the anchor (T-0446).
Status: VERIFIED (Excel-authored files)
Implemented in: crates/rustsheet-eval/src/xlsx_snapshot.rs (`parse_sheet`), crates/rustsheet-eval/src/volatile.rs (`scorable_cells`).
Tasks: T-0446

## Iterative calculation is `calcPr@iterate`, with count and delta only when not the defaults
Rule: desktop Excel saves iterative calculation as `<calcPr iterate="1"/>` in workbook.xml. It
writes `iterateCount` and `iterateDelta` only when they differ from the defaults, 100 and 0.001:
`<calcPr calcId="191029" iterate="1" iterateCount="200" iterateDelta="9.9999999999999995E-7"/>`.
The delta is a shortest round-trip double with an upper-case exponent.
Evidence: calipers pv_goldens (desktop Excel, Calipers 9e47ea4f). 68 workbooks store a bare
`iterate="1"`, and pv26_007 and pv26_2_007 store the count and delta above (T-0447).
Ours: the import reads all three. The export always writes `iterate="1"` with the count and delta,
defaults included, which Excel reads the same and which keeps a round trip exact. Unprobed, a decision-0014 choice for untrusted input: an
`iterateCount` outside 1 to 32,767 (Excel's dialog range), or a delta that is negative, NaN or
unparsable, imports as the default.
Status: VERIFIED (Excel-authored files) for the storage; the fallback is our choice
Implemented in: crates/rustsheet-xlsx/src/import/workbook.rs (`iteration_from`), crates/rustsheet-xlsx/src/export/workbook.rs
Tasks: T-0447

## LAMBDA optional parameters are stored `_xlop.`
Rule: Excel stores an optional LAMBDA parameter `[y]` as `_xlop.y`, brackets dropped, and every use of
it as `_xlpm.y` like any other parameter: `=LAMBDA(x,[y],ISOMITTED(y))(1)` is saved as
`_xlfn.LAMBDA(_xlpm.x,_xlop.y,_xlfn.ISOMITTED(_xlpm.y))(1)` (as an array formula, `t="array"`).
A file that stores `[_xlpm.y]` instead is refused by Excel for the web as corrupt
(`FileCorruptTryRepair`, T-1083). FORMULATEXT shows the typed form `=LAMBDA(x,[y],ISOMITTED(y))(1)`.
Evidence: formulas entered through the Graph range API and the file Excel for the web saved
(excel-graph, 2026-10-11, T-1274), sheet1.xml:
`<f t="array" ref="A1">_xlfn.LAMBDA(_xlpm.x,_xlop.y,_xlfn.ISOMITTED(_xlpm.y))(1)</f>` and
`<f t="array" ref="A3">_xlfn.LAMBDA(_xlpm.x,_xlop.y,IF(_xlfn.ISOMITTED(_xlpm.y),_xlpm.x,_xlpm.x+_xlpm.y))(1)</f>`;
probes/lambda-optional-params B1 (FORMULATEXT).
Status: VERIFIED (web)
Implemented in: engine import (expressions/parser/lambda.rs) and export (stringify.rs) already use
`_xlop.`; the case builder writes it since T-1274 (spec2xlsx.py `prefix_bound_names`).
Tasks: T-1083, T-1274

## Cells and rows without r are positional
Rule: `r` is optional on `<row>` and `<c>`. A row without `r` is the row after the previous one (row
1 if first); a cell without `r` is the column after the previous cell in its row (column A if first).
Evidence: ECMA-376 Part 1 §18.3.1.73 (row) and §18.3.1.4 (c) make `r` optional. POI workbooks
written this way by other tools: poi/54288.xlsx (r-less cells after `C3`), poi/59746_NoRowNums.xlsx
and poi/InlineString.xlsx (r-less rows and cells). They hold no formulas, so no Excel value is
scored; the placement is the schema's, as the eval snapshot reader already applies it.
DEVIATION (directive 5, decision 0014): an r-less cell past XFD or r-less row past row 1,048,576 is an import error, not a placed cell; an explicit `r` of i32::MAX followed by an r-less row is the same error, never an overflow.
Implemented in: `rustsheet-xlsx` `import/sheet_data.rs` `start_row` and `start_cell` (an r-less cell or row past XFD or row 1,048,576 is an import error).
Status: UNVERIFIED (probe pending; ECMA-376 and POI files only, no Excel observation)
Tasks: T-0181

## Part names match case-insensitively and across backslashes
Rule: Excel finds a package part whatever the case of its zip entry name, and reads a backslash in
an entry name as `/`. poi/49609.xlsx (not Excel-authored) names its entries `[content_types].xml`,
`_rels\.rels`, `xl\workbook.xml`, `xl\sharedstrings.xml`, `xl\worksheets\sheet1.xml`, while its
relationships point at `/xl/workbook.xml` and `/xl/sharedStrings.xml`; Excel opens it with both
sheets (FAM, VIC) and their shared-string text (B1 "Cycle"). OPC part names are case-insensitive
(ECMA-376 Part 2); the backslash reading is Excel's leniency, the schema does not allow it.
Equivalent entries (`xl/a.xml` and `XL/A.xml` in one zip) are invalid OPC and Excel's choice is
not probed: an exact name wins, else the first in byte order (our choice; UNVERIFIED, probe
pending). Percent-encoded part names (`%41` for `A`) are not normalized (not probed either).
Evidence: oracle: excel-graph, 2026-10-06T19:27Z, T-0112, input sha256 4237e7a06cfba518904fac9d2085bc38c4912290c9c698acd38234b86691c546:
the unmodified file was uploaded directly (the canary embedder cannot read such a package, so this
run has no canary and no sidecar; the file has no formulas to score) and Excel for the web opened it and returned a normalized copy with sheets FAM and VIC, B1:E1 "Cycle", "A1", "A2", "A3".
Status: VERIFIED (Excel for the web; desktop not observed), except the equivalent-entries tie-break
Implemented in: `PartNames` in crates/rustsheet-xlsx/src/import/package.rs (importer and the eval
snapshot reader)
Tasks: T-0112

## Strict OOXML uses other namespaces
Rule: a workbook saved as "Strict Open XML Spreadsheet" writes `r:id` in the namespace
`http://purl.oclc.org/ooxml/officeDocument/relationships` (Transitional:
`http://schemas.openxmlformats.org/officeDocument/2006/relationships`), and stores a date cell as
`t="d"` with ISO 8601 text (`<c r="A4" s="1" t="d"><v>1990-01-01</v></c>`) instead of a serial.
Strict files also declare `<workbookPr dateCompatibility="0">`, which ECMA-376 ties to the 1900
date base compatibility (serial 60 and dates before 1900-03-01); the importer ignores the attribute
and it changes nothing Excel computes (T-0407, below).
Evidence: poi/SimpleStrict.xlsx, poi/SampleSS.strict.xlsx, poi/sample.strict.xlsx and
poi/57914.xlsx (Excel-authored): all use the purl namespace for every `<sheet r:id>` and declare
`dateCompatibility="0"`; SimpleStrict "Sheet Number 2"!A4 is the `t="d"` cell above. Their cached
formula values pass once `r:id` is read (T-0112).
Status: VERIFIED for `r:id`. VERIFIED for `t="d"` (T-0406): Excel reads the text as a serial,
`1990-01-01` = 32874, `1990-01-01T12:00:00` = 32874.5, `1900-02-28` = 59, `1900-03-01` = 61, and
`ISNUMBER` of the cell is TRUE (fixtures/t-d-iso-date-cells, Excel for the web). Excel for the web
refuses the whole workbook (`unsupportedWorkbook`) when a `t="d"` cell holds unparsable text
(`abc`), so there is no value to match: the importer leaves such a cell empty. VERIFIED for
`dateCompatibility="0"` (T-0407, fixtures/strict-date-compatibility-0, Excel for the web): the attribute
does not switch off the 1900 leap-year bug. `DATE(1900,2,28)` = 59, `DATE(1900,3,1)` = 61, `DATE(1900,2,29)` = 60,
`TEXT(60,"yyyy-mm-dd")` = `1900-02-29`, `DAY(59)` = 28, `DATEVALUE("1900-01-01")` = 1, and a `t="d"` cell
`1900-01-01` reads as 1 (a number). The probe package was Transitional (standard namespace) carrying the attribute, not a purl-namespace package. The result is the same as a Transitional workbook, so the importer's
ignoring the attribute is correct and no follow-up is needed. Probed one form per workbook (Excel for the web rejects
a workbook with unparsable text; T-0409, fixtures/t-d-*): `1990-01-01T12:00:00Z` = 32874.5 (the `Z`
is ignored), `1990-01-01T12:00:00.5` = 32874.500005787035 (fractional seconds count), `1900-02-29` = 60,
time-only `12:00:00` = 0.5, and `1899-12-31` (before 1900-01-01) is a `#VALUE!` cell
(`ISNUMBER` FALSE). UNVERIFIED: dates after 9999-12-31, `24:00:00`, non-zero-padded dates (the importer
maps these to `#VALUE!` too) and a `t="d"` cell with no `<v>` (imported empty). Probe run 2026-10-07 (plain forms), 2026-10-14 (the rest), Excel for the web.
VERIFIED for a `t="d"` cell in a 1904 workbook (`<workbookPr date1904="1">`, T-2515, Excel for the web,
2026-10-23): the text names a date, and the cell holds that date's serial in the 1904 system.
`2024-01-01T00:00:00` = 43830 (45292 in a 1900 workbook), `1904-01-01` = 0, time-only `12:00:00` = 0.5
(a day fraction in both systems), and a date before the 1904 epoch (`1903-12-31`, `1900-01-01`) is a
`#VALUE!` cell (`ISNUMBER` FALSE), as a date before 1900-01-01 is in a 1900 workbook. `TEXT(A1,"yyyy-mm-dd
hh:mm:ss")` shows the written date (`2024-01-01 00:00:00`, `1904-01-01 12:00:00` for the time-only cell).
Evidence: fixtures/t-d-1904, t-d-1904-epoch, t-d-1904-time-only, t-d-1904-pre-epoch, t-d-1904-1900-01-01.
UNVERIFIED: `1900-02-29` (the 1900 system's phantom day) in a 1904 workbook; it is before the epoch, so
the importer reads it as `#VALUE!` too.
Implemented in: `XmlNode::relationship_id` in crates/rustsheet-xlsx/src/import/xml.rs;
`iso8601_to_serial` and `iso8601_to_date_system_serial` in crates/rustsheet-xlsx/src/import/iso_date.rs
(shared with pivot cache `<d>` items and the eval snapshot reader)
Tasks: T-0112, T-0406, T-0407, T-2515

## Structured references in the file
Rule (excel-graph uploads of spec2xlsx files, 2026-10-16, T-1607): the file form of a structured reference
differs from what a user types, and Excel for the web refuses or misreads the typed forms.
- A bare table name is stored as `Sales[]`. A stored bare `Sales` (`=ROWS(Sales)`, as a dynamic-array or a
  legacy formula) makes the upload fail as corrupt (`createSession` 501 `FileCorruptTryRepair` /
  `unsupportedWorkbook`); `ROWS(Sales[])` loads and gives the row count.
- `@` is stored as `#This Row`: `Sales[[#This Row],[A]]`, `Sales[[#This Row],[A]:[B]]`, `Sales[#This Row]`.
  A stored `Sales[@A]` loads but evaluates to `#REF!`.
- The escape `'` goes before the special character: `Sales[C'#]` loads; `Sales['C#]` (a quote first) is
  refused as corrupt.
- A totals row needs its metadata: a `<tableColumn>` with `totalsRowLabel` for a label cell, or
  `totalsRowFunction="custom"` and `<totalsRowFormula>` for a formula cell (written as a plain `<f>`). A table
  with `totalsRowCount="1"` whose totals cells lack it (a label and a dynamic-array SUBTOTAL) is refused as
  corrupt; with it the file loads and `Sales[[#Totals],[Val]]` reads the totals cell.
- Excel's own file (crates/rustsheet-xlsx/tests/calc_test_tables/tables.xlsx, Excel-authored) also shows: a column
  name that ends with a space keeps its inner brackets, `SUBTOTAL(109,Table3[[the two ]])` (Hidden!B6); an `@`
  inside a column name is not escaped, `SUM(Table9[Totals '[']^&*@'#$,,<>])` (Escape!E6); `#Totals` of a table
  without a totals row is kept as written, `SUM(Table2[#Totals])` cached `#REF!` (Sheet2!F4). The engine's printer
  writes these forms (T-2196); a leading `@` is still escaped, since `[@` reads as `#This Row`.
- Table names that look like cell addresses (`T1`) are rejected by XlsxWriter; a header that is a number in
  text (`2024`) is fine.
Evidence: the corrupt-file refusals are upload errors, not cell values (throwaway probes during T-1607, not
kept; to re-run one, a sheet with table `Sales` at A1:D4 or A1:B4 and one formula: `=ROWS(Sales)` stored bare,
`=SUM(Sales['C#])` with header `C#`, or a totals row A4 "Total", B4 `=SUBTOTAL(109,Sales[Val])` without metadata); the forms that load are what gen-m8b's 3,368 filled cells use. tools/tests/test_spec2xlsx_tables.py
pins each written form.
Status: VERIFIED (web)
Implemented in: tools/src/rustsheet_tools/spec2xlsx.py (`table_refs_to_file_form`, `_totals_column`); the engine
reads `[#This Row]`; `Table[]` is T-1674; the xlsx export writes these forms (T-2196, "Table parts in the export").
Tasks: T-1607

## Table parts in the export
Rule (ours, T-2196): the export writes each table as `xl/tables/tableN.xml` with the forms of "Structured
references in the file" above: a totals-row formula cell as `totalsRowFunction="custom"` plus `<totalsRowFormula>`,
a totals-row text cell as `totalsRowLabel`, and formulas keep their structured references (`Sales[]` for a bare
name, `[#This Row]` for `@`). Excel itself writes `totalsRowFunction="sum"` (and the other named functions) for a
totals cell it built, with the cell formula `SUBTOTAL(109,Table1[Feb])` (calc_test_tables/tables.xlsx
`xl/tables/table1.xml`); `custom` with that formula is the form the T-1607 uploads showed Excel loads.
The import keeps only whether a table's `<autoFilter>` applies a filter (`has_filters`, which makes SUBTOTAL call
the sheet's hidden rows filtered), not its criteria, so the export writes an applied filter as a
`<filterColumn colId="0"/>` without a criterion. That is schema-valid and our own import reads it back as
filtered; whether Excel opens it without repair, and still calls the hidden rows filtered, is unprobed (T-2838).
A totals-row cell holding a number, boolean or error constant gets no metadata (T-1607 covered label and formula
cells only); whether Excel loads that is unprobed too (T-2838's probe). A structured reference that lowers to an
error because the table's metadata does not fit (an unparsable `ref`, an overflowing row count; directive 5) is
written as that error.
Evidence: crates/rustsheet-xlsx/tests/calc_test_tables/tables.xlsx (Excel-authored, 9 tables) now round-trips
(import, export, import) with the same values (generated test `test_tables_tables`) and the same formula text
(`excel_authored_tables_keep_their_formula_text`, export/test/test_tables.rs).
Status: UNVERIFIED (the exported file has not been opened in Excel)
Implemented in: crates/rustsheet-xlsx/src/export/tables.rs
Tasks: T-2196, T-2838

## A table part without a name attribute is named by its displayName
Rule: Excel opens a workbook whose `xl/tables/tableN.xml` has `displayName` but no `name` attribute (as
LibreOffice writes it), and its formulas reach the table by the displayName: `SUBTOTAL(109,Таблица1[Sum for Q3])`
in the totals row of `<table displayName="Таблица1" ref="A2:E13" totalsRowCount="1">` computes.
Evidence: poi/LIBRE_OFFICE-128382-0.xlsx Basic features!B13:E13 (19239.04, 25215.790000000005, 28393.92,
27527.01; excel-graph, 2026-10-04, eval/suites/poi/oracle/LIBRE_OFFICE-128382-0.oracle.json).
Ours: IMPLEMENTED (T-2208): `import/tables.rs` takes each of `name`/`displayName` from the other when one is
missing; a part with neither is still an import error. Tables stay keyed by `name`: which attribute resolves when
they differ is unprobed (T-2320).
Status: VERIFIED (web)
Tasks: T-2208

## Typed whitespace is saved in the `<f>` text; a line feed is written as raw CR LF bytes
Rule: the `<f>` text Excel saves keeps the whitespace FORMULATEXT shows, so a typed formula round-trips with its spaces:
`=1 + 2` is `<f>1 + 2</f>`, `=SUM( 1 )` is `<f>SUM( 1 )</f>`, `=IF(1, ,2)` is `<f>IF(1, ,2)</f>`, `=SUM(1,  )` is
`<f>SUM(1,  )</f>`, `=NOW( )` is `<f ca="1">NOW( )</f>`, `=SUM(, 1)` is `<f>SUM(, 1)</f>`. **A typed line feed is written as the two raw bytes CR LF** (no `&#13;` reference: 8 CR LF pairs in `<f>`/`<v>` text, 0 references, in
sheet1.xml), in the `<f>` and in FORMULATEXT's cached `<v>` alike; an XML parser's end-of-line normalization reads them
back as one LF, which is what the oracle sidecar shows. A writer emitting LF therefore matches. A space after the line
feed is kept. The drops FORMULATEXT shows (trailing whitespace, a run before a comma after an operand) are made before
the save. A structured reference is stored with its normalized spaces: typed `T[ [Col] ]` saves as `SUM(T[ Col ])`,
`T[ [#Data] , [Col] ]` as `SUM(T[ [#Data], [Col] ])`, and a this-row reference as `T[[#This Row],[Col]]`.
Evidence: the workbook Excel for the web saved for probes/t2786-whitespace-structured-refs-runs (excel-graph, 2026-10-26,
T-2786; cache key `input_sha256` 2161fcb9526ce30cb3c8c4df7b77633f621d88c0048bdac1b22c81cabb3f77c8, refetch with `rustsheet-oracle fill --force`), sheet1.xml H1:H17 and E4:E15. A typed `=T[#Headers]` (E13) is `#SPILL!` in the sidecar but `#VALUE!` in the saved array `<v>`; not relied on. Excel's whitespace semantics are in syntax.md.
Ours: export writes the tree with no whitespace (SPEC-0023 keeps the written runs for printing only), so the saved text is
canonical; the runs beside a missing argument are filed as T-2944; the export of kept whitespace as T-2948.
Status: VERIFIED (excel-graph)
Tasks: T-2786, T-2944, T-2801

## Cached volatile values are those of the last calculation, near the save time
Rule: a workbook's cached values for `TODAY()`, `NOW()` and their dependents are those of Excel's last
calculation before the save, so without the METADATA `NOW` convention (SPEC-0008) the closest clock for them
is the save time, `docProps/core.xml` `dcterms:modified`. That is UTC, while Excel's `TODAY()` uses the
author's local time, and the last calculation can come before the save: a `NOW()` value can differ, and a
`TODAY()` value can be off by a day for saves near midnight.
Evidence: crates/rustsheet-xlsx/tests/templates/wordle.xlsx `Wordle!N2`
(`INDEX(FILTER(Words!$A$2:$A$1000,Words!$B$2:$B$1000=TRUE),MOD(TODAY()-DATE(2024,1,1),Words!$C$2)+1)`) caches
"RHINO", the word for 2026-06-25, and `dcterms:modified` is 2026-06-25T13:36:18Z (Excel-authored; no oracle
run).
Ours: `rustsheet_xlsx::compare` pins such a workbook's clock to `dcterms:modified` for its comparison tests
(T-0468, decision 0117); the engine itself reads the system clock unless pinned.
Status: VERIFIED (desktop, one workbook)
Tasks: T-0468
