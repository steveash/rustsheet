---
id: T-1389
title: "Database functions: field and criteria lift over array literals; DCOUNT/DCOUNTA with an empty field count records (probes/db-argument-modes)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: sonnet
depends_on: [T-1770]
spec: docs/specs/SPEC-0011-scalar-argument-lifting.md
---

## Goal
The database functions lift `field` and `criteria` given as array literals elementwise, as Excel does,
and DCOUNT and DCOUNTA with an empty `field` count the matching records.

## Context
SPEC-0011 rollout step 3 (docs/specs/SPEC-0011-scalar-argument-lifting.md). `field` lifts through `lift_call` as `LiftArrays`: an array lifts and a two-cell range is `#VALUE!` (D11:D22). `database` is `Whole`. `criteria` is the spec's fifth kind, `LiftArraysRangeWhole`: it lifts over an array literal (F11:F23), but a range is the criteria table, read whole. This task adds that variant to `ArgKind`. The one catalog rule (SPEC-0011, stated in T-1776 too): `LiftArraysRangeWhole` is catalog `mode: criteria` or `range` with `lift: arrays`, derived from the table by T-1776's command. This is the shared lifting helper this task waits for.

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
- [ ] The entries it adds make the variant live, and it removes the `#[allow(dead_code)]` on `ArgKind::LiftArraysRangeWhole` (functions/lift/kinds.rs) (T-1770 left the allow with a `TODO(T-1389)`).

## Out of scope
- The semantic gaps found on generated data (T-1390).
