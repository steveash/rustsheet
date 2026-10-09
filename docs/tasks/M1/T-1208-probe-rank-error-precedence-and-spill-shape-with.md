---
id: T-1208
title: "Probe RANK error precedence and spill shape with a non-reference array ref"
milestone: M1
lane: research
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Know what Excel returns, and over which shape, when RANK/RANK.EQ/RANK.AVG get a non-reference array `ref` (`IF(A1:C1>0,A1:C1)`) together with an error or text `number`, or an error `order`.

## Context
From T-1203 review: since T-1203 such formulas spill a `#VALUE!` array (docs/excel-semantics/statistics.md "RANK of a non-reference array"), but the engine checks `number` first: `RANK(#N/A,IF(..))` is a single `#N/A` (no spill), `RANK("x",IF(..))` a single `#VALUE!`, `RANK(5,IF(..),#DIV/0!)` spills `#VALUE!`. Excel may evaluate per element. Code: functions/statistical/rank_eq_avg.rs.

## Acceptance criteria
- [ ] Oracle-filled probe cases cover error `number`, text `number`, error `order` with an IF-array ref for RANK and RANK.AVG, checking the spill cells too.
- [ ] Findings recorded in statistics.md; an engine fix task filed if the engine differs.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
