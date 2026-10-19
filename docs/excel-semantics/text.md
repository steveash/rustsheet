# Text

## Broadcast text output cap
Rule: a text function broadcast over arrays (LEFT, REPT, SUBSTITUTE, ...) returns a single scalar `#VALUE!` (no spill) once the total bytes of its string results exceed 64 MiB, to bound allocation on untrusted input (CLAUDE.md directive 5). A broadcast whose shape is above 1,048,576 cells is refused with `#ERROR!` before any element is computed, so that bound takes precedence over this one (arrays.md "Array results above 1,048,576 cells", T-1771). Excel's behavior (expected, not measured): compute the whole array, limited only by memory and the 32,767-char per-cell limit. Ours differs on legitimate data too: the budget counts UTF-8 bytes (not Excel's UTF-16 characters, so CJK text hits it about 3x sooner in characters), and a full-column text formula such as `=UPPER(A:A)` over ~65-char strings exceeds 64 MiB and returns `#VALUE!` where Excel spills normally.
Evidence: none for Excel's behavior; this is a resource limit, not an Excel observation.
Status: DEVIATION (decision 0014)
Implemented in: functions/text/util.rs (`MAX_BROADCAST_TEXT_BYTES`, `within_text_budget`), checked by functions/text/common.rs (`broadcast_text`), the lift driver and FORMULATEXT over a range

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
Matching is case-sensitive (`SUBSTITUTE("AaA","aa","x",2)` "AaA"); an instance past the last match leaves the
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
UNVERIFIED (T-1272 guesses, nothing probed; T-1824 probes them): when two delimiters of a list match at the same
position, the first in list order sets the match length; every entry longer than the text gives the single-delimiter
error (`#VALUE!` with more than two arguments, `#N/A` with two); a REGEX pattern that does not compile is `#VALUE!` in its own element
only; if_not_found, pad_with, match_end and return_all lift like the other scalars, and if_not_found is evaluated even
when the delimiter is found; REGEXEXTRACT's return_all keeps the first value of a lifted element. A 1x1 array lifts too, so
`COLUMNS(TEXTSPLIT({"a,b"},","))`, `COLUMNS(TEXTSPLIT(FILTER(A1:A2,A1:A2="a,b"),","))` and
`COLUMNS(REGEXEXTRACT({"a1b2"},"[0-9]",1))` are 1 (2 before T-1272), by analogy with `COLUMNS(XLOOKUP({2},A1:A4,C1:E4))`
= 1 (lookup.md); a common pattern, so T-1824 probes it first.
Ours: all of this since T-1272. The work caps are the next entry's. A multi-cell range as the TEXTBEFORE/TEXTAFTER
delimiter is still read as one text, as before T-1272 (gen-m7/851eb2099ae3f805 Calc!A25 is `#VALUE!`); its list rule is
T-1361's.
Status: VERIFIED (excel-graph) for the cited cells; UNVERIFIED (probe pending, T-1824) for the rest
Tasks: T-1272, T-1824 (probe the guesses), T-1361 (delimiter ranges)

## Lifted TEXTSPLIT, TEXTBEFORE, TEXTAFTER and REGEX* calls are capped
Rule: Excel computes these calls whatever their size.
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
state), the PikeVM its span times its NFA's state count (5 a state-byte) before it runs, 64 a search, and a call may
spend 2^30 (about 1 s of the slowest searches; `SUM(--REGEXTEST(REPT("y",200)&SEQUENCE(100000),"x"))` fits, 20 MB).
A pattern whose DFA thrashes over a lot of text (`a[ab]{100}c|b[ab]{100}c` over 560 texts of 30,000 bytes) and a call
past any of these limits is `#ERROR!` (`ROWS(REGEXTEST("a","a"&SEQUENCE(5000)))`). Iterating matches on the PikeVM
charges every successive search the rest of the text, and a pattern with `\b` runs on the PikeVM over non-ASCII
text (the lazy DFA quits there), so `COLUMNS(REGEXEXTRACT(REPT("naïve ",500),"\b\w+\b",1))` is `#ERROR!` where
Excel returns an array (T-1829 charges it by the work done). A REGEXREPLACE searches twice,
once to size its result: over 65,535 UTF-16 units it is
`#CALC!` as Excel's is, its length known before it is built (`LEN(REGEXREPLACE(REPT("a",32767),"","bbbbbbbbbb"))`;
"REGEXREPLACE results are capped at 65,535 UTF-16 units" below); a lifted REGEXREPLACE whose results pass the text
broadcast cap (64 MiB, first entry) is one `#VALUE!`.
The REGEX* dialect is the regex crate's on every target (wasm32 used `regex_lite`, whose `\w`, `\d` and `\s` are
ASCII and which rejects `\pL`, before T-1272): `\w` and `\b` are Unicode (`REGEXTEST("é","\w")` is TRUE), as
Excel's are (T-1817, below); the POSIX classes are not (T-1903).
The REGEXREPLACE replacement syntax is Excel's since T-1688: see "REGEXREPLACE replacement syntax (T-1688)"; the
pattern dialect still differs from Excel's PCRE2: see "REGEX* patterns and match iteration are PCRE2 (T-1688)".
Evidence: none needed for Excel's side (it has no such limit); CLAUDE.md directive 5.
Status: DEVIATION (decision 0014)
Implemented in: `Model::lift_text_call`, `text_call_cost` and `ScanCost` (functions/text/lifted.rs), `PatternCache`
and `Replacement` (functions/text/regex_functions.rs), `Engine` and `Search` (functions/text/regex_engine.rs), `SplitList` (functions/text/textsplit.rs)
Tasks: T-1272, T-1829

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
instance numbers behave the same on `"İİİ,a,b"`. Sigma is positional, unlike TEXTSPLIT and SEARCH: `Σ` followed by a letter is `σ`, any
other `Σ` is `ς`, and `ς` is only `ς`. So the delimiter `Σ` alone matches a text `ς` but not `σ`, the delimiter `ς` matches a final `Σ`
("aΟΣ") but neither `σ` nor a `Σ` followed by a letter, and the delimiter `σ` matches a `Σ` followed by a letter but not `ς`.
Evidence: probes/textbefore-casefold-length A1-A25 (excel-graph, 2026-10-17, T-1796).
Ours: `fold_for_before_after` (text/casefold.rs) folds char by char with `fold_for_split`'s İ/Kelvin exceptions plus the sigma and `ẞ` rules,
and `match_text` (util.rs) maps the match back to byte offsets of the original with `str::get`. Before T-1796 it searched `to_lowercase()` of the text
and sliced the original at those offsets, which panicked on `TEXTBEFORE("İİİ,a",",",1,1)`. UNVERIFIED: a delimiter with the same `Σ` as the text, which our separate folds miss (`TEXTBEFORE("aΣb","Σ",1,1)` is `#N/A` here: the text's `Σ` is `σ`, the lone delimiter `Σ` is `ς`), and a `Σ` followed by a space, digit or punctuation (`"ΟΔΟΣ ΚΑΙ"`; only a `Σ` at the end of the text is probed, A25), both T-1799; the exact sigma context rule
(whether a `Σ` after a non-letter, or before a combining mark, is final; the code ignores what precedes it), the folding of other chars (Å U+212B, ſ, µ,
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
Implemented in: `fold_case` (functions/text/casefold.rs).
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

## DOLLAR and FIXED decimals
Rule: `decimals` is truncated toward zero, not floored: `DOLLAR(1234.5,-1.5)` is "$1,230" (decimals -1), `FIXED(1234.5,-0.5)`
is "1,235" (decimals 0), `DOLLAR(1234.5,1.9)` is "$1,234.5". A `decimals` of 128 or more (also 128.5) is `#VALUE!`;
127 is fine and prints 127 fraction digits; large negatives just give "0" (`FIXED(1,-400)`, `DOLLAR(1,-129)`).
`FIXED(1234,0,"x")` is `#VALUE!`; a numeric no_commas is a boolean (`FIXED(1234,0,2)` = "1234").
Evidence: probes/dollar-fixed-unichar-edges A1-A29, A46-A50, D69-D75 (excel-graph, T-1281).
FIXED of a value that rounds to zero drops the sign (`FIXED(-0.004,2)` "0.00", `FIXED(-0.4,0)` "0"; DOLLAR keeps
parentheses: `DOLLAR(-0.004)` "($0.00)"). Implemented (T-1257), see "FIXED drops the sign of a value that rounds to zero".
Status: VERIFIED (excel-graph); desktop confirmation pending (T-1081)

## UNICHAR code point bounds
Rule: UNICHAR truncates toward zero and lifts over arrays (`UNICHAR({65,66})` spills "A","B"). Below 1 (0, -1, 0.5) or
above 1114111 (U+10FFFF) is `#VALUE!`. A surrogate (55296-57343) or a Unicode noncharacter is `#N/A`: U+FDD0-U+FDEF
(64976-65007) and the last two code points of every plane (65534, 65535, 131070, 131071, 196607, ..., 1114110,
1114111). The neighbours are fine: 55295, 57344, 64975, 65008, 65533, 65536. A code point above 65535 is a
surrogate pair (`LEN(UNICHAR(65536))` is 2). A fractional value is truncated first (`UNICHAR(55296.5)` is `#N/A`).
Evidence: probes/dollar-fixed-unichar-edges A30-A45, A51-A53, D54-D68 (excel-graph, T-1281).
Status: VERIFIED (excel-graph); desktop confirmation pending (T-1081)

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
Rule: `case_sensitivity` 1 is case-insensitive and 0 (or omitted) case-sensitive for both functions (`REGEXTEST("Hello World 123","hello",1)` TRUE, `,0` FALSE; REGEXREPLACE's fifth argument likewise, `"O"` with 0,1 replaces every o). REGEXREPLACE occurrence: 0 or omitted replaces all, n > 0 the n-th match, n < 0 counts from the end (-1 the last), an out-of-range occurrence (3 of 2 matches, 9, -9) returns the text unchanged; 1.9 truncates to 1; "x" is `#VALUE!`. The replacement reads `$0`, `$1`, `$$` (a literal `$`), `${name}` and also `\1`; a backreference in the pattern (`(\w)\1`) works. An empty pattern or `x*` matches between every character and at both ends (`-H-e-l-l-o-...-3-`; an empty text gives "-"); `^` and `$` anchor the text. Lazy empty matches are not skipped: `a*?` on "aaa" with "b" is "bbbbbbb" (PCRE2-style global replace: after an empty match a non-empty match is tried again at the same position, so each "a" is consumed too, unlike a Rust-regex iterator's "bababab"). `\p{L}` and `.` match each character of BMP text ("日本語" is 3); surrogate pairs are UNVERIFIED (not probed). Numbers and booleans are read as text; an error argument is returned. A range argument lifts and spills (`#SPILL!` when blocked). Invalid patterns and a missing group (`$3`) give an error type the oracle does not decode yet; those cells stay pending, as do REGEXTEST case_sensitivity 2 and -1 (C23, C24); UNVERIFIED.
Evidence: probes/t1626-regextest, t1626-regexreplace (excel-graph, 2026-10-16).
Ours: #ERROR! for the optional arguments (T-1360); `\1` in the replacement matches since T-1688; the pattern
backreference (REGEXTEST C29) is T-1900 and the lazy empty matches (REGEXREPLACE C34) T-1899.
Status: VERIFIED (excel-graph); desktop confirmation pending (T-1081)
Tasks: T-1360, T-1688, T-1272 (range lift)

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
LOWER (`\Uß` is `ß`). Final sigma (T-1934, VERIFIED, probes/regexreplace-final-sigma, excel-graph 2026-10-19): a `\L` run is lowered whole, so a capital sigma sees the text after it in the next piece (`\L$1x` with `$1` = ΑΣ is `ασx`, but `\L$1` alone is `ας`; A2, A4, A8); a `\E` ends the run (`\L$1\E$2` with `$1`, `$2` = Α, Σ keeps `Σ`: `αΣ`, A10); a `\l` head character is lowered alone (`\lΣa` is `ςa`, A1). `\Q...\E` quotes up to
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
Ours: matches. Implemented in `Replacement::new`, `CaseState` (functions/text/regex_functions.rs); the size bound
`Replacement::result_len` allows case mapping to lengthen text by half.
Status: VERIFIED (excel-graph)
Tasks: T-1688, T-1897

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
ERROR.TYPE 3; with `x*` too, as an undecoded error), where ours puts `-` around the emoji.
Evidence: probes/regex-pcre2-features C1-C19, D1-D14, B27-B29 (excel-graph, 2026-10-18).
Ours: the `regex` crate's dialect: it rejects backreferences, lookaround, atomic groups, `\K`, conditionals and
comments as invalid (`#VALUE!`), reads `a++` as `a+` followed by `+` (so `a++a` matches `aaa`, C11, B29), and
iterates as Rust does (`bababab`). The matcher is T-1898
(spec), T-1899 (iteration) and T-1900 (pattern features).
Status: VERIFIED (excel-graph)
Tasks: T-1688, T-1898, T-1899, T-1900

## CLEAN character set and PROPER word starts and case mapping (T-1307)
Rule (CLEAN): code points 0-31 and the C1 controls 128-159 (`CHAR(129)`, 141, 143, 144, 157 and `UNICHAR(133)` included) are removed; 32, DEL (127), NBSP (160), the soft hyphen (173), U+200B, U+2028 and U+FEFF stay. TAB, LF and CR go (`CLEAN("a"&CHAR(10)&"b"&CHAR(1)&"c")` is "abc"). The engine already matched all 38 CLEAN cells.
Rule (PROPER; UPPER, LOWER and further characters in the T-1747 section below): a letter after any non-letter, digits, apostrophes (straight or curly), `_` and `-` included, is upper-cased and the other letters lowered (`2Nd Place`, `O'Neil Mc-Donald`, `It'S 3Rd`, `A1B2C3`, `Hello_World`; full-width letters follow the same rule). Case mapping is per character and one-to-one: the characters whose full mapping expands stay as they are (`PROPER("ßa straße")` is `ßa Straße`, ŉ, ǰ and the ﬁ/ﬂ ligatures unchanged), where Rust's `to_uppercase` gives "SS", "ʼN" and "FI". İ folds to ASCII (`PROPER("İstanbul")` is `Istanbul`, `PROPER("aİb")` `Aib`) and ΐ, ΰ upper-case to Ϊ, Ϋ. Titlecase digraphs map to their capital (`ǅ ǈ ǋ ǲ` give `Ǆ Ǉ Ǌ Ǳ`). A capital sigma ending a word lowers to the final ς, with no letter or digit after it (`ΣΊΣΥΦΟΣ` is `Σίσυφος`, `ΑΣ.ΑΣ-Σ ΑΣ1` is `Ας.Ας-Σ Ασ1`, so a following digit keeps σ and a word-initial Σ stays). A combining mark is not a letter, so the next letter is capitalised (`e` + U+0301 + `cole` is `ÉCole`), and a zero-width space (U+200B) is not a letter either (`PROPER(U+200B&"abc def")` is `<U+200B>Abc Def`).
Evidence: probes/clean-proper-chars (excel-graph, 2026-10-16, 88 cells; engine matches all after T-1307). Implemented in `proper_case` (functions/text/string_format.rs).
Status: VERIFIED (excel-graph)
Tasks: T-1307

## UPPER, LOWER and PROPER: one-to-one case mapping, final-sigma context (T-1747)
Rule: UPPER, LOWER and PROPER all use the simple one-to-one mapping, so characters whose full mapping expands stay unchanged in every case (ᾳ, ẖ, ẘ, ﬀ, ﬃ, ﬆ, և, ß, ŉ, ǰ, ﬁ; `UPPER("ß")` is `ß`, not "SS"). ΐ upper-cases to Ϊ in UPPER and PROPER (LOWER leaves it; ΰ to Ϋ in UPPER is unprobed, carried over from PROPER). İ is the exception to consistency: PROPER and LOWER fold it to I / i (`LOWER("aİb İc")` is `aib ic`) but UPPER leaves it as İ. A capital Σ lowers to the final ς in LOWER and PROPER when the next character is the end of the text, a space, a straight apostrophe or `-`, and to σ when a letter or digit, a combining mark (probed: U+0301, U+0308; the rest of U+0300-U+036F is assumed, other combining blocks and U+200C/U+200D/U+2018 are unprobed and give ς), a curly apostrophe (U+2019) or a zero-width space (U+200B) follows. In LOWER a lone `Σ`, `Σ ` and `1Σ` give ς (no preceding letter needed), but `Σ`+U+0308 gives σ̈; lower-case σ is never changed (`LOWER("ασ")` is `ασ`). PROPER keeps a word-initial Σ capital.
Evidence: probes/case-mapping-expanding (excel-graph, 2026-10-17, 81 cells; engine matches all).
Status: VERIFIED (excel-graph). Implemented in: `lower_case`, `upper_case`, `proper_case` (functions/text/string_format.rs). Tasks: T-1747

## ARRAYTOTEXT: a single empty cell is #VALUE!, an empty cell in a larger range is empty text
Rule: `ARRAYTOTEXT` of a reference to one empty cell is `#VALUE!` in both formats, and so are a 1x1 range of an empty
cell, `(B1)`, `IF(TRUE,B1)`, `INDEX(B1:B2,1)` of an empty cell and an omitted first argument (`ARRAYTOTEXT(,1)`). Within a
multi-cell range an empty cell is empty text (`B1:B2` all empty is ", ", strict "{;}"; `C1:C3` with C2 empty is "1, , 3",
strict "{1;;3}"). Computed empties are not references: `B1+0` is "0" and `B1&""` is "". `VALUETOTEXT(B1)` of an empty
cell is "" in both formats, and a typed `""` is "" / `{""}`. An array element is not a reference: GROUPBY hands a one-blank group to
its function as an array and `ARRAYTOTEXT` of it is "" (probes/groupby-blank-values).
Evidence: probes/arraytotext-empty-ref A1-A25 (excel-graph, 2026-10-17, T-1718).
Status: VERIFIED (excel-graph). Implemented in: `fn_arraytotext`. Tasks: T-1718

## VALUE lifts over a range or an array (T-1791)
Rule: VALUE lifts over a multi-cell range or an array, each element read as a scalar argument: text converts as in scalar VALUE (`"1 "` is 1, `" $2 "` 2, `"1/2/2020"` 43832, `" 3%"` 0.03), a number stays, a blank cell is 0, a boolean or `""` is `#VALUE!` and an error stays (`VALUE(A1:A8)` spills 8 rows). So `{=SUM(IFERROR(VALUE(LEFT(range,FIND(...)-1)),0))}` adds the numbers in front of the found text (corpus-bench/42181 B14 is 3).
Not probed: a multi-cell range in a legacy (non-dynamic-array) cell, which takes the implicit intersection of the other lifted text functions.
Evidence: probes/value-argument-lift (excel-graph, 2026-10-18); corpus-bench/42181 B14 (Excel cached value).
Implemented in: `fn_value` (functions/text/common.rs), through `text_arg` and `broadcast_text`.
Status: VERIFIED (excel-graph)
Tasks: T-1791

## REGEXREPLACE results are capped at 65,535 UTF-16 units, not 32,767; replacement syntax and Unicode classes (T-1817)
Rule: a REGEXREPLACE result may be up to 65,535 UTF-16 units; one unit more is `#CALC!`. `LEN(REGEXREPLACE(REPT("a",32767),"a","bb"))` is 65534; with `REPT("a",N)&REPT("c",M)` and `"a"` → `"bbb"`, 65,535 units is 65535 and 65,536 or 65,537 is `#CALC!`; the cap counts UTF-16 units, not characters (`"a"` → `"b"&UNICHAR(128512)`: 65,535 units in 49,151 characters is 65535, 65,536 units in 49,150 characters `#CALC!`), and not matches (32,767 matches giving 65,534 units pass). 98,301 to 327,670 units (`REPT("a",32767)`, `"a"` → 3 to 10 b's) and the empty pattern with `bbbbbbbbbb` (360,447) are `#CALC!`. Lifted, the cap is per element: `REGEXREPLACE(REPT("a",{1,32767}),"","bbbbbbbbbb")` is `"bbbbbbbbbbabbbbbbbbbb"` and `#CALC!` (probes/regexreplace-result-cap-lifted). A cell holds such a result: `=REGEXREPLACE(REPT("a",32767),"a","bb")` is the 65,534 b's, `LEN` of the cell 65534, `ISERROR` FALSE.
Other functions on a result over 32,767 units: `LEN`, `LEFT`, `MID(..,40000,10)` (10), `ISTEXT` (TRUE), `ISERR`/`ISERROR` (FALSE), `IFERROR(LEN(..),-1)` (65534), a nested `REGEXREPLACE`, `REGEXEXTRACT` and `REGEXTEST(..,"^b{65534}$")` (TRUE) take it as text; `SUBSTITUTE(..,"b","c")` and `UPPER(..)` are `#VALUE!` (ours 65534; T-1904). `TYPE` is 64 and `ERROR.TYPE` 3, as for any text over 255 units (information.md "TYPE and ERROR.TYPE of text over 255 UTF-16 units").
Replacement syntax: `$1$1` and `${1}x` on `(a)` give `aab` and `axb`, `$0$0` gives `aab`; named group `${x}$x` gives `aab`. A bare `$`, `a$` and `$|$$` forms are rich-value errors the oracle does not decode yet (A12, A17, A18 pending).
Unicode: `\w`, `\pL`, `\p{L}`, `\b` and the POSIX class `[[:alpha:]]` all treat é as a letter (`REGEXTEST("é","[[:alpha:]]")` is TRUE; ours FALSE, the regex crate's POSIX classes are ASCII; T-1903); `REGEXEXTRACT("naïve","\w+")` is "naïve".
Probing note: excel-graph refuses to open (OpenWorkbookBlockedWorkbook, "exceeds the size limit") a probe workbook that builds more than two or so 65,534-unit texts, or a `REPT("b",32767)` replacement matched twice, so the long-result questions are split over probes/regexreplace-long-result-a to -i; `LEN(REGEXREPLACE(REPT("a",32767),"a","bb")&"")` could not be opened even alone (unprobed).
Evidence: probes/regexreplace-limit-and-syntax, probes/regexreplace-result-cap, probes/regexreplace-result-cap-lifted, probes/regexreplace-long-result-a to -g and -i (excel-graph, 2026-10-18).
Implemented in: `MAX_REPLACED_UNITS` / `over_cap` in functions/text/regex_functions.rs, checked on the measured length before the result is built.
Status: VERIFIED (excel-graph)
Tasks: T-1817
