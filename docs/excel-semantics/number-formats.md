# Number formats (`TEXT` and cell formats)

All evidence below is from Apache POI's format test workbooks, scored by the excel-graph oracle
(sidecars in `eval/suites/poi/oracle/`, 2026-10-04) and checked against the values desktop Excel
cached in the same workbooks where it saved them. In each workbook `Tests!An` is
`=TEXT(Cn, Bn)` (TextFormatTests: `=TEXT(Cn, ";;;"&Bn)`), so `Bn` is the format and `Cn` the value.

## An empty format string returns empty text
Rule: `TEXT(<blank>, <blank format>)` returns `""`, not `#VALUE!`. On a number an empty format shows
nothing but a negative's minus, also when the value would round to zero: `TEXT(0,"")` = "",
`TEXT(5,"")` = "", `TEXT(-5,"")` = "-", `TEXT(-0.0001,"")` = "-" (so `INDIRECT(TEXT(-0.0001,""))` is `#REF!`);
the result is text (`ISTEXT` TRUE, `LEN` 0). Text and booleans come back unchanged (`TEXT("a","")` = "a",
`TEXT(TRUE,"")` = "TRUE").
Evidence: poi/ElapsedFormatTests.xlsx A52:A215; poi/FormatChoiceTests.xlsx A27:A53; poi/GeneralFormatTests.xlsx
A69, A71; poi/NumberFormatApproxTests.xlsx A249; poi/FormatConditionTests.xlsx A38, A51:A54 (Excel-cached);
probes/text-format-bare-literals A32-A39 (excel-graph, 2026-10-15, T-1262); gen-m6/17223361bef5b5cb Calc!A12.
Unprobed: an omitted value argument (`TEXT(,"")`).
Status: VERIFIED
Implemented in: `format_text` and `format_number` (no sections) in formatter/format.rs, used by `fn_text` (functions/text/common.rs).
Tasks: T-0933, T-1262

## Bare letters and digits in a number format
Rule: on a number, unquoted letters that are not format tokens (see "Unquoted ASCII letters in a number format"
below, T-1450), non-ASCII letters (only `ü` is probed; the engine treats every non-ASCII letter so, inferred) and the digits 1-9 print as literal characters: `TEXT(PI(),"ü")` = "ü",
`TEXT(PI(),"x")` = "x", `TEXT(5,"2")` = "2", `TEXT(1.5,"9")` = "9", `TEXT(5,"1/2")` = "1/2", `TEXT(5,"3;4")` = "3",
`TEXT(-5,"3;4")` = "4". Among digit placeholders they sit where they are and the digits fill around them
(`TEXT(1,"0a0")` = "0a1", `TEXT(1234,"0x00")` = "12x34", `TEXT(12.3,"0x.0")` = "12x.3").
- With no integer digit placeholder a one-section format still shows a negative's minus first:
  `TEXT(-PI(),"x")` = "-x", `TEXT(-5,"2")` = "-2". The value itself does not show (`TEXT(12.3,"x%")` = "x%").
- A decimal point with no digit placeholder before it shows the integer part right before it, as `#.` would:
  `TEXT(12.3,".")` = "12.", `TEXT(-12.3,".")` = "-12.", `TEXT(0.5,".")` = "1.", `TEXT(0,".")` = ".",
  `TEXT(12.3,"x.")` = "x12.", `TEXT(12.3,".x")` = "12.x", `TEXT(12.3,"1.0")` = "112.3", `TEXT(12.3,"1.5")` = "112.5",
  `TEXT(-1,"2.5")` = "-21.5", `TEXT(1.005,"-21.5")` = "-211.5". A point with no decimal placeholder after it
  still prints (`TEXT(12.3,"0.1")` = "12.1").
- Letters that are date tokens make the section a date one: `TEXT(PI(),"ab")` = "a43" (`b`, Buddhist year), and
  `TEXT(12.3,"a.b")` is `#VALUE!` (a point and a date token). `n`/`N` and a bare `E` are `#VALUE!`.
- `General` may stand among literals: `TEXT(5,"General x")` = "5 x", `TEXT(5,"x General")` = "x 5".
- A one-section format holding `@` and other text applies General to a number: `TEXT(5,"x@")` = "5".
Evidence: probes/text-format-bare-literals A1-A31, A57-A76 (excel-graph, 2026-10-15, T-1262); gen-m6/2c1a6e93a1974f48
Calc!A15, gen-m6/63012866a9d8b7db Calc!A3 (excel-graph, 2026-10-09, T-1239).
Status: VERIFIED
Implemented in: `Lexer::next_token` (formatter/lexer.rs), `Parser::parse_part` (formatter/parser.rs, the implicit `#`
before a point and `General` among literals), `format_number` (formatter/format.rs). Unprobed: `General` next to digit
placeholders or date tokens, or twice (the engine says `#VALUE!`; T-1614), a negative's minus for a value that rounds to
zero under a literal-only format or `.` (`TEXT(-0.4,"x")` is "x" and `TEXT(-0.5,".")` "1." here; T-1129), and a lowercase
scientific `0.0e+0` (always the year token here, so `#VALUE!` as before).
Tasks: T-1262, T-1450

## Buddhist-era year, era and calendar tokens
Rule: in a date section `b`/`bb` is the two-digit Buddhist-era year (year + 543) and `bbb`+ the four-digit one;
`e` (any run, lowercase only) is the four-digit year; `g`/`G` (any run that does not spell `General`) is the era,
empty for the Gregorian calendar; `B1`/`b1` and `B2`/`b2` are calendar prefixes that print nothing. Serial 100
(1900-04-09): `b`, `bb`, `BB` = "43", `BBB`, `BBBB`, `bbbb` = "2443", `e`, `ee` = "1900", `e+` = "1900+",
`g`, `gg`, `ggg`, `G` = "", `B1`, `B2`, `b1`, `b2` = "", `ABC` = "A43C", `x d` = "x 9", `dx` = "9x". A lone uppercase `B`
that ends the code is `#VALUE!` (`TEXT(100,"B")`, `TEXT(1,"dB")`), as is uppercase `E` without a sign.
Evidence: probes/text-format-bare-literals A40-A63 (excel-graph, 2026-10-15, T-1262); gen-m7/6db767d720c5b55a Calc!A30;
probes/text-format-unquoted-letters A53-A104.
Status: VERIFIED
Implemented in: `Lexer::next_token` (formatter/lexer.rs), `BUDDHIST_ERA_OFFSET` (formatter/format.rs). `B2` (Hijri) with
any date or time token that prints is `#VALUE!` in the engine: the Hijri calendar is not implemented (unprobed). Whether
the lone-`B` error is about the end of the code or of the section is inferred from `"B"` and `"dB"` only.
Tasks: T-1262

## Text and booleans use the format's text section
Rule: for a text or boolean value, `TEXT` applies the 4th (text) section; `@` stands for the
value's text and booleans become "TRUE"/"FALSE". `;;;"hi"` → "hi" (A3), `;;;\@@\@` → "@jello@"
(A6), `;;;-@-@-` → "-jello-jello-" (A10), `TRUE` with `;;;-@-` → "-TRUE-" (A17), and
`TEXT(TRUE,"General")` = "TRUE".
Evidence: poi/TextFormatTests.xlsx A3:A10, A12:A29; poi/GeneralFormatTests.xlsx A72, A73.
A doubled quote inside a quoted literal: `;;;"h""I"` gives "hI" in Excel (A4, A14, A23), not `h"I`, so
it is not an escape there: see "A doubled quote in a format literal is not an escape" below.
Status: VERIFIED
Implemented in: `format_text_value` (formatter/format.rs) used by `fn_text` (functions/text/common.rs), scalars and array elements; a text value with a format of fewer than four sections is unprobed and returns the text unchanged.
Tasks: T-0935

## TEXT of a text value is `#VALUE!` over 255 characters
Rule: `TEXT` of a text value (a boolean's "TRUE"/"FALSE" included) is `#VALUE!` once its result is over
255 characters, whatever the format: `LEN(TEXT(REPT("a",255),";;;@"))` = 255 (A10) but 256 characters
is `#VALUE!` with `;;;@`, `@`, `General` and `0` alike (A11, A19-A21); `;;;@@` on 128 characters
(256) is `#VALUE!` (A24) while `;;;@@"x"` on 127 (255) is fine (A25); longer texts up to 32,767 are
`#VALUE!` (A1-A8, A12-A15). Whether the cap is on the result or also on the input cannot be told
apart from these cells (every input over 255 also gives a result over 255). Characters vs UTF-16
units is unprobed. Compare the other 255-character rules in numbers.md and dates.md.
Evidence: probes/text-format-text-limit A1-A25 (excel-graph, 2026-10-08, T-0935).
Status: VERIFIED
Implemented in: `format_text_value` (formatter/format.rs), `MAX_TEXT_FORMAT_CHARS`, checked as the
result grows so a format of many `@` cannot allocate a huge string (CLAUDE.md directive 5).
Unprobed (T-0935 follow-up): a text value with a 1-3 section format and no `@` section (the
engine returns the text unchanged) and a General 4th section. `0;@`-style formats, `_x`/`*x` and
digit or date tokens beside `@` are now probed: see "TEXT of a text value: `@` with other tokens in
one section" (T-1656).
Tasks: T-0935

## A doubled quote in a format literal is not an escape
Rule: inside a number-format string, `""` within a quoted literal ends that literal and starts the
next, so it shows nothing (it is not an escaped quote). In every section probed:
`TEXT(42, B1)` with B1 = `0 """Millions"""` is "42 Millions" (C1; the value is 42 in C1-C7, "x" in C8-C11), `0 "a""b"` = "42 ab" (C2),
`"h""I"` = "hI" (C3), `0 ""` and `0 """"` = "42 " (C4, C5), `0 "a"""` = "42 a" (C6),
`0" ""x"" "0` = "4 x 2" (C7); on text, `;;;"h""I"` = "hI" (C8), `;;;"""x"""` = "x" (C9),
`;;;"a"""@` = "ax" (C10), `@" ""q"""` = "x q" (C11).
Evidence: probes/format-doubled-quotes C1-C11 (excel-graph, 2026-10-08, T-0935); poi/TextFormatTests.xlsx A4, A14, A23.
Status: VERIFIED
Implemented in: `consume_string` (formatter/lexer.rs) ends the literal at every `"`.
Tasks: T-0935

## Elapsed-time tokens
Rule (observed, ElapsedFormatTests, a 1904-date-system workbook):
- Fractional elapsed seconds work: `[h]:m:s.000` on 3.14159 → "75:23:53.376" (A2).
- A plain `h`/`ss` after `[h]`/`[s]` of the same unit shows the elapsed total too:
  "It was 75 [yes, 75] hours and 23:53" (A10), "271433 [yes, 271433] seconds" (A11).
- A negative value prints one leading "-" before any literal text:
  "-It was 2 [yes, 2] hours and 03:04" (A40).
In the 1900 date system TEXT of a negative under a format that mixes an elapsed token with a date/time token (`[h]:mm`, `[s] yy`, `[s].00`) is #VALUE! instead; a format whose only date/time token is one elapsed token still prints a minus, with any literals (below; dates.md "Negative serials under an explicit negative section", probes/text-negative-serial-elapsed-mixed, probes/negative-date-sections-1900, T-1079/T-1105); the minus above is the 1904-workbook behavior. Two elapsed tokens in one section (`[h]:[mm]`) are #VALUE! in both systems, for positive values too (same evidence).
An elapsed token works on the magnitude of the value and keeps a minus for a negative one, in both date systems (VERIFIED, probes/elapsed-tiny-negatives-1900/-1904 and elapsed-fractions-1900/-1904, excel-graph, T-1338, T-1346): `[s]`/`[ss]` round it to nearest (1.7 s = "2", 1.2 s = "1", 1.5 s = "2", -0.9 s = "-1", -1.7 s = "-2", `TEXT(-1E-10,"[s]")` = "-0"); `[m]`/`[mm]`/`[h]`/`[hh]` truncate it (1.7 min = "1", 1.7 h = "1", -1.7 min = "-1", -0.3 h = "-0"). Padding follows the sign (`[ss]` of -1E-10 = "-00", of -1.7 s = "-02", of 1.7 s = "02"). Binary noise is absorbed: -1.5/86400 days is "-2", `TEXT(-(3-1E-7)/24,"[h]")` = "-3", `TEXT(-(2-1E-9)/1440,"[m]")` = "-2", yet `TEXT(-0.49999999999/86400,"[s]")` = "-0" and `TEXT(-(5-1E-6)/86400,"[s]")` = "-5". The engine models that as a 1e-12 snap of the seconds for `[s]` and a millisecond snap for `[h]`/`[m]`: fitted to these cells, not known (the true threshold is unprobed). The positive near-integers behave the same (VERIFIED, probes/elapsed-positive-large-1900/-1904, excel-graph, T-1362): `TEXT((3-1E-7)/24,"[h]")` = "3", `TEXT((5-1E-6)/86400,"[s]")` = "5", `TEXT(0.49999999999/86400,"[s]")` = "0". The count is plain arithmetic with no 32-bit limit in either date system: `TEXT(30000,"[s]")` = "2592000000", `TEXT(24856,"[s]")` = "2147558400", `TEXT(2958465,"[s]")` = "255611376000", `TEXT(1E9,"[h]")` = "24000000000". `TEXT(-1.5,"[s]")` = "-129600".

**Elapsed tokens beyond the i64 range** (VERIFIED, probes/elapsed-huge-a/b0/b1/b2/c/d/e/f/g/h/i-1900, excel-graph, T-1375): the count is printed in full, never in scientific notation, but only its first 15 significant digits are kept and the rest are zeros, from 1e15 up (`TEXT(1E15,"[s]")` = "86400000000000000000", `TEXT(1.234567890123456E16,"[s]")` = "1066666657066660000000", `TEXT(1E100,"[h]")` = 1 followed by 99 zeros times 24; the same at `[m]`/`[ss]`; negatives keep the minus). Below 1e15 it is the exact integer. The result is `#VALUE!` once it is over 255 characters (`TEXT(1E253,"[h]")` is 255 digits and works, `TEXT(1E254,"[h]")` is #VALUE!; `TEXT(1E250,"[s]")` works, `TEXT(-1E250,"[s]")` is #VALUE! because of the minus; `TEXT(1E300,"[s]")` is #VALUE!). When the count itself overflows a double the result is "0" ("-0" for a negative): `[s]` once days*86400 overflows (`TEXT(2.0807E303,"[s]")` = "0" while `TEXT(2.0806E303,"[s]")` is #VALUE!), `[m]`/`[h]` once days*1440 / days*24 does (`TEXT(1.5E307,"[h]")` = "0", `TEXT(1E305,"[m]")` is still #VALUE!). A literal 1.7E308 in a formula made excel-graph reject the whole workbook, so serials that high are unprobed. The 255-character `#VALUE!` is probed for elapsed tokens only; the engine applies the 255-character cap to every date/time section as an assumption. Corrected (T-1388): a far lower limit on the format code itself decides first (see "TEXT format code complexity limit" below), so a date/time TEXT cannot reach 255 characters through repeated tokens; whether any can exceed 255 is unprobed.
Evidence: poi/ElapsedFormatTests.xlsx A2, A10, A11, A40 (web and the desktop Excel that saved the
workbook agree).
Status: VERIFIED
Tasks: T-0934

## CONFLICTING: elapsed token after a plain `s`
Excel for the web and desktop Excel disagree. The cells stay pending (decision 0019); the
excel-desktop probe is Phase 4 (T-1075), and until then a task that must choose matches the web
(decision 0039). `s:m" @ hour "[hh]` and `s" secs and "[m]" mins"`:

| cell | Excel for the web | desktop Excel (cached in the workbook) |
|---|---|---|
| A8 | "53:23 @ hour 1293" | "53:23 @ hour 75" |
| A9 | "53 secs and 77581 mins" | "53 secs and 4523 mins" |
| A18 | "4:3 @ hour 108" | "4:3 @ hour 02" |
| A19 | "4 secs and 6480 mins" | "4 secs and 123 mins" |
| A28 | "0:0 @ hour 12" | "0:0 @ hour 00" |
| A29 | "0 secs and 720 mins" | "0 secs and 0 mins" |

A38/A39 and A48/A49 are the negated A18/A19 and A8/A9. The desktop values are the ordinary
elapsed totals; the web values fit hours = (sec + 0.5)·24, which looks like a web renderer bug.
The cells stay pending (dropped from the sidecar by the fill's conflict check).
Evidence: poi/ElapsedFormatTests.xlsx A8:A49 (oracle: excel-graph 2026-10-04; workbook cached values).
Status: CONFLICTING (web vs desktop; excel-desktop probe pending)
Tasks: T-0934, T-1075

## CONFLICTING: lowercase `e` exponent in a format
Excel for the web returns `#VALUE!` for `TEXT` with a lowercase-`e` scientific format; the Mac Excel
2008 (12.0) that saved the workbook cached real strings: `|#,e-#|` on 123456.789 → "|1e5|" (A2),
`|#,######.####|e-|#|` → "|123,456.789|e|0|" (A10), `|#,e-#|` on 1.23456789E-05 → "|1e-5|" (A42).
238 of the workbook's 249 cells are of this kind and stay pending. Uppercase `E` formats agree
(see below).
Evidence: poi/NumberFormatApproxTests.xlsx Tests!A2:A39, A42 onward (oracle: excel-graph
2026-10-04; workbook cached values).
Also: Excel for the web refuses a whole workbook whose cell `numFmt` is `0.00e+00` (createSession
501 `FileCorruptTryRepair`/`unsupportedWorkbook`; the same workbook with the other gen-m10 codes
loads), so the M10 generator does not use lowercase exponents (T-1611).
Status: CONFLICTING (web vs desktop; excel-desktop probe pending)
Tasks: T-1075 (Phase 4; decision 0039)

## Conditional sections
Rule: a format with a single `[cond]` section that does not match prints the number in General
(`[<10]#" Wow"` on 10 → "10", A3; `[>10]#" Big"` on 10 → "10", A5); `[<>10]` is a valid
condition (A17 "1 Wow", A18 "10"); spaces inside the brackets are allowed (`[< 10]` → "1 Wow",
A23; `[<      10]`, A25).
Evidence: poi/FormatChoiceTests.xlsx A3:A25.
Status: VERIFIED
Tasks: T-0936

## Scientific formats with literals after `E`
Rule: literals may sit between `E+`/`E-` and the exponent digits; the exponent sign is printed
next to the digits, after the literal, and a zero mantissa prints "0.": `|#.#|E+|#|` → "|1.2|E|+5|"
(A41) and "|0.|E|+0|" for 0 (A137); `E-` omits a positive sign: "|1.2|E|5|" (A40), "|1.2|E|-5|"
(A96).
Evidence: poi/NumberFormatApproxTests.xlsx A40, A41, A96, A97, A136, A137, A176, A177, A216, A217.
Status: VERIFIED
Tasks: T-0937

## `TEXT(x,"General")` uses an 11-character fixed-notation limit
Rule: General prints fixed notation, with at most 9 decimals ("0." plus 9 decimals is 11
characters; the sign does not count), when `1E-09 <= |x| < 1E+11` and the fixed form shows at least
as many significant digits as the E form (6 significant digits; 5 from `|exp| >= 99`, `numbers.md`
"Exponent width"). Otherwise it prints E notation.
- Fixed: 1E-09 → "0.000000001" (A27), -1E-09 → "-0.000000001" (A61), 1E10 → "10000000000"
  (A17), 1.2345678912 → "1.234567891" (A13, 10 digits against 6), 9.9999996E+10 →
  "99999996000" (boundary A9).
- E notation: 1E-10 → "1E-10" (A28; below 1E-09), 1E11 → "1E+11" (A18; 12 characters),
  `1.23456789012345E-05` → "1.23457E-05" (D8: the fixed "0.000012346" fits but keeps 5 digits
  against 6), and 9.9999996E-10 → "1E-09" (boundary A10: below 1E-09, although the fixed
  "0.000000001" would show the same digit).
This differs from number→text conversion (`numbers.md`, 15 digits).
Single-digit ties go to fixed (A27: "0.000000001" against "1E-09"). Multi-digit ties also go to
fixed when the digit counts are equal: `0.000123456789` → "0.000123457", `1.2345E-05` →
"0.000012345". When fixed shows fewer digits than E, E wins: `1.5E-09` → "1.5E-09" (fixed
"0.000000002", 1 digit), `2.5E-09` → "2.5E-09", `1.4E-09` → "1.4E-09", `1.23456789E-06` →
"1.23457E-06".
The 11-character limit also applies from 1 on: the sign does not count, the decimal point does,
and the value is rounded to fit, so decimals are cut to `11 - integer digits - 1`:
`123456.123456789` → "123456.1235", `-123456.123456789` → "-123456.1235", `22.712470704` →
"22.7124707", `1234567.891234` → "1234567.891", `100000.123456789` → "100000.1235",
`12345.123456789` → "12345.12346", `1234.5678901234` → "1234.56789". Rounding is half away from zero on the 15-significant-digit decimal, not on the binary value (the ROUND
family's rule, `numbers.md`): `12345678902.5` → "12345678903", `12345678903.5` → "12345678904",
`1234567890.5` → "1234567891", `123456789.25` → "123456789.3", `123456789.75` → "123456789.8",
`123456.12345` → "123456.1235", `1234567.8915` → "1234567.892" (A18-A24). Values whose integer part has
10 or 11 digits round to the integer: `9999999999.5` → "10000000000", `12345678901.5` → "12345678902", and
`99999999999.6` rounds up to 1E+11 and prints "1E+11".
Ties and near-ties below 1 follow the same rule (probes/text-general-ties-below-one A1-A19, excel-graph, 2026-10-13, T-1429): the value is rounded half away from zero on the 15-digit decimal to at most 9 decimals, then trailing zeros are dropped. `0.1234567895` → "0.12345679" (the tie rounds up to ...790), `0.1234567885` → "0.123456789", `0.5000000005` → "0.500000001", `0.12345678951` → "0.12345679", `0.123456789499` → "0.123456789", `-0.1234567895` → "-0.12345679", `0.0123456785` → "0.012345679", `0.9999999995` and `0.9999999996` → "1", `0.9999999994` → "0.999999999", `0.000000015` → "0.000000015". The digit-count rule still picks E notation where fixed would show fewer significant digits: the ties `1.5E-09`, `2.5E-09`, `3.5E-09` (and `-1.5E-09`) stay "1.5E-09", "2.5E-09", "3.5E-09", and `0.0000001235`/`0.00000012345` stay "1.235E-07"/"1.2345E-07" (their fixed forms need more than 9 decimals). The engine matches.
Evidence: poi/GeneralFormatTests.xlsx A13, A17, A18, A26-A28, A61; probes/number-text-exponent-digits
Calc!D8 (excel-graph); probes/text-general-exponent-boundary A9, A10 (excel-graph, 2026-10-07); probes/text-general-ties-wide-fixed
A1-A24 (excel-graph, 2026-10-12).
Status: VERIFIED
Implemented in: `format_general` (formatter/general.rs).
Tasks: T-0938, T-1225, T-1422, T-1429

## A comma between digit placeholders turns on thousands grouping
Rule: a `,` anywhere between digit placeholders (not only in `#,##0`) means "group the integer
part in threes". It is not a literal at that position. `0`/`?` placeholders beyond the number's
digits pad the grouped result, and the padding is grouped too: with `0` a padded group shows its
separator, and with `?` the padded separator becomes a space. A trailing `,` still scales by 1000
and keeps the sign.
- `0,0000000` on 1234567 → "01,234,567" (A67); `0,00000000` → "001,234,567" (A68);
  `0,000000000` → "0,001,234,567" (A69); `$000,000,000` on 314159 → "$000,314,159" (A335).
- `?,???????` → " 1,234,567" (A44); `?,????????` → "  1,234,567" (A45); `?,?????????` →
  "    1,234,567" (A46, A112). Negatives put "-" first: "- 1,234,567" (A80), "-    1,234,567"
  (A82, A118).
- `#,,` on -1234567 → "-1" (A95).
Evidence: poi/NumberFormatTests.xlsx Tests!A44:A46, A67:A69, A80:A82, A95, A112, A118, A335
(desktop Excel cached values, Microsoft Macintosh Excel 16.0; no web sidecar).
Status: VERIFIED
Implemented in: crates/rustsheet-engine/src/formatter/format.rs (group positions count from the format's last digit placeholder; `?` padding separator is a space; a value exactly one unit at the last shown place keeps its sign)
Tasks: T-0952

## `£`, `¥` and `|` print as literals without quotes
Rule: these characters need no quotes or backslash in a number format; they print as written,
and the number's "-" goes in front of everything (also before a quoted prefix). (`€` already works.)
- `£000.00` on 12.3 → "£012.30" (A5); `¥#.00" after"` → "¥12.30 after" (A18), on -12.3
  → "-¥12.30 after" (A21); `"before "¥#.00` → "before ¥12.30" (A19), on -12.3 → "-before ¥12.30"
  (A22).
- `|?.?|` on 1.1 → "|1.1|" (A107); `|??.?|` → "| 1.1|" (A108); `|?.??|` → "|1.1 |" (A109);
  `|??.??|` → "| 1.1 |" (A110); on -1.1 → "-|1.1|" … "-| 1.1 |" (A113:A116).
Evidence: poi/NumberFormatTests.xlsx Tests!A5, A18, A19, A21, A22, A107:A110, A113:A116 (desktop
Excel cached values, Microsoft Macintosh Excel 16.0).
Implemented in: crates/rustsheet-engine/src/formatter/lexer.rs (`next_token` literal list)
Status: VERIFIED
Tasks: T-0953

## `a/p` and `A/P` AM/PM markers
Rule: `a/p` prints "a" or "p" and `A/P` prints "A" or "P", like `AM/PM`: hours switch to the
12-hour clock. `hh:mm:ss a/p` on 17816.607951388887 → "02:35:27 p" (A11); `HH:MM:SS A/P` →
"02:35:27 P" (A16); on 36191.1702084375 → "04:05:06 a" (A30), "04:05:06 A" (A35).
The short marker keeps the case it is written in, but the long one does not: lowercase `am/pm` still
prints uppercase "PM" (`hh:mm:ss.000 am/pm` → "02:35:27.000 PM", A13; `HH:MM:SS.000 AM/PM` gives the
same, A18).
Evidence: poi/DateFormatTests.xlsx Tests!A11, A13, A16, A18, A30, A35 (desktop Excel cached values,
Microsoft Macintosh Excel 16.0; the workbook uses the 1904 date system, which does not change the
time of day).
Status: VERIFIED
Tasks: T-0954

## Fractional seconds in clock formats
Rule: `.0`, `.00` or `.000` after a plain `s`/`ss` prints the fraction of the second, rounded to that
many digits: on 36191.1702084375 (04:05:06.009) `hh:mm:ss.000 am/pm` → "04:05:06.009 AM" (A32),
`HH:MM:SS.000 AM/PM` → "04:05:06.009 AM" (A37), `h:m:s.00` → "4:5:6.01" (A40); on 17816.607951388887
`hh:mm:ss.000 am/pm` and `HH:MM:SS.000 AM/PM` → "02:35:27.000 PM" (A13, A18). With the short marker:
`h:m:s.000 A/P` → "2:35:27.000 P" (A12), `H:M:S.00 a/p` → "2:35:27.00 p" (A17), `h:m:s.00 A/P` →
"4:5:6.01 A" (A31), `H:M:S.00 a/p` → "4:5:6.01 a" (A36). Every cell shows rounding, not truncation: 36191.1702084375 is 04:05:06.008999934 and 17816.607951388887 is 14:35:26.99999985 (truncation would give .008, .00 and 26.999).
Evidence: poi/DateFormatTests.xlsx Tests!A12, A13, A17, A18, A31, A32, A36, A37, A40 (desktop Excel cached
values, Microsoft Macintosh Excel 16.0; the 1904 date system does not change the time of day).
Status: VERIFIED
Tasks: T-0981

## TEXT format code complexity limit
Rule: `TEXT` is `#VALUE!` when the format code is too complex, which happens long before 255 characters. The model below is fitted to the probed boundaries of probes c to k (524 cells; VERIFIED, excel-graph; excel-desktop unprobed); every probed cell fits. Its weights look like the size in bytes of a compiled code, a quoted character being one UTF-16 unit (2) and an unquoted one 3.
- **Sections:** more than four is `#VALUE!`, even when the fifth is empty (`"0;;;;"`, `"0;0;0;@;"`); `"0;;;"` works.
- **Elements:** at most 32 elements per section. An element is a run of digit placeholders together with their `,` and `.` (`REPT("0.",20)` and `REPT("#,",17)` work), a run of literals (unquoted, escaped and quoted text merge into one run, so `REPT("""a"" ",17)` works), or any other token (`d`, `mm`, `dddd`, `%`, `@`, `_x`, `*x`, `[h]`, `AM/PM`, ...). `REPT("dm",16)` works (32 elements), `REPT("dm",16)&"d"` fails (33). The count restarts at each `;` (two sections of 17 elements work).
- **Cost:** at most 250 over the whole code (T-1434; the earlier 166 model counted the same boundaries in units of 1.5), counted as follows:
  - 3 once per section holding a digit placeholder, plus 2 per placeholder run, plus 2 per `.` in a run (`"0000000000"` costs 5; `0.0` costs 7; `"0-0"` costs 10: `"0-0"` + 120 quoted works and 121 fails, `"0-0-0"` + 117 works and 118 fails, `"0-0;"` + 120 works and 121 fails; probes/text-format-length-limit-o A1:A4, -p A5:A7, -r, T-1486). A section after the first that holds both a placeholder and quoted text pays 1 more: `"0-0;0-0"` + 114 quoted works and 115 fails, `"0;0"` + 120 fails, `"0-0;0"` and `"0;0-0"` + 117 work and 118 fail (-q A14:A15, -r); A color between two placeholders does not split their run (`"0[Red]0"` + 121 quoted works, -o A8);
  - 3 per unquoted or escaped literal character; `$` costs 1; a run of colons costs 3 once (see "Colon runs");
  - 2 per quoted character;
  - 2 per other token (`d`, `mm`, `%`, `@`, `[h]`, ...); `AM/PM` costs 11 to 13 and `A/P` 8 to 10 (the engine uses 11 and 8; see "`A/P` letters and a locale-only `[$-409]`", T-1714);
  - 2 per color (`"[Red]0"` + 120 quoted works, 122 fails; `"[Red]"` + 124 quoted works, 125 fails);
  - 6 to 8 per condition (`"[>5]0"` plus 79 spaces works, 80 fails; the engine uses 7), 3 more in a section that also holds quoted text and a placeholder (without a placeholder it is not paid: `"d"` + 119 quoted + `"[>5]d"` works; `"[>5]0"` + 116 quoted works and 118 fails on 7 and on 1; `"[>5]"` + 120 quoted works, 122 fails);
  - 0 per `;`.

  Examples: `"d"&REPT(" ",82)` (248) works and `"d"&REPT(" ",83)` (251) fails; `REPT("-",83)` (249) works and `REPT("-",84)` fails; `"d"&REPT("$",160)` works and `REPT("$",250)` fails; `"0"&REPT(" ",40)&";0"&REPT(" ",40)` (250) works and 41/40 fails; `"0;0;0;"&REPT(" ",78)` works.
- **Quoted text depends on its neighbours** (T-1434, T-1449, T-1455; probes/text-format-length-limit-f to -k):
  - **What follows quoted text** (T-1486; probes/text-format-length-limit-n, -o, -p): a token right after it pays 4 (below); a token after unquoted text that follows the quoted text pays 2 (`"a" @` + 120 quoted works and 121 fails, `"a"-@` + 122 works and 123 fails, `"a" d` + 121 works and 122 fails, `"a"--d` + 120 works and 121 fails, a space + 120 quoted + `d` works and 121 fails); a token after a color or condition pays 1 (`"d"` + 121 quoted + `"[Red]d"` works and 122 fails, + 119 quoted + `"[>5]d"` works and 120 fails, -q A16:A19); a placeholder run pays only 1 right after it or after unquoted text (`"a" 0` + 119 quoted works and 120 fails, 121 quoted + `0` works, `"0"` + 121 quoted + `"0"` works) and nothing after a color or condition (`"0"` + 120 quoted + `"[Red]0"` works and 121 fails, 121 quoted + `"[Red]0"` works and 122 fails, `"0"` + 116 quoted + `"[>5]0"` works and 117 fails).
  - **Quoted text followed by more** pays 4 extra when a token follows it directly, or when the literal run that closes the section goes on with unquoted text after it: 122 quoted + `d` works and 123 fails; `"d"` + 121 quoted + `d` works and 122 fails; 124 quoted + `@` (on text) fails; `"d"` + 120 quoted + 2 spaces works and 3 fails, + 122 quoted + 1 space fails, 110 + 8 spaces works and 9 fails, 100 + 15 works and 16 fails, 50 + 48 works and 50 fails; 120 quoted + 2 dashes works and 3 fails. Unquoted text that is followed by more quoted text or by a token does not pay it: `"d"` + 61 quoted + a space + 61 quoted works (247), `"d"` + 120 quoted + `" d"` works (247); nor does unquoted text before quoted text (50 spaces + 50 quoted works, 52 fails).
  - **Date, time and text tokens are free** in a section whose closing literal run holds quoted text: `"d"`, `"dd"`, `"m"`, `"h"`, `"yyyy"` or `"@"` + 125 quoted works like 125 quoted alone, and + 126 fails; `"d "` or `"dd "` + 123 quoted works and 124 fails; `"h:mm"` + 123 quoted works (both `h` and `mm` free); `"0;d"` + 122 quoted works and 123 fails. They keep their 2 before unquoted text (`"d"&REPT(" ",83)` fails) and before quoted text that a token follows (above). Elapsed tokens, `AM/PM`, `%` and placeholders are not free: `"[h]"` + 124 quoted works and 125 fails, `"%"` + 124 works and 125 fails, after `0`, `#`, `?` or `00` quoted text stops at 122, after `0.0` or `0 ` at 121. Seconds, `ddd`, `dddd`, `mmm`, `mmmm` are free too (`"ss"`, `"ddd"`, `"mmm"`, `"dddd"`, `"mmmm"` + 125 quoted works, 126 fails; probes/text-format-length-limit-l A27:A38); `A/P` is not (+ 125 quoted fails, A33; the engine already treats it like `AM/PM`). **`@` is refunded 1 more than the others** (T-1481): `TEXT("x","@ "&<124 quoted>)` works and 125 fails (`"@-"` and `"@\-"` the same), `"@  "` (two spaces) + 122 quoted works and 123 fails, `"@ - "` + 121 works and 122 fails, `" @ "` + 122 works and 123 fails (probes/text-format-length-limit-l A1:A3, A7:A9, A16:A18, -m), while `"d "` + 124 fails; `@` + unquoted text alone (no quoted) is unchanged (`"@"` + 82 spaces works, 83 fails). On a number the section prints General and the same limit applies (`TEXT(5,"@ "&<124 quoted>)` is "5", 125 is #VALUE!).
- **Two conditions in one section** are `#VALUE!` whatever the value and length: `TEXT(7,"[>5][<9]0")`, `TEXT(1,..)`, `TEXT(10,..)`, `"[>5]0[<9]"` (probes/text-format-length-limit-j A31:A34; also g A37:A41; only the first section is probed, the engine applies the rule to every section, inferred). A color with a condition works in either order (`"[Red][>5]0"` and `"[>5][Red]0"` on 7 give "7"). Next to spaces a condition costs 6 to 8 whatever its threshold (`"[>55555]0"` plus 79 spaces works and 80 fails, like `"[>5]0"`).
- **An unterminated code is `#VALUE!`**, whatever the value and whichever section it would use: `TEXT("x","0""")`, `TEXT("x","""")`, `TEXT("x","0;0;0;""")`, `TEXT("x","@;""")`, `TEXT(5,"0;""")`, `TEXT(-5,"0;""")`, `TEXT("x","0;[Red")` are `#VALUE!` (an unclosed quote or bracket; probes/text-format-length-limit-q A4:A13; T-1486). So are codes left with an unclosed quote after `_` or `*`, which take the next `"` as their character (`@_"a"`, `@_""`, `@*""`, `@"`, and `@_"` + 123 to 125 quoted characters; -n A3, A5, -o A36, A41, -l A13:A15). A valid code without a text section still leaves text unchanged: `TEXT("x","0:a:")`, `"0\x"` and `"0 a"` are "x" (-q A1:A3). A code ending in `\`, `_` or `*` is `#VALUE!` the same way: `TEXT("x","0\")`, `"0_"`, `"0*"`, `"@\"`, `TEXT(7,"0_")`, `TEXT(5,"0;0\")`, `TEXT(5,"0;0_")`, `TEXT(-5,"0;0\")` (-s). Only unterminated codes (a quote or bracket left open, a `\`, `_` or `*` at the end) are probed; the engine rejects exactly those (`is_unterminated` in formatter/complexity.rs), not every code our parser fails on. Other parse failures in a section the value does not use (e.g. `TEXT(5,"0;0 n")`: our lexer rejects `n`, as Excel does in a section it uses, probes/text-format-unquoted-letters A14, A40) are unprobed.
- **Text section with a number:** a single section holding `@` does not apply to a number, which prints in General: `TEXT(1,"@"&""""&REPT("a",118)&"""")` is "1" (probes/text-format-length-limit-i A19:A26).
- **Fourth section:** a fourth section holding a number or date token is `#VALUE!` whatever the value: `TEXT(1,"0;0;0;0")`, `"0;0;0;d"`, `"d;0;0;0"`, `"0;d;0;0"`, `"0;0;d;0"`, `TEXT(-1,"d;d;d;d")`, `TEXT(0,..)`, `TEXT("x",..)`. `"d;d"` and `"d;d;d"` give "1". This explains `"d;d;d;d"`. Only `d` and `0` are probed there; the engine extends the rule to every digit, date, time, elapsed, AM/PM and `E+` token (inferred).
- **Error precedence:** an error value wins over a too-complex code: `TEXT(NA(),REPT("dm",17))` is `#N/A`, `TEXT(1/0,"0;;;;")` `#DIV/0!`. Over an array, each element is `#VALUE!` except an error element, which keeps its error (`TEXT({1,#N/A},REPT("dm",17))` is `#VALUE!`, `#N/A`). An error inside an array of formats is that element's result (`TEXT(1,{"0",#N/A})` is "1", `#N/A`).
- **Colon runs:** consecutive colons print as one, whatever the run's length: `TEXT(1,"d"&REPT(":",n))` is "1:" for n from 2 to 85, and `"0"&REPT(":",n)` is "1:" for n from 2 to 84. A colon run broken by anything else prints in full: `"0:a:"` gives "1:a:", `"0::a"` "1:a", `"0: :"` "1: :", `d:"":` "1::". This is a formatter rule, not a cost rule. Escaped colons do not join a run (`TEXT(1,"0\:\:")` = "1::") and `h::mm` is "12:00" (probes/text-format-unquoted-letters A105:A111, T-1450; implemented T-1262).

Evidence: probes/text-date-255-char-limit, text-format-length-limit, text-format-length-limit-b (T-1388), text-format-length-limit-c, -d, -e (T-1393), -f, -g, -h, -i (T-1434), -j (T-1449), -k (T-1455), -l, -m (T-1481), -n to -s (T-1486), all excel-graph.
Not explained or not implemented (known failures):
- Unquoted letters after `_"` or `*"`: `_` and `*` take a following `"` as their character, so `@_"a` is "x a", `@_"aa` "x aa", `@_"a-"b"` "x a-b", `@*"a` "xa", `0_"a` on 7 "7 a" and `0*"a` "7a" (probes/text-format-length-limit-n A1, A6, -o A35, A37:A40; matched since T-1262 prints bare letters, except -o A37: `*` takes the `"` as its fill, which adds nothing in a number section, but we print a space for every fill, giving "7 a"; T-1735). `@_"aaa` is `#VALUE!`, and so are 20, 40 and 80 letters (n A2, A7:A9). We print the letters for those ("x aaa"). Unexplained: `aaa` may read as a weekday code, which T-1656's `@`-with-a-date-token rule would reject (T-1880).
- `TEXT(1,"[>5]0"&REPT(" ",79))` is "1" (also with 40 spaces): an unmatched single condition prints General, dropping the padding. So is `"[>55555]0"` plus 79 spaces (-j A43), and `TEXT(1,"d"&<119 quoted>&"[>5]d")` is "1" (-q A16). We do not apply conditions yet (see "Conditional sections", T-0936).
- `TEXT(1,"0:a:")` is "1:a:" and an escaped backslash followed by an unquoted `x` prints "\x": Excel accepts these letters unquoted (-g A42, A43; -i A100:A104). T-1450; implemented T-1262.

Cell display with a long format could not be probed: excel-graph rejects such workbooks (createSession 501 unsupportedWorkbook). The colon-run and text-section rules live in the shared formatter, so they apply to cell display too; they are probed through TEXT only.
Ours: `formatter/complexity.rs` (`rejects_format_code`: `exceeds_complexity_limit` plus `is_unterminated`), applied by TEXT only; colon runs in `formatter/parser.rs`; the text section with a number in `format_number` (formatter/format.rs).
Status: VERIFIED (excel-graph, fitted model)
Tasks: T-1388, T-1393, T-1434, T-1449, T-1450, T-1451, T-1455, T-1481, T-1486, T-1735, T-1880

TEXT over an array of format codes (T-1451): one result per format, value and formats broadcast together (see text.md). `TEXT(1,{"0",#N/A})` is "1", `#N/A` (-f A54:B54). The error-precedence and lifting rules are verified by probes/text-format-array (T-1458) for error and too-complex format elements; a syntactically invalid format element next to an error value is not probed.

## Unquoted ASCII letters in a number format (T-1450)

Excel prints most ASCII letters that are not format tokens as literals, with no quotes or backslash. Probed as `TEXT(1,"0"&L)` and `TEXT(1,"d"&L)` for every ASCII letter in both cases (probes/text-format-unquoted-letters A1:A104, excel-graph 2026-10-14). This refines "Bare letters and digits in a number format" above: a date token next to a digit placeholder is an error, but with no digit placeholder it is a date token (`TEXT(100,"ABC")` = "A43C"; `TEXT(1,"0B")` is `#VALUE!`).

- **After `0`:** the letters `b d e g h m n s y` and their capitals (`B D E G H M N S Y`) are `#VALUE!` (date/time tokens do not fit next to a digit placeholder; the cause for `n` and `N`, which are not en-US date tokens, is unknown). Every other letter prints as a literal: `TEXT(1,"0a")` is "1a", and likewise `c f i j k l o p q r t u v w x z` and `A C F I J K L O P Q R T U V W X Z`. `TEXT(1,"0x")` is "1x".
- **After `d`** (a date format), the date tokens keep their meaning. The output is the day "1" followed by the token's value: `db` "143" (b is the two-digit Buddhist year, "43"), `dd` "01", `de` "11900" (e is the year 1900), `dg` "1" (g is the era, empty), `dh` "10", `dm` "11", `ds` "10", `dy` "100" (y is "00"). `dD`, `dG`, `dH`, `dM`, `dS`, `dY` match the lowercase. `dB`, `dE`, `dn` and `dN` are `#VALUE!`. The remaining letters (`a c f i j k l o p q r t u v w x z` and capitals `A C F I J K L O P Q R T U V W X Z`) print as literals ("1a").
- **Colons** (A105:A111): `TEXT(1,"0\:\:")` is "1::" (escaped colons do not collapse); `TEXT(0.5,"h::mm")` and `"h:::mm"` are "12:00" (a run collapses to one); `"0::"` and `"0:::"` are "1:"; `"0:a:"` is "1:a:" and `"0::a"` is "1:a".

Evidence strength: excel-graph oracle, 111 cells.
Implemented (T-1262): all 111 cells pass; escaped colons are `Token::Escaped` and do not join a colon run.
Status: VERIFIED (excel-graph)
Tasks: T-1262, T-1450

## TEXT of a text value: `@` with other tokens in one section (T-1656)

Rule: a format whose section holds `@` is `#VALUE!` (for a text value, and for numbers too: see Sections below) when the same section also holds a date or number token, whichever side of the `@` it sits on: `y yy yyyy m mm mmm d dd ddd h hh s ss b bb e ee g gg ggg`, `[h] [m] [s]`, `0 # ? . /`, `E+0`, `General`, `%` and a condition such as `[>5]`. These are literals, not errors: `a`, `,`, space, `\y`, `"y"`, `*-`, `_x`, `[Red]`, `[$-409]` (`TEXT("abc","x@\y")` is "xabcy"; `"x@_x"` prints a space for the spacer). `A/P` and `AM/PM` give a stray first letter or two: `TEXT("abc","x@A/P")` is "xabcA" and `"x@AM/PM"` "xabcAM" (`"A/P@"` "Aabc", `"AM/PM@"` "AMabc"). `@@` prints the text twice. A fill `*x` prints nothing in the text section (`"x@*-"` is "xabc") while `_x` leaves a space.
Sections (probes/text-at-token-sections, 96 cells: text, 1, -1 and 0 on each format): `@` may stand only in the last section, and a format that breaks either rule is `#VALUE!` for every value, numbers included. `"@;0"`, `"x@;0"`, `"[Red]x@;0"` and `"x@\y;0;0;@"` are `#VALUE!`; `"x@y;0"`, `"0;x@y"`, `"0;0;x@y"`, `"x@y;0;0;@"`, `"0;0;x@y;@"` and `"0;x@0"` are `#VALUE!` because one section holds `@` beside a token. A last section with `@` is the text section whatever the count: `TEXT("abc","0;x@")` and `"0;0;x@"` are "xabc", and numbers use the sections before it (`TEXT(-1,"0;x@")` is "-1", `TEXT(-1,"0;0;x@")` "1", `TEXT(0,"0;0;x@")` "0"). `"0;0;0;y"` stays `#VALUE!` (4th-section rule) and `"0;0;0;x@\y"` gives "xabcy". This reconciles the poi 4th-section entry: literals in a text section print, but a number or date token beside `@` invalidates the format.
Evidence: probes/text-at-with-tokens (89 cells), probes/text-at-token-sections (96 cells) (excel-graph, 2026-10-16, T-1656).
Implemented in: `exceeds_complexity_limit` (formatter/complexity.rs, both placement rules, checked by TEXT before formatting); `format_text_value` and `format_number` (formatter/format.rs: a trailing `@` section, `*x`, `AM/PM` as "AM"). `A/P` and `[$-409]` pass since T-1702. The same formatter paths serve cell display, where the trailing `@` section and `*x` follow TEXT's probed behavior (inferred for display: unprobed, and a display fill would repeat to the column width).
Status: VERIFIED (excel-graph)
Tasks: T-1656

## `A/P` letters and a locale-only `[$-409]` (T-1702)

Rule: `A/P` prints one letter, each exactly as typed: the first letter (`A`, `a`) before noon and the second (`P`, `p`) after, so case is per half and mixed forms work (`h A/p` is "6 A" at 0.25 and "6 p" at 0.75; `h a/P` is "6 P" at 0.75). Like `AM/PM` it switches hours to the 12-hour clock, and in the text section it prints the morning letter (`x@a/p` "xabca", `x@A/p` "xabcA", `x@a/P` "xabca"). `a/` followed by anything but `p` is literal text in a date format (`h a/x` at 0.75 is "18 a/x"; the 24-hour clock stays) and `#VALUE!` beside `@`. `[$-409]` (a locale with no currency symbol) prints nothing in front of or after any code: `[$-409]0` of 5 is "5", `[$-409]0.00` "5.00", `[$-409]d` of 45000 "15", `[$-409]yyyy-mm-dd` "2023-03-15", `[$-409]mmmm` "March", `[$-409]dddd` "Wednesday", `[$-409]@`/`General` of 5 "5", `0[$-409]` "5".
Evidence: probes/text-ap-locale-bracket (28 cells), probes/text-at-with-tokens A22, B22, A35, probes/text-at-token-sections A20 (excel-graph, 2026-10-16).
Implemented in: `Token::AP` and `Token::Locale` (formatter/lexer.rs), `TextToken::AP` (parser.rs, format.rs).
Status: VERIFIED (excel-graph)
Tasks: T-1702, T-1714

T-1714 probed the inferences (probes/text-ap-locale-inferences, 81 cells; probes/displayed-text-ap-locale, 8 cells; excel-graph, 2026-10-16):
- **A locale tag is not silent.** `[$-zzz]0`, `[$-]0` and `[$-40C]0`/`0.00` print as without the tag, but `[$-40C]mmmm` on 45000 is "mars", `[$-407]mmmm` "März", `[$-40C]dddd` "mercredi", and `[$-F800]dddd` the system long date "Wednesday, March 15, 2023". See "Locale tags localize date names" (T-1716).
- **`AM/PM` may stand in a fourth section**, with or without `@`: `TEXT(5,"0;0;0;x@AM/PM")` is "5", `TEXT("abc","0;0;0;x@AM/PM")` "xabcAM", `TEXT(0.75,"0;0;0;AM/PM")` "1". `"0;0;0;0"` stays `#VALUE!`. `A/P` there: `TEXT(5,"0;0;0;A/P")` is "5", but `TEXT("abc","0;0;0;A/P")` is "A" (a text value prints the morning letter alone without `@`; A13:A14) and `"0;0;0;x@A/P"` gives "xabcA" (A15). `AM/PM` with a text value and no `@` was not probed. Implemented: `exceeds_complexity_limit` exempts `AM/PM`.
- **Length-limit costs** (text, 0.75 or 5 as the value): `AM/PM` + 79 spaces works and 80 fails, `AM/PM` + 120 quoted works and 121 fails; `A/P` + 80 spaces works and 81 fails, `A/P` + 122 quoted works and 123 fails; `[$-409]` + 118 quoted works and 119 fails, `[$-409]` + 78 spaces works and 79 fails. The probes pin `AM/PM` to 11 to 13 and `A/P` to 8 to 10 (the engine uses 11 and 8, both refunded 2 before a closing quoted run) and `[$-409]` to 14. The cost of other tags is inferred from `[$-409]` (7 characters at 2), not probed. The earlier "`AM/PM` costs at least 3" fit was wrong.
- **Cell display of `A/P`** equals TEXT's (`h A/P`, `h a/p`, `A/P`, `h:mm a/P` on cells; probes/displayed-text-ap-locale A1:A4).

## Locale tags localize date names (T-1716)
Rule: a `[$-tag]` with no currency symbol prints nothing, but in its own section it sets the language of `mmm`, `mmmm`, `mmmmm`, `ddd`, `dddd` and of the day-half strings, from Excel's locale data rather than CLDR: `[$-407]mmm` of March is "Mrz" (de-CH `[$-807]` "Mär", `ddd` "Mi."), `[$-40A]ddd` "mi.", `[$-405]mmm` Roman ("III"), `[$-411]mmm` "3". `mmmmm` is the first character of the month name (`[$-804]` December "十", `[$-411]` December "1"). Numbers, separators and `yyyy`/`d` are not localized (`[$-40C]0.00` "1234.50", `[$-40C]dd/mm/yyyy` "15/03/2023"). Under a resolved tag `A/P` and `a/p` print the locale's `AM/PM` strings, so `[$-409]h A/P` is "6 PM" and `[$-411]h a/p` "6 午後" (Western European, Italian, Russian, Polish, Nordic Danish/Swedish, Dutch and Portuguese stay "AM"/"PM"; `[$-414]` "a.m."/"p.m.", `[$-40B]` "ap."/"ip.", `[$-405]` "dop."/"odp.", `[$-40E]` "de."/"du.", `[$-41F]` "ÖÖ"/"ÖS", `[$-412]` "오전"/"오후", `[$-804]`/`[$-404]` "上午"/"下午"). An unresolved tag (`[$-zzz]`, `[$-]`, `[$-FFFF]`) keeps the letters.
- **Resolution.** Hex LCIDs are case-insensitive and may carry a calendar digit above the low four (`[$-1040C]`, `[$-0040C]` are French). An LCID with no locale of its own falls back to its language (inferred; it holds only where the variants share names, so every regional variant that differs has its own table: see "Regional variants" below; T-1858) (`[$-80C]`, `[$-C]` French, `[$-7]` German, `[$-A]` Spanish, `[$-11]` Japanese, `[$-16]` Portuguese) and `[$-4]` is Traditional Chinese. 400 and 800 are en-US (A/P becomes AM/PM); `[$-0]`, `[$-40C0]`, `[$-2000000]`, `[$-FFFF]` and `[$-40C,1]` print English. BCP-47 tags match case-insensitively and fall back to their language: `[$-fr-FR]`, `[$-FR-fr]`, `[$-fr]`, `[$-fr-XX]` are French, `[$-de]` German (not LCID 0xDE), `[$-zh]` Traditional Chinese, `[$-xx]` English. Underscore tags select calendars: `[$-fr_FR]mmmm` of 45000 is "December" (see "Calendars of a locale tag"; T-1924).
- **System tags.** `[$-F800]` and `[$-x-sysdate]` replace the whole section with the system long date, en-US "dddd, mmmm d, yyyy", whatever else it holds (`TEXT(45000.75,"x[$-F800]0")` is "Wednesday, March 15, 2023", 0.5 "Saturday, January 0, 1900", -1 and 3e6 `#VALUE!`); `[$-F400]`/`[$-x-systime]` likewise give "h:mm:ss AM/PM". Only a first section shows it: `TEXT(-1,"0;[$-F800]")`, `TEXT(0,"0;0;[$-F800]")` and `TEXT("abc","0;0;0;[$-F800]")` are "". With `@` the section is a text section as usual (`[$-F800]@` of 45000.75 is "45000.75").
- **One tag per section.** A second tag in a section is `#VALUE!`, even the same one or a system tag (`[$-40C]mmmm [$-40C]mmmm`, `[$-40C]mmmm [$€-407]mmmm`, `[$-F800][$-F400]`); tags in different sections are fine.
- **Regional variants and more languages** (probes/text-locale-regional-lcids, 252 cells, T-1858). The language fallback does not hold across regions: `[$-C07]` de-AT has "Jän"/"Jänner" for January, "Mär" for March, and `ddd` "Mi."/"Do." (de-DE differs in all three); `[$-1004]` zh-SG mixes the two: short months as zh-TW ("一月"), days as zh-CN ("周三"); `[$-C04]` zh-HK equals zh-TW ("週三"); `[$-80A]` es-MX and `[$-2C0A]` es-AR differ from es-ES in `ddd` and the day halves ("a. m."/"p. m."), and es-AR's halves use a non-breaking space where es-MX's use a plain one. Languages that now resolve: `[$-408]` Greek ("Ιαν", "πμ"/"μμ"), `[$-41E]` Thai (Thai names, AM/PM stay Latin), `[$-41B]` Slovak (short months are the numerals "1"…"12", like Japanese), `[$-422]` Ukrainian (AM/PM stay Latin), `[$-401]` Arabic ("ص"/"م"), `[$-40D]` Hebrew (AM/PM stay Latin), `[$-439]` Hindi ("पूर्वाह्न"/"अपराह्न"). Calendar and digit switches of Thai, Arabic and Hebrew are T-1859's.
- **Not yet modelled:** locales not in the table list print English and keep `A/P` letters, though Excel very likely localizes them. Eras and calendar digits are in "Calendars of a locale tag" below.
- **A currency tag with a locale** (`[$€-40C]`, `[$€-fr-FR]`) is a symbol plus a locale tag in one: the symbol prints where it is written in a date section (`[$€-40C]d mmmm` "€15 mars", `d [$€-40C]mmmm` "15 €mars", `mmmm[$€-40C]` "mars€", `dddd [$€-40C]` "mercredi €", `[$€-40C]yyyy` "€2023", `[$€-40C]h:mm AM/PM` "€6:00 PM", `[$€]mmmm` "€March") and the locale localizes the section like `[$-40C]`. It counts as the section's one tag (`[$€-40C]mmmm [$-40C]mmmm` is `#VALUE!`; tags in different sections are fine). A system locale drops the symbol (`[$€-F800]` is the system long date). A multi-character symbol with a locale (`[$USD-40C]mmmm` is "USDmars") is not modelled yet (T-1883). Evidence: probes/text-currency-locale-date (excel-graph, 2026-10-18, T-1865).
- The cell display path shows the same text (probes/displayed-text-locale-names).
Evidence: probes/text-locale-date-names (124 cells), probes/text-locale-name-tables (601), probes/text-locale-am-neutral (47), probes/displayed-text-locale-names (5), probes/text-locale-regional-lcids (252) (excel-graph, 2026-10-18). Morning strings of locales whose afternoon is "PM" are inferred as "AM" beyond fr, de, de-AT, th, sk, uk and he (probed).
Implemented in: locale/excel_date_names.rs (tables), formatter/locale_tag.rs (resolution), `Token::Locale` (lexer.rs), `DatePart::names` and `Parser::system_part` (parser.rs), format.rs.
Status: VERIFIED (excel-graph)
Tasks: T-1716, T-1858, T-1865

## Calendars of a locale tag (T-1859)
Rule: a hex locale tag `[$-NNCCLLLL]` holds a numeral system (NN), a calendar (CC) and the LCID (LLLL). The calendar decides what `yy`/`yyyy`, `b`, `e` and `g` print and may override the month and day names. `AM/PM` always follows the LCID. Values below are for serial 45000 (2023-03-15).

**Japanese eras.** They apply under calendar 3 with any LCID, and under a Japanese LCID's own calendar (0, or `[$-ja-JP]`, `[$-ja]`, `[$-11]`).
- `g`, `gg` and `ggg`+ print the era's letter, first character and name: "R|令|令和". Longer runs print the name.
- `e` prints the era year ("5"). `ee`+ prints it with at least two digits ("05"; "7981" at 9999-12-31).
- Eras start at serial 4595 (Taisho, 1912-07-30), 9856 (Showa), 32516 (Heisei) and 43586 (Reiwa, 2019-05-01). Earlier serials, 0 included, are Meiji ("M|33" for 1900).
- Under calendar 3, `yy` and `yyyy` print the era year ("05"). Under the Japanese LCID's own calendar they do so only after a `g` earlier in the section: `[$-411]yy|g|yy|yyyy` is "23|R|05|05" and `[$-411]e|yy|yyyy` is "5|23|2023". A `g` written before the tag counts too (`g[$-411]yyyy` is "R05").
- Calendar 1 is Gregorian even for Japanese. Under a Japanese LCID with a non-Japanese calendar, a `g` earlier in the section makes `yy` and `yyyy` print the whole Gregorian year: `[$-10411]g|e|yy` is "|2023|2023" and `[$-70411]g|yyyy` is "|2023" (Thai would otherwise print 2566). Other LCIDs are unaffected: `[$-409]g|yy` is "|23", `[$-70409]g|yy|yyyy` "|66|2566", `[$-50409]g|yy|yyyy` "|56|4356", and with no tag `g|yy` is "|23" (probes/text-locale-era-gregorian). Calendars 5 and 9 under a Japanese LCID (`[$-50411]g|yyyy`, `[$-90411]g|yy`) are inferred to follow the same rule; they are not probed.
- Names under calendar 3 are Japanese for a Japanese LCID and en-US for any other (`[$-3040C]mmmm` is "March").

**Other calendar digits.**

| Digit | Calendar and names | Example |
|---|---|---|
| 2, D | Gregorian, en-US names | `[$-20411]mmmm` is "March" |
| 4 | Gregorian, zh-TW names; `yy` prints four digits | `[$-40409]ddd\|mmm\|yy` is "週三\|三月\|2023" |
| 5 | Korean years (Gregorian + 2333) for `yy`/`yyyy`; Korean names under a Korean LCID, en-US otherwise | "4356" |
| 7 | Thai years (+ 543) for `yy`/`yyyy` and `b`; Thai names | "2566", "มีนาคม" |
| 9 | Gregorian, French names | "mars", "mer." |
| 15 and above | ignored; the LCID's own calendar (probed 15, 1E, 1F, FF) | `[$-FF040C]mmmm` is "mars" |

Under every one of these, `e` stays the Gregorian four-digit year, and so do `b` and `bbbb` except under Thai. Day halves follow the LCID under every calendar digit: `[$-20411]`, `[$-40411]` and `[$-50411]h AM/PM` are "6 午後", and `[$-70412]`/`[$-90412]` give "6 오후".

**Not modelled** (the tag prints English and keeps `A/P`):
- 6 is Hijri ("1444-08-23", "Sha'ban", or French "Chaaban" under 40C; T-1613).
- 8 is Hebrew ("5783-06-22", "Adar", "Yom Revi'i").
- 10 is Saka ("1944-12-24", "Phalguna", "Budhavara").
- A is Gregorian with Levant Arabic names ("آذار"); B and C give "مارس".
- 14 is Gregorian with native Korean names ("삼월").
- E, F, 11, 12 and 13 are the Chinese lunar date: month 2, day 24, `mmmm` "二月", the first month "正月". `yyyy` and `e` stay 2023. E gives Japanese weekdays, F and 11 Chinese ones, 12 and 13 English names (T-1922; the others are T-1923).
- Numeral digits `[$-8000411]e` print native digits ("୫"; T-1925).
- Underscore tags pick calendars: `[$-fr_FR]` gives the Saka date with English Gregorian month names ("December|1944-12-24"); `[$-de_DE]`, `[$-en_US]` and `[$-ja_JP]` give the lunar date in English; `[$-_]` gives Russian names (T-1924).

**`b` and `B1`/`B2` under a tag.** With no tag, `b` is the Buddhist year ("66|2566"). Under any resolved tag it is the Gregorian year ("23|2023"), except under calendar 7. `B1` and `B2` after a tag change nothing: `[$-40C]B2mmmm` is "mars" and `[$-401]B2yyyy-mm-dd` is "2023-03-15". An unresolved tag ignores `B2` too: `[$-zzz]B2mmmm` is "March", and `[$-FFFF]B2yyyy` and `[$-]B2yyyy` are "2023". The Hijri calendar digit is not ignored: `[$-60409]B2yyyy` is "1444". `B2` before a tag still selects Hijri: `B2[$-40C]mmmm` is "Chaaban" (T-1613).

Evidence: probes/text-locale-calendars (145 cells), text-locale-calendars-detail (41), text-locale-japanese-year (20), text-locale-era-gregorian (23) (excel-graph, 2026-10-18).
Implemented in: formatter/locale_tag.rs (`resolve_lcid`, `Calendar`, `japanese_era`), format.rs (years, `e`, `g`).
Status: VERIFIED (excel-graph)
Tasks: T-1859, T-1613, T-1922, T-1923, T-1924, T-1925

## Displayed text (`Range.text`) of a formatted cell (T-1610)
Rule: excel-graph's `Range.text` is the cell's value under its number format, and it does not
depend on the column width. At width 4 a General `12345678` stays "12345678" and `=1/3` stays
"0.333333333" (the 11-character General rule of `TEXT(x,"General")`, not a width-fitted
shortening), and text is not cut ("long text"). A number a date format cannot show (a negative date
serial) is 255 `#` characters, not a width-sized `####`. The other probed cells:
General `1.5` "1.5", `1234567890123` "1.23457E+12", `1E-10` "1E-10"; `0.00` on PI() "3.14";
`#,##0.00` on -1234.5 "-1,234.50"; `0%`/`0.0%` on 0.256 "26%"/"25.6%"; `yyyy-mm-dd` on 45000
"2023-03-15"; `m/d/yyyy h:mm` on 45000.75 "3/15/2023 18:00"; `0.00;(0.00)` on -5 "(5.00)";
`0.00;-0.00;"zero"` on 0 "zero"; `dddd` on DATE(2024,2,29) "Thursday"; `# ?/?` on 12.5 "12 1/2";
`0.00E+00` on 123456789 "1.23E+08"; `[h]:mm:ss` on 0.5 "12:00:00"; `$#,##0.00` "$3.00"; TRUE
"TRUE"; `#DIV/0!` "#DIV/0!"; an empty reference "0"; `TEXT(1234.5,"#,##0")` "1,235".
Evidence: probes/displayed-text-formats S!A1:A24, B1:B4 (excel-graph `text`, 2026-10-16, T-1610).
Desktop `.Text` does depend on the width (`####` when a number does not fit), unprobed here: the
excel-graph text is what the M10 suite scores (decision 0068).
Not implemented: ours gives " 1/2" for `# ?/?` on 12.5 (T-2030), "1,234" for
`TEXT(1234.5,"#,##0")` (a value failure too) and `#VALUE!` for the negative date (both T-1711).
Status: VERIFIED (Excel for the web)
Tasks: T-1610, T-1711, T-2030

## Displayed text over an edge-value pool (T-1611)
Rule: suite gen-m10 records Excel's displayed text (`Range.text`) of 135 format codes (every built-in,
custom codes with 1-4 sections, conditions, colors, literals, `*`/`_`, `@`, elapsed and fractional
seconds, fractions, scientific, scaling commas, percent) over 44 edge values, 10,400 cells, plus
TEXT() with the same codes. Points the earlier probes did not show:
- A `*x` fill adds nothing to the text: `_("$"* #,##0.00_);...` on 36.5 is " $36.50 ", `*.0.00` and
  `0.00*-` on PI() "3.14"; `_x` still adds one space (`_(* #,##0_)` on 1000 " 1,000 ").
- A date or clock format over a serial it cannot show (negative, or 1234567890 past 9999-12-31) is
  255 `#`; 787 such cells. An elapsed code with a clock or fraction token joins them (`[h]:mm`,
  `[m]:ss`, `[ss].00`, `[h]:mm:ss.0` on any negative, even -0.5, and on 1234567890), but a bare
  elapsed code does not: `[s]` on -1.5 is "-129600" and on 1E15 "86400000000000000000" (see
  "Elapsed-time tokens"). TEXT() on the same codes is `#VALUE!` exactly where the display is 255 `#`;
  every other paired TEXT() result equals the displayed text.
- A fraction whose numerator overflows is 255 `#` too: `?/100` on 1000, 999.9999 and 1234567890
  (numerator 100000 and up; 99.5 is "9950/100"), `?/?` on 123456789012345 and 1E15 while 1234567890
  is "1234567890/1". The threshold is unprobed (T-1740).
- Clock and date fields come from the serial rounded to the shown second: 0.9999999999 is "0:00"
  under `h:mm`, "24:00:00" under `[h]:mm:ss`, "1900-01-01" under `yyyy-mm-dd`, "Sunday" under `dddd`.
- `mm:ss` with no hour is minutes: 123456.789 is "56:10".
- The text section formats booleans as well as text: `"pre "@` on TRUE is "pre TRUE", `;;;` shows
  nothing, `0.0;[Red]-0.0;0.0;"<"@">"` on an empty string "<>".
- A minus goes before a leading literal: `"x"General` on -1.5 is "-x1.5". A value that rounds to
  nothing shows no minus: `#,` on -0.5 "", `0.0,,` on -1234.5678 "0.0"; `#,` on 1.5 is "" and on
  1000 "1".
- `?.??` keeps the point on an integer ("1000.  "); `#%` on 0.0049 is "%".
- Two conditions: `[<0]"neg";[>0]"pos";"zero"` on 1 is "pos"; `[=0]"nil";General` on PI() is
  "3.141592654"; `[<=-1]0;[<1]0.0000;0.0` on -1.5 "2"; `[>=1]0.00;0.000` on -0.5 "-0.500".
- `[$USD]` is currency text ("-1.50 USD") and `[Cyan]` a color.
- Engineering and padded mantissas: `##0.0E+0` on 123456.789 is "123.5E+3", `00.0E+0` on 36.5
  "36.5E+0", `0.000E-00` on 1.5 "1.500E00".
Evidence: eval/suites/gen-m10 (excel-graph `text`, 2026-10-16, T-1611); the report's "Displayed
text" section lists every cell ours gets wrong.
Not implemented: ours passes 6,961 of 10,400 text cells (66.9%). Fill characters (T-1735),
out-of-range dates (T-1731), fraction overflow (T-1740), second rounding (T-1732), `mm:ss` (T-1733), conditions (T-1734),
the text section on text and booleans (T-1736), `[$USD]`/`[Cyan]` (T-1737), `?.??`/`#%` (T-1738),
scientific (T-1730), fractions (T-2029, T-2030, T-0982; "Fraction formats" below), round-half (T-1711), the sign of -0.5 (T-1129),
fractional seconds (T-0981, T-0934), 15 significant digits (T-0980), `"x"General` (T-1614).
Status: VERIFIED (Excel for the web)
Tasks: T-1611, T-1711, T-1730 to T-1738, T-1740, T-2029, T-2030

## Fraction formats
Rule (gen-m10 `F!` text cells, TEXT() on sheet T gives the same text; cells named by case-id prefix):
- **Fixed denominator** (`# ?/4`, `# ??/16`, `?/100`): the numerator is the fraction part times the
  denominator rounded half away from zero, never reduced (`# ?/4` on 1.5 "1 2/4", 039accab F7; 0.125
  " 1/4", F12; `# ??/16` on 1.5 "1  8/16", 05c1c646 A3). A numerator that rounds up to the denominator
  carries into the integer (`# ?/2` on 1.75 "2    ", bfb1a4b0 S16).
- **Placeholder denominator** (`?/?`, `??/??`, `???/???`): the fraction is the last continued-fraction
  convergent of the fraction part whose denominator has at most that many digits, not the closest
  fraction: `# ??/??` on PI() is "3  1/7 " (4c6a49fe G13; 14/99 is closer) and on 123456.789 "123456 15/19"
  (20512106 L16; 71/90 is closer); `# ???/???` on PI() "3  16/113" (05c1c646 C18). Fitted during gap
  triage 2026-10-19 on all 317 gen-m10 text cells that show an n/d (the closest fraction fits 308);
  the stopping rule on binary noise is not pinned down (T-2030).
- **No integer part** (`?/?`, `?/100`): the whole value is the numerator, so the fraction is improper:
  `?/?` on 36.5 "73/2", PI() "22/7", 1234567890 "1234567890/1", 0 "0/1" (039accab E18, 43980980 Z14,
  039accab E2, 43980980 Z5); `?/100` on 61 "6100/100" (05c1c646 D5).
- **A zero numerator** blanks the fraction to spaces of its width and the integer prints even under `#`:
  `# ?/4` on 1E-5 "0    ", `# ??/16` on 61 "61      ", `# ?/2` on 0.125 "0    " (039accab F4, 05c1c646
  A5, 43980980 N10); `?/?` keeps "0/1".
- **A zero integer part** is left out under `#` with its space kept (`# ?/?` on 0.125 " 1/8", 20512106
  P1) and printed under `0` (`0 ?/?` on 1/3 "0 1/3", 0e3d8eba W4). The minus leads, also on a value that
  rounds to zero: `# ?/4` on -0.5 "- 2/4" (9a8b7d68 A1), `# ?/2` on -1/7 "-0    " (bfb1a4b0 S4), `0 ?/?`
  on -1/7 "-0 1/7" (71ad5f93 G7).
- **Padding:** `?` pads the numerator on the left and the denominator on the right (`# ???/???` on 1.5
  "1   1/2  ", 05c1c646 C3; `??/??` on 0.5 " 1/2 "); `#` does not (`# #/#` on 0.5 "1/2", on 1234567890
  "1234567890", 039accab K8, K2).
- A numerator that overflows is 255 `#` ("Displayed text over an edge-value pool" above, T-1740).
Evidence: eval/suites/gen-m10 (excel-graph `text`, 2026-10-16, T-1611); 829 numeric text cells under 12 fraction
codes, all failing in the 2026-10-19 report.
- **The numerator rounds at 15 significant digits**, as ROUND does: `?/100` on 12.345 is "1235/100" (05c1c646 D11)
  (12.345 × 100 is 1234.4999999999998 in binary), while `# ?/10` on 12.345 is "12 3/10" (43980980 P16; the binary
  fraction part 0.34499999999999886 × 10 stays below the half).
Ours: fixed denominators are implemented as above (T-2029, formatter/fraction.rs), in TEXT and the
displayed text. Unprobed and implemented by inference (T-2061): the minus before a currency symbol
or leading literals (`$# ?/4` on -0.5 is "-$ 2/4"), `%`/`,` scaling before the fraction, `#`/`0`
numerator padding, "-0/100" for a tiny negative, and no grouping (`#,##0 ?/4`). Placeholder
denominators still print `/` as a literal with plain digits around it (`# ?/?` on PI() is "  /3";
T-2030), and an overflowing numerator prints in full (T-1740).
Status: VERIFIED (Excel for the web)
Tasks: T-1611, T-2029, T-2030, T-1740, T-0982, T-2061
