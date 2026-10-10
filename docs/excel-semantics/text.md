# Text

## Broadcast text output cap
Rule: a text function broadcast over arrays (LEFT, REPT, SUBSTITUTE, ...) returns a single scalar `#VALUE!` (no spill) once the total bytes of its string results exceed 64 MiB, to bound allocation on untrusted input (CLAUDE.md directive 5). Excel's behavior (expected, not measured): compute the whole array, limited only by memory and the 32,767-char per-cell limit. Ours differs on legitimate data too: the budget counts UTF-8 bytes (not Excel's UTF-16 characters, so CJK text hits it about 3x sooner in characters), and a full-column text formula such as `=UPPER(A:A)` over ~65-char strings exceeds 64 MiB and returns `#VALUE!` where Excel spills normally.
Evidence: none for Excel's behavior; this is a resource limit, not an Excel observation.
Status: DEVIATION (decision 0014)
Implemented in: functions/text/util.rs (`MAX_BROADCAST_TEXT_BYTES`, `within_text_budget`), checked by functions/text/common.rs (`broadcast_text`)

## Text functions broadcast over array arguments
Rule: EXACT, FIND, LEN, REPT, SEARCH, SUBSTITUTE, TRIM, CLEAN, PROPER and REPLACE lift element-wise over
array and range arguments and spill, in the argument positions probed (`REPT("a",{1,2,3})` = a, aa, aaa;
`SUBSTITUTE("aaa","a","x",{1,2,3})` = xaa, axa, aax; `REPLACE("abcdef",{1,2},{2,3},"Q")` = Qcdef, aQef;
`EXACT({"a","b"},{"a";"b"})` is a 2x2 grid). An element that errors is an error only in its own cell
(`FIND({"b","z"},"abc")` = 2, `#VALUE!`). Not probed: arrays in FIND/SEARCH `within_text` or `start_num`.
Array constants may not contain expressions (see syntax.md, "Array constants take a single optional sign").
Evidence: eval/suites/probes/cases/indirect-text-broadcast.case.json A30:C124 (oracle: excel-graph, 2026-10-05, T-0957).
Status: VERIFIED (Excel for the web only; the engine matches all 43 text-broadcast cells)

## The *B text functions equal their non-B twins and spill (T-0271)
Rule: for ASCII (single-byte) text in en-US (non-DBCS) FINDB, LEFTB, LENB, MIDB, REPLACEB, RIGHTB and SEARCHB return what FIND, LEFT, LEN, MID,
REPLACE, RIGHT and SEARCH return, and lift element-wise over range and array arguments and spill like them
(`LENB(A1:C1)` = 11, 4, 5; `FINDB("o",A1:C1)` = 5, `#VALUE!`, `#VALUE!`; `LENB(A21:A24)` over blanks spills four 0s).
Evidence: eval/suites/probes/cases/text-b-functions-broadcast.case.json A3:A13 (oracle: excel-graph, 2026-10-07; engine matches all 29 cells).
Status: VERIFIED (Excel for the web only)
Implemented in: `byte_functions.rs` delegates to the twins; `static_analysis.rs` gives them `scalar_arguments` (before T-0271 they were `not_implemented`, so no spill was planned and a fuzz input hit the scalar-context debug_assert).

## SUBSTITUTE result longer than 32,767 characters
Rule: a SUBSTITUTE result over 32,767 characters is `#VALUE!` (unlike CONCATENATE and `&`, which truncate), for
replace-all and for a given instance number alike; exactly 32,767 is fine. The limit applies to the result, not the
inputs: when nothing matches the text is returned unchanged even if text plus new_text would exceed it.
Evidence: probes/text-result-limit C1-C7, C14 (over limit, `#VALUE!`), C11-C13 (no match, unchanged) (excel-graph,
2026-10-05; excel-desktop not used).
Open: REPT and CONCATENATE count the limit in UTF-16 code units (coercion.md "Text longer than 32,767
characters"); SUBSTITUTE is unprobed with surrogate pairs and the code still counts `chars()`; input text already over the limit (only reachable
through an oversized xlsx string) passes through when the result does not grow, an unprobed engine choice.
Status: VERIFIED
Implemented in: `substitute_exceeds_limit`/`substituted_chars` in functions/text/util.rs size the result before building it (T-0241)
Tasks: T-0241

## CONCAT and TEXTJOIN past 32,767 characters
Rule: the joined text of CONCAT or TEXTJOIN over 32,767 characters is `#CALC!` (not truncated, not `#VALUE!`).
Evidence: probes/text-result-limit C8-C9, C15-C23 (excel-graph, 2026-10-05 and 2026-10-07; excel-desktop not used).
Exactly 32,767 characters is fine (C15, C21); the limit counts UTF-16 code units, so an emoji is 2 (C16/C22 fit at 32,767, C17/C23 are `#CALC!` one unit over). An error *argument* after the limit wins over `#CALC!` (`CONCAT(REPT("a",32767),"b",NA())` and the TEXTJOIN equivalent are `#N/A`, C18-C19), but a cell error read from a reference or range does not: `TEXTJOIN(",",TRUE,REPT("a",32767),A1:A2)` with `A1` = "b" and `A2` = `#N/A` is `#CALC!` (C20). The cell error is dropped and evaluation goes on, so a later error argument still wins (`TEXTJOIN(",",TRUE,REPT("a",32767),A1:A2,NA())` is `#N/A`, errors C4).
Cell errors past the limit (probes/text-result-limit-errors, excel-graph, 2026-10-08, T-0284; `A1` = "b", `A2` = `#N/A`, `A3` empty): a cell error is `#CALC!` once the text read so far, plus for TEXTJOIN the delimiter that precedes the cell, is over 32,767, and the error itself otherwise.
- CONCAT: `CONCAT(REPT("a",32767),A1:A2)` is `#CALC!` ("b" takes it over, C1) and `CONCAT(REPT("a",32767),"b",A2)` is `#CALC!` (C10), but `CONCAT(REPT("a",32767),A2:A2)` and `...,A2)` are `#N/A` (exactly 32,767 is not over, C5-C6).
- TEXTJOIN: with a 32,767 prefix the delimiter takes it over before the cell, so `TEXTJOIN(",",TRUE,REPT("a",32767),A2:A2)` and `...,A2)` are `#CALC!` (C2-C3) while an empty delimiter leaves `#N/A` (C7); with a 32,766 prefix `","` is `#N/A` (C9) and `",,"` is `#CALC!` (C8); with ignore_empty FALSE an empty cell before the error adds a delimiter, so `TEXTJOIN(",",FALSE,REPT("a",32767),A3,A2)` is `#CALC!` (C11).
Other reference kinds behave like a direct A1 cell or range (probes/text-result-limit-errors-refs, excel-graph, 2026-10-08, T-0285; same layout, `Sheet2` repeats `A1:A2`): a defined name, `INDEX`, `IF`, `OFFSET`, `INDIRECT`, `INDEX(..):INDEX(..)` and a 3-D reference (`Sheet1:Sheet2!A1:A2`) are references, so a range that includes "b" gives `#CALC!` for CONCAT and TEXTJOIN (C1-C2, C4-C5, C7, C10, D1-D2, D4-D5, D7, D10), a single cell holding `#N/A` gives `#N/A` for CONCAT (exactly 32,767, C3, C6, C8, C11-C12) and `#CALC!` for TEXTJOIN (the delimiter takes it over, D3, D6, D8, D11-D12). `@A1:A2` from row 9 or 10 is `#VALUE!` (no intersection; C9, D9), which says nothing about the limit. Unprobed: TEXTJOIN with ignore_empty FALSE over a whole-column 3-D span (the engine's 3-D walk does not add the delimiters of the empty cells outside the used area, unlike the single-sheet walk).
Status: VERIFIED
Implemented in: `fn_concat`/`fn_textjoin` in functions/text/concatenate.rs (`ConcatText`, `JoinSize`) stop appending text once over the limit but still evaluate later arguments. `fn_concat` drops a range or reference cell error met once over the limit and continues; `fn_textjoin` does the same for range cells (through `JoinSize::push_cell`, also used for the sheet-by-sheet 3-D walk) and, in its `Node::ReferenceKind` arm, for a direct single-cell reference, counting the preceding delimiter through `JoinSize::exceeds_with_separator`. Both read their arguments through `evaluate_aggregate_arg` (which also treats `:` expressions, `Node::OpRangeKind`, as references), and `get_range` (reference_operators.rs) keeps an INDEX operand of `:` a reference (T-0243, T-0262, T-0284, T-0285)
Tasks: T-0243, T-0262, T-0284, T-0285

## TEXTJOIN with ignore_empty FALSE joins every cell of a whole column or row
Rule: with ignore_empty FALSE every cell of a whole-column or whole-row argument adds a delimiter, however
small the used area: `TEXTJOIN(",",FALSE,D:D)` is `#CALC!` (1,048,576 values join to over 32,767
characters) and `TEXTJOIN(",",FALSE,10:10)` with H10 = "x" and J10 = "y" is ",,,,,,,x,,y" followed by
16,374 more commas. With ignore_empty TRUE only the non-empty cells count (`TEXTJOIN(",",TRUE,D:D)` =
"a,c"). `TEXTJOIN(",",FALSE,D1:D16385)` over the same sheet has length 16,386.
Unprobed: an error cell met after the skipped empties have taken the join over the limit (for example
`TEXTJOIN(",",FALSE,1:3)` with an error in A3); the engine gives `#CALC!`, as for a walked range (C20).
Evidence: probes/whole-column-empty-walkers E2, E3, E6, F1, F2, F5, F8 (excel-graph, 2026-10-07, T-1114).
Status: VERIFIED
Implemented in: `fn_textjoin` (functions/text/concatenate.rs) walks the used area and adds the skipped
empty cells per row and after the last row through `JoinSize::push_empties`, which stops at the limit.
Tasks: T-1114

## JIS is `#NAME?` in en-US Excel for the web
Rule: `JIS`, although Microsoft's en-US function list includes it, is not a known function in
en-US Excel for the web: `JIS(1)` is `#NAME?`, stored bare or as `_xlfn.JIS`. `_xlfn.DBCS(1)` is
recognised.
Evidence: probes/catalog/availability-22 A8 (TRUE = #NAME?), function-name-forms-2 A1 and A2
(ERROR.TYPE 5), availability-10 A4 (`_xlfn.DBCS`, recognised) (oracle: excel-graph, 2026-10-07,
T-0040). Desktop Excel not yet checked: T-1081 (M14, decision 0039).
Status: VERIFIED (excel-graph); desktop confirmation pending (T-1081)


## FIXED drops the sign of a value that rounds to zero
Rule: FIXED prints no minus sign when the number rounds to zero at the requested decimals: `FIXED(-2,-1,TRUE)` = "0"
(`EVEN(-0.0001)` is -2).
Evidence: gen-m6/f94b5b8236742618 Calc!A24 (excel-graph, 2026-10-09, T-1239).
Ours: "-0" (the sign is taken from the unrounded value). Other shapes (`FIXED(-0.4,0)`) are UNVERIFIED.
Status: VERIFIED (one cell)
Tasks: T-1257

## REPLACE with a huge start_num is `#VALUE!`
Rule: `REPLACE(-0.0001,123456789.123,3,"a*")` is `#VALUE!` in Excel; there is an upper bound on start_num (the engine
appends past the end of the text instead). Where the bound lies (32767?) and whether MID/num_chars share it is unprobed.
Evidence: gen-m6/a62335e72e44cb4e Calc!A9 (excel-graph, 2026-10-09, T-1239).
Status: UNVERIFIED (probe pending: the bound)
Tasks: T-1258

## Text function argument modes
Rule: the M5 text functions take scalar arguments that lift over a range or array (one result per cell, a spill;
probes/text-argument-modes F1:F26: LEN, CODE, EXACT, VALUETOTEXT, REPT, FIND), with four exceptions: TEXTJOIN's text
arguments and ARRAYTOTEXT's array take a range whole, PHONETIC's argument must be a reference, and T does not lift:
`T(A1:A3)` in a dynamic-array formula is one "" with no spill although A2 = "ab" (F9; see references.md "T of a
multi-cell range is not intersected"). TEXTJOIN turns numbers and booleans in a range into their text
(`TEXTJOIN("-",TRUE,A1:A6)` over 5, "ab", TRUE, blank, "x", "2" is "5-ab-TRUE-x-2"; with ignore_empty FALSE the blank
adds a delimiter); ARRAYTOTEXT gives "5, ab, TRUE, , x, 2" and, with format 1, `{5;"ab";TRUE;;"x";"2"}`. A boolean read
from a cell (A3 = TRUE; literal TRUE arguments are unprobed) is its text where the function wants text (`CODE(A3)` = 84,
`PROPER(A3)` = "True", `TEXT(A3,"0")` = "TRUE", `LENB(A3)` = 4) and 1 where it wants a number (`CHAR(A3)` and
`UNICHAR(A3)` = CHAR(1), `REPT("a",A3)` = "a"), but `VALUE(A3)` and `NUMBERVALUE(A3)` are `#VALUE!`. `FIXED(A2)` with
A2 = "ab" is `#VALUE!`, `FIXED(A6)` with A6 = "2" (text) is "2.00". Also: `ASC(A2)` with A2 = 5 is "5", `DBCS(A1)` with
A1 = "abc" is "abc", `CLEAN(A3)` = "TRUE", `REPT("z",A1:A3)` spills "zzzzz", `#VALUE!` (for "ab"), "z", and
`LENB(A1:A2)` spills 3, 1 (text-modes-dbcs C1/C3, text-argument-modes C28/F21:F23, text-modes-bytes C7:C8). On the web JIS is `#NAME?` and PHONETIC of any
reference (a text cell, a number cell, a range) is `#N/A`. excel-graph refuses a workbook (HTTP 501
`unsupportedWorkbook`) that calls `PHONETIC("abc")` or `MUNIT(SUBTOTAL(9,..))` whose sum is a large dimension (seen
while generating, not committed: a refused workbook has no oracle file); both are web-only observations, desktop
unconfirmed (decision 0039).
Evidence: probes/text-argument-modes, text-modes-bytes, text-modes-dbcs, text-modes-bahttext, text-modes-phonetic,
-phonetic-range, -phonetic-number (excel-graph, 2026-10-09, T-1087).
Ours: matches except BAHTTEXT (T-1214), which is not implemented. DBCS returns its text unchanged and PHONETIC of any reference is `#N/A` (T-1213); DBCS widening of non-ASCII text (katakana, full-width forms), DBCS of a number cell or range, and PHONETIC of an error or empty cell are UNVERIFIED (T-1248). CODE and VALUETOTEXT
lift over ranges (T-1212). VALUETOTEXT's format (probes/valuetotext-format, excel-graph, T-1229): 1 quotes text and doubles embedded quotes (`"a""b"`; empty text is `""`), while a blank cell is empty, numbers, booleans and errors print as in format 0 (an error spells out, `#N/A`). The format is coerced to a number like any argument (`"1"`, TRUE and a blank cell work; other text is `#VALUE!`); a value in `[0, 2)` is truncated (`1.9` is 1, `1E-20` is 0), and anything else, `2`, `2.5`, `-0.5`, `-1`, is `#VALUE!`; an error format is returned as it is when the value is plain text; which error wins when the value is an error or the format is invalid as well, and a format given as a range reference, are UNVERIFIED (T-1288). The format lifts over arrays like the value (`VALUETOTEXT(A1:A2,{0,1})` is 2x2, `VALUETOTEXT(A1,{0,1})` 1x2). Ours: matches.
Status: VERIFIED (excel-graph)
Tasks: T-1087, T-1212, T-1213, T-1214, T-1229, T-1248
