---
id: T-1354
title: "YEARFRAC basis 1 swaps a start date after the end date (#NUM! today)"
milestone: M6
lane: functions-date
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
YEARFRAC with basis 1 and a start date after the end date swaps them and returns the positive fraction, as Excel does (basis 0 takes the absolute 30/360 count; see T-1826).

## Context
T-1275's gen-m7 suite (excel-graph sidecars, 2026-10-11): gen-m7/c263b43217a5898b Calc!A38 `=YEARFRAC(52874,24708,1)`: Excel 77.11295191295191, engine `#NUM!`.
The engine's `YEARFRAC(24708,52874,1)` is 77.11295191295191, so only the order check is wrong; `YEARFRAC(52874,24708,0)` is 77.11111111111111 in the engine. docs/excel-semantics/dates.md "YEARFRAC swaps reversed dates for basis 1".
Code: `fn_yearfrac` in crates/rustsheet-engine/src/functions/date_and_time.rs (the basis-1 path). Check bases 2, 3 and 4 with
reversed dates in unit tests too.

## Acceptance criteria
- [x] gen-m7/c263b43217a5898b A38 passes in `./target/release/rustsheet-eval show gen-m7/<case>.case.json`, with no new failing gen-m7 cell and no new known failures in eval/baseline/.
- [x] Unit tests with reversed dates for bases 0-4.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Basis 1 now swaps reversed dates (A38 `=YEARFRAC(52874,24708,1)` = 77.11295191295191). Unit test covers bases 0-4 reversed. Eval: gen-m7 is ungated (no baseline), gen-m7/c263b43217a5898b now passes 40/40, no regressions. Bases 0, 2-4 are unverified: follow-up T-1826.
