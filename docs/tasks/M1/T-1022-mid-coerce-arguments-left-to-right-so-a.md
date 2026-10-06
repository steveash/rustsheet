---
id: T-1022
title: "MID: coerce arguments left to right so a bad start/length beats a later error argument"
milestone: M1
lane: functions-text
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`MID("abc","x",NA())` and `MID({"abc"},"x",NA())` are `#VALUE!` as in Excel.

## Context
Found by T-0994. Failing cells: probes/error-precedence-text Calc!C11, Calc!C12 (engine returns #N/A). Rule in docs/excel-semantics/arrays.md ("Errors broadcast through..."). Code: `broadcast_text` in crates/rustsheet-engine/src/functions/text/common.rs.

## Acceptance criteria
- [ ] Both cells pass; MID cases C13/C14 (`#N/A`) keep passing; baseline shrinks.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other text functions (already match Excel in the probe).
