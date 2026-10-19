---
id: T-2000
title: "Probe T.TEST type, FORECAST x and an error data argument of PERCENTRANK, PROB and FREQUENCY against data errors"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: sonnet
depends_on: [T-1982]
---

## Goal
Oracle evidence for five error orders that T-1982 implemented by inference, each recorded in
docs/excel-semantics/statistics.md "RANK, FORECAST, PROB, Z.TEST and T.TEST: lifted scalar arguments".

## Context
T-1982's Excel-fidelity review (MINOR) found these orders unprobed. Data: D1:D3 = 1, `=NA()`, 3;
A1:A8 = 1,2,4,3,7,5,8,6; B1:B8 = 2,3,5,4,9,6,10,8. Use the `excel-probe` skill.
- T.TEST `type` against the samples' error: `T.TEST(D1:D3,A1:A3,1,4)` and `T.TEST(D1:D3,A1:A3,1,"a")`.
  Ours, which the code infers from P88's `tails`, gives `#NUM!` and `#VALUE!`.
- FORECAST `x` against the data's own validation: `FORECAST("a",A1:A2,B1:B3)` (unequal sizes),
  `FORECAST("a",A1:A1,B1:B1)` (one point) and `FORECAST("a",{1;1},{2;2})` (all x equal).
  Ours: `#VALUE!` for each.
- An error data argument: `PERCENTRANK(NA(),1)`, `QUARTILE(NA(),1)` and `PERCENTILE.INC(NA(),0.5)`
  (ours: `#N/A` since T-1982), and `PROB(NA(),F1:F4,1)` and `FREQUENCY(NA(),{1;2})` (ours: `#VALUE!`,
  via `numeric_flat_of` in statistical/prob.rs).

## Acceptance criteria
- [ ] A probes case holding these cells, filled from excel-graph and committed with its oracle.
- [ ] The findings are written into statistics.md. Every mismatch either gets its own fix task
  or is fixed in this PR if it is a one-line reader change.
- [ ] `make check` green; reviewers approve.

## Out of scope
- RANK (T-1999).
