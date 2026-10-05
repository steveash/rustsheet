---
id: T-0956
title: "OFFSET: negative height/width should be valid as in Excel (currently #VALUE!)"
milestone: M9
lane: functions-lookup
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`OFFSET` accepts negative height/width as Excel does, instead of returning `#VALUE!`.

## Context
`fn_offset` (functions/lookup_and_reference/mod.rs) follows the Excel documentation, which calls negative sizes invalid, not Excel's behavior (its comment says so). No oracle evidence is recorded yet: probe first with the excel-probe skill (negative height/width, with and without 0/-1 edge cases) and record the rule in docs/excel-semantics/references.md. Found in the T-0955 reviews.

## Acceptance criteria
- [ ] Oracle-backed probe case and references.md entry; `fn_offset` matches it with unit tests.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
