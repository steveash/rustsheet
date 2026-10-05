---
id: T-0125
title: "Early-1900 serials: format/EDATE/EOMONTH of serial 60 and DAY/MONTH/YEAR(0)"
milestone: M1
lane: engine
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
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
