---
id: T-2163
title: "SERIESSUM drops blank coefficient cells, raises with the ^ kernel, and an overflowing term beats a text coefficient"
milestone: M5
lane: functions-math
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
SERIESSUM skips blank coefficient cells (the next coefficient takes the power), raises x with the worksheet `^` kernel, and gives `#NUM!` for an overflowing term even when a text cell of the range comes later.

## Context
Gap triage of gen-m5 (T-2114, excel-graph oracle, 2026-10-20). `fn_seriessum` (crates/rustsheet-engine/src/functions/math_and_trigonometry/seriessum.rs) reads a blank coefficient as a 0 term,
uses its own power (an ulp off `^` at 6a8bb7cd A373 and ee166741 A29) and checks range text before computing terms. Rule:
docs/excel-semantics/numbers.md "SERIESSUM: blank coefficient cells are dropped; an overflowing term beats a text coefficient"
(added by this triage; it also corrects the old "a blank coefficient cell adds 0" line). Use `integer_power`/the `^` kernel in
arithmetic.rs. 1d22a002 A157 (`x^-2.5`) misses with our `^` as well: if it still fails after the change, leave it as a listed
residual of the non-integer power (numbers.md "`^` / POWER are not correctly rounded") rather than fitting here. The
`#NUM!`-before-`#VALUE!` order must keep probes/seriessum-x-zero-error-precedence passing (x = 0 rules differ).

Failing cells (`make eval-show CASE=gen-m5/<case>.case.json`), 8 in eval/baseline/gen-m5.txt:

| case / cell | formula | Excel | ours |
|---|---|---|---|
| gen-m5/1d22a002d4f380b3 Calc!A157 | `=SERIESSUM(LOG10(999999999999999),Data!B11,1,SUM(Data!A7:A7,Data!A1:A3))` | 0.011481243964090428 | 0.01148124396409043 |
| gen-m5/34f69d858a39f497 Calc!A170 | `=SERIESSUM(CODE("a*"),0,0.5,Data!B11:E11)` | 260.6221445044903 | 1977.3005580529345 |
| gen-m5/461d1c8dbb044769 Calc!A47 | `=SERIESSUM((SECH(0.00625)*-Data!C10),0,AVERAGE(FACT(7)),Data!A4:A6)` | "#NUM!" | "#VALUE!" |
| gen-m5/5704666097cbc5b2 Calc!A5 | `=SERIESSUM(Data!$C$8,Data!D7,3,Data!B5:C6)` | -437500000000000.0 | 54687500000000.0 |
| gen-m5/5704666097cbc5b2 Calc!A267 | `=SERIESSUM(SUBTOTAL(2,Data!A7:C10,Data!B11:E12,Data!D2:E2),MAX(Data!B11:B11,INDEX(Data!D1:E1,2)),2...` | 2719.9375 | 696319.9375 |
| gen-m5/6a8bb7cd6d7769ad Calc!A373 | `=SERIESSUM((1.005/42),3,1,Data!C5)` | 1.3769443103134108e-05 | 1.3769443103134107e-05 |
| gen-m5/6b831f5e2a20bc34 Calc!A130 | `=CEILING(SERIESSUM(TYPE(Data!B8:B9),1000000000000000.0,0,Data!E3:E12),COUNTIF(Data!A11:D11,"a*"))` | "#NUM!" | "#VALUE!" |
| gen-m5/ee1667411bbe2ccc Calc!A29 | `=SERIESSUM((((-0.0001)-Data!$A10)-MAX(Data!C11)),Data!E8,2,Data!D10)` | 4.200176403616261e+85 | 4.2001764036162545e+85 |

## Acceptance criteria
- [ ] The gen-m5 SERIESSUM cells above leave eval/baseline/gen-m5.txt (1d22a002 A157 may stay as a listed residual), with no new known failure in any suite.
- [ ] numbers.md entry marked implemented.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Non-integer power last digits.
