# Formula syntax

## IF needs at least two arguments
Rule: `=IF(x)` (one argument) is not a valid formula in Excel; `value_if_true` is required. A workbook whose formula text contains a one-argument `IF` is
rejected on load by Excel for the web, the whole workbook, not just that cell. The generator's
catalog therefore marks `value_if_true` as required (`eval/catalog/functions.yaml`, checked by
`tools/tests/test_catalog_arity.py`).
Evidence: single-cell workbooks sent with `rustsheet-oracle recalc -b excel-graph` on 2026-10-05:
`{"sheets":[{"name":"Sheet1","cells":{"A1":"=IF(TRUE)"}}]}` → createSession 501
`FileCorruptTryRepair`; the same with `=IF(TRUE,1)` loads and calculates. Re-run those two specs to
reproduce (rejected inputs cannot be cached or committed). Found in T-0203, where every gen-m2
workbook from the old catalog was refused; bisecting one isolated
`=MIN(ROUND(LEN(Data!$B5),6),IF(IFERROR(Data!C10,Data!$B3)),YEAR(59343))`.
Status: VERIFIED

## Unary minus binds tighter than `^`
Rule: `-x^y` is `(-x)^y`: `-1^2` = 1 and `-3^0` = 1 (not -1). A text operand of the minus is
coerced first: `-" 2 "^2` = 4, `-"0.5"^2` = 0.25, `-"1e3"^2` = 1000000, `-"TRUE"^2` = `#VALUE!`.
Unary minus also binds tighter than `&` (`-0&1.5` = "01.5", see numbers.md "Negative zero prints as 0").
Evidence: gen-m1/6462bf910ef9f28f Calc!A66, A130; gen-m1/10f579127ffb6d9b Calc!A13, A162, A156,
A153 (oracle: excel-graph, 2026-10-05, T-0109).
Status: VERIFIED
