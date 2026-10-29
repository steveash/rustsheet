---
id: T-3137
title: "Probe where YEAR, MONTH and DAY roll a serial just below midnight to the next day, and fit the rule"
milestone: M2
lane: research
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
Oracle evidence fixes how YEAR, MONTH and DAY (M2 scope) read a serial whose fraction is within a second of
midnight: is it the date the floor gives, or the next day, and where is the cut? dates.md then states the rule as
VERIFIED and names the cut. If the engine differs, a follow-up task owns every failing cell.

## Context
- The engine floors the serial: `Model::date_part` (crates/rustsheet-engine/src/functions/date_and_time.rs) calls
  `to_1900_serial(serial.floor() as i64)`. So `DAY(45000.99999999)` is the day of 45000.
- HOUR, MINUTE and SECOND round the time of day to the nearest second, and the carry goes past midnight: at
  23:59:59.6 all three are 0 (dates.md "HOUR, MINUTE and SECOND round to the nearest second"; probes/second-rounding,
  T-1408; `time_of_day_parts`). If HOUR says it is midnight while DAY says it is still the old day, the two disagree
  about the same serial. Nothing tells whether Excel's YEAR/MONTH/DAY take the same carry.
- The upper edge points that way: dates.md "HOUR, MINUTE and SECOND past 9999-12-31" says 2958465.9999949
  (23:59:59.559) is `#NUM!` for HOUR, MINUTE and SECOND. That fits a rounding that carries into serial 2958466, past
  `MAXIMUM_DATE_SERIAL_NUMBER`. Whether `YEAR(2958465.9999949)` is 9999 or `#NUM!` is unprobed.
- Existing date-part evidence never sits inside half a second of midnight. `WEEKDAY(0.9999)` (probes/weekday-serial-zero,
  8.6 s before midnight) floors. gen-m2 and gen-m5 use 59.999988426, a whole second before serial 60. `YEAR(-1)` is
  `#NUM!` (dates.md, 1904 section). The DATE arguments use a different near-integer rule
  (`near_integer_whole`, dates.md "DATE: a year, month or day just below an integer reads as that integer", T-2268),
  which is a third candidate model.
- Why opus: the probe has to separate the candidate models and locate a cut in binary noise. The models are:
  (a) a plain floor; (b) the half-second carry of `time_of_day_parts`, an absolute bound of 0.5/86400 in day units
  whose double rounding depends on the serial's magnitude; (c) a millisecond rounding (as typed clock text and
  `TIMEVALUE("23:59:59.9995")` = 0 suggest, dates.md "Time text"); (d) an absolute near-integer tolerance like
  `near_integer_whole` (2^-22 + 2^-33 day, about 20 ms). The cut must be bracketed over a few magnitudes, because
  the ULP of a serial near 2958465 is about 4.7E-10 day:
  - small serials: 0, 1, 59 (the fictitious 1900-02-29 is serial 60) and 60;
  - mid serials: 45000, plus a month end (45016 is 31 Mar 2023) and a year end (45291 is 31 Dec 2023), so MONTH and
    YEAR roll as well;
  - the top: 2958465.
  At each one, test offsets of 1 s, 0.6 s, 0.5 s ± 1 ULP, 0.4 s, 30 ms, 20 ms, 10 ms, 1 ms and 2^-30 below the
  next integer. Add tiny negatives too: `DAY(-1E-10)`, `YEAR(-2^-40)`, `MONTH(-0.4/86400)`. A carry model reads
  these as serial 0, and a floor gives `#NUM!`.
- Compare against WEEKDAY of the same serials, with HOUR as a control: one column of HOUR next to each date part,
  so that every row shows whether the carry is shared.
- Method: the `excel-probe` skill, excel-graph backend. Build the serials with exact arithmetic in the formula
  (`45000+1-0.5/86400`, `2958466-2^-30`), not as typed decimals. Typed decimals are cut to 15 digits on entry.

## Acceptance criteria
- [ ] A probe case `probes/t3137-date-parts-near-midnight` (milestone tag M2), oracle-filled on excel-graph, holds
      YEAR, MONTH, DAY, WEEKDAY and HOUR of the serials and offsets listed above, at 0, 59, 60, 45000, 45016, 45291
      and 2958465, plus the tiny negatives. That is at least 150 formula cells.
- [ ] dates.md has a section "YEAR, MONTH and DAY of a serial just below midnight". It gives the rule that fits
      every cell, citing cells: floor, half-second carry, millisecond rounding or a near-integer bound. It states
      the cut to the resolution probed, and whether WEEKDAY shares it. The model the evidence rules out is named.
      dates.md's "HOUR, MINUTE and SECOND past 9999-12-31" section says what YEAR does at 2958465.9999949.
- [ ] `make eval-baseline` records the case as new coverage with no regressions. If any cell fails, a functions-date
      M2 task is filed. It names this case and each failing cell (or the whole case), and implements the rule
      through one shared helper with `time_of_day_parts` if the carry is shared. `make exit-owners MILESTONE=M2`
      reports 0.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing `date_part` or `time_of_day_parts` (the follow-up task).
- The 1904 date system, EDATE/EOMONTH, DATEDIF, YEARFRAC, WORKDAY and the other date readers. File a follow-up (M5
  or M9, by the function's milestone) if the rule should carry to them.
- Desktop Excel (M14, decision 0039).
