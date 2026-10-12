---
id: T-1435
title: "1904 workbooks: date text coerced to a number, typed into a cell, and in criteria counts from 1904-01-01"
milestone: M1
lane: functions-date
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
In a 1904 workbook, date text that is coerced to a number, typed into a cell, or used in a criterion gives the 1904 serial, as Excel does.

## Context
Split from T-1416. probes/date-system-1904-readers (excel-graph, 2026-10-12) has 9 known failures here. We count all of them from 1900, 1462 too high:
- `="1904-01-02"+0` = 1 (A20).
- `="2024-03-15"+0`, `=--"3/15/2024"` and `=VALUE("2024-03-15")` = 43904 (A21-A23).
- `COUNTIF(B1:B3,"2024-03-15")` and `COUNTIF(B1:B3,">2024-03-14")` = 1 over a 1904 `DATE(2024,3,15)` (A24, A25), and `COUNTIF(E1,"<=1/1/1908")` = 0 for E1 = 2000 (A51).
- Typed `1904-01-02` and `3/15/2024` (`{"entered":...}`) store 1 and 43904 (A38, A39).

Code paths, which are all 1900-only today:
- `parse_formatted_number` (formatter/format.rs), read by `Model::parse_number_text` (cast.rs) and by typed input (model.rs, the `parse_formatted_number` call in the set-user-input path).
- `parse_date_criterion` (functions/criteria.rs).
- `time_text::parse_date_clock_serial`, for date-time text.

Shift the 1900 serial these return with `DateSystem::serial_from_1900_serial` (formatter/dates.rs), as `Model::parse_datevalue_text` does since T-1330. Two cases are unprobed. A negative 1904 holiday serial (`=WORKDAY(DATE(2024,3,15),1,-1)`) is #NUM! in ours, which needs a check. Text naming a day before 1904-01-01 is also unprobed: probe it (`="1903-12-31"+0`) before deciding.

## Acceptance criteria
- [ ] probes/date-system-1904-readers A20-A25, A38, A39 and A51 pass, and the baseline is updated.
- [ ] A pre-epoch date text probe is added and matched, and the result is recorded in dates.md "The 1904 date system".
- [ ] `make check` green; reviewers approve.

## Out of scope
- Financial date arguments (T-1436).

## Outcome
`parse_formatted_number` and the date criteria now shift the 1900 serial of date text into the workbook's date system (`DateSystem::serial_from_1900_serial`), so number coercion, VALUE, typed input and COUNTIF/SUMIF/*IFS/D-function criteria read 1904 serials. probes/date-system-1904-readers: 9 known failures fixed (A20-A25, A38, A39, A51). New probe probes/date-system-1904-pre-epoch-text (15 cells, all pass): pre-epoch date text is #VALUE!, typed stays text; recorded in dates.md. Date-clock text in a 1904 workbook is still 1900-based (unprobed; T-1441).
