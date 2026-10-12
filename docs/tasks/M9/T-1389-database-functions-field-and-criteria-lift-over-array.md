---
id: T-1389
title: "Database functions: field and criteria lift over array literals; DCOUNT/DCOUNTA with an empty field count records (probes/db-argument-modes)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
The database functions lift `field` and `criteria` given as array literals elementwise, as Excel does,
and DCOUNT and DCOUNTA with an empty `field` count the matching records.

## Context
Evidence: probes/db-argument-modes (excel-graph, 2026-10-11, T-1086), docs/excel-semantics/statistics.md
"Database function argument modes". Known failures:
- C11:C22, `ROWS(DSUM(A1:C6,{2;3},E1:E2))` is 2 for every D-function. We give `#N/IMPL!`.
- D11:D22, a two-cell range as `field` is `#VALUE!`, so a range does not lift. We give `#N/IMPL!`.
- F11:F22 and F23, criteria `{"Qty";">2"}` lifts to a two-row result of `#VALUE!` per element. It is
  `#SPILL!` where it is blocked, and `#VALUE!` in F22:F23 where it spills. We give `#VALUE!` in place.
- K12:K13, `DCOUNT(A1:C6,,E1:E2)` and `DCOUNTA(..)` are 4, the number of matching records. The other
  D-functions give `#VALUE!`. We give `#VALUE!` for all of them.
Use the shared lifting helper if one exists by then (see T-1290, T-1372, T-1383). Do not write a
per-function copy.

## Acceptance criteria
- [ ] Those probes/db-argument-modes cells leave eval/baseline/probes.txt with no new F lines.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The semantic gaps found on generated data (T-1390).
