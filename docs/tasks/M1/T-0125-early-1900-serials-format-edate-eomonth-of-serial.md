---
id: T-0125
title: "Early-1900 serials: format/EDATE/EOMONTH of serial 60 and DAY/MONTH/YEAR(0)"
milestone: M1
lane: engine
status: blocked
priority: P2
size: S
tier: opus
depends_on: [T-0213]
---

## Goal
The four known failures in probes/dates-early-1900 (A9, A10, A12, A16) pass: `EOMONTH(31,1)` and `EDATE(31,1)` (59), `TEXT(0,"m/d/yyyy")` ("1/0/1900"), `WEEKNUM(1)` (1); DAY/MONTH/YEAR(0) also match Excel. Probe and fix as well: `EOMONTH(59,0)` (likely 59), `EOMONTH(1,-1)` (likely #NUM!), `TEXT(DATE(1900,1,0),"d")` ("0").

## Context
T-0108 fixed DATE, DAY/MONTH/YEAR and number formatting for serials 1..60 but left `from_excel_date` as a weekday-faithful stand-in calendar (see docs/excel-semantics/dates.md). Oracle values are already in probes/dates-early-1900.

## Acceptance criteria
- [ ] The 43 gen-m2 DAY/MONTH/YEAR cells of serials in [0,1) and FALSE leave eval/baseline/gen-m2.txt (T-0203 triage): 1b6ceed7489e0800 A44 A61 A400; 207a5753da48c7e0 A94; 2cfe279fe77a000d A139 A144; 60a543744fc4a8f7 A209; 61cacc22c4244179 A227; 629e6e8b76fab8b9 A95; 7469c3e920e1ade4 A45 A81 A215 A334; 75840b9c71a50596 A75 A257 A279; 942bdade381b7537 A267; 96a0952412cefa6c A8 A35; a339ea84ef64382e A54 A129 A139 A279; b4392a6514a36ade A182 A319 A392 A398; caa981b0753f455c A157; cbdabe32a32a4012 A159 A251 A378; d04d501154afc771 A382; d3e2df0bef2f173a A52; dd99be1f2454159f A11; e8493fafa2a5378e A66 A173 A331; ef5c55a81783b7d5 A11 A15; f0d1bacec5de8146 A145; fa437879253827be A41 A158 (Calc sheet; 207a5753da48c7e0 A333, dd99be1f2454159f A114, f7e6d351c8538ef5 A329 also need T-0209).
- [ ] The probe cells above leave eval/baseline/probes.txt with no regressions.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Handoff (resumed)
Branch has the implementation (commit "early-1900 serials in EDATE/EOMONTH/WEEKNUM/TEXT/DAY-MONTH-YEAR"):
`excel_serial_to_ymd` accepts serial 0, new `month_arithmetic_date`/`real_date_to_serial` in
`formatter/dates.rs` used by EDATE/EOMONTH, WEEKNUM year fix, TEXT uses ymd validity; probe
`dates-early-1900` extended (A17-A25, oracle filled) and passes 25/25; semantics note added.
`make eval-check` shows 41 gen-m2 cells fixed but 2 REGRESSIONS that passed only by accident
(DAY/YEAR(0) used to be #NUM!, hiding other engine bugs):
- gen-m2/a339ea84ef64382e Calc!A138 `=DAY(ROUNDDOWN(123456789.123,ROUNDUP(1,-5)))` expected #NUM!, now 0:
  the engine's ROUNDDOWN with huge digits (100000) returns 0; Excel evidently returns the value
  unchanged (DAY of 123456789 is #NUM!). Fix ROUNDDOWN/ROUNDUP/ROUND with out-of-range digits (probe first).
- gen-m2/a6d4377c7b1b92cd Calc!A278 `=IFERROR(YEAR(INDEX(Data!C11:E12,1)),-SUM(Data!C3:C5))` expected 1:
  my reading is that the legacy (non-spilling) formula implicit-intersects the INDEX row range
  and gives #VALUE!, so IFERROR falls back; the engine computes YEAR of the first element (1900).
  Needs an implicit-intersection probe.
Next: fix or probe those two (or split them to their own tasks and fix first), then
`make eval-baseline`, `make check`, review.

## Blocked because
Waits for T-0213 (INDEX with one index into a 2-D range is #REF!). After the opus resume
(2026-10-05), the branch fixes 46 gen-m2 cells and 4 probe cells. One regression is left:
- `gen-m2/a6d4377c7b1b92cd` Calc!A278 `=IFERROR(YEAR(INDEX(Data!C11:E12,1)),-SUM(Data!C3:C5))`
  expects 1. This is not implicit intersection, as the handoff guessed: in Excel,
  `INDEX(C11:E12,1)` is #REF! (T-0213; A17 of the same case), so IFERROR falls back. The engine
  returns C11 (blank). YEAR of blank used to be #NUM!, which made the cell pass by accident; it is
  now 1900.

The handoff's other regression (A138, ROUNDDOWN with 100000 digits) is fixed on this branch by
`round_beyond_double_range`, a partial T-0218 fix limited to digits whose magnitude is below
2^31. To resume once T-0213 is merged: merge main, run `make eval-check` (expect 0 regressions),
`make eval-baseline` (probes also reports one new-coverage failure, from the dates-early-1900
case growing from 16 to 25 cells; check it), then `make check` and review.
