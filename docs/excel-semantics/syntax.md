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

## Formula nesting depth cap
Rule: RustSheet rejects a formula whose tree is deeper than `MAX_FORMULA_DEPTH` (256) as a parse
error ("Formula is nested too deeply"); typed into a cell through the API it is refused
(`EditError::InvalidInput`), and a workbook formula that deep loads as a parse error. Each operand
(number, reference, parenthesis, function call, array) costs two levels and each operator one, so
up to 127 nested parentheses or function calls and up to 255 terms in one operator chain
(`=A1+A2+...`) are accepted. Excel caps function nesting at 64 levels (Microsoft, "Excel
specifications and limits": "Nested levels of functions: 64") and formula text at 8,192
characters, so every formula within Excel's nesting cap fits; but Excel accepts longer operator
chains (about 4,000 terms fit in 8,192 characters; lifting that is T-0128) and its paren-nesting
cap is not measured. Conversely RustSheet accepts 65 to 127 nested calls, which Excel refuses
per its published limit (UNVERIFIED: no oracle probe yet).
This bound keeps the recursive parser, evaluator, printer and tree drop from overflowing the stack
on untrusted formulas (CLAUDE.md directive 5); before T-0931, 200 nested parentheses crashed
the process. A new recursive consumer of the tree (or a heavier evaluator frame) must be
re-measured against this limit (the at-limit tests run on a 1 MiB stack in release builds).
Evidence: none for Excel's behavior beyond the published limits; this is a resource limit.
Status: DEVIATION (decision 0014)
Implemented in: expressions/parser/depth.rs (`MAX_FORMULA_DEPTH`, `Parser::parse_primary`, `Parser::grow`); model.rs (`Model::parse_cell_formula`)
