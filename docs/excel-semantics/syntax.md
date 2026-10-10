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

## Minimum argument counts: a leading optional still takes its position, and PROB needs 3
Rule: a call with fewer arguments than a function's minimum makes Excel for the web reject the
whole workbook (as for `IF`, above). The minimum is not always what Microsoft's syntax line suggests:
`REDUCE([initial_value], array, lambda)` and `SCAN([initial_value], array, lambda)` need all three
positions (the optional `initial_value` is left empty, not dropped), and `PROB(x_range, prob_range,
[lower_limit], [upper_limit])` needs three although its page marks `lower_limit` Optional.
eval/catalog/excel-functions.tsv records the corrected counts in its `note` column.
Evidence: probes/catalog/availability-30 (PROB) and -32/-33 (REDUCE, SCAN) load and calculate with
three `$Z$1` arguments (oracle: excel-graph, 2026-10-07). With two, single-call workbooks sent with
`rustsheet-oracle recalc -b excel-graph` got createSession 501 `FileCorruptTryRepair` /
`unsupportedWorkbook` (2026-10-07, T-0040; rejected inputs cannot be committed). Every other
in-scope catalog function loaded at the count in excel-functions.tsv (probes/catalog/availability-*).
`XNPV(rate, values, dates)` needs all three: `=XNPV(1,A1:B2)` got the same 501
`FileCorruptTryRepair` (`rustsheet-oracle recalc -b excel-graph`, 2026-10-07, T-0275), while
`=XNPV(1,A1:B2,A1:B2)` loads and gives 9.962109338017495 (probes/xnpv-argument-count D1; D3 shows
values and dates of different lengths are `#NUM!`). RustSheet gives `#ERROR!` for a call below the
minimum, as for every other function (`fn_xnpv`).
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
RustSheet today: all of these forms are rejected (`parse_array_item` takes one sign directly attached to a number token; T-0156).
Status: VERIFIED

## Formula nesting depth cap
Rule: RustSheet rejects a formula whose tree is deeper than `MAX_FORMULA_DEPTH` (256) as a parse
error ("Formula is nested too deeply"); typed into a cell through the API it is refused
(`EditError::InvalidInput`), and a workbook formula that deep loads as a parse error. Each operand
(number, reference, parenthesis, function call, array) costs two levels and each reference operator
one, so up to 127 nested parentheses or function calls are accepted; union operand lists too
(T-0134). Each unary minus is a node (T-0132), so a run of more than about 250 minus signs
(`=----...1`) is also a parse error; plus signs add no node. Excel caps function nesting at 64
levels (Microsoft, "Excel specifications and limits": "Nested levels of functions: 64") and formula
text at 8,192 characters, so every formula within Excel's nesting cap fits; Excel's paren-nesting
cap is not measured. Conversely RustSheet accepts 65 to 127 nested calls, which Excel refuses per
its published limit (UNVERIFIED: no oracle probe yet).
Operator chains (`=A1+A2+...`, `&`, `* /`, `^` and comparisons, mixed or not) are not bounded by
depth (T-0128): every consumer walks a chain in a loop, so its links add no level. Instead a formula
may hold at most `MAX_VALUE_OPERATORS` (4,096) value operators ("Formula has too many operators");
Excel's published 8,192 character limit fits at most 4,096 of them (`n` terms take `2n-1`
characters), so no formula within the published limit is refused for its chains. Excel for the web
refuses a longer formula on load (T-1091, below), so RustSheet's cap never refuses a chain Excel
accepts. Probe `operator-chain-limit`
(T-1060, excel-graph): `=1+1+...` of 8,191, 8,192 and 8,193 formula characters (4,095 to 4,096
terms; `&` chains 4,093 to 4,094; a trailing `11` term pads a formula to its exact length) all evaluate (4105, 4096, 4106), and
so do `=LEN(1&1&...)` chains of the same lengths (4093, 4094, 4094); RustSheet matches all six.
Excel for the web loaded the 8,193 character formula from a file; that is 8,192 stored characters
without the leading `=`, exactly the published limit (typing and desktop Excel were not probed).
Probes of lone `=1+1+...` formulas of 8,194, 8,195, 8,196, 8,200, 16,384 and 32,767 characters
(T-1091, excel-graph; the cases were dropped after the run because Excel cannot load them, so no
sidecar can ever exist): every one made the workbook unloadable (createSession 501
`FileCorruptTryRepair`, so there is no Excel value). The spec-to-xlsx writer stores formulas without
the leading `=`, so the file text was 8,192 characters for the longest formula that loaded (T-1060's
8,193 including `=`) and 8,193 for the first refused: Excel enforces exactly the published 8,192
characters on file load. At
most 4,096 single-digit terms fit, within `MAX_VALUE_OPERATORS`, so the operator cap never refuses a
chain Excel accepts. RustSheet today has no formula text length limit: it loads and evaluates
chains of 8,194 and 8,195 characters (4,096 operators) and any longer formula under the operator
cap, where Excel refuses the workbook (an engine extension; follow-up task to decide whether to
refuse formulas over 8,192 stored characters on load).
Before T-0128 chains were capped at 255 terms.
These bounds keep the recursive parser, evaluator, printer and tree copy/drop from overflowing the
stack on untrusted formulas (CLAUDE.md directive 5); before T-0931, 200 nested parentheses crashed
the process. A new recursive consumer of the tree (or a heavier evaluator frame) must be
re-measured against these limits (the at-limit tests run on a 1 MiB stack in release builds), and
must walk operator chains with `expressions/parser/chain.rs`.
Evidence: probes/operator-chain-limit (chains at and one character past the published limit) and the T-1091 refusals above; otherwise only the published limits; this is a resource limit.
Status: DEVIATION (decision 0014)
Implemented in: expressions/parser/depth.rs (`MAX_FORMULA_DEPTH`, `MAX_VALUE_OPERATORS`, `Parser::parse_primary`, `Parser::grow`, `Parser::grow_chain`); expressions/parser/chain.rs; model.rs (`Model::parse_cell_formula`, `Model::evaluate_operator_chain`)

## Input left over after a complete formula is a syntax error
Rule: text after a complete expression (`=1 2`, `=1+2 3`, `=SUM(A1:A2))`, `="a" "b"`, `=SIN(1)(3)`,
`=A1 1`, `=1 A1`) makes the formula invalid: Excel for the web refuses a workbook holding any of them
(`FileCorruptTryRepair`), so there is no Excel value. RustSheet stores it as a parse error, whose value
is `#ERROR!`. Before T-0105 the parser stopped after the first expression and silently
dropped the rest (`=A1:B2 B1:B3` was stored as `=A1:B2`). Spaces and line feeds between two
reference operands are not left-over input but the intersection operator. Which operands and separators
Excel accepts is in references.md ("Intersection operator: separators", "Intersection operands must
be references"). One exception: `=(A1)(A2)` with no space is not left-over input but a call
of the parenthesized operand, and is `#REF!` (references.md "Calling a parenthesized operand",
T-0149; smoke/trailing-input B6).
Evidence: scratch runs of `rustsheet-oracle recalc -b excel-graph` on one formula per workbook
(2026-10-05, T-0138); a refused workbook has no values to commit. `=(A1)(A2)`:
probes/intersection-operands H13 (oracle: excel-graph, 2026-10-05). smoke/trailing-input.case.json holds
RustSheet's hand expectations.
Status: VERIFIED (load-time refusal observed in Excel for the web only)
Implemented in: expressions/parser/reference_operators.rs (`Parser::expect_end`, `Parser::parse_intersection`)
Tasks: T-0105, T-0138, T-0149

## Newer error literals: only `#GETTING_DATA` loads as a formula literal (stored: also `#BUSY!`)
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
Stored constants (`<c t="e"><v>#BUSY!</v></c>`, spec2xlsx `{"t":"e","v":"#BUSY!","stored":true}`, T-0999)
are different from formula literals: of the newer names only `#BUSY!` (and `#GETTING_DATA`, `#N/A`) load.
A workbook storing `#BLOCKED!`, `#CONNECT!`, `#FIELD!`, `#UNKNOWN!`, `#CALC!` or `#SPILL!` is rejected on load
(createSession 501 `unsupportedWorkbook`), like the formula literals (`=#CALC!` itself loads). A
stored `#BUSY!` is an error that is not `#N/A`: `ISERROR` and `ISERR` TRUE, `ISNA` FALSE, `TYPE` 16, `IFNA`
passes it through, `A1&""` is `#BUSY!`, and `ERROR.TYPE` is 0 (not 7; confirmed by `ERROR.TYPE(A1)+100` =
100, since XlsxWriter caches 0 for every formula and a stale cell would be indistinguishable from the bare
0). `#GETTING_DATA` stored: `ERROR.TYPE` 8, `ISERR` TRUE, `ISNA` FALSE, `TYPE` 16, `IFNA` passes through
(reported `#N/A`), `&""` is reported `#N/A`; stored `#N/A`: 7, `ISNA` TRUE, `IFNA` gives the fallback.
The engine-only kinds `#CIRC!`, `#ERROR!` and `#N/IMPL` stored as constants are rejected on load the same way (createSession 501 `unsupportedWorkbook`, excel-graph, 2026-10-08, T-1133; throw-away one-cell cases, no sidecar), so the exporter writes them as `#VALUE!` (`exported_error_literal`, `crates/rustsheet-xlsx/src/export/worksheets.rs`).
Unprobed: `#PYTHON!`, `#EXTERNAL!` stored. The rejected names were re-run as throw-away one-cell cases
(`A1` = `{"t":"e","v":"<name>","stored":true}`, nothing else), so to reproduce one, write that case.
Also unprobed: what typing `#BLOCKED!` or `#GETTING_DATA` as cell input gives. SPEC-0003 keeps
today's input table plus `#GETTING_DATA` (implemented in T-1010: typed `#GETTING_DATA` is the error,
typed `#BLOCKED!` is text); T-1016 probes it on desktop; UNVERIFIED. The COUNTIF criterion question
is settled by T-1012 (information.md): `"#BLOCKED!"` does not match the error.
Stored `#BUSY!` design: SPEC-0003 "Stored `#BUSY!`" (T-1020). It gets its own kind with ERROR.TYPE 0,
stays out of the formula, input and criteria tables, and is exported as is (T-1222).
Stored `#BUSY!` behaviors (probes/error-stored-busy-behaviors, excel-graph 2026-10-10, T-1228; A1 and A4 are
the stored error, A2 = 1): `VALUETOTEXT(A1)` and `VALUETOTEXT(A1,1)` are the text "#BUSY!" (C1, C3);
`ARRAYTOTEXT(A1)` is the *error* `#BUSY!`, not text (C2; the same for stored `#N/A`, C17: a single cell that
only propagates the error, see arrays.md), while `ARRAYTOTEXT(A1:A2,1)` is "{#BUSY!;1}" (C4).
For the operators the leftmost error wins and `#BUSY!` has no special rank: `ERROR.TYPE(A1+#DIV/0!)` 0, `ERROR.TYPE(#DIV/0!+A1)` 2 (C5, C6),
likewise for `=` (C7 0, C8 2) and `*` with `#N/A` (C9 0, C10 7). In SUM an error in a direct argument beats an error in a
referenced cell whichever comes first (`SUM(A1,1/0)` and `SUM(1/0,A1)` are both ERROR.TYPE 2, C11, C12), see "SUM and MAX:
direct-argument errors beat cell errors" below. COUNTIF: `"#BUSY!"` and the cell itself as criteria count 0 (unlike an `#N/A` criterion, criteria.md, which counts `#N/A` cells), `"<>#BUSY!"` counts both
cells (C13-C15). `IFERROR(A1,"y")` is "y" (C16). `ERROR.TYPE(A1:A2)` spills 0 then `#N/A` (E1:E2).
`UNIQUE` keeps a stored `#BUSY!` and a stored `#NULL!` as distinct rows and merges equal `#BUSY!`s
(C18-C20: 2, 3, 3 rows).
Evidence: probes/error-literals-new2 (S!A1:A15) and probes/error-literals-new (S!A1:A18), both excel-graph 2026-10-06; the eight single-cell
rejections were run as throw-away one-cell cases (not committed; no value to record);
probes/error-stored-constants (S!A1:I3, excel-graph 2026-10-06) for the loading stored names, the five
stored rejections as throw-away one-cell cases (same); the three engine-only stored rejections (`#CIRC!`, `#ERROR!`, `#N/IMPL`) likewise (excel-graph 2026-10-08, T-1133).
Status: VERIFIED
Tasks: T-0175, T-0177, T-0195, T-0999, T-1010, T-1012, T-1016, T-1020, T-1133, T-1014, T-1222, T-1228

## SUM and MAX: an error in a direct argument beats an error in a referenced cell
Rule: in `SUM` and `MAX` an error produced by a direct argument (the literal `#DIV/0!` or a computed `1/0`) wins over any error held in a
referenced cell, whichever argument comes first; among referenced cells the leftmost wins. This is for aggregate functions: the
operators `+`, `=`, `*` take the leftmost error whatever its source (`A3+1/0` is #VALUE!, `1/0+A3` is #DIV/0!). It holds for a stored
`#BUSY!`, `#N/A` and `#VALUE!` cell alike (so `#BUSY!` has no special rank). With A3 = stored `#VALUE!`, A4 = stored `#REF!`, A6 = stored `#NUM!`
(ERROR.TYPE of each result): `SUM(A3,1/0)` and `SUM(1/0,A3)` 2 (C11, C12); `SUM(A3,#DIV/0!)` 2 (C9); `SUM(A3,A6,1/0)`,
`SUM(1/0,A3,A6)` and `SUM(A3,1/0,A6)` 2 (C17-C19); `MAX(A3,1/0)` and `MAX(1/0,A3)` 2 (C20, C21); `SUM(A3,A4)` 3 but `SUM(A4,A3)` 4
(C13, C14, leftmost reference); `SUM(A3:A4)` and `SUM(A4:A3)` both 3 (C15, C16, a range is read top to bottom whichever way it is written).
The "leftmost error operand wins ... `SUM(a,b)`" sentence in "Newer error literals" above rests on direct-versus-direct pairs, so both hold. Not probed: AVERAGE, MIN, PRODUCT and the
other aggregates, and an error inside an array constant or a nested function result.
Evidence: eval/suites/probes/cases/sum-error-precedence.case.json C1-C23 (oracle: excel-graph, 2026-10-10, T-1315).
Status: VERIFIED
Tasks: T-1315, T-1317

## FORMULATEXT keeps every written parenthesis
Rule: Excel keeps redundant parentheses as written in the stored formula; FORMULATEXT prints every level back in every
probed shape (plain operators, unary minus, intersection, function arguments and `@` operands): `=A1+(B1)`, `=SUM((A1)+(B1))`, `=(A1) (B1)`, `=-(A1)`, `=(A1)*((B1))`, `=A1+(B1*C1)`,
`=((A1))`, `=IF((A1>0),(B1),C1)`, `=@(A1+(B1))`, `=@((A1) (B1))`, `=@(((A1)) B1)`, `=@((A1)+(B1))`,
`=@(A1+((B1)))`, `=@((A1+B1))`, `=@(A1 B1) C1` and `=@(A1):B1 C1` all print as stored. Whether typed input is
kept the same way is UNVERIFIED (excel-graph loads file text; an M14 desktop question, decision 0039). The values are unaffected
(`=@(A1+(B1))` = 3, `=@((A1) (B1))` = `#NULL!` with A1 = 1, B1 = 2).
Evidence: probes/at-operand-nested-groups (36 cells, excel-graph, 2026-10-09, T-1178).
Ours: the parsed formula keeps no groups except the levels around a whole `@` operand, a whole parenthesized formula
(`GroupedRootKind`) and a few cut/union shapes, so the printer re-derives parentheses from precedence: `=A1+(B1)` prints
`=A1+B1`, `=-(A1)` prints `=-A1`, `=@(A1+(B1))` prints `=@(A1+B1)`. All 18 values match; 14 FORMULATEXT cells differ.
Status: VERIFIED
Tasks: T-1178, T-1244 (spec for keeping groups in the parse tree), T-1200 (the `@` group that is not the whole operand)
