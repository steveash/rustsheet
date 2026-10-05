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
Engine still differs on `EOMONTH(31,1)`, `EDATE(31,1)`, `TEXT(0,...)` and `WEEKNUM(1)` (T-0125).
Still approximate below serial 61: EDATE/EOMONTH month arithmetic, `TEXT(0,...)`, WEEKNUM, and DAY/MONTH/YEAR(0)
(Excel gives 0/1/1900; the engine returns #NUM!) (T-0125).
The same holds for any serial in [0, 1) and for FALSE: `YEAR(0.2)` = 1900, `MONTH(0)` = 1,
`DAY(0.15)` = 0, `DAY(FALSE)` = 0 (gen-m2/ef5c55a81783b7d5 A15, 1b6ceed7489e0800 A44,
04cfd82f54dbc2b2 A21, fa437879253827be A41; Oracle: excel-graph, 2026-10-04 (gen-m2 sidecars, T-0203).)

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
- TIMEVALUE trims surrounding spaces but coercion does not: `0+" 10:11:12.5 "` is #VALUE!
  (timevalue-fractional-seconds2 C11).
- Oddity not implemented: `TIMEVALUE("10:11.5:12")` = 0.424369212962963 (36665.5 s); the engine
  returns #VALUE! (timevalue-component-limits B26, baselined).
Status: VERIFIED. Implemented for TIMEVALUE in `functions::date_and_time::time_text::parse_clock_text`
(T-0910); text-to-number coercion (`cast_number`) uses `time_text::parse_time_serial` (T-0146).
Open: `0+"10:11.5:12"` (timevalue-component-limits C26) has the same unimplemented oddity as B26.
Unverified (no oracle evidence yet, T-0153): hour-only AM/PM coercion (`--"6 PM"` = 0.75 is only an
upstream IronCalc unit test; `"6 PM."`, `"10 P"`, `"0 PM"`, `"1/2/2020 6 PM"`), date-clock text with a
month name (`"Jan 2, 2020 10:00"` stays #VALUE!), VALUE of time text (`VALUE("10:11")` is #VALUE!
here, not probed), direct-argument coercion in AVERAGE/SUM/CHOOSE, and `+10:00` typed into a cell
(stored as text now).

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
(docs/research/oracle-calibration.md).
Evidence: probes/date-text-locale B1–B10.
Oracle: excel-graph, 2026-10-04 (the canary pins the oracle to en-US). Evidence is Excel for
the web only; desktop en-US is not yet confirmed (notably `"26-01-01"`, a possible yy-mm-dd
reading), since web and desktop differ in places (docs/architecture/oracle.md).
Status: VERIFIED. The engine still reads `"26-01-01"` day-first and rejects month-name text in
date arguments (T-0024).
