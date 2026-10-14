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
has no Function variant for the last three or for GROUPBY/PIVOTBY (not on XlsxWriter's list), so nothing is
exported for them. PERCENTOF has an arm since T-0416. The case builder (which reads these arms) now prefixes it, and a
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
ignoring the attribute is correct and no follow-up is needed. UNVERIFIED (no probe): a trailing `Z`, fractional
seconds, `1900-02-29` (serial 60), dates before 1900-01-01 and time-only text; the importer reads the
first two as `serial + fraction` and leaves the rest empty. Probe run 2026-10-07, Excel for the web.
Implemented in: `XmlNode::relationship_id` in crates/rustsheet-xlsx/src/import/xml.rs;
`iso8601_to_serial` in crates/rustsheet-xlsx/src/import/iso_date.rs (shared with the eval snapshot reader)
Tasks: T-0112, T-0406, T-0407
