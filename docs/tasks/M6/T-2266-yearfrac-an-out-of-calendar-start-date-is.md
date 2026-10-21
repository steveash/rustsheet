---
id: T-2266
title: "YEARFRAC: an out-of-calendar start date is #NUM! before an error in the end date"
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
YEARFRAC is `#NUM!` when its start date is out of the calendar even if the end date is an error, as Excel gives.

## Context
Gap triage of gen-m6 (T-2180, excel-graph oracle, 2026-10-21). Rule and cells: docs/excel-semantics/dates.md "YEARFRAC reads its start's
range before the end's error" (added by this triage). In both cells the start is out of the calendar (-1; 123456790.12) and the end
is `#VALUE!` (a SUMIFS shape mismatch; NETWORKDAYS.INTL with a FALSE holiday cell); Excel says `#NUM!`, ours the end's `#VALUE!`.
Code: `fn_yearfrac` / `year_fraction_parts` (crates/rustsheet-engine/src/functions/date_and_time/yearfrac.rs): check the start's range
as soon as it is read, before the end. Keep an error *start* first (its own error), and keep the existing probes green
(probes/fin-argument-modes, probes/date-argument-modes rows for YEARFRAC).

Failing cells (`make eval-show CASE=gen-m6/<case>.case.json`), 2 in eval/baseline/gen-m6.txt:

| case / cell | formula | Excel | ours |
|---|---|---|---|
| gen-m6/3516b9fbbd60269b Calc!A301 | `=YEARFRAC(SUM(-1,Data!E2:E3),(SUMIFS(Data!D10:D12,Data!A5:B7,"abc")+(Data!C$11/Data!C1)),0)` | "#NUM!" | "#VALUE!" |
| gen-m6/d0c30225243fde60 Calc!A266 | `=YEARFRAC(((Data!A1+Data!D6)-SWITCH(Data!E3,"1.5",Data!E4,Data!E11,Data!E$6,Data!B6)),(NETWORKDAYS.INTL(513...` | "#NUM!" | "#VALUE!" |

## Acceptance criteria
- [x] These 2 known-failure lines disappear from eval/baseline/gen-m6.txt (`make eval-baseline`), with no new known failure in any suite.
- [x] A unit test covers `YEARFRAC(-1,1/0)` (`#NUM!`) and `YEARFRAC(NA(),-1)` (`#N/A`).
- [x] dates.md's entry says Ours: implemented (T-2266).
- [x] `make check` green; reviewers approve.

## Out of scope
- The discount-security functions' error order.

## Visible-parity stage
Decision 0083: every cell above is visibly off (a wrong error, a wrong type or a value off in a leading digit). The acceptance criteria above are the stage.

## Outcome
YEARFRAC range-checks its start (same serial range on every basis) before reading the end, so an out-of-calendar start is #NUM! even with an error end; an error start keeps its own error. Eval: 2 gen-m6 known failures fixed, no regressions.
