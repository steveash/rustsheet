---
id: T-2518
title: "Cap per-record re-evaluation of D-function formula conditions (T-1390 security follow-up)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
A D-function whose criteria hold formula conditions cannot run unbounded work: the per-record
re-evaluations of one call are charged against a budget, and past it the call is `#ERROR!`
(a DEVIATION, decision 0014).

## Context
T-1390 re-evaluates a formula condition under a header that names no database column once per
record (`db_computed_condition` in functions/database.rs; statistics.md "Database criteria"). Nesting
is limited to one level, but a non-D-function heavy condition (e.g. a whole-column SUM) over
1,048,575 records has no cap beyond the calling formula's area budget (security review of T-1390).
Reuse an existing bound (`too_many_visits` / `MAX_LIFTED_CELL_VISITS` in limits.rs, or `area_budget`)
rather than adding a new one.

## Acceptance criteria
- [ ] A unit test shows a crafted call (many records times a heavy formula condition) finishes quickly
      with `#ERROR!`, and ordinary computed criteria (probes/db-computed-criteria-shapes) still pass.
- [ ] The DEVIATION is recorded in docs/excel-semantics/statistics.md "Database criteria".
- [ ] `make check` green; reviewers approve.

## Out of scope
- Excel semantics of computed criteria (T-1390).
