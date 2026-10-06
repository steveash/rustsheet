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

## Text and booleans use the format's text section
Rule: for a text or boolean value, `TEXT` applies the 4th (text) section; `@` stands for the
value's text and booleans become "TRUE"/"FALSE". `;;;"hi"` → "hi" (A3), `;;;\@@\@` → "@jello@"
(A6), `;;;-@-@-` → "-jello-jello-" (A10), `TRUE` with `;;;-@-` → "-TRUE-" (A17), and
`TEXT(TRUE,"General")` = "TRUE".
Evidence: poi/TextFormatTests.xlsx A3:A10, A12:A29; poi/GeneralFormatTests.xlsx A72, A73.
Status: VERIFIED
Tasks: T-0935

## Elapsed-time tokens
Rule (observed, ElapsedFormatTests, a 1904-date-system workbook):
- Fractional elapsed seconds work: `[h]:m:s.000` on 3.14159 → "75:23:53.376" (A2).
- A plain `h`/`ss` after `[h]`/`[s]` of the same unit shows the elapsed total too:
  "It was 75 [yes, 75] hours and 23:53" (A10), "271433 [yes, 271433] seconds" (A11).
- A negative value prints one leading "-" before any literal text:
  "-It was 2 [yes, 2] hours and 03:04" (A40).
Evidence: poi/ElapsedFormatTests.xlsx A2, A10, A11, A40 (web and the desktop Excel that saved the
workbook agree).
Status: VERIFIED
Tasks: T-0934

## CONFLICTING: elapsed token after a plain `s`
Excel for the web and desktop Excel disagree; neither is golden until an excel-desktop probe on
current Microsoft 365 decides (decision 0019). `s:m" @ hour "[hh]` and `s" secs and "[m]" mins"`:

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
Tasks: T-0934

## CONFLICTING: lowercase `e` exponent in a format
Excel for the web returns `#VALUE!` for `TEXT` with a lowercase-`e` scientific format; the Mac Excel
2008 (12.0) that saved the workbook cached real strings: `|#,e-#|` on 123456.789 → "|1e5|" (A2),
`|#,######.####|e-|#|` → "|123,456.789|e|0|" (A10), `|#,e-#|` on 1.23456789E-05 → "|1e-5|" (A42).
238 of the workbook's 249 cells are of this kind and stay pending. Uppercase `E` formats agree
(see below).
Evidence: poi/NumberFormatApproxTests.xlsx Tests!A2:A39, A42 onward (oracle: excel-graph
2026-10-04; workbook cached values).
Status: CONFLICTING (web vs desktop; excel-desktop probe pending)

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
Rule: General keeps fixed notation while the digits fit in 11 characters, sign not counted:
1E-09 → "0.000000001" (A27), -1E-09 → "-0.000000001" (A61), but 1E-10 → "1E-10" (A28) and
1E11 → "1E+11" (A18). This differs from number→text conversion (`numbers.md`, 15 digits).
Evidence: poi/GeneralFormatTests.xlsx A18, A27, A28, A61.
Status: VERIFIED
Tasks: T-0938

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
