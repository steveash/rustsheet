---
id: T-1136
title: "Port IronCalc DATEVALUE fixes: spaces around date separators and T24:00 rollover"
milestone: M1
lane: functions-date
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
DATEVALUE accepts spaces around date separators (`"2026 - 01 - 01"`, `"1 / 2 / 2026"`, `"4 - Feb - 2026"`, `"2026 - 01 - 01 12:00"`) and rolls `"2026-01-01T24:00:00"` over to the next day, as Excel does, ported from IronCalc with its Excel-authored evidence workbooks.

## Context
Upstream IronCalc commits 753da658 and 03beb38e (DATEVALUE/TIMEVALUE ignore spaces around separators, upstream issues 1380 and 1402) and 106c587d (DATEVALUE rolls over when the time part reaches 24 hours, issue 1379), triaged by the 2026-10-08 steward run. They add Excel-authored workbooks `xlsx/tests/calc_tests/DATE_AND_TIME/DATEVALUE_issue_1380.xlsx`, `TIMEVALUE_issue_1402.xlsx` and `DATEVALUE_issue_1379.xlsx`, plus unit tests `base/src/test/test_datevalue_timevalue_spaces.rs` and `test_datevalue_rollover.rs`.

Checked on `main` (bccce59) through the facade: all four spaced DATEVALUE forms above return `#VALUE!` ("Invalid date"), and `DATEVALUE("2026-01-01T24:00:00")` returns `#VALUE!` (upstream: 46024). Already matching upstream: TIMEVALUE with spaced separators (`"4 : 35"`, `"12 : 00 : 00"`, `"3 : 30 PM"`) and the space-separated 24:00/25:00 rollover (46024).

Code: `crates/rustsheet-engine/src/functions/date_and_time.rs` and the date-text parser it uses (search with `rg` before adding a helper; VALUE and text coercion share the date parser, so check `probes` date-text cases for regressions).

## Acceptance criteria
- [ ] The three upstream workbooks are copied into `crates/rustsheet-xlsx/tests/calc_tests/DATE_AND_TIME/` and pass in `ironcalc-calc` (any cell that does not pass is baselined only with a filed follow-up).
- [ ] Unit tests cover the spaced DATEVALUE forms and the `T24:00:00` rollover.
- [ ] Each quirk is recorded in `docs/excel-semantics/` citing the workbooks.
- [ ] Commit messages carry `Ported-From: ironcalc@753da658`, `@03beb38e`, `@106c587d`; `make check` green; reviewers approve.

## Out of scope
- Other date-text forms not covered by the upstream workbooks (file a probe task instead).
