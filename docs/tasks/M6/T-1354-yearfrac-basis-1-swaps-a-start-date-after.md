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
YEARFRAC with basis 1 and a start date after the end date swaps them and returns the positive fraction, as Excel does and as the engine already does for basis 0.

## Context
T-1275's gen-m7 suite (excel-graph sidecars, 2026-10-11): gen-m7/c263b43217a5898b Calc!A38 `=YEARFRAC(52874,24708,1)`: Excel 77.11295191295191, engine `#NUM!`.
The engine's `YEARFRAC(24708,52874,1)` is 77.11295191295191, so only the order check is wrong; `YEARFRAC(52874,24708,0)` already
swaps (77.11111111111111). docs/excel-semantics/dates.md "YEARFRAC swaps reversed dates for every basis".
Code: `fn_yearfrac` in crates/rustsheet-engine/src/functions/date_and_time.rs (the basis-1 path). Check bases 2, 3 and 4 with
reversed dates in unit tests too.

## Acceptance criteria
- [ ] gen-m7/c263b43217a5898b A38 passes in `./target/release/rustsheet-eval show gen-m7/<case>.case.json`, with no new failing gen-m7 cell and no new known failures in eval/baseline/.
- [ ] Unit tests with reversed dates for bases 0-4.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Basis 1 now swaps reversed dates like the other bases (A38 `=YEARFRAC(52874,24708,1)` = 77.11295191295191). Unit test covers bases 0-4 reversed. Eval: no baseline entry changed.
