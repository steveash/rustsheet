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
A call with more arguments than the maximum is refused the same way, so there is no Excel value to
match; RustSheet gives `#ERROR!` (e.g. two-argument `HEX2DEC` and `OCT2DEC`, T-1383).
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
Typed (API `Input::Formula`/`Parse`), Excel for the web refuses a call with fewer or more arguments than the function
takes: `=PV(1)` (Graph PATCH 400, T-2567), and `=SUM()`, `=IF(1)`, `=ABS(1,2)` (too many) and
`=ACCRINTM(DATE(2020,1,1),DATE(2021,1,1),0.1)` (`par` is required) (Graph PATCH 400 `InvalidArgument`, 2026-10-25, T-2632;
the control `=ABS(1)` is accepted; refused inputs cannot be committed). A count between the least and most that ends inside a
repeating group is refused too: `=SORTBY(C1:C2,D1:D2,1,E1:E2)` (the second pair lacks its sort order; same probe). The engine
refuses typed by the catalog's counts for every function (`functions/kinds/arg_counts.txt`: the minimum of excel-functions.tsv; the
maximum of Microsoft's syntax line, 255 when it repeats, widened by a non-repeating functions.yaml signature, or a cited
override in place of both: INDEX's reference form, `INDEX(reference,row,column,area_num)`, 4 (probes/index-area-num-snap); checked by tools/tests/test_catalog_arity.py) and, for lifted functions, whole
`repeat` cycles (`ArgKinds::accepts`); stored text keeps such a call (`#ERROR!`).
Maxima and pairs, typed (T-2682, Graph PATCH 2026-10-25). Excel accepts each call at its maximum: `XLOOKUP` 6, `IF` 3,
INDEX's reference form 4, `FIND`/`FINDB`/`SEARCHB` 3, `LEFTB`/`RIGHTB` 2, `LENB` 1, `MIDB` 3, `REPLACEB` 4, and
`TRANSLATE` 3 (`#BUSY!` at the PATCH, logged, no sidecar). It also accepts whole pairs, `IFS(TRUE,1)`,
`IFS(FALSE,1,TRUE,2)`, `LET(x,1,x)` and `LET(x,1,y,2,x+y)`, and `SWITCH(1,1,2)` and `SWITCH(1,2,3,4)` (a default after
the pairs). See probes/t2682-typed-max-arg-counts A1:A17.
It refuses (400 `InvalidArgument`, refused inputs cannot be committed) one argument past each of those maxima
(`=XLOOKUP(1,{1,2},{3,4},0,0,1,1)`, `=IF(TRUE,2,3,4)`, `=INDEX((Sheet1!B1:C2,Sheet1!D1:E2),1,1,2,1)`, `=FIND("a","abc",1,1)` and the
same for FINDB, SEARCHB, LEFTB, RIGHTB, LENB, MIDB, REPLACEB, `=DETECTLANGUAGE("hello","x")`,
`=TRANSLATE("hello","en","es","x")`) and a count that ends inside an IFS or LET pair (`=IFS(TRUE,1,2)`,
`=IFS(FALSE,1,TRUE,2,3)`, `=LET(x,1,y,2)`, `=LET(x,1,y,2,z,3)`) and `=SWITCH(1,2)`, below its minimum of 3. The engine's table
takes these maxima as cited overrides (tools/tests/test_catalog_arity.py `MAX_OVERRIDES`; DETECTLANGUAGE and TRANSLATE are
not implemented, `#NAME?`), and `arg_counts::takes_count` refuses IFS and LET off whole pairs. UNVERIFIED: that `DETECTLANGUAGE("hello")` (1, the catalog minimum) is accepted
typed (only the refusal at 2 is logged); the pair rule above 6 arguments (probed
IFS 2 to 5, LET 3 to 6).
The repeating maxima, typed (T-2709, excel-graph 2026-10-25): `SUM` and `MAX` take 255 arguments (probes/t2709-typed-sum-255,
-max-255) and are refused at 256; `CONCAT` and `TEXTJOIN` take 254 (probes/t2709-typed-concat-253, -concat-254,
-textjoin-254, which joins 252 texts) and are refused at 255 and 256 (Graph PATCH 400 `InvalidArgument`, "The argument is
invalid or missing or has an incorrect format", the response to any formula Excel refuses). It is the count, not the
length: 255 one-character arguments (`=CONCAT(1,...)`, 518 characters; `=TEXTJOIN("",TRUE,1,...)`, 524) are refused while
254 (-concat-254-short, -textjoin-254-short) and a 2,548-character CONCAT of 254 arguments (-concat-254-long) are accepted. So CONCAT takes 254 texts, one more than Microsoft's
documented 253, and TEXTJOIN the documented 252; the engine table holds 254 for both as cited overrides.
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

## GETPIVOTDATA with an odd count of five or more arguments refuses the workbook
Rule: Excel for the web refuses a whole workbook holding `GETPIVOTDATA(data_field, pivot_table, field1, item1,
field2)` (5 arguments) or `(..., field1, item1, field2, item2, field3)` (7), with or without a pivot table at the
reference and inside `IFERROR` too. Three arguments (`data_field, pivot_table, field1`, the item missing) load and
are `#REF!` (web.md "GETPIVOTDATA against a pivot table"), and even counts load. So the field/item pairs are checked
when the formula is read, but only past the first pair. RustSheet loads such a formula and gives `#ERROR!` like the
other refused argument counts (T-2403; `fn_getpivotdata` checks the count first); with no Excel value to match, a known
gap, not a deviation. The generator draws only the three-argument odd call (tools/src/rustsheet_tools/gen/pivots.py).
Evidence: single-formula workbooks over a generated pivot (`Pivot!H3`, a Count of Units pivot, rows Product,
columns Rep) sent with `rustsheet-oracle fill -b excel-graph` on 2026-10-22 (T-2396): `=GETPIVOTDATA("Units",Pivot!$H$3,"Product","C","Rep")`,
the same against `Pivot!$Z$40` (outside the pivot), wrapped in `IFERROR(...,-1)`, and
`=GETPIVOTDATA("Units",Pivot!$H$3,"Product","C","Rep","Bo","Region")` → createSession 501 `FileCorruptTryRepair` /
`unsupportedWorkbook`; `=GETPIVOTDATA("Units",Pivot!$H$3,"Product")` loads (`#REF!`) and
`=GETPIVOTDATA("Units",Pivot!$H$3,"Product","C","Rep","Bo")` loads (3). Rejected inputs cannot be committed; the
loading three-argument shape is probes/getpivotdata-real-pivot H15.
Status: VERIFIED (excel-graph); desktop unprobed.

## Formulas over 8,192 characters refuse the workbook
Rule: Excel for the web refuses a whole workbook holding a formula of more than 8,192 stored characters (ASCII; non-BMP text below; the
`<f>` text without the leading `=`; createSession 501 `FileCorruptTryRepair`). A formula of 8,192 characters loads.
RustSheet's xlsx importer refuses the same workbooks with `XlsxError::Workbook` (decision 0047).
Evidence: T-1091, probe `operator-chain-limit` (T-1060), probes `formula-length-nonbmp-33pieces` and `name-body-length-*` (T-1386; two more inputs were dropped after the run because Excel cannot load them, so no sidecar can exist: LEN of 34 `&`-joined 120-emoji literals, and an 8,192-character `1+1+...+11` name body), all excel-graph; see "Formula nesting depth cap" below.
Non-BMP text: Excel counts UTF-16 code units (an emoji is two). A formula holding 33 concatenated 120-emoji literals loads
(4,063 Unicode scalars, 8,023 units) and one holding 34 is refused (4,186 scalars, 8,266 units; createSession 501). Boundary
probe `formula-length-nonbmp-8192u` (T-1397, excel-graph; emoji and ASCII literals of at most 255 units): a stored formula of exactly 8,192
UTF-16 units loads; 8,193 and 8,194 units are refused (createSession 501; those two cases were dropped after the run, so only the loading 8,192-unit case has a sidecar).
The importer counts UTF-16 units and so refuses the same workbooks (T-1397).
Defined names: a name body of 8,191 characters (`=1+1+...` stored without `=`) loads and one of 8,192 is refused, so
the body limit is one below the 8,192 of a cell formula (bodies of 255, 256, 1,000, 4,000, 6,000 and 8,000 characters
all load and evaluate). Non-BMP bodies count UTF-16 units the same way: probe `name-body-nonbmp-8191u` (8,191 units, emoji
included) loads (it has the only sidecar) and a body of 8,192 units is refused (T-1397; that case was dropped). The importer refuses a name body over 8,191 units
(`XlsxError::Workbook`, T-1397).
Typed (T-2709, Graph PATCH, excel-graph 2026-10-25): the same 8,192 limit holds, `=1+1+...+1` of 8,192 characters is
accepted (probes/t2709-typed-sum-8192-chars, B1 `LEN(FORMULATEXT)` 8192) and one of 8,193 is refused (400
`InvalidArgument`). A string literal holds at most 4,095 UTF-16 units, typed or stored (T-2739, excel-graph 2026-10-26 UTC): a typed
`="a...a"` of 4,095 is accepted (probes/t2739-typed-4095, `LEN` 4095) and 4,096 is refused (400 `InvalidArgument`),
the same in a function argument (t2739-typed-len-4095; `LEN("<4,096>")` refused). It counts the value, not the source:
a doubled quote is one character (t2739-typed-dq-4094 and -dq-4095, 4,093 or 4,094 letters and `""`, are accepted;
4,095 letters and `""` are refused). It counts UTF-16 units: 2,047 emoji (4,094 units, t2739-typed-emoji-2047, `LEN`
4094) are accepted and 2,048 (4,096 units) refused. The limit is per literal (t2739-typed-two-4000, two literals of
4,000, is accepted). Stored, 256 and 4,095 load (t2739-stored-256, -stored-4095) and 4,096 makes Excel refuse the
workbook (createSession 501), so the published 255 is no limit at all. The engine refuses a literal over 4,095 units in the lexer: a typed one is rejected
and the cell keeps its content (`is_limit_error`), and the importer refuses the workbook, for cell formulas and defined-name
bodies (T-2762; name bodies are unprobed; the importer's scan stops at the first token the lexer rejects, so a literal
after one loads as a parse error).
Unprobed: built-in (`_xlnm.*`) and sheet-scoped name bodies (the limit is applied to every definedName), a name defined through the API, desktop Excel, and `_xHHHH_` escapes inside `<f>` text (the spec-to-xlsx
writer cannot emit them raw; T-1075 item 8).
Notes: the first attempts (a single string literal of 4,090 or 4,100 emoji) were refused as well; the 4,095-unit string-literal limit (see "Typed" above; the published figure is 255) is the cause: 4,090 emoji are 8,180 units for the literals, which is why the cases split them.
Status: VERIFIED (excel-graph, file load)
Implemented in: rustsheet-xlsx `import::sheet_data::check_formula_length`
Tasks: T-1091, T-1093, T-1386, T-1397, T-2739, T-2762

## Formula nesting depth cap
Rule: RustSheet rejects a formula whose tree is deeper than `MAX_FORMULA_DEPTH` (256) as a parse
error ("Formula is nested too deeply"); typed into a cell through the API it is refused
(`EditError::InvalidInput`), and a workbook formula that deep loads as a parse error. Each operand
(number, reference, parenthesis, function call, array) costs two levels and each reference operator
one, so up to 127 nested parentheses or function calls are accepted (126 around the operator chain
`0.1+0.2-0.3+B1`, which costs a level of its own; T-1169); union operand lists too
(T-0134). Each unary minus is a node (T-0132), and so is each unary plus kept in the formula text
(T-2723), so a run of more than about 250 signs (`=----...1`, `=++++..."a"`) is also a parse error;
a plus directly before a number literal folds into it and adds no node. Typed whitespace adds no
level either (SPEC-0023, decision 0127): `=SUM( - SUM( - …` nests as deep as `=SUM(-SUM(-…`. Excel caps function nesting at 64
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
chain Excel accepts. The engine itself has no formula text length limit: through the API it accepts
and evaluates chains of 8,194 and 8,195 characters (4,096 operators) and any longer formula under the
operator cap. The xlsx importer refuses a workbook holding one, as Excel does (see "Formulas over
8,192 characters refuse the workbook" above).
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
**the leftmost error operand wins** for `=`, `+`, `-`, `*` and `SUM` of direct arguments (SUM ranks a referenced cell's error below a direct argument's error, see "Stored `#BUSY!` behaviors"; the general rule is in arrays.md), whichever error it is:
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
likewise for `=` (C7 0, C8 2) and `*` with `#N/A` (C9 0, C10 7). SUM differs: `SUM(A1,1/0)` and `SUM(1/0,A1)` are both
ERROR.TYPE 2 (C11, C12). Rule (probes/sum-error-precedence, excel-graph 2026-10-11, T-1315/T-1314): in SUM an error in a
*direct* argument (literal `#DIV/0!` or computed `1/0`; `SUM(#N/A,A1)` and `SUM(A1,#N/A)` are both 7) beats an error in a
referenced cell whichever comes first, for any referenced error kind (`#BUSY!`, `#N/A`, `#VALUE!`), and among referenced
cells the leftmost wins (`SUM(A1,A2)` 0, `SUM(A2,A1)` 7, `SUM(A3,A4)` 3, `SUM(A4,A3)` 5; `SUM(A1:A2)` is the same as `SUM(A1,A2)`),
so `#BUSY!` has no special rank. COUNTIF: `"#BUSY!"` and the cell itself as criteria count 0 (unlike an `#N/A` criterion, criteria.md, which counts `#N/A` cells), `"<>#BUSY!"` counts both
cells (C13-C15). `IFERROR(A1,"y")` is "y" (C16). `ERROR.TYPE(A1:A2)` spills 0 then `#N/A` (E1:E2).
`UNIQUE` keeps a stored `#BUSY!` and a stored `#NULL!` as distinct rows and merges equal `#BUSY!`s
(C18-C20: 2, 3, 3 rows).
Aggregate error precedence (probes/aggregate-error-precedence, excel-graph 2026-10-11, T-1334; A1 #BUSY!, A2 #N/A, A3 #VALUE!):
PRODUCT, MIN, MAX and AVERAGE share SUM's rule: a *scalar* direct error (literal `#DIV/0!` or computed `1/0`) beats a
referenced-cell error whichever comes first (`PRODUCT(A2,#DIV/0!)`, `MIN(A2,1/0)`, `AVERAGE(A1,#DIV/0!)`, `MAX(A2,#DIV/0!)` all
ERROR.TYPE 2, C1-C32), and between two referenced errors the leftmost wins (`MIN(A2,A3)` 7, `MIN(A3,A2)` 3). COUNT ignores errors
(0) and COUNTA counts them (the argument count). SUM only: an error *inside an array* (`{1,#N/A}` or `A2:A3*1`) is not a scalar direct
error: a scalar direct error beats it either way (`SUM({1,#N/A},#DIV/0!)` 2, `SUM(#DIV/0!,{1,#N/A})` 2, `SUM(A2:A3*1,#DIV/0!)` 2),
but against a referenced error the leftmost wins (`SUM({1,#N/A},A3)` 7, `SUM(A3,{1,#N/A})` 3), and between arrays the leftmost wins
(C54, C55); a scalar computed from a reference (`A2*1`) is a direct error (`SUM(A3,A2*1)` 7). Uncoercible direct text `"x"` is `#VALUE!`
in the argument's position among the direct arguments (`SUM("x",#DIV/0!)` 3, `SUM(#DIV/0!,"x")` 2) and, being direct, beats a referenced
error (`SUM(A2,"x")` 3); text in a referenced cell is ignored (`SUM(A5)` 0, `SUM(A5:A6)` 2). A direct `#REF!` behaves like any scalar direct
error: it beats referenced errors and `{1,#N/A}` (`SUM(A2,#REF!)` 4, `SUM(#REF!,{1,#N/A})` 4, `SUM({1,#N/A},#REF!)` 4, C77) and ties by position (`SUM(#REF!,#DIV/0!)` 4,
`SUM(#DIV/0!,#REF!)` 2).
Aggregates over picked cells, direct LAMBDA/blank values and the A variants (probes/aggregate-picked-direct-a-variants, excel-graph 2026-10-14, T-1566; B1 "x", A5 #N/A, A6 #VALUE!):
- PRODUCT reads a cell picked by XLOOKUP, TAKE or DROP as a reference, exactly like SUM: the text cell is skipped (`PRODUCT(XLOOKUP(1,A1:A2,B1:B2),2)`, `PRODUCT(TAKE(B1:B2,1),2)`, `PRODUCT(DROP(B1:B2,-1),2)` are all 2, D1-D6). Ours matches all of D1-D6 since T-1571. Verified (probes/aggregate-product-trimrange-picks, excel-graph, T-1601): PRODUCT equals SUM for a TRIMRANGE pick (`PRODUCT(TRIMRANGE(A1:A4),2)` over 3,4 and two blanks is 24; a text cell is skipped), for a picked FALSE or blank cell (skipped: `PRODUCT(TAKE(B1:B2,1),5)` is 5, not 0), and a picked `#N/A` cell ranks as a referenced error, so a direct `#DIV/0!` wins on either side (`PRODUCT(TAKE(E1:E2,1),#DIV/0!)` and `PRODUCT(#DIV/0!,TAKE(..))` are #DIV/0!, ERROR.TYPE 2; alone the pick is #N/A). Ours matches all 15 cells.
- A direct LAMBDA value is `#VALUE!` (ERROR.TYPE 3) for SUM, PRODUCT, MIN, MAX **and AVERAGE and MEDIAN** (D7-D12); only COUNT skips it (`COUNT(LAMBDA(x,x),2)` 1, D13). Ours matches since T-1572. Verified for the rest of the family (probes/aggregate-direct-lambda-average-family, excel-graph 2026-10-15, T-1602): GEOMEAN, HARMEAN, VAR, VAR.P, STDEV, STDEV.P, SKEW, SKEW.P, KURT, DEVSQ, AVEDEV and SUMSQ all give `#VALUE!` for a direct LAMBDA too. **The LAMBDA's `#VALUE!` ranks with the referenced errors, not the direct ones**: `SUM(LAMBDA(x,x),#N/A)` and `SUM(#N/A,LAMBDA(x,x))` are `#N/A` (a direct error wins in either order), `SUM(A5,LAMBDA(x,x))` is `#N/A` (A5 holds #N/A and comes first) but `SUM(LAMBDA(x,x),A5)` is `#VALUE!` (the LAMBDA comes first); the same for AVERAGE, DEVSQ (`DEVSQ(LAMBDA(x,x),A5)` is `#VALUE!`) and SUMSQ. A function name passed directly (`SUM(_xleta.SUM)`, `SUM(_xleta.SUM,2)`, `AVERAGE(_xleta.SUM,2)`) is also `#VALUE!`, and `COUNT(_xleta.SUM,2)` skips it (1). Ours matched the family's `#VALUE!` but returned the LAMBDA error at once (so `SUM(LAMBDA(x,x),#N/A)` was `#VALUE!`); fixed in T-1602 by holding it like a referenced error (`AggregateWalk::lambda_error`). PRODUCT, MIN and MAX rank it the same way as SUM in all nine shapes (probes/aggregate-direct-lambda-product-min-max, excel-graph 2026-10-15, T-1604): a direct `#N/A` wins in either order, the leftmost of a referenced error and the LAMBDA wins (`PRODUCT(A5,LAMBDA(x,x))` `#N/A`, `PRODUCT(LAMBDA(x,x),A5)` `#VALUE!`), a LAMBDA before an array error `{1,#N/A}` gives `#VALUE!` and after it `#N/A`, and `MIN(A7:A8,LAMBDA(x,x))` gives the range's `#DIV/0!`. Ours returned the LAMBDA error at once in the three; fixed in T-1604 (`fold_min_max`, `fn_product`).
- A blank `LET(b,Z99,b)` is read as a referenced blank: skipped by SUM, PRODUCT, MIN, MAX (`MAX(..,-5)` is -5, not 0), AVERAGE and COUNT (D14-D19). Ours agrees.
- MAXA, MINA, AVERAGEA follow the same precedence as MAX/MIN/AVERAGE: a scalar direct error beats a referenced one either way (`MAXA(A5,#DIV/0!)`, `MAXA(#DIV/0!,A5)`, `AVERAGEA(A5,1/0)`, `AVERAGEA(1/0,A5)` all 2, D20-D23), and between referenced errors the leftmost wins (`MINA(A5,A6)` 7, `MINA(A6,A5)` 3, `MAXA(A5,A6)` 7, `AVERAGEA(A6,A5)` 3, D24-D27). Ours matches since T-1573. UNVERIFIED: the array-error rank (as for AVERAGE), a cached linked-workbook error (ranked as a referenced error) and "Unexpected Range" (returns at once).
Evidence: probes/error-literals-new2 (S!A1:A15) and probes/error-literals-new (S!A1:A18), both excel-graph 2026-10-06; the eight single-cell
rejections were run as throw-away one-cell cases (not committed; no value to record);
probes/error-stored-constants (S!A1:I3, excel-graph 2026-10-06) for the loading stored names, the five
stored rejections as throw-away one-cell cases (same); the three engine-only stored rejections (`#CIRC!`, `#ERROR!`, `#N/IMPL`) likewise (excel-graph 2026-10-08, T-1133).
Verified (probes/aggregate-error-precedence-stat-family, 128 cells, excel-graph 2026-10-16, T-1439; A5 #N/A, A6 #VALUE!, A7:A8 #DIV/0! then #NUM!): MEDIAN, GEOMEAN, HARMEAN, SKEW, KURT, VAR, VAR.P, STDEV, STDEV.P, **DEVSQ, AVEDEV and SUMSQ**, PRODUCT, MIN, MAX and AVERAGE all rank errors like SUM: `fn(A5,#DIV/0!)` and `fn(#DIV/0!,A5)` are `#DIV/0!` (a direct error wins either way), `fn(A5,A6)` 7 and `fn(A6,A5)` 3 (leftmost referenced), `fn(A7:A8,#DIV/0!)` 2, `fn({1,#N/A},#DIV/0!)` 2, `fn(A6,{1,#N/A})` 3 and `fn({1,#N/A},A6)` 7. DEVSQ, AVEDEV and SUMSQ returned the first error at once until T-1439 (`ErrorRule::Ranked` for `for_each_value_skip_empty`); the rest already matched.
An error inside an array ranks with the referenced errors in MIN, MAX, AVERAGE and MEDIAN as in SUM and PRODUCT (coercion.md PRODUCT array entry, T-1287): the leftmost of the referenced, array and direct-LAMBDA errors wins, a direct error beats them all. Probed (probes/aggregate-array-error-vs-ref-error, 40 cells, excel-graph, 2026-10-15, T-1615; A5 #N/A, A6 #VALUE!, A7:A8 #DIV/0! then #NUM!): for SUM, MIN, MAX, AVERAGE and MEDIAN alike `fn(A6,{1,#N/A})` is `#VALUE!` (ERROR.TYPE 3), `fn({1,#N/A},A6)` `#N/A` (7), `fn(A7:A8,{1,#N/A})` `#DIV/0!` (2), `fn({1,#N/A},A7:A8)` `#N/A` and `fn(LAMBDA(x,x),{1,#N/A})` `#VALUE!` (the LAMBDA of T-1604 is just the leftmost). A direct error beats an array error whichever comes first, in all five functions: `fn({1,#N/A},#DIV/0!)`, `fn(#DIV/0!,{1,#N/A})` and `fn({1,#N/A},1/0)` are `#DIV/0!` (D26-D40). Ours returned the array error at once in MIN, MAX and AVERAGE/MEDIAN after a referenced error or a direct LAMBDA (10 cells); fixed in T-1615 by dropping `ArrayErrorRank` and `lambda_held` from the walker: every `Ranked` policy holds an array error like a referenced one. Verified for the rest of the AVERAGE family (probes/aggregate-array-error-vs-ref-error-average-family, 87 cells, excel-graph, 2026-10-15, T-1629): GEOMEAN, HARMEAN, SKEW, KURT, VAR, STDEV, AVERAGEA, MAXA and MINA rank all eight shapes exactly like SUM (3, 7, 2, 7, 3, 2, 2, 2). **A direct LAMBDA is `#VALUE!` in AVERAGEA, MAXA and MINA too**, ranked with the referenced errors (`AVERAGEA(LAMBDA(x,x),2)` and `AVERAGEA(2,LAMBDA(x,x))` 3, `AVERAGEA(LAMBDA(x,x),#DIV/0!)` 2, `AVERAGEA(A5,LAMBDA(x,x))` 7, `AVERAGEA(LAMBDA(x,x),A5)` 3, D73-D87; same for MAXA, MINA). Ours skipped the LAMBDA in the A variants (`CastA` ignored it), so `AVERAGEA(LAMBDA(x,x),{1,#N/A})`, MAXA and MINA gave 7; fixed in T-1629. VARA, VARPA, STDEVA and STDEVPA rank errors exactly like SUM (probes/aggregate-error-precedence-a-variants, 72 cells, excel-graph, 2026-10-16, T-1696): `fn(A5,#DIV/0!)` and `fn({1,#N/A},#DIV/0!)` are `#DIV/0!` (2, a direct error wins), `fn(A5,A6)` 7, `fn(A6,A5)` 3, `fn(A6,{1,#N/A})` 3, `fn({1,#N/A},A6)` 7. `fn(A7:A8,#N/A)` is `#N/A` (7) for these and for SUM, AVERAGE, MEDIAN and DEVSQ: a direct error beats a range whose first error is `#DIV/0!` (T-1439's `fn(A7:A8,#DIV/0!)` could not tell that from "the range's first error wins"). Ours returned the first error at once (`ErrorRule::First`) and now uses `Ranked`.
Status: VERIFIED
Tasks: T-0175, T-0177, T-0195, T-0999, T-1010, T-1012, T-1016, T-1020, T-1133, T-1014, T-1222, T-1228, T-1314, T-1315, T-1334, T-1379, T-1439, T-1604, T-1615, T-1629, T-1696
Implemented in: statistical/variance.rs (`VARA_ARGS`), math_and_trigonometry/sum.rs (`sum_with_last_addition`, reference and array errors held back), product.rs, mathematical.rs (`fold_min_max`) and statistical/count_and_average.rs (`for_each_value`, AVERAGE; reference errors held back, T-1379), functions/criteria.rs (`build_criteria_with`, `#BUSY!` criterion matches nothing); expressions/token.rs (`Error::GETTINGDATA`/`CONNECT`/`BLOCKED`/`FIELD`/`BUSY`, `Error::error_type_code`, `Error::stored_literal`, `get_error_by_english_name`, `get_error_by_name`); rustsheet value.rs (`ErrorKind`); rustsheet-xlsx export/worksheets.rs (`exported_error_literal`)

## A typed unary plus stays in the formula text, except on a number literal
Rule: typed (Graph PATCH), Excel keeps a unary plus in the formula text everywhere except when it is directly followed by
a number literal, where it folds into the number. Typing `=+5` stores `=5`, `=+1E+3` stores `=1000`, `=+.5` stores
`=0.5` and `=+5%` stores `=5%`. With more than one sign, only the plus directly before the number folds: `=++5` stores
`=+5`, `=1++2` stores `=1+2` and `=-+5` stores `=-5`. A space breaks the fold: `=+ 5` stays as typed. On anything else the
plus is kept as typed: `=+"a"`, `=+TRUE`, `=+B1`, `=+B1:B2`, `=+SUM(1,2)`, `=+(1)`, `=+-5`, `=++"a"`, `=1++"2"`,
`=+{1,2}`, `=+#N/A`, `=+"a"&"b"`, `=SUM(+"1",2)`, `=IF(TRUE,+"a")`, `=+(+"a")`, `=+ "a"` and `=+LAMBDA(x,x)(1)`. The plus
is the identity: every value matches the operand's.
Evidence: probes/t2723-unary-plus-formula-text A1:C26 (excel-graph `entered`, 2026-10-25, T-2723). Stored, a plus on text
or a boolean loads and is kept too (probes/t2601-index-scalar-forms-t-n B14, B15), and so do stored `=+10^1` and `=+Z1`
forms (gen-m1/6462bf910ef9f28f, probes/lookup-argument-edges). The full set stored in one workbook was refused
(FileCorruptTryRepair); which formula caused it is unprobed (`=+LAMBDA(x,x)(1)` is the likely one: T-2405 records
`=SUM(1)(2)` the same way). The same rule appears in references.md (`=@+1` prints `=@1`) and lookup.md (INDEX's
`+"a"`).
Ours: matches since T-2723. A kept plus is a `Node::UnaryPlus`, the identity, which every analysis sees through as it
sees through a written group. The fold is decided in `parse_power`. Two cells still fail: the tree keeps no whitespace, so
`=+ 5` and `=+ "a"` print `=+5` and `=+"a"` (C24, C25; "Formula text keeps typed whitespace" below, T-2776).
Stored, a plus on a number literal folds on load as it does typed: a plain `<f>+5</f>` prints `=5`, a dynamic one too,
`<f>SUM(+5,1)</f>` prints `=SUM(5,1)` and `<f>+5+1</f>` prints `=5+1`, while a stored plus on anything else is kept
(`ABS(+{5})`, `INDIRECT(+"K1")`, `+B1:B3` below) (probes/t2772-unary-plus-static-readings D19:E20,
D27:E28; T-2772). Typed, Excel refuses a plus before a LET name or a LAMBDA parameter: `=LET(+x,1,x)` and
`=LAMBDA(+x,x)(1)` both get Graph PATCH 400 `InvalidArgument` (docs/research/t2772-let-lambda-plus-refusal-2026-10-27.md; refused inputs cannot be committed
as cases). We refuse the LAMBDA form (`#ERROR!`, test_fn_let.rs) but evaluate the LET form to `#VALUE!` (T-2887). Where an analysis matches a literal, a plus changes nothing, legacy or
dynamic: `N(IF(+TRUE,B1:B3))` and `N(CHOOSE(+TRUE,Z1,{2;3}))` match their plus-free controls (1 and 7 off the range's
rows; 1 and 1 on row 2 of B1:B3, where no `@` is added either), `SUBTOTAL(9,TAKE(IF(INDIRECT(+"K1"),H1:H2,I1:I2),2))`
is 7 like `INDIRECT("K1")`, legacy `ABS(+{5})` prints `=ABS(+@{5})` (5) and legacy `N(+{2;3})` prints `=N(+@{2;3})` (2);
a dynamic `N(+{2;3})` lifts (`#SPILL!` there, blocked). FORMULATEXT keeps every such plus
(probes/t2772-unary-plus-static-readings, probes/t2772-unary-plus-in-range-rows D:I). A plus on a whole N argument does
make the range a value: legacy `=N(+B1:B3)` on row 2 prints `=N(+@B1:B3)` and is 2, where `=N(B1:B3)` is 1 with no `@`;
legacy `=+B1:B3` and `=ABS(+B1:B3)` print `+@B1:B3` (2) (probes/t2772-unary-plus-in-range-rows J:M). We give 1 and print
`=N(+B1:B3)`: `is_plain_reference` sees through the top-level plus (T-2885). Unprobed: a plus on an IF arm or on OFFSET's
reference under N (T-2885 probes them).
Status: VERIFIED (excel-graph)

## Formula text keeps typed whitespace, except at the end, before a comma and in an array constant
Rule: typed (Graph PATCH), Excel keeps the spaces and line feeds written in a formula, and FORMULATEXT prints them back:
around a binary operator (`=1 + 2`, `=1 +2`, `=1+ 2`, `=1  +  2`, `=B1 + B2`, `=1+` line feed `2`), after `=` (`= 1+2`),
after a unary sign (`=- 5`, `=-+ 5`, `=+  5`, and `=+ 5`, `=+ "a"` of T-2723), before `%` (`=5 %`), after `(` and
before `)` of a call or a group (`=SUM( 1 )`, `=SUM(B1:B2 )`, `=(1 )+2`, `=( 1)+2`) and after a comma (`=SUM(1, 2)`).
It drops three kinds: whitespace at the end of the formula (`=1+2 ` prints `=1+2`, `=1=1 ` prints `=1=1`), whitespace
before a comma (`=SUM(1 ,2)` prints `=SUM(1,2)`, `=IF( TRUE ,1,2)` prints `=IF( TRUE,1,2)`) and whitespace inside an
array constant (`={1, 2}` prints `={1,2}`). A plus that folds into a number literal keeps the spaces before it:
`=  +5` prints `=  5`. Whitespace never changes a value (every probed value is the space-free formula's).
Evidence: probes/t2769-formula-text-whitespace A1:C23 (excel-graph `entered`, 2026-10-26, T-2769).
More shapes (probes/t2776-formula-text-whitespace-shapes, t2776-ws-after-at, -after-bang, -after-colon,
-before-colon; excel-graph `entered` and stored, 2026-10-26, T-2776):
- Kept as typed: spaces between the references of an intersection beyond the one that is the operator
  (`=B1:B2  B2`, `=SUM(B1:B2   B2)`), after `@` (`=@ B1:B2`), around `&`, `=` and `*` (`="a" & "b"`, `=B1 = 1`,
  `=SUM(B1:B2) * 2`), inside LET (`=LET(x, 1, x + 1)`), between unary signs (`=-  -5`, `=1 +  - 5`), inside a group
  (`=(  1  )`) and a line feed after `=` (`=` LF `1+2`). A folded plus keeps the spaces before it after an operator
  too: `=1+  +5` prints `=1+  5`, `=1+ +5` prints `=1+ 5`.
- Dropped: around `:` (`=SUM(B1 :B2)` and `=SUM(B1: B2)` print `=SUM(B1:B2)`), after a sheet prefix's `!` (`=S! B1`
  prints `=S!B1`, value 1), a trailing line feed (`=1+2` LF and `=1+2 ` LF ` ` print `=1+2`).
- **A run keeps only its line feeds and the spaces after the last one**: `=1+ ` LF ` 2` prints `=1+` LF ` 2`,
  `=1+` LF ` ` LF `2` prints `=1+` LF LF `2`, 10 spaces, 3 line feeds and 2 spaces print as the 3 line feeds and 2
  spaces.
- **A run's two counts each wrap at 256, on their own**: `=1+` + 300 spaces + `2` prints with 44 spaces (`LEN` 48).
  255, 256 and 257 typed spaces print 255, none and 1; 255, 256, 257 and 300 typed line feeds print 255, none, 1 and
  44; 256 line feeds then 3 spaces print the 3 spaces; 3 spaces then 256 line feeds print nothing, so the spaces before
  the last line feed are dropped even when the line-feed count wraps to none. Every value is 3
  (probes/t2782-whitespace-run-wrap, excel-graph, 2026-10-26). Excel stores a run as a one-byte line-feed count and a
  one-byte space count (the BIFF `PtgAttrSpace` shape; the counts are the probes' evidence, not the documentation's).
- Refused: `=SUM (1)` (a space between a function name and its `(`) and `=SUM( )` (a run as the only argument):
  Graph's PATCH answers `400 InvalidArgument` ("The argument is invalid or missing or has an incorrect format.") for
  both (probes/t2786-whitespace-structured-refs-runs, each tried alone, 2026-10-26, T-2786); the same text for every
  refusal below. Nothing is stored.
- Stored in the file (`<f>` text, no typing): interior whitespace loads and prints as written (`1 + 2`, `SUM( 1 )`,
  `1+` LF `2`, `B1:B2  B2`, `(1 )+2`), and the typed drops apply (`SUM(1 ,2)`, `SUM({1, 2})`, a trailing space), but
  whitespace at the start of the stored text is dropped: `<f>  +5</f>` prints `=5` and `<f> 1+2</f>` prints `=1+2`,
  where the same text typed after `=` keeps it. Every value matches the space-free formula.
Whitespace beside a missing argument and inside a structured reference is the next entry; the `<f>` text Excel saves
keeps the whitespace (xlsx-storage.md, T-2786).
Ours (T-2781, SPEC-0023 step 1; line feeds T-2782, step 2): the parser keeps each run of spaces and line feeds as a
`Node::Spaced` around the operand it touches and drops the runs Excel drops, so FORMULATEXT, the formula text, the
R1C1 storage text, row and column edits, sheet renames and moves print the runs back. Two runs that the parser joins
around one operand (a folded plus) add their line feeds and keep the earlier spaces only when the later run has none;
a later run of exactly 256 line feeds counts as none there (unprobed). A cell stores two trees: the written one, which
prints, and one with no whitespace, which evaluation, units and static analysis read, so no argument-shape test sees a
`Spaced` (`formula_cells::ParsedFormula`); defined names, conditional formats and outside-cell formulas evaluate the
stripped tree too. A file's stored text drops the spaces it starts with (`Parser::loaded`; E1, E2, E7, E8, E10 pass);
other leading whitespace characters are kept as before (unprobed). The spaces beyond an intersection's one are kept as the right operand's leading run (T-2783; C3, C5, E9 pass). Still open: FORMULATEXT of an
intersection whose separator holds line feeds (`B1:B2` LF ` B2` drops the LF; unprobed, T-2949), whitespace around LAMBDA parameter names (dropped; unprobed, T-2949), and the xlsx export, which writes the
stripped tree where Excel saves the kept whitespace (T-2948). `=S! B1` is a parse error (`#ERROR!`) where Excel reads `S!B1`
(T-2785).
Status: VERIFIED (excel-graph)
Tasks: T-2769, T-2776, SPEC-0023

## Whitespace beside a missing argument is kept; a structured reference keeps some and normalizes the rest
Rule: a run beside a missing operand is kept as typed, and FORMULATEXT prints it back: `=NOW( )` and `=TODAY(  )`
(value as without the run), `=IF(1, ,2)`, `=IF(1,  ,2)`, `=IF(1,` LF `,2)`, `=SUM(1, )`, `=SUM(1,  )`, `=IF(1,2, )`,
`=SUM(1,` LF `)`, `=SUM(, 1)` and `=SUM(,1)` all print as typed. Values match the run-free formula (`=IF(1, ,2)` is
0, `=SUM(1, )` is 1, `=SUM(, 1)` is 1). This differs from "a run followed by `,` ... is dropped" (above): a run before a
comma is dropped only when an operand precedes it (`=SUM(1 ,2)`), and a run between two commas, or after the last comma,
is the missing argument's own and stays. Two shapes are refused (see "Refused" above): `=SUM (1)` and `=SUM( )`.
Inside a structured reference (table `T`, columns `Col`, `Other`; typed through Graph, FORMULATEXT and the saved `<f>`
agree):
- Refused by Graph's API (`400 InvalidArgument`; typing in desktop Excel unprobed, and Graph is stricter than Excel's own saved text): a space inside a bare column specifier (`T[ Col]`, `T[Col ]`, `T[ Col ]`), a space
  before a column-specifier bracket (`T [Col]`), spaces around the `:` of a column range (`T[[Col] : [Other]]`) and a
  union of two column specifiers (`T[[Col], [Other]]`).
- Accepted: a space inside the outer brackets around nested specifiers (`T[ [Col]:[Other] ]` prints as typed, value 77),
  `T[ [#Data] , [Col] ]` prints `T[ [#Data], [Col] ]` (the space before the comma is dropped, as in a call),
  `T[[#Data]  ,  [Col]]` prints `T[[#Data], [Col]]` (the run after the comma collapses to one space; one data point) and `T[[#Data],[Col]:[Other]]` prints with no space (none written).
- **A single column in outer-padded brackets is rewritten to the bare form**: `T[ [Col] ]` prints `T[ Col ]`, a form that
  typing `T[ Col ]` itself refuses, and which Excel itself saves (cached value 7); loading a hand-written stored `T[ Col ]` is unprobed (T-2800). Spaces around the specifier are kept as typed.
- Whitespace around the whole reference is an ordinary operand run (`=SUM( T[Col] )` prints as typed).
- The calculated-column fill rewrote `=T[[#This Row], [Col]]` and `=T[ [#This Row] , [Col] ]` typed into C2 and C3 to the
  formula typed into C4 (`T[[#This Row],[Col]] + T[[#This Row],[Other]]`), so this-row shapes are unanswered
  (the case's C2:C4); a probe needs a table without a calculated column fill (T-2800).
Evidence: probes/t2786-whitespace-structured-refs-runs (excel-graph `entered`, 2026-10-26, T-2786): E4:E15 (F4:F15), H1:H13
(I1:I13). Each refused formula was entered alone against the same case.
Ours: the runs SPEC-0023 keeps are in the entry above; a run beside a missing argument is T-2944. Inside a structured
reference (T-2801) the lexer keeps the runs after the outer `[` and before the outer `]` as typed and whether any
whitespace followed the comma, which prints as one space (`TableSpacing`); F4:F15 match. Two choices are ours, not
probed: a run around a lone specifier keeps the outer brackets it was typed in (`T[ [#Data] ]` prints as typed), and a
stored `T[ Col ]`, padded on both sides, reads back as column `Col` with its padding (so our own export round-trips).
One-sided `T[ Col]` and `T[Col ]` read as before, as a column whose name has the space.
Status: VERIFIED (excel-graph)
Implemented in: expressions/lexer/structured_references.rs, expressions/parser/structured_reference.rs (`text`)
Tasks: T-2786, T-2944, T-2800, T-2801

## FORMULATEXT keeps every written parenthesis
Rule: Excel keeps redundant parentheses as written in the stored formula; FORMULATEXT prints every level back in every
probed shape (plain operators, unary minus, intersection, function arguments and `@` operands): `=A1+(B1)`, `=SUM((A1)+(B1))`, `=(A1) (B1)`, `=-(A1)`, `=(A1)*((B1))`, `=A1+(B1*C1)`,
`=((A1))`, `=IF((A1>0),(B1),C1)`, `=@(A1+(B1))`, `=@((A1) (B1))`, `=@(((A1)) B1)`, `=@((A1)+(B1))`,
`=@(A1+((B1)))`, `=@((A1+B1))`, `=@(A1 B1) C1` and `=@(A1):B1 C1` all print as stored. Whether typed input is
kept the same way is UNVERIFIED (excel-graph loads file text; an M14 desktop question, decision 0039). The values are unaffected
(`=@(A1+(B1))` = 3, `=@((A1) (B1))` = `#NULL!` with A1 = 1, B1 = 2).
Evidence: probes/at-operand-nested-groups (36 cells, excel-graph, 2026-10-09, T-1178).
More shapes (probes/formulatext-written-groups-shapes, excel-graph 2026-10-11, T-1380; A1 = 4; desktop UNVERIFIED as above), all printed as written: a group before a postfix `%` `=(A1)%` (value 0.04), a group as a LAMBDA callee `=(LAMBDA(x,x))(1)` (1), a left-grouped operand `=(1/2)/3`, a right-grouped operand `=1+(3+5)` (9), a group around an intersection `=(D:D 2:2)&""` (empty text) and a group around a negated base `=(-A1)^2` (16). The engine's FORMULATEXT matches in all six (12 cells).
The converse holds too: Excel adds no parentheses where none were written (probes/operand-wrap-text, excel-graph
2026-10-12, T-1344; A1 = 3, A2 = 50): `=SUM(A1)^2` (9), `=2^SUM(A1)` (8), `=2^-A1` (0.125), `=2^A2%` (1.4142135623730951),
`=SUM({1,2}^2)`, `=SUM(2^{1,2})`, `=IFERROR(#N/A^2,7)`, `=-A1%` (-0.03), `=2*-A1`, `=LAMBDA(x,x)(A1)^2`,
`=2^LAMBDA(x,x)(A1)`, `=-SUM(A1)^2` (9), `=A1^2%`, `=2^-A1^2` (0.015625, so `(2^-A1)^2`) and `=2^--A1` all print as
written. The engine printed a call, array, error or LAMBDA call operand of `^` and a unary right operand of `^` in
added parentheses (`=(SUM(A1))^2`, `=2^(-A1)`) until T-1344 replaced its per-operator tables with one rule: an
operand is parenthesized only where the bare text would parse back to a different tree.
Ours: a written group is a `Node::Group` (SPEC-0007, T-1340), transparent to evaluation and printed as written, around
and inside `@` operands too (T-1343). All 36 cells of the T-1178 probe match. The T-1380 shapes above are separate from that count.
A cut and paste prints the same text (`to_string_moved` is `stringify` with moved references, T-1344).
Status: VERIFIED
The stored file text keeps them as well (xlsx-storage.md "Written parentheses are stored as written").
Design: SPEC-0007 (a `Group` node).
Tasks: T-1178, T-1244 (spec for keeping groups in the parse tree), T-1200 (the `@` group that is not the whole operand), T-1340 (the `Group` node), T-1343 (the `@` groups), T-1380 (more shapes), T-1344 (no added parentheses; one printer)
