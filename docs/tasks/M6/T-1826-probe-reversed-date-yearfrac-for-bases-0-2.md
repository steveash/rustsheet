---
id: T-1826
title: "Probe reversed-date YEARFRAC for bases 0, 2, 3, 4"
milestone: M6
lane: functions-date
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
YEARFRAC with a start date after the end date matches Excel for bases 0, 2, 3 and 4, backed by oracle cells.

## Context
T-1354 fixed basis 1 (Excel swaps the dates). Bases 2-4 take the absolute day count (symmetric) and basis 0 takes `abs` of the asymmetric US 30/360 count in `yearfrac_of` (crates/rustsheet-engine/src/functions/date_and_time/yearfrac.rs), so e.g. `YEARFRAC(DATE(2024,3,31),DATE(2024,1,15),0)` is 75/360 while its forward pair is 76/360. No oracle evidence exists. Use the `excel-probe` skill; see docs/excel-semantics/dates.md "YEARFRAC swaps reversed dates for basis 1".

## Acceptance criteria
- [x] A probe case with reversed pairs for bases 0, 2, 3, 4 (including day 30/31 and end-of-February edges) has Excel oracle values.
- [x] The engine matches them, or a fix task is filed; the dates.md entry is updated.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probe `probes/yearfrac-reversed-bases` (72 cells, excel-graph): Excel swaps reversed dates for every basis, so each reversed cell equals its forward pair (basis 0 at `DATE(2024,3,31)`..`DATE(2024,1,15)` is 76/360, not 75/360). The engine now swaps up front for all bases; dates.md entry updated and VERIFIED. Eval: the new case is added at 72/72 cells, no other change.
