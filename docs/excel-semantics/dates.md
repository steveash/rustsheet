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
Negative serials under a date/time format (T-1035, VERIFIED, probes/text-ddd-negative-fraction, excel-graph 2026-10-07): `TEXT(-0.5,"dddd")`, `TEXT(-0.5,"d")`, `TEXT(-0.0001,"ddd")`, `TEXT(-0.0001,"d")`, `TEXT(-0.5,"yyyy-mm-dd")`, `TEXT(-0.5,"h:mm")` and `TEXT(-1,"dddd")` are all #VALUE!: a negative value never truncates toward serial 0 (A1-A9). Elapsed-time sections (`[h]`, `[mm]`, `[ss]`) are exempt and print a leading "-" (number-formats.md "Elapsed-time tokens"). Implemented in the `ParsePart::Date` arm of formatter/format.rs, which also serves cell number-format rendering (cell display of negatives is unprobed).
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
Status: VERIFIED, except the stale state: DEVIATION (decision 0038). RustSheet always gives
`#NUM!` for 22+ and negatives, which is Excel's value only when no valid WEEKNUM ran earlier in
the recalc. The stale value depends on Excel's calc order (and so on its calc chain and edit
history), which RustSheet does not reproduce; probes/weeknum-stale-return-type Up!A1, Mixed!A3,
Mixed!A4, Type21!A1 and Type1!A1 stay known failures.
Implemented in: `fn_weeknum`, `iso_week` (crates/rustsheet-engine/src/functions/date_and_time/weeknum.rs), `fn_isoweeknum`.

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
  numbers, 46023 and 46057 with format `d-mmm-yy` for the latter; year-less `"1 - 2"` and
  `"10 / 20"` become dates too, 2 Jan and 20 Oct of the oracle year, B3/B4, C3/C4 unscored). The
  ISO `T` separator is not accepted: `"2026-01-01T24:00:00"` and `"2026-01-01T12:00:00"` are
  #VALUE! in DATEVALUE and VALUE (rows 5-6), so IronCalc's `T24:00:00` rollover test (upstream
  issue 1379, whose DATEVALUE_issue_1379.xlsx only evidences the space-separated
  `"2026-01-01 24:00:00"` = 46024)   is not ported. Unimplemented: a space before the comma of month-day-year text is accepted
  (`"July 1 , 2026"` = 46204, row 15).
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
