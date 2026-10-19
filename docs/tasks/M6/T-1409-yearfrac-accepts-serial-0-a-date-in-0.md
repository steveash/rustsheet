---
id: T-1409
title: "YEARFRAC accepts serial 0 (a date in [0,1)) as start or end date"
milestone: M6
lane: functions-date
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
YEARFRAC accepts serial 0 (and a fractional date in [0, 1), truncated to 0) as its start or end date, as Excel does,
instead of `#NUM!`.

## Context
T-1289's gen-m9 suite (excel-graph sidecars, 2026-10-12; `./target/release/rustsheet-eval show gen-m9/<case>.case.json`):
- gen-m9/a2e51f48717af2a1 Calc!A32 `=((0.3+YEARFRAC(33752,Data!$C12))^SUM((-0.0001)))` with C12 = 0: Excel 0.9995471537332011,
  engine `#NUM!` (`YEARFRAC(33752,0)` and `YEARFRAC(33752,0.5)` are `#NUM!`). Solving for YEARFRAC gives 92.41111111111,
  which is basis 0 (US 30/360) from 1900-01-00 (serial 0, day 0) to 1992-05-28 (33752): (92*360 + 4*30 + 28)/360 = 33268/360.
Serial 0 is Saturday 0 January 1900 (docs/excel-semantics/dates.md "WEEKDAY of serial 0 and return_type 0"); T-1253 and
T-1254 accept it in WEEKDAY, WORKDAY and NETWORKDAYS: reuse what they use. Code: `fn_yearfrac`
(crates/rustsheet-engine/src/functions/date_and_time.rs).

## Acceptance criteria
- [x] gen-m9/a2e51f48717af2a1 A32 passes in `./target/release/rustsheet-eval show gen-m9/<case>.case.json`, with no new failing gen-m9 cell and no new known failures in eval/baseline/.
- [x] Unit tests: `YEARFRAC(33752,0)` = 33268/360 and `YEARFRAC(0,33752)` the same; other bases with serial 0 only if probed.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
YEARFRAC basis 0 accepts a start or end date in [0, 1) as day 0 of January 1900 (shared `days_30_360_us_ymd` count (financial::dates), ordered like every basis); gen-m9/a2e51f48717af2a1 A32 passes (the case's other mismatches A24, A34 are not this task's). gen-m9 baseline unchanged. Bases 1-4 with serial 0 stay `#NUM!`: unprobed. Unit test `test_yearfrac_basis_0_accepts_serial_0`.
