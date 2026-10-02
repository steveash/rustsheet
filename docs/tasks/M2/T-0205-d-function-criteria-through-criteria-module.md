---
id: T-0205
title: "Route D-function criteria through the shared criteria parser"
milestone: M2
lane: functions
status: todo
priority: P2
size: M
depends_on: [T-0201]
---

## Goal
`database.rs::criteria_cell_matches` still has its own operator parser and number/date handling.
Fold it into `functions/criteria.rs` (a `Criteria` flavor flag for the D-function quirk that plain
text equality matches by prefix) so there is a single criteria grammar.

## Context
Left out of T-0201 because D-function equality is prefix-based ("ab" matches "abc"), which differs
from COUNTIF. Needs oracle evidence from `probes/criteria-grammar` (C11) before changing behavior.

## Acceptance criteria
- [ ] `database.rs` has no operator parsing of its own; eval unchanged or improved.
- [ ] `make check` green.

## Out of scope
Changing COUNTIF semantics.
