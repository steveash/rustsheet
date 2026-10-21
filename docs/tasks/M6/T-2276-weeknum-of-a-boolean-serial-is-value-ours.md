---
id: T-2276
title: "WEEKNUM of a boolean serial is #VALUE! (ours 0 or 1)"
milestone: M6
lane: functions-date
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
campaign: visible
---

## Goal
WEEKNUM of a boolean serial read from a cell is `#VALUE!`, before any return_type check, as Excel gives.

## Context
Gap triage of gen-m6 (T-2180, excel-graph oracle, 2026-10-21). Rule and cells: docs/excel-semantics/dates.md "WEEKNUM of a boolean serial is
`#VALUE!`" (added by this triage). `fn_weeknum` (crates/rustsheet-engine/src/functions/date_and_time/weeknum.rs) reads FALSE as serial 0.
WEEKDAY and ISOWEEKNUM read a boolean as serial 1 (dates.md "Date function argument modes"), so change WEEKNUM's serial reading only.
A typed boolean (`WEEKNUM(TRUE)`) is not in the evidence: probe it with the excel-probe skill if the code cannot treat both the same
without guessing, or leave it unchanged and say so.

Failing cells (`make eval-show CASE=gen-m6/<case>.case.json`), 4 in eval/baseline/gen-m6.txt:

| case / cell | formula | Excel | ours |
|---|---|---|---|
| gen-m6/0bcac2d96e96b3df Calc!A234 | `=WEEKNUM(Data!A12,1)` | "#VALUE!" | 0.0 |
| gen-m6/3a26794015dfbe9d Calc!A242 | `=WEEKNUM(Data!D$4,SUM(0.5))` | "#VALUE!" | "#NUM!" |
| gen-m6/3df034e67b14fccb Calc!A149 | `=WEEKNUM(Data!C9,2)` | "#VALUE!" | 1.0 |
| gen-m6/4ca39e0876796fa5 Calc!A274 | `=WEEKNUM(Data!$E$2)` | "#VALUE!" | 0.0 |

## Acceptance criteria
- [ ] These 4 known-failure lines disappear from eval/baseline/gen-m6.txt (`make eval-baseline`), with no new known failure in any suite.
- [ ] Unit tests cover a FALSE and a TRUE cell as serial, with return_type 1 and 0.5.
- [ ] dates.md's entry says Ours: implemented (T-2276).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The stale return_type state (T-1484).

## Visible-parity stage
Decision 0083: every cell above is visibly off (a wrong error, a wrong type or a value off in a leading digit). The acceptance criteria above are the stage.
