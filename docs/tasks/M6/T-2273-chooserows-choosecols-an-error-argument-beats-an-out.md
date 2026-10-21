---
id: T-2273
title: "CHOOSEROWS/CHOOSECOLS: an error argument beats an out-of-range index's #VALUE!"
milestone: M6
lane: functions-lookup
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
campaign: visible
---

## Goal
CHOOSEROWS and CHOOSECOLS return an error value found in any argument (the array or any index) before the `#VALUE!` of a 0 or out-of-range index, as Excel does.

## Context
Gap triage of gen-m6 (T-2180, excel-graph oracle, 2026-10-21). Rule and cells: docs/excel-semantics/lookup.md "CHOOSEROWS and CHOOSECOLS: an
error argument beats an out-of-range index" (added by this triage). `Model::chosen_indices`
(crates/rustsheet-engine/src/functions/lookup_and_reference/choosecols_chooserows.rs) stops at the first problem in argument order. Scan
all arguments for an error value first, then validate the indexes. Which of two errors wins is not decided by the cells: keep the
first in argument order and say so in lookup.md (UNVERIFIED).
Three cells need an inner fix as well: c4edd76f46980c5f A232 (DATEDIF unknown unit, T-2262), da0289bc74cdf6af A195 (TIME past the bound,
T-2267), 7f3606fef4cb9df7 A111 (EXP(999999999999999) must be `#NUM!`, T-2169).

Failing cells (`make eval-show CASE=gen-m6/<case>.case.json`), 26 in eval/baseline/gen-m6.txt:

| case / cell | formula | Excel | ours |
|---|---|---|---|
| gen-m6/07171fa3f7493f51 Calc!A333 | `=CHOOSEROWS(Data!A7:B8,4,FLOOR(IFNA(Data!E8,100),((-0.0001)-1e-15)),1)` | "#NUM!" | "#VALUE!" |
| gen-m6/121a7c17092038d9 Calc!A298 | `=CHOOSECOLS(Data!A1,1000000000000000.0,ACOSH((Data!$A4-Data!E$12)))` | "#NUM!" | "#VALUE!" |
| gen-m6/216e3c05f407ca79 Calc!A105 | `=CHOOSEROWS(SUBTOTAL(4,Data!B5:C5,Data!B10:D12,Data!C8:C11),6,INDEX(Data!D7:E9,100%,MAX(42,Data!D11:D11,Dat...` | "#REF!" | "#VALUE!" |
| gen-m6/216e3c05f407ca79 Calc!A333 | `=CHOOSEROWS(Data!B10:B10,0,OFFSET(Data!A9:C11,6,-1,1),-1)` | "#REF!" | "#VALUE!" |
| gen-m6/22ce5678fa646215 Calc!A57 | `=CHOOSEROWS((SUMIF(Data!A5:D5,"<>")*(Data!E5+Data!D1)),-3,-2,IF(VLOOKUP(Data!$E3,Data!B6:E6,3,Data!E9),(Dat...` | "#N/A" | "#VALUE!" |
| gen-m6/3516b9fbbd60269b Calc!A224 | `=CHOOSECOLS(Data!B8:B12,DAYS(59.999988426,367),3,AGGREGATE(VLOOKUP(Data!E$2,Data!C10:D10,3,FALSE),3,Data!A4...` | "#N/A" | "#VALUE!" |
| gen-m6/3516b9fbbd60269b Calc!A376 | `=CHOOSECOLS(DAYS(ROUNDDOWN(Data!D1,1),49802),1,3,INDEX(Data!A5:A5,5))` | "#REF!" | "#VALUE!" |
| gen-m6/38d4bcdafaed1275 Calc!A261 | `=CHOOSECOLS(IFERROR((Data!D7-Data!D$2),"ü"),2,0,DATEDIF(1E+15,53907,Data!E6))` | "#NUM!" | "#VALUE!" |
| gen-m6/4ca39e0876796fa5 Calc!A70 | `=CHOOSEROWS(Data!B6:D6,-3,((Data!C3/Data!E2)/HLOOKUP(Data!$E8,Data!C2:E3,3,Data!$B3)),IFERROR(COUNTIFS(Data...` | "#DIV/0!" | "#VALUE!" |
| gen-m6/4ca39e0876796fa5 Calc!A98 | `=CHOOSECOLS("TRUE",((Data!$B$11/Data!D10)-IFNA(Data!C9,"abc")),HLOOKUP((Data!B$12/Data!D8),Data!A5:B9,3,FAL...` | "#N/A" | "#VALUE!" |
| gen-m6/50f0dd9a8ee2cb57 Calc!A375 | `=CHOOSECOLS("1",(ROUNDDOWN(1+2^-52,-1)+EVEN(Data!A5)),IFNA(Data!$E11,ISBLANK(#DIV/0!)),ROUNDUP(AVERAGE(Data...` | "#DIV/0!" | "#VALUE!" |
| gen-m6/535caabe1e635e2d Calc!A136 | `=CHOOSEROWS(CHOOSEROWS(Data!D11:E12,1),Data!A7,6,(Data!A$4-FLOOR(7,0)))` | "#DIV/0!" | "#VALUE!" |
| gen-m6/535caabe1e635e2d Calc!A364 | `=CHOOSEROWS(IF(Data!$A9,(0<=Data!A5),"ABC"),Data!D9,(LOOKUP(Data!$D2,Data!E2:E5)/Data!B4))` | "#N/A" | "#VALUE!" |
| gen-m6/60bcd25271767317 Calc!A312 | `=CHOOSEROWS(TAN(MIN(Data!$E4)),5,3.14159,(INDEX(Data!E9:E10,3)^(Data!$C$10/Data!C11)))` | "#REF!" | "#VALUE!" |
| gen-m6/752959b8d3db0285 Calc!A85 | `=CHOOSECOLS(" x ",AGGREGATE(1,IFERROR(#NUM!,Data!B3),Data!A10:C12,Data!C9:C9,Data!D9:E10),(INDIRECT("Data!E...` | "#REF!" | "#VALUE!" |
| gen-m6/7ac4ea222977f3b4 Calc!A391 | `=CHOOSEROWS(Data!B11:E11,2,-1,((123456789.123*Data!B9)+SUMX2MY2(Data!A8:B10,Data!B10:E12)))` | "#N/A" | "#VALUE!" |
| gen-m6/7f3606fef4cb9df7 Calc!A111 | `=CHOOSEROWS(MIN(1e-15,(Data!D$10*Data!B1)),1,2.5,EXP(999999999999999))` | "#NUM!" | "#VALUE!" |
| gen-m6/9130ade4b1924cc8 Calc!A260 | `=CHOOSEROWS(Data!C11:D12,-3,2,(VLOOKUP("abc",Data!B11:E11,4,Data!B8)+(Data!D5/Data!B$10)))` | "#N/A" | "#VALUE!" |
| gen-m6/9c615d338d990644 Calc!A254 | `=CHOOSEROWS(DAY(26649),2,MATCH(123456789.123,Data!A9:D9,(1000000000000000.0/Data!$E$3)))` | "#DIV/0!" | "#VALUE!" |
| gen-m6/c4edd76f46980c5f Calc!A232 | `=CHOOSECOLS(Data!C$11,DATE(1968,8,10),DATEDIF(12816,29202,(" x "&Data!D11)),ACOTH((123456789.123-2)))` | "#NUM!" | "#VALUE!" |
| gen-m6/c6621490a4d67f4e Calc!A57 | `=CHOOSECOLS(Data!C6:C10,-(Data!$D12*0.1),-2,(SUM(Data!D9:D11,1000000000000000.0)*LOOKUP("ABC",Data!C6:C10)))` | "#N/A" | "#VALUE!" |
| gen-m6/d0c30225243fde60 Calc!A79 | `=CHOOSEROWS(Data!A10:C10,Data!C12,LOG(IF(FALSE,Data!D2),(-0.5)))` | "#NUM!" | "#VALUE!" |
| gen-m6/d0c30225243fde60 Calc!A400 | `=CHOOSECOLS(Data!A5,1,-2,VLOOKUP(("Hello World"="x~*"),Data!B11:E12,2))` | "#N/A" | "#VALUE!" |
| gen-m6/da0289bc74cdf6af Calc!A195 | `=CHOOSEROWS((IFERROR("a",Data!D1)&(Data!B$4&Data!C9)),6,(TIME(Data!A12,44,1000000000000000.0)*1),4)` | "#NUM!" | "#VALUE!" |
| gen-m6/de083028d8b676d2 Calc!A244 | `=CHOOSEROWS(Data!A6:A10,0,NETWORKDAYS(43981,19954,-Data!E11),-2)` | "#NUM!" | "#VALUE!" |
| gen-m6/ee56a4ed5a612a0d Calc!A19 | `=CHOOSECOLS(SQRT(2958465),2,-2,INDEX(Data!A8:D12,6,2))` | "#REF!" | "#VALUE!" |

## Acceptance criteria
- [ ] The 23 known-failure lines without an inner dependency disappear from eval/baseline/gen-m6.txt (`make eval-baseline`), with no new known failure in any suite; the other 3 disappear here or are named in the Outcome with the task they wait for.
- [ ] Unit tests cover `CHOOSEROWS(A1:B2,9,NA())` (`#N/A`) and `CHOOSECOLS(A1:B2,0,1/0)` (`#DIV/0!`).
- [ ] lookup.md's entry says Ours: implemented (T-2273).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The order between two errors (unprobed).

## Visible-parity stage
Decision 0083: every cell above is visibly off (a wrong error, a wrong type or a value off in a leading digit). The acceptance criteria above are the stage.
