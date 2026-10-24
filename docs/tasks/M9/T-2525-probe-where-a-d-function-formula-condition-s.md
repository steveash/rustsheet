---
id: T-2525
title: "Probe where a D-function formula condition's references point when the criteria sit on another row or sheet"
milestone: M9
lane: research
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
docs/excel-semantics/statistics.md "Database criteria" states, from oracle evidence, how Excel moves a
formula condition's relative references from record to record when the criteria block is not on the
first record's row and sheet, and whether a lowercase `"true"` computed criterion holds.

## Context
T-1390 fidelity review: probes/db-computed-criteria-shapes only puts a formula condition (`=B2>2` under a
blank header) on the same row and sheet as the first record, so two readings fit: references move by the
record's offset from the criteria cell (ours, `db_computed_condition` in functions/database.rs), or are
rebased onto the first record. The rule is marked UNVERIFIED for other placements. Write an excel-probe case
with the criteria block below the database, above it, and on another sheet (each with `=B2>2`, `=B5>2` and an
absolute `=$B$2>2`), plus a lowercase `"true"` under a blank header.

## Acceptance criteria
- [ ] The probe is committed with excel-graph values and the rule is recorded (VERIFIED) in statistics.md.
- [ ] If ours differs, a follow-up task is filed (or the fix is made here if it is size S).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Capping re-evaluation work (T-2518).

## Outcome
Probe `probes/db-criteria-formula-placement` (excel-graph): a formula condition's relative references are
rebased onto the first record for criteria below, above or on another sheet; absolute references stay put;
`"true"` in any case holds. Our `db_computed_condition` already agrees (27/27 cells pass), so no code change
and no follow-up. Recorded VERIFIED in statistics.md. Eval baseline unchanged.
