---
id: T-0961
title: "Probe SUMX2MY2/SUMX2PY2/SUMXMY2 scalar text, boolean, empty args and error-vs-#N/A precedence"
milestone: M9
lane: functions-math
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Excel's behavior for SUMX2MY2/SUMX2PY2/SUMXMY2 with scalar text, numeric text, boolean and empty
arguments, and error-versus-`#N/A` precedence, is established by oracle probes and implemented.

## Context
T-0944 treats any non-number scalar as one skipped pair (so `SUMXMY2("a","b")` is `#DIV/0!`) and
returns the first error found before the element-count `#N/A` check. Neither choice has Excel
evidence (reviewer finding). Code: `sumx_values` / `sumx_fold` in
`crates/rustsheet-engine/src/functions/math_and_trigonometry/sum.rs`. Use the `excel-probe` skill;
cases: `SUMXMY2("2",4)`, `SUMXMY2(TRUE,1)`, `SUMXMY2(,1)`, `SUMXMY2(range_with_error,longer_range)`.

- gen-m5 (gap triage 2026-10-20, T-2114, excel-graph): 28 more cells, in eval/baseline/gen-m5.txt. These settle most of this task's open questions with Excel evidence; see docs/excel-semantics/coercion.md "SUMX2MY2 / SUMX2PY2 / SUMXMY2 pairing", gen-m5 paragraph (added by this triage). In short: a blank one-cell argument is `#VALUE!` whatever the other argument is (range of another size, number, text, error); a number against an array of another count is `#N/A`; two non-blank equal-count ranges with no numeric pair are 0. The probes listed above (`SUMXMY2("2",4)`, `SUMXMY2(,1)`, ...) are still worth running for the cases gen-m5 does not cover. Several cells are nested (BASE, EXP, COMBINA, SEARCH, TRUNC callers): the SUMX call is the diverging subexpression in each. Acceptance addition: these 28 lines leave eval/baseline/gen-m5.txt with no new known failure.

| case / cell | formula | Excel | ours |
|---|---|---|---|
| gen-m5/033e65b92653f425 Calc!A20 | `=SUMXMY2(IFS(Data!D10,(Data!$A9<1e-15),ISNUMBER("TRUE"),AVERAGE(Data!B6:B6,Data!B6),EXACT(Data!B9,...` | "#VALUE!" | "#DIV/0!" |
| gen-m5/043ee562f32eb901 Calc!A36 | `=SUMXMY2(Data!$B$12,Data!D1:E8)` | "#VALUE!" | "#N/A" |
| gen-m5/043ee562f32eb901 Calc!A65 | `=SUMX2PY2(Data!B3,Data!A3:A8)` | "#VALUE!" | "#N/A" |
| gen-m5/043ee562f32eb901 Calc!A196 | `=SUMX2PY2(LENB(Data!B5),Data!A4:A4)` | "#VALUE!" | "#DIV/0!" |
| gen-m5/1962de140dae0111 Calc!A388 | `=SUMX2PY2(Data!C$4,MATCH(SQRTPI(Data!E4),Data!B9:B9))` | "#VALUE!" | "#NUM!" |
| gen-m5/1d22a002d4f380b3 Calc!A211 | `=TRUNC(SUMX2PY2(Data!C8:C8,Data!A4:C9))` | "#VALUE!" | "#N/A" |
| gen-m5/204b68ebb5812d50 Calc!A218 | `=SUMX2PY2(Data!B4,(MMULT(Data!A11,Data!C8)^INDEX(Data!B6:E11,0,1)))` | "#N/A" | "#VALUE!" |
| gen-m5/49bfbf67a1409563 Calc!A307 | `=EXP(FLOOR.MATH(Data!$E6,SUMX2PY2(Data!$A10,Data!A6)))` | "#VALUE!" | "#DIV/0!" |
| gen-m5/4d099bea7898ebf1 Calc!A296 | `=SUMX2MY2(Data!$B2,SUMIF(Data!E7:E11,">0"))` | "#VALUE!" | "#DIV/0!" |
| gen-m5/5704666097cbc5b2 Calc!A47 | `=BASE(475,(SUMX2MY2(Data!$E$6,Data!A$10)+(Data!D$8-1e-15)),Data!C11)` | "#VALUE!" | "#DIV/0!" |
| gen-m5/6a511beadb56b932 Calc!A127 | `=SUMX2MY2(Data!C8:C9,Data!B$10)` | "#VALUE!" | "#N/A" |
| gen-m5/6a511beadb56b932 Calc!A389 | `=SUMX2MY2(Data!D8:D11,Data!C7:C7)` | "#VALUE!" | "#N/A" |
| gen-m5/6a8bb7cd6d7769ad Calc!A334 | `=SUMX2PY2(Data!B7:D7,Data!D9)` | "#VALUE!" | "#N/A" |
| gen-m5/6b831f5e2a20bc34 Calc!A16 | `=SUMX2PY2(Data!C7,DAY(48329))` | "#VALUE!" | "#DIV/0!" |
| gen-m5/6ba5e4aceeef8209 Calc!A350 | `=SUMX2PY2(IF((1.005>Data!A5),Data!E5,ARABIC(Data!C6)),Data!A11:C11)` | "#VALUE!" | "#N/A" |
| gen-m5/7d1a5d223624fc70 Calc!A36 | `=SUMXMY2(Data!B5:B8,Data!E10:E10)` | "#VALUE!" | "#N/A" |
| gen-m5/8a387c26d182e73c Calc!A307 | `=SUMX2MY2(Data!A8:D9,Data!C6:D9)` | 0.0 | "#DIV/0!" |
| gen-m5/8df4830f888f16ae Calc!A101 | `=COMBINA((-0.5),SUMXMY2(Data!B1:D12,Data!A8))` | "#VALUE!" | "#N/A" |
| gen-m5/8df4830f888f16ae Calc!A324 | `=SUMX2MY2(Data!A5,Data!B4:E10)` | "#VALUE!" | "#N/A" |
| gen-m5/901e91c09b347914 Calc!A330 | `=SUMX2MY2(Data!B7:D12,Data!$E9)` | "#VALUE!" | "#N/A" |
| gen-m5/aac6230c2fb40e1e Calc!A11 | `=SUMX2MY2(Data!C10:D12,(INDEX(Data!B4:E11,3,Data!B9)*(1e-15-1)))` | "#N/A" | "#VALUE!" |
| gen-m5/b237245054b4fedb Calc!A294 | `=SUMX2MY2(Data!$A$1,Data!A7:B12)` | "#VALUE!" | "#N/A" |
| gen-m5/d1b251fd6e9c4b4b Calc!A397 | `=SUMX2MY2(Data!C10:D12,Data!C11:E12)` | 0.0 | "#DIV/0!" |
| gen-m5/d9fb1e918971111b Calc!A35 | `=SUMXMY2(Data!B11,Data!B$6)` | "#VALUE!" | "#DIV/0!" |
| gen-m5/eb942f4a59b1c004 Calc!A231 | `=SUMX2MY2(YEAR(50111),Data!D1)` | "#VALUE!" | "#DIV/0!" |
| gen-m5/ee1667411bbe2ccc Calc!A84 | `=SUMX2PY2(Data!B5,(SIN(1E+9)^MATCH((-1),Data!A4:D7,1)))` | "#VALUE!" | "#NUM!" |
| gen-m5/ee1667411bbe2ccc Calc!A94 | `=SUMXMY2(COUNTIF(Data!A3:C11,"<5"),Data!D11:D11)` | "#VALUE!" | "#DIV/0!" |
| gen-m5/f0f205865a3a4328 Calc!A103 | `=SEARCH(VLOOKUP(SUMXMY2(Data!$B$8,Data!E12),Data!D11:E12,4,(Data!A$6="#N/A")),((""&"#N/A")&IFERROR...` | "#VALUE!" | "#DIV/0!" |

## Acceptance criteria
- [ ] Probe cases in eval/suites/probes with oracle values; coercion.md entry updated to VERIFIED.
- [ ] Implementation matches the oracle values.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other `fn_get_two_matrices` callers.
