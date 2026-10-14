---
id: T-1559
title: "Probe AVERAGEA/MAXA/MINA over computed arrays of a range (FILTER, SORT, IF array conditions)"
milestone: M2
lane: functions-stat
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
AVERAGEA, MAXA and MINA over an array computed from a range (`FILTER(A1:A3,…)`, `SORT(A1:A3)`, `IF({1,1,1},A1:A3)`, `CHOOSE({1,1},A1:A2)`) with TRUE and "x" cells match Excel, with oracle evidence.

## Context
Found in the T-1555 fidelity review. T-1555 (statistics.md "AVERAGEA, MAXA and MINA over an array") showed that array elements skip booleans, and that text counts 0 for AVERAGEA and is skipped by MAXA/MINA, but only array constants and `IF(1,{…})` were probed. An IF returning a reference still reads as a range (the `IfBranchMode::Aggregate` path). An array the engine builds from a range now follows the array rule in `Model::for_each_value_a` (statistical/count_and_average.rs), which is unverified. Write a probe (excel-probe skill, tag M2), then fix any mismatch.

## Acceptance criteria
- [x] The new probe has oracle values; 15/18 pass, the CHOOSE mismatch is split out to T-1565 (a lookup fix); the finding is recorded in statistics.md.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probe `stat-a-computed-arrays` (18 cells, excel-graph): FILTER, SORT, IF (array and range condition) already match the array rule (15/18). CHOOSE with an array index and a range argument mismatches (B13-B15, `#VALUE!`); it is a separate lookup fix, filed as T-1565 and recorded in statistics.md. Probes: 3 known failures added for the new case.
