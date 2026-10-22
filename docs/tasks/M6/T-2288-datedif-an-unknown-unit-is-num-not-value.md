---
id: T-2288
title: "DATEDIF: an unknown unit is #NUM!, not #VALUE!, whatever the dates"
milestone: M6
lane: functions-date
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
campaign: visible
---

## Goal
DATEDIF is `#NUM!` for a unit it does not know (any text other than Y, M, D, YM, YD, MD in any case, a number or a boolean), also for a well-ordered pair of in-calendar dates, as Excel gives; the unit's place in the error order stays as probed.

## Context
Gap triage of gen-m6 (T-2180, excel-graph oracle, 2026-10-21). `datedif_value` (crates/rustsheet-engine/src/functions/date_and_time.rs)
ends its unit match with `_ => Err(Error::VALUE)`. dates.md "DATEDIF and DAYS360 error order with a bad unit or method" already
says an unknown unit is `#NUM!` after a reversed or out-of-calendar pair (probes/datedif-days360-error-order A4, A5, A13, A17); the
gen-m6 cells show it is `#NUM!` for an ordinary pair too (the sentence added there by this triage). A unit error value (`NA()`)
still comes before the dates' `#NUM!` and stays as it is. Four cells have DATEDIF inside another function
(LOOKUP, OFFSET, TRIMRANGE), where its `#NUM!` becomes the outer result; CHOOSECOLS gen-m6/c4edd76f46980c5f A232 also needs T-2273.

Failing cells (`make eval-show CASE=gen-m6/<case>.case.json`), 40 in eval/baseline/gen-m6.txt:

| case / cell | formula | Excel | ours |
|---|---|---|---|
| gen-m6/03333cc38ec06a14 Calc!A318 | `=DATEDIF(27355,51120,(("Hello World"&Data!$D9)&Data!B3))` | "#NUM!" | "#VALUE!" |
| gen-m6/03333cc38ec06a14 Calc!A356 | `=DATEDIF(24355,45382,("a*"&RIGHT(Data!B11)))` | "#NUM!" | "#VALUE!" |
| gen-m6/0bcac2d96e96b3df Calc!A247 | `=TRIMRANGE(Data!B2:E5,(7-DATEDIF(28113,32802,Data!A7)),WORKDAY(5276,-12,Data!A6:C11))` | "#NUM!" | "#VALUE!" |
| gen-m6/0fd242eb9b2456a8 Calc!A104 | `=DATEDIF(22562,26714,ROMAN(1483))` | "#NUM!" | "#VALUE!" |
| gen-m6/121a7c17092038d9 Calc!A114 | `=DATEDIF(LOOKUP((0*Data!E9),Data!B2:B12,Data!A5:A12),57135,IF(NOT(Data!A2),SUBTOTAL(5,Data!B9:E10),(2=Data!...` | "#NUM!" | "#VALUE!" |
| gen-m6/121a7c17092038d9 Calc!A304 | `=DATEDIF(12032,53377,(LOWER(Data!C5)&"1"))` | "#NUM!" | "#VALUE!" |
| gen-m6/1a404a0379a1372c Calc!A20 | `=DATEDIF(19466,73050,Data!$D4)` | "#NUM!" | "#VALUE!" |
| gen-m6/216e3c05f407ca79 Calc!A69 | `=DATEDIF(59.999988426,10801,((Data!C4&"#N/A")&(Data!C$2&Data!C6)))` | "#NUM!" | "#VALUE!" |
| gen-m6/22ce5678fa646215 Calc!A332 | `=DATEDIF(SUM(Data!D9:D9),38858,Data!C10)` | "#NUM!" | "#VALUE!" |
| gen-m6/3516b9fbbd60269b Calc!A201 | `=DATEDIF(24593,34105,(("abc"&Data!D3)&INDIRECT("Data!A1",Data!A5)))` | "#NUM!" | "#VALUE!" |
| gen-m6/3516b9fbbd60269b Calc!A277 | `=DATEDIF(1,14091,(TRIM(CHAR(9))&BAHTTEXT(Data!A4)))` | "#NUM!" | "#VALUE!" |
| gen-m6/38d4bcdafaed1275 Calc!A205 | `=DATEDIF(365,52613,TRIM(LEFT(" x ")))` | "#NUM!" | "#VALUE!" |
| gen-m6/3df034e67b14fccb Calc!A3 | `=DATEDIF(23955,47677,(("Hello World"&Data!A8)&SUBSTITUTE("ABC",Data!B10,Data!D2)))` | "#NUM!" | "#VALUE!" |
| gen-m6/3e51213f35889d06 Calc!A322 | `=DATEDIF(1462,57369,"x~*")` | "#NUM!" | "#VALUE!" |
| gen-m6/4ca39e0876796fa5 Calc!A172 | `=DATEDIF(22476,23757,"")` | "#NUM!" | "#VALUE!" |
| gen-m6/4ca39e0876796fa5 Calc!A286 | `=DATEDIF(9109,29615,LEFT(IFNA(Data!C6,"abc")))` | "#NUM!" | "#VALUE!" |
| gen-m6/4ca39e0876796fa5 Calc!A400 | `=DATEDIF((LEN("=1+1")*SIGN(1000000000000000.0)),12168,"Hello World")` | "#NUM!" | "#VALUE!" |
| gen-m6/50f0dd9a8ee2cb57 Calc!A122 | `=DATEDIF(30203,44926,(IF(Data!E11,Data!B3)&Data!$C8))` | "#NUM!" | "#VALUE!" |
| gen-m6/535caabe1e635e2d Calc!A313 | `=DATEDIF(24241,32087,(("a"&"x~*")&(Data!D3&Data!C8)))` | "#NUM!" | "#VALUE!" |
| gen-m6/7ac4ea222977f3b4 Calc!A44 | `=DATEDIF(3755,23632,(CONCATENATE(Data!A6)&"a"))` | "#NUM!" | "#VALUE!" |
| gen-m6/7ac4ea222977f3b4 Calc!A82 | `=DATEDIF(21657,43349,CHAR(45))` | "#NUM!" | "#VALUE!" |
| gen-m6/870c5e38f51ce5d4 Calc!A24 | `=DATEDIF(1394,55843,(Data!$C6&(Data!$D9&Data!B9)))` | "#NUM!" | "#VALUE!" |
| gen-m6/870c5e38f51ce5d4 Calc!A224 | `=OFFSET(Data!B6:C9,PRODUCT(Data!C8,DATEDIF(7856,36261,Data!C12)),((Data!B12^(-1))/(Data!B6+0.1)),2)` | "#NUM!" | "#VALUE!" |
| gen-m6/9615a6de0f3ac417 Calc!A283 | `=DATEDIF(COMBIN(23,1),17479,("abc"&UNICHAR(507)))` | "#NUM!" | "#VALUE!" |
| gen-m6/9843ec1f29523dd2 Calc!A327 | `=DATEDIF(18856,47084,"")` | "#NUM!" | "#VALUE!" |
| gen-m6/9c615d338d990644 Calc!A117 | `=DATEDIF(38900,53006,LEFTB("",3))` | "#NUM!" | "#VALUE!" |
| gen-m6/9e6a1b733ee1e1e0 Calc!A12 | `=DATEDIF(16934,34631,IFERROR(1/0,AVERAGE(Data!A8)))` | "#NUM!" | "#VALUE!" |
| gen-m6/a165b0a457367d4f Calc!A302 | `=DATEDIF(14132,(DATEDIF(1636,18755,"a")^MONTH(0)),"MD")` | "#NUM!" | "#VALUE!" |
| gen-m6/a1e6448d0590f7c3 Calc!A206 | `=DATEDIF(13659,15878,MID(RIGHT(Data!B2),3,4))` | "#NUM!" | "#VALUE!" |
| gen-m6/acdfeaf6e6eadd4e Calc!A188 | `=OFFSET(Data!D10:E10,0,3,(100/DATEDIF(2061,49302,Data!B12)),2)` | "#NUM!" | "#VALUE!" |
| gen-m6/b54448af3fde07ca Calc!A93 | `=DATEDIF(31,15818,(CONCAT(Data!B5:B11,Data!D$10,Data!A6:A7)&"1"))` | "#NUM!" | "#VALUE!" |
| gen-m6/b62dd1ebd266b71c Calc!A316 | `=LOOKUP(DATEDIF(31,36967,CONCAT(Data!A11:A12,Data!A4:A8)),Data!D10:D11,Data!C8:C8)` | "#NUM!" | "#VALUE!" |
| gen-m6/be0e3a1b0fc3f7a0 Calc!A134 | `=DATEDIF(33177,54911,((Data!B$6&Data!D8)&INDEX(Data!B2:D3,1,Data!C2)))` | "#NUM!" | "#VALUE!" |
| gen-m6/c4edd76f46980c5f Calc!A226 | `=DATEDIF(52170,59928,LEFT((Data!D4&Data!B$12),1))` | "#NUM!" | "#VALUE!" |
| gen-m6/c4edd76f46980c5f Calc!A378 | `=DATEDIF(3661,21828,(SWITCH(1.005,0.1,Data!A11,2)&MID(Data!$B10,1,Data!$E10)))` | "#NUM!" | "#VALUE!" |
| gen-m6/c6621490a4d67f4e Calc!A250 | `=DATEDIF(45187,58960,MID(LOOKUP(Data!C$3,Data!C3:C4,Data!B11:B11),7,8))` | "#NUM!" | "#VALUE!" |
| gen-m6/c96ac1ada46179c4 Calc!A198 | `=DATEDIF(47952,50579," x ")` | "#NUM!" | "#VALUE!" |
| gen-m6/c96ac1ada46179c4 Calc!A236 | `=DATEDIF(41221,44533,"ü")` | "#NUM!" | "#VALUE!" |
| gen-m6/d1a403971712513c Calc!A56 | `=DATEDIF(17229,52166,IF((Data!B9>=Data!A$11),RIGHT("a*"),COUNTIFS(Data!D6:E9,"<>")))` | "#NUM!" | "#VALUE!" |
| gen-m6/ee56a4ed5a612a0d Calc!A299 | `=DATEDIF(60,45000.75,"a*")` | "#NUM!" | "#VALUE!" |

gen-m7 cells (T-2225 triage, 2026-10-21): An unknown DATEDIF unit under other calls: `DATEDIF(1720,27965,"ABC")` in a MAP argument and `DATEDIF(1077,5272,0)` (unit 0) under WORKDAY and DROP are `#NUM!` in Excel, ours `#VALUE!`.

Owned gen-m7 cells (T-2225 triage, excel-graph, 2026-10-21): 2 known-failure lines in eval/baseline/gen-m7.txt, 2 formulas. One line per formula: case, anchor, the anchor formula, Excel vs ours at the first failing cell, then every failing cell (sheet Calc; a range is consecutive cells of one row).

- gen-m7/2c5ea078f25443a2.case.json A1226 `=MAP(DATEDIF(1720,27965,"ABC"),MATCH(0.1,Data!B4:B6,(100^123456789.123)),Data!B3:B4,LAMBDA(v,w,x,Data!E8))`: A1226 Excel #NUM!, ours #VALUE!; cells (1): A1226
- gen-m7/c36ff5f42a1a4d47.case.json A1051 `=DROP(WORKDAY(DATEDIF(1077,5272,Data!A4),-4),-2)`: A1051 Excel #NUM!, ours #VALUE!; cells (1): A1051

## Acceptance criteria
- [x] The 2 gen-m7 cells listed under Context (T-2225 triage) match Excel: their 2 known-failure lines leave eval/baseline/gen-m7.txt (`make eval-baseline`), with no new known failures.
- [x] These 40 known-failure lines disappear from eval/baseline/gen-m6.txt (`make eval-baseline`), with no new known failure in any suite.
- [x] probes/datedif-days360-error-order and probes/date-math-lift-driver-edges still pass; a unit test covers a well-ordered pair with "x", "", a number and TRUE as unit.
- [x] dates.md's new sentence says Ours: implemented (T-2288).
- [x] `make check` green; reviewers approve.

## Out of scope
- DATEDIF's day counts (T-2263, T-2264).

## Visible-parity stage
Decision 0083: every cell above is visibly off (a wrong error, a wrong type or a value off in a leading digit). The acceptance criteria above are the stage.

## Outcome
`datedif_value`'s fallback arm is now `#NUM!` (the day-0 special case folded in). 40 gen-m6 and 2 gen-m7 known failures removed from the baseline; no new failures. Unit test `datedif_unknown_unit_is_num`; dates.md updated.
