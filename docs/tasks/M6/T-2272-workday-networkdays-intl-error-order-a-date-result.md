---
id: T-2272
title: "WORKDAY/NETWORKDAYS(.INTL) error order: a date, result or numeric-holiday #NUM! beats a text or boolean holiday's #VALUE!"
milestone: M6
lane: functions-date
status: in-progress
priority: P2
size: M
tier: opus
depends_on: [T-2270]
campaign: visible
---

## Goal
WORKDAY, WORKDAY.INTL, NETWORKDAYS and NETWORKDAYS.INTL report errors in Excel's order: a `#NUM!` from a date, the result or a numeric holiday beats a text or boolean holiday's `#VALUE!`, the result's `#NUM!` beats a holidays error value, and an all-weekend mask's `#VALUE!` beats a negative holiday's `#NUM!`, backed by a probe.

## Context
Gap triage of gen-m6 (T-2180, excel-graph oracle, 2026-10-21). Cells and observations: docs/excel-semantics/dates.md "WORKDAY and NETWORKDAYS
holidays: blank and number-text cells, and the error order", second bullet (added by this triage). Ours: `holiday_serials`
(functions/date_and_time/holidays.rs) refuses a text or boolean holiday at once with `#VALUE!`, before `workday_of`/`networkdays_of`
check the dates and walk. Most cells pair a holidays range holding text or booleans (generator Data ranges) with a negative or
out-of-calendar date, a negative numeric holiday, an invalid weekend or a result off the calendar. Six cells are the same functions
nested in EOMONTH, MINUTE, LOOKUP, XLOOKUP and XMATCH. XLOOKUP gen-m6/9615a6de0f3ac417 A188 also needs CHOOSEROWS' order (T-2273).
The cells conflict with a simple "holidays first" reading of dates.md "An error-valued or 3-D weekend" (WORKDAY.INTL reads the start,
the days and the holidays first): `WORKDAY.INTL(1,-18,13,ERROR.TYPE("x~*"))` is `#NUM!`, not `#N/A`.
Probe (excel-probe skill, excel-graph, about 50 cells), per function: a holidays range with "a", TRUE, -1 or 3000000, each against a
negative start, an out-of-calendar end, a result before serial 0 or past 2958465, an invalid weekend number and "1111111", and a holidays
`NA()` against the same; with 0 days for WORKDAY. Fold T-1954's open order question in if it is still open. Record the order in dates.md,
then implement it in `holiday_serials` / `workday_call` / `networkdays_of`.
Why opus: the order needs a probe and touches the four functions' shared reading.

Failing cells (`make eval-show CASE=gen-m6/<case>.case.json`), 58 in eval/baseline/gen-m6.txt:

| case / cell | formula | Excel | ours |
|---|---|---|---|
| gen-m6/04fdc9a67474aab9 Calc!A330 | `=WORKDAY.INTL(9625,27,(MAX(Data!B$11,Data!B10:B10,Data!D2:D4)/Data!D1),Data!A8:D8)` | "#NUM!" | "#VALUE!" |
| gen-m6/07171fa3f7493f51 Calc!A77 | `=NETWORKDAYS.INTL(SUM(Data!$D9,Data!D4),1,5,Data!B9:D10)` | "#NUM!" | "#VALUE!" |
| gen-m6/07171fa3f7493f51 Calc!A351 | `=WORKDAY.INTL(36570,SUM(Data!A2:A7),7,Data!A11:B11)` | "#NUM!" | "#VALUE!" |
| gen-m6/0fd242eb9b2456a8 Calc!A284 | `=WORKDAY.INTL(3327,-21,(IFERROR(#N/A,(-0.0001))^DATEVALUE("12/31/1899")),Data!D2:D10)` | "#NUM!" | "#VALUE!" |
| gen-m6/121a7c17092038d9 Calc!A95 | `=WORKDAY(2958465,18,Data!A9:D11)` | "#NUM!" | "#VALUE!" |
| gen-m6/1a404a0379a1372c Calc!A41 | `=WORKDAY.INTL(CEILING((Data!C$9-2),Data!E12),-6,7,Data!A4:A8)` | "#NUM!" | "#VALUE!" |
| gen-m6/1a404a0379a1372c Calc!A199 | `=WORKDAY(0,-23,Data!A5:A5)` | "#NUM!" | "#VALUE!" |
| gen-m6/1a404a0379a1372c Calc!A290 | `=NETWORKDAYS.INTL((-1),COTH(DAYS(75,40159)),16,Data!E7:E9)` | "#NUM!" | "#VALUE!" |
| gen-m6/216e3c05f407ca79 Calc!A108 | `=NETWORKDAYS.INTL(0,COT(ROUNDUP(Data!E12,(-0.5))),13,Data!C8:C12)` | "#NUM!" | "#VALUE!" |
| gen-m6/216e3c05f407ca79 Calc!A312 | `=WORKDAY(10651,-7,Data!B8:D12)` | "#NUM!" | "#VALUE!" |
| gen-m6/22ce5678fa646215 Calc!A120 | `=NETWORKDAYS(MAX(Data!C6:C7,Data!$B$8),(-1),(SUMIFS(Data!D10:D10,Data!D5:D5,"<>",Data!C9:C12,"a*")*INDEX(Da...` | "#NUM!" | "#VALUE!" |
| gen-m6/2ba5bd794e7e8987 Calc!A393 | `=WORKDAY.INTL(14401,-0.3,(FIND(Data!B10,"e"&UNICHAR(769))*(Data!$E$5*Data!E5)),Data!B7:E12)` | "#NUM!" | "#VALUE!" |
| gen-m6/3516b9fbbd60269b Calc!A355 | `=WORKDAY.INTL(10512,3,8,Data!A3:D6)` | "#NUM!" | "#VALUE!" |
| gen-m6/36d5ce763ea63631 Calc!A392 | `=NETWORKDAYS(8303,44989,Data!B9:B12)` | "#NUM!" | "#VALUE!" |
| gen-m6/38d4bcdafaed1275 Calc!A48 | `=MINUTE(COMBINA(NETWORKDAYS.INTL(31779,16033,13,Data!A7:B9),(1e-15/Data!A$3)))` | "#NUM!" | "#VALUE!" |
| gen-m6/38d4bcdafaed1275 Calc!A275 | `=NETWORKDAYS.INTL(49098,31,17,Data!A6:B12)` | "#NUM!" | "#VALUE!" |
| gen-m6/3df034e67b14fccb Calc!A99 | `=NETWORKDAYS((-1),CHOOSE(1,(Data!D6<"?b"),1e-15))` | "#NUM!" | "#VALUE!" |
| gen-m6/3e51213f35889d06 Calc!A40 | `=EOMONTH((NETWORKDAYS(1E+15,45291,Data!A5:D10)-(2-Data!D4)),-10)` | "#NUM!" | "#VALUE!" |
| gen-m6/3e51213f35889d06 Calc!A138 | `=WORKDAY(30512,DATEDIF(34263,DATE(1996,4,Data!C3),"D"),Data!C9:E12)` | "#NUM!" | "#VALUE!" |
| gen-m6/3e51213f35889d06 Calc!A200 | `=NETWORKDAYS(58859,(-1),Data!$E1%)` | "#NUM!" | "#VALUE!" |
| gen-m6/3e51213f35889d06 Calc!A352 | `=NETWORKDAYS(15642,2958466,IF(IF(Data!D10,"ABC"),""))` | "#NUM!" | "#VALUE!" |
| gen-m6/4ca39e0876796fa5 Calc!A66 | `=XMATCH(SUM((Data!$A1^(-0.0001))),Data!B10:B12,WORKDAY(28781,15,Data!A4:D11),SUM(Data!D11:D12,3.14159))` | "#NUM!" | "#VALUE!" |
| gen-m6/4ca39e0876796fa5 Calc!A166 | `=WORKDAY(32112,-10,Data!A3:C12)` | "#NUM!" | "#VALUE!" |
| gen-m6/50f0dd9a8ee2cb57 Calc!A68 | `=WORKDAY(41436,19,Data!C2:D7)` | "#NUM!" | "#VALUE!" |
| gen-m6/535caabe1e635e2d Calc!A255 | `=WORKDAY.INTL(56253,2,0.3,Data!B2:C11)` | "#NUM!" | "#VALUE!" |
| gen-m6/535caabe1e635e2d Calc!A337 | `=NETWORKDAYS(2958466,21782,Data!C3:E6)` | "#NUM!" | "#VALUE!" |
| gen-m6/870c5e38f51ce5d4 Calc!A8 | `=NETWORKDAYS((-1),3214,Data!B5)` | "#NUM!" | "#VALUE!" |
| gen-m6/9130ade4b1924cc8 Calc!A361 | `=WORKDAY.INTL(1,-18,13,ERROR.TYPE(IFERROR("x~*",Data!C$4)))` | "#NUM!" | "#N/A" |
| gen-m6/9615a6de0f3ac417 Calc!A188 | `=XLOOKUP(CHOOSEROWS(Data!$C8,-3,NETWORKDAYS.INTL(42381,1E+15,14,Data!C11:E12),123456789.123),Data!E11:E12,D...` | "#NUM!" | "#VALUE!" |
| gen-m6/9615a6de0f3ac417 Calc!A270 | `=WORKDAY.INTL(56712,18,((100+Data!$C1)--0.2),Data!A8:C9)` | "#NUM!" | "#VALUE!" |
| gen-m6/9615a6de0f3ac417 Calc!A346 | `=WORKDAY.INTL(LEN("=1+1"),-2,10,Data!B4:C4)` | "#NUM!" | "#VALUE!" |
| gen-m6/9843ec1f29523dd2 Calc!A331 | `=WORKDAY.INTL(42,-10,18,Data!C9:E9)` | "#NUM!" | "#VALUE!" |
| gen-m6/a165b0a457367d4f Calc!A171 | `=NETWORKDAYS(10840,(-0.0001),Data!B7:B7)` | "#NUM!" | "#VALUE!" |
| gen-m6/acdfeaf6e6eadd4e Calc!A376 | `=NETWORKDAYS(3042,LOOKUP(1000000000000000.0,Data!C3:C7),Data!B6:C11)` | "#NUM!" | "#VALUE!" |
| gen-m6/b62dd1ebd266b71c Calc!A35 | `=NETWORKDAYS(Data!E$10,(MAX(Data!A9:A10,Data!A4)-Data!B$1),WEEKDAY(CELL(Data!D10,Data!B10)))` | "#NUM!" | "#VALUE!" |
| gen-m6/be0e3a1b0fc3f7a0 Calc!A156 | `=WORKDAY.INTL(24256,IFERROR(SUM(Data!A1:A5,Data!A$7),(Data!D7+(-0.5))),4,Data!A10:C12)` | "#NUM!" | "#VALUE!" |
| gen-m6/be0e3a1b0fc3f7a0 Calc!A382 | `=NETWORKDAYS.INTL(SUBTOTAL(5,Data!B7:B10),(-1),14,SUM((Data!D$7/Data!A3),Data!A1:A8))` | "#NUM!" | "#VALUE!" |
| gen-m6/bf63ed6c6f592475 Calc!A49 | `=NETWORKDAYS((-1),ABS(Data!C7),Data!A7:D10)` | "#NUM!" | "#VALUE!" |
| gen-m6/bf63ed6c6f592475 Calc!A277 | `=NETWORKDAYS(Data!E7,36397,Data!A3:B12)` | "#NUM!" | "#VALUE!" |
| gen-m6/bf63ed6c6f592475 Calc!A309 | `=WORKDAY.INTL(14250,2,9,Data!B5:D12)` | "#NUM!" | "#VALUE!" |
| gen-m6/c4edd76f46980c5f Calc!A31 | `=WORKDAY.INTL(Data!D8,29,9,Data!A8:D12)` | "#NUM!" | "#VALUE!" |
| gen-m6/c4edd76f46980c5f Calc!A183 | `=WORKDAY.INTL(POWER(CEILING(Data!B7,Data!A$6),0.2),12,8,Data!B9:C10)` | "#NUM!" | "#VALUE!" |
| gen-m6/c4edd76f46980c5f Calc!A278 | `=NETWORKDAYS.INTL(SUM(Data!D8:D10,Data!A1:A5,((-1)/Data!C7)),3537,16,TANH(Data!$C$9%))` | "#NUM!" | "#VALUE!" |
| gen-m6/c4edd76f46980c5f Calc!A353 | `=NETWORKDAYS(2958466,48983,Data!D5:E9)` | "#NUM!" | "#VALUE!" |
| gen-m6/c6621490a4d67f4e Calc!A139 | `=LOOKUP((INDEX(Data!B8:E12,1,Data!$D10)+NETWORKDAYS(50614,(-0.0001),Data!A9:D10)),Data!B3:B5)` | "#NUM!" | "#VALUE!" |
| gen-m6/c93ea588c271840e Calc!A128 | `=WORKDAY(24221,SUM(Data!B2:B7),Data!A9:A12)` | "#NUM!" | "#VALUE!" |
| gen-m6/c96ac1ada46179c4 Calc!A234 | `=WORKDAY((ODD(123456789.123)*OFFSET(Data!D9:E11,3,3,1,0.3)),-13,SUMPRODUCT(Data!C4:E12,Data!A11))` | "#NUM!" | "#VALUE!" |
| gen-m6/c96ac1ada46179c4 Calc!A362 | `=NETWORKDAYS((SUM(Data!A2:A5,Data!B6:B12,Data!C11:C12)-1000000000000000.0),44727,Data!B11:E11)` | "#NUM!" | "#VALUE!" |
| gen-m6/d0c30225243fde60 Calc!A66 | `=WORKDAY(0.2,-12,(IF(FALSE,Data!C8)^Data!E6))` | "#NUM!" | "#VALUE!" |
| gen-m6/d0c30225243fde60 Calc!A99 | `=NETWORKDAYS.INTL(((2+Data!A10)+(-2.5)),35805,1,Data!B2:E8)` | "#NUM!" | "#VALUE!" |
| gen-m6/d0c30225243fde60 Calc!A332 | `=WORKDAY(1462,NETWORKDAYS.INTL(21128,12887,"1000001"),((-2.5)/SUMPRODUCT(Data!$E$8)))` | "#NUM!" | "#DIV/0!" |
| gen-m6/d183b62596b8e174 Calc!A351 | `=XMATCH((MONTH(57321)<WORKDAY.INTL(Data!E2,-19,9,Data!B3:E9)),Data!D5:D11)` | "#NUM!" | "#VALUE!" |
| gen-m6/d1a403971712513c Calc!A89 | `=WORKDAY.INTL(1,-20,14,Data!B1:D12)` | "#NUM!" | "#VALUE!" |
| gen-m6/ee56a4ed5a612a0d Calc!A56 | `=NETWORKDAYS(WORKDAY.INTL(Data!D7,-27,Data!B2,Data!B6:E10),27706,Data!B9:C12)` | "#NUM!" | "#VALUE!" |
| gen-m6/ee56a4ed5a612a0d Calc!A395 | `=WORKDAY(Data!C$11,-19,Data!E6:E8)` | "#NUM!" | "#VALUE!" |
| gen-m6/ef10bf3afb9658f6 Calc!A346 | `=WORKDAY.INTL(45382,10,"1111111",Data!B2:C5)` | "#VALUE!" | "#NUM!" |
| gen-m6/fde83d52a9d1fc63 Calc!A178 | `=NETWORKDAYS.INTL(1E+15,FLOOR(MOD(Data!A6,Data!D12),(0.5+Data!$B11)),12,Data!B$4)` | "#NUM!" | "#VALUE!" |
| gen-m6/fde83d52a9d1fc63 Calc!A181 | `=WORKDAY.INTL(LOOKUP(NETWORKDAYS((-1),45351,Data!B10:D11),Data!E6:E8,Data!B7:B11),INDEX(Data!B1:D9,AVERAGE(...` | "#NUM!" | "#VALUE!" |

## Acceptance criteria
- [ ] These 58 known-failure lines disappear from eval/baseline/gen-m6.txt (`make eval-baseline`), with no new known failure in any suite (gen-m6/9615a6de0f3ac417 A188 may wait for T-2273; say so in the Outcome).
- [ ] A new probes case (excel-graph) holds the order grid; every cell passes or is a filed task named in the Outcome.
- [ ] dates.md states the order with cells; probes/workday-*, probes/date-text-weekend-3d and probes/workday-serial-zero still pass.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Blank and number-text holiday cells (T-2270).
- 3-D holidays (T-2238).

## Visible-parity stage
Decision 0083: every cell above is visibly off (a wrong error, a wrong type or a value off in a leading digit). The acceptance criteria above are the stage.
