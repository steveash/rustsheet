# Number formats (`TEXT` and cell formats)

All evidence below is from Apache POI's format test workbooks, scored by the excel-graph oracle
(sidecars in `eval/suites/poi/oracle/`, 2026-10-04). In each workbook `Tests!An` is
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
- An elapsed token after a plain `s` gives values that look wrong but are Excel's:
  `s:m" @ hour "[hh]` → "53:23 @ hour 1293" (A8), "4:3 @ hour 108" (A18), "0:0 @ hour 12" (A28);
  `s" secs and "[m]" mins"` → "53 secs and 77581 mins" (A9), A19 and A29 likewise. A hypothesis
  that fits all 10 cells: the elapsed token then measures the seconds component as days plus
  half a day (hours = (sec + 0.5)·24; 53.376 → 1293.02 h = 77581 min). The mechanism is
  UNVERIFIED; the cell values are verified.
Evidence: poi/ElapsedFormatTests.xlsx A2:A51.
Status: VERIFIED (values); UNVERIFIED (the `s`-then-elapsed mechanism)
Tasks: T-0934

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
