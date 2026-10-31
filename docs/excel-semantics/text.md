# Text

## Broadcast text output cap
Rule: a text function broadcast over arrays (LEFT, REPT, SUBSTITUTE, ...) returns a single scalar `#VALUE!` (no spill) once the total bytes of its string results exceed 64 MiB, to bound allocation on untrusted input (CLAUDE.md directive 5). A broadcast whose shape is above 1,048,576 cells is refused with `#ERROR!` before any element is computed, so that bound takes precedence over this one (arrays.md "Array results above 1,048,576 cells", T-1771). Excel's behavior (expected, not measured): compute the whole array, limited only by memory and the 32,767-char per-cell limit. Ours differs on legitimate data too: the budget counts UTF-8 bytes (not Excel's UTF-16 characters, so CJK text hits it about 3x sooner in characters), and a full-column text formula such as `=UPPER(A:A)` over ~65-char strings exceeds 64 MiB and returns `#VALUE!` where Excel spills normally.
Evidence: none for Excel's behavior; this is a resource limit, not an Excel observation.
Status: DEVIATION (decision 0014)
Implemented in: functions/text/util.rs (`MAX_BROADCAST_TEXT_BYTES`, `within_text_budget`), checked by the lift driver (functions/lift/driver.rs, which runs the element-wise text functions since T-1775) and FORMULATEXT over a range

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

## Text positions count UTF-16 code units, except in LEFT and RIGHT
Rule: LEN, MID, FIND, SEARCH, REPLACE and their *B twins count UTF-16 code units, so an emoji
(a surrogate pair) counts two: `LEN("a😀b")` is 4, `MID("a😀b",2,2)` is "😀", `FIND("b","a😀b")` is
4, `REPLACE("a😀b",2,2,"x")` is "axb", and a SEARCH `?` matches one unit (`SEARCH("?b","😀b")` is 2,
`SEARCH("a?b","a😀b")` is `#VALUE!`, `SEARCH("a??b","a😀b")` is 1). LEFT and RIGHT (and LEFTB,
RIGHTB) instead count characters (code points, not graphemes): `LEFT("a😀b",3)` is "a😀b",
`LEN(LEFT("a😀b",2))` is 3, `RIGHT("a😀b",2)` is "😀b", `LEFT("e"&UNICHAR(769)&"x",1)` is "e" and
`LEFT("👍🏽x",1)` is "👍" without its skin-tone modifier. FIND and SEARCH may start inside a pair
(`FIND("b","😀b",2)` is 3). FIND takes a start one past the end, where only "" is found
(`FIND("","abc",4)` is 4, `FIND("","",1)` is 1, `FIND("","abc",5)` and `FIND("c","abc",4)` are
`#VALUE!`), while SEARCH does not (`SEARCH("","abc",4)` and `SEARCH("*","abc",4)` are `#VALUE!`).
Evidence: probes/text-surrogate-positions A1-A44, B1-B15, C1-C9 (excel-graph, 2026-10-09, T-0249);
probes/concatenate-limit-spill E7, E13, G97.
Backend: excel-graph only; excel-desktop not used.
Status: VERIFIED
Implemented in: `len_element`, `mid_element`, `find_element` in functions/text/common.rs and
`replace_element` in functions/text/string_format.rs, through `Text::slice_utf16` (text.rs) and
`one_char_per_utf16_unit` (functions/text/util.rs); `left_element`/`right_element` count chars.
Tasks: T-0249

## MID and REPLACE positions inside a surrogate pair keep the lone half
Rule: a MID or REPLACE position that splits a surrogate pair keeps the lone surrogate:
`LEN(MID("a😀b",3,2))` is 2 (low surrogate and "b"), `LEN(MID("a😀b",2,1))` is 1,
`LEN(MID("😀",2,5))` is 1, `LEN(REPLACE("a😀b",3,1,"x"))` is 4, and
`LEFT("a😀b",2)&MID("a😀b",3,2)` is "a😀", a lone U+DE00, then "b" (LEFT already took the whole
emoji). `UNICODE` of a lone low surrogate is its code
(`UNICODE(MID("a😀b",3,1))` is 56832), unlike a lone high one (`#VALUE!`, coercion.md
"Truncation inside a surrogate pair keeps a lone high surrogate"). LEFT and RIGHT treat a lone
surrogate as one character and do not pair a lone high surrogate with a following character
(`LEN(RIGHT(MID("a😀b",2,1)&"x",1))` is 1).
Evidence: probes/text-surrogate-positions A6, A11, A12, A29, A38, B6, B7, B10 (excel-graph,
2026-10-09, T-0249).
Ours: MID and REPLACE slice a `rustsheet_engine::Text` by UTF-16 units (`Text::slice_utf16`), so the
lone half survives (T-1579). LEFT, RIGHT and UNICODE read the exact units
(`Text::left_chars`, `Text::right_chars`, `chars_lossless`), so A38, B6 and B7 pass (T-1581).
`UNICODE` of a lone high surrogate followed by another character is `#VALUE!`
(`UNICODE(MID("😀",1,1)&"x")`); of a lone low one followed by another it is its code (56832). Note that
`UNICODE(MID("a😀b",2,2))` is not a lone unit: units 2-3 are the whole pair (128512). A pair followed by
a lone high one reads the pair, and `CODE` of a pair is 63 (probes/text-unicode-lone-surrogate-followed
A1-A10, excel-graph, 2026-10-15, T-1635; ours matches all 10). Whether `&` of a lone high and a lone low surrogate
forms the pair (`EXACT(MID("😀",1,1)&MID("😀",2,1),"😀")`) is TRUE: `Text::concat` pairs
them by UTF-16's rule (probes/text-lone-surrogates, T-1582).
Status: VERIFIED
Implemented in: `mid_element`, `left_element`, `right_element` and `fn_unicode` in
functions/text/common.rs, `replace_element` in functions/text/string_format.rs
Tasks: T-0249, T-1483, T-1575, T-1579, T-1581

## Lone surrogates through CODE, comparisons, case, search and cleanup functions
Built with `H = MID("😀",1,1)` (lone high, D83D) and `L = MID("😀",2,1)` (lone low, DE00). Excel text is
exact UTF-16 units throughout:
- `&` concatenates unit sequences, so a high followed by a low re-pairs: `EXACT(H&L,"😀")` is TRUE, `LEN(H&L)` is 2,
  `UNICODE(RIGHT(H&L,1))` is 128512, `LEN(LEFT(H&L,1))` is 2, `REPT(H&L,2)` is "😀😀". Low then high does not
  (`EXACT(L&H,"😀")` is FALSE, length 2). `MID("a😀b",2,1)&MID("a😀b",3,1)` is "😀".
- `CODE(H)` and `CODE(L)` are 63 (as for any non-Windows-1252 character). `UNICODE(H)` is `#VALUE!`,
  `UNICODE(L)` is 56832.
- A lone surrogate is never equal to U+FFFD: `EXACT(H,UNICHAR(65533))`, `H=UNICHAR(65533)` and the same for L are FALSE.
  `H=H` and `EXACT(H,H)` are TRUE, `H=L` and `EXACT(H,L)` FALSE.
- Order: `H<L` TRUE, `L<H` FALSE, `H<"😀"` TRUE, `"😀"<L` TRUE, `H<"a"` FALSE, but `H<UNICHAR(65533)` and
  `L<UNICHAR(65533)` are both FALSE (so it is not plain code-unit order against U+FFFD).
- MATCH(…,0) of H or L in `{"x";"😀"}` or `{…;"�"}` is `#N/A` (a lone half never matches a pair or U+FFFD).
- UPPER, LOWER, PROPER, TRIM and CLEAN keep a lone surrogate unchanged (`EXACT(UPPER(H),H)`, `UNICODE(UPPER(L))` 56832,
  `LEN(TRIM(" "&H&"  "&L&" "))` 3 (H, one inner space, L: TRIM keeps a single space, nothing re-pairs), `LEN(CLEAN(L&CHAR(7)&H))` 2); `H&""` keeps it too, and `LEN(T(H))`,
  `LEN(TEXT(H,"@"))`, `LEN(TEXT(L,"General"))` are 1 and `LEN(REPT(H,3))` is 3. T, TEXT and REPT keep the *exact* unit, not U+FFFD, and `REPT(L&H,2)` re-pairs at the join: `EXACT(T(H),H)`, `EXACT(REPT(H,2),H&H)`, `EXACT(TEXT(H,"@"),H)` and `EXACT(REPT(L&H,2),L&H&L&H)` are TRUE, and `UNICODE(MID(TEXT(L,"""x""@""y"""),2,1))` is 56832 while the same with H is `#VALUE!` (VERIFIED, probes/text-lone-surrogates A82-A84, A86, A97-A101, excel-graph, 2026-10-15, T-1647). The 255-character limit limits the doubled `@@` output length (254 ok, 256 not; whether it counts UTF-16 units or code points is unprobed, H is one of each): `LEN(TEXT(H&REPT("a",126),"@@"))` is 254 and with 127 `a` it is `#VALUE!` (A87, A88). Note `TEXT(text,"x@y")` is `#VALUE!` for the text values probed ("😀", H, L; A85, A89-A92, A94), even though `"x"@"y"` works; the cause is inferred to be `y` (a date token) next to `@`, unprobed alone, see T-1656. The engine returns "x" followed by the text with the `y` dropped (A93 passes only by accident) (T-1656); `TEXT(L,"@x")` is 2 units and `ISERROR(TEXT(L,"@@"))` is FALSE (A95, A96).
- `VALUE(H)`, `VALUE(L&"1")` and `H+0` are errors.
- FIND and SEARCH with a lone half as find_text *can* split a pair: `FIND(H,"a😀b")` 2, `FIND(L,"a😀b")` 3,
  `SEARCH` the same, `FIND(L,H&L)` 2, `FIND("😀",H&L)` 1, `SEARCH(H&"*","a😀b")` matches. A lone half does not match
  U+FFFD (`FIND(UNICHAR(65533),H)` and `SEARCH(UNICHAR(65533),L)` are `#VALUE!`).
- SUBSTITUTE with a lone half as old_text matches inside a pair, like FIND: `UNICODE(SUBSTITUTE("😀",H,"x"))` is 120
  (the result is "x" then the lone low half, `LEN` 2), `LEN(SUBSTITUTE("😀",L,"x"))` is 2, `LEN(SUBSTITUTE("a😀b😀",H,"x"))` is 6
  (one unit for one unit); `SUBSTITUTE("😀",H&L,"x")` replaces the whole pair (`LEN` 1); `LEN(SUBSTITUTE(H,H,"x"))` is 1.
- A lone high surrogate is cached as `_xD83D_` (like the low one, `_xDE00_`): `H` is `_xD83D_`, `"a"&H` is `a_xD83D_`,
  `MID("😀a😀",1,1)&MID("😀a😀",2,2)&MID("😀a😀",4,1)` is `😀a_xD83D_`.
- 32,767 limit: `LEN(REPT("a",32766)&"😀")` is 32767 (the kept unit is the high half, coercion.md "Truncation inside a surrogate pair"); `REPT("😀",16384)` is `#VALUE!`;
  CONCAT and TEXTJOIN one unit over (a pair straddling the limit) are `#CALC!`, not truncated (existing entry below).
Evidence: probes/text-lone-surrogates A1-A81, B1-B6 (excel-graph, 2026-10-15, T-1582).
Ours: comparisons, EXACT and exact-mode MATCH read lone units exactly (T-1585): `cmp_text` in functions/util/collation.rs weighs a lone unit (and each unit of an astral character) above every BMP character, by unit value, which gives H<L, H<"😀"<L, and both H and L above U+FFFD (`H<UNICHAR(65533)` FALSE); COUNTIF-style criteria and wildcard MATCH/XMATCH read lone units exactly as well (T-1642, criteria paragraph below). UPPER, LOWER, PROPER, TRIM and CLEAN keep lone units (`Text::map_runs`; a lone unit breaks a word for PROPER, unprobed). FIND, SEARCH and SUBSTITUTE read lone units exactly (T-1586: `find_element` in functions/text/position.rs maps each unit, a lone one included, to one character; `substitute_units` in functions/text/util.rs matches unit for unit, so a lone half splits a pair; with an `instance_num` instances overlap, as in the well-formed path, `SUBSTITUTE("aaa","aa","x",2)` is "ax", as Excel gives: "SUBSTITUTE instances overlap" below). CODE and VALUE already behaved as probed. REPT repeats the exact units (`Text::repeat`; `REPT(L&H,2)` re-pairs at the join like `&`), TEXT prints the exact text where `@` is (`format_text_value_exact` in formatter/format.rs formats a one-character private-use stand-in the format does not hold, so the 255-character limit and every format rule stay the well-formed path's, then swaps the exact text in; the limit is re-checked on the real length), and T returns its argument unchanged (T-1587; exactness VERIFIED, T-1647).
Status: VERIFIED
**Criteria and wildcards (VERIFIED, probes/criteria-lone-surrogates, excel-graph 2026-10-16, T-1642).** Excel's criteria (SUMIF, COUNTIFS) and wildcard MATCH/XMATCH read lone units exactly, unit by unit. Data: H, L, "😀", U+FFFD, "x", H&"a", "a"&L, "a".
- A lone-half criterion (`H`, `"="&H`) matches H only. It does not match U+FFFD or the other half. `"<>"&H` matches everything else.
- `"<"&H` matches U+FFFD, "x", "a"&L and "a". `">"&H` matches L, "😀" and H&"a". `"<"&L` matches all but L, and `">"&L` matches nothing. This is the `cmp_text` unit order.
- `?` is one UTF-16 unit: `"?"` matches H, L, U+FFFD, "x" and "a". `"??"` matches "😀", H&"a" and "a"&L. So a pair counts as two `?`.
- `H&"*"` matches H, "😀" and H&"a"; `"*"&L` matches L, "😀" and "a"&L. A wildcard splits a pair. `H&"?"` matches "😀" and H&"a".
- `"~"&H` matches nothing: a tilde before a non-wildcard is read literally.
- MATCH(..,0) and XMATCH(..,2) agree: `MATCH("??",..)` is 3 and `MATCH(H&"?",..)` is 3.
Ours: all 48 cells pass (T-1642). The text criteria (`result_is_*_string` in functions/criteria.rs) take the operand as `Text` and compare with `cmp_text`; `WildcardPattern` (same file) matches `?`, `*` and `~` over lower-cased UTF-16 units, for COUNTIF-style criteria and for MATCH/XMATCH/XLOOKUP's `wildcard_text_matcher`. D-function prefix matching (`result_starts_with_units`) compares lower-cased UTF-16 units and agrees with Excel (probes/d-function-prefix-lone-surrogates, criteria.md "D-function plain-text equality matches by prefix", T-1667). Exact-mode MATCH with a lone-half lookup value now goes through `WildcardPattern` like any other text, so it matches unit for unit with no collation (as MATCH(..,0) does for well-formed text, coercion.md); the lone-half cells A31 and A32 agree.
Tasks: T-1582, T-1585, T-1586, T-1587, T-1647 (probe follow-up), T-1642, T-1667

## SUBSTITUTE instances overlap (T-1644)
Rule: with an `instance_num`, SUBSTITUTE counts every match of old_text, overlapping ones included, and
replaces only the nth: `SUBSTITUTE("aaa","aa","x",1)` "xa", `,2)` "ax", `,3)` "aaa" (no third match);
`SUBSTITUTE("aaaa","aa","x",2)` "axa", `,3)` "aax"; `SUBSTITUTE("aaaaa","aaa","x",3)` "aax";
`SUBSTITUTE("abab","bab","x",1)` "ax"; `SUBSTITUTE("ababab","abab","x",2)` "abx". Without one, it replaces
left-to-right non-overlapping matches: `SUBSTITUTE("aaaa","aa","x")` "xx", `SUBSTITUTE("aaa","aa","x")` "xa".
Matching is case-sensitive (`SUBSTITUTE("AaA","aa","x",2)` "AaA"); an instance past the last match (up to 2147483646, below) leaves the
text unchanged; 0 and -1 are `#VALUE!`; 1.9 truncates to 1; an empty old_text changes nothing (probed with an instance_num); the instance lifts
over an array (`{1,2,3}` gives "xa", "ax", "aaa").
Evidence: probes/substitute-overlapping-instances (excel-graph, 2026-10-16).
Status: VERIFIED (excel-graph); ours matches (`nth_overlapping_match`, functions/text/util.rs).
Tasks: T-1644

## SUBSTITUTE result longer than 32,767 characters
Rule: a SUBSTITUTE result over 32,767 characters is `#VALUE!` (unlike CONCATENATE and `&`, which truncate), for
replace-all and for a given instance number alike; exactly 32,767 is fine. The limit applies to the result, not the
inputs: when nothing matches the text is returned unchanged even if text plus new_text would exceed it.
Evidence: probes/text-result-limit C1-C7, C14 (over limit, `#VALUE!`), C11-C13 (no match, unchanged) (excel-graph,
2026-10-05; excel-desktop not used).
Over-limit input: SUBSTITUTE keys on the result length only. On the 65,534-unit REGEXREPLACE text a same-length result is
`#VALUE!` (`SUBSTITUTE(REGEXREPLACE(REPT("a",32767),"a","bb"),"b","c")`, probes/regexreplace-long-result-d A2, T-1904),
but results that shrink to 0 and to 32,767 units are returned (probes/text-long-input-e A1, A2).
Per function (the text is the 65,534-unit REGEXREPLACE result; excel-graph, 2026-10-18): UPPER, LOWER, PROPER, TRIM,
CLEAN and REPLACE give `#VALUE!` for the input itself (SUBSTITUTE: see above) (probes/regexreplace-long-result-e A2, text-long-input-a, -b, -c); LEN,
LEFT, MID, ISTEXT and the REGEX* functions take the text. `&` with the text as the right operand truncates to 32,767 units like any other `&` (below). REPLACE's error precedence against an error in
another argument is unprobed (ours coerces all arguments first, then checks the length).
Open: REPT and CONCATENATE count the limit in UTF-16 code units (coercion.md "Text longer than 32,767
characters"); SUBSTITUTE is unprobed with surrogate pairs and the code still counts `chars()`.
Status: VERIFIED
Implemented in: `substitute_exceeds_limit`/`substituted_chars` in functions/text/util.rs size the result before building it (T-0241)

`&` with a 65,534-unit operand (probes/text-long-input-concat-g, -h, -k, -l, -m, -o; excel-graph, 2026-10-18, T-1913): the text as the
right operand is accepted and the result is truncated, not an error: `LEN("c"&R)`, `LEN(""&R)`, `LEN(INDEX({"c"},1)&R)` and
`LEN("c"&R&"d")` are 32,767, `ISERROR("c"&R)` is FALSE and `LEN(RIGHT("c"&R,2))` is 2 (R = the REGEXREPLACE result). A 32,767-unit
operand read from a cell and a short one give the same: `LEN(C1&"b")` with C1 = `REPT("a",32767)` is 32,767 and `ISERROR(C1&"b")` is FALSE
(probes/text-long-input-concat-d C3-C4). Ours matches.
Open (desktop question, M14): excel-graph refuses the workbook when the long text is the left operand (`R&""`, `R&"c"`, `R&R`, `LET(x,R,x&"c")`,
`IF(TRUE,R)&"c"`) and when it is read from a cell holding it, so the left-operand case is unmeasured.
Tasks: T-0241, T-1904, T-1913

## CONCAT and TEXTJOIN past 32,767 characters
Rule: the joined text of CONCAT or TEXTJOIN over 32,767 characters is `#CALC!` (not truncated, not `#VALUE!`).
Evidence: probes/text-result-limit C8-C9, C15-C23 (excel-graph, 2026-10-05 and 2026-10-07; excel-desktop not used).
Exactly 32,767 characters is fine (C15, C21); the limit counts UTF-16 code units, so an emoji is 2 (C16/C22 fit at 32,767, C17/C23 are `#CALC!` one unit over). An error *argument* after the limit wins over `#CALC!` (`CONCAT(REPT("a",32767),"b",NA())` and the TEXTJOIN equivalent are `#N/A`, C18-C19), but a cell error read from a reference or range does not: `TEXTJOIN(",",TRUE,REPT("a",32767),A1:A2)` with `A1` = "b" and `A2` = `#N/A` is `#CALC!` (C20). The cell error is dropped and evaluation goes on, so a later error argument still wins (`TEXTJOIN(",",TRUE,REPT("a",32767),A1:A2,NA())` is `#N/A`, errors C4).
Cell errors past the limit (probes/text-result-limit-errors, excel-graph, 2026-10-08, T-0284; `A1` = "b", `A2` = `#N/A`, `A3` empty): a cell error is `#CALC!` once the text read so far, plus for TEXTJOIN the delimiter that precedes the cell, is over 32,767, and the error itself otherwise.
- CONCAT: `CONCAT(REPT("a",32767),A1:A2)` is `#CALC!` ("b" takes it over, C1) and `CONCAT(REPT("a",32767),"b",A2)` is `#CALC!` (C10), but `CONCAT(REPT("a",32767),A2:A2)` and `...,A2)` are `#N/A` (exactly 32,767 is not over, C5-C6).
- TEXTJOIN: with a 32,767 prefix the delimiter takes it over before the cell, so `TEXTJOIN(",",TRUE,REPT("a",32767),A2:A2)` and `...,A2)` are `#CALC!` (C2-C3) while an empty delimiter leaves `#N/A` (C7); with a 32,766 prefix `","` is `#N/A` (C9) and `",,"` is `#CALC!` (C8); with ignore_empty FALSE an empty cell before the error adds a delimiter, so `TEXTJOIN(",",FALSE,REPT("a",32767),A3,A2)` is `#CALC!` (C11).
Other reference kinds behave like a direct A1 cell or range (probes/text-result-limit-errors-refs, excel-graph, 2026-10-08, T-0285; same layout, `Sheet2` repeats `A1:A2`): a defined name, `INDEX`, `IF`, `OFFSET`, `INDIRECT`, `INDEX(..):INDEX(..)` and a 3-D reference (`Sheet1:Sheet2!A1:A2`) are references, so a range that includes "b" gives `#CALC!` for CONCAT and TEXTJOIN (C1-C2, C4-C5, C7, C10, D1-D2, D4-D5, D7, D10), a single cell holding `#N/A` gives `#N/A` for CONCAT (exactly 32,767, C3, C6, C8, C11-C12) and `#CALC!` for TEXTJOIN (the delimiter takes it over, D3, D6, D8, D11-D12). `@A1:A2` from row 9 or 10 is `#VALUE!` (no intersection; C9, D9), which says nothing about the limit. Unprobed: TEXTJOIN with ignore_empty FALSE over a whole-column 3-D span (the engine's 3-D walk does not add the delimiters of the empty cells outside the used area, unlike the single-sheet walk).
Array arguments (probes/round-concat-array-edges F1-F7, with F1 the same `#CALC!` for the error-free row form, excel-graph 2026-10-13, T-1400): an error element of an array constant behaves like a range cell error. After 32,767 characters `CONCAT(REPT("a",32767),{"b",#N/A})` is `#CALC!` (so are `{"b";#DIV/0!}` and `{"b";"c"}`); an error met before the limit wins (`CONCAT({"b",#N/A},REPT("a",32767))` is `#N/A`, and `CONCAT(REPT("a",32767),{#N/A,"b"})` is `#N/A` since the error comes before any character overflows). With two errors the first wins (`CONCAT({"b",#N/A},{"c",#DIV/0!})` is `#N/A`). The engine matches.
Status: VERIFIED
Implemented in: `fn_concat`/`fn_textjoin` in functions/text/concatenate.rs (`ConcatText`, `JoinSize`) stop appending text once over the limit but still evaluate later arguments. `fn_concat` drops a range or reference cell error met once over the limit and continues; `fn_textjoin` does the same for range cells (through `JoinSize::push_cell`, also used for the sheet-by-sheet 3-D walk) and, in its `Node::ReferenceKind` arm, for a direct single-cell reference, counting the preceding delimiter through `JoinSize::exceeds_with_separator`. Both read their arguments through `evaluate_aggregate_arg` (which also treats `:` expressions, `Node::OpRangeKind`, as references), and `get_range` (reference_operators.rs) keeps an INDEX operand of `:` a reference (T-0243, T-0262, T-0284, T-0285)
Tasks: T-0243, T-0262, T-0284, T-0285, T-1400

## CONCAT joins an array argument element by element
Rule: `CONCAT` of an array (a function result, not only a range) joins its elements row by row, like a range; `CONCAT(MID(INDEX(Data!B3:E8,0.1,0),7,0))` is "".
Evidence: gen-m2/d0c29dd5c530e28c Calc!A246; probes/round-concat-array-lift (excel-graph, 2026-10-05): the first error wins in a CONCAT array.
Status: VERIFIED
Implemented in: functions/text/concatenate.rs (`fn_concat`)
Unprobed: an error element met inside an array once the text is over 32,767 characters is dropped like a range cell error (`CONCAT(REPT("a",32767),{"b",#N/A})`), unverified
Tasks: T-0222

## TEXTJOIN with ignore_empty FALSE joins every cell of a whole column or row
Rule: with ignore_empty FALSE every cell of a whole-column or whole-row argument adds a delimiter, however
small the used area: `TEXTJOIN(",",FALSE,D:D)` is `#CALC!` (1,048,576 values join to over 32,767
characters) and `TEXTJOIN(",",FALSE,10:10)` with H10 = "x" and J10 = "y" is ",,,,,,,x,,y" followed by
16,374 more commas. With ignore_empty TRUE only the non-empty cells count (`TEXTJOIN(",",TRUE,D:D)` =
"a,c"). `TEXTJOIN(",",FALSE,D1:D16385)` over the same sheet has length 16,386.
An error cell met after the skipped empties have taken the join over the limit is dropped: with A1 = "x",
A2 = "y" and A3 = `=NA()`, `TEXTJOIN(",",FALSE,1:3)` and `1:2` are `#CALC!`, while `1:1` joins to 16,384
characters and `TEXTJOIN(",",TRUE,1:3)`, `3:3` and `A1:A3` are `#N/A` (probes/textjoin-whole-rows-error-cell
A1-A7, excel-graph, 2026-10-29, T-3098).
Past the limit Excel stops reading cells: with A1 = "x", A2 = "y" and `=TEXTJOIN(",",FALSE,1:10)` in B5 (a cell
of its own range, below the row where the padding passes the limit) the result is `#CALC!`, not a circular
reference, while `=TEXTJOIN(",",TRUE,1:10)` in D3 is circular (0), since ignore_empty TRUE adds no padding and reads
every cell (probes/textjoin-whole-rows-self-reference B5, D3, excel-graph, 2026-10-29, T-3098). The engine's
whole-row walk with ignore_empty FALSE stops at the limit (`textjoin_push_padded_range`), also when the used area
spans every column (A1, XFD1, A2, XFD2 set: B5 `=TEXTJOIN(",",FALSE,1:10)` and B8 `=TEXTJOIN(",",FALSE,A1:XFD10)`
are `#CALC!`, D3 with ignore_empty TRUE is circular (0); probes/textjoin-whole-rows-self-reference-full-width,
excel-graph, 2026-10-29, T-3123).
Evidence: probes/whole-column-empty-walkers E2, E3, E6, F1, F2, F5, F8 (excel-graph, 2026-10-07, T-1114); probes/textjoin-whole-rows-error-cell A1-A7 and probes/textjoin-whole-rows-self-reference B5, D3 (excel-graph, 2026-10-29, T-3098); probes/textjoin-whole-rows-self-reference-full-width B5, B8, D3 (T-3123).
Status: VERIFIED
Implemented in: `fn_textjoin` (functions/text/concatenate.rs) walks the used area through the shared
sparse whole-range walker (T-2713) and adds the skipped empty cells per row (one walk per row)
and after the last row through `JoinSize::push_empties`, which stops at the limit.
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
Implemented in: `fn_fixed` (T-1257): the sign is dropped when the rounded text has no non-zero digit. Further shapes: `FIXED(-0.004,2)` and `FIXED(-0.4,0)` (probes/dollar-fixed-unichar-edges A49, A50).
Status: VERIFIED (excel-graph: gen-m6 A24, probes/dollar-fixed-unichar-edges A49, A50); desktop confirmation pending (T-1081). `FIXED(-0.005,2)` keeping its sign ("-0.01") is UNVERIFIED
Tasks: T-1257

## Count and position bounds of LEFT, RIGHT, MID and REPLACE
Rule: LEFT and RIGHT (and LEFTB) accept any non-negative num_chars, however large: `LEFT("abc",1E+100)` is "abc".
MID (and MIDB) is `#VALUE!` once start_num or num_chars reaches 2^31 (2^31-1 is fine: `MID("abc",2^31-1,1)` is
""). REPLACE is `#VALUE!` once start_num or num_chars exceeds 32,767, even past the end of a short text
(`REPLACE("abc",32768,1,"z")`, `REPLACE(REPT("a",10),11,32768,"z")`; 32,767 is fine and appends).
A negative count is `#VALUE!`: -1E-300 for LEFT (N18), -1E-15 for RIGHT and MID (B24-D24); 1E-300 counts as 0.
Evidence: probes/text-count-argument-bounds F-M rows 1-14, F20-F24, N17-N19 (excel-graph, 2026-10-11, T-0219);
gen-m6/a62335e72e44cb4e Calc!A9 (`REPLACE(-0.0001,123456789.123,3,"a*")` is `#VALUE!`, T-1239).
Status: VERIFIED. REPLACEB and RIGHTB delegate to REPLACE and RIGHT (unprobed themselves).
Implemented in: functions/text/common.rs (`mid_element`, `MID_ARGUMENT_LIMIT`), functions/text/string_format.rs
(`replace_element`, bounded by `MAX_TEXT_UNITS`).
Tasks: T-0219, T-1258

## LEFT, RIGHT and REPLACE round counts up near an integer
Rule: LEFT, RIGHT, LEFTB and REPLACE (start_num and num_chars) truncate their numbers, except that a value
within 2^-22 + 2^-33 (about 2.3842E-7) below an integer counts as that integer: `LEFT(t,1-2^-22-2^-33)`
is one character, `LEFT(t,1-2^-22-9*2^-36)` none; `REPLACE("abcdef",2.99999999,1,"z")` replaces the 3rd
character. The bound is absolute, identical below 1, 10, 1000 and 30000, and it is not round-to-nearest
(0.5 is 0, 2.5 is 2, 0.999999 is 0). MID and MIDB do **not** do this: they truncate exactly, so
`MID(t,1,1-1E-15)` is "" and `MID(t,1-1E-15,5)` is `#VALUE!` (start below 1). The negative check sees the
raw value (`LEFT(t,-1E-300)` is `#VALUE!`); REPLACE's start_num < 1 check comes after the rounding
(`REPLACE(t,1-2E-7,1,"z")` starts at 1, `1-1E-5` is `#VALUE!`). A computed `TRUE-1E-15` count is 1 for LEFT
(gen-m2/b4392a6514a36ade Calc!A14). INDEX's row and column indexes share it (lookup.md, T-1407); whether other
functions (REPT, FIND/SEARCH start_num, CHAR) do is UNVERIFIED. The cutoff is known to 2^-36: 2^-22 + 8·2^-36 (= 2^-22 + 2^-33) rounds up, 2^-22 + 9·2^-36 does not;
the engine takes the bound as exactly 2^-22 + 2^-33, inclusive. Unprobed: REPLACE start_num 32767.9999999 (rounds
up to 32,768, so the engine gives `#VALUE!`).
Evidence: probes/text-count-argument-bounds A-D, N-S, T-V, X-Y, Z, AA (excel-graph, 2026-10-11, T-0219).
Status: VERIFIED
Implemented in: functions/util.rs (`near_integer_count`), used by `left_element`, `right_element`
(common.rs) and `replace_element` (string_format.rs).
Tasks: T-0219

## Text function argument modes
Rule: the M5 text functions take scalar arguments that lift over a range or array (one result per cell, a spill;
probes/text-argument-modes F1:F26: LEN, CODE, EXACT, VALUETOTEXT, REPT, FIND), with four exceptions: TEXTJOIN's text
arguments and ARRAYTOTEXT's array take a range whole, PHONETIC's argument must be a reference, and T does not lift:
`T(A1:A3)` in a dynamic-array formula is one "" with no spill although A2 = "ab" (F9; see references.md "T of a
multi-cell range is not intersected"; T of an array value does lift, information.md "T over an array lifts
element-wise"). TEXTJOIN turns numbers and booleans in a range into their text
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
T-1248 (excel-graph, 2026-10-13; probes/text-modes-dbcs-phonetic-b, text-modes-dbcs-lift): en-US DBCS widens nothing: half-width katakana (`DBCS("ｱｲｳ")` = "ｱｲｳ", LEN 3), ASCII punctuation (`abc,.!?` stays, LENB 7), full-width, accented and CJK text, a literal argument and `~-` come back unchanged; a number cell gives text (`DBCS(5)` = "5"), an empty cell "", an error cell its error. DBCS lifts over a range like ASC does (`DBCS(A3:A5)` spills "5", "ｱｲｳ", "カタカナ"; `ASC(A1:A2)` spills two elements; `DBCS(A11:A12)` with the spill range blocked is `#SPILL!`). PHONETIC is `#N/A` for every reference probed: an error cell (`#N/A` and `#DIV/0!` both give `#N/A`), an empty cell, half-width and full-width katakana text, and ranges of any content, including a one-cell range and `ISNA(PHONETIC(..))` TRUE; the error is not propagated.
Generated cells (gen-m9, excel-graph, 2026-10-12, T-1289): ASC lifts over a range (`ASC(INDEX(A6:D8,0,3))` is
`#SPILL!`, gen-m9/65faaeb48419087d Calc!A5; ours matches, T-1411), and TEXT lifts over its format argument
(`TEXT(E7,FILTER(B1:E12,..))` and `TEXT(x,UNIQUE(D9:D12)&..)` are `#SPILL!`, gen-m9/89693ab0cafc29df A36 and
95efb37a9874f82e A16; ours matches, T-1410 and T-1458).
Ours: matches. DBCS returns its text unchanged and PHONETIC of any reference is `#N/A` (T-1213); DBCS and ASC lift over a range or array like Excel (T-1411, T-1488); the T-1248 findings are above. CODE and VALUETOTEXT
lift over ranges (T-1212). VALUETOTEXT's format (probes/valuetotext-format, excel-graph, T-1229): 1 quotes text and doubles embedded quotes (`"a""b"`; empty text is `""`), while a blank cell is empty, numbers, booleans and errors print as in format 0 (an error spells out, `#N/A`). The format is coerced to a number like any argument (`"1"`, TRUE and a blank cell work; other text is `#VALUE!`); a value in `[0, 2)` is truncated (`1.9` is 1, `1E-20` is 0), and anything else, `2`, `2.5`, `-0.5`, `-1`, is `#VALUE!`; an error format is returned as it is when the value is plain text; an error value with an invalid format wins as `#VALUE!` (`VALUETOTEXT(#N/A,2)`, `(#N/A,-1)` and `(#N/A,"x")` are `#VALUE!`), but with an error format the format's error wins (`VALUETOTEXT(#N/A,#DIV/0!)` and `VALUETOTEXT(5,#DIV/0!)` are `#DIV/0!`); an error value with a valid format prints as text (`VALUETOTEXT(#N/A,0)` and `(#N/A,1)` are "#N/A"). A one-cell range reference as the format works like a number (`VALUETOTEXT(A1,B1)` and `(A1,B1:B1)` with B1 = 1 give the quoted text `"ab"`), and a 2-cell range gives `#SPILL!` when its spill is blocked (it is an array result; the lifted values are UNVERIFIED) (probes/valuetotext-error-format, excel-graph, 2026-10-14, T-1288). The format lifts over arrays like the value (`VALUETOTEXT(A1:A2,{0,1})` is 2x2, `VALUETOTEXT(A1,{0,1})` 1x2). Ours: matches.
Status: VERIFIED (excel-graph)
Tasks: T-1087, T-1212, T-1213, T-1214, T-1229, T-1248, T-1288, T-1410, T-1411, T-1488

## BAHTTEXT
Rule: the number is rounded to 15 significant digits (as ROUND does), then rounded half away from zero to two decimals
(satang): `BAHTTEXT(0.995)` = "หนึ่งบาทถ้วน", `BAHTTEXT(2.675)` = "สองบาทหกสิบแปดสตางค์" and 1.005 gives one satang, so the
rounding is decimal, not binary. Digits past the 15th are zeros: 123456789012345.67 reads ...สี่สิบหกบาทถ้วน, 1e15+0.5 is
หนึ่งพันล้านล้านบาทถ้วน and 1.797e308 reads in full. Zero baht prints "ศูนย์บาทถ้วน", whole baht end in "บาทถ้วน", satang alone
print no baht ("ห้าสิบสตางค์" for 0.5) and both print "…บาท…สตางค์". The sign is a "ลบ" prefix and prints even when the amount
rounds to zero (-0.001 is "ลบศูนย์บาทถ้วน"; -0.005 is "ลบหนึ่งสตางค์"). Reading: 10 is สิบ, 20 ยี่สิบ, 21 ยี่สิบเอ็ด, a units
1 after anything is เอ็ด (101, 1001, 1000001), and the digits go in groups of six joined by ล้าน, even over an all-zero
group (10^12 is หนึ่งล้านล้าน, 1e100 is หนึ่งหมื่น plus sixteen ล้าน). The argument coerces like a number: "5", " 5 " and
"1e2" work, TRUE is 1, an empty cell is 0, "abc", "" and "1,5" are `#VALUE!`, an error
passes through; a range or array argument lifts (`BAHTTEXT({1,2.5})` spills "หนึ่งบาทถ้วน", "สองบาทห้าสิบสตางค์").
A satang that rounds to 100 carries into the baht (`BAHTTEXT(9.995)` = "สิบบาทถ้วน", `99.995` = "หนึ่งร้อยบาทถ้วน",
`999999.999` = "หนึ่งล้านบาทถ้วน"), also for a negative (`-9.995` = "ลบสิบบาทถ้วน"); a negative with baht and satang reads
`-1234.56` = "ลบหนึ่งพันสองร้อยสามสิบสี่บาทห้าสิบหกสตางค์"; 1000001000000 reads "หนึ่งล้านเอ็ดล้านบาทถ้วน" (the inner group's
units 1 is เอ็ด). UNVERIFIED: the argument count (`BAHTTEXT()`, `BAHTTEXT(1,2)`): excel-graph refuses a workbook with a
wrong-arity formula (createSession 501 unsupportedWorkbook), so only desktop Excel can answer (M14, decision 0039).
Evidence: probes/text-modes-bahttext, text-bahttext-wide, -wide2, -form1 to -form8, -text1 to -text3 (excel-graph,
2026-10-09, T-1214), text-bahttext-carry (excel-graph, 2026-10-14, T-1270).
Ours: matches (`fn_bahttext`, functions/text/bahttext.rs).
Status: VERIFIED (excel-graph)
Tasks: T-1214, T-1270

## TEXTSPLIT, TEXTBEFORE, TEXTAFTER and REGEX* lift their text and scalar arguments
Rule: the text of TEXTSPLIT, TEXTBEFORE, TEXTAFTER, REGEXTEST, REGEXEXTRACT and REGEXREPLACE lifts over a multi-cell range
or array, one call per element (`TEXTBEFORE(C1:C3,{",";"-"})` over "a,b", "c-d", "e,f" is 3x1 "a", "c", "e"), as do their
scalar arguments (`TEXTBEFORE("a,b,c",",",{1;2})` is 2x1 "a", "a,b"; the REGEX patterns, `REGEXTEST("abc",{"a";"z"})` 2x1).
The delimiters of TEXTSPLIT, TEXTBEFORE and TEXTAFTER are one list, not lifted: any entry matches, so `TEXTAFTER(C1:C3,
{",";"-"})` is "b", "d", "f". VERIFIED: probes/m7-argument-modes F13-K13 (excel-graph, 2026-10-09; T-1083), and gen-m7 cells
whose lifted result is `#SPILL!` because its spill range is occupied (gen-m7/93976dd9a3578ed9 Calc!A11, 59cc50701f4937c9 A24).
A lifted element whose call returns an array keeps its first value: TEXTSPLIT, probes/m7-argument-modes F13:H13.
An error inside a delimiter array is that error and a boolean is its spelling (T-1361, next entry).
VERIFIED (probes/textsplit-lift-guesses, excel-graph, 2026-10-19, T-1824):
- When several delimiters of a list match at one position the LONGEST wins, whatever the list order (`TEXTAFTER("abcd",{"ab","abc"})` and `{"abc","ab"}` are both "d"; `TEXTSPLIT("abcd",{"ab","abc"})` is {"","d"}; a negative instance_num alike, `TEXTAFTER("abcd",{"abc","ab"},-1)` is "d", A1-A9). T-1272 guessed the first in list order.
- Every list entry longer than the text gives the single-delimiter error: `#VALUE!` with more than two arguments, if_not_found included (`TEXTBEFORE("a","aa",1)`, `TEXTBEFORE("a",{"aa","bb"},1)`, `TEXTAFTER("a",{"aa","bb"},1,0,0,"nf")`), `#N/A` with two (A12-A14, A16, A17). If one entry fits the text the call simply finds nothing: `#N/A`, or if_not_found (`TEXTBEFORE("a",{"aa","b"},1,0,0,"nf")` is "nf"; A10, A11, A21; `#N/A` with 3 or more arguments and no if_not_found is inferred). `TEXTBEFORE("a",{"aa","a"},1)` is "" because "a" is found (A15). TEXTSPLIT returns the text when every entry is too long (A18, A19); `TEXTSPLIT("a",{"aa","a"})` splits into two empty pieces because "a" fits (A20).
- A REGEX pattern that does not compile is `#VALUE!` in its own element only (`REGEXTEST("abc",{"a";"(";"b"})` is TRUE, `#VALUE!`, TRUE; REGEXEXTRACT and REGEXREPLACE alike; A22-A27).
- if_not_found, match_end, match_mode, TEXTSPLIT's ignore_empty and REGEXEXTRACT's return_mode lift like the other scalars, and an array if_not_found lifts even when the delimiter is found (`TEXTBEFORE("a,b",",",1,,,{"x";"y"})` is 2x1 "a","a"); a scalar error as if_not_found is returned only when the delimiter is not found (`TEXTBEFORE("a,b",",",1,,,1/0)` is "a", with instance 3 `#DIV/0!`; A28-A34, A39-A48).
- TEXTSPLIT's pad_with does NOT lift: an array of any size, a 1x1 one included, is `#VALUE!` (`TEXTSPLIT("a,b;c",",",";",,,{"x";"y"})`, `COLUMNS(TEXTSPLIT("a,b,c",",",,,,{"x"}))`; A35-A38, A65). A single-cell or multi-cell range as pad_with is UNVERIFIED (ours: a single cell is its value, a range `#VALUE!`). An error argument comes before the array pad_with's `#VALUE!`, which is per element of a lifted call (see "TEXTSPLIT: argument errors first, an all-empty result").
- REGEXEXTRACT in a lifted call keeps the first value of each element (`REGEXEXTRACT({"a1b2";"c3d4"},"[0-9]",1)` is 2x1 "1","3"; A47-A52).
- A 1x1 array lifts the call: `COLUMNS(TEXTSPLIT({"a,b"},","))`, `COLUMNS(TEXTSPLIT(FILTER(Data!A1:A2,Data!A1:A2="a,b"),","))` and `COLUMNS(REGEXEXTRACT({"a1b2"},"[0-9]",1))` are 1, as is `ROWS(...)` (every lifted element keeps its first value, so `COLUMNS(TEXTSPLIT({"a,b";"c,d,e"},","))` is 1 and its ROWS 2); `COLUMNS(TEXTSPLIT(Data!A3,","))` over a single cell is 3, a non-lifting call (A56-A70).
- A 1x1 array as the delimiter does not lift (`COLUMNS(TEXTSPLIT("a,b",{","}))` is 2, A64); as ignore_empty or a REGEX pattern it does (A66, A67). A one-cell range `Data!A3:A3` does not lift (A70). `INDEX(TEXTSPLIT({"a,b"},","),1,2)` is `#REF!` because the lifted call has one column (A68).
- A scalar boolean delimiter is its spelling in TEXTSPLIT as in TEXTBEFORE (`TEXTSPLIT("a TRUE b",TRUE)` is "a "," b", A71-A73). Of two different errors in one delimiter list the first element wins (whether the scan is row-major or column-major is UNVERIFIED: every 2x2 cell has its winner at (1,1); ours row-major; `{#N/A,#DIV/0!}` is `#N/A`, `{#DIV/0!,#N/A}` is `#DIV/0!`, `{#REF!,#N/A;#VALUE!,#DIV/0!}` is `#REF!`, A74-A81), and an error in the list beats an invalid instance_num (`TEXTBEFORE("abc",{"",#N/A},0)` is `#N/A`; without the error `#VALUE!`, A82-A87).
- REGEXEXTRACT's return_mode 2 keeps the first group of each lifted element (`REGEXEXTRACT({"a1b2";"c3d4"},"([a-z])([0-9])",2)` is "a","c"; A53) and its fourth argument case_sensitivity lifts (`REGEXEXTRACT("a1B2","b",0,{0;1})` is `#N/A`,"B", so its TEXTJOIN is `#N/A`; A55). Ours since T-1360 ("REGEX* optional arguments" below).
Ours: all of this since T-1272. The work caps are the next entry's. A multi-cell range as the TEXTBEFORE/TEXTAFTER
delimiter is still read as one text, as before T-1272 (gen-m7/851eb2099ae3f805 Calc!A25 is `#VALUE!`); its list rule is
T-1361's.
Status: VERIFIED (excel-graph) for the cited cells; UNVERIFIED as noted above
Tasks: T-1272, T-1824 (probe the guesses), T-1361 (delimiter ranges)

## Lifted TEXTSPLIT, TEXTBEFORE, TEXTAFTER and REGEX* calls are capped
Rule: Excel computes these calls whatever their size. The REGEX* limits below also bound an XMATCH or XLOOKUP call in
regex mode (`match_mode` 3, T-2680): one call, all its lifted elements included, has one REGEX* budget
(`LookupRegexes`; lookup.md "XMATCH and XLOOKUP regex mode").
Ours: a call is `#ERROR!` (as the *IF(S) lift) when its work passes `limits::MAX_LIFTED_CELL_VISITS` (2^27): over
every operand, each element's length plus 1, times how often the broadcast repeats it; the text's length is further
multiplied, per position, by the delimiter count plus the delimiter bytes compared there (each delimiter at most the
text's length) over 64. A call that does not lift is capped the same way (`TEXTBEFORE(REPT("a",32767),SEQUENCE(10000))`,
`TEXTBEFORE(REPT("a",32767),REPT("a",16383)&"b"&SEQUENCE(512))`); one long cell in a blank column costs its own length
once (`ROWS(TEXTBEFORE(A1:A100000,","))` is 100000). TEXTSPLIT searches each delimiter again from
each piece, so its delimiter bytes count 8 to a visit rather than 64
(`ROWS(TEXTSPLIT(REPT("a",32767)&SEQUENCE(16),HSTACK("a",REPT("a",16000))))` is `#ERROR!`). A REGEX* call compiles each
distinct pattern once, under the smallest of the size limits 64 KiB, 256 KiB, 1 MiB, 4 MiB and 10 MiB (the regex
crate's default, so every pattern that compiled before T-1272 still does) it fits; it may spend 128 MiB of limits on
compiling, too-big attempts included (2,048 small patterns), and hold 64 MiB of patterns and search caches, measured
after each search (none is evicted, so patterns that cycle compile once). Its searches (functions/text/regex_engine.rs)
run on a lazy DFA that gives up when it thrashes (after 3 cache clears, below 10 bytes a state, the regex crate's own
setting) and then on a PikeVM; the DFA is charged what its cache says it scanned (4 a byte) and built (8 a byte of
state), the PikeVM (T-1829) 21 for each NFA state a scan visits that measures the span a leftmost-first search needs, and
54 more for each look-around it evaluates (a Unicode `\b` or `\B` costs about three visits; a chain of 40 over
`REPT("éa ",10000)&SEQUENCE(300)` is `#ERROR!` after about 1 s), and 3 more for each range of a sparse class state
(a large Unicode class) its byte step walks (T-1988: `\b[\w\s\d]{30}q` over `REPT("日a ",8000)&SEQUENCE(80)` is
`#ERROR!` after about 1 s, not 1.6) (the PikeVM then runs over that span alone; an anchored search is charged its span times its state count, 5 a
state-byte), 64 a search, and a call may
spend 2^30 (about 1 s of the slowest searches; `SUM(--REGEXTEST(REPT("y",200)&SEQUENCE(100000),"x"))` fits, 20 MB).
A pattern whose DFA thrashes over a lot of text (`a[ab]{100}c|b[ab]{100}c` over 560 texts of 30,000 bytes) and a call
past any of these limits is `#ERROR!` (`ROWS(REGEXTEST("a","a"&SEQUENCE(5000)))`). A pattern with `\b` runs on the
PikeVM over non-ASCII text (the lazy DFA quits there); iterating its matches costs what the searches read
(`COLUMNS(REGEXEXTRACT(REPT("naïve ",500),"\b\w+\b",1))` is 500 since T-1829, the match count Excel's
`REPT("naïve ",3)` case (3, probes/regex-ascii-classes A5) leads one to expect; T-1272 charged each
search the rest of the text; a lazy `.*?` is charged up to each match since T-2015, the scan dropping the threads
below a match as the PikeVM does: `COLUMNS(REGEXEXTRACT(REPT("é ab ",4000),"\bab.*?\b",1))` is 4000), while a
search that truly reads to the end every time (`a.*z|a|\bq` over
`REPT("a",20000)&"é"`) is still `#ERROR!`. A REGEXREPLACE searches twice,
once to size its result: over 65,535 UTF-16 units it is
`#CALC!` as Excel's is, its length known before it is built (`LEN(REGEXREPLACE(REPT("a",32767),"","bbbbbbbbbb"))`;
"REGEXREPLACE results are capped at 65,535 UTF-16 units" below); a lifted REGEXREPLACE whose results pass the text
broadcast cap (64 MiB, first entry) is one `#VALUE!`. The brackets holding a POSIX class under `(?i)` of one
pattern that would together be rewritten past 64 KiB (their other items folded explicitly, "REGEX* POSIX classes are Unicode (T-1903)") is an invalid
pattern, `#VALUE!` (T-1989). Folding an item also spends one byte of that 64 KiB per 256 code points its ranges cover, however short it
prints: about 15 `(?i)[[:upper:]\x{0}-\x{10FFFF}]` are `#VALUE!` (T-2098; Excel accepts them).
Any pattern that may fold case (case_sensitivity 1 or a `(?` group) is also charged one byte of the call's compile budget per code point its
explicit ranges cover, before it compiles: about 110 distinct `(?i)[\x{0}-\x{10FFFF}]` in one call are `#ERROR!` (T-2138; Excel accepts them).
The REGEX* dialect is the regex crate's on every target (wasm32 used `regex_lite`, whose `\w`, `\d` and `\s` are
ASCII and which rejects `\pL`, before T-1272): `\w` and `\b` are Unicode (`REGEXTEST("é","\w")` is TRUE), as
Excel's are (T-1817, below); POSIX classes are rewritten to Excel's Unicode ones ("REGEX* POSIX classes are Unicode (T-1903)").
The REGEXREPLACE replacement syntax is Excel's since T-1688: see "REGEXREPLACE replacement syntax (T-1688)"; the
pattern dialect still differs from Excel's PCRE2: see "REGEX* patterns and match iteration are PCRE2 (T-1688)".
Evidence: none needed for Excel's side (it has no such limit); CLAUDE.md directive 5.
Pattern limits of our PCRE2 parser (T-1973, functions/text/pcre/parse.rs):
- Nesting is capped at 250 levels. That is PCRE2's default parentheses limit, UNVERIFIED for Excel (T-1972).
- Stacked quantifiers and nested brackets also count toward that cap. This is our own directive-5 limit, a
  DEVIATION: `REPT("(",250)&"a*"&REPT(")",250)` is `#VALUE!`, as it already was under regex-syntax's nest limit.
- Repeat counts are capped at 65,535 (PCRE2's), as are capture groups. Group names are capped at 32 characters
  (PCRE2 before 10.44, which may have raised it). All of these are UNVERIFIED for Excel (T-1972).
Backtracker limits (T-1900, functions/text/pcre/backtrack.rs and program.rs):
- Every executed instruction, pushed frame, backreference character, atomic-group scan and start position is
  charged to the call's one work budget. The stack may not pass the heap the call may still hold (64 MiB less what
  it holds). Past either, the call is `#ERROR!`. Stock PCRE2 has a required-character check and a match limit
  instead. `REGEXTEST(REPT("a",30)&"c","\G(a|a)*b")` is `#ERROR!` here, where PCRE2 answers FALSE at once.
  `\G(a|a)*b`, `\G(a*)*b`, `\G((a+)+)+b` and `(.*)\1x` over `REPT("a",32767)` are `#ERROR!` or FALSE. Excel's
  answers are UNVERIFIED (T-2356).
- Counted repeats are expanded into copies, and every node a copy visits counts against the compile size limit.
  `(?:(?:){65535}){65535}(a)\1` is too big, and `#VALUE!`, where PCRE2 compiles it.
- The memoized mode (T-2355; a regular pattern with `$` or `\Z` over a text with a line feed before its last
  character) keeps one bit per instruction and byte position it reaches, in chunks of 64 positions, held with the
  stack within what the call may still hold (64 MiB at most): 4 KiB per instruction over a 32,767-character ASCII
  text. A search that would pass it is `#ERROR!`, where Excel answers FALSE (checked once on excel-graph,
  2026-10-22, for T-2355; not kept as a case, being this deviation)
  (`REGEXTEST("a"&CHAR(10)&REPT("a",32764)&CHAR(10),"(?:b[a-z]{20000}|a)*b$")`). Each chunk it reaches is charged one unit
  a word, and each state it enters a step.
Status: DEVIATION (decision 0014)
Implemented in: the fold budget of `pcre::parse` (functions/text/pcre/parse.rs, charged per code point a fold walks), `Model::lift_text_call`, `text_call_cost` and `ScanCost` (functions/text/lifted.rs), `PatternCache` and `LookupRegexes` (regex_functions.rs)
and `Replacement` (functions/text/regex_replacement.rs), `Engine` and `Search` (functions/text/regex_engine.rs), `Backtracker` and `Program` (functions/text/pcre/), `SplitList` (functions/text/textsplit.rs)
Tasks: T-1272, T-1829, T-1988, T-1989, T-2098, T-2138, T-2355, T-2680

## Lifted FIND and SEARCH calls are capped
Rule: Excel computes a lifted FIND, FINDB, SEARCH or SEARCHB whatever its texts' lengths.
Ours: a lifted call is `#ERROR!` ("Too many cells to visit") before its first element when its work passes
`limits::MAX_LIFTED_CELL_VISITS` (2^27). Each element is charged both texts' UTF-8 lengths plus 1 and, at each of the
`within_text` length plus 1 start positions, 1 plus the bytes compared there (at most the shorter text's length) over
256 for FIND (a `memcmp`) and over 8 for SEARCH (its wildcard segments compare unit by unit); a number or a logical
counts as 32 bytes, an error or a blank as none. `FIND(REPT("a",32766)&"b",REPT("a",32763)&SEQUENCE(1000))`, about
10^9 compared bytes per element, is `#ERROR!`; `SUM(FIND("a",A1:A100000))` over short texts runs, and a call that does
not lift is not charged (`FIND(REPT("a",32766)&"b",REPT("a",32767))` is `#VALUE!`, after one scan).
Even a one-character search is charged each start position, as TEXTBEFORE reads each position: a lift whose
`within_text`s add up past about 2^27 bytes is `#ERROR!` (`SUM(--ISNUMBER(SEARCH("x",A1:A5000)))` over 32,767-character
cells), where 100,000 texts of 1,000 bytes run.
Status: DEVIATION (decision 0014)
Implemented in: `ValueWork::TextScan` and `text_scan_units` (functions/lift/kinds/mod.rs), the FIND and SEARCH entries
(functions/lift/kinds/text.rs)
Tasks: T-1775, T-1961

## TEXTSPLIT, TEXTBEFORE and TEXTAFTER delimiter ranges
Rule: a multi-cell range of any shape is a delimiter list, like an array constant (arrays.md "Dynamic-array and LAMBDA
function argument modes"). Each element is read as text: a number as its text (`1E+15` as `1000000000000000`, the
spelling `number_to_text` gives, so `TEXTBEFORE("a1E+15b",{1E+15,"q"})` is `#N/A`), a boolean as TRUE or FALSE
(`TEXTBEFORE("a false b",{FALSE,"q"},,1)` is "a ", case-folded with match_mode 1), and a blank cell or an empty-string
cell as the empty delimiter. An error element makes the call that error, even beside the empty delimiter
(`TEXTBEFORE("abc",{"",#N/A})` is `#N/A`; a range holding `=1/0` gives `#DIV/0!`; TEXTSPLIT alike). So in TEXTSPLIT a
blank cell in the col_delimiter range is `#VALUE!` (the empty delimiter, see "TEXTSPLIT with an empty delimiter is
#VALUE!"), and so is one in the row_delimiter range (`TEXTSPLIT("abc",",",A9:B9)` with A9 blank). In TEXTBEFORE
and TEXTAFTER the empty delimiter decides the result alone, see the next entry; this explains the gen-m9 cells that
looked inconsistent (`TEXTBEFORE(2,A9:D10)` is "2" because the text is one character long, `TEXTBEFORE(42,C11:D11,1E+15)`
is "" because the empty delimiter ignores instance_num's size), and `TEXTAFTER(" x ",A6:A10,A9)` (A6 blank, A9 42) is
" x ", which the gen-m7 cell's subtraction turns into `#VALUE!`.
Evidence: probes/textbefore-delimiter-range (54 cells), probes/textbefore-delimiter-list-elements A1-A7, A22, A23,
A31-A34, probes/textbefore-number-delimiter (excel-graph, 2026-10-18, T-1361); gen-m7/8ed8a680e9794b77 Calc!A17, A26;
gen-m7/0e40208f51c2454c Calc!A17; gen-m7/4629e931955aa4b8 Calc!A19; gen-m7/851eb2099ae3f805 Calc!A25 (excel-graph,
2026-10-11, T-1275); gen-m9/0c7fc447bc2eebfd A6, 4df1b22d3ed63f58 A14, A30, 3ec66461400faf2f A3, 696af2c8cee2f57e A23,
95efb37a9874f82e A25 (excel-graph, 2026-10-12, T-1289).
Ours: matches (`delimiter_list` in functions/text/lifted.rs). The error of a list with two different errors is the first in row-major order (UNVERIFIED).
Status: VERIFIED (web)
Tasks: T-1361, T-1412

## TEXTBEFORE and TEXTAFTER with the empty delimiter
Rule: when the delimiter (a scalar `""`, a blank cell, or any element of a list) is empty, the result depends only on the
text and the sign of instance_num, not on the other delimiters, the size of instance_num (`TEXTBEFORE("42","",1E+15)` is
"", `TEXTBEFORE("abc","",2)` is "" too), match_mode, match_end or if_not_found: the empty delimiter matches at the start
of the text, so TEXTBEFORE is "" and TEXTAFTER the whole text, and with a negative instance_num at its end (TEXTBEFORE the
whole text, TEXTAFTER ""). A text one UTF-16 unit long always matches at the end, whatever the sign:
`TEXTBEFORE("2","")`, `TEXTBEFORE("é","")`, `TEXTBEFORE(1,"")` and `TEXTBEFORE("q",{"","q"})` are the whole text, their
TEXTAFTER "", while `TEXTBEFORE("😀","")` (two units) is "". An empty text gives "" (not the usual `#N/A` of
`TEXTBEFORE("",x)`). instance_num 0, a match_end other than 0 or 1 and a non-numeric instance_num are still `#VALUE!`.
Evidence: probes/textbefore-empty-delimiter-list (240 cells: texts of 0-3 characters, the empty delimiter alone and
beside delimiters of each length, instance_num 1, -1 and 2), probes/textbefore-delimiter-list-elements A8-A21, A24-A30,
A35, A36 (excel-graph, 2026-10-18, T-1361).
Ours: matches (`around_empty_delimiter` in functions/text/before_after.rs).
Status: VERIFIED (web)
Tasks: T-1361

## TEXTBEFORE and TEXTAFTER: overlapping delimiter instances
Rule: TEXTBEFORE and TEXTAFTER count only disjoint delimiter matches, unlike SUBSTITUTE's instance_num ("SUBSTITUTE
instances overlap" above). From the start, the scan counts the longest delimiter that starts at a position and resumes
after its end: `TEXTBEFORE("aaa","aa",2)` is `#N/A`, `TEXTBEFORE("aaaa","aa",2)` "aa", `,3)` `#N/A`, `TEXTAFTER("aaaaa","aa",2)`
"a"; `TEXTBEFORE("-2.5",{"2.5","-2.5"},2)` is `#N/A` in either list order ("-2.5" at unit 0 hides the "2.5" inside it);
`TEXTBEFORE("xabcx",{"b","abc"},2)` is `#N/A` ("abc" at unit 1 hides the "b"). With a negative instance_num the scan
takes start positions from the last back, and after a counted match steps back by that match's length and goes on from
there, checking the whole rest of the text at each position, so a longer delimiter may still run into the counted match
(`TEXTAFTER("3.141590.2-0.5x~*",C5:D12,-2)` with "0.2" and "2" among the delimiters counts "2" at unit 9, then "0.2" at
unit 7, and is "-0.5x~*"; gen-m7/a470dc9e165bdb20 Calc!A1441), while the same delimiter cannot:
`TEXTBEFORE("aaa","aa",-1)` "a", `-2` `#N/A`; `TEXTBEFORE("aaaaa","aa",-2)` "a"; `TEXTBEFORE("ababa","aba",-1)` "ab",
`-2` `#N/A`; `TEXTBEFORE("xabcx",{"abc","b"},-1)` is "xa" (the later start wins, even though "abc" ends later);
`TEXTBEFORE("abcd",{"ab","bc","cd"},-2)` is "" ("cd", then "ab"; "bc" overlaps "cd"). match_mode 1 counts the same way
(`TEXTBEFORE("AAA","aa",2,1)` `#N/A`, `TEXTBEFORE("aAaA","AA",2,1)` "aA"). With match_end 1 the end of the text (the
start, from the end) is one more instance after the last disjoint match, not only when nothing matches:
`TEXTBEFORE("aaa","aa",2,,1)` is "aaa", `TEXTBEFORE("aaa","aa",-2,,1)` "", `TEXTAFTER("aaa","aa",2,,1)` ""; and
`TEXTBEFORE("aaa","aa",2,,,"nf")` is "nf". The `#VALUE!` bound on instance_num ("instance_num past the text" below)
counts positions, not disjoint matches: `TEXTBEFORE("aaa","aa",3)` is `#VALUE!` but `TEXTBEFORE("aaaa","aa",3)` `#N/A`.
TEXTSPLIT splits at the same disjoint matches: `TEXTSPLIT("aaa","aa")` is "","a", `TEXTSPLIT("aaaaa","aa")` "","","a",
`TEXTSPLIT("-2.5",{"2.5","-2.5"})` "","", `TEXTSPLIT("ababa","aba")` splits only at unit 0, `TEXTSPLIT("xabcx",{"b","abc"})` "x","x".
Evidence: probes/t3335-textbefore-overlapping-instances A1-A58, C1-E29 (excel-graph, 2026-10-31, T-3335);
gen-m9/cde551ea787f713a Calc!A37 (`TEXTBEFORE("-2.5",{2.5,-2.5},2)` inside BETADIST, `#N/A`); gen-m7/a470dc9e165bdb20
Calc!A1441 (excel-graph). The backward step is inferred from these cells: whether it is the match's length or the
shortest delimiter's (they agree on every cell) is unprobed, as is a forward scan's step past a match whose delimiter
list holds a shorter entry starting inside it.
Ours: matches (`Instances::Disjoint` in functions/text/util.rs; TEXTSPLIT's `split_grid` already scanned disjointly).
Status: VERIFIED (excel-graph)
Tasks: T-3335

## TEXTSPLIT of empty text is #VALUE!
Rule: TEXTSPLIT of empty text is `#VALUE!` whatever its valid other arguments: a literal `""`, a blank cell and an empty-string cell,
with or without a row delimiter, with `ignore_empty` TRUE, `match_mode` 1 or a `pad_with`. `TEXTSPLIT("","a","TRUE")` is `#VALUE!` too, so
in `B5>TEXTSPLIT("","a",CONCAT(D9))` with B5 blank the comparison is `#VALUE!`. (TEXTBEFORE("",x) is `#N/A` instead, see the entry above.)
Evidence: gen-m7/3a62e0950c1074fd Calc!A5 (excel-graph, 2026-10-11, T-1275); probes/textsplit-empty-text Calc!A1-A18 (excel-graph,
2026-10-17; an empty delimiter with empty text, A9 and A10, is `#VALUE!` as well).
An error-valued argument wins over the empty text: `TEXTSPLIT("",NA())` and `TEXTSPLIT("",",",NA())` are `#N/A`,
`TEXTSPLIT("",",",,1/0)` and `TEXTSPLIT("",",",,,1/0)` are `#DIV/0!`. Over an array or range text each empty element is `#VALUE!`
and the others split normally: `TEXTSPLIT({"a","";"b","c"},",")` is `{"a",#VALUE!;"b","c"}`.
Ours: matches (T-1347, `textsplit_element` in functions/text/textsplit.rs).
Status: VERIFIED (web)
Tasks: T-1347

## TEXTSPLIT with an empty delimiter is #VALUE!
Rule: an empty delimiter is `#VALUE!` in either list, even when the other list has a non-empty one: a `""` string
(`TEXTSPLIT("abc","b","")`, `TEXTSPLIT("abc","","b")`), an empty element of a list (`{"b",""}`, `{",",""}`), a blank
cell alone or in a range, and an empty-string cell. An omitted delimiter is no delimiter and is fine beside the other one
(`TEXTSPLIT("a,b",",",,TRUE)` is {"a","b"}). `TEXTSPLIT("TRUE"&"abc"&"x~*","TRUE1E-15a*","")` is `#VALUE!`.
Evidence: probes/textsplit-empty-delimiter A1-A12 (excel-graph, 2026-10-18, T-1361); probes/textbefore-delimiter-range
A30, A31, A33, A34, A44, A45; gen-m9/d244411dd90f7fbd Calc!A6 (excel-graph, 2026-10-12, T-1289); gen-m7/a0cbc75358b429b6
Calc!A10 (a blank cell in a row_delimiter range).
Ours: matches (`textsplit_element` in functions/text/textsplit.rs; `delimiter_list` reads a blank cell as "").
Status: VERIFIED (web)
Tasks: T-1412, T-1361

## TEXTSPLIT match_mode 1 case folding
Rule: with match_mode 1 TEXTSPLIT compares case-insensitively without changing the length of the text. `İ` (U+0130) matches `i`
(`TEXTSPLIT("aİbİc","i",,,1)` splits at both İ, and so does the delimiter `İ` against text containing `İ`: {"a","b","c"}; the delimiter `İ` against a plain `i` or `I`, and `I` against `İ`, are UNVERIFIED), unlike SEARCH, where `SEARCH("i","İx")`
is `#VALUE!`. The Kelvin sign U+212A does not match `k` (`TEXTSPLIT("a"&UNICHAR(8490)&"bkc","k",,,1)` is {"aKb","c"}), unlike SEARCH, where
`SEARCH("k",UNICHAR(8490))` is 1. `ß` and `ẞ` do not match `ss` (`TEXTSPLIT("aßbSSc","ss",,,1)` is {"aßb","c"}), and `Ä` matches `ä`.
Pieces are cut from the original text, so they keep their case. Final sigma follows SEARCH (`ς` and `Σ` both fold to `σ`), which changed from the pre-T-1794 string-level lowercase; it is UNVERIFIED for TEXTSPLIT.
Evidence: probes/textsplit-casefold-length A1-A8 (excel-graph, 2026-10-17, T-1794).
Ours: `fold_for_split` (text/casefold.rs) folds char by char (one char in, one char out) with those two exceptions; positions are mapped back to
byte offsets of the original text. Before T-1794 the code sliced the original at byte offsets found in `to_lowercase()`, which panicked on
`TEXTSPLIT("İİİ,a",",",,,1)`. The folding of other chars (Å U+212B, ſ, µ, final sigma, titlecase digraphs) is UNVERIFIED and follows SEARCH's.
Status: VERIFIED (web) for the listed forms
Tasks: T-1794

## TEXTBEFORE and TEXTAFTER match_mode 1 case folding
Rule: with match_mode 1 TEXTBEFORE and TEXTAFTER fold case like TEXTSPLIT, never changing the length of the text: `İ` (U+0130) matches `i`
in both directions (`TEXTBEFORE("aİbİc","i",1,1)` is "a", `TEXTAFTER` of it is "bİc", and the delimiter `İ` finds the `i` of "aibic"),
the Kelvin sign U+212A does not match `k` in either direction (`TEXTBEFORE("a"&UNICHAR(8490)&"bkc","k",1,1)` is "aKb"; the delimiter
U+212A against "akbkc" is `#N/A`), `ß` does not match `ss` (`TEXTBEFORE("aßbSSc","ss",1,1)` is "aßb") and `ẞ` (U+1E9E) does not match
the delimiter `ß` (`TEXTBEFORE("assbẞc","ß",1,1)` is `#N/A`, although `ẞ` lowercases to `ß`), and `Ä` matches `ä`
(`TEXTBEFORE("ÄxäyÄ","ä",2,1)` is "Äx"). The result is cut from the original text, so it keeps its case. Positive and negative
instance numbers behave the same on `"İİİ,a,b"`. Sigma is positional, unlike TEXTSPLIT and SEARCH, with LOWER's final-sigma rule: `Σ` followed by a letter or digit is `σ`, any
other `Σ` (end of text, space, punctuation) is `ς`, and `ς` is only `ς`. So the delimiter `Σ` alone matches a text `ς` but not `σ`, the delimiter `ς` matches a final `Σ`
("aΟΣ") but neither `σ` nor a `Σ` followed by a letter, and the delimiter `σ` matches a `Σ` followed by a letter but not `ς`.
The same holds for the text and a delimiter that are both `Σ` (`TEXTBEFORE("aΣb","Σ",1,1)` is `#N/A`: the text's `Σ` is `σ`, the lone delimiter `Σ` is `ς`), and for TEXTAFTER.
A `Σ` before a digit is `σ` (`TEXTBEFORE("ΟΔΟΣ1","σ",1,1)` is "ΟΔΟ", delimiter `ς` is `#N/A`); before a space or a comma it is `ς` (`"ΟΔΟΣ ΚΑΙ"` and `"ΟΔΟΣ,Α"` match `ς`, not `σ`).
Evidence: probes/textbefore-casefold-length A1-A25 (excel-graph, 2026-10-17, T-1796); probes/textbefore-sigma-contexts A1-A18 (excel-graph, 2026-10-19, T-1799).
Ours: `fold_for_before_after` (text/casefold.rs) folds char by char with `fold_for_split`'s İ/Kelvin exceptions plus the sigma and `ẞ` rules,
and `match_text` (util.rs) maps the match back to byte offsets of the original with `str::get`. Before T-1796 it searched `to_lowercase()` of the text
and sliced the original at those offsets, which panicked on `TEXTBEFORE("İİİ,a",",",1,1)`. Ours: the final-sigma rule is `sigma_is_final` (string_format.rs), shared with LOWER. UNVERIFIED: whether a `Σ` before a combining mark, `’` or a zero-width space is `σ` (LOWER's rule, assumed), whether a letter must precede the `Σ` (the code ignores what precedes it), the folding of other chars (Å U+212B, ſ, µ,
titlecase digraphs, which follow SEARCH's `fold_char`), `ẞ` against the delimiter `ß`'s mirror (text `ß`, delimiter `ẞ`), and whether TEXTSPLIT shares the
`ẞ` and sigma rules (its probe has no such cell).
Status: VERIFIED (web) for the listed forms
Tasks: T-1796

## ARRAYTOTEXT number text
Rule: ARRAYTOTEXT writes a number as General text with 15 significant digits, like VALUETOTEXT and `&`:
`ARRAYTOTEXT(YEARFRAC(1911,31142))` is "80.0277777777778".
Evidence: gen-m7/d38b52bfa1a9f9e6 Calc!A2 (excel-graph, 2026-10-11, T-1275).
Also probed (probes/arraytotext-number-text, 15 cells, excel-graph, 2026-10-16): `ARRAYTOTEXT(-0.0000001234567890123)` is "-1.234567890123E-07"; the array `{0.3333333333333333,1E+20}` is "0.333333333333333, 1E+20" (format 0) and "{0.333333333333333,1E+20}" (format 1). **A scalar is a one-element array in strict form**: `ARRAYTOTEXT(1/3,1)` is "{0.333333333333333}", `ARRAYTOTEXT("abc",1)` "{""abc""}" and `ARRAYTOTEXT(TRUE,1)` "{TRUE}"; format 0 gives the bare text; `ARRAYTOTEXT(#N/A,1)` is `#N/A`. **A reference to one empty cell is `#VALUE!`** in both formats (the engine gives ""; T-1718).
Implemented (T-1356): `format_array_value` uses `number_to_text`, the helper VALUETOTEXT and `&` use; `ARRAYTOTEXT(1/3)` is "0.333333333333333" and `ARRAYTOTEXT({1E+20})` "1E+20".
Status: VERIFIED (web)
Tasks: T-1356

## SEARCH folds case one character at a time
Rule: SEARCH lowercases per character and never changes the text's length: a character whose full lowercase is longer (U+0130 "İ" is "i" + U+0307 in Unicode) is left as it is, so it neither shifts later positions nor matches "i".
`SEARCH("x","İx")` is 2, `SEARCH("x","İİx")` is 3 (also with start 3), `SEARCH("i","İx")` and `SEARCH("i̇","İx")` are `#VALUE!`; `SEARCH("x","ẞx")` is 2 and `SEARCH("ss","ẞx")` is `#VALUE!`. `SEARCH("","")` and `SEARCH("","",1)` are `#VALUE!` (no start fits), `SEARCH("","abc")` is 1 and `SEARCH("","abc",4)` is `#VALUE!`.
Also: ς (final sigma) matches Σ and σ in either direction (`SEARCH("ς","ΟΣ")` is 2, `SEARCH("σ","ΟΣ")` is 2), ǆ matches ǅ, and U+212A (Kelvin) matches k; ſ/s, µ/μ, ϐ/β and ﬀ/ff do not match (`#VALUE!`).
Evidence: probes/text-search-casefold-length A1-A24 (oracle: excel-graph, 2026-10-12, T-1296).
Status: VERIFIED (excel-graph)
Implemented in: `fold_case` (functions/util/collation.rs; shared with exact-match lookups and wildcard criteria, T-3133).
Tasks: T-1296

## TEXT over an array of format codes
Rule: TEXT lifts over its format argument like over its value: a range or array of formats gives one result per format (value and formats broadcast together). Each format gets its own complexity check (a too-complex format is `#VALUE!` in its element only) and an error element of the format array stays that error.
`TEXT(1,{"0",#N/A})` is `{"1",#N/A}`.
Evidence: probes/text-format-length-limit-f A54:B54 (T-1434); probes/text-format-array: `TEXT(NA(),{"0","0"})` is `{#N/A,#N/A}`; `TEXT({1,#N/A},{"0",#DIV/0!})` is `{"1",#N/A}` (an error value wins over an error format element); a too-complex format (`REPT("dm",17)`) in a range of formats is `#VALUE!` in its own element (`{"1",#VALUE!}`), but `#N/A` where the value is an error; value column against format row, `{1,2,3}` against `{"0.0","00"}` and `{1;2;3}` against a 2x2 give the max shape with `#N/A` padding; text and TRUE pass through `@` and `0` formats unchanged.
Status: VERIFIED (excel-graph, probes/text-format-array, T-1458)
Implemented in: `fn_text` (functions/text/common.rs); the format argument is evaluated once (TEXT nested in a format must not be exponential).
Tasks: T-1451

## CLEAN and CODE scalar edges
Rule: CLEAN removes code points 0-31 and the C1 controls U+0080-U+009F (128-159; including the Windows-1252
holes 129, 141, 143, 144, 157; CHAR(128..159) strings were not probed by that case; CODE of them is in the next entry), and no other probed code point: DEL (127), NBSP (160) and U+200B stay. CODE of a character
outside Windows-1252 is 63 (the code of "?"), not `#VALUE!`: `CODE(UNICHAR(256))`, `CODE(UNICHAR(960))` and
`CODE(UNICHAR(128512))` are 63; `CODE("€")` is 128 and `CODE("é")` is 233. `CODE("")` is `#VALUE!`; `CODE(TRUE)`
is 84. The C1 controls are the exception to the 63: `CODE(UNICHAR(n))` for n = 128-159 is n itself, even where 1252 puts another
character in that slot (`CODE(UNICHAR(159))` is 159, not 63, though 1252's 159 is Ÿ; `CODE(UNICHAR(150))` is 150), while the real 1252
characters come back as their 1252 code (`CODE("€")` 128, `CODE(UNICHAR(8218))` 130, `CODE(UNICHAR(376))` 159, `CODE(UNICHAR(338))` 140). Also
`CODE(" ")` is 32 and `CODE("€abc")` 128. `UNICHAR(65535)` and so `CODE(UNICHAR(65535))` are `#N/A`, as is UNICHAR of a surrogate
(55296, 57343); UNICHAR of 0, -1 or 1E+10 is `#VALUE!`; `UNICHAR(65.9)` is "A"; `UNICODE(UNICHAR(65535))` is `#N/A` and
`UNICODE(UNICHAR(128512))` 128512.
Evidence: probes/text-scalar-edges A32-A39, A40-A50, A51-A62 (excel-graph, T-1280); probes/text-code-unichar-edges, text-code-c1-controls
(excel-graph, 2026-10-14, T-1306). Implemented in `char_to_win1252` (functions/text/char_code.rs).
Also verified there, all matching the engine: EXACT is case-sensitive, compares numbers by their text
(`EXACT(0.1+0.2,"0.3")` TRUE) and does not normalise Unicode (composed vs decomposed é is FALSE); REPLACE
truncates fractional counts, appends when start_num is past the end, and is `#VALUE!` for start_num 0 or a
negative count; PROPER capitalises after any non-letter (`O'Neil`, `Abc123Def`, `X.Y Z_W`) and lowers the rest
(`PROPER(TRUE)` = "True").
Status: VERIFIED (excel-graph); desktop confirmation pending (T-1081)

## FIXED and DOLLAR return at most 255 characters
Rule: FIXED and DOLLAR are `#VALUE!` when their result would be longer than 255 characters, whatever the value's
magnitude: `FIXED(1E+188,3)` (255 characters with commas) and `FIXED(1E+254,0,TRUE)` (255) print, `FIXED(1E+188,4)` and
`FIXED(1E+255,0,TRUE)` (256) are `#VALUE!`, and `FIXED(1E+200,2,TRUE)` (204) prints. The sign counts (`FIXED(-1E+188,2)`
is 255 and prints, `FIXED(-1E+188,3)` is `#VALUE!`), as do the 127 decimals (`FIXED(1E+126,127,TRUE)` prints,
`FIXED(1E+127,127,TRUE)` does not). DOLLAR has the same bound with its `$` and parentheses counted, except that a
value shown without parentheses counts one character more, as if the `_)` padding of its positive section were part of
the text: `DOLLAR(1E+188,1)` (254) prints but `DOLLAR(1E+188,2)` and `DOLLAR(1E+190,0)` (255) are `#VALUE!`, while the
negative `DOLLAR(-1E+187,1)` (255) prints and `DOLLAR(-1E+188,1)` (256) does not. So `FIXED(99^100)` (269 characters with
commas) is `#VALUE!`.
Evidence: probes/fixed-dollar-length-bound A1:B41 (oracle: excel-graph, 2026-10-22, T-2378); gen-m5/55afc805e00a1509
Calc!A392.
Status: VERIFIED (excel-graph); desktop confirmation pending
Implemented in: crates/rustsheet-engine/src/functions/text/string_format.rs (`MAX_FORMATTED_LEN`, `formatted_result`)

## DOLLAR and FIXED decimals
Rule: `decimals` is truncated toward zero, not floored: `DOLLAR(1234.5,-1.5)` is "$1,230" (decimals -1), `FIXED(1234.5,-0.5)`
is "1,235" (decimals 0), `DOLLAR(1234.5,1.9)` is "$1,234.5". A `decimals` of 128 or more (also 128.5) is `#VALUE!`;
127 is fine and prints 127 fraction digits; large negatives just give "0" (`FIXED(1,-400)`, `DOLLAR(1,-129)`).
`FIXED(1234,0,"x")` is `#VALUE!`; a numeric no_commas is a boolean (`FIXED(1234,0,2)` = "1234").
Evidence: probes/dollar-fixed-unichar-edges A1-A29, A46-A50, D69-D75 (excel-graph, T-1281).
FIXED of a value that rounds to zero drops the sign (`FIXED(-0.004,2)` "0.00", `FIXED(-0.4,0)` "0"; DOLLAR keeps
parentheses: `DOLLAR(-0.004)` "($0.00)"). Implemented (T-1257), see "FIXED drops the sign of a value that rounds to zero".
Status: VERIFIED (excel-graph); desktop confirmation pending (T-1081)

## FIXED, DOLLAR and BAHTTEXT cut to 15 significant digits first (T-2148, T-2149)
Rule: FIXED and DOLLAR round as ROUND does: the value is first cut to 15 significant digits (an exact tie at the 16th digit
goes toward zero), then rounded half away from zero at `decimals`, and the digits past the 15th print as zeros.
`DOLLAR(1.005)` is "$1.01", `FIXED(INT(2^53+1),0)` "9,007,199,254,740,990", `FIXED(999999999999998.13)`
"999,999,999,999,998.00", `DOLLAR(3000000123456798.5)` "$3,000,000,123,456,800.00", `DOLLAR(999999999999954.88)`
"$999,999,999,999,955.00" and `DOLLAR(4999999999999995)` (`ROUNDDOWN(9999999999999999,8)/2`) "$4,999,999,999,999,990.00" (the
16th-digit tie goes toward zero). BAHTTEXT, whose 15-digit rounding is above, takes the same tie toward zero:
`BAHTTEXT(999999999999997.5)` reads ...เก้าสิบเจ็ดบาทถ้วน (997).
With a negative `decimals`, a value that rounds to zero has no sign: `DOLLAR(-3.75,-1)`, `DOLLAR(-2.02005,-1)` and
`DOLLAR(-0.5,-1)` are "$0", while `DOLLAR(-0.004)` keeps "($0.00)" (above).
At `decimals` 0 and 1 the parentheses stay: `DOLLAR(-0.4,0)` is "($0)", `DOLLAR(-0.04,1)` "($0.0)", `DOLLAR(-0.4,1)` "($0.4)" and
`DOLLAR(-0.5,0)` "($1)"; only a negative `decimals` drops the sign (evidence: excel-graph, 2026-10-27, probes/t2184-dollar-neg-rounds-to-zero Sheet1!A1:A4, T-2184).
Past 2^64, and with more than about 20 decimals, the 15 digits are followed by zeros as well: `DOLLAR(1.0000001234567907E+30,-2)`
is "$1,000,000,123,456,790,000,000,000,000,000", `DOLLAR(4.33884297520652E+28,3)` "$43,388,429,752,065,200,000,000,000,000.000",
`DOLLAR(1.2244897707231034E+29,0)` "$122,448,977,072,310,000,000,000,000,000", `FIXED(90909090909108.86,24.2)`
"90909090909108.900000000000000000000000" and `FIXED(ASINH(123456790.123),100)` "19.3245489558243" and 85 zeros. The old saturated-u64 output "$18,446,744,073,709,551,615" or "...108.000018446744073709551615".
Evidence (excel-graph, 2026-10-20, T-2114): gen-m5/3b813881f835cd90 Calc!A48, gen-m5/0362d48f56108faa Calc!A43, Calc!A119,
gen-m5/2abdc1524e5aa685 Calc!A221, gen-m5/461d1c8dbb044769 Calc!A249, gen-m5/50d8a085335c05e6 Calc!A158, gen-m5/4d099bea7898ebf1 Calc!A66,
gen-m5/399071ff6782ab16 Calc!A316, gen-m5/5704666097cbc5b2 Calc!A274, gen-m5/ae9ed30812c4d62c Calc!A183 (T-2148); gen-m5/6b831f5e2a20bc34 Calc!A232,
gen-m5/e0087a1fb9a36188 Calc!A291, gen-m5/fc81c2f704bef138 Calc!A366, gen-m5/399071ff6782ab16 Calc!A122, gen-m5/b0ecb7ca78460b77 Calc!A128 (T-2149).
Status: VERIFIED (excel-graph); implemented for 15-digit cut, tie toward zero and the DOLLAR sign rule (T-2148, shared `significant_15` in FIXED, DOLLAR and BAHTTEXT); values past 2^64 and many decimals fall out of the same code, T-2149 only has to confirm them.

## NUMBERVALUE reads text as VALUE does, with its own separators (T-2151, T-2152)
Rule: after the separators are applied, NUMBERVALUE accepts what VALUE accepts: currency `"$1,000.50"` = 1000.5,
parentheses `"(100)"` = -100, dates `"2024-02-29"` = 45351 and `"1900-02-29"` = 60, and times past 24 hours `"25:00"` =
1.0416666666666667 (`"12:30"` = 0.5208333333333334). Only the first character of each separator counts: decimal
separator `"FALSE"` is "F", `"MDCVIII"` (`ROMAN(1608,4)`) is "M", `"0.2"` is "0". Every occurrence of the group separator's
first character is removed from the text before parsing, digits included: `NUMBERVALUE("25:00",UNICHAR(12288),"5")` is "2:00"
= 0.08333333333333333 and `NUMBERVALUE("25:00","-0.01","2")` is "5:00" = 0.20833333333333334; with the group separator "0",
"25:00" still reads 1.0416666666666667 ("25:", an hour and a trailing colon, which VALUE reads too:
dates.md "Time text"; `NUMBERVALUE("25:")`, `NUMBERVALUE("12:30:")` and `NUMBERVALUE("1:","F","0")` agree,
probes/t2352-time-text-trailing-colon, T-2352). A decimal separator that is a digit of the number breaks it:
`NUMBERVALUE(0.3,"0.2")` and `NUMBERVALUE(0.1,"0.3x","ABCFALSE")` are `#VALUE!` (ours 0.3 and 0.1). Text that is empty, or
empty once the group separator is removed, is 0: a blank cell, `""` from MID, RIGHT, T or REPT, and
`NUMBERVALUE("A","0","ABC?b#N/A")` (ours `#VALUE!`). An empty group separator (a blank cell) is `#VALUE!`:
`NUMBERVALUE(" 42 ","#N/A",<blank>)` (ours 42).
Evidence (excel-graph, 2026-10-20, T-2114): gen-m5/033e65b92653f425 Calc!A161, gen-m5/ae9ed30812c4d62c Calc!A285,
gen-m5/109fc90dceb50ba9 Calc!A144, gen-m5/1962de140dae0111 Calc!A80, gen-m5/6b831f5e2a20bc34 Calc!A227, gen-m5/4d099bea7898ebf1 Calc!A261,
gen-m5/38b1b8e51dbad326 Calc!A340, gen-m5/34f69d858a39f497 Calc!A227, gen-m5/b237245054b4fedb Calc!A128, gen-m5/eb942f4a59b1c004 Calc!A115 (T-2151);
gen-m5/5704666097cbc5b2 Calc!A261, Calc!A392, gen-m5/bbbd316c346ce645 Calc!A76, gen-m5/b237245054b4fedb Calc!A390, gen-m5/a6fec430ac350c1a Calc!A32,
gen-m5/ecd1b828aebad1e8 Calc!A60 (T-2152); the array lift is in arrays.md (T-2160).
Every space is removed before the text is read, so `NUMBERVALUE("12:30 PM")` is `#VALUE!` ("12:30PM") where
`VALUE("12:30 PM")` is 0.5208333333333334 (gen-m5/bbbd316c346ce645 A207, fc81c2f704bef138 A194, 6a8bb7cd6d7769ad A357
against gen-m5/49bfbf67a1409563 A217, 901e91c09b347914 A13; Microsoft's NUMBERVALUE page says spaces are ignored "even in the
middle"; that "12:30PM" is the reason, not a refusal of AM/PM, is the simplest reading and unprobed).
An omitted group separator defaults to "," unless the decimal separator starts with ",", when it defaults to ".":
`NUMBERVALUE("1.5",",")` = 15, `("1.000,5",",")` = 1000.5, `("1.5.2",",")` = 152, while `("1,5",";")` = 15, `("1,5","x")` = 15
and `("1.5",".")` = 1.5 (excel-graph, 2026-10-28, probes/numbervalue-default-group A1-A14, T-2441). A multi-character decimal separator starting with "," is unprobed (only the first character is read elsewhere).
A decimal separator equal to the group separator is `#VALUE!` whatever the text: `NUMBERVALUE("1.5",",",",")` and
`NUMBERVALUE("1.5",".",".")` (excel-graph, 2026-10-20, T-2198, probes/t2198-m5-lift-argument-kinds A38, B38).
A separator over 255 characters is `#VALUE!`, though only its first character counts: `NUMBERVALUE(0.2,REPT("A",255),"b")` is 0.2
and `NUMBERVALUE(0.2,REPT("A",256),"b")`, `NUMBERVALUE("2",REPT("A",256),"b")` and `NUMBERVALUE(0.2,"b",REPT("a",256))` are
`#VALUE!` (and `NUMBERVALUE(0.2,PROPER(REPT("ab",300)),"a*2")`, gen-m5/9a7a94353845ea12 Calc!A162, while 200 characters give
0.2). Separators differing only in case do not clash: `NUMBERVALUE(0.2,"A","a")` is 0.2, `NUMBERVALUE("1A5","A","a")` and
`NUMBERVALUE("1x5","x","X")` are 1.5, but `NUMBERVALUE("1,5","A","a")` is `#VALUE!` (the "," is no separator) and the decimal
separator matches case (`NUMBERVALUE("1a5","A","b")` is `#VALUE!`). A letter decimal separator leaves "0.2" readable
(`NUMBERVALUE("0.2","A","b")` and `NUMBERVALUE("0.2","A")` are 0.2). With a "," decimal separator and no group separator,
`NUMBERVALUE("1.5",",")` is 15: the default group separator is then "." (rule above, T-2441). Separators whose first characters agree clash even when
the rest differs: `NUMBERVALUE("1A5","A","AB")` is `#VALUE!` (A20).
Evidence: probes/numbervalue-separator-edges A1:A20 (excel-graph, 2026-10-22, T-2379). Whether the 255 counts characters or
UTF-16 units is unprobed (the engine counts as for the text, `number_text_too_long`).
Unprobed: date text with spaces (`"29 Feb 2024"`, `"Feb 29, 2024"`; ours reads it without its spaces), a group separator after the decimal separator, `%` with custom separators, whitespace-only text and `"%"` (ours: spaces-only is 0, `"%"` is `#VALUE!`), a separator outside the BMP (ours: first code point, Excel counts UTF-16 units).
Status: VERIFIED (excel-graph); implemented: first-character separators and empty text (T-2152), VALUE-like parsing
after the separators with every space removed (T-2151, `numbervalue_of` in text/string_format.rs), the trailing
colon after a bare hour (T-2352, in the shared clock parser).

## REPT, CODE, UNICODE and T refuse a text argument over 255 UTF-16 units (T-2155; T of a cell reference gives "", T-2305)
Rule: REPT's text, CODE's and UNICODE's argument are `#VALUE!` when the text is longer than 255 characters, whatever the
count: `REPT(REPT("ab",300),0)`, `REPT(REPT("ab",300),3)`, `REPT(REPT("ab",300),5)`, `CODE(REPT("ab",300))` and
`UNICODE(REPT("ab",300))` are `#VALUE!` (ours: the repeated text, "", 97, 97), while `REPT(REPT("x",255),3)` works
(gen-m5/033e65b92653f425 Calc!A55) and LENB, PROPER, UPPER, EXACT, FIND and SEARCH take the same 600-character text.
Evidence (excel-graph, 2026-10-20): gen-m5/6a8bb7cd6d7769ad Calc!A191, gen-m5/8a387c26d182e73c Calc!A228, gen-m5/901e91c09b347914 Calc!A45,
gen-m5/4d099bea7898ebf1 Calc!A359, gen-m5/ecd1b828aebad1e8 Calc!A1, gen-m5/b237245054b4fedb Calc!A394.
Cut (probes/t2199-text-length-and-combin-cuts, excel-graph, T-2199): the limit is 255 UTF-16 units. Text of 254 and 255 "x" works
in all three (A1, A2, B1-B2, C1-C2: REPT doubles it, CODE and UNICODE give 120) and 256, 257, 300 and 32767 are `#VALUE!`
(A3-A6, B3-B6, C3-C6). It counts UTF-16 units, not code points: 254 "x" plus `UNICHAR(128512)` is 255 code points but 256
units (`LEN` 256) and is `#VALUE!` in all three (row 7), as is 255 "x" plus it (row 8). T shares the cut:
`T(REPT("x",256))` is `#VALUE!` (A12). CLEAN, TRIM, LOWER and LEFT(..,1) take the same 256-character text
(A9-A11, A13, `LEN` 256 for the first three). `VALUE(REPT("1",256))` is `#VALUE!` (A14), which the probe cannot tell
from a numeral-length limit. From a cell, as an array element and with an error count (T-2305; probes/text-255-units-from-cell, -array-element,
-rept-error-count, excel-graph 2026-10-24): a cell holding 256 "x" (typed, D1, or a formula result, D3) is `#VALUE!` in
`REPT(D1,2)`, `CODE` and `UNICODE`, but `T(D1)` and `T(D3)` are empty text "", not `#VALUE!`; 255 from a cell works in all
four. As an array element (`REPT("x",256)&{""}`) REPT, CODE, UNICODE and T are all `#VALUE!`;
255 works. excel-graph refused to open a workbook whose array constants held 256-character literals (cause not isolated), so the probe builds the long text with REPT. REPT with a
long text and an error count gives the count's error: `REPT(REPT("x",256),#N/A)` is `#N/A`, `(…,#DIV/0!)` is `#DIV/0!`; `(…,-1)` is `#VALUE!`; `REPT(#N/A,REPT("x",256))` is `#N/A`. T of such a cell is "" however the cell is reached (T-3107; probes/text-255-units-reference-shapes, excel-graph 2026-10-29): `T(D1:D2)` (top-left cell), a defined name, `INDIRECT`, `OFFSET`, `INDEX`, `IF`, `CHOOSE`, `LET`, a spill reference (`T(F1#)`), a sheet-qualified reference (same sheet or another), `T((D1))`, a single-cell structured reference, and a legacy formula's `T(D1)`, `T(D1:D2)` and `T(name)` are all "". A computed value stays `#VALUE!`, and so does the unary plus: `T(+D1)` is `#VALUE!`. Implemented by `reads_cell_text` in t_function.rs.
Status: VERIFIED (excel-graph); implemented (T-2155: `reject_over_short_limit` in functions/text/common.rs); the cell, array and error-count rules are implemented (T-2627, T-3107: `T` of any reference reads "", an array element is `#VALUE!`, REPT reads its count's error before the text limit).

## UNICHAR code point bounds
Rule: UNICHAR truncates toward zero and lifts over arrays (`UNICHAR({65,66})` spills "A","B"). Below 1 (0, -1, 0.5) or
above 1114111 (U+10FFFF) is `#VALUE!`. A surrogate (55296-57343) or a Unicode noncharacter is `#N/A`: U+FDD0-U+FDEF
(64976-65007) and the last two code points of every plane (65534, 65535, 131070, 131071, 196607, ..., 1114110,
1114111). The neighbours are fine: 55295, 57344, 64975, 65008, 65533, 65536. A code point above 65535 is a
surrogate pair (`LEN(UNICHAR(65536))` is 2). A fractional value is truncated first (`UNICHAR(55296.5)` is `#N/A`).
Evidence: probes/dollar-fixed-unichar-edges A30-A45, A51-A53, D54-D68 (excel-graph, T-1281).
Status: VERIFIED (excel-graph); desktop confirmation pending (T-1081)

## UNICODE lifts over arrays and ranges
Rule: UNICODE's argument lifts element by element over an array or a multi-cell range and spills, as CODE's does.
`UNICODE(INDIRECT("Data!A2:A4",VSTACK(Data!C6:E7)))` lifts over INDIRECT's 2x3 result, and in a cell whose spill area
is occupied it is `#SPILL!`. The element values (each element's scalar UNICODE, an empty element `#VALUE!`) are
unprobed: the evidence fixes only the shape.
Evidence: gen-m9/675763979c1defb6 Calc!A25 (excel-graph).
Status: VERIFIED (excel-graph) for the spill; element values UNVERIFIED
Implemented in: `fn_unicode`, `unicode_element` (functions/text/common.rs)
Tasks: T-2090, T-2104

## UNICHAR argument shapes
Rule: UNICHAR's argument is coerced like any number argument: TRUE is 1 (`UNICHAR(TRUE)` and `UNICHAR(A4)` with A4 = TRUE
give U+0001), numeric text converts (`UNICHAR(A5)` with A5 = "67" is "C"), an empty cell is 0 so `#VALUE!`, and
non-numeric text and "" are `#VALUE!`. An `#N/A` argument is returned as it came (`UNICHAR(#N/A)`; other error types and errors inside a range are unprobed). A
multi-cell range lifts element by element and spills: `UNICHAR(A1:A2)` is "A","B"; `UNICHAR(A1:A3)` with A3 blank is
"A","B",`#VALUE!`; an all-empty range (`UNICHAR(A7:A8)`) is `#VALUE!` in every row; `UNICHAR(A4:A5)` (TRUE, "67") is
U+0001, "C"; the lifted result broadcasts through `&` (`UNICHAR(A1:A2)&"!"` is "A!","B!"); `UNICHAR(A5:A6)` ("67", "x") is "C", `#VALUE!`. The engine already matches all of these.
Evidence: probes/unichar-arg-shapes D1-D7, F1-F2, H1-H3, J1-J2, L1-L2, N1-N2, P1-P2, R1-R2 (excel-graph, T-1464).
Status: VERIFIED (excel-graph); desktop confirmation pending (T-1081)

## TEXTJOIN over arrays, and an array or range delimiter (T-1482)
Rule: TEXTJOIN joins the elements of an array literal or function result row by row, like a range: `TEXTJOIN(",",FALSE,{1,2;3,4})` is `1,2,3,4`, `SEQUENCE(2,2)` and `SORT(B1:B3)` likewise. The first error element (row-major) is the result (`{#DIV/0!,#N/A}` is `#DIV/0!`, `{1,2},#N/A` is `#N/A`). An empty element (a blank cell that went through SORT, UNIQUE or TRANSPOSE) is joined as empty with ignore_empty FALSE (`SORT(B1:B3)` with B2 blank is `apple,pear,`) and skipped with TRUE. An empty *string* is skipped with TRUE wherever it comes from: an array element (`{"a","","c"}` is `a,c`), a direct argument (`TEXTJOIN(",",TRUE,"a","","c")`) and a range cell holding `=""` (`B1:B3` with B2 = `=""` is `a,c`; with FALSE it is `a,,c`). An error element past the 32,767 limit is dropped like a range cell's error, so the result is `#CALC!` (`TEXTJOIN(",",TRUE,REPT("a",32767),{"b",#N/A})`) unless a later error *argument* wins (`...,NA()` is `#N/A`); with a 32,766 prefix the lone `{#N/A}` is `#N/A`.
The delimiter may be an array or a range, and then it cycles between the joined values: `TEXTJOIN({",",";"},FALSE,{1,2,3,4,5})` is `1,2;3,4;5` and `TEXTJOIN(B1:B2,FALSE,1,2,3)` with B1 = "-", B2 = "+" is `1-2+3`; a delimiter `{""}` joins with nothing. The delimiter before the n-th value (n >= 1) is element (n-1) mod len of the delimiter in row-major order, so `{",";"-"}` and `{",","-"}` cycle alike (probes/textjoin-delimiter-array A1-A6, excel-graph; T-1510). An error element anywhere in the delimiter array or range is returned as that error, even when the cycle never reaches it (`TEXTJOIN({"x",#N/A},FALSE,1,2)` and `TEXTJOIN({#N/A,"x"},FALSE,1)` are `#N/A`, as are a range holding `=NA()` and a scalar `NA()` delimiter with one value; `{#DIV/0!}` gives `#DIV/0!`). Number and boolean elements are text: `TEXTJOIN({1,2},FALSE,"a","b","c")` is `a1b2c`, `{TRUE,FALSE}` joins the words TRUE and FALSE (`a` + "TRUE" + `b` + "FALSE" + `c`), a scalar `1.5` is `a1.5b`. An empty cell element is an empty delimiter (`A1:A2` with A1 empty, A2 "-" gives `ab-c`, and so does `A2:A1`). Cycled delimiters of different sizes count by their own size toward the 32,767 limit: `{"ab","c"}` over `REPT("a",32762),"b","c"` is exactly 32,767 and fine, one more `a` is `#CALC!`; with a fourth value 32,759 fits and 32,760 is `#CALC!`; only the delimiters actually used count (`{"abc","d"}` over a prefix and one value: 32,763 fits, 32,764 is `#CALC!`). With ignore_empty TRUE the cycle advances only for joined (non-empty) values, so empty values in between change nothing (C22 gives the same string as C16, ending `abbcc`; skipping the cycle on empties would end `acbabc`).
(probes/textjoin-delimiter-elements.case.json C1-C23 plus the A4 input, excel-graph, 2026-10-14, T-1548; the engine matches all 24 cells.)
Evidence: probes cases `textjoin-array-arguments` (A1-A28, A30-A35), `textjoin-array-limit-empty-strings` (A1-A10) and `textjoin-delimiter-array` (A1-A6). Oracle: excel-graph, 2026-10-14.
Status: VERIFIED (excel-graph); the array argument (T-1482) and the cycling delimiter (T-1510) are implemented.
Tasks: T-1482, T-1510

## ASC and JIS in en-US: ASC is the identity (T-1626)
Rule: in en-US `ASC` changes nothing: full-width letters and digits (`ASC("ＡＢＣ")` = "ＡＢＣ", `ASC("１２３")`), the ideographic space, `！ ＂ ＇ ￥ ゛`, voiced and long-sound katakana (`ガ パ ー`), `、 。`, half-width katakana, surrogate pairs, numbers (`ASC(A7)` with A7 = 7 is "7"), TRUE ("TRUE") and an empty string ("") come back as given (a truly blank cell was not probed); an error argument is returned. `JIS` is `#NAME?` for every argument probed (`ERROR.TYPE(JIS(..))` = 5 for 28 inputs).
The byte functions (SEARCHB, FINDB, REPLACEB, RIGHTB, LENB) are the character functions for all of these inputs; see the byte-function rule at the top of this file.
Evidence: probes/t1626-asc, t1626-jis-0..3, t1626-searchb, -findb, -replaceb, -rightb, -lenb (excel-graph, 2026-10-16). PHONETIC of every reference in probes/t1626-phonetic is `#N/A` as before.
Ours: matches (ASC is the identity since T-1863).
Status: VERIFIED (excel-graph); desktop confirmation pending (T-1081)
Implemented in: `Model::fn_asc` (functions/text/char_code.rs). Tasks: T-1863

## REGEXTEST and REGEXREPLACE edge cases (T-1626)
Rule: `case_sensitivity` 1 is case-insensitive and 0 (or omitted) case-sensitive for both functions (`REGEXTEST("Hello World 123","hello",1)` TRUE, `,0` FALSE; REGEXREPLACE's fifth argument likewise, `"O"` with 0,1 replaces every o). REGEXREPLACE occurrence: 0 or omitted replaces all, n > 0 the n-th match, n < 0 counts from the end (-1 the last), an out-of-range occurrence (3 of 2 matches, 9, -9) returns the text unchanged; 1.9 truncates to 1; "x" is `#VALUE!`. The replacement reads `$0`, `$1`, `$$` (a literal `$`), `${name}` and also `\1`; a backreference in the pattern (`(\w)\1`) works. An empty pattern or `x*` matches between every character and at both ends (`-H-e-l-l-o-...-3-`; an empty text gives "-"); `^` and `$` anchor the text. Lazy empty matches are not skipped: `a*?` on "aaa" with "b" is "bbbbbbb" (PCRE2-style global replace: after an empty match a non-empty match is tried again at the same position, so each "a" is consumed too, unlike a Rust-regex iterator's "bababab"). `\p{L}` and `.` match each character of BMP text ("日本語" is 3); `.` matches a surrogate pair as one character
(`REGEXREPLACE("a😀",".","-")` is `--`, probes/regex-iteration-surrogates A9). Numbers and booleans are read as text; an error argument is returned. A range argument lifts and spills (`#SPILL!` when blocked). Invalid patterns and a missing group (`$3`) give an error type the oracle does not decode yet; those cells stay pending; UNVERIFIED. REGEXTEST case_sensitivity 2 and -1 (C23, C24, undecoded here) are `#VALUE!` (probes/regex-optional-args A3, A4; next entry but two).
Evidence: probes/t1626-regextest, t1626-regexreplace (excel-graph, 2026-10-16).
Ours: the optional arguments since T-1360 (next entry but two, "REGEX* optional arguments"); `\1` in the replacement matches since T-1688; the pattern
backreference (REGEXTEST C29) is T-1900 and the lazy empty matches (REGEXREPLACE C34) match since T-1899.
Status: VERIFIED (excel-graph); desktop confirmation pending (T-1081)
Tasks: T-1360, T-1688, T-1272 (range lift)

## REGEX* optional arguments: case_sensitivity, return_mode, occurrence (T-1360)
Rule (coercion, all three): an omitted, empty (`REGEXTEST("ABC","abc",)`) or blank-cell argument is 0; a number is
truncated toward zero (0.5 and 0.9 are 0, 1.5 is 1; as occurrence -0.5 is 0, -1.5 is -1); TRUE is 1, FALSE 0; number
text is its number ("1" probed for case_sensitivity and return_mode, "2" and "-1" for occurrence); other text, "" included, is `#VALUE!`; an error argument is returned. Arguments are read left to
right, so an error in the text beats one in a later argument (`REGEXTEST(NA(),"a",1/0)` is `#N/A`).
Rule (case_sensitivity, REGEXTEST's third, REGEXEXTRACT's fourth, REGEXREPLACE's fifth): after truncation 0 is
case-sensitive and 1 case-insensitive; anything else (2, -1, 2^53) is `#VALUE!`, also for an array element (`{0;1;2}`
gives FALSE, TRUE, `#VALUE!`). Case-insensitive matching uses Unicode case folding (`ſ` matches `s`, the Kelvin sign
`k`, `ÄÖ` `äö`) and a character class (`[a-c]+$` matches `ABC`).
Rule (REGEXEXTRACT return_mode): 0 is the first match, WHOLE even when the pattern has capture groups
(`REGEXEXTRACT("a1b2c3","([a-z])([0-9])")` is "a1"); 1 is every match, whole, as a 1xN ROW ("a1","b2","c3"; one match
is 1x1); 2 is the capture groups of the first match as a 1xG ROW ("a","1"; named groups alike), a group that took
no part in the match being `#N/A` (`REGEXEXTRACT("b","(a)?(b)",2)` is `#N/A`,"b"), and a pattern without groups is
`#VALUE!` whether or not it matches (`REGEXEXTRACT("abc","z",2)`). Any other value after truncation (3, -1) is `#VALUE!`,
checked before matching (`REGEXEXTRACT("abc","z",5)` is `#VALUE!`, not `#N/A`). No match is `#N/A` in every mode. In a
lifted call each element keeps the first value of its row (mode 1: the first match; mode 2: the first group, `#N/A` if
it did not take part).
Rule (REGEXREPLACE occurrence), extending the T-1626 entry above: truncated toward zero (2.9 is 2, -1.5 is -1, 0.1 and
-0.5 replace all); matches are counted as the global replace finds them, empty ones included (`REGEXREPLACE("abc","",
"-",4)` is `abc-`, 5 leaves `abc`); |n| past the match count, 1E+300 and -1E+300 included, leaves the text unchanged. An
invalid replacement is `#VALUE!` only when the match to replace exists (`REGEXREPLACE("ab","a","$9",2)` is `ab`, with 1
`#VALUE!`), and a group that took no part is empty (`REGEXREPLACE("ab","(a)|(b)","[$2]",1)` is `[]b`).
Evidence: probes/regex-optional-args A1-A148 (excel-graph, 2026-10-19); A19 (TEXTJOIN over the `{0;1;2}` result, a
rich error) stays undecoded, its per-element ERROR.TYPE is A145.
Ours: matches since T-1360, except where the matcher differs (T-1898-T-1900: lookaround, backreferences, PCRE2
empty-match iteration). Implemented in `Model::regex_option`, `Occurrence`, `ReturnMode`
(functions/text/regex_functions.rs); case folding is `regex-automata`'s `case_insensitive` syntax flag.
Status: VERIFIED (excel-graph)
Tasks: T-1360

## REGEXREPLACE replacement syntax (T-1688, T-1897)
Rule: the replacement is read as PCRE2's extended substitution. `$n` and `\n` are group n, where n is every digit
that follows (`$1a` is group 1 then "a", `[\10]` is group 10); `${n}`, `${name}` and `$name` work too (`${x}$x` is
`aab`); `$$` and `\$` are `$`, `\\` is `\`; `\n` LF, `\t` TAB, `\r` CR, `\a` 7, `\b` 8 (not a word boundary), `\e` 27,
`\f` 12, `\v` 11. A backslash before any other non-alphanumeric character is that character (`[\-]`, `[\é]`), and a group
that took no part in the match is empty (`REGEXREPLACE("b","(a)?b","[$1]")` is `[]`).
Characters: `\xh` and `\xhh` (one or two hex digits, `\x41g` is `Ag`); `\x{h...}` and `\o{o...}` (spaces and tabs
inside the braces are ignored, `\x{ 41 }` is `A`; any number of leading zeros; a value past U+10FFFF or a surrogate is
an error); `\0` plus one or two octal digits (`\012` LF, `\0123` LF then "3", `\00` NUL); `\cX` is the upper-cased ASCII
X XOR 64 (`\ca` is 1, `\c@` 0, `\c?` 127, `\c1` "q", `\c]` 29; a non-ASCII X is an error). `\` and digits 1-9: all
the digits are a decimal number, a group if it is below 10 or no more than the pattern's groups (a missing group is an
error), else, if it starts with three octal digits, those as a character up to `\377` (one or two such digits are an error) (`\101` is `A`, `\1010` is `A0`; with 12
groups `\10` is group 10 but `\20` and `\77` are errors; `\400` an error; `\18`, `\181` errors).
Case forcing: `\U` and `\L` set a mode for the rest of this match's replacement (literals, groups and the characters
the escapes above make alike: `\L\x41\U\x61` is `aA`); `\u` upper-cases the next character only and keeps the mode
(`\L\uxyz` is `Xyz`, `\u\L$1` too); `\l` is unlike PCRE2's: it lower-cases the next character and sets the `\L` mode, so
`\Uab\lcd` is `ABcd` and `\U\lxyz` is `xyz` (and `\u\lxyz` is `xyz`, `\l\uxyz` is `Xyz`, `\l\Uabc` is `aBC`); `\E`
ends the mode and a pending `\u`; an empty group or text keeps a pending `\u` for the text after it (`\u$2b` with
`$2` empty is `B`); the state starts afresh with each match (`REGEXREPLACE("abab","(a)(b)","$2\U$1")` is `bAbA`); only
the replacement is case-forced, never the subject text around the match. Case mapping is the one-to-one mapping of UPPER and
LOWER (`\Uß` is `ß`). Final sigma (T-1934, VERIFIED, probes/regexreplace-final-sigma, excel-graph 2026-10-19): a `\L` run is lowered whole, so a capital sigma sees the text after it in the next piece (`\L$1x` with `$1` = ΑΣ is `ασx`, but `\L$1` alone is `ας`; A2, A4, A8); a `\E` ends the run (`\L$1\E$2` with `$1`, `$2` = Α, Σ keeps `Σ`: `αΣ`, A10); a `\l` head character is lowered alone (`\lΣa` is `ςa`, A1). Whether a `\u`, `\l` or `\U` inside a `\L` run ends the run for the sigma context is UNVERIFIED (ours: it does, as `\E` does). `\Q...\E` quotes up to
the first `\E` (or the end): `\Q$1\E` is `$1`, `\Qa\\Eb` is `a\b`; quoted text is case-forced, and `\E` ending a quote
does not end the mode (`\Uab\Qcd\Eef` is `ABCDEF`).
`#VALUE!` (ERROR.TYPE 3, so the undecoded rich errors are `#VALUE!`): a group the pattern does not have (`$2`, `\2`,
`$10`, `${y}`, `$y` with one unnamed group), a `$` that starts no reference (`$` alone), a trailing `\`, `\0` with no octal
digit after it (`\0`, `\08`, `\0a`), `\x` with no hex digits or a bad `{...}`, `\o` without a valid `{...}`, `\c` at the
end, and every other letter escape (`\q`, `\d`, `\w`, `\s`, `\h`, `\R`, `\g`, `\k`, `\p`, `\N`, `\z`...), but only
when something matches: with no match the text comes back unchanged (`REGEXREPLACE("zz","(a)","$2")`, `"\q"` and
`"$"` are `zz`).
Evidence: probes/regexreplace-extended-escapes (244 cells, excel-graph, 2026-10-18, T-1897); probes/regex-pcre2-features
B1-B30, E1-E7 (excel-graph, 2026-10-18); t1626-regexreplace C14-C19; T-1817's probes/regexreplace-limit-and-syntax A12-A16.
Ours: matches. Implemented in `Replacement::new`, `CaseState` (functions/text/regex_replacement.rs); the size bound
`Replacement::result_len` allows case mapping to lengthen text by half.
Status: VERIFIED (excel-graph)
Tasks: T-1688, T-1897

## REGEX* character classes are Unicode; `$` and `.` around line breaks (T-1829)
Rule: Excel's PCRE2 runs with Unicode properties (UCP): `\w` matches any letter or digit, `é`, `日` and the Kelvin sign
included (`REGEXEXTRACT("naïve","\w+")` is "naïve"; `\W` does not match `é`; `[\w]`, `[^\W]` alike); `\b` sees
`é` as a word character (`REGEXREPLACE("café foo","\bfoo\b","x")` is "café x", `"caféfoo"` is unchanged, `a\b` does
not match in "aé"); `\d` matches the Arabic-Indic three and the full-width digit; `\s` matches NBSP, U+2003, U+3000,
U+0085 and VT but not the zero-width space U+200B; the POSIX classes `[[:alpha:]]` and `[[:word:]]` match `é`;
`(*UCP)`, `\X`, `\h` and `\R` are accepted. `$` matches at the end and before a final LF only (`"a"&CHAR(10)` matches
`a$`; without `(?m)`, an LF, CR or U+2028 inside the text does not end a line for `$` or `^`), and `.` does not match LF. A final CR or
CRLF counts as a final LF does (next entry, T-2364).
Evidence: probes/regex-ascii-classes A1-A42 (excel-graph, 2026-10-19).
Ours: matches, `(*UCP)`, `\h` and `$` before a final LF included since T-1973 (our PCRE2 parser, functions/text/pcre/).
`\X` and `\R` (A27, A42) since T-2287 (next entry).
`\Z` follows `$`'s final-LF rule, as SPEC-0013 reads PCRE2, and Excel's does too (probes/t2364-regex-newline-convention
B18-B20, T-2364); before T-1973 it was `#VALUE!`.
Other syntax that T-1973 changed without a probe (UNVERIFIED, T-1972 probes them):
- Now accepted as PCRE2 reads it: `(?n)`, `(?-)`, `(?'name'..)`, `\e`, `\cX`, `[\b]`, `\p{^Lu}`, a `{` that starts
  no quantifier (a literal), and spaces inside a bracket under `(?x)` (literal).
- Changed meaning: `\<` is a literal `<`, not a word start.
- Unchanged, but unlike stock PCRE2: `(?m)^` also matches after a final LF (`REGEXREPLACE("a"&CHAR(10),"(?m)^","x")` is
  "xa\nx" in ours).
- Now `#VALUE!` (fail closed): `\v` (was VT), `^*`, and spaces inside quantifier braces (`a{1, 2}`, a quantifier
  before). Also `--`, `~~` and `||` inside a bracket, which were set operations; Excel's nested brackets suggest a
  UTS #18 dialect, where they may be set operations too.
`$` and `\Z` on a text that holds another LF before its final one also match before the final LF, as Excel's do
(`REGEXTEST("b"&CHAR(10)&"a"&CHAR(10),"a$")` is TRUE, `REGEXREPLACE("a"&CHAR(10)&CHAR(10),"$","x")` puts an x before
and after the final LF; probes/t2355-regex-dollar-inner-lf A1-A10, excel-graph, 2026-10-22), since T-2355: the pattern
then runs on the backtracker's memoized mode. Before, they matched only at the very end. A text with an LF inside
but none at the end stays on the automata, `$` meaning its end (A7, A8).
Status: VERIFIED (excel-graph)
Tasks: T-1829 (probe), T-1900, T-1903, T-2355 (probe t2355-regex-dollar-inner-lf), T-1973

## REGEX* newlines are CR, LF and CRLF (PCRE2's ANYCRLF convention)
Rule: Excel's PCRE2 counts CR, LF and the pair CRLF as newlines, and nothing else (not U+2028, NEL, VT or FF). `$`
without `(?m)` and `\Z` match at the end and before a final LF, CR or CRLF, so also between the CR and LF of a final
CRLF: `REGEXTEST("a"&CHAR(13),"a$")`, `REGEXTEST("a"&CHAR(13)&CHAR(10),"a$")` and `"a\r$"` on it are TRUE, and
`REGEXREPLACE("a"&CHAR(13)&CHAR(10),"$","x")` puts an x before the CR, before the LF and at the end; two CRs or LF CR
at the end are not one newline (`"a"&CHAR(13)&CHAR(13)` and `"a"&CHAR(10)&CHAR(13)` do not match `a$`); `\z` stays the
very end. `(?m)$` matches at the end and before every CR and every LF, also between a CR and its LF (`"a"&CHAR(13)&
CHAR(10)&"b"` with `(?m)$` -> x gives `ax<CR>x<LF>bx`). `(?m)^` matches after an LF and after a CR that no LF follows,
never between a CR and its LF (`xa<CR><LF>xb`), and not after U+2028. `.` matches neither CR nor LF (`a.b` on
`"a"&CHAR(13)&"b"` and `a..b` on a CRLF are FALSE) but matches U+2028.
Evidence: probes/t2364-regex-newline-convention B1-B25 and probes/t2364-regex-dollar-final-cr A1-A5 (excel-graph,
2026-10-22 and 2026-10-25).
Ours: matches since T-2364. The backtracker's `holds` and `.` read these rules; the automata use regex-syntax's CRLF
forms (`Dot::AnyCharExceptCRLF`, `Look::StartCRLF`, `Look::EndCRLF`), and a regular pattern holding `$`, `\Z` or
`(?m)$` over a text with a CR runs on the backtracker's memoized mode (`FinalLf::Cr`), since no `Look` gives those
there. Unprobed (T-1972): `(?m)^` at the very end after a final LF (ours matches there, as before T-2364) or a final CR
(ours matches there since T-2364, through `Look::StartCRLF`; stock PCRE2 matches after neither); and PCRE2's ANYCRLF
bump-along, which after a failed attempt at a CR skips the position between it and its LF unless the pattern holds a
literal CR or LF (`REGEXTEST("a"&CHAR(13)&CHAR(10)&"b","\s\w")` would then be FALSE; ours is TRUE).
Status: VERIFIED (excel-graph)
Tasks: T-2364

## `\R` and `\X` are atomic; `\X` is PCRE2's grapheme cluster (T-2287)
Rule: `\R` is one line break, CR LF or any one of LF, VT, FF, CR, NEL, U+2028 and U+2029, and it never gives back the
LF of a CR LF. `REGEXEXTRACT(CHAR(13)&CHAR(10),"\R",1)` is one match of length 2. `CHAR(10)&CHAR(13)` is two matches,
and `CHAR(13)&CHAR(13)&CHAR(10)` is 1 then 2. `^\R$` matches CR LF, but `^\R\n$`, `^\R\R$` and `^\R{2}$` do not. `[\R]` and
`[\X]` are `#VALUE!`.
`\X` is one extended grapheme cluster, also atomic: `^\X$` matches `"e"&UNICHAR(769)`, while `^\X\X$` and `^\X.$` do
not, and `^.\X$` does. The clusters follow UAX #29 without GB9c, so a Devanagari conjunct (U+0915 U+094D U+0937) is
two clusters, 2 then 1. That points to PCRE2 before 10.43. These all join as UAX #29 says: CR LF; Hangul L V T, LV V
and L LV; Extend and SpacingMark after anything but a control; Prepend before a letter, a Prepend or an indicator; an
emoji ZWJ sequence (8 units for the family, 7 with a skin tone); a flag pair (indicators pair up: three are 4 then 2);
and `©` ZWJ `©`. Two of PCRE2's join-table quirks break where UAX #29 joins. A regional indicator joins nothing but a
second indicator, so `UNICHAR(127482)&UNICHAR(769)` is 2 then 1, and a ZWJ after an indicator starts a new cluster.
A Prepend does not join an emoji: `UNICHAR(1536)&UNICHAR(128512)` is 1 then 2. A ZWJ joins an emoji only after an emoji
and its Extend marks. `"a"` ZWJ emoji is 2 then 2, emoji SpacingMark ZWJ emoji is 4 then 2, emoji ZWJ ZWJ emoji is
4 then 2, and two emoji with nothing between them are two clusters. `\X` matches nothing in `""`. `\X$` on
`"ab"&CHAR(13)&CHAR(10)` matches "b" (probes/t2364-regex-dollar-final-cr A1-A2; T-2364: `$` before a final CR LF).
Evidence: probes/t2287-regex-r-x-spans A1-A49, A51-A53, A58-A64, B1-B24 (excel-graph, 2026-10-22).
Ours: matches. `\R` is parsed into that atomic group. `\X` is an instruction of the backtracker that charges each
character it takes (`pcre/grapheme.rs`, with break values from regex-syntax's Unicode tables).
Unprobed:
- Characters whose break value changed between Unicode versions. Ours uses regex-syntax's tables, and Excel's PCRE2
  version is unknown.
- `\X` starting inside a cluster (after `.` or `\G`). Ours reads a cluster as if one started there, but PCRE2 may look
  back at earlier indicators or emoji.
- `\R` and `\X` inside a lookbehind.
Status: VERIFIED (excel-graph)
Tasks: T-2287, T-2364

## REGEX* patterns and match iteration are PCRE2 (T-1688)
Rule: Excel's patterns are PCRE2's. Backreferences match (`^(ab)\1$` on `abab` TRUE, on `abac` FALSE;
`(?<n>abc)\k<n>`, `(a)\g1`; with case_sensitivity 1 `(a)\1` matches `Aa`), a backreference to a missing group is
`#VALUE!` (`(a)\2`). Lookahead `(?=..)`, `(?!..)` and lookbehind `(?<=X)`, `(?<!1)` work; a variable-length lookbehind
`(?<=a+)` is `#VALUE!`. Possessive `a++a` and atomic `(?>a+)a` do not match `aaa` (REGEXEXTRACT `#N/A`, ERROR.TYPE 7).
`foo\Kbar` extracts `bar`; `(a)?(?(1)b|c)` extracts `ab`; `(?#note)` is a comment; `\Q.\E` is a literal `.`;
`(?x)` ignores pattern spaces; `(?i)h(?-i)ELLO` does not match `Hello`. An invalid pattern (`(`) is `#VALUE!`.
Global iteration is PCRE2's: after an empty match a non-empty match anchored at the same position is tried before
advancing one character, so `a*?` and `a??` on `aaa` replace to `bbbbbbb`, `(?:ab)*?` on `abab` to `-----`, `|b` on
`abc` to `-a---c-`, `b|` to `-a--c-` and `(?=b)|b` to `a--c`; REGEXEXTRACT's all-matches mode gives 7 matches for
`a*?` on `aaa` (`,a,,a,,a,`), and REGEXREPLACE's occurrence counts the same matches (occurrence 2 is `baa`, 4 is
`aba`, -1 `aaab`). An empty pattern over text with a surrogate pair is `#VALUE!` (`REGEXREPLACE("a"&UNICHAR(128512)&"b","","-")`,
ERROR.TYPE 3; with `x*` too, ERROR.TYPE 3). Only an iteration that reaches the pair that way fails: a non-empty match
steps over it (`REGEXREPLACE("😀b","😀|b","-")` is `--`, `.` on `a😀` gives `--`), an n-th occurrence stops at its
match (`REGEXREPLACE("a😀","","-",1)` is `-a😀`; with `a*` on `😀a` `-😀a`, `b|` occurrence 2 on `ab😀` `a-😀`, `a|`
occurrence 1 `-b😀`), while occurrence 3 of `""` on `a😀` and REGEXEXTRACT's all-matches mode over it are `#VALUE!`;
REGEXEXTRACT's first match is unaffected (`""`). A negative occurrence needs the whole iteration, so it is `#VALUE!`
in ours whenever the iteration splits a pair: UNVERIFIED (not probed).
Evidence: probes/regex-pcre2-features C1-C19, D1-D14, B27-B29 (excel-graph, 2026-10-18; D11 and D14 undecoded there,
answered by the next case); probes/regex-iteration-surrogates A1-A11 (excel-graph, 2026-10-22, T-1899).
Ours: PCRE2's grammar since T-1973 (functions/text/pcre/): comments, `\Q..\E`, `(?x)` and scoped flags match
(C14-C19). Backreferences (C1-C5), atomic groups and possessive quantifiers (B29, C11) and `\G` match on T-1900's
backtracker (functions/text/pcre/backtrack.rs). Several PCRE2 behaviors it implements are UNVERIFIED for Excel
(T-2356 probes them):
- a backreference ignores case as its scope's flags say;
- a backreference to a group that did not take part fails to match (PCRE2's default);
- a group's capture changes only when the group closes, so `^(a|b\1)+$` is FALSE on `abab`;
- `\G` stands at each iteration search's start;
- the empty-loop rule;
- captures made inside an atomic group are undone on backtracking.
`(?P=name)` is `#VALUE!` in ours, though PCRE2 accepts it. Lookaround, `\K` and conditionals run on the backtracker
since T-2354 (C7-C10, C13, C14, D12; probes/regex-lookaround-keepout-conditionals A1-A19, excel-graph, 2026-10-23): a positive
lookaround keeps its captures but is atomic (`^(?=(a+))a*b\1` is FALSE on `aaab`, and `(?=(a+))a*b\1` extracts
`aba` from `baaabac`), a negative one keeps none (`(?!(x))a(?(1)c|b)` on `ab` is `ab`), each top-level lookbehind
alternative has its own fixed length (`(?<=a|bc)d` finds `d` in `xbcd`), a lookbehind over an astral character steps
back one character (`(?<=.)b` on `😀b` is `b`), `\K` inside a lookaround is `#VALUE!` (ERROR.TYPE 3: PCRE2 10.38 and
later reject it), `\K` moves each match's start in iteration (`REGEXREPLACE("aaa","a\K","-")` is `a-a-a-`), and an
assertion condition is accepted and tested once (`(?(?=a)ab|c)` extracts `ab` from `ab` and `c` from `cd`). Iteration is PCRE2's since T-1899
(functions/text/regex_search.rs `each_match`): the retry runs on the backtracker, a regular pattern's on its memoized
mode, and an advance of one UTF-16 unit into a surrogate pair is `#VALUE!` (the cause, Excel iterating UTF-16
units, is inferred; every regex-iteration-surrogates cell matches it). The matcher is designed in SPEC-0013 (T-1898): T-1972 (probe of limits and
unprobed features), T-1973 (parser), T-1900 (backtracker), T-2354 (lookaround, `\K`, conditionals), T-2355 (memoized
mode), T-1899 (iteration), T-1974 (costs), T-2356 (probe of the backtracker's behaviors).
Status: VERIFIED (excel-graph); the backtracker behaviors listed above UNVERIFIED (T-2356)
Tasks: T-1688, T-1898, T-1899, T-1900, T-1972, T-1973, T-1974, T-2354, T-2355, T-2356

## CLEAN character set and PROPER word starts and case mapping (T-1307)
Rule (CLEAN): code points 0-31 and the C1 controls 128-159 (`CHAR(129)`, 141, 143, 144, 157 and `UNICHAR(133)` included) are removed; 32, DEL (127), NBSP (160), the soft hyphen (173), U+200B, U+2028 and U+FEFF stay. TAB, LF and CR go (`CLEAN("a"&CHAR(10)&"b"&CHAR(1)&"c")` is "abc"). The engine already matched all 38 CLEAN cells.
Rule (PROPER; UPPER, LOWER and further characters in the T-1747 section below): a letter after any non-letter, digits, apostrophes (straight or curly), `_` and `-` included, is upper-cased and the other letters lowered (`2Nd Place`, `O'Neil Mc-Donald`, `It'S 3Rd`, `A1B2C3`, `Hello_World`; full-width letters follow the same rule). Case mapping is per character and one-to-one: the characters whose full mapping expands stay as they are (`PROPER("ßa straße")` is `ßa Straße`, ŉ, ǰ and the ﬁ/ﬂ ligatures unchanged), where Rust's `to_uppercase` gives "SS", "ʼN" and "FI". İ folds to ASCII (`PROPER("İstanbul")` is `Istanbul`, `PROPER("aİb")` `Aib`) and ΐ, ΰ upper-case to Ϊ, Ϋ. Titlecase digraphs map to their capital (`ǅ ǈ ǋ ǲ` give `Ǆ Ǉ Ǌ Ǳ`). A capital sigma ending a word lowers to the final ς, with no letter or digit after it (`ΣΊΣΥΦΟΣ` is `Σίσυφος`, `ΑΣ.ΑΣ-Σ ΑΣ1` is `Ας.Ας-Σ Ασ1`, so a following digit keeps σ and a word-initial Σ stays). A combining mark is not a letter, so the next letter is capitalised (`e` + U+0301 + `cole` is `ÉCole`), and a zero-width space (U+200B) is not a letter either (`PROPER(U+200B&"abc def")` is `<U+200B>Abc Def`).
Evidence: probes/clean-proper-chars (excel-graph, 2026-10-16, 88 cells; engine matches all after T-1307). Implemented in `proper_case` (functions/text/string_format.rs).
Status: VERIFIED (excel-graph)
Tasks: T-1307

## UPPER, LOWER and PROPER: one-to-one case mapping, final-sigma context (T-1747)
Rule: UPPER, LOWER and PROPER all use the simple one-to-one mapping, so characters whose full mapping expands stay unchanged in every case (ᾳ, ẖ, ẘ, ﬀ, ﬃ, ﬆ, և, ß, ŉ, ǰ, ﬁ; `UPPER("ß")` is `ß`, not "SS"). ΐ upper-cases to Ϊ in UPPER and PROPER (LOWER leaves it), and ΰ to Ϋ in UPPER (probes/t3298-case-mapping-sweep; see "UPPER and LOWER use an old per-character case table" below for every other character). İ is the exception to consistency: PROPER and LOWER fold it to I / i (`LOWER("aİb İc")` is `aib ic`) but UPPER leaves it as İ. A capital Σ lowers to the final ς in LOWER and PROPER when the next character is the end of the text, a space, a straight apostrophe or `-`, and to σ when a letter or digit, a combining mark (probed: U+0301, U+0308; the rest of U+0300-U+036F is assumed, other combining blocks and U+200C/U+200D/U+2018 are unprobed and give ς), a curly apostrophe (U+2019) or a zero-width space (U+200B) follows. In LOWER a lone `Σ`, `Σ ` and `1Σ` give ς (no preceding letter needed), but `Σ`+U+0308 gives σ̈; lower-case σ is never changed (`LOWER("ασ")` is `ασ`). PROPER keeps a word-initial Σ capital.
Evidence: probes/case-mapping-expanding (excel-graph, 2026-10-17, 81 cells; engine matches all).
Status: VERIFIED (excel-graph). Implemented in: `lower_case`, `upper_case`, `proper_case` (functions/text/string_format.rs). Tasks: T-1747

## UPPER and LOWER use an old per-character case table (T-3298)
Rule: UPPER and LOWER map a character only if Excel's own case table has it, and that table is far older than current
Unicode (it reads like the Windows NT-era tables): many letters whose case pair Unicode added later are left unchanged.
Swept over all 3037 code points whose Rust std (rustc 1.97) `to_lowercase` or `to_uppercase` differs from itself, BMP and
supplementary. Where Excel maps a character it gives Rust's one-to-one mapping, except as listed here.
- UPPER leaves these unchanged (code point ranges of the swept characters: every swept character inside a range is unchanged, and the characters
  in a range that were not swept are not claimed):
  00B5, 017F, 019B, 023F-0240, 0250-0252, 025C, 0261, 0264-0266, 026A, 026C, 0271, 0282, 0287, 029D-029E, 0345,
  0371-0373, 0377, 037B-037D, 03D7-03D9, 03F2-03F5, 03F8, 03FB, 0515-052F, 10D0-10FA, 10FD-10FF, 13F8-13FD, 1C80-1C8A,
  1D79, 1D8E, 1E9B, 1EFB-1F07, 1F10-1F15, 1F20-1F27, 1F30-1F37, 1F40-1F45, 1F51, 1F53, 1F55, 1F57, 1F60-1F67, 1F70-1F7D, 1FB0-1FB1,
  1FBE, 1FD0-1FD1, 1FE0-1FE1, 1FE5, 2170-217F, 2C30-2C5F, 2C73, 2C81-2CE3, 2CEC-2CEE, 2CF3, 2D00-2D27, 2D2D,
  A641-A66D, A681-A69B, A723-A72F, A733-A76F, A77A-A77C, A77F-A787, A78C, A791-A794, A797-A7A9, A7B5-A7C3, A7C8-A7CA,
  A7CD-A7DB, A7F6, AB53, AB70-ABBF, and every supplementary-plane letter swept (Deseret 10428-1044F, Osage 104D8-104FB,
  Vithkuqi 10597-105BC, Old Hungarian 10CC0-10CF2, Garay 10D70-10D85, Warang Citi 118C0-118DF, Medefaidrin
  16E60-16E7F, 16EBB-16ED3, Adlam 1E922-1E943). So `UPPER("µ")` is `µ`, `UPPER("ſ")` `ſ`, Georgian Mkhedruli,
  Glagolitic, Coptic, Cherokee, the small Roman numerals and most of Greek Extended stay as they are.
- UPPER drops the tonos of the Greek accented vowels: ά έ ή ί ό ύ ώ give Α Ε Η Ι Ο Υ Ω (Rust: Ά Έ Ή Ί Ό Ύ Ώ), and ΐ ΰ give
  Ϊ Ϋ (Rust has no one-to-one mapping for them).
- LOWER leaves these unchanged: 0370-0372, 0376, 037F, 03CF, 03D8, 03F4, 03F7-03FA, 03FD-03FF, 0514-052E, 10C7, 10CD,
  13A0-13F5, 1C89, 1C90-1CBA, 1CBD-1CBF, 1E9E, 1EFA-1EFE, 1F08-1F0F, 1F18-1F1D, 1F28-1F2F, 1F38-1F3F, 1F48-1F4D,
  1F59-1F5F, 1F68-1F6F, 1F88-1F8F, 1F98-1F9F, 1FA8-1FAF, 1FB8-1FBC, 1FC8-1FCC, 1FD8-1FDB, 1FE8-1FEC, 1FF8-1FFC, 2126,
  212A-212B, 2160-216F, 2C00-2C2F, 2C6D-2C72, 2C7E-2CE2, 2CEB-2CED, 2CF2, A640-A66C, A680-A69A, A722-A72E, A732-A76E,
  A779-A786, A78B-A78D, A790-A792, A796-A7DC, A7F5, and every supplementary-plane capital swept (10400-10427,
  104B0-104D3, 10570-10595, 10C80-10CB2, 10D50-10D65, 118A0-118BF, 16E40-16E5F, 16EA0-16EB8, 1E900-1E921). So
  `LOWER("ẞ")` is `ẞ`, and the Ohm, Kelvin and Ångström signs (U+2126, U+212A, U+212B) stay as they are.
- LOWER maps Georgian Asomtavruli U+10A0-U+10C5 to Mkhedruli U+10D0-U+10F5 (Rust: to Nuskhuri U+2D00-U+2D25), and
  İ (U+0130) to i (Rust's full mapping is i+U+0307; T-1747 already had `LOWER("aİb")`).
A surrogate pair is never case-mapped (no supplementary letter changes in either direction).
Characters whose full mapping expands (ß, ŉ, ǰ, the ligatures U+FB00-U+FB17, the Greek with ypogegrammeni) stay
unchanged, as T-1747 found.
Evidence: probes/t3298-case-mapping-sweep Calc!A1:A50 (UPPER of 1580 characters, 32 per cell, Data!A) and
Calc!B1:B47 (LOWER of 1488, Data!B) (excel-graph, 2026-10-31).
Status: VERIFIED (excel-graph). Not implemented: the engine maps every character through Rust's tables
(`simple_case`), so the cells listed in eval/baseline/probes.txt fail. Tasks: T-3298 (probe), T-3443 (engine).

## ARRAYTOTEXT: a single empty cell is #VALUE!, an empty cell in a larger range is empty text
Rule: `ARRAYTOTEXT` of a reference to one empty cell is `#VALUE!` in both formats, and so are a 1x1 range of an empty
cell, `(B1)`, `IF(TRUE,B1)`, `INDEX(B1:B2,1)` of an empty cell and an omitted first argument (`ARRAYTOTEXT(,1)`). Within a
multi-cell range an empty cell is empty text (`B1:B2` all empty is ", ", strict "{;}"; `C1:C3` with C2 empty is "1, , 3",
strict "{1;;3}"). Computed empties are not references: `B1+0` is "0" and `B1&""` is "". `VALUETOTEXT(B1)` of an empty
cell is "" in both formats, and a typed `""` is "" / `{""}`. An array element is not a reference: GROUPBY hands a one-blank group to
its function as an array and `ARRAYTOTEXT` of it is "" (probes/groupby-blank-values).
Evidence: probes/arraytotext-empty-ref A1-A25 (excel-graph, 2026-10-17, T-1718).
Status: VERIFIED (excel-graph). Implemented in: `fn_arraytotext`. Tasks: T-1718

## Element-wise text functions on the lift driver: a LAMBDA, a 1x1 array, VALUETOTEXT's error format (T-1775)
Rule (the engine's, unprobed): since the element-wise text functions (LEFT/RIGHT/MID and their *B twins, LEN/LENB, FIND/SEARCH, REPT, SUBSTITUTE, REPLACE, EXACT, TEXT, VALUE, VALUETOTEXT, CODE, UNICHAR, BAHTTEXT, FILTERXML, CONCATENATE, DATEVALUE, TIMEVALUE) lift through the SPEC-0011 driver, three edges follow the driver's rule rather than the old `broadcast_text`'s:
- a LAMBDA argument makes every element `#VALUE!`, whatever the other arguments hold, as before (`FILTERXML(LAMBDA(x,x),NA())` is `#VALUE!` in Excel, probes/filterxml-array-forms A9; an error before the LAMBDA, `LEFT(NA(),LAMBDA(x,x))`, is unprobed), but an array argument now keeps its shape (`COLUMNS(LEFT({"a","b"},LAMBDA(x,x)))` is 2, was one `#VALUE!`);
- a 1x1 array argument gives a 1x1 array, as in every other lifted family and in TEXTSPLIT ("TEXTSPLIT, TEXTBEFORE, TEXTAFTER and REGEX* lift ..." above): `TYPE(LEFT({"abc"},2))` is 64, `ARRAYTOTEXT(MID({"abc"},"x",NA()),1)` "{#VALUE!}" (was 2 and `#VALUE!`). A one-cell range is still its cell's value (`TYPE(LEFT(A1:A1,2))` is 2);
- VALUETOTEXT with an array value and a scalar error format is an array of that error, one per element (was the one error).
A legacy (pre-dynamic-array) `RIGHT(A1:A3)` now gets the implicit-intersection `@`: the old hand-written static arm wanted two arguments, the table says one or two (`RIGHT(text, [num_chars])`), as LEFT's arm did.
The probed behaviour is unchanged (error precedence, the broadcast padding, the size and text bounds; eval 0 fixes, 0 regressions).
Evidence: probes/filterxml-array-forms A9 for the LAMBDA's precedence; none for the shapes (crates/rustsheet-engine/src/test/text_functions/lift_driver.rs pins the engine's values).
Status: UNVERIFIED (probe pending, T-1963)
Implemented in: `Model::lift_text` (functions/text/text_kernel.rs), the text table entries (functions/lift/kinds/text.rs).
Tasks: T-1775, T-1963

## VALUE lifts over a range or an array (T-1791)
Rule: VALUE lifts over a multi-cell range or an array, each element read as a scalar argument: text converts as in scalar VALUE (`"1 "` is 1, `" $2 "` 2, `"1/2/2020"` 43832, `" 3%"` 0.03), a number stays, a blank cell is 0, a boolean or `""` is `#VALUE!` and an error stays (`VALUE(A1:A8)` spills 8 rows). So `{=SUM(IFERROR(VALUE(LEFT(range,FIND(...)-1)),0))}` adds the numbers in front of the found text (corpus-bench/42181 B14 is 3).
Not probed: a multi-cell range in a legacy (non-dynamic-array) cell, which takes the implicit intersection of the other lifted text functions.
Evidence: probes/value-argument-lift (excel-graph, 2026-10-18); corpus-bench/42181 B14 (Excel cached value).
Implemented in: `fn_value` (functions/text/common.rs), through the lift driver (`Model::lift_text`, T-1775).
Status: VERIFIED (excel-graph)
Tasks: T-1791

## VALUE reads a mixed fraction, a sign before a space and a one-digit-year dash date (T-2646)
Rule: `VALUE("- 5")` is -5, `VALUE("3 1/2")` 3.5 and `VALUE("1-1-1")` 36892 (2001-01-01, month-day-year with a
one-digit year). Readers built on VALUE's grammar follow it: `IMABS("3 1/2")` is 3.5 (engineering.md "How an inumber
is read").
Evidence: probes/eng-inumber-date-boundary B21:B23, A22 (excel-graph, 2026-10-25).
A criterion reads `"1 1/2"` as 1.5 too (criteria.md "Criterion operands and text cells read numbers with VALUE's grammar",
probes/t3136-criteria-formatted-number-operands H25).
Edges (probes/value-sign-space-mixed-fraction-dash-date, excel-graph, 2026-10-29): spaces after a sign are skipped
(`"- 5"`, `"-  5"` -5, `"+ 5"` 5, `"- 5.5"` -5.5, and `="- 5"+0` -5, so number coercion shares it). A mixed fraction takes
a sign (`"-3 1/2"`, `"- 3 1/2"`, `"-  3 1/2"` -3.5, `"+3 1/2"` 3.5), a `$` (3.5) and a `%` (`"3 1/2%"` is
0.034999999999999996, the whole and the fraction divided by 100 apart); an improper fraction reads (`"1 3/2"` 2.5), a
numerator with a leading zero too (`"3 01/02"` 3.5), many digits (`"123456789 1/2"`), a denominator of 200 (3.005); but
`"3 1/0"`, `"3  1/2"` (two spaces), `"3 1 /2"` and `"3 1/ 2"` are `#VALUE!`. A dash date reads a one-digit year as 200x
and `"1/1/1"` is the same 36892 (`"1-1-0"` 36526, `"1-1-99"` 36161, `"1-1-30"` 1930); `"13-1-1"` and `"0-1-1"` are `#VALUE!`.
Typed input, criteria and coercion (probes/typed-input-sign-space-mixed-fraction, excel-graph, 2026-10-30, T-3178): typed
`- 5`, `+ 5`, `1-1-1` and `1/1/1` are numbers (-5, 5, 36892, 36892), and so are the typed mixed fractions `-$3 1/2` (-3.5),
`-3 1/2%` (-0.034999999999999996) and `+ 3 1/2` (3.5); typed `€3 1/2` (no euro in en-US) and `$3 1/2%` stay text; unsigned `3 1/2`, `$3 1/2` and `3 1/2%` are numbers (B10-B12). Number
coercion reads `"€3 1/2"` 3.5, `"-$3 1/2"`, `"-3 1/2%"` and `"+ 3 1/2"` too, but `"$3 1/2%"` is `#VALUE!`, and so is `VALUE("$3 1/2%")` (J1): a currency
symbol and a `%` do not combine in a mixed fraction. A criterion operand reads `- 5`, `+ 5`, both dates, `-$3 1/2`,
`-3 1/2%` and `+ 3 1/2` but not `€3 1/2` or `$3 1/2%` (`COUNTIF(3.5,"€3 1/2")` is 0), while a text cell `€3 1/2` does read
as 3.5 (`COUNTIF(A5,A5+0)` is 1). A typed mixed fraction gets the number format `# ?/?` when its denominator is a single digit (`3 1/2`, `-$3 1/2`,
`+ 3 1/2`, `$3 1/2`, `-3 1/2`, `1 3/2`, `3 01/02`, `3 001/2`, `3 1/002`; no currency format even with `$`) and `# ??/??` when
it is 10 or more (`3 1/10`, `3 1/12`, `1 1/16`, `-$1 7/16`, `3 1/99`, `3 1/999`, `3 1/10000`), leading zeros not counted, the typed denominator not the reduced one (`3 5/10` is `??/??`, `3 2/4` `?/?`) and the numerator's width not at all (`3 12/5` displays `5 2/5`, `3 10/5` displays `5`);
one with a `%` gets `0.00%` whatever the denominator (`3 1/2%`, `-3 1/2%`, `2 5/16%`). Displayed, `3 1/12` is `3  1/12`
and `3 1/200` (three digits fit no `??/??`) is `3` plus six spaces. Typed `-$3 1/2%` stays text, like `$3 1/2%`, but the cell still gets `0.00%` (A15 ENTERED numberFormat)
(probes/typed-mixed-fraction-number-format, excel-graph, 2026-10-30, T-3237; raw ENTERED lines and displayed text in
docs/research/typed-mixed-fraction-number-format-2026-10-30.md).
Ours: all implemented (`mixed_fraction`, mixed_fraction.rs, read through `value_number_text_with`, cast.rs; `parse_number`, parse_formatted.rs; `parse_separated`, date_text.rs).
Status: VERIFIED (excel-graph); implemented (T-3151, T-2646).
Tasks: T-2646, T-3151, T-3178

## REGEX* POSIX classes are Unicode (T-1903)
Rule: inside a bracket, a POSIX class matches what PCRE2 in UCP mode matches, by Unicode property, not ASCII only:
`alnum` L or N (`²`, `½`, `Ⅳ`, Arabic-Indic and fullwidth digits too); `alpha` L (`ª`, CJK; not `²` or `Ⅳ`); `ascii` U+0000-U+007F;
`blank` tab and the Zs spaces (NBSP, U+3000; not LF, VT, NEL or U+2028); `cntrl` Cc (NEL too; not the soft hyphen, ZWSP or BOM);
`digit` Nd (Arabic-Indic and fullwidth digits; not `²`); `graph` L, M, N, P, S and Cf (soft hyphen, ZWSP, BOM, a combining mark and an
emoji are graphic; spaces and controls are not); `print` graph plus the Zs spaces (not tab or U+2028); `punct` every P plus the ASCII
symbols (`$`, `+`, `<`, `^`, `@`, the backtick and `~` are punctuation; `×` and `€` are not); `space` White_Space (NEL, NBSP, U+2028, U+3000; not ZWSP); `lower`
Ll and `upper` Lu (`ª` is not lower, `ǅ` neither); `word` L, N, Mn and Pc (`_`, `²`, a combining mark; not an emoji); `xdigit` 0-9,
A-F, a-f and their fullwidth forms (not Arabic-Indic digits). `[:^name:]` and `[^[:name:]]` are the complements.
`[:upper:]` and `[:lower:]` ignore case-insensitive matching: `REGEXTEST("a","(?i)[[:upper:]]")` and `REGEXTEST("A","(?i)[[:lower:]]")`
are FALSE, and so is `(?i)[[[:upper:]]]` on `a`; negated, they complement the unfolded class (`(?i)[^[:upper:]]` and `(?i)[[:^upper:]]` on `a`
are TRUE). Next to a literal they still are (`(?i)[[:upper:]x]` on `a` FALSE). The other items of the same bracket do fold:
`(?i)[^[:upper:]x]` is FALSE on `X`, `x` and `B` and TRUE on `b` (the complement of unfolded Lu plus folded x), `(?i)[[:^upper:]x]` matches `X`,
`(?i)[[:lower:]a-c]` matches `B` but not `D`. `(?i)` (and case_sensitivity 1) does fold `\p{..}` and `\P{..}`, unlike older PCRE2 documentation, which says caseless matching leaves `\p` alone: `(?i)\p{Lu}` and `(?i)[\p{Lu}]` match `a`, `(?i)\p{Ll}` matches `A`, `(?i)[^\p{Ll}]` does not match `A`, `(?i)\P{Lu}` and `(?i)[[:digit:]\P{Lu}]` do not match `a` (the complement of the folded set), `(?i)[^\P{Lu}]` does; the same inside a bracket beside a POSIX class (`(?i)[[:digit:]\p{Lu}]`, Kelvin sign against `[[:digit:]\p{Ll}]`), `\w`, `\d`, `\s`, `\W` and `\D` match as usual inside and outside such a bracket (A12-A17, A32-A37), but these inputs cannot show whether they fold (UNVERIFIED). Ours matches all forty cells (probes/regex-p-fold A1-A40, excel-graph, 2026-10-20, T-2092). Through `&&` and a scoped or re-set flag too: `(?i)[[:upper:]&&[:alpha:]]`,
`(?i:[[:upper:]])` and `(?i)(?-i)(?i)[^[:^upper:]]` are FALSE on `a`. No POSIX class folds: `(?i)[[:ascii:]]` is FALSE on the Kelvin sign
(U+212A) and long s (U+017F), though `(?i)k` matches the Kelvin sign (for the other classes folding changes nothing).
Brackets nest, unlike stock PCRE2 (which reads `[[a]]` as `[[a]` then `]`): `[[a]]` matches `a` but not `[`, `(?i)[[[:upper:]]]`
matches `A` and not `[]` (probes/regex-posix-classes-c A1-A26, excel-graph, 2026-10-20, T-1989). Ours matches all of these (T-1989).
The case_sensitivity argument 1 behaves the same (`REGEXTEST("a","[[:upper:]]",1)` is FALSE). `&&` in a bracket intersects (`[[:alpha:]&&[:lower:]]`
on `A` and `&` is FALSE); a literal `^`, `-` or `&` next to a class stays literal (`[[:upper:]^a]` matches `^`, not `b`).
Syntax: `[a[:digit:]]` and `[[:alpha:][:digit:]]` mix classes with other items; `[\[:alpha:]]` is no class (`":"` FALSE).
`[:alpha:]` outside a bracket, an unknown or upper-case name (`[[:foo:]]`, `[[:ALPHA:]]`), `[[.a.]]`, `[[=a=]]` and a range from a class
(`[[:alpha:]-z]`, `[a[:upper:]-z]`) are invalid patterns (`#VALUE!`, ERROR.TYPE 3; probes/regex-posix-classes Z1-Z3, Z8-Z10, regex-posix-classes-b A4, A5; T-1985).
`[[:alpha:]-]` and `[-[:alpha:]]` stay valid (a literal hyphen next to a class); a top-level `[.a.]` or `[..]`, a multi-character `[[.ab.]]` and a range that ends at a class (`[a-[:alpha:]]`, `[a-[:digit:]]`, and `[^a-[:alpha:]]`) are invalid patterns (`#VALUE!`, ERROR.TYPE 3), as PCRE2's `check_posix_syntax` has it (probes/regex-posix-syntax-edges A1-A14, excel-graph, 2026-10-19, T-2010). Ours already agrees on all fourteen.
U+061C, U+180E and U+2066-U+2069 are not graph; U+061C is not cntrl either; U+180E is blank and space (so space is White_Space plus U+180E,
not White_Space alone; ours includes it since T-1989); U+0903 (a spacing mark, Mc) is graph but neither word nor alpha (probes/regex-posix-classes-c
A27-A36, excel-graph, 2026-10-20, T-1989). UNVERIFIED: the same code points in print.
Evidence: probes/regex-posix-classes (excel-graph, 2026-10-19): 47 characters × 19 patterns, and column Z for the syntax;
probes/regex-posix-classes-b (bracket neighbours, nested and negated `(?i)`, `&&`, case_sensitivity 1);
probes/regex-posix-classes-c (nesting, folding of the other items, `[:ascii:]` under `(?i)`, the code points above; T-1989);
probes/regex-posix-syntax-edges (hyphen next to a class, `[.a.]`, `[..]`, `[[.ab.]]`, a range ending at a class);
probes/regexreplace-limit-and-syntax A23.
Implemented in: `Parser::bracket` and `Parser::posix_class` in functions/text/pcre/parse.rs (T-1973, replacing T-1903's
pre-compile rewrite). Each POSIX class is built as its Unicode class. Where `(?i)` is on (tracked through flag groups
and scopes, starting from the case_sensitivity argument), a bracket's other items are case-folded and its POSIX classes
are not. Case folding is charged per code point to the call's compile budget (the REGEX* limits entry above).
Status: VERIFIED (excel-graph)

## REGEXREPLACE results are capped at 65,535 UTF-16 units, not 32,767; replacement syntax and Unicode classes (T-1817)
Rule: a REGEXREPLACE result may be up to 65,535 UTF-16 units; one unit more is `#CALC!`. `LEN(REGEXREPLACE(REPT("a",32767),"a","bb"))` is 65534; with `REPT("a",N)&REPT("c",M)` and `"a"` → `"bbb"`, 65,535 units is 65535 and 65,536 or 65,537 is `#CALC!`; the cap counts UTF-16 units, not characters (`"a"` → `"b"&UNICHAR(128512)`: 65,535 units in 49,151 characters is 65535, 65,536 units in 49,150 characters `#CALC!`), and not matches (32,767 matches giving 65,534 units pass). 98,301 to 327,670 units (`REPT("a",32767)`, `"a"` → 3 to 10 b's) and the empty pattern with `bbbbbbbbbb` (360,447) are `#CALC!`. Lifted, the cap is per element: `REGEXREPLACE(REPT("a",{1,32767}),"","bbbbbbbbbb")` is `"bbbbbbbbbbabbbbbbbbbb"` and `#CALC!` (probes/regexreplace-result-cap-lifted). A cell holds such a result: `=REGEXREPLACE(REPT("a",32767),"a","bb")` is the 65,534 b's, `LEN` of the cell 65534, `ISERROR` FALSE.
Other functions on a result over 32,767 units: `LEN`, `LEFT`, `MID(..,40000,10)` (10), `ISTEXT` (TRUE), `ISERR`/`ISERROR` (FALSE), `IFERROR(LEN(..),-1)` (65534), a nested `REGEXREPLACE`, `REGEXEXTRACT` and `REGEXTEST(..,"^b{65534}$")` (TRUE) take it as text; `SUBSTITUTE(..,"b","c")` and `UPPER(..)` are `#VALUE!` (ours 65534; T-1904). `TYPE` is 64 and `ERROR.TYPE` 3, as for any text over 255 units (information.md "TYPE and ERROR.TYPE of text over 255 UTF-16 units").
Replacement syntax: `$1$1` and `${1}x` on `(a)` give `aab` and `axb`, `$0$0` gives `aab`; named group `${x}$x` gives `aab`. A bare `$`, `a$` and `$|$$` forms are rich-value errors the oracle does not decode yet (A12, A17, A18 pending).
Unicode: `\w`, `\pL`, `\p{L}`, `\b` and the POSIX class `[[:alpha:]]` all treat é as a letter (`REGEXTEST("é","[[:alpha:]]")` is TRUE; ours too since T-1903); `REGEXEXTRACT("naïve","\w+")` is "naïve".
Probing note: excel-graph refuses to open (OpenWorkbookBlockedWorkbook, "exceeds the size limit") a probe workbook that builds more than two or so 65,534-unit texts, or a `REPT("b",32767)` replacement matched twice, so the long-result questions are split over probes/regexreplace-long-result-a to -i; `LEN(REGEXREPLACE(REPT("a",32767),"a","bb")&"")` could not be opened even alone (unprobed).
Evidence: probes/regexreplace-limit-and-syntax, probes/regexreplace-result-cap, probes/regexreplace-result-cap-lifted, probes/regexreplace-long-result-a to -g and -i (excel-graph, 2026-10-18).
Implemented in: `MAX_REPLACED_UNITS` / `over_cap` in functions/text/regex_functions.rs, checked on the measured length before the result is built.
Status: VERIFIED (excel-graph)
Tasks: T-1817

## SUBSTITUTE accepts an instance_num of 1 to 2147483646 only (T-2279)
Rule: SUBSTITUTE floors instance_num and gives `#VALUE!` unless the result is in 1..=2147483646
(2^31 - 2), whether or not text holds a match, and also for an empty text or empty old_text:
`SUBSTITUTE("bbb","b","x",2147483646.9)` is "bbb" and `SUBSTITUTE("bbb","b","x",2147483647)` is
`#VALUE!`; 32767, 65536, 2^30 and 2147483520 are fine; 2^31, 2^32, 2^53, 1E+10, 1E+15 and 1E+300 are
`#VALUE!`; 0.5 is `#VALUE!` and 1.5 acts as 1. This explains gen-m6/9615a6de0f3ac417 Calc!A144
(`LOOKUP(SUBSTITUTE(...,1E+15),...)` is `#VALUE!`).
Evidence: probes/substitute-instance-num-bound (excel-graph, 2026-10-21). Its D5 (an array instance_num) is
`#SPILL!` only because the probe cell has no room to spill; it says nothing about the bound.
Status: implemented (`substitute_element`)
Tasks: T-2279

## ARRAYTOTEXT: an error in the array argument wins over an invalid format (T-2250)
Rule: `ARRAYTOTEXT(<error>, 2)` is the array argument's error, not the `#VALUE!` of the invalid format 2: `ARRAYTOTEXT(SINH(134217727),2)` is `#NUM!`. Whether this holds for every error kind and for a non-scalar array is unprobed.
Ours: `#VALUE!` (the format is checked first).
Evidence: gen-m5/4febabc66cfdf9a1 Calc!A156 (excel-graph, 2026-10-20, T-2114).
Status: VERIFIED (excel-graph); not implemented (T-2250).
Tasks: T-2169, T-2250

## TEXTBEFORE and TEXTAFTER: instance_num past the text, a number if_not_found
Rule: an instance_num whose size is greater than the number of positions the shortest delimiter could start at (the text's length
less that delimiter's, plus one, counted in UTF-16 units) is `#VALUE!`, whatever match_end and if_not_found (the empty delimiter
excepted, see above). `TEXTBEFORE("abcde","xyz",3)` and `-3` are `#N/A`, `4` and `-4` `#VALUE!`; `TEXTBEFORE("abcde",{"xyz","qq"},4)`
is `#N/A`, `5` `#VALUE!`; `TEXTBEFORE("ab","xyz",1)` is `#VALUE!`; a number delimiter is its text, so `TEXTBEFORE("42ABC",1.005,-2)`,
also from a cell, is `#VALUE!` (gen-m9 8d3588d8df0812df A313) but `-1` `#N/A`; `TEXTAFTER("😀b","😀😀",2)` is `#VALUE!` (probes/
t2581-textsplit-textbefore A30-A50, excel-graph, 2026-10-24; T-2581). Unprobed: a delimiter whose case folding changes its UTF-16 length under
match_mode 1 (ours measures the unfolded delimiter), and a delimiter that fits the text by UTF-8 bytes but not by UTF-16 positions
(A50 is decided by ours' earlier every-delimiter-longer check). With a one-unit delimiter this is the text's length: `TEXTBEFORE("b","a",-2,1,1)`, `TEXTBEFORE("ab","a",3)` and
`TEXTBEFORE("ab","a",3,0,0,"nf")` are `#VALUE!`, while `TEXTBEFORE("ab","a",2)` is the usual `#N/A`. `TEXTAFTER("😀","a",2)` is `#N/A`
and `TEXTAFTER("😀","a",3)` `#VALUE!` (two units), `TEXTAFTER("é","a",2)` `#VALUE!` (one). Gen-m7 shows it on `TEXTBEFORE("x~*","1",
WORKDAY(..))` (611ce3bbfab27cef A336) and `TEXTBEFORE(..,"-",DATE(2060,10,19),0,0,6)` (b24ce74f08d3f91a A767).
match_mode is cut toward zero and must then be 0 or 1, else `#VALUE!`, also beside the empty delimiter: 2, -1, 255 and 123456789.123
are `#VALUE!` (`TEXTBEFORE("aBc","b",1,2)`; `TEXTBEFORE("abc","",2,123456789.123)`, which is c36ff5f42a1a4d47 A148), 0.5 is 0
(`#N/A` for a case-different delimiter), 1.5 and TRUE are 1. A number or boolean if_not_found comes back as its text:
`TEXTBEFORE("b","a",1,0,0,5)` is "5", `-2.5` is "-2.5", TRUE is "TRUE" (also gen-m7 ac67253061bba16a A776, A1345).
Evidence: probes/t2314-textbefore-instance-bound (excel-graph, 2026-10-22) and gen-m7 (excel-graph, 2026-10-21, T-2225).
Ours: matches (`text_around_element`, `around_options` and `around_not_found` in functions/text/before_after.rs, T-2314). The probe's
`TEXTBEFORE("abc","b",1,-0.5)` is "a", not `#VALUE!`: flooring -0.5 would give the invalid -1 (A2), so match_mode is cut toward zero.
match_end is read the same way: -0.0001 is 0, so `TEXTAFTER(LOWER("'a"),Data!A11:B12,1,1,<-0.0001>)`, where no delimiter is in the text,
is `#N/A` (with match_end 1 the end of the text would match and give ""): gen-m7/2d9b48c6363e7a16 Calc!A660 (excel-graph, 2026-10-22); 0.5 is 0 too: `TEXTAFTER("1.5","-",-1,1,0.5)` is `#N/A`, gen-m9/b9a32ce9a1fd0d71 Calc!A142.
Ours: implemented (T-2412), `around_options` cuts match_end toward zero. The cut value must then be 0 or 1, else `#VALUE!`, also when the
delimiter is found and beside an if_not_found: 1.5 (also from a cell), 1.9999, TRUE (also from a cell), the text "1" (also from a cell) and the text "1.5" are 1, 0.9999,
-0.5 and FALSE are 0, while 2, -1 and -1.5 are `#VALUE!` (`TEXTAFTER("ab","b",1,0,2)` is `#VALUE!`, not ""), and the text "TRUE" is `#VALUE!`.
TEXTBEFORE reads it the same way (A16-A30): with the delimiter not found, a value cut to 1 gives the whole text "ab", one cut to 0 `#N/A`, and the same values are `#VALUE!`. `TEXTAFTER("a","zz",1,0,<any>)` is `#VALUE!` for every value (the longer-delimiter rule above; for 2 and -1 both checks give `#VALUE!`, so their order is not shown). Evidence: probes/t2498-textafter-match-end (excel-graph,
2026-10-25, T-2498). Ours: matches (`around_options`).
An instance_num that is the empty value (`CELL("contents",<blank>)`) behaves as 0 (`#VALUE!`), not as an omitted argument:
`TEXTAFTER(((Data!C$12&"")&Data!E10),"-",CELL("contents",Data!B3),1,1)` with Data!B3 blank (gen-m7/9359f03972e94026 Calc!A1420,
excel-graph). A left-out argument (`TEXTAFTER(..,"-",,1)`) keeps the default 1. Ours: matches since T-3053 (`around_options` reads
a written empty value as 0, so match_mode and match_end do too, unprobed: T-3087; information.md "CELL("contents", ref) as a VSTACK argument").
TEXTAFTER's result takes the shape of an array if_not_found even when the text is an error (T-2414, probes/t2414-gen-m7-unexplained, excel-graph, 2026-10-23):
`ROWS(TEXTAFTER("abc","",1,-2^53,0,{1;2}))` is 2 (invalid match_mode: each cell `#VALUE!`), `TEXTAFTER("abc","b",1,0,0,{1;2})` is "c","c", and
`ROWS(TEXTAFTER(<#VALUE!>,"",1,-2^53,0,OFFSET(B9:C12,5,1,DROP(A3:A5,-2,0),2)))` is 2. The OFFSET alone is a 2x2 reference
(`ROWS`, `COLUMNS` 2, 2; `INDEX(..,1,1)` one 0, no spill), although `DROP(A3:A5,-2,0)` is the one cell A3 (1E-15): UNVERIFIED mechanism
(Excel reads the one-cell DROP result as a height giving 2 rows, ours as a lifted 1x1). The gen-m7 `#SPILL!` (4dac39f89709e2fc A2870) is
then consistent with a 2x2 TEXTAFTER of `#VALUE!` colliding; only ROWS = 2 of it is probed.
Status: VERIFIED (excel-graph)
Tasks: T-2314, T-2412, T-2581

## TEXTSPLIT: argument errors first, an all-empty result
Rule: TEXTSPLIT checks its arguments in this order, the first failure being the result: the text (an error); a delimiter that is
itself an error (col_delimiter, then row_delimiter); ignore_empty (an error, or a value that is not a boolean or number: `#VALUE!`);
match_mode, alike; an array pad_with (`#VALUE!`); an error element of a delimiter list (an array holding an error, or an OFFSET array
of references, which is `#VALUE!`); then TEXTSPLIT's own `#VALUE!` for an empty delimiter, no delimiter or empty text.
So `TEXTSPLIT("abc",",",NA(),1/0)` is `#N/A` and `TEXTSPLIT("abc",",",NA(),"x")` `#N/A`, but `TEXTSPLIT("abc",{",",#N/A},,1/0)` is
`#DIV/0!`; `TEXTSPLIT("abc",",",,1/0,NA())` is `#DIV/0!`, `TEXTSPLIT("abc",",",,"x",NA())` `#VALUE!`; `TEXTSPLIT("abc",",",,1/0,,{1,2})`
is `#DIV/0!`, `TEXTSPLIT("abc",NA(),,,,{1,2})` and `TEXTSPLIT("abc",",",,,NA(),{1,2})` are `#N/A`; `TEXTSPLIT("abc",",",
OFFSET(C1:C2,{0;1},0))` is `#VALUE!` but with ignore_empty `NA()` `#N/A` (probes/textsplit-error-order A1-A10, A13-A18). A split whose
every piece ignore_empty drops is `#CALC!`, with or without a row delimiter (`TEXTSPLIT(",",",",,TRUE)`, `TEXTSPLIT(",;,",",",";",TRUE)`;
A11, A12). The array pad_with does not lift, but its `#VALUE!` is per element: beside a lifted text or ignore_empty the call spills one
`#VALUE!` per element, and an error element of the text stays that error.
Evidence: probes/textsplit-error-order (excel-graph, 2026-10-23, T-2315). gen-m7 (excel-graph, 2026-10-21, T-2225):
`TEXTSPLIT("ABC",Data!D4:E7,Data!C5:E9,-1,<#DIV/0!>)` is `#DIV/0!` (90de8755a2e565e6 Calc!A1643, the delimiter ranges hold blanks),
`TEXTSPLIT("","-",Data!A11:D12,<#REF!>)` is `#REF!` (ac67253061bba16a A184), `TEXTSPLIT(100,{1.005,100},"a",TRUE)` under LOWER and
REGEXEXTRACT gives `#CALC!` (07d27aa0add0247f A768), and an array ignore_empty lifts (`TEXTSPLIT("x~*",Data!C7:C11," ",UNIQUE(Data!C4:E8),1,..)`
spills 15 `#VALUE!`: 8044dce50616cfe9 A1064).
More of the same (gen-m7, excel-graph, 2026-10-22, T-2254 triage): an error ignore_empty beats the blank-delimiter `#VALUE!` for `#REF!`
(`INDIRECT(..)`, 21b4eef18424ce94 A733), `#DIV/0!` (2d9b48c6363e7a16 A2333) and `#N/A` (b2078b5d702c691a A356, beside an OFFSET array of
references as the row delimiter; 4dac39f89709e2fc A3213 over a 2-row text); an error text beats an array pad_with's `#VALUE!`
(`Data!D$12&BYCOL(Data!E5:E6,LAMBDA(x,x))` is `#CALC!`, f7e94a817e5385ac A2186); an array match_mode lifts beside an array pad_with
(`TOCOL(Data!A11:D12)` gives 8 rows of `#VALUE!`, 2d9b48c6363e7a16 A3261); a 1x3 text against an 11x2 ignore_empty broadcasts with `#N/A`
past the shorter one (column 3 of bc4a69b0b512bc04 A538:C548); and a delimiter that is an error beats a text ignore_empty
(`VLOOKUP(..)` `#N/A` against `"a"`, c36ff5f42a1a4d47 A1409).
TRANSPOSE of a single value is that value, not a 1x1 array (T-2414, probes/t2414-gen-m7-unexplained, excel-graph, 2026-10-23): `COLUMNS(TEXTSPLIT(TRANSPOSE("a,b"),","))`
and `COLUMNS(TEXTSPLIT(TRANSPOSE(Data!G1),","))` (G1 = "a,b") are 2, so the text does not lift to one element as `{"a,b"}` does
(`COLUMNS(TEXTSPLIT({"a,b"},","))` is 1); `ISREF(TRANSPOSE(Data!G1))` is FALSE; consistent with f7e94a817e5385ac B2882.
A one-cell range `TRANSPOSE(B2:B2)` is the value too, while a 1x1 array stays an array, a constant or computed:
`COLUMNS(TEXTSPLIT(TRANSPOSE({"a,b"}),","))`, `TRANSPOSE(IF(TRUE,{"a,b"}))`, `TRANSPOSE(SEQUENCE(1)&"a,b")` and
`TRANSPOSE(UNIQUE({"a,b"}))` split to 1 column (probes/t2455-transpose-single-value A12:A16, A70:A72, excel-graph, 2026-10-23); a numeric argument follows from F.TEST
(`FTEST(TRANSPOSE(5),{1;2;3})` is the one-value `#DIV/0!`, A10, statistics.md "F.TEST of a single value") and from UNIQUE
(`UNIQUE(1,TRANSPOSE("1"))` is `#VALUE!` like `UNIQUE(1,"1")` and `UNIQUE(1,{"1"})`, A1:A3).
A blank cell transposes to a blank value: `ISBLANK(TRANSPOSE(B10))` is FALSE and `FTEST(TRANSPOSE(B10),{1;2;3})` is the
one-value `#DIV/0!`, not the blank-cell `#VALUE!` (A92, A93).
Ours: since T-2455 (`fn_transpose`: a 1x1 result of an argument that did not evaluate to an array is its element).
Unprobed: an array pad_with against an error element of a delimiter list (ours: the pad_with `#VALUE!`), and a one-cell reference to an
error cell as the delimiter, which ours reads as a list, so `TEXTSPLIT("abc",B1,,1/0)` with B1 `#N/A` is ours `#DIV/0!`.
Ours: this order and the `#CALC!` since T-2315 (`fn_textsplit`, `textsplit_element` in functions/text/textsplit.rs).
Status: VERIFIED (excel-graph)
Tasks: T-2315


## TEXTSPLIT: column delimiters first, match_mode 0 or 1
Rule: TEXTSPLIT scans the text once, left to right, for both delimiter lists: the earliest match ends a piece, and a row delimiter
also ends the row. At one position a column delimiter beats a row delimiter whatever their lengths (within one list the longest
wins, see "TEXTSPLIT, TEXTBEFORE, TEXTAFTER and REGEX* lift their text and scalar arguments"). So it is not a split into rows and
then of each row into columns: `TEXTSPLIT("abcd","bc","cd")` is {"a","d"} (the column match at 1 consumes the c of "cd"), while
`TEXTSPLIT("abcd","cd","bc")` is {"a";"d"}; `TEXTSPLIT("a*","a","a*")` is {"","*"}, `TEXTSPLIT("ab|c","b","b|")` {"a","|c"},
`TEXTSPLIT("a*","a*","a")` {"",""}, `TEXTSPLIT("a1b2c","1","b")` {"a","";"2c",#N/A} and `TEXTSPLIT("aXbc","b","aXb")` {"";"c"}.
match_mode is cut toward zero and must then be 0 or 1, else `#VALUE!` (as in TEXTBEFORE): 2, -1 and 255 are `#VALUE!`, 0.5 and -0.5 are
0, 1.5 and TRUE are 1. That check comes after an error element of a delimiter list (`TEXTSPLIT("aBc",{"b",#N/A},,,2)` is `#N/A`);
beside an invalid ignore_empty, an array pad_with, an empty delimiter or empty text the result is `#VALUE!` either way (A24-A28).
Evidence: probes/t2581-textsplit-textbefore A1-A29 (excel-graph, 2026-10-24). The gen-m9 cells it explains (T-2436 probe):
`TEXTSPLIT("a*","a",Data!A6:B12,FALSE,0)` is the 1x2 {"","*"} (3e98b26cf6713e77 A383, "a*" is also a row delimiter) and
`TEXTSPLIT(2,"a",Data!B5:D6,2.5,2,"a*")` `#VALUE!` (984cba67bbe6972b A160, match_mode 2).
Ours: since T-2581 (`split_grid`, `textsplit_element` in functions/text/textsplit.rs).
Status: VERIFIED (excel-graph)
Tasks: T-2581

## ARRAYTOTEXT error before format
An error value as ARRAYTOTEXT's first argument is the result before the format is looked at: `ARRAYTOTEXT(NA(),2)`
is `#N/A`, `ARRAYTOTEXT(1/0,2)` `#DIV/0!`, `ARRAYTOTEXT(SINH(710),2)` `#NUM!`, `ARRAYTOTEXT(NA(),"x")` `#N/A`, and a
reference to an error cell (also `B1:B1`) the same. With two errors the first argument's wins (`ARRAYTOTEXT(NA(),1/0)`
`#N/A`, `ARRAYTOTEXT(1/0,NA())` `#DIV/0!`); a valid first argument with an error format gives the format's error
(`ARRAYTOTEXT(1,NA())` `#N/A`). An error inside an array is not the argument's error: `ARRAYTOTEXT({1,#N/A},2)` is
`#VALUE!`, as `ARRAYTOTEXT(1,2)` and `ARRAYTOTEXT(1,"x")` are. VERIFIED (probes/arraytotext-error-vs-format, excel-graph
2026-10-27, T-2250). VALUETOTEXT differs: `VALUETOTEXT(NA(),2)` is `#VALUE!`, not the argument's error (matches today's
engine, unprobed beyond that cell). UNPROBED: a multi-cell range of only errors and a 1x1 array (`{#N/A}`) with format 2 (the engine gives `#VALUE!`); a reference to an error cell with an error or text format, such as `ARRAYTOTEXT(B1,NA())` or `ARRAYTOTEXT(B1,"x")` with B1 holding `#N/A` (the engine reads the format first and returns its error).

See also: RIGHT num_chars numeric-text coercion is recorded in criteria.md "Zero and negative SUMIFS criteria; IF error arguments; RIGHT numeric-text num_chars" (T-2980).

## CHAR lifts over ranges and arrays

`CHAR` takes a range or array argument and returns an array, like `UNICHAR`, `CODE` and
`UNICODE`; inside `CONCAT`, `TEXTJOIN` (as text or delimiter) and `CONCAT`'s mixed arguments the
elements are joined whole (`=CONCAT(CHAR(A1:A3))` is "Hi!" for 72, 105, 33), not implicitly
intersected. An error element (`CHAR(0)` is `#VALUE!`, a `#N/A` cell) is the join's result.
Evidence: probes/t3302-text-funcs-arrays (E1-E20, oracle-filled). The probe shows
`CONCAT`/`TEXTJOIN` over `CHAR`, `UNICHAR`, `CODE`, `UNICODE` ranges and computed arrays already
agree; the only gap was `CHAR`, fixed in T-3302.
