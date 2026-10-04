# Dates (1900 date system)

## Serial 60 is the fictitious 1900-02-29
Rule: Excel treats 1900 as a leap year for compatibility with Lotus 1-2-3.
`DATE(1900,2,29)` = 60 and `DATE(1900,3,1)` = 61.
Evidence: probes/dates-and-blanks.case.json A1 (60), A2 (`DATE(1900,3,1)-DATE(1900,2,28)` = 2),
A3 (`DAY(60)` = 29). Oracle: excel-graph, 2026-10-04.
Status: VERIFIED. Implemented for DATE, DAY/MONTH/YEAR, text date parsing and number formatting
(T-0108): `DATE(1900,1,1)` = 1, days wrap linearly from the first of the month so
`DATE(1900,2,30)` = 61. Excel is believed to number WEEKDAY below serial 60 as if serial 1 were a Sunday (UNVERIFIED, needs
a probe; the Excel-authored calendar templates in ironcalc-calc depend on it), so
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
exact 16:35/24 = 0.69097222222222221. Time-only text is exact.
Evidence: ironcalc-calc DATE_AND_TIME/TIMEVALUE.xlsx B18 (Excel-cached).
Fractional seconds (`"10:11:12.5"`) remain UNVERIFIED (T-0910 stays open for that).
Status: VERIFIED (Excel-cached value, one cell)

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
