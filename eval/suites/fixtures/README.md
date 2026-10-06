# fixtures suite

Hand-built xlsx files for behavior a spec case cannot express (raw XML). The expected values are
Excel's, in `<file>.oracle.json` next to each workbook (`rustsheet-oracle fill -b excel-graph --suite
fixtures`; delete the sidecar to refill). The files claim `Application` = `RustSheet fixture` in
docProps/app.xml so that their placeholder cached values are never mistaken for Excel evidence.

- `shared-string-phonetic-runs.xlsx` (T-0170): `build_xlsx` of A1 = "Tokyo", B1 = `=LEN(A1)`,
  C1 = `=A1&"!"`, D1 = `=EXACT(A1,"Tokyo")`, then xl/sharedStrings.xml patched so the string is
  `<si><t>Tokyo</t><rPh sb="0" eb="5"><t>トウキョウ</t></rPh><phoneticPr fontId="0" type="Hiragana"/></si>`
  and the app name replaced.
