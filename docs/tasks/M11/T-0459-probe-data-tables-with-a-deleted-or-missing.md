---
id: T-0459
title: "Probe: data tables with a deleted or missing input or a spilling header, and iteration over cycles that change type"
milestone: M11
lane: research
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Oracle evidence for four choices recalc.md records as unprobed, and the engine matching Excel's
answers.

## Context
Raised by the Excel-fidelity review of T-0441, T-0447 and T-0455:
- data_tables.rs gives `#REF!` for every result of a table whose input was deleted
  (`del1`/`del2`), or that has no input.
- `scalar_at` reads a table formula or header cell that spills or gives a range as its top-left
  value.
- iteration.rs `change()` never counts a member that changes type, text, boolean or error as
  converged.
probes/data-tables and the circular-iteration probes cover none of these.

## Acceptance criteria
- [ ] Probe cases, recalculated by the oracle, for:
  - a data table whose input cell is deleted after the table is made;
  - a header cell holding `=SEQUENCE(2)`, and a table formula returning a range;
  - cycles that alternate between text, boolean or error and a number, under a large
    iterateDelta.
- [ ] recalc.md records each as VERIFIED with the evidence, or rustsheet changes to match Excel.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Data tables inside iterating workbooks (T-0454).
