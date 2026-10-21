---
id: T-2270
title: "WORKDAY/NETWORKDAYS(.INTL): a blank one-cell holidays reference and numeric-text holiday cells are accepted (ours #VALUE!)"
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
WORKDAY, WORKDAY.INTL, NETWORKDAYS and NETWORKDAYS.INTL accept a one-cell holidays reference to a blank cell (no holiday) and holiday cells holding number text, as Excel does, instead of `#VALUE!`.

## Context
Gap triage of gen-m6 (T-2180, excel-graph oracle, 2026-10-21). Rule and cells: docs/excel-semantics/dates.md "WORKDAY and NETWORKDAYS holidays:
blank and number-text cells, and the error order", first bullet (added by this triage). `holiday_serials`
(crates/rustsheet-engine/src/functions/date_and_time/holidays.rs) returns `#VALUE!` for a blank one-cell reference and for a text
cell, number text included. Non-number text stays `#VALUE!` (probes/workday-serial-zero F56). Whether number text in a range counts as
a holiday or is skipped is not decided by these cells; pick skip-or-count so that every cell passes and say which in dates.md (UNVERIFIED
if both pass). DAYS360 bf63ed6c6f592475 A282 has `WORKDAY(58488,2,<blank cell>)` inside its method argument.

Failing cells (`make eval-show CASE=gen-m6/<case>.case.json`), 11 in eval/baseline/gen-m6.txt:

| case / cell | formula | Excel | ours |
|---|---|---|---|
| gen-m6/1aa50086e42bff9a Calc!A67 | `=WORKDAY(18260,-30,Data!C4)` | 18218.0 | "#VALUE!" |
| gen-m6/3e51213f35889d06 Calc!A314 | `=NETWORKDAYS(17247,33116,Data!E9)` | 11336.0 | "#VALUE!" |
| gen-m6/50f0dd9a8ee2cb57 Calc!A199 | `=NETWORKDAYS.INTL(45291,14675,4,Data!C9)` | -21869.0 | "#VALUE!" |
| gen-m6/752959b8d3db0285 Calc!A393 | `=WORKDAY(58719,5,Data!D5:D7)` | 58726.0 | "#VALUE!" |
| gen-m6/870c5e38f51ce5d4 Calc!A236 | `=NETWORKDAYS(40614,26046,Data!$A$4)` | -10406.0 | "#VALUE!" |
| gen-m6/9615a6de0f3ac417 Calc!A155 | `=NETWORKDAYS(53964,27350,Data!C$9)` | -19010.0 | "#VALUE!" |
| gen-m6/acc70339aac84a23 Calc!A182 | `=NETWORKDAYS(55827,10481,SWITCH(Data!$E7,Data!E1,CEILING.PRECISE(Data!D8,Data!D6),ROUNDDOWN(0,1),(7*100),Da...` | -32391.0 | "#VALUE!" |
| gen-m6/acdfeaf6e6eadd4e Calc!A83 | `=WORKDAY.INTL(60,PI(),4,Data!E5)` | 63.0 | "#VALUE!" |
| gen-m6/b62dd1ebd266b71c Calc!A111 | `=NETWORKDAYS(59460,0,Data!C$3)` | -42471.0 | "#VALUE!" |
| gen-m6/bf63ed6c6f592475 Calc!A282 | `=DAYS360(2958465,37504,(WORKDAY(58488,2,Data!E9)<=Data!D3))` | -2879035.0 | "#VALUE!" |
| gen-m6/ee56a4ed5a612a0d Calc!A45 | `=NETWORKDAYS.INTL(30156,(SUMIF(Data!B11:C12,1,Data!D5:D7)/CEILING.MATH((-2.5),Data!$C7,Data!B11)),4,Data!D1...` | -21540.0 | "#VALUE!" |

## Acceptance criteria
- [ ] These 11 known-failure lines disappear from eval/baseline/gen-m6.txt (`make eval-baseline`), with no new known failure in any suite.
- [ ] Unit tests cover a blank one-cell holidays and a "1" holiday cell for each of the four functions.
- [ ] dates.md's first bullet says Ours: implemented (T-2270).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The error order between holiday problems and the dates (T-2272).

## Visible-parity stage
Decision 0083: every cell above is visibly off (a wrong error, a wrong type or a value off in a leading digit). The acceptance criteria above are the stage.

## Outcome
`collect_serial_numbers` (date_and_time.rs) accepts a blank one-cell reference (no holiday) and a text cell that reads as a number (a holiday), in a range and as a scalar (computed text included, A182); other text stays `#VALUE!`. Number text in a range is counted, not skipped (UNVERIFIED, dates.md). Unit test covers all four functions with a blank and a "1" cell. Eval: 20 gen-m6 known failures fixed (the 11 here plus 9 error-order cells of T-2272 that now pass by scan order, not because that rule is implemented), no regressions.
