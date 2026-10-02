---
id: T-0205
title: "Route D-function criteria through the shared criteria parser"
milestone: M2
lane: functions
status: done
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
- [x] `database.rs` has no operator parsing of its own; eval unchanged or improved.
- [x] `make check` green.

## Out of scope
Changing COUNTIF semantics.

## Outcome
`database.rs::criteria_cell_matches` now delegates to `functions::criteria::build_criteria_with` with
`CriteriaFlavor::Database` (prefix text equality; criteria text trimmed via `normalize_database_criterion`).
Intentional shifts to the shared grammar: numeric/boolean/error criteria and `<>text` behave like COUNTIF.
No oracle available in this run; the prefix quirk is recorded UNVERIFIED in `docs/excel-semantics/criteria.md`
(probe `criteria-grammar` C11 still pending). Added D-function unit test and hand-expected smoke case.
