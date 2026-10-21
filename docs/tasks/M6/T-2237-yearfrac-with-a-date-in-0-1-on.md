---
id: T-2237
title: "YEARFRAC with a date in [0, 1) on bases 1-4 and next to January-February 1900: probe and match (today #NUM! until probed)"
milestone: M6
lane: functions-date
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
YEARFRAC gives Excel's value or error when its start or end date is in [0, 1) (serial 0,
1900-01-00) on every basis, and when that date is paired with a date in January or February 1900,
including the fictitious serial 60. Both cases are backed by oracle evidence. Today the engine
returns `#NUM!` for bases 1-4 only because nothing was probed.

## Context
Gap triage 2026-10-21. YEARFRAC is an M6 function. One generated cell showed that Excel accepts
serial 0 on basis 0: gen-m9/a2e51f48717af2a1 Calc!A32,
`=((0.3+YEARFRAC(33752,Data!$C12))^SUM((-0.0001)))` with Data!C12 = 0, is 0.9995471537332011, which
(0.3+92.41111111111111)^-0.0001 reproduces bit for bit: `YEARFRAC(33752,0)` is derived as 92.41111111111,
counted from 1900-01-00 as (92*360 + 4*30 + 28)/360 (T-1409). Everything around that cell is
still a guess:
- `functions/date_and_time/yearfrac.rs` `year_fraction_parts` says that a serial in [0, 1) is "a
  date to the 30/360 day count of basis 0 only ... other bases keep #NUM! until probed". It cites
  dates.md "YEARFRAC accepts serial 0", but no section has that name. The note sits in "WEEKDAY of
  serial 0 and return_type 0".
- dates.md there: "other bases with serial 0 stay `#NUM!`, UNVERIFIED; ... serial 0 paired with a
  date in January-February 1900, including the fictitious serial 60, is UNVERIFIED".
- WORKDAY and NETWORKDAYS take serial 0 on a weekday-faithful calendar (dates.md "WORKDAY and
  NETWORKDAYS from serial 0", probes/workday-serial-zero). So serial 0 being a date in YEARFRAC's
  other bases is plausible, but its day counts (actual/actual, actual/360, actual/365, European
  30/360) cannot be inferred.

Probe (excel-probe skill, excel-graph oracle; about 50 cells) `YEARFRAC(s,e,b)` for b in 0..4:
- s in {0, 0.5, 0.9999}, with e in {1, 31, 59, 60, 61, 366, 33752, 45000};
- the reversed pairs;
- `YEARFRAC(0,0,b)`;
- each basis with s = 60 against e in {1, 59, 61}.

Add a 1904-workbook twin for a few cells: `financial_1900_serial` shifts 1904 serials by 2·1462,
and serial 0 there is 1904-01-01. Record the rules in dates.md as a section named "YEARFRAC accepts
serial 0", or fix the code comment's reference. Then implement them in `year_fraction_parts`.
The discount-security functions also call `year_fraction_parts` (financial.md "Discount securities:
day count and year length"). Check that no financial cell moves, or measure and list the ones that do.

## Acceptance criteria
- [ ] A new probes case, oracle-filled (excel-graph), covers the grid above. Every cell passes, or
      its known failure is recorded with `make eval-baseline` and has a filed task named in the Outcome.
- [ ] dates.md states the rule for each basis with its cells, and the UNVERIFIED notes above are gone.
- [ ] Unit tests in `test/test_yearfrac_basis.rs` cover each basis's serial-0 rule. No gated baseline
      gains a line, and no financial cell changes, or each change is listed in the Outcome.
- [ ] `make check` green; reviewers approve.

## Out of scope
- DAYS360, COUP* and the bond functions at serial 0 (probe separately if YEARFRAC's rule suggests
  they differ).
- Negative serials (still `#NUM!`).
