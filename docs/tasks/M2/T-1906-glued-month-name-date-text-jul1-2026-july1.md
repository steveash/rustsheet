---
id: T-1906
title: "Glued month-name date text: Jul1 , 2026, July1, 2026, 1-Jul2026 and 1July1 are dates; typed Jul1 , 2026 too"
milestone: M2
lane: functions-date
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
The engine parses the glued month-name shapes Excel accepts: `"Jul1 , 2026"`, `"July1, 2026"`, `"1-Jul2026"` (2026-07-01) and `"1July1"` (2001-07-01), in VALUE, DATEVALUE, criteria and typed input, with or without a clock suffix (`"Jul1 , 2026 10:00"`).

## Context
Oracle evidence: probes/date-text-glued-month-2 rows 6-9 and 12 (cells B-E, typed O6:Q6), recorded in docs/excel-semantics/dates.md (glued-month entry). The engine returns #VALUE! (and leaves typed `Jul1 , 2026` as text). Code: `split_glued_month` in crates/rustsheet-engine/src/formatter/date_text.rs and its callers in `parse_date`. Found by T-1879.

## Acceptance criteria
- [ ] Cells B6:E9, B12:E12 and O6:Q6 of probes/date-text-glued-month-2 pass; eval baseline shrinks.
- [ ] Existing glued-month cases still pass (`July1,2026`, `Jun1 2021` stay #VALUE!).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Locales other than en-US.

## Outcome
`parse_date` (formatter/date_text.rs) now reads `Jul1 , 2026` and `July1, 2026` (glued month and day, comma, year), `1-Jul2026` (day, hyphen, glued month and year) and `1July1` (glued single digit as year 01). B6:E9's TIMEVALUE column C stays with T-1907. Eval: probes known failures 2130 -> 2111 (fixed 19, regressions 0).
