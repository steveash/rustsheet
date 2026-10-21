---
id: T-2274
title: "ADDRESS: an error in any argument beats ADDRESS's own #VALUE!, and a TRUE/FALSE sheet_text is quoted"
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
ADDRESS returns an error value found in any argument before its own `#VALUE!` for a bad row, column or abs_num, and quotes a sheet_text of "TRUE" or "FALSE", as Excel does.

## Context
Gap triage of gen-m6 (T-2180, excel-graph oracle, 2026-10-21). Rule and cells: docs/excel-semantics/lookup.md "ADDRESS: argument errors first,
sheet_text "TRUE"/"FALSE" quoted, bounds", first two bullets (added by this triage). `address_of`
(crates/rustsheet-engine/src/functions/lookup_and_reference/address_areas.rs) checks the arguments left to right and returns the row's
`#VALUE!` before reading a later error; `address_sheet_text` leaves "TRUE"/"FALSE" bare. Look for an existing quoting helper (the
formula printer quotes sheet names) before adding one; check whether other boolean-like or cell-like names (`"A1"`, `"R1C1"`) are already
quoted. 04fdc9a67474aab9 A326 also needs TIME's bound (T-2267).

Failing cells (`make eval-show CASE=gen-m6/<case>.case.json`), 17 in eval/baseline/gen-m6.txt:

| case / cell | formula | Excel | ours |
|---|---|---|---|
| gen-m6/04fdc9a67474aab9 Calc!A326 | `=ADDRESS((Data!$E6*(1-Data!B9)),TIME(22,FLOOR(1000000000000000.0,Data!$B$3),PRODUCT(Data!E9)))` | "#NUM!" | "#VALUE!" |
| gen-m6/07171fa3f7493f51 Calc!A245 | `=ADDRESS(18,0.1,-IF(Data!A2,Data!E6,Data!$E$3),(MINUTE((-0.5))<ROUNDDOWN(Data!D7,3)),(IF(TRUE,3.14159,"a*")...` | "#NUM!" | "#VALUE!" |
| gen-m6/0fd242eb9b2456a8 Calc!A280 | `=ADDRESS((-1),3.14159,2,NA())` | "#N/A" | "#VALUE!" |
| gen-m6/2ba5bd794e7e8987 Calc!A133 | `=ADDRESS(1,WEEKNUM(16794),ABS((Data!D12/Data!B2)),((Data!C6/100)>=HLOOKUP(Data!A11,Data!A7:B8,4,Data!A6)),V...` | "#N/A" | "#VALUE!" |
| gen-m6/38d4bcdafaed1275 Calc!A35 | `=ADDRESS(10,((Data!E10/Data!E1)/DATE(2055,Data!A3,5)),3,INDEX(Data!A6:A7,3))` | "#REF!" | "#VALUE!" |
| gen-m6/7ac4ea222977f3b4 Calc!A176 | `=ADDRESS(MAX(LEN("#N/A")),((Data!B10*Data!$B10)*(-0.0001)),2,(Data!B$10<>COS(Data!A1)),"Other Sheet")` | "#NUM!" | "#VALUE!" |
| gen-m6/7f3606fef4cb9df7 Calc!A232 | `=ADDRESS(3,(-0.0001),1,IFS(FALSE,LEFT(Data!A$3,3),ISODD(0),SUBTOTAL(Data!D11,Data!B1:E11)))` | "#N/A" | "#VALUE!" |
| gen-m6/870c5e38f51ce5d4 Calc!A229 | `=ADDRESS(18,18,ROUNDUP(COUNT(Data!E10:E10,Data!C4:C7,1000000000000000.0),-3),IF(ISNA(1000000000000000.0),(D...` | "#N/A" | "#VALUE!" |
| gen-m6/9c615d338d990644 Calc!A238 | `=ADDRESS(Data!D$12,17,2,ISNA(Data!C2),INDEX(Data!B7:E8,1))` | "#REF!" | "#VALUE!" |
| gen-m6/a1e6448d0590f7c3 Calc!A69 | `=ADDRESS(11,FLOOR.MATH(TAN(710)),4,SWITCH(Data!D$6,DATE(Data!D11,(-1),4),((-0.5)-Data!A12)),"Data")` | "#N/A" | "#VALUE!" |
| gen-m6/b62dd1ebd266b71c Calc!A326 | `=ADDRESS(5,6,SUM(IF(Data!A10,0.5)),VLOOKUP(MIN(Data!B4,1000000000000000.0,Data!D10:D10),Data!B2:D7,4,Data!D...` | "#REF!" | "#VALUE!" |
| gen-m6/be0e3a1b0fc3f7a0 Calc!A129 | `=ADDRESS(Data!B7,11,N(Data!A8),VLOOKUP((1e-15-(-0.0001)),Data!B8:D10,2,AND(Data!E5:E11,TRUE)))` | "#N/A" | "#VALUE!" |
| gen-m6/be0e3a1b0fc3f7a0 Calc!A281 | `=ADDRESS((Data!B11*(-0.0001))%,VLOOKUP((1^(-0.0001)),Data!D11:D11,2),4,(COUNT(Data!A11,Data!D11:D11)<=IF(Da...` | "#REF!" | "#VALUE!" |
| gen-m6/c96ac1ada46179c4 Calc!A197 | `=ADDRESS(9,26,3,(COUNT(Data!A11:A11)<""),"TRUE")` | "TRUE'!$Z9" | "TRUE!$Z9" |
| gen-m6/d0c30225243fde60 Calc!A291 | `=ADDRESS(4,COUNTIF(Data!B1:D3,""),MIN(Data!A8:A11,Data!C11:C12),INDIRECT(INDEX(Data!C10:E12,6,4)))` | "#REF!" | "#VALUE!" |
| gen-m6/ee56a4ed5a612a0d Calc!A268 | `=ADDRESS(Data!C1,SWITCH((Data!$D12/0.3),EVEN(60),(Data!E$7<Data!B9)),4,(Data!C$3<=PRODUCT(3.14159)),CHOOSE(...` | "#N/A" | "#VALUE!" |
| gen-m6/ee56a4ed5a612a0d Calc!A306 | `=ADDRESS(6,20,4,(MINUTE(0.5)>(Data!B$12-Data!$A$11)),TRANSPOSE(("a*">=Data!E5)))` | "FALSE'!T6" | "FALSE!T6" |

## Acceptance criteria
- [ ] These 17 known-failure lines disappear from eval/baseline/gen-m6.txt (`make eval-baseline`), with no new known failure in any suite (04fdc9a67474aab9 A326 may wait for T-2267; say so in the Outcome).
- [ ] Unit tests cover `ADDRESS(-1,1,1,NA())` (`#N/A`) and `ADDRESS(1,1,1,TRUE,"FALSE")` (`'FALSE'!$A$1`).
- [ ] lookup.md's entry says Ours: implemented (T-2274) for these bullets.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Row/column bounds and relative R1C1 offsets (T-2275).

## Visible-parity stage
Decision 0083: every cell above is visibly off (a wrong error, a wrong type or a value off in a leading digit). The acceptance criteria above are the stage.
