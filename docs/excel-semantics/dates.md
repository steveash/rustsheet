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
EOMONTH reads only the month and returns a real month end: `EOMONTH(60,0)` = `EOMONTH(59,0)` = 59. EDATE reads the displayed day and clamps on the real calendar: serial 60 is 29 February, so `EDATE(60.5,5)` = 211 (29 July 1900) and `EDATE(60.5,10)` = 364 (29 December 1900), while a February 1900 result has 28 days (`EDATE(60,0)` = 59). Results below serial 1 are #NUM! (`EOMONTH(1,-1)`, `EDATE(1,-1)`). Evidence: gen-m6 and probes/dates-early-1900-displayed, see "Serials below 61 in DATEDIF, EDATE, DAYS360, WEEKNUM and YEARFRAC". `TEXT(DATE(1900,1,0),"d")` = "0"; `DAY(0)` = 0, `MONTH(0)` = 1, `YEAR(0)` = 1900.
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
type 21, ISOWEEKNUM and invalid types: "WEEKNUM return_type edge cases" and "WEEKNUM's stale return_type state" below).
Negative serials under a date/time format (T-1035, VERIFIED, probes/text-ddd-negative-fraction, excel-graph 2026-10-07): `TEXT(-0.5,"dddd")`, `TEXT(-0.5,"d")`, `TEXT(-0.0001,"ddd")`, `TEXT(-0.0001,"d")`, `TEXT(-0.5,"yyyy-mm-dd")`, `TEXT(-0.5,"h:mm")` and `TEXT(-1,"dddd")` are all #VALUE!: a negative value never truncates toward serial 0 (A1-A9). Elapsed-time sections are NOT exempt in the 1900 date system (T-1079, VERIFIED, probes/text-negative-serial-elapsed-mixed, excel-graph 2026-10-07): `TEXT(-0.5,"[h] dddd")`, `TEXT(-1.5,"[h]:mm yyyy")`, `TEXT(-0.5,"[h]:mm")`, `TEXT(-1.5,"[h] d")`, `TEXT(-0.5,"[mm] mmm")` and `TEXT(-0.5,"[s] yy")` are all #VALUE! (A1-A6); non-negative values with the same formats work (`TEXT(0.5,"[h] dddd")` = "12 Saturday", A8). Those cells used to print a leading "-" because the poi ElapsedFormatTests workbook (A40-A47) is a 1904 workbook, where negatives are valid; the formatter now reads the workbook's date system (T-1105, SPEC-0005; see "Negative serials under an explicit negative section" below for the exact 1900 rule). Implemented in the `ParsePart::Date` arm of formatter/format.rs, which also serves cell number-format rendering (cell display of negatives is unprobed).
Implemented in: `fn_weeknum` (T-0995); TEXT ddd/dddd via `excel_weekday_index` (formatter/format.rs, T-0996,
`(serial - 1) mod 7` from Sunday, one formula for every non-negative serial; negatives are #VALUE!, see above). EDATE/EOMONTH of serial 0 via `month_arithmetic_date` (T-0997); EDATE of serial 0 returns one serial before the shifted month's first day (VERIFIED for months 0, 1, 2, 3, 12, 13, 14, 24 and 25, T-1052: `EDATE(0,2)` = 60, the fictitious 1900-02-29, `EDATE(0,3)` = 91, `EDATE(0,13)` = 397, `EDATE(0,14)` = 425, `EDATE(0,24)` = 731, `EDATE(0,25)` = 762, `EOMONTH(0,2)` = 91; negative months are #NUM! (tested: -1 and -12): `EDATE(0,-1)`, `EDATE(0,-12)`, `EOMONTH(0,-1)`; evidence probes/edate-serial-0-months A1-A12, excel-graph 2026-10-07).
The same holds for any serial in [0, 1) and for FALSE: `YEAR(0.2)` = 1900, `MONTH(0)` = 1,
`DAY(0.15)` = 0, `DAY(FALSE)` = 0 (gen-m2/ef5c55a81783b7d5 A15, 1b6ceed7489e0800 A44,
04cfd82f54dbc2b2 A21, fa437879253827be A41; Oracle: excel-graph, 2026-10-04 (gen-m2 sidecars, T-0203).)

### WEEKNUM and ISOWEEKNUM at modern year boundaries (T-1308, VERIFIED)
Rule: for modern dates ISOWEEKNUM and `WEEKNUM(d,21)` are the ISO-8601 week, and WEEKNUM types 1, 2 and 11-17 count
from the week containing 1 January (system 1) on the real calendar, with no 1900 anchor quirk. ISOWEEKNUM is 53 on
2020-12-31, 2021-01-01 and 2021-01-03 (all of ISO 2020), 1 on 2021-01-04 and on 2024-12-30, 52 on 2024-12-29, 53 on
2026-12-31, 2027-01-01 and 2027-01-03, on 2015-12-31, 2016-01-03, 2032-12-31 and 2033-01-02, and 52 on 9999-12-31.
A fractional serial truncates (`ISOWEEKNUM(44197.99)` = 53), a serial past 9999-12-31 is `#NUM!`
(`ISOWEEKNUM(2958466)`), and date text is read as a date (`ISOWEEKNUM("2021-01-03")` = 53). The engine already matches
all 67 cells, WEEKNUM types 1, 2 and 11-17 on 2023-01-01, 2022-01-01, 2021-12-31 and 2024-12-31 included.
A year can have 54 weeks: `WEEKNUM(DATE(2024,12,31),12)` is 54 (Tuesday week start, 2024-01-01 a Monday; A59).
Evidence: probes/weeknum-modern-boundaries A1-A67 (excel-graph, 2026-10-17).

### WEEKNUM return_type edge cases and serial 0 (T-1023)
Rule: return_type 1, 2, 11-17 and 21 are valid (a fraction truncates: 2.9 is 2, 21.5 is 21);
0, 3-10 and 18-20 are always `#NUM!` (not `#VALUE!`). A return_type of 22 or more, or negative, is
NOT simply an error: it reads a start-weekday state that other WEEKNUM calls of the same recalc
leave behind, and is `#NUM!` only when the state is empty (see "WEEKNUM's stale return_type state"
below for which calls set and clear it and in what order Excel runs them). A boolean
return_type is `#VALUE!` (`WEEKNUM(100,TRUE)`, also from a reference), text that reads as a number is
accepted (`WEEKNUM(100,"2")` = 16), other text is `#VALUE!`. An error in the serial wins
(`WEEKNUM(-1,21)` is `#NUM!`, `WEEKNUM(0,NA())` is `#N/A`). Under ISO numbering serial 0 shares
serial 1's week: `WEEKNUM(0,21)` = `ISOWEEKNUM(0)` = 52 = `WEEKNUM(1,21)`; `ISOWEEKNUM(-1)` is `#NUM!`.
Evidence (oracle: excel-graph, 2026-10-06, T-1023): probes/weeknum-serial-0 (57 cells),
probes/weeknum-invalid-type (8 cells, no valid WEEKNUM in the workbook: all `#NUM!`),
probes/weeknum-stale-return-type (13 cells over five layouts: Up, Down, Mixed, Type21, Type1);
the first 94-cell version of weeknum-serial-0 (commit 042ad31) reproduces deterministically.
Status: VERIFIED
Implemented in: `fn_weeknum`, `iso_week` (crates/rustsheet-engine/src/functions/date_and_time/weeknum.rs), `fn_isoweeknum`.

### WEEKNUM's stale return_type state (T-1484)
Rule: one start-weekday state, Sunday, Monday or empty, lives for one recalculation. It is
empty when the recalc starts, the WEEKNUM and ISOWEEKNUM calls Excel runs update it in the order
it runs them, and a WEEKNUM with a return_type of 22 or more, or negative, reads it:
- **Reading.** A return_type of 22+ or negative numbers the weeks as return_type 1 does (system 1),
  starting on the state's weekday, and is `#NUM!` when the state is empty. Reading leaves the state
  as it is: six reads in one formula all see it. After type 21 or ISOWEEKNUM the read is system 1
  with a Monday start, not ISO (`WEEKNUM(100,99)` = 16 while `WEEKNUM(100,21)` = 15).
- **Setting Sunday:** a scalar WEEKNUM with return_type 1 (or omitted) or 17.
- **Setting Monday:** a scalar WEEKNUM with return_type 2, 11 or 21, and ISOWEEKNUM (serial 0
  included). Text and fractions convert first: `"2"` and 2.9 set Monday.
- **Emptying:** a scalar WEEKNUM with return_type 12-16 (Tuesday to Saturday starts are never
  kept), one with an always-#NUM! return_type (0, 3-10, 18-20; probed 0, 3, 10, 18, 20), and any WEEKNUM that lifts over an
  array, valid types included (`SUM(WEEKNUM(100,{2,2}))`, `SUM(WEEKNUM({100,101},2))`).
- **A dynamic-array formula empties it** (T-2068, gen-m6, excel-graph 2026-10-21): with a
  dynamic-array formula between a reader and the setter that runs before it in the order, the
  reader is `#NUM!` (OFFSET, TRANSPOSE, TRIMRANGE, CHOOSEROWS or CHOOSECOLS, none of them
  calling WEEKNUM). In the evidence none of them spilled: each is `#SPILL!`, an error or a 1×1
  result (A17 `#SPILL!`, 22ce5678fa646215 A237 `"a*"`). Whether a formula that does spill
  empties it too is unprobed (T-2070); we treat every dynamic anchor alike. Eight readers show it, each with a
  Monday setter after it and a spilling formula in between: gen-m6/9c615d338d990644 Calc!A16
  (A17 `=OFFSET(Data!B11:E12,2,2,3)`), 07171fa3f7493f51 A246, 22ce5678fa646215 A236
  (A237 `=TRIMRANGE(Data!A10:D10)`), a165b0a457367d4f A203, b54448af3fde07ca A243,
  c4edd76f46980c5f A307 and fde83d52a9d1fc63 A131 and A359, all `#NUM!`; the four readers with no
  dynamic-array formula in between read the setter (121a7c17092038d9 A238 = 33, 3e51213f35889d06 A74,
  c96ac1ada46179c4 A219 and A295). Whether it is the spill itself or where Excel runs the anchor
  that empties it is open (T-2070).
- **No effect:** a call whose serial is an error (`WEEKNUM(-1,2)`, `WEEKNUM(-1,0)`,
  `ISOWEEKNUM(-1)`), a `#VALUE!` return_type (`TRUE`, `"x"`), a call IF does not run, and WEEKDAY of
  any return_type.
- **Order.** On a freshly loaded workbook (the probes read the values after the oracle's
  FullRebuild; load, Full and FullRebuild agree, research note) Excel runs the formula cells in the reverse of the order
  they were loaded: the last sheet first, and within a sheet the bottom row first and each row right
  to left (reverse row-major). The state carries from one sheet to the next. A cell whose formula
  reads a formula cell not yet calculated is abandoned at that read, moved to just after that
  precedent in the order, and run again from its start when its new turn comes; the cells between
  run first. The abandoned run leaves the state as it was: `WEEKNUM(100,2)&reader-of-A1` in A3
  sets Monday before its read of A1, yet the plain reader in A2, which runs next, reads nothing
  (probes/weeknum-stale-precedent-abort-keeps). With A1 a Monday marker, A2 a
  reader-then-Sunday cell and A3 a reader of A1 that then sets Sunday, the order is A2, A1, A3: A2
  reads nothing and A3 reads Monday (not A1, A3, A2). `WEEKNUM(100,12)&reader-of-A1` in A2 below a
  Sunday marker in A1 reads nothing: the re-run starts with its own type 12. Moving a sheet does
not change the order: it keeps its load position. `calcChain.xml` is ignored on a full-calc load:
  three different chains for one sheet gave the layout's values (docs/research/t1484-weeknum-calc-order-2026-10-19.md).
- **Recalcs and edits.** The state does not outlive a recalc: a volatile reader recalculated alone
  is `#NUM!` though its marker ran in the previous recalc, and a formula entered or edited in a
  session is recalculated alone, so a reader is `#NUM!` then. A marker edited in a session does not
  recalculate the readers that used it. Formulas entered in a live session also stop sharing the
  state with the formulas loaded from the file, even at a later full recalc (FullRebuild); that
  history is not in the file, so this part is not reproducible from a workbook (same research note).
Evidence (oracle: excel-graph, 2026-10-19, T-1484): probes/weeknum-stale-setters (28 sheets),
probes/weeknum-stale-sheet-order, probes/weeknum-stale-sheet-moved, probes/weeknum-stale-grid,
probes/weeknum-stale-order-row, -column, -column-3..6 (excel-graph, 2026-10-21), -column-40, -2x3 and -3x2, probes/weeknum-stale-dependency,
probes/weeknum-stale-reset-num-type, -reset-isoweeknum and -reset-error-serial,
probes/weeknum-stale-precedent-between, -precedent-restart and -precedent-abort-keeps, probes/weeknum-stale-num-types,
probes/weeknum-stale-down-alone and probes/weeknum-stale-volatile-reader; the grid case gave the same
values twice. This corrects two earlier readings: ISOWEEKNUM does set the state (to Monday), and the
state is not "the last valid WEEKNUM": types 12-16 empty it. probes/weeknum-stale-return-type
Down!A2 is `#NUM!` because Mixed!A1 (type 0) emptied the state just before it, not because the
state resets per sheet.
Ours (T-2068): the state lives in `Evaluation::weeknum_start`, updated by `weeknum_of` and
`fn_isoweeknum` and read by `weeknum_of`. A pass in which a reader ran is followed by one in
Excel's order (crates/rustsheet-engine/src/excel_order.rs), whose values are kept; later
evaluations start in that order until a pass has no reader. Dynamic anchors still run first in
that pass, with their recursion, and empty the state when the order reaches their position.
Known gaps: the WEEKNUM calls inside a spilling formula, or inside a cell it reads, run when the
anchor runs, not at their place in the order (T-2070). Sheets are ordered by their load rank
(`Worksheet::load_rank`), so a moved sheet keeps its load position (T-2069,
probes/weeknum-stale-sheet-moved). Outside a recalc (conditional formats, `evaluate_formula`) the
state lives for one formula.
Status: VERIFIED
Unprobed: the rank of a sheet added or duplicated after load (we give the next free rank), a reader that lifts over an array (`WEEKNUM(100,{99,99})`), an ISOWEEKNUM that
lifts over one (we leave the state as it is), where a spilling formula sits
in the order, WEEKNUM in a defined name, LET or LAMBDA, conditional formatting formulas, and
a return_type in (-1, 0) (truncates to 0, so presumably empties), and whether a reference to an
uncalculated cell in an IF branch that is not taken also moves the cell.

### WEEKNUM readers past the move budget of Excel's order (T-2068)
Rule (Excel): the chain moves a cell that read an uncalculated formula cell to just after it, as
often as it takes; a running total read in reverse needs `n(n-1)/2` moves.
Ours: a pass in Excel's order makes at most `4 × formula cells + 2^24` moves
(`excel_order::move_budget`); past that, the rest of the pass evaluates on demand, as the
default pass does, and WEEKNUM readers can differ from Excel. A reverse chain of about 5,800
cells fits the floor alone; a cycle of formulas ends there too. A workbook without a reader
never runs this pass. The budget counts moves, not work: a cell runs again (up to its next
blocking read) each time the precedent it waits on is calculated, so a layout of range reads
such as `B_k = SUM(B$1:B_{k-1})+1` costs about n^1.3 to n^1.5 (420 ms at 8,000 rows, 1.5 s at
20,000, security review of T-2068). Measured (release, 2026-10-22): a running total plus one reader takes
34 ms at 10k rows and 75 ms at 100k rows, against 1.5 ms and 23 ms without the second pass.
Evidence: directive 5 (CLAUDE.md), SPEC-0017 "Cost".
Status: DEVIATION (decision 0014)
Implemented in: `run_chain`, `move_budget` (crates/rustsheet-engine/src/excel_order.rs).

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
  A bare hour with one trailing `:` is that hour: `VALUE("12:")`, `"12:"+0` and `TIMEVALUE("12:")` are 0.5,
  `VALUE("25:")` is 1.0416666666666667 (TIMEVALUE 0.04166666666666674), `VALUE("0:")` is 0, `VALUE("9999:")` is
  416.625, `VALUE("12: ")` is 0.5, `VALUE("12:30:45:")` is 0.5213541666666667 and `VALUE("2024-01-01 12:")` is
  45292.5; `"10000:"`, `"12::"`, `"1.5:"`, `"-12:"`, `":30"`, `"12:PM"` and `"12: PM"` (a bare hour, `:`, then an
  AM/PM suffix) are #VALUE! (probes/t2352-time-text-trailing-colon, excel-graph, 2026-10-22, T-2352;
  implemented in `parse_clock_text`). Unprobed: `H:M:` before an AM/PM suffix (`"10:30: PM"`; the engine reads it as 10:30 PM).
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

## Date text with a month name glued to digits (no separator)
Rule: date text whose month name touches a number with no space or separator still parses as a
date. Seen in real workbooks (corpus-bench, Excel for the web agreeing with the cached values;
evidence is the corpus-bench sidecars, recalculated by excel-graph 2026-10-17, T-0427):
- `"July1"` (month name then day): `DATE(F2,MONTH(D2&1),E2)` with D2 = `"July"` is a July date
  (corpus-bench/1726 `July-KPIs!J12` = 45483, J13 = 45504), so `MONTH("July1")` = 7.
- `"January0"` .. `"December0"` (month name then `0`, presumably the year 2000 as `"Jul-0"` is):
  `MONTH(E$4&0)` with E4 = `"January"` feeds a SUMPRODUCT month filter and does not error
  (corpus-bench/55392 `Monthly Expenses Summary!E6` = 0, not the `IFERROR` fallback `""`); G6, the
  same formula over the G4 month, is 35000, so a glued month matches the data.
- `"1JUN2021"` (day, abbreviated month, 4-digit year): `DATEVALUE("1"&B1&B2)` with B1 = `"JUN"`,
  B2 = 2021 is 44348 (2021-06-01; corpus-bench/CF_18814 `attendance-Dec!E1`).
Probed (probes/date-text-glued-month, excel-graph 2026-10-17, Sheet1 rows 1-24; columns B-D are
MONTH/DAY/YEAR of DATEVALUE, E and F are VALUE and `0+` minus DATEVALUE, G-H MONTH/DAY of the text, I
the serial): a month name glued to one number reads as with a space between them, but a month glued to a
day that a year follows does not parse; DATEVALUE, VALUE, `0+`, MONTH and DAY agree on every form (E and F
are 0 wherever DATEVALUE parses):
- Month then one number is the year-less "Month d" rule above: a day of that month in the current year
  (`"July1"`, `"July10"`, `"July31"`, `"Jul1"`, `"Sept1"`: rows 1-3, 7, 20), otherwise the 1st of
  that month in that year, one or two digits as a two-digit year (`"July32"` 1932-07-01, `"July45"` 1945-07-01,
  `"July0"` 2000-07-01, `"January0"` 2000-01-01, `"December0"` 2000-12-01: rows 4-6, 22-23) and four
  digits as the year (`"Jul2026"`, `"July2026"`: 2026-07-01, rows 8-9). A number that is a
  day of no 30-day month or of February (`"Jun31"`, `"Feb29"`) is read as a year (probes/date-text-glued-month-2).
- Day then month, with or without a year, glued on either side or both: `"1July"` (current year,
  row 12), `"1Jul2026"`, `"01JUN21"` (2021), `"1June2021"`, `"1jun2021"` (case-insensitive),
  `"1JUN 2021"`, `"1 JUN2021"`, `"1JUN2021"` (rows 13-16, 18-19, 24).
- `#VALUE!`: a month glued to a day followed by a year, whatever joins them (`"July1,2026"`,
  `"July12026"`, `"Jun1 2021"`: rows 10, 11, 17), and a name that is no month (`"Ju1"`, row 21).
Probed (probes/date-text-glued-month-2, excel-graph 2026-10-18, T-1879; rows refer to Sheet1 of that case):
- Shapes the first probe lacked are dates in Excel, not `#VALUE!`: `"Jul1 , 2026"` and `"July1, 2026"` are
  2026-07-01 (rows 6-7: a glued month and day, then a comma that a space follows, then the year; contrast
  `"July1,2026"` and `"Jun1 2021"` above, which stay `#VALUE!`), and so is `"1-Jul2026"` (row 8: day, hyphen,
  glued month and year); `"1July1"` is 2001-07-01 (serial 37073, day 1, month July, two-digit year 01; row 9).
  A day number past the month's end reads as a year, following `"July32"`: `"Jun31"` is 1931-06-01 (serial
  11475) and `"Feb29"` is 2029-02-01 (serial 47150): the 29 is the two-digit year 2029, not the clock year (rows 10-11,
  MONTH 6/2 and DAY 1 scored). Implemented (T-1906): rows 6-9 and 12 parse; `"1July1"` pads the glued single digit to the two-digit year 01.
- Date+time text goes through the same split: `"July1 10:00"`, `"Jul1 10 AM"`, `"1JUN2021 6 PM"`, `"1JUN2021 18:30"`,
  `"July1 10:00:30"` and `"Jul1 , 2026 10:00"` are date-times in VALUE (fractions 0.41666..., 0.75, 0.7708...,
  0.41701...; rows 1-5, 12), and TIMEVALUE of the same text is just the clock fraction. TIMEVALUE of date-only
  text (`"Jul1 , 2026"`, `"1Jul 2026"`, `"Jul2026"`, `"July01"`, `"Jun31"`, `"Feb29"`) is 0, not an error
  (rows 6-8, 10-11, 13-15, column C). Non-glued date-only text gives 0 as well (`"1/2/2020"`, `"July 1, 2026"`, `"2020-01-02"`, `"1-Jan-2020"`, `"7/1"`; probes/date-text-non-glued-timevalue A1-A5, excel-graph, T-1952). In a 1904 workbook TIMEVALUE and number coercion of a date before 1904-01-01 (`"12/31/1903"`, `"1903-12-31"`) are `#VALUE!`, while `"1/1/1904"` is 0 and `"1/2/2020"` is 42370 (probes/date-system-1904-timevalue-non-glued). Implemented in T-1907.
- Criteria read the glued forms as dates: over serials 44347, 44348, 44349 `COUNTIF(">1JUN2021")` = 1,
  `"1JUN2021"` = 1, `">=1JUN2021"` = 2, `"<1JUN2021"` = 1, `">1Jun2021"` = 1, `SUMIF(">1JUN2021")` sums
  44349; `COUNTIF(date,"July1")`, `"Jul1"` and `"July01"` match the 1 July of the clock year (K1-K8).
- Typed input (the oracle's ENTERED log, number formats; raw log in docs/research/glued-month-typed-input-2026-10-18.md): `"July1"` and `"July01"` -> 1 July of the clock year,
  format `d-mmm`; `"Jul2026"` -> 2026-07-01, `mmm-yy`; `"Jun31"` -> 1931-06-01, `mmm-yy`; `"1Jul 2026"`,
  `"Jul1 , 2026"` and `"1JUN2021"` -> d-mmm-yy dates (2026-07-01, 2026-07-01, 2021-06-01); `"July1 10:00"` ->
  46204.41666..., format `m/d/yyyy h:mm`; `"Jun1.5"` stays text (General). The engine matches (T-1907 made typed
  date+time text a number with `m/d/yyyy h:mm`; only a typed `H:MM` clock is converted). Typed non-glued date + clock text is a number with the same format `m/d/yyyy h:mm` (`1/2/2020 10:00` -> 43832.41666..., `2020-01-02 10:00`, `July 1, 2026 10:00`, year-less `7/1 10:00` -> 46204.41666..., the serial inferred from the oracle's 2026 clock and not scored), and so are clocks with seconds (`1/2/2020 10:00:30` -> 43832.41701...) or AM/PM (`1/2/2020 10:00 AM`); a bare typed date `1/2/2020` is 43832 with format `m/d/yyyy`. In a 1904 workbook `1/2/1904 10:00` is 1.41666... and `12/31/1903 10:00` stays text (General) (probes/date-text-non-glued-timevalue rows 11-17, probes/date-system-1904-timevalue-non-glued rows 11-13, excel-graph, T-1952; formats from the ENTERED log in docs/research/non-glued-typed-input-2026-10-19.md). Further typed clock shapes after a date (probes/date-text-typed-clock-shapes, excel-graph, T-1979; formats from docs/research/typed-clock-shapes-2026-10-19.md): `H:MM:SS AM/PM`, lowercase or mixed-case `pm`, several spaces before AM/PM and an hour-only `10 AM`/`10 PM` are numbers with `m/d/yyyy h:mm`; a fractional second (`10:00:30.5`, `10:00.5` read as minutes and seconds) is a number with format `mm:ss.0`; a component out of its usual range (`24:00` -> 43833, `25:00`, `10:60`, `10:00:60`) is a number with format General; `13:00 PM` and a glued `10:00AM` stay text. Implemented in `typed_clock_shape` (formatter/parse_formatted.rs); the value is the date-clock serial of coercion. `0:00 AM`, `12:00 AM` and `12:30 PM` after a date are numbers with `m/d/yyyy h:mm` (0:00 AM is accepted; 12 AM is midnight, 12 PM noon). Text that stays text can still carry a format (T-1986, probes/date-text-rejected-clock-formats N1-N16 and date-text-typed-clock-shapes N14-N15, excel-graph; formats from docs/research/rejected-clock-text-formats-2026-10-19.md): a date followed by a number-like clock (digits and colons only) and a spaced `AM`/`PM` that is rejected (`13:00 PM`, `13:00 AM`, `24:00 PM`, `13 PM`, `25:00 PM`, `13:00:00 PM`, `99:00 PM`, `1300 PM`) is text whose cell format is that of its date part: `m/d/yyyy` for slash and ISO dates, `d-mmm-yy` for `July 1, 2026 13:00 PM`. Everything else stays General: a glued `10:00AM`, `10:00 XM`, `10:00 PM PM`, `1/2/2020 abc` and a clock with no date (`13:00 PM`). Implemented in `rejected_clock_text_format` (formatter/parse_formatted.rs); `1/2/2020 10:00 A` is a number with `m/d/yyyy h:mm` (N7; engine gap T-1994); the format of a bare typed ISO date (`2020-01-02`) is unprobed, and the engine's `yyyy-mm-dd` for it is not Excel evidence. UNVERIFIED (engine extrapolations, not probed): hours or minutes of 99 (`99:00`, `10:99`) as numbers, a fraction followed by AM/PM, `24:00 AM` as text, hour-only `0 AM`/`12 PM`, trailing spaces after the clock (including after a rejected `13:00 PM`), a rejected clock after a percent, currency or grouped number (`$5 13:00 PM`; the engine leaves General), a year-less date before a rejected clock, a malformed clock of digits and colons (`:5 PM`, `1:2:3:4 PM`), a date part that already carries a time (`1/2/2020 10:00 13:00 PM`), several spaces between the date and the clock (the engine gives all of these the date part's format), a date typed into an already date-formatted cell (the engine keeps its format), and fractions of 3-9 digits.
UNVERIFIED:
- shapes the glued-month rules accept or reject beyond the probed rows (T-1906): `"1Jul5"` (any glued single digit after a day reads as year 0d), `"1/Jul2026"` and other separators, `"1-Jul26"`, the format typed `"1-Jul2026"` gets, and the hand-derived `#VALUE!` of `"July1 , 2026 5"`, `"1Jul1 , 2026"` and `"July1,, 2026"`;
- the clock-year results (`"July1"`, `"July01"`, criteria K6-K8) for clocks other than the oracle's 2026;
- locales other than en-US (the split uses the locale's month names).
Status: VERIFIED (web) for probes/date-text-glued-month rows 1-24 and probes/date-text-glued-month-2; engine gaps are tasks (below).
Implemented in: `split_glued_month` in `crates/rustsheet-engine/src/formatter/date_text.rs` (used by `parse_date`)
Tasks: T-1788, T-1789, T-1879

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
Implemented in: the `WholeByCaller` holidays of the WORKDAY, WORKDAY.INTL, NETWORKDAYS and NETWORKDAYS.INTL entries in `crates/rustsheet-engine/src/functions/lift/kinds/general.rs`, read by `Model::holiday_serials` (`functions/date_and_time/holidays.rs`; T-1236)
Tasks: T-0983, T-1053

## WEEKDAY of serial 0 and return_type 0
Rule: WEEKDAY of a serial in [0, 1) is 7 (serial 0 is Saturday 0 January 1900, as in `TEXT(0,"dddd")`):
`WEEKDAY(SIN(SECH(2.5)))` (0.1623) = 7. A return_type of -0.0001 (0 or -1 after conversion) is `#NUM!`:
`WEEKDAY(31330,-0.0001)` is the column index of `VLOOKUP(FALSE,C7:D9,WEEKDAY(31330,-0.0001),INDEX(B1:E7,4))&REPLACE(NA(),..)`,
whose lookup value is *not* in the table and whose range_lookup is `#REF!`; the cell is `#NUM!`, and nothing else in
the formula can give `#NUM!` (a value index's error wins over not-found and over range_lookup, lookup.md).
WORKDAY and NETWORKDAYS take serial 0 too (now VERIFIED, see "WORKDAY and NETWORKDAYS from serial 0" below; first inferred from): `XLOOKUP(NETWORKDAYS(<blank>,7872),E8:E11,A1:B5,..)`
is `#VALUE!` (XLOOKUP's size mismatch, so NETWORKDAYS(0,7872) is a number: an error lookup_value would beat the size
mismatch, lookup.md "XLOOKUP: an error lookup_value beats a size mismatch") and
`VLOOKUP(WORKDAY(28131,-16,<blank>*1),C2:D5,1,XLOOKUP(..))` is `#VALUE!` (so a holiday of 0 is accepted).
Evidence: gen-m6/ec7b2cc2fb9f0354 Calc!A33, gen-m6/5cc04d6f42b23b51 Calc!A27, gen-m6/63012866a9d8b7db Calc!A39,
gen-m6/6b2aa08e3b916da5 Calc!A35 (excel-graph, 2026-10-09, T-1239).
Probed directly (probes/weekday-serial-zero A1:A18, excel-graph, 2026-10-17, T-1253): `WEEKDAY(0)`, `(0.5)`, `(0.9999)` are 7, `WEEKDAY(0.5,2)` 6, `(0.5,3)` 5, `(0,11)` 6, `(0,17)` 7, `WEEKDAY(1)` 1, `WEEKDAY(-0.5)` `#NUM!`; return_type 0, -1, -0.0001, 0.5, 0.9999, 4, 10 and 18 are all `#NUM!`, never `#VALUE!` (0.5 and 0.9999 truncate to 0).
Ours: WEEKDAY implemented (T-1253); WORKDAY and NETWORKDAYS implemented (T-1254).
More serial-0 cells (gen-m9, excel-graph, 2026-10-12, T-1289): `YEARFRAC(33752,0)` is 92.41111111111 (from
`(0.3+YEARFRAC(33752,Data!$C12))^SUM(-0.0001)` = 0.9995471537332011 with C12 = 0), which is basis 0 counted from
1900-01-00: (92*360 + 4*30 + 28)/360 (gen-m9/a2e51f48717af2a1 Calc!A32; every basis, and serial 0 next to the fictitious serial 60, is now probed: see "YEARFRAC accepts serial 0" below).
`NETWORKDAYS.INTL(44900,0.2)` is a number (`AVERAGE(NETWORKDAYS.INTL(44900,0.2)/<#VALUE!>,..)` is `#VALUE!`,
gen-m9/50f10c4062fb2e98 Calc!A12; ours `#NUM!`). `WORKDAY(33393,0,A3:D3)` with holidays "a", 0.3, 0.3, 0.5 is 33393
(`COLUMNS(B8:E10)%+WORKDAY(..)` = 33393.04, gen-m9/288b99797f58f149 Calc!A10): neither the fractional holidays below 1
nor the text cell are an error, at least with 0 days (ours `#VALUE!` for the text, `#NUM!` for 0.3). Since T-1236 the holidays are read and checked once per call, before any element (`Model::workday_call`), so a "0 days skips the holiday check" rule would go there.
Status: VERIFIED for WEEKDAY (excel-graph); UNVERIFIED for the WORKDAY/NETWORKDAYS values (probe pending, T-1254)
Tasks: T-1253, T-1254, T-1409

## Date function argument modes
Rule: DATE, DATEDIF, DATEVALUE, DAYS, DAYS360, HOUR, MINUTE, SECOND, ISOWEEKNUM, TIME, TIMEVALUE and WEEKDAY lift
over a multi-cell range and over an array in every probed argument (`ROWS(HOUR(A1:A3))` = 3,
`INDEX(DAYS(46000,A1:A3),2)` = 969). The Analysis ToolPak-era functions do not take a range: EDATE, EOMONTH,
WEEKNUM, WORKDAY, WORKDAY.INTL, NETWORKDAYS, NETWORKDAYS.INTL and YEARFRAC are `#VALUE!` (the whole result, not
per element) when a date, month count, day count, return type or basis argument is a multi-cell range, yet lift over
an array literal in the same place (`EDATE(A1:A3,1)` is `#VALUE!`, `INDEX(EDATE({45000.25;45031.5;45100.75},1),2)`
= 45061). The refusal holds on the range's own rows and in a later argument (`ROWS(EDATE(45000,B1:B60))`,
`ROWS(YEARFRAC(45000,46000,B1:B60))`), and also in a legacy formula, which puts no `@` on the range. Next to an array
the range makes every element `#VALUE!` (`EDATE({45000;45031},M1:M60)` is 2 rows of `#VALUE!`, and `EDATE(A1:A60,{1;2})` and `EDATE({45000,45031},M1:M60)`
have every element `#VALUE!`, probes/scalar-lift-open-a W24-W30, W70-W73). An error argument beside the range follows the
function's check order: `EDATE(NA(),A1:A60)` is `#N/A`, `EDATE(A1:A60,NA())` is `#VALUE!`, and YEARFRAC checks `basis`, start, end
(`YEARFRAC(1/0,45000,NA())` is `#N/A`; probes/scalar-lift-open-a W1-W8, -d W1-W7). YEARFRAC range-checks the basis
before it reads either date: `YEARFRAC(NA(),45000,5)`, `YEARFRAC(45000,NA(),5)`, `YEARFRAC("x",45000,5)`,
`YEARFRAC(TRUE,45000,5)` and `YEARFRAC(NA(),45000,-1)` are `#NUM!`, while a basis that is not a number wins with
its own error (`YEARFRAC(NA(),45000,"x")` and `YEARFRAC(NA(),45000,TRUE)` are `#VALUE!`); numeric text is a basis
(`YEARFRAC(NA(),45000,"5")` is `#NUM!`, `YEARFRAC(45000,46000,"1")` = 2.737226277372263; probes/yearfrac-basis-order,
excel-graph, 2026-10-17, T-1800). A 3-D range in EDATE is `#VALUE!`
(probes/scalar-lift-open-b W4, W5, W7). See probes/scalar-lift-shapes,
-b and -c (T-1767) and arrays.md "Scalar arguments lift: shapes, element errors, and the Analysis ToolPak range rule".
The `weekend` argument of WORKDAY.INTL and NETWORKDAYS.INTL is the exception: it lifts over a range too
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
Ours: YEARFRAC's kernel (`yearfrac_of`) reads and range-checks the basis, then reads the start and the end date, as Excel does, so probes/scalar-lift-open-d W1-W7, -a W5-W8 and probes/yearfrac-basis-order pass (T-1800). The financial callers of `get_yearfrac` (ACCRINTM, DISC and the rest) pass three numbers, where every failing check is `#NUM!` in either order, so the reorder changes nothing for them.
DATEDIF's unit and DAYS360's method lift too, over an array and over a range on and off its rows (`ROWS(DATEDIF(45000,46000,{"d";"m"}))` and `ROWS(DATEDIF(45000,46000,B1:B2))` are 2, `INDEX(..,2)` is 32 for "m"; `INDEX(DAYS360(45031,45351,C1:C2),1)` is 314), and in a legacy formula the unit's range takes the `@`: legacy `DATEDIF(45000,46000,B1:B2)` is 1000 on B1's row ("d") and 32 on B2's ("m"), so it intersects with the formula's row rather than taking the first cell. A 3-D reference (`Sheet1:Sheet2!A1`) in DAY, HOUR, DATE, WEEKDAY, DATEDIF or ISOWEEKNUM is `#REF!`, as in the other worksheet functions' lifted arguments. A one-cell range is that cell's value, not a 1x1 array, so a scalar error beside it is the result: `ROWS(DAYS(E1:E1,NA()))` is `#N/A`, `ROWS(WEEKDAY(E1:E1,"x"))` and `ROWS(DAYS360(E1:E1,"x"))` are `#VALUE!`, `ROWS(HOUR(E1:E1))` is 1 (N3, N4, N6, N13). Evidence: probes/date-math-lift-driver-edges G1-G10, G24-G26, G30-G32, G40-G41, K1, K5, L2, N3-N13 (excel-graph, 2026-10-18/19, T-1785).
Ours: DAY, MONTH, YEAR, HOUR, MINUTE, SECOND, DATE, TIME, DAYS, DAYS360, DATEDIF, WEEKDAY and ISOWEEKNUM are `Lift` entries of the SPEC-0011 driver in every argument (T-1785); DATEVALUE and TIMEVALUE lift like the text functions (T-1235), as does the `weekend` of WORKDAY.INTL and NETWORKDAYS.INTL. EDATE, EOMONTH, WEEKNUM and YEARFRAC lift
over an array and refuse a multi-cell range in every argument, in dynamic and legacy formulas (T-1770, SPEC-0011
`LiftArrays`), and so do the date and day-count arguments of WORKDAY, WORKDAY.INTL, NETWORKDAYS and NETWORKDAYS.INTL,
whose `weekend` lifts over both and whose `holidays` are read whole once per call (T-1236). Their legacy formulas put
no `@` on a range in a date or day-count argument and are `#VALUE!` on the range's rows and off them, as EDATE's are
(`=WORKDAY(A1:A60,5)`, `=NETWORKDAYS(A1:A60,46000)`, the .INTL forms and `=WORKDAY(45000,B1:B60)` on rows 3 and 70,
FORMULATEXT unchanged; probes/workday-lift-errors E-I, P-T). In a lifted call an error among the holidays (`"y"`, `NA()`)
or an all-weekend mask is each element's, after that element's own argument errors: `WORKDAY({45000;#N/A},1,"y")` is
{`#VALUE!`; `#N/A`}, `WORKDAY({-1;45000},1,"y")` element 1 is `#NUM!`, `NETWORKDAYS({#DIV/0!;45000},1,NA())` element 1
is `#DIV/0!`, `ROWS(WORKDAY({45000;45001},1,"y"))` is 2, and `WORKDAY.INTL(45000,1,{1;"1111111"})` is {45001; `#VALUE!`}
while NETWORKDAYS.INTL counts 0 for `"1111111"` (probes/workday-lift-errors K1:K15, excel-graph, 2026-10-19, T-1236).
In a 1904 workbook DATEDIF reads date text as DATEVALUE does (from
1904-01-01; text before it is `#VALUE!`), and only plain number text is cast as a number (T-1235 on top of T-1330). It counts the days on the 1904 calendar since T-1416 (`DATEDIF(0,1000,"ym")` = 8, probes/date-system-1904-readers A11).
Status: VERIFIED
Implemented in: EDATE, EOMONTH, WEEKNUM, YEARFRAC, WORKDAY(.INTL) and NETWORKDAYS(.INTL): `Function::arg_kinds` (crates/rustsheet-engine/src/functions/lift/kinds/general.rs); the holidays: crates/rustsheet-engine/src/functions/date_and_time/holidays.rs
and `Model::lift_call` (functions/lift/call.rs), with the kernels `edate_of`, `eomonth_of`
(functions/date_and_time/month_shift.rs), `weeknum_of` (date_and_time/weeknum.rs) and `yearfrac_of` (date_and_time/yearfrac.rs).
Tasks: T-1090 (generator modes: every date parameter is `value`, holidays `array`), follow-ups below

## DATEDIF and DAYS360 error order with a bad unit or method
Rule (probes/datedif-days360-error-order, excel-graph, 2026-10-19, T-1430): both functions read every argument's
value first, left to right, and only then check the dates' range. An error or non-date text in a date argument
comes first, start before end (`DATEDIF(1/0,5,NA())` and `DAYS360(1/0,5,NA())` `#DIV/0!`, `DATEDIF("abc",5,"x")`
and `DAYS360("abc",5,NA())` `#VALUE!`, `DATEDIF(NA(),1/0,"d")` and `DAYS360(NA(),1/0)` `#N/A`; A2, A6-A9, A11,
A12, A15, C2, C4-C6, C8-C10, C13, C14). Then the unit's or method's error: `DATEDIF(10,5,NA())` (a reversed pair),
`DATEDIF(-5,5,NA())`, `DATEDIF(5,-1,NA())`, `DATEDIF(3000000,5,NA())`, `DAYS360(-5,5,NA())`, `DAYS360(5,-5,NA())`
and `DAYS360(3000000,5,NA())` are all `#N/A` (A1, A3, A10, A16, C1, C7, C18). DAYS360's method text that is no
logical is `#VALUE!` at that same step, before the dates' `#NUM!` (`DAYS360(-5,5,"x")` `#VALUE!`, C3; `DAYS360(5,10,"x")` too, C19). Only then the
dates' `#NUM!` (reversed pair or out of the calendar), and DATEDIF's unknown unit last: `DATEDIF(10,5,"x")` and
`DATEDIF(-5,5,"x")` are `#NUM!` (A4, A5, A13, A17; C11, C17). Lifted forms keep the order per element
(`INDEX(DATEDIF({10,1},5,NA()),1)` and `INDEX(DATEDIF(46000,45000,{"d";#N/A}),2)` `#N/A`, A18-A20, C15, C16;
probes/date-math-lift-driver-edges G42, G43; a reversed or out-of-calendar element with an unknown unit is `#NUM!`,
`INDEX(DATEDIF(10,5,{"d","x"}),2)` and `INDEX(DATEDIF({-5,1},5,{"x","x"}),1)`, A21, A22). `TIMEVALUE(C1:C1)` and `DATEVALUE(B1:B1)` of a one-cell range are that
cell's value (1 row), and a one-cell or 3-D `weekend` is answered in "An error-valued or 3-D weekend, 3-D date text
and holidays" (probes/date-text-weekend-3d, T-1428). DATEDIF's unit and DAYS360's method over an array are answered
in "Date function argument modes" (they lift).
A well-ordered, in-calendar pair with an unknown unit is `#NUM!` too, whatever makes the unit unknown: text (`"x~*"`, `""`, `" x "`, `"ü"`, `"Hello World"`), a number (`DATEDIF(16934,34631,IFERROR(1/0,AVERAGE(..)))`) or a boolean (gen-m6/3e51213f35889d06 Calc!A322, 4ca39e0876796fa5 A172, c96ac1ada46179c4 A198/A236, 9e6a1b733ee1e1e0 A12, 121a7c17092038d9 A114; all 40 cells are listed in T-2288; excel-graph 2026-10-21). Ours: implemented (T-2288).
Also under other calls in gen-m7 (excel-graph, 2026-10-22): `TOROW(DATEDIF(..,(Data!C8&Data!D$8)),..)` (gen-m7/2d9b48c6363e7a16 A1306), a
FILTER unit array lifting DATEDIF into TOCOL's ignore (8 `#NUM!`, 7ea6ef6ceabacafd A1875) and `DATEDIF(17490,55420,"?b")` in a SCAN step
(9bb61f6ae34c7e20 A2505) are `#NUM!`.
Status: VERIFIED (excel-graph). Before T-1430 the engine checked DATEDIF's reversed pair before the unit's error.
Implemented in: `datedif_value` (crates/rustsheet-engine/src/functions/date_and_time.rs).
Tasks: T-1355, T-1430, T-2288

## An error-valued or 3-D weekend, 3-D date text and holidays
Rule (probes/date-text-weekend-3d, excel-graph, 2026-10-19, T-1428):
- A 3-D reference in DATEVALUE's or TIMEVALUE's text is `#REF!`, one cell or several
  (`DATEVALUE(Sheet1:Sheet2!B1)`, `ROWS(TIMEVALUE(Sheet1:Sheet2!C1:C3))`; G1-G5, H1-H5), as in the other lifted
  arguments; a one-sheet span `Sheet2:Sheet2!B1` is a plain reference (G6, H6).
- WORKDAY.INTL never passes an error-valued `weekend` on: `NA()`, `1/0`, a cell holding `#REF!` or `#DIV/0!` and a 3-D
  reference (read as `#REF!`, one cell or several) are all `#NUM!`, the code of an invalid weekend number (P1-P25,
  J1-J5, M7). It reads the start date, the day count and a single holiday first (an array or multi-cell range of holidays comes after the weekend, and start + days off the calendar before both; "WORKDAY and NETWORKDAYS holidays: blank and number-text cells, and the error order", T-2272): `WORKDAY.INTL(1/0,5,NA())` is `#DIV/0!`,
  `WORKDAY.INTL(45000,"x",NA())` and `WORKDAY.INTL(45000,5,NA(),"y")` `#VALUE!`, and the holidays' error beats an
  invalid weekend too: `WORKDAY.INTL(45000,5,99,NA())` and `WORKDAY.INTL(45000,5,"abc",NA())` are `#N/A` (P31, P33). A day
  count of 0 still skips the weekend (`WORKDAY.INTL(45000,0,Sheet1:Sheet2!W1)` = 45000, M5).
- NETWORKDAYS.INTL reads the `weekend` before anything else, and an error-valued one is `#VALUE!`:
  `NETWORKDAYS.INTL(1/0,45030,NA())`, `NETWORKDAYS.INTL(45000,"x",X1)` and `NETWORKDAYS.INTL(45000,45030,1/0,"y")`
  are all `#VALUE!` (Q1-Q30), and so is the 3-D weekend whatever the dates (K1-K5, N1-N5). An invalid weekend wins
  over the dates with its own code: `NETWORKDAYS.INTL(NA(),45030,99)` is `#NUM!`, `NETWORKDAYS.INTL(NA(),45030,"abc")`
  `#VALUE!` (Q32, Q33).
- Lifted, each element follows the same order: `INDEX(WORKDAY.INTL(45000,5,{1;#N/A}),2)` is `#NUM!` and
  `INDEX(WORKDAY.INTL(1/0,5,{1;#N/A}),2)` `#DIV/0!`, while every `INDEX(NETWORKDAYS.INTL(..,{1;#N/A}),2)` is `#VALUE!`
  (P26-P30, Q26-Q30); `ROWS(WORKDAY.INTL({45000;45001},5,Sheet1:Sheet2!W1))` is 2 (M6, N6).
- A 3-D `holidays` is `#VALUE!`, also as a branch of IF and in WORKDAY and NETWORKDAYS
  (`WORKDAY.INTL(45000,5,1,Sheet1:Sheet2!A1)`, `NETWORKDAYS(45000,45030,IF(TRUE,Sheet1:Sheet2!A1))`; M9-N13). Only the
  branch IF or CHOOSE takes counts (probes/t2238-holidays-if-3d-branch, excel-graph 2026-10-22, T-2238):
  `IF(FALSE,Sheet1:Sheet2!A1,A1)`, `IF(TRUE,A1,Sheet1:Sheet2!A1)`, `IF(A9,Sheet1:Sheet2!A1,A1)` with A9 FALSE and
  `CHOOSE(2,Sheet1:Sheet2!A1,A1)` are A1 in all four functions (D1:G4), while a taken 3-D branch, `CHOOSE(1,..)` included,
  is `#VALUE!` (D5:G7). Lifted, the error is per element: `ROWS(WORKDAY({45000;45001},5,IF(TRUE,Sheet1:Sheet2!A1,A1)))`
  is 2 and its second element `#VALUE!` (H5, I5). `IF(FALSE,Sheet1:Sheet2!A1)` is FALSE as holidays, `#VALUE!` (row 9).
  An array condition beside a 3-D branch: if any element takes it the call is one `#VALUE!`, no array
  (`WORKDAY(45000,5,IF({TRUE;FALSE},Sheet1:Sheet2!A1,A1))`, its ROWS too, ERROR.TYPE 3; NETWORKDAYS alike), and with
  `{FALSE;FALSE}` A1 is read (45008, ROWS 1; NETWORKDAYS 22) (M1-M8). Unprobed: a real `#REF!` value in the
  untaken branch under an array condition (ours `#VALUE!`, as any `#REF!` there is read as the 3-D one) and CHOOSE with
  an array index beside a 3-D arm (`CHOOSE({1;2},Sheet1:Sheet2!A1,A1)`).
  A day count of 0 (or 0.5) never reads the holidays: WORKDAY and WORKDAY.INTL give the start date, a weekend start
  included, with a 3-D, `NA()` or `"x"` holidays (K1-K4, K8-K9, K11-K14), but 1 day is `#VALUE!` (K10); NETWORKDAYS
  and NETWORKDAYS.INTL over one day still read them (`NETWORKDAYS(45000,45000,Sheet1:Sheet2!A1)` is `#VALUE!`, K5-K7).
- A one-cell `weekend` range is that cell's value (`WORKDAY.INTL(45000,5,W1:W1)` = 45007, 1 row; J7-J9, K7-K9).
Ours: `Model::weekend_mask` turns an error-valued weekend into the caller's error (`#NUM!` or `#VALUE!`);
`workday_of` reads the holidays before the weekend and `networkdays_of` reads the weekend first; `holiday_serials`
follows the branch IF picks (`if_pick`) and refuses a 3-D reference there with `#VALUE!`, and an array condition
whose result holds a 3-D branch's `#REF!` too (T-2238;
crates/rustsheet-engine/src/functions/date_and_time/).
Status: VERIFIED
Tasks: T-1428, T-2238

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
- WEEKDAY, WEEKNUM, ISOWEEKNUM, EDATE, EOMONTH, DATEDIF, DAYS360, NETWORKDAYS(.INTL) and WORKDAY(.INTL) (with their holidays) use `DateSystem::date_of_serial_from_zero` or `to_1900_serial`.
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

## The 1904 date system: ACCRINT, COUPNUM and the TBILL functions read the date itself
Rule: in a 1904 workbook ACCRINT, COUPNUM, TBILLPRICE, TBILLYIELD and TBILLEQ read 1904 serial s as the 1900 serial
s + 1462, the date itself, as every non-financial reader does: they give exactly the values of a 1900 workbook with
the same dates. Their cuts sit at the 1904 calendar's own edges: a date before 1904-01-01 or after 9999-12-31 (1904
serial 2957003) is `#NUM!`, and so is a coupon date before 1904-01-01 (financial.md "COUPNUM in a 1904 workbook",
"ACCRINT at the calendar's start"). Every other financial function and YEARFRAC still reads s + 2·1462 (next entry).
- `COUPNUM(DATE(2030,2,27),DATE(2030,8,30),4)` = 3, as in 1900, while `COUPPCD(DATE(2030,1,15),DATE(2030,8,30),4)` is
  still the one-day-early 2029-11-29 (-financial-shift A5, A1).
- `ACCRINT(DATE(9990,1,1),DATE(9990,6,30),DATE(9996,1,1),0.05,1000,2,0,FALSE)` = 300 (-accrint-settlement-bound A1),
  ACCRINT and COUPNUM compute with any first_interest or maturity up to 9999-12-31 (`COUPNUM(DATE(2024,1,15),
  DATE(9999,12,31),2)` = 15952, -coupon-bound A13), and a settlement past 2957003 is `#NUM!` (-financial-settlement-bound
  A10, D10:F10). The coupon schedule break from 9995-12-31 of the next entry does not apply to them; it still applies
  to COUPPCD, COUPNCD, COUPDAYBS, COUPDAYS, COUPDAYSNC, PRICE, YIELD, DURATION and MDURATION.
- `TBILLPRICE(DATE(2023,2,28),DATE(2024,2,29),0.09)` is `#NUM!` (366 days, more than a year; -financial-shift A31:A33),
  `TBILLPRICE(DATE(9996,1,1),DATE(9996,6,30),0.05)` and `TBILLEQ(DATE(9996,1,1),DATE(9996,12,31),0.05)` are numbers
  while YEARFRAC, DISC, ACCRINTM, AMORLINC, COUPDAYBS and PRICE of the same 9996 dates are `#NUM!`
  (fin-1904-calendar-readers A1:A20). A settlement in 9999 is `#NUM!` for the TBILL functions in both systems
  (financial.md "T-bills settled in 9999").
- `ACCRINT(0,DATE(1904,7,1),DATE(1904,3,1),0.06,1000,2,0)` is `#NUM!` (issue on serial 0, as in 1900, though 1904-01-01
  is a date here), while an issue of 1 computes (fin-coupnum-1904-maturity-sweep C2, C3).
History: until at least 2026-10-20 Excel for the web read these five 2·1462 days on too, as the next entry describes
(sidecars produced 2026-10-11 to 2026-10-20). A re-fill on 2026-10-21 (T-2125) changed 111 of the 1,041 cells of the
24 then committed 1904 probe cases, all of them ACCRINT, COUPNUM and TBILL* cells: ACCRINT lost its 2·1462-day
settlement slices (410415.83 → 300) and its near-9999 `#NUM!`s, COUPNUM computed past the broken schedule and turned
`#NUM!` for a previous coupon before 1904-01-01 (`COUPNUM(0,DATE(1910,3,31),4)` 25 → `#NUM!`), and the TBILLs over a
29 February maturity turned `#NUM!`. A second re-fill on 2026-10-27 (T-2337) gave the same 111 values and changed
nothing else. Desktop Excel is unprobed (M14 question, decision 0039).
UNVERIFIED: the argument-error order in a 1904 workbook. Ours range-checks these five functions' dates in the new
reading as they are read, so `TBILLEQ(DATE(9996,1,1),NA(),0.05)` is `#N/A` (it was `#NUM!` in the old reading); no 1904
probe pairs a late date with a later error argument (T-2843).
Evidence: the 27 `*-1904` probe cases re-filled 2026-10-27 (excel-graph): in all of them every ACCRINT, COUPNUM and
TBILL* cell matches our 1900 evaluation of the same formulas except cells that pass a literal serial (0, 200, 2957004),
while COUPPCD, COUPNCD, COUPDAY*, PRICE, YIELD, DURATION, YEARFRAC, the discount securities, the odd-period functions and
AMORLINC keep the shifted values; fin-coupnum-1904-epoch, fin-coupnum-1904-maturity-sweep (2026-10-21);
fin-1904-calendar-readers, fin-1900-tbill-last-day (2026-10-27).
Ours: `DateReading::Calendar` (functions/financial/dates.rs, `DateSystem::calendar_1900_serial` and
`first_coupon_1900_serial` in formatter/dates.rs), taken by `accrint_of`, COUPNUM's `coupon_call` and `tbill_arguments`
(T-2337).
Status: VERIFIED (web)
Tasks: T-2125, T-2337, T-2835

## The 1904 date system: financial day counts read a 1904 serial as the 1900 serial + 2·1462
Rule: in a 1904 workbook, YEARFRAC and the financial functions with date arguments read serial s as the 1900
serial s + 2·1462 (T-1489, T-1492), while every other reader adds 1462. ACCRINT, COUPNUM, TBILLPRICE, TBILLYIELD and
TBILLEQ did so until 2026-10-20 and read the date itself since (previous entry); what this entry says of them is
marked as the old behavior. Within a four-year span with one 29 February
that is the date a day later than the 1900 reading s + 1462, so the day counts look "a day late"; near the non-leap
century years the two differ (the 18 cells T-1493 leaves, the odd-period notes below). A date they return
(COUPNCD, COUPPCD) comes back 2·1462 serials lower (one day early within a four-year span with one 29 February). The coupon schedule, the end-of-month rule and the 30/360 and
actual/actual counts all run on the shifted dates.
- `YEARFRAC(DATE(2024,1,31),DATE(2024,2,29),0)` = 30/360 (2/1 to 3/1). The same cell in a 1900 workbook is 29/360.
  `YEARFRAC(0,59,0)` = 59/360. `YEARFRAC(DATE(2023,2,28),DATE(2024,2,29),1)` = 1 (3/1 to 3/1).
- `COUPPCD(DATE(2030,1,15),DATE(2030,8,30),4)` = 2029-11-29, not 2029-11-30. The maturity reads as 8/31, a
  month end, so the coupons fall on month ends and come back a day early. `COUPNCD` of the same is 2030-02-27.
  (COUPNUM of 2030-02-27 to that maturity was 2 until 2026-10-20 and is 3 now, previous entry.)
- DISC, INTRATE, PRICEDISC, YIELDDISC, RECEIVED, ACCRINTM, PRICEMAT and YIELDMAT (through YEARFRAC),
  AMORLINC and AMORDEGRC change the same way. `DISC(DATE(2024,1,31),DATE(2024,2,29),97,100,0)` = 0.36 (30 days).
  `AMORLINC(2400,DATE(2023,12,31),DATE(2024,12,31),300,0,0.15,1)` = 360 (1900: 360.986).
- PRICE, YIELD, DURATION, MDURATION, ODDFPRICE, ODDFYIELD, ODDLPRICE and ODDLYIELD change too.
  `ODDFPRICE(DATE(2024,1,31),DATE(2030,2,28),DATE(2023,12,1),DATE(2024,2,29),0.05,0.06,100,2,0)` = 94.9666 (1900: 94.9546).
- Old behavior: TBILLPRICE, TBILLYIELD and TBILLEQ applied their one-year limit to the shifted dates until 2026-10-20
  (`TBILLPRICE(DATE(2023,2,28),DATE(2024,2,29),0.09)` was 90.85; `#NUM!` now, previous entry).
- Unaffected: DAYS360 (`DAYS360(DATE(2023,8,31),DATE(2024,2,29))` = 179 in both systems) and the other date
  readers of the previous entry.

At the edges:
- Serial 0 is a valid date: `PRICE(0,DATE(1905,6,30),...)` and `AMORLINC(2400,0,...)` give values (`ACCRINT(0,...)` did
  until 2026-10-20 and is `#NUM!` now, previous entry).
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
  #NUM!. COUPDAYS on bases 0, 2, 3 and 4 keeps its fixed length (180, 180, 182.5, 180). PRICE, YIELD,
  DURATION and MDURATION are #NUM! (COUPNUM was until 2026-10-20; it now counts the coupons, previous entry). A maturity past the last date (`1E+300` included) is #NUM! as before.
  The 1900 twin has no such break: every cell gives the normal schedule up to 9999-12-31.
  Old behavior: until 2026-10-20 ACCRINT broke on its first_interest the same way, `#NUM!` from 9995-12-31 on for any
  settlement and calc_method (`ACCRINT(DATE(9994,1,1),DATE(9995,12,31),DATE(9995,12,21),0.05,1000,2,0,TRUE)`); it now
  gives the 1900 value, 98.61111111111111 (previous entry).
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
  workbook with every date moved 1462 days later; COUPPCD and COUPNCD are 2·1462 serials lower. All 165 cells agreed
  bit for bit on 2026-10-12 (probes/date-system-1900-financial-readers-shifted against probes/date-system-1904-financial-readers);
  since 2026-10-21 the 13 ACCRINT cells that move (rows 11, 22 and 33) follow the previous entry and COUPNUM's do not move.
  The s + 1463 reading differed in 93 of the 165 cells (COUPNCD on basis 1 70308
  against 70309, COUPDAYS 181 against 182, YEARFRAC basis 0 4.32778 against 4.33056) and missed 4 more in the last
  digits. Ours reads s + 2·1462 since T-1492 (`DateSystem::financial_1900_serial`, results mapped back by
  `serial_from_financial_1900_serial`): 147 of the 165 cells pass, the other 18 fail the same way in the shifted 1900 twin
  (probes/date-system-1900-financial-readers-shifted), the 1900 gaps of the next sentence. The unshifted 1900 twin (probes/date-system-1900-financial-readers) has its own gaps: COUPDAYS
  basis 1 around 2096-02-29 (financial.md, "COUPDAYS basis 1 counts from the chained next coupon"; fixed in T-1493) and YIELD and
  DURATION last digits (financial.md, "YIELD stops short of PRICE's root"; T-1374).
  A date read past 9999-12-31 is #NUM! in YEARFRAC and DISC (1904 serial 2955542, 9995-12-31, onward).
  Old behavior: until 2026-10-20 ACCRINT with calc_method FALSE took its settlement up to the last 1904 date 2957003,
  counting bases 0 and 4 with first_interest's 1900 serial as the settlement slice
  (`ACCRINT(DATE(9990,1,1),DATE(9990,6,30),DATE(9996,1,1),0.05,1000,2,0,FALSE)` was 410415.8333333334), and was
  #NUM! with calc_method TRUE for a settlement from 9995-12-30 on (probes/date-system-1904-accrint-settlement-bound,
  -financial-settlement-bound, excel-graph 2026-10-13, T-1492, T-1494). Both now give the 1900 values (300 for that
  cell; previous entry).
  The break threshold of COUPPCD/COUPNCD/COUPDAYS is unchanged: the previous coupon of a broken schedule is 1900 serial 1462 in the doubled reading. All probes excel-graph, 2026-10-13. In 1900 itself, ODDLPRICE and ODDLYIELD with maturity 9999-12-31 (last interest
  9999-05-15) are #NUM!, and so is ODDFPRICE/ODDFYIELD with first coupon 9998-12-31 and maturity 9999-06-30
  (issue 9998-07-01: a first period of exactly 180 days 30/360, not the calendar; financial.md, T-1490).
  COUPDAYS on basis 1 in the broken range ignores the settlement: 2958465 for settlements 9999-01-01, 9998-03-01 and
  9996-02-01.
  Probes: date-system-1904-odd-accrint-bound, date-system-1904-odd-threshold and their 1900 twins (excel-graph,
  2026-10-12, T-1447). The odd-period cells are T-1457.
  Probes: date-system-1904-coupon-bound and its twin date-system-1900-coupon-bound (excel-graph, 2026-10-12, T-1440).
  Ours: `DateSystem::coupon_schedule_breaks` (formatter/dates.rs), applied by the COUP* functions other than COUPNUM,
  PRICE, YIELD and DURATION (functions/financial/financial_bonds.rs).

Probes: date-system-1904-financial, date-system-1904-financial-shift, and the twins date-system-1900-financial-daycount
and date-system-1904-financial-daycount (the same 40 cells in each system). All excel-graph, 2026-10-12, T-1436.
Ours: `DateSystem::financial_1900_serial` / `serial_from_financial_1900_serial` (formatter/dates.rs), applied by
YEARFRAC and the financial date readers. Every shift cell matches, COUPNCD at 9999-12-31 (A43) since T-1440.
The other failing cells are last-digit misses (DISC, INTRATE, PRICEDISC, PRICEMAT, YIELDDISC, YIELDMAT, TBILL*; T-1374)
or iteration stopping points (XIRR, YIELD, ODDFYIELD; T-0916). Most fail in the 1900 twin too; PRICEDISC
daycount A20 is 1 ulp off in 1904 only.
Since 2026-10-21 the ACCRINT, COUPNUM and TBILL* cells of these probes follow the previous entry (re-filled
2026-10-27, T-2337).
Status: VERIFIED (web)
Tasks: T-1436, T-1440, T-1447, T-1457, T-1489, T-1492, T-1494, T-2337

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
is `#VALUE!` in both systems too (A23). T-1366 probed the positive value: `TEXT(1.5,"[ss] AM/PM")`, `"[h] AM/PM"`,
`"[mm] AM/PM"` and `"[s] A/P"` are `#VALUE!` (an elapsed token before the marker invalidates the format), but the marker
first is valid: `TEXT(1.5,"AM/PM [h]")` = "PM 12" (the 12-hour clock applies to the elapsed hour too; the engine prints
"PM 36", a known failure). `[h]_)` and `*-[h]` are plain literals/fill, so they behave like the lone token (`TEXT(-1.5,"[h]_)")`
= "-36 ", `TEXT(-1.5,"*-[h]")` = "-36", positive "36 " and "36"); `TEXT(-2,"[<-1]d;0")` is `#VALUE!`; `TEXT(1.5,"[h] h")`
= "36 36", `TEXT(1.5,"[ss]")` = "129600", `TEXT(0.6,"h AM/PM")` = "2 PM"
(probes/text-elapsed-two-tokens, excel-graph 2026-10-26).
Evidence: probes/negative-date-sections-1900 and probes/negative-date-sections-1904 (same 23 questions, excel-graph,
2026-10-11, T-1105).
Ours: the sign rule is `formatter::dates::negative_serial_allowed`, applied in the `ParsePart::Date` arm of
formatter/format.rs to the original value. Two elapsed tokens, and an elapsed token before AM/PM or A/P, are rejected in the section parser (`bracketed_elapsed`,
formatter/parser.rs; T-1366); the 1904 negative sections read the 1904 epoch (T-1330).
Status: VERIFIED (web)
Tasks: T-1105, T-1366, T-1330

## HOUR, MINUTE and SECOND past 9999-12-31
Rule: HOUR and MINUTE of a serial far beyond the last date (2958465, 9999-12-31) are `#NUM!`: `HOUR(123456789.123)` and
`MINUTE(7+123456789.123)` are `#NUM!`, as DAY is.
Evidence: gen-m7/bf889c3917eb54c5 Calc!A1, gen-m7/851eb2099ae3f805 Calc!A39 (excel-graph, 2026-10-11, T-1275).
The bound (T-1353, probes/time-parts-past-max-date A1-A14, excel-graph, 2026-10-17): a serial of 2958466 or more is `#NUM!`
for HOUR, MINUTE and SECOND alike (`HOUR(2958466)`, `MINUTE(2958466.5)`, `SECOND(2958466)`, `SECOND(1E+15)`), while
2958465.99999 still has a time (`HOUR` = 23) and `HOUR(2958465.9)` = 21, `MINUTE(2958465.9)` = 36, `SECOND(2958465.9)` = 0.
Implemented in `time_part_fn!` (functions/date_and_time.rs).
The bound is checked after rounding the serial to the nearest second (T-1815, probes/time-parts-rounding-window,
excel-graph, 2026-10-18): 2958465.9999942 (23:59:59.499) and 2958465.999994 give 23, 59, 59, while 2958465.9999949
(23:59:59.559), 2958465.999999 and 2958465.9999999999 are `#NUM!` for HOUR, MINUTE and SECOND alike, so the cut lies between 23:59:59.499 and
23:59:59.559 (the exact .5 tie: next paragraph).
The exact tie and YEAR (T-3137, probes/t3137-date-parts-near-midnight R!B106:H120, excel-graph, 2026-10-29): the cut
is the half-second carry of "YEAR, MONTH and DAY of a serial just below midnight" below. `fl(2958466-0.5/86400)` =
2958465.9999942128 (R!A110), the exact tie, is `#NUM!` for HOUR, MINUTE and SECOND (R!F110:H110; ours 0, 0, 0) and
for YEAR, MONTH, DAY and WEEKDAY too, while one ULP below it (2958465.9999942123, R!109) is 9999-12-31 23:59:59. So
`YEAR(2958465.9999949)` is `#NUM!` like HOUR: every serial from that tie up to 2958466 is.
Status: VERIFIED (web)

## YEAR, MONTH and DAY of a serial just below midnight
Rule: YEAR, MONTH, DAY and WEEKDAY take the same half-second carry as HOUR, MINUTE and SECOND: the day is
`floor(x + 0.5/86400)` computed in double precision, so a serial within half a second of the next midnight reads as
the next day, and `#NUM!` when that day is past 2958465. The cut is the same double for the date parts and the time
parts on every probed serial: at each base B in 0, 1, 59, 60, 45000, 45016, 45291 and 2958465 (N = B+1),
`fl(N-0.5/86400)` and everything above it (0.4 s, 30, 21, 20, 10, 1 and 0.5 ms, 2^-30 below N) is day N with HOUR 0,
while `fl(N-0.5/86400)` minus one ULP, 0.6 s and 1 s below N are day B with HOUR 23, MINUTE 59, SECOND 59. Examples:
`DAY(1-0.5/86400)` is 1 (ours 0), `DAY(59.99999421296296)` is 29 (the fictitious 1900-02-29, serial 60) and
`MONTH(60.99999421296296)` is 3, `MONTH(45016.999994212965)` is 4 (2023-04-01), `YEAR(45291.999994212965)` is 2024,
`WEEKDAY(45000.999994212965)` is 5 (the next day's). Every serial from `fl(2958466-0.5/86400)` up is `#NUM!` for all
seven functions.
Of T-3137's four candidate models this is (b), the half-second carry. Ruled out: (a) a plain floor (the engine's `date_part`; 220 cells); (c) a millisecond rounding (0.4 s to 30 ms carry);
(d) an absolute near-integer bound like `near_integer_whole`, about 20.6 ms (30 ms and 0.4 s carry, and the cut sits at
the half second, not at 2^-22 + 2^-33 day).
Negatives are `#NUM!` before any carry: `DAY(0-1E-10)`, `YEAR(0-2^-40)`, `MONTH(0-0.4/86400)` and the 0.5, 0.6 and 1 s
negatives are `#NUM!` for all seven functions (R!B121:H126), not serial 0. (A formula `=-2^-40` would be positive:
unary minus binds before `^`, so the probe writes `0-2^-40`.)
Evidence: probes/t3137-date-parts-near-midnight R!A1:H126 (V!A holds the serials, built exactly in formulas;
excel-graph, 2026-10-29, T-3137).
Ours: matches. `carried_day` (`floor(v + 0.5/86400)`, negatives only floored) serves YEAR, MONTH, DAY, WEEKDAY and the bound of HOUR,
MINUTE and SECOND (T-3176; all 223 cells of the probe pass).
Status: VERIFIED (web)

## HOUR, MINUTE and SECOND round to the nearest second
Rule: HOUR, MINUTE and SECOND take the time of day rounded to the nearest second, and the carry goes all the way up:
- `SECOND(TIME(10,59,59)+0.6/86400)` is 0, MINUTE 0 and HOUR 11.
- At 23:59:59.6, HOUR, MINUTE and SECOND are all 0.
- `SECOND("10:11:12.6")` is 13, and `HOUR("23:59:59.6")` is 0.
- `SECOND(DDB(81074,518,21,6,1))`, at 22:24:01.729, is 2.

All 40 probe cells fit one computation in double precision:
1. Add half a second to the serial: `y = x + 0.5/86400`.
2. Take the fraction of the day times 24: `h = (y - floor(y))*24`. HOUR is `floor(h)`.
3. `m = (h - floor(h))*60`. MINUTE is `floor(m)`.
4. SECOND is `floor((m - floor(m))*60)`.

So an exact half goes whichever way the binary noise of these steps takes it, not by a fixed tie rule:
- `SECOND(0.5+1.5/86400)` is 1, `0.5+2.5/86400` 2, `0.5+3.5/86400` 4.
- `1+1.5/86400` is 2, `45000+1.5/86400` 1, `0.5+59.5/86400` 59.
- `SECOND("10:11:12.5")` is 12, and `SECOND("10:11.5")` (611.5 s) is 11.

Neither half-up nor half-even on `frac*86400` or `x*86400` fits these ties, and neither does the exact product: it
lies above `.5` for 2.5 and 59.5, which round down. The fractions .4, .6 and .999 round as expected.
Evidence: probes/second-rounding A1-A40 (excel-graph, 2026-10-19, T-1408); gen-m9/55fc8c4ffbd7c7cc Calc!A40
(excel-graph, 2026-10-12, T-1289); probes/clock-text-oddities F43 (excel-graph, 2026-10-06, T-0153).
Ours: matches since T-1408 (`time_of_day_parts`, functions/date_and_time.rs). Before, MINUTE and SECOND rounded to the
millisecond and then truncated, and HOUR truncated.
Status: VERIFIED (excel-graph)
Tasks: T-1408

## YEARFRAC swaps reversed dates
Rule: YEARFRAC with a start date after the end date swaps them for every basis: the reversed result equals the
forward result, including the asymmetric US 30/360 (basis 0) and European (basis 4) counts at day 30/31 and
end of February. `YEARFRAC(DATE(2024,3,31),DATE(2024,1,15),0)` = 76/360 = its forward pair (not 75/360), and
`YEARFRAC(DATE(2024,3,30),DATE(2024,2,29),4)` = 31/360.
Evidence: probes/yearfrac-reversed-bases (excel-graph, 2026-10-18, T-1826): 9 pairs x bases 0, 2, 3, 4, every
reversed cell equal to its forward cell; gen-m7/c263b43217a5898b Calc!A38 for basis 1 (T-1275).
Ours: all bases swap before counting (T-1354 for basis 1, T-1826 for the rest).
Status: VERIFIED (web)
Tasks: T-1354, T-1826

## YEARFRAC accepts serial 0
Rule: YEARFRAC takes a start or end date in [0, 1) (serial 0, 1900-01-00) on every basis; only a negative serial
is `#NUM!`. The pair is ordered first (see "YEARFRAC swaps reversed dates"). Each basis then reads the dates its
own way:
- Basis 0 (US 30/360) counts on Excel's own day, month and year: serial 0 is day 0 of January, serial 60 is the
  fictitious 1900-02-29, and 1900-02-28 is still February's last day, so it is read as day 30. That makes
  `YEARFRAC(60,59,0)` = **-1/360** (probed reversed; by the swap rule `(59,60,0)` is the same), a negative fraction, and `YEARFRAC(60,61,0)` = 2/360
  (29 February is not February's last day). `YEARFRAC(0,59,0)` = 58/360, `(0,60,0)` = 59/360, `(0,61,0)` = 61/360,
  `(0,366,0)` = 361/360, `(0,33752,0)` = 92.41111111111111.
- Bases 1, 2 and 3 count serials as days, so the fictitious 1900-02-29 is a day: `YEARFRAC(0,60,3)` = 60/365,
  `(59,60,2)` = 1/360. Basis 1's year length reads the real calendar: serial 0 is a day of 1900 and 1900-02-29 is
  no leap day, so `YEARFRAC(0,33752,1)` = 33752 / (average year of 1900-1992) = 92.4086198775318 (counted from
  1899 it would be 92.40928...), `YEARFRAC(1,33752,1)` = 92.4058820065944 also averages from 1900, and
  `YEARFRAC(59,60,1)` = `(60,61,1)` = 1/365.
- Basis 4 (European 30/360) reads serial 0 as day 0 and serial 60 as day 29: `(0,31,4)` = 30/360,
  `(0,366,4)` = 1, `(59,60,4)` = 1/360, `(60,61,4)` = 2/360.
- `YEARFRAC(0,0,b)` and `YEARFRAC(0.5,0.25,b)` are 0 on every basis; 0.5 and 0.9999 count as serial 0.
The discount securities read the same parts (financial.md "Discount securities: day count and year length"), so
they meet the same rules, negative day count included: `ACCRINTM(59,60,0.1,1000,0)` = -0.2777777777777778,
`DISC(59,60,95,100,0)` = -18.000000000000014, `PRICEDISC(59,60,0.05,100,0)` = 100.01388888888889, and serial 0
settles on bases 1-4 (`DISC(0,366,95,100,1)` = 0.04986338797814212).
In a 1904 workbook the financial day counts read a 1904 serial 0 as 1900-serial 2924, so none of this applies there:
`YEARFRAC(0,59,1)` = 59/366 (1908 is a leap year), `YEARFRAC(0,366,0)` = 1.
Evidence: probes/yearfrac-serial-zero (205 cells), probes/yearfrac-serial-zero-1904 (25 cells),
probes/discount-serial-zero (50 cells), excel-graph, 2026-10-21, T-2237.
Ours: `Model::year_fraction_parts` (functions/date_and_time/yearfrac.rs): basis 0 on `excel_serial_to_ymd`, basis 1
on `month_arithmetic_date`, bases 1-3 serial differences; YEARFRAC and ACCRINTM keep the sign (T-2237).
Status: VERIFIED (excel-graph)
Tasks: T-1409, T-2237

## DATEDIF argument errors before date checks
Rule: an error in DATEDIF's unit argument wins over an invalid start date: `DATEDIF(-1,38627,CHOOSE(3,x))` (the unit is
`#VALUE!`) is `#VALUE!`, not the `#NUM!` of a negative date.
Evidence: gen-m7/f68fafa0ca2eba26 Calc!A1 (`Data!D10^DATEDIF((Data!E$5+(-1)),38627,CHOOSE(3,Data!B12))`, E5 blank) (excel-graph, 2026-10-11, T-1275).
Ours: the unit's error wins over a reversed pair and over any date outside the calendar (T-1355, T-1430).
Status: VERIFIED (web); the reversed pair, a high out-of-calendar date, a negative end and an invalid unit text with a
negative start are answered in "DATEDIF and DAYS360 error order with a bad unit or method" (probes/datedif-days360-error-order, T-1430)
Tasks: T-1355, T-1430

## WEEKDAY: date text with literal quote characters is #VALUE!, even with an error return_type
Date text whose string value contains the double-quote characters themselves (`"07/01/2026"` as text, quotes included) does not parse as a date, so WEEKDAY's date argument is `#VALUE!` and wins over an error `return_type` (`1/0`, `NA()`), by the left-to-right rule in arrays.md.
Evidence: ironcalc-calc/DATE_AND_TIME/WEEKDAY_WEEKNUM_ISOWEEKNUM_issue_1502.xlsx (Excel-cached, T-1517).
Ours: matches (all 10 cells pass).
Status: VERIFIED (upstream Excel-authored workbook)
Tasks: T-1517

## Date function lifts: weekend with a lifted date, TAKE/DROP of a range, mismatched shapes, boolean elements (T-1807)
Rule (probes/date-lift-open, excel-graph, 2026-10-18): the lifted `weekend` of WORKDAY.INTL and NETWORKDAYS.INTL broadcasts with
a lifted date like any other argument pair: the result has the largest height and the largest width of the two (what the shorter one pads with is not probed)
(`ROWS(WORKDAY.INTL({45000;45031},5,B1:B3))` = 3, `COLUMNS(...)` = 1; `WORKDAY.INTL(45000,{1;2},{1,2})` is 2x2;
`NETWORKDAYS.INTL(45000,{46000,46100},B1:B3)` is 3x2). A column against a row gives the full grid in every function probed
(`EDATE({45000;45031},{1,2,3})` is 2x3 and `EDATE({45000,45031},{1;2;3})` is 3x2). Mismatched same-axis arrays take the longer length
(`ROWS(EDATE({45000;45031;45100},{1;2}))` = 3, likewise EOMONTH, WORKDAY, NETWORKDAYS, YEARFRAC, WEEKNUM and DAYS).
`TAKE` and `DROP` of a range act like the multi-cell range they cut from (the range rule above applies; whether they return a
reference is not probed): `EDATE(TAKE(A1:A3,2),1)`, `EDATE(DROP(A1:A3,1),1)`, `EOMONTH(45000,DROP(B1:B3,1))`, `WEEKNUM`,
`WORKDAY`, `NETWORKDAYS`, `YEARFRAC` and `WORKDAY.INTL` of a 2-row `TAKE` are `#VALUE!` as a whole (ROWS = COLUMNS = `#VALUE!`), while a 1x1
`TAKE` is that cell (`EOMONTH(TAKE(A1:A3,1),1)` = 45046). `DAYS(TAKE(A1:A3,2),1)` lifts (2 rows) and so does the `weekend`
of NETWORKDAYS.INTL (`NETWORKDAYS.INTL(45000,46000,TAKE(B1:B3,2))` is 2 rows). A boolean element of an array month count is `#VALUE!` for
that element only (`EDATE(45000,{TRUE;2})` = {`#VALUE!`; 45061}, `EOMONTH(45000,{FALSE;1})` = {`#VALUE!`; 45046},
`WORKDAY(45000,{TRUE;5})` = {`#VALUE!`; 45007}, `WEEKNUM(45000,{TRUE;2})` = {`#VALUE!`; 12}).
Ours: matches, all 175 cells pass since WORKDAY, WORKDAY.INTL, NETWORKDAYS and NETWORKDAYS.INTL lift (T-1236).
Status: VERIFIED (excel-graph)
Tasks: T-1807, T-1236

## WORKDAY and NETWORKDAYS from serial 0
Rule (VERIFIED, probes/workday-serial-zero, excel-graph, 2026-10-19, T-1254): WORKDAY, WORKDAY.INTL, NETWORKDAYS and
NETWORKDAYS.INTL take serial 0 (Saturday 0 January 1900) as a start date, an end date, a holiday and a result, on the
weekday-faithful calendar where serial 1 is a Sunday; a negative serial is still `#NUM!`.
- `NETWORKDAYS(0,10)` = 7, `NETWORKDAYS(10,0)` = -7, `NETWORKDAYS(0,0)` = 0, `NETWORKDAYS(0,7872)` = 5623,
  `NETWORKDAYS.INTL(0,10,11)` = 9 (Sunday-only weekend: serial 0 counts), `NETWORKDAYS.INTL(44900,0.2)` = -32071 (the end
  truncates to 0), `NETWORKDAYS(-1,10)` `#NUM!` (F1:F5, F20:F23, F42).
- `WORKDAY(0,5)` = 6, `WORKDAY(10,-6)` = 2, `WORKDAY(10,-7)` `#NUM!` (it would land on -1; serial 0 is a weekend day),
  `WORKDAY.INTL(10,-7,11)` = 2, `WORKDAY.INTL(1,-1,"0000000")` = 0, `WORKDAY.INTL(0,-1,"0000000")` `#NUM!`,
  `WORKDAY(-1,1)` `#NUM!` (F6:F14, F24:F30, F41, F43:F44).
- A holiday of 0 is accepted: `WORKDAY(28131,-16,0)` = 28109, `NETWORKDAYS(1,10,0)` = 7 (F15:F18, F23, F28).
- **A day count of 0 returns the start date without reading the weekend or the holidays** (the count truncates toward
  negative infinity first: 0.4 is 0, -0.5 is -1). `WORKDAY(33393,0,{"a",0.3})`, `WORKDAY(33393,0,-1)`,
  `WORKDAY(33393,0,NA())`, `WORKDAY(33393,0,1/0)`, `WORKDAY(33393,0.4,{"a"})`, `WORKDAY.INTL(33393,0,"1111111")`,
  `WORKDAY.INTL(33393,0,"x")` and `WORKDAY(0,0,"a")` all return the start date (33393 or 0), while
  `WORKDAY(33393,1,{"a",0.3})`, `WORKDAY(33393,-0.5,{"a"})` (-1 days) and `WORKDAY(33393,1,D1:D4)` (a text cell) are
  `#VALUE!`, and `WORKDAY(33393,1,-1)` is `#NUM!`. The start date is still checked: `WORKDAY(-1,0)` is `#NUM!`
  (F31:F38, F45:F58). NETWORKDAYS has no such shortcut: `NETWORKDAYS(33393,33393,{"a"})` and
  `NETWORKDAYS(33393,33400,D1:D4)` are `#VALUE!` (F35, F37, F39, F56).
- `WORKDAY(10,-0.5)` = 9 and `WORKDAY(10,0.5)` = 10 (F53:F54).
Implemented in: `DateSystem::date_of_serial_from_zero` (formatter/dates.rs), which `Model::excel_date`
(functions/date_and_time.rs) reads for every WORKDAY and NETWORKDAYS date and holiday, and the zero-day shortcut and
lower walk bound in `Model::workday_of` / `Model::step_workdays` (functions/date_and_time/workday.rs).
Tasks: T-1254

## Typed date plus clock with a single-letter A or P suffix is a number (T-1994)
Rule: typed text `<date> <clock> A` or `P` (any case, space before the letter) is a number with
`m/d/yyyy h:mm`, like `AM`/`PM`: `1/2/2020 10:00 A` = 43832.41666..., `10:00 P` = 43832.91666...,
`10 A` and `10:00:30 A` too, `July 1, 2026 10:00 A` = 46204.41666.... The letter does not extend to
a glued suffix (`10:00A`, General text) or a doubled one (`10:00 A A`, General text); an out-of-range
hour (`13:00 A`) stays text with the date's format `m/d/yyyy`. Evidence: probes/date-text-single-letter-ap
N1-N8, N10, N11 (excel-graph 2026-10-19), probes/date-text-rejected-clock-formats N7.
A trailing dot after the letter is accepted wherever the letter is: typed `1/2/2020 10:00 A.` is 43832.41666... with
`m/d/yyyy h:mm` (N12), so are `1/2/2020 10 A.` and `1/2/2020 10:00:30 P.` (N17, N18), and
`1/2/2020 13:00 A.` stays text with `m/d/yyyy` (N16). A bare typed clock with the letter is a time of day:
`10:00 A`, `10:00 p`, `10:00 A.`, `10 A` and `10:00 AM` are 0.41666... (0.91666... for p) with `h:mm AM/PM`,
but `10:00:30 A` is 0.41701... with `h:mm:ss AM/PM` (N9, N13-N15, N19, N20; excel-graph 2026-10-19, T-2011).
Further bare shapes (N21-N29, T-2053): `10 A.`, `10 AM` and `10 PM` are numbers with `h:mm AM/PM`; `0:00 A` and
`12:00 A` are 0 with `h:mm AM/PM`; `10:00:30 AM` has `h:mm:ss AM/PM`; a fractional second keeps its own format,
`10:00:30.5 A` is 0.41701... and `10:00.5 A` 0.00695... with `mm:ss.0`; bare `13:00 A` stays text with General.
Status: VERIFIED (web)
Implemented in: `typed_clock_shape`, `rejected_clock_text_format`, `parse_typed_number`, `parse_bare_letter_clock` and `without_single_letter_dot` in `crates/rustsheet-engine/src/formatter/parse_formatted.rs`
Evidence note: docs/research/single-letter-ap-typed-clock-2026-10-19.md
Tasks: T-1994, T-2011

## Serials below 61 in DATEDIF, EDATE, DAYS360, WEEKNUM and YEARFRAC (gen-m6 triage, T-2263, T-2265)
Rule (excel-graph, gen-m6 and probes/dates-early-1900-displayed, 2026-10-21): these functions read a serial below 61 on the
*displayed* 1900 calendar, where serial 1 is 1900-01-01, serial 60 the fictitious 1900-02-29 and a serial in [0, 1) is day 0
of January 1900, not on the weekday-faithful stand-in (serial 1 = 1899-12-31) that WEEKDAY, WORKDAY and NETWORKDAYS read:
- DATEDIF counts Y, M, YM, MD and YD on the displayed `(year, month, day)` (with day 0 for [0, 1)) and D on the serials;
  February 1900 has 29 days. `DATEDIF(0.5,60,"D")` = 60, `DATEDIF(0.5,17471,"Y")` = 47, `DATEDIF(0.5,44241,"YD")` = 45,
  `DATEDIF(<blank>,45382,"MD")` = 31 (day 0), `DATEDIF(1,45291,"Y")` = 123, `DATEDIF(2,1462,"MD")` = 30,
  `DATEDIF(60.5,45291,"MD")` = 2 (day 29), `DATEDIF(59.999988426,38168,"YD")` = 123 (gen-m6/144bceacd894868a Calc!A14,
  3a26794015dfbe9d A190, a1e6448d0590f7c3 A168, 1a404a0379a1372c A324, 07171fa3f7493f51 A210, 0bcac2d96e96b3df A94,
  ee56a4ed5a612a0d A185, b54448af3fde07ca A55; 26 cells). Probe A50-A98: `DATEDIF(59,61,"MD")` = 2 and `DATEDIF(60,61,"MD")` = 1
  (February 1900 lends 29 days), `DATEDIF(0,61,"MD")` = 1, `DATEDIF(0,60,"YD")` = 60, `DATEDIF(0,366,"YD")` = 366,
  `DATEDIF(0,367,"YD")` = 1, `DATEDIF(60,1520,"Y")` = 3, `DATEDIF(60,1521,"Y")` = 4, `DATEDIF(60,426,"M")` = 12,
  `DATEDIF(60,1520,"YM")` = 11, `DATEDIF(0,0,"D")` = 0; a negative date is `#NUM!` (`DATEDIF(-1E-15,10,"D")`). An unknown unit is `#NUM!` for any pair, day 0 included ("DATEDIF and DAYS360 error order with a bad unit or method", T-2288):
  `DATEDIF(0.1,874,"1a*")`, `DATEDIF(<blank>,0,<bad unit>)` (gen-m6/03333cc38ec06a14 A269, 3a26794015dfbe9d A225, 4ca39e0876796fa5 A362,
  60bcd25271767317 A272, 9130ade4b1924cc8 A137, d0c30225243fde60 A129, gen-m7/44ef4f5638e25bcd A18).
- EDATE reads the displayed day and clamps on the real calendar: serial 60 is 29 February, so `EDATE(60,1)` = 89 (29 March),
  `EDATE(60.5,5)` = 211, `EDATE(60.5,10)` = 364, `EDATE(60,12)` = 425 (1901-02-28), `EDATE(60,48)` = 1521 (1904-02-29),
  `EDATE(60,-1)` = 29, while a February 1900 result has 28 days: `EDATE(60,0)` = `EDATE(29,1)` = `EDATE(30,1)` = 59
  (0fd242eb9b2456a8 A152, 1aa50086e42bff9a A183; probe A38-A47). EOMONTH only reads the month: `EOMONTH(60.5,1)` = 91,
  `EOMONTH(60,-1)` = 31 (A37, A48, A49).
- DAYS360 reads the displayed date and counts 1900-02-29 as February's last day (US method): `DAYS360(60,1)` = -59,
  `DAYS360(59,60)` = 1, `DAYS360(59,90)` = 32, `DAYS360(31,60)` = 29, `DAYS360(0,60)` = 59, `DAYS360(60,425)` = 358,
  `DAYS360(60,366,TRUE)` = 301 (36d5ce763ea63631 A380; probe A17-A28). Any negative date is `#NUM!`, however tiny:
  `DAYS360(100,-1E-15)`, `DAYS360(-1E-8,100)` and `DAYS360(56330,-5E-15)` (50f0dd9a8ee2cb57 A12), while `DAYS360(1E-15,100)`
  = 99 (A29-A36).
- WEEKNUM and ISOWEEKNUM need nothing new: `WEEKNUM(60.5,21.99113)` = 9, `WEEKNUM(60,21)` = `ISOWEEKNUM(59)` = `ISOWEEKNUM(60)`
  = `ISOWEEKNUM(61)` = 9, and a fractional return_type truncates (`WEEKNUM(45000,21.99)` = 11, `WEEKNUM(45000,20.99)` is
  `#NUM!`; A1-A16). The `#VALUE!` of gen-m6/c96ac1ada46179c4 A105 comes from its return_type
  `PRODUCT(Data!D11:D12)+SUMPRODUCT(Data!A3)`: `SUMPRODUCT` of a lone blank cell is `#VALUE!` (coercion.md "SUMPRODUCT: a blank
  one-cell argument beside other arguments is `#VALUE!`"; probes/weeknum-sumproduct-blank-type A1-A12).
- YEARFRAC basis 0 of a date in serials 1-60 counts from the displayed date: `YEARFRAC(1,30145)` = 82.5333 (29712/360),
  `YEARFRAC(22676,7,0)` = 62.06389, `YEARFRAC(3.14159,45291)` = 123.99444, while serial 60 is 29 February without the
  end-of-February adjustment: `YEARFRAC(55862,60)` = 152.7778 (55000/360), `YEARFRAC(60.5,34993)` = 95.64444,
  `YEARFRAC(60,425,0)` = 359/360 and `YEARFRAC(59,425,0)` = 1 (752959b8d3db0285 A35, 121a7c17092038d9 A175, 7f3606fef4cb9df7 A395,
  3e51213f35889d06 A320, 4ca39e0876796fa5 A308; 25 basis-0 cells in gen-m6). Basis 1 counts serial days (the fictitious day
  included) over the real calendar's average year: `YEARFRAC(48344,1,1)` = 132.35660175388034, `YEARFRAC(25538,1,1)` =
  69.91786287010599, `YEARFRAC(1,366,1)` = 1, `YEARFRAC(1,367,1)` = 366/365, `YEARFRAC(1,60,1)` = 59/365, `YEARFRAC(60,1520,1)` =
  3.9978094194961664. Bases 2 and 3 count serial days (the fictitious day included) over 360 and 365: `YEARFRAC(1,366,2)` = 365/360,
  `YEARFRAC(60,366,3)` = 306/365. Basis 4 is European 30/360 on the displayed date: `YEARFRAC(1,366,4)` = 359/360,
  `YEARFRAC(60,366,4)` = 301/360 (29 February is day 29). probes/yearfrac-early-1900 holds serials 1, 31, 59, 60, 60.5 and 61 against 366 and
  45000 on bases 0-4, plus A61-A70 (70 cells, excel-graph 2026-10-21). Implemented by T-2237 (`year_fraction_parts`); T-2265
  added the probe.
Status: VERIFIED for DATEDIF, EDATE, EOMONTH, DAYS360, WEEKNUM and YEARFRAC (the cells above). Implemented in `datedif_value`,
`shift_displayed_date` (EDATE), `fn_days360` (T-2263) and `year_fraction_parts` (T-2237).
Tasks: T-2263, T-2265

## DATEDIF "MD" and "YD" day counts (gen-m6 triage, T-2264, T-2263)
Rule (excel-graph): "MD" is the end's day minus the start's day, plus the length of the month before the end's month when that is
negative, and the sum can stay negative (`DATEDIF(45382,45944,"MD")` (2024-03-31 to 2025-10-14) = 13 = 14-31+30,
`DATEDIF(19510,55815,"MD")` = 22, `DATEDIF(28064,42206,"MD")` = 20). "YD" is the start moved on by the "YM" months inside the
start's year, with DATE's day overflow (31 January + 1 month is 3 March, 29 February 1900 exists), plus the "MD" days:
`YD = DATE(year, month + YM, day) - start + MD`. Cells: `DATEDIF(29872,35271,"YD")` (1981-10-13 to 1996-07-25) = 285,
`DATEDIF(51118,54889,"YD")` = 119, `DATEDIF(41671,51446,"YD")` = 278, `DATEDIF(15974,43001,"YD")` = 364, `DATEDIF(6946,30954,"YD")` = 266
(gen-m6/c96ac1ada46179c4 Calc!A274, 22ce5678fa646215 A294, 3516b9fbbd60269b A87, 144bceacd894868a A356, 3516b9fbbd60269b
A353, 9c615d338d990644 A231, c6621490a4d67f4e A60, de083028d8b676d2 A38). The rule is not "move the end into the start's year":
from a 29 February start, `DATEDIF(1521,1887,"YD")` (1904-02-29 to 1905-03-01) = 0, `DATEDIF(1521,1888,"YD")` = 1,
`DATEDIF(1521,1979,"YD")` (to 1905-06-01) = 93, `DATEDIF(1521,2983,"YD")` (to 1908-03-01) = 1, `DATEDIF(1521,1886,"YD")` = 365,
`DATEDIF(60,1155,"YD")` = 365 and `DATEDIF(60,1156,"YD")` = 0; and from a 31 January start
`DATEDIF(DATE(2024,1,31),DATE(2025,3,1),"YD")` = 29 (MD is -2), `DATEDIF(DATE(2023,1,31),DATE(2023,3,5),"YD")` = 33,
`DATEDIF(DATE(2023,3,31),DATE(2023,5,1),"YD")` = 31, `DATEDIF(DATE(2023,8,31),DATE(2024,3,1),"YD")` = 183
(probes/dates-early-1900-displayed A55-A59, A74-A79, A96-A116). The rule fits all 39 YD cells; a clamped DATE misses 8.
Status: VERIFIED for these cells.
Also VERIFIED (T-2295, probes/datedif-feb29-negative-md, 66 excel-graph cells, all matching the rule above): from a 29 February
start in a non-leap end year, `DATEDIF(DATE(2020,2,29),DATE(2021,2,28),"YD")` = 365 (A4; "MD" 30, "YM" 11: `DATE(2020,13,29)` is
2021-01-29, 335 days on, plus 30), to 2021-03-01 = 0 ("MD" 0), to 2021-03-15 = 14, to 2021-02-27 = 364; an end of 29 February
reached from a 31 January start, `DATEDIF(DATE(2015,1,31),DATE(2016,2,29),"YD")` = 29 ("MD" 29); and a negative "MD", which stays
negative: `DATEDIF(DATE(2015,1,31),DATE(2015,3,1),"MD")` = -2 (from 2015-01-30: -1; 2015-12-31 to 2016-03-01: -1). A start after the
end is `#NUM!` for every unit (A34-A39).
Tasks: T-2264, T-2263, T-2295

## TIME: an argument past 32767 is `#NUM!`; a near-integer hour counts as that integer (T-2267)
Rule (excel-graph, 2026-10-21): TIME reads each of its hour, minute and second on its own, in that argument's own unit and
at any magnitude: it floors the value, except that a value less than about 2.4E-7 below an integer counts as that integer, and
for a negative value the window is half as wide, about 1.2E-7. Measured edges: positive, `1-(2^-22+1E-12)` counts as 1 and
`1-2.4E-7` does not (same at 5, 30000 and 32768); negative, `-1-(2^-23+1E-12)` and `-(2^-23+1E-12)` count as -1 and 0, while
`-1-1.2E-7` and `-1.2E-7` floor (same at -5, -30000, -32767). The positive window is the one LEFT, RIGHT, REPLACE and
INDEX use (`NEAR_INTEGER_TOLERANCE`, 2^-22+2^-33, text.md and lookup.md), which lies inside the measured (2^-22+1E-12, 2.4E-7);
we take half of it for a negative value, inside (2^-23+1E-12, 1.2E-7). The exact constants inside those brackets are unprobed
for TIME and DATE. The XMATCH and XLOOKUP modes, read by the same `near_integer_whole`, bracket both constants to a relative
2^-14 (lookup.md "XMATCH and XLOOKUP floor a fractional mode", T-2656). Then any argument outside
-32768..32767 is `#NUM!` (before the total is formed, so a minute of -32768 is legal and one of -32769 is not, whatever the
hour), and so is a negative total; otherwise the value is `((h*3600+m*60+s)/86400) % 1` as in "TIME divides by 86400 first".
Mind Excel's precedence when probing: `-2^-22` is `(-2)^-22`, a positive number; write `-(2^-22)`.
- Bound: `TIME(32767,0,0)` = 0.29166666666674246, `TIME(600,32767,0)` = 0.7548611111111114, `TIME(10,0,32767)` =
  0.7959143518518519; the same with 32767.5 or 32767.9999; 32768, 65535, 65536 and -32769 in any position are `#NUM!`;
  `TIME(600,-32768,0)` = 0.24444444444444446, `TIME(10,0,-32768)` = 0.03740740740740741; `TIME(32767,32767,32767)` =
  0.4257754629629744; `TIME(600,-32768.5,0)` and `TIME(10,0,-32768.5)` are `#NUM!` (floored to -32769), while
  `TIME(600,-32768-1E-7,0)` is a number (inside the window). `TIME("32768",0,0)` is `#NUM!`, `TIME("32767",0,0)` a number.
  (probes/time-argument-bound A1-A35, A77-A79; time-argument-window A73-A76.)
- Window: `TIME(1-2^-22,0,0)` = 1:00 and `TIME(1-2.4E-7,0,0)` = 0:00; the same edge for the minute (`TIME(0,1-2^-22,0)` =
  0:01) and the second, at 1, 5, 30000 and 32768 alike (`TIME(0,0,32768-2.3E-7)` is `#NUM!`, `-2.4E-7` gives 9:06:07);
  `TIME(10,0,-1E-7)` = 10:00 but `TIME(10,0,-1.2E-7)` = 9:59:59, `TIME(10,0,-1-1E-7)` = 9:59:59, `TIME(1,-0.5,0)` = 0:59,
  `TIME(1,-1E-15,0)` = 1:00,
  `TIME(-0.5,0,1)` and `TIME(0,0,-0.0005)` are `#NUM!`. So `1/POWER(0.1,2)` (99.99999999999999), 100-1E-7 and 23.9999999 hours
  count as 100 and 24 (`TIME(23.9999999,0,0)` = 0, `HOUR(TIME(99.99999999999999,0,0))` = 4), but 32767.9999 floors.
  Each argument floors on its own: `TIME(0,0.5,30)` = 0:00:30 and `TIME(0,0.99999,0.0006)` = 0.
  (probes/time-argument-bound A36-A76, time-argument-rounding, time-argument-window A1-A72 and A77-A81, time-argument-negative,
  time-argument-negative-window.)
- gen-m6 cells (34 in T-2267): `TIME(9,DATEVALUE("1/15/2024"),39)`, `TIME(DATEVALUE("2023-03-01"),27,26)`, `TIME(18,33,FACT(13))`,
  `TIME(1E+15,44,15)`, `SECOND(TIME(1E+15,10,48))` are `#NUM!`; `TIME(COLUMNS(A9)/POWER(0.1,2),12,35)` = 0.17540509259259274.
Ours: `time_argument` in `functions/date_and_time.rs` (T-2267).
Status: VERIFIED (excel-graph probes above, 2026-10-21).
Tasks: T-2267

## DATE: a year, month or day just below an integer reads as that integer (T-2268)
Rule (excel-graph, 2026-10-21): DATE reads its year, month and day as TIME reads its arguments ("TIME: an argument past 32767 is
`#NUM!`" above): each is floored, except that a value less than about 2.4E-7 below an integer counts as that integer, and half that
(about 1.2E-7) when the value is negative; then the existing bounds apply ("DATE month and day bounds"). Measured edges, the same
for year, month and day and the same as TIME's: `DATE(2020,3,2-(2^-22+1E-12))`, `DATE(2020,3-(2^-22+1E-12),2)` and
`DATE(2021-(2^-22+1E-12),3,2)` are 2 March 2020 / 2021 while `2-2.4E-7` floors (`DATE(2020,3,2-2.4E-7)` = 43891,
`DATE(2020,3-2.4E-7,2)` = 43863, `DATE(2021-2.4E-7,3,2)` = 43892); `DATE(2020,3,-(2^-23+1E-12))` = 43890 (day 0) and
`DATE(-(2^-23+1E-12),3,2)` = 62 (year 1900), while `-1.2E-7` floors (`DATE(2020,3,-1.2E-7)` = 43889, `DATE(-1.2E-7,3,2)` is
`#NUM!`); the same at -5 and at ±30000 (`DATE(2020,3,30000-2^-22)` = 73890, `-2.4E-7` gives 73889). The window comes before the
bounds: `DATE(9999,12,31+1-1E-7)` is `#NUM!` (1 January 10000) and `DATE(10000-1E-7,1,1)` is `#NUM!`, `DATE(1900-1E-7,1,1)` = 1,
`DATE(2020,32767-1E-7,1)` is `#NUM!` (month 32767), `DATE(2000,-32768-1E-7,1)` is `#NUM!`. gen-m6 cells: `DATE(1983,12,-1E-15)` =
30650, `DATE(2015,-1E-21,3)` = 41976, `DATE(-4.2E-16,1,23)` = 23, a day of `COS(-0.0001)` (0.999999995) is 1
(gen-m6/216e3c05f407ca79 Calc!A173, 03333cc38ec06a14 A267, 9843ec1f29523dd2 A265, b54448af3fde07ca A379, 07171fa3f7493f51 A311).
Evidence: probes/date-argument-window (90 cells, excel-graph, 2026-10-21).
Ours: `fn_date` reads each argument through the same near-integer floor as TIME (T-2268).
Status: VERIFIED
Tasks: T-2268

## WORKDAY and NETWORKDAYS holidays: text holidays skip the day, they are not just accepted (T-2369)
See also the older number-text sections below (T-2270, T-2272).
Rule (excel-graph, probes/t2369-workday-text-holidays, 2026-10-25): a text holiday is coerced like a number-valued
argument and then counts as a holiday wherever it lands, in a cell, a literal, a formula result or an array. With start
45000 (Wed), +5 workdays and holiday 45001 (Thu): `WORKDAY(45000,5,A1)` is 45008 (one day skipped) and
`NETWORKDAYS(45000,45007,A1)` is 5 (not 6) for A1 `"45001"`, and identically for a literal `"45001"`, `TEXT(45001,"0")`,
`{"45001"}`, `"45001.5"`, `" 45001"` (padded), date text `"3/16/2023"`, currency text `"$45001"`, and a TEXT() date.
Percent and time text are no error but no holiday in the range (their coerced serials, 0.5 and 100000, are inferred, not observed): `"50%"` and `"12:00"` (0.5)
leave WORKDAY at 45007 and NETWORKDAYS at 6, as does `"1E5"` (100000, on the calendar but past the walk). The `.INTL`
forms (weekend 1) agree with WORKDAY/NETWORKDAYS in every case, and `{"45001";"45002"}` gives NETWORKDAYS 4.
Ours already matched all 68 cells (`collect_serial_numbers` counts what `cast_number` reads); no code change.

## WORKDAY and NETWORKDAYS holidays: blank and number-text cells, and the error order (T-2270, T-2272)
Rule (excel-graph, gen-m6, 2026-10-21):
- A one-cell `holidays` reference to a blank cell is no holiday, not `#VALUE!`: `NETWORKDAYS(17247,33116,Data!E9)` = 11336,
  `NETWORKDAYS(59460,0,Data!C$3)` = -42471, `WORKDAY.INTL(60,PI(),4,Data!E5)` = 63 (gen-m6/3e51213f35889d06 Calc!A314, b62dd1ebd266b71c
  A111, acdfeaf6e6eadd4e A83). A holiday cell holding number text is accepted: `WORKDAY(18260,-30,Data!C4)` with C4 `"1"` = 18218,
  `WORKDAY(58719,5,Data!D5:D7)` over 1.005, `"1.5"`, 0.3 = 58726 (1aa50086e42bff9a A67, 752959b8d3db0285 A393). (Ours was `#VALUE!` for both before T-2270.)
  (Non-number text is still `#VALUE!`: "WORKDAY and NETWORKDAYS from serial 0" F56.)
  Number text inside a holidays *array* is a holiday too: `WORKDAY.INTL(26624,-19,5,{"1.5"})` = 26599, `WORKDAY(26624,-19,{"1.5"})`
  = 26597, `NETWORKDAYS.INTL(26600,26624,5,{"1.5"})` = 19, while `{"a"}`, `{"ü"}`, `{""}`, `{TRUE}` and `{FALSE}` are `#VALUE!`
  (probes/t2360-array-lambda-holidays W1-W15, excel-graph, 2026-10-23, T-2360). So a GROUPBY or PIVOTBY whose LAMBDA passes a group's
  values to WORKDAY.INTL errs only in the groups holding non-number text, a boolean or a number off the calendar (G!A1:H13; the
  t2311-pivotby-value-cause D46 and D51 counts 13 and 27). Ours since T-2360 (`collect_serial_numbers`); before, `{"1.5"}` was `#VALUE!`.
- Blanks inside a holidays array are no error either: with Data!Z1:Z2 blank, `NETWORKDAYS(8812,58239,HSTACK(Data!Z1:Z2,{1;2}))` = 35306,
  `WORKDAY(46645,26,HSTACK(Data!Z1:Z2,{1;2}))` = 46681 and `WORKDAY.INTL(47074,-9,3,HSTACK(..))` = 47061 (gen-m6/3eef60b5dc0145e9 A73,
  449aebbde77d8ce1 A30, A306; excel-graph, 2026-10-22, T-2253). Whether a blank element is skipped or read as serial 0 does not change
  these results. Implemented (T-2395): `collect_serial_numbers` skips an `ArrayNode::Empty` element; serial 0 would give the same results.
- `#NUM!` from the dates, the result or a numeric holiday beats a text or boolean holiday's `#VALUE!`: `WORKDAY(0,-23,Data!A5:A5)`
  ("ABC", the result would be before serial 0), `WORKDAY(2958465,18,..)` (past the calendar), `NETWORKDAYS(-1,3214,Data!B5)` ("abc"),
  `WORKDAY(10651,-7,Data!B8:D12)` (holidays "1.5", "ABC" and -1) are all `#NUM!`; and an invalid weekend's `#NUM!` beats a boolean holiday's `#VALUE!`: `WORKDAY.INTL(42,-10,18,Data!C9:E9)`
  (weekend 18 is not a weekend number; holidays TRUE, 0, FALSE) is `#NUM!`
  (1a404a0379a1372c A199, 121a7c17092038d9 A95, 870c5e38f51ce5d4 A8, 216e3c05f407ca79 A312, 9843ec1f29523dd2 A331; 58 cells in T-2272).
  The result's `#NUM!` beats a holidays error value too: `WORKDAY(1462,NETWORKDAYS.INTL(21128,12887,"1000001"),-2.5/0)` and
  `WORKDAY.INTL(1,-18,13,ERROR.TYPE("x~*"))` are `#NUM!` (d0c30225243fde60 A332, 9130ade4b1924cc8 A361), which "An error-valued or 3-D
  weekend" (holidays read first) does not cover. An all-weekend mask's `#VALUE!` beats a negative holiday's `#NUM!`:
  `WORKDAY.INTL(45382,10,"1111111",Data!B2:C5)` (holidays include -0.5 and " x ") is `#VALUE!` (ef10bf3afb9658f6 A346).
- The full order (probes/workday-holidays-error-order, excel-graph, 2026-10-22, T-2272; 536 formula cells, 541 checked: twelve
  holidays columns, A-M, against 36 call shapes, rows 1-36, plus column Q). WORKDAY and WORKDAY.INTL: (1) the start date, its type and then its
  range (`WORKDAY(-1,"x")` and `WORKDAY(-1,TRUE)` are `#NUM!`, Q56:Q58); (2) the day count's type (`WORKDAY(45000,"x",..)`
  `#VALUE!` over every holidays, row 6); a day count of 0 is the start date; (3) `#NUM!` when start + days is off the
  calendar, before the holidays and the weekend, an invalid one, `"abc"` and an all-weekend mask included (rows 4, 5, 12, 13,
  18, 19; `WORKDAY.INTL(1,-18,"abc")` `#NUM!`, Q29; `WORKDAY.INTL(5,-6,"1111111")` `#NUM!` but `(5,-5,"1111111")` `#VALUE!`,
  Q30:Q36; `WORKDAY(0,-1,"a")` `#NUM!`, `WORKDAY(1,-1,"a")` `#VALUE!`, Q65:Q67); (4) a single holiday (a scalar or a one-cell
  reference) that is no number: text and booleans `#VALUE!`, an error itself (`WORKDAY.INTL(45000,5,99,"a")` `#VALUE!`,
  `(..,99,NA())` and `(..,"1111111",NA())` `#N/A`, `(..,99,H1:H1)` with H1 "a" `#VALUE!`; rows 14-17, Q37:Q48); (5) the weekend
  (WORKDAY.INTL), an all-weekend mask included; (6) every other holidays error: a single number off the calendar
  (`WORKDAY.INTL(45000,5,"1111111",-1)` `#VALUE!`, Q45), and anything in an array or a multi-cell range, a one-element array too
  (`WORKDAY.INTL(45000,5,99,{"a"})` `#NUM!`, `(..,"1111111",{#N/A,1})` `#VALUE!`, Q42, Q49; rows 14-17 columns F-L); (7) a walk
  that leaves the calendar although start + days does not (`WORKDAY(5,-5)` and `WORKDAY(10,-8)` `#NUM!`, Q26, Q27, but with
  holidays "a", NA(), -1 those holidays' errors, Q1:Q25). NETWORKDAYS and NETWORKDAYS.INTL: the weekend first (any error,
  rows 30-36), then the start date, its type and range, then the end date likewise (`NETWORKDAYS(-1,"x")` `#NUM!`,
  `NETWORKDAYS("x",-1)` `#VALUE!`, `NETWORKDAYS(45000,-1,"x")` `#NUM!`; rows 23-26, Q50:Q55), then the holidays.
  Within the holidays the first bad one in reading order decides: "a" then -1 is `#VALUE!`, -1 then "a" `#NUM!`, NA() then "a"
  `#N/A` (columns F-L of rows 1, 22, 27-29; Q60:Q64).
  T-1954's question: `WORKDAY.INTL(45000,5,"1111111",NA())` is `#N/A` (row 15 column E), and with the holidays in an array or a
  range the all-weekend `#VALUE!` wins.
Ours: implemented (T-2270, T-2272): a blank one-cell reference is no holiday; a cell holding number text is a holiday, counted rather than skipped (serial 1 in 1aa50086e42bff9a A67 and 752959b8d3db0285 A393 lies outside every range, so skip-vs-count is UNVERIFIED). Also UNVERIFIED: the scalar branch accepts any text `cast_number` reads (a literal `"45001"`, a formula's number text, date, currency or percent text) as a holiday; the cells only show holidays reached through a reference. Other text is `#VALUE!` as before. The order is `Model::workday_of` and `Model::networkdays_of`, with `HolidaysError::before_weekend` for step (4) (crates/rustsheet-engine/src/functions/date_and_time/). Unprobed: whether single number text off the calendar (a `"-1"` cell) is step (4) or (6); ours (6) (T-2271 probes it).
Status: VERIFIED
Tasks: T-2270, T-2272

## WORKDAY.INTL and NETWORKDAYS.INTL: weekend number coercion (T-2271)
Rule (excel-graph, gen-m6, 2026-10-21): a weekend number truncates like the other integer arguments, a boolean weekend does not raise its own error before the
start's, and a blank weekend cell is weekend 0 (`#NUM!`), not the default: `NETWORKDAYS.INTL(16492,11827,3.14159)` = -3334, `NETWORKDAYS.INTL(0.5,45351,2.5^0.3)`
= 32394, `WORKDAY.INTL(45351,10,1.005)` = 45365, `NETWORKDAYS.INTL(44926," x ",3.14159)` is the end's `#VALUE!` (not the weekend's `#NUM!`),
`NETWORKDAYS.INTL((0/0)^..,40608,IFERROR(#N/A,TRUE),..)` is the start's `#DIV/0!`, `WORKDAY.INTL(16895,22,<blank>,..)` is `#NUM!`, and so is `NETWORKDAYS.INTL(13803,28521,<blank>)` (gen-m6/afe9267df3df26a8 A47, T-2253)
(gen-m6/c93ea588c271840e Calc!A31, 1a404a0379a1372c A62, ee56a4ed5a612a0d A186, c96ac1ada46179c4 A287, ee56a4ed5a612a0d A387,
fde83d52a9d1fc63 A219). A boolean weekend is its number: TRUE acts as weekend 1 (`WORKDAY.INTL(45000,5,TRUE)` = 45007, `NETWORKDAYS.INTL(45000,45030,TRUE)` = 23, as weekend 1; on these dates weekends 2 and 3 give the same, so the probe shows TRUE is one of 1-3) and FALSE is weekend 0 (`#NUM!`), both as a literal and read from a cell; 3.9 is weekend 3 and 0.5 is 0 (`#NUM!`), so a number truncates (11.5 and 1.5 match truncation, but there 11 and 12 agree) and a blank cell is `#NUM!` (probes/weekend-number-coercion A1:A26, excel-graph, 2026-10-21). An omitted weekend is still weekend 1; an empty argument (`WORKDAY.INTL(x,5,)`) is unprobed (the engine keeps the default). T-2591 probes these gaps.
A FALSE weekend cell raises no `#VALUE!` ahead of a negative holiday's `#NUM!`: `NETWORKDAYS.INTL(39588,SUMIF(Data!A10:C11,"=a"),Data!C$5,Data!B4:E7)`
with C5 FALSE and -1 among the holidays (B7) is `#NUM!` (gen-m6/f93597dff1cfa5b6 A252, excel-graph, 2026-10-22, T-2253), as FALSE is weekend 0.
Ours: implemented (T-2271; the fraction truncation first landed with T-2272).
Status: VERIFIED
Tasks: T-2271

## WEEKNUM of a boolean serial is `#VALUE!` (T-2276)
Rule (excel-graph, gen-m6, 2026-10-21): a boolean serial read from a cell is `#VALUE!` in WEEKNUM (unlike WEEKDAY and ISOWEEKNUM,
where it is serial 1, "Date function argument modes"), and it beats an invalid return_type: `WEEKNUM(Data!A12,1)`, `WEEKNUM(Data!C9,2)`,
`WEEKNUM(Data!$E$2)` and `WEEKNUM(Data!D$4,SUM(0.5))` with FALSE cells are `#VALUE!` (gen-m6/0bcac2d96e96b3df Calc!A234,
3df034e67b14fccb A149, 4ca39e0876796fa5 A274, 3a26794015dfbe9d A242). A typed boolean is `#VALUE!` as well: `WEEKNUM(TRUE)`, `WEEKNUM(FALSE,0.5)`, `WEEKNUM({TRUE})` and `WEEKNUM(IF(TRUE,TRUE))`, and so is the text "TRUE"; `WEEKNUM(B1+0)` is 1 (probes/weeknum-boolean-serial, excel-graph, 2026-10-21). Ours: implemented (T-2276).
Status: VERIFIED
Tasks: T-2276

## YEARFRAC reads its start's range before the end's error (T-2266)
Rule (excel-graph, gen-m6, 2026-10-21): a start date out of the calendar is `#NUM!` even when the end is an error:
`YEARFRAC(SUM(-1,FALSE,<blank>),SUMIFS(D10:D12,A5:B7,"abc")+..,0)` (end `#VALUE!` from the SUMIFS shape mismatch) and
`YEARFRAC(123456790.12..,NETWORKDAYS.INTL(51358,34485,15,<FALSE cell>)+1E-15,0)` are `#NUM!` (gen-m6/3516b9fbbd60269b Calc!A301,
d0c30225243fde60 A266). Ours: implemented (T-2266).
Status: VERIFIED for these two cells; an error start against an error end keeps the start's error (VERIFIED: probes/scalar-lift-open-d W2 `YEARFRAC(NA(),1/0,1)` and W5 `YEARFRAC(1/0,NA(),1)`). An out-of-calendar start against a text or multi-cell-range end (`YEARFRAC(-1,"x")`, now #NUM!) is UNVERIFIED.
Tasks: T-2266

## NETWORKDAYS.INTL: the blank weekend's #NUM! before the holidays' error (T-2436)
Rule: `NETWORKDAYS.INTL(31,4886,<blank>,{1,2;#N/A,""})` is `#NUM!`, the blank weekend cell's error (T-2271) before the holidays'
`#N/A` (gen-m9 c2974c061384fdd9 A149, probes/t2436/c2974c061384fdd9, excel-graph, 2026-10-24).
Ours: implemented (T-2271's weekend read comes before the holidays' error; T-2583 pins it with a unit test).
Status: VERIFIED (excel-graph), one cell
Tasks: T-2583
