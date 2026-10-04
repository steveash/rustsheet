---
id: T-0024
title: "en-US date text: month-first DATEVALUE and month-name dates in date arguments"
milestone: M1
lane: functions
status: todo
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
Date text parses as en-US Excel does: `DATEVALUE("26-01-01")` is #VALUE! (no day-first
reading), and month-name text such as "July 1, 2026" is accepted wherever a date argument is
coerced (DAYS, DAYS360, NETWORKDAYS, NETWORKDAYS.INTL, and the other date functions sharing
that coercion).

## Context
Oracle calibration (docs/research/oracle-calibration.md) showed ironcalc-calc's workbooks were
saved by a day-first Excel, and the engine copied that locale where those cached values gated
it. en-US evidence: `probes/date-text-locale` (excel-graph, canary-pinned to en-US), whose
known failures B1, B4, B5, B6, B7 are this task. Rules: docs/excel-semantics/dates.md
("Date text is parsed month-first"). VALUE already parses en-US (B2, B3, B10 pass); reuse its
parser rather than adding another (one definition; see T-0929 for VALUE's own gaps).

**Conflict to resolve:** fixing the engine makes these ironcalc-calc cells fail against their
day-first cached values: `DATE_AND_TIME/DATEVALUE.xlsx` DATEVALUE!B48,
`DAYS_DAYS360.xlsx` Sheet1!C71/D71, `NETWORKDAYS_NETWORKDAYS.INTL.xlsx` Sheet1!C60/D60 (and
`T_VALUE_VALUETOTEXT.xlsx` C15/I5 already "fail" for the same reason). The harness needs a
documented way to replace or exclude cached expectations known to be non-en-US; a per-cell
override with its evidence is better than accepting baseline regressions. Propose it (decision
record if it changes gate semantics) before using `--allow-regressions`.

## Acceptance criteria
- [ ] probes/date-text-locale B1, B4, B5, B6, B7 pass; their baseline lines are removed.
- [ ] The ironcalc-calc cells above are handled by the agreed mechanism, with the evidence cited; no unexplained regressions.
- [ ] Unit tests for the date-text parser (month-first numeric forms, month names, rejecting day-first-only forms).
- [ ] `make check` green; reviewers approve.

## Out of scope
Locales other than en-US; VALUE's percent and d-Mon-yyyy gaps (T-0929).
