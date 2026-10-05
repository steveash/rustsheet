# Number formats (`TEXT` and cell formats)

All evidence below is from Apache POI's format test workbooks, scored by the excel-graph oracle
(sidecars in `eval/suites/poi/oracle/`, 2026-10-04) and checked against the values desktop Excel
cached in the same workbooks where it saved them. In each workbook `Tests!An` is
`=TEXT(Cn, Bn)` (TextFormatTests: `=TEXT(Cn, ";;;"&Bn)`), so `Bn` is the format and `Cn` the value.

## An empty format string returns empty text
Rule: `TEXT(<blank>, <blank format>)` returns `""`, not `#VALUE!`. Only a blank value has been
probed.
Evidence: poi/ElapsedFormatTests.xlsx A52:A215; poi/FormatChoiceTests.xlsx A27:A53.
Status: VERIFIED
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
