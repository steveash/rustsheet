---
id: T-2197
title: "Probe how Excel resizes, converts and deletes a table and what its structured references become"
milestone: M8
lane: research
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
docs/excel-semantics/references.md "Structured references when the table changes" is VERIFIED, or its unknowns are listed with the probe that would settle them.

## Context
SPEC-0016 keeps structured references in the parsed formula and re-derives their area when the table list changes, but Excel's own behavior is unprobed: row and column inserts and deletes inside and at the edge of a table, a table resize, "Convert to Range", and deleting a table. The engine's tables do not move on structural edits (`insert_rows` never touches `workbook.tables`). Use the `excel-probe` skill; the oracle may only be able to evaluate files, not run table edits, so say which questions it cannot answer.

## Acceptance criteria
- [ ] Probe cases or a documented reason why each question cannot be probed with the oracle.
- [ ] references.md updated with the findings; an implementation task filed for what is probed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Implementation.
