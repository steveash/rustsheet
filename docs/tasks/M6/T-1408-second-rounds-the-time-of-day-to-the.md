---
id: T-1408
title: "SECOND rounds the time of day to the nearest second instead of truncating"
milestone: M6
lane: functions-date
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
HOUR, MINUTE and SECOND take the time of day rounded to the nearest second, as Excel does, instead of truncating the
seconds after rounding to the millisecond.

## Context
T-1289's gen-m9 suite (excel-graph sidecars, 2026-10-12; `./target/release/rustsheet-eval show gen-m9/<case>.case.json`):
- gen-m9/55fc8c4ffbd7c7cc Calc!A40 `=SECOND(DDB(81074,518,21,6,1))`: DDB is 3024.9333533459067 in the engine (period 6 of a
  21-year life, factor 1: 81074*(20/21)^5/21; a last-digit difference cannot move it by 0.27 s), whose time of day is
  22:24:01.729. Excel 2, engine 1 (`SECOND(0.5+1.7/86400)` is 1 in the engine).
Existing evidence that a rule must also fit: probes/clock-text-oddities F43 `SECOND("10:11.5")` is 11 in Excel (611.5 s, a half that
resolves down, whether by the tie rule or by the serial's binary noise; T-0252). Opus: the tie and carry behaviour is unprobed. Probe with the
excel-probe skill SECOND of times with .4, .5, .6 and .999 s fractions, MINUTE and HOUR at x:59:59.6 (does the carry reach
MINUTE/HOUR?), 23:59:59.6 (HOUR 0?), and the same through TIMEVALUE text. Record the rule in docs/excel-semantics/dates.md.
Code: `seconds_of_day` and `time_part_fn!` (crates/rustsheet-engine/src/functions/date_and_time.rs); HOUR uses its own
`floor` extractor today, so one shared rounding must serve all three.
Semantics: docs/excel-semantics/dates.md "HOUR, MINUTE and SECOND round to the nearest second".

## Acceptance criteria
- [ ] gen-m9/55fc8c4ffbd7c7cc A40 passes in `./target/release/rustsheet-eval show gen-m9/<case>.case.json`, and probes/clock-text-oddities still passes, with no new failing gen-m9 cell and no new known failures in eval/baseline/.
- [ ] The rounding probe cells pass; dates.md's entry is VERIFIED.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
