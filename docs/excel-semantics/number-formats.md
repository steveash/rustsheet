# Number formats (`TEXT` and cell formats)

All evidence below is from Apache POI's format test workbooks, scored by the excel-graph oracle
(sidecars in `eval/suites/poi/oracle/`, 2026-10-04) and checked against the values desktop Excel
cached in the same workbooks where it saved them. In each workbook `Tests!An` is
`=TEXT(Cn, Bn)` (TextFormatTests: `=TEXT(Cn, ";;;"&Bn)`), so `Bn` is the format and `Cn` the value.

## An empty format string returns empty text
Rule: `TEXT(<blank>, <blank format>)` returns `""`, not `#VALUE!`. Only a blank value has been
probed.
Evidence: poi/ElapsedFormatTests.xlsx A52:A215; poi/FormatChoiceTests.xlsx A27:A53; poi/GeneralFormatTests.xlsx
A69, A71; poi/NumberFormatApproxTests.xlsx A249; poi/FormatConditionTests.xlsx A38, A51:A54 (Excel-cached).
Unprobed: an omitted value argument (`TEXT(,"")`) and `TEXT(0,"")`.
Status: VERIFIED
Implemented in: `format_text` (formatter/format.rs) used by `fn_text` (functions/text/common.rs), blank value only; other values with `""` stay unprobed.
Tasks: T-0933

## Bare letters and digits in a number format
Rule: on a number, unquoted letters and the digits 1-9 in a format print as literal characters instead of making the
format invalid: `TEXT(PI(),"ü")` = "ü". With a decimal point but no digit placeholder the integer part seems to print
right before the point: `TEXT(1.005, TEXT(-1,"2.5"))` = "-211.5", consistent with `TEXT(-1,"2.5")` = "-21.5" and
`TEXT(1.005,"-21.5")` = "-2" & "1" & "1.5" (inferred from the one composite cell). An empty format on a number is not an
error: `INDIRECT(TEXT(-0.0001,""),..)` is `#REF!`, not `#VALUE!`. Not every letter is a literal: `TEXT(100,"ABC")` =
"A43C", so `B` prints "43", which fits a two-digit Buddhist-era year (serial 100 is 1900-04-09; 1900 + 543 = 2443).
Evidence: gen-m6/2c1a6e93a1974f48 Calc!A15, gen-m6/63012866a9d8b7db Calc!A3, gen-m6/17223361bef5b5cb Calc!A12
(excel-graph, 2026-10-09, T-1239); gen-m7/6db767d720c5b55a Calc!A30 `TEXT(Data!A3,"ABC")` with A3 = 100 (excel-graph, 2026-10-11, T-1275).
Ours: `#VALUE!` for all four (the lexer rejects these formats).
Status: UNVERIFIED (probe pending: which letters are tokens, the integer-before-point rule, the value of `TEXT(x,"")`)
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
Unprobed (T-0935 follow-up): a text value with a 1-3 section format and no lone `@` section (the
engine returns the text unchanged), `0;@`-style formats, `_x`/`*x` in the text section (one space
here), digit or date tokens in the text section (dropped here), and a General 4th section.
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
In the 1900 date system TEXT of a negative under any format containing an elapsed token is #VALUE! instead (dates.md, probes/text-negative-serial-elapsed-mixed, T-1079/T-1105); the minus above is the 1904-workbook behavior.
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
Single-digit ties go to fixed (A27: "0.000000001" against "1E-09"). Unprobed (T-1225): multi-digit
ties (`0.000123456789`, `1.2345E-05`: the engine prints fixed)
and fixed values from 1 on whose integer digits plus 9 decimals exceed 11 characters (the engine
keeps 9 decimals, `123456.123456789` → "123456.123456789"; if the 11-character limit holds there
too it would be "123456.1235").
Evidence: poi/GeneralFormatTests.xlsx A13, A17, A18, A26-A28, A61; probes/number-text-exponent-digits
Calc!D8 (excel-graph); probes/text-general-exponent-boundary A9, A10 (excel-graph, 2026-10-07).
Status: VERIFIED (multi-digit ties and wide fixed values: INFERRED)
Implemented in: `format_general` (formatter/format.rs).
Tasks: T-0938, T-1225

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
