---
id: T-2775
title: "Z.TEST gen-m9 cells left after T-2470: nested sub-expressions and the untraced A294/A324 DIV/0"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
The gen-m9 cells T-2470 owned but did not fix (17 of its 41 known-failure lines, whose Z.TEST call is nested in another function) have Excel's type, error and text.

## Context
T-2470 fixed 24 of the 41 owned lines (one data value before sigma; a lone non-number cell is `#N/A`). The cells it owns that remain in eval/baseline/gen-m9.txt (list: docs/tasks/M9/T-2470-*.md) fail for a reason beyond Z.TEST itself: a nested sub-expression, or the x or sigma argument of 7980ffb6095a340d A294 and a63d1c7d6f96c56d A324 (UNVERIFIED in statistics.md, "Z.TEST: an error array first"). Trace each inner expression (excel-probe skill) before changing code. Code: `fn_z_test` in crates/rustsheet-engine/src/functions/statistical/z_test.rs.

## Acceptance criteria
- [ ] Each T-2470-owned cell still in eval/baseline/gen-m9.txt is fixed (`make eval-baseline`) or its remaining cause is recorded in docs/excel-semantics/statistics.md with a task for it.
- [ ] `make check` green; reviewers approve.

Also probe (excel-probe) whether a single cell reached through a defined name, INDEX or OFFSET takes the same `#N/A` as a plain cell reference (T-2470 handles only a plain reference).

## Out of scope
-
