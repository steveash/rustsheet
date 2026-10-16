# Coercion and argument handling

## AND / OR / XOR evaluate every argument; the first error wins
Rule: AND/OR do not short-circuit: `=OR(TRUE,(2<COUNTIFS(…shape mismatch…)))` → `#VALUE!`.
Evidence: gen-m2/78bc88199fc96650 Calc!A217; OR: 1b6ceed7489e0800 A143, 893302d6c5b99e2e A170, dd99be1f2454159f A3; AND: 75840b9c71a50596 A56, 96a0952412cefa6c A126. Oracle: excel-graph, 2026-10-04 (gen-m2 sidecars, T-0203).
Which error: the first one, taking arguments left to right and each range or array row by row, for AND, OR and XOR
alike, whatever value decides the result: `OR(TRUE,NA(),1/0)` → `#N/A`, `OR(TRUE,1/0,NA())` → `#DIV/0!`,
`AND(FALSE,1/0,NA())` → `#DIV/0!`; with D1:F2 = TRUE, `#DIV/0!`, FALSE / `#N/A`, TRUE, `#NUM!`, `OR(D1:F2)`,
`AND(D1:F2)` and `XOR(D1:F2)` are `#DIV/0!` (row-major first) and `OR(TRANSPOSE(D1:F2))` is `#N/A`;
`OR({TRUE,#DIV/0!,#N/A})` → `#DIV/0!`; `OR(TRUE,B1:B2)` with an error in B1:B2 is that error; text before an error
in a range neither wins nor makes `#VALUE!` (`OR(H1:H2)` with H1 = "x", H2 = `#N/A` → `#N/A`; a text-only range and
direct text arguments are not covered here); an intersection `OR(D1:F2 E1:F2)` reads its own cells.
Evidence: probes/and-or-xor-error-order (30 cells; oracle: excel-graph, 2026-10-08, T-0229).
Status: VERIFIED
Implemented in: `logical_nary` (functions/logical/and_or_xor_not.rs; unit test `first_error_wins_row_by_row` in test/test_fn_or_xor.rs).
Tasks: T-0215, T-0229

## MIN / MAX coerce value arguments like SUM; ranges and arrays give only their numbers
Rule: a value argument (literal or function result, e.g. IFERROR or CONCATENATE) is coerced like
SUM's: non-numeric text is `#VALUE!` (`=MIN("a")`, `=MAX(CONCATENATE("x"))`), numeric text counts
(`=MIN("2")` → 2, `=MAX("1e2",5)` → 100), TRUE/FALSE count as 1/0 (`=MIN(TRUE,5)` → 1), and an empty
argument counts as 0 (`=MIN(5,)` → 0, `=MAX(-5,)` → 0). A reference, also one returned by IF/INDEX
(references.md), and an array contribute only their numbers: `=MIN(A1)` with A1 = "a" → 0,
`=MAX(IF(TRUE,A1),A2)` → A2, `=MIN({TRUE,5})` → 5, `=MAX({"9",-1})` → -1. IFERROR returns a value,
so `=MIN(IFERROR(A1,1))` with A1 = "a" is `#VALUE!` and an IFERROR of a blank cell is 0.
Evidence: probes/min-max-value-args (C1–C16); gen-m2/f7e6d351c8538ef5 Calc!A14,
dd99be1f2454159f A73/A264, 2cfe279fe77a000d A138, 04cfd82f54dbc2b2 A182.
Oracle: excel-graph, 2026-10-05 (probe), 2026-10-04 (gen-m2 sidecars, T-0203).
Status: VERIFIED
Implemented in: functions/math_and_trigonometry/mathematical.rs (`fold_min_max`)
Tasks: T-0217

## AVERAGE, MEDIAN, GEOMEAN, HARMEAN, SKEW, KURT, STDEV/VAR, COUNT value arguments
Rule: these coerce like MIN/MAX above: booleans inside an array are ignored (`=AVERAGE({TRUE,5})` → 5,
`=MEDIAN({TRUE,1,3})` → 2, `=GEOMEAN({TRUE,4})` → 4, `=STDEV({TRUE,1,3})` → 1.414...), a direct boolean counts
as 1/0 (`=AVERAGE(5,TRUE)` → 3), text inside an array is ignored, and an empty argument counts as 0
(`=AVERAGE(5,)` → 2.5, `=MEDIAN(1,)` → 0.5, `=GEOMEAN(4,)` → `#NUM!`, `=STDEV(1,3,)` → 1.5275..., VAR/VARP/STDEVP likewise,
`=COUNT(5,)` → 2, `=COUNT({TRUE,5})` → 1, HARMEAN/SKEW/SKEW.P/KURT likewise (`=HARMEAN({TRUE,4})` → 4, `=HARMEAN(4,)` → `#NUM!`, `=SKEW(1,2,4,)` → 0.7528...), `=AVERAGE(A1:A2,)` with A1 = TRUE, A2 = 5 → 2.5).
Evidence: probes/average-value-args F1–F32 (oracle: excel-graph, 2026-10-05)
Status: VERIFIED
Implemented in: `for_each_value`, `fn_count` (also HARMEAN/SKEW/SKEW.P/KURT via `for_each_value`) in functions/statistical/count_and_average.rs; `fn_stdev_s/p`, `fn_var_s/p` (standard_dev.rs, variance.rs)
Tasks: T-0236

## LEFT / RIGHT / MID numeric arguments
Rule: booleans coerce (TRUE = 1: `MID(…,TRUE,5)`); a start or length around 1E+15 is `#VALUE!`
(`MID("ü",1000000000000000.0,4)`); in LEFT a length a hair below 1 (`TRUE - 1E-15`) counts as 1
(seen for LEFT only, and the cell cannot tell whether the subtraction or LEFT rounds: probe in T-0219).
Numeric text coerces too, literal or from a cell, and is then truncated like a number:
`LEFT("a string","6")` is "a stri", `LEFT(E12,E17)` with E17 = "2.99999" (text) is "no",
`LEFT("a string",TRUE)` is "a"; RIGHT the same ("string", "ce", "g").
Evidence: gen-m2/ef5c55a81783b7d5 A40, d0c29dd5c530e28c A269, 78bc88199fc96650 A345,
b4392a6514a36ade A14 (Calc sheet). Oracle: excel-graph, 2026-10-04 (gen-m2 sidecars, T-0203).
poi/FormulaEvalTestData_Copy.xlsx (Excel-cached, Excel 15) EverythingTests!E824, H824, K824 (LEFT),
E1168, H1168, K1168 (RIGHT).
Status: VERIFIED
Implemented in: `Model::array_node_to_number` (cast.rs), used by `fn_left`/`fn_right`/`fn_mid` (T-0257).
Tasks: T-0219 (huge and near-integer counts), T-0257 (numeric text and booleans)

## Text comparison uses Excel's collation, not code-point order
Rule: `<`, `>`, `>=` between texts follow Excel's collation: "x" >= "ü" is TRUE, "1.5" > "?b" is TRUE
(code-point order says otherwise for both: "x" < "ü" and "1" < "?").
Evidence: gen-m2/2cfe279fe77a000d Calc!A318, 96a0952412cefa6c Calc!A284. Oracle: excel-graph, 2026-10-04 (gen-m2 sidecars, T-0203).
Status: VERIFIED (two cells here; the full order is below, T-1469 probes/text-collation-order)
CHAR(160) and spaces (T-0244, VERIFIED, probes/text-order-nbsp-space, 333 cells, excel-graph
2026-10-09): texts compare character by character, with the space first, CHAR(160) right after it,
then digits and letters: `" 1" < CHAR(160)&"1" < "0" < "1" < "1"&CHAR(160)`, `" " < CHAR(160)`,
`"a b" < "a"&CHAR(160)&"b" < "ab"`, `CHAR(160)&CHAR(160)&"1" < CHAR(160)&"0"`; CHAR(160) is never
equal to a space or ignored (`CHAR(160)&"1"="1"` is FALSE). `>` and COUNTIF's `">"` agree on every
pair whose operand is text (a criterion operand like `" 0"` is numeric, criteria.md). In passing:
a hyphen is skipped, `"-1" > "0"` is TRUE (Data!A17 vs C3; implemented by T-0220).
Implemented in: `functions::util::collation::cmp_text_excel` (T-0220: the three passes below), used by
`compare_values` (operators including `=`, SWITCH, SORT, XMATCH, XLOOKUP) and the ordered text criteria (`<`, `<=`, `>`, `>=`) in
functions/criteria.rs. The `=`/`<>` text criteria use it too (T-1594). MATCH(...,0) and VLOOKUP(...,FALSE) do not use the collation
(T-1590, below). Inferred, not probed: SORT orders texts the same way; control characters sort below the space (see T-1590 for CHAR(9), 10, 13).
Full order (T-1469, VERIFIED, probes/text-collation-order, 146 strings ranked by
`SUMPRODUCT(--(range<x))`, excel-graph 2026-10-13). `=`, `<` and `>` ignore case completely (`"a"<"A"` and `"a">"A"` are both FALSE) but not accents; `<`/`>` order, lowest first:
1. The empty string, then a lone `'`, `''`, `-`, `--`, then space, `  `, then the punctuation
   ``! " # $ % & ( ) * , . / : ; ? @ [ \ ] ^ _ ` { | } ~``, then `+ < = >`, then digits `0 1 2 ... 9`, then letters.
2. Hyphen and apostrophe are ignored at the first pass and only break ties afterwards (Excel's "word sort"):
   `"ab" < "a'b" < "a-b"`, `"coop" < "co-op"` (but `"co op" < "coop"`: a space is not ignored), `"1" < "'1" < "-1" < "2"`,
   `"a" < "a'" < "a-" < "'a" < "-a" < "aa"`. `"-1" > "0"` is TRUE.
3. Case is ignored at every level (`"a" = "A"`, `"ab" = "AB"`); accents are a later tiebreak, not a different letter:
   `"a" < "á" < "à"="À" < "â" < "ä" < "å" < "aa"`; `"é"="É"` sits after `"E"` and before `"F"`; `"résumé" > "resume"`.
4. Expansions and multi-letter folds: `"æ"="ae"` and `"ß"="ss"` are equal (`=` TRUE); `"ø" > "o"` but `"ø"` sorts after `"ö"` and before `"P"`
   (observed position only; whether it is a separate letter or a heavier accent of `o` is not probed); `"ÿ"` sorts after `"Y"` and before `"Z"`.
Ranks for every string are in the oracle sidecar (Calc!A, ties in Calc!B).
Fitted key (T-0220; all 486 probe cells and the gen-m2 cells above pass): compare in three passes,
case folded throughout. (1) Letter weights: control characters, space, CHAR(160), the punctuation
in the order above, digits, letters; an accented Latin-1 letter weighs as its base letter, `æ` as
`a e`, `ß` as `s s`; hyphen and apostrophe weigh nothing. (2) Accents, letter by letter: none <
acute < grave < circumflex < diaeresis < ring, then tilde, cedilla and stroke (`ø`). Tilde and
cedilla are placed by Windows' sort-key order, UNVERIFIED. (3) Hyphens and apostrophes as (position,
weight), a later position first, apostrophe before hyphen. Characters outside ASCII and the Latin-1
letters (`þ`, `ð`, `œ`, Greek, symbols such as `§`) are UNVERIFIED: they keep code-point order after
the letters.
### Collation equality in lookups and criteria; unprobed characters (T-1590)
Evidence: probes/text-collation-lookups (66 formula cells) and probes/text-collation-unprobed (90 strings ranked by
`SUMPRODUCT(--(range<x))`), excel-graph 2026-10-15. Status: VERIFIED.
- Equality by collation (`"æ"="ae"`, `"ß"="ss"` TRUE; `"a-b"="ab"`, `"a"="á"`, `"ø"="o"`, `"co-op"="coop"` FALSE) applies in
  **XMATCH, XLOOKUP, SWITCH** (`XMATCH("ae","æ",0)` = 1, `SWITCH("ae","æ",...)` hits) and in **COUNTIF**'s text criteria (below).
  It does **not** apply in **MATCH(...,0)** and **VLOOKUP(...,FALSE)**: both give `#N/A` for `"ae"` against `"æ"` and `"ss"` against `"ß"`.
  (The engine already agrees on every lookup cell; the old "inferred: exact-match lookups treat æ and ae as equal" held for XMATCH, XLOOKUP and SWITCH only.)
- COUNTIF over {"æ","ae","a-e"}: criteria `"ae"`, `"=ae"`, `"æ"` count 2 (æ and ae); `"<>ae"` and `"<>æ"` 1 (a-e); `"<ae"` 0; `"<=ae"` and `"<=æ"` 2;
  `">ae"` 1; `">=ae"` 3; `"a-e"` 1; `"<>a-e"` 2. So the criteria use the same collation for `=`, `<>` and the ordered operators (see criteria.md). SUMIF, SUMIFS and COUNTIFS agree with COUNTIF (T-1597, criteria.md); wildcard `?`/`*` do not expand.
- Order of the characters not probed in T-1469 (lowest first, `=` marks equal; whole table in the oracle sidecar): empty, CHAR(1), CHAR(31), CHAR(127),
  `–` `—` (en and em dash behave like a hyphen: ignored, tiebreak only: `"a'b" < "a-b" < "a–b" < "a—b"`, all after `"ab"`),
  CHAR(160), CHAR(9), CHAR(10), CHAR(13) (these three sort above CHAR(160), not below the space), then
  `! ~ ¡ ¦ ¨ ¯ ´ ¸ ¿`, `‘ ’ “ ”` (`’` is an ordinary punctuation mark, not ignored: `"a’b" < "ab"`), `¢ £ ¤ ¥ €`, `+ ±`, `« »`, `× ÷`, `§ © ¬ ® ° µ`, `… •`,
  `0`, `¼ ½ ¾`, `¹ ² ³`, `9`, then letters: `a=A < ª < á < ä < ã < å`, `c < ç`, `d < đ < ð`, `i < ı < ij=ĳ`, `l < ł`, `n < ñ`, `o < º < ö < ø < œ`,
  `s`, `t < þ`, `u`, `z`, `α < β < ω`. Notes: `ĳ` equals `ij` (an expansion like `æ`/`ß`). `œ` sits between `ø` and `s`; whether it equals `oe` is not probed. `ã` sorts after `ä`.
  `ç`, `đ`, `ð`, `ł`, `ñ`, `þ` and the digit-like symbols were probed as single characters only (the symbols against `0` and `9` only); the choices made for them are in the next bullet.
- Fitted in `cmp_text_excel` (T-1595; all 270 cells pass), with these choices where the probe is single-character only (each UNVERIFIED):
  control characters other than CHAR(9)/CHAR(10)/CHAR(13) are skipped like the hyphen (tiebreak weight below the apostrophe, by code point), so
  `"a"&CHAR(1)&"b"` sorts just after `"ab"`; CHAR(11) and CHAR(12) sit with CHAR(9)-CHAR(13) above CHAR(160); the en and em dash are skipped
  everywhere, also alone (`"" < "–"` by tiebreak, below CHAR(160)); the Latin-1 marks, quotes and currency signs go between `~` and `+`, the
  symbols `± « » × ÷ § © ¬ ® ° µ … •` after `>`; `¼ ½ ¾` between `0` and `1` (by value), `¹ ² ³` as `1 2 3` with a tiebreak accent;
  `ª º`, `đ ł` (stroke, like `ø`) and `ð ı þ` are tiebreak accents on `a o d l i t`, not letters of their own; `œ` = `oe` like `æ`.
Tasks: T-0220, T-0244, T-1469, T-1590, T-1594, T-1595

## Double unary minus coerces to a number
Rule: `--x` is not a no-op. Each unary minus coerces its operand to a number: `--FALSE` = 0,
`--("">1.5)` = 1, `--""` (a cell holding empty text) = `#VALUE!`, `--100` = 100.
Evidence: gen-m1/6462bf910ef9f28f Calc!A49 (`--Data!C4`, C4 = FALSE), A84 (`--Data!$A11`, A11 = ""),
A17; gen-m1/57fe331cb7611dbf Calc!A167 (oracle: excel-graph, 2026-10-05, T-0109). Upstream IronCalc's
Excel-authored `TEXT/MINUS_MINUS.xlsx` (`ironcalc@9b1ae3fd`; 72 of its 73 cells pass with T-0132)
agrees: `--TRUE` = 1, `---TRUE` = -1, `--"3"` = 3, `--"abc"` = #VALUE!, `--A1:A4` lifts element-wise.
A plus sign is a no-op (`+x` stays text or boolean).
A single unary `-` or `%` also works element by element on a range or array: `INDEX(...,1,0)%`
= 0 (gen-m2/f9d4df4efdfbcbd5 A125, oracle excel-graph); the spill shapes asserted in
`test/test_unary_operators.rs` are upstream's, not yet oracle-checked.
Status: VERIFIED
Implemented in: `Parser::parse_power` keeps one `UnaryKind::Minus` per minus sign; `Model::apply_number_unary`
evaluates it (element-wise over ranges and arrays); `stringify`/`move_formula` print `--x` as typed and
parenthesise a lower-precedence operand (T-0132, `Ported-From: ironcalc@9b1ae3fd`). `--"12:00"`
(MINUS_MINUS.xlsx C12, Excel 0.5) still fails: time text coercion is T-0146, which adds that workbook.
Tasks: T-0132

## Text in a range operand of arithmetic is coerced
Rule: arithmetic on a range coerces a numeric text cell to a number, so `Data!D7:D7/1` with
D7 = " 2 " (spaces around the digit) is 2 and `Data!D3:D3/0` with the same text is `#DIV/0!`. Every
observed number-producing text cell is " 2 ". That this is the same routine as scalar text
coercion is an inference for T-0131's probe to confirm.
Evidence: gen-m1/1ce5b2467b3b19fd Calc!A14, A79, A99; gen-m1/9646257ac139cd1e Calc!A25, A81 (oracle:
excel-graph, 2026-10-05, T-0109). The scalar form `-" 2 "*2` = -4 (gen-m1/10f579127ffb6d9b A106).
The same holds for a one-cell intersection of a whole row: `('Q''1'!5:5 'Q''1'!B5)+2` with B5 = " 2 "
is 4 (gen-m1/7cb99acdd82402f1 Calc!A72, A103, A217, A235; found in T-0105).
Status: VERIFIED
Implemented in: `Model::array_node_to_number` (crates/rustsheet-engine/src/cast.rs), via `cast_number`; only the " 2 " form is oracle-covered, other text forms follow the scalar path.
Tasks: T-0131

## An error in a range operand of arithmetic wins over the right operand's error
Rule: when the left operand is a one-cell range holding an error and the right operand is an error,
the result is the left error: `(Data!C1:C10 Data!B9:D9)/Data!D$5` with C9 = `#N/A` and
D5 = `#DIV/0!` is `#N/A`. `handle_arithmetic` lifts a scalar right-hand error to a 1x1 array, so the
left range's element error is seen first (`arithmetic_on_values`, case 4).
The same holds element by element for a multi-cell range. Each element shows its own error if it has
one, and otherwise the scalar's error. A text element fails with `#VALUE!` before the scalar's error.
With C8 = 7, C9 = `#N/A`, C10 = "x" and D5 = `#DIV/0!`: `=Data!C8:C9/Data!D5` is {`#DIV/0!`; `#N/A`},
`=Data!C9:C10/Data!D5` and `=Data!C9:C10+Data!D5` are {`#N/A`; `#VALUE!`}, and a scalar left error wins
over every element (`=Data!D5/Data!C8:C9` is `#DIV/0!` twice).
Evidence: gen-m1/4dc8ed1de05a61be Calc!A250 (oracle: excel-graph, 2026-10-05; found in T-0105);
probes/arith-range-error-precedence (oracle: excel-graph, 2026-10-05, T-0138).
Status: VERIFIED
Implemented in: `Model::handle_arithmetic` (crates/rustsheet-engine/src/arithmetic.rs)
Tasks: T-0137, T-0138

## Text longer than 32,767 characters
Rule: concatenation does not fail past Excel's 32,767-character cell limit, it truncates:
`LEN(CONCATENATE(REPT("a",32767),"b"))` and `LEN(REPT("a",32767)&"b")` are both 32767, and the
kept text is the leading part (`RIGHT(CONCATENATE(REPT("a",32767),"b"),1)` is "a"). REPT with a
longer result is `#VALUE!` (`REPT("a",32768)`). The limit counts UTF-16 code units, so an emoji
(a surrogate pair) counts two: `REPT("😀",16383)` is fine, `REPT("😀",16384)` is `#VALUE!`, and
`LEN(CONCATENATE(REPT("😀",16383),"ab"))` is 32767. Truncation applies per element of a spilled
CONCATENATE (`LEN(CONCATENATE({"a";"bb"},REPT("c",32767)))` is {32767;32767}).
Evidence: probes/resource-limits C9–C11; probes/concatenate-limit-spill E1–E7, E12, G85–G104
(excel-graph, 2026-10-06, T-0233). LEN counts UTF-16 units too (text.md "Text positions count
UTF-16 code units", T-0249).
Backend: excel-graph only; excel-desktop not used.
Status: VERIFIED
Implemented in: `Text::concat`/`Text::truncate_utf16` (text.rs), shared by CONCATENATE and `&` (T-0240, T-1580);
REPT's limit in `rept_element` (functions/text/common.rs); both count UTF-16 units (T-0233).
CONCAT/TEXTJOIN give `#CALC!` (probes/text-result-limit C8-C9, not implemented).
Tasks: T-0240, T-0233, T-0249

## Truncation inside a surrogate pair keeps a lone high surrogate
Rule: when the 32,767-unit limit falls inside a surrogate pair, Excel keeps the high surrogate:
`LEN(CONCATENATE(REPT("a",32766),"😀"))` and `LEN(REPT("😀",16383)&"😀")` are 32767, and
`UNICODE(RIGHT(CONCATENATE(REPT("a",32766),"😀"),1))` is `#VALUE!` (UNICODE of a lone surrogate).
Evidence: probes/concatenate-limit-spill E8–E11 (excel-graph, 2026-10-06, T-0233).
Ours: `&` and CONCATENATE truncate with `Text::truncate_utf16`, which keeps the high half of a cut
pair (E8, E9, E11 pass since T-1580). Decision 0033 (drop the pair whole) is superseded by
decision 0052. E10 (UNICODE of the lone half) passes since T-1581. SPEC-0010.
Arguments after the cut change nothing but an error still wins (probes/concatenate-after-cut, excel-graph,
2026-10-15, T-1612; the engine matches all nine cells): `LEN(CONCATENATE(REPT("a",32766),"😀","zz",1))` and
`LEN(CONCATENATE(REPT("a",32767),"zz",1,TRUE))` are 32767 (A1, A4), and at a character boundary the kept text still ends in "a" (A9); an error
argument after the cut is the result (`#N/A` for `...,"😀",NA())` and `...,"zz",NA())`). Among several errors the first argument's wins, wherever the cut falls: `...,1/0,NA())` is `#DIV/0!` (A6), while `...,NA(),1/0)` and `CONCATENATE(NA(),REPT("a",32767),1/0)` are `#N/A` (A7, A8). `ISERROR` of the first error case is TRUE (A10).
Status: VERIFIED
Implemented in: `concat_limited` in arithmetic.rs (`&`), `fn_concatenate` in
functions/text/concatenate.rs, `Text::truncate_utf16` in text.rs
Tasks: T-0233, T-1483, T-1580

## SUMX2MY2 / SUMX2PY2 / SUMXMY2 pairing
Rule: the two arrays are paired element by element in row-major order and need only the same
element count, not the same shape (a 3x2 range pairs with a 6x1 range); a pair where either side
is not a number (blank, text, boolean in a range) is skipped; no numeric pair left is `#DIV/0!`;
different element counts are `#N/A`; scalar arguments are allowed (`SUMXMY2(2,4)` = 4).
Evidence: poi/FormulaEvalTestData_Copy.xlsx (Excel-cached, Excel 15) EverythingTests!D1336, E1336,
G1336, I1336, J1336 (`#N/A`), K1336, E1340, F1340, H1340, J1340 (`SUMX2PY2(G7:H9,J7:J12)` =
105.7931362501), K1340, F1344, G1344, H1344 (`#N/A`), I1344, J1344.
Status: VERIFIED (Excel-cached)
Implemented in: `Model::sumx_fold` / `sumx_values` (math_and_trigonometry/sum.rs); other `fn_get_two_matrices` callers are unchanged.
Open (unverified, T-0961): scalar text, boolean and empty arguments are currently one skipped pair
(`#DIV/0!`); the first error in either array wins over the element-count `#N/A`.
Tasks: T-0944

## STDEV / VAR / DEVSQ / AVEDEV / SUMSQ typed arguments; AVEDEV and DEVSQ of nothing is #NUM!
Rule: numeric text and booleans typed directly as arguments are counted (`STDEV("1","2")` =
0.7071…, `VAR(TRUE,FALSE,TRUE)` = 0.333…, `SUMSQ("2","3",4)` = 29, `AVEDEV(TRUE,FALSE)` = 0.5),
while text and booleans in ranges are ignored. With no number at all AVEDEV and DEVSQ return
`#NUM!`, whereas STDEV/VAR/VARP return `#DIV/0!` (inconsistent, but that is Excel).
Evidence: poi/FormulaEvalTestData_Copy.xlsx (Excel-cached, Excel 15) EverythingTests!G1288, K1288,
G1448, K1448, G1456, K1456, I392, L176, M1332, N1332; `#NUM!`: F176, G176, H392; `#DIV/0!`: F1288,
F1448, F1456, H1288.
Status: VERIFIED (Excel-cached)
Implemented in: `Model::for_each_value` users `fn_stdev_s`/`fn_stdev_p`, `fn_var_s`/`fn_var_p`,
`fn_devsq`, `fn_avedev`, `fn_sumsq`. Open: I392 (`DEVSQ("1","2",2)`) and K1456
(`VARP(TRUE,FALSE,TRUE)`) differ in the last digit (accumulation order, like E392/F392/J392).
Open: an empty argument (`DEVSQ(1,)`) is ignored by DEVSQ/AVEDEV/SUMSQ but counted as 0 by STDEV/VAR
(probes/average-value-args); no evidence yet for the first group (T-0236).
Tasks: T-0945

## MODE: text arguments and blank single-cell references are #VALUE!
Rule: a text value argument, even numeric text (`MODE(B8,"1",2,3)`), makes MODE `#VALUE!`, and so
does a single-cell reference to a blank cell: `MODE(B9,2,D7)` with B9 = 2 and D7 blank is `#VALUE!`
although 2 appears twice. Blanks and text inside a multi-cell range are ignored
(`MODE(C16:E16,-1.1)` = -1.1 with D16 blank). Also
`MODE("1",1,#DIV/0!,2,3)` is `#VALUE!` (whether argument order matters is unprobed). An error picked by CHOOSE propagates: legacy `=MODE(CHOOSE(1,B1:B6*1))` off the range's rows is `#VALUE!` (references.md "Legacy `@` on CHOOSE operator arms under aggregates"; other direct errors unprobed, T-1651). Not isolated yet: a single-cell reference to text on its own.
Evidence: poi/FormulaEvalTestData_Copy.xlsx (Excel-cached, Excel 15) EverythingTests!G936, H936,
K936, M936 (`#VALUE!`); D936, F936, I936, J936, L936.
Status: VERIFIED (Excel-cached)
Implemented in: `collect_mode_values` (mode_functions.rs). MODE.SNGL/MODE.MULT share it by inference
(unprobed), as do non-literal text-valued expressions (e.g. `A1&""`).
Tasks: T-0947

## MAXA / MINA: single-cell references are read like ranges
Rule: a single-cell reference argument to text counts as 0 and to a boolean as 1/0, as in a range:
`MAXA(F14,E19,F12)` over three text cells is 0, `MINA(B16,B10,B9)` with B16 = TRUE is 1.
Evidence: poi/FormulaEvalTestData_Copy.xlsx (Excel-cached, Excel 15) EverythingTests!J888, F912.
Status: VERIFIED (Excel-cached)
Implemented in: `for_each_value_a` (statistical/count_and_average.rs); covers direct single-cell
references only. A defined name pointing to one cell and numeric text in a referenced cell (counted as
0 by inference) are unprobed. AVERAGEA shares this coercion since T-1526, and the helper reads a
cell that XLOOKUP or INDEX picks as a reference (`AVERAGEA(XLOOKUP(1,A1:A2,B1:B2),5,7)` over "x" is 4;
probes/subtotal-reference-operands-d H22). An array argument is not read like a range: see
statistics.md "AVERAGEA, MAXA and MINA over an array" (T-1555). Engine choices with no oracle cell yet
(T-1551): the same for MAXA/MINA over a picked text cell, and direct date or currency text (read by
`cast_number`).
Tasks: T-0949, T-1526, T-1555

## "inf", "NaN" and out-of-range number text are text
Rule: the words "inf", "infinity", "NaN" (any case, any sign) and number text of 1E+308 or more
are never read as numbers: arithmetic, `--`, VALUE and COUNT/SUM/AVERAGE text arguments give
#VALUE! or skip them, criteria compare them as text; `"+1"` is the number 1.
Evidence: probes/text-number-words; full rule in numbers.md "Number text of 1E+308 or more is not a number".
Oracle: excel-graph, 2026-10-05.
Status: VERIFIED
Tasks: T-0920, T-0140

## Direct text arguments of COUNT coerce like arithmetic
Rule: a direct text argument of COUNT counts whenever arithmetic would read it as a number: clock text
(`"10:00"`), date text (`"1/2/2020"`, `"1/1/1900"`), currency, percent, thousands separators,
a leading blank before plain number text (`" 1"`), `"6 PM"` and `"12:60"` all count (`COUNT("10:00","x")` = 1). "inf" and the text `"TRUE"` do
not count. Reference cells holding text still never count.
Evidence: probes/count-direct-text (excel-graph, 2026-10-06), clock-text-oddities F37.
Implemented in: `functions/statistical/count_and_average.rs` `fn_count`.
Status: VERIFIED
Tasks: T-0252

## Parenthesised number text is negative
Rule: `"(1)"` is read as -1 wherever text is coerced to a number: `0+"(1)"` = -1, `SUM("(1)")` = -1,
`0+"(1,000)"` = -1000, `COUNT("(1)","x")` = 1; but `0+"-(1)"` is #VALUE!. The engine reads it in `parse_formatted_number` (formatter/format.rs), after the currency and date forms, so VALUE and typed cell input inherit it. Probed variants (all through `0+`, `VALUE` and `ISNUMBER` of a text cell; `ISNUMBER` is FALSE for all): padding inside the parentheses is accepted (`( 1 )` = -1), currency and percent inside are accepted (`($1)` = -1, `(1%)` = -0.01), decimals are accepted (`(1.5)` = -1.5); a sign inside (`(-1)`), nested parentheses (`((1))`) and trailing text (`(1)x`) are #VALUE!. The engine matches (T-0279, T-0282). More forms (probes/paren-number-text-forms, excel-graph, T-0281): e-notation is accepted (`(1e3)` = -1000, `(1E-3)` = -0.001, `(1.5e2)` = -150), `(0)` and `($0)` are 0 (the sign of zero is not observable), `($1)` is accepted but a sign after the currency (`($-1)`, `($ -1)`), a sign before it (`(-$1)`), a plus sign (`(+1)`), a minus before e-notation (`(-1e3)`, `(-0)`) and currency after the digits (`(1$)`) are #VALUE!. The engine rejects these as Excel does (T-0282). More forms (probes/paren-number-text-currency-variants, excel-graph 2026-10-10, T-1300): `($+1)`, `(1 $)` and `(€-1)` are #VALUE!, but a trailing euro sign is accepted: `(1€)` = -1, so `€` is the one currency that may follow the digits inside parentheses (`(1$)` stays #VALUE!). Euro suffix forms (probes/paren-number-text-euro-suffix, excel-graph 2026-10-10, T-1309): `(1 €)`, `(1,000€)`, `(1.5€)`, `(1e3€)` and `(1€ )` are all negative (-1, -1000, -1.5, -1000, -1), and the unparenthesised controls `1€` and `1 €` are 1, so inside parentheses a euro suffix behaves as it does outside, and the engine matches. Other currencies after the digits are unprobed beyond `$`; the engine rejects them. Typed into a cell (probes/typed-paren-number-input, excel-graph 2026-10-07, typed entry through a Graph range `values` write, T-0280): `(1)` and `( 1 )` are -1 with format General, `(1,000)` is -1000 with `#,##0`, `($1)` is -1 and `($1,000)` -1000 with `$#,##0_);[Red]($#,##0)`, `(1%)` is -0.01 with `0%`, and `(-1)` stays text. The unparenthesised controls get the same formats: `1`, `-1` General, `1,000` `#,##0`, `$1` `$#,##0_);[Red]($#,##0)`, `1%` `0%`. So the parentheses only negate; the format comes from the inner text, and typed currency gets the red-negative accounting-style format, not `$#,##0`. The engine's values and formats all match since T-0283 ("Typed currency and percent formats"). Formats are VERIFIED by the oracle's `ENTERED` log, committed in docs/research/typed-input-formats-2026-10-07.md (not scored: `CELL("format")` is `#VALUE!` on excel-graph).
Evidence: probes/paren-negative-text, probes/paren-negative-variants, probes/paren-number-text-forms, probes/paren-number-text-currency-variants, probes/paren-number-text-euro-suffix, probes/typed-paren-number-input (excel-graph, 2026-10-07; formats: docs/research/typed-input-formats-2026-10-07.md).
Status: VERIFIED
Tasks: T-0259, T-0277, T-0280, T-0281, T-0282, T-0283, T-1300, T-1309

## Typed currency and percent formats
Rule (en-US, text typed into a cell; probes/typed-currency-percent-input, excel-graph 2026-10-11, raw `ENTERED` log in docs/research/typed-currency-percent-input-2026-10-11.md, T-0283): a leading `$` (also `-$`, `$-`, and inside parentheses) gives `$#,##0_);[Red]($#,##0)`, or `$#,##0.00_);[Red]($#,##0.00)` when the text has any decimal digits (`$1.50`, `$1.0`, `$1,234.5`); a `%` gives `0%` or `0.00%` with decimals (`1.5%`, `1.0%`, `12.345%` -> `0.00%`, the value keeps all digits) and ignores grouping (`1,000%` is `0%`); scientific text keeps `0.00E+00` whichever symbol is around it (`$1e3`, `1e3%`, `-$1e3`, `($1e3)`); plain grouped text gets `#,##0` / `#,##0.00` (`(1,000.5)` -> `#,##0.00`). `-$1e3` is -1000 (the engine used to return +1000, also through `0+"-$1e3"` and VALUE, which are unprobed). A euro sign is not an en-US currency: `1€`, `1.5€`, `-1€` and `€1` stay text with General. The engine matches the `$` and percent formats (T-0283) and keeps typed euro text as text (T-1335: `set_user_input` no longer lists `€` among the currencies; `0+` coercion of `1€` still reads a number, a separate probed path (paren-number-text-euro-suffix); VALUE shares that code path (cast.rs, text/common.rs) but has no oracle cell of its own). Other prefix symbols keep the plain `{symbol}#,##0` format (unprobed).
Evidence: probes/typed-currency-percent-input (excel-graph, 2026-10-11).
Status: VERIFIED
Tasks: T-0283, T-1335

## AND/OR/XOR text arguments
Rule: in Microsoft 365, AND, OR and XOR skip every text argument that is not "TRUE" or "FALSE",
whether it is literal or computed, empty or not: `AND("1",TRUE)`, `AND("abc",TRUE)`, `AND(" ",TRUE)`,
`AND("",TRUE)`, `AND(""&"1",TRUE)`, `AND(IFERROR("1",0),TRUE)`, `AND(T("1"),TRUE)`, `AND(("1"),TRUE)`
and `AND("1",1)` are TRUE, and `AND("1",FALSE)` is FALSE. Text in an array literal (`{"1",TRUE}`) and in
referenced cells (A1 = "x", A2 = "1") is skipped as well. A literal "TRUE" argument counts as TRUE,
case-insensitively (`OR("true",FALSE)` = TRUE; computed, array and cell "TRUE" text are unprobed; "FALSE" next to a logical is unprobed, while `AND("FALSE")` alone is FALSE in ironcalc E48), but numeric text does not count as a number (`OR("1",FALSE)` = FALSE).
When nothing but skipped text is left, the result is `#VALUE!` (`AND("1")`, `AND("")`, `OR("","")`). An
error argument wins (`AND("1",#N/A)` = `#N/A`). OR and XOR behave the same.
CONFLICTING (decision 0030 2: Excel 15 is below AppVersion 16; decision 0039: the web value is the Phase 1-3
target, and the desktop check is T-1075 item 7): poi/FormulaEvalTestData_Copy.xlsx
EverythingTests!J136 caches `#VALUE!` for `AND("1",1)`, where current Excel for the web gives TRUE. The engine
follows the web, and J136 stays a known failure until a 16.x desktop value decides (M14). I136 `AND(E14,E17,"1")`
is not a conflict: E14 and E17 hold text or are empty, so every argument is skipped and both versions give `#VALUE!`.
Evidence: probes/and-or-xor-text-arguments (84 cells, excel-graph, 2026-10-11, T-0258); ironcalc-calc
LOGICAL/AND_OR_XOR.xlsx (Excel-cached) "AND XOR OR"!C46:E53; gen-m2/893302d6c5b99e2e Calc!A108 (excel-graph).
Status: VERIFIED (web); CONFLICTING with the Excel 15 cached J136. The engine passes every probe cell.
Tasks: T-0258, T-1075

## GCD / LCM / MULTINOMIAL range cells
Rule: GCD, LCM and MULTINOMIAL take ranges, but unlike SUM they do not skip what is not a number: a
text or boolean cell in a range is `#VALUE!` (`GCD(G1:G3)` with 4, "x", 6; `GCD(H1:H3)` with 4, TRUE,
6), a numeric-text cell counts as its number (`GCD(I1:I3)` with 4, "2", 6 is 2, LCM 12) and a blank
cell reads as 0 (`LCM(F1:F3)` with 4, blank, 6 is 0). Typed, numeric text is a number
(`GCD("8",4)` = 4, `MULTINOMIAL("2",3)`) but a boolean is `#VALUE!` (`GCD(TRUE,4)`,
`MULTINOMIAL(TRUE,3)`; typed LCM arguments are unprobed), whereas
`FACT(TRUE)` = 1 and `COMBIN("5",2)` = 10. SERIESSUM's coefficient range is stricter still: text,
boolean and numeric-text cells are all `#VALUE!` (a blank reads as 0), while a typed numeric-text
`x` is accepted (`SERIESSUM("1",0,1,{1,2})` = 3). SUMPRODUCT is the opposite: anything that is not a
number counts as 0, a typed scalar too (`SUMPRODUCT("2")` = 0, `SUMPRODUCT(TRUE)` = 0,
`SUMPRODUCT("2",3)` = 0, `SUMPRODUCT(3)` = 3), and text in a range is 0 (`SUMPRODUCT(A1:A6)` = 10).
PRODUCT and SUMSQ skip text and booleans in ranges and coerce typed ones (`PRODUCT(TRUE,3)` = 3).
PRODUCT of an array argument (`PRODUCT(IF(SEQUENCE(5),7))`) multiplies its numbers left to right
(probes/geomean-single-and-copies Copies!B1:B35). Text and booleans inside the array are skipped, an array with no
numbers gives 0 (`PRODUCT({"a"})`, `PRODUCT({TRUE,FALSE})`), and the first error element wins
(`PRODUCT({2,#N/A,#DIV/0!})` = #N/A) (probes/product-array-elements, excel-graph).
In a 2D array the first error in row-major order wins (`PRODUCT({1,#N/A;#DIV/0!,2})` = #N/A, `PRODUCT({1,#DIV/0!;#N/A,2})` = #DIV/0!). A
no-number array with a scalar gives the scalar (`PRODUCT({"a"},3)` = `PRODUCT(3,{"a"})` = `PRODUCT({TRUE,FALSE},3)` = 3). An error inside an array
ranks like a referenced-cell error, as in SUM: a direct scalar error beats it either way (`PRODUCT({1,#N/A},#NUM!,#DIV/0!)` = #NUM!,
`PRODUCT(#NUM!,{1,#N/A})` = #NUM!), and against a referenced error or another array the leftmost wins (`PRODUCT({1,#N/A},C1)` with C1=#NUM! is #N/A,
`PRODUCT(C1,{1,#N/A})` #NUM!, `PRODUCT({#N/A},{#DIV/0!})` #N/A) (probes/product-array-2d-and-scalar, excel-graph 2026-10-14, T-1287).
MDETERM, MINVERSE and MMULT are `#VALUE!` for a text cell in a matrix (arrays.md has blanks).
These decide the generator's argument modes (eval/catalog/functions.yaml `array`, T-1082); the
SUMX2MY2 family's `array` mode rests on "SUMX2MY2 / SUMX2PY2 / SUMXMY2 pairing" above, and
PERCENTOF's `aggregate` on Microsoft's page and on probes/percentof-snap-overflow-union (T-0416): direct text `"1"` and TRUE count as numbers (`PERCENTOF("1",2)` and `PERCENTOF(TRUE,2)` = 0.5), as in SUM.
The type check wins over a negative argument whatever the order: `GCD(-1,B1:B11)` with the text "TRUE" in B1 is
`#VALUE!`, and `MULTINOMIAL(A6:D9)` whose first cell is -2.5 and later cells hold FALSE and text is `#VALUE!`
(gen-m6/5cc04d6f42b23b51 Calc!A11, gen-m6/2c1a6e93a1974f48 Calc!A35, gen-m7/4629e931955aa4b8 Calc!A7, excel-graph; fixed in
T-1261). A negative's `#NUM!` loses to every other error, whatever the position and kind of the other argument, in GCD, LCM
and MULTINOMIAL alike: typed text (`GCD(-1,"a")`, `GCD("a",-1)`), a typed TRUE, a typed error (`GCD(-1,NA())`, `GCD(NA(),-1)`), a text,
TRUE or error cell of a range before or after it, and an array-literal element (`GCD(-1,{4,#N/A})`, `GCD(-1,{4,"a"})`, `GCD(-1,{4,TRUE})`) are
`#VALUE!` or the error itself; only with no other error is the answer `#NUM!`. Array-literal text or TRUE elements next to a negative are the same in LCM and MULTINOMIAL
(`LCM(-1,{4,"a"})`, `LCM({4,TRUE},-1)`, `MULTINOMIAL(-1,{4,#N/A})`: probes/gcd-lcm-multinomial-error-order A43:A52, T-1655). An argument above 2^53 is
`#NUM!` the same deferred way, a loser to every other error in either order, in GCD, LCM and MULTINOMIAL (typed text, `NA()` and a negative, range cells and array-literal elements are all probed, T-1661): `GCD(10^16,"a")` and `GCD("a",10^16)` are `#VALUE!`,
`GCD(10^16,NA())` and `GCD(NA(),10^16)` `#N/A` (A9:A12, A23:A26, A37:A40), and `GCD(10^16,-1)` / `GCD(-1,10^16)` are `#NUM!` (A13:A14); the engine used to return the
"Argument too large" `#NUM!` at once for GCD and LCM.
LCM's result above 2^53 is deferred the same way: `LCM(2^26+1,2^27+1)` is `#NUM!` but `LCM(2^26+1,2^27+1,"a")` is `#VALUE!`, `...,NA())` `#N/A`, and a later 0 makes the result 0 (`LCM(2^26+1,2^27+1,0)`); a later negative stays `#NUM!` (probes/gcd-lcm-overflow-ranges A1:A56, excel-graph, 2026-10-16, T-1661).
An omitted argument: the first one is `#N/A` (`GCD(,4)`, `GCD(,-1)`, `GCD(,)`, `MULTINOMIAL(,4)`), a later one is skipped, not read as 0
(`LCM(4,)` and `LCM(2,4,)` are 4, `GCD(4,)` 4, `GCD(4,,-1)` and `GCD(-1,,4)` `#NUM!`). Two different non-negative errors are judged left to right, the first argument's winning: `GCD("a",NA())` is `#VALUE!`,
`GCD(NA(),"a")` `#N/A`, `GCD(1/0,NA())` `#DIV/0!`, `GCD(NA(),1/0)` `#N/A`, and an omitted first argument is `#N/A` ahead of any later error (`GCD(,"a")`, `GCD(,NA())`) while a later omitted one is
skipped, so the earlier error shows (`GCD("a",)` is `#VALUE!`, `GCD(NA(),)` `#N/A`); LCM and MULTINOMIAL alike (A1:A8, A15:A22, A29:A36).
Evidence: probes/gcd-lcm-multinomial-negative-precedence E1:E71 (excel-graph, 2026-10-15, T-1574); engine fixed in T-1574. probes/gcd-lcm-multinomial-error-order A1:A52 (excel-graph, 2026-10-15, T-1655); GCD and LCM fixed in T-1655.
Evidence: probes/math-argument-modes Sheet1!C1:C56 (excel-graph, 2026-10-08).
Implemented in: the engine matches every cell except MULTINOMIAL numeric text (C9, C47, C52; T-1190)
and SERIESSUM with a boolean coefficient cell (C44, engine 11; T-1191).
Status: VERIFIED
Tasks: T-1082, T-1190, T-1191, T-1261, T-1574, T-1655, T-1661
