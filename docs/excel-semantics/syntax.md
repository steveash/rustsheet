# Formula syntax

## IF needs at least two arguments
Rule: `=IF(x)` (one argument) is not a valid formula in Excel; `value_if_true` is required. A workbook whose formula text contains a one-argument `IF` is
rejected on load by Excel for the web, the whole workbook, not just that cell. The generator's
catalog therefore marks `value_if_true` as required (`eval/catalog/functions.yaml`, checked by
`tools/tests/test_catalog_arity.py`).
Evidence: T-0203, 2026-10-04: every gen-m2 workbook from the old catalog was refused by excel-graph;
bisecting one workbook to a single formula isolated
`=MIN(ROUND(LEN(Data!$B5),6),IF(IFERROR(Data!C10,Data!$B3)),YEAR(59343))`; with `IF` given two
arguments all 30 regenerated workbooks loaded and filled. Observed on Excel for the web; the
rejected inputs are not committed because Excel cannot load them.
Status: VERIFIED
