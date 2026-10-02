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
