---
id: T-1559
title: "Probe AVERAGEA/MAXA/MINA over computed arrays of a range (FILTER, SORT, IF array conditions)"
milestone: M2
lane: functions-stat
status: todo
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
- [ ] The new probe has oracle values and passes, or each mismatch is fixed; the finding is recorded in statistics.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
