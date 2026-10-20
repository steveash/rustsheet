---
id: T-2156
title: "TYPE of a one-cell range is the cell's type, not 64"
milestone: M5
lane: functions-logic
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
TYPE of a one-cell range (`TYPE(D7:D7)`) is the type of its cell, as Excel does; multi-cell ranges stay 64.

## Context
Gap triage of gen-m5 (T-2114, excel-graph oracle, 2026-10-20). `fn_type` (crates/rustsheet-engine/src/functions/information/mod.rs) returns 64 for any range reference. Rule:
docs/excel-semantics/information.md "TYPE of a one-cell range, ISEVEN/ISODD past 2^63, CELL over a range info_type" (added by
this triage). Check that `TYPE(INDEX(..))` and `TYPE(OFFSET(..))` of one cell follow (unprobed; keep the engine's current
behaviour there unless the same path decides it).

Failing cells (`make eval-show CASE=gen-m5/<case>.case.json`), 2 in eval/baseline/gen-m5.txt:

| case / cell | formula | Excel | ours |
|---|---|---|---|
| gen-m5/2abdc1524e5aa685 Calc!A371 | `=TYPE(Data!D7:D7)` | 1.0 | 64.0 |
| gen-m5/c0651f0e0fb544d0 Calc!A203 | `=TYPE(Data!B3:B3)` | 1.0 | 64.0 |

## Acceptance criteria
- [ ] These 2 known-failure lines disappear from eval/baseline/gen-m5.txt (`make eval-baseline`), with no new known failure in any suite.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Legacy implicit intersection of TYPE (T-1526, done).
