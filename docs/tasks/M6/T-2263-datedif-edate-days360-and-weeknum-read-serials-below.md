---
id: T-2263
title: "DATEDIF, EDATE, DAYS360 and WEEKNUM read serials below 61 on Excel's displayed 1900 calendar (serial 0 is 1900-01-00, 60 is 1900-02-29)"
milestone: M6
lane: functions-date
status: in-progress
priority: P2
size: M
tier: opus
depends_on: []
campaign: visible
---

## Goal
DATEDIF, EDATE, DAYS360 and WEEKNUM give Excel's results for serials below 61: a serial in [0, 1) is 1900-01-00 in DATEDIF, serials 1-60 are read on the displayed calendar (serial 60 is 29 February 1900), DAYS360 refuses a negative end a few ulps below 0, and WEEKNUM(60.5,21.99) is `#VALUE!` once the reason is probed.

## Context
Gap triage of gen-m6 (T-2180, excel-graph oracle, 2026-10-21). Rule and cells: docs/excel-semantics/dates.md "Serials below 61 in DATEDIF,
EDATE, DAYS360, WEEKNUM and YEARFRAC" (added by this triage). These functions read dates through `DateSystem::date_of_serial`
(crates/rustsheet-engine/src/formatter/dates.rs), the weekday-faithful stand-in where serial 1 is 1899-12-31, so every serial
below 61 is a day early and serial 0 is no date (`#NUM!`). `formatter::dates::excel_serial_to_ymd` already gives the displayed
components (DAY/MONTH/YEAR use it). Keep WEEKDAY, WORKDAY and NETWORKDAYS on the weekday-faithful calendar (dates.md "Serial 60 is the
fictitious 1900-02-29", "WORKDAY and NETWORKDAYS from serial 0").
- DATEDIF (26 cells): count Y/M/D/YM/YD/MD on the displayed (year, month, day), with day 0 for [0, 1). Its MD/YD arithmetic is T-2264;
  the early YD cells (b54448af3fde07ca A55, d0c30225243fde60 A81) fit T-2264's rule on the displayed calendar, so land T-2264 first or together.
- EDATE (2): serial 60 is 29 February; moving it keeps day 29 and only a February result clamps (`EDATE(60,0)` = 59 stays, probes/dates-early-1900).
- DAYS360 (2): `DAYS360(60,1)` = -59; a negative serial above -1 (`-5E-15`) is `#NUM!`.
- WEEKNUM (1): `WEEKNUM(60.5,21.99113)` is `#VALUE!`. Probe first (excel-probe skill, excel-graph): `WEEKNUM(60,21)`, `WEEKNUM(60.5,21)`,
  `WEEKNUM(61,21.99)`, `WEEKNUM(45000,21.99)`, `ISOWEEKNUM(60)`, `WEEKNUM(59,21)`, and record the rule.
Probe the open edges too (about 30 cells): DATEDIF "MD"/"YD"/"YM" with start 60 against ends in February and March of leap and non-leap
years, `DAYS360(60,90)`, `DAYS360(59,60)`, `DAYS360(60,60,TRUE)`, `EOMONTH(60.5,1)`, `EDATE(60,12)`, DAYS360 with -1E-15 as start.
Why opus: probing and a calendar change shared by several functions.

Failing cells (`make eval-show CASE=gen-m6/<case>.case.json`), 31 in eval/baseline/gen-m6.txt:

| case / cell | formula | Excel | ours |
|---|---|---|---|
| gen-m6/03333cc38ec06a14 Calc!A128 | `=DATEDIF(3.14159,22195,"MD")` | 3.0 | 4.0 |
| gen-m6/07171fa3f7493f51 Calc!A210 | `=DATEDIF(1,45291,"Y")` | 123.0 | 124.0 |
| gen-m6/0bcac2d96e96b3df Calc!A94 | `=DATEDIF(2,1462,"MD")` | 30.0 | 0.0 |
| gen-m6/0fd242eb9b2456a8 Calc!A28 | `=DATEDIF(2,42303,"MD")` | 24.0 | 25.0 |
| gen-m6/0fd242eb9b2456a8 Calc!A152 | `=EDATE(60.5,5)` | 211.0 | 210.0 |
| gen-m6/0fd242eb9b2456a8 Calc!A180 | `=DATEDIF(LOG10(WEEKDAY(5610,Data!$E11)),16092,"Y")` | 44.0 | "#NUM!" |
| gen-m6/121a7c17092038d9 Calc!A152 | `=DATEDIF((Data!E$12*1.005),7872,"YM")` | 6.0 | "#NUM!" |
| gen-m6/144bceacd894868a Calc!A14 | `=DATEDIF(0.5,60,"D")` | 60.0 | "#NUM!" |
| gen-m6/1a404a0379a1372c Calc!A324 | `=DATEDIF(Data!$C$4,45382,"MD")` | 31.0 | "#NUM!" |
| gen-m6/1aa50086e42bff9a Calc!A183 | `=EDATE(60.5,10)` | 364.0 | 363.0 |
| gen-m6/36d5ce763ea63631 Calc!A380 | `=DAYS360(60,ROUNDDOWN(Data!B$3,5))` | -59.0 | -58.0 |
| gen-m6/3a26794015dfbe9d Calc!A190 | `=DATEDIF(0.5,17471,"Y")` | 47.0 | "#NUM!" |
| gen-m6/44194b2fefa30de2 Calc!A379 | `=DATEDIF(Data!$A4,31233,"MD")` | 5.0 | "#NUM!" |
| gen-m6/50f0dd9a8ee2cb57 Calc!A12 | `=DAYS360(56330,(((-2.5)+(-2.5))/VALUE(Data!D$4)))` | "#NUM!" | -55522.0 |
| gen-m6/752959b8d3db0285 Calc!A372 | `=DATEDIF(SIN(1E+8)%,366,"YM")` | 11.0 | "#NUM!" |
| gen-m6/9843ec1f29523dd2 Calc!A175 | `=DATEDIF(Data!A$10,42578,"Y")` | 116.0 | "#NUM!" |
| gen-m6/9c615d338d990644 Calc!A193 | `=DATEDIF(Data!C2,Data!$A$4,"YD")` | 0.0 | "#NUM!" |
| gen-m6/a1e6448d0590f7c3 Calc!A168 | `=DATEDIF(0.5,44241,"YD")` | 45.0 | "#NUM!" |
| gen-m6/acc70339aac84a23 Calc!A382 | `=DATEDIF(ABS(2.5),37737,"MD")` | 24.0 | 25.0 |
| gen-m6/b54448af3fde07ca Calc!A55 | `=DATEDIF(59.999988426,38168,"YD")` | 123.0 | 124.0 |
| gen-m6/b62dd1ebd266b71c Calc!A241 | `=DATEDIF(0.5,(7^IF(Data!$D4,(-0.5))),"D")` | 0.0 | "#NUM!" |
| gen-m6/c6621490a4d67f4e Calc!A326 | `=DATEDIF(0.5,20361,"YD")` | 273.0 | "#NUM!" |
| gen-m6/c96ac1ada46179c4 Calc!A105 | `=WEEKNUM(60.5,(PRODUCT(Data!D11:D12)+SUMPRODUCT(Data!A3)))` | "#VALUE!" | 9.0 |
| gen-m6/c96ac1ada46179c4 Calc!A312 | `=DATEDIF(SUBTOTAL(6,Data!A2:D9),0.5,"Y")` | 0.0 | "#NUM!" |
| gen-m6/d0c30225243fde60 Calc!A81 | `=DATEDIF(Data!D11,59221,"YD")` | 48.0 | 49.0 |
| gen-m6/d0c30225243fde60 Calc!A271 | `=DATEDIF(SUM(Data!A2:A2,0.1),52156,"Y")` | 142.0 | "#NUM!" |
| gen-m6/d1a403971712513c Calc!A360 | `=DATEDIF(Data!C12,58242,"Y")` | 159.0 | "#NUM!" |
| gen-m6/de083028d8b676d2 Calc!A190 | `=DATEDIF(SERIESSUM(100,2,UNICODE(" x "),SECOND(Data!$B8)),47705,"MD")` | 10.0 | "#NUM!" |
| gen-m6/de083028d8b676d2 Calc!A304 | `=DATEDIF(TIMEVALUE("1:30 PM"),33063,"D")` | 33063.0 | "#NUM!" |
| gen-m6/e3a562557696e2fc Calc!A190 | `=DATEDIF(IF(FALSE,Data!B5)%,37032,"YD")` | 142.0 | "#NUM!" |
| gen-m6/ee56a4ed5a612a0d Calc!A185 | `=DATEDIF(60.5,45291,"MD")` | 2.0 | 3.0 |

## Acceptance criteria
- [ ] These 31 known-failure lines disappear from eval/baseline/gen-m6.txt (`make eval-baseline`), with no new known failure in any suite.
- [ ] A new probes case (excel-graph) holds the edge grid above; every cell passes or is a filed task named in the Outcome.
- [ ] dates.md's entry states the rule per function with its cells and drops the UNVERIFIED notes it settles; probes/dates-early-1900 and -edges still pass.
- [ ] `make check` green; reviewers approve.

## Out of scope
- YEARFRAC (T-2265, T-2237).
- WEEKDAY, WORKDAY and NETWORKDAYS (already weekday-faithful, probed).

## Visible-parity stage
Decision 0083: every cell above is visibly off (a wrong error, a wrong type or a value off in a leading digit). The acceptance criteria above are the stage.
