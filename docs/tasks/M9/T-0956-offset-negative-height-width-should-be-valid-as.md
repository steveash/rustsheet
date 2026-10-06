---
id: T-0956
title: "OFFSET: negative height/width should be valid as in Excel (currently #VALUE!)"
milestone: M9
lane: functions-lookup
status: done
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

## Outcome
Probe `offset-negative-height-width` (80 cells, Excel oracle). Negative height/width extend up/left from the anchor; fractions and 0 follow the rule in references.md (0 is #REF!, (0,1) reaches one cell back, (-1,0) one cell on). `offset_size`/`fn_offset` rewritten; unit test added. probes +217 passing cells, 0 regressions.
The baseline also gains `probes/value-clock-text.case.json` (T-0162's case, already passing on main), recorded by `make eval-baseline`.
