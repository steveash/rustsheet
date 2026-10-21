---
id: T-2271
title: "WORKDAY.INTL/NETWORKDAYS.INTL weekend number: a fraction truncates, a blank weekend cell is #NUM!; probe a boolean weekend"
milestone: M6
lane: functions-date
status: todo
priority: P2
size: S
tier: opus
depends_on: []
campaign: visible
---

## Goal
WORKDAY.INTL and NETWORKDAYS.INTL read a weekend number as Excel does: a fraction truncates (3.14159 is 3) and a blank weekend cell is weekend 0 (`#NUM!`), not the default. What a TRUE or FALSE weekend gives on
valid dates is unprobed (the one boolean-weekend cell, ee56a4ed5a612a0d A387, only shows the start's error wins): probe it with the
excel-probe skill before implementing it.

## Context
Gap triage of gen-m6 (T-2180, excel-graph oracle, 2026-10-21). Rule and cells: docs/excel-semantics/dates.md "WORKDAY.INTL and NETWORKDAYS.INTL:
weekend number coercion" (added by this triage). `Model::weekend_mask` (crates/rustsheet-engine/src/functions/date_and_time.rs) refuses a
fractional number with `#NUM!`, a boolean with `#VALUE!`, and takes a blank cell as the default weekend. With the weekend valid, the
dates' own error shows (`NETWORKDAYS.INTL(44926," x ",3.14159)` is `#VALUE!`, `NETWORKDAYS.INTL(#DIV/0!,..,TRUE,..)` `#DIV/0!`). Keep the
probed error-valued and 3-D weekend rules (dates.md "An error-valued or 3-D weekend", probes/date-text-weekend-3d).

Failing cells (`make eval-show CASE=gen-m6/<case>.case.json`), 9 in eval/baseline/gen-m6.txt:

| case / cell | formula | Excel | ours |
|---|---|---|---|
| gen-m6/121a7c17092038d9 Calc!A308 | `=WORKDAY.INTL(47872,21,(Data!$E9*-Data!A7))` | 47903.0 | "#NUM!" |
| gen-m6/1a404a0379a1372c Calc!A62 | `=NETWORKDAYS.INTL(0.5,45351,(2.5^0.3))` | 32394.0 | "#NUM!" |
| gen-m6/870c5e38f51ce5d4 Calc!A125 | `=NETWORKDAYS.INTL(2001,12162,((2.5-0.5)+Data!C$10))` | 7258.0 | "#NUM!" |
| gen-m6/c93ea588c271840e Calc!A31 | `=NETWORKDAYS.INTL(16492,11827,3.14159)` | -3334.0 | "#NUM!" |
| gen-m6/c96ac1ada46179c4 Calc!A287 | `=NETWORKDAYS.INTL(44926,Data!E11,Data!D11)` | "#VALUE!" | "#NUM!" |
| gen-m6/d1a403971712513c Calc!A217 | `=NETWORKDAYS.INTL(44957,36409,(CSC(1.005)^EVEN(2.5)),2)` | -6107.0 | "#NUM!" |
| gen-m6/ee56a4ed5a612a0d Calc!A186 | `=WORKDAY.INTL(45351,10,Data!A7)` | 45365.0 | "#NUM!" |
| gen-m6/ee56a4ed5a612a0d Calc!A387 | `=NETWORKDAYS.INTL(((Data!D$12/Data!D7)^SUMIFS(Data!C10:C11,Data!A2:D9,"",Data!A6:C8,"<5",Data!B4:C6,1)),406...` | "#DIV/0!" | "#VALUE!" |
| gen-m6/fde83d52a9d1fc63 Calc!A219 | `=WORKDAY.INTL(16895,22,Data!B$8,COUNTA(Data!A4:A6,EDATE(23398,8),Data!B1:B3))` | "#NUM!" | 16925.0 |

## Acceptance criteria
- [ ] These 9 known-failure lines disappear from eval/baseline/gen-m6.txt (`make eval-baseline`), with no new known failure in any suite.
- [ ] A probe case (excel-graph) gives TRUE and FALSE weekends on valid dates for both functions; dates.md records it.
- [ ] Unit tests cover weekend 3.9, 11.5, TRUE, FALSE (as probed) and a blank cell for both functions; probes/date-text-weekend-3d still passes.
- [ ] dates.md's entry says Ours: implemented (T-2271).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Weekend strings (unchanged).

## Visible-parity stage
Decision 0083: every cell above is visibly off (a wrong error, a wrong type or a value off in a leading digit). The acceptance criteria above are the stage.

## Handoff
Done on this branch: probe probes/weekend-number-coercion (excel-graph values filled, A1:A26: a number weekend truncates, TRUE is 1, FALSE and a blank cell are 0 = `#NUM!`, for both functions), `weekend_mask` rewritten to match (crates/rustsheet-engine/src/functions/date_and_time.rs), a unit test in test/test_fn_weekend_lift.rs, the dates.md entry updated. The engine tests pass.

Blocker: `make eval-baseline` fixes the 9 owned cells but reports 1 REGRESSION, gen-m6/ef10bf3afb9658f6 Calc!A302 `=NETWORKDAYS.INTL(0,10485,2.5,Data!B3:E12)`: Excel `#NUM!`, ours `#VALUE!`. It passed before only because the fractional weekend 2.5 gave `#NUM!` at once; with weekend 2 valid, the holidays (text "1.5", " x ", "ü", TRUE and negative numbers -1, -2.5) decide, and the error order among them is Excel's, not ours. I tried "a numeric holiday out of the calendar beats a text holiday anywhere in the range" in `collect_serial_numbers`: 130 regressions (e.g. gen-m6/121a7c17092038d9 A171 `WORKDAY(16968,-2,Data!B5:D12)` is `#VALUE!` in Excel), so that was reverted. The holiday error order is T-2272's open question (dates.md "WORKDAY and NETWORKDAYS holidays ... the error order", UNVERIFIED). Next: probe the holiday scan order (text, boolean and negative-number holidays in different positions, NETWORKDAYS.INTL vs WORKDAY.INTL, weekend valid) with excel-probe, implement it in `collect_serial_numbers`/`holiday_serials`, then `make eval-baseline` (needs 0 regressions), `make check`, review.
