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
zero under a literal-only format or `.` (`TEXT(-0.4,"x")` is "x" and `TEXT(-0.5,".")` is "-1." here, unprobed; T-1129), and a lowercase
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
date tokens is "The `B2` (Hijri) calendar" below (T-1613). Whether
the lone-`B` error is about the end of the code or of the section is inferred from `"B"` and `"dB"` only.
Tasks: T-1262

## The `B2` (Hijri) calendar
Rule: a `B2` (or `b2`) prefix with no locale tag puts the section's date tokens in the tabular Islamic calendar that Excel
uses (a 30-year cycle with leap years 2, 5, 7, 10, 13, 15, 18, 21, 24, 26 and 29, months of 30 and 29 days alternating,
the twelfth 30 in a leap year, and 1 Muharram 1 AH on the proleptic Gregorian 622-07-18). `TEXT(45000,"B2yyyy-mm-dd")`
is "1444-08-23", 45125 is "1445-01-01", 2958465 is "9666-04-03". This one rule replays all 103 probed dates (serials
1 to 2958465); the usual Kuwaiti variant with leap year 16 instead of 15 misses two (952726, 1303730).
Rule: serial 0 shows serial 1's date ("1317-08-29") and serial 60 (the fictitious 1900-02-29) is `#VALUE!`; 59 and 61
are consecutive Hijri days ("1317-10-28", "1317-10-29"). A negative serial is `#VALUE!`, as in Gregorian.
Rule: the year tokens read the Hijri year: `yyyy`, `yyy`, `e` and `ee` are "1444"; `y` and `yy` "44"; `b`/`bbbb` add
543 to it ("1987", `bb` "87"); `g` is "". Time tokens are unchanged (`B2h:mm AM/PM` of 45000.75 is "6:00 PM",
`B2[h]:mm` "1080018:00"). `dB2` is `#VALUE!` (the prefix must come before any printing token).
Rule: names are Arabic, and `mmm` = `mmmm`, `ddd` = `dddd` (full names; `mmmmm` is the first letter, "ش"). Two-word month
names are joined by a no-break space ("ذو\u00a0الحجة"). Months from Muharram: محرم, صفر, ربيع الأول, ربيع الثاني,
جمادى الأولى, جمادى الثانية, رجب, شعبان, رمضان, شوال, ذو القعدة, ذو الحجة. Weekdays from Sunday: الأحد, الإثنين,
الثلاثاء, الأربعاء, الخميس, الجمعة, السبت.
Rule (quirk): every name token after the first in a `B2` section reads the first name token's value. A weekday after a
month name is weekday number min(month, 7), Sunday being 1: `B2mmmm|dddd` of 45000 (Wednesday, 1444-08-23) is "شعبان|السبت"
(Saturday, as for every month from 7 on), of 45148 (Muharram) "محرم|الأحد" (Sunday), of 45177 (Safar) Monday. A month
name after a weekday prints the weekday again, `mmmmm` the whole of it: `B2dddd|mmmm` and `B2dddd|mmmmm` are
"الأربعاء|الأربعاء", `B2dddd|mmmm|dddd` the weekday three times. `mmmmm` first counts as the month (`B2mmmmm|dddd` is
"ش|السبت", `B2mmmmm|mmmm` "ش|شعبان"), and `mmmm|dddd|mmmm` is "شعبان|السبت|شعبان", `mmmm|dddd|dddd` "شعبان|السبت|السبت".
Rule (quirk): `mmm` always prints the month and becomes the value the names after it read: `B2dddd|mmm|dddd` is
"الأربعاء|شعبان|السبت" and `B2ddd|mmm|mmmm` "الأربعاء|شعبان|شعبان" (probes/t2729-hijri-locale-tag-names A239-A277,
under the unresolved `[$-zzz]`, which leaves a bare `B2`).
Rule (quirk): a day, month or year number before the first name token turns the quirk off for the section:
`B2yyyy-mm-dd|mmmm|dddd` is "1444-08-23|شعبان|الأربعاء" (probes/text-locale-calendars A124). Numbers after it change
nothing (`B2mmmm|dddd|d` is "شعبان|السبت|23"), and a lone name after a number is the real one (`B2m|dddd`, `B2d|dddd`,
`B2yyyy|dddd`). Whether a number between two names (`B2mmmm|d|dddd`) turns it off is unprobed; the engine keeps it on.
Gregorian sections do not do this (`mmmm|dddd` of 45148 is "August|Thursday").
Rule: as a cell number format `B2` shows the same text as TEXT: `B2yyyy-mm-dd` is "1444-08-23" and `B2mmmm|dddd` "شعبان|السبت" (Range.text, probes/t2729-hijri-locale-tag B1, B2).
Evidence: probes/t1613-hijri-b2 (144 cells), probes/t1613-hijri-b2-weekday (78 cells) (excel-graph, 2026-10-25, T-1613);
probes/t2729-hijri-locale-tag A151-A168 (excel-graph, 2026-10-27, T-2729).
Unprobed: the 1904 date system (the engine converts through the 1900 serial); a serial-0 weekday.
Implemented in: formatter/date_names.rs (`hijri_ymd`, `NameTokens`), the `hijri` flag of `DatePart` (formatter/parser.rs).
Under a locale tag: "`B2` before a locale tag and calendar digit 6" below.
Status: VERIFIED (excel-graph)
Tasks: T-1613, T-2729

## `B2` before a locale tag and calendar digit 6 (T-2729)
Rule: calendar digit 6 (`[$-6LLLL]`) is the Hijri calendar of "The `B2` (Hijri) calendar" above, and so is a `B2` written
anywhere before a tag whose calendar digit is 0 (`B2[$-40C]`, `B2"x"[$-40C]`, `B2mmmm[$-40C]`, `b2[$-fr-FR]`). Every date
token prints the Hijri date: `yy` "44", `yyyy`, `e` and `ee` "1444", `b` and `bb` "44", `bbbb` "1444" (no Buddhist 543
as under a bare `B2`), `g` nothing. Serial 0 is serial 1's date, 60 and negatives are `#VALUE!`, 2958465 is
"9666-04-03". Day halves follow the LCID (`[$-60401]h AM/PM` of 45000 is "12 ص").
Rule: the names are the LCID's Hijri names, `mmm` = `mmmm` and `mmmmm` the first letter, each token reading its own value
(no first-name quirk: `[$-60409]mmmm|dddd` of 45000 is "Sha'ban|AlArbia'a"):
- An Arabic LCID (401, 801, C01, 2801), LCID 0 and any LCID with no locale (`[$-60000]`, `[$-608DE]`, `B2[$-0]`,
  `B2[$-FFFF]`): the Arabic names of the bare `B2`, ar-SA weekdays.
- French (40C, C0C, `[$-6000C]`): Mouharam, Safar, Rabi' Aoual, Rabi' Thani, Jamada El Oula, Jamada El Thaniah, Rajab,
  Chaaban, Ramadan, Chaoual, Zoul Ki'dah, Zoul Hijjah, with the ordinary French weekdays ("mer.", "mercredi").
- Turkish (41F): Muharrem, Safer, Rebiyülevvel, Rebiyülahir, Cemaziyelevvel, Cemaziyelahir, Recep, Şaban, Ramazan,
  Şevval, Zilkade, Zilhicce, with the ordinary Turkish weekdays ("Çar", "Çarşamba").
- Every other probed LCID (409, 809, 400, 404, 407, 807, C07, 40A, C0A, 80A, 2C0A, 410, 411, 412, 413, 414, 415, 416,
  816, 419, 41B, 41D, 41E, 422, 405, 406, 408, 40B, 40D, 40E, 439, 804, 1004, C04): English transliterations, Muharram,
  Safar, Rabi' Awwal, Rabi' Thani, Jamada El Oula, Jamada El Thaniah, Rajab, Sha'ban, Ramadan, Shawwal, Thoul Ki'dah,
  Thoul Hijjah, and weekdays (`ddd` = `dddd`) AlAhad, AlEthnien, AthThulatha, AlArbia'a, AlKhamis, AlJumaa, AsSabt.
  Plain spaces; the Arabic names keep their no-break spaces.
Rule (quirk): after a `B2` before the tag, `mmm` prints the Arabic month instead, and from it on every name token takes
the bare `B2` path with its first-name quirk, `mmm` being the first name: `B2[$-409]mmmm|mmm|mmmm` is
"Sha'ban|شعبان|شعبان", `B2[$-409]dddd|mmm|dddd` "AlArbia'a|شعبان|السبت", `B2[$-40C]mmm|dddd` "شعبان|السبت", while
`B2[$-409]mmmmm|mmmm|dddd` is "S|Sha'ban|AlArbia'a". `B2[$-60409]mmm` is Arabic too; `[$-60409]mmm` and
`[$-60409]B2mmm` are "Sha'ban". The same as a cell number format (B1-B3 of t2729-hijri-locale-tag-names).
Rule: any other calendar digit wins over a `B2` before the tag: `B2[$-10409]`, `B2[$-20409]`, `B2[$-90409]` and
`B2[$-A0409]` print as without the `B2` ("2023-03-15|March"), `B2[$-110409]` the lunar date, `B2[$-80409]` the Hebrew
date. An unresolved tag leaves a bare `B2` (`B2[$-zzz]`, `B2[$-]`).
Rule (quirk, T-2932): before a calendar-3, -5 or -7 tag `B2` converts every year token once more than it would without
the `B2`, wherever in the section before the tag it stands (`b2`, `B2B2`, `B2"x"` and `B2mmmm` before the tag too; `B1`
before the tag and `B2` after it change nothing):
- Thai (7): `yy`, `yyyy`, `b`, `bbbb` add 543 twice, `e` and `ee` once: `B2[$-70409]yyyy|yy|e|ee|b|bb|bbbb|g` of 45000 is
  "3109|09|2566|2566|09|09|3109|", of 9999-12-31 "11085|85|10542|10542|08|08|11085|". Under a Japanese LCID the whole
  year after a `g` is converted once (`B2[$-70411]g|yyyy` is "|2566").
- Korean (5): `yy` and `yyyy` add 2333 twice, `e`, `b` and `bbbb` once: `B2[$-50409]yyyy|yy|e|b|bbbb` is
  "6689|89|4356|56|4356", of 9999-12-31 "14665|65|12332|33|12332".
- Japanese (3, any LCID): the era year is read as the Gregorian year of a date with the same month and day and converted
  again. Every year below 1868 is Meiji, so the era year is year - 1867, negative, printed as an unsigned 32-bit number
  by `yy`, `yyyy`, `e` and `ee` alike, and `g` is Meiji: 2023-03-15 (Reiwa 5) is "4294965434|M" (5 - 1867 = -1862),
  2019-05-01 (Reiwa 1) 4294965430, 2019-04-30 (Heisei 31) 4294965460, 1989-01-07 (Showa 64) 4294965493, 1920-01-01
  (Taisho 9) 4294965438, 1900-01-01 (Meiji 33) 4294965462. 9999-12-31 (Reiwa 7981) is "5963|R": 7981 is in Reiwa again.
  Which era a twice-converted year from 1868 to 2019 takes is unreachable (no serial gives one).
- Month and day tokens and names print as without the `B2` (`B2[$-30411]yyyy-mm-dd|mmmm|dddd` is
  "4294965434-03-15|3月|水曜日"), except a weekday after `mmm` under Thai (T-3029).
- Before calendar 4 (Taiwan) the years are neither: `B2[$-40409]yyyy|yy|e|b|bbbb` is "01|01|01|2\u{1c}|01" (not modelled,
  T-3028).
- The same as a cell number format (B1-B5).
Rule: `b` and `bb` print the year's third and fourth digits, not its last two, which differ past year 9999: with no tag
`b|bbbb|yy` of 9999-12-31 is "54|10542|99", `[$-70409]b|yy` "54|42", and the `B2` cases above "08" and "33"
(probes/t2932-b2-calendar-twice A24, A64-A68). `yy` stays the last two digits.
Rule (quirk): under a tag, a number token before the `mmm` switch does not turn the first-name quirk off, unlike in a bare
`B2` section: `B2[$-409]yyyy|mmm|dddd` is "1444|شعبان|السبت" (Saturday, from the month) and so are `d|mmm|dddd` and
`e|mmm|dddd`.
`TEXT(45000,"B1@")` and `"B2@"` are "45000".
Evidence: probes/t2932-b2-calendar-twice (73 cells), excel-graph 2026-10-28.
Rule: a second calendar prefix after a tag is `#VALUE!` (`B2[$-409]B2mmmm`, `B2[$-409]B1mmmm`, `B1[$-409]B2mmmm`,
`B2[$-60409]B1mmmm`, `[$-60409]B2B1mmmm`), as is any number token in a `B2` section (`B2[$-409]0`), but two before
the tag are fine (`B2B2[$-409]mmmm` is "Sha'ban") and so is `B1` before a tag (`B1[$-40C]mmmm` is "mars").
Evidence: probes/t2729-hijri-locale-tag (278 cells), probes/t2729-hijri-locale-tag-names (320 cells), excel-graph
2026-10-27; text-locale-calendars A47-A49, A87, A88, A124; text-locale-calendars-detail A7, A8; text-locale-era-gregorian A21.
Unprobed: calendar digits 15 and above after `B2` (the engine lets them win like the others); the extended-hex tags that
resolve to a locale (`B2[$-g]`; the engine leaves them Gregorian); Hijri names of LCIDs with no table here (TODO(T-1875); the engine gives them the Arabic names
of an LCID with no locale, so Persian `[$-60429]` prints "شعبان").
Implemented in: `resolve_after_hijri_prefix`, `hijri_names` and `Calendar::Hijri` (formatter/locale_tag.rs), the
lexer's `hijri_prefix`, `NameTokens` (formatter/date_names.rs), the Hijri tables in locale/excel_date_names.rs;
the twice-converted years in `Localized::year_twice`, `japanese_era` (locale_tag.rs) and `year_token_text`
(formatter/calendar_dates.rs).
Status: VERIFIED (excel-graph)
Tasks: T-2729, T-2932

## Bare digits beside a date token
Rule: a bare (unquoted, unescaped) digit 1-9 is a literal in a number section (`TEXT(5,"2")` = "2", `"4a"` = "4a",
`"4 0"` = "4 3"). But a section holding one beside any date or time token, before or after it, makes the whole code
`#VALUE!`. That holds for 3.14159, 100 and -3 alike, with `"42abc"`, `"4b"`, `"b4"`, `"4 b"`, `"1b"`, `"9b"`, `"4d"`, `"d4"`,
`"4 d"`, `"d 4"`, `"dd 4"`, `"4yy"`, `"4mmm"`, `"4h"`, `"4s"`, `"4:mm"`, `"4e"`, `"4g"`, `"[h]4"`, `"4[h]"`, `"4/d"`, `"b.4"`,
`"4%b"` and `"x4b"` (rows 1-6, 9-15, 22-24, 27-28, 30-34, 37-39; `"0b"` and `"#b"`, rows 7-8, are the digit-placeholder rule of T-2202). AM/PM alone does not count (`"4am/pm"` and `"4AM/PM"` are "4AM"), and
a quoted or escaped digit prints (`"\4b"` and `"""4""b"` are "443"). The check covers every number section, also one the
value does not use: `TEXT(3.14159,"0;4b")` is `#VALUE!`, while `"4;b"` gives "4" and `"b;4"` "43". A text or boolean value
is rejected too (`TEXT("x","42abc")`, `TEXT(TRUE,"42abc")` and `TEXT("5","42abc")` are `#VALUE!`; `TEXT("x","4;b")` is "x";
column D), as "TEXT checks the code before it looks at a text or boolean value" describes. `TEXT(-3,"b1")` is "-": a
calendar prefix alone is no date token for a negative value. Ours is `#VALUE!` there (T-2388).
Evidence: probes/text-format-digits-date-token (excel-graph, 2026-10-22, T-2377, 132 cells); gen-m5/2b01cadfd03df4b5 Calc!A211.
Status: VERIFIED
Implemented in: `Parser::parse_part` (`bare_digit`, `is_calendar_or_clock`, formatter/parser.rs); `format_number` rejects a
code with any bad number section (formatter/format.rs).
Tasks: T-2377, T-2388

## Text and booleans use the format's text section
Rule: for a text or boolean value, `TEXT` applies the 4th (text) section; `@` stands for the
value's text and booleans become "TRUE"/"FALSE". `;;;"hi"` → "hi" (A3), `;;;\@@\@` → "@jello@"
(A6), `;;;-@-@-` → "-jello-jello-" (A10), `TRUE` with `;;;-@-` → "-TRUE-" (A17), and
`TEXT(TRUE,"General")` = "TRUE". An empty 4th section prints nothing: `TEXT("text",";;;")` is ""
(gen-m10/0e3d8eba9432a18d T!A24; ours "text", T-2945), as `;;;` hides a number (gen-m10/810fee377a42575b A19).
Empty and missing text sections (probes/text-format-text-sections B2:J15, excel-graph 2026-10-28, T-2975): an empty
4th section gives "" for text, TRUE, FALSE and "" alike (`;;;`, `0;0;0;`, `0;0;General;`; LEN 0), while numbers keep their own sections there (5 and -5 give "5", 0 "0"); `;;;` hides numbers
too. A format with no 4th section (`0;0;0`) and `0;0;0;@` give the text, "TRUE" or "FALSE" unchanged. Any format with
five or more sections (`0;;;;`, `0;;;;@`, `0;0;0;;`, `;;;@;`) is `#VALUE!` for every value, text and booleans included.
Evidence: poi/TextFormatTests.xlsx A3:A10, A12:A29; poi/GeneralFormatTests.xlsx A72, A73; gen-m10/0e3d8eba9432a18d T!A24; probes/text-format-text-sections.
A doubled quote inside a quoted literal: `;;;"h""I"` gives "hI" in Excel (A4, A14, A23), not `h"I`, so
it is not an escape there: see "A doubled quote in a format literal is not an escape" below.
Status: VERIFIED
Implemented in: `format_text_value` (formatter/format.rs) used by `fn_text` (functions/text/common.rs), scalars and array elements; a text value with a format of three sections returns the text unchanged (probed, T-2975; one or two sections unprobed). An empty 4th section (`;;;`; the parser drops the empty section after a trailing `;`, so `Parser::has_empty_text_section` detects it) gives empty text (gen-m10/0e3d8eba9432a18d T!A24). A five-section format, `TEXT(TRUE,";;;")`, `"0;0;0;"` and `"0;0;General;"` on a text value match Excel (probes/text-format-text-sections, T-2975).
Tasks: T-0935, T-2945, T-2975

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
Unprobed (T-0935 follow-up): a text value with a 1-2 section format and no `@` section (the
engine returns the text unchanged; a 3-section `0;0;0` returns it unchanged in Excel too, T-2975), and a General 4th
section over a non-empty value. `0;@`-style formats, `_x`/`*x` and
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
- Fractional elapsed seconds work: `[h]:m:s.000` on 3.14159 → "75:23:53.376" (A2), `[m]:s.000` →
  "4523:53.376" (A3), `[s].000` → "271433.376" (A4), `[ss].000` on 0 → "00.000" (A27). The value is
  rounded to the shown digit first and every field is cut from that: `[hh]:mm:ss.0` → "75:23:53.4"
  (A5; .376 rounds up), and an elapsed count before the fraction is the whole part of that rounded
  total (`[s].000` is 271433 then ".376", A4).
- A plain `h`/`s`/`ss` after `[h]`/`[hh]`/`[s]`/`[ss]` shows the elapsed total too:
  "It was 75 [yes, 75] hours and 23:53" (A10), "271433 [yes, 271433] seconds" (A11), "7384 [yes,
  7384] seconds" (A21); padding stays (`ss` on 0 is "00", A31). `m` after `[m]` is unprobed; the
  engine keeps it a clock minute (T-2135).
- Past 9999-12-31 a fraction of a second is #VALUE! with or without an elapsed token:
  `TEXT(1234567890,"[mm]:ss.0")` and `TEXT(1234567890,"h:mm:ss.00")` (gen-m10 810fee377a42575b
  T!A16, b4f66526f822ff2c T!A20; fbdab487cb11409e T!A25 on 123456789012345). Four or more zeros
  (`s.0000`) are unprobed; the engine keeps them #VALUE!. Also unprobed: where that cutoff sits
  (the engine uses serial 2958466, also in a 1904 workbook) and whether a date token rolls over
  with a fraction that rounds past midnight (the engine's does not); T-2135 probes them.
- A negative value prints one leading "-" before any literal text:
  "-It was 2 [yes, 2] hours and 03:04" (A40).
In the 1900 date system TEXT of a negative under a format that mixes an elapsed token with a date/time token (`[h]:mm`, `[s] yy`, `[s].00`) is #VALUE! instead; a format whose only date/time token is one elapsed token still prints a minus, with any literals (below; dates.md "Negative serials under an explicit negative section", probes/text-negative-serial-elapsed-mixed, probes/negative-date-sections-1900, T-1079/T-1105); the minus above is the 1904-workbook behavior. Two elapsed tokens in one section (`[h]:[mm]`) are #VALUE! in both systems, for positive values too (same evidence).
An elapsed token works on the magnitude of the value and keeps a minus for a negative one, in both date systems (VERIFIED, probes/elapsed-tiny-negatives-1900/-1904 and elapsed-fractions-1900/-1904, excel-graph, T-1338, T-1346): `[s]`/`[ss]` round it to nearest (1.7 s = "2", 1.2 s = "1", 1.5 s = "2", -0.9 s = "-1", -1.7 s = "-2", `TEXT(-1E-10,"[s]")` = "-0"); `[m]`/`[mm]`/`[h]`/`[hh]` truncate it (1.7 min = "1", 1.7 h = "1", -1.7 min = "-1", -0.3 h = "-0"). Padding follows the sign (`[ss]` of -1E-10 = "-00", of -1.7 s = "-02", of 1.7 s = "02"). Binary noise is absorbed: -1.5/86400 days is "-2", `TEXT(-(3-1E-7)/24,"[h]")` = "-3", `TEXT(-(2-1E-9)/1440,"[m]")` = "-2", yet `TEXT(-0.49999999999/86400,"[s]")` = "-0" and `TEXT(-(5-1E-6)/86400,"[s]")` = "-5". Up to 9999-12-31 the engine counts every elapsed token from the serial rounded to the whole second, then truncates `[m]`/`[h]` (T-2385, probes/elapsed-no-seconds-boundary); past it a lone token uses a 1e-12 snap of the seconds for `[s]` and a millisecond snap for `[h]`/`[m]`: fitted to these cells, not known (the true threshold is unprobed). The positive near-integers behave the same (VERIFIED, probes/elapsed-positive-large-1900/-1904, excel-graph, T-1362): `TEXT((3-1E-7)/24,"[h]")` = "3", `TEXT((5-1E-6)/86400,"[s]")` = "5", `TEXT(0.49999999999/86400,"[s]")` = "0". The count is plain arithmetic with no 32-bit limit in either date system: `TEXT(30000,"[s]")` = "2592000000", `TEXT(24856,"[s]")` = "2147558400", `TEXT(2958465,"[s]")` = "255611376000", `TEXT(1E9,"[h]")` = "24000000000". `TEXT(-1.5,"[s]")` = "-129600".

**Elapsed tokens beyond the i64 range** (VERIFIED, probes/elapsed-huge-a/b0/b1/b2/c/d/e/f/g/h/i-1900, excel-graph, T-1375): the count is printed in full, never in scientific notation, but only its first 15 significant digits are kept and the rest are zeros, from 1e15 up (`TEXT(1E15,"[s]")` = "86400000000000000000", `TEXT(1.234567890123456E16,"[s]")` = "1066666657066660000000", `TEXT(1E100,"[h]")` = 1 followed by 99 zeros times 24; the same at `[m]`/`[ss]`; negatives keep the minus). Below 1e15 it is the exact integer. The result is `#VALUE!` once it is over 255 characters (`TEXT(1E253,"[h]")` is 255 digits and works, `TEXT(1E254,"[h]")` is #VALUE!; `TEXT(1E250,"[s]")` works, `TEXT(-1E250,"[s]")` is #VALUE! because of the minus; `TEXT(1E300,"[s]")` is #VALUE!). When the count itself overflows a double the result is "0" ("-0" for a negative): `[s]` once days*86400 overflows (`TEXT(2.0807E303,"[s]")` = "0" while `TEXT(2.0806E303,"[s]")` is #VALUE!), `[m]`/`[h]` once days*1440 / days*24 does (`TEXT(1.5E307,"[h]")` = "0", `TEXT(1E305,"[m]")` is still #VALUE!). A literal 1.7E308 in a formula made excel-graph reject the whole workbook, so serials that high are unprobed. The 255-character `#VALUE!` is probed for elapsed tokens only; the engine applies the 255-character cap to every date/time section as an assumption. Corrected (T-1388): a far lower limit on the format code itself decides first (see "TEXT format code complexity limit" below), so a date/time TEXT cannot reach 255 characters through repeated tokens; whether any can exceed 255 is unprobed.
Evidence: poi/ElapsedFormatTests.xlsx A2-A7, A10-A17, A21-A27, A31-A37, A40-A47, A50, A51 (web and
the desktop Excel that saved the workbook agree); gen-m10 cells above.
Status: VERIFIED
Implemented in: `Parser::parse_part` (formatter/parser.rs: `TextToken::SecondFraction`, plain
`h`/`s` after an elapsed one), `FractionalSeconds` and `elapsed_text` (formatter/elapsed.rs).
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
(`[<10]#" Wow"` on 10 → "10", A3; `[>10]#" Big"` on 10 → "10", A5); `[<>10]` is a valid condition
(A17 "1 Wow", A18 "10"); spaces between the comparator and the number are allowed (`[< 10]` → "1
Wow", A23; `[< 10]`, A25). The same holds when the condition sits in a date section or after other
tokens (`TEXT(1,"d"&<119 quoted a>&"[>5]d")` is "1", probes/text-format-length-limit-q A16) and when
literals pad it (`TEXT(1,"[>5]0"&REPT(" ",79))` is "1", -d S!A31, -f A41:A42, -j A43). The evidence
does not decide whether the fallback is General or the section without its literals: every probed
value prints the same either way. The engine uses General, the simpler rule. The condition compares
the signed value, and a matched one-section format keeps the minus: `[=-10]#" Wow"` on -11 is "-11",
`[<>-10]#" Wow"` on -11 is "-11 Wow" and on -9 "-9 Wow" (poi/FormatConditionTests A32, A35, A37).
Two or three sections with a condition on the first: the first section takes a value its condition
holds for; otherwise a two-section format uses the second, and a three-section format tests the
second's condition and falls to the third (`[<100]0.0;[>=1000]#,##0;0` prints 999.9999 as "1000",
2958465.99999 as "2,958,466"; `[<=-1]0;[<1]0.0000;0.0` prints 1234567890 as "1234567890.0";
`[>=1]0.00;0.000` prints 0.5 as "0.500"; `[<0]"neg";[>0]"pos";"zero"` prints 1 as "pos").
The matched section keeps the minus (`[<100]0.0` on -0.5 is "-0.5", `[<1]0.0000` on -0.5 "-0.5000",
the `else` section of `[>=1]0.00;0.000` on -0.5 "-0.500", `[=0]"nil";General` on -123456.789
"-123456.789") except when its condition holds for negatives only: `[<=-1]0` on -1.5 is "2".
Unprobed (T-2768): that cut for `[<=0]`, `[=-1]`, `[<0]` with digits, negatives reaching a third
section, a three-section format whose second section has no condition (the sign rules apply), and four sections; the engine drops the sign only for `<` with a bound <= 0 and `<=` with a bound < 0.
Evidence: gen-m10 (excel-graph, 2026-10-16) 039accabcfe1e6c7 F!Q1:Q20, 03a284c57e24ff37 and siblings (T-1734).
Evidence: poi/FormatChoiceTests.xlsx A3:A26 (excel-graph); probes/text-format-length-limit-d
S!A31, -f Sheet1!A41:A42, -j Sheet1!A43, -q Sheet1!A16 (excel-graph); poi/FormatConditionTests.xlsx
A3:A50 (Excel-cached).
Status: VERIFIED
Tasks: T-0936 (implemented), T-1734 (implemented)

## Scientific formats with literals after `E`
Rule: literals may sit between `E+`/`E-` and the exponent digits; the exponent sign is printed
next to the digits, after the literal, and a zero mantissa prints "0.": `|#.#|E+|#|` → "|1.2|E|+5|"
(A41) and "|0.|E|+0|" for 0 (A137); `E-` omits a positive sign: "|1.2|E|5|" (A40), "|1.2|E|-5|"
(A96). The `E` stays where the format puts it; only the sign moves to the digits.
- The zero-mantissa digit also shows in a cell's displayed text: `#E+0` on 0 displays "0E+0", not "E+0"
  (gen-m10/5eb95c63c7d6180c F!G13, gen-m10/71ad5f93362f4ff4 F!V19). The minus of a negative number still leads
  everything ("-|1.2|E|5|", A176).
Evidence: poi/NumberFormatApproxTests.xlsx A40, A41, A96, A97, A136, A137, A176, A177, A216, A217
(excel-graph); gen-m10 cells above (excel-graph).
Ours: `formatter/parser.rs` pushes the `E` as a literal at the format's `E+`/`E-`; `formatter/format.rs`
prints the sign before the first exponent digit and a `0` for the last `#` integer placeholder of a zero
mantissa (T-0937).
Status: VERIFIED
Tasks: T-0937 (implemented)

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
and keeps the sign. A `,` right after a `%` does not scale: it prints as a literal (`0%,` on 0.5 is "50%,", on 1234567
"123456700%,"; probes/fraction-overflow-scaled A79-A80, excel-graph 2026-10-27, T-2522; see "Scaling commas in fraction
formats").
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
Implemented in: `Token::AP` / `Token::AMPM` (formatter/lexer.rs), `TextToken::AP` (formatter/format.rs); tests in formatter/test/test_time.rs.
Tasks: T-0954

## `m`/`mm` directly before `:ss` is minutes, with no hour
Rule: `m` or `mm` followed by `:` and a plain `s`/`ss` is a minute even when the code has no hour
token (otherwise it is a month): `mm:ss` shows 1000 as "00:00", 123456.789 as "56:10" and 1.5 as
"00:00". After an hour it was already minutes (`h:mm`).
Evidence: suite gen-m10 displayed-text cells (excel-graph, 2026-10-16, T-1611), 71 of 80 `mm:ss` cells
failed before the fix. Status: VERIFIED (excel-graph).
Implemented in: `minute_before_seconds` (formatter/parser.rs). Probed: `mm:ss`, `mm:ss.0`, `mm:ss.00`. Unprobed
(implemented by the same rule): single `m:ss`, `:s`, `[m]`/`[s]` forms, `mmss` with no colon, and any other
separator or literal between the `m` and the seconds (those still read as months).
Tasks: T-1733

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
Implemented in: `FractionalSeconds` (formatter/elapsed.rs), shared with the elapsed tokens (T-0934).
Tasks: T-0981, T-0934

## TEXT format code complexity limit
Rule: `TEXT` is `#VALUE!` when the format code is too complex, which happens long before 255 characters. The model below is fitted to the probed boundaries of probes c to k (524 cells; VERIFIED, excel-graph; excel-desktop unprobed); every probed cell fits. Its weights look like the size in bytes of a compiled code, a quoted character being one UTF-16 unit (2) and an unquoted one 3.
- **Sections:** more than four is `#VALUE!`, even when the fifth is empty (`"0;;;;"`, `"0;0;0;@;"`); `"0;;;"` works.
- **Elements:** at most 32 elements per section. An element is a run of digit placeholders together with their `,` and `.` (`REPT("0.",20)` and `REPT("#,",17)` work), a run of literals (unquoted, escaped and quoted text merge into one run, so `REPT("""a"" ",17)` works), or any other token (`d`, `mm`, `dddd`, `%`, `@`, `_x`, `*x`, `[h]`, `AM/PM`, ...). `REPT("dm",16)` works (32 elements), `REPT("dm",16)&"d"` fails (33). The count restarts at each `;` (two sections of 17 elements work).
- **Cost:** at most 250 over the whole code (T-1434; the earlier 166 model counted the same boundaries in units of 1.5), counted as follows:
  - 3 once per section holding a digit placeholder, plus 2 per placeholder run, plus 2 per `.` in a run (`"0000000000"` costs 5; `0.0` costs 7; `"0-0"` costs 10: `"0-0"` + 120 quoted works and 121 fails, `"0-0-0"` + 117 works and 118 fails, `"0-0;"` + 120 works and 121 fails; probes/text-format-length-limit-o A1:A4, -p A5:A7, -r, T-1486). A section after the first that holds both a placeholder and quoted text pays 1 more: `"0-0;0-0"` + 114 quoted works and 115 fails, `"0;0"` + 120 fails, `"0-0;0"` and `"0;0-0"` + 117 work and 118 fail (-q A14:A15, -r); A color between two placeholders does not split their run (`"0[Red]0"` + 121 quoted works, -o A8);
  - 3 per unquoted or escaped literal character; `$` costs 1 (plus a once-per-section 3, see "Unquoted `$`" below); a run of colons costs 3 once (see "Colon runs");
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
- **Currency tags** (`[$€]`, `[$€-40C]`, `[$USD]`) cost like quoted text of their symbol plus 13, and the locale part of a currency tag costs nothing (a bare `[$-409]` costs 14): `[$€]` and `[$€-40C]` + 117 quoted work and 118 fail (15 + 2 per character), `[$USD]` and `[$USD-40C]` + 115 work and 116 fail (19). Followed by unquoted text the tag pays 2 (not quoted text's 4) for the literal run going on unquoted: `[$€-40C]` and `[$€]` + 77 spaces work and 78 fail, `[$USD]` + 76 spaces works and 77 fails (probes/t2836-dollar-run-cost A63:A73; 4 would reject 76). A system-locale tag's locale costs nothing too: `[$€-F800]` + 117 quoted works and 118 fails (A74:A81). Evidence: probes/t1883-currency-locale-edges A9:A63, probes/t2812-currency-tag-cost (excel-graph, T-2812), probes/t2836-dollar-run-cost (T-2836).
- **Unquoted `$`** costs 1 per character, and a section holding one but no digit placeholder pays 3 once, like the 3 a placeholder section pays (2 when the section also holds a date token): `REPT("$",247)` (250) works and 248 fails; `"d"&REPT("$",246)` and `REPT("$",246)&"d"` (250) work and 247 fail; `"0"&REPT("$",245)` (250) works and 246 fails, so `$` adds nothing to a placeholder section's 3. Quoted text whose closing literal run goes on with only `$` pays 2 for it, not 4 (quoted `"a"` + 243 `$` works and 244 fails, `[$€]` + 230 `$` works and 231 fails, `[$USD]` + 226 works and 227 fails), while quoted text followed by spaces pays 4 (`"d"` + 122 quoted + 1 space fails). Unquoted text directly after a locale tag pays 1 (`[$-409]` + 232 `$` works and 233 fails; `[$-409]` + 78 spaces works and 79 fails). These are fitted per context: no single additive `$` cost fits every ladder. Probed in more contexts (probes/t2842-dollar-run-contexts, excel-graph, 2026-10-27, T-2842): `m`, `yyyy`, `h` and `ss` pay the section's 2 like `d` (+ 246 `$` works, 247 fails), `AM/PM` the full 3 (+ 236 works, 237 fails), and `@` on text the 2 of a date token (`"@"` + 246 `$` works, 247 fails). An escaped `\$` is an ordinary escaped literal: 3 each and no `$` section cost (`REPT("\$",83)` works, 84 fails), and quoted text followed by `\$` pays the full 4 (`"a"` + 81 works, 82 fails). Dashes after a locale tag pay its 1 like spaces (`[$-409]` + 78 `-` works, 79 fails). Evidence: probes/t2836-dollar-run-cost (excel-graph, 2026-10-26, T-2836), t2842-dollar-run-contexts.
- **A format code over 255 characters is `#VALUE!`** in TEXT, whatever its cost; only runs of free characters (colons after the first, commas in a placeholder run) reach it: `"0"` + 254 commas (255 characters) works and 255 fails, `"d"` + 254 colons works and 255 fails, `[$-409]` + 248 colons works and 249 fails, `[$-409]0` + 247 commas works and 248 fails (probes/t2842-dollar-run-contexts J, K; T-2842). Codes with characters outside the BMP, and the limit on a text value, are unprobed (the engine counts characters and applies it to every value; T-2964).
- **Two conditions in one section** are `#VALUE!` whatever the value and length: `TEXT(7,"[>5][<9]0")`, `TEXT(1,..)`, `TEXT(10,..)`, `"[>5]0[<9]"` (probes/text-format-length-limit-j A31:A34; also g A37:A41; only the first section is probed, the engine applies the rule to every section, inferred). A color with a condition works in either order (`"[Red][>5]0"` and `"[>5][Red]0"` on 7 give "7"). Next to spaces a condition costs 6 to 8 whatever its threshold (`"[>55555]0"` plus 79 spaces works and 80 fails, like `"[>5]0"`).
- **An unterminated code is `#VALUE!`**, whatever the value and whichever section it would use: `TEXT("x","0""")`, `TEXT("x","""")`, `TEXT("x","0;0;0;""")`, `TEXT("x","@;""")`, `TEXT(5,"0;""")`, `TEXT(-5,"0;""")`, `TEXT("x","0;[Red")` are `#VALUE!` (an unclosed quote or bracket; probes/text-format-length-limit-q A4:A13; T-1486). So are codes left with an unclosed quote after `_` or `*`, which take the next `"` as their character (`@_"a"`, `@_""`, `@*""`, `@"`, and `@_"` + 123 to 125 quoted characters; -n A3, A5, -o A36, A41, -l A13:A15). A valid code without a text section still leaves text unchanged: `TEXT("x","0:a:")`, `"0\x"` and `"0 a"` are "x" (-q A1:A3). A code ending in `\`, `_` or `*` is `#VALUE!` the same way: `TEXT("x","0\")`, `"0_"`, `"0*"`, `"@\"`, `TEXT(7,"0_")`, `TEXT(5,"0;0\")`, `TEXT(5,"0;0_")`, `TEXT(-5,"0;0\")` (-s). Only unterminated codes (a quote or bracket left open, a `\`, `_` or `*` at the end) are probed; the engine rejects exactly those (`is_unterminated` in formatter/complexity.rs), not every code our parser fails on. Other parse failures in a section the value does not use (e.g. `TEXT(5,"0;0 n")`: our lexer rejects `n`, as Excel does in a section it uses, probes/text-format-unquoted-letters A14, A40) are unprobed.
- **Text section with a number:** a single section holding `@` does not apply to a number, which prints in General: `TEXT(1,"@"&""""&REPT("a",118)&"""")` is "1" (probes/text-format-length-limit-i A19:A26).
- **Fourth section:** a fourth section holding a number or date token is `#VALUE!` whatever the value: `TEXT(1,"0;0;0;0")`, `"0;0;0;d"`, `"d;0;0;0"`, `"0;d;0;0"`, `"0;0;d;0"`, `TEXT(-1,"d;d;d;d")`, `TEXT(0,..)`, `TEXT("x",..)`. `"d;d"` and `"d;d;d"` give "1". This explains `"d;d;d;d"`. Only `d` and `0` are probed there; the engine extends the rule to every digit, date, time, elapsed, AM/PM and `E+` token (inferred).
- **Error precedence:** an error value wins over a too-complex code: `TEXT(NA(),REPT("dm",17))` is `#N/A`, `TEXT(1/0,"0;;;;")` `#DIV/0!`. Over an array, each element is `#VALUE!` except an error element, which keeps its error (`TEXT({1,#N/A},REPT("dm",17))` is `#VALUE!`, `#N/A`). An error inside an array of formats is that element's result (`TEXT(1,{"0",#N/A})` is "1", `#N/A`).
- **Colon runs:** consecutive colons print as one, whatever the run's length: `TEXT(1,"d"&REPT(":",n))` is "1:" for n from 2 to 85, and `"0"&REPT(":",n)` is "1:" for n from 2 to 84. A colon run broken by anything else prints in full: `"0:a:"` gives "1:a:", `"0::a"` "1:a", `"0: :"` "1: :", `d:"":` "1::". This is a formatter rule, not a cost rule. Escaped colons do not join a run (`TEXT(1,"0\:\:")` = "1::") and `h::mm` is "12:00" (probes/text-format-unquoted-letters A105:A111, T-1450; implemented T-1262).

Evidence: probes/text-date-255-char-limit, text-format-length-limit, text-format-length-limit-b (T-1388), text-format-length-limit-c, -d, -e (T-1393), -f, -g, -h, -i (T-1434), -j (T-1449), -k (T-1455), -l, -m (T-1481), -n to -s (T-1486), all excel-graph.
Not explained or not implemented (known failures):
- Unquoted letters after `_"` or `*"`: `_` and `*` take a following `"` as their character, so `@_"a` is "x a", `@_"aa` "x aa", `@_"a-"b"` "x a-b", `@*"a` "xa", `0_"a` on 7 "7 a" and `0*"a` "7a" (probes/text-format-length-limit-n A1, A6, -o A35, A37:A40; matched since T-1262 prints bare letters, except -o A37: `*` takes the `"` as its fill, which adds nothing in a number section, matched since T-1735). `@_"aaa` is `#VALUE!`, and so are 20, 40 and 80 letters (n A2, A7:A9): `aaa` is the weekday code, which `@` rejects (see "`aaa` is the weekday" below; matched since T-1880).
- `TEXT(1,"[>5]0"&REPT(" ",79))` is "1" (also with 40 spaces): an unmatched single condition prints General, dropping the padding. So is `"[>55555]0"` plus 79 spaces (-j A43), and `TEXT(1,"d"&<119 quoted>&"[>5]d")` is "1" (-q A16). Implemented by T-0936 (see "Conditional sections").
- `TEXT(1,"0:a:")` is "1:a:" and an escaped backslash followed by an unquoted `x` prints "\x": Excel accepts these letters unquoted (-g A42, A43; -i A100:A104). T-1450; implemented T-1262.

Cell display with a long format could not be probed: excel-graph rejects such workbooks (createSession 501 unsupportedWorkbook). The colon-run and text-section rules live in the shared formatter, so they apply to cell display too; they are probed through TEXT only.
Ours: `formatter/complexity.rs` (`rejects_format_code`: `exceeds_complexity_limit` plus `is_unterminated`), applied by TEXT only; colon runs in `formatter/parser.rs`; the text section with a number in `format_number` (formatter/format.rs).
Status: VERIFIED (excel-graph, fitted model)
Tasks: T-1388, T-1393, T-1434, T-1449, T-1450, T-1451, T-1455, T-1481, T-1486, T-1735, T-1880, T-2812, T-2836

TEXT over an array of format codes (T-1451): one result per format, value and formats broadcast together (see text.md). `TEXT(1,{"0",#N/A})` is "1", `#N/A` (-f A54:B54). The error-precedence and lifting rules are verified by probes/text-format-array (T-1458) for error and too-complex format elements; a syntactically invalid format element next to an error value is not probed.

## TEXT checks the code before it looks at a text or boolean value (T-2157)
Rule: a code that is `#VALUE!` for a number is `#VALUE!` for a text or boolean value too, even where the text section would
apply: `TEXT(" x ","TRUE")`, `TEXT("TRUE",TRUE)`, `TEXT("a",FALSE)`, `TEXT(FALSE,FALSE)`, `TEXT(FALSE,"FALSE")`,
`TEXT(TRUE,"#N/A")`, `TEXT("TRUE","HELLO ")`, `TEXT("a*",FALSE)` and `TEXT(FALSE,"x~*0.1Hello World")` are `#VALUE!` (a bare
`E` without a sign, an `N`, or a date token beside digit placeholders: "Bare letters and digits in a number format" above).
Ours returns the text or boolean unchanged. Also `TEXT(2.5,"a*?b1")` is `#VALUE!` (ours "a "); which part Excel rejects (the
part is `b1` after a prefix) was identified in T-2202 (below).
Probed (excel-graph, 2026-10-21, T-2202, probes/t2202-text-code-check-on-text, Calc!A1:A88; each code against the number 2.5,
"a", "" and TRUE): for every probed code the check is all-or-nothing across the four values: a rejected code is `#VALUE!` for every one of them, and
an accepted code is for none. Rejected: `"*?b1"`, `"ab1"`, `"*xb1"`, `"a*?b1"`, `"n"`, `"0E0"`, `"0b"`, `"0.b"`, and each of
the last four with an `@` text section added (`"n;n;n;@"`, `"0E0;0E0;0E0;@"`, `"0b;0b;0b;@"`, `"0.b;0.b;0.b;@"`), so an `@` section
does not rescue a rejected code. Accepted: `"a*?"`, `"b1"`, `"*?"`, `"b"`, `"a*?b"` (2.5 gives "a43"), `"0.0"`, `"@"`,
`"0;-0;0;@"`, `"\"x\""` and `"General"`; text, empty text and booleans come back unchanged ("a", "", "TRUE") under all of them.
Answer for `"a*?b1"`: the part Excel rejects is `b1` (a calendar prefix, "Buddhist-era year, era and calendar tokens" above) when something precedes it. Neither
`*?` alone (`"*?"`, `"a*?"`) nor `b1` alone (`"b1"`) nor `*?` followed by `b` without the digit (`"a*?b"`) is rejected, but
`b1` after the probed prefixes `a` (`"ab1"`), the fill `*x` (`"*xb1"`) and the fill `*?` (`"*?b1"`, `"a*?b1"`) is. Other prefixes
(quoted or escaped literals, digit placeholders, spaces, date tokens) were not probed; the likely rule, unprobed, is that a
calendar prefix is valid only at the start of a section. `TEXT(2.5,"b1")` is "" and `TEXT(2.5,"b")` is "43" (a Buddhist-year
date token).
Evidence (excel-graph, 2026-10-20, T-2114): gen-m5/25768de48ae7f2f9 Calc!A395, gen-m5/5704666097cbc5b2 Calc!A133,
gen-m5/651e713f5ae232ed Calc!A3, gen-m5/6b831f5e2a20bc34 Calc!A320, gen-m5/8a387c26d182e73c Calc!A281, gen-m5/8df4830f888f16ae Calc!A149,
gen-m5/a6fec430ac350c1a Calc!A331, gen-m5/c688ca172fe533dc Calc!A139, gen-m5/d1b251fd6e9c4b4b Calc!A120, gen-m5/d9fb1e918971111b Calc!A243,
gen-m5/f0f205865a3a4328 Calc!A262.
Ours (T-2157): `format_text_value` (formatter/format.rs) is `#VALUE!` when a number section of the code does not parse, and
the lexer rejects a `B1`/`B2` calendar prefix after any token that prints in its section (a locale bracket before it is
fine, probes/text-locale-calendars; a color, condition or currency bracket before it, and `0;b1`, are accepted too,
UNVERIFIED, T-2303), so the 11 gen-m5 cells above and the rejected probe codes pass. A bad section a number would
not use in a multi-section code (`TEXT("a","0;ab1")`) rejects a text value too: UNVERIFIED (T-2303). The fill characters of the
accepted codes (`TEXT(2.5,"a*?")` "a") add nothing, see "Fill characters" below (T-1735).
Status: VERIFIED (excel-graph); implemented (T-2157). Related: T-1130 (text values under 1-3 section codes).

## TEXT reads a number-like text value as its number; control characters are literals (T-2158)
Rule: a text value that reads as a number is formatted as that number: `TEXT("1","0,000.0")` is "0,001.0" and
`TEXT("$3.14","h:mm:ss.00")` (`DOLLAR(3.14159)`) is "3:21:36.00" (ours "1" and "$3.14"). A control character in the code prints
as a literal: `TEXT(100,CHAR(10))` is a line feed and `TEXT(-2.5,CHAR(1))` is "-" followed by CHAR(1) (ours `#VALUE!` for both).
Evidence (excel-graph, 2026-10-20): gen-m5/901e91c09b347914 Calc!A210, gen-m5/eb942f4a59b1c004 Calc!A54,
gen-m5/d586a1e0c9896942 Calc!A38, gen-m5/c0651f0e0fb544d0 Calc!A336.
Whatever the code's sections (T-2332, probes/text-number-like-text-and-control-chars, 29 cells, excel-graph 2026-10-24): number-like text is always formatted as its number, never as text. `TEXT("001","@")` is "1", `TEXT("1","0;0;0;@")` and `TEXT("1","0;0")` are "1", `TEXT("1","0;0;0;""@;""@")` is "1", and a quoted or escaped `@` is a literal: `TEXT("1","""@""0")` is "@1", `TEXT("1","\@")` is "@", `TEXT(1,"\@0")` is "@1". Text that does not read as a number takes the text section (`TEXT("abc","0;0;0;""x""@")` is "xabc", `TEXT("abc","0;0;0;""@;""@")` is "@;abc"); `TEXT("1","0;0;0;;")` is `#VALUE!`. Date and percent text coerce: `TEXT("1/2/2020","0")` is 43832, `TEXT("1/2/2020","m/d/yyyy")` "1/2/2020", `TEXT("50%","0.00")` "0.50", `TEXT("50%","0%")` "50%", `TEXT("1E2","0")` "100", `TEXT("$1","0")` "1", `TEXT("1,000","0")` "1000". Leading and trailing spaces are ignored (`TEXT(" 1 ","0")`, `("1 ","0")`, `(" 1","0")` are "1"; a tab or other whitespace is unprobed, and ours does not coerce `TEXT(CHAR(9)&"1","0")`; number-like text read from a cell reference is unprobed too). CHAR(9), CHAR(13), CHAR(127), CHAR(1) and CHAR(10) in the code are each one literal character (`LEN(TEXT(1,"0"&CHAR(n)&"0"))` is 3). Ours had kept text with an `@` or four sections as text (4 cells); fixed in T-2332.
Status: VERIFIED (excel-graph); implemented (T-2158, T-2332).

## Unquoted ASCII letters in a number format (T-1450)

Excel prints most ASCII letters that are not format tokens as literals, with no quotes or backslash. Probed as `TEXT(1,"0"&L)` and `TEXT(1,"d"&L)` for every ASCII letter in both cases (probes/text-format-unquoted-letters A1:A104, excel-graph 2026-10-14). This refines "Bare letters and digits in a number format" above: a date token next to a digit placeholder is an error, but with no digit placeholder it is a date token (`TEXT(100,"ABC")` = "A43C"; `TEXT(1,"0B")` is `#VALUE!`).

- **After `0`:** the letters `b d e g h m n s y` and their capitals (`B D E G H M N S Y`) are `#VALUE!` (date/time tokens do not fit next to a digit placeholder; the cause for `n` and `N`, which are not en-US date tokens, is unknown). Every other letter prints as a literal: `TEXT(1,"0a")` is "1a", and likewise `c f i j k l o p q r t u v w x z` and `A C F I J K L O P Q R T U V W X Z`. `TEXT(1,"0x")` is "1x".
- **After `d`** (a date format), the date tokens keep their meaning. The output is the day "1" followed by the token's value: `db` "143" (b is the two-digit Buddhist year, "43"), `dd` "01", `de` "11900" (e is the year 1900), `dg` "1" (g is the era, empty), `dh` "10", `dm` "11", `ds` "10", `dy` "100" (y is "00"). `dD`, `dG`, `dH`, `dM`, `dS`, `dY` match the lowercase. `dB`, `dE`, `dn` and `dN` are `#VALUE!`. The remaining letters (`a c f i j k l o p q r t u v w x z` and capitals `A C F I J K L O P Q R T U V W X Z`) print as literals ("1a").
- **Colons** (A105:A111): `TEXT(1,"0\:\:")` is "1::" (escaped colons do not collapse); `TEXT(0.5,"h::mm")` and `"h:::mm"` are "12:00" (a run collapses to one); `"0::"` and `"0:::"` are "1:"; `"0:a:"` is "1:a:" and `"0::a"` is "1:a".

Evidence strength: excel-graph oracle, 111 cells.
Implemented (T-1262): all 111 cells pass; escaped colons are `Token::Escaped` and do not join a colon run.
Status: VERIFIED (excel-graph)
Tasks: T-1262, T-1450

## `aaa` is the weekday (T-1880)

Rule: three or more of one `a`, all lowercase or all uppercase, are the weekday: `aaa` is the short name like `ddd`, `aaaa` and longer runs the full name like `dddd`. `TEXT(1,"aaa")` is "Sun", `"aaaa"` "Sunday", `"aaaaaaaaaa"` "Sunday", `"AAA"` "Sun", `"AAAA"` "Sunday", `TEXT(45000,"aaa")` "Wed", `TEXT(0,"aaa")` "Sat", `TEXT(60,"aaa")` "Wed" (the fictitious 1900-02-29). One or two letters, and any mix of cases, print as literals: `"a"`, `"aa"`, `"a aa"`, `"aAa"`, `"aaA"`, `"Aaa"`, `"AAa"`, `"aaAA"`, `"AAaa"` print as typed. The code is read from the first same-case run of three or more: `"Aaaa"` is "ASun", `"aAAA"` "aSun", `"aaaA"` "SunA", `"AAAa"` "Suna", `"aaaaA"` "SundayA". A run of three or more of the other case right after it joins the code and counts toward its length: `"aaaAAA"` and `"aaaaAAAA"` are "Sunday", not two names. The code is localized like `ddd` (`[$-407]aaa` "So", `[$-407]aaaa` "Sonntag", `[$-411]aaa` "日") and mixes with other tokens (`"h aaa"` at 0.5 "12 Sat", `"aaa d"` "Sun 1", `"ddd aaa"` "Sun Sun", `"[h] aaa"` "24 Sun", `"aaa AM/PM"` at 0.75 "Sat PM", `"aaa\a"` "Suna"). Because it is a date token, it is `#VALUE!` beside `@` (`TEXT("x","@aaa")`, `"aaa@"`, `"@ aaa"`, `"@_""aaa"`, `"@_""xaaa"`, `"@_""AAA"`; but `"@_""Aaa"` is "x Aaa" and `"@""a""aa"` "xaaa") and beside a digit placeholder (`TEXT(1,"0aaa")`, `"0 aaa"`, `TEXT(7,"0_""aaa")`), and a negative value in its section is `#VALUE!` (`TEXT(-1,"0;aaa")`). Quoted or escaped `a`s stay literal (`TEXT(1,"0""a""aa")` is "1aaa"). The other token letters after `@_"` are rejected the same way: `"@_""b"`, `"@_""bb"`, `"@_""bbb"`, `"@_""aab"` (its `b`), `"@_""ddd"`, `"@_""eee"`, `"@_""ggg"`, `"@_""yyy"`, `"@_""nnn"` (A6, A5, A33:A39), while non-token letters print (`"@_""ccc"` "x ccc", `"@_""aax"` "x aax"). Evidence: probes/text-format-aaa-weekday A1:A50, -b A1:A30, -c A1:A10 (excel-graph 2026-10-25). Unprobed: `aaa` directly before `m/pm` (`aaam/pm`), and a third joined run (`aaaAAAaaa`; the engine joins it); T-2745.

Status: VERIFIED (excel-graph)
Tasks: T-1880

## TEXT of a text value: `@` with other tokens in one section (T-1656)

Rule: a format whose section holds `@` is `#VALUE!` (for a text value, and for numbers too: see Sections below) when the same section also holds a date or number token, whichever side of the `@` it sits on: `y yy yyyy m mm mmm d dd ddd aaa h hh s ss b bb e ee g gg ggg`, `[h] [m] [s]`, `0 # ? . /`, `E+0`, `General`, `%` and a condition such as `[>5]`. The calendar prefixes are not among them: `TEXT(45000,"B2[$-409]@")` is "45000" (probes/t2729-hijri-locale-tag-names A289; `B1@` and a bare `B2@` too, probes/t2932-b2-calendar-twice A58, A59). These are literals, not errors: `a`, `,`, space, `\y`, `"y"`, `*-`, `_x`, `[Red]`, `[$-409]` (`TEXT("abc","x@\y")` is "xabcy"; `"x@_x"` prints a space for the spacer). `A/P` and `AM/PM` give a stray first letter or two: `TEXT("abc","x@A/P")` is "xabcA" and `"x@AM/PM"` "xabcAM" (`"A/P@"` "Aabc", `"AM/PM@"` "AMabc"). `@@` prints the text twice. A fill `*x` prints nothing in the text section (`"x@*-"` is "xabc") while `_x` leaves a space.
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
- **Resolution.** Hex LCIDs are case-insensitive and may carry a calendar digit above the low four (`[$-1040C]`, `[$-0040C]` are French). An LCID with no locale of its own falls back to its language (inferred; it holds only where the variants share names, so every regional variant that differs has its own table: see "Regional variants" below; T-1858) (`[$-80C]`, `[$-C]` French, `[$-7]` German, `[$-A]` Spanish, `[$-11]` Japanese, `[$-16]` Portuguese) and `[$-4]` is Traditional Chinese. 400 and 800 are en-US (A/P becomes AM/PM); `[$-0]`, `[$-40C0]`, `[$-2000000]`, `[$-FFFF]` and `[$-40C,1]` print English. BCP-47 tags match case-insensitively and fall back to their language: `[$-fr-FR]`, `[$-FR-fr]`, `[$-fr]`, `[$-fr-XX]` are French, `[$-de]` German (not LCID 0xDE), `[$-zh]` Traditional Chinese, `[$-xx]` English. A tag that is neither hex nor BCP-47 is read as hex anyway, every character a digit: a letter counts from 10 in either case (`g` 16, `r` 27, `z` 35), any other character below `a` from `0` (`_` 47, `-` -3), and one above `z` from `a` (`{` 36, `~` 39). Only `_` is probed among the other characters below `a`; `-` as -3 and the sum wrapping at 32 bits are inferred (no probed tag overflows). So `[$-g]` is Italian (0x10), `[$-h]` Japanese, `[$-3gC]` and `[$-2wC]` French (0x40C), `[$-6g40C]` Thai calendar over French (0x7040C), `[$-0_]` and `[$-_]` Macedonian (0x2F: "март|среда", "претпл."), `[$-1_]` Kazakh (0x3F), `[$-~]` Lithuanian, `[$-{]` Slovenian, `[$-x]` Indonesian and `[$-r]` Slovak, while `[$-28]` is Tajik ("март|чоршанбе"), so `_` is not 40. `[$-fr_FR]` (and `fR_fr`, `Fr_Fr`) is 0x10E00B: the Saka calendar over an LCID with no locale, so `mmmm` of 45000 is "December" (Saka month 12 by its en-US name). BCP-47 is tried first: `[$-fr-FR_x]` is French. A value with an LCID that has a sublanguage but no locale prints English: `[$-x_y]` (0x2412) is "March", not Korean; but `[$-_409]` (0xF409) prints "6 PM" for `h A/P`, so where Excel falls back to the language is not settled. Evidence: probes/t1924-locale-tag-extended-hex (excel-graph, 2026-10-26), text-locale-calendars A135-A145. Ours: implemented (`extended_hex`; T-1924), with the language fallback only for neutral LCIDs (below 0x400); the missing tables (0x21, 0x24, 0x27, 0x28, 0x2F, 0x3F) are T-1875's, the Saka cells T-2779's, `[$-fr_FR-x]`'s Myanmar digits (numeral byte 0x10) T-1925's, and 0xF409 is open.
- **System tags.** `[$-F800]` and `[$-x-sysdate]` replace the whole section with the system long date, en-US "dddd, mmmm d, yyyy", whatever else it holds (`TEXT(45000.75,"x[$-F800]0")` is "Wednesday, March 15, 2023", 0.5 "Saturday, January 0, 1900", -1 and 3e6 `#VALUE!`); `[$-F400]`/`[$-x-systime]` likewise give "h:mm:ss AM/PM". Only a first section shows it: `TEXT(-1,"0;[$-F800]")`, `TEXT(0,"0;0;[$-F800]")` and `TEXT("abc","0;0;0;[$-F800]")` are "". With `@` the section is a text section as usual (`[$-F800]@` of 45000.75 is "45000.75").
- **One tag per section.** A second tag in a section is `#VALUE!`, even the same one or a system tag (`[$-40C]mmmm [$-40C]mmmm`, `[$-40C]mmmm [$€-407]mmmm`, `[$-F800][$-F400]`); tags in different sections are fine.
- **Regional variants and more languages** (probes/text-locale-regional-lcids, 252 cells, T-1858). The language fallback does not hold across regions: `[$-C07]` de-AT has "Jän"/"Jänner" for January, "Mär" for March, and `ddd` "Mi."/"Do." (de-DE differs in all three); `[$-1004]` zh-SG mixes the two: short months as zh-TW ("一月"), days as zh-CN ("周三"); `[$-C04]` zh-HK equals zh-TW ("週三"); `[$-80A]` es-MX and `[$-2C0A]` es-AR differ from es-ES in `ddd` and the day halves ("a. m."/"p. m."), and es-AR's halves use a non-breaking space where es-MX's use a plain one. Languages that now resolve: `[$-408]` Greek ("Ιαν", "πμ"/"μμ"), `[$-41E]` Thai (Thai names, AM/PM stay Latin), `[$-41B]` Slovak (short months are the numerals "1"…"12", like Japanese), `[$-422]` Ukrainian (AM/PM stay Latin), `[$-401]` Arabic ("ص"/"م"), `[$-40D]` Hebrew (AM/PM stay Latin), `[$-439]` Hindi ("पूर्वाह्न"/"अपराह्न"). Calendar and digit switches of Thai, Arabic and Hebrew are T-1859's.
- **Not yet modelled:** locales not in the table list print English and keep `A/P` letters, though Excel very likely localizes them. Eras and calendar digits are in "Calendars of a locale tag" below.
- **A currency tag with a locale** (`[$€-40C]`, `[$€-fr-FR]`) is a symbol plus a locale tag in one: the symbol prints where it is written in a date section (`[$€-40C]d mmmm` "€15 mars", `d [$€-40C]mmmm` "15 €mars", `mmmm[$€-40C]` "mars€", `dddd [$€-40C]` "mercredi €", `[$€-40C]yyyy` "€2023", `[$€-40C]h:mm AM/PM` "€6:00 PM", `[$€]mmmm` "€March") and the locale localizes the section like `[$-40C]`. It counts as the section's one tag (`[$€-40C]mmmm [$-40C]mmmm` is `#VALUE!`; tags in different sections are fine). A system locale drops the symbol (`[$€-F800]` is the system long date). A multi-character symbol with a locale (`[$USD-40C]mmmm` is "USDmars") prints as text, the locale applying as above (implemented in T-2157). An empty or unresolvable locale on a currency tag still counts as the section's one tag: `[$€-]mmmm [$-40C]mmmm` and `[$€-zzz]mmmm [$-40C]mmmm` are `#VALUE!`, as is `[$€-40C]mmmm [$€-40C]mmmm`, while `[$€-]mmmm` and `[$€-zzz]mmmm` alone print "€March" in the default locale (and `[$€-]mmmm;[$-40C]mmmm` is fine: different sections). Format-code length limit (probes/t1883-currency-locale-edges, excel-graph, 2026-10-26): `[$€-40C]` + a quoted run works to 117 characters and fails at 118, + spaces works to 77 and fails at 78; `[$€]` + a quoted run works to 117 and fails at 118 (plain `[$-409]` allows 118 quoted and 78 spaces). The locale part of a currency tag therefore adds nothing: `[$€-40C]` and `[$€]` hit the same limits, on quoted text and on spaces (probes/t2812-currency-tag-cost A23:A28). T-2812 fits the cost (see "TEXT format code complexity limit"). Evidence: probes/text-currency-locale-date (excel-graph, 2026-10-18, T-1865), probes/t1883-currency-locale-edges (T-1883).
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

**Other calendar digits.** Probed again in probes/t1923-calendar-digits (excel-graph, 2026-10-26, T-1923) for every
month and weekday of A, B, C and 14, with day halves under 401 ("6 م") and 412 ("6 오후"):

| Digit | Calendar and names | Example |
|---|---|---|
| 2, D | Gregorian, en-US names | `[$-20411]mmmm` is "March" |
| 4 | Gregorian, zh-TW names; `yy` prints four digits | `[$-40409]ddd\|mmm\|yy` is "週三\|三月\|2023" |
| 5 | Korean years (Gregorian + 2333) for `yy`/`yyyy`; Korean names under a Korean LCID, en-US otherwise | "4356" |
| 7 | Thai years (+ 543) for `yy`/`yyyy` and `b`; Thai names | "2566", "มีนาคม" |
| 6 | Hijri, the LCID's Hijri names ("`B2` before a locale tag and calendar digit 6" above) | "1444-08-23", "Sha'ban", "Chaaban" |
| 9 | Gregorian, French names | "mars", "mer." |
| A | Gregorian, Levant Arabic months (`mmm` the same; two-word names joined by a no-break space: "كانون الثاني"), ar-SA weekdays | "آذار", "الأربعاء" |
| B, C | Gregorian, ar-SA names (`mmm` the same) | "مارس", "الأربعاء" |
| 14 | Gregorian, Korean native month names ("정월" … "섣달", `mmm` the same), Korean weekdays | "삼월", "수" |
| 15 and above | ignored; the LCID's own calendar (probed 15, 1E, 1F, FF) | `[$-FF040C]mmmm` is "mars" |
Under A, B, C and 14, `e` and `g` are probed ("2023", ""); `yy`, `b` and `bbbb` are inferred to print as under Gregorian (unprobed).

Under every one of these, `e` stays the Gregorian four-digit year, and so do `b` and `bbbb` except under Thai. Day halves follow the LCID under every calendar digit: `[$-20411]`, `[$-40411]` and `[$-50411]h AM/PM` are "6 午後", and `[$-70412]`/`[$-90412]` give "6 오후".

**Hebrew (8) and Saka (10)** (T-2779; probes/t1923-calendar-digits A85:A213, t2779-hebrew-saka-names, t2779-hebrew-saka-more, excel-graph 2026-10-26/27):
- 8 is Hebrew ("5783-06-22", "Adar", "Yom Revi'i"), the arithmetic calendar. Months number from Tishrei (1), so a leap
  year has 13: 5784's AdarI is month 6, AdarII 7, Elul 13 (2024-09-20 is "5784-13-17|Elul"). The year turns at Rosh
  Hashanah (2023-09-16 is "5784-01-01"). `yy`, `b`, `bb` and `y` print two digits, `e`, `ee`, `yyy`, `yyyyy` and `bbbb`
  the Hebrew year; `g` prints nothing. With a locale, `mmm`, `mmmm` and `mmmmm` all print the whole month name.
  - Under a Hebrew or Arabic LCID (40D, 401, 801, C01, 3C01, neutral D) the names are Hebrew script: months "תשרי" …
    "אלול", AdarI "אדר א" and AdarII "אדר ב" with a plain space; `ddd` is the weekday's letter ("א" … "ו", "ש") and
    `dddd` the he-IL name ("יום רביעי" with a no-break space, "שבת"; A211, A212).
  - Under every other LCID with a locale (409, 40C, 407, 411, 412, 404, 41E, 419, 439, every other table LCID, 400, 800
    and neutral C and 39) they are Latin: Tishrei, Cheshvan, Kislev, Tevet, Shvat, Adar (AdarI and AdarII in a leap
    year), Nisan, Iyar, Sivan, Tamuz, Av, Elul, and `ddd` = `dddd` = "Yom Rishon" … "Yom Shishi", "Shabat".
  - Serials 0 and 1 are "5660-05-01|Shvat", 59 "5660-06-29|AdarI". Excel matches the arithmetic calendar at every
    probed date to 2238-11-01, and ours follows it to 2238-11-07 (29 Cheshvan 5999); from Kislev 5999 its dates drift ahead (2238-12-01 is
    "5999-03-24", the arithmetic 23; 2239-09-24 "6000-01-01"; 4000-01-01 "7760-04-24", the arithmetic 7760-04-11), and
    from some date between 4000-01-01 and 5000-01-01 to 9999-12-31 they are `#VALUE!` (t2779-hebrew-saka-more A1:A35).
    Ours is `#VALUE!` past 2238-11-07 (T-2954).
- 10 is Saka ("1944-12-24", "Phalguna", "Budhavara"). The year turns on 22 March, or 21 March in a Gregorian leap year
  (2023-03-22 is "1945-01-01", 2024-03-21 "1946-01-01"), and Chaitra then has 31 days; months 2-6 have 31, 7-12 30.
  `yy`, `b`, `bb` and `y` print two digits ("44"), `e`, `ee`, `yyy`, `yyyyy` and `bbbb` the Saka year. Serial 1 is
  "1821-10-11|Pausa", and so is serial 0; 59 is "1821-12-09". 9999-12-31 is "9921-10-10" with `yy` "21".
  - Latin names under every LCID with a locale but an Arabic or Hindi one (40D and bn-IN 445 included): months Chaitra,
    Vaisakha, Jyaistha, Asadha, Sravana, Bhadra, Asvina, Karthika, Agrahayana, Pausa, Magha, Phalguna (`mmm` the same,
    `mmmmm` the first letter); `ddd` Ravi, Som, Mangal, Budh, Guru, Sukra, Sani; `dddd` Ravivara, Somavara, Mangalvara,
    Budhavara, Guruvara, Sukravara, Sanivara.
  - Under an Arabic or Hindi LCID (401, 801, 439, neutral 39) the months are Devanagari ("चैत्र" … "फल्गुण", `mmmmm` the first
    character) and `ddd` and `dddd` both the hi-IN `dddd` ("बुधवार").
- Under both, serial 60 (the fictitious 1900-02-29) is `#VALUE!`, even for a section with only time tokens
  (`TEXT(60.5,"[$-80409]h:mm")`); `TEXT(0.25,"[$-80409]h:mm|yyyy-mm-dd")` is "6:00|5660-05-01". A `B2` before the tag
  changes nothing (`B2[$-80409]yyyy-mm-dd|mmmm` is "5783-06-22|Adar"). Day halves follow the LCID (`[$-8040D]h A/P` is
  "6 PM").
- LCID 1 has no locale under both (`[$-80001]mmmm` of 2024-03-21 is "July", `[$-100001]` "December"), unlike
  neutral C, D and 39. With no locale the month names are en-US by the calendar's month number (`[$-80000]mmmm` of
  AdarII 5784 is "July", `[$-100000]` of 1 Chaitra "January"), and Hebrew month 13 prints nothing for `mmm` and
  `mmmm` and U+0001 for `mmmmm` (`[$-80000]m|mmm|mmmm|mmmmm` of 2024-09-20 is "13|||\u{1}").
- Digits A, B, C and 14 are Gregorian for `yy`, `b`, `bbbb` and `e` too (`[$-A0409]yy|b|bbbb|e` is "23|23|2023|2023").

**Numeral digits and LCIDs with no locale**:
- Numeral digits `[$-8000411]e` print native digits ("୫"; not modelled, T-1925).
- A calendar digit over an LCID with no locale (LCID 0 included) keeps its calendar's date. The names are en-US, except under 7 (Thai names) and A, B, C (Arabic), and `A/P` stays as written: `[$-E08DE]mmmm|yyyy-mm-dd` of 45000 is "February|2023-02-24", `[$-708DE]` "มีนาคม|2566-03-15", `[$-10D90B]` the Saka "December|1944-12-24", and `h A/P` is "6 P" under each (probes/t1924-locale-tag-extended-hex A13-A15, B13-B15). Underscore tags reach these through the extended hex reading above: `[$-de_DE]` (0xE0FDE), `[$-en_US]` (0xFA0FC) and `[$-ja_JP]` (0x13D049) give the lunar date in English, `[$-a_b_c]` (0xCFDFC) ar-SA names (T-1924). Digits 3, 4, 5, 9 and 14 over such an LCID are unprobed; ours leaves them unresolved.

**Lunar calendars (E, F, 11, 12, 13; T-1922).** Every date token prints the lunar date: serial 45000 (2023-03-15) is lunar 2023, month 2, day 24, and `yyyy`, `yy`, `e`, `b` and `bbbb` all print the lunar year (`[$-110409]bbbb|b|e|yyyy|yy` of 45300, 2024-01-09, is "2023|23|2023|2023|23"). `g` prints nothing.
- Months are numbered 1 to 13 (or 14) through the year, a leap month included: 2023's leap second month (from 2023-03-22) prints `m` "3" and `mmmm` "三月", and the twelfth month of lunar 2023 (from 2024-01-11) is month 13. Months 13 and 14 take the twelfth month's name ("十二月", "December", "섣달").
- Names: E gives the lunar months "正月", "二月" … "十二月" (`mmm` the same, `mmmmm` its first character, "正") with Japanese weekdays ("水", "水曜日"), or under a Korean LCID the Korean lunar months "정월", "이월", "삼월", "사월", "오월", "유월", "칠월", "팔월", "구월", "시월", "동짓달", "섣달" with Korean weekdays. F and 11 give the same Chinese months with zh-CN weekdays ("周三", "星期三") under every LCID, Korean included. 12 and 13 give en-US names by lunar month ("Feb", "February", "Wednesday"). Day halves follow the LCID ("6 午後", "6 下午", "6 PM").
- Two month tables: F and 11 follow the Chinese calendar; E, 12 and 13 start some months a day later (lunar month 5 of 2023 starts on 2023-05-19 under 11, 2023-05-20 under E). Between lunar new years 1960 and 2051 every month has 29 or 30 days; outside that Excel's calendar is not the published one: 1-, 15- and 28-day months, 13- and 14-month years (`[$-110409]m|d` of 1949-02-11 is "14|1"), new years off by up to a fortnight (1901-02-18 is "1/15"), and on 1960-01-01 and 2051-01-01 the year turns a month early (1959-12-31 is "1959/12/18", 1960-01-01 "1960/12/19", 1960-01-28 "1960/1/1").
- Serial 60 (the fictitious 1900-02-29) and negative values are `#VALUE!`; serial 0 reads as serial 1 ("1899/12/16"), its weekday aside. A lunar digit wins over `B2` before the tag (`B2[$-110409]yyyy|m` of 45300 is "2023|12"; since T-2729).
- Under LCID 0 the calendar still applies but the names stay en-US (`[$-110000]mmmm|d` and `[$-120000]mmmm|d` are "February|24"; modelled since T-1924, which keeps a calendar digit over any LCID without a locale).
- Implemented as tables of Excel's own output, to 2100 from probes/text-locale-lunar-months-1900 and past it as below.
- Past 2100 (probes/t2759-lunar-months-2101 and t2759-lunar-packed-2101 to -9101, excel-graph 2026-10-27, T-2759): Excel's lunar year follows the Gregorian year closely (2101-01-01 is "2100/12/17", 9999-01-01 "9998/12/16", 9999-12-31 "9999/11/26"), the 1900-1960 artefacts return (1-day months at the year turn, as 2105-01-30 month 13 then 2105-01-31 month 1; a 1-day month 14 on 2109-02-15; 15-, 28- and 31-day months), and month starts are not the true new moons (at 2101 about half a lunation off). Day numbers always count up by one within a month and months by one through a year, and the year turns only at month 1, with one exception: the early year of 1960 and 2051 recurs every 91 years (every 33,236 to 33,238 days) to 9999. On 1 January 2142, 2233, 2324, …, 9968 the year turns while the month is 12 (2141-12-31 is "2141/12/18", 2142-01-01 "2142/12/19"), and month 1 starts 27 days later without turning it again; both tables do this on the same days. No closed form was found (T-2759's progress notes: the starts track neither the true nor a mean new moon), so RustSheet stores Excel's month lengths to 9999 (formatter/lunar_2101.bin, packed from the probes by docs/research/t2759_lunar_tables.py). Only digits 11 and E were probed past 2100; F (with 11) and 12, 13 (with E) are assumed to keep their 1900-2100 pairing.
- `[$-E0411]h AM/PM|A/P|aaa|aaaa|B1yyyy|bbbb` is `#VALUE!` though each token alone prints (probes/text-locale-lunar-edges A32, text-locale-lunar-tokens A10-A49); RustSheet gives `#VALUE!` too, but which token causes it is not isolated, so the match may be by chance.
Evidence: probes/text-locale-lunar-months-1900 (1206 cells), text-locale-lunar-edges (61), text-locale-lunar-tokens (85) (excel-graph, 2026-10-26).

**`b` and `B1`/`B2` under a tag.** With no tag, `b` is the Buddhist year ("66|2566"). Under any resolved tag it is the Gregorian year ("23|2023"), except under calendar 7. `B1` and `B2` after a tag change nothing: `[$-40C]B2mmmm` is "mars" and `[$-401]B2yyyy-mm-dd` is "2023-03-15". An unresolved tag ignores `B2` too: `[$-zzz]B2mmmm` is "March", and `[$-FFFF]B2yyyy` and `[$-]B2yyyy` are "2023". The Hijri calendar digit is not ignored: `[$-60409]B2yyyy` is "1444". `B2` before a tag still selects Hijri: `B2[$-40C]mmmm` is "Chaaban" ("`B2` before a locale tag and calendar digit 6" above).

Evidence: probes/text-locale-calendars (145 cells), text-locale-calendars-detail (41), text-locale-japanese-year (20), text-locale-era-gregorian (23) (excel-graph, 2026-10-18).
Implemented in: formatter/locale_tag.rs (`resolve_lcid`, `Calendar`, `japanese_era`), formatter/lunar_calendar.rs (lunar tables, `lunar_2101.bin` past 2100), format.rs (years, `e`, `g`).
Status: VERIFIED (excel-graph)
Tasks: T-1859, T-1613, T-1922, T-1923, T-1924, T-1925, T-2759

## Displayed text (`Range.text`) of a formatted cell (T-1610)
Rule: excel-graph's `Range.text` is the cell's value under its number format, and it does not
depend on the column width. At width 4 a General `12345678` stays "12345678" and `=1/3` stays
"0.333333333" (the 11-character General rule of `TEXT(x,"General")`, not a width-fitted
shortening), and text is not cut ("long text"). A number a date format cannot show (a negative date
serial; more cases in "Displayed text over an edge-value pool" below) is 255 `#` characters, not a
width-sized `####`. The other probed cells:
General `1.5` "1.5", `1234567890123` "1.23457E+12", `1E-10` "1E-10"; `0.00` on PI() "3.14";
`#,##0.00` on -1234.5 "-1,234.50"; `0%`/`0.0%` on 0.256 "26%"/"25.6%"; `yyyy-mm-dd` on 45000
"2023-03-15"; `m/d/yyyy h:mm` on 45000.75 "3/15/2023 18:00"; `0.00;(0.00)` on -5 "(5.00)";
`0.00;-0.00;"zero"` on 0 "zero"; `dddd` on DATE(2024,2,29) "Thursday"; `# ?/?` on 12.5 "12 1/2";
`0.00E+00` on 123456789 "1.23E+08"; `[h]:mm:ss` on 0.5 "12:00:00"; `$#,##0.00` "$3.00"; TRUE
"TRUE"; `#DIV/0!` "#DIV/0!"; an empty reference "0"; `TEXT(1234.5,"#,##0")` "1,235".
Evidence: probes/displayed-text-formats S!A1:A24, B1:B4 (excel-graph `text`, 2026-10-16, T-1610).
Desktop `.Text` does depend on the width (`####` when a number does not fit), unprobed here: the
excel-graph text is what the M10 suite scores (decision 0068).
Implemented (T-2206): Excel rounds the shown place on the 15-digit decimal half away from zero, as
General and ROUND do, so `TEXT(1234.5,"#,##0")` is "1,235".
Not implemented: ours gives " 1/2" for `# ?/?` on 12.5 (T-2030).
Implemented in: formatter/formatted.rs (`Formatted::displayed`: every serial the formatter rejects as a
date, where TEXT is `#VALUE!`, displays 255 `#`; T-1711).
Status: VERIFIED (Excel for the web)
Tasks: T-1610, T-1711, T-2030, T-2206

## Displayed text over an edge-value pool (T-1611)
Rule: suite gen-m10 records Excel's displayed text (`Range.text`) of 135 format codes (every built-in,
custom codes with 1-4 sections, conditions, colors, literals, `*`/`_`, `@`, elapsed and fractional
seconds, fractions, scientific, scaling commas, percent) over 44 edge values, 10,400 cells, plus
TEXT() with the same codes. Points the earlier probes did not show:
- A `*x` fill adds nothing to the text: `_("$"* #,##0.00_);...` on 36.5 is " $36.50 ", `*.0.00` and
  `0.00*-` on PI() "3.14"; `_x` still adds one space (`_(* #,##0_)` on 1000 " 1,000 ").
- A date or clock format over a serial it cannot show (negative, or 1234567890 past 9999-12-31) is
  255 `#`; 787 such cells. An elapsed code with a clock or fraction token joins them (`[h]:mm`,
  `[m]:ss`, `[ss].00`, `[h]:mm:ss.0` on any negative, even -0.5, and on 1234567890; so do plain clock
  codes `h:mm`, `h AM/PM` past 2958465.99999, gen-m5 6a511beadb56b932 Calc!A126, 6a8bb7cd6d7769ad
  Calc!A299, a6fec430ac350c1a Calc!A356), but a bare
  elapsed code does not: `[s]` on -1.5 is "-129600" and on 1E15 "86400000000000000000" (see
  "Elapsed-time tokens"). TEXT() on the same codes is `#VALUE!` exactly where the display is 255 `#`;
  every other paired TEXT() result equals the displayed text.
- A fraction whose numerator overflows is 255 `#` too: `?/100` on 1000, 999.9999 and 1234567890,
  `?/?` on 123456789012345 and 1E15 while 1234567890 is "1234567890/1". The thresholds: "Fraction
  formats", "A numerator that overflows".
- Clock and date fields come from the serial rounded to the shown second: 0.9999999999 is "0:00"
  under `h:mm`, "24:00:00" under `[h]:mm:ss`, "1900-01-01" under `yyyy-mm-dd`, "Sunday" under `dddd`.
  Also `[m]:ss` "1440:00", `d \d\a\y\s h:mm` "1 days 0:00", `dd/mm/yy` "01/01/00" (VERIFIED, gen-m10, T-1732). Implemented in
  `DisplayClock` (formatter/elapsed.rs, rounded to whole seconds): clock fields and the day always
  round, and so does an elapsed count with or without a seconds token: `[h]` on 0.4 s short of 3 h is "3", `[h]:mm` "3:00",
  `[m]` "180"; 0.6 s short they are "2", "2:59", "179" (VERIFIED, probes/elapsed-no-seconds-boundary A1-A29, T-2385).
  In the same probe: `[h]:mm AM/PM` is #VALUE! (A31, T-2683; the general rule, an elapsed token before AM/PM or A/P, is in dates.md and implemented in the section parser, T-1366), `[h]:mm.0` on 0.6 s short of 3 h is "2:59.4", the
  tenths of the seconds after the minutes with no seconds token (A32, T-2684), and `[h]:mm` of a negative is #VALUE!
  while `[h]` and `[m]` alone show "-2" and "-4" (A24-A26).
- `mm:ss` with no hour is minutes: 123456.789 is "56:10".
- The text section formats booleans as well as text: `"pre "@` on TRUE is "pre TRUE", `;;;` shows
  nothing, `0.0;[Red]-0.0;0.0;"<"@">"` on an empty string "<>".
- The accounting formats' text section `_(@_)` pads a text, a boolean or an empty string with one space on each side:
  `_(* #,##0_);_(* \(#,##0\);_(* "-"_);_(@_)` on "text" is " text " (gen-m10 05c1c646c95a2d60 F!M4), on TRUE " TRUE "
  (eec351e03ccedfe0 F!R2) and on `=""` two spaces (5255389143dbcef6 F!H11); the `$` variants do the same
  (0e3d8eba9432a18d F!A13, F!A20). 27 cells across the four built-in accounting codes (gap triage 2026-10-25). Ours shows
  the plain value (T-1736).
- A minus goes before a leading literal: `"x"General` on -1.5 is "-x1.5" (implemented, T-2097: the minus leads and the General token prints the absolute value; gen-m10 F!D19 -1E15 is "-x1E+15"). A value that rounds to Unprobed: literals after General on a negative (`General x`, unchanged here).
  nothing shows no minus: `#,` on -0.5 "", `0.0,,` on -1234.5678 "0.0"; `#,` on 1.5 is "" and on
  1000 "1".
- `?.??` keeps the point on an integer ("1000.  "); `#%` on 0.0049 is "%".
- Two conditions: `[<0]"neg";[>0]"pos";"zero"` on 1 is "pos"; `[=0]"nil";General` on PI() is
  "3.141592654"; `[<=-1]0;[<1]0.0000;0.0` on -1.5 "2"; `[>=1]0.00;0.000` on -0.5 "-0.500".
- `[$USD]` is currency text ("-1.50 USD") and `[Cyan]` a color.
- Engineering and padded mantissas: `##0.0E+0` on 123456.789 is "123.5E+3", `00.0E+0` on 36.5
  "36.5E+0", `0.000E-00` on 1.5 "1.500E00".
- Rule (T-1730, T-2226): the exponent is the decimal exponent rounded down to a
  multiple of the integer-placeholder count (`##0.0E+0` on 61 "61.0E+0"); a zero exponent prints `E+00`
  (never `E-00`) and `E00` under `E-`. The group size for grouped codes (`#,##0.0E+0`) is unprobed.
- Implemented (VERIFIED, gen-m10, T-2226): an engineering mantissa (two or more integer placeholders) rounds after
  the shift, to its own integer digits plus the decimals (`##0.0E+0` on 1/3 "333.3E-3", on -1/7 "-142.9E-3",
  `##0.00E+00` on 1/3 "333.33E-03", `00.0E+0` on 0.125 "12.5E-2"; 28 gen-m10 text cells). Ours: a rounding carry
  into the next power of ten moves the exponent to the next multiple (`##0.0E+0` on 999.96 "1.0E+3"); see the
  T-2598 bullet below for the probe evidence.
- Implemented (T-2598; VERIFIED, probes/t2598-engineering-scientific, excel-graph 2026-10-27; ours matches all 12
  cells): an engineering mantissa that rounds up to the next power of ten renormalises only when it reaches 10 to the
  number of integer placeholders. `##0.0E+0` on 999.96 is "1.0E+3", on 9999.96 "10.0E+3", on -999.96 "-1.0E+3", but on
  99.96 it stays "100.0E+0" (100 is a legal three-digit mantissa). `00.0E+0` on 9.96 is "10.0E+0" and on 99.96 it is
  "01.0E+2" (100 needs three digits, so the exponent moves to the next multiple of 2 and the mantissa is padded).
  `##0.00E+00` on 999.995 is "1.00E+03". The mantissa rounds on the 15-digit decimal, half away from zero (T-2206):
  `##0E+0` on 0.0145 is "15E-3" (the double is 0.01449999…, so only this cell tells decimal from binary
  rounding), on 0.0125 "13E-3", on 1.5E-5 "15E-6". The group size of a grouped code counts
  as integer placeholders: `#,##0.0E+0` has four integer placeholders, so the exponent is a multiple of 4 and the mantissa keeps its
  group separator: 1/3 is "3,333.3E-4" and 12345.6 is "1.2E+4".
- Not implemented (VERIFIED, gen-m10): zero prints all integer placeholders (`##0.0E+0` on 0
  "000.0E+0", `##0.00E+00` "000.00E+00", `#E+0` "0E+0") and a carry renormalises (`0.0E+00` on 99.5
  "1.0E+02") (T-2227).
- Implemented (T-2705; VERIFIED, gen-m10 and probes/t2705-scientific-mantissa-rounding, excel-graph 2026-10-25): a
  scientific code with one integer placeholder rounds its mantissa once, from the value, at its shown digits (one
  plus the decimals, at most 15). The rounding is on the 15-digit decimal, half away from zero, as the fixed codes
  and ROUND round (T-2206). `0E+0` and `#E+0` on 2.5 are "3E+0" and on -2.5 "-3E+0". `0.00E+00` on 1.005 (the double
  1.00499999…) is "1.01E+00", and on 2.675 it is "2.68E+00". `0E+0` on 2.5-2^-51 is "3E+0", `0.0E+0` on 0.285 is "2.9E-1",
  and on 4.35 it is "4.4E+0" (probe A1:A6, A12). `0.00E+00` on 12.345 is "1.23E+01" (12.345 is the double
  12.34500000000000063…, no tie). Ours showed "1.24E+01" because it rounded twice: first to the integer digits plus
  the decimals (12.35), then the mantissa 1.235 again. Engineering codes (T-2598) and zero mantissas (T-2227) are
  separate. A `%` after the exponent does not scale (`0.00E+00%` on 2.675 is "2.68E+00%", probe A13; see the
  next bullet). Cells: displayed text 05c1c646c95a2d60 F!I10 and F!L11, 5255389143dbcef6 F!D13 and F!G20,
  5eb95c63c7d6180c F!G8 and F!G19, 659cd5776d423ce3 F!B10, b4f66526f822ff2c F!L18, eec351e03ccedfe0 F!Q15,
  fbdab487cb11409e F!Q3 (and F!Q20, bfbcd16cee266336 F!W12); TEXT 5eb95c63c7d6180c T!A7 `TEXT(2.5,"#E+0")` "3E+0".
- Implemented (T-2720; VERIFIED, probes/t2720-scientific-percent, excel-graph 2026-10-25): a scientific code never
  scales its value. A `%` anywhere in it is shown as written: `0.00E+00%` on 2.675 is "2.68E+00%", `0.00%E+00` is
  "2.68%E+00", `%0.00E+00` is "%2.68E+00", `0.00E+00%%` is "2.68E+00%%" and `0.0%0E+00` is "2.6%8E+00". On 0.5, `0%E+0`
  is "5%E-1". Under an engineering code, `##0.0E+0%` on 2.675 is "2.7E+0%". A trailing or embedded `,` is dropped
  without dividing: `0.00E+00,` and `0.00,E+00` on 2675 are "2.68E+03". The parser gives a scientific part no
  `percent` or `comma` scaling (formatter/parser.rs). Unprobed: a fraction followed by an exponent (`# ?/?%E+0`),
  which the same rule covers.
Evidence: eval/suites/gen-m10 (excel-graph `text`, 2026-10-16, T-1611); the report's "Displayed
text" section lists every cell ours gets wrong.
Implemented (T-2206): the shown place rounds the 15-digit decimal half away from zero (`0`, `#,##0`,
`0%`, `"("0")"` on 2.5, -2.5, 36.5, 0.125 and 12.345 are "3", "-3", "37", "13%" and "1235%"). Rounding now runs
before the sign test, so a negative that rounds to one unit keeps its minus (-0.5 under `0` is "-1",
`00000` "-00001", `[$¥-411]#,##0` "-¥1"; gen-m10 810fee377a42575b F!B16, e1c29d3fe448682c F!B19,
9a8b7d68507d7ada F!B1) and 0.0049 under `0%` is "0%" and under `%0` "%0" (no double rounding through 0.5;
gen-m10 659cd5776d423ce3 F!F16, 8e073c5b2145567a F!C10). A tie at the 16th significant digit goes toward zero, as ROUND
and General do (unprobed for displayed formats).
Implemented in: formatter/formatted.rs (`Formatted::displayed`, T-1711): every serial the formatter
rejects as a date (TEXT `#VALUE!`) displays 255 `#`; formatter/format.rs (T-1731) rejects any date or
clock token other than a lone elapsed token past serial 2958465.99999.
A `?` that pads the first decimal prints the point before its space, whatever follows it (T-1738,
probes/t1738-question-first-decimal, excel-graph 2026-10-25): `TEXT(3,"0.?")` and `TEXT(3,"#.?")` are "3. ",
`"0.?0"` "3. 0", `"0.??"` and `"#,##0.??"` "3.  ", `"0.?#"` "3. ", `"?.??E+00"` on 1000 "1.  E+03", `"?.??"` on 0
" .  " and `"0.?"` on -3 "-3. "; implemented in formatter/format.rs. `#%` on 0.0049 is "%" since T-2206's rounding.
Implemented since: `[$USD]` currency text (T-2157) and `[Cyan]` (T-1737).
Not implemented: before T-2206 ours passed 8,797 of 10,400 text cells (84.6%). fraction overflow (T-1740), second rounding (T-1732), `mm:ss` (T-1733), conditions (T-1734),
the text section on text and booleans (T-1736), scientific (T-1730), fractions (T-2029, T-2030, T-0982; "Fraction formats" below), the sign of a value that rounds to zero and `#,` on -500 (T-1129), fractional seconds (done in T-0934), 15 significant digits (T-0980), `"x"General` (T-1614).
Status: VERIFIED (Excel for the web)
Tasks: T-1611, T-1711, T-1730 to T-1738, T-1740, T-2029, T-2030, T-2206

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
  triage 2026-10-19 on all 317 gen-m10 text cells that show an n/d (the closest fraction fits 308).
  Implemented (T-2030): the continued fraction of the binary fraction part, stopping at the first
  convergent whose denominator would not fit or when the remainder drops below 1e-12. Every one of
  the 488 placeholder-denominator cells matches with that rule, but the 1e-12 cut-off is not pinned
  down by them (no cell has a remainder near it); 123456.789 (fraction part 0.78900000000430737...)
  gives 15/19 under `??/??` either way.
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
  With no integer part a numerator that rounds to zero takes no minus: `?/100` on -0.001 "0/100" and `?/00`
  "0/01" (probes/fraction-format-features F9, F13, excel-graph, T-2074).
- **Grouping:** a comma between integer placeholders groups the integer digits: `#,##0 ?/4` on 1234.5
  "1,234 2/4" (probes/fraction-format-features I10, T-2074).
- **Padding:** `?` pads the numerator on the left and the denominator on the right (`# ???/???` on 1.5
  "1   1/2  ", 05c1c646 C3; `??/??` on 0.5 " 1/2 "); `#` does not (`# #/#` on 0.5 "1/2", on 1234567890
  "1234567890", 039accab K8, K2). Under a `#` numerator the literal between the integer and the
  fraction goes too when either side prints nothing ("1/2", not " 1/2"; "1234567890" with no
  trailing blanks), yet stays when both print (`# #/#` on 1.5 "1 1/2").
- **A numerator that overflows** is 255 `#` as displayed text and `#VALUE!` from TEXT (VERIFIED,
  probes/fraction-numerator-overflow and -overflow2, excel-graph 2026-10-21, T-1740; cells A<n> of sheet S,
  TEXT on T). Only a fraction with no integer part overflows. With a fixed denominator the rounded
  numerator may be at most 32767 (2^15 - 1) in magnitude, whatever the denominator: `?/1` on 32767 is
  "32767/1" and on 32768 255 `#`; `?/2` 16383.5 / 16384, `?/10` 3276.7 / 3276.8, `?/100` 327.67 / 327.68 and
  -327.67 / -327.68, `?/1000` 32.767 / 32.768 and `?/16` 2047.9375 / 2048 alike (overflow2 A1-A52). The
  numerator is the 15-digit rounded one: `?/100` on 327.674 is "32767/100" and on 327.675 255 `#`
  (A55-A56). With a placeholder denominator (`?/?`, `??/??`, `???/???`) the value may be at most just under
  2147483647 (2^31 - 1) in magnitude: `?/?` on 2147483646.9 is "2147483647/1", on 2147483647, 2147483647.5,
  -2147483647 and 2147483648 255 `#`; -2147483646 is "-2147483646/1" (overflow A43-A64, overflow2 A58-A64).
  A whole part takes the rest: `# ?/100` on 1E15 is "1000000000000000      ", `# ?/?` on 2147483648.5
  "2147483648 1/2" and `0 ?/10` on 1E14+0.5 "100000000000000 5/10" (overflow A37-A42, A74-A76).
  Implemented in `format_fraction` (formatter/fraction.rs). Both bounds apply to the value after `%` scaling
  (VERIFIED, probes/fraction-overflow-scaled A1-A27, excel-graph 2026-10-27, T-2522): `?/?%` on 21474836.469 is
  "2147483647/1%" and on 21474836.47, 21474836.48 and -21474836.48 255 `#`; `??/??%` on 21474836.46 "2147483646/1 %";
  `# ?/?%` on 21474836.48 "2147483648    %"; `?/100%` on 3.27674 "32767/100%" and on 3.27675 255 `#`; the 2^30 stop
  rule below too (`?/?%` on 10737418.245 "1073741825/1%", 10737418.235 "2147483647/2%"). For `,` scaling see
  "Scaling commas in fraction formats"; for denominators past 32767 "Fixed denominators past 32768".
- **Many scaling commas wrap** (VERIFIED, probes/t2842-dollar-run-contexts K1:K3, K10:K11, excel-graph 2026-10-27, T-2842):
  outside a fraction, Excel keeps the scaling exponent -3n of n trailing commas in a signed byte, so `TEXT(5,"0"&REPT(",",252))`
  is "5000000000000" (-756 wraps to 12), 253 commas "5000000000" (9), 254 "5000000" (6), and `[$-409]0` + 246 commas
  5 followed by 30 zeros. Counts from 43 (where -129 first wraps) to 245 are unprobed; the engine applies the same wrap.
- **Scaling commas in fraction formats** (VERIFIED, probes/fraction-overflow-scaled, excel-graph 2026-10-27, T-2522).
  A `,` right after the denominator (`?/?,`, `# ?/4,`, `?/4,%`) multiplies the value by 10^((-3n) mod 256) for n such
  commas, as if Excel kept the exponent -3n in an unsigned byte: 10^253 for one, 10^250 for two, 10^247 for three. So
  `?/?,` on 1E-250 is "1000/1", on 1E-290 and 1E-300 "0/1", and on 0.5, 1, 1500 or 1E-200 255 `#` (TEXT `#VALUE!`)
  under the 2^31 - 1 bound; `?/4,` on 1E-250 "4000/4"; `?/?,,` on 1E-250 "1/1", 1E-247 "1000/1"; `?/?,,,` on 1E-238
  "1000000000/1". With an integer part the integer prints in full: `# ?/4,` on 1E-12 is "1" + 241 zeros + 4 spaces,
  and on 2 the text would pass 255 characters, so 255 `#` and `#VALUE!`. Zero is unscaled (`?/?,` on 0 "0/1", `# ?/4,`
  "0    "). The scaled value is cut to 15 significant digits: `?/?,,` on 1E-251 is "0/1" (0.1, whose double product
  0.10000000000000002 would pick 1/9; A65, the only cell that tells). A `,` anywhere else prints or vanishes: after a
  `%` or a literal it is a literal (`?/?%,` on 1E-252 "0/1%,", `?/?% ,` "0/1% ,", `?/? ,` "0/1 ,"), and a `,` after the
  integer neither scales nor groups (`0, ?/4` on 2 is "2    ", on 1234567.5 "1234567 2/4"; `#, ?/4` alike). Outside a
  fraction a `,` after `%` is a literal too: `0%,` on 0.5 is "50%," and on 1234567 "123456700%,". Implemented in the
  parser (`Token::Comma`) and `fraction_scale` (formatter/fraction.rs). Unprobed: 86 or more commas (the exponent wraps
  past 0), a `,` between the numerator and the `/`.
- **Fixed denominators past 32768** (VERIFIED, probes/fraction-large-fixed-denominator and -six-digit-denominator,
  excel-graph 2026-10-27, T-2522). Up to 32768 a fixed denominator is used as written (`?/32768` on 0.5 is
  "16384/32768", on 1 255 `#`, the numerator bound 32767). From 32769 to 99999 Excel ignores it and picks the
  denominator as for placeholders, the last convergent with a denominator up to 9999999 whatever the code's digit
  count, under the placeholder value bound and 2^31 stop rule (`?/40000` on 0.5 "1/4    ", 0.8192 "512/400  ",
  0.8191874 "4095937/40000", 1/9999999 "1/40000", 1E-7 "0/4    ", 2147483646.9 "2147483647/4    ", 2147483647 255 `#`;
  `?/32769` on 1/9999999 "1/32769"). The code's digits print in place of the denominator's, cut to its digit count
  (625 shows "400", 5000000 "40000"), and pad with spaces to the code's width under a `?` numerator and not under a `#`
  or `0` one (`??/40000` on 0.5 " 1/4    ", `0/40000` and `#/40000` "1/4", `# #/40000` on 1.5 "1 1/4", `# ?/40000`
  "1 1/4    "). Six or more digits (`?/100000`, `?/123456`) are #VALUE! from TEXT for every value, and Excel for the
  web refuses a workbook that applies one as a cell format (FileCorruptTryRepair), so its displayed text is unknown.
  Implemented in `format_fraction` (`MAX_FIXED_DENOMINATOR`, `LARGE_FIXED_MAX_DENOMINATOR`) and the parser. Unprobed: a
  numerator mixing `?` with `#` or `0`, a blank fraction (`# ?/40000` on a whole number), and
  whether the #VALUE! rule is about the digit count or the value (only 6-digit codes of 100000 or more were probed;
  7+ digits and leading zeros are extrapolated).
- **Placeholder fractions past 2^30** are not the last convergent (VERIFIED, excel-graph 2026-10-21, T-1740,
  probe cells kept out of the suites until the follow-up implements them; T-2233): `?/?` on 1073741824.5 is
  "1073741825/1" (not 2147483649/2), -1073741824.5 "-1073741825/1", 1431655765.5 "1431655766/1",
  1610612735.5 "1610612736/1", 2147483646.5 "2147483647/1", 1073741824.25 and 1073741823.75 "1073741824/1",
  1E9+0.25 "1000000000/1", 2147483646.4 "2147483646/1", yet 2147483646.6 "4294967293/2" (not 10737418233/5)
  and 715827882.5 "1431655765/2", 5E8+0.25 "2000000001/4"; `??/??` on 2147483646.5 "2147483647/1 " and on
  1073741824.5 "1073741825/1 ". No bound on the numerator or on the denominator times the value fits every
  cell (4294967293/2 is refused for 2147483646.5 and taken for 2147483646.6).
  Rule (fitted, T-2233, probes/fraction-placeholder-past-2-30, 75/75 cells, excel-graph 2026-10-23): the
  continued fraction stops where a partial quotient `a` times the last numerator `p1` passes 2^31 - 1, while the new
  numerator `a*p1 + p0` may pass it (only the product is checked). 2147483646.6: 1 x 2147483647 is taken, so
  4294967293/2 shows; then 2 x 4294967293 stops. 2147483646.5: 2 x 2147483646 stops. If that stop comes before any
  fractional term (the last convergent still has denominator 1), the value is rounded half away from zero to an
  integer over 1: 1073741824.5 "1073741825/1", 1073741824.25 "1073741824/1". At the bound: 1073741823.5 "2147483647/2"
  (2 x 1073741823 = 2^31 - 2 is taken). The sum alone passing 2^31 is taken: 1500000000.6 "3000000001/2", and
  715827883+1/3 (the double 715827883.3333334, partial quotients 2 then 1) "2147483650/3", where 1 x 1431655767 is under
  the bound but 1431655767 + 715827883 is past 2^31; 715827882+1/3 "2147483647/3". Under `%` scaling the rule applies to the scaled value (T-2522, above).
Evidence: eval/suites/gen-m10 (excel-graph `text`, 2026-10-16, T-1611); 829 numeric text cells under 12 fraction
codes, all failing in the 2026-10-19 report.
- **The numerator rounds at 15 significant digits**, as ROUND does: `?/100` on 12.345 is "1235/100" (05c1c646 D11)
  (the double 12.345 is 12.34500000000000063…), while `# ?/10` on 12.345 is "12 3/10" (43980980 P16; the binary
  fraction part 0.34499999999999886 × 10 stays below the half).
Probed (T-2061, excel-graph 2026-10-19, probes/fraction-format-features, cells named by row of column A; values
B1:I1 = 1.5, -0.5, 0, 1.99, -0.001, 1, 0.5, 1234.5):
- **Currency and leading literals** sit after the minus: `$# ?/4` on -0.5 "-$ 2/4" (C2), `"x"# ?/4` "-x 2/4" (C3),
  `€# ?/4` (C17) and `[Red]$# ?/4` (C16) alike; a tiny negative keeps the minus: "-$0    " (F2).
- **`%` scales the value before the fraction** and prints after the blanked fraction: `# ?/4%` on 1.5 "150    %" (B4),
  1234.5 "123450    %" (I4), -0.001 "-0    %" (F4).
- **A trailing `,` after the denominator scales the value up, not down by 1000**: `# ?/4,` is #VALUE! for every value but 0, which prints
  "0    " (B5:I5); `# ?/4,,` multiplies the value by 1E250 and prints the integer in full with the fraction blanked ("15" + 249
  zeros + 4 spaces on 1.5, B6; -0.001 "-1" + 247 zeros, F6; 253 to 255 characters), and is #VALUE! on 1234.5 (I6),
  whose result would pass the 255-character TEXT limit. The single `,` row is #VALUE! because one comma multiplies
  by 10^253 (settled by T-2522, "Scaling commas in fraction formats" above; implemented, rows 5-6 pass).
- **`0` numerator placeholders print**: a zero numerator is not blanked, it pads with zeros: `# 00/16` on 0 "00/16",
  on 1.99 "2 00/16" (D8, E8), and `# 0/4` on 0 "0/4", 1 "1 0/4" (D15, G15); the integer part is dropped under `#` when
  zero and the separator goes with it (`# 00/16` on 0.5 "08/16", H8; `# 0/4` on 0.5 "2/4", H15; -0.5 "-08/16", "-2/4",
  C8, C15). `# #/4` blanks the fraction and its separator: on 0 "0", 1.99 "2", -0.001 "-0", 1 "1" (D7:G7), on
  0.5 "2/4" (H7), -0.5 "-2/4" (C7). With `0` as the integer, `0 #/4` on 0.5 is "0 2/4" and on -0.5 "-0 2/4" (H12, C12):
  the separator stays.
- **A tiny negative loses its minus in a format with no integer section**: `?/100` on -0.001 is "0/100" (F9), as is
  `?/00` "0/01" (F13); formats with an integer section keep it ("-0    ", F11; `# 00/16` "-00/16", F8; `# 0/4`
  "-0/4", F15).
- **`?/100` on 1234.5 is #VALUE!** (I9), although 12.345 and 61 print (05c1c646 D11, D5).
- **Grouping applies**: `#,##0 ?/4` on 1234.5 "1,234 2/4" (I10), on -0.5 "-0 2/4" (C10).
- **`00` in a denominator is a placeholder denominator, not a fixed 00**: `?/00` is the best fraction of at most
  two digits, zero padded to two: 1.5 "3/02", -0.5 "-1/02", 0 "0/01", 1.99 "197/99", 1 "1/01", 0.5 "1/02",
  1234.5 "2469/02" (B13:I13); `# ?/00` on 1.5 "1 1/02", 1.99 "1 98/99", 1 "1     " (B14:G14).
Ours: fixed denominators are implemented as above (T-2029, formatter/fraction.rs), in TEXT and the displayed text.
Mismatches with the probes (18 of 128 cells after T-0982, which fixed T-2073 and the `00` padding) are filed as
T-2072 (`,` after the denominator) and T-2074 (the minus of `?/100` and `?/00` on a tiny negative, `?/100` on
1234.5, the grouped integer). Placeholder denominators are implemented as above
(T-2030). An overflowing numerator still prints in full (T-1740).
Literals between the parts (poi/NumberFormatTests.xlsx Tests!A120:A332, desktop Excel cached values, Mac Excel
16.0; `=TEXT(Cn, Bn)` with `|` literals around each code; a 3×3×3 matrix `|I\:N=/=D|` of integer, numerator and
denominator kinds over 0, ±0.75, ±1, T-0982):
- **Literals may stand on either side of the `/`** (`=/=`, `\:`, quoted text, `_x`): the numerator is the last run of
  placeholders before the `/`, the denominator the first after it: `|#\:#=/=#|` on 23.75 "|23:3=/=4|" (A134);
  `|#_#/#|` on 3.75 "|15 /4|" (A165: `_#` is a space, so there is no integer part).
- **`0` in a placeholder denominator pads on the left**: `|#\:? ?0#/000` on 23.75 "|2:3  03/004" (A160); `?` pads
  on the right as before.
- **A zero numerator prints when the numerator has a `0`**: `|#\:0=/=#|` on 1 "|1:0=/=1|" (A258); on 0 the
  integer is dropped under `#` with its separator ("|0=/=1|", A204) and prints under `?` or `0` ("|0:0=/=1|",
  A213, A224).
- **Otherwise a zero numerator blanks the fraction**: into spaces of its full width, separator and literals
  included, under a `?` numerator, a `?` integer or a `?` in the denominator (`|#\:#=/=?|` on 1 "|1      |",
  A253; `|?\:#=/=#|` A261); into nothing, separator included, when all of them are `#` or `0`
  (`|#\:#=/=#|`, `|#\:#=/=0|`, `|0\:#=/=#|` on 1 "|1|", A252, A254, A270).
- **The separator after a zero integer** prints under a `0` integer ("|0:3=/=4|", A243), turns into spaces under a
  `?` integer ("|  3=/=4|", A234) or under a `#` integer before a `?` numerator ("| 3=/=4|", A228), and is dropped
  under a `#` integer otherwise ("|3=/=4|", A225, A231). Literals between integer placeholders always print
  (`|#-#-#\:#/#|` on 0.75 "|--3/4|", A129).
Ours: implemented in formatter/parser.rs (`is_fraction_literal`) and formatter/fraction.rs (T-0982); all 325 cells pass.
Unprobed engine choices (T-2209): a mixed `0`/`?` denominator puts its zeros left and spaces right; plain spaces or quoted
text around the `/` also make a fraction; a `/` with no denominator after its literals stays a literal.
Status: VERIFIED (Excel for the web; desktop Mac Excel cached values for the literals between the parts)
Tasks: T-1611, T-2029, T-2030, T-1740, T-0982, T-2061, T-2072, T-2073, T-2074

## Fill characters (`*x`) add nothing to the text
A `*x` fill (repeat) character contributes nothing to the displayed text or to TEXT(): `TEXT(7,"0*""a")` is "7a",
`TEXT(3,"*.0.00")` is "3.00", PI() under `0.00*-` is "3.14", and 1000 under `_(* #,##0_);...` is " 1,000 " (only the `_(`
and `_)` skips add spaces). Evidence: gen-m10 accounting built-ins and `*.0.00`/`0.00*-` text cells (excel-graph,
2026-10-16, T-1611), gen-m5/3e134e52393061b1 A240, probes/text-format-length-limit-o A37. Implemented in both the number and the date
branch of `format_number` (formatter/format.rs, T-1735). Status: VERIFIED (excel-graph) for number sections; a fill in a date or clock code (`TEXT(1,"d*x")`) and in a fraction code (fraction.rs still prints a space) is UNVERIFIED (T-2480).

- Implemented (T-2227; VERIFIED, gen-m10 659cd5776d423ce3 F!J14, 4c6a49fe4a6fa404 F!R4, 5eb95c63c7d6180c F!G13, bfbcd16cee266336 F!W12, excel-graph): a scientific
  code on zero prints every integer placeholder of the mantissa as `0`, `#` included (`##0.0E+0` → "000.0E+0",
  `##0.00E+00` → "000.00E+00", `#E+0` → "0E+0"). A carry out of the mantissa renormalises (`0.0E+00` on 99.5 is
  "1.0E+02"; already correct, pinned by test). "-x1E+15" is the General sign rule (T-2097, T-2850); no gen-m10 scientific cell with an empty text remains. Still open: `#E+0` ties. UNVERIFIED: grouped (`#,##0.0E+0`) and `?` (`??0.0E+0`) codes on zero.
