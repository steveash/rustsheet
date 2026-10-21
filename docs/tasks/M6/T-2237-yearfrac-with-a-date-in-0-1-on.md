---
id: T-2237
title: "YEARFRAC with a date in [0, 1) on bases 1-4 and next to January-February 1900: probe and match (today #NUM! until probed)"
milestone: M6
lane: functions-date
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
YEARFRAC gives Excel's value or error when its start or end date is in [0, 1) (serial 0,
1900-01-00) on every basis, and when that date is paired with a date in January or February 1900,
including the fictitious serial 60. Both cases are backed by oracle evidence. Today the engine
returns `#NUM!` for bases 1-4 only because nothing was probed.

## Context
Gap triage 2026-10-21. YEARFRAC is an M6 function. One generated cell showed that Excel accepts
serial 0 on basis 0: gen-m9/a2e51f48717af2a1 Calc!A32,
`=((0.3+YEARFRAC(33752,Data!$C12))^SUM((-0.0001)))` with Data!C12 = 0, is 0.9995471537332011, which
(0.3+92.41111111111111)^-0.0001 reproduces bit for bit: `YEARFRAC(33752,0)` is derived as 92.41111111111,
counted from 1900-01-00 as (92*360 + 4*30 + 28)/360 (T-1409). Everything around that cell is
still a guess:
- `functions/date_and_time/yearfrac.rs` `year_fraction_parts` says that a serial in [0, 1) is "a
  date to the 30/360 day count of basis 0 only ... other bases keep #NUM! until probed". It cites
  dates.md "YEARFRAC accepts serial 0", but no section has that name. The note sits in "WEEKDAY of
  serial 0 and return_type 0".
- dates.md there: "other bases with serial 0 stay `#NUM!`, UNVERIFIED; ... serial 0 paired with a
  date in January-February 1900, including the fictitious serial 60, is UNVERIFIED".
- WORKDAY and NETWORKDAYS take serial 0 on a weekday-faithful calendar (dates.md "WORKDAY and
  NETWORKDAYS from serial 0", probes/workday-serial-zero). So serial 0 being a date in YEARFRAC's
  other bases is plausible, but its day counts (actual/actual, actual/360, actual/365, European
  30/360) cannot be inferred.

Probe (excel-probe skill, excel-graph oracle; about 50 cells) `YEARFRAC(s,e,b)` for b in 0..4:
- s in {0, 0.5, 0.9999}, with e in {1, 31, 59, 60, 61, 366, 33752, 45000};
- the reversed pairs;
- `YEARFRAC(0,0,b)`;
- each basis with s = 60 against e in {1, 59, 61}.

Add a 1904-workbook twin for a few cells: `financial_1900_serial` reads a 1904 serial 0 as 1900-serial 2924,
so it never reaches the [0, 1) branch; the twin checks that the 1904 shift is unaffected
(`YEARFRAC(0,59,0)` = 59/360 is already probed, date-system-1904-financial-shift). Record the rules in dates.md as a section named "YEARFRAC accepts
serial 0", or fix the code comment's reference. Then implement them in `year_fraction_parts`.
The discount-security functions also call `year_fraction_parts` (financial.md "Discount securities:
day count and year length"). Check that no financial cell moves, or measure and list the ones that do.

gen-m6 triage (T-2180, excel-graph oracle, 2026-10-21): YEARFRAC with a start or end in [0, 1) on bases 1-4 (blank cells, 0.3, 0.5, MINUTE(SECOND(<blank>)), SECH(1E+16) ...): Excel gives a number on every basis, ours `#NUM!`. These cells are evidence for the probe's grid (basis 1 `YEARFRAC(49382,0,1)` = 135.20326938175668, basis 2 `YEARFRAC(0,42305,2)` = 117.51388888888889, basis 3 `YEARFRAC(36821,0.3,3)` = 100.87945205479453, basis 4 `YEARFRAC(0.5,366,4)` = 1). One cell is YEARFRAC inside TIME (04fdc9a67474aab9 A315, basis 4). YEARFRAC of serials 1-60 is T-2265.

Failing cells (`make eval-show CASE=gen-m6/<case>.case.json`), 22 in eval/baseline/gen-m6.txt:

| case / cell | formula | Excel | ours |
|---|---|---|---|
| gen-m6/03333cc38ec06a14 Calc!A175 | `=YEARFRAC(INDIRECT("Data!A1"),46914,1)` | 128.44421334125687 | "#NUM!" |
| gen-m6/04fdc9a67474aab9 Calc!A35 | `=YEARFRAC(7011,Data!E6,2)` | 19.475 | "#NUM!" |
| gen-m6/04fdc9a67474aab9 Calc!A315 | `=TIME(2,YEARFRAC((Data!D3*Data!A$12),14513,4),26)` | 0.11071759259259259 | "#NUM!" |
| gen-m6/121a7c17092038d9 Calc!A61 | `=YEARFRAC(52878,0,AVERAGE((Data!A$8*Data!C$7),Data!E5,INDIRECT("Data!Z99",Data!$B2)))` | 144.7727573119843 | "#NUM!" |
| gen-m6/38d4bcdafaed1275 Calc!A356 | `=YEARFRAC(10506,MINUTE(SECOND(Data!E$6)),2)` | 29.183333333333334 | "#NUM!" |
| gen-m6/44194b2fefa30de2 Calc!A42 | `=YEARFRAC(13963,SECH(1E+16),3)` | 38.25479452054795 | "#NUM!" |
| gen-m6/4ca39e0876796fa5 Calc!A118 | `=YEARFRAC(36821,0.3,3)` | 100.87945205479453 | "#NUM!" |
| gen-m6/6684de07ac74bcb8 Calc!A368 | `=YEARFRAC(0,42305,2)` | 117.51388888888889 | "#NUM!" |
| gen-m6/7ac4ea222977f3b4 Calc!A225 | `=YEARFRAC(Data!$E6,(CODE(UNICHAR(12288))-(Data!$A2+Data!C$2)),2)` | 0.16944444444444445 | "#NUM!" |
| gen-m6/9130ade4b1924cc8 Calc!A250 | `=YEARFRAC(43155,(AREAS(Data!A8:C12)/Data!B3),3)` | 118.23287671232876 | "#NUM!" |
| gen-m6/9615a6de0f3ac417 Calc!A7 | `=YEARFRAC(26267,0.5,4)` | 71.91666666666667 | "#NUM!" |
| gen-m6/9615a6de0f3ac417 Calc!A235 | `=YEARFRAC(POWER(2/3,Data!$D$9),41949,4)` | 114.85 | "#NUM!" |
| gen-m6/9c615d338d990644 Calc!A123 | `=YEARFRAC(49382,0,1)` | 135.20326938175668 | "#NUM!" |
| gen-m6/9c615d338d990644 Calc!A161 | `=YEARFRAC(45276,0,1)` | 123.96166924265842 | "#NUM!" |
| gen-m6/acdfeaf6e6eadd4e Calc!A111 | `=YEARFRAC(0.5,28080,2)` | 78.0 | "#NUM!" |
| gen-m6/b62dd1ebd266b71c Calc!A271 | `=YEARFRAC(SECOND(INDIRECT("Data!Z99")),18137,3)` | 49.69041095890411 | "#NUM!" |
| gen-m6/bf63ed6c6f592475 Calc!A90 | `=YEARFRAC(0.5,366,4)` | 1.0 | "#NUM!" |
| gen-m6/d183b62596b8e174 Calc!A238 | `=YEARFRAC(21576,ROUND(1e-15,-1),1)` | 59.074564205530706 | "#NUM!" |
| gen-m6/d183b62596b8e174 Calc!A314 | `=YEARFRAC(COUNTIF(Data!A6:C7,1),56133,4)` | 153.68333333333334 | "#NUM!" |
| gen-m6/da0289bc74cdf6af Calc!A308 | `=YEARFRAC(22926,0.3,3)` | 62.81095890410959 | "#NUM!" |
| gen-m6/da0289bc74cdf6af Calc!A346 | `=YEARFRAC(55825,0,1)` | 152.84120394395433 | "#NUM!" |
| gen-m6/ee56a4ed5a612a0d Calc!A60 | `=YEARFRAC(32421,SUMIF(Data!E1:E12,"*c",Data!B11:E11),3)` | 88.82465753424657 | "#NUM!" |

## Acceptance criteria
- [ ] The 22 gen-m6 known-failure lines above disappear from eval/baseline/gen-m6.txt (`make eval-baseline`), or each remaining one is named in the Outcome with its cause.
- [ ] A new probes case, oracle-filled (excel-graph), covers the grid above. Every cell passes, or
      its known failure is recorded with `make eval-baseline` and has a filed task named in the Outcome.
- [ ] dates.md states the rule for each basis with its cells, and the UNVERIFIED notes above are gone.
- [ ] Unit tests in `test/test_yearfrac_basis.rs` cover each basis's serial-0 rule. No gated baseline
      gains a line, and no financial cell changes, or each change is listed in the Outcome.
- [ ] `make check` green; reviewers approve.

## Out of scope
- DAYS360, COUP* and the bond functions at serial 0 (probe separately if YEARFRAC's rule suggests
  they differ).
- Negative serials (still `#NUM!`).
