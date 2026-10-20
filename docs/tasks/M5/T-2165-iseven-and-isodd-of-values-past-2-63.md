---
id: T-2165
title: "ISEVEN and ISODD of values past 2^63 (ISODD(INT(1E+21)) is FALSE)"
milestone: M5
lane: functions-logic
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
ISEVEN and ISODD treat values at or past 2^63 as even, as Excel does (`ISODD(INT(1E+21))` is FALSE).

## Context
Gap triage of gen-m5 (T-2114, excel-graph oracle, 2026-10-20). `fn_iseven`/`fn_isodd` (functions/information/mod.rs) cast the truncated magnitude to i64, which saturates at i64::MAX (odd).
Every double from 2^53 up is an even integer. Rule: docs/excel-semantics/information.md "TYPE of a one-cell range, ISEVEN/ISODD
past 2^63, CELL over a range info_type" (added by this triage).

Failing cells (`make eval-show CASE=gen-m5/<case>.case.json`), 3 in eval/baseline/gen-m5.txt:

| case / cell | formula | Excel | ours |
|---|---|---|---|
| gen-m5/1315170eef6c413e Calc!A396 | `=ISEVEN((SUBTOTAL(1,Data!D2:D2,Data!D9:D10,Data!D9:E11)/MAX(Data!C9:C10)))` | true | false |
| gen-m5/50d8a085335c05e6 Calc!A92 | `=ISEVEN(MAX(Data!A7:A10,Data!B8:B10,AGGREGATE(11,7,Data!A7:D7,Data!B11:E12,Data!C9:E9)))` | true | false |
| gen-m5/ee1667411bbe2ccc Calc!A159 | `=ISODD(INT(1E+21))` | false | true |

## Acceptance criteria
- [ ] These 3 known-failure lines disappear from eval/baseline/gen-m5.txt (`make eval-baseline`), with no new known failure in any suite.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Parity now uses an f64 remainder instead of an i64 cast, so values past 2^63 are even. Removed 3 known failures from eval/baseline/gen-m5.txt.
