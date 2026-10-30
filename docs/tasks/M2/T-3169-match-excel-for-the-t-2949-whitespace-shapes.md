---
id: T-3169
title: "Match Excel for the T-2949 whitespace shapes (intersection LF, stored leading LF, stored even-arity LET)"
milestone: M2
lane: engine
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
FORMULATEXT prints the shapes probed in T-2949 as Excel does.

## Context
Evidence and rules: docs/excel-semantics/syntax.md "Remaining whitespace shapes (T-2949)"; cases probes/t2949-typed-* and t2949-stored-*. LAMBDA and LET name shapes already match. Fix what fails: an intersection separator's line feed is kept (typed-4, typed-5, stored-14); a stored leading LF is dropped (stored-5); a stored even-arity LET `=LET( a ,1, b ,a+1)` loads, prints `=LET( a,1, b,a+1,b)` and returns 2 (stored-11).

Cells (named so `make exit-owners MILESTONE=M2` counts them as owned; gap triage 2026-10-30):
- probes/t2949-stored-11 S!G11, S!H11
- probes/t2949-stored-14 S!H14
- probes/t2949-stored-5 S!H5
- probes/t2949-typed-4 S!E4
- probes/t2949-typed-5 S!E5

## Acceptance criteria
- [x] Every t2949 probe cell passes (or its difference is recorded as a DEVIATION).
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Fixed the three engine gaps: intersection separators holding a line feed print whole, a stored leading LF is dropped, and a stored even-arity LET appends its last name as the result. All six t2949 cells now pass; unit tests added in `test_spacing.rs`.
