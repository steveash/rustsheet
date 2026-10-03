# Dates (1900 date system)

## Serial 60 is the fictitious 1900-02-29
Rule (hypothesis, well known): Excel treats 1900 as a leap year for compatibility with Lotus 1-2-3.
`DATE(1900,2,29)` = 60 and `DATE(1900,3,1)` = 61. IronCalc returns 61 for `DATE(1900,2,29)`.
Evidence: probes/dates-and-blanks.case.json A1–A3 (pending).
Status: UNVERIFIED (probe pending)

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
