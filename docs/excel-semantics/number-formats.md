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
  - 5 per placeholder run, plus 2 per `.` in it (`"0000000000"` costs 5; `0.0` costs 7);
  - 3 per unquoted or escaped literal character; `$` costs 1; a run of colons costs 3 once (see "Colon runs");
  - 2 per quoted character;
  - 2 per other token (`d`, `mm`, `%`, `@`, `[h]`, ...); `AM/PM` costs at least 3 (`"AM/PM"` + 124 quoted fails; the engine uses 3, and the same for `A/P`, unprobed);
  - 2 per color (`"[Red]0"` + 120 quoted works, 122 fails; `"[Red]"` + 124 quoted works, 125 fails);
  - 6 to 8 per condition (`"[>5]0"` plus 79 spaces works, 80 fails; the engine uses 7), 3 more in a section that also holds quoted text (`"[>5]0"` + 116 quoted works and 118 fails on 7 and on 1; `"[>5]"` + 120 quoted works, 122 fails);
  - 0 per `;`.

  Examples: `"d"&REPT(" ",82)` (248) works and `"d"&REPT(" ",83)` (251) fails; `REPT("-",83)` (249) works and `REPT("-",84)` fails; `"d"&REPT("$",160)` works and `REPT("$",250)` fails; `"0"&REPT(" ",40)&";0"&REPT(" ",40)` (250) works and 41/40 fails; `"0;0;0;"&REPT(" ",78)` works.
- **Quoted text depends on its neighbours** (T-1434, T-1449, T-1455; probes/text-format-length-limit-f to -k):
  - **Quoted text followed by more** pays 4 extra when a token or placeholder follows it directly, or when the literal run that closes the section goes on with unquoted text after it: 122 quoted + `d` works and 123 fails; `"d"` + 121 quoted + `d` works and 122 fails; 124 quoted + `@` (on text) fails; `"d"` + 120 quoted + 2 spaces works and 3 fails, + 122 quoted + 1 space fails, 110 + 8 spaces works and 9 fails, 100 + 15 works and 16 fails, 50 + 48 works and 50 fails; 120 quoted + 2 dashes works and 3 fails. Unquoted text that is followed by more quoted text or by a token does not pay it: `"d"` + 61 quoted + a space + 61 quoted works (247), `"d"` + 120 quoted + `" d"` works (247); nor does unquoted text before quoted text (50 spaces + 50 quoted works, 52 fails).
  - **Date, time and text tokens are free** in a section whose closing literal run holds quoted text: `"d"`, `"dd"`, `"m"`, `"h"`, `"yyyy"` or `"@"` + 125 quoted works like 125 quoted alone, and + 126 fails; `"d "` or `"dd "` + 123 quoted works and 124 fails; `"h:mm"` + 123 quoted works (both `h` and `mm` free); `"0;d"` + 122 quoted works and 123 fails. They keep their 2 before unquoted text (`"d"&REPT(" ",83)` fails) and before quoted text that a token follows (above). Elapsed tokens, `AM/PM`, `%` and placeholders are not free: `"[h]"` + 124 quoted works and 125 fails, `"%"` + 124 works and 125 fails, after `0`, `#`, `?` or `00` quoted text stops at 122, after `0.0` or `0 ` at 121. Seconds, `ddd`, `dddd`, `mmm`, `mmmm` are free too (`"ss"`, `"ddd"`, `"mmm"`, `"dddd"`, `"mmmm"` + 125 quoted works, 126 fails; probes/text-format-length-limit-l A27:A38); `A/P` is not (+ 125 quoted fails, A33; the engine already treats it like `AM/PM`). **`@` is refunded 1 more than the others** (T-1481): `TEXT("x","@ "&<124 quoted>)` works and 125 fails (`"@-"` and `"@\-"` the same), `"@  "` (two spaces) + 122 quoted works and 123 fails, `"@ - "` + 121 works and 122 fails, `" @ "` + 122 works and 123 fails (probes/text-format-length-limit-l A1:A3, A7:A9, A16:A18, -m), while `"d "` + 124 fails; `@` + unquoted text alone (no quoted) is unchanged (`"@"` + 82 spaces works, 83 fails). On a number the section prints General and the same limit applies (`TEXT(5,"@ "&<124 quoted>)` is "5", 125 is #VALUE!).
- **Two conditions in one section** are `#VALUE!` whatever the value and length: `TEXT(7,"[>5][<9]0")`, `TEXT(1,..)`, `TEXT(10,..)`, `"[>5]0[<9]"` (probes/text-format-length-limit-j A31:A34; also g A37:A41; only the first section is probed, the engine applies the rule to every section, inferred). A color with a condition works in either order (`"[Red][>5]0"` and `"[>5][Red]0"` on 7 give "7"). Next to spaces a condition costs 6 to 8 whatever its threshold (`"[>55555]0"` plus 79 spaces works and 80 fails, like `"[>5]0"`).
- **Text section with a number:** a single section holding `@` does not apply to a number, which prints in General: `TEXT(1,"@"&""""&REPT("a",118)&"""")` is "1" (probes/text-format-length-limit-i A19:A26).
- **Fourth section:** a fourth section holding a number or date token is `#VALUE!` whatever the value: `TEXT(1,"0;0;0;0")`, `"0;0;0;d"`, `"d;0;0;0"`, `"0;d;0;0"`, `"0;0;d;0"`, `TEXT(-1,"d;d;d;d")`, `TEXT(0,..)`, `TEXT("x",..)`. `"d;d"` and `"d;d;d"` give "1". This explains `"d;d;d;d"`. Only `d` and `0` are probed there; the engine extends the rule to every digit, date, time, elapsed, AM/PM and `E+` token (inferred).
- **Error precedence:** an error value wins over a too-complex code: `TEXT(NA(),REPT("dm",17))` is `#N/A`, `TEXT(1/0,"0;;;;")` `#DIV/0!`. Over an array, each element is `#VALUE!` except an error element, which keeps its error (`TEXT({1,#N/A},REPT("dm",17))` is `#VALUE!`, `#N/A`). An error inside an array of formats is that element's result (`TEXT(1,{"0",#N/A})` is "1", `#N/A`).
- **Colon runs:** consecutive colons print as one, whatever the run's length: `TEXT(1,"d"&REPT(":",n))` is "1:" for n from 2 to 85, and `"0"&REPT(":",n)` is "1:" for n from 2 to 84. A colon run broken by anything else prints in full: `"0:a:"` gives "1:a:", `"0::a"` "1:a", `"0: :"` "1: :", `d:"":` "1::". This is a formatter rule, not a cost rule. Only unquoted colons after `0` and `d` are probed; the engine also collapses escaped `\:` runs and `h::mm` (unprobed, T-1450).

Evidence: probes/text-date-255-char-limit, text-format-length-limit, text-format-length-limit-b (T-1388), text-format-length-limit-c, -d, -e (T-1393), -f, -g, -h, -i (T-1434), -j (T-1449), -k (T-1455), -l, -m (T-1481), all excel-graph.
Not explained or not implemented (known failures):
- Probe round l/m cells outside the `@` fit (T-1486): `@_` + 123 to 125 quoted characters (`_"` takes the opening quote as its pad character) is `#VALUE!` in Excel and we differ (l A13:A15); `"0"` + 118 quoted + `[Red]0`, a color after quoted text, works in Excel and we reject it (l A40); a quoted `a` then ` @ ` + 121 quoted fails in Excel and we accept it (m A22). Unfitted.
- `TEXT(1,"[>5]0"&REPT(" ",79))` is "1" (also with 40 spaces): an unmatched single condition prints General, dropping the padding. So is `"[>55555]0"` plus 79 spaces (-j A43). We do not apply conditions yet (see "Conditional sections", T-0936).
- `TEXT(1,"0:a:")` is "1:a:" and an escaped backslash followed by an unquoted `x` prints "\x": Excel accepts these letters unquoted, our lexer rejects them (-g A42, A43; -i A100:A104). T-1450.

Cell display with a long format could not be probed: excel-graph rejects such workbooks (createSession 501 unsupportedWorkbook). The colon-run and text-section rules live in the shared formatter, so they apply to cell display too; they are probed through TEXT only.
Ours: `formatter/complexity.rs` (`exceeds_complexity_limit`), applied by TEXT only; colon runs in `formatter/parser.rs`; the text section with a number in `format_number` (formatter/format.rs).
Status: VERIFIED (excel-graph, fitted model)
Tasks: T-1388, T-1393, T-1434, T-1449, T-1450, T-1451, T-1455

TEXT over an array of format codes (T-1451): one result per format, value and formats broadcast together (see text.md). `TEXT(1,{"0",#N/A})` is "1", `#N/A` (-f A54:B54). The error-precedence and lifting rules are verified by probes/text-format-array (T-1458) for error and too-complex format elements; a syntactically invalid format element next to an error value is not probed.
