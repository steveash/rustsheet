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
  (implicit intersection proves the formulas stayed legacy).
