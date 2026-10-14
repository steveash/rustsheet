# fixtures suite

Hand-built xlsx files for behavior a spec case cannot express (raw XML). The expected values are
Excel's, in `<file>.oracle.json` next to each workbook (`rustsheet-oracle fill -b excel-graph --suite
fixtures`; delete the sidecar to refill). The files claim `Application` = `RustSheet fixture` in
docProps/app.xml so that their placeholder cached values are never mistaken for Excel evidence.

- `shared-string-phonetic-runs.xlsx` (T-0170): `build_xlsx` of A1 = "Tokyo", B1 = `=LEN(A1)`,
  C1 = `=A1&"!"`, D1 = `=EXACT(A1,"Tokyo")`, then xl/sharedStrings.xml patched so the string is
  `<si><t>Tokyo</t><rPh sb="0" eb="5"><t>トウキョウ</t></rPh><phoneticPr fontId="0" type="Hiragana"/></si>`
  and the app name replaced.
- `workday-intl-legacy-holidays.xlsx` (T-1053): legacy (plain `<f>`, not dynamic-array) formulas,
  which a spec case cannot express (decision 0007). It was written with XlsxWriter `write_formula`
  and the app name replaced. C2 = 42429, D2:D5 = 42422, 42423, 42437, 42499; B2 and B7 =
  `=WORKDAY.INTL(C2,6,1,D2:D5)`, B3 = `=WORKDAY.INTL(C2,6,"0000011",D2:D5)`, B5 =
  `=WORKDAY(C2,6,D2:D5)`, B6 = `=NETWORKDAYS.INTL(C2,C2+14,1,D2:D5)`, and control E3 = `=D2:D5*1`
  (implicit intersection proves the formulas stayed legacy). The names are written without the
  `_xlfn.` prefix Excel itself would store for these Excel 2010 functions; Excel resolved them anyway.
- `ifs-legacy-range-criteria.xlsx` (T-0263): legacy formulas (XlsxWriter `write_formula`, app name replaced).
  A1:A6 = a,b,a,c,b,a, B1:B6 = 1,2,3,1,2,3, D1 = "a", D2 = "c"; rows 1 and 2 hold, with the range criterion
  `D1:D2`: F COUNTIF, G COUNTIFS, H SUMIF, I SUMIF (one-cell sum range B1), J SUMIFS, K AVERAGEIF,
  L AVERAGEIFS, M `_xlfn.MAXIFS`, N `_xlfn.MINIFS`; control O = `=B1:B6*1` (implicit intersection proves the formulas stayed
  legacy). MAXIFS/MINIFS carry the `_xlfn.` prefix Excel itself writes.
- `rich-error-export.xlsx` (T-1014): written by RustSheet's own exporter (app name then replaced), so
  it is evidence that Excel reads the exported rich-value form. A1:A5 are the constants `#SPILL!`,
  `#CONNECT!`, `#BLOCKED!`, `#FIELD!`, `#CALC!`, each stored as `t="e"` `#VALUE!` with `vm` 1-5 into
  `xl/metadata.xml` and one `_error` rich value per kind (`subType` 0); B = `ERROR.TYPE(A)`,
  C = `VALUETOTEXT(A)`, D = `ISERROR(A)`. Excel for the web loads it and gives 9, 10, 11, 13, 14.
- `rich-error-export-formulas.xlsx` (T-1148): the other shapes of the exported rich-value form, written by
  RustSheet's own exporter (app name replaced) from `set_user_input` of A1 `=FIELDVALUE(1,"Price")` (array
  anchor, `cm="1" vm="4"`), A2 `=IMAGE("https://example.com/a.png")` (plain formula with `vm="3"`),
  G1:G3 = 1, 2, 3 with D1 `=FILTER(G1:G3,G1:G3>5)` (anchor `cm="1" vm="5"`), I1 `=SEQUENCE(2)` (a spilling `cm`
  cell, no `vm`) and K1 `=SEQUENCE(2)` blocked by K2 = 5 (anchor `cm="1" vm="1"`), with ERROR.TYPE in B1, B2, E1, L1,
  `=SUM(I1:I2)` in M1 and VALUETOTEXT in N1:N4. Excel for the web loads it and gives #FIELD!, #BLOCKED!, #CALC!,
  #SPILL! and ERROR.TYPE 13, 11, 14, 9. `rich_error_fixtures_match_the_exporter`
  (crates/rustsheet-xlsx/tests/error_cell_export.rs) loads this and `rich-error-export.xlsx`, saves them again and
  requires the same `vm` cells, `xl/metadata.xml`, `xl/richData/*` and content-type overrides, so a change to the
  exporter's parts needs new fixtures and an oracle refill.
- `external-link-cached-ranges.xlsx` (T-1460): poi/ref-56737.xlsx (Apache-2.0, Excel-written, one
  `externalReference` to `56737.xlsx`) with its calcChain dropped, docProps replaced (app name
  `RustSheet fixture`), sheet `Refs` rewritten and `xl/externalLinks/externalLink1.xml` given a
  bigger cache: sheet `Uses` A1 "Hello!", B1 10, C1 `#N/A`, A2 5, B2 the text "7" (`t="str"`), C2 1,
  A3 TRUE; `Defines` A1 "Test A1", B2 142; workbook names `NR_Global_B2` (`'Defines'!$B$2`) and
  `NR_Range` (`'Uses'!$A$1:$B$3`), sheet-scoped (`Defines`) `NR_To_A1` and `NR_Local_Range`
  (`'Defines'!$A$1:$B$2`). `Refs` holds a label in column A and, in column B, one legacy formula
  per row (placeholder cached 0) over those cached ranges: aggregates, COUNTA, the reference
  readers, lookups, COUNTIF/SUMIF/COUNTBLANK, names, whole columns, unions. A formula over the
  cache's third sheet (`refreshError="1"`) is left out: Excel for the web refuses to open such a
  workbook.
- `strict-date-compatibility-0.xlsx` (T-0407): `t-d-iso-date-cells.xlsx` with `<workbookPr dateCompatibility="0">`
  (as in every Excel-saved Strict file). A1 is a `t="d"` cell `1900-01-01`; B1:B6 = `DATE(1900,2,28)`, `DATE(1900,3,1)`,
  `DATE(1900,2,29)`, `TEXT(60,"yyyy-mm-dd")`, `DAY(59)`, `DATEVALUE("1900-01-01")`; B7 = `A1`, C7 = `ISNUMBER(A1)`.
  Excel for the web gives 59, 61, 60, `1900-02-29`, 28, 1, 1, TRUE.
- `external-link-cached-text.xlsx` (T-1550): `external-link-cached-ranges.xlsx` with sheet `Refs`
  rewritten (one legacy formula per row in column B, placeholder cached 0) and the cached `Uses`
  sheet replaced: A1 the empty string (`t="str"`, `<v></v>`), B1 18, A2 12, B2 TRUE, A3 the text
  "8", A4 "abc", A5 30. It probes COUNTA over the cached empty string, GCD/LCM over cached text,
  booleans, the empty string and blanks, and SUM/COUNT/MAX/AVERAGEA over an area taller than the
  array bound.
