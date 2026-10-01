# Dates (1900 date system)

## Serial 60 is the fictitious 1900-02-29
Rule (hypothesis, well known): Excel treats 1900 as a leap year for compatibility with Lotus 1-2-3.
`DATE(1900,2,29)` = 60 and `DATE(1900,3,1)` = 61. IronCalc returns 61 for `DATE(1900,2,29)`.
Evidence: probes/dates-and-blanks.case.json A1–A3 (pending).
Status: UNVERIFIED (probe pending)
