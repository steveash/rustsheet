---
id: T-2851
title: "Probe plain-range shrink on column deletes, whole-row ranges and multi-row deletes across a range edge"
milestone: M8
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel evidence for the plain-range shrink shapes that T-2338 implements by hand or leaves alone.

## Context
T-2338 probed one-row deletes at a range's edge with no table (probes/table-edits/plain-range-delete-row-first,
-last: `=SUM(S!C4:C7)` becomes `S!C4:C6`). `Displacement::range_corners` (stringify.rs) also shrinks
for a multi-row delete across an edge (rows 3-5 over C4:C7 give C3:C4), which is a hand
generalisation. Whole-row ranges (`=SUM(S!4:7)`) and every column delete still turn a deleted corner
into `#REF!`, although Excel very likely shrinks them too. Probe, each on sheet F with FORMULATEXT:
- delete rows 3-5 and rows 6-8 under `=SUM(S!C4:C7)`;
- delete row 4 under `=SUM(S!4:7)`;
- delete column C and column F under `=SUM(S!C4:F4)`.
references.md "Structured references under row deletes, column edits, convert, table and sheet
delete" records the finding.

## Acceptance criteria
- [ ] probes/table-edits cases for each shape, oracle-filled (excel-graph).
- [ ] references.md updated; a follow-up engine task filed for each shape the engine gets wrong.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the engine.
