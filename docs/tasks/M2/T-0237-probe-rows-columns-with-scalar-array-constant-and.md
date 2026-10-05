---
id: T-0237
title: "Probe ROWS/COLUMNS with scalar, array-constant and SEQUENCE arguments"
milestone: M2
lane: functions-lookup
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Establish with oracle evidence (excel-probe skill) what ROWS/COLUMNS do for arguments T-0228 left unprobed, and make the engine match.

## Context
T-0228 made ROWS/COLUMNS count array values. Unprobed: scalar non-reference arguments (`ROWS(5)`, `ROWS(NA())`, `COLUMNS(TRUE)`; the engine returns `#VALUE!` / passes the error through) and oracle backing for the hand-written unit expectations over array constants (`ROWS({1;2;3})`, `COLUMNS({1,2,3,4})`, `COLUMNS({1,2;3,4})`) and an array function result (`ROWS(SEQUENCE(5))`). Code: `rows_columns_extent` in `crates/rustsheet-engine/src/cast.rs`; rule in docs/excel-semantics/arrays.md.

## Acceptance criteria
- [ ] A probe case covers the scalar, array-constant and SEQUENCE arguments above.
- [ ] Engine matches the oracle values; docs/excel-semantics/arrays.md drops the "Unprobed" note.
- [ ] `make check` green; reviewers approve.

## Out of scope
- ROW/COLUMN.
