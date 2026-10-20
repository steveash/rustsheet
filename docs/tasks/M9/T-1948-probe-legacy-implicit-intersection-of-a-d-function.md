---
id: T-1948
title: "Probe legacy implicit intersection of a D-function field range, and a TRUE field over a numeric first column (T-1389 review)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle values for two D-function cases the T-1389 review found unprobed, recorded in statistics.md and made to pass.

## Context
T-1389 derives the D-functions' signatures from the lifting table, so `field` is a `LiftArrays` argument: a legacy
(non-dynamic-array) `=DSUM(A1:C6,G1:G2,E1:E2)` no longer gets an implicit `@` and gives `#VALUE!`. Only the Analysis
ToolPak functions (EDATE) were probed for that rule. Separately `resolve_db_field_column` (functions/database.rs) maps a
`TRUE` field to the first column, where statistics.md says it selects no column; the probe passes only because the first
column of the probe data is text.

## Acceptance criteria
- [ ] A probe case (excel-probe skill) for the legacy FORMULATEXT and value of `=DSUM(A1:C6,G1:G2,E1:E2)` and `=DSUM(A1:C6,G1:G2&"",E1:E2)`.
- [ ] A probe of DSUM/DAVERAGE/DGET with a `TRUE` field over a database whose first column is numeric; the code follows the oracle.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
