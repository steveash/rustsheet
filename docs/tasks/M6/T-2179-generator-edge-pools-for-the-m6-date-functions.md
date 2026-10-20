---
id: T-2179
title: "Generator edge pools for the M6 date functions: date serials, date and time text, weekend masks and return types"
milestone: M6
lane: harness
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`rustsheet-gen --milestone M6` draws the date-function arguments where Excel's rules change (the serial
bounds, the fictitious 1900-02-29, fractional serials, date and time text, weekend strings, every return_type
and basis) as well as the uniform integers it draws today, so the 20,000-cell gen-m6 suite (T-2180) measures
those edges.

## Context
- M6's exit (docs/milestones.md "same as M5"; decision 0085) needs gen-m6 at >= 20,000 scored cells and
  >= 99.9% bit-exact. gen-m6 has 25 cases and 1,000 cells today (`make eval`, 2026-10-20), and the M6 date params in
  eval/catalog/functions.yaml have no `pool:` key. The M5 pools (T-2109, T-2110) are the model:
  `POOLS` in tools/src/rustsheet_tools/gen/pools.py, drawn 35% of the time by `generator.py` (`POOL_RATE`).
- What the catalog draws today: every `start_date`/`end_date`/`serial_number`/`date` param is an integer 1 .. 60000
  (DATEDIF, DAYS, DAYS360, EDATE, EOMONTH, ISOWEEKNUM, NETWORKDAYS(.INTL), WEEKDAY, WEEKNUM, WORKDAY(.INTL), YEARFRAC);
  HOUR/MINUTE/SECOND `serial_number` has no domain; DATEVALUE and TIMEVALUE draw from 5 fixed strings;
  WEEKDAY `return_type` is 1 .. 3 and WEEKNUM's 1 .. 2 (Excel also takes 11 .. 17, and 21 for WEEKNUM);
  `weekend` is 1 .. 17 (Excel also takes 7-character "0000011" strings); EDATE/EOMONTH `months` is an integer.
- Rules the pools should reach (docs/excel-semantics/dates.md): "Serial 60 is the fictitious 1900-02-29",
  "WEEKDAY of serial 0 and return_type 0", "The en-US date-text grammar", "Date text with a month name glued to
  digits", "Time text", "HOUR, MINUTE and SECOND round to the nearest second", "HOUR, MINUTE and SECOND past
  9999-12-31", "WORKDAY and NETWORKDAYS from serial 0", "YEARFRAC swaps reversed dates".

## Acceptance criteria
- [ ] pools.py has these pools (formula texts, `entries(..)` style), each with a comment naming the rule it probes:
      - `date_serial`: 0, 1, 59, 60, 61, 366, 367, 1462, 2958465, 2958466, (-1), 0.5, 59.999988426, 60.5,
        45000.75, 45351 (2024-02-29), 1E+15, plus a few month-end and year-end serials;
      - `time_serial`: 0, 0.5, 0.999988426, 0.99999999, 1/86400, 0.5/86400, 1.5/86400, 2958465.99999, (-0.5);
      - `date_text`: en-US date strings per the grammar entry (month-first, month names, two-digit years on both sides
        of the 1929/1930 century cut, ISO `2024-02-29`, `1900-02-29`, glued month names, surrounding spaces, an
        invalid `2/30/2024`), no year-less dates and no `m-n` text that reads as a day of the current year (both depend
        on the clock; synthetic-generator.md §6.2, dates.md "The en-US date-text grammar");
      - `time_text`: 12- and 24-hour times, seconds, `25:00`, `12:00 AM`, a single-letter `a`/`p` suffix, date-time text;
      - `weekend_mask`: "0000011", "1000001", "1111111", "0000000", "000001", "00000011", "abc", 1, 7, 11, 17, 18;
      - `weekday_return` (1, 2, 3, 11 .. 17, 0, 4) and `weeknum_return` (1, 2, 11 .. 17, 21, 3);
      - `month_offset`: 0, 1, (-1), 1.5, (-1.5), 12, 1200, (-1200), 120000.
- [ ] eval/catalog/functions.yaml puts `pool:` on the matching M6 date params (dates on every start/end/serial param,
      time_serial on HOUR/MINUTE/SECOND, the text pools on DATEVALUE/TIMEVALUE, weekend_mask on both `.INTL`
      `weekend`, the return pools on WEEKDAY/WEEKNUM `return_type`, month_offset on EDATE/EOMONTH `months`), and
      widens no `hard` domain; the T-2109 pool-name test passes.
- [ ] tools tests: every pool entry is at most 255 characters and no `date_text` or `time_text` entry is year-less
      (the existing `test_text_pools_respect_the_string_literal_limit_and_clock_free_values` extended).
- [ ] One generated M6 case that uses every new pool converts with spec2xlsx and `rustsheet-eval run` executes it
      without a crash (command in the Outcome).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Regenerating or oracle-filling gen-m6 (T-2180).
- Lookup-function pools (functions-lookup lane's params keep their domains).
- Engine changes for anything the new edges reveal (T-2180 files them).
