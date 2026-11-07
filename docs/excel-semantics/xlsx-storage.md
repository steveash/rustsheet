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

## A cell that stores no value is blank
Rule: a constant (formula-less) cell that stores no value reads as blank in Excel whatever its `t`:
`<c t="str"/>`, `<c t="s"/>`, `<c t="s"><v></v></c>`, `<c t="inlineStr"/>` (no `<is>`), `<c t="n"/>`,
`<c t="b"/>` and `<c t="e"/>` all give ISBLANK TRUE, ISTEXT FALSE, COUNTA 0 and TYPE 1. A cell that
stores the empty string is text: `t="s"` to an empty shared string, `<c t="str"><v></v></c>` (an empty
`<v>`, unlike a missing one) and `<is><t></t></is>` give ISBLANK FALSE, ISTEXT TRUE, COUNTA 1, LEN 0
and TYPE 2. So a missing `<v>` and an empty `<v>` differ for `t="str"` but not for `t="s"` (an empty
index names no string; it is not shared string 0). A whitespace-only index, `<c t="s"><v> </v></c>`, is
blank too (ISBLANK TRUE, COUNTA 0, TYPE 1), and a bare `<c t="inlineStr"><is/></c>` (an `<is>` with no `<t>`)
is the empty string, as `<is><t></t></is>` is (ISBLANK FALSE, ISTEXT TRUE, COUNTA 1, LEN 0, TYPE 2).
Evidence: fixtures/valueless-cell-forms (excel-graph, 2026-11-06, T-4148) and
fixtures/valueless-cell-forms-ws-index (excel-graph, 2026-11-07, T-4743).
Status: VERIFIED (excel-graph)
Implemented in: crates/rustsheet-xlsx/src/import/worksheets.rs (`is_valueless_constant`). A spill
member's cached value and `t="d"` keep their own rules.

## A merged range keeps the values of its covered cells
Rule: Excel keeps a value or formula stored in a covered (non-anchor) cell of a `<mergeCell>`
range and calculates with it: with A1:D1 merged over A1 "anchor", B1 "x", C1 7 and D1 the empty
shared string, `=B1` is "x", `=C1` 7, `=ISBLANK(B1)` FALSE, `=ISTEXT(D1)` TRUE, `=COUNTA(A1:D1)` 4,
`=SUM(A1:D1)` 7, `=COUNTBLANK(A1:D1)` 1, `=LEN(B1)` 1; a covered `<c t="str"/>` (no value) is blank
(`=ISBLANK(B2)` TRUE, `=COUNTA(A2:B2)` 1). A covered formula `=A1+1` in B1 of A1:B1 is recalculated
(2), `=B1` is 2, ISFORMULA TRUE and `=COUNT(A1:B1)` 2. This is the file-level form of the
"merge with Format Painter keeps the hidden values" behaviour.
Ours: the importer empties every covered cell (`clear_covered_cells`) because covered cells holding
no content is an engine invariant (merge, autofill and clipboard rely on it), so these cells are
visibly off and poi/bug66675.xlsx and other census files lose covered values on a round trip.
Evidence: fixtures/merged-covered-values, fixtures/merged-covered-formula (excel-graph, 2026-11-06, T-4148).
Status: VERIFIED (excel-graph); not implemented (owner task in the fixtures' baseline entries).
What Excel's UI merge then does with such a range (unmerge, edit, autofill) is UNVERIFIED.

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
A password-protected xlsx (an OLE compound file, SPEC-0031) has bounds of its own. The whole
container is refused past the byte budget (`max_uncompressed_bytes`, before `cfb` parses it), and a
container with more than 4,096 directory entries is refused as an unreadable OLE file. Its
`EncryptedPackage` may declare at most the byte budget, both as a stream and as the plain size in its
first 8 bytes, and `EncryptionInfo` at most 1 MiB, its Agile XML at most 64 elements nested 8 deep.
The key derivation may ask for at most `max_spin_count` rounds (`spinCount`), 1,000,000 by default,
before any hashing: MS-OFFCRYPTO allows 10,000,000 (about 2 to 4 s of SHA-512 that any file could
demand without knowing the password) and Excel writes 100,000, so a file with 1,000,001 to
10,000,000 rounds that Excel opens is refused with the `SpinCount` limit until the caller raises the
limit (SPEC-0031 D3): on the public API that is `LoadOptions::max_spin_count` (default 1,000,000) and
the refusal is `LoadError::Options`, raised before any hashing (T-4266). A load runs at most one key
derivation: only the password the caller gives is tried. Past 10,000,000 a file is malformed for the
format, not a limit.
Evidence: none for Excel; these are resource limits (CLAUDE.md directive 5), not Excel observations.
Status: DEVIATION (decision 0014)
Implemented in: crates/rustsheet-xlsx/src/import/package.rs (`ImportLimits`, `Package`),
crates/rustsheet-xlsx/src/import/xml.rs (`reference_of`), crates/rustsheet-xlsx/src/import/ole.rs
(`classify_ole_compound_file`, `decrypt_ole_package`, `MAX_DIRECTORY_ENTRIES`),
crates/rustsheet-xlsx/src/import/encryption.rs (`decrypt_package`, `MAX_SPIN_COUNT`)

## Password-protected and non-zip packages
Rule (Excel): a password-protected xlsx is an OLE compound file (signature `D0 CF 11 E0 A1 B1 1A E1`) holding an
ECMA-376 `EncryptedPackage`, not a zip. Excel asks for the password and then opens it. UNVERIFIED (probe pending,
T-4263): no oracle run has opened the two poi fixtures with their passwords (POI's own tests publish passtika's). Excel
also opens a legacy binary workbook (`.xls`, the same container) without a password.
Ours: a file that starts with that signature is refused before the zip is opened. Since T-4264 the container is
parsed (the `cfb` crate, over the bytes in memory, after they are bounded by `max_uncompressed_bytes`) and classified by
the streams of its root storage (SPEC-0031 D1); no stream is read:
- `EncryptionInfo` and `EncryptedPackage` (names compare case-insensitively): `XlsxError::EncryptedPackage` ("the file
  is a password-protected xlsx, which cannot be opened yet"). An `EncryptedPackage` that declares more than
  `max_uncompressed_bytes` is the uncompressed-bytes limit error instead.
- `Workbook` (BIFF8) or `Book` (BIFF5): `XlsxError::LegacyBinaryWorkbook` ("save it as .xlsx to open it").
- Anything else, a container with more than 4,096 directory entries, and a malformed container (a bad header, a sector
  chain or directory cycle): `XlsxError::OleCompoundFile`.
A stream whose declared size runs past the sectors it has is not detected here: `cfb` checks a chain against the size
only when the stream is read, so T-4265, which reads them, must treat a short read as malformed. In a version 3 file a
stream size is 32 bits (MS-CFB 2.6.3: the high half is ignored).
That is a known gap, not a deviation (decision 0279). SPEC-0031 (T-4243) plans the close: a password on the load
API, ECMA-376 Agile and Standard decryption (T-4265 to T-4267) and the `.xls` plan, which waits on an owner ruling
(T-4268). Any other file that is not a zip stays a zip error.
Since T-4265 the decryption exists below the load API: `decrypt_ole_package` reads the two streams (a stream that
ends before its declared size is `MalformedEncryption`, never a shorter package) and `decrypt_package` decrypts
Agile (4.4: AES-CBC with SHA-1 or SHA-2, its HMAC verified before any plain text is returned) and Standard (2.2,
3.2, 4.2 with `fCryptoAPI` and `fAES`: AES-ECB, SHA-1) packages. A password that does not open the package, or one
longer than 255 UTF-16 code units (refused, not truncated; E3 is UNVERIFIED), is `WrongPassword`; Extensible
encryption, RC4, a certificate-only key encryptor, a cipher other than AES-CBC and a hash other than SHA-1/SHA-2
are `UnsupportedEncryption`; a broken descriptor or package, and an Agile package whose HMAC does not match, are
`MalformedEncryption` (SPEC-0031 D5, until T-4263 shows whether Excel opens such a file). With the passwords
POI's tests use, `protected_passtika.xlsx` (`tika`) and `58616.xlsx` (`VelvetSweatshop`, the default password of
E2) both decrypt with msoffcrypto-tool 6.1.0, and both are Standard encryption (`EncryptionInfo` 4.2, AES-128).
Since T-4266 the load API calls it: `LoadOptions::password` (Python `password=`) decrypts the package before the
zip is opened, and the facade maps the importer errors to `LoadError::PasswordRequired` (no password given),
`WrongPassword`, `Unsupported(UnsupportedKind::{LegacyBinaryWorkbook, Encryption(kind), OtherOleFile})`, `Options`
(a limit) and `Format` (malformed or tampered), in Python `PasswordRequiredError` and `WrongPasswordError`
(subclasses of `LoadError`). No `VelvetSweatshop` retry exists: E2 is unverified (T-4263), so a file encrypted with
Excel's default password needs that password passed explicitly. A password given for a file that is not encrypted
is ignored.
- poi `58616.xlsx` and `protected_passtika.xlsx` (Apache POI's encrypted fixtures, `CDFV2 Encrypted`, version 3
  containers with the `\x06DataSpaces` storage, `EncryptionInfo` of 248 bytes and `EncryptedPackage`) are refused as
  `EncryptedPackage`. Their baseline lines stay `!InvalidCase` (now with that message), owned by T-4267.
- poi `deep-data.xlsx` is a corrupt package (17 local headers with garbled names, no end-of-central-directory record),
  not an encrypted one: it stays refused as a zip error (T-0489).
Status: UNVERIFIED (probe pending, T-4263); known gap owned by T-4265 to T-4267 (SPEC-0031).
Implemented in: `Package::new`, `is_ole_compound_file` (crates/rustsheet-xlsx/src/import/package.rs),
`classify_ole_compound_file` and `decrypt_ole_package` (import/ole.rs), `decrypt_package` (import/encryption.rs),
the `XlsxError` variants (error.rs); `Workbook::from_xlsx_bytes` and `import_load_error`
(crates/rustsheet/src/workbook.rs); the eval snapshot reader (crates/rustsheet-eval/src/xlsx_snapshot.rs). Tests:
crates/rustsheet-xlsx/tests/ole_container.rs, crates/rustsheet-xlsx/tests/encrypted_package.rs,
crates/rustsheet/tests/encrypted_load.rs, bindings/python/tests/test_facade.py; fuzz targets
`cfb_container`, `encrypted_package`.
Tasks: T-3972, T-4243, T-4263, T-4264, T-4265, T-4266, T-4267, T-4268

## A workbook whose indices dangle is refused outright
Rule: `Model::from_workbook`, which every load ends in (xlsx import, `Model::from_bytes`,
`UserModel::from_bytes`), refuses a workbook whose indices point at nothing the model would later read
unchecked: a cell, row or column style past `cellXfs` (or no `cellXfs` at all), a style's font, fill or
border past its list, a negative number format id, a shared-formula index past its list, a spill range
or anchor off the grid, a spill range of more than 1,048,576 cells (the array limit of arrays.md "Array results above
1,048,576 cells": a CSE anchor's whole range is filled on evaluation, so a crafted range of the whole sheet would write
2^34 cells; T-2896; a real file whose cached spill or CSE range is larger is refused too, unprobed), anchors that
together claim more spill cells than the workbook stores, past one whole array (a few bytes per anchor could otherwise
claim 2^20 cells each; the engine stores every cell of a spill and resets a blocked spill's range to its anchor, and the
622 array formulas in the repository's xlsx files store all 3,447 cells they claim, but what range Excel writes for a
#SPILL! anchor is UNVERIFIED, T-3436; the bound is the cells stored plus 2^20 per load; T-3426), array formula `ref`s that, when each anchor is
reached, cover more cells than one per 16 bytes of the workbook's sheets read so far plus 4 x 2^20 once per load. The
xlsx importer checks this itself, before the model exists, because it maps every covered cell to its anchor; worst
case, a hostile 256 MiB package still makes that map about 21M cells (1 to 1.5 GB), the accepted ceiling. An anchor
comes before its cells, so more than four whole-column spills anchored ahead of their cells (`=A:A+1` in five
columns of row 1) are refused even in a file Excel wrote; the 22 bytes a covered cell takes in Excel's files (it
writes `r` on every cell) and 14 without `r` only matter once the cells are read; T-3437), a row or column record or a merged range off the grid, and cell storage whose
rows or columns are out of order or off the grid. The error names the sheet and the index
(`Sheet 'Sheet1': cell R1C2 has style 9999, which does not exist`). Indices the model reads with a
checked lookup that tolerates a dangling value load as before: a shared-string index (it reads as
empty text; poi/sample-beta.xlsx, an Office 2007 beta package, imports with no shared strings), a
conditional format's dxf id, a named style's `xfId`, a defined name's local sheet id, a view's selected
sheet, the frozen panes, and a workbook with no sheets.
Excel: desktop Excel offers to repair a corrupt file and Excel for the web refuses one
(`FileCorruptTryRepair`, seen for other corruptions in `arrays.md` and `number-formats.md`); what either
does with these particular indices is not probed. Without the check the model panicked later (T-2882:
style 9999 at the next edit, formula index 77 with no shared formula at the next evaluation).
Evidence: none for Excel; this is the no-panic rule (CLAUDE.md directive 5), not an Excel observation.
Status: DEVIATION (decision 0014); what Excel does with these indices is UNVERIFIED (probe pending, T-3405: an
xlsx with a dangling `s=` and one with a `fontId` past `<fonts>`, on excel-graph; a spill or CSE range above 2^20
cells: T-3436)
Implemented in: crates/rustsheet-engine/src/workbook_check.rs (`check_indices`),
crates/rustsheet-xlsx/src/import/array_refs.rs (`ArrayClaims`, the importer's own check),
crates/rustsheet-engine/src/sheet_data.rs (`SheetData::check_layout`)

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
were refilled with the prefixed form.
GAMMALN and ISO.CEILING are stored bare, like CEILING (T-0417): Excel-authored
rustsheet-xlsx/tests/calc_tests/STATISTICAL/GAMMALN_GAMMALNPRECISE.xlsx (Microsoft Excel 16.0300) stores `GAMMALN(A2)`
beside `_xlfn.GAMMALN.PRECISE(A2)`, and MATH_AND_TRIGONOMETRY/FLOOR_CEILING.xlsx (Microsoft Excel Online) stores
`ISO.CEILING($A175,B$2)`; neither is on XlsxWriter 3.2.9's list. The exporter wrote both prefixed until T-0417, and so did
the case builder: Excel accepts the prefixed forms, since none of the 16,517 GAMMALN and 335 ISO.CEILING cells of the
eval suites' excel-graph oracle files built that way (for example probes-numeric/t1042-gamma-int-1-90 A91 `GAMMALN(1)` =
0, probes/t2715-ceiling-floor-subnormal-ratio G1) is `#NAME?`. The arms are gone, so the exporter and the case builder
write them bare (test `test_export_gammaln_and_iso_ceiling_have_no_xlfn_prefix`).
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

## Pivot cache dates below serial 61 use the 1899-12-30 epoch
Observed: in the pivot caches it saves, Excel writes a date or time below serial 61 as 1899-12-30 plus the serial, in
days (the OLE epoch). It does not use the 1900-system calendar that a `t="d"` cell is read with (section above:
`1900-01-01` = 1, `1899-12-31` is `#VALUE!`). Evidence (Excel-authored, docProps/app.xml Microsoft Excel 16.0300):
- corpus-bench/52379/1_52379_input.xlsx, pivotCacheDefinition1 field `Duration`. Source `Table1` is sheet FCR, column E.
  The field has `minDate="1899-12-30T00:00:08"` and `maxDate="1900-01-02T16:03:58"`, and records such as
  `1899-12-30T00:07:23`. The source cell FCR!E241 = 2.6748611111111114 (2 days 16:11:48), and the cache holds the item
  `1900-01-01T16:11:48`. The cache predates the sheet's current values (it holds items no cell matches), so the
  pairing is by value: E241 is the only source value of at least 1.
- corpus-bench/59932/1_59932_input.xlsx, field `Date`. It has `minDate="1899-12-30T00:00:00"`, and its `Quarters` and
  `Years` groups have `rangePr startDate="1899-12-30T00:00:00"`. Excel labels the first group item `<1/0/1900`, which is
  serial 0.
So within a pivot cache, `1899-12-30T00:00:00` is 0, a time alone is `1899-12-30T<time>`, and serials 1 to 60 come out
one day earlier than the cell calendar reads them. Since T-3182 the importer reads a cache `<d>` item dated 1899-12-30
to 1900-02-28 on the OLE epoch in a 1900 workbook (`pivot_cache_date_serial`), and later dates with the cell rule
(`iso8601_to_date_system_serial`); before, it rejected these dates and skipped the whole cache ("bad date"). The cutoff
at 61 is inferred from where the two calendars agree (1900-03-01), not observed. Serials 1, 2, 31, 59, 60 (the phantom
1900-02-29) and 61 are VERIFIED since T-4177 (below). UNVERIFIED: a cached `1900-02-29` (the importer reads it as 60,
as it reads `1900-02-28`); whether `rangePr` dates follow the same rule (since T-4225 the importer reads `startDate` and
`endDate` with it, so 59932's `1899-12-30T00:00:00` start is serial 0, which matches its `<1/0/1900` label);
a 1904 cache date before 1904-01-01 (the importer skips the cache). T-4177's GETPIVOTDATA cells (below) depend on it.
Implemented in: crates/rustsheet-xlsx/src/import/iso_date.rs (`pivot_cache_date_serial`), import/pivots.rs (`read_item`)
Implemented in: crates/rustsheet-xlsx/src/export/pivots/cache.rs (`date_text`, T-4176): the export writes the
inverse, a 1900 workbook's cache date as 1899-12-30 plus the serial and a 1904 workbook's as 1904-01-01 plus it, to
the second, so export then import is a fixed point.
Evidence (VERIFIED, excel-graph, 2026-11-07, T-4177): eval/suites/pivot-export t4177-cache-dates-1900.export.json, a
pivot over date cells 1, 2, 31, 59, 60 (the phantom 1900-02-29), 61 and 62, and t4177-cache-dates-1904.export.json,
the same in a 1904 workbook over 0, 1, 2, 59, 60, 61 and 1462. Without `refreshOnLoad` Excel reads the exported cache
(`1899-12-31T00:00:00` … `1900-02-28T00:00:00` for 60, `1900-03-01T00:00:00` for 61; `1904-01-01T00:00:00` for 0 …
`1908-01-02T00:00:00`) and GETPIVOTDATA on each serial finds its own item (1, 2, 4, … 64). With `refreshOnLoad`
patched on (the `-refresh` cases) Excel rebuilds the cache from the cells and writes those same dates itself, with
`containsNonDate="0"`, `minDate` the first date and `maxDate` a day past the last (`1900-03-03T00:00:00`,
`1908-01-03T00:00:00`). So serial 60 is `1900-02-28T00:00:00` in a cache, and serial 59 is `1900-02-27T00:00:00`, not
the cell calendar's 1900-02-28. Excel shows the refreshed items as dates (`2/29/1900` for 60; serial 0 of the 1904
workbook as `12:00:00 AM`), the engine as serials (T-4943).
Tasks: T-3182, T-4176, T-4177

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
The import keeps a table's `<filterColumn>` criteria (`filter_columns`: values filters with their blank choice,
date groups and `calendarType`, custom filters with `and`, values decoded from `_xHHHH_` escapes; T-2838) and, since
T-4004, top 10 (`top`, `percent`, `val`, `filterVal`), dynamic (`type`, `val`, `valIso`, `maxVal`, `maxValIso`), color
(`dxfId`, `cellColor`) and icon (`iconSet`, `iconId`) filters, each checked against its schema type (enumerations,
finite doubles, ISO dates, `unsignedShort` date fields; `filter_criteria.rs`). The export writes them back, so every
kind survives a round trip with the same meaning (Excel's own tables.xlsx filters come back byte for
byte; `blank="true"` is written `blank="1"`, `and="0"`, `top="1"`, `percent="0"` and `cellColor="1"` are dropped as
schema defaults, a `<filter>` without `val` is dropped, and a color filter's `dxfId` past the workbook's dxfs is
dropped, as the table's own dxf attributes are); column inserts and deletes inside the table move them with their
columns. A criterion that fails its check (an unknown element such as an `extLst`-only column, an enumeration
outside the schema, a non-finite number) keeps only its column and is written back as a criterion-less
`<filterColumn colId="N"/>`; a `<filterColumn>` without a valid `colId` is dropped, and the
`hiddenButton`/`showButton` attributes are not kept. Only a kept criterion inside the table makes SUBTOTAL call the
sheet's hidden rows filtered (T-4003). A table filtered through the API with no criterion (`has_filters`, no kept
filterColumn) is written, since T-4363, as a values filter on its first column listing the formatted text of each
visible data row (`blank="1"` when a visible one is blank; `export::tables::api_filter_column`), the form spec2xlsx
writes; before, it was a criterion-less `<filterColumn colId="0"/>`, which Excel and our import read as not filtered.
Excel reads our export as filtered (excel-graph upload of our export, 2026-11-07, T-4363: `Sales` A1:B7, `Cat`
a,b,a,c,a,b and `Val` 1-6, added with `filtered: true` and rows 3 and 5 hidden, exported as
`<filters><filter val="a"/><filter val="b"/></filters>`, "b" also on hidden row 3): `SUBTOTAL(9,Sales[Val])` and
`109` both 15, `SUBTOTAL(3,Sales[Cat])` and `103` both 4, the engine's values before the export, and our re-import
keeps the table filtered (`a_filtered_table_without_a_criterion_exports_its_visible_values`). Excel does not
re-apply the filter on load, so a text on a visible and a hidden row changes nothing there. Excel also reads the
`blank="1"` forms as filtered and an empty values filter as not (excel-graph uploads of our export, 2026-11-07, T-4749;
same `Sales` shape, `filtered: true`, SUBTOTAL 9, 109, 3, 103, 2, 102 read from Excel, every file opened without
repair):
- `Cat` a,b,(blank),c,a,b with rows 3 and 6 hidden, exported as `<filters blank="1"><filter val="a"/><filter val="b"/>
  <filter val="c"/></filters>` (input sha256 `dcb798ae88f44239595fe305cf9e909602faa77685bd247a170d6bcba82615d8`):
  14, 14, 3, 3, 4, 4, filtered, as ours;
- `Cat` (blank),b,(blank),c,(blank),b with rows 3, 5 and 7 hidden (only blanks visible), exported as
  `<filters blank="1"></filters>` (`a2018a38db40b36725a4470a1537b1a4ba9c631c97f48bb25f9c9c71da52eb4b`): 9, 9, 0, 0, 3,
  3, filtered, as ours;
- every data row hidden, exported before T-4749 as `<filters></filters>`
  (`7ae67c477d0072ee5bd4c6b131a7439e2d311a0e84944eaca4919b107787022a`): 21, 0, 6, 0, 6, 0, **not filtered**, where the
  engine and our import then gave 0 throughout. The same file with `<filters/>`
  (`4a21a4b32ca9f8110d72df3cef6000f0582cd83f10fca33619197341e4975efb`) or `<filters blank="0"></filters>`
  (`3584008b4af9e3a00c162bd3a1d8c4d11ca02174f527ccf598698f0ed25a9654`) reads the same, not filtered; with
  `<filters><filter val=""/></filters>` (`cd63953b7eda56d22bdf6c76b014b6308fb5cfcf89c2d13778db50438bee78ab`) Excel
  refuses the file (`FileCorruptTryRepair`); with a values filter listing the hidden rows' text
  (`be41612037aa55ecd2ef1353dcb69e616f7cdc6bc3819bb44f0573db0e9d1fbd`) or with `<customFilters and="1"><customFilter
  val="a"/><customFilter operator="notEqual" val="a"/></customFilters>`
  (`a2838913b0e16de165c1f47ea5878614e996b0edda29bca43205d3c99414e5ea`) it reads 0 throughout, filtered.
The same holds for a spec2xlsx table with every data row hidden (probes/t4749-empty-values-filter, 12 cells,
excel-graph, 2026-11-07: SUBTOTAL 3 6, 103 0, 9 21, 109 0, as the unfiltered control).
Rule (ours, T-4749): the import reads a `<filters>` with no element inside and no true `blank` as no criterion
(`is_criterion`; probed only as `<filters/>`, `<filters></filters>` and `<filters blank="0">`, so other attribute-only
forms such as one with just `calendarType` follow the rule unprobed), a column edit does not count such a filter as a
kept criterion (`FilterCriterion::is_empty_values`), the eval adapter does not count a spec values filter with no
visible data row, and the export writes an API-filtered table with no visible data row as that custom filter, which keeps
no row if re-applied (a values filter of the hidden rows' text would show them again).
Excel keeps the kept criteria's meaning after our round trip (excel-graph, 2026-11-05, T-4004): on `Sales`-shaped
tables (A1:B7, `Cat` a,b,a,c,a,b and `Val` 1-6, rows hidden as the filter would, no re-application) an
openpyxl-written file and our import-then-export of it give the same `SUBTOTAL(3,[Cat])`, `SUBTOTAL(103,[Cat])`,
`SUBTOTAL(9,[Val])` and `SUBTOTAL(109,[Val])`: a top 10 filter (top 2 of `Val`, rows 2-5 hidden) 2, 2, 11, 11; a
dynamic `aboveAverage` filter (rows 2-4 hidden) 3, 3, 15, 15; a color filter (`dxfId="0"`, rows 3 and 5 hidden) and
an icon filter (`3Arrows` icon 0, rows 3 and 5 hidden) 4, 4, 15, 15; a values filter holding only a `dateGroupItem`
(rows 3-7 hidden) 1, 1, 1, 1; and the criterion-less control (rows 3 and 5 hidden) 6, 4, 21, 15 in both. Before
T-4004 the first five exported criterion-less and read as manual hides.
Excel (excel-graph uploads of our export, 2026-11-04, T-3081) opens every one of these filter forms without repair,
but calls the hidden rows filtered only for a filterColumn with a criterion inside the table. On `Sales` (A1:B7,
`Cat` a,b,a,c,a,b and `Val` 1-6) with rows hidden as the filter would:
- a values filter (keep "a", rows 3, 5, 7 hidden): `SUBTOTAL(109,Sales[Val])` and `SUBTOTAL(9,Sales[Val])` both 9,
  `SUBTOTAL(103,Sales[Cat])` and `SUBTOTAL(3,Sales[Cat])` both 3, the same as ours; a custom filter (`Val` > 3, rows
  2-4 hidden) 15, 15, 3, 3, also ours; two filterColumns with the same `colId` 0 behave as the values filter;
- a criterion-less `<filterColumn colId="0"/>` (rows 3 and 5 hidden): SUBTOTAL 109 15 but SUBTOTAL 9 21, SUBTOTAL 103
  4 but SUBTOTAL 3 6, as for rows hidden by hand; ours called them filtered (9 = 15, 3 = 4) until T-4003, which reads
  every form in this list as Excel does (probes/t4003-filter-column-criterion). The same holds for
  a top 10 filter exported criterion-less (rows 2-5 hidden: 11 and 21, 2 and 6; ours 11 and 11, 2 and 2), so the round
  trip changed Excel's SUBTOTAL 9 from the original filter's 11 to 21 until T-4004 kept the criterion; for a values filter whose `colId` is
  past the table's width (5 on 2 columns: 9 and 21, 3 and 6; ours 9 and 9); and for the `colId="0"` fallback our
  export wrote after the filtered column is deleted (15 and 21, 3 and 6; ours 15 and 15; since T-4003 that delete
  unfilters the table and the export writes no filterColumn).
A color filter with no usable `dxfId` is no criterion (excel-graph, 2026-11-06, T-4404; `Sales`-shaped table, rows 3
and 5 hidden): the openpyxl file with `ColorFilter(dxfId=7)` on a workbook with no dxfs is refused (`createSession` 501
`FileCorruptTryRepair`), which is why the export drops a `dxfId` past the dxfs. Our export of it (`<colorFilter/>`) and a
directly written `<colorFilter/>` and `<colorFilter cellColor="0"/>` all open without repair and read as the
criterion-less control: `SUBTOTAL(9,[Val])` 21, `109` 15, `SUBTOTAL(3,[Cat])` 6, `103` 4, manual hides. Our import
therefore reads a `<colorFilter>` without `dxfId` as criterion-less, in `has_active_criterion` too (before, it kept
the criterion and called the rows filtered: 15 and 4). Probe case probes/t4558-color-filter-subtotal
(excel-graph, 2026-11-06, T-4558) pins the three forms (`<colorFilter/>`, `<colorFilter cellColor="0"/>`, no filter):
all read 6, 4, 21, 15. A source file with a dangling `dxfId` (past the workbook's dxfs) imports as unfiltered, the same
as its export, which drops the id: SUBTOTAL is equal in memory and after a save. Excel refuses such a source file, so
this is a choice that keeps the two equal, not an Excel value (T-4558).
A totals-row cell holding a number, boolean or error constant got no metadata until T-4005 (T-1607 covered label
and formula cells only), and Excel refuses such a file (`createSession` 501 `FileCorruptTryRepair` / `unsupportedWorkbook`): a
totals row with the label "Total" and the number 42 is refused, and so is the same file with 42 written as
`totalsRowLabel="42"` or as a custom `<totalsRowFormula>42</totalsRowFormula>`, while it loads with a SUBTOTAL formula
in that cell. **The form Excel loads for a constant is `totalsRowFunction="custom"` with no `<totalsRowFormula>`**
(excel-graph uploads, 2026-11-06, T-4005; `Sales` A1:B8 with `Cat`/`Val` data rows, A8 "Total" as `totalsRowLabel`,
`=Sales[[#Totals],[Val]]` read in D1): the number 42 loads and reads 42, TRUE (`t="b"`) TRUE and `#N/A` (`t="e"`)
`#N/A`, the cell kept a constant. Also loading, but changing the cell: the cell written as the formula `=42`
(`=TRUE`, `=#N/A`) with `<totalsRowFormula>` of the same text reads the same values; the text `"42"` with
`totalsRowLabel="42"` reads as text. Still refused: the number with `totalsRowLabel="42"`, and the number with
`custom` and `<totalsRowFormula>42</totalsRowFormula>`. The export writes the bare `custom` for a number, boolean or
error totals cell (T-4005); an upload of our export of such a table (`Item`, `N`, `B`, `E` with totals "Total", 42,
TRUE, `#N/A`) loads and `Sales[[#Totals],[N]]`, `[B]`, `[E]` read 42, TRUE, `#N/A`.
**A value spilled into the totals row** from a dynamic array beside the table takes the same bare `custom`
(excel-graph uploads of our export, 2026-11-07, T-4421; `Sales` B1:C5, `Cat` a,b,c and `Val` 1-3, totals row 5 with
B5 and C5 empty, the dynamic array in A5, `Sales[[#Totals],[Val]]` in E1, `[Cat]` in E2, `ROWS(A5#)*10+COLUMNS(A5#)`
in E5): before T-4421 the export wrote no metadata for B5 and C5 and Excel refused the file (`FileCorruptTryRepair`),
and refused it too with the spilled cells' `<c>` removed, the `<f t="array" ref="A5:C5">` alone. With bare
`totalsRowFunction="custom"` on both columns Excel loads it and keeps the spill, no `#SPILL!`: `=SEQUENCE(1,3)` spills
1, 2, 3, E1 reads 3, E2 2, E5 13, the same with no cached values in B5:C5 and with stale ones (91, 92, 93, and 77
in E1), so Excel recomputes them; `=HSTACK(1,"y","z")` (B5:C5 written `t="str"`) gives "z", "y", 13; `={1,TRUE,#N/A}`
gives `#N/A`, TRUE, 13; `=SEQUENCE(1,2)` (custom on `Cat` only, C5 empty) gives 0, 2, 12. Ours gives the same values
before and after the export (`a_value_spilled_into_the_totals_row_gets_custom_metadata`). A spill from above the table
cannot reach the totals row: it would cross the header row, whose cells are never empty (unprobed).
Recipe (throwaway, not kept, as T-1607's): openpyxl writes each table (`Table`, `AutoFilter`, `FilterColumn` with
`Filters`, `CustomFilters` or `Top10`, and for T-4004 `DynamicFilter`, `ColorFilter` with a
`DifferentialStyle` in `wb._differential_styles`, `IconFilter` and `Filters` of `DateGroupItem`s, hidden rows through
`row_dimensions`), the engine imports and exports it
(`rustsheet_xlsx::import::load_from_xlsx`, `Model::delete_columns` for the deleted-column case,
`rustsheet_xlsx::export::save_to_xlsx`), and `rustsheet-oracle recalc -b excel-graph <file>` uploads the export
(T-4004 also uploaded the openpyxl file, to compare). A
table named `T1` reads as a cell reference, not a table (`T1[Val]` is `#VALUE!`); the uploads used `Sales`. T-4005's
variants set openpyxl's `TableColumn` `totalsRowLabel`, `totalsRowFunction` and `totalsRowFormula` (`TableFormula`), with B8's `<c>` patched in the zip to `t="b"` or `t="e"` for the boolean and error.
A structured reference that lowers to an
error because the table's metadata does not fit (an unparsable `ref`, an overflowing row count; directive 5) is
written as that error.
Evidence: crates/rustsheet-xlsx/tests/calc_test_tables/tables.xlsx (Excel-authored, 9 tables) now round-trips
(import, export, import) with the same values (generated test `test_tables_tables`) and the same formula text
(`excel_authored_tables_keep_their_formula_text`, export/test/test_tables.rs).
Status: VERIFIED (web, T-3081, T-4005, T-4421): Excel loads every filter form, and a constant or spilled totals cell
with bare `custom` metadata; T-4003 and T-4004 own the filter differences.
Implemented in: crates/rustsheet-xlsx/src/export/tables.rs
Tasks: T-2196, T-2838, T-3081, T-4003, T-4004, T-4005, T-4404, T-4363, T-4421

## A calculated column is saved as `calculatedColumnFormula` plus one formula per row
Rule (excel-graph downloads of the T-2957 probes, 2026-10-27): a column Excel filled (references.md, "Calculated
columns") gets `<calculatedColumnFormula>` inside its `<tableColumn>`, plus a `dataDxfId`, and every data cell keeps
its own `<f>` and cached value. The column formula is the typed formula in file form:
`T[[#This Row],[Col]]*2` for a typed `=[@Col]*2` (the `@` written as `#This Row` with the table name, as in "Structured
references in the file"), `A2*2` for `=A2*2` typed into C2 (which row's A1 text is saved when the formula is typed below the first
data row is unprobed). Structured cells are written as plain `<f>` per
cell; the A1 fill as a shared formula (`<f t="shared" ref="C2:C4" si="0">A2*2</f>`). A re-fill replaces the column
formula (`*3` after references.md rule 2); an exception does not change it (`*2` is still saved after a
constant in C3 and `*3` typed into C4). A column that was not filled (rule 4) gets no `calculatedColumnFormula`.
A table that grew by typing beside it gets the new `<tableColumn name="Column1">` with the column formula, or
`name="0"` and none for the header-row case. With a totals row, the table keeps `totalsRowCount="1"` and the
label column's `totalsRowLabel`; the filled column gets no totals metadata.
A stored `calculatedColumnFormula` does not fill blank cells on load (next entry, T-2969).
Ours (T-3033): import keeps its text on `TableColumn::calculated_column_formula`; a typed fill stores the first data
row's formula in the same file form, and export writes the stored text, so an exception in a data row does not change
it. Whether Excel opens our exported column formula without repair is unprobed.
Through a row or column insert or delete, Excel saves the column formula that the edit made of the first data cell's
formula (VERIFIED, excel-graph 2026-11-06, the `xl/tables/table1.xml` of the cached workbooks of
probes/table-edits/t4274-calc-column-*, T on A1:C4 with C2 typed `=A2*2`): `B2*2` after insert_column A (table
B1:D4), `A2*2` after delete_column B (a column it does not read; table A1:B4), `A3*2` after insert_row 1 (table
A2:C5), and `A2*2` after insert_row 3 inside the data rows (table A1:C5; the cells below read `=A4*2`, `=A5*2`).
A delete of the first data row saves the next row's formula moved up: `A2*2` after delete_row 2 (table A1:C3, cells
`<f t="shared" ref="C2:C3">A2*2</f>`), not `#REF!*2` (t4274-calc-column-delete-row-2, carried from #3739 by T-4769;
its FORMULATEXT of the deleted `S!C2` from another sheet is `#REF!`, F!A1, ours `#ERROR!`, T-4770).
A structured column formula keeps its text: `T[[#This Row],[Col]]*2` after insert_column A, with FORMULATEXT
`=[@Col]*2` in the moved cells and their values 2, 4, 8 (t4274-calc-column-structured-insert-column-a). The row that
insert_row 3 adds inside the data rows gets the column's formula (its Calc cell is 0, the oracle's S!C3).
A row a row insert adds to the data rows gets the column formula filled down, and the stored text is the formula of
the first data row (VERIFIED, excel-graph 2026-11-06, probes/table-edits/t4692-*, scored through
`FORMULATEXT(INDEX(S!C:C,n))` and the cached `table1.xml`): insert_row 3 inside the data rows writes `<f>A3*2</f>`
into C3 (and twice gives `A3*2`, `A4*2`); insert_row 2 at the first data row writes `A2*2` into the new C2 and saves
`calculatedColumnFormula` `A2*2` (not the moved cell's `A3*2`); insert_row 5 at the totals row of A1:C5 writes `A5*2`
into the new data row; a structured column fills `T[[#This Row],[Col]]*2`. Each filled cell is saved with `s="1"`,
and that xf shows nothing (VERIFIED, excel-graph 2026-11-07, `xl/styles.xml` of t4692-insert-row-inside's cached
workbook, T-4744): cellXfs 1 is `numFmtId="0" fontId="0" fillId="0" borderId="0" xfId="0" applyNumberFormat="1"`,
xf 0 with only the apply flag added, and the Calc column's `dataDxfId="0"` is a dxf of `numFmt` 0 "General". Its
number format, font, fill and border are xf 0's, so no CELL("format"), number format or TEXT of the value can tell
the two apart. Whether a column whose dataDxf has another number format fills cells in that format is unprobed.
Ours (T-4274, T-4692): the row and column insert and delete rewrite the stored text as the first data cell's formula
is displaced (`Model::displaced_column_formulas`), read where that cell moved and printed in file form at the new first
data row (so an insert at the first data row keeps `A2*2`), and a row insert fills the stored formula into the rows it
adds to a table's data rows (`Model::fill_inserted_data_rows`), each cell keeping its own style (Excel's `s="1"` is
not matched; moot, nothing observable differs, T-4744); a text with a `[`
(a structured reference) keeps its text, and so does one whose first data cell the edit deletes (which matches
Excel's `A2*2` after delete_row 2; a delete of a cell the formula reads is unprobed, T-4771). Moves and
cell shifts leave the text as it is (unprobed). Ours read their rows too since T-4680 (before, the moved `[@Col]` cells were `#ERROR!`). An empty `<calculatedColumnFormula/>` reads as no column formula, and an
`array="1"` attribute is dropped on import (export writes the plain element): engine choices, what Excel does with
either is unprobed.
Evidence: the saved `xl/tables/table1.xml` and `xl/worksheets/sheet1.xml` of probes/t2957-fill-one-totals,
-refill-second, -constant-exception, -constant-then-refill, -existing-constant, -a1-reference and -expand-right-*
(the eval/.cache/oracle/excel-graph/<input_sha256>.xlsx of each sidecar).
Status: VERIFIED (excel-graph)
Implemented in: crates/rustsheet-xlsx/src/import/tables.rs, crates/rustsheet-xlsx/src/export/tables.rs, crates/rustsheet-engine/src/calculated_columns.rs
Tasks: T-2957, T-2958, T-2969, T-3033, T-4274, T-4680, T-4692, T-4769

## Excel does not fill a calculated column on load
Rule: a table part whose `<tableColumn>` holds `<calculatedColumnFormula>` does not make Excel fill the
column when the workbook opens: blank data cells stay blank, a cell holding the formula leaves its
neighbours blank, and a constant stays a constant. Only a formula typed into a data cell afterwards
fills the whole column (as in the entry above).
Evidence: probes/t2969-calculated-column-load (excel-graph, 2026-10-30), column formula
`T[[#This Row],[Val]]*2` on C:F of T A1:F5: FORMULATEXT is #N/A for every cell of C (all blank), for D3:D5
(only D2 holds the formula, `=[@Val]*2`) and for E2:E5 (E3 holds 99); F4 typed `=B4*3` fills F2:F5.
Status: VERIFIED (excel-graph)
Implemented in: yes (import only marks the column as calculated; nothing fills on load)
Tasks: T-2969, T-3033

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
`T[ [#Data] , [Col] ]` as `SUM(T[ [#Data], [Col] ])`, and a this-row reference as `T[[#This Row],[Col]]`. Excel does not read that saved text back: a stored `T[ Col ]` loads as `#REF!` (syntax.md, T-2800).
Evidence: the workbook Excel for the web saved for probes/t2786-whitespace-structured-refs-runs (excel-graph, 2026-10-26,
T-2786; cache key `input_sha256` 2161fcb9526ce30cb3c8c4df7b77633f621d88c0048bdac1b22c81cabb3f77c8, refetch with `rustsheet-oracle fill --force`), sheet1.xml H1:H17 and E4:E15. A typed `=T[#Headers]` (E13) is `#SPILL!` in the sidecar but `#VALUE!` in the saved array `<v>`; not relied on. Excel's whitespace semantics are in syntax.md.
Ours: export writes the tree as written, its kept whitespace included (T-2948), a line feed as the reference `&#xA;`,
which reads back as LF like Excel's raw bytes. The runs beside a missing argument and inside an empty call print back and
are saved (T-2944, T-2966). The legacy `@` rewrite on import and export decides on the formula with no whitespace; where one
of its shape tests does not look past the whitespace, the formula loses its whitespace, never a value (no shape is known to
fall back since T-2970: `=N( IF(1,{2;3}) )` keeps its spaces). Unprobed: what Excel saves for a typed leading run (`= 1+2`); we write it, and
reading stored text drops it (T-2784). The saved text beside `@`, LET names and a sign is derived, not probed (T-2971).
Status: VERIFIED (excel-graph)
Tasks: T-2786, T-2944, T-2801, T-2948, T-2970, T-2971

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

## Round trips

What Excel itself rewrites when it saves a workbook. This decides which differences the round-trip suites
(SPEC-0033) may normalize. A difference is normalized only once a row here shows Excel rewriting it on every save
(SPEC-0033 D4.4). Until then the candidates N1–N12 are scored as `changed`, `lost` or `added` and tagged, and
T-4503 probes them.

Status: UNVERIFIED (probe pending). Tasks: T-4503, T-4172, T-4509.

| Item | Excel's behavior | Evidence |
|---|---|---|
| N1–N12 | not yet probed | T-4503 |

Status: VERIFIED (excel-graph; desktop unprobed for NBSP). Tasks: T-2784, T-2949.

Excel drops a stored formula's leading spaces, TAB or LF on import (T-2784; syntax.md "Remaining whitespace shapes
(T-2949)"). A leading NBSP is refused by excel-graph instead; desktop Excel is unprobed. Dropping those leading spaces,
TABs or LFs is therefore no semantic difference (SPEC-0033 X5). A leading NBSP is not covered.

## A patched package keeps the source's hints
The patch tier of the source-preserving export (SPEC-0034 D3, T-4560, T-4561) rewrites only the changed `<c>`
elements. The rest of the package is kept as the source had it, apart from the edits below. Each assumes that Excel
opens the result without a repair:
- **Row `spans`.** ECMA-376 Part 1 §18.3.1.73 calls them an optimization hint. A new cell outside a row's span
  leaves the span stale.
- **`<dimension>`.** It grows to cover new cells but never shrinks. Readers that trust it, such as openpyxl's
  read-only mode, then see a range larger than the cells.
- **Empty rows.** Clearing every cell of a row leaves its `<row>` element in place. A row without cells is valid
  (§18.3.1.73) and is how Excel stores a row that has only a height or a style.
- **`calcChain`.** It is copied unchanged while no formula cell is added or removed. When the set of formula cells
  changes, the chain is dropped with its relationship and override, as the regenerating export drops it
  (SPEC-0034 E2, T-4561). The regenerating export never writes a chain, and that Excel opens such packages
  without one is equally unprobed. Excel's repair of a chain that names a non-formula cell ("Removed Records:
  Formula from /xl/calcChain.xml") is never relied on.
- **Added metadata.** A source with no `metadata.xml` of its own gets the export's parts once a patched cell needs
  `vm` or `cm`, with one relationship and one override each (T-4561). Their names, content types and relationship
  types match Excel's own (`tests/calc_tests/LOOKUP_AND_REFERENCE/DROP_TAKE.xlsx`), and the parts are what a
  regenerated save writes.

Evidence: oracle excel-graph, 2026-11-07, T-4890 (docs/research/xlsx-round-trip/excel-resave-probe-2026-11-07.md).
Excel for the web opened and saved all 104 patched saves of the fixtures and of 23 of the 24 sampled corpus cases (the
24th, an `.xlsm`, Graph cannot open even as the source): R0, R1/E6 and R1/all. Among them:
- 79 rewrote a worksheet;
- 38 changed (grew) a `<dimension>`;
- 2 hold a cell outside its row's `spans`;
- 15 copied a calcChain;
- 6 added the metadata parts;
- 52 appended style records;
- 23 appended shared strings.

51 of the 52 R0 saves come back from Excel's save the same as the source does after Excel's save, apart from the
ids, dates and customXml every SharePoint save writes. The 52nd has a calcChain too large to compare.
Status: VERIFIED (excel-graph; desktop not observed) for the spans, the dimension, a copied calcChain, the added
metadata parts, and appended styles and strings. UNVERIFIED (not exercised by the standard edits): a dropped calcChain and
rows emptied by clearing. Tasks: T-4560, T-4561, T-4623, T-4890, T-4180.

## An empty `dcterms:created` makes Excel refuse the package
Rule: Excel for the web refuses to open a package whose `docProps/core.xml` holds an empty
`<dcterms:created xsi:type="dcterms:W3CDTF"></dcterms:created>`. Graph reports createSession 501
`FileCorruptTryRepair`. The same package without that one element opens.
Evidence: oracle excel-graph, 2026-11-07, T-4890 (docs/research/xlsx-round-trip/excel-resave-probe-2026-11-07.md §3).
- The fixtures valueless-cell-forms R1/E5 save (sha256 788ba5e3c8c07445a3803a461175c6dfda052a56a564dbf7d2958b346625b0a9)
  is refused.
- Removing only that element (0120e27f60475870c510a84d458730052173d6aa1c3d59a75800ccdd8c984a74) makes it open.
- Removing core.xml (ff7146f3fc2236882167360b238ddbe76c875bb77d73fd7409374a849e94c24a) makes it open.
- Each of these one-part changes leaves it refused:
  - metadata.xml removed;
  - theme removed;
  - the source's workbook.xml;
  - the source's styles.xml;
  - `<t></t>` written as `<t/>`.

An empty `modified`, or another malformed W3CDTF value, is not probed.
Ours: the regenerating export writes the empty element whenever the source records no creation date
(`export/doc_props.rs` `get_core_xml`).
Status: VERIFIED (excel-graph; desktop not observed); not implemented (T-4909).

## An empty `cellStyles` makes Excel refuse the package
Rule: Excel for the web refuses to open a package whose `xl/styles.xml` holds `<cellStyles count="0"></cellStyles>`,
an empty CT_CellStyles. Graph reports `FileCorruptTryRepair`. The same package opens when the element is removed,
and also when it holds the Normal style (`<cellStyle name="Normal" xfId="0" builtinId="0"/>`).
Evidence: oracle excel-graph, 2026-11-07, T-4890.
- The poi link-external-workbook-a R1/all save (814c986a989f7227691ef5feae2077925ec1886d55c1dcc7ea2eb29044af4cb6) is
  refused. Its core.xml has a real `created` date, and it has no `[n]` references.
- With the element removed (5d941486ce51b2b3b71561c55f7615141666e81748e116d8885e1f7c41ab1310), it opens.
- With the Normal style added (73e6065fe084b0a5d3b3e189dcfb2bf4de950cf2509a5edc10af9ce3a61fdb0a), it opens.

Ours: the regenerating export writes the element with the table's count, even when the count is 0
(`export/styles.rs`, the `cellStyles` section).
Status: VERIFIED (excel-graph; desktop not observed); not implemented (T-4909).

## A [n] reference without its externalLink part makes Excel refuse the package
Rule: Excel for the web refuses to open a package whose formulas use `[1]Sheet!A1` while the package has no
`xl/externalLinks/externalLink1.xml` and no `<externalReferences>`. Graph reports `FileCorruptTryRepair`.
Evidence: oracle excel-graph, 2026-11-07, T-4890. Both directions were isolated on packages that do not have the empty
`created` element:
- **The part removed.** external-link-cached-text R1/E5 without that element
  (9b892ce5f3c370a5689191d44a07bcd4db327e375424caee63d4f425760f0d65) is refused. The same package with the source's
  externalLink part, its relationship, its override and `<externalReferences>` added back
  (299381fbc90f67fc3aace3ec4b50dc6f440e77cda96e6bb6dee3a5e4b861f5e5) opens.
- **The reference added.** valueless-cell-forms R1/E5 without the empty `created` opens. The same package with one
  `[1]Uses!$A$1` formula added (dbeb46a9ad85d763e6930dc0b2eff226684dcacc8943805549ff9ded3d46c6c7) is refused.

The R1/all refusals of the other six fixtures external-link-* also carry the empty `created` element, so they confirm
neither rule on their own.
Ours: the regenerating export drops the parts and keeps the formulas.
Status: VERIFIED (excel-graph; desktop not observed); not implemented (T-4143).

## Shared-string index with surrounding whitespace (T-4145)
A `t="s"` cell whose `<v>` holds its index with surrounding whitespace (`<v>2 </v>`) is read as that
index. poi/bug69769.xlsx Tabelle1!B2 stores exactly that, with the sst `Konto`, `Name`, `Mustermann`; the
importer used to read index 0 ("Konto"), the harness snapshot reader read "Mustermann".
Evidence: the file only (app.xml: Microsoft Excel Online 16.0300); no formula reads B2, so no cached value
shows how Excel resolves the index. We read it as index 2 on the strength of POI's bug report that Excel opened it.
Status: UNVERIFIED (probe pending, T-4711). Also unprobed: which characters count as whitespace (the code trims all
Unicode White_Space, not only XML whitespace) and whether a leading `+` is accepted (the code accepts it).
Implemented in: `parse_shared_string_index` in crates/rustsheet-xlsx/src/util.rs, shared by the importer
(`get_cell_from_excel`) and the snapshot reader (`decode_value`). A negative index (`<v>-1</v>`) stays a dangling index in both (empty text); an
unparseable index still loads as index 0 in the importer and as empty in the snapshot reader (not aligned; predates T-4145).

## `_xHHHH_` escapes in inline strings (T-4150)
An inline string (`t="inlineStr"`, `<is><t>`) decodes `_xHHHH_` escapes like a shared string does:
`<is><t>Rich Text_x000d_` + LF is `Rich Text` CR LF (poi/60289.xlsx TABLE!B2). The importer left them undecoded, so a
reload showed the literal `_x000d_` and the next save wrote `_x005F_x000d_`.
Evidence: the file and the harness snapshot reader, which has always decoded them; no Excel-computed value for an inline
string with an escape. Status: UNVERIFIED (Excel's decoding of escapes inside `<is>` is assumed to match `<si>`; a probe would
open a file with `<is><t>a_x0041_b</t></is>` and read `=A1`).
Implemented in: `get_cell_from_excel` (import/worksheets.rs), both the constant and the formula-value inline paths.


## externalLink parts: relationship order, sparse `sheetData`, cached text escapes (T-4143)
A workbook that links to other workbooks holds one `xl/externalLinks/externalLinkN.xml` part per link, the `N`th
`<externalReference>` of workbook.xml (between `<sheets>` and `<definedNames>`), so `[N]` in a formula names it. What Excel
writes, from the fixtures and the corpora:
- **Relationship order.** Excel numbers the workbook's relationship to each externalLink part right after the sheets'
  (eval/suites/fixtures/cases/external-link-cached-text.xlsx: sheets `rId1`–`rId3`, externalLink `rId4`, then theme, styles,
  sharedStrings). The export does the same and numbers the pivot caches after them; no evidenced file holds both, so
  that part of the order is RustSheet's choice. The part's own relationship is `externalLinkPath`, `TargetMode="External"`, with the target as the source
  wrote it: a bare file name (`56737.xlsx`, poi/ref-56737.xlsx) or a `file:///` UNC URL with `%20` escapes
  (corpus-bench/59932).
- **Sparse `sheetData`.** The cache does not hold a `<sheetData>` for every cached sheet: fixtures/external-link-uncached-lookups
  names 5 sheets and writes `sheetData` for sheets 0, 1, 2 (`refreshError="1"`) and an empty one for 4, none for 3. The
  import keeps which sheets had one (`ExternalBook::data_sheets`) and the export writes the same elements back.
- **Unread links.** A link the import could not read as a workbook (DDE, OLE, a missing part) is written as an empty
  `<externalLink/>`, so later books keep their `[n]`. Status: UNVERIFIED (whether Excel opens it without a repair is
  unprobed; T-4937).
- **Cached text.** A `t="str"` cache cell's `_xHHHH_` escapes are decoded as a shared string's are. Status: UNVERIFIED (no
  Excel-written cache with an escape was seen; the ST_Xstring rule is assumed to apply, as for inline strings above).
Evidence: the files named above (Excel-written); the round trip of every fixtures `external-link-*` case, poi/ref-56737 and
corpus-bench/59932 keeps the books and every cell value (crates/rustsheet-xlsx/tests/external_link_export.rs).
Implemented in: `export/external_links.rs`, `import/external_links.rs`, `export/workbook_xml_rels.rs`.
