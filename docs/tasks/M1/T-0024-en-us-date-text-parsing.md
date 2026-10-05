---
id: T-0024
title: "en-US date text: month-first DATEVALUE and month-name dates in date arguments"
milestone: M1
lane: functions-date
status: done
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
- [x] probes/date-text-locale B1, B4, B5, B6, B7 pass; their baseline lines are removed.
- [x] The ironcalc-calc cells above are handled by the agreed mechanism, with the evidence cited; no unexplained regressions.
- [x] Unit tests for the date-text parser (month-first numeric forms, month names, rejecting day-first-only forms).
- [x] `make check` green; reviewers approve.

## Out of scope
Locales other than en-US; VALUE's percent and d-Mon-yyyy gaps (T-0929).

## Outcome
- One en-US date-text grammar, `formatter::date_text::parse_date`, now serves VALUE, DATEVALUE,
  date arguments (`cast_number`), criteria and typed input. DATEVALUE's separate grammar is
  gone: it reads that grammar plus a clock suffix (`time_text::parse_date_clock_serial`). The
  rules come from two new oracle probes, `probes/date-text-grammar` (63 forms, DATEVALUE and
  VALUE) and `probes/date-text-dotted-criteria`, and are recorded in dates.md and criteria.md.
- Found while doing this: criteria text `<=`/`>=` compared the wrong way round. Fixed, with evidence.
- Mechanism for the day-first ironcalc-calc cells: decision 0032. `cached-overrides.tsv`
  scores 7 cells against en-US probe values, and `rustsheet_xlsx`'s generated tests skip them.
- Eval: ironcalc-calc known failures 1580 → 1577 (T_VALUE C15, I5, I7 fixed), 0 regressions;
  probes: date-text-locale B1/B4–B7 fixed. 5 new-coverage known failures (VALUE of date+clock
  text: T-0159). Year-less "current year" forms: T-0160.
