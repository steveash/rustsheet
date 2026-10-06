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

## `&` binds looser than `+ - * /`
Rule: `=2.5+0.5&1E+3` adds first, then concatenates (`"3"&"1000"` = "31000"), so a printed
formula needs parentheses to keep a `&` operand of an arithmetic operator (`("a"&1)+1`, T-0157).
Evidence: gen-m1/57fe331cb7611dbf (`=(((0.1&1E+3)+100)<>...`) and the gen cases with `=2.5+0.5&1E+3`.
Status: VERIFIED

## Array constants take a single optional sign on a number literal
Rule: inside `{...}` an item is a literal, optionally preceded by one `+` or `-` that is directly
attached to a number. `{+1}` = 1, `{-1}` = -1, `{-1.5E1}` = -15, `{1,-2,+3}` sums to 2. Every other
sign form tested was a syntax error, so Excel for the web refuses the whole workbook on load (same
mechanism as `=IF(TRUE)` above): sign runs `{--1}`, `{++1}`, `{-+1}`, `{+-1}`, `{---1}`, `{1,--2}`;
a space after the sign `{- 1}`, `{+ 1}`; a sign on a boolean, text or error literal `{-TRUE}`,
`{+"a"}`, `{-#N/A}`, `{+#N/A}`.
Array-constant items that are expressions (`{"a"&CHAR(7)&"b","c"}`) are rejected the same way (found bisecting
the T-0957 probe, excel-graph 2026-10-05); the engine rejects them too (`parse_array_item`).
Evidence: eval/suites/probes/cases/array-constant-signs.case.json (oracle: excel-graph, 2026-10-05,
T-0147) for the accepted forms; each rejected form was sent as a single-cell workbook
(e.g. `{"sheets":[{"name":"Sheet1","cells":{"A1":"=SUM({--1})"}}]}`) and got createSession 501
`FileCorruptTryRepair` (rejected inputs cannot be committed; re-run such specs to reproduce).
RustSheet today: `{--1}`, `{---1}`, `{1,--2}`, `{-TRUE}` and `{-#N/A}` are rejected (T-0132 keeps each
minus as a node). Wrongly accepted: `{++1}`, `{-+1}`, `{+-1}`, `{+"a"}`, `{+#N/A}` (the parser
drops plus signs in `parse_power`) and `{- 1}`, `{+ 1}` (whitespace after the sign is lost before
parsing). Measured with the release build, T-0147.
Status: VERIFIED

## Formula nesting depth cap
Rule: RustSheet rejects a formula whose tree is deeper than `MAX_FORMULA_DEPTH` (256) as a parse
error ("Formula is nested too deeply"); typed into a cell through the API it is refused
(`EditError::InvalidInput`), and a workbook formula that deep loads as a parse error. Each operand
(number, reference, parenthesis, function call, array) costs two levels and each operator one, so up
to 127 nested parentheses or function calls and up to 255 terms in one operator chain (`=A1+A2+...`)
are accepted; union operand lists too (T-0134). Each unary minus is a node (T-0132), so a run of more than about 250 minus signs
(`=----...1`) is also a parse error; plus signs add no node. Excel caps function nesting at 64
levels (Microsoft, "Excel specifications and limits": "Nested levels of functions: 64") and formula
text at 8,192 characters, so every formula within Excel's nesting cap fits; but Excel accepts longer
operator chains (about 4,000 terms fit in 8,192 characters; lifting that is T-0128) and its
paren-nesting cap is not measured. Conversely RustSheet accepts 65 to 127 nested calls, which Excel
refuses per its published limit (UNVERIFIED: no oracle probe yet).
This bound keeps the recursive parser, evaluator, printer and tree drop from overflowing the stack
on untrusted formulas (CLAUDE.md directive 5); before T-0931, 200 nested parentheses crashed
the process. A new recursive consumer of the tree (or a heavier evaluator frame) must be
re-measured against this limit (the at-limit tests run on a 1 MiB stack in release builds).
Evidence: none for Excel's behavior beyond the published limits; this is a resource limit.
Status: DEVIATION (decision 0014)
Implemented in: expressions/parser/depth.rs (`MAX_FORMULA_DEPTH`, `Parser::parse_primary`, `Parser::grow`); model.rs (`Model::parse_cell_formula`)

## Input left over after a complete formula is a syntax error
Rule: text after a complete expression (`=1 2`, `=1+2 3`, `=SUM(A1:A2))`, `="a" "b"`, `=SIN(1)(3)`,
`=A1 1`, `=1 A1`) makes the formula invalid: Excel for the web refuses a workbook holding any of them
(`FileCorruptTryRepair`), so there is no Excel value. RustSheet stores it as a parse error, whose value
is `#ERROR!`. Before T-0105 the parser stopped after the first expression and silently
dropped the rest (`=A1:B2 B1:B3` was stored as `=A1:B2`). Spaces and line feeds between two
reference operands are not left-over input but the intersection operator. Which operands and separators
Excel accepts is in references.md ("Intersection operator: separators", "Intersection operands must
be references"). One exception: `=(A1)(A2)` with no space loads and is `#REF!`, so
smoke/trailing-input B6 (`#ERROR!`) is wrong (T-0149).
Evidence: scratch runs of `rustsheet-oracle recalc -b excel-graph` on one formula per workbook
(2026-10-05, T-0138); a refused workbook has no values to commit. `=(A1)(A2)`:
probes/intersection-operands H13 (oracle: excel-graph, 2026-10-05). smoke/trailing-input.case.json holds
RustSheet's hand expectations.
Status: VERIFIED (load-time refusal observed in Excel for the web only)
Implemented in: expressions/parser/reference_operators.rs (`Parser::expect_end`, `Parser::parse_intersection`)
Tasks: T-0105, T-0138, T-0149

## Newer error literals: only `#GETTING_DATA` loads
Rule: of the error names beyond the classic eight plus `#SPILL!`/`#CALC!`, Excel for the web accepts
only `=#GETTING_DATA` (also lower case `=#getting_data`) as a formula literal. A workbook with the
formula `=#BLOCKED!`, `=#CONNECT!`, `=#FIELD!`, `=#BUSY!`, `=#UNKNOWN!`, `=#PYTHON!`, `=#EXTERNAL!` or
`=#blocked!` is rejected on load (createSession 501 `unsupportedWorkbook`, "The workbook contains
unsupported features", the same refusal as 1E+308 literals in numbers.md), so those names are not
formula grammar there (they only arise as results, arrays.md).
`#GETTING_DATA` behaves as an error that is not `#N/A`: the Graph API reports the cell value as
`#N/A`, `ERROR.TYPE` is 8, `ISERR` TRUE, `ISNA` FALSE, `ISERROR` TRUE, `IFERROR(...)` takes the
fallback, but `IFNA(#GETTING_DATA,"y")` passes it through, `VALUETOTEXT` is the text
"#GETTING_DATA", `TYPE` is 16, and any comparison or `&` with it is the error (`#N/A` reported).
Wrapped in `ERROR.TYPE` (T-0195) the winner is visible: `#GETTING_DATA` is 8 and `#N/A` is 7, and
**the leftmost error operand wins** for `=`, `+`, `-`, `*` and `SUM` (the general rule is in arrays.md), whichever error it is:
`#N/A=#GETTING_DATA` is 7 but `#GETTING_DATA=#N/A` is 8; `#GETTING_DATA+#DIV/0!` is 8 but
`#DIV/0!+#GETTING_DATA` is 2 (same for `*`, `-`, `SUM(a,b)`); `#N/A+#GETTING_DATA` is 7 and
`#GETTING_DATA+#N/A` is 8; `#GETTING_DATA&"a"` and `#GETTING_DATA=#GETTING_DATA` are 8. No
`#GETTING_DATA` special precedence: it is an ordinary error kind.
Not probed (harness gap, T-0999): error constants as `t="e"` cell values, since spec2xlsx writes error
literals as formulas (`<v>#BLOCKED!</v>` cannot be emitted), so whether a stored `#BLOCKED!` etc.
cell loads is unknown. Also unprobed: what typing `#BLOCKED!` or `#GETTING_DATA` as cell input
gives, and whether a COUNTIF criterion `"#BLOCKED!"` matches the error (SPEC-0003 keeps today's
input table plus `#GETTING_DATA`, and keeps COUNTIF-family criteria on today's twelve names; T-1012
probes the criterion, T-1016 typed input on desktop; UNVERIFIED).
Evidence: probes/error-literals-new2 (S!A1:A15) and probes/error-literals-new (S!A1:A18), both excel-graph 2026-10-06; the eight single-cell
rejections were run as throw-away one-cell cases (not committed; no value to record).
Status: VERIFIED
Tasks: T-0175, T-0177, T-0195, T-0999, T-1010, T-1012, T-1016
