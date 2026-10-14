---
id: T-1570
title: "Probe FILTERXML with an error only in the second argument and with errors in both"
milestone: M5
lane: functions-text
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence (excel-graph) for FILTERXML when only the second argument is an error and when both are, replacing the hand expectations in T-1255.

## Context
T-1255 propagates the first error left to right; only the first-argument case is oracle-backed (docs/excel-semantics/web.md). Unit test A3/A4 in crates/rustsheet-engine/src/test/test_fn_filterxml.rs are hand-expected: `FILTERXML("x",NA())` and `FILTERXML(1/0,NA())`. Use the `excel-probe` skill.

## Acceptance criteria
- [ ] A probe case with `FILTERXML("x",NA())`, `FILTERXML(1/0,NA())` and `FILTERXML(NA(),1/0)` has oracle values; web.md's UNPROBED note is replaced with the result and the engine/tests match (fix fn_filterxml if Excel differs).
- [ ] `make check` green; reviewers approve.

## Out of scope
-
