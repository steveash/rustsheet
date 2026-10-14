# Dates (1900 date system)

## Serial 60 is the fictitious 1900-02-29
Rule: Excel treats 1900 as a leap year for compatibility with Lotus 1-2-3.
`DATE(1900,2,29)` = 60 and `DATE(1900,3,1)` = 61.
Evidence: probes/dates-and-blanks.case.json A1 (60), A2 (`DATE(1900,3,1)-DATE(1900,2,28)` = 2),
A3 (`DAY(60)` = 29). Oracle: excel-graph, 2026-10-04.
Status: VERIFIED. Implemented for DATE, DAY/MONTH/YEAR, text date parsing and number formatting
(T-0108): `DATE(1900,1,1)` = 1, days wrap linearly from the first of the month so
`DATE(1900,2,30)` = 61. Excel is believed to number WEEKDAY below serial 60 as if serial 1 were a Sunday (VERIFIED by probes/dates-early-1900 A14/A15; the
Excel-authored calendar templates in ironcalc-calc depend on it), so
`from_excel_date` stays a weekday-faithful stand-in calendar (serial 1 = 1899-12-31) for WEEKDAY,
WORKDAY etc.; exact components come from `formatter::dates::excel_serial_to_ymd`.
Oracle-confirmed (excel-graph, 2026-10-04, probes/dates-early-1900): `DATE(1900,1,1)` = 1,
`DATE(1900,2,30)` = 61, `DATE(1900,3,0)` = 60, `DATE(1900,1,0)` = 0, `YEAR(1)` = 1900,
`MONTH(60)` = 2, `DAY(59)` = 28, `EOMONTH(1,0)` = 31, `EOMONTH(31,1)` = `EDATE(31,1)` = 59,
`TEXT(60,"m/d/yyyy")` = "2/29/1900", `TEXT(0,"m/d/yyyy")` = "1/0/1900", `DATEVALUE("1/1/1900")` = 1,
`WEEKDAY(1)` = 1, `WEEKDAY(60)` = 4, `WEEKNUM(1)` = 1.
All of these now match (T-0125).
NOW and TODAY number early-1900 days the same way (T-1479, through DATE's conversion): under a fixed
clock (SPEC-0008) `TODAY()` is 1 on 1900-01-01, 59 on 1900-02-28 and 61 on 1900-03-01. Status of this
sentence: UNVERIFIED (unprobeable: Excel's clock is the host's), inferred from the DATE rule above.
EDATE/EOMONTH do month arithmetic on real calendar dates: serial 60 counts as 28 February 1900 (`EDATE(60,0)` = `EOMONTH(60,0)` = 59), `EOMONTH(59,0)` = 59, and results below serial 1 are #NUM! (`EOMONTH(1,-1)`, `EDATE(1,-1)`). `TEXT(DATE(1900,1,0),"d")` = "0"; `DAY(0)` = 0, `MONTH(0)` = 1, `YEAR(0)` = 1900.
Evidence: probes/dates-early-1900 A17-A25. Oracle: excel-graph, 2026-10-05.
Early-1900 edges (T-0182, VERIFIED, probes/dates-early-1900-edges, excel-graph 2026-10-06): serial 0 works in
EDATE/EOMONTH as day 0 of January 1900: `EDATE(0,1)` = `EOMONTH(0,0)` = 31, `EOMONTH(0,1)` = 59, `EDATE(0,12)` = 366,
`EDATE(0,0)` = 0, `EOMONTH(0.5,1)` = 59, `EDATE(0.5,1)` = 31 (cells A36-A44); `EOMONTH(0,-1)` is #NUM! and
`EDATE(40000,2000000000)` is #NUM!. `TEXT` with ddd/dddd reads the Sunday-based weekday: `TEXT(0,"dddd")` =
"Saturday", `TEXT(0.5,"ddd h:mm")` = "Sat 12:00", `TEXT(1,"dddd")` = "Sunday", `TEXT(59,"dddd")` = "Tuesday",
`TEXT(60,"dddd")` = "Wednesday", `TEXT(61,"dddd")` = "Thursday", `TEXT(0,"dddd d mmmm yyyy")` =
"Saturday 0 January 1900" (A26-A35). WEEKNUM numbers weeks from a 1 January 1900 that is a Sunday (serial 1):
`WEEKNUM(0)` = 0, `WEEKNUM(0,2)` = 1, `WEEKNUM(1,2)` = 1, `WEEKNUM(2,2)` = 2, `WEEKNUM(7,2)` = 2, `WEEKNUM(8,2)` = 2,
`WEEKNUM(9,2)` = 3, `WEEKNUM(59,2)` = `WEEKNUM(60,2)` = `WEEKNUM(61,2)` = 10, `WEEKNUM(1,21)` = ISOWEEKNUM(1) = 52,
`WEEKNUM(60)` = 9, `WEEKNUM(7)` = 1, `WEEKNUM(8)` = 2 (A1-A25). Past serial 60 the Sunday-based weekday equals the real one (`TEXT(61,"dddd")` = "Thursday"), so what differs is where
WEEKNUM puts the week-1 anchor: serial 1 counts as a Sunday. For return type 2 that makes serial 61 week 10 where a real-calendar
anchor gives 9 (A11); type 21 agrees with the real calendar at serial 61 (`WEEKNUM(61,21)` = 9, A23). The anchor holds through 1900: probes/weeknum-1900-later has `WEEKNUM(61)` = 9,
`WEEKNUM(100,2)` = 16, `WEEKNUM(100)` = 15, `WEEKNUM(200,2)` = 30, `WEEKNUM(366,2)` = 54, `WEEKNUM(366)` = 53, `WEEKNUM(0,12)` = 1, `WEEKNUM(0,17)` = 0,
and 1901 is back on the real calendar (`WEEKNUM(367,2)` = 1). Rule: week = floor((serial - first)/7) + 1 with first the last serial <= 1 on the
start weekday (serial 1 = Sunday), for serials 0-366 and return types 1, 2, 11-17 (VERIFIED;
type 21, ISOWEEKNUM and invalid types: "WEEKNUM return_type edge cases" below).
Negative serials under a date/time format (T-1035, VERIFIED, probes/text-ddd-negative-fraction, excel-graph 2026-10-07): `TEXT(-0.5,"dddd")`, `TEXT(-0.5,"d")`, `TEXT(-0.0001,"ddd")`, `TEXT(-0.0001,"d")`, `TEXT(-0.5,"yyyy-mm-dd")`, `TEXT(-0.5,"h:mm")` and `TEXT(-1,"dddd")` are all #VALUE!: a negative value never truncates toward serial 0 (A1-A9). Elapsed-time sections are NOT exempt in the 1900 date system (T-1079, VERIFIED, probes/text-negative-serial-elapsed-mixed, excel-graph 2026-10-07): `TEXT(-0.5,"[h] dddd")`, `TEXT(-1.5,"[h]:mm yyyy")`, `TEXT(-0.5,"[h]:mm")`, `TEXT(-1.5,"[h] d")`, `TEXT(-0.5,"[mm] mmm")` and `TEXT(-0.5,"[s] yy")` are all #VALUE! (A1-A6); non-negative values with the same formats work (`TEXT(0.5,"[h] dddd")` = "12 Saturday", A8). Those cells used to print a leading "-" because the poi ElapsedFormatTests workbook (A40-A47) is a 1904 workbook, where negatives are valid; the formatter now reads the workbook's date system (T-1105, SPEC-0005; see "Negative serials under an explicit negative section" below for the exact 1900 rule). Implemented in the `ParsePart::Date` arm of formatter/format.rs, which also serves cell number-format rendering (cell display of negatives is unprobed).
Implemented in: `fn_weeknum` (T-0995); TEXT ddd/dddd via `excel_weekday_index` (formatter/format.rs, T-0996,
`(serial - 1) mod 7` from Sunday, one formula for every non-negative serial; negatives are #VALUE!, see above). EDATE/EOMONTH of serial 0 via `month_arithmetic_date` (T-0997); EDATE of serial 0 returns one serial before the shifted month's first day (VERIFIED for months 0, 1, 2, 3, 12, 13, 14, 24 and 25, T-1052: `EDATE(0,2)` = 60, the fictitious 1900-02-29, `EDATE(0,3)` = 91, `EDATE(0,13)` = 397, `EDATE(0,14)` = 425, `EDATE(0,24)` = 731, `EDATE(0,25)` = 762, `EOMONTH(0,2)` = 91; negative months are #NUM! (tested: -1 and -12): `EDATE(0,-1)`, `EDATE(0,-12)`, `EOMONTH(0,-1)`; evidence probes/edate-serial-0-months A1-A12, excel-graph 2026-10-07).
The same holds for any serial in [0, 1) and for FALSE: `YEAR(0.2)` = 1900, `MONTH(0)` = 1,
`DAY(0.15)` = 0, `DAY(FALSE)` = 0 (gen-m2/ef5c55a81783b7d5 A15, 1b6ceed7489e0800 A44,
04cfd82f54dbc2b2 A21, fa437879253827be A41; Oracle: excel-graph, 2026-10-04 (gen-m2 sidecars, T-0203).)

### WEEKNUM return_type edge cases and serial 0 (T-1023)
Rule: return_type 1, 2, 11-17 and 21 are valid (a fraction truncates: 2.9 is 2, 21.5 is 21);
0, 3-10 and 18-20 are always `#NUM!` (not `#VALUE!`). A return_type of 22 or more, or negative, is
NOT simply an error: it reuses the start weekday of the last WEEKNUM with a valid type evaluated
earlier in the same recalc (type 21 leaves "start Monday"; ISOWEEKNUM leaves nothing), and is
`#NUM!` only when none has run. On a fresh load Excel evaluates bottom to top, so with
`A1=WEEKNUM(100,99)` and `A2=WEEKNUM(100,2)` both are 16, while the swapped layout gives 16 and
`#NUM!`; `WEEKNUM(0,99)` was 0 or 1 in the 94-cell workbook depending on its neighbours. A boolean
return_type is `#VALUE!` (`WEEKNUM(100,TRUE)`, also from a reference), text that reads as a number is
accepted (`WEEKNUM(100,"2")` = 16), other text is `#VALUE!`. An error in the serial wins
(`WEEKNUM(-1,21)` is `#NUM!`, `WEEKNUM(0,NA())` is `#N/A`). Under ISO numbering serial 0 shares
serial 1's week: `WEEKNUM(0,21)` = `ISOWEEKNUM(0)` = 52 = `WEEKNUM(1,21)`; `ISOWEEKNUM(-1)` is `#NUM!`.
Evidence (oracle: excel-graph, 2026-10-06, T-1023): probes/weeknum-serial-0 (57 cells),
probes/weeknum-invalid-type (8 cells, no valid WEEKNUM in the workbook: all `#NUM!`),
probes/weeknum-stale-return-type (13 cells over five layouts: Up, Down, Mixed, Type21, Type1);
the first 94-cell version of weeknum-serial-0 (commit 042ad31) reproduces deterministically.
Ours: RustSheet always gives `#NUM!` for 22+ and negatives, which is Excel's value only when no
valid WEEKNUM ran earlier in the recalc. The stale value depends on Excel's calc order (and so on
its calc chain and edit history), which RustSheet does not reproduce yet;
probes/weeknum-stale-return-type Up!A1, Mixed!A3, Mixed!A4, Type21!A1 and Type1!A1 stay known
failures. A known gap, not a deviation (decision 0052): T-1484.
Status: VERIFIED
Implemented in: `fn_weeknum`, `iso_week` (crates/rustsheet-engine/src/functions/date_and_time/weeknum.rs), `fn_isoweeknum`.

## DATE month and day bounds
Rule: DATE floors its month and day, then a month outside -32767..=32766 is `#NUM!` even when the
date it names is in the calendar, and a day outside the 16-bit range -32768..=32767 counts as 32767
(not `#NUM!`, and the sign does not matter). The month carry is range-checked before the day: a
month that leaves 1900-01-01..9999-12-31 is `#NUM!` even if the day would come back in range.
`DATE(2000,32766,1)` = 1033789 but `DATE(2000,32767,1)`, `DATE(1900,32767,1)`,
`DATE(2024,45031,1)`, `DATE(2000,65537,1)` and `DATE(2000,1E+15,1)` are `#NUM!`;
`DATE(9999,-32767,1)` = 1960745 but `DATE(9999,-32768,1)` is `#NUM!`. `DATE(2000,1,32767)` =
`DATE(2000,1,32768)` = `DATE(2000,1,-32769)` = `DATE(2000,1,1E+15)` = `DATE(2000,1,-1E+15)` =
`DATE(2000,1,4294967297)` = 69292, `DATE(2000,1,-32768)` = 3757, `DATE(2000,1,-32768.5)` = 69292
(floored to -32769 first), `DATE(9999,-32767.5,1)` is `#NUM!` (the month is floored too) while
`DATE(2000,32766.5,1)` = 1033789, `DATE(1900,1,2958466)` = 32767; `DATE(9999,13,-30)` is `#NUM!` (the month
carry leaves the calendar) while `DATE(0,13,-1)` = 365.
Evidence: probes/date-huge-month-day A1-A63 (excel-graph, 2026-10-12, T-1419).
Status: VERIFIED. Implemented in `fn_date` (T-1419); the month carry uses checked chrono month
arithmetic, so a month near `i32::MAX` no longer panics (fuzz crash T-1419).

## DATE adds 1900 to years below 1900
Rule: `DATE(1899,12,31)` is 3799-12-31 (serial 693962), not an error.
Evidence: probes/dates-and-blanks.case.json A4. Oracle: excel-graph, 2026-10-04.
Status: VERIFIED

## NOW() keeps sub-second precision
Rule: `NOW()` returns the serial with a fractional-second part (not truncated to whole seconds).
Evidence: eval/suites/ironcalc-calc DATE_AND_TIME/NOW_TODAY.xlsx cached `=NOW()` = 46247.682900231484
(59002.58 s into the day). The engine truncated to whole seconds before T-0005; fixed in `tz/os_tz.rs`
(the wasm `tz/browser_tz.rs` path still truncates).
Status: VERIFIED (Excel-cached values)
Unprobed: whether Excel reads its clock once per recalc, so that two `=NOW()` cells and
`=NOW()-NOW()` agree exactly, or once per call. RustSheet's system clock reads once per call;
a fixed clock (SPEC-0008) makes the question moot.

## TIMEVALUE of "date time" text keeps serial-subtraction error
Rule: for text carrying a date and a time, Excel builds the full serial and returns its fractional
part, so `TIMEVALUE("7/7/2026 4:35 PM")` = 0.69097222221898846 (46210.6909… minus 46210), not the
exact 16:35/24 = 0.69097222222222221. Time-only text below 24 hours is exact.
Evidence: ironcalc-calc DATE_AND_TIME/TIMEVALUE.xlsx B18 (Excel-cached).
The same holds for fractional seconds and hours past 23 in "date time" text:
`TIMEVALUE("1/2/2020 10:11:12.5")` = 0.42445023148320615, `TIMEVALUE("1/2/2020 25:00")` =
0.04166666666424135 (probes/timevalue-fractional-seconds B16, timevalue-ampm-limits B14).
Status: VERIFIED (Excel-cached value; oracle: excel-graph, 2026-10-05)

## Time text
Rule: en-US Excel reads clock text as `H:M`, `H:M:S[.f]` or `M:S.f` and its serial is
`(total_ms / 1000) / 86400`; TIMEVALUE returns that serial minus its floor, coercion (`0+text`)
returns it whole.
Details, all from probes/timevalue-fractional-seconds, timevalue-fractional-seconds2,
timevalue-component-limits and timevalue-ampm-limits (oracle: excel-graph, 2026-10-05):
- Two components with a fraction are **minutes and seconds**: `"11:12.5"` is 672.5 s, `"10:11 .5"`
  is 611.5 s; without a fraction they are hours and minutes (`"10:11"`, and `"10:11."` with an
  empty fraction).
- Fractional seconds round **half-up on the decimal digits** to milliseconds: `.1234` → .123,
  `.4995` → .500 (a float-based rounding would give .499), `.0005` → .001, `.0004` → 0,
  `.9999` carries into the next second; digits past the fourth are ignored.
- Components are 1–4 digits, at most 9999 (`"9999:59:59"` parses, `"10000:00:00"`,
  `"10:10000:00"`, `"0:0:10000"` are #VALUE!), and a leading zero is allowed only in a 2-digit
  component (`"010:11:12"`, `"10:011:12"`, `"000:00"` are #VALUE!).
- Without AM/PM, minutes and seconds may exceed 59 and hours 23: `"10:61"`, `"10:11:99.5"`,
  `"25:00"`. TIMEVALUE wraps them by subtraction, so it keeps the float error:
  `TIMEVALUE("25:00")` = 0.04166666666666674, `TIMEVALUE("24:00:00.5")` = 5.78703703713046e-06,
  `TIMEVALUE("23:59:59.9995")` = 0 (coercion gives 1).
- An AM/PM suffix needs whitespace before it (`"10:11:12.5PM"` is #VALUE!), may be `AM`, `PM`,
  `A` or `P` in any case (`"a.m."` is #VALUE!), and then the hour must be at most 12 and minutes
  and seconds at most 59 before rounding (`"13:00:00.5 PM"`, `"10:61 PM"`, `"10:11:60.5 PM"` are
  #VALUE!; `"10:11:59.9995 PM"` is 22:12:00). PM adds 12 to hours 0–11 (`"0:11:12.5 PM"` is
  12:11:12.5, `"10:11.5 PM"` is 12:10:11.5); AM maps 12 to 0.
- Spaces are allowed around `:` and `.` (`"10 :11:12"`, `"10:11:12 . 5"`). A comma is not a
  decimal separator (`"10:11:12,5"`), and a second `.` or an exponent is #VALUE!.
- TIMEVALUE and VALUE trim surrounding spaces; coercion rejects a **leading** space but allows
  trailing ones: `0+" 10:11:12.5 "` and `0+" 10:00 "` are #VALUE!, `0+"10:00 "`, `0+"6 PM "`,
  `0+"10:00 AM  "` and `0+"1/2/2020 10:00 "` parse (timevalue-fractional-seconds2 C11;
  value-clock-text C14, C34-C37). Only the space character U+0020 counts, for trimming and before
  an AM/PM suffix: a tab, a no-break space (U+00A0) or an ideographic space (U+3000) is #VALUE! in
  all three (`"10:00\t"`, `"10:00\u00a0"`, `"10:00\u00a0AM"`, `"\u300010:00"`, and
  `VALUE("1/2/2020\u00a0")`; value-clock-text rows 38, 42-45). Unprobed: such characters inside the fraction of `H:M.f:S` text (inferred from this rule) and between
  the date and the clock of date-clock text.
- **VALUE of clock text** is the coercion serial of the trimmed text, unwrapped like coercion
  (`VALUE("25:00")` = 1.0416666666666667, `VALUE("24:00")` = 1, `VALUE("10:11.5")` =
  0.007077546296296296, `VALUE("1/2/2020 6 PM")` = 43832.75, `VALUE("Jan 2, 2020 10:00")` =
  43832.416666666664); whatever coercion rejects VALUE rejects (`"10:00PM"`, `"-10:00"`,
  `"+10:00"`, `"11.5:00"`, `"13 PM"`). Evidence: probes/value-clock-text (excel-graph,
  2026-10-06, T-0162), column B (VALUE) against C (`0+text`) and D (TIMEVALUE), 45 texts.
- **Hour-only AM/PM** (`"6 PM"` = 0.75) takes the hour 0–12 (`"0 PM"` = 0.5, `"0 AM"` = 0,
  `"12 AM"` = 0, `"13 PM"` is #VALUE!), the suffix `AM`, `PM`, `A` or `P` in any case, an optional
  trailing `.` (`"6 P."`, `"6 A."`, `"6 PM."`) and several spaces before it (`"6  PM"`); the same
  in VALUE, coercion and TIMEVALUE, also as the clock of date-clock text (`"1/2/2020 6 P"` =
  43832.75, `"1/2/2020 0 AM"` = 43832) (value-clock-text rows 4-11, 25, 28-33, 39-41).
- **`H:M.f:S` reads the fraction digits twice**, as whole seconds and as the fraction, and
  ignores `S`: `"10:11.5:12"` is 10:11 plus 5.5 s (0.424369212962963), `"10:11.25:12"` plus
  25.25 s, `"10:11.0123:12"` plus 123.012 s, `"10:11.12345:12"` plus 12345.123 s, `"10:11.9999:12"`
  plus 10000 s (half-up ms rounding as above); `S` must still be a valid component (1–4 digits,
  `"10:11.5:99"` and `"10:11.5:1234"` parse, `"10:11.5:012"`, `"10:11.5:12345"`, `"10:11.5:-1"`,
  `"10:11.5:12.5"`, `"10:11.5:12:13"` are #VALUE!), the fraction must have digits (`"10:11.:12"`
  is #VALUE!), and only a two-component head takes it (`"10:11:12.5:13"` is #VALUE!). With AM/PM the
  whole seconds count against the 59 limit (`"10:11.59:12 PM"` parses, `"10:11.60:12 PM"` is
  #VALUE!). Also in date-clock text (`0+"1/2/2020 10:11.5:12"` = 43832.42436921296).
- **At most one component may be out of its usual range** (hours from 24, minutes from 60,
  whole seconds from 60, counted before millisecond rounding), in every clock form: `"10:61"`,
  `"23:60"` (= 1), `"0:24:60"`, `"24:59:59"`, `"0:59:9999.9999"`, `"0:9999:59.9999"`, `"59:60.5"`,
  `"0:11.9999:1"` (11 min + 10000 s) and `"10:9999.5:12"` parse, but `"24:60"`, `"24:0:60"`,
  `"0:60:60"`, `"9998:60:00"`, `"0:9990:599"`, `"60:60.5"` (M:S.f), `"0:9998.60:1"` and
  `"0:9999:9999.9999"` are #VALUE!. Evidence: probes/clock-text-oddities4 to -7 (excel-graph,
  2026-10-06). An `H:M.f:S` fraction may have any number of digits (T-0253): the digits are
  the whole seconds, accumulated as `w * 10 + d` in a double, so 19 or more digits differ from the
  correctly rounded value (`"0:1.1234567890123456789:0"` TIMEVALUE 0.234375, `"0:1.12345678901234567890:0"`
  0.34375); `0+text` gives `1.4288980209762232e+19` for 25 digits and `1.4288980209762231e+34` for 40, and
  TIMEVALUE of a serial that large is 0. Below 2^53 ms the serial is `total_ms / 1000 / 86400` as before;
  above, `((clock_seconds + whole) + ms / 1000) / 86400`. Leading zeros do not count
  (`"0:1.0000000000001:0"` is 61 s). With AM/PM, a whole-seconds value over 59 is #VALUE!
  (`"10:11.1234567890123:12 PM"`), and the one-out-of-range rule still holds
  (`"24:1.1234567890123:0"`, `"0:60.1234567890123:0"` are #VALUE!). Evidence: probes/clock-text-long-fraction, -2, -3.
- **Clock text of more than 255 characters is #VALUE!**, counted on the raw text with its spaces
  (255 parses, 256 does not, in H:M:S.f, H:M with trailing spaces and the long-fraction forms), so
  a 250-digit fraction is the longest. Evidence: probes/clock-text-long-fraction4, clock-text-length-limit.
  Date-clock text (`1/2/2020 10:11:12.5...`) has the same limit on the whole text (T-0266,
  probes/date-clock-text-length-limit, excel-graph): 255 characters parse (`0+text` and VALUE), 256,
  318 and 418 are #VALUE! whether the extra length is fraction digits or spaces between date and
  clock, and for VALUE also when it is leading spaces (VALUE trims them but counts them: 255
  characters parse, 256 and 318 are #VALUE!; `0+text` rejects a leading space at any length). A
  301-digit `H:M.f:S` fraction (316 characters) is #VALUE! while a 238-digit one (253 characters)
  gives 1.1574074074074075e+232. Whole seconds around the 2^53 ms formula switch
  (`1/2/2020 0:0.9007199254730:0` to `...50:0`) match the engine. The engine applies the shared
  `MAX_NUMBER_TEXT_LEN` limit to date-clock text (T-1101, closing T-0273). Plain number text has the same limit (numbers.md "Number text of 1E+308 or more is not a number", T-1097).
- **One trailing `:` is ignored**: `"10:11:"` is 10:11, `"10:11:12.5:"` is 10:11:12.5, `"10:11.5:"`
  is `"10:11.5"` (M:S.f, 611.5 s) and `"10:11.5:12 :"` parses; `"10:11.5::"` is #VALUE!.
  Unprobed: a trailing `:` before an AM/PM suffix.
  Evidence for these bullets: probes/clock-text-oddities, clock-text-oddities2 to -7 (excel-graph, 2026-10-06, T-0153), TIMEVALUE, `0+text` and VALUE agree.
- **Clock text as a direct argument** is read as its coercion serial by SUM, AVERAGE, MAX, MIN,
  PRODUCT, LARGE, SMALL, ABS, INT, MOD, HOUR/MINUTE/SECOND, DAY and array math (`SUM("10:00","6 PM")`
  = 1.1666666666666667, `AVERAGE("10:00 ")` = 0.4166666666666667, `HOUR("25:00")` = 1,
  `SECOND("10:11.5")` = 11, `SUM(-{"10:00";"6 PM"})` = -1.1666666666666667); a leading space is
  #VALUE! like coercion (`SUM(" 10:00")`); `COUNT("10:00","x")` is 1 (implemented in T-0252). Evidence: probes/clock-text-oddities F31:F55 (excel-graph, 2026-10-06).
Status: VERIFIED. Implemented for TIMEVALUE in `functions::date_and_time::time_text::parse_clock_text`
(T-0910); text-to-number coercion (`cast_number`) uses `time_text::parse_time_serial` (T-0146).
Implemented for VALUE in `fn_value` (T-0162). Verified since T-0153 was filed: hour-only AM/PM
coercion, `"1/2/2020 6 PM"` and date-clock text with a month name (`"Jan 2, 2020 10:00"` =
43832.416666666664, value-clock-text row 27; the engine already matched).
`H:M.f:S`, the one-out-of-range limit and the trailing `:` implemented in `parse_clock_text` (T-0153).
Unprobed: `+10:00` typed into a cell (the engine stores it as text). The oracle can now type
cell input through `{"entered": ...}` spec cells (T-0280).

## TIME divides by 86400 first, then drops whole days
Rule: `TIME(h,m,s)` = `((h*3600+m*60+s)/86400) % 1.0` in floating point, not
`(total % 86400)/86400`; they differ in the last bits when the total exceeds a day, e.g.
`TIME(24,2,3)` = 0.0014236111111110006, `TIME(34,1,2)` = 0.41738425925925915.
Evidence: ironcalc-calc DATE_AND_TIME/TIME_HOUR_MINUTE_SECOND.xlsx D3/D14/D17, DATE.xlsx D4 (Excel-cached).
Status: VERIFIED (Excel-cached values)

## Date text is parsed month-first (en-US); month names are accepted
Rule: in en-US Excel, text dates in DATEVALUE/VALUE and in date arguments (DAYS, DAYS360,
NETWORKDAYS, NETWORKDAYS.INTL) are read month-first and long forms with a month name parse:
`VALUE("2/3/2023")` = 44960 (3 Feb), `VALUE("10-12-1999")` = 36445 (12 Oct),
`DATEVALUE("1/2/2001")` = 36893 (2 Jan), `DATEVALUE("26-01-01")` = #VALUE! (no month 26),
`DAYS("July 1, 2026","June 1, 2026")` = 30, `NETWORKDAYS("June 1, 2026","July 1, 2026")` = 23.
Day-first readings belong to other locales: several ironcalc-calc workbooks were saved by a
day-first Excel, so their cached values for these cells are NOT en-US evidence
(docs/research/oracle-calibration.md); `eval/suites/ironcalc-calc/cached-overrides.tsv` scores
them against the en-US probe values instead (decision 0032).
Evidence: probes/date-text-locale B1–B10.
Oracle: excel-graph, 2026-10-04 (the canary pins the oracle to en-US). Evidence is Excel for
the web only; desktop en-US is not yet confirmed (notably `"26-01-01"`, a possible yy-mm-dd
reading), since web and desktop differ in places (docs/architecture/oracle.md).
Status: VERIFIED
Implemented in: `formatter::date_text::parse_date` (T-0024)

## The en-US date-text grammar (DATEVALUE, VALUE, date arguments)
Rule: DATEVALUE and VALUE accept the same date text (probes/date-text-grammar, column B is
DATEVALUE and column C VALUE of the text in column A; 63 forms):
- **Numeric**: `m/d/y` and `m-d-y` (month first, 1–2 digit parts), and year first `y/m/d`,
  `y-m-d` with a 4-digit year. Day-first text is #VALUE! when its "month" exceeds 12
  (`"13/1/2026"`, `"31/12/99"`, `"15-01-2023"`, `"29/02/2020"`). Dots are not separators
  (`"1.7.2026"`, `"7.1.2026"`, ironcalc `DATEVALUE("2026.01.01")` are #VALUE!).
- **Month names**, case-insensitive: the short name or any prefix of at least 3 letters of the
  full name (`"Sept 1, 2026"`, `"Septem 1, 2026"` are 1 Sep, `"Augu 1, 2026"` 1 Aug; `"Ju 1, 2026"`
  is #VALUE!). Accepted shapes:
  `"July 1, 2026"` (comma right after the day, then a space; `"July 1 2026"`, `"July 1,2026"`,
  `"July, 1 2026"` are #VALUE!), `"1 July 2026"` (no comma; `"1 July, 2026"` is #VALUE!),
  `"1-Jul-2026"`, `"1/Jul/26"`, and month-year `"Jul-2026"`, `"July 2026"` = the 1st. A month
  name never comes first in a separated form or with a year first (`"Jul-1-2026"`, `"Jul/1/2026"`,
  `"2023/Jan/15"`, `"2023-Jan-15"` are #VALUE!), and `"Jul. 1, 2026"`, `"July 1st, 2026"`,
  `"Wednesday, July 1, 2026"`, `"2026 July 1"` are #VALUE!.
- **Years**: 2 or 4 digits; 2-digit years below 30 are 20xx (`"Jul 1, 26"` = 2026,
  `"Jul 1, 29"` = 2029, `"Jul 1, 30"` = 1930; also `"1 July 26"`, `"1/Jul/26"`). The date must exist and lie in 1900-01-01..9999-12-31
  (`"Feb 29, 2025"`, `"July 32, 2026"`, `"Jan 1, 1899"`, `"Jan 1, 10000"` are #VALUE!;
  `"Feb 29, 1900"` = 60).
- **Spaces**: VALUE and DATEVALUE trim surrounding spaces and treat a run of inner spaces as one
  (`"  July 1, 2026  "`, `"July  1, 2026"`); number coercion does not trim
  (`MROUND(" 2024-01-10 ",7)` is #VALUE!; ironcalc-calc MROUND_TRUNC_INT.xlsx MROUND!C32).
  Spaces around a `-` or `/` separator are ignored by VALUE and DATEVALUE (`"2026 - 01 - 01"` =
  46023, `"1 / 2 / 2026"` = 46024, `"4 - Feb - 2026"` = 46057, `"2026- 01-01"`, `"2026 -01-01"`,
  `"Jul - 2026"`, `"1 / Jul / 2026"`; a clock may follow: `"2026 - 01 - 01 12:00"`; probes/
  datevalue-separator-spaces rows 1-4, 7-10, 13-14, 16-18, excel-graph, 2026-10-08, and
  ironcalc-calc DATEVALUE_issue_1380.xlsx, Excel 16.03). Year-less `"1 - Jul"` and `"7 - 1"` are
  accepted too (rows 11-12, 46204 on the 2026 oracle clock; in the sidecar but unscored). The same
  holds on the other paths that read date text (probes/datevalue-separator-spaces-paths): text
  coercion (`0+"2026 - 01 - 01"` = 46023, B1), a `COUNTIF` criterion (`"2026 - 01 - 01"` matches
  the serial 46023, G1) and typed cell input (`"2026 - 01 - 01"` and `"4 - Feb - 2026"` become
  numbers, 46023 and 46057 (the number format is UNVERIFIED); year-less `"1 - 2"` and
  `"10 / 20"` become dates too, 2 Jan and 20 Oct of the oracle year; B3/B4 are scored, C3/C4 unscored: clock). The
  ISO `T` separator is not accepted: `"2026-01-01T24:00:00"` and `"2026-01-01T12:00:00"` are
  #VALUE! in DATEVALUE and VALUE (rows 5-6), so IronCalc's `T24:00:00` rollover test (upstream
  issue 1379, whose DATEVALUE_issue_1379.xlsx only evidences the space-separated
  `"2026-01-01 24:00:00"` = 46024) is not ported. In month-day-year text the comma may be a word of its own
  after the day or the month (`"July 1 , 2026"`, `"July 1  ,  2026"`, `"July 1 ,  2026"`,
  `"July , 1, 2026"`, `"July , 1 , 2026"`, `"July 1 , 26"` = 46204; probes/datevalue-separator-spaces row 15 and
  probes/datevalue-comma-spaces rows 2-5, 8, 9, 15), but it still needs a space after it: `"July 1 ,2026"`,
  `"Jul 1 ,2026"`, `"1 July , 2026"` and `"1 July, 2026"` are #VALUE!, as are `"July , 1 2026"` and
  `"July 1, , 2026"`. A clock suffix works as for `"July 1, 2026 10:00"` (`"July 1 , 2026 10:00"`: DATEVALUE
  46204, VALUE 46204.41666…); any other fifth word is #VALUE! (`"July 1 , 2026 foo"`; probes/datevalue-comma-spaces rows 11-16; row 12 is `"July , 1, 2026 10:00"`, same values).
- **Clock suffix**: date text may end in a clock time (`"July 1, 2026 10:00"`,
  `"Jul 1, 2026 4:35 PM"`, `"July 1, 2026 6 PM"`): VALUE adds it, DATEVALUE returns the day the
  sum falls on (`DATEVALUE("1/2/2020 25:00")` = 43833, 3 Jan), and TIMEVALUE takes the time
  (D30, D55: 0.41666666666424135, 0.6909722222189885, with the serial-subtraction error). Anything else after the date is
  #VALUE! (`"2026-01-01 garbage"`, ISO `"2026-07-01T10:00"`).
- **Year-less text** (`mmm-n`, `mmm/n`, `mmm n`, `n-mmm`, `n mmm`, `mmmm n`, and month-first `m/n`,
  `m-n`): a one- or two-digit `n` that is a valid day of that month *in the current year* gives that
  day of the current year: `"Jul-26"`, `"Jul 26"` and `"Jul/26"` are 26 July (46229 in 2026),
  `"7/1"`, `"7-1"`, `"1-Jul"`, `"1 Jul"`, `"July 1"` are 1 July, `"12/31"`, `"Jul-31"` too, and a
  clock time may follow (`"7/1 10:00"`). Otherwise `n` is a two-digit year (below 30 is 20xx) and
  the date is the first of the month: `"Jul-45"` 16619 (1945-07-01), `"Jul-32"` 11871, `"Jul-0"`
  36708 (2000), `"Jul-99"` 36342, `"2/30"` 10990 (1930-02-01). February 29 follows the current year:
  in 2026 `"2/29"` and `"Feb-29"` are 47150 (2029-02-01). `"13/1"` is #VALUE!. VALUE and DATEVALUE
  agree on all of them. Evidence: probes/date-text-yearless (clock-independent cells: MONTH/DAY of
  the current-year forms, full dates of the two-digit-year forms). UNVERIFIED (not scored, clock-
  dependent scratch runs on 2026-10-05/06): the years 2026 and 2029, `"Jul-26"` = 46229 and
  `"2/29"` = 47150. Also (probes/date-text-yearless-2, excel-graph, 2026-10-06, T-0167): a numeric
  month with a four-digit year is the first of that month (`"7/2026"`, `"7-2026"` = 2026-07-01,
  `"12/1999"` = 1999-12-01); `0` is the year 2000 in numeric forms too (`"7/0"`, `"7-0"`, `"1-0"`);
  the two-digit-year reading covers every month-first spelling (`"Jul 45"`, `"Jul/45"`,
  `"July 45"`, `"7-45"`, and `"7/45 10:00"` with a clock) but not the day-first ones (`"45-Jul"`,
  `"45 Jul"` are #VALUE!); `"13/45"`, `"0/45"` and a space between two numbers (`"7 45"`) are
  #VALUE!. And (probes/date-text-yearless-3, excel-graph, 2026-10-06, T-0167): day-first text takes
  `/` too and a leading zero (`"1/Jul"`, `"01-Jul"`, `"1 July"`), month-name text takes `-` after the
  full name (`"July-1"`, `"Jul-01"`), and a clock may follow either order (`"Jul 1 10:00"`,
  `"1-Jul 10:00"`, `"Jul-26 6 PM"`); in day-first order a number that is not a day is #VALUE!, never
  a year (`"45/Jul"`, `"0-Jul"`, `"32 Jul"`). Criteria read year-less text the same way
  (`COUNTIF(dates,"7/1")`, `">=Jul-2"`, `"<2-Jul"`), and so does number coercion (`DAY("7/2"+0)` = 2);
  `MATCH("7/2",dates,0)` is #N/A (MATCH never coerces text).
Evidence: probes/date-text-grammar A1–A68 (B/C, D30/D55). Oracle: excel-graph, 2026-10-05.
UNVERIFIED (not scored): the display format typed month-name text gets (`d-mmm-yy`, `mmm-yy`
for month-year, `m/yy` for numeric month-year such as `"2/30"`), which the engine assigns to typed
input.
Status: VERIFIED. Implemented, year-less text included (T-0160, T-0167): the current year comes
from the model's clock and timezone (`Model::current_year`, the clock NOW and TODAY read), passed to
the parser, which asks for it only when the number can be a day of the month. Typed year-less text
gets the format `d-mmm` (UNVERIFIED). Typed `+`/`-` input is classified as before (the text after
the sign read with no clock), so `-4-2` stays the formula -6 although `4-2` is a date; how Excel
classifies typed sign-prefixed input is unprobed. UNVERIFIED (hand-derived from the rule, not
oracle-observed, since the oracle clock is 2026): in a leap year `"2/29"`, `"Feb-29"`, `"29-Feb"`
and `"Feb 29"` are 29 February; in a non-leap year `"29-Feb"` is #VALUE!.
Implemented in: `formatter::date_text::parse_date`, `Model::parse_datevalue_text`, `Model::parse_number_text`,
`functions::date_and_time::time_text::parse_date_clock_serial` (T-0024), `functions::text::common::fn_value` (clock suffix, T-0159); `remove_separator_spaces` in `parse_date` (spaces around `-` and `/`, T-1136, Ported-From: ironcalc@753da658)

## WORKDAY holidays take the whole range
Rule: in a legacy (non-dynamic-array) formula, WORKDAY's `holidays` range is read whole, with no
implicit intersection: `=WORKDAY(C2,6,D2:D5)` in row 2 skips the holiday in D4.
Evidence: poi/59106.xlsx (Excel-cached) Test!B2 = 42438 (start 42429, holidays 42422, 42423, 42437,
42499; without D4 the result would be 42437) and A2 = "09 Mar 2016". WORKDAY.INTL's `holidays` (4th
argument) and NETWORKDAYS.INTL's are read whole too, even when the formula's row is outside the
range: fixtures/workday-intl-legacy-holidays.xlsx (legacy `<f>` formulas, same inputs in C2 and
D2:D5) gives `=WORKDAY.INTL(C2,6,1,D2:D5)` = 42438 in B2 and B7, with weekend `"0000011"` = 42438
in B3, `=WORKDAY(C2,6,D2:D5)` = 42438 in B5, and `=NETWORKDAYS.INTL(C2,C2+14,1,D2:D5)` = 10 in B6. A
control in the same file, legacy `=D2:D5*1` in E3, gives 42423 (implicit intersection, no spill),
so the oracle kept the formulas legacy (T-1053, Excel for the web, 2026-10-07).
Status: VERIFIED (Excel-cached; Excel oracle for WORKDAY.INTL and NETWORKDAYS.INTL)
Implemented in: `args_signature_with_holidays` in `crates/rustsheet-engine/src/expressions/parser/static_analysis.rs` (WORKDAY, WORKDAY.INTL, NETWORKDAYS, NETWORKDAYS.INTL)
Tasks: T-0983, T-1053

## WEEKDAY of serial 0 and return_type 0
Rule: WEEKDAY of a serial in [0, 1) is 7 (serial 0 is Saturday 0 January 1900, as in `TEXT(0,"dddd")`):
`WEEKDAY(SIN(SECH(2.5)))` (0.1623) = 7. A return_type of -0.0001 (0 or -1 after conversion) is `#NUM!`:
`WEEKDAY(31330,-0.0001)` is the column index of `VLOOKUP(FALSE,C7:D9,WEEKDAY(31330,-0.0001),INDEX(B1:E7,4))&REPLACE(NA(),..)`,
whose lookup value is *not* in the table and whose range_lookup is `#REF!`; the cell is `#NUM!`, and nothing else in
the formula can give `#NUM!` (a value index's error wins over not-found and over range_lookup, lookup.md).
WORKDAY and NETWORKDAYS take serial 0 too (inferred, values unprobed): `XLOOKUP(NETWORKDAYS(<blank>,7872),E8:E11,A1:B5,..)`
is `#VALUE!` (XLOOKUP's size mismatch, so NETWORKDAYS(0,7872) is a number, assuming an error lookup_value would beat
the size-mismatch `#VALUE!`, which is unprobed) and
`VLOOKUP(WORKDAY(28131,-16,<blank>*1),C2:D5,1,XLOOKUP(..))` is `#VALUE!` (so a holiday of 0 is accepted).
Evidence: gen-m6/ec7b2cc2fb9f0354 Calc!A33, gen-m6/5cc04d6f42b23b51 Calc!A27, gen-m6/63012866a9d8b7db Calc!A39,
gen-m6/6b2aa08e3b916da5 Calc!A35 (excel-graph, 2026-10-09, T-1239).
Ours: `WEEKDAY(0.16)`, `NETWORKDAYS(0,7872)` and `WORKDAY(28131,-16,0)` are `#NUM!`; `WEEKDAY(31330,0)` is `#VALUE!`.
More serial-0 cells (gen-m9, excel-graph, 2026-10-12, T-1289): `YEARFRAC(33752,0)` is 92.41111111111 (from
`(0.3+YEARFRAC(33752,Data!$C12))^SUM(-0.0001)` = 0.9995471537332011 with C12 = 0), which is basis 0 counted from
1900-01-00: (92*360 + 4*30 + 28)/360 (gen-m9/a2e51f48717af2a1 Calc!A32; ours `#NUM!`, T-1409).
`NETWORKDAYS.INTL(44900,0.2)` is a number (`AVERAGE(NETWORKDAYS.INTL(44900,0.2)/<#VALUE!>,..)` is `#VALUE!`,
gen-m9/50f10c4062fb2e98 Calc!A12; ours `#NUM!`). `WORKDAY(33393,0,A3:D3)` with holidays "a", 0.3, 0.3, 0.5 is 33393
(`COLUMNS(B8:E10)%+WORKDAY(..)` = 33393.04, gen-m9/288b99797f58f149 Calc!A10): neither the fractional holidays below 1
nor the text cell are an error, at least with 0 days (ours `#VALUE!` for the text, `#NUM!` for 0.3).
Status: VERIFIED for WEEKDAY (excel-graph); UNVERIFIED for the WORKDAY/NETWORKDAYS values (probe pending, T-1254)
Tasks: T-1253, T-1254, T-1409

## Date function argument modes
Rule: DATE, DATEDIF, DATEVALUE, DAYS, DAYS360, HOUR, MINUTE, SECOND, ISOWEEKNUM, TIME, TIMEVALUE and WEEKDAY lift
over a multi-cell range and over an array in every probed argument (`ROWS(HOUR(A1:A3))` = 3,
`INDEX(DAYS(46000,A1:A3),2)` = 969). The Analysis ToolPak-era functions do not take a range: EDATE, EOMONTH,
WEEKNUM, WORKDAY, WORKDAY.INTL, NETWORKDAYS, NETWORKDAYS.INTL and YEARFRAC are `#VALUE!` (the whole result, not
per element) when a date, month count, day count, return type or basis argument is a multi-cell range, yet lift over
an array literal in the same place (`EDATE(A1:A3,1)` is `#VALUE!`, `INDEX(EDATE({45000.25;45031.5;45100.75},1),2)`
= 45061). The `weekend` argument of WORKDAY.INTL and NETWORKDAYS.INTL is the exception: it lifts over a range too
(`INDEX(WORKDAY.INTL(45000,5,B1:B3),2)` = 45007); `holidays` is read whole (see "WORKDAY holidays take the whole
range"). Typed coercion differs the same way: a boolean date is `#VALUE!` for EDATE, WORKDAY and YEARFRAC, but
counts as serial 1 for DAYS (`DAYS(TRUE,1)` = 0), WEEKDAY (1), DATEDIF (45999 for `(TRUE,46000,"d")`) and
ISOWEEKNUM (52). Numeric text ("45000") and date text ("1/15/2024") are dates for the seven probed functions other
than HOUR, DATEDIF included (`DATEDIF("45000",46000,"d")` = 1000); HOUR's cells are all 0 and cannot tell coercion from
a whole-day serial. `DATE(2024,45031.5,1)` is `#NUM!` although that date would fall around year 5776, well inside
the 9999 limit: the month argument is bounded (see "DATE month and day bounds"). A huge *day* is not `#NUM!`:
`DATE(2042,8,1E+15)` = 84845 = `DATE(2042,8,32767)` (gen-m6/5cc04d6f42b23b51 Calc!A4, `DATE(..)%*88` = 74663.6;
excel-graph 2026-10-09), the same 32767 cap (T-1419).
Evidence: probes/date-argument-modes (excel-graph, 2026-10-09, T-1090).
Ours: DATEDIF, DATEVALUE, DAYS, DAYS360, HOUR, MINUTE, SECOND, TIME, TIMEVALUE and WEEKDAY lift (T-1235), as do
DATE and ISOWEEKNUM, and the `weekend` of WORKDAY.INTL and NETWORKDAYS.INTL. Still `#N/IMPL!` (T-1236) for an array literal in
a date argument: EDATE, EOMONTH, WEEKNUM, WORKDAY, WORKDAY.INTL, NETWORKDAYS, NETWORKDAYS.INTL, YEARFRAC.
In a 1904 workbook DATEDIF reads date text as DATEVALUE does (from
1904-01-01; text before it is `#VALUE!`), and only plain number text is cast as a number (T-1235 on top of T-1330). It counts the days on the 1904 calendar since T-1416 (`DATEDIF(0,1000,"ym")` = 8, probes/date-system-1904-readers A11).
Status: VERIFIED
Tasks: T-1090 (generator modes: every date parameter is `value`, holidays `array`), follow-ups below

## DATEDIF and DAYS360 error order with a bad unit or method (inferred)
Rule (inferred from the leftmost-error rule in arrays.md, not probed): a date argument's own error comes first
(`DATEDIF(1/0,5,NA())` and `DAYS360(1/0,5,NA())` are `#DIV/0!`). After that ours keeps each function's pre-lift
order: DATEDIF reports the dates' `#NUM!` (end before start, out of range) before the unit's error
(`DATEDIF(10,5,NA())` is `#NUM!`), while DAYS360 reports the method's error before the dates' `#NUM!`
(`DAYS360(-5,5,NA())` is `#N/A`). Also unprobed: `TIMEVALUE(B3:B3)` of a single cell is that cell's value, and a
single-cell-range or 3D-range `weekend` is `#VALUE!`/first-sheet in ours.
Status: UNVERIFIED (probe pending)
Tasks: T-1430 (error order), T-1428 (3D ranges, unit/method over an array)

## TIMEVALUE of chrono-pattern date-time text in a 1904 workbook (T-1445)
Rule: in a 1904 workbook `TIMEVALUE("7/7/2026 4:35 PM")` and `TIMEVALUE("2024-03-15 16:35")` both give
0.6909722222189885, the fraction of the coerced 1904 serial (`"7/7/2026 4:35 PM"+0` = 44748.69097222222,
`"2024-03-15 16:35"+0` = 43904.69097222222), with the same bits as the 1900 system's 0.69097222221898846
(both serials share a binade, so the probe does not tell which path produced the bits).
Evidence: probes/date-system-1904-timevalue-chrono A1-A4 (excel-graph 2026-10-12); see also the
T-1444 entry on 1904 TIMEVALUE below.
Status: VERIFIED. The engine already matches.

## The 1904 date system: negative serials keep a minus; epoch at 1904-01-01
Rule: in a 1904 workbook (`<workbookPr date1904="1"/>`), serial 0 is 1904-01-01 (`TEXT(0,"yyyy-mm-dd")` =
"1904-01-01", `DATE(1904,1,1)` = 0, `YEAR(0)` = 1904, `DATEVALUE("1904-01-02")` = 1). A negative serial
under any date or time format is formatted as its absolute value with a leading minus, for date tokens
too: `TEXT(-1,"d")` = "-2", `TEXT(-1.5,"yyyy")` = "-1904", `TEXT(-0.5,"h:mm")` = "-12:00",
`TEXT(-1.5,"[h] d")` = "-36 2", `TEXT(-1.5,"[h]:mm")` = "-36:00", `TEXT(-1.5,"[mm]:ss")` = "-2160:00",
`TEXT(-1E-10,"[h]:mm")` = "-0:00". In the 1900 system each of these is `#VALUE!`, an explicit negative section
included (`TEXT(-1.5,"[h]:mm;-[h]:mm")`), and so is `TEXT(B1,"[h]:mm")` for a cell B1 = -1.5 formatted
`[h]:mm`, whose value stays -1.5 (`=B1`). `-0` is 0 in both systems (`TEXT(-0,"[h]:mm")` = "0:00").
Quirk: a format whose only token is one elapsed token (`[h]`, `[hh]`, `[m]`, `[mm]`, `[s]`, `[ss]`; probes/elapsed-tiny-negatives-1900, T-1338) prints negatives in the 1900 system too: `TEXT(-1.5,"[s]")` =
"-129600" and `TEXT(-1E-10,"[s]")` = "-0" (the minus stays on a magnitude that rounds or truncates to 0; see number-formats.md "Elapsed-time tokens") in both systems, while
`[s] yy` is `#VALUE!` (probes/text-negative-serial-elapsed-mixed A6).
Not probeable through excel-graph: cell display (`#####` or text) of a negative under a date format; the oracle
reads cached values, not displayed text.
Evidence: probes/date-system-1900-negatives and probes/date-system-1904-negatives (same 18 questions, excel-graph,
2026-10-11, T-1268).
Ours: both cases pass 18 of 18. The formatter reads the date system since T-1105. Since T-1330 (SPEC-0005 step 4),
TEXT and cell-format date tokens (day, month, year and weekday names), DATE, DATEVALUE (and every reader of
date text through `Model::parse_datevalue_text`) and YEAR/MONTH/DAY shift by the 1462 days of
`DateSystem::to_1900_serial` / `serial_from_1900_serial` (formatter/dates.rs), checked, with no 1904 serial
below 0 (a day before the epoch is #NUM! in DATE and YEAR/MONTH/DAY, probed in the next entry). This also passes 18 cells of poi
DateFormatTests.xlsx and DateFormatNumberTests.xlsx (1904 workbooks). The other date readers were probed by T-1416 (next entry).
Since T-1416 the date functions read 1904 serials the same way:
- WEEKDAY, WEEKNUM, ISOWEEKNUM, EDATE, EOMONTH, DATEDIF, DAYS360, NETWORKDAYS(.INTL) and WORKDAY(.INTL) (with their holidays) use `DateSystem::date_of_serial` or `to_1900_serial`.
- YEARFRAC and the financial functions read every date one day late, through `DateSystem::financial_1900_serial` (T-1436, the entry after next).
- EDATE and EOMONTH return their result through `signed_serial_from_1900_serial`. WORKDAY(.INTL) uses the checked `serial_from_1900_serial`.
- Date text coerced to a number, typed date input and criteria date text read 1904 serials since T-1435 and date-clock text since T-1441.
Status: VERIFIED (web)
Tasks: T-1268, T-1105, T-1330, T-1416, T-1435, T-1436, T-1441

## The 1904 date system: date readers, the epoch's edges and EDATE/EOMONTH before it
Rule: in a 1904 workbook every date reader counts serials from 1904-01-01, not only TEXT, DATE, DATEVALUE and YEAR/MONTH/DAY:
- **Weekdays and weeks.** `WEEKDAY(0)` = 6 (1904-01-01 is a Friday). `WEEKDAY(DATE(2024,3,15))` = 6. `WEEKNUM(0)` = 1. `WEEKNUM(DATE(2024,3,15))` = `ISOWEEKNUM(DATE(2024,3,15))` = 11.
- **Month arithmetic.** `EDATE(DATE(2024,1,31),1)` = 43889 (2024-02-29). `EOMONTH(DATE(2024,1,15),0)` = 43860.
- **Day counts.** `DATEDIF(0,1000,"ym")` = 8. `DAYS360(DATE(2024,1,31),DATE(2024,2,29))` = 29. `YEARFRAC(0,365,1)` = 365/366 (but YEARFRAC reads each date a day late; see the next entry). `WORKDAY(DATE(2024,3,15),1)` = 43907 (Monday). `NETWORKDAYS` over March 2024 = 21.
- **Coupon days.** `COUPDAYS(DATE(2024,1,15),DATE(2030,6,30),2,1)` = 182 (the financial functions read each date a day late; see the next entry).
- **Date text and typed input.** `"1904-01-02"+0` = 1. `"2024-03-15"+0` = `--"3/15/2024"` = `VALUE("2024-03-15")` = 43904. Typing `1904-01-02` stores 1 and typing `3/15/2024` stores 43904, both formatted `m/d/yyyy`.
- **Criteria.** A criterion's date text is a 1904 serial: `COUNTIF(..,"2024-03-15")` matches a cell holding `DATE(2024,3,15)`, and `COUNTIF(E1,"<=1/1/1908")` is 0 for E1 = 2000 (1/1/1908 is 1461).
- **TODAY and NOW** agree with DATE: `TODAY()-DATE(YEAR(TODAY()),MONTH(TODAY()),DAY(TODAY()))` = 0.

Before the epoch:
- `DATE(1903,12,31)`, `DATE(0,1,1)` (year 0 is 1900), `YEAR(-1)`, `MONTH(-1)` and `WEEKDAY(-1)` are #NUM!.
- `DATE(5,1,1)` = 366 (1905-01-01).
- Quirk: EDATE and EOMONTH return a day before 1904-01-01 as a negative serial instead of #NUM!: `EDATE(0,-1)` = -31 (1903-12-01) and `EOMONTH(0,-1)` = -1 (1903-12-31).
- They do not accept such a serial back: `EDATE(-31,1)` and `EOMONTH(-1,0)` are #NUM!.
- WORKDAY and WORKDAY.INTL are #NUM! for a result before the epoch: `WORKDAY(0,-1)`, `WORKDAY.INTL(1,-2)`.
- A negative serial is #NUM! in ISOWEEKNUM, WEEKNUM, DAYS360, DATEDIF and NETWORKDAYS too: `ISOWEEKNUM(-1)`, `WEEKNUM(-1)`, `DAYS360(-1,0)`, `DATEDIF(-1,0,"d")`, `NETWORKDAYS(-5,0)`.
- Date text naming a day before 1904-01-01 is no date: `"1903-12-31"+0`, `VALUE("1903-12-31")`, `--"12/31/1903"` and `DATEVALUE("1903-12-31")` are #VALUE! (ERROR.TYPE 3), while `"1904-01-01"+0` is 0. Typing `1903-12-31` or `12/31/1903` stays text. Over `DATE(2024,3,15)`, `DATE(1904,1,1)` and `DATE(1904,1,5)`, `COUNTIF(..,"<1903-12-31")`, `">1903-12-31"` and `"1903-12-31"` all count 0 (the criterion is plain text, matching no number) and `"<=1904-01-01"` counts 1 (probes/date-system-1904-pre-epoch-text, excel-graph 2026-10-12, T-1435).

The upper bound matches the 1900 system's 9999-12-31:
- `TEXT(2957003,"yyyy-mm-dd")` = "9999-12-31", `YEAR(2957003)` = 9999 and `DATEVALUE("12/31/9999")` = 2957003.
- 2957004 is `#VALUE!` in TEXT and #NUM! in YEAR.
- This confirms the hand-derived bounds in formatter/test/test_dates.rs.

Evidence: probes/date-system-1904-readers A1-A52 (excel-graph, 2026-10-12, T-1416).
Ours: every cell matches. The text coercion, typed input and criteria cells A20-A25, A38, A39 and A51 read the 1904 serial since T-1435, and COUPDAYS A18 since T-1436. Date-clock text counts from 1904-01-01 too (T-1441, probes/date-system-1904-clock-text, excel-graph 2026-10-12): `"1904-01-02 10:00"+0` = 1.4166666666666667, `"2024-03-15 18:00"+0` = `"3/15/2024 6:00 PM"+0` = 43904.75, `VALUE("2024-03-15 12:00")` = 43904.5, `DATEVALUE` of it = 43904 and `TIMEVALUE` = 0.5. A clock on a day before the epoch is #VALUE! (`"1903-12-31 10:00"+0`, ERROR.TYPE 3), like plain date text. A negative holiday serial is #NUM! (ERROR.TYPE 6) in WORKDAY, WORKDAY.INTL and NETWORKDAYS: `WORKDAY(DATE(2024,3,15),1,-1)`, `WORKDAY(..,1,-5)`, `NETWORKDAYS(..,..,-1)` and `WORKDAY.INTL(..,1,1,-1)` (A8-A11). Ours already matched. TIMEVALUE of date-clock text reads the same shifted date: `TIMEVALUE("1904-01-02 10:11:12.5")` = 0.42445023148148153 (the fraction of the 1904 sum, not of the 1900 sum), `TIMEVALUE("1/2/2020 25:00")` = `TIMEVALUE("3/15/2024 25:00")` = 0.04166666666424135 (same bits as in 1900), and a pre-epoch day is #VALUE! with a clock past 23 or not (`TIMEVALUE("1903-12-31 10:00")`, `"1903-12-31 25:00"+0`; probes/date-system-1904-timevalue A1-A7, excel-graph 2026-10-12, T-1444).

Status: VERIFIED (web)
Tasks: T-1416, T-1435, T-1436, T-1441, T-1444

## The 1904 date system: financial day counts read a 1904 serial as the 1900 serial + 2·1462
Rule: in a 1904 workbook, YEARFRAC and the financial functions with date arguments read serial s as the 1900
serial s + 2·1462 (T-1489, T-1492), while every other reader adds 1462. Within a four-year span with one 29 February
that is the date a day later than the 1900 reading s + 1462, so the day counts look "a day late"; near the non-leap
century years the two differ (the 18 cells T-1493 leaves, the odd-period notes below). A date they return
(COUPNCD, COUPPCD) comes back 2·1462 serials lower (one day early within a four-year span with one 29 February). The coupon schedule, the end-of-month rule and the 30/360 and
actual/actual counts all run on the shifted dates.
- `YEARFRAC(DATE(2024,1,31),DATE(2024,2,29),0)` = 30/360 (2/1 to 3/1). The same cell in a 1900 workbook is 29/360.
  `YEARFRAC(0,59,0)` = 59/360. `YEARFRAC(DATE(2023,2,28),DATE(2024,2,29),1)` = 1 (3/1 to 3/1).
- `COUPPCD(DATE(2030,1,15),DATE(2030,8,30),4)` = 2029-11-29, not 2029-11-30. The maturity reads as 8/31, a
  month end, so the coupons fall on month ends and come back a day early. `COUPNCD` of the same is 2030-02-27.
  `COUPNUM(DATE(2030,2,27),DATE(2030,8,30),4)` = 2, not 3.
- DISC, INTRATE, PRICEDISC, YIELDDISC, RECEIVED, ACCRINTM, PRICEMAT and YIELDMAT (through YEARFRAC), ACCRINT,
  AMORLINC and AMORDEGRC change the same way. `DISC(DATE(2024,1,31),DATE(2024,2,29),97,100,0)` = 0.36 (30 days).
  `AMORLINC(2400,DATE(2023,12,31),DATE(2024,12,31),300,0,0.15,1)` = 360 (1900: 360.986).
- PRICE, YIELD, DURATION, MDURATION, ODDFPRICE, ODDFYIELD, ODDLPRICE and ODDLYIELD change too.
  `ODDFPRICE(DATE(2024,1,31),DATE(2030,2,28),DATE(2023,12,1),DATE(2024,2,29),0.05,0.06,100,2,0)` = 94.9666 (1900: 94.9546).
- TBILLPRICE, TBILLYIELD and TBILLEQ apply their one-year limit to the shifted dates.
  `TBILLPRICE(DATE(2023,2,28),DATE(2024,2,29),0.09)` = 90.85 (3/1 to 3/1 is one year; the 1900 twin of this cell is unprobed).
- Unaffected: DAYS360 (`DAYS360(DATE(2023,8,31),DATE(2024,2,29))` = 179 in both systems) and the other date
  readers of the previous entry.

At the edges:
- Serial 0 is a valid date: `PRICE(0,DATE(1905,6,30),...)`, `ACCRINT(0,...)` and `AMORLINC(2400,0,...)` give values.
- A negative serial is #NUM!: `COUPDAYBS(-1,400,2)`, `YEARFRAC(-1,10,1)`.
- A returned date may be negative: `COUPPCD(10,DATE(1905,6,30),2)` = -1 and `COUPPCD(0,400,2)` = -150.
- Quirk: the last date fails once shifted. `YEARFRAC(0,2957003,1)` and `DISC(0,2957003,97,100,0)` are #NUM!, but
  `COUPNCD(DATE(9999,1,1),DATE(9999,12,31),2)` = 2957003, the maturity itself. `COUPPCD(DATE(2024,1,15),2957004,2)` is #NUM!.
- Quirk: the coupon schedule breaks for any maturity serial m with m + 2·1462 past the last date 2958465, that is
  from 9995-12-31 on (9995-12-30 still has one), as if the 1904 offset were added twice. Excel then takes the
  previous coupon as 1900 serial 1 and the next as the maturity read, whatever the settlement:
  `COUPPCD` is -1462 and `COUPNCD` the maturity (`COUPNCD(DATE(2024,1,15),DATE(9999,12,31),2)` = 2957003, in 1900
  2024-06-30). The threshold is the same for annual, semiannual and quarterly coupons. On the actual bases (1-3)
  COUPDAYBS and COUPDAYSNC are serial differences from those two dates, and so is COUPDAYS on basis 1:
  `COUPDAYS(DATE(9999,1,1),DATE(9999,12,31),2,1)` = 2958465, `COUPDAYBS` = 2958101, `COUPDAYSNC` = 364;
  `COUPDAYSNC(DATE(9998,1,10),DATE(9999,12,31),2,3)` = 720. On the 30/360 bases 0 and 4 COUPDAYBS and COUPDAYSNC are
  #NUM!. COUPDAYS on bases 0, 2, 3 and 4 keeps its fixed length (180, 180, 182.5, 180). COUPNUM, PRICE, YIELD,
  DURATION and MDURATION are #NUM!. A maturity past the last date (`1E+300` included) is #NUM! as before.
  The 1900 twin has no such break: every cell gives the normal schedule up to 9999-12-31.
  ACCRINT breaks on its first_interest the same way: from 9995-12-31 on it is #NUM! for any settlement and calc_method,
  on bases 0 and 1 with semiannual coupons (other bases and frequencies UNVERIFIED; ours breaks them all):
  `ACCRINT(DATE(9994,1,1),DATE(9995,12,31),DATE(9995,12,21),0.05,1000,2,0,TRUE)`, while 9995-12-30
  accrues 98.47 as in 1900, and a first_interest of 9995-06-30 settled on 9995-12-31 still gives 75.28.
  The odd-period functions break differently, and not on that threshold. In 1904, ODDFPRICE, ODDFYIELD, ODDLPRICE
  and ODDLYIELD are #NUM! for maturities 9995-12-30, 9995-12-31 and 9999-06-30 (issue and settlement five years
  earlier, or in the last year), where 1900 gives values. ODDF* with issue, settlement and first coupon in the year
  before a 30 June maturity are #NUM! for 9000 and 9900 but give the 1900 values for 9980 through 9995, and ODDL* for
  9000 and 9900 differ from 1900 (`ODDLPRICE(DATE(9000,3,1),DATE(9000,6,30),DATE(9000,2,15),0.05,0.06,100,2,0)` is
  99.6718 against 99.6715). Settled in 2024, every maturity to 9995 gives the 1900 value.
  The break follows the non-leap century years, not the end of the calendar (probes/date-system-1904-odd-years and its
  1900 twin, T-1457): with issue, settlement and first coupon in the year before a 30 June maturity, ODDF* are #NUM!
  in 1904 for maturity years 2096, 2100, 2200, 8100, 8996, 9000, 9100, 9300, 9700, 9800, 9896 and 9900, and give
  the 1900 values for 2104, 2400, 8000, 9004, 9200, 9600 and 9904. In the failing years ODDL* (last interest
  15 February, settlement 1 March, maturity 30 June) differ from 1900 in the last digits: 99.67182 for 2100, 2200,
  8100, 9000 and the other century years, and 99.67128 for 2096, 8996 and 9896, against 99.67155 in 1900.
  Rule (T-1457, fits every probed odd-period cell): the four odd-period functions read a 1904 serial s as the 1900
  serial s + 2·1462, the date 1462 days later, and count days in that calendar. Across four years with one 29 February
  that is the date four years and a day later, so the day counts are those of the one-day-late reading above. Within
  four years before 1 March of a non-leap century year it is the date four years later with no extra day, so dates on
  either side of that 1 March move apart by a day: 2096-02-15 reads as 2100-02-16 but 2096-03-01 as 2100-03-03,
  which gives 99.67128, and 2100-02-15 reads as 2104-02-17 but 2100-03-01 as 2104-03-02, which gives 99.67182.
  The later calendar's 29 February counts too: last interest 2100-02-26, 2100-02-27 and 2100-02-28 read as
  2104-02-28, 2104-02-29 (the last day of February) and 2104-03-01, giving ODDLPRICE 99.70905, 99.70791 and 99.70977
  (probes/date-system-1904-odd-feb29). The ODDF* #NUM! cells are dates moved off the first coupon's schedule
  (financial.md "ODDFPRICE and ODDFYIELD need the first coupon on the maturity's schedule"): 2099-06-30 reads as
  2103-07-02 but 2100-06-30 as 2104-07-01. A date read past 9999-12-31 (1904 serials from 9995-12-31 on, or a
  quasi-coupon date there) is #NUM!. With a first coupon on 30 June of the maturity year and maturity 31 December,
  ODDFPRICE and ODDLPRICE differ from 1900 in every probed year, 2104 included (98.8602 against 95.9686 for ODDFPRICE,
  99.1983 against 99.1825 for ODDLPRICE); in the failing years the values follow the same rule (98.8631 and 99.1990
  for 2100, 98.8572 and 99.1977 for 2096).
  The other financial readers read s + 2·1462 too, not s + 1463 (T-1489, VERIFIED):
  COUPPCD, COUPNCD, COUPDAYBS, COUPDAYS, COUPDAYSNC, COUPNUM, PRICE, YIELD, DURATION, YEARFRAC and ACCRINT on bases 0, 1
  and 4, for settlements 2096-03-01, 2100-02-15, 2100-03-01, 2099-12-31 and 2096-02-15 and maturities 2100-06-30,
  2104-02-27, 2104-06-30, 2100-12-31 and 2100-02-28 (165 cells), give in a 1904 workbook exactly the values of a 1900
  workbook with every date moved 1462 days later; COUPPCD and COUPNCD are 2·1462 serials lower. All 165 cells agree
  bit for bit (probes/date-system-1900-financial-readers-shifted against probes/date-system-1904-financial-readers).
  The s + 1463 reading differed in 93 of the 165 cells (COUPNCD on basis 1 70308
  against 70309, COUPDAYS 181 against 182, YEARFRAC basis 0 4.32778 against 4.33056) and missed 4 more in the last
  digits. Ours reads s + 2·1462 since T-1492 (`DateSystem::financial_1900_serial`, results mapped back by
  `serial_from_financial_1900_serial`): 147 of the 165 cells pass, the other 18 fail the same way in the shifted 1900 twin
  (probes/date-system-1900-financial-readers-shifted), the 1900 gaps of the next sentence. The unshifted 1900 twin (probes/date-system-1900-financial-readers) has its own gaps: COUPDAYS
  basis 1 around 2096-02-29 (financial.md, "COUPDAYS basis 1 around 29 February"; fixed in T-1493) and YIELD and
  DURATION last digits (financial.md, "YIELD stops short of PRICE's root"; T-1374).
  A date read past 9999-12-31 is #NUM! in YEARFRAC and DISC (1904 serial 2955542, 9995-12-31, onward), but ACCRINT with
  basis 1 and calc_method FALSE takes its *settlement* up to the last 1904 date 2957003:
  `ACCRINT(DATE(9990,1,1),DATE(9990,6,30),DATE(9999,12,31),0.05,1000,2,1,FALSE)` is 504.28 while 2957004 is #NUM!
  (probes/date-system-1904-financial-settlement-bound column A (60 cells in all), excel-graph, 2026-10-13, T-1492;
  `DateSystem::accrint_settlement_limit`). Every basis with calc_method FALSE takes it that far (T-1494): bases 1, 2
  and 3 count the actual days to the settlement read past 9999-12-31 (`ACCRINT(DATE(9990,1,1),DATE(9990,6,30),
  DATE(9996,1,1),0.05,1000,2,2,FALSE)` is 304.31, 2191 days from the issue read 9994-01-02 to 10000-01-02), while
  bases 0 and 4, which need the settlement's date for 30/360, take first_interest's 1904 serial plus 1462 (its 1900-system
  serial, actual days from the 1900 epoch) as the day count of the settlement slice, whatever the settlement and the issue: 410415.83 = 25 * 2954994 / 180 for
  first_interest 9990-06-30 (1904 serial 2953532), for every settlement from 9995-12-31 to 9999-12-31 and issues
  9990-01-01 to 9990-06-29; moving first_interest by -91 days moves the count by -91, and an issue before the
  regular period adds its stub as usual (179/180 for issue 9989-01-01 or 9980-01-01). The slice is over the normal
  period (360/frequency days): 410415.83 for frequencies 1 and 2, 410428.19 for 4 (with an 89/90 stub). With
  calc_method TRUE, every basis is #NUM! for a settlement read past 9999-12-31, and from 9995-12-30 on too, since the
  next coupon is then past 9999-12-31 (financial.md, "ACCRINT with calc_method TRUE: a next coupon past the calendar
  is #NUM!"; it holds in 1900 too). Evidence: probes/date-system-1904-accrint-settlement-bound A1:A22 (excel-graph,
  2026-10-13, T-1494); its 1900 twin date-system-1900-accrint-settlement-bound gives #NUM! for those settlements
  (2958465 is the 1900 input bound). Ours: `fn_accrint` (accrint.rs) passes the day count to `compute_accrint`.
  The break threshold of COUPPCD/COUPNCD/COUPDAYS is unchanged: the previous coupon of a broken schedule is 1900 serial 1462 in the doubled reading. All probes excel-graph, 2026-10-13. In 1900 itself, ODDLPRICE and ODDLYIELD with maturity 9999-12-31 (last interest
  9999-05-15) are #NUM!, and so is ODDFPRICE/ODDFYIELD with first coupon 9998-12-31 and maturity 9999-06-30
  (issue 9998-07-01: a first period of exactly 180 days 30/360, not the calendar; financial.md, T-1490).
  COUPDAYS on basis 1 in the broken range ignores the settlement: 2958465 for settlements 9999-01-01, 9998-03-01 and
  9996-02-01.
  Probes: date-system-1904-odd-accrint-bound, date-system-1904-odd-threshold and their 1900 twins (excel-graph,
  2026-10-12, T-1447). Ours: ACCRINT follows `coupon_schedule_breaks` (T-1447). The odd-period cells are T-1457.
  Probes: date-system-1904-coupon-bound and its twin date-system-1900-coupon-bound (excel-graph, 2026-10-12, T-1440).
  Ours: `DateSystem::coupon_schedule_breaks` (formatter/dates.rs), applied by the COUP* functions, PRICE, YIELD and
  DURATION (functions/financial/financial_bonds.rs).

Probes: date-system-1904-financial, date-system-1904-financial-shift, and the twins date-system-1900-financial-daycount
and date-system-1904-financial-daycount (the same 40 cells in each system). All excel-graph, 2026-10-12, T-1436.
Ours: `DateSystem::financial_1900_serial` / `serial_from_financial_1900_serial` (formatter/dates.rs), applied by
YEARFRAC and the financial date readers. Every shift cell matches, COUPNCD at 9999-12-31 (A43) since T-1440.
The other failing cells are last-digit misses (DISC, INTRATE, PRICEDISC, PRICEMAT, YIELDDISC, YIELDMAT, TBILL*; T-1374)
or iteration stopping points (XIRR, YIELD, ODDFYIELD; T-0916). Most fail in the 1900 twin too; PRICEDISC
daycount A20 is 1 ulp off in 1904 only.
Status: VERIFIED (web)
Tasks: T-1436, T-1440, T-1447, T-1457, T-1489, T-1492, T-1494

## Negative serials under an explicit negative section; lone elapsed tokens
Rule: Excel decides whether a negative serial formats from the value's own sign, before a negative section drops it.
In the 1900 system a date/time section is `#VALUE!` for a negative value even when it is the explicit negative
section and so would format the magnitude: `TEXT(-1,"d;d")`, `TEXT(-1,"d;d;d")`, `TEXT(-1.5,"yyyy;yyyy")`,
`TEXT(-0.5,"h:mm;h:mm")`, `TEXT(-1.5,"[h];d")` and `TEXT(-1.5,"0.0;d")` are all `#VALUE!`. The one exception is a
section whose only date/time token is a single elapsed token, whatever literals, quoted text or color surround it:
`TEXT(-1.5,"[h];[h]")` = "36", `TEXT(-1.5,"d;[h]")` = "36", `TEXT(-1.5,"0;[h]")` = "36" (the negative section prints
the magnitude), and one section prints a leading minus in front of everything: `TEXT(-1.5,"[h]"" h""")` = "-36 h",
`TEXT(-1.5,"[h]\h")` = "-36h", `TEXT(-1.5,"""x""[h]")` = "-x36", `TEXT(-1.5,"[Red][h]")` = "-36",
`TEXT(-1.5,"[mm]")` = "-2160". Fractional seconds are a second token: `TEXT(-1.5,"[s].00")` is `#VALUE!` in 1900 and
"-129600.00" in 1904. A number section is unaffected: `TEXT(-1.5,"[h];0")` = "2". In the 1904 system every negative
formats: a negative section gets the magnitude (`TEXT(-1,"d;d")` = "2", `TEXT(-1.5,"[h];d")` = "2", 1904 epoch) and a
single section a leading minus.
Quirk: a format with two elapsed tokens is `#VALUE!` in both systems, also for a positive value:
`TEXT(1.5,"[h]:[mm]")`, `TEXT(0.5,"[mm]:[ss]")` and `TEXT(-1.5,"[h]:[mm]")` (A9, A21, A22). `TEXT(-1.5,"[ss] AM/PM")`
is `#VALUE!` in both systems too (A23; the positive value is unprobed).
Evidence: probes/negative-date-sections-1900 and probes/negative-date-sections-1904 (same 23 questions, excel-graph,
2026-10-11, T-1105).
Ours: the sign rule is `formatter::dates::negative_serial_allowed`, applied in the `ParsePart::Date` arm of
formatter/format.rs to the original value. Two elapsed tokens and `[ss] AM/PM` are not rejected yet (T-1366);
the 1904 negative sections read the 1904 epoch (T-1330).
Status: VERIFIED (web)
Tasks: T-1105, T-1366, T-1330

## HOUR, MINUTE and SECOND past 9999-12-31
Rule: HOUR and MINUTE of a serial far beyond the last date (2958465, 9999-12-31) are `#NUM!`: `HOUR(123456789.123)` and
`MINUTE(7+123456789.123)` are `#NUM!`, as DAY is.
Evidence: gen-m7/bf889c3917eb54c5 Calc!A1, gen-m7/851eb2099ae3f805 Calc!A39 (excel-graph, 2026-10-11, T-1275).
Ours: only negative serials are rejected (`HOUR(123456789.123)` = 2).
Status: VERIFIED (web) for these values; SECOND and the exact bound (2958466 or above) are UNVERIFIED
Tasks: T-1353

## HOUR, MINUTE and SECOND round to the nearest second
Rule: SECOND of a time of day 22:24:01.729 is 2: `SECOND(DDB(81074,518,21,6,1))`, where DDB is 3024.9333533459067 (a
last-digit difference in DDB cannot move it by 0.27 s). So the seconds are rounded, not truncated. Yet
`SECOND("10:11.5")` is 11 (probes/clock-text-oddities F43), so an exact half does not round up there (tie rule or the
serial's binary noise). The tie rule and whether the carry reaches MINUTE and HOUR (x:59:59.6, 23:59:59.6) are unprobed.
Evidence: gen-m9/55fc8c4ffbd7c7cc Calc!A40 (excel-graph, 2026-10-12, T-1289); probes/clock-text-oddities F43 (excel-graph,
2026-10-06, T-0153).
Ours: the time is rounded to the millisecond and the seconds truncated (`SECOND(0.5+1.7/86400)` = 1).
Status: VERIFIED (web) for these two cells; ties and carries UNVERIFIED
Tasks: T-1408

## YEARFRAC swaps reversed dates for every basis
Rule: YEARFRAC with a start date after the end date swaps them for basis 1 as for basis 0: `YEARFRAC(52874,24708,1)` =
77.11295191295191 = `YEARFRAC(24708,52874,1)`.
Evidence: gen-m7/c263b43217a5898b Calc!A38 (excel-graph, 2026-10-11, T-1275).
Ours: `#NUM!` for basis 1 (basis 0 already swaps); bases 2-4 unchecked.
Status: VERIFIED (web)
Tasks: T-1354

## DATEDIF argument errors before date checks
Rule: an error in DATEDIF's unit argument wins over an invalid start date: `DATEDIF(-1,38627,CHOOSE(3,x))` (the unit is
`#VALUE!`) is `#VALUE!`, not the `#NUM!` of a negative date.
Evidence: gen-m7/f68fafa0ca2eba26 Calc!A1 (`Data!D10^DATEDIF((Data!E$5+(-1)),38627,CHOOSE(3,Data!B12))`, E5 blank) (excel-graph, 2026-10-11, T-1275).
Ours: `#NUM!`.
Status: VERIFIED (web); an invalid unit text with a negative start date is UNVERIFIED
Tasks: T-1355
