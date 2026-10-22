---
id: T-2376
title: "ACOT of a negative zero is PI()/2 (ACOT(CEILING.PRECISE(-0.0001)))"
milestone: M5
lane: functions-math
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
campaign: visible
---

## Goal
ACOT of a zero is PI()/2 whatever the sign of the double zero, so `ACOT(CEILING.PRECISE(-0.0001))` matches Excel.

## Context
Gap triage of gen-m5 round 2 (T-2252, excel-graph oracle, 2026-10-22). CEILING.PRECISE(-0.0001) is the double -0.0 here (so are CEILING, CEILING.MATH, ISO.CEILING, ECMA.CEILING of a small negative,
unary minus `-(0)` and `INT(-0)`), and `fn_acot` (mathematical.rs) tests `f >= 0.0`, true for -0.0, then returns
`atan(1/-0.0)` = -PI()/2. Excel shows no negative zero (numbers.md "Negative zero prints as 0", paragraph added by this
triage). Fix ACOT's zero branch; do not change the CEILING family's results (the oracle cannot see a zero's sign elsewhere).
Check the other functions with a sign-of-zero branch the same way (`rg -n '>= 0.0|< 0.0|is_sign' functions/`), in this task only
if a test shows a visible difference.

Failing cells (`make eval-show CASE=gen-m5/<case>.case.json`), 1 in eval/baseline/gen-m5.txt:

| case / cell | formula | Excel | ours |
|---|---|---|---|
| gen-m5/6efa59fc1257c475 Calc!A7 | `=ACOT(CEILING.PRECISE(Data!C9))` | 1.5707963267948966 | -1.5707963267948966 |

## Acceptance criteria
- [ ] Unit tests: `ACOT(-(0))`, `ACOT(CEILING.PRECISE(-0.0001))` and `ACOT(INT(-0.0))` are 1.5707963267948966.
- [ ] This 1 known-failure line disappears from eval/baseline/gen-m5.txt (`make eval-baseline`), with no new known failure in any suite.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Normalising -0.0 engine-wide (an engine-lane task if ever needed).

## Visible-parity stage
Decision 0083 (gap triage 2026-10-22, T-2252): every cell this task owns has the wrong type, error, text or boolean, or is a
number further than half a unit in Excel's 15th significant digit, so the task is in the campaign. Its acceptance
criteria above are already the visible-parity stage.
